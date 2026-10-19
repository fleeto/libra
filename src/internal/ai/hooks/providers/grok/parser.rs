//! Parse one Grok hook envelope and read that session's `updates.jsonl`.
//!
//! The session directory is `sessions/<url-encoded cwd>/<session id>/`.
//! A supplied `transcript_path` is accepted only when it is that directory's
//! ordinary `updates.jsonl`. This module does not start a process.

use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

use chrono::Utc;
use serde_json::Value;
use sha2::{Digest, Sha256};

use super::super::super::{
    lifecycle::{LifecycleEvent, LifecycleEventKind, SessionHookEnvelope},
    provider::CANONICAL_DEDUP_IDENTITY_KEYS,
};
use crate::internal::ai::observed_agents::grok_updates::{
    GrokReadCursor, GrokTranscriptEvent, read_grok_updates,
};

const SESSION_ID_MAX: usize = 128;
const ENCODED_CWD_MAX: usize = 255;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GrokObserveOutcome {
    pub exit_code: i32,
    pub stdout: String,
    pub complete: bool,
    pub pending: bool,
    pub lifecycle_only: bool,
    pub refused: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
struct CaptureStore {
    thoughts: Vec<String>,
    links: Vec<String>,
    offset: u64,
    seen: BTreeSet<String>,
    complete: bool,
}

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

pub fn observe_grok_hook(
    sessions_root: &Path,
    state_dir: &Path,
    event: &Value,
    budget_ms: Option<u64>,
) -> GrokObserveOutcome {
    let event_name = event
        .get("hook_event_name")
        .and_then(Value::as_str)
        .unwrap_or("");
    let session_id = event
        .get("sessionId")
        .or_else(|| event.get("session_id"))
        .and_then(Value::as_str)
        .unwrap_or("");
    let cwd = event.get("cwd").and_then(Value::as_str).unwrap_or("");
    let parent = event
        .get("parentSessionId")
        .and_then(Value::as_str)
        .filter(|value| session_id_is_safe(value))
        .map(str::to_string);
    if !recognizes_grok_event(event_name) || !session_id_is_safe(session_id) || cwd.is_empty() {
        write_lifecycle(state_dir, session_id, event_name);
        return outcome(false, false, true, false);
    }
    if event_name == "SessionEnd" && budget_ms.is_some_and(|budget| budget < 2_000) {
        write_pending(state_dir, session_id);
        let mut store = load_store(state_dir, session_id);
        store.complete = false;
        save_store(state_dir, session_id, &store);
        return outcome(false, true, false, false);
    }
    let envelope = SessionHookEnvelope {
        hook_event_name: event_name.to_string(),
        session_id: session_id.to_string(),
        cwd: cwd.to_string(),
        transcript_path: event
            .get("transcript_path")
            .and_then(Value::as_str)
            .map(str::to_string),
        extra: serde_json::Map::new(),
    };
    if parse_grok_hook_event(event_name, &envelope).is_err() {
        write_lifecycle(state_dir, session_id, event_name);
        return outcome(false, false, true, false);
    }
    if event_name == "PreCompact" {
        let store = load_store(state_dir, session_id);
        write_json(
            &state_path(state_dir, session_id).join("checkpoint.json"),
            &serde_json::json!({ "thoughts": store.thoughts }),
        );
    }
    let Some(updates) = resolve_updates(
        sessions_root,
        cwd,
        session_id,
        envelope.transcript_path.as_deref(),
    ) else {
        write_lifecycle(state_dir, session_id, event_name);
        return outcome(false, false, true, true);
    };
    let mut store = load_store(state_dir, session_id);
    let seen_before = store.seen.clone();
    let offset_before = store.offset;
    let file_len = fs::symlink_metadata(&updates).map(|meta| meta.len()).ok();
    if file_len.is_some_and(|len| len < store.offset) {
        store.offset = 0;
        store.seen.clear();
    }
    let session_dir = updates.parent().unwrap_or(sessions_root);
    let read = read_grok_updates(
        session_dir,
        &updates,
        &GrokReadCursor {
            offset: store.offset,
            seen_event_ids: store.seen.clone(),
        },
    );
    let Ok(read) = read else {
        write_pending(state_dir, session_id);
        store.complete = false;
        save_store(state_dir, session_id, &store);
        return outcome(false, true, false, false);
    };
    if !read.is_complete() {
        write_pending(state_dir, session_id);
        store.complete = false;
        save_store(state_dir, session_id, &store);
        return outcome(false, true, false, false);
    }
    for event in &read.events {
        if let GrokTranscriptEvent::Thought { text, .. } = event {
            store.thoughts.push(text.clone());
        }
    }
    if repeated_event_id(&updates, offset_before, &seen_before) {
        store.seen.clear();
        store.offset = file_len.unwrap_or(read.next_offset);
    } else {
        store.offset = read.next_offset;
        store.seen = read.seen_event_ids;
    }
    store.complete = matches!(
        event_name,
        "Stop" | "StopCancelled" | "StopFailure" | "PreCompact"
    );
    save_store(state_dir, session_id, &store);
    if let Some(parent_id) = parent.as_deref() {
        let mut parent_store = load_store(state_dir, parent_id);
        if !parent_store.links.iter().any(|link| link == session_id) {
            parent_store.links.push(session_id.to_string());
        }
        save_store(state_dir, parent_id, &parent_store);
    }
    outcome(store.complete, false, false, false)
}

fn resolve_updates(
    sessions_root: &Path,
    cwd: &str,
    session_id: &str,
    supplied: Option<&str>,
) -> Option<PathBuf> {
    let derived = sessions_root
        .join(encode_grok_cwd(cwd))
        .join(session_id)
        .join("updates.jsonl");
    let path = match supplied {
        None => derived,
        Some(supplied) => {
            if !supplied_matches_session(&derived, Path::new(supplied)) {
                return None;
            }
            PathBuf::from(supplied)
        }
    };
    let meta = fs::symlink_metadata(&path).ok()?;
    if meta.file_type().is_symlink() || !meta.file_type().is_file() {
        return None;
    }
    Some(path)
}

fn supplied_matches_session(derived: &Path, supplied: &Path) -> bool {
    if supplied_is_unsafe(supplied) {
        return false;
    }
    let Ok(meta) = fs::symlink_metadata(supplied) else {
        return false;
    };
    if meta.file_type().is_symlink() || !meta.file_type().is_file() {
        return false;
    }
    let Ok(supplied_canon) = fs::canonicalize(supplied) else {
        return false;
    };
    let Ok(derived_canon) = fs::canonicalize(derived) else {
        return false;
    };
    supplied_canon == derived_canon
        && supplied_canon
            .file_name()
            .is_some_and(|name| name == "updates.jsonl")
}

fn supplied_is_unsafe(path: &Path) -> bool {
    path.components()
        .any(|component| matches!(component, std::path::Component::ParentDir))
}

fn repeated_event_id(path: &Path, offset: u64, seen: &BTreeSet<String>) -> bool {
    if seen.is_empty() {
        return false;
    }
    let Ok(bytes) = fs::read(path) else {
        return false;
    };
    let start = usize::try_from(offset).unwrap_or(bytes.len());
    let tail = bytes.get(start..).unwrap_or_default();
    let text = String::from_utf8_lossy(tail);
    for line in text.lines() {
        let Ok(value) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        let id = value
            .pointer("/params/_meta/eventId")
            .and_then(Value::as_str)
            .unwrap_or("");
        if seen.contains(id) {
            return true;
        }
    }
    false
}

fn outcome(
    complete: bool,
    pending: bool,
    lifecycle_only: bool,
    refused: bool,
) -> GrokObserveOutcome {
    GrokObserveOutcome {
        exit_code: 0,
        stdout: "{}".to_string(),
        complete,
        pending,
        lifecycle_only,
        refused,
    }
}

fn session_id_is_safe(session_id: &str) -> bool {
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

fn state_path(state_dir: &Path, session_id: &str) -> PathBuf {
    state_dir.join(session_id)
}

fn load_store(state_dir: &Path, session_id: &str) -> CaptureStore {
    let path = state_path(state_dir, session_id).join("store.json");
    let Ok(bytes) = fs::read(&path) else {
        return CaptureStore::default();
    };
    serde_json::from_slice(&bytes).unwrap_or_default()
}

fn save_store(state_dir: &Path, session_id: &str, store: &CaptureStore) {
    write_json(
        &state_path(state_dir, session_id).join("store.json"),
        &serde_json::json!(store),
    );
}

fn write_pending(state_dir: &Path, session_id: &str) {
    write_json(
        &state_path(state_dir, session_id).join("pending.json"),
        &serde_json::json!({ "pending": true, "complete": false }),
    );
}

fn write_lifecycle(state_dir: &Path, session_id: &str, event_name: &str) {
    let id = if session_id_is_safe(session_id) {
        session_id
    } else {
        "unknown"
    };
    write_json(
        &state_path(state_dir, id).join("lifecycle.json"),
        &serde_json::json!({ "event": event_name }),
    );
}

fn write_json(path: &Path, value: &impl serde::Serialize) {
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let Ok(bytes) = serde_json::to_vec_pretty(value) else {
        return;
    };
    let _ = fs::write(path, bytes);
}
