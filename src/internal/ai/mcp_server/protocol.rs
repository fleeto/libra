//! MCP protocol constants, JSON-RPC framing, and the tool-error schema.
//!
//! This module is the only place MCP protocol versions, JSON-RPC error codes,
//! the frame cap, and `TOOL_ERROR_SCHEMA` are defined (GC-MC-02).

use serde_json::{Map, Value};
use sha2::{Digest, Sha256};

/// Legacy handshake versions this server accepts (ADR-MC-04).
pub const LEGACY_PROTOCOL_VERSIONS: &[&str] = &["2025-11-25", "2025-06-18"];

/// Version echoed when the client asks for anything outside the legacy set.
pub const DEFAULT_PROTOCOL_VERSION: &str = "2025-11-25";

/// Maximum accepted stdin line, excluding the trailing newline (ADR-MC-08).
pub const MAX_LINE_BYTES: usize = 256 * 1024;

pub const JSONRPC_PARSE_ERROR: i64 = -32700;
pub const JSONRPC_INVALID_REQUEST: i64 = -32600;
pub const JSONRPC_METHOD_NOT_FOUND: i64 = -32601;
pub const JSONRPC_INVALID_PARAMS: i64 = -32602;

pub const SERVER_NAME: &str = "libra";

/// Stable tool-error object. Every tool `outputSchema.anyOf` includes this.
pub fn tool_error_schema() -> Value {
    serde_json::json!({
        "type": "object",
        "additionalProperties": false,
        "required": ["stable_code", "bridge_code", "message", "retryable"],
        "properties": {
            "stable_code": {"type": "string"},
            "bridge_code": {"type": ["string", "null"]},
            "message": {"type": "string"},
            "retryable": {"type": "boolean"}
        }
    })
}

/// `outputSchema` required by the 2025 legacy wire schemas: root `type` is
/// `object`, and the error shape is always one `anyOf` branch.
pub fn output_schema_with_error(success_schema: Value) -> Value {
    serde_json::json!({
        "type": "object",
        "anyOf": [success_schema, tool_error_schema()]
    })
}

pub fn negotiate_legacy_version(requested: Option<&str>) -> &'static str {
    match requested {
        Some(version) if LEGACY_PROTOCOL_VERSIONS.contains(&version) => {
            if version == "2025-06-18" {
                "2025-06-18"
            } else {
                DEFAULT_PROTOCOL_VERSION
            }
        }
        _ => DEFAULT_PROTOCOL_VERSION,
    }
}

pub fn initialize_result(protocol_version: &str) -> Value {
    serde_json::json!({
        "protocolVersion": protocol_version,
        "capabilities": {"tools": {"listChanged": false}},
        "serverInfo": {
            "name": SERVER_NAME,
            "version": env!("CARGO_PKG_VERSION")
        }
    })
}

pub fn tool_success_result(structured: &Value) -> Value {
    let text = serde_json::to_string(structured).unwrap_or_else(|_| "{}".to_string());
    serde_json::json!({
        "content": [{"type": "text", "text": text}],
        "structuredContent": structured,
        "isError": false
    })
}

pub fn tool_error_result(stable_code: &str, message: &str, retryable: bool) -> Value {
    let structured = serde_json::json!({
        "stable_code": stable_code,
        "bridge_code": Value::Null,
        "message": message,
        "retryable": retryable
    });
    let text = serde_json::to_string(&structured).unwrap_or_else(|_| "{}".to_string());
    serde_json::json!({
        "content": [{"type": "text", "text": text}],
        "structuredContent": structured,
        "isError": true
    })
}

pub fn jsonrpc_result(id: Value, result: Value) -> Value {
    serde_json::json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": result
    })
}

pub fn jsonrpc_error(id: Value, code: i64, message: &str) -> Value {
    serde_json::json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": {"code": code, "message": message}
    })
}

/// Whether `value` satisfies the JSON Schema subset used by MCP tool inputs.
pub fn value_matches_schema(value: &Value, schema: &Value) -> bool {
    schema_mismatch(value, schema).is_none()
}

