//! Integration tests for `libra auth` (lore.md §1.6): the full token
//! lifecycle — write (stdin only, never argv), read/status, expiry
//! detection, revoke — plus the config-namespace lockdown and the
//! never-echo-the-secret invariant.
//!
//! **Layer:** L1 — deterministic (global store isolated per test repo).

use super::*;

const TOKEN: &str = "s3cr3t-token-value-42";

fn login(p: &Path, host: &str, extra: &[&str]) -> std::process::Output {
    let mut argv = vec!["auth", "login", "--host", host, "--with-token"];
    argv.extend_from_slice(extra);
    run_libra_command_with_stdin(&argv, p, &format!("{TOKEN}\n"))
}

#[test]
fn auth_lifecycle_login_status_logout() {
    let repo = create_committed_repo_via_cli();
    let p = repo.path();
    // login via stdin; the secret never appears in ANY output.
    let out = login(p, "git.example.com", &[]);
    assert_cli_success(&out, "login");
    assert!(
        !String::from_utf8_lossy(&out.stdout).contains(TOKEN)
            && !String::from_utf8_lossy(&out.stderr).contains(TOKEN),
        "token never echoed"
    );
    // status: json has the host and state but NO token field anywhere.
    let status = run_libra_command(&["--json", "auth", "status"], p);
    assert_cli_success(&status, "status");
    let text = String::from_utf8_lossy(&status.stdout).to_string();
    assert!(!text.contains(TOKEN), "token never in JSON: {text}");
    let json = parse_json_stdout(&status);
    let tokens = json["data"]["tokens"].as_array().unwrap();
    assert_eq!(tokens.len(), 1, "{json}");
    assert_eq!(tokens[0]["host"].as_str(), Some("git.example.com"));
    assert_eq!(tokens[0]["state"].as_str(), Some("valid"));
    assert!(tokens[0].get("token").is_none(), "no token field: {json}");
    // Scriptable single-host contract.
    let hit = run_libra_command(&["auth", "status", "--host", "git.example.com"], p);
    assert_eq!(hit.status.code(), Some(0));
    let miss = run_libra_command(&["auth", "status", "--host", "other.example.com"], p);
    assert_eq!(miss.status.code(), Some(1));
    // Host normalization: the same scope spelled differently matches.
    let normalized = run_libra_command(&["auth", "status", "--host", "https://GIT.Example.COM"], p);
    assert_eq!(
        normalized.status.code(),
        Some(0),
        "normalized spellings match"
    );
    // Re-login overwrites (single token per scope), then logout removes.
    assert_cli_success(
        &login(p, "git.example.com", &["--username", "bot"]),
        "relogin",
    );
    let logout = run_libra_command(
        &["--json", "auth", "logout", "--host", "git.example.com"],
        p,
    );
    assert_cli_success(&logout, "logout");
    assert_eq!(
        parse_json_stdout(&logout)["data"]["removed"].as_i64(),
        Some(1)
    );
    // Idempotent; clear works with nothing stored.
    let again = run_libra_command(&["auth", "logout", "--host", "git.example.com"], p);
    assert_cli_success(&again, "logout idempotent");
    assert_cli_success(&run_libra_command(&["auth", "clear"], p), "clear empty");
}

#[test]
fn auth_expiry_matrix() {
    let repo = create_committed_repo_via_cli();
    let p = repo.path();
    // Relative expiry that has already lapsed by the time status runs.
    assert_cli_success(&login(p, "h1.example", &["--expires-in", "1s"]), "login 1s");
    std::thread::sleep(std::time::Duration::from_secs(2));
    let status = run_libra_command(&["--json", "auth", "status", "--host", "h1.example"], p);
    assert_eq!(status.status.code(), Some(1), "expired token exits 1");
    let json = parse_json_stdout(&status);
    assert_eq!(json["data"]["tokens"][0]["state"].as_str(), Some("expired"));
    // Refusals: past absolute expiry, bad duration forms, both flags.
    let past = login(p, "h2.example", &["--expires-at", "2020-01-01T00:00:00Z"]);
    assert_eq!(past.status.code(), Some(129), "past expiry refused");
    let bare = login(p, "h2.example", &["--expires-at", "20270101"]);
    assert_eq!(bare.status.code(), Some(129), "bare date refused with hint");
    assert!(
        String::from_utf8_lossy(&bare.stderr).contains("RFC3339"),
        "{}",
        String::from_utf8_lossy(&bare.stderr)
    );
    for bad in ["1h30m", "10x", "d", "99999999999999999999d"] {
        let out = login(p, "h2.example", &["--expires-in", bad]);
        assert_eq!(out.status.code(), Some(129), "{bad} refused");
    }
    let both = login(
        p,
        "h2.example",
        &["--expires-at", "2030-01-01T00:00:00Z", "--expires-in", "1d"],
    );
    assert_eq!(both.status.code(), Some(129), "flags mutually exclusive");
}

