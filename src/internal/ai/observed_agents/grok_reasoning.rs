//! Merge synthetic Grok `chat_history.jsonl` reasoning rows into a GK-02 transcript.
//!
//! Ciphertext stays in the ordinary transcript. Safe projection, metadata,
//! typed coverage, Debug, doctor JSON, the coverage digest, and log lines
//! do not receive that plaintext. This module never decrypts, never opens
//! `GROK_HOME`, and never edits OpenCode export.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs::File,
    io::Read,
    path::Path,
    time::Instant,
};

use serde_json::{Map, Value};
use sha2::{Digest, Sha256};

use crate::internal::ai::observed_agents::{
    grok_updates::{GROK_LINE_MAX_BYTES, GrokTranscriptEvent, GrokUpdatesRead},
    redaction::{Redactor, is_durable_default_rule_id},
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
    #[error(
        "Grok reasoning normalization exceeded its deadline; retry with a fresh source snapshot"
    )]
    Deadline,
    #[error("Grok reasoning normalization exceeded its {limit} byte {resource} limit")]
    Budget { resource: &'static str, limit: u64 },
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
    merge_reasoning(rows, updates, None, false)
}

/// Ordinary retained body budget for the new snapshot normalizer.
pub const GROK_THOUGHT_MAX_BYTES: u64 = 8 * 1024 * 1024;
/// Bound repeated reasoning attachments as well as their input sources.
pub const GROK_NORMALIZED_MAX_BYTES: u64 = 2 * TRANSCRIPT_READ_HARD_CAP_BYTES;

/// A pure interpretation result, without repository/session turn ownership.
///
/// Even `turn_completed` supplies usage, not a verified conversation turn.
/// GK-08 must establish ownership before a shared checkpoint writer can use
/// these events. This type never creates user/assistant messages or coverage.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GrokNormalizedTranscript {
    pub transcript: GrokReasoningImport,
    pub unattributed_event_count: usize,
}

impl GrokNormalizedTranscript {
    pub fn transcript_document(&self) -> Value {
        let mut value = self.transcript.transcript_document();
        value["unattributed_event_count"] = self.unattributed_event_count.into();
        value
    }
}

/// Normalize authorized history bytes and already interpreted update bytes.
///
/// No file, environment, process, SQL, ref or checkpoint access occurs here.
/// Each source is bounded to 16 MiB, each history line to 1 MiB, retained
/// thought/summary text to 8 MiB and aggregate output strings to 32 MiB.
/// Bounds or deadline expiry return an explicit error rather than silently
/// dropping ordinary text. Body strings are credential-scanned after JSON
/// decoding; opaque reasoning fields retain their existing uninterpreted
/// representation and are never decrypted or promoted to native evidence.
/// The old path API retains its field-completeness semantics; this new result
/// additionally marks all events partial until GK-08 supplies real ownership.
pub fn normalize_grok_reasoning_bytes(
    history: &[u8],
    updates: &GrokUpdatesRead,
    deadline: Option<Instant>,
) -> Result<GrokNormalizedTranscript, GrokReasoningError> {
    check_deadline(deadline)?;
    validate_updates(updates, deadline)?;
    let rows = parse_history_bytes(history, deadline)?;
    let mut transcript = merge_reasoning(rows, &updates.events, deadline, true)?;
    let unattributed_event_count = transcript.events.len();
    transcript.partial |= updates.partial || unattributed_event_count > 0;
    check_deadline(deadline)?;
    Ok(GrokNormalizedTranscript {
        transcript,
        unattributed_event_count,
    })
}