fn schema_mismatch(value: &Value, schema: &Value) -> Option<String> {
    let Some(schema) = schema.as_object() else {
        return Some("schema is not an object".to_string());
    };
    if let Some(branches) = schema.get("anyOf").and_then(Value::as_array) {
        if branches
            .iter()
            .any(|branch| schema_mismatch(value, branch).is_none())
        {
            return None;
        }
        return Some("value matched no anyOf branch".to_string());
    }
    if let Some(expected) = schema.get("const")
        && value != expected
    {
        return Some("value did not equal const".to_string());
    }
    if let Some(options) = schema.get("enum").and_then(Value::as_array)
        && !options.iter().any(|option| option == value)
    {
        return Some("value is not in enum".to_string());
    }
    if let Some(type_name) = schema.get("type")
        && !value_has_type(value, type_name)
    {
        return Some("value has the wrong JSON type".to_string());
    }
    if let Some(object) = value.as_object() {
        if let Some(required) = schema.get("required").and_then(Value::as_array) {
            for key in required {
                let Some(name) = key.as_str() else {
                    continue;
                };
                if !object.contains_key(name) {
                    return Some(format!("missing required property {name}"));
                }
            }
        }
        if let Some(properties) = schema.get("properties").and_then(Value::as_object) {
            for (key, child) in object {
                if let Some(child_schema) = properties.get(key) {
                    if let Some(reason) = schema_mismatch(child, child_schema) {
                        return Some(format!("{key}: {reason}"));
                    }
                } else if !additional_property_allowed(schema, child) {
                    return Some(format!("unexpected property {key}"));
                }
            }
        } else if schema.get("additionalProperties") == Some(&Value::Bool(false))
            && !object.is_empty()
        {
            return Some("object contains unexpected properties".to_string());
        }
    }
    if let Some(items) = value.as_array()
        && let Some(item_schema) = schema.get("items")
    {
        for item in items {
            if let Some(reason) = schema_mismatch(item, item_schema) {
                return Some(reason);
            }
        }
    }
    None
}

fn additional_property_allowed(schema: &Map<String, Value>, child: &Value) -> bool {
    match schema.get("additionalProperties") {
        Some(Value::Bool(false)) => false,
        Some(Value::Bool(true)) | None => true,
        Some(extra) => schema_mismatch(child, extra).is_none(),
    }
}

fn value_has_type(value: &Value, type_name: &Value) -> bool {
    if let Some(name) = type_name.as_str() {
        return matches_type_name(value, name);
    }
    type_name.as_array().is_some_and(|names| {
        names
            .iter()
            .any(|name| name.as_str().is_some_and(|n| matches_type_name(value, n)))
    })
}

fn matches_type_name(value: &Value, name: &str) -> bool {
    match name {
        "object" => value.is_object(),
        "array" => value.is_array(),
        "string" => value.is_string(),
        "boolean" => value.is_boolean(),
        "null" => value.is_null(),
        "number" => value.is_number(),
        "integer" => value.as_i64().is_some() || value.as_u64().is_some(),
        _ => false,
    }
}

/// One stdin line after the framing layer has accepted its size.
pub enum InboundLine {
    Empty,
    Message(Vec<u8>),
    TooLarge,
}

/// Methods whose names may be copied onto a span. Anything else is `unknown`.
const KNOWN_METHODS: &[&str] = &[
    "initialize",
    "notifications/initialized",
    "notifications/cancelled",
    "ping",
    "tools/list",
    "tools/call",
];

pub struct Session {
    pub initialized: bool,
    repository_id: String,
}

impl Session {
    pub fn new() -> Self {
        Self {
            initialized: false,
            repository_id: "unavailable".to_string(),
        }
    }

    /// `repository_id` is the server-side `libra.repoid`, never a client field.
    pub fn with_repository_id(repository_id: impl Into<String>) -> Self {
        Self {
            initialized: false,
            repository_id: repository_id.into(),
        }
    }
}

impl Default for Session {
    fn default() -> Self {
        Self::new()
    }
}

