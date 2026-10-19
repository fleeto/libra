//! Claude Code hook envelope parser.
//!
//! Event names come from [`super::events`]. This module does not keep a second
//! name table. Unknown names and events that are not on the current observe
//! ingress return a fixed diagnostic that does not include the payload.

use anyhow::{Result, bail};

use super::{
    super::super::lifecycle::{LifecycleEvent, SessionHookEnvelope, build_lifecycle_event},
    events,
};

/// Translate a Claude hook event name into a canonical lifecycle event.
///
/// Functional scope: names on the current ingress take their lifecycle kind
/// from the single registry and then [`build_lifecycle_event`] extracts the
/// standard fields from `envelope.extra`.
///
/// Boundary conditions: an unknown name, or a known name that is not on the
/// current ingress, is an error whose text does not include the supplied name
/// or any field value. Unknown extra keys stay out of that error; callers
/// classify them with [`events::classify_claude_fields`].
///
/// See: `tests::parser_maps_canonical_hooks`, `tests::parser_rejects_unknown_hook`.
pub(super) fn parse_claude_hook_event(
    hook_event_name: &str,
    envelope: &SessionHookEnvelope,
) -> Result<LifecycleEvent> {
    let Some(spec) = events::spec(hook_event_name) else {
        bail!("unknown Claude Code hook event");
    };
    if !spec.on_current_ingress {
        bail!("Claude hook event is not on the current observe ingress");
    }
    let Some(kind) = spec.kind else {
        bail!("Claude hook event is not on the current observe ingress");
    };
    let mut event = build_lifecycle_event(kind, envelope);
    // DATA-CAP: a thinking string on the hook frame is ordinary content.
    // It stays off the diagnostic. Callers that log metadata must not print it.
    if event.assistant_message.is_none()
        && let Some(thinking) = envelope
            .extra
            .get("thinking")
            .and_then(|value| value.as_str())
    {
        event.assistant_message = Some(thinking.to_string());
    }
    Ok(event)
}

#[cfg(test)]
mod tests {
    use serde_json::{Map, Value};

    use super::{super::super::super::lifecycle::LifecycleEventKind, *};

    fn canonical_envelope() -> SessionHookEnvelope {
        SessionHookEnvelope {
            hook_event_name: "SessionStart".to_string(),
            session_id: "s1".to_string(),
            cwd: "/tmp".to_string(),
            transcript_path: Some("/tmp/transcript.jsonl".to_string()),
            extra: {
                let mut map = Map::new();
                map.insert("prompt".to_string(), Value::String("hello".to_string()));
                map
            },
        }
    }

    // Scenario: the well-known Claude hook names map to their canonical kinds.
    #[test]
    fn parser_maps_canonical_hooks() {
        let envelope = canonical_envelope();
        let cases = [
            ("SessionStart", LifecycleEventKind::SessionStart),
            ("UserPromptSubmit", LifecycleEventKind::TurnStart),
            ("PreToolUse", LifecycleEventKind::ToolUse),
            ("PostToolUse", LifecycleEventKind::ToolUse),
            ("Stop", LifecycleEventKind::TurnEnd),
            ("SessionEnd", LifecycleEventKind::SessionEnd),
        ];

        for (name, kind) in cases {
            let event = parse_claude_hook_event(name, &envelope).expect("parse should succeed");
            assert_eq!(event.kind, kind);
        }
    }

    // Scenario (DR-00): a `PreToolUse` envelope carries no `tool_response` (the
    // tool has not run yet). It must still parse to a `ToolUse` lifecycle event
    // with `tool_response == None`, never a hard error.
    #[test]
    fn parser_maps_pretooluse_without_response() {
        let mut envelope = canonical_envelope();
        envelope.hook_event_name = "PreToolUse".to_string();
        envelope
            .extra
            .insert("tool_name".to_string(), Value::String("Bash".to_string()));
        // Deliberately no "tool_response" key in the pre-phase envelope.
        let event =
            parse_claude_hook_event("PreToolUse", &envelope).expect("PreToolUse must parse");
        assert_eq!(event.kind, LifecycleEventKind::ToolUse);
        assert!(
            event.tool_response.is_none(),
            "PreToolUse carries no tool result yet"
        );
    }

    // Scenario: a hook name not in the known set returns an error.
    #[test]
    fn parser_rejects_unknown_hook() {
        let mut envelope = canonical_envelope();
        envelope.hook_event_name = "UnknownHook".to_string();
        assert!(parse_claude_hook_event("UnknownHook", &envelope).is_err());
    }
}
