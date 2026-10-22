//! Codex 0.162.1/0.163.0 reasoning fields at the ordinary source boundary.
//!
//! This classifier grants no native authority. Associated JSON values remain
//! ordinary transcript data; only decoded strings undergo credential scanning.
//! Number tokens are copied unchanged so unknown data is never rounded.

use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
    time::Instant,
};

use serde::Serialize;
use zeroize::Zeroizing;

use super::{
    RedactionReport, Redactor,
    coverage::{CanonValue, SemanticRecord, merge_redaction_reports},
    reasoning::{
        ReasoningFieldError, ReasoningProvider, ReasoningSourceKind,
        classify_provider_visible_reasoning,
    },
};

const MAX_LINE_BYTES: usize = 1024 * 1024;
const MAX_FIELDS: usize = 4096;

/// An ordinary decoded JSON string associated with a pinned Codex rollout.
/// The original, credential-scanned JSONL remains the durable carrier. This
/// view neither authenticates nor decrypts `encrypted_content`.
#[derive(Clone, PartialEq, Eq)]
pub struct CodexAttachedField {
    pub session_id: String,
    pub turn_key: Option<String>,
    pub record_id: Option<String>,
    pub line_ordinal: u32,
    pub record_ordinal: u32,
    pub history_index: Option<u32>,
    pub value: String,
    pub availability: &'static str,
}

#[derive(Default, Clone, PartialEq, Eq)]
pub struct CodexAttachedProjection {
    pub fields: Vec<CodexAttachedField>,
    pub readable: Vec<String>,
    pub duplicate_rejected: u32,
    pub unsupported: u32,
    pub refused: bool,
}

impl CodexAttachedProjection {
    pub fn data_obtained(&self) -> bool {
        self.readable.iter().any(|text| !text.is_empty())
            || self.fields.iter().any(|field| !field.value.is_empty())
    }

    pub fn readable_obtained(&self) -> bool {
        self.readable.iter().any(|text| !text.is_empty())
    }
}

/// Project only a rollout whose filename, header session and supported writer
/// version all agree. The caller must first authorize this source for the
/// current repository and pass its credential-scanned ordinary bytes.
pub fn authorize_codex_attached_fields(
    file_name: &str,
    session_id: &str,
    source: &[u8],
) -> CodexAttachedProjection {
    let leaf = Path::new(file_name)
        .file_name()
        .and_then(|leaf| leaf.to_str());
    let valid_leaf = !session_id.is_empty()
        && session_id.len() <= 128
        && session_id
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() || byte == b'-')
        && leaf.is_some_and(|leaf| {
            leaf.starts_with("rollout-") && leaf.ends_with(&format!("-{session_id}.jsonl"))
        });
    if !valid_leaf {
        return CodexAttachedProjection {
            refused: true,
            ..CodexAttachedProjection::default()
        };
    }
    project_catalog_bound_codex_attached_fields(session_id, source)
}

/// For a transcript already loaded through a repository-scoped committed
/// checkpoint: its catalog provider session supplies the independent binding,
/// while the rollout header is still checked from the original bytes.
pub(crate) fn project_catalog_bound_codex_attached_fields(
    session_id: &str,
    source: &[u8],
) -> CodexAttachedProjection {
    let mut result = CodexAttachedProjection::default();
    if session_id.is_empty()
        || session_id.len() > 128
        || !session_id
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() || byte == b'-')
        || source.len() as u64 > super::TRANSCRIPT_READ_HARD_CAP_BYTES
    {
        result.refused = true;
        return result;
    }
    let mut header_valid = false;
    let mut current_turn: Option<String> = None;
    let mut seen = BTreeSet::new();
    for (line_index, line) in source.split(|byte| *byte == b'\n').enumerate() {
        if line.iter().all(u8::is_ascii_whitespace) {
            continue;
        }
        if line.len() > MAX_LINE_BYTES || line_index > u32::MAX as usize {
            if !header_valid {
                result.refused = true;
                return result;
            }
            result.unsupported = result.unsupported.saturating_add(1);
            current_turn = None;
            continue;
        }
        let entry = match super::coverage::parse_canon_value(line) {
            Ok(value) => value,
            Err(error) => {
                if !header_valid {
                    result.refused = true;
                    return result;
                }
                if error.to_string().contains("duplicate") {
                    result.duplicate_rejected = result.duplicate_rejected.saturating_add(1);
                } else {
                    result.unsupported = result.unsupported.saturating_add(1);
                }
                current_turn = None;
                continue;
            }
        };
        let tag = entry.get("type").and_then(CanonValue::as_str);
        let payload = entry.get("payload");
        if !header_valid {
            header_valid = tag == Some("session_meta")
                && payload
                    .and_then(|value| value.get("id"))
                    .and_then(CanonValue::as_str)
                    == Some(session_id)
                && payload
                    .and_then(|value| value.get("cli_version"))
                    .and_then(CanonValue::as_str)
                    .is_some_and(super::coverage::supported_codex_cli_version);
            if !header_valid {
                result.refused = true;
                return result;
            }
            continue;
        }
        if tag == Some("session_meta") {
            let same_session = payload
                .and_then(|value| value.get("id"))
                .and_then(CanonValue::as_str)
                == Some(session_id);
            let supported_version = payload
                .and_then(|value| value.get("cli_version"))
                .and_then(CanonValue::as_str)
                .is_some_and(super::coverage::supported_codex_cli_version);
            if !same_session || !supported_version {
                return CodexAttachedProjection {
                    refused: true,
                    ..CodexAttachedProjection::default()
                };
            }
            current_turn = None;
            continue;
        }
        let Some(payload) = payload else { continue };
        if payload
            .get("thread_id")
            .is_some_and(|thread| thread.as_str() != Some(session_id))
        {
            result.unsupported = result.unsupported.saturating_add(1);
            continue;
        }
        let event = payload.get("type").and_then(CanonValue::as_str);
        if tag == Some("turn_context")
            || (tag == Some("event_msg") && matches!(event, Some("task_started" | "turn_started")))
        {
            current_turn = payload
                .get("turn_id")
                .and_then(CanonValue::as_str)
                .filter(|key| super::coverage::valid_provider_turn_id(key))
                .map(str::to_owned);
        }
        if tag == Some("event_msg")
            && matches!(
                event,
                Some("task_complete" | "turn_complete" | "turn_aborted")
            )
        {
            let closing = payload.get("turn_id").and_then(CanonValue::as_str);
            if closing.is_none() || closing == current_turn.as_deref() {
                current_turn = None;
            }
        }
        if tag == Some("response_item") {
            if let Some(turn_key) = current_turn.as_deref() {
                absorb_attached_record(
                    &mut result,
                    &mut seen,
                    AttachedCoordinates {
                        session_id,
                        turn_key: Some(turn_key),
                        line_ordinal: line_index as u32,
                        record_ordinal: 0,
                        history_index: None,
                    },
                    payload,
                );
            } else if payload.get("type").and_then(CanonValue::as_str) == Some("reasoning") {
                result.unsupported = result.unsupported.saturating_add(1);
            }
        } else if tag == Some("compacted") {
            if let Some(CanonValue::Array(records)) = payload.get("replacement_history") {
                for (index, record) in records.iter().take(MAX_FIELDS).enumerate() {
                    absorb_attached_record(
                        &mut result,
                        &mut seen,
                        AttachedCoordinates {
                            session_id,
                            turn_key: None,
                            line_ordinal: line_index as u32,
                            record_ordinal: index as u32,
                            history_index: Some(index as u32),
                        },
                        record,
                    );
                }
                if records.len() > MAX_FIELDS {
                    result.unsupported = result.unsupported.saturating_add(1);
                }
            } else if payload.get("replacement_history").is_some() {
                result.unsupported = result.unsupported.saturating_add(1);
            }
        }
    }
    if !header_valid {
        result.refused = true;
    }
    result
}

