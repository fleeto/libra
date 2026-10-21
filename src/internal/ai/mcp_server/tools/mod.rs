//! MCP tool registry.
//!
//! A tool cannot be registered without an input schema, a success output
//! schema (the error schema is attached here), a conformance fixture, and an
//! invoke function (GC-MC-10).

use std::path::Path;

use serde_json::Value;

use super::protocol::{
    output_schema_with_error, tool_error_result, tool_success_result, value_matches_schema,
};

pub struct ToolFixture {
    pub success_arguments: Value,
    pub error_arguments: Value,
    cleanup: Option<Box<dyn FnOnce() + Send>>,
}

impl ToolFixture {
    pub fn new(success_arguments: Value, error_arguments: Value) -> Self {
        Self {
            success_arguments,
            error_arguments,
            cleanup: None,
        }
    }

    pub fn with_cleanup(
        success_arguments: Value,
        error_arguments: Value,
        cleanup: impl FnOnce() + Send + 'static,
    ) -> Self {
        Self {
            success_arguments,
            error_arguments,
            cleanup: Some(Box::new(cleanup)),
        }
    }
}

impl Drop for ToolFixture {
    fn drop(&mut self) {
        if let Some(cleanup) = self.cleanup.take() {
            cleanup();
        }
    }
}

pub struct Tool {
    pub name: &'static str,
    pub description: &'static str,
    pub input_schema: Value,
    pub output_schema: Value,
    pub read_only: bool,
    fixture: fn(&Path) -> ToolFixture,
    invoke: fn(&Value) -> Result<Value, String>,
}

impl Tool {
    /// Register one tool. `success_schema` is the non-error branch; the shared
    /// tool-error schema is always added, so a caller cannot omit it.
    pub fn declare(
        name: &'static str,
        description: &'static str,
        input_schema: Value,
        success_schema: Value,
        fixture: fn(&Path) -> ToolFixture,
        invoke: fn(&Value) -> Result<Value, String>,
    ) -> Self {
        Self {
            name,
            description,
            input_schema,
            output_schema: output_schema_with_error(success_schema),
            read_only: true,
            fixture,
            invoke,
        }
    }

    pub fn fixture(&self, repo: &Path) -> ToolFixture {
        (self.fixture)(repo)
    }

    pub fn invoke(&self, arguments: &Value) -> Result<Value, String> {
        (self.invoke)(arguments)
    }
}

pub fn registered_tools() -> Vec<Tool> {
    // The only push is `cfg(debug_assertions)`. A release build leaves this
    // empty, and `sort_by` then cannot infer the element type (E0282).
    let mut tools: Vec<Tool> = Vec::new();
    #[cfg(debug_assertions)]
    if std::env::var("LIBRA_TEST").ok().as_deref() == Some("1") {
        tools.push(echo_tool());
        tools.push(blob_tool());
        tools.push(sleep_tool());
        tools.push(block_tool());
        tools.push(hash_kind_tool());
    }
    tools.sort_by(|left, right| left.name.cmp(right.name));
    tools
}

pub fn tools_list_wire() -> Vec<Value> {
    registered_tools()
        .iter()
        .map(|tool| {
            serde_json::json!({
                "name": tool.name,
                "description": tool.description,
                "inputSchema": tool.input_schema,
                "outputSchema": tool.output_schema
            })
        })
        .collect()
}

/// Run one tool the way `tools/call` does.
///
/// Unknown names are protocol errors (`Err`). Schema failures and invoke
/// failures are tool results (`Ok` with `isError: true`).
pub fn call_tool(name: &str, arguments: &Value) -> Result<Value, String> {
    let Some(tool) = registered_tools()
        .into_iter()
        .find(|tool| tool.name == name)
    else {
        return Err(format!(
            "unknown MCP tool {name}; call tools/list for the tools this server exposes"
        ));
    };
    if !value_matches_schema(arguments, &tool.input_schema) {
        return Ok(tool_error_result(
            "LBR-AGENT-027",
            "arguments do not match this tool's input schema",
            false,
        ));
    }
    if tool.read_only {
        return super::exec::execute_read_tool(name, arguments);
    }
    let invoke = tool.invoke;
    let arguments = arguments.clone();
    match super::output::with_read_deadline(move || invoke(&arguments)) {
        Ok(Ok(structured)) => Ok(tool_success_result(&structured)),
        Ok(Err(message)) => Ok(tool_error_result("LBR-AGENT-027", &message, false)),
        Err(super::output::ReadStop::Deadline) => Ok(tool_error_result(
            "LBR-AGENT-036",
            "read tool exceeded the 30 s deadline",
            true,
        )),
        Err(super::output::ReadStop::Stopped) => Ok(tool_error_result(
            "LBR-AGENT-027",
            "tool stopped before producing a result",
            false,
        )),
    }
}

