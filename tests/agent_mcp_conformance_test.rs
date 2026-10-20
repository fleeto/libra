//! Registry-level MCP tool schema conformance (MC-01, GC-MC-10).
//!
//! Drives `libra agent mcp --stdio` for the wire checks and the same
//! `call_tool` path the server uses for success and error examples.

use std::{
    io::Write,
    path::Path,
    process::{Command, Output, Stdio},
};

use serde_json::Value;
use serial_test::serial;

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
        .expect("init");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    repo
}

fn mcp(cwd: &Path, body: &str) -> Output {
    let mut child = isolated_command(&["agent", "mcp", "--stdio"], cwd)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn");
    child
        .stdin
        .take()
        .expect("stdin")
        .write_all(body.as_bytes())
        .expect("write");
    let output = child.wait_with_output().expect("wait");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

fn frames(output: &Output) -> Vec<Value> {
    String::from_utf8(output.stdout.clone())
        .expect("utf-8")
        .lines()
        .filter(|line| !line.is_empty())
        .map(|line| serde_json::from_str(line).expect("json"))
        .collect()
}

fn tools_list(cwd: &Path) -> Vec<Value> {
    let body = concat!(
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"test","version":"0"}}}"#,
        "\n",
        r#"{"jsonrpc":"2.0","id":2,"method":"tools/list"}"#,
        "\n",
    );
    let listed = &frames(&mcp(cwd, body))[1]["result"]["tools"];
    listed.as_array().expect("tools").clone()
}

fn error_schema() -> Value {
    libra::internal::ai::mcp_server::protocol::tool_error_schema()
}

fn enable_test_tools() {
    // SAFETY: this test binary only sets LIBRA_TEST to "1" and never restores
    // a different value. Tests that read the registry run after this call.
    unsafe { std::env::set_var("LIBRA_TEST", "1") };
}

#[test]
fn every_tool_schema_includes_error_shape() {
    let repo = init_repo();
    let tools = tools_list(repo.path());
    if cfg!(debug_assertions) {
        assert!(
            tools.iter().any(|tool| tool["name"] == "libra_test_echo"),
            "LIBRA_TEST=1 debug server must register libra_test_echo: {tools:?}"
        );
    }
    let error_schema = error_schema();
    for tool in &tools {
        let branches = tool["outputSchema"]["anyOf"]
            .as_array()
            .unwrap_or_else(|| panic!("{} outputSchema.anyOf missing: {tool}", tool["name"]));
        assert!(
            branches.iter().any(|branch| branch == &error_schema),
            "{} outputSchema is missing TOOL_ERROR_SCHEMA",
            tool["name"]
        );
    }
}

#[test]
#[serial(env)]
fn every_tool_success_example_is_success() {
    enable_test_tools();
    let repo = init_repo();
    for tool in libra::internal::ai::mcp_server::tools::registered_tools() {
        let fixture = tool.fixture(repo.path());
        let body = format!(
            "{}\n{}\n",
            r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"test","version":"0"}}}"#,
            serde_json::json!({
                "jsonrpc": "2.0",
                "id": 2,
                "method": "tools/call",
                "params": {"name": tool.name, "arguments": fixture.success_arguments}
            })
        );
        let result = &frames(&mcp(repo.path(), &body))[1]["result"];
        assert_ne!(
            result["isError"], true,
            "{} success example failed: {result}",
            tool.name
        );
    }
}

#[test]
#[serial(env)]
fn every_tool_success_matches_success_branch() {
    enable_test_tools();
    let repo = init_repo();
    for tool in libra::internal::ai::mcp_server::tools::registered_tools() {
        let fixture = tool.fixture(repo.path());
        let outcome = libra::internal::ai::mcp_server::tools::call_tool(
            tool.name,
            &fixture.success_arguments,
        )
        .unwrap_or_else(|error| panic!("{}: {error}", tool.name));
        assert_ne!(outcome["isError"], true);
        let success_schema = &tool.output_schema["anyOf"][0];
        assert_valid(success_schema, &outcome["structuredContent"], tool.name);
    }
}

#[test]
#[serial(env)]
fn every_tool_error_example_is_error() {
    enable_test_tools();
    let repo = init_repo();
    for tool in libra::internal::ai::mcp_server::tools::registered_tools() {
        let fixture = tool.fixture(repo.path());
        let body = format!(
            "{}\n{}\n",
            r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"test","version":"0"}}}"#,
            serde_json::json!({
                "jsonrpc": "2.0",
                "id": 2,
                "method": "tools/call",
                "params": {"name": tool.name, "arguments": fixture.error_arguments}
            })
        );
        let result = &frames(&mcp(repo.path(), &body))[1]["result"];
        assert_eq!(
            result["isError"], true,
            "{} error example was not a tool error: {result}",
            tool.name
        );
    }
}

#[test]
#[serial(env)]
fn every_tool_error_matches_error_schema() {
    enable_test_tools();
    let repo = init_repo();
    let error_schema = error_schema();
    for tool in libra::internal::ai::mcp_server::tools::registered_tools() {
        let fixture = tool.fixture(repo.path());
        let outcome =
            libra::internal::ai::mcp_server::tools::call_tool(tool.name, &fixture.error_arguments)
                .unwrap_or_else(|error| panic!("{}: {error}", tool.name));
        assert_eq!(outcome["isError"], true);
        assert_valid(&error_schema, &outcome["structuredContent"], tool.name);
    }
}

#[test]
fn tools_list_matches_2025_11_25_schema() {
    let repo = init_repo();
    let result = tools_list_result(repo.path());
    let schema: Value =
        serde_json::from_str(include_str!("fixtures/mcp/schema-2025-11-25.json")).expect("schema");
    assert_valid(&list_tools_entry(&schema, "$defs"), &result, "2025-11-25");
}

#[test]
fn tools_list_matches_2025_06_18_schema() {
    let repo = init_repo();
    let result = tools_list_result(repo.path());
    let schema: Value =
        serde_json::from_str(include_str!("fixtures/mcp/schema-2025-06-18.json")).expect("schema");
    assert_valid(
        &list_tools_entry(&schema, "definitions"),
        &result,
        "2025-06-18",
    );
}

fn tools_list_result(cwd: &Path) -> Value {
    let body = concat!(
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"test","version":"0"}}}"#,
        "\n",
        r#"{"jsonrpc":"2.0","id":2,"method":"tools/list"}"#,
        "\n",
    );
    frames(&mcp(cwd, body))[1]["result"].clone()
}

fn list_tools_entry(document: &Value, defs_key: &str) -> Value {
    let mut entry = serde_json::Map::new();
    if let Some(dialect) = document.get("$schema") {
        entry.insert("$schema".to_string(), dialect.clone());
    }
    entry.insert(defs_key.to_string(), document[defs_key].clone());
    entry.insert(
        "$ref".to_string(),
        Value::String(format!("#/{defs_key}/ListToolsResult")),
    );
    Value::Object(entry)
}

fn assert_valid(schema: &Value, instance: &Value, label: &str) {
    let validator =
        jsonschema::validator_for(schema).unwrap_or_else(|error| panic!("{label} schema: {error}"));
    if let Err(error) = validator.validate(instance) {
        panic!("{label} instance rejected: {error}\ninstance={instance}");
    }
}