/// Legacy committed checkpoints can carry a normalized turn followed by
/// ordinary source reasoning lines. The catalog/ref binding is established
/// by the caller; this strict shape check prevents a bare legacy key from
/// fabricating obtained data or native authority.
pub(crate) fn catalog_bound_stored_thinking_obtained(source: &[u8]) -> bool {
    if source.is_empty() || source.len() as u64 > super::TRANSCRIPT_READ_HARD_CAP_BYTES {
        return false;
    }
    let mut lines = source
        .split(|byte| *byte == b'\n')
        .filter(|line| !line.iter().all(u8::is_ascii_whitespace));
    let Some(first) = lines.next() else {
        return false;
    };
    if first.len() > MAX_LINE_BYTES {
        return false;
    }
    let Ok(header) = super::coverage::parse_canon_value(first) else {
        return false;
    };
    if header.get("agent_kind").and_then(CanonValue::as_str) != Some("codex")
        || !header
            .get("logical_turn_key")
            .and_then(CanonValue::as_str)
            .is_some_and(super::coverage::valid_provider_turn_id)
        || !matches!(header.get("schema_version"), Some(CanonValue::Int(1 | 2)))
        || !matches!(header.get("records"), Some(CanonValue::Array(_)))
    {
        return false;
    }
    for line in lines {
        if line.len() > MAX_LINE_BYTES {
            continue;
        }
        let Ok(entry) = super::coverage::parse_canon_value(line) else {
            continue;
        };
        if entry.get("type").and_then(CanonValue::as_str) != Some("response_item") {
            continue;
        }
        let Some(payload) = entry.get("payload") else {
            continue;
        };
        let Some(classified) = classify_payload(payload) else {
            continue;
        };
        if !classified.records.is_empty()
            || matches!(payload.get("encrypted_content"), Some(CanonValue::Str(value)) if !value.is_empty())
        {
            return true;
        }
    }
    false
}

struct AttachedCoordinates<'a> {
    session_id: &'a str,
    turn_key: Option<&'a str>,
    line_ordinal: u32,
    record_ordinal: u32,
    history_index: Option<u32>,
}

fn absorb_attached_record(
    result: &mut CodexAttachedProjection,
    seen: &mut BTreeSet<String>,
    coordinates: AttachedCoordinates<'_>,
    payload: &CanonValue,
) {
    if payload.get("type").and_then(CanonValue::as_str) != Some("reasoning") {
        return;
    }
    let record_id = payload
        .get("id")
        .and_then(CanonValue::as_str)
        .filter(|id| !id.is_empty() && id.len() <= 1024);
    if coordinates.history_index.is_some() && record_id.is_none() {
        result.unsupported = result.unsupported.saturating_add(1);
        return;
    }
    if let Some(id) = record_id
        && (seen.len() >= MAX_FIELDS || !seen.insert(id.to_owned()))
    {
        result.unsupported = result.unsupported.saturating_add(1);
        return;
    }
    if let Some(CanonValue::Str(value)) = payload.get("encrypted_content") {
        if !value.is_empty() {
            if result.fields.len() >= MAX_FIELDS {
                result.unsupported = result.unsupported.saturating_add(1);
            } else {
                result.fields.push(CodexAttachedField {
                    session_id: coordinates.session_id.to_owned(),
                    turn_key: coordinates.turn_key.map(str::to_owned),
                    record_id: record_id.map(str::to_owned),
                    line_ordinal: coordinates.line_ordinal,
                    record_ordinal: coordinates.record_ordinal,
                    history_index: coordinates.history_index,
                    value: value.clone(),
                    availability: "unproven",
                });
            }
        }
    } else if payload
        .get("encrypted_content")
        .is_some_and(|value| !matches!(value, CanonValue::Null))
    {
        result.unsupported = result.unsupported.saturating_add(1);
    }
    if let Some(classified) = classify_payload(payload) {
        result.unsupported = result.unsupported.saturating_add(
            u32::try_from(classified.counts.unsupported_records).unwrap_or(u32::MAX),
        );
        for record in classified.records {
            if let SemanticRecord::Reasoning { text, .. } = record {
                if result.readable.len() >= MAX_FIELDS {
                    result.unsupported = result.unsupported.saturating_add(1);
                } else {
                    result.readable.push(text.as_str().to_owned());
                }
            }
        }
    }
}