/// Invoke a tool inside the read worker. This does not spawn another worker.
pub fn invoke_in_worker(name: &str, arguments: &Value) -> Result<Value, String> {
    let Some(tool) = registered_tools()
        .into_iter()
        .find(|tool| tool.name == name)
    else {
        return Err(format!(
            "unknown MCP tool {name}; call tools/list for the tools this server exposes"
        ));
    };
    if !value_matches_schema(arguments, &tool.input_schema) {
        return Err("arguments do not match this tool's input schema".to_string());
    }
    (tool.invoke)(arguments)
}

#[cfg(debug_assertions)]
fn echo_tool() -> Tool {
    Tool::declare(
        "libra_test_echo",
        "Test-only tool that returns its arguments. Hidden unless LIBRA_TEST=1.",
        serde_json::json!({
            "type": "object",
            "additionalProperties": false,
            "required": ["text"],
            "properties": {"text": {"type": "string"}}
        }),
        serde_json::json!({
            "type": "object",
            "additionalProperties": false,
            "required": ["text"],
            "properties": {"text": {"type": "string"}}
        }),
        echo_fixture,
        echo_invoke,
    )
}

#[cfg(debug_assertions)]
fn echo_fixture(_repo: &Path) -> ToolFixture {
    ToolFixture::new(
        serde_json::json!({"text": "ok"}),
        serde_json::json!({"text": "ok", "extra": true}),
    )
}

#[cfg(debug_assertions)]
fn echo_invoke(arguments: &Value) -> Result<Value, String> {
    Ok(arguments.clone())
}

#[cfg(debug_assertions)]
fn blob_tool() -> Tool {
    Tool::declare(
        "libra_test_blob",
        "Test-only tool that returns a string of the requested length. Hidden unless LIBRA_TEST=1.",
        serde_json::json!({
            "type": "object",
            "additionalProperties": false,
            "required": ["bytes"],
            "properties": {"bytes": {"type": "integer"}}
        }),
        serde_json::json!({
            "type": "object",
            "additionalProperties": false,
            "required": ["len", "data"],
            "properties": {
                "len": {"type": "integer"},
                "data": {"type": "string"}
            }
        }),
        blob_fixture,
        blob_invoke,
    )
}

#[cfg(debug_assertions)]
fn blob_fixture(_repo: &Path) -> ToolFixture {
    ToolFixture::new(
        serde_json::json!({"bytes": 1}),
        serde_json::json!({"bytes": -1}),
    )
}

#[cfg(debug_assertions)]
fn blob_invoke(arguments: &Value) -> Result<Value, String> {
    let bytes = arguments
        .get("bytes")
        .and_then(Value::as_i64)
        .ok_or_else(|| "bytes must be an integer".to_string())?;
    if !(0..=200_000).contains(&bytes) {
        return Err("bytes must be from 0 through 200000".to_string());
    }
    let len =
        usize::try_from(bytes).map_err(|_| "bytes must be from 0 through 200000".to_string())?;
    Ok(serde_json::json!({
        "len": bytes,
        "data": "a".repeat(len),
    }))
}

#[cfg(debug_assertions)]
fn sleep_tool() -> Tool {
    Tool::declare(
        "libra_test_sleep",
        "Test-only tool that sleeps for the requested milliseconds. Hidden unless LIBRA_TEST=1.",
        serde_json::json!({
            "type": "object",
            "additionalProperties": false,
            "required": ["ms"],
            "properties": {"ms": {"type": "integer"}}
        }),
        serde_json::json!({
            "type": "object",
            "additionalProperties": false,
            "required": ["slept_ms"],
            "properties": {"slept_ms": {"type": "integer"}}
        }),
        sleep_fixture,
        sleep_invoke,
    )
}

#[cfg(debug_assertions)]
fn sleep_fixture(_repo: &Path) -> ToolFixture {
    ToolFixture::new(serde_json::json!({"ms": 0}), serde_json::json!({"ms": -1}))
}

