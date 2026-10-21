//! Tool-result redaction for `libra agent mcp` (plan-20261019 MC-16).

use std::{
    io::Write,
    path::Path,
    process::{Command, Output, Stdio},
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
        .env("LC_ALL", "C")
        .env("LIBRA_TEST", "1");
    command
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

fn run_mcp(cwd: &Path, stdin_body: &str) -> Output {
    let mut child = isolated_command(&["agent", "mcp", "--stdio"], cwd)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn libra agent mcp");
    child
        .stdin
        .take()
        .expect("stdin")
        .write_all(stdin_body.as_bytes())
        .expect("write stdin");
    child.wait_with_output().expect("wait for mcp")
}

fn session(call: &str) -> String {
    format!(
        "{{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{{}},\"clientInfo\":{{\"name\":\"test\",\"version\":\"0\"}}}}}}\n{{\"jsonrpc\":\"2.0\",\"method\":\"notifications/initialized\"}}\n{call}\n"
    )
}

fn response_with_id(stdout: &str, id: i64) -> Value {
    stdout
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .find(|message| message.get("id") == Some(&Value::from(id)))
        .expect("jsonrpc response")
}

fn contains_esc(value: &Value) -> bool {
    match value {
        Value::String(text) => text.chars().any(|c| c == '\u{001b}'),
        Value::Array(items) => items.iter().any(contains_esc),
        Value::Object(map) => {
            map.keys().any(|key| key.contains('\u{001b}')) || map.values().any(contains_esc)
        }
        _ => false,
    }
}

#[test]
fn envelope_redacts_secret() {
    let secret = format!("{}_{}", "ghp", "abcdefghijklmnopqrstuvwxyz0123456789");
    let repo = init_repo();
    let call = format!(
        r#"{{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{{"name":"libra_test_echo","arguments":{{"text":"\u001b{secret}"}}}}}}"#
    );
    let output = run_mcp(repo.path(), &session(&call));
    assert!(
        output.status.success(),
        "mcp failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("stdout utf-8");
    assert!(
        !stdout.contains(&secret),
        "tool result still contains the secret: {stdout}"
    );
    assert!(
        stdout.contains("\"id\":2"),
        "tools/call was not answered: {stdout}"
    );
    let message = response_with_id(&stdout, 2);
    let result = &message["result"];
    let text = result["content"][0]["text"].as_str().expect("content text");
    assert!(
        !text.contains(&secret),
        "text copy still contains the secret"
    );
    let decoded: Value = serde_json::from_str(text).expect("text copy is json");
    assert_eq!(&decoded, &result["structuredContent"]);
    assert!(
        !contains_esc(&decoded),
        "decoded text copy still contains ESC: {decoded}"
    );
    let echoed = decoded["text"].as_str().expect("echo text");
    assert!(
        echoed.contains('\u{FFFD}'),
        "ESC was not replaced with U+FFFD: {echoed}"
    );
    assert!(
        !echoed.contains(&secret),
        "decoded text still contains the secret"
    );
}

#[test]
fn envelope_scrubs_control_chars() {
    let repo = init_repo();
    let call = r#"{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"libra_test_echo","arguments":{"text":"a\u001b[31mb"}}}"#;
    let output = run_mcp(repo.path(), &session(call));
    assert!(
        output.status.success(),
        "mcp failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        !output.stdout.contains(&0x1b),
        "ESC byte survived in the tool result: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains('\u{001b}'),
        "ESC character survived in the tool result: {stdout}"
    );
    let message = response_with_id(&stdout, 2);
    let result = &message["result"];
    let structured = &result["structuredContent"];
    let text = result["content"][0]["text"].as_str().expect("content text");
    let decoded: Value = serde_json::from_str(text).expect("text copy is json");
    assert_eq!(&decoded, structured);
    assert!(
        !contains_esc(structured),
        "structuredContent still contains ESC: {structured}"
    );
    assert!(
        !contains_esc(&decoded),
        "decoded text copy still contains ESC: {decoded}"
    );
    let echoed = structured["text"].as_str().expect("echo text");
    assert!(
        echoed.contains('\u{FFFD}'),
        "ESC was not replaced with U+FFFD: {echoed}"
    );
}