/// Closed helper data, never a native/source capability or durable metadata.
/// Each already-redacted source line belongs to one attributed turn only.
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CodexRetentionDto {
    version: u8,
    turns: BTreeMap<String, String>,
    #[serde(default)]
    unavailable: bool,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "optional_summary"
    )]
    summary: Option<super::session_summary::SessionSummaryFacts>,
}

fn optional_summary<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<super::session_summary::SessionSummaryFacts>, D::Error> {
    let value = <Option<serde_json::Value> as serde::Deserialize>::deserialize(deserializer)?;
    Ok(value.and_then(|value| serde_json::from_value(value).ok()))
}

/// Parent-owned retention after bounded, idempotent credential validation.
pub(crate) struct CodexRetention {
    turns: BTreeMap<String, super::RedactedBytes>,
    unavailable: bool,
    summary: Option<super::session_summary::SessionSummaryFacts>,
}

pub(crate) fn unavailable_statistics() -> serde_json::Value {
    serde_json::json!({"schema_version":1,"source_status":"unproven","turn_ordinal_namespace":"codex_explicit_turn","partial":true,"statistics_unavailable":true})
}

impl CodexRetentionDto {
    pub(crate) fn unavailable() -> Self {
        Self {
            version: 1,
            turns: BTreeMap::new(),
            unavailable: true,
            summary: None,
        }
    }

    pub(crate) fn encode(
        turns: &[super::NormalizedTurn],
        mut slices: BTreeMap<usize, Vec<u8>>,
    ) -> Result<Self, ReasoningFieldError> {
        let mut retained = BTreeMap::<String, String>::new();
        for turn in turns {
            if let Some(bytes) = slices.remove(&turn.ordinal) {
                let text =
                    String::from_utf8(bytes).map_err(|_| ReasoningFieldError::MalformedSource)?;
                // An ambiguous legacy turn key cannot erase earlier source
                // values. Keep each slice once; the claim gate still owns
                // whether those semantic turns are admissible.
                retained
                    .entry(turn.logical_turn_key.clone())
                    .or_default()
                    .push_str(&text);
            }
        }
        if !slices.is_empty() {
            return Err(ReasoningFieldError::UnsupportedShape);
        }
        Ok(Self {
            version: 1,
            turns: retained,
            unavailable: false,
            summary: None,
        })
    }

    pub(crate) fn decode(
        self,
        turns: &[super::NormalizedTurn],
        source_bytes: usize,
        deadline: Instant,
    ) -> Result<CodexRetention, ReasoningFieldError> {
        if self.version != 1 {
            return Err(ReasoningFieldError::UnsupportedShape);
        }
        let summary = self
            .summary
            .and_then(super::session_summary::SessionSummaryFacts::checked);
        if self.unavailable {
            if !self.turns.is_empty() {
                return Err(ReasoningFieldError::UnsupportedShape);
            }
            return Ok(CodexRetention {
                turns: BTreeMap::new(),
                unavailable: true,
                summary,
            });
        }
        let keys = turns
            .iter()
            .map(|turn| turn.logical_turn_key.as_str())
            .collect::<std::collections::BTreeSet<_>>();
        // Each source line is retained once, with at most one added final
        // newline. Session length is bounded by bytes, not a new turn limit.
        let mut remaining = source_bytes.saturating_add(1);
        let mut retained = BTreeMap::new();
        for (key, text) in self.turns {
            if !keys.contains(key.as_str()) || text.is_empty() {
                return Err(ReasoningFieldError::UnsupportedShape);
            }
            remaining = remaining
                .checked_sub(text.len())
                .ok_or(ReasoningFieldError::SourceTooLarge)?;
            let (decoded, report) = prepare_ordinary(text.as_bytes(), Some(deadline))?;
            let (redacted, raw_report) = Redactor::new_default().redact(&decoded);
            if report.match_count() != 0
                || raw_report.match_count() != 0
                || redacted.bytes() != text.as_bytes()
            {
                return Err(ReasoningFieldError::UnsupportedShape);
            }
            retained.insert(key, redacted);
        }
        Ok(CodexRetention {
            turns: retained,
            unavailable: false,
            summary,
        })
    }

    pub(crate) fn with_summary(
        mut self,
        summary: Option<super::session_summary::SessionSummaryFacts>,
    ) -> Self {
        self.summary = summary;
        self
    }
}

impl CodexRetention {
    pub(crate) fn unavailable() -> Self {
        Self {
            turns: BTreeMap::new(),
            unavailable: true,
            summary: None,
        }
    }

    pub(crate) fn is_unavailable(&self) -> bool {
        self.unavailable
    }

    pub(crate) fn for_turn(&self, key: &str) -> Option<&super::RedactedBytes> {
        self.turns.get(key)
    }

    pub(crate) fn summary_metadata(&self) -> Option<serde_json::Value> {
        self.summary
            .as_ref()
            .map(super::session_summary::SessionSummaryFacts::extraction_metadata)
    }
}