#[cfg(debug_assertions)]
fn sleep_invoke(arguments: &Value) -> Result<Value, String> {
    let ms = arguments
        .get("ms")
        .and_then(Value::as_i64)
        .ok_or_else(|| "ms must be an integer".to_string())?;
    if !(0..=120_000).contains(&ms) {
        return Err("ms must be from 0 through 120000".to_string());
    }
    let duration_ms =
        u64::try_from(ms).map_err(|_| "ms must be from 0 through 120000".to_string())?;
    std::thread::sleep(std::time::Duration::from_millis(duration_ms));
    Ok(serde_json::json!({"slept_ms": ms}))
}

#[cfg(debug_assertions)]
fn block_tool() -> Tool {
    Tool::declare(
        "libra_test_block",
        "Test-only tool that blocks inside the read worker. Hidden unless LIBRA_TEST=1.",
        serde_json::json!({
            "type": "object",
            "additionalProperties": false,
            "required": ["pid_file"],
            "properties": {"pid_file": {"type": "string"}}
        }),
        serde_json::json!({
            "type": "object",
            "additionalProperties": false,
            "required": ["pid"],
            "properties": {"pid": {"type": "integer"}}
        }),
        block_fixture,
        block_invoke,
    )
}

#[cfg(debug_assertions)]
fn block_fixture(repo: &Path) -> ToolFixture {
    let pid_file = repo.join("block.pid");
    ToolFixture::new(
        serde_json::json!({"pid_file": pid_file.display().to_string()}),
        serde_json::json!({"pid_file": 1}),
    )
}

#[cfg(debug_assertions)]
fn block_invoke(arguments: &Value) -> Result<Value, String> {
    let path = arguments
        .get("pid_file")
        .and_then(Value::as_str)
        .ok_or_else(|| "pid_file must be a string".to_string())?;
    let pid = std::process::id();
    std::fs::write(path, pid.to_string())
        .map_err(|_| "pid file could not be written".to_string())?;
    // Exec tests set this so a descendant outside the worker process group
    // keeps stdout open after the worker is killed.
    #[cfg(unix)]
    if std::env::var("LIBRA_MCP_TEST_HOLD_STDOUT").ok().as_deref() == Some("1") {
        spawn_stdout_holder(path);
    }
    // The conformance fixture must return. Exec tests set this to hold the slot.
    if std::env::var("LIBRA_MCP_TEST_BLOCK").ok().as_deref() == Some("1") {
        loop {
            std::thread::sleep(std::time::Duration::from_secs(60));
        }
    }
    Ok(serde_json::json!({"pid": pid}))
}

/// Fork a child that leaves the worker's process group and inherits stdout.
///
/// The child only calls libc. It must not return into the multi-threaded
/// runtime that `fork` duplicated.
#[cfg(all(debug_assertions, unix))]
fn spawn_stdout_holder(pid_file: &str) {
    // SAFETY: this runs only in the debug test tool. The child calls `setpgid`
    // and `sleep` and does not return into the multi-threaded runtime.
    let child = unsafe { libc::fork() };
    if child == 0 {
        // SAFETY: this is the forked child. It must not run Rust destructors.
        unsafe {
            libc::setpgid(0, 0);
            loop {
                libc::sleep(3600);
            }
        }
    }
    if child <= 0 {
        return;
    }
    let started = std::time::Instant::now();
    while started.elapsed() < std::time::Duration::from_secs(2) {
        // SAFETY: `child` is the pid just returned by `fork`.
        let pgid = unsafe { libc::getpgid(child) };
        if pgid == child {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    let _ = std::fs::write(format!("{pid_file}.hold"), child.to_string());
}

#[cfg(debug_assertions)]
fn hash_kind_tool() -> Tool {
    Tool::declare(
        "libra_test_hash_kind",
        "Test-only tool that returns the worker hash kind. Hidden unless LIBRA_TEST=1.",
        serde_json::json!({
            "type": "object",
            "additionalProperties": false,
            "properties": {}
        }),
        serde_json::json!({
            "type": "object",
            "additionalProperties": false,
            "required": ["hash_kind"],
            "properties": {"hash_kind": {"type": "string"}}
        }),
        hash_kind_fixture,
        hash_kind_invoke,
    )
}

#[cfg(debug_assertions)]
fn hash_kind_fixture(_repo: &Path) -> ToolFixture {
    ToolFixture::new(serde_json::json!({}), serde_json::json!({"extra": true}))
}

#[cfg(debug_assertions)]
fn hash_kind_invoke(_arguments: &Value) -> Result<Value, String> {
    Ok(serde_json::json!({
        "hash_kind": crate::internal::object_format::as_str(git_internal::hash::get_hash_kind())
    }))
}