#[test]
fn auth_input_rules_and_namespace_lockdown() {
    let repo = create_committed_repo_via_cli();
    let p = repo.path();
    // Non-TTY without --with-token is a hard usage error with the pipe hint.
    let no_tty = run_libra_command(&["auth", "login", "--host", "h.example"], p);
    assert_eq!(no_tty.status.code(), Some(129));
    assert!(
        String::from_utf8_lossy(&no_tty.stderr).contains("--with-token"),
        "{}",
        String::from_utf8_lossy(&no_tty.stderr)
    );
    // There is NO --token flag by design.
    let flag = run_libra_command(&["auth", "login", "--host", "h.example", "--token", "x"], p);
    assert_eq!(
        flag.status.code(),
        Some(129),
        "--token flag refused by clap"
    );
    // Bad hosts refused.
    for bad in [
        "",
        "ssh://h.example",
        "https://u:p@h.example",
        "h.example/path",
    ] {
        let out = login(p, bad, &[]);
        assert_eq!(out.status.code(), Some(129), "{bad:?} refused");
    }
    // Store one token, then verify the config surface cannot see or touch it.
    assert_cli_success(&login(p, "h.example", &[]), "login");
    let list = run_libra_command(&["config", "--global", "--list"], p);
    let listing = String::from_utf8_lossy(&list.stdout).to_string();
    assert!(
        !listing.contains(TOKEN),
        "config list never shows the secret: {listing}"
    );
    // Empty token refused.
    let empty = run_libra_command_with_stdin(
        &["auth", "login", "--host", "h2.example", "--with-token"],
        p,
        "\n",
    );
    assert_eq!(empty.status.code(), Some(129), "empty token refused");
}

fn login_with_username(p: &Path, host: &str, username: &str, token: &str) -> std::process::Output {
    run_libra_command_with_stdin(
        &[
            "auth",
            "login",
            "--host",
            host,
            "--username",
            username,
            "--with-token",
        ],
        p,
        &format!("{token}\n"),
    )
}

#[test]
fn auth_output_redacts_embedded_credentials() {
    let repo = create_committed_repo_via_cli();
    let p = repo.path();
    // `auth login` rejects `:` in a username, so a stored name cannot be a
    // URL. The cleartext sink is the bare token username itself.
    let username = "ghp_embeddedtoken";
    assert_cli_success(
        &login_with_username(p, "git.example.com", username, TOKEN),
        "login",
    );
    let human = run_libra_command(&["auth", "status", "--host", "git.example.com"], p);
    assert_cli_success(&human, "status");
    let text = String::from_utf8_lossy(&human.stdout);
    assert!(
        !text.contains(username),
        "human status keeps the token username: {text}"
    );
    assert!(
        text.contains("[REDACTED]"),
        "token username is replaced: {text}"
    );
    assert!(
        text.starts_with("git.example.com:"),
        "host without userinfo stays byte-identical: {text}"
    );
    let json = run_libra_command(&["--json", "auth", "status", "--host", "git.example.com"], p);
    assert_cli_success(&json, "status json");
    let body = String::from_utf8_lossy(&json.stdout);
    assert!(
        body.contains(username),
        "json username stays the stored value: {body}"
    );
}