fn validate_updates(
    updates: &GrokUpdatesRead,
    deadline: Option<Instant>,
) -> Result<(), GrokReasoningError> {
    let mut bytes = 0_u64;
    for event in &updates.events {
        check_deadline(deadline)?;
        let fields = match event {
            GrokTranscriptEvent::Thought {
                text,
                prompt_id,
                event_id,
            } => text
                .len()
                .saturating_add(prompt_id.len())
                .saturating_add(event_id.len()),
            GrokTranscriptEvent::Tool { name, event_id } => {
                name.len().saturating_add(event_id.len())
            }
            GrokTranscriptEvent::Unknown {
                session_update,
                line_sha256,
            } => session_update.len().saturating_add(line_sha256.len()),
            GrokTranscriptEvent::Turn { event_id, .. } => event_id.len(),
        };
        if fields as u64 > GROK_LINE_MAX_BYTES {
            return Err(GrokReasoningError::Budget {
                resource: "typed update",
                limit: GROK_LINE_MAX_BYTES,
            });
        }
        bytes = bytes.saturating_add(fields as u64).saturating_add(64);
        if bytes > TRANSCRIPT_READ_HARD_CAP_BYTES {
            return Err(GrokReasoningError::Budget {
                resource: "typed updates",
                limit: TRANSCRIPT_READ_HARD_CAP_BYTES,
            });
        }
    }
    Ok(())
}

struct RetainedBudget {
    enabled: bool,
    body: u64,
    total: u64,
}

impl RetainedBudget {
    fn charge(&mut self, body: usize, fields: usize) -> Result<(), GrokReasoningError> {
        self.body = self.body.saturating_add(body as u64);
        self.total = self.total.saturating_add(fields as u64).saturating_add(64);
        if self.enabled && self.body > GROK_THOUGHT_MAX_BYTES {
            return Err(GrokReasoningError::Budget {
                resource: "retained body",
                limit: GROK_THOUGHT_MAX_BYTES,
            });
        }
        if self.enabled && self.total > GROK_NORMALIZED_MAX_BYTES {
            return Err(GrokReasoningError::Budget {
                resource: "normalized output",
                limit: GROK_NORMALIZED_MAX_BYTES,
            });
        }
        Ok(())
    }
}

