//! Read a synthetic or caller-supplied Grok `updates.jsonl`.
//!
//! Each `agent_thought_chunk` string becomes its own transcript event.
//! Chunks that share a `promptId` are not joined. The caller supplies the
//! session directory; this module never opens `GROK_HOME`.

use std::{
    collections::BTreeSet,
    fs::File,
    io::{BufReader, Read, Seek, SeekFrom},
    path::{Path, PathBuf},
};

use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::internal::ai::observed_agents::transcript_source::TRANSCRIPT_READ_HARD_CAP_BYTES;

/// One JSONL line above this size is not a complete thought.
pub const GROK_LINE_MAX_BYTES: u64 = 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GrokReadCursor {
    pub offset: u64,
    pub seen_event_ids: BTreeSet<String>,
}

#[derive(Clone, PartialEq, Eq)]
pub enum GrokTranscriptEvent {
    Thought {
        text: String,
        prompt_id: String,
        event_id: String,
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
}

impl std::fmt::Debug for GrokTranscriptEvent {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Thought {
                text,
                prompt_id,
                event_id,
            } => formatter
                .debug_struct("Thought")
                .field("text_len", &text.len())
                .field("prompt_id_len", &prompt_id.len())
                .field("event_id_len", &event_id.len())
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
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GrokUpdatesRead {
    pub events: Vec<GrokTranscriptEvent>,
    pub partial: bool,
    pub next_offset: u64,
    pub seen_event_ids: BTreeSet<String>,
}

impl GrokUpdatesRead {
    /// A read is complete only when every accepted line was understood
    /// without a dropped field or an oversized line.
    pub fn is_complete(&self) -> bool {
        !self.partial
    }
}

#[derive(Debug, thiserror::Error)]
pub enum GrokUpdatesError {
    #[error("refusing to read '{path}': {reason}")]
    Refused { path: String, reason: String },
    #[error("failed to read '{path}': {detail}")]
    Io { path: String, detail: String },
}

/// Read `updates.jsonl` from `cursor.offset`.
///
/// `session_dir` is the only directory the file may live in. A symlink,
/// a non-file, a path outside that directory, or a source larger than
/// 16 MiB is refused and does not become a transcript.
pub fn read_grok_updates(
    session_dir: &Path,
    updates_path: &Path,
    cursor: &GrokReadCursor,
) -> Result<GrokUpdatesRead, GrokUpdatesError> {
    let path_text = updates_path.display().to_string();
    let file_meta =
        std::fs::symlink_metadata(updates_path).map_err(|error| GrokUpdatesError::Io {
            path: path_text.clone(),
            detail: error.to_string(),
        })?;
    if file_meta.file_type().is_symlink() {
        return Err(GrokUpdatesError::Refused {
            path: path_text,
            reason: "path is a symlink".to_string(),
        });
    }
    if !file_meta.file_type().is_file() {
        return Err(GrokUpdatesError::Refused {
            path: path_text,
            reason: "path is not a regular file".to_string(),
        });
    }
    if file_meta.len() > TRANSCRIPT_READ_HARD_CAP_BYTES {
        return Err(GrokUpdatesError::Refused {
            path: path_text,
            reason: format!(
                "source is {} bytes, above the {TRANSCRIPT_READ_HARD_CAP_BYTES} byte limit",
                file_meta.len()
            ),
        });
    }
    let session_root =
        std::fs::canonicalize(session_dir).map_err(|error| GrokUpdatesError::Io {
            path: session_dir.display().to_string(),
            detail: error.to_string(),
        })?;
    let canonical_file =
        std::fs::canonicalize(updates_path).map_err(|error| GrokUpdatesError::Io {
            path: path_text.clone(),
            detail: error.to_string(),
        })?;
    if !canonical_file.starts_with(&session_root) {
        return Err(GrokUpdatesError::Refused {
            path: path_text,
            reason: format!(
                "path is outside the session directory {}",
                session_root.display()
            ),
        });
    }

    let mut file = File::open(&canonical_file).map_err(|error| GrokUpdatesError::Io {
        path: path_text.clone(),
        detail: error.to_string(),
    })?;
    let start = cursor.offset.min(file_meta.len());
    file.seek(SeekFrom::Start(start))
        .map_err(|error| GrokUpdatesError::Io {
            path: path_text.clone(),
            detail: error.to_string(),
        })?;

    let mut reader = BufReader::new(file);
    let mut events = Vec::new();
    let mut seen = cursor.seen_event_ids.clone();
    let mut partial = false;
    let mut consumed = start;

    loop {
        match read_capped_line(&mut reader, GROK_LINE_MAX_BYTES).map_err(|error| {
            GrokUpdatesError::Io {
                path: path_text.clone(),
                detail: error.to_string(),
            }
        })? {
            CappedLine::Eof => break,
            CappedLine::TooLong { skipped } => {
                partial = true;
                consumed = consumed.saturating_add(skipped);
            }
            CappedLine::Line { bytes, len } => {
                consumed = consumed.saturating_add(len);
                if bytes.is_empty() {
                    continue;
                }
                match interpret_line(&bytes, &mut seen) {
                    LineEffect::Skip => {}
                    LineEffect::Partial(event) => {
                        partial = true;
                        if let Some(event) = event {
                            events.push(event);
                        }
                    }
                    LineEffect::Event(event) => events.push(event),
                    LineEffect::Refuse => {
                        return Err(GrokUpdatesError::Refused {
                            path: path_text,
                            reason: "line is outside the grok field matrix".to_string(),
                        });
                    }
                }
            }
        }
    }

    Ok(GrokUpdatesRead {
        events,
        partial,
        next_offset: consumed,
        seen_event_ids: seen,
    })
}

enum CappedLine {
    Eof,
    Line { bytes: Vec<u8>, len: u64 },
    TooLong { skipped: u64 },
}

fn read_capped_line(reader: &mut impl Read, max_bytes: u64) -> std::io::Result<CappedLine> {
    let mut bytes = Vec::new();
    let mut one = [0_u8; 1];
    let mut seen = 0_u64;
    loop {
        let read = reader.read(&mut one)?;
        if read == 0 {
            if seen == 0 {
                return Ok(CappedLine::Eof);
            }
            return Ok(CappedLine::Line { bytes, len: seen });
        }
        seen += 1;
        if one[0] == b'\n' {
            return Ok(CappedLine::Line { bytes, len: seen });
        }
        if seen > max_bytes {
            let mut skipped = seen;
            loop {
                let read = reader.read(&mut one)?;
                if read == 0 {
                    break;
                }
                skipped += 1;
                if one[0] == b'\n' {
                    break;
                }
            }
            return Ok(CappedLine::TooLong { skipped });
        }
        bytes.push(one[0]);
    }
}

enum LineEffect {
    Skip,
    Event(GrokTranscriptEvent),
    Partial(Option<GrokTranscriptEvent>),
    Refuse,
}

fn interpret_line(bytes: &[u8], seen: &mut BTreeSet<String>) -> LineEffect {
    let Ok(value) = serde_json::from_slice::<Value>(bytes) else {
        return LineEffect::Refuse;
    };
    let method = value.get("method").and_then(Value::as_str).unwrap_or("");
    if method != "session/update" && method != "_x.ai/session/update" {
        return LineEffect::Refuse;
    }
    let params = value.get("params");
    let event_id = params
        .and_then(|params| params.get("_meta"))
        .and_then(|meta| meta.get("eventId"))
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    if !event_id.is_empty() && !seen.insert(event_id.clone()) {
        return LineEffect::Skip;
    }
    let update = params.and_then(|params| params.get("update"));
    let session_update = update
        .and_then(|update| update.get("sessionUpdate"))
        .and_then(Value::as_str)
        .unwrap_or("");
    let missing_event_id = event_id.is_empty();
    let effect = match session_update {
        "agent_thought_chunk" => interpret_thought(update, &event_id),
        "tool_call" | "tool_call_update" => LineEffect::Event(GrokTranscriptEvent::Tool {
            name: tool_name(update),
            event_id,
        }),
        "turn_completed" => {
            let reasoning_tokens = update
                .and_then(|update| update.get("usage"))
                .and_then(|usage| usage.get("reasoningTokens"))
                .and_then(Value::as_u64)
                .unwrap_or(0);
            LineEffect::Event(GrokTranscriptEvent::Turn {
                reasoning_tokens,
                event_id,
            })
        }
        other => LineEffect::Event(GrokTranscriptEvent::Unknown {
            session_update: other.to_string(),
            line_sha256: hex_sha256(bytes),
        }),
    };
    if missing_event_id {
        return match effect {
            LineEffect::Event(event) => LineEffect::Partial(Some(event)),
            other => other,
        };
    }
    effect
}

fn interpret_thought(update: Option<&Value>, event_id: &str) -> LineEffect {
    let content = update.and_then(|update| update.get("content"));
    let content_type = content
        .and_then(|content| content.get("type"))
        .and_then(Value::as_str);
    let text = content.and_then(|content| content.get("text"));
    // A non-string `content.text`, or a content type other than `text`, is
    // outside the 1.0.62 field matrix. Drop that chunk and mark the read
    // partial. Do not refuse the rest of the file.
    let Some(text) = text.and_then(Value::as_str) else {
        return LineEffect::Partial(None);
    };
    if content_type != Some("text") {
        return LineEffect::Partial(None);
    }
    let prompt_id = update
        .and_then(|update| update.get("promptId"))
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    LineEffect::Event(GrokTranscriptEvent::Thought {
        text: text.to_string(),
        prompt_id,
        event_id: event_id.to_string(),
    })
}

fn tool_name(update: Option<&Value>) -> String {
    update
        .and_then(|update| update.get("title"))
        .and_then(Value::as_str)
        .or_else(|| {
            update
                .and_then(|update| update.get("toolCallId"))
                .and_then(Value::as_str)
        })
        .unwrap_or("tool")
        .to_string()
}

fn hex_sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

/// Test helper: the canonical file path after the jail check.
pub fn session_file(session_dir: &Path, name: &str) -> PathBuf {
    session_dir.join(name)
}