#[test]
fn auth_status_human_redacts_token_username() {
    let repo = create_committed_repo_via_cli();
    let p = repo.path();
    let names = [
        "ghp_exampletoken",
        "sk-exampletoken",
        "github_pat_exampletoken",
    ];
    for (index, username) in names.iter().enumerate() {
        assert_cli_success(
            &login_with_username(p, &format!("h{index}.example"), username, TOKEN),
            "login",
        );
    }
    let human = run_libra_command(&["auth", "status"], p);
    assert_cli_success(&human, "status");
    let text = String::from_utf8_lossy(&human.stdout);
    for username in names {
        assert!(
            !text.contains(username),
            "human status still shows {username}: {text}"
        );
    }
    assert!(
        text.contains("[REDACTED]"),
        "prefix usernames are replaced: {text}"
    );
    let json = run_libra_command(&["--json", "auth", "status"], p);
    let body = String::from_utf8_lossy(&json.stdout);
    for username in names {
        assert!(body.contains(username), "json keeps {username}: {body}");
    }
}

struct AccountFixture {
    base: String,
    _thread: std::thread::JoinHandle<()>,
}

fn spawn_account_fixture(exchange_host: Option<String>, whoami_host: String) -> AccountFixture {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind fixture");
    let port = listener.local_addr().expect("addr").port();
    let base = format!("http://127.0.0.1:{port}");
    let exchange_host = exchange_host.unwrap_or_else(|| base.clone());
    let handle = std::thread::spawn(move || {
        for stream in listener.incoming() {
            let mut stream = match stream {
                Ok(stream) => stream,
                Err(_) => continue,
            };
            let _ = stream.set_read_timeout(Some(std::time::Duration::from_secs(5)));
            let mut buf = [0u8; 4096];
            let n = std::io::Read::read(&mut stream, &mut buf).unwrap_or(0);
            let request = String::from_utf8_lossy(&buf[..n]);
            let path = request
                .lines()
                .next()
                .and_then(|line| line.split_whitespace().nth(1))
                .unwrap_or("/");
            let body = if path.starts_with("/api/cli/exchange") {
                format!(
                    r#"{{"session_token":"sess-fixture","username":"alice","user_id":"u1","github_id":"g1","host":"{exchange_host}","issued_at":"2026-10-31T00:00:00Z","expires_at":"2026-11-30T00:00:00Z"}}"#
                )
            } else if path.starts_with("/api/cli/whoami") {
                format!(
                    r#"{{"username":"alice","user_id":"u1","github_id":"g1","host":"{whoami_host}","expires_at":"2026-11-30T00:00:00Z"}}"#
                )
            } else {
                r#"{"ok":true}"#.to_string()
            };
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            let _ = std::io::Write::write_all(&mut stream, response.as_bytes());
        }
    });
    AccountFixture {
        base,
        _thread: handle,
    }
}

fn drive_login(repo: &Path, base: &str, json: bool) -> std::process::Output {
    let mut args = Vec::new();
    if json {
        args.push("--json");
    }
    args.extend(["login", "--host", base, "--no-browser"]);
    let mut child = spawn_libra_command_with_env(&args, repo, &[]);
    let mut stdout = child.child.stdout.take().expect("login stdout");
    let mut stderr = child.child.stderr.take().expect("login stderr");
    let (url_tx, url_rx) = std::sync::mpsc::channel();
    let (out_tx, out_rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let mut buf = Vec::new();
        let mut tmp = [0u8; 512];
        let mut sent = false;
        loop {
            match std::io::Read::read(&mut stdout, &mut tmp) {
                Ok(0) => break,
                Ok(n) => {
                    buf.extend_from_slice(&tmp[..n]);
                    if !sent && String::from_utf8_lossy(&buf).contains("Verification code:") {
                        let _ = url_tx.send(String::from_utf8_lossy(&buf).into_owned());
                        sent = true;
                    }
                }
                Err(_) => break,
            }
        }
        let _ = out_tx.send(buf);
    });
    let err_handle = std::thread::spawn(move || {
        let mut buf = Vec::new();
        let _ = std::io::Read::read_to_end(&mut stderr, &mut buf);
        buf
    });
    let prelude = url_rx
        .recv_timeout(std::time::Duration::from_secs(30))
        .expect("login did not print its URL");
    let callback_port = prelude
        .split("callback_port=")
        .nth(1)
        .and_then(|rest| rest.split(['&', ' ', '\n']).next())
        .expect("callback port")
        .to_string();
    let state = prelude
        .split("state=")
        .nth(1)
        .and_then(|rest| rest.split(['&', ' ', '\n']).next())
        .expect("state")
        .to_string();
    let mut callback = std::net::TcpStream::connect(format!("127.0.0.1:{callback_port}"))
        .expect("callback connect");
    let request = format!(
        "GET /?code=fixture-code&state={state} HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n"
    );
    std::io::Write::write_all(&mut callback, request.as_bytes()).expect("callback write");
    let status = child.child.wait().expect("login wait");
    let stdout = out_rx
        .recv_timeout(std::time::Duration::from_secs(20))
        .unwrap_or_default();
    let stderr = err_handle.join().unwrap_or_default();
    std::process::Output {
        status,
        stdout,
        stderr,
    }
}