fn merge_reasoning(
    rows: Vec<HistoryRow>,
    updates: &[GrokTranscriptEvent],
    deadline: Option<Instant>,
    bounded: bool,
) -> Result<GrokReasoningImport, GrokReasoningError> {
    let mut budget = RetainedBudget {
        enabled: bounded,
        body: 0,
        total: 0,
    };
    let mut slots = Vec::new();
    for event in updates {
        check_deadline(deadline)?;
        match event {
            GrokTranscriptEvent::Thought {
                text,
                prompt_id,
                event_id,
            } => {
                let (stored, _) = scan_prose(text);
                check_deadline(deadline)?;
                budget.charge(
                    stored.len(),
                    stored
                        .len()
                        .saturating_add(prompt_id.len())
                        .saturating_add(event_id.len()),
                )?;
                slots.push(ThoughtSlot {
                    text_raw: text.clone(),
                    text_stored: stored,
                    prompt_id: prompt_id.clone(),
                    event_id: event_id.clone(),
                    reasoning: None,
                });
            }
            GrokTranscriptEvent::Tool { name, event_id } => {
                budget.charge(0, name.len().saturating_add(event_id.len()))?
            }
            GrokTranscriptEvent::Unknown {
                session_update,
                line_sha256,
            } => budget.charge(0, session_update.len().saturating_add(line_sha256.len()))?,
            GrokTranscriptEvent::Turn { event_id, .. } => budget.charge(0, event_id.len())?,
        }
    }
    let index = AttachIndex::build(&slots, deadline)?;
    let mut partial = false;
    let mut standalone = Vec::new();
    for row in rows {
        check_deadline(deadline)?;
        match row {
            HistoryRow::Partial => partial = true,
            HistoryRow::Reasoning(row) => {
                if row.summaries.is_empty() {
                    budget.charge(0, row.field_bytes())?;
                    standalone.push(ciphertext_only(row));
                    continue;
                }
                let mut kept_text = false;
                for summary in &row.summaries {
                    check_deadline(deadline)?;
                    let Some(summary_raw) = summary else {
                        partial = true;
                        continue;
                    };
                    kept_text = true;
                    push_summary(
                        &mut slots,
                        &index,
                        &mut standalone,
                        &row,
                        summary_raw,
                        &mut budget,
                        deadline,
                    )?;
                }
                if !kept_text {
                    budget.charge(0, row.field_bytes())?;
                    standalone.push(ciphertext_only(row));
                }
            }
        }
    }
    let mut events = Vec::new();
    let mut slot_iter = slots.into_iter();
    for event in updates {
        check_deadline(deadline)?;
        match event {
            GrokTranscriptEvent::Thought { .. } => {
                let slot = slot_iter
                    .next()
                    .ok_or_else(|| GrokReasoningError::Malformed {
                        reason: "thought mapping is inconsistent".to_string(),
                    })?;
                events.push(GrokMergedEvent::Thought {
                    text: slot.text_stored,
                    prompt_id: slot.prompt_id,
                    event_id: slot.event_id,
                    reasoning: slot.reasoning,
                });
            }
            GrokTranscriptEvent::Tool { name, event_id } => events.push(GrokMergedEvent::Tool {
                name: name.clone(),
                event_id: event_id.clone(),
            }),
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
    check_deadline(deadline)?;
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
    index: &AttachIndex,
    standalone: &mut Vec<GrokMergedEvent>,
    row: &ReasoningRow,
    summary_raw: &str,
    budget: &mut RetainedBudget,
    deadline: Option<Instant>,
) -> Result<(), GrokReasoningError> {
    if let Some(index) = index.find(slots, summary_raw, deadline)?
        && slots[index].reasoning.is_none()
    {
        budget.charge(0, row.field_bytes())?;
        slots[index].reasoning = Some(GrokAttachedReasoning {
            id: row.id.clone(),
            status: row.status.clone(),
            encrypted_content: row.encrypted_content.clone(),
        });
        return Ok(());
    }
    let (stored, _) = scan_prose(summary_raw);
    check_deadline(deadline)?;
    budget.charge(stored.len(), stored.len().saturating_add(row.field_bytes()))?;
    standalone.push(GrokMergedEvent::Reasoning {
        id: row.id.clone(),
        status: row.status.clone(),
        summary_text: Some(stored),
        encrypted_content: row.encrypted_content.clone(),
    });
    Ok(())
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
    let mut redacted_samples = BTreeSet::new();
    for event in &import.events {
        match event {
            GrokMergedEvent::Thought { text, .. } => {
                if text.contains("<REDACTED:") {
                    redacted_samples.extend(redaction_markers(text));
                }
            }
            GrokMergedEvent::Reasoning {
                summary_text: Some(text),
                ..
            } if text.contains("<REDACTED:") => redacted_samples.extend(redaction_markers(text)),
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

fn redaction_markers(text: &str) -> BTreeSet<String> {
    let mut markers = BTreeSet::new();
    let mut rest = text;
    while let Some(at) = rest.find("<REDACTED:") {
        rest = &rest[at..];
        let Some(end) = rest.find('>') else {
            break;
        };
        if is_durable_default_rule_id(&rest[10..end]) {
            markers.insert(rest[..=end].to_string());
        }
        rest = &rest[end + 1..];
    }
    markers
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

// Hash indexes avoid rescanning every thought for every history summary.
// Candidate bytes are still checked, so a digest collision cannot misattach.
type AttachKey = (usize, [u8; 32]);
struct AttachIndex {
    exact: BTreeMap<AttachKey, Vec<usize>>,
    windows: BTreeMap<AttachKey, Vec<(usize, usize)>>,
}

impl AttachIndex {
    fn build(slots: &[ThoughtSlot], deadline: Option<Instant>) -> Result<Self, GrokReasoningError> {
        let mut index = Self {
            exact: BTreeMap::new(),
            windows: BTreeMap::new(),
        };
        for (start, first) in slots.iter().enumerate() {
            check_deadline(deadline)?;
            let mut hash = Sha256::new();
            let mut len = 0_usize;
            for (count, slot) in slots[start..].iter().take(4).enumerate() {
                if slot.prompt_id != first.prompt_id {
                    break;
                }
                for chunk in slot.text_raw.as_bytes().chunks(4096) {
                    check_deadline(deadline)?;
                    hash.update(chunk);
                }
                len = len.saturating_add(slot.text_raw.len());
                let key = (len, hash.clone().finalize().into());
                if count == 0 {
                    index.exact.entry(key).or_default().push(start);
                } else {
                    index
                        .windows
                        .entry(key)
                        .or_default()
                        .push((start, count + 1));
                }
            }
        }
        check_deadline(deadline)?;
        Ok(index)
    }

    fn find(
        &self,
        slots: &[ThoughtSlot],
        summary: &str,
        deadline: Option<Instant>,
    ) -> Result<Option<usize>, GrokReasoningError> {
        let mut hash = Sha256::new();
        for chunk in summary.as_bytes().chunks(4096) {
            check_deadline(deadline)?;
            hash.update(chunk);
        }
        let key = (summary.len(), hash.finalize().into());
        if let Some(candidates) = self.exact.get(&key) {
            for &at in candidates {
                check_deadline(deadline)?;
                if slots[at].text_raw == summary {
                    return Ok(Some(at));
                }
            }
        }
        if let Some(candidates) = self.windows.get(&key) {
            for &(at, count) in candidates {
                check_deadline(deadline)?;
                let mut rest = summary.as_bytes();
                let mut matches = true;
                for slot in &slots[at..at + count] {
                    if let Some(suffix) = rest.strip_prefix(slot.text_raw.as_bytes()) {
                        rest = suffix;
                    } else {
                        matches = false;
                        break;
                    }
                }
                if matches && rest.is_empty() {
                    return Ok(Some(at));
                }
            }
        }
        check_deadline(deadline)?;
        Ok(None)
    }
}

fn check_deadline(deadline: Option<Instant>) -> Result<(), GrokReasoningError> {
    if deadline.is_some_and(|limit| Instant::now() >= limit) {
        return Err(GrokReasoningError::Deadline);
    }
    Ok(())
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

impl ReasoningRow {
    fn field_bytes(&self) -> usize {
        self.id
            .len()
            .saturating_add(self.status.len())
            .saturating_add(self.encrypted_content.len())
    }
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
    let mut bytes = Vec::new();
    file.take(TRANSCRIPT_READ_HARD_CAP_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| GrokReasoningError::Io {
            path: path_text,
            detail: error.to_string(),
        })?;
    parse_history_bytes(&bytes, None)
}

fn parse_history_bytes(
    bytes: &[u8],
    deadline: Option<Instant>,
) -> Result<Vec<HistoryRow>, GrokReasoningError> {
    check_deadline(deadline)?;
    if bytes.len() as u64 > TRANSCRIPT_READ_HARD_CAP_BYTES {
        return Err(GrokReasoningError::Read(TranscriptReadError::ExceedsCap {
            cap: TRANSCRIPT_READ_HARD_CAP_BYTES,
        }));
    }
    let mut rows = Vec::new();
    for line in bytes.split_inclusive(|byte| *byte == b'\n') {
        check_deadline(deadline)?;
        let body = line.strip_suffix(b"\n").unwrap_or(line);
        if body.len() as u64 > GROK_LINE_MAX_BYTES {
            return Err(GrokReasoningError::Malformed {
                reason: "a chat_history line exceeds 1 MiB".to_string(),
            });
        }
        if !body.is_empty() {
            rows.push(parse_history_line(body, deadline)?);
        }
    }
    check_deadline(deadline)?;
    Ok(rows)
}

fn parse_history_line(
    bytes: &[u8],
    deadline: Option<Instant>,
) -> Result<HistoryRow, GrokReasoningError> {
    check_deadline(deadline)?;
    let value: Value =
        serde_json::from_slice(bytes).map_err(|_| GrokReasoningError::Malformed {
            reason: "chat_history line is not JSON".to_string(),
        })?;
    check_deadline(deadline)?;
    let Some(object) = value.as_object() else {
        return Err(GrokReasoningError::Malformed {
            reason: "chat_history line is not an object".to_string(),
        });
    };
    let kind = object.get("type").and_then(Value::as_str).unwrap_or("");
    if kind != "reasoning" {
        return Err(GrokReasoningError::Malformed {
            reason: "chat_history type is outside the field matrix".to_string(),
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
        check_deadline(deadline)?;
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