/// Handle one accepted line. `None` means the client gets no stdout line
/// (empty input or a notification).
pub fn handle_message(session: &mut Session, line: &InboundLine) -> Option<Value> {
    if matches!(line, InboundLine::Empty) {
        return None;
    }
    let fields = span_fields(line);
    let span = tracing::info_span!(
        "agent.mcp.request",
        method = fields.method,
        id_hash = %fields.id_hash,
        repository_id = %session.repository_id,
        tool = tracing::field::Empty,
    );
    if let Some(tool) = fields.tool {
        span.record("tool", tool);
    }
    let _entered = span.enter();
    match line {
        InboundLine::TooLarge => Some(jsonrpc_error(
            Value::Null,
            JSONRPC_INVALID_REQUEST,
            "MCP message exceeds 256 KiB; send a shorter line",
        )),
        InboundLine::Message(bytes) => handle_bytes(session, bytes),
        InboundLine::Empty => None,
    }
}

struct SpanFields {
    method: &'static str,
    id_hash: String,
    tool: Option<&'static str>,
}

/// Fields safe to put on `agent.mcp.request`. Client-controlled strings are
/// either replaced with `unknown` or reduced to a hash prefix.
fn span_fields(line: &InboundLine) -> SpanFields {
    let InboundLine::Message(bytes) = line else {
        return SpanFields {
            method: "unknown",
            id_hash: id_hash(&Value::Null),
            tool: None,
        };
    };
    let Ok(text) = std::str::from_utf8(bytes) else {
        return SpanFields {
            method: "unknown",
            id_hash: id_hash(&Value::Null),
            tool: None,
        };
    };
    let Ok(parsed) = serde_json::from_str::<Value>(text) else {
        return SpanFields {
            method: "unknown",
            id_hash: id_hash(&Value::Null),
            tool: None,
        };
    };
    let Some(message) = parsed.as_object() else {
        return SpanFields {
            method: "unknown",
            id_hash: id_hash(&Value::Null),
            tool: None,
        };
    };
    let method = message
        .get("method")
        .and_then(Value::as_str)
        .map(sanitized_method)
        .unwrap_or("unknown");
    let id_hash = message
        .get("id")
        .map(id_hash)
        .unwrap_or_else(|| id_hash(&Value::Null));
    let tool = if method == "tools/call" {
        let name = message
            .get("params")
            .and_then(|params| params.get("name"))
            .and_then(Value::as_str)
            .unwrap_or("");
        Some(sanitized_tool(name))
    } else {
        None
    };
    SpanFields {
        method,
        id_hash,
        tool,
    }
}

fn sanitized_method(method: &str) -> &'static str {
    KNOWN_METHODS
        .iter()
        .copied()
        .find(|known| *known == method)
        .unwrap_or("unknown")
}

fn sanitized_tool(name: &str) -> &'static str {
    crate::internal::ai::mcp_server::tools::registered_tools()
        .into_iter()
        .find(|tool| tool.name == name)
        .map(|tool| tool.name)
        .unwrap_or("unknown")
}

/// First 16 hex characters of SHA-256 over the canonical JSON encoding of `id`.
fn id_hash(id: &Value) -> String {
    let canonical = serde_json::to_vec(id).unwrap_or_else(|_| b"null".to_vec());
    let digest = Sha256::digest(&canonical);
    let mut encoded = String::with_capacity(16);
    for byte in digest.iter().take(8) {
        const HEX: &[u8; 16] = b"0123456789abcdef";
        encoded.push(HEX[(*byte >> 4) as usize] as char);
        encoded.push(HEX[(*byte & 0x0f) as usize] as char);
    }
    encoded
}

