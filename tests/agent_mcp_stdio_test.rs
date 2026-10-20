//! `libra agent mcp --stdio` legacy handshake, errors, and framing (MC-01).
//!
//! Each test drives the built `libra` binary. stdout must be JSON-RPC only.

use std::{
    io::Write,
    path::Path,
    process::{Command, Output, Stdio},
    time::{Duration, Instant},
};

use serde_json::Value;

fn isolated_command(args: &[&str], cwd: &Path) -> Command {
    let home = cwd.join(".libra-test-home");
    let config_home = home.join(".config");
    let global_db = home.join(".libra").join("config.db");
    let system_db = home.join(".libra").join("system-config.db");
    std::fs::create_dir_all(&config_home).expect("isolated config dir");
    let mut command = Command::new(env!("CARGO_BIN_EXE_libra"));
    command
        .args(args)
        .current_dir(cwd)
        .env_clear()
        .env("PATH", "/usr/bin:/bin:/usr/sbin:/sbin")
        .env("HOME", &home)
        .env("USERPROFILE", &home)
        .env("XDG_CONFIG_HOME", &config_home)
        .env("LIBRA_CONFIG_GLOBAL_DB", &global_db)
        .env("LIBRA_CONFIG_SYSTEM_DB", &system_db)
        .env("LANG", "C")
        .env("LC_ALL", "C");
    command
}

fn run_mcp(cwd: &Path, stdin_body: &[u8], libra_test: bool) -> Output {
    let mut command = isolated_command(&["agent", "mcp", "--stdio"], cwd);
    if libra_test {
        command.env("LIBRA_TEST", "1");
    }
    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn libra agent mcp");
    child
        .stdin
        .take()
        .expect("stdin")
        .write_all(stdin_body)
        .expect("write stdin");
    child.wait_with_output().expect("wait for mcp")
}

