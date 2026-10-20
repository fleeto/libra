//! Tool-result envelope (plan-20261019 MC-16, GC-MC-03).
//!
//! `structuredContent` is scrubbed, serialized, passed through
//! `review::redact_untrusted`, parsed again, and scrubbed once more.
//! Scrubbing first keeps a JSON `\u` escape from gluing to a following secret.
//! `content[0].text` is rebuilt from that scrubbed value so it cannot keep a
//! pre-scrub encoding.
//! A failure becomes tool error `LBR-AGENT-034`. The original bytes are never
//! returned.

use serde_json::Value;

use super::protocol;

/// Redact a JSON-RPC response when its `result` is a tool result.
///
/// Handshake and `ping` results have no `structuredContent` and are unchanged.
/// A tool result's text copy is rebuilt from the scrubbed `structuredContent`.
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
            let Some(object) = message.as_object_mut() else {
                return protocol::jsonrpc_result(
                    id,
                    protocol::tool_error_result(
                        "LBR-AGENT-034",
                        "tool result could not be replaced; the original result was discarded",
                        false,
                    ),
                );
            };
            object.insert("result".to_string(), clean);
            message
        }
        Err(reason) => protocol::jsonrpc_result(
            id,
            protocol::tool_error_result("LBR-AGENT-034", &reason, false),
        ),
    }
}

fn redact_tool_result(result: &Value) -> Result<Value, String> {
    let Some(structured) = result.get("structuredContent") else {
        return Err(
            "tool result is missing structuredContent; the original result was discarded"
                .to_string(),
        );
    };
    // JSON would encode ESC as `\u001b`. That trailing `b` glues to a following
    // token and the redactor's word boundary misses it. Scrub first.
    let scrubbed = scrub_json(structured.clone());
    let bytes = serde_json::to_vec(&scrubbed).map_err(|_| {
        "tool result could not be serialized for redaction; the original result was discarded"
            .to_string()
    })?;
    let (redacted, _) = crate::internal::ai::review::redact_untrusted(&bytes);
    let parsed: Value = serde_json::from_str(&redacted).map_err(|_| {
        "tool result redaction did not produce JSON; the original result was discarded".to_string()
    })?;
    let clean = scrub_json(parsed);
    let text = serde_json::to_string(&clean).map_err(|_| {
        "tool result could not be serialized after redaction; the original result was discarded"
            .to_string()
    })?;
    let mut rebuilt = result.clone();
    let Some(object) = rebuilt.as_object_mut() else {
        return Err(
            "tool result was not a JSON object; the original result was discarded".to_string(),
        );
    };
    object.insert("structuredContent".to_string(), clean);
    object.insert(
        "content".to_string(),
        serde_json::json!([{ "type": "text", "text": text }]),
    );
    Ok(rebuilt)
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

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn text_copy_is_rebuilt_from_scrubbed_structured_content() {
        let secret = format!("{}_{}", "ghp", "abcdefghijklmnopqrstuvwxyz0123456789");
        let structured = json!({"text": format!("a\u{001b}{secret}")});
        let message =
            protocol::jsonrpc_result(json!(2), protocol::tool_success_result(&structured));
        let clean = finalize_tool_result(message);
        let result = &clean["result"];
        let text = result["content"][0]["text"].as_str().expect("content text");
        let decoded: Value = serde_json::from_str(text).expect("text is json");
        assert_eq!(&decoded, &result["structuredContent"]);
        let rendered = serde_json::to_string(&clean).expect("render response");
        assert!(!rendered.contains(&secret));
        assert!(!rendered.contains('\u{001b}'));
        let decoded_text = decoded["text"].as_str().expect("echo text");
        assert!(!decoded_text.contains('\u{001b}'));
        assert!(decoded_text.contains('\u{FFFD}'));
    }
}