/// A data-only preparation failure cannot discard the older V1 projection.
/// Keep an explicit, body-free diagnostic for both callers and operators.
pub(crate) fn mark_retention_unavailable(report: &mut serde_json::Value) {
    if !report.is_object() {
        *report = serde_json::json!({"partial":true,"redaction_report_invalid":true});
    }
    if !report
        .get("codex_reasoning")
        .is_some_and(serde_json::Value::is_object)
    {
        report["codex_reasoning"] = unavailable_statistics();
    }
    report["codex_reasoning"]["retention_unavailable"] = serde_json::json!(true);
    report["codex_reasoning"]["partial"] = serde_json::json!(true);
}

/// Counts only: neither provider text, opaque values nor their hashes appear.
#[derive(Default, Serialize)]
pub(crate) struct Counts {
    readable_parts: usize,
    readable_utf8_bytes: usize,
    empty_parts: usize,
    opaque_nonempty: usize,
    encrypted_empty: usize,
    encrypted_null: usize,
    encrypted_missing: usize,
    unproven_fields: usize,
    unsupported_records: usize,
}

pub(crate) struct ClassifiedReasoning {
    pub(crate) records: Vec<SemanticRecord>,
    pub(crate) counts: Counts,
}

impl ClassifiedReasoning {
    pub(crate) fn unsupported(&self) -> bool {
        self.counts.unsupported_records != 0
    }
}

fn nonempty(value: &CanonValue) -> bool {
    match value {
        CanonValue::Null => false,
        CanonValue::Str(text) => !text.is_empty(),
        CanonValue::Array(values) => !values.is_empty(),
        CanonValue::Object(values) => !values.is_empty(),
        _ => true,
    }
}

/// A single source-pinned response_item payload; no file, token or turn reader.
pub(crate) fn classify_payload(payload: &CanonValue) -> Option<ClassifiedReasoning> {
    if payload.get("type").and_then(CanonValue::as_str) != Some("reasoning") {
        return None;
    }
    let mut classified = ClassifiedReasoning {
        records: Vec::new(),
        counts: Counts::default(),
    };
    for (field, tag, kind) in [
        ("summary", "summary_text", ReasoningSourceKind::Summary),
        ("content", "reasoning_text", ReasoningSourceKind::Reasoning),
    ] {
        match payload.get(field) {
            Some(CanonValue::Array(parts)) => {
                if parts.len() > MAX_FIELDS {
                    classified.counts.unsupported_records += 1;
                    continue;
                }
                for part in parts {
                    if let CanonValue::Object(fields) = part {
                        classified.counts.unproven_fields += fields
                            .iter()
                            .filter(|(key, value)| {
                                !matches!(key.as_str(), "type" | "text") && nonempty(value)
                            })
                            .count();
                    }
                    let part_type = part.get("type").and_then(CanonValue::as_str);
                    let known =
                        part_type == Some(tag) || (field == "content" && part_type == Some("text"));
                    if let (true, Some(text)) =
                        (known, part.get("text").and_then(CanonValue::as_str))
                    {
                        if !text.trim().is_empty() {
                            classified.counts.readable_parts += 1;
                            classified.counts.readable_utf8_bytes += text.len();
                            let (_, text) = classify_provider_visible_reasoning(
                                ReasoningProvider::Codex,
                                kind,
                                text.to_owned(),
                            );
                            classified.records.push(SemanticRecord::Reasoning {
                                provider: ReasoningProvider::Codex,
                                source_kind: Some(kind),
                                text,
                            });
                        } else {
                            classified.counts.empty_parts += 1;
                        }
                    } else {
                        classified.counts.unsupported_records += 1;
                    }
                }
            }
            None | Some(CanonValue::Null) if field == "content" => {}
            _ => classified.counts.unsupported_records += 1,
        }
    }
    match payload.get("encrypted_content") {
        None => classified.counts.encrypted_missing += 1,
        Some(CanonValue::Null) => classified.counts.encrypted_null += 1,
        Some(CanonValue::Str(text)) if text.is_empty() => classified.counts.encrypted_empty += 1,
        Some(CanonValue::Str(_)) => classified.counts.opaque_nonempty += 1,
        Some(value) => {
            classified.counts.unsupported_records += 1;
            classified.counts.unproven_fields += usize::from(nonempty(value));
        }
    }
    if let CanonValue::Object(fields) = payload {
        for (key, value) in fields {
            if !matches!(
                key.as_str(),
                "type" | "id" | "summary" | "content" | "encrypted_content"
            ) && nonempty(value)
            {
                classified.counts.unproven_fields += 1;
            }
        }
    }
    Some(classified)
}

/// Scan decoded JSON string tokens without reserializing numbers or collapsing
/// duplicate fields. Ambiguous envelopes are retained for strict readers to
/// mark partial; they still undergo the same credential scan. This also covers
/// complete strings in a truncated final record.
pub(crate) fn prepare_ordinary(
    source: &[u8],
    deadline: Option<Instant>,
) -> Result<(Vec<u8>, RedactionReport), ReasoningFieldError> {
    prepare_ordinary_bounded(
        source,
        deadline,
        super::TRANSCRIPT_READ_HARD_CAP_BYTES,
        false,
    )
    .map(|(bytes, report, _partial)| (bytes, report))
}

/// Stored ordinary transcripts retain their configured reader budget. This
/// shares the capture scanner, without imposing its smaller admission cap on
/// checkpoints that were already stored by an older release.
pub(crate) fn prepare_ordinary_for_read(
    source: &[u8],
    deadline: Option<Instant>,
    read_cap: u64,
) -> Result<(Vec<u8>, RedactionReport, bool), ReasoningFieldError> {
    prepare_ordinary_bounded(source, deadline, read_cap, true)
}

// Serde's bytes visitor decodes JSON escapes into raw/WTF-8 bytes, including
// complete legacy strings that String rejects. The lossy text is only a scan
// view: unchanged tokens are copied from the original source, never from it.
struct LegacyScanText(Zeroizing<String>);

