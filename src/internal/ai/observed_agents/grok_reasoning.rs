//! Merge synthetic Grok `chat_history.jsonl` reasoning rows into a GK-02 transcript.
//!
//! Ciphertext stays in the ordinary transcript. Safe projection, metadata,
//! typed coverage, Debug, doctor JSON, the coverage digest, and log lines
//! do not receive that plaintext. This module never decrypts, never opens
//! `GROK_HOME`, and never edits OpenCode export.

use std::{
    fs::File,
    io::{BufReader, Read},
    path::Path,
};

use serde_json::{Map, Value};
use sha2::{Digest, Sha256};

use crate::internal::ai::observed_agents::{
    grok_updates::{GROK_LINE_MAX_BYTES, GrokTranscriptEvent},
    redaction::Redactor,
    transcript_source::{TRANSCRIPT_READ_HARD_CAP_BYTES, TranscriptReadError},
};

#[derive(Clone, PartialEq, Eq)]
pub struct GrokAttachedReasoning {
    pub id: String,
    pub status: String,
    pub encrypted_content: String,
}

impl std::fmt::Debug for GrokAttachedReasoning {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("GrokAttachedReasoning")
            .field("id_len", &self.id.len())
            .field("status_len", &self.status.len())
            .field("encrypted_content_len", &self.encrypted_content.len())
            .finish()
    }
}

#[derive(Clone, PartialEq, Eq)]
pub enum GrokMergedEvent {
    Thought {
        text: String,
        prompt_id: String,
        event_id: String,
        reasoning: Option<GrokAttachedReasoning>,
    },
    Tool {
        name: String,
        event_id: String,
    },
    Unknown {
        session_update: String,
        line_sha256: String,
    },
    Turn {
        reasoning_tokens: u64,
        event_id: String,
    },
    Reasoning {
        id: String,
        status: String,
        summary_text: Option<String>,
        encrypted_content: String,
    },
}