fn handle_bytes(session: &mut Session, bytes: &[u8]) -> Option<Value> {
    let text = match std::str::from_utf8(bytes) {
        Ok(text) => text,
        Err(_) => {
            return Some(jsonrpc_error(
                Value::Null,
                JSONRPC_PARSE_ERROR,
                "MCP message is not UTF-8 JSON",
            ));
        }
    };
    let parsed: Value = match serde_json::from_str(text) {
        Ok(value) => value,
        Err(_) => {
            return Some(jsonrpc_error(
                Value::Null,
                JSONRPC_PARSE_ERROR,
                "MCP message is not valid JSON",
            ));
        }
    };
    if parsed.is_array() {
        return Some(jsonrpc_error(
            Value::Null,
            JSONRPC_INVALID_REQUEST,
            "MCP JSON-RPC batches are not supported",
        ));
    }
    let Some(message) = parsed.as_object() else {
        return Some(jsonrpc_error(
            Value::Null,
            JSONRPC_INVALID_REQUEST,
            "MCP message must be a JSON object",
        ));
    };
    if message.get("jsonrpc").and_then(Value::as_str) != Some("2.0") {
        return Some(jsonrpc_error(
            request_id(message),
            JSONRPC_INVALID_REQUEST,
            "MCP message must set jsonrpc to \"2.0\"",
        ));
    }
    if !message.contains_key("id") {
        // Notifications, including `notifications/initialized` and
        // `notifications/cancelled`, produce no response.
        return None;
    }
    let id = match request_id_checked(message) {
        Ok(id) => id,
        Err(response) => return Some(response),
    };
    let Some(method) = message.get("method").and_then(Value::as_str) else {
        return Some(jsonrpc_error(
            id,
            JSONRPC_INVALID_REQUEST,
            "MCP request is missing a method string",
        ));
    };
    if method != "initialize" && !session.initialized {
        return Some(jsonrpc_error(
            id,
            JSONRPC_INVALID_REQUEST,
            "MCP client must call initialize before other methods",
        ));
    }
    match method {
        "initialize" => Some(handle_initialize(session, id, message.get("params"))),
        "ping" => Some(jsonrpc_result(id, serde_json::json!({}))),
        "tools/list" => Some(handle_tools_list(id, message.get("params"))),
        "tools/call" => Some(handle_tools_call(id, message.get("params"))),
        _ => Some(jsonrpc_error(
            id,
            JSONRPC_METHOD_NOT_FOUND,
            "MCP method is not implemented by libra agent mcp",
        )),
    }
}

fn handle_initialize(session: &mut Session, id: Value, params: Option<&Value>) -> Value {
    let requested = params
        .and_then(|value| value.get("protocolVersion"))
        .and_then(Value::as_str);
    if params.is_some_and(|value| !value.is_object()) {
        return jsonrpc_error(
            id,
            JSONRPC_INVALID_PARAMS,
            "initialize params must be a JSON object",
        );
    }
    session.initialized = true;
    jsonrpc_result(id, initialize_result(negotiate_legacy_version(requested)))
}

fn handle_tools_list(id: Value, params: Option<&Value>) -> Value {
    if params.is_some_and(|value| !value.is_object()) {
        return jsonrpc_error(
            id,
            JSONRPC_INVALID_PARAMS,
            "tools/list params must be a JSON object",
        );
    }
    jsonrpc_result(
        id,
        serde_json::json!({"tools": crate::internal::ai::mcp_server::tools::tools_list_wire()}),
    )
}

fn handle_tools_call(id: Value, params: Option<&Value>) -> Value {
    let Some(params) = params.and_then(Value::as_object) else {
        return jsonrpc_error(
            id,
            JSONRPC_INVALID_PARAMS,
            "tools/call params must be a JSON object",
        );
    };
    let Some(name) = params.get("name").and_then(Value::as_str) else {
        return jsonrpc_error(
            id,
            JSONRPC_INVALID_PARAMS,
            "tools/call params.name must be a string",
        );
    };
    let arguments = match params.get("arguments") {
        None => Value::Object(Map::new()),
        Some(value) if value.is_object() => value.clone(),
        Some(_) => {
            return jsonrpc_error(
                id,
                JSONRPC_INVALID_PARAMS,
                "tools/call params.arguments must be a JSON object",
            );
        }
    };
    match crate::internal::ai::mcp_server::tools::call_tool(name, &arguments) {
        Ok(outcome) => jsonrpc_result(id, outcome),
        Err(message) => jsonrpc_error(id, JSONRPC_INVALID_PARAMS, &message),
    }
}

fn request_id(message: &Map<String, Value>) -> Value {
    message.get("id").cloned().unwrap_or(Value::Null)
}

fn request_id_checked(message: &Map<String, Value>) -> Result<Value, Value> {
    match message.get("id") {
        Some(id @ (Value::Null | Value::String(_) | Value::Number(_))) => Ok(id.clone()),
        Some(_) => Err(jsonrpc_error(
            Value::Null,
            JSONRPC_INVALID_REQUEST,
            "MCP request id must be a string, number, or null",
        )),
        None => Ok(Value::Null),
    }
}
