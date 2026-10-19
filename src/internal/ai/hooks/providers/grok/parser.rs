//! Pure Grok hook-envelope decoding. No source reads, state writes, or scope grants.

use chrono::Utc;
use sha2::{Digest, Sha256};

use super::super::super::{
    lifecycle::{LifecycleEvent, LifecycleEventKind, SessionHookEnvelope},
    provider::CANONICAL_DEDUP_IDENTITY_KEYS,
};

const SESSION_ID_MAX: usize = 128;
const ENCODED_CWD_MAX: usize = 255;

pub fn recognizes_grok_event(name: &str) -> bool {
    matches!(
        name,
        "Stop" | "StopCancelled" | "StopFailure" | "PreCompact" | "SessionEnd"
    )
}

pub fn parse_grok_hook_event(
    hook_event_name: &str,
    envelope: &SessionHookEnvelope,
) -> anyhow::Result<LifecycleEvent> {
    if !recognizes_grok_event(hook_event_name) {
        anyhow::bail!("unsupported grok hook event");
    }
    if envelope.session_id.trim().is_empty() || !session_id_is_safe(&envelope.session_id) {
        anyhow::bail!("grok hook session id is missing or unsafe");
    }
    let kind = match hook_event_name {
        "PreCompact" => LifecycleEventKind::Compaction,
        "SessionEnd" => LifecycleEventKind::SessionEnd,
        _ => LifecycleEventKind::TurnEnd,
    };
    Ok(LifecycleEvent {
        kind,
        session_id: envelope.session_id.clone(),
        session_ref: None,
        prompt: None,
        model: None,
        source: Some(serde_json::json!({
            "provider": "grok",
            "keys": CANONICAL_DEDUP_IDENTITY_KEYS,
        })),
        tool_name: None,
        tool_input: None,
        tool_response: None,
        assistant_message: None,
        timestamp: Utc::now(),
    })
}

/// URL-encode `cwd` into one session-directory name.
///
/// Bytes outside RFC 3986 unreserved characters, including `/`, become `%HH`.
/// Names longer than 255 bytes become a short slug, a hyphen, and a SHA-256
/// prefix so the directory name fits.
pub fn encode_grok_cwd(cwd: &str) -> String {
    let mut encoded = String::new();
    for byte in cwd.as_bytes() {
        if is_unreserved(*byte) {
            encoded.push(*byte as char);
        } else {
            encoded.push('%');
            encoded.push(hex_digit(byte >> 4));
            encoded.push(hex_digit(byte & 0x0f));
        }
    }
    if encoded.len() <= ENCODED_CWD_MAX {
        return encoded;
    }
    let digest = format!("{:x}", Sha256::digest(cwd.as_bytes()));
    let hash = &digest[..32];
    let budget = ENCODED_CWD_MAX - 1 - hash.len();
    let mut slug = String::new();
    for byte in cwd.as_bytes() {
        if !is_unreserved(*byte) {
            continue;
        }
        if slug.len() + 1 > budget {
            break;
        }
        slug.push(*byte as char);
    }
    if slug.is_empty() {
        slug.push_str("cwd");
    }
    format!("{slug}-{hash}")
}

pub(super) fn session_id_is_safe(session_id: &str) -> bool {
    !session_id.is_empty()
        && session_id.len() <= SESSION_ID_MAX
        && session_id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-')
}

fn is_unreserved(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~')
}

fn hex_digit(value: u8) -> char {
    char::from(b"0123456789ABCDEF"[usize::from(value)])
}
