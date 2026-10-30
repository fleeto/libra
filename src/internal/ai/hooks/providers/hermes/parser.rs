//! Read only the Hermes session header. Message bodies stay unread.

use anyhow::{Result, bail};
use chrono::Utc;
use serde_json::{Map, Value};

use super::{
    super::super::lifecycle::{LifecycleEvent, LifecycleEventKind, SessionHookEnvelope},
    scope::{HermesScopeCandidate, HermesScopeDecision, HermesVerifiedWorktree, admit_receiver},
};

const ALLOWED_FIELDS: &[&str] = &[
    "source",
    "created_source",
    "user_id",
    "profile_name",
    "turn_id",
    "message_id",
    "phase",
    "outcome",
    "event_id",
];

const OBSERVERS: &[&str] = &[
    "post_llm_call",
    "on_session_end",
    "on_session_finalize",
    "on_session_reset",
    "agent_loop_stopped",
];

/// Admit or refuse a session value before any message body is walked.
pub fn parse_hermes_scope_value(
    value: &Value,
    verified: &HermesVerifiedWorktree,
) -> HermesScopeDecision {
    let Some(object) = value.as_object() else {
        return HermesScopeDecision::Refuse("hermes scope session is not an object");
    };
    parse_hermes_scope_header(object, verified)
}

pub fn parse_hermes_scope_header(
    session: &Map<String, Value>,
    verified: &HermesVerifiedWorktree,
) -> HermesScopeDecision {
    let candidate = HermesScopeCandidate {
        platform: text_field(session.get("source")),
        created_source: text_field(session.get("created_source")),
        user_id: text_field(session.get("user_id")),
        cwd: text_field(session.get("cwd")),
        recorded_profile: text_field(session.get("profile_name")),
    };
    admit_receiver(verified, &candidate)
}

fn text_field(value: Option<&Value>) -> Option<String> {
    match value {
        Some(Value::String(text)) => Some(text.clone()),
        _ => None,
    }
}

pub fn recognizes_hermes_event(name: &str) -> bool {
    OBSERVERS.contains(&name)
}

pub fn hermes_event_is_ingested(name: &str) -> bool {
    matches!(
        name,
        "post_llm_call" | "on_session_end" | "on_session_finalize" | "agent_loop_stopped"
    )
}

/// Decode one proven observer. Control hooks, subagent events, and message
/// bodies are not accepted. `on_session_end` is a turn result, not session
/// teardown.
pub fn parse_hermes_hook_event(
    hook_event_name: &str,
    envelope: &SessionHookEnvelope,
) -> Result<LifecycleEvent> {
    if !hermes_event_is_ingested(hook_event_name) {
        bail!("unsupported hermes hook event");
    }
    if envelope.transcript_path.is_some() {
        bail!("hermes hook does not accept a transcript path");
    }
    if let Some(key) = envelope
        .extra
        .keys()
        .find(|key| !ALLOWED_FIELDS.contains(&key.as_str()))
    {
        let _ = key;
        bail!("hermes hook field is not allowed");
    }
    let verified = verified_worktree(envelope)?;
    let decision = admit_receiver(
        &verified,
        &HermesScopeCandidate {
            platform: text_field(envelope.extra.get("source")),
            created_source: text_field(envelope.extra.get("created_source")),
            user_id: text_field(envelope.extra.get("user_id")),
            cwd: Some(envelope.cwd.clone()),
            recorded_profile: text_field(envelope.extra.get("profile_name")),
        },
    );
    if let HermesScopeDecision::Refuse(reason) = decision {
        bail!(reason);
    }
    let turn_id = required_text(&envelope.extra, "turn_id")?;
    let message_id = required_text(&envelope.extra, "message_id")?;
    let phase = required_text(&envelope.extra, "phase")?;
    let event_id = required_text(&envelope.extra, "event_id")?;
    let outcome = text_field(envelope.extra.get("outcome"));
    let kind = lifecycle_kind(hook_event_name, outcome.as_deref())?;
    Ok(LifecycleEvent {
        kind,
        session_id: envelope.session_id.clone(),
        session_ref: None,
        prompt: None,
        model: None,
        source: Some(serde_json::json!({
            "provider": "hermes",
            "event_id": event_id,
            "turn_id": turn_id,
            "message_id": message_id,
            "phase": phase,
        })),
        tool_name: None,
        tool_input: None,
        tool_response: None,
        assistant_message: None,
        timestamp: Utc::now(),
    })
}

fn verified_worktree(envelope: &SessionHookEnvelope) -> Result<HermesVerifiedWorktree> {
    let invocation = std::env::current_dir()
        .map_err(|_| anyhow::anyhow!("hermes scope cwd is not the verified worktree"))?;
    let scope = crate::internal::worktree_scope::RequestScope::try_resolve(invocation)
        .map_err(|_| anyhow::anyhow!("hermes scope cwd is not the verified worktree"))?
        .ok_or_else(|| anyhow::anyhow!("hermes scope cwd is not the verified worktree"))?;
    let root = scope
        .worktree_root
        .canonicalize()
        .map_err(|_| anyhow::anyhow!("hermes scope cwd is not the verified worktree"))?;
    let reported = std::path::Path::new(&envelope.cwd)
        .canonicalize()
        .map_err(|_| anyhow::anyhow!("hermes scope cwd is not the verified worktree"))?;
    if reported != root {
        bail!("hermes scope cwd is not the verified worktree");
    }
    let profile =
        text_field(envelope.extra.get("profile_name")).unwrap_or_else(|| "default".to_string());
    Ok(HermesVerifiedWorktree {
        root: reported,
        profile,
    })
}

fn required_text(extra: &Map<String, Value>, key: &str) -> Result<String> {
    match text_field(extra.get(key)) {
        Some(value) if !value.is_empty() => Ok(value),
        _ => bail!("hermes hook is missing an identity field"),
    }
}

fn lifecycle_kind(event: &str, outcome: Option<&str>) -> Result<LifecycleEventKind> {
    match event {
        "post_llm_call" => Ok(LifecycleEventKind::TurnEnd),
        "on_session_finalize" => Ok(LifecycleEventKind::SessionEnd),
        "agent_loop_stopped" => Ok(LifecycleEventKind::TurnInterrupted),
        "on_session_end" => match outcome {
            Some("completed") => Ok(LifecycleEventKind::TurnEnd),
            Some("failed" | "interrupted") => Ok(LifecycleEventKind::TurnInterrupted),
            _ => bail!("hermes turn outcome is not recognized"),
        },
        _ => bail!("unsupported hermes hook event"),
    }
}
