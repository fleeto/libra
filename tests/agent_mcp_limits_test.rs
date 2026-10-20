//! MCP result size and read-deadline gates (plan-20261019 MC-19).

use std::{
    io::Write,
    path::Path,
    process::{Command, Output, Stdio},
};

use serde_json::Value;

fn isolated_command(args: &[&str], cwd: &Path) -> Command {
    let home = cwd.join(".libra-test-home");
    let config_home = home.join(".config");
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
        .env(
            "LIBRA_CONFIG_GLOBAL_DB",
            home.join(".libra").join("config.db"),
        )
        .env(
            "LIBRA_CONFIG_SYSTEM_DB",
            home.join(".libra").join("system-config.db"),
        )
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

fn session(calls: &str) -> String {
    format!(
        "{{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{{}},\"clientInfo\":{{\"name\":\"test\",\"version\":\"0\"}}}}}}\n{{\"jsonrpc\":\"2.0\",\"method\":\"notifications/initialized\"}}\n{calls}\n"
    )
}

fn response_with_id(stdout: &str, id: i64) -> Value {
    stdout
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .find(|message| message.get("id") == Some(&Value::from(id)))
        .expect("jsonrpc response")
}

fn tool_call(id: i64, name: &str, arguments: &str) -> String {
    format!(
        r#"{{"jsonrpc":"2.0","id":{id},"method":"tools/call","params":{{"name":"{name}","arguments":{arguments}}}}}"#
    )
}

#[test]
fn envelope_caps_full_response() {
    let repo = init_repo();
    let output = run_mcp(
        repo.path(),
        &session(&tool_call(2, "libra_test_blob", r#"{"bytes":143360}"#)),
    );
    assert!(
        output.status.success(),
        "mcp failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("stdout utf-8");
    let message = response_with_id(&stdout, 2);
    assert_eq!(
        message["result"]["structuredContent"]["stable_code"],
        "LBR-AGENT-028"
    );
    assert_eq!(message["result"]["isError"], true);
    assert!(
        !stdout.contains(&"a".repeat(1000)),
        "oversized blob was returned"
    );
}

#[test]
fn envelope_allows_under_cap() {
    let repo = init_repo();
    let output = run_mcp(
        repo.path(),
        &session(&tool_call(2, "libra_test_blob", r#"{"bytes":102400}"#)),
    );
    assert!(
        output.status.success(),
        "mcp failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("stdout utf-8");
    let message = response_with_id(&stdout, 2);
    assert_eq!(message["result"]["structuredContent"]["len"], 102400);
}

#[test]
fn envelope_under_cap_fits_wire_limit() {
    let repo = init_repo();
    let output = run_mcp(
        repo.path(),
        &session(&tool_call(2, "libra_test_blob", r#"{"bytes":102400}"#)),
    );
    assert!(
        output.status.success(),
        "mcp failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("stdout utf-8");
    let line = stdout
        .lines()
        .find(|line| line.contains("\"id\":2"))
        .expect("tools/call line");
    assert!(
        line.len() <= 262_144,
        "response line is {} bytes",
        line.len()
    );
}

#[test]
fn read_deadline_is_tool_error() {
    let repo = init_repo();
    let output = run_mcp(
        repo.path(),
        &session(&tool_call(2, "libra_test_sleep", r#"{"ms":31000}"#)),
    );
    assert!(
        output.status.success(),
        "mcp failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("stdout utf-8");
    let message = response_with_id(&stdout, 2);
    assert_eq!(
        message["result"]["structuredContent"]["stable_code"],
        "LBR-AGENT-036"
    );
}

#[test]
fn read_deadline_is_retryable() {
    let repo = init_repo();
    let output = run_mcp(
        repo.path(),
        &session(&tool_call(2, "libra_test_sleep", r#"{"ms":31000}"#)),
    );
    assert!(
        output.status.success(),
        "mcp failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("stdout utf-8");
    let message = response_with_id(&stdout, 2);
    assert_eq!(message["result"]["structuredContent"]["retryable"], true);
}

#[test]
fn server_survives_deadline() {
    let repo = init_repo();
    let calls = format!(
        "{}\n{}",
        tool_call(2, "libra_test_sleep", r#"{"ms":31000}"#),
        r#"{"jsonrpc":"2.0","id":3,"method":"ping"}"#
    );
    let output = run_mcp(repo.path(), &session(&calls));
    assert!(
        output.status.success(),
        "mcp failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("stdout utf-8");
    let ping = response_with_id(&stdout, 3);
    assert_eq!(ping["result"], serde_json::json!({}));
}