impl<'de> serde::Deserialize<'de> for LegacyScanText {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct BytesVisitor;
        impl<'de> serde::de::Visitor<'de> for BytesVisitor {
            type Value = LegacyScanText;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("a JSON string decoded as bytes for credential scanning")
            }

            fn visit_bytes<E: serde::de::Error>(self, bytes: &[u8]) -> Result<Self::Value, E> {
                Ok(LegacyScanText(Zeroizing::new(
                    String::from_utf8_lossy(bytes).into_owned(),
                )))
            }
        }
        deserializer.deserialize_bytes(BytesVisitor)
    }
}

fn prepare_ordinary_bounded(
    source: &[u8],
    deadline: Option<Instant>,
    read_cap: u64,
    preserve_complete_legacy_tokens: bool,
) -> Result<(Vec<u8>, RedactionReport, bool), ReasoningFieldError> {
    if read_cap == 0 || source.len() as u64 > read_cap {
        return Err(ReasoningFieldError::SourceTooLarge);
    }
    let redactor = Redactor::new_default();
    let mut report = RedactionReport::default();
    let mut source_partial = false;
    let mut out = Zeroizing::new(Vec::with_capacity(source.len()));
    let check_deadline = || {
        if deadline.is_some_and(|d| Instant::now() >= d) {
            Err(ReasoningFieldError::UnsupportedShape)
        } else {
            Ok(())
        }
    };
    check_deadline()?;
    for line in source.split_inclusive(|b| *b == b'\n') {
        check_deadline()?;
        let body = line.strip_suffix(b"\n").unwrap_or(line);
        let mut cursor = 0;
        let mut copied_until = 0;
        let mut next_deadline_check = 4096;
        while cursor < body.len() {
            if cursor >= next_deadline_check {
                check_deadline()?;
                next_deadline_check = cursor.saturating_add(4096);
            }
            if body[cursor] != b'"' {
                cursor += 1;
                continue;
            }
            let start = cursor;
            cursor += 1;
            while cursor < body.len() {
                if cursor >= next_deadline_check {
                    check_deadline()?;
                    next_deadline_check = cursor.saturating_add(4096);
                }
                match body[cursor] {
                    b'\\' => cursor = (cursor + 2).min(body.len()),
                    b'"' => break,
                    _ => cursor += 1,
                }
            }
            let complete = cursor < body.len();
            let end = if complete { cursor + 1 } else { body.len() };
            // Temporarily terminate an otherwise valid final string solely for
            // scanning. Its missing closing quote stays missing in the output.
            // A broken escape/UTF-8 value is unreadable. Redact that token
            // only, keeping the record invalid and retaining all other source
            // evidence. It cannot erase the rest of an authorized transcript.
            let mut token = Zeroizing::new(body[start..end].to_vec());
            if !complete {
                token.push(b'"');
            }
            let decoded = serde_json::from_slice::<String>(&token)
                .map(Zeroizing::new)
                .or_else(|error| {
                    // A mask may turn a lossy scan view back into valid JSON.
                    // Keep the source qualification even after that repair.
                    source_partial = true;
                    if complete && preserve_complete_legacy_tokens {
                        serde_json::from_slice::<LegacyScanText>(&token).map(|text| text.0)
                    } else {
                        Err(error)
                    }
                });
            let decoded = match decoded {
                Ok(text) => text,
                Err(_) => {
                    out.extend_from_slice(&body[copied_until..start]);
                    // The fixed invalid token cannot expand a small damaged
                    // source beyond the shared output budget.
                    out.push(b'?');
                    copied_until = end;
                    cursor = end;
                    report.dropped_matches += 1;
                    report.bytes_redacted += end - start;
                    continue;
                }
            };
            drop(token);
            check_deadline()?;
            let (redacted, mut matches) = redactor.redact(decoded.as_bytes());
            drop(decoded);
            if matches.match_count() != 0 {
                out.extend_from_slice(&body[copied_until..start]);
                let text = std::str::from_utf8(redacted.bytes())
                    .map_err(|_| ReasoningFieldError::MalformedSource)?;
                let mut encoded =
                    serde_json::to_vec(text).map_err(|_| ReasoningFieldError::MalformedSource)?;
                if !complete {
                    encoded.pop();
                }
                out.extend_from_slice(&encoded);
                copied_until = end;
            }
            // String-local byte offsets are not offsets into the source.
            // Preserve exact match counts through the existing omitted-sample
            // field; the ordinary byte scan owns source bytes_scanned.
            let count = matches.match_count();
            matches.matches.clear();
            matches.dropped_matches = count;
            matches.bytes_scanned = 0;
            merge_redaction_reports(&mut report, matches);
            cursor = end;
        }
        out.extend_from_slice(&body[copied_until..]);
        if line.ends_with(b"\n") {
            out.push(b'\n');
        }
        // Use the same admitted expansion as the shared snapshot boundary.
        if out.len() as u64
            > crate::internal::ai::authorized_read::capture_redacted_output_cap(source.len() as u64)
        {
            return Err(ReasoningFieldError::SourceTooLarge);
        }
    }
    check_deadline()?;
    Ok((std::mem::take(&mut *out), report, source_partial))
}

impl Counts {
    fn merge(&mut self, other: Self) {
        self.readable_parts += other.readable_parts;
        self.readable_utf8_bytes += other.readable_utf8_bytes;
        self.empty_parts += other.empty_parts;
        self.opaque_nonempty += other.opaque_nonempty;
        self.encrypted_empty += other.encrypted_empty;
        self.encrypted_null += other.encrypted_null;
        self.encrypted_missing += other.encrypted_missing;
        self.unproven_fields += other.unproven_fields;
        self.unsupported_records += other.unsupported_records;
    }
}

