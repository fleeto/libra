//! Read only the Hermes session header. Message bodies stay unread.

use serde_json::{Map, Value};

use super::scope::{
    HermesScopeCandidate, HermesScopeDecision, HermesVerifiedWorktree, admit_receiver,
};

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