#[test]
fn login_print_redacts_exchanged_host_credentials() {
    let secret_host = "https://user:pass@libra.tools";
    let fixture = spawn_account_fixture(Some(secret_host.to_string()), secret_host.to_string());
    let repo = create_committed_repo_via_cli();
    let out = drive_login(repo.path(), &fixture.base, false);
    assert_cli_success(&out, "login");
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(
        !text.contains("user:pass"),
        "human login echoes exchanged credentials: {text}"
    );
    assert!(
        text.contains("Logged in to https://libra.tools as alice"),
        "credential origin is redacted without an added slash: {text}"
    );
    let clean = spawn_account_fixture(None, fixture.base.clone());
    let clean_out = drive_login(repo.path(), &clean.base, false);
    assert_cli_success(&clean_out, "clean login");
    let clean_text = String::from_utf8_lossy(&clean_out.stdout);
    assert!(
        clean_text.contains(&format!("Logged in to {} as alice", clean.base)),
        "origin without userinfo stays byte-identical: {clean_text}"
    );
}

#[test]
fn login_whoami_print_uses_display_host() {
    let repo = create_committed_repo_via_cli();
    let whoami_host = "https://user:pass@libra.tools";
    // Session lookup uses the normalized CLI host, so the exchange host is
    // that same origin. The whoami response body carries the credential URL.
    let fixture = spawn_account_fixture(None, whoami_host.to_string());
    let login = drive_login(repo.path(), &fixture.base, false);
    assert_cli_success(&login, "login");
    let human = run_libra_command(&["whoami", "--host", &fixture.base], repo.path());
    assert_cli_success(&human, "whoami");
    let text = String::from_utf8_lossy(&human.stdout);
    assert!(
        !text.contains("user:pass"),
        "human whoami echoes body.host: {text}"
    );
    assert!(
        text.contains("on https://libra.tools"),
        "whoami human host is redacted: {text}"
    );
    let json = run_libra_command(&["--json", "whoami", "--host", &fixture.base], repo.path());
    assert_cli_success(&json, "whoami json");
    let body = String::from_utf8_lossy(&json.stdout);
    assert!(
        body.contains(whoami_host),
        "json whoami keeps the server host: {body}"
    );
    assert!(
        !String::from_utf8_lossy(&login.stdout).contains("user:pass"),
        "login human path does not print the whoami credential"
    );
}

#[test]
fn json_byte_identical() {
    let repo = create_committed_repo_via_cli();
    let p = repo.path();
    let username = "ghp_exampletoken";
    assert_cli_success(
        &login_with_username(p, "git.example.com", username, TOKEN),
        "auth login",
    );
    let auth_json = run_libra_command(&["--json", "auth", "status"], p);
    assert_cli_success(&auth_json, "auth json");
    let auth_body = parse_json_stdout(&auth_json);
    assert_eq!(
        auth_body["data"]["tokens"][0]["username"].as_str(),
        Some(username)
    );
    let secret_host = "https://user:pass@libra.tools";
    let fixture = spawn_account_fixture(Some(secret_host.to_string()), secret_host.to_string());
    let login = drive_login(p, &fixture.base, true);
    assert_cli_success(&login, "login json");
    let raw = String::from_utf8_lossy(&login.stdout);
    let json_start = raw.find("\n{").map(|index| index + 1).expect("login json");
    let login_body: serde_json::Value =
        serde_json::from_str(raw[json_start..].trim()).expect("login json parse");
    assert_eq!(login_body["data"]["host"].as_str(), Some(secret_host));
    assert_eq!(login_body["data"]["username"].as_str(), Some("alice"));
}