/// Derive a payload-free sidecar from the retained, credential-scanned bytes.
/// Turn IDs are used only for correlation in memory; output has ordinal refs.
/// Usage is the latest turn snapshot, never a sum of cumulative counters.
pub(crate) fn sidecar(
    source: &[u8],
    deadline: Option<Instant>,
) -> Result<Option<serde_json::Value>, ReasoningFieldError> {
    use super::coverage::parse_canon_value;
    let mut turns = Vec::<Counts>::new();
    let mut tokens = Vec::<Option<u64>>::new();
    let mut keys = BTreeMap::<String, usize>::new();
    let mut ordinals = BTreeMap::new();
    let mut seen = std::collections::BTreeSet::new();
    let mut invalid_tokens = std::collections::BTreeSet::new();
    let mut current = None;
    let mut unattributed = None;
    let mut partial = false;
    let mut found = false;
    let mut statistics_unavailable = false;
    if source.len() as u64 > super::TRANSCRIPT_READ_HARD_CAP_BYTES {
        return Err(ReasoningFieldError::SourceTooLarge);
    }
    for line in source.split(|b| *b == b'\n') {
        if deadline.is_some_and(|d| Instant::now() >= d) {
            return Err(ReasoningFieldError::UnsupportedShape);
        }
        if line.iter().all(u8::is_ascii_whitespace) {
            continue;
        }
        if line.len() > MAX_LINE_BYTES {
            partial = true;
            statistics_unavailable = true;
            continue;
        }
        let entry = match parse_canon_value(line) {
            Ok(value) => Zeroizing::new(value),
            Err(_) => {
                partial = true;
                statistics_unavailable = true;
                continue;
            }
        };
        let Some(payload) = entry.get("payload") else {
            partial = true;
            continue;
        };
        let tag = entry.get("type").and_then(CanonValue::as_str);
        let event = payload.get("type").and_then(CanonValue::as_str);
        let turn = if tag == Some("turn_context")
            || (tag == Some("event_msg") && matches!(event, Some("task_started" | "turn_started")))
        {
            payload.get("turn_id").and_then(CanonValue::as_str)
        } else {
            None
        };
        if let Some(key) = turn.filter(|k| !k.is_empty() && k.len() <= 1024) {
            let index = if let Some(index) = keys.get(key) {
                *index
            } else {
                if turns.len() >= MAX_FIELDS {
                    return Err(ReasoningFieldError::SourceTooLarge);
                }
                let index = turns.len();
                ordinals.insert(index, keys.len());
                keys.insert(key.to_owned(), index);
                turns.push(Counts::default());
                tokens.push(None);
                index
            };
            current = Some(index);
        } else if turn.is_some() {
            current = None;
            partial = true;
        }
        if tag == Some("event_msg")
            && matches!(
                event,
                Some("task_complete" | "turn_complete" | "turn_aborted")
            )
        {
            current = None;
        }
        if tag == Some("token_usage_record") {
            if let Some(index) = payload
                .get("turn_id")
                .and_then(CanonValue::as_str)
                .and_then(|key| keys.get(key))
                .copied()
            {
                let value = match payload
                    .get("turn_token_usage")
                    .and_then(|v| v.get("reasoning_output_tokens"))
                {
                    Some(CanonValue::Int(n)) if *n >= 0 => Some(*n as u64),
                    _ => None,
                };
                if value.is_none() || tokens[index].zip(value).is_some_and(|(old, new)| new < old) {
                    partial = true;
                    invalid_tokens.insert(index);
                    tokens[index] = None;
                } else if !invalid_tokens.contains(&index) {
                    tokens[index] = value;
                }
            } else {
                partial = true;
            }
        }
        let candidates = if tag == Some("response_item") {
            vec![payload]
        } else if tag == Some("compacted") {
            payload
                .get("replacement_history")
                .and_then(CanonValue::as_array)
                .map(|values| values.iter().collect::<Vec<_>>())
                .unwrap_or_default()
        } else {
            Vec::new()
        };
        for payload in candidates {
            let Some(classified) = classify_payload(payload) else {
                continue;
            };
            found = true;
            partial |= tag == Some("compacted");
            // Repeated or conflicting IDs are never counted twice. The complete
            // source records stay in the transcript; statistics remain partial.
            if let Some(id) = payload
                .get("id")
                .and_then(CanonValue::as_str)
                .filter(|id| !id.is_empty() && id.len() <= 1024)
            {
                if seen.contains(id) {
                    partial = true;
                    continue;
                }
                if seen.len() >= MAX_FIELDS {
                    return Err(ReasoningFieldError::SourceTooLarge);
                }
                seen.insert(id.to_owned());
            } else {
                partial = true;
            }
            let index = match if tag == Some("compacted") {
                unattributed
            } else {
                current
            } {
                Some(index) => index,
                None => {
                    partial = true;
                    if turns.len() >= MAX_FIELDS {
                        return Err(ReasoningFieldError::SourceTooLarge);
                    }
                    let index = turns.len();
                    turns.push(Counts::default());
                    tokens.push(None);
                    unattributed = Some(index);
                    if tag != Some("compacted") {
                        current = Some(index);
                    }
                    index
                }
            };
            partial |= classified.unsupported()
                || classified.counts.opaque_nonempty != 0
                || classified.counts.unproven_fields != 0;
            turns[index].merge(classified.counts);
        }
    }
    if !found && !statistics_unavailable {
        return Ok(None);
    }
    let unproven_data = turns
        .iter()
        .any(|t| t.opaque_nonempty != 0 || t.unproven_fields != 0);
    let rows = turns.into_iter().zip(tokens).enumerate().map(|(ordinal, (counts, tokens))| {
        serde_json::json!({"turn_ordinal":ordinals.get(&ordinal),"reasoning_tokens":tokens,"counts":counts})
    }).collect::<Vec<_>>();
    Ok(Some(
        serde_json::json!({"schema_version":1,"source_status":"unproven","turn_ordinal_namespace":"codex_explicit_turn","partial":partial,"statistics_unavailable":statistics_unavailable,"has_unproven_data":unproven_data,"turns":rows}),
    ))
}