impl std::fmt::Debug for GrokMergedEvent {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Thought {
                text,
                prompt_id,
                event_id,
                reasoning,
            } => formatter
                .debug_struct("Thought")
                .field("text_len", &text.len())
                .field("prompt_id_len", &prompt_id.len())
                .field("event_id_len", &event_id.len())
                .field("reasoning", reasoning)
                .finish(),
            Self::Tool { name, event_id } => formatter
                .debug_struct("Tool")
                .field("name_len", &name.len())
                .field("event_id_len", &event_id.len())
                .finish(),
            Self::Unknown {
                session_update,
                line_sha256,
            } => formatter
                .debug_struct("Unknown")
                .field("session_update", session_update)
                .field("line_sha256", line_sha256)
                .finish(),
            Self::Turn {
                reasoning_tokens,
                event_id,
            } => formatter
                .debug_struct("Turn")
                .field("reasoning_tokens", reasoning_tokens)
                .field("event_id_len", &event_id.len())
                .finish(),
            Self::Reasoning {
                id,
                status,
                summary_text,
                encrypted_content,
            } => formatter
                .debug_struct("Reasoning")
                .field("id_len", &id.len())
                .field("status_len", &status.len())
                .field(
                    "summary_len",
                    &summary_text.as_ref().map(String::len).unwrap_or(0),
                )
                .field("encrypted_content_len", &encrypted_content.len())
                .finish(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GrokReasoningImport {
    pub events: Vec<GrokMergedEvent>,
    pub partial: bool,
}

impl GrokReasoningImport {
    pub fn is_complete(&self) -> bool {
        !self.partial
    }

    pub fn transcript_document(&self) -> Value {
        Value::Object(transcript_map(self))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum GrokReasoningError {
    #[error("{0}")]
    Read(#[from] TranscriptReadError),
    #[error("malformed transcript: {reason}")]
    Malformed { reason: String },
    #[error("refusing to read '{path}': {reason}")]
    Refused { path: String, reason: String },
    #[error("failed to read '{path}': {detail}")]
    Io { path: String, detail: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GrokSinkBundle {
    pub metadata_json: String,
    pub typed_coverage_json: String,
    pub debug_text: String,
    pub safe_projection_json: String,
    pub doctor_json: String,
    pub coverage_digest: String,
    pub log_line: String,
}

struct ThoughtSlot {
    text_raw: String,
    text_stored: String,
    prompt_id: String,
    event_id: String,
    reasoning: Option<GrokAttachedReasoning>,
}

/// Read `chat_history.jsonl` and attach reasoning rows to `updates` events.
///
/// A summary attaches only when its text equals one thought, or equals the
/// ordered join of at most four consecutive thoughts that share a `promptId`.
/// Anything else becomes its own event. Empty summaries keep ciphertext and
/// do not invent body text.
pub fn import_grok_reasoning(
    session_dir: &Path,
    chat_history: &Path,
    updates: &[GrokTranscriptEvent],
) -> Result<GrokReasoningImport, GrokReasoningError> {
    let rows = read_history(session_dir, chat_history)?;
    let mut slots = Vec::new();
    for event in updates {
        if let GrokTranscriptEvent::Thought {
            text,
            prompt_id,
            event_id,
        } = event
        {
            let (stored, _) = scan_prose(text);
            slots.push(ThoughtSlot {
                text_raw: text.clone(),
                text_stored: stored,
                prompt_id: prompt_id.clone(),
                event_id: event_id.clone(),
                reasoning: None,
            });
        }
    }
    let mut partial = false;
    let mut standalone = Vec::new();
    for row in rows {
        match row {
            HistoryRow::Partial => partial = true,
            HistoryRow::Reasoning(row) => {
                if row.summaries.is_empty() {
                    standalone.push(ciphertext_only(row));
                    continue;
                }
                let mut kept_text = false;
                for summary in &row.summaries {
                    let Some(summary_raw) = summary else {
                        partial = true;
                        continue;
                    };
                    kept_text = true;
                    push_summary(&mut slots, &mut standalone, &row, summary_raw);
                }
                if !kept_text {
                    standalone.push(ciphertext_only(row));
                }
            }
        }
    }
    let mut events = Vec::new();
    let mut slot_at = 0;
    for event in updates {
        match event {
            GrokTranscriptEvent::Thought { .. } => {
                let slot = &slots[slot_at];
                slot_at += 1;
                events.push(GrokMergedEvent::Thought {
                    text: slot.text_stored.clone(),
                    prompt_id: slot.prompt_id.clone(),
                    event_id: slot.event_id.clone(),
                    reasoning: slot.reasoning.clone(),
                });
            }
            GrokTranscriptEvent::Tool { name, event_id } => {
                events.push(GrokMergedEvent::Tool {
                    name: name.clone(),
                    event_id: event_id.clone(),
                });
            }
            GrokTranscriptEvent::Unknown {
                session_update,
                line_sha256,
            } => events.push(GrokMergedEvent::Unknown {
                session_update: session_update.clone(),
                line_sha256: line_sha256.clone(),
            }),
            GrokTranscriptEvent::Turn {
                reasoning_tokens,
                event_id,
            } => events.push(GrokMergedEvent::Turn {
                reasoning_tokens: *reasoning_tokens,
                event_id: event_id.clone(),
            }),
        }
    }
    events.extend(standalone);
    Ok(GrokReasoningImport { events, partial })
}

fn ciphertext_only(row: ReasoningRow) -> GrokMergedEvent {
    GrokMergedEvent::Reasoning {
        id: row.id,
        status: row.status,
        summary_text: None,
        encrypted_content: row.encrypted_content,
    }
}

fn push_summary(
    slots: &mut [ThoughtSlot],
    standalone: &mut Vec<GrokMergedEvent>,
    row: &ReasoningRow,
    summary_raw: &str,
) {
    if let Some(index) = attach_index(slots, summary_raw)
        && slots[index].reasoning.is_none()
    {
        slots[index].reasoning = Some(GrokAttachedReasoning {
            id: row.id.clone(),
            status: row.status.clone(),
            encrypted_content: row.encrypted_content.clone(),
        });
        return;
    }
    let (stored, _) = scan_prose(summary_raw);
    standalone.push(GrokMergedEvent::Reasoning {
        id: row.id.clone(),
        status: row.status.clone(),
        summary_text: Some(stored),
        encrypted_content: row.encrypted_content.clone(),
    });
}

/// Ordinary export bytes. The checkpoint transcript is left untouched.
/// Keys removed: `encrypted_content`, `metadata.openai.reasoningEncryptedContent`,
/// and `metadata.anthropic.signature`.
pub fn export_grok_ordinary(document: &Value) -> Result<Vec<u8>, GrokReasoningError> {
    let mut copy = document.clone();
    strip_export_value(&mut copy);
    serde_json::to_vec(&copy).map_err(|error| GrokReasoningError::Malformed {
        reason: format!("export could not be encoded: {error}"),
    })
}

pub fn sink_bundle(import: &GrokReasoningImport) -> GrokSinkBundle {
    let mut redacted_samples = Vec::new();
    for event in &import.events {
        match event {
            GrokMergedEvent::Thought { text, .. } => {
                if text.contains("<REDACTED:") {
                    redacted_samples.push(text.clone());
                }
            }
            GrokMergedEvent::Reasoning {
                summary_text: Some(text),
                ..
            } if text.contains("<REDACTED:") => redacted_samples.push(text.clone()),
            _ => {}
        }
    }
    let projection = serde_json::json!({
        "source": "unproven",
        "completeness": "partial",
        "events": import.events.iter().map(projection_event).collect::<Vec<_>>(),
    });
    let projection_bytes = serde_json::to_vec(&projection).unwrap_or_default();
    let digest = format!("{:x}", Sha256::digest(&projection_bytes));
    let metadata = serde_json::json!({
        "agent_kind": "grok",
        "source": "unproven",
        "completeness": "partial",
        "event_count": import.events.len(),
    });
    let coverage = serde_json::json!({
        "schema": "grok-unproven",
        "completeness": "partial",
        "source": "unproven",
        "record_count": import.events.len(),
    });
    let doctor = serde_json::json!({
        "agent_kind": "grok",
        "partial": import.partial,
        "event_count": import.events.len(),
        "scanned_text": redacted_samples,
    });
    let debug_text = format!(
        "grok reasoning events={} partial={}",
        import.events.len(),
        import.partial
    );
    let log_line = format!(
        "grok capture events={} partial={}",
        import.events.len(),
        import.partial
    );
    GrokSinkBundle {
        metadata_json: metadata.to_string(),
        typed_coverage_json: coverage.to_string(),
        debug_text,
        safe_projection_json: String::from_utf8_lossy(&projection_bytes).into_owned(),
        doctor_json: doctor.to_string(),
        coverage_digest: digest,
        log_line,
    }
}

fn projection_event(event: &GrokMergedEvent) -> Value {
    match event {
        GrokMergedEvent::Thought { event_id, .. } => {
            serde_json::json!({"kind": "thought", "event_id": event_id})
        }
        GrokMergedEvent::Tool { name, event_id } => {
            serde_json::json!({"kind": "tool", "name": name, "event_id": event_id})
        }
        GrokMergedEvent::Unknown { session_update, .. } => {
            serde_json::json!({"kind": "unknown", "session_update": session_update})
        }
        GrokMergedEvent::Turn {
            reasoning_tokens,
            event_id,
        } => serde_json::json!({
            "kind": "turn",
            "reasoning_tokens": reasoning_tokens,
            "event_id": event_id,
        }),
        GrokMergedEvent::Reasoning { id, .. } => {
            serde_json::json!({"kind": "reasoning", "present": true, "id_len": id.len()})
        }
    }
}

fn transcript_map(import: &GrokReasoningImport) -> Map<String, Value> {
    let mut map = Map::new();
    map.insert("source".to_string(), Value::String("unproven".to_string()));
    map.insert(
        "completeness".to_string(),
        Value::String("partial".to_string()),
    );
    map.insert("partial".to_string(), Value::Bool(import.partial));
    let events = import
        .events
        .iter()
        .map(transcript_event)
        .collect::<Vec<_>>();
    map.insert("events".to_string(), Value::Array(events));
    map
}

fn transcript_event(event: &GrokMergedEvent) -> Value {
    match event {
        GrokMergedEvent::Thought {
            text,
            prompt_id,
            event_id,
            reasoning,
        } => {
            let mut value = serde_json::json!({
                "kind": "thought",
                "text": text,
                "prompt_id": prompt_id,
                "event_id": event_id,
            });
            if let Some(reasoning) = reasoning {
                value["reasoning"] = serde_json::json!({
                    "id": reasoning.id,
                    "status": reasoning.status,
                    "encrypted_content": reasoning.encrypted_content,
                });
            }
            value
        }
        GrokMergedEvent::Tool { name, event_id } => serde_json::json!({
            "kind": "tool",
            "name": name,
            "event_id": event_id,
        }),
        GrokMergedEvent::Unknown {
            session_update,
            line_sha256,
        } => serde_json::json!({
            "kind": "unknown",
            "session_update": session_update,
            "line_sha256": line_sha256,
        }),
        GrokMergedEvent::Turn {
            reasoning_tokens,
            event_id,
        } => serde_json::json!({
            "kind": "turn",
            "reasoning_tokens": reasoning_tokens,
            "event_id": event_id,
        }),
        GrokMergedEvent::Reasoning {
            id,
            status,
            summary_text,
            encrypted_content,
        } => serde_json::json!({
            "kind": "reasoning",
            "id": id,
            "status": status,
            "summary_text": summary_text,
            "encrypted_content": encrypted_content,
        }),
    }
}

fn strip_export_value(value: &mut Value) {
    match value {
        Value::Array(items) => {
            for item in items {
                strip_export_value(item);
            }
        }
        Value::Object(map) => {
            map.remove("encrypted_content");
            if let Some(metadata) = map.get_mut("metadata").and_then(Value::as_object_mut) {
                if let Some(openai) = metadata.get_mut("openai").and_then(Value::as_object_mut) {
                    openai.remove("reasoningEncryptedContent");
                }
                if let Some(anthropic) =
                    metadata.get_mut("anthropic").and_then(Value::as_object_mut)
                {
                    anthropic.remove("signature");
                }
            }
            for child in map.values_mut() {
                strip_export_value(child);
            }
        }
        _ => {}
    }
}

fn attach_index(slots: &[ThoughtSlot], summary: &str) -> Option<usize> {
    if let Some(index) = slots.iter().position(|slot| slot.text_raw == summary) {
        return Some(index);
    }
    let mut start = 0;
    while start < slots.len() {
        let prompt = &slots[start].prompt_id;
        let mut end = start + 1;
        while end < slots.len() && slots[end].prompt_id == *prompt {
            end += 1;
        }
        let run = &slots[start..end];
        for window_start in 0..run.len() {
            let mut joined = String::new();
            for slot in run[window_start..].iter().take(4) {
                let at_first = joined.is_empty();
                joined.push_str(&slot.text_raw);
                if at_first {
                    continue;
                }
                if joined == summary {
                    return Some(start + window_start);
                }
            }
        }
        start = end;
    }
    None
}

fn scan_prose(text: &str) -> (String, bool) {
    let redactor = Redactor::new_default();
    let (redacted, report) = redactor.redact(text.as_bytes());
    match String::from_utf8(redacted.into_inner()) {
        Ok(stored) => (stored, report.match_count() > 0),
        Err(_) => ("<REDACTED:invalid-utf8>".to_string(), true),
    }
}

struct ReasoningRow {
    id: String,
    status: String,
    summaries: Vec<Option<String>>,
    encrypted_content: String,
}

enum HistoryRow {
    Reasoning(ReasoningRow),
    Partial,
}

fn read_history(
    session_dir: &Path,
    chat_history: &Path,
) -> Result<Vec<HistoryRow>, GrokReasoningError> {
    let path_text = chat_history.display().to_string();
    let meta = std::fs::symlink_metadata(chat_history).map_err(|error| GrokReasoningError::Io {
        path: path_text.clone(),
        detail: error.to_string(),
    })?;
    if meta.file_type().is_symlink() {
        return Err(GrokReasoningError::Refused {
            path: path_text,
            reason: "path is a symlink".to_string(),
        });
    }
    if !meta.file_type().is_file() {
        return Err(GrokReasoningError::Refused {
            path: path_text,
            reason: "path is not a regular file".to_string(),
        });
    }
    if meta.len() > TRANSCRIPT_READ_HARD_CAP_BYTES {
        return Err(GrokReasoningError::Read(TranscriptReadError::ExceedsCap {
            cap: TRANSCRIPT_READ_HARD_CAP_BYTES,
        }));
    }
    let session_root =
        std::fs::canonicalize(session_dir).map_err(|error| GrokReasoningError::Io {
            path: session_dir.display().to_string(),
            detail: error.to_string(),
        })?;
    let canonical =
        std::fs::canonicalize(chat_history).map_err(|error| GrokReasoningError::Io {
            path: path_text.clone(),
            detail: error.to_string(),
        })?;
    if !canonical.starts_with(&session_root) {
        return Err(GrokReasoningError::Refused {
            path: path_text,
            reason: format!(
                "path is outside the session directory {}",
                session_root.display()
            ),
        });
    }
    let file = File::open(&canonical).map_err(|error| GrokReasoningError::Io {
        path: path_text.clone(),
        detail: error.to_string(),
    })?;
    let mut reader = BufReader::new(file);
    let mut rows = Vec::new();
    loop {
        match read_capped_line(&mut reader).map_err(|error| GrokReasoningError::Io {
            path: path_text.clone(),
            detail: error.to_string(),
        })? {
            Line::Eof => break,
            Line::TooLong => {
                return Err(GrokReasoningError::Malformed {
                    reason: "a chat_history line exceeds 1 MiB".to_string(),
                });
            }
            Line::Bytes(bytes) if bytes.is_empty() => {}
            Line::Bytes(bytes) => rows.push(parse_history_line(&bytes)?),
        }
    }
    Ok(rows)
}

fn parse_history_line(bytes: &[u8]) -> Result<HistoryRow, GrokReasoningError> {
    let value: Value =
        serde_json::from_slice(bytes).map_err(|_| GrokReasoningError::Malformed {
            reason: "chat_history line is not JSON".to_string(),
        })?;
    let Some(object) = value.as_object() else {
        return Err(GrokReasoningError::Malformed {
            reason: "chat_history line is not an object".to_string(),
        });
    };
    let kind = object.get("type").and_then(Value::as_str).unwrap_or("");
    if kind != "reasoning" {
        return Err(GrokReasoningError::Malformed {
            reason: format!("chat_history type '{kind}' is outside the field matrix"),
        });
    }
    let Some(id) = object.get("id").and_then(Value::as_str) else {
        return Err(GrokReasoningError::Malformed {
            reason: "reasoning id is not a string".to_string(),
        });
    };
    let Some(status) = object.get("status").and_then(Value::as_str) else {
        return Err(GrokReasoningError::Malformed {
            reason: "reasoning status is not a string".to_string(),
        });
    };
    let Some(summary) = object.get("summary").and_then(Value::as_array) else {
        return Err(GrokReasoningError::Malformed {
            reason: "reasoning summary is not a list".to_string(),
        });
    };
    let Some(encrypted_value) = object.get("encrypted_content") else {
        return Err(GrokReasoningError::Malformed {
            reason: "reasoning row has no encrypted_content".to_string(),
        });
    };
    let mut summaries = Vec::new();
    for item in summary {
        let Some(item) = item.as_object() else {
            return Err(GrokReasoningError::Malformed {
                reason: "summary item is not an object".to_string(),
            });
        };
        if item.get("type").and_then(Value::as_str) != Some("summary_text") {
            return Err(GrokReasoningError::Malformed {
                reason: "summary item type is not summary_text".to_string(),
            });
        }
        match item.get("text").and_then(Value::as_str) {
            Some(text) => summaries.push(Some(text.to_string())),
            None => summaries.push(None),
        }
    }
    let Some(encrypted) = encrypted_value.as_str() else {
        return Ok(HistoryRow::Partial);
    };
    Ok(HistoryRow::Reasoning(ReasoningRow {
        id: id.to_string(),
        status: status.to_string(),
        summaries,
        encrypted_content: encrypted.to_string(),
    }))
}

enum Line {
    Eof,
    Bytes(Vec<u8>),
    TooLong,
}

fn read_capped_line(reader: &mut impl Read) -> std::io::Result<Line> {
    let mut bytes = Vec::new();
    let mut one = [0_u8; 1];
    let mut seen = 0_u64;
    loop {
        let read = reader.read(&mut one)?;
        if read == 0 {
            if seen == 0 {
                return Ok(Line::Eof);
            }
            return Ok(Line::Bytes(bytes));
        }
        seen += 1;
        if one[0] == b'\n' {
            return Ok(Line::Bytes(bytes));
        }
        if seen > GROK_LINE_MAX_BYTES {
            return Ok(Line::TooLong);
        }
        bytes.push(one[0]);
    }
}
