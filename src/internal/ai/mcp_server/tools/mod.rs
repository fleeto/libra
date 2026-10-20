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
    match tool.invoke(arguments) {
        Ok(structured) => Ok(tool_success_result(&structured)),
        Err(message) => Ok(tool_error_result("LBR-AGENT-027", &message, false)),
    }
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