pub(crate) fn merge_sidecar_report(report: &mut serde_json::Value, sidecar: serde_json::Value) {
    if let Some(fields) = report.as_object_mut() {
        fields.insert("codex_reasoning".to_owned(), sidecar);
    } else {
        // A malformed old diagnostic cannot become a panic or an apparently
        // successful complete report. Never copy its unknown values forward.
        *report = serde_json::json!({"partial":true,"redaction_report_invalid":true,"codex_reasoning":sidecar});
    }
}

#[cfg(test)]
#[cfg(feature = "test-live-agent")]
mod tests {
    use super::*;

    #[test]
    #[ignore = "operator-local Codex legacy source acceptance; never run in CI"]
    fn codex_legacy_stored_turn_requires_bound_shape_and_real_reasoning() {
        let header = br#"{"agent_kind":"codex","completeness":"complete","logical_turn_key":"turn-A","ordinal":0,"records":[],"schema_version":2}"#;
        let reasoning = br#"{"type":"response_item","payload":{"type":"reasoning","id":"reason-A","encrypted_content":"OPAQUE"}}"#;
        let source = [header.as_slice(), b"\n", reasoning.as_slice(), b"\n"].concat();
        assert!(catalog_bound_stored_thinking_obtained(&source));
        assert!(!catalog_bound_stored_thinking_obtained(reasoning));
        let foreign = String::from_utf8(source.clone())
            .expect("fixture UTF-8")
            .replace("\"codex\"", "\"claude_code\"");
        assert!(!catalog_bound_stored_thinking_obtained(foreign.as_bytes()));
        let key_only = [
            header.as_slice(),
            b"\n",
            br#"{"encrypted_content":"OPAQUE"}"#,
            b"\n",
        ]
        .concat();
        assert!(!catalog_bound_stored_thinking_obtained(&key_only));
        let duplicate = [header.as_slice(), b"\n", br#"{"type":"response_item","payload":{"type":"reasoning","encrypted_content":"OPAQUE","encrypted_content":"OTHER"}}"#, b"\n"].concat();
        assert!(!catalog_bound_stored_thinking_obtained(&duplicate));
    }

    #[test]
    #[ignore = "operator-local Codex acceptance; never run in CI"]
    fn codex_summary_statistics_exact_id_budget_allows_duplicate_replay() {
        let mut source = String::from(
            "{\"type\":\"event_msg\",\"payload\":{\"type\":\"task_started\",\"turn_id\":\"cap-turn\"}}\n",
        );
        let mut last = String::new();
        for i in 0..MAX_FIELDS {
            last = format!(
                "{{\"type\":\"response_item\",\"payload\":{{\"type\":\"reasoning\",\"id\":\"cap-{i}\",\"summary\":[{{\"type\":\"summary_text\",\"text\":\"visible\"}}]}}}}\n"
            );
            source.push_str(&last);
        }
        assert!(sidecar(source.as_bytes(), None).unwrap().is_some());
        source.push_str(&last);
        let stats = sidecar(source.as_bytes(), None)
            .expect("a replay consumes no new identity")
            .unwrap();
        assert_eq!(stats["statistics_unavailable"], false);
        assert_eq!(stats["partial"], true);
        assert_eq!(stats["turns"][0]["counts"]["readable_parts"], MAX_FIELDS);
    }

    #[test]
    #[ignore = "operator-local Codex acceptance; never run in CI"]
    fn codex_summary_full_source_cap_has_measured_bounded_scan() {
        let cap = usize::try_from(super::super::TRANSCRIPT_READ_HARD_CAP_BYTES).unwrap();
        for layout in ["outside-string", "long-string", "jsonl"] {
            let mut source = if layout == "jsonl" {
                let row = serde_json::json!({"type":"response_item","payload":{
                    "type":"reasoning","id":"scan-repeat","summary":[{
                        "type":"summary_text","text":"x".repeat(4000)}],
                    "encrypted_content":null}})
                .to_string()
                    + "\n";
                row.repeat(cap / row.len()).into_bytes()
            } else {
                let mut bytes = br#"{"probe":"CX15-FULL-SCAN"#.to_vec();
                if layout == "outside-string" {
                    bytes.push(b'"');
                }
                let tail = if layout == "long-string" {
                    &b"\"}\n"[..]
                } else {
                    &b"}\n"[..]
                };
                bytes.resize(cap - tail.len(), b' ');
                bytes.extend_from_slice(tail);
                bytes
            };
            source.resize(cap, b' ');
            for line in source
                .split(|byte| *byte == b'\n')
                .filter(|line| !line.iter().all(u8::is_ascii_whitespace))
            {
                assert!(serde_json::from_slice::<serde_json::Value>(line).is_ok());
            }
            let start = Instant::now();
            let (ordinary, report) =
                prepare_ordinary(&source, Some(start + std::time::Duration::from_secs(30)))
                    .unwrap();
            println!(
                "CX15_FULL_SCAN bytes={} layout={layout} elapsed_ms={}",
                source.len(),
                start.elapsed().as_millis()
            );
            assert_eq!(ordinary, source);
            assert_eq!(report.match_count(), 0);
            assert!(prepare_ordinary(&source, Some(Instant::now())).is_err());
            source.push(b' ');
            assert!(matches!(
                prepare_ordinary(&source, None),
                Err(ReasoningFieldError::SourceTooLarge)
            ));
        }
    }

    #[test]
    #[ignore = "operator-local Codex acceptance; never run in CI"]
    fn codex_summary_truncated_tail_preserves_evidence_and_scans_strings() {
        let source = br#"{"type":"response_item","payload":{"type":"reasoning","summary":[{"type":"summary_text","text":"before \u0073k-ant-EXAMPLE0000000000000000"#;
        let (retained, report) = prepare_ordinary(source, None).unwrap();
        assert!(retained.ends_with(b"before <REDACTED:anthropic-api-key>"));
        assert!(serde_json::from_slice::<serde_json::Value>(&retained).is_err());
        assert_eq!(report.match_count(), 1);
        assert!(
            !std::str::from_utf8(&retained)
                .unwrap()
                .contains("EXAMPLE0000")
        );

        let (bad, _) = prepare_ordinary(br#"{"text":"private\u00"#, None).unwrap();
        assert!(serde_json::from_slice::<serde_json::Value>(&bad).is_err());
        assert!(!bad.windows(7).any(|part| part == b"private"));
    }

    #[test]
    #[ignore = "operator-local Codex acceptance; never run in CI"]
    fn codex_summary_limits_count_body_bytes_and_observe_deadline() {
        let body = "x".repeat(MAX_LINE_BYTES);
        let with_newline = format!("{body}\n");
        assert_eq!(
            prepare_ordinary(with_newline.as_bytes(), None).unwrap().0,
            with_newline.as_bytes()
        );
        let over = format!("{body}x");
        assert_eq!(
            prepare_ordinary(over.as_bytes(), None).unwrap().0,
            over.as_bytes(),
            "statistics line limits never drop an ordinary source"
        );
        assert!(matches!(
            prepare_ordinary(b"{}\n", Some(Instant::now())),
            Err(ReasoningFieldError::UnsupportedShape)
        ));
    }
    #[test]
    #[ignore = "operator-local Codex acceptance; never run in CI"]
    fn codex_summary_malformed_diagnostic_keeps_only_fixed_partial_status() {
        for mut report in [
            serde_json::Value::Null,
            serde_json::json!(["diagnostic-body-canary"]),
        ] {
            merge_sidecar_report(
                &mut report,
                serde_json::json!({"schema_version":1,"partial":true}),
            );
            assert_eq!(report["partial"], true);
            assert_eq!(report["redaction_report_invalid"], true);
            assert_eq!(report["codex_reasoning"]["schema_version"], 1);
            assert!(!report.to_string().contains("diagnostic-body-canary"));
        }
        let mut report = serde_json::json!({"bytes_scanned":8});
        merge_sidecar_report(&mut report, serde_json::json!({"partial":true}));
        assert_eq!(report["bytes_scanned"], 8);
        assert_eq!(report["codex_reasoning"]["partial"], true);
    }
    #[test]
    #[ignore = "operator-local Codex acceptance; never run in CI"]
    fn codex_summary_retention_dto_accepts_more_than_4096_bounded_turns() {
        use super::super::coverage::{Completeness, NormalizedTurn};
        let source = br#"{"type":"response_item","payload":{"type":"reasoning","summary":[{"type":"summary_text","text":"retained"}]}}
"#;
        let turns = (0..4097)
            .map(|ordinal| NormalizedTurn {
                logical_turn_key: format!("ordinal:{ordinal}"),
                ordinal,
                completeness: Completeness::Complete,
                started_at: None,
                ended_at: None,
                records: Vec::new(),
            })
            .collect::<Vec<_>>();
        let slices = (0..4097)
            .map(|ordinal| (ordinal, source.to_vec()))
            .collect();
        let dto = CodexRetentionDto::encode(&turns, slices).unwrap();
        let retained = dto
            .decode(
                &turns,
                source.len() * 4097,
                Instant::now() + std::time::Duration::from_secs(30),
            )
            .unwrap();
        assert!(!retained.is_unavailable());
        assert_eq!(retained.for_turn("ordinal:4096").unwrap().bytes(), source);
    }

    #[test]
    #[ignore = "operator-local Codex acceptance; never run in CI"]
    fn codex_summary_unavailable_retention_is_explicit_without_authority_or_body() {
        let retained = CodexRetentionDto::unavailable()
            .decode(&[], 0, Instant::now() + std::time::Duration::from_secs(1))
            .unwrap();
        assert!(retained.is_unavailable());
        assert!(retained.for_turn("ordinal:0").is_none());
        let mut report = serde_json::json!({"bytes_scanned":10,"matches":[]});
        mark_retention_unavailable(&mut report);
        assert_eq!(report["codex_reasoning"]["retention_unavailable"], true);
        assert_eq!(report["codex_reasoning"]["partial"], true);
        assert_eq!(report["bytes_scanned"], 10);
    }
    #[test]
    #[ignore = "operator-local Codex acceptance; never run in CI"]
    fn codex_summary_duplicate_legacy_keys_never_overwrite_retained_values() {
        use super::super::coverage::{Completeness, NormalizedTurn};
        let turns = (0..2)
            .map(|ordinal| NormalizedTurn {
                logical_turn_key: "legacy-key".into(),
                ordinal,
                completeness: Completeness::Incomplete,
                started_at: None,
                ended_at: None,
                records: Vec::new(),
            })
            .collect::<Vec<_>>();
        let first = b"{\"retained\":\"first\"}\n";
        let second = b"{\"retained\":\"second\"}\n";
        let slices = BTreeMap::from([(0, first.to_vec()), (1, second.to_vec())]);
        let dto = CodexRetentionDto::encode(&turns, slices).unwrap();
        let retained = dto
            .decode(
                &turns,
                first.len() + second.len(),
                Instant::now() + std::time::Duration::from_secs(1),
            )
            .unwrap();
        assert_eq!(
            retained.for_turn("legacy-key").unwrap().bytes(),
            [first.as_slice(), second.as_slice()].concat()
        );
    }
}