fn init_repo() -> tempfile::TempDir {
    let repo = tempfile::tempdir().expect("tempdir");
    let output = isolated_command(&["init"], repo.path())
        .output()
        .expect("run init");
    assert!(
        output.status.success(),
        "init failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    repo
}

fn frames(output: &Output) -> Vec<Value> {
    let text = String::from_utf8(output.stdout.clone()).expect("stdout utf-8");
    text.lines()
        .filter(|line| !line.is_empty())
        .map(|line| {
            serde_json::from_str(line)
                .unwrap_or_else(|error| panic!("stdout line is not JSON ({error}): {line}"))
        })
        .collect()
}

fn initialize(version: &str, id: i64) -> String {
    format!(
        r#"{{"jsonrpc":"2.0","id":{id},"method":"initialize","params":{{"protocolVersion":"{version}","capabilities":{{}},"clientInfo":{{"name":"test","version":"0"}}}}}}"#
    )
}

fn assert_ok(output: &Output) {
    assert!(
        output.status.success(),
        "mcp exited {:?}\nstderr {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn initialize_echoes_2025_11_25() {
    let repo = init_repo();
    let output = run_mcp(
        repo.path(),
        format!("{}\n", initialize("2025-11-25", 1)).as_bytes(),
        false,
    );
    assert_ok(&output);
    let frames = frames(&output);
    assert_eq!(frames.len(), 1);
    assert_eq!(frames[0]["result"]["protocolVersion"], "2025-11-25");
}

#[test]
fn initialize_echoes_2025_06_18() {
    let repo = init_repo();
    let output = run_mcp(
        repo.path(),
        format!("{}\n", initialize("2025-06-18", 1)).as_bytes(),
        false,
    );
    assert_ok(&output);
    assert_eq!(
        frames(&output)[0]["result"]["protocolVersion"],
        "2025-06-18"
    );
}

#[test]
fn initialize_unknown_version_falls_back() {
    let repo = init_repo();
    let output = run_mcp(
        repo.path(),
        format!("{}\n", initialize("1900-01-01", 1)).as_bytes(),
        false,
    );
    assert_ok(&output);
    assert_eq!(
        frames(&output)[0]["result"]["protocolVersion"],
        "2025-11-25"
    );
}

#[test]
fn initialize_server_name() {
    let repo = init_repo();
    let output = run_mcp(
        repo.path(),
        format!("{}\n", initialize("2025-11-25", 1)).as_bytes(),
        false,
    );
    assert_ok(&output);
    assert_eq!(frames(&output)[0]["result"]["serverInfo"]["name"], "libra");
}

#[test]
fn initialize_server_version() {
    let repo = init_repo();
    let output = run_mcp(
        repo.path(),
        format!("{}\n", initialize("2025-11-25", 1)).as_bytes(),
        false,
    );
    assert_ok(&output);
    assert_eq!(
        frames(&output)[0]["result"]["serverInfo"]["version"],
        env!("CARGO_PKG_VERSION")
    );
}

#[test]
fn initialize_capabilities_tools_only() {
    let repo = init_repo();
    let output = run_mcp(
        repo.path(),
        format!("{}\n", initialize("2025-11-25", 1)).as_bytes(),
        false,
    );
    assert_ok(&output);
    assert_eq!(
        frames(&output)[0]["result"]["capabilities"],
        serde_json::json!({"tools": {"listChanged": false}})
    );
}

#[test]
fn request_before_initialize_is_error() {
    let repo = init_repo();
    let body = "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"tools/list\"}\n";
    let output = run_mcp(repo.path(), body.as_bytes(), false);
    assert_ok(&output);
    let frame = &frames(&output)[0];
    assert!(
        frame.get("error").is_some(),
        "expected a JSON-RPC error: {frame}"
    );
    assert!(
        frame.get("result").is_none(),
        "error must not also be a result: {frame}"
    );
}

#[test]
fn ping_after_initialize() {
    let repo = init_repo();
    let body = format!(
        "{}\n{}\n",
        initialize("2025-11-25", 1),
        r#"{"jsonrpc":"2.0","id":2,"method":"ping"}"#
    );
    let output = run_mcp(repo.path(), body.as_bytes(), false);
    assert_ok(&output);
    let frames = frames(&output);
    assert_eq!(frames[1]["id"], 2);
    assert_eq!(frames[1]["result"], serde_json::json!({}));
}

#[test]
fn initialized_notification_is_silent() {
    let repo = init_repo();
    let body = format!(
        "{}\n{}\n{}\n",
        initialize("2025-11-25", 1),
        r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#,
        r#"{"jsonrpc":"2.0","id":2,"method":"ping"}"#
    );
    let output = run_mcp(repo.path(), body.as_bytes(), false);
    assert_ok(&output);
    let frames = frames(&output);
    assert_eq!(
        frames.len(),
        2,
        "notification must not add a stdout line: {frames:?}"
    );
    assert_eq!(frames[1]["result"], serde_json::json!({}));
}

#[test]
fn batch_is_rejected() {
    let repo = init_repo();
    let body = format!(
        "{}\n{}\n",
        initialize("2025-11-25", 1),
        r#"[{"jsonrpc":"2.0","id":2,"method":"ping"}]"#
    );
    let output = run_mcp(repo.path(), body.as_bytes(), false);
    assert_ok(&output);
    assert_eq!(frames(&output)[1]["error"]["code"], -32600);
}

#[test]
fn unknown_method_is_32601() {
    let repo = init_repo();
    let body = format!(
        "{}\n{}\n",
        initialize("2025-11-25", 1),
        r#"{"jsonrpc":"2.0","id":2,"method":"no/such"}"#
    );
    let output = run_mcp(repo.path(), body.as_bytes(), false);
    assert_ok(&output);
    assert_eq!(frames(&output)[1]["error"]["code"], -32601);
}

#[test]
fn invalid_json_is_32700() {
    let repo = init_repo();
    let output = run_mcp(repo.path(), b"this-is-not-json\n", false);
    assert_ok(&output);
    assert_eq!(frames(&output)[0]["error"]["code"], -32700);
}

#[test]
fn test_tools_hidden_in_production() {
    let repo = init_repo();
    let body = format!(
        "{}\n{}\n",
        initialize("2025-11-25", 1),
        r#"{"jsonrpc":"2.0","id":2,"method":"tools/list"}"#
    );
    let output = run_mcp(repo.path(), body.as_bytes(), false);
    assert_ok(&output);
    let parsed = frames(&output);
    let tools = parsed[1]["result"]["tools"]
        .as_array()
        .expect("tools array");
    assert!(
        tools.iter().all(|tool| {
            tool["name"]
                .as_str()
                .is_some_and(|name| !name.starts_with("libra_test_"))
        }),
        "production tools/list leaked a test tool: {tools:?}"
    );
}

#[test]
fn unknown_tool_is_32602() {
    let repo = init_repo();
    let body = format!(
        "{}\n{}\n",
        initialize("2025-11-25", 1),
        r#"{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"missing_tool","arguments":{}}}"#
    );
    let output = run_mcp(repo.path(), body.as_bytes(), false);
    assert_ok(&output);
    assert_eq!(frames(&output)[1]["error"]["code"], -32602);
}

#[test]
fn schema_violation_is_tool_error() {
    let repo = init_repo();
    let body = format!(
        "{}\n{}\n",
        initialize("2025-11-25", 1),
        r#"{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"libra_test_echo","arguments":{"text":"ok","extra":true}}}"#
    );
    let output = run_mcp(repo.path(), body.as_bytes(), true);
    assert_ok(&output);
    let result = &frames(&output)[1]["result"];
    assert_eq!(result["isError"], true);
    assert_eq!(result["structuredContent"]["stable_code"], "LBR-AGENT-027");
}

#[test]
fn stdout_is_only_jsonrpc() {
    let repo = init_repo();
    let body = format!(
        "{}\nnot-json\n{}\n",
        initialize("2025-11-25", 1),
        r#"{"jsonrpc":"2.0","id":2,"method":"ping"}"#
    );
    let output = run_mcp(repo.path(), body.as_bytes(), false);
    assert_ok(&output);
    let text = String::from_utf8(output.stdout.clone()).expect("utf-8");
    assert!(!text.is_empty());
    for line in text.lines().filter(|line| !line.is_empty()) {
        let value: Value =
            serde_json::from_str(line).unwrap_or_else(|error| panic!("{error}: {line}"));
        assert_eq!(value["jsonrpc"], "2.0", "{line}");
        assert!(
            value.get("result").is_some() || value.get("error").is_some(),
            "{line}"
        );
    }
}

#[test]
fn oversized_line_is_32600() {
    let repo = init_repo();
    let pad = "x".repeat(256 * 1024 + 1);
    let line = format!(r#"{{"jsonrpc":"2.0","id":1,"method":"ping","pad":"{pad}"}}"#);
    let output = run_mcp(repo.path(), format!("{line}\n").as_bytes(), false);
    assert_ok(&output);
    assert_eq!(frames(&output)[0]["error"]["code"], -32600);
}

#[test]
fn server_survives_oversized_line() {
    let repo = init_repo();
    let pad = "y".repeat(256 * 1024 + 8);
    let oversized = format!(r#"{{"jsonrpc":"2.0","id":9,"method":"ping","pad":"{pad}"}}"#);
    let body = format!(
        "{}\n{oversized}\n{}\n",
        initialize("2025-11-25", 1),
        r#"{"jsonrpc":"2.0","id":2,"method":"ping"}"#
    );
    let output = run_mcp(repo.path(), body.as_bytes(), false);
    assert_ok(&output);
    let frames = frames(&output);
    assert_eq!(frames.len(), 3, "{frames:?}");
    assert_eq!(frames[2]["id"], 2);
    assert_eq!(frames[2]["result"], serde_json::json!({}));
}

#[test]
fn eof_exits_zero() {
    let repo = init_repo();
    let output = run_mcp(repo.path(), b"", false);
    assert_ok(&output);
    assert!(output.stdout.is_empty());
}

#[test]
fn outside_repository_fails() {
    let dir = tempfile::tempdir().expect("tempdir");
    let output = run_mcp(dir.path(), b"", false);
    assert_ne!(output.status.code(), Some(0));
}

#[test]
fn outside_repository_exits_without_eof() {
    let dir = tempfile::tempdir().expect("tempdir");
    let mut child = isolated_command(&["agent", "mcp", "--stdio"], dir.path())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn");
    let started = Instant::now();
    let status = loop {
        if let Some(status) = child.try_wait().expect("try_wait") {
            break status;
        }
        if started.elapsed() > Duration::from_secs(5) {
            let _ = child.kill();
            panic!("libra agent mcp stayed alive outside a repository with stdin open");
        }
        std::thread::sleep(Duration::from_millis(20));
    };
    assert!(!status.success(), "outside a repository must exit non-zero");
    assert!(started.elapsed() < Duration::from_secs(5));
}

#[test]
fn missing_transport_is_usage_error() {
    let repo = init_repo();
    let output = isolated_command(&["agent", "mcp"], repo.path())
        .output()
        .expect("run");
    assert_eq!(output.status.code(), Some(129));
}
