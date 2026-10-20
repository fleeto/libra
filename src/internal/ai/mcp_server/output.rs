//! Tool-result envelope (plan-20261019 MC-16, GC-MC-03).
//!
//! Every tool result is serialized, passed through `review::redact_untrusted`,
//! parsed again, and then stripped of control characters. A failure becomes
//! tool error `LBR-AGENT-034`. The original bytes are never returned.

use serde_json::Value;

use super::protocol;

/// Redact a JSON-RPC response when its `result` is a tool result.
///
/// Handshake and `ping` results have no `structuredContent` and are unchanged.
pub fn finalize_tool_result(message: Value) -> Value {
    let Some(result) = message.get("result").cloned() else {
        return message;
    };
    if result.get("structuredContent").is_none() {
        return message;
    }
    let id = message.get("id").cloned().unwrap_or(Value::Null);
    match redact_tool_result(&result) {
        Ok(clean) => {
            let mut message = message;
            if let Some(object) = message.as_object_mut() {
                object.insert("result".to_string(), clean);
            }
            message
        }
        Err(reason) => protocol::jsonrpc_result(
            id,
            protocol::tool_error_result("LBR-AGENT-034", &reason, false),
        ),
    }
}

fn redact_tool_result(result: &Value) -> Result<Value, String> {
    let bytes = serde_json::to_vec(result).map_err(|_| {
        "tool result could not be serialized for redaction; the original result was discarded"
            .to_string()
    })?;
    let (redacted, _) = crate::internal::ai::review::redact_untrusted(&bytes);
    let parsed: Value = serde_json::from_str(&redacted).map_err(|_| {
        "tool result redaction did not produce JSON; the original result was discarded".to_string()
    })?;
    Ok(scrub_json(parsed))
}

fn scrub_json(value: Value) -> Value {
    match value {
        Value::String(text) => Value::String(crate::internal::ai::review::scrub_controls(&text)),
        Value::Array(items) => Value::Array(items.into_iter().map(scrub_json).collect()),
        Value::Object(map) => Value::Object(
            map.into_iter()
                .map(|(key, item)| {
                    (
                        crate::internal::ai::review::scrub_controls(&key),
                        scrub_json(item),
                    )
                })
                .collect(),
        ),
        other => other,
    }
}
