//! Coverage v1 — canonical turn form, digest, and the shared turn splitter.
//!
//! Implements `docs/development/tracing/coverage-v1.md` (plan-20260713
//! ADR-DR-08 / ADR-DR-12 / DR-05c-0). Live hook writers, import writers (M4)
//! and the OpenCode export bridge (M3) all normalize transcripts through this
//! module so the same content produces the same
//! `(logical_turn_key, coverage_digest)` on every path — the cross-path
//! idempotence the `agent_coverage_claim` gate depends on.
//!
//! Strictness notes:
//! - [`CanonValue`] parsing rejects duplicate object keys (coverage-v1.md
//!   §4.6) instead of silent last-wins — different producers would otherwise
//!   disagree on the digest for the same malformed source. Non-integer
//!   numbers parse as [`CanonValue::Float`] so PROVENANCE fields (timestamps,
//!   usage) never poison a turn; a float in a SEMANTIC position (tool input)
//!   marks the turn `incomplete` and is sanitized before canonicalization
//!   (coverage-v1.md §4.5).
//! - Canonical bytes use minimal escaping, raw UTF-8 and recursive key
//!   sorting by Unicode code point (coverage-v1.md §4).

use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
    time::Instant,
};

use serde::{
    Deserialize, Serialize,
    de::{self, Deserializer, MapAccess, SeqAccess, Visitor},
    ser::{SerializeMap, SerializeSeq},
};
use sha2::{Digest, Sha256};
use zeroize::Zeroizing;

use super::reasoning;

/// Wire tag for a reasoning provider (coverage-v1 canonical form).
fn reasoning_provider_tag(provider: reasoning::ReasoningProvider) -> &'static str {
    match provider {
        reasoning::ReasoningProvider::ClaudeCode => "claude_code",
        reasoning::ReasoningProvider::Codex => "codex",
        reasoning::ReasoningProvider::OpenCode => "opencode",
    }
}

/// Wire tag for a reasoning source kind (coverage-v1 canonical form).
fn reasoning_source_kind_tag(kind: reasoning::ReasoningSourceKind) -> &'static str {
    match kind {
        reasoning::ReasoningSourceKind::Reasoning => "reasoning",
        reasoning::ReasoningSourceKind::Thinking => "thinking",
        reasoning::ReasoningSourceKind::Summary => "summary",
        reasoning::ReasoningSourceKind::Signature => "signature",
        reasoning::ReasoningSourceKind::RedactedThinkingData => "redacted_thinking_data",
        reasoning::ReasoningSourceKind::EncryptedContent => "encrypted_content",
    }
}

/// Coverage schema version this module implements (`coverage-v1.md`).
pub const COVERAGE_SCHEMA_VERSION: i64 = 1;

/// The claim's semantic coverage domain. This is independent of the public
/// `agent list` and `agent graph` envelope versions.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CoverageSchemaVersion {
    V1,
    V2,
}

impl CoverageSchemaVersion {
    pub const fn as_i64(self) -> i64 {
        match self {
            Self::V1 => COVERAGE_SCHEMA_VERSION,
            Self::V2 => 2,
        }
    }
}

/// A rejected version never falls back to v1 or includes turn content in its
/// diagnostic. The legacy serializer remains available only for frozen v1
/// compatibility oracles until the production writers adopt this API.
#[derive(Debug, thiserror::Error)]
pub enum CoverageCodecError {
    #[error("unsupported coverage schema version {0}; supported versions are 1 and 2")]
    UnsupportedVersion(i64),
    #[error("coverage schema v1 cannot encode typed reasoning; use schema v2")]
    ReasoningRequiresV2,
}

impl TryFrom<i64> for CoverageSchemaVersion {
    type Error = CoverageCodecError;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        match value {
            COVERAGE_SCHEMA_VERSION => Ok(Self::V1),
            2 => Ok(Self::V2),
            other => Err(CoverageCodecError::UnsupportedVersion(other)),
        }
    }
}

/// Strict JSON value for the coverage v1 domain: integers only, duplicate
/// object keys rejected at parse time, objects held sorted (BTreeMap) so the
/// canonical writer just iterates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CanonValue {
    Null,
    Bool(bool),
    Int(i64),
    /// Fractional / beyond-i64 number, stored as raw `f64` bits. Tolerated in
    /// PROVENANCE positions (timestamps, usage counters in the line envelope)
    /// so a harmless non-integer field never poisons a turn; a `Float` inside
    /// a SEMANTIC position (tool input) marks the turn `incomplete` and is
    /// sanitized to `Null` before canonicalization (coverage-v1.md §4.5 —
    /// non-integers never enter a digest as numbers).
    Float(u64),
    Str(String),
    Array(Vec<CanonValue>),
    Object(BTreeMap<String, CanonValue>),
}

impl Serialize for CanonValue {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Null => serializer.serialize_unit(),
            Self::Bool(value) => serializer.serialize_bool(*value),
            Self::Int(value) => serializer.serialize_i64(*value),
            Self::Float(bits) => serializer.serialize_f64(f64::from_bits(*bits)),
            Self::Str(value) => serializer.serialize_str(value),
            Self::Array(items) => {
                let mut sequence = serializer.serialize_seq(Some(items.len()))?;
                for item in items {
                    sequence.serialize_element(item)?;
                }
                sequence.end()
            }
            Self::Object(items) => {
                let mut map = serializer.serialize_map(Some(items.len()))?;
                for (key, value) in items {
                    map.serialize_entry(key, value)?;
                }
                map.end()
            }
        }
    }
}

impl CanonValue {
    pub fn get(&self, key: &str) -> Option<&CanonValue> {
        match self {
            CanonValue::Object(map) => map.get(key),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            CanonValue::Str(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            CanonValue::Bool(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&[CanonValue]> {
        match self {
            CanonValue::Array(items) => Some(items),
            _ => None,
        }
    }
}

impl<'de> Deserialize<'de> for CanonValue {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct CanonVisitor;

        impl<'de> Visitor<'de> for CanonVisitor {
            type Value = CanonValue;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("a coverage-v1 JSON value (integer numbers, unique keys)")
            }

            fn visit_unit<E>(self) -> Result<CanonValue, E> {
                Ok(CanonValue::Null)
            }

            fn visit_bool<E>(self, v: bool) -> Result<CanonValue, E> {
                Ok(CanonValue::Bool(v))
            }

            fn visit_i64<E>(self, v: i64) -> Result<CanonValue, E> {
                Ok(CanonValue::Int(v))
            }

            fn visit_u64<E: de::Error>(self, v: u64) -> Result<CanonValue, E> {
                Ok(i64::try_from(v)
                    .map(CanonValue::Int)
                    .unwrap_or_else(|_| CanonValue::Float((v as f64).to_bits())))
            }

            fn visit_f64<E: de::Error>(self, v: f64) -> Result<CanonValue, E> {
                Ok(CanonValue::Float(v.to_bits()))
            }

            fn visit_str<E>(self, v: &str) -> Result<CanonValue, E> {
                Ok(CanonValue::Str(v.to_string()))
            }

            fn visit_string<E>(self, v: String) -> Result<CanonValue, E> {
                Ok(CanonValue::Str(v))
            }

            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<CanonValue, A::Error> {
                let mut items = Vec::new();
                while let Some(item) = seq.next_element::<CanonValue>()? {
                    items.push(item);
                }
                Ok(CanonValue::Array(items))
            }

            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<CanonValue, A::Error> {
                let mut object = BTreeMap::new();
                while let Some((key, value)) = map.next_entry::<String, CanonValue>()? {
                    if object.insert(key.clone(), value).is_some() {
                        return Err(de::Error::custom(format!(
                            "coverage v1 rejects duplicate object key '{key}'"
                        )));
                    }
                }
                Ok(CanonValue::Object(object))
            }
        }

        deserializer.deserialize_any(CanonVisitor)
    }
}

/// Parse a strict coverage-v1 value from raw JSON bytes.
pub fn parse_canon_value(bytes: &[u8]) -> Result<CanonValue, serde_json::Error> {
    serde_json::from_slice(bytes)
}

/// Append the canonical (minimal-escape) JSON string form of `s` to `out`
/// (coverage-v1.md §4.4): only `"`, `\` and control chars < U+0020 escape;
/// two-char forms where defined, lowercase `\u00xx` otherwise; raw UTF-8 for
/// everything else.
fn write_canon_string(out: &mut Vec<u8>, s: &str) {
    out.push(b'"');
    for ch in s.chars() {
        match ch {
            '"' => out.extend_from_slice(b"\\\""),
            '\\' => out.extend_from_slice(b"\\\\"),
            '\u{0008}' => out.extend_from_slice(b"\\b"),
            '\t' => out.extend_from_slice(b"\\t"),
            '\n' => out.extend_from_slice(b"\\n"),
            '\u{000C}' => out.extend_from_slice(b"\\f"),
            '\r' => out.extend_from_slice(b"\\r"),
            c if (c as u32) < 0x20 => {
                out.extend_from_slice(format!("\\u{:04x}", c as u32).as_bytes());
            }
            c => {
                let mut buf = [0u8; 4];
                out.extend_from_slice(c.encode_utf8(&mut buf).as_bytes());
            }
        }
    }
    out.push(b'"');
}

fn write_canon_value(out: &mut Vec<u8>, value: &CanonValue) {
    match value {
        CanonValue::Null => out.extend_from_slice(b"null"),
        CanonValue::Bool(true) => out.extend_from_slice(b"true"),
        CanonValue::Bool(false) => out.extend_from_slice(b"false"),
        CanonValue::Int(n) => out.extend_from_slice(n.to_string().as_bytes()),
        // Defensive: semantic positions are float-sanitized before
        // canonicalization (splitter marks the turn incomplete and nulls the
        // value); an unexpected leftover serializes as null, never as a
        // platform-dependent decimal rendering.
        CanonValue::Float(_) => out.extend_from_slice(b"null"),
        CanonValue::Str(s) => write_canon_string(out, s),
        CanonValue::Array(items) => {
            out.push(b'[');
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push(b',');
                }
                write_canon_value(out, item);
            }
            out.push(b']');
        }
        CanonValue::Object(map) => {
            out.push(b'{');
            for (i, (key, value)) in map.iter().enumerate() {
                if i > 0 {
                    out.push(b',');
                }
                write_canon_string(out, key);
                out.push(b':');
                write_canon_value(out, value);
            }
            out.push(b'}');
        }
    }
}

/// One semantic record of a normalized turn (coverage-v1.md §3). Exactly the
/// digest allowlist — provenance (model/usage/timestamps/paths) never appears
/// here.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum SemanticRecord {
    User {
        text: String,
    },
    Assistant {
        text: String,
    },
    ToolCall {
        call_id: Option<String>,
        input: CanonValue,
        name: String,
    },
    ToolResult {
        call_id: Option<String>,
        content: String,
        is_error: bool,
    },
    /// Provider-visible readable reasoning (ADR-RG-01 layer 2). The text is
    /// gated behind [`reasoning::ProviderVisibleText`]: only the classification path
    /// that proved `ProviderVisible` availability can mint the variant, and
    /// the text reaches persistence only through typed redaction.
    Reasoning {
        provider: reasoning::ReasoningProvider,
        source_kind: Option<reasoning::ReasoningSourceKind>,
        text: reasoning::ProviderVisibleText,
    },
}

fn write_opt_string(out: &mut Vec<u8>, value: &Option<String>) {
    match value {
        Some(s) => write_canon_string(out, s),
        None => out.extend_from_slice(b"null"),
    }
}

impl SemanticRecord {
    /// Canonical serialization of one record. Key order is the sorted order
    /// pinned by coverage-v1.md §3 for each shape.
    fn write_canonical(&self, out: &mut Vec<u8>) {
        match self {
            SemanticRecord::User { text } => {
                out.extend_from_slice(b"{\"role\":\"user\",\"text\":");
                write_canon_string(out, text);
                out.push(b'}');
            }
            SemanticRecord::Assistant { text } => {
                out.extend_from_slice(b"{\"role\":\"assistant\",\"text\":");
                write_canon_string(out, text);
                out.push(b'}');
            }
            SemanticRecord::ToolCall {
                call_id,
                input,
                name,
            } => {
                out.extend_from_slice(b"{\"call_id\":");
                write_opt_string(out, call_id);
                out.extend_from_slice(b",\"input\":");
                write_canon_value(out, input);
                out.extend_from_slice(b",\"name\":");
                write_canon_string(out, name);
                out.extend_from_slice(b",\"role\":\"tool_call\"}");
            }
            SemanticRecord::ToolResult {
                call_id,
                content,
                is_error,
            } => {
                out.extend_from_slice(b"{\"call_id\":");
                write_opt_string(out, call_id);
                out.extend_from_slice(b",\"content\":");
                write_canon_string(out, content);
                out.extend_from_slice(b",\"is_error\":");
                out.extend_from_slice(if *is_error { b"true" } else { b"false" });
                out.extend_from_slice(b",\"role\":\"tool_result\"}");
            }
            SemanticRecord::Reasoning {
                provider,
                source_kind,
                text,
            } => {
                // Key order is alphabetical (coverage-v1.md §3): provider,
                // role, source_kind, text.
                out.extend_from_slice(b"{\"provider\":\"");
                out.extend_from_slice(reasoning_provider_tag(*provider).as_bytes());
                out.extend_from_slice(b"\",\"role\":\"reasoning\",\"source_kind\":");
                match source_kind {
                    Some(kind) => {
                        out.push(b'"');
                        out.extend_from_slice(reasoning_source_kind_tag(*kind).as_bytes());
                        out.push(b'"');
                    }
                    None => out.extend_from_slice(b"null"),
                }
                out.extend_from_slice(b",\"text\":");
                write_canon_string(out, text.as_str());
                out.push(b'}');
            }
        }
    }
}

/// Canonical bytes of one turn: the JSON array of its semantic records
/// (coverage-v1.md §4).
pub fn canonical_turn_bytes(records: &[SemanticRecord]) -> Vec<u8> {
    let mut out = Vec::new();
    out.push(b'[');
    for (i, record) in records.iter().enumerate() {
        if i > 0 {
            out.push(b',');
        }
        record.write_canonical(&mut out);
    }
    out.push(b']');
    out
}

/// `coverage_digest`: lowercase-hex SHA-256 of the canonical turn bytes.
pub fn coverage_digest_hex(records: &[SemanticRecord]) -> String {
    hex::encode(Sha256::digest(canonical_turn_bytes(records)))
}

/// Turn completeness (coverage-v1.md §6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Completeness {
    Incomplete,
    Complete,
}

impl Completeness {
    pub fn as_db_str(self) -> &'static str {
        match self {
            Completeness::Incomplete => "incomplete",
            Completeness::Complete => "complete",
        }
    }
}

/// One normalized logical turn (coverage-v1.md §2).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NormalizedTurn {
    pub logical_turn_key: String,
    pub ordinal: usize,
    pub completeness: Completeness,
    /// Provider-derived audit chronology. These fields are provenance only
    /// and never participate in coverage canonicalization or digests.
    pub started_at: Option<i64>,
    pub ended_at: Option<i64>,
    pub records: Vec<SemanticRecord>,
}

/// One version-bound encoding of an already redacted logical turn. The
/// canonical bytes can contain user text, so this type deliberately has no
/// `Debug` or `Serialize` implementation.
pub struct EncodedCoverageTurn {
    pub schema_version: CoverageSchemaVersion,
    pub canonical_bytes: Vec<u8>,
    pub digest_hex: String,
}

/// Select the production coverage domain from typed semantic content. Merely
/// observing a reasoning-like source field does not construct this variant or
/// grant native-source authority.
pub fn select_coverage_schema(turn: &NormalizedTurn) -> CoverageSchemaVersion {
    if turn
        .records
        .iter()
        .any(|record| matches!(record, SemanticRecord::Reasoning { .. }))
    {
        CoverageSchemaVersion::V2
    } else {
        CoverageSchemaVersion::V1
    }
}

/// Encode a redacted turn for an explicitly supplied claim version. Validate
/// the version and semantic allowlist before constructing any output bytes;
/// future readers may request v2 even for a turn without a reasoning record,
/// while the automatic production selector still chooses v1 for that turn.
pub fn encode_coverage_turn_for_schema(
    version: i64,
    turn: &NormalizedTurn,
) -> Result<EncodedCoverageTurn, CoverageCodecError> {
    let schema_version = CoverageSchemaVersion::try_from(version)?;
    if schema_version == CoverageSchemaVersion::V1
        && turn
            .records
            .iter()
            .any(|record| matches!(record, SemanticRecord::Reasoning { .. }))
    {
        return Err(CoverageCodecError::ReasoningRequiresV2);
    }
    let canonical_bytes = canonical_turn_bytes(&turn.records);
    let digest_hex = hex::encode(Sha256::digest(&canonical_bytes));
    Ok(EncodedCoverageTurn {
        schema_version,
        canonical_bytes,
        digest_hex,
    })
}

/// Encode under the selected production version. Call only after typed-field
/// redaction, before coverage claim reservation or any persistence sink.
pub fn encode_coverage_turn(
    turn: &NormalizedTurn,
) -> Result<EncodedCoverageTurn, CoverageCodecError> {
    encode_coverage_turn_for_schema(select_coverage_schema(turn).as_i64(), turn)
}

impl NormalizedTurn {
    pub fn digest_hex(&self) -> String {
        coverage_digest_hex(&self.records)
    }
}

fn timestamp_seconds(value: Option<&CanonValue>) -> Option<i64> {
    match value? {
        CanonValue::Int(value) if *value > 10_000_000_000 => Some(*value / 1_000),
        CanonValue::Int(value) => Some(*value),
        CanonValue::Str(value) => chrono::DateTime::parse_from_rfc3339(value)
            .ok()
            .map(|timestamp| timestamp.timestamp()),
        _ => None,
    }
}

fn observe_turn_timestamp(turn: &mut NormalizedTurn, timestamp: Option<i64>) {
    let Some(timestamp) = timestamp else {
        return;
    };
    turn.started_at = Some(
        turn.started_at
            .map_or(timestamp, |current| current.min(timestamp)),
    );
    turn.ended_at = Some(
        turn.ended_at
            .map_or(timestamp, |current| current.max(timestamp)),
    );
}

/// Does this (semantic-position) value contain any fractional number?
fn contains_float(value: &CanonValue) -> bool {
    match value {
        CanonValue::Float(_) => true,
        CanonValue::Array(items) => items.iter().any(contains_float),
        CanonValue::Object(map) => map.values().any(contains_float),
        _ => false,
    }
}

/// Replace every `Float` with `Null` so canonical bytes stay well-defined
/// (the enclosing turn is already marked `incomplete` by the caller).
fn sanitize_floats(value: &mut CanonValue) {
    match value {
        CanonValue::Float(_) => *value = CanonValue::Null,
        CanonValue::Array(items) => items.iter_mut().for_each(sanitize_floats),
        CanonValue::Object(map) => map.values_mut().for_each(sanitize_floats),
        _ => {}
    }
}

/// Validate a provider-supplied turn/message id for use as a
/// `logical_turn_key` (coverage-v1.md §2): non-empty, ≤ 64 chars, restricted
/// alphabet. Anything else falls back to the ordinal key.
fn valid_provider_turn_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 64
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

fn ordinal_key(ordinal: usize) -> String {
    format!("ordinal:{ordinal}")
}

fn make_logical_turn_keys_unique(turns: &mut [NormalizedTurn]) {
    let mut seen = BTreeSet::new();
    for turn in turns {
        if !seen.insert(turn.logical_turn_key.clone()) {
            // Provider identifiers cannot contain ':', so a normalized
            // ordinal key cannot collide with a validated provider key. The
            // ordinal itself is unique after each normalizer's retain pass.
            turn.logical_turn_key = ordinal_key(turn.ordinal);
            seen.insert(turn.logical_turn_key.clone());
        }
    }
}

/// Extract plain text out of a strict message `content` value (string form or
/// the block-array form with `{"type":"text","text":…}` entries), joining
/// multiple text blocks with `\n` in source order (coverage-v1.md §3).
/// Text extraction WITH semantic type validation (coverage-v1.md §3/§4.5):
/// a semantic text position holding a non-string (number, float, object, …)
/// is a faithfulness failure — the caller must mark the turn `incomplete`
/// instead of silently defaulting the value away.
struct ExtractedText {
    text: String,
    type_violation: bool,
}

fn extract_content_text(content: &CanonValue) -> ExtractedText {
    match content {
        CanonValue::Str(text) => ExtractedText {
            text: text.clone(),
            type_violation: false,
        },
        CanonValue::Array(blocks) => {
            let mut parts: Vec<&str> = Vec::new();
            let mut type_violation = false;
            for block in blocks {
                if block.get("type").and_then(CanonValue::as_str) == Some("text") {
                    match block.get("text") {
                        Some(CanonValue::Str(text)) => parts.push(text),
                        // A `text` block whose payload is not a string —
                        // wrong-typed semantic content.
                        Some(_) | None => type_violation = true,
                    }
                }
            }
            ExtractedText {
                text: parts.join("\n"),
                type_violation,
            }
        }
        // Content that is neither a string nor a block array is wrong-typed
        // semantic content, not merely "no text".
        CanonValue::Null => ExtractedText {
            text: String::new(),
            type_violation: false,
        },
        _ => ExtractedText {
            text: String::new(),
            type_violation: true,
        },
    }
}

/// Does this user-line content carry actual human input (a string body or at
/// least one `text` block), as opposed to being purely tool_result plumbing?
fn user_content_has_human_text(content: &CanonValue) -> bool {
    match content {
        CanonValue::Str(_) => true,
        CanonValue::Array(blocks) => blocks
            .iter()
            .any(|block| block.get("type").and_then(CanonValue::as_str) == Some("text")),
        _ => false,
    }
}

/// Split a Claude Code session JSONL transcript into normalized turns.
///
/// Line shapes follow the live envelope observed by `extract_claude_code`:
/// `{"type":"user"|"assistant","uuid":…,"message":{"role":…,"content":…}}`.
/// A user line with human text starts a new logical turn; user lines carrying
/// only `tool_result` blocks attach to the current turn (they answer the
/// assistant's tool calls — coverage-v1.md §2). Unparseable or duplicate-key
/// lines mark the enclosing turn `incomplete` and are skipped; they never
/// contribute partial content to a digest.
pub fn normalize_claude_transcript(data: &[u8]) -> Vec<NormalizedTurn> {
    // INVARIANT: `None` disables the only early-exit condition.
    let Some(turns) = normalize_claude_transcript_inner(data, None) else {
        return Vec::new();
    };
    turns
}

pub(crate) fn normalize_claude_transcript_until(
    data: &[u8],
    deadline: Instant,
) -> Option<Vec<NormalizedTurn>> {
    normalize_claude_transcript_inner(data, Some(deadline))
}

fn normalize_claude_transcript_inner(
    data: &[u8],
    deadline: Option<Instant>,
) -> Option<Vec<NormalizedTurn>> {
    let mut turns: Vec<NormalizedTurn> = Vec::new();

    fn open_turn(turns: &mut Vec<NormalizedTurn>, key: Option<String>) -> &mut NormalizedTurn {
        let ordinal = turns.len();
        let logical_turn_key = key
            .filter(|id| valid_provider_turn_id(id))
            .unwrap_or_else(|| ordinal_key(ordinal));
        turns.push(NormalizedTurn {
            logical_turn_key,
            ordinal,
            completeness: Completeness::Complete,
            started_at: None,
            ended_at: None,
            records: Vec::new(),
        });
        turns.last_mut().expect("just pushed") // INVARIANT: push above
    }

    fn current_or_open(turns: &mut Vec<NormalizedTurn>) -> &mut NormalizedTurn {
        if turns.is_empty() {
            return open_turn(turns, None);
        }
        turns.last_mut().expect("non-empty checked") // INVARIANT: checked above
    }

    for line in data.split(|b| *b == b'\n') {
        if deadline.is_some_and(|deadline| Instant::now() >= deadline) {
            return None;
        }
        if line.iter().all(u8::is_ascii_whitespace) {
            continue;
        }
        let entry = match parse_canon_value(line) {
            Ok(value) => value,
            Err(_) => {
                // Corrupt / truncated / duplicate-key line: poison the
                // enclosing turn instead of guessing at its content.
                current_or_open(&mut turns).completeness = Completeness::Incomplete;
                continue;
            }
        };
        let entry_type = entry.get("type").and_then(CanonValue::as_str).unwrap_or("");
        let entry_timestamp = timestamp_seconds(entry.get("timestamp"));
        let uuid = entry
            .get("uuid")
            .and_then(CanonValue::as_str)
            .map(str::to_string);
        let Some(message) = entry.get("message") else {
            continue; // metadata/system line — not semantic content
        };
        let Some(content) = message.get("content") else {
            continue;
        };
        // "Present but wrong-typed" optional string field (call_id / name /
        // tool_use_id): a semantic type violation, distinct from absent.
        fn opt_str_field(block: &CanonValue, key: &str, violation: &mut bool) -> Option<String> {
            match block.get(key) {
                Some(CanonValue::Str(s)) => Some(s.clone()),
                Some(CanonValue::Null) | None => None,
                Some(_) => {
                    *violation = true;
                    None
                }
            }
        }

        match entry_type {
            "user" => {
                if user_content_has_human_text(content) {
                    let extracted = extract_content_text(content);
                    let turn = open_turn(&mut turns, uuid);
                    if extracted.type_violation {
                        turn.completeness = Completeness::Incomplete;
                    }
                    turn.records.push(SemanticRecord::User {
                        text: extracted.text,
                    });
                    // Rare but possible: the same user line also carries
                    // tool_result blocks; fall through to collect them below.
                } else if extract_content_text(content).type_violation {
                    // Wrong-typed user content (e.g. a number where text
                    // belongs): poison rather than silently dropping the
                    // record.
                    current_or_open(&mut turns).completeness = Completeness::Incomplete;
                }
                if let CanonValue::Array(blocks) = content {
                    for block in blocks {
                        if block.get("type").and_then(CanonValue::as_str) == Some("tool_result") {
                            let mut violation = false;
                            let call_id = opt_str_field(block, "tool_use_id", &mut violation);
                            let result_content = block
                                .get("content")
                                .map(|content| {
                                    let extracted = extract_content_text(content);
                                    violation |= extracted.type_violation;
                                    extracted.text
                                })
                                .unwrap_or_default();
                            let is_error = match block.get("is_error") {
                                Some(CanonValue::Bool(b)) => *b,
                                Some(CanonValue::Null) | None => false,
                                Some(_) => {
                                    // Present but not a bool — wrong-typed
                                    // semantic field.
                                    violation = true;
                                    false
                                }
                            };
                            let turn = current_or_open(&mut turns);
                            if violation {
                                turn.completeness = Completeness::Incomplete;
                            }
                            turn.records.push(SemanticRecord::ToolResult {
                                call_id,
                                content: result_content,
                                is_error,
                            });
                        }
                    }
                }
            }
            "assistant" => {
                let turn = current_or_open(&mut turns);
                let extracted = extract_content_text(content);
                if extracted.type_violation {
                    turn.completeness = Completeness::Incomplete;
                }
                if !extracted.text.is_empty() {
                    turn.records.push(SemanticRecord::Assistant {
                        text: extracted.text,
                    });
                }
                if let CanonValue::Array(blocks) = content {
                    for block in blocks {
                        if block.get("type").and_then(CanonValue::as_str) == Some("tool_use") {
                            let mut violation = false;
                            let name =
                                opt_str_field(block, "name", &mut violation).unwrap_or_default();
                            let call_id = opt_str_field(block, "id", &mut violation);
                            let mut input = block.get("input").cloned().unwrap_or(CanonValue::Null);
                            // coverage-v1.md §4.5: fractional numbers in a
                            // SEMANTIC position never enter a digest — the
                            // turn fails closed to `incomplete` and the value
                            // is sanitized so canonical bytes stay defined.
                            if contains_float(&input) {
                                violation = true;
                                sanitize_floats(&mut input);
                            }
                            let turn = current_or_open(&mut turns);
                            if violation {
                                turn.completeness = Completeness::Incomplete;
                            }
                            turn.records.push(SemanticRecord::ToolCall {
                                call_id,
                                input,
                                name,
                            });
                        }
                    }
                }
            }
            _ => {} // summary / system / other line kinds: not semantic
        }
        if matches!(entry_type, "user" | "assistant")
            && let Some(turn) = turns.last_mut()
        {
            observe_turn_timestamp(turn, entry_timestamp);
        }
    }

    // Drop turns that ended up with no semantic records (e.g. a poisoned
    // fragment before the first real turn) unless they were poisoned — a
    // poisoned empty turn still matters as evidence of unreadable content.
    turns.retain(|turn| !turn.records.is_empty() || turn.completeness == Completeness::Incomplete);
    // Re-number ordinals (and ordinal-derived keys) after the retain so the
    // ordinal fallback stays gap-free.
    for (i, turn) in turns.iter_mut().enumerate() {
        if turn.ordinal != i {
            if turn.logical_turn_key == ordinal_key(turn.ordinal) {
                turn.logical_turn_key = ordinal_key(i);
            }
            turn.ordinal = i;
        }
    }
    make_logical_turn_keys_unique(&mut turns);
    Some(turns)
}

/// Split a Codex rollout JSONL stream into coverage-v1 turns.
///
/// Codex records semantic messages under `type=response_item` / `payload`.
/// A user message opens a turn, assistant messages and function-call records
/// attach to it, and `task_started` / `turn_started` contributes a stable
/// provider turn id when present. `event_msg` mirrors are provenance-only and
/// are intentionally ignored so the same user/assistant text is not counted
/// twice. Malformed semantic records poison only the current turn.
pub fn normalize_codex_rollout(data: &[u8]) -> Vec<NormalizedTurn> {
    // INVARIANT: `None` disables the only early-exit condition.
    let Some(turns) = normalize_codex_rollout_inner(data, None) else {
        return Vec::new();
    };
    turns
}

pub(crate) fn normalize_codex_rollout_until(
    data: &[u8],
    deadline: Instant,
) -> Option<Vec<NormalizedTurn>> {
    normalize_codex_rollout_inner(data, Some(deadline))
}

fn normalize_codex_rollout_inner(
    data: &[u8],
    deadline: Option<Instant>,
) -> Option<Vec<NormalizedTurn>> {
    let mut turns: Vec<NormalizedTurn> = Vec::new();
    let mut pending_turn_id: Option<String> = None;

    fn open_turn(turns: &mut Vec<NormalizedTurn>, key: Option<String>) -> &mut NormalizedTurn {
        let ordinal = turns.len();
        turns.push(NormalizedTurn {
            logical_turn_key: key
                .filter(|id| valid_provider_turn_id(id))
                .unwrap_or_else(|| ordinal_key(ordinal)),
            ordinal,
            completeness: Completeness::Complete,
            started_at: None,
            ended_at: None,
            records: Vec::new(),
        });
        turns.last_mut().expect("just pushed") // INVARIANT: push above
    }

    fn current_or_open(turns: &mut Vec<NormalizedTurn>) -> &mut NormalizedTurn {
        if turns.is_empty() {
            return open_turn(turns, None);
        }
        turns.last_mut().expect("non-empty checked") // INVARIANT: checked above
    }

    fn text_blocks(content: &CanonValue, violation: &mut bool) -> String {
        match content {
            CanonValue::Str(text) => text.clone(),
            CanonValue::Array(blocks) => {
                let mut parts = Vec::new();
                for block in blocks {
                    if !matches!(block, CanonValue::Object(_)) {
                        *violation = true;
                        continue;
                    }
                    if let Some("input_text" | "output_text" | "text") =
                        block.get("type").and_then(CanonValue::as_str)
                    {
                        match block.get("text") {
                            Some(CanonValue::Str(text)) => parts.push(text.clone()),
                            _ => *violation = true,
                        }
                    }
                }
                parts.join("\n")
            }
            _ => {
                *violation = true;
                String::new()
            }
        }
    }

    for line in data.split(|byte| *byte == b'\n') {
        if deadline.is_some_and(|deadline| Instant::now() >= deadline) {
            return None;
        }
        if line.iter().all(u8::is_ascii_whitespace) {
            continue;
        }
        let entry = match parse_canon_value(line) {
            Ok(entry) => entry,
            Err(_) => {
                current_or_open(&mut turns).completeness = Completeness::Incomplete;
                continue;
            }
        };
        let entry_type = entry.get("type").and_then(CanonValue::as_str).unwrap_or("");
        let entry_timestamp = timestamp_seconds(entry.get("timestamp"));
        let Some(payload) = entry.get("payload") else {
            continue;
        };
        if entry_type == "event_msg" {
            let kind = payload
                .get("type")
                .and_then(CanonValue::as_str)
                .unwrap_or("");
            if matches!(kind, "task_started" | "turn_started") {
                pending_turn_id = payload
                    .get("turn_id")
                    .and_then(CanonValue::as_str)
                    .filter(|id| valid_provider_turn_id(id))
                    .map(str::to_string);
            }
            continue;
        }
        if entry_type != "response_item" {
            continue;
        }

        match payload
            .get("type")
            .and_then(CanonValue::as_str)
            .unwrap_or("")
        {
            "message" => {
                let mut violation = false;
                let role = match payload.get("role") {
                    Some(CanonValue::Str(role)) => role.as_str(),
                    _ => {
                        violation = true;
                        ""
                    }
                };
                let text = payload
                    .get("content")
                    .map(|content| text_blocks(content, &mut violation))
                    .unwrap_or_else(|| {
                        violation = true;
                        String::new()
                    });
                match role {
                    "user" => {
                        let turn = open_turn(&mut turns, pending_turn_id.take());
                        if violation {
                            turn.completeness = Completeness::Incomplete;
                        }
                        turn.records.push(SemanticRecord::User { text });
                    }
                    "assistant" => {
                        let turn = current_or_open(&mut turns);
                        if violation {
                            turn.completeness = Completeness::Incomplete;
                        }
                        if !text.is_empty() {
                            turn.records.push(SemanticRecord::Assistant { text });
                        }
                    }
                    _ if violation => {
                        current_or_open(&mut turns).completeness = Completeness::Incomplete;
                    }
                    _ => {}
                }
            }
            kind @ ("function_call" | "custom_tool_call") => {
                let mut violation = false;
                let name = match payload.get("name") {
                    Some(CanonValue::Str(name)) => name.clone(),
                    _ => {
                        violation = true;
                        String::new()
                    }
                };
                let call_id = match payload.get("call_id") {
                    Some(CanonValue::Str(id)) => Some(id.clone()),
                    Some(CanonValue::Null) | None => None,
                    Some(_) => {
                        violation = true;
                        None
                    }
                };
                let mut input = match kind {
                    "function_call" => match payload.get("arguments") {
                        Some(CanonValue::Str(raw)) => match parse_canon_value(raw.as_bytes()) {
                            Ok(value) => value,
                            Err(_) => {
                                violation = true;
                                CanonValue::Str(raw.clone())
                            }
                        },
                        Some(value) => {
                            violation = true;
                            value.clone()
                        }
                        None => {
                            violation = true;
                            CanonValue::Null
                        }
                    },
                    "custom_tool_call" => match payload.get("input") {
                        Some(CanonValue::Str(raw)) => CanonValue::Str(raw.clone()),
                        Some(value) => {
                            violation = true;
                            value.clone()
                        }
                        None => {
                            violation = true;
                            CanonValue::Null
                        }
                    },
                    _ => CanonValue::Null,
                };
                if contains_float(&input) {
                    sanitize_floats(&mut input);
                    violation = true;
                }
                let turn = current_or_open(&mut turns);
                if violation {
                    turn.completeness = Completeness::Incomplete;
                }
                turn.records.push(SemanticRecord::ToolCall {
                    call_id,
                    input,
                    name,
                });
            }
            "function_call_output" | "custom_tool_call_output" => {
                let mut violation = false;
                let call_id = match payload.get("call_id") {
                    Some(CanonValue::Str(id)) => Some(id.clone()),
                    Some(CanonValue::Null) | None => None,
                    Some(_) => {
                        violation = true;
                        None
                    }
                };
                let content = match payload.get("output") {
                    Some(CanonValue::Str(output)) => output.clone(),
                    Some(CanonValue::Null) | None => String::new(),
                    Some(_) => {
                        violation = true;
                        String::new()
                    }
                };
                let turn = current_or_open(&mut turns);
                if violation {
                    turn.completeness = Completeness::Incomplete;
                }
                turn.records.push(SemanticRecord::ToolResult {
                    call_id,
                    content,
                    is_error: false,
                });
            }
            _ => {}
        }
        if entry_type == "response_item"
            && let Some(turn) = turns.last_mut()
        {
            observe_turn_timestamp(turn, entry_timestamp);
        }
    }

    turns.retain(|turn| !turn.records.is_empty() || turn.completeness == Completeness::Incomplete);
    for (ordinal, turn) in turns.iter_mut().enumerate() {
        if turn.logical_turn_key == ordinal_key(turn.ordinal) {
            turn.logical_turn_key = ordinal_key(ordinal);
        }
        turn.ordinal = ordinal;
    }
    make_logical_turn_keys_unique(&mut turns);
    Some(turns)
}

/// Split native OpenCode 2.0.26 or classic `info/parts` exports into turns.
/// Each human message opens a turn keyed by its validated native `id` or
/// classic `info.id` (otherwise ordinal). Assistant text blocks join with
/// newlines, followed by ordered tool call/results, matching coverage-v1.
/// Native tools use `name/id/state.input/content`, classic tools use
/// `tool/callID/state.input/output`. In-flight tools, unsupported semantic
/// fields and unknown/reasoning content keep their turn incomplete.
/// Native idle settles all pending human splits but never clears violations;
/// native semantic messages without a subsequent valid idle stay incomplete.
pub fn normalize_opencode_export(data: &[u8]) -> Vec<NormalizedTurn> {
    // INVARIANT: `None` disables the only early-exit condition.
    let Some(turns) = normalize_opencode_export_inner(data, None, None) else {
        return Vec::new();
    };
    turns
}

pub(crate) fn normalize_opencode_export_until(
    data: &[u8],
    deadline: Instant,
) -> Option<Vec<NormalizedTurn>> {
    normalize_opencode_export_inner(data, Some(deadline), None)
}

struct BoundedOrdinaryWriter {
    bytes: Zeroizing<Vec<u8>>,
    cap: usize,
}

impl BoundedOrdinaryWriter {
    fn into_bytes(mut self) -> Vec<u8> {
        std::mem::take(&mut *self.bytes)
    }
}

impl std::io::Write for BoundedOrdinaryWriter {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        let next = self
            .bytes
            .len()
            .checked_add(bytes.len())
            .filter(|next| *next <= self.cap)
            .ok_or_else(|| std::io::Error::other("bounded OpenCode output exceeded source cap"))?;
        let additional = next - self.bytes.len();
        self.bytes
            .try_reserve(additional)
            .map_err(|_| std::io::Error::other("bounded OpenCode output allocation failed"))?;
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

/// Serialize an already separated ordinary tree under its existing byte cap.
/// No source/capture/native authority is constructed by this memory helper.
pub(crate) fn encode_bounded_json(
    value: &impl serde::Serialize,
    cap: usize,
) -> Result<Vec<u8>, reasoning::ReasoningFieldError> {
    let mut writer = BoundedOrdinaryWriter {
        bytes: Zeroizing::new(Vec::new()),
        cap,
    };
    serde_json::to_writer(&mut writer, value)
        .map_err(|_| reasoning::ReasoningFieldError::SourceTooLarge)?;
    Ok(writer.into_bytes())
}

pub(crate) struct CoverageRedactionLimits {
    pub(crate) max_source_bytes: u64,
    pub(crate) max_output_bytes: usize,
    pub(crate) max_working_set_bytes: usize,
    pub(crate) deadline: Option<Instant>,
}

/// Redacted data only. This is not source authorization, a native witness,
/// a serialized DTO, or permission to commit a checkpoint.
pub(crate) struct RedactedCoveragePreparation {
    pub(crate) transcript: super::RedactedBytes,
    pub(crate) turns: Vec<NormalizedTurn>,
    pub(crate) report: super::RedactionReport,
    pub(crate) partial: bool,
}

/// Pure provider data preparation; no source or native authority is issued.
pub(crate) fn prepare_coverage_preview(
    agent_kind: &str,
    bytes: Vec<u8>,
    limits: CoverageRedactionLimits,
) -> Result<RedactedCoveragePreparation, reasoning::ReasoningFieldError> {
    use super::AgentKind;
    let mut bytes = Zeroizing::new(bytes);
    if bytes.len() as u64 > limits.max_source_bytes
        || bytes.len() as u64 > super::TRANSCRIPT_READ_HARD_CAP_BYTES
    {
        return Err(reasoning::ReasoningFieldError::SourceTooLarge);
    }
    let kind = AgentKind::from_db_str(agent_kind)
        .ok_or(reasoning::ReasoningFieldError::UnsupportedShape)?;
    let max_output = limits.max_output_bytes;
    let max_working = limits.max_working_set_bytes;
    if kind == AgentKind::OpenCode {
        let source = match super::opencode_reasoning::classify_opencode_coverage_source(
            std::mem::take(&mut *bytes),
        ) {
            reasoning::OpenCodeCoverageClassification::Parsed(source) => *source,
            reasoning::OpenCodeCoverageClassification::Partial(reason) => {
                return Err(reason);
            }
        };
        if limits
            .deadline
            .is_some_and(|deadline| Instant::now() >= deadline)
        {
            return Err(reasoning::ReasoningFieldError::UnsupportedShape);
        }
        // This data-only consumer destroys, rather than validates or returns,
        // every sealed candidate/native/permit capability.
        let (ordinary, source_partial) =
            reasoning::discard_prepared_authority_for_coverage_preview(source.prepared);
        let partial = source.partial || source_partial;
        let prepared = prepare_opencode_redacted_coverage(
            ordinary,
            source.source_session.as_str(),
            source.visible_parts,
            partial,
            CoverageRedactionLimits {
                max_source_bytes: limits.max_source_bytes,
                max_output_bytes: max_output,
                max_working_set_bytes: max_working,
                deadline: limits.deadline,
            },
        );
        if limits
            .deadline
            .is_some_and(|deadline| Instant::now() >= deadline)
        {
            return Err(reasoning::ReasoningFieldError::UnsupportedShape);
        }
        return prepared;
    }
    // Claude Code and Codex currently have no reasoning preprocessor port;
    // retain their established ordinary redactor/normalizer decisions. This
    // preview never asks a source-authorization adapter to grant authority.
    if !matches!(kind, AgentKind::ClaudeCode | AgentKind::Codex) {
        return Err(reasoning::ReasoningFieldError::UnsupportedShape);
    }
    let ordinary = std::mem::take(&mut *bytes);
    if limits
        .deadline
        .is_some_and(|deadline| Instant::now() >= deadline)
    {
        return Err(reasoning::ReasoningFieldError::UnsupportedShape);
    }
    let (transcript, mut report) = super::Redactor::new_default()
        .redact_owned_bounded(ordinary, max_output, max_working)
        .ok_or(reasoning::ReasoningFieldError::SourceTooLarge)?;
    let mut turns = match (kind, limits.deadline) {
        (AgentKind::ClaudeCode, Some(deadline)) => {
            normalize_claude_transcript_until(transcript.bytes(), deadline)
        }
        (AgentKind::Codex, Some(deadline)) => {
            normalize_codex_rollout_until(transcript.bytes(), deadline)
        }
        (AgentKind::ClaudeCode, None) => Some(normalize_claude_transcript(transcript.bytes())),
        (AgentKind::Codex, None) => Some(normalize_codex_rollout(transcript.bytes())),
        _ => return Err(reasoning::ReasoningFieldError::UnsupportedShape),
    }
    .ok_or(reasoning::ReasoningFieldError::UnsupportedShape)?;
    let partial = false;
    merge_redaction_reports(&mut report, redact_turns_with_report(&mut turns));
    if limits
        .deadline
        .is_some_and(|deadline| Instant::now() >= deadline)
    {
        return Err(reasoning::ReasoningFieldError::UnsupportedShape);
    }
    Ok(RedactedCoveragePreparation {
        transcript,
        turns,
        report,
        partial,
    })
}

fn source_part_view(
    document: &CanonValue,
    message_ordinal: usize,
    part_ordinal: usize,
) -> Option<(&str, &CanonValue)> {
    let message = document.get("messages")?.as_array()?.get(message_ordinal)?;
    let (id, parts) = if message.get("type").is_some() {
        if message.get("type")?.as_str()? != "assistant"
            || message.get("info").is_some()
            || message.get("parts").is_some()
        {
            return None;
        }
        (
            message.get("id")?.as_str()?,
            message.get("content")?.as_array()?,
        )
    } else {
        let info = message.get("info")?;
        if info.get("role")?.as_str()? != "assistant" || message.get("content").is_some() {
            return None;
        }
        (info.get("id")?.as_str()?, message.get("parts")?.as_array()?)
    };
    Some((id, parts.get(part_ordinal)?))
}

fn source_part_fields_mut(
    document: &mut CanonValue,
    message_ordinal: usize,
    part_ordinal: usize,
) -> Option<&mut BTreeMap<String, CanonValue>> {
    let CanonValue::Object(root) = document else {
        return None;
    };
    let CanonValue::Array(messages) = root.get_mut("messages")? else {
        return None;
    };
    let CanonValue::Object(message) = messages.get_mut(message_ordinal)? else {
        return None;
    };
    let key = if message.contains_key("type") {
        "content"
    } else {
        "parts"
    };
    let CanonValue::Array(parts) = message.get_mut(key)? else {
        return None;
    };
    let CanonValue::Object(part) = parts.get_mut(part_ordinal)? else {
        return None;
    };
    Some(part)
}

pub(crate) fn prepare_opencode_redacted_coverage(
    ordinary: Vec<u8>,
    source_session: &str,
    mut visible: Vec<reasoning::OpenCodeVisibleReasoningPart>,
    mut partial: bool,
    limits: CoverageRedactionLimits,
) -> Result<RedactedCoveragePreparation, reasoning::ReasoningFieldError> {
    use reasoning::ReasoningFieldError;
    let mut ordinary = Zeroizing::new(ordinary);
    if ordinary.len() as u64 > limits.max_source_bytes || visible.len() > 4096 {
        return Err(ReasoningFieldError::SourceTooLarge);
    }
    let elapsed = || {
        limits
            .deadline
            .is_some_and(|deadline| Instant::now() >= deadline)
    };
    if elapsed() {
        return Err(ReasoningFieldError::UnsupportedShape);
    }
    // Bind the original classifier output before any redactor can transform
    // IDs or text. Coordinates locate a part; text equality only confirms it.
    if !visible.is_empty() {
        let original = Zeroizing::new(
            parse_canon_value(&ordinary).map_err(|_| ReasoningFieldError::MalformedSource)?,
        );
        if original
            .get("info")
            .and_then(|i| i.get("id"))
            .and_then(CanonValue::as_str)
            != Some(source_session)
        {
            return Err(ReasoningFieldError::UnsupportedShape);
        }
        let mut coordinates = BTreeSet::new();
        for part in &visible {
            let (message_id, field) =
                source_part_view(&original, part.message_ordinal, part.part_ordinal)
                    .ok_or(ReasoningFieldError::UnsupportedShape)?;
            let SemanticRecord::Reasoning {
                provider: reasoning::ReasoningProvider::OpenCode,
                source_kind: Some(reasoning::ReasoningSourceKind::Reasoning),
                text,
            } = &part.record
            else {
                return Err(ReasoningFieldError::UnsupportedShape);
            };
            if message_id != part.source_message.as_str()
                || field.get("id").and_then(CanonValue::as_str) != Some(part.source_part.as_str())
                || field.get("type").and_then(CanonValue::as_str) != Some("reasoning")
                || field.get("text").and_then(CanonValue::as_str) != Some(text.as_str())
                || !coordinates.insert((part.message_ordinal, part.part_ordinal))
            {
                return Err(ReasoningFieldError::UnsupportedShape);
            }
        }
    }
    let redactor = super::Redactor::new_default();
    let (mut transcript, mut report) = redactor
        .redact_owned_bounded(
            std::mem::take(&mut *ordinary),
            limits.max_output_bytes,
            limits.max_working_set_bytes,
        )
        .ok_or(ReasoningFieldError::SourceTooLarge)?;
    if elapsed() {
        return Err(ReasoningFieldError::UnsupportedShape);
    }
    let mut redacted_session = source_session.to_owned();
    if !visible.is_empty() {
        let mut document = Zeroizing::new(
            parse_canon_value(transcript.bytes())
                .map_err(|_| ReasoningFieldError::MalformedSource)?,
        );
        // The original decoded thinking value, not a possibly half-matched
        // JSON escape spelling, is the input to typed-field redaction.
        for part in &mut visible {
            let SemanticRecord::Reasoning { text, .. } = &mut part.record else {
                return Err(ReasoningFieldError::UnsupportedShape);
            };
            merge_redaction_reports(&mut report, text.redact_with(&redactor));
            let field =
                source_part_fields_mut(&mut document, part.message_ordinal, part.part_ordinal)
                    .ok_or(ReasoningFieldError::UnsupportedShape)?;
            field.insert("text".to_owned(), CanonValue::Str(text.as_str().to_owned()));
        }
        // Decode-aware redaction also prevents canonical serialization from
        // revealing an escaped secret in ordinary metadata or tool fields.
        // Extra decoded matches are conservatively partial; visible thinking
        // was already redacted above and does not itself poison completeness.
        let mut decoded_report = super::RedactionReport::default();
        redact_canon_value(&mut document, &redactor, &mut decoded_report);
        partial |= decoded_report.match_count() != 0;
        merge_redaction_reports(&mut report, decoded_report);
        redacted_session = document
            .get("info")
            .and_then(|i| i.get("id"))
            .and_then(CanonValue::as_str)
            .ok_or(ReasoningFieldError::UnsupportedShape)?
            .to_owned();
        for part in &mut visible {
            let (message_id, field) =
                source_part_view(&document, part.message_ordinal, part.part_ordinal)
                    .ok_or(ReasoningFieldError::UnsupportedShape)?;
            part.source_message = Zeroizing::new(message_id.to_owned());
            part.source_part = Zeroizing::new(
                field
                    .get("id")
                    .and_then(CanonValue::as_str)
                    .ok_or(ReasoningFieldError::UnsupportedShape)?
                    .to_owned(),
            );
        }
        let bytes = encode_bounded_json(&*document, limits.max_output_bytes)?;
        // Serialization introduced no new content; re-enter the RedactedBytes
        // constructor through the same bounded redactor, not a raw wrapper.
        (transcript, decoded_report) = redactor
            .redact_owned_bounded(bytes, limits.max_output_bytes, limits.max_working_set_bytes)
            .ok_or(ReasoningFieldError::SourceTooLarge)?;
        merge_redaction_reports(&mut report, decoded_report);
    }
    let mut turns = normalize_opencode_typed_parts(
        transcript.bytes(),
        &redacted_session,
        visible,
        partial,
        limits.deadline,
        limits.max_output_bytes,
    )
    .ok_or(ReasoningFieldError::UnsupportedShape)?;
    merge_redaction_reports(&mut report, redact_turns_with_report(&mut turns));
    if elapsed() {
        return Err(ReasoningFieldError::UnsupportedShape);
    }
    Ok(RedactedCoveragePreparation {
        transcript,
        turns,
        report,
        partial,
    })
}

/// Private preparation path: original classifier coordinates must still
/// identify exactly the same visible part in the prepared ordinary export.
/// This function does not redact or authorize persistence; only the snapshot
/// service may consume its raw typed result before doing typed redaction.
pub(crate) fn normalize_opencode_typed_parts(
    data: &[u8],
    source_session: &str,
    visible_parts: Vec<reasoning::OpenCodeVisibleReasoningPart>,
    partial: bool,
    deadline: Option<Instant>,
    max_redacted_bytes: usize,
) -> Option<Vec<NormalizedTurn>> {
    if deadline.is_some_and(|deadline| Instant::now() >= deadline)
        || data.len() > max_redacted_bytes
        || visible_parts.len() > 4096
    {
        return None;
    }
    let mut records = BTreeMap::new();
    if !visible_parts.is_empty() {
        let document = Zeroizing::new(parse_canon_value(data).ok()?);
        if document.get("info")?.get("id")?.as_str()? != source_session {
            return None;
        }
        let messages = document.get("messages")?.as_array()?;
        for part in visible_parts {
            if deadline.is_some_and(|deadline| Instant::now() >= deadline) {
                return None;
            }
            let message = messages.get(part.message_ordinal)?;
            let (message_id, parts) = if message.get("type").is_some() {
                if message.get("type")?.as_str()? != "assistant"
                    || message.get("info").is_some()
                    || message.get("parts").is_some()
                {
                    return None;
                }
                (
                    message.get("id")?.as_str()?,
                    message.get("content")?.as_array()?,
                )
            } else {
                let info = message.get("info")?;
                if info.get("role")?.as_str()? != "assistant" || message.get("content").is_some() {
                    return None;
                }
                (info.get("id")?.as_str()?, message.get("parts")?.as_array()?)
            };
            let original = parts.get(part.part_ordinal)?;
            let SemanticRecord::Reasoning {
                provider: reasoning::ReasoningProvider::OpenCode,
                source_kind: Some(reasoning::ReasoningSourceKind::Reasoning),
                text,
            } = &part.record
            else {
                return None;
            };
            if message_id != part.source_message.as_str()
                || original.get("id")?.as_str()? != part.source_part.as_str()
                || original.get("type")?.as_str()? != "reasoning"
                || original.get("text")?.as_str()? != text.as_str()
                || records
                    .insert((part.message_ordinal, part.part_ordinal), part.record)
                    .is_some()
            {
                return None;
            }
        }
    }
    // With no readable part, keep the legacy record order and byte behavior.
    let mut turns =
        normalize_opencode_export_inner(data, deadline, (!records.is_empty()).then_some(records))?;
    if partial {
        for turn in &mut turns {
            turn.completeness = Completeness::Incomplete;
        }
    }
    Some(turns)
}

fn normalize_opencode_export_inner(
    data: &[u8],
    deadline: Option<Instant>,
    mut typed_parts: Option<BTreeMap<(usize, usize), SemanticRecord>>,
) -> Option<Vec<NormalizedTurn>> {
    let mut turns: Vec<NormalizedTurn> = Vec::new();
    let Ok(document) = parse_canon_value(data) else {
        // Whole-document parse failure (truncated export, duplicate keys):
        // one poisoned empty turn is the honest representation.
        turns.push(NormalizedTurn {
            logical_turn_key: ordinal_key(0),
            ordinal: 0,
            completeness: Completeness::Incomplete,
            started_at: None,
            ended_at: None,
            records: Vec::new(),
        });
        return Some(turns);
    };
    let Some(messages) = document.get("messages").and_then(CanonValue::as_array) else {
        // Valid JSON but structurally NOT an export document (missing or
        // wrong-typed `messages`): poison, never "no work" — advancing the
        // job on this would silently drop capture (Codex M3 R1 P1-5).
        turns.push(NormalizedTurn {
            logical_turn_key: ordinal_key(0),
            ordinal: 0,
            completeness: Completeness::Incomplete,
            started_at: None,
            ended_at: None,
            records: Vec::new(),
        });
        return Some(turns);
    };

    fn open_turn(turns: &mut Vec<NormalizedTurn>, key: Option<String>) -> &mut NormalizedTurn {
        let ordinal = turns.len();
        let logical_turn_key = key
            .filter(|id| {
                !id.is_empty()
                    && id.len() <= 64
                    && id
                        .chars()
                        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
            })
            .unwrap_or_else(|| ordinal_key(ordinal));
        turns.push(NormalizedTurn {
            logical_turn_key,
            ordinal,
            completeness: Completeness::Complete,
            started_at: None,
            ended_at: None,
            records: Vec::new(),
        });
        turns.last_mut().expect("just pushed") // INVARIANT: push above
    }
    fn current_or_open(turns: &mut Vec<NormalizedTurn>) -> &mut NormalizedTurn {
        if turns.is_empty() {
            return open_turn(turns, None);
        }
        turns.last_mut().expect("non-empty checked") // INVARIANT: checked above
    }

    fn valid_mention(value: Option<&CanonValue>) -> bool {
        let Some(value) = value else {
            return true;
        };
        let number = |key| match value.get(key) {
            Some(CanonValue::Int(_)) => true,
            Some(CanonValue::Float(bits)) => f64::from_bits(*bits).is_finite(),
            _ => false,
        };
        number("start") && number("end") && value.get("text").and_then(CanonValue::as_str).is_some()
    }

    fn native_user_attachments_supported(message: &CanonValue) -> bool {
        let files_supported = match message.get("files") {
            None => true,
            Some(CanonValue::Array(files)) => files.is_empty(),
            _ => false,
        };
        let agents_supported = match message.get("agents") {
            None => true,
            Some(CanonValue::Array(agents)) => agents.iter().all(|agent| {
                agent.get("name").and_then(CanonValue::as_str).is_some()
                    && valid_mention(agent.get("mention"))
            }),
            _ => false,
        };
        let skills_supported = match message.get("skills") {
            None => true,
            Some(CanonValue::Array(skills)) => skills.iter().all(|skill| {
                skill.get("id").and_then(CanonValue::as_str).is_some()
                    && skill.get("name").and_then(CanonValue::as_str).is_some()
                    && skill.get("text").is_none()
                    && valid_mention(skill.get("mention"))
            }),
            _ => false,
        };
        files_supported && agents_supported && skills_supported
    }

    fn native_tool(
        turn: &mut NormalizedTurn,
        part: &CanonValue,
        deadline: Option<Instant>,
    ) -> Option<()> {
        let mut violation = false;
        let name = match part.get("name") {
            Some(CanonValue::Str(name)) => name.clone(),
            _ => {
                violation = true;
                String::new()
            }
        };
        let call_id = match part.get("id") {
            Some(CanonValue::Str(id)) => Some(id.clone()),
            _ => {
                violation = true;
                None
            }
        };
        let state = part.get("state");
        let status = state
            .and_then(|s| s.get("status"))
            .and_then(CanonValue::as_str);
        let mut input = match state.and_then(|s| s.get("input")) {
            Some(value @ CanonValue::Object(_)) => value.clone(),
            _ => {
                violation = true;
                CanonValue::Null
            }
        };
        if contains_float(&input) {
            violation = true;
            sanitize_floats(&mut input);
        }
        turn.records.push(SemanticRecord::ToolCall {
            call_id: call_id.clone(),
            input,
            name,
        });
        match status {
            Some("completed" | "error") => {
                let is_error = status == Some("error");
                let error = state.and_then(|s| s.get("error"));
                let error_message = error
                    .and_then(|e| e.get("message"))
                    .and_then(CanonValue::as_str);
                if is_error
                    && (error
                        .and_then(|e| e.get("type"))
                        .and_then(CanonValue::as_str)
                        .is_none()
                        || error_message.is_none())
                {
                    violation = true;
                }
                let mut fragments = Vec::new();
                match state.and_then(|s| s.get("content")) {
                    Some(CanonValue::Array(content)) if !content.is_empty() => {
                        for item in content {
                            if deadline.is_some_and(|d| Instant::now() >= d) {
                                return None;
                            }
                            match item.get("type").and_then(CanonValue::as_str) {
                                Some("text") => match item.get("text").and_then(CanonValue::as_str)
                                {
                                    Some(text) => fragments.push(text),
                                    None => violation = true,
                                },
                                // Attachment URIs/bytes and future content cannot be represented
                                // by coverage-v1 text. Do not silently claim a complete result.
                                _ => violation = true,
                            }
                        }
                    }
                    None if is_error => {
                        if let Some(text) = error_message {
                            fragments.push(text);
                        }
                    }
                    _ => violation = true,
                }
                turn.records.push(SemanticRecord::ToolResult {
                    call_id,
                    content: fragments.join("\n"),
                    is_error,
                });
            }
            _ => violation = true,
        }
        if violation {
            turn.completeness = Completeness::Incomplete;
        }
        Some(())
    }

    fn classic_tool(turn: &mut NormalizedTurn, part: &CanonValue) {
        let mut part_violation = false;
        let name = match part.get("tool") {
            Some(CanonValue::Str(t)) => t.clone(),
            Some(_) => {
                part_violation = true;
                String::new()
            }
            None => String::new(),
        };
        let call_id = match part.get("callID") {
            Some(CanonValue::Str(c)) => Some(c.clone()),
            Some(CanonValue::Null) | None => None,
            Some(_) => {
                part_violation = true;
                None
            }
        };
        let state = part.get("state");
        let status = state
            .and_then(|st| st.get("status"))
            .and_then(CanonValue::as_str)
            .unwrap_or("");
        let mut input = state
            .and_then(|st| st.get("input"))
            .cloned()
            .unwrap_or(CanonValue::Null);
        if contains_float(&input) {
            part_violation = true;
            sanitize_floats(&mut input);
        }
        turn.records.push(SemanticRecord::ToolCall {
            call_id: call_id.clone(),
            input,
            name,
        });
        match status {
            "completed" | "error" => {
                let content = match state.and_then(|st| st.get("output")) {
                    Some(CanonValue::Str(o)) => o.clone(),
                    Some(CanonValue::Null) | None => String::new(),
                    Some(_) => {
                        part_violation = true;
                        String::new()
                    }
                };
                turn.records.push(SemanticRecord::ToolResult {
                    call_id,
                    content,
                    is_error: status == "error",
                });
            }
            // Tool still in flight at export time: the turn is
            // not faithfully complete yet.
            _ => part_violation = true,
        }
        if part_violation {
            turn.completeness = Completeness::Incomplete;
        }
    }

    fn flush_typed_text(turn: &mut NormalizedTurn, fragments: &mut Vec<&str>) {
        if !fragments.is_empty() {
            let text = fragments.join("\n");
            fragments.clear();
            if !text.is_empty() {
                turn.records.push(SemanticRecord::Assistant { text });
            }
        }
    }

    fn typed_message_parts(
        turn: &mut NormalizedTurn,
        parts: &[CanonValue],
        message_ordinal: usize,
        native: bool,
        typed: &mut BTreeMap<(usize, usize), SemanticRecord>,
        deadline: Option<Instant>,
    ) -> Option<()> {
        let mut fragments = Vec::new();
        for (part_ordinal, part) in parts.iter().enumerate() {
            if deadline.is_some_and(|deadline| Instant::now() >= deadline) {
                return None;
            }
            match part.get("type").and_then(CanonValue::as_str) {
                Some("text") => match part.get("text").and_then(CanonValue::as_str) {
                    Some(text) => fragments.push(text),
                    None => turn.completeness = Completeness::Incomplete,
                },
                Some("reasoning") => {
                    flush_typed_text(turn, &mut fragments);
                    match typed.remove(&(message_ordinal, part_ordinal)) {
                        Some(record) => turn.records.push(record),
                        None => turn.completeness = Completeness::Incomplete,
                    }
                }
                Some("tool") => {
                    flush_typed_text(turn, &mut fragments);
                    if native {
                        native_tool(turn, part, deadline)?;
                    } else {
                        classic_tool(turn, part);
                    }
                }
                Some(
                    "step-finish" | "patch" | "step-start" | "snapshot" | "retry" | "compaction"
                    | "agent" | "subtask" | "file",
                ) if !native => {}
                _ => turn.completeness = Completeness::Incomplete,
            }
        }
        flush_typed_text(turn, &mut fragments);
        Some(())
    }

    let mut native_seen = false;
    let mut classic_seen = false;
    let mut pending_native = BTreeSet::new();

    for (message_ordinal, message) in messages.iter().enumerate() {
        if deadline.is_some_and(|deadline| Instant::now() >= deadline) {
            return None;
        }
        if message.get("type").is_some() {
            native_seen = true;
            let kind = message.get("type").and_then(CanonValue::as_str);
            let id = message.get("id").and_then(CanonValue::as_str);
            let mut violation = id.is_none_or(|id| {
                id.is_empty()
                    || id.len() > 64
                    || !id
                        .chars()
                        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
            }) || message.get("info").is_some()
                || message.get("parts").is_some();
            let timestamp = message.get("time").and_then(|time| {
                timestamp_seconds(time.get("created"))
                    .or_else(|| timestamp_seconds(time.get("completed")))
            });
            match kind {
                Some("user") => {
                    let text = match message.get("text").and_then(CanonValue::as_str) {
                        Some(text) => {
                            let filtered = super::extract::strip_injection_prefixes(text);
                            violation |= filtered.malformed;
                            if filtered.removed && filtered.text.trim().is_empty() {
                                None
                            } else {
                                Some(filtered.text.into_owned())
                            }
                        }
                        None => {
                            violation = true;
                            Some(String::new())
                        }
                    };
                    violation |= !native_user_attachments_supported(message);
                    let turn = open_turn(&mut turns, id.map(str::to_owned));
                    if violation {
                        turn.completeness = Completeness::Incomplete;
                    }
                    if let Some(text) = text {
                        turn.records.push(SemanticRecord::User { text });
                    }
                    observe_turn_timestamp(turn, timestamp);
                    pending_native.insert(turn.ordinal);
                }
                Some("assistant") => {
                    let turn = current_or_open(&mut turns);
                    // Human-message keys remain stable when several prompts are steered
                    // in during one native busy→idle interval.
                    pending_native.insert(turn.ordinal);
                    if violation {
                        turn.completeness = Completeness::Incomplete;
                    }
                    if let Some(typed) = typed_parts.as_mut()
                        && typed
                            .range((message_ordinal, 0)..=(message_ordinal, usize::MAX))
                            .next()
                            .is_some()
                    {
                        match message.get("content").and_then(CanonValue::as_array) {
                            Some(content) => {
                                typed_message_parts(
                                    turn,
                                    content,
                                    message_ordinal,
                                    true,
                                    typed,
                                    deadline,
                                )?;
                            }
                            None => turn.completeness = Completeness::Incomplete,
                        }
                        observe_turn_timestamp(turn, timestamp);
                        continue;
                    }
                    match message.get("content").and_then(CanonValue::as_array) {
                        Some(content) => {
                            let mut fragments = Vec::new();
                            for part in content {
                                if deadline.is_some_and(|d| Instant::now() >= d) {
                                    return None;
                                }
                                match part.get("type").and_then(CanonValue::as_str) {
                                    Some("text") => {
                                        match part.get("text").and_then(CanonValue::as_str) {
                                            Some(text) => fragments.push(text),
                                            None => turn.completeness = Completeness::Incomplete,
                                        }
                                    }
                                    Some("tool") => {}
                                    // Reasoning has a separate verified-source owner. Its bytes
                                    // and provider state never enter this ordinary projection.
                                    _ => turn.completeness = Completeness::Incomplete,
                                }
                            }
                            // coverage-v1 joins all text blocks in one message; retain
                            // the same record order as the classic and live projections.
                            let text = fragments.join("\n");
                            if !text.is_empty() {
                                turn.records.push(SemanticRecord::Assistant { text });
                            }
                            for part in content {
                                if deadline.is_some_and(|d| Instant::now() >= d) {
                                    return None;
                                }
                                if part.get("type").and_then(CanonValue::as_str) == Some("tool") {
                                    native_tool(turn, part, deadline)?;
                                }
                            }
                        }
                        None => turn.completeness = Completeness::Incomplete,
                    }
                    observe_turn_timestamp(turn, timestamp);
                }
                Some("idle") => {
                    let valid_outcome = matches!(
                        message.get("outcome").and_then(CanonValue::as_str),
                        Some("succeeded" | "failed" | "interrupted")
                    );
                    if violation || !valid_outcome {
                        current_or_open(&mut turns).completeness = Completeness::Incomplete;
                    } else {
                        // Idle settles capture, not task success. It never upgrades a
                        // structurally incomplete turn; all human splits in this interval settle.
                        pending_native.clear();
                    }
                    if let Some(turn) = turns.last_mut() {
                        observe_turn_timestamp(turn, timestamp);
                    }
                }
                Some(
                    "agent-switched" | "model-switched" | "location-switched" | "synthetic"
                    | "system" | "skill" | "shell" | "compaction",
                ) if !violation => {}
                _ => {
                    let turn = current_or_open(&mut turns);
                    turn.completeness = Completeness::Incomplete;
                    pending_native.insert(turn.ordinal);
                }
            }
            continue;
        }
        classic_seen = true;
        // Structural validation: a message must be an object with an object
        // `info` carrying a string `role`, and an array `parts`. Anything
        // else poisons the enclosing turn instead of defaulting away
        // (Codex M3 R1 P1-5).
        let mut structural_violation = false;
        let info = message.get("info");
        if !matches!(message, CanonValue::Object(_)) || !matches!(info, Some(CanonValue::Object(_)))
        {
            structural_violation = true;
        }
        let role = match info.and_then(|i| i.get("role")) {
            Some(CanonValue::Str(role)) => role.as_str(),
            _ => {
                structural_violation = true;
                ""
            }
        };
        let msg_id = match info.and_then(|i| i.get("id")) {
            Some(CanonValue::Str(id)) => Some(id.clone()),
            Some(CanonValue::Null) | None => None,
            Some(_) => {
                structural_violation = true;
                None
            }
        };
        let message_timestamp = info.and_then(|info| info.get("time")).and_then(|time| {
            timestamp_seconds(time.get("created"))
                .or_else(|| timestamp_seconds(time.get("updated")))
        });
        let parts: &[CanonValue] = match message.get("parts") {
            Some(CanonValue::Array(parts)) => parts,
            _ => {
                structural_violation = true;
                &[]
            }
        };
        if role == "assistant"
            && let Some(typed) = typed_parts.as_mut()
            && typed
                .range((message_ordinal, 0)..=(message_ordinal, usize::MAX))
                .next()
                .is_some()
        {
            let turn = current_or_open(&mut turns);
            if structural_violation {
                turn.completeness = Completeness::Incomplete;
            }
            typed_message_parts(turn, parts, message_ordinal, false, typed, deadline)?;
            observe_turn_timestamp(turn, message_timestamp);
            continue;
        }
        // A structural violation poisons THIS message's own turn (Codex M3 R2
        // P1-3), never a neighbor — fold it into the per-turn `violation` flag
        // that is applied at role dispatch below.
        let mut text_fragments: Vec<&str> = Vec::new();
        let mut violation = structural_violation;
        for part in parts {
            // A part that is not even an object is itself malformed — poison
            // the turn rather than silently skipping it (Codex M3 R2 P1-3).
            if !matches!(part, CanonValue::Object(_)) {
                violation = true;
                continue;
            }
            match part.get("type").and_then(CanonValue::as_str) {
                Some("text") => match part.get("text") {
                    Some(CanonValue::Str(t)) => text_fragments.push(t),
                    Some(_) | None => violation = true,
                },
                Some("tool") => violation |= role != "assistant",
                Some(
                    "step-finish" | "patch" | "step-start" | "snapshot" | "retry" | "compaction"
                    | "agent" | "subtask" | "file",
                ) => {}
                _ => violation = true,
            }
        }
        let text = text_fragments.join("\n");

        match role {
            "user" => {
                let filtered = super::extract::strip_injection_prefixes(&text);
                let turn = open_turn(&mut turns, msg_id);
                if violation || filtered.malformed {
                    turn.completeness = Completeness::Incomplete;
                }
                if !filtered.removed || !filtered.text.trim().is_empty() {
                    turn.records.push(SemanticRecord::User {
                        text: filtered.text.into_owned(),
                    });
                }
            }
            "assistant" => {
                let turn = current_or_open(&mut turns);
                if violation {
                    turn.completeness = Completeness::Incomplete;
                }
                if !text.is_empty() {
                    turn.records.push(SemanticRecord::Assistant { text });
                }
                for part in parts {
                    if part.get("type").and_then(CanonValue::as_str) == Some("tool") {
                        classic_tool(current_or_open(&mut turns), part);
                    }
                }
            }
            // Unreadable / unsupported role. A structural violation must still
            // surface as an incomplete turn of its OWN (Codex M3 R2 P1-3)
            // rather than vanishing into a neighbor; a clean unknown role is
            // simply skipped.
            _ => {
                if violation {
                    open_turn(&mut turns, msg_id).completeness = Completeness::Incomplete;
                }
            }
        }
        if let Some(turn) = turns.last_mut() {
            observe_turn_timestamp(turn, message_timestamp);
        }
    }

    if typed_parts.as_ref().is_some_and(|parts| !parts.is_empty()) {
        return None;
    }

    for index in pending_native {
        if let Some(turn) = turns.get_mut(index) {
            turn.completeness = Completeness::Incomplete;
        }
    }
    if native_seen && classic_seen {
        for turn in &mut turns {
            turn.completeness = Completeness::Incomplete;
        }
    }

    turns.retain(|turn| !turn.records.is_empty() || turn.completeness == Completeness::Incomplete);
    for (i, turn) in turns.iter_mut().enumerate() {
        if turn.ordinal != i {
            if turn.logical_turn_key == ordinal_key(turn.ordinal) {
                turn.logical_turn_key = ordinal_key(i);
            }
            turn.ordinal = i;
        }
    }
    make_logical_turn_keys_unique(&mut turns);
    Some(turns)
}

/// Redact every allowlisted string field of the normalized turns IN PLACE —
/// coverage-v1.md §1 pins the order: typed normalize → **typed-field redact**
/// → canonicalize/digest. Digests are therefore always computed over
/// secret-free content, and every path (live/import/export) applying the
/// same default redactor reproduces the same digest.
pub fn redact_turns(turns: &mut [NormalizedTurn]) {
    let _ = redact_turns_with_report(turns);
}

pub(crate) fn merge_redaction_reports(
    aggregate: &mut crate::internal::ai::observed_agents::RedactionReport,
    report: crate::internal::ai::observed_agents::RedactionReport,
) {
    aggregate.bytes_scanned = aggregate.bytes_scanned.saturating_add(report.bytes_scanned);
    aggregate.bytes_redacted = aggregate
        .bytes_redacted
        .saturating_add(report.bytes_redacted);
    aggregate.dropped_matches = aggregate
        .dropped_matches
        .saturating_add(report.dropped_matches);
    for matched in report.matches {
        if aggregate.matches.len()
            < crate::internal::ai::observed_agents::MAX_REDACTION_MATCH_SAMPLES
        {
            aggregate.matches.push(matched);
        } else {
            aggregate.dropped_matches = aggregate.dropped_matches.saturating_add(1);
        }
    }
}

fn redact_string(
    s: &mut String,
    redactor: &crate::internal::ai::observed_agents::Redactor,
    aggregate: &mut crate::internal::ai::observed_agents::RedactionReport,
) {
    let (bytes, report) = redactor.redact(s.as_bytes());
    *s = String::from_utf8_lossy(bytes.as_ref()).into_owned();
    merge_redaction_reports(aggregate, report);
}

pub(crate) fn redact_canon_value(
    value: &mut CanonValue,
    redactor: &crate::internal::ai::observed_agents::Redactor,
    aggregate: &mut crate::internal::ai::observed_agents::RedactionReport,
) {
    match value {
        CanonValue::Str(s) => redact_string(s, redactor, aggregate),
        CanonValue::Array(items) => {
            for item in items {
                redact_canon_value(item, redactor, aggregate);
            }
        }
        CanonValue::Object(map) => {
            let mut redacted = BTreeMap::new();
            for (original_key, mut item) in std::mem::take(map) {
                redact_canon_value(&mut item, redactor, aggregate);
                let (_, key_report) = redactor.redact(original_key.as_bytes());
                let key_matched = !key_report.matches.is_empty();
                merge_redaction_reports(aggregate, key_report);
                let original_digest = hex::encode(Sha256::digest(original_key.as_bytes()));
                let mut output_key = if key_matched {
                    format!("redacted-key-sha256-{original_digest}")
                } else {
                    original_key
                };
                if redacted.contains_key(&output_key) {
                    let collision_base = format!("redacted-collision-key-sha256-{original_digest}");
                    output_key = collision_base.clone();
                    let mut ordinal = 0_u64;
                    while redacted.contains_key(&output_key) {
                        ordinal += 1;
                        output_key = format!("{collision_base}-{ordinal}");
                    }
                }
                redacted.insert(output_key, item);
            }
            *map = redacted;
        }
        _ => {}
    }
}

/// Redact normalized turns and return the aggregate typed-field evidence for
/// persistence by audit-sensitive callers such as historical import.
pub fn redact_turns_with_report(
    turns: &mut [NormalizedTurn],
) -> crate::internal::ai::observed_agents::RedactionReport {
    let redactor = crate::internal::ai::observed_agents::Redactor::new_default();
    let mut aggregate = crate::internal::ai::observed_agents::RedactionReport::default();
    for turn in turns {
        let (_, key_report) = redactor.redact(turn.logical_turn_key.as_bytes());
        if !key_report.matches.is_empty() {
            let digest = hex::encode(Sha256::digest(turn.logical_turn_key.as_bytes()));
            turn.logical_turn_key = format!("redacted-id-sha256-{digest}");
            merge_redaction_reports(&mut aggregate, key_report);
        } else {
            merge_redaction_reports(&mut aggregate, key_report);
        }
        for record in &mut turn.records {
            match record {
                SemanticRecord::User { text } | SemanticRecord::Assistant { text } => {
                    redact_string(text, &redactor, &mut aggregate);
                }
                SemanticRecord::ToolCall {
                    call_id,
                    input,
                    name,
                } => {
                    if let Some(call_id) = call_id {
                        redact_string(call_id, &redactor, &mut aggregate);
                    }
                    redact_canon_value(input, &redactor, &mut aggregate);
                    redact_string(name, &redactor, &mut aggregate);
                }
                SemanticRecord::ToolResult {
                    call_id, content, ..
                } => {
                    if let Some(call_id) = call_id {
                        redact_string(call_id, &redactor, &mut aggregate);
                    }
                    redact_string(content, &redactor, &mut aggregate);
                }
                SemanticRecord::Reasoning { text, .. } => {
                    // OG-11/RG-04: canary hits in readable reasoning count
                    // toward the report; the redaction rewrites the gated
                    // text in place (no arbitrary-text minting route).
                    let report = text.redact_with(&redactor);
                    merge_redaction_reports(&mut aggregate, report);
                }
            }
        }
    }
    aggregate
}

fn canon_to_json(value: &CanonValue) -> serde_json::Value {
    match value {
        CanonValue::Null | CanonValue::Float(_) => serde_json::Value::Null,
        CanonValue::Bool(value) => serde_json::Value::Bool(*value),
        CanonValue::Int(value) => serde_json::Value::Number((*value).into()),
        CanonValue::Str(value) => serde_json::Value::String(value.clone()),
        CanonValue::Array(values) => {
            serde_json::Value::Array(values.iter().map(canon_to_json).collect())
        }
        CanonValue::Object(values) => serde_json::Value::Object(
            values
                .iter()
                .map(|(key, value)| (key.clone(), canon_to_json(value)))
                .collect(),
        ),
    }
}

/// Serialize one already-normalized/redacted turn into the provider-tagged,
/// allowlist-only import projection required by ADR-DR-04/12. Unknown source
/// fields cannot enter this shape because it is rebuilt solely from the typed
/// coverage model.
pub fn safe_turn_projection(agent_kind: &str, turn: &NormalizedTurn) -> serde_json::Value {
    project_turn_for_schema(agent_kind, turn, COVERAGE_SCHEMA_VERSION)
}

/// Build an allowlist-only projection for an explicitly validated coverage
/// version. Unlike the frozen legacy helper, this rejects typed Reasoning in
/// v1 and never silently downgrades an unknown version.
pub fn safe_turn_projection_for_schema(
    version: i64,
    agent_kind: &str,
    turn: &NormalizedTurn,
) -> Result<serde_json::Value, CoverageCodecError> {
    let schema_version = CoverageSchemaVersion::try_from(version)?;
    if schema_version == CoverageSchemaVersion::V1
        && turn
            .records
            .iter()
            .any(|record| matches!(record, SemanticRecord::Reasoning { .. }))
    {
        return Err(CoverageCodecError::ReasoningRequiresV2);
    }
    Ok(project_turn_for_schema(
        agent_kind,
        turn,
        schema_version.as_i64(),
    ))
}

fn project_turn_for_schema(
    agent_kind: &str,
    turn: &NormalizedTurn,
    schema_version: i64,
) -> serde_json::Value {
    let records = turn
        .records
        .iter()
        .map(|record| match record {
            SemanticRecord::User { text } => serde_json::json!({
                "type": "user", "text": text,
            }),
            SemanticRecord::Assistant { text } => serde_json::json!({
                "type": "assistant", "text": text,
            }),
            SemanticRecord::ToolCall {
                call_id,
                input,
                name,
            } => serde_json::json!({
                "type": "tool_call", "call_id": call_id,
                "name": name, "input": canon_to_json(input),
            }),
            SemanticRecord::ToolResult {
                call_id,
                content,
                is_error,
            } => serde_json::json!({
                "type": if *is_error { "tool_error" } else { "tool_result" },
                "call_id": call_id, "content": content,
            }),
            SemanticRecord::Reasoning {
                provider,
                source_kind,
                text,
            } => serde_json::json!({
                "type": "reasoning",
                "provider": reasoning_provider_tag(*provider),
                "source_kind": source_kind.map(reasoning_source_kind_tag),
                "text": text.as_str(),
            }),
        })
        .collect::<Vec<_>>();
    serde_json::json!({
        "schema_version": schema_version,
        "agent_kind": agent_kind,
        "logical_turn_key": turn.logical_turn_key,
        "ordinal": turn.ordinal,
        "completeness": turn.completeness.as_db_str(),
        "records": records,
    })
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::*;

    #[test]
    fn og26_bounded_ordinary_writer_rejects_expansion() {
        let mut writer = BoundedOrdinaryWriter {
            bytes: Zeroizing::new(Vec::new()),
            cap: 4,
        };
        writer.write_all(b"safe").expect("exact output cap");
        assert!(writer.write_all(b"extra").is_err());
        assert_eq!(writer.into_bytes(), b"safe");
    }

    fn cov2_visible_turn(text: &str) -> NormalizedTurn {
        let (_, visible_text) = reasoning::classify_provider_visible_reasoning(
            reasoning::ReasoningProvider::OpenCode,
            reasoning::ReasoningSourceKind::Reasoning,
            text.to_owned(),
        );
        NormalizedTurn {
            logical_turn_key: "synthetic-turn".to_owned(),
            ordinal: 0,
            completeness: Completeness::Complete,
            started_at: None,
            ended_at: None,
            records: vec![
                SemanticRecord::User {
                    text: "hi".to_owned(),
                },
                SemanticRecord::Reasoning {
                    provider: reasoning::ReasoningProvider::OpenCode,
                    source_kind: Some(reasoning::ReasoningSourceKind::Reasoning),
                    text: visible_text,
                },
                SemanticRecord::Assistant {
                    text: "hello".to_owned(),
                },
            ],
        }
    }

    fn user_hi_assistant_hello() -> Vec<SemanticRecord> {
        vec![
            SemanticRecord::User {
                text: "hi".to_string(),
            },
            SemanticRecord::Assistant {
                text: "hello".to_string(),
            },
        ]
    }

    #[test]
    fn cov2_selects_from_typed_turn() {
        let visible = cov2_visible_turn("synthetic");
        assert_eq!(select_coverage_schema(&visible), CoverageSchemaVersion::V2);
        let encoded = encode_coverage_turn(&visible).expect("v2 typed turn encodes");
        assert_eq!(encoded.schema_version.as_i64(), 2);
        let projection = safe_turn_projection_for_schema(2, "opencode", &visible)
            .expect("v2 projection encodes");
        assert_eq!(projection["schema_version"], 2);
        assert_eq!(projection["records"][1]["type"], "reasoning");

        let plain = NormalizedTurn {
            logical_turn_key: "plain-turn".to_owned(),
            ordinal: 0,
            completeness: Completeness::Complete,
            started_at: None,
            ended_at: None,
            records: user_hi_assistant_hello(),
        };
        assert_eq!(select_coverage_schema(&plain), CoverageSchemaVersion::V1);
        let v1 = encode_coverage_turn(&plain).expect("plain turn encodes as v1");
        assert_eq!(v1.schema_version.as_i64(), 1);
        assert_eq!(v1.canonical_bytes, canonical_turn_bytes(&plain.records));
        assert_eq!(v1.digest_hex, plain.digest_hex());
        assert_eq!(
            safe_turn_projection_for_schema(1, "opencode", &plain).expect("plain v1 projection"),
            safe_turn_projection("opencode", &plain)
        );

        for unsupported in [0, -1, 3, i64::MAX] {
            assert!(matches!(
                encode_coverage_turn_for_schema(unsupported, &visible),
                Err(CoverageCodecError::UnsupportedVersion(value)) if value == unsupported
            ));
            assert!(matches!(
                safe_turn_projection_for_schema(unsupported, "opencode", &visible),
                Err(CoverageCodecError::UnsupportedVersion(value)) if value == unsupported
            ));
        }
        assert!(matches!(
            encode_coverage_turn_for_schema(1, &visible),
            Err(CoverageCodecError::ReasoningRequiresV2)
        ));
        assert!(matches!(
            safe_turn_projection_for_schema(1, "opencode", &visible),
            Err(CoverageCodecError::ReasoningRequiresV2)
        ));
        assert_eq!(
            CoverageCodecError::UnsupportedVersion(3).to_string(),
            "unsupported coverage schema version 3; supported versions are 1 and 2"
        );
        assert_eq!(
            CoverageCodecError::ReasoningRequiresV2.to_string(),
            "coverage schema v1 cannot encode typed reasoning; use schema v2"
        );
        assert!(
            !CoverageCodecError::ReasoningRequiresV2
                .to_string()
                .contains("synthetic")
        );
    }

    #[test]
    fn cov2_closed_reasoning_golden() {
        let golden: serde_json::Value = serde_json::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/agent_transcripts/reasoning/coverage_v2_golden.json"
        )))
        .expect("independent v2 golden is valid JSON");
        let mut visible = cov2_visible_turn("synthetic");
        let encoded = encode_coverage_turn(&visible).expect("synthetic v2 turn encodes");
        assert_eq!(golden["schema_version"], encoded.schema_version.as_i64());
        assert_eq!(golden["canonical_byte_len"], encoded.canonical_bytes.len());
        assert_eq!(
            encoded.canonical_bytes,
            golden["canonical_utf8"]
                .as_str()
                .expect("golden canonical bytes")
                .as_bytes()
        );
        assert_eq!(
            encoded.digest_hex,
            golden["sha256"].as_str().expect("golden sha256")
        );
        let canonical: serde_json::Value =
            serde_json::from_slice(&encoded.canonical_bytes).expect("canonical JSON parses");
        let reasoning = canonical[1].as_object().expect("reasoning record object");
        assert_eq!(
            reasoning.keys().map(String::as_str).collect::<Vec<_>>(),
            ["provider", "role", "source_kind", "text"]
        );
        assert_eq!(reasoning["role"], "reasoning");
        assert_eq!(reasoning["provider"], "opencode");
        assert_eq!(reasoning["source_kind"], "reasoning");
        assert_eq!(reasoning["text"], "synthetic");

        visible.started_at = Some(123);
        visible.ended_at = Some(456);
        visible.logical_turn_key = "another-provenance-key".to_owned();
        let changed_provenance =
            encode_coverage_turn(&visible).expect("provenance-only edit encodes");
        assert_eq!(changed_provenance.canonical_bytes, encoded.canonical_bytes);
        assert_eq!(changed_provenance.digest_hex, encoded.digest_hex);

        let mut with_secret =
            cov2_visible_turn("reasoning sk-ant-EXAMPLE0000000000000000 should be redacted");
        redact_turns_with_report(std::slice::from_mut(&mut with_secret));
        let redacted = encode_coverage_turn(&with_secret).expect("redacted v2 turn encodes");
        let redacted_projection = safe_turn_projection_for_schema(2, "opencode", &with_secret)
            .expect("redacted v2 projection encodes");
        assert!(!String::from_utf8_lossy(&redacted.canonical_bytes).contains("sk-ant-"));
        assert!(!redacted_projection.to_string().contains("sk-ant-"));
    }

    // Frozen dcc468b v1 cases. Expected bytes and digests below are copied
    // from coverage-v1.md §5, not regenerated by the versioned codec.
    fn cov2_frozen_v1_cases() -> [NormalizedTurn; 3] {
        let turn = |key: &str, records: Vec<SemanticRecord>| NormalizedTurn {
            logical_turn_key: key.to_owned(),
            ordinal: 0,
            completeness: Completeness::Complete,
            started_at: None,
            ended_at: None,
            records,
        };
        [
            turn(
                "frozen-v1-1",
                vec![
                    SemanticRecord::User {
                        text: "hi".to_owned(),
                    },
                    SemanticRecord::Assistant {
                        text: "hello".to_owned(),
                    },
                ],
            ),
            turn(
                "frozen-v1-2",
                vec![
                    SemanticRecord::ToolCall {
                        call_id: Some("c1".to_owned()),
                        input: parse_canon_value(br#"{"b":2,"a":"x"}"#)
                            .expect("frozen unsorted input is valid"),
                        name: "grep".to_owned(),
                    },
                    SemanticRecord::ToolResult {
                        call_id: Some("c1".to_owned()),
                        content: "2 matches".to_owned(),
                        is_error: false,
                    },
                ],
            ),
            turn(
                "frozen-v1-3",
                vec![SemanticRecord::User {
                    text: "line1\nline2 \"quoted\" \\ 中文".to_owned(),
                }],
            ),
        ]
    }

    #[test]
    fn cov2_v1_vector_bytes_frozen() {
        let expected: [&[u8]; 3] = [
            br#"[{"role":"user","text":"hi"},{"role":"assistant","text":"hello"}]"#,
            br#"[{"call_id":"c1","input":{"a":"x","b":2},"name":"grep","role":"tool_call"},{"call_id":"c1","content":"2 matches","is_error":false,"role":"tool_result"}]"#,
            r#"[{"role":"user","text":"line1\nline2 \"quoted\" \\ 中文"}]"#.as_bytes(),
        ];
        for (index, (turn, expected)) in
            cov2_frozen_v1_cases().into_iter().zip(expected).enumerate()
        {
            assert_eq!(select_coverage_schema(&turn), CoverageSchemaVersion::V1);
            let encoded = encode_coverage_turn_for_schema(1, &turn)
                .expect("frozen ordinary turn stays encodable as v1");
            assert_eq!(encoded.schema_version, CoverageSchemaVersion::V1);
            assert_eq!(
                encoded.canonical_bytes.as_slice(),
                expected,
                "v1 vector {}",
                index + 1
            );
            assert_eq!(
                canonical_turn_bytes(&turn.records).as_slice(),
                expected,
                "legacy v1 vector {}",
                index + 1
            );
        }
    }

    #[test]
    fn cov2_v1_vector_digests_frozen() {
        let expected = [
            "df991cd9a1ac5c12c32b8cdf0254c3dfbbf26485b505a5afc83a90d1128ebc54",
            "af085109b1584fd77bb3b752c1141c1c0f0f12b51a163e94b0f91ddd329460a4",
            "f1e76bf75df5d6b0f67a46806abb256cd6de30eaea30e45254a8a66cf5183356",
        ];
        for (index, (turn, expected)) in
            cov2_frozen_v1_cases().into_iter().zip(expected).enumerate()
        {
            let encoded = encode_coverage_turn_for_schema(1, &turn)
                .expect("frozen ordinary turn stays encodable as v1");
            assert_eq!(encoded.digest_hex, expected, "v1 vector {}", index + 1);
            assert_eq!(
                turn.digest_hex(),
                expected,
                "legacy v1 vector {}",
                index + 1
            );
        }
    }

    /// coverage-v1.md §5 vector 1 — canonical bytes and digest are normative.
    #[test]
    fn golden_vector_1_user_assistant() {
        let records = user_hi_assistant_hello();
        assert_eq!(
            canonical_turn_bytes(&records),
            br#"[{"role":"user","text":"hi"},{"role":"assistant","text":"hello"}]"#.to_vec()
        );
        assert_eq!(
            coverage_digest_hex(&records),
            "df991cd9a1ac5c12c32b8cdf0254c3dfbbf26485b505a5afc83a90d1128ebc54"
        );
    }

    /// coverage-v1.md §5 vector 2 — unsorted source keys canonicalize sorted.
    #[test]
    fn golden_vector_2_tool_call_and_result() {
        let input = parse_canon_value(br#"{"b":2,"a":"x"}"#).expect("strict parse");
        let records = vec![
            SemanticRecord::ToolCall {
                call_id: Some("c1".to_string()),
                input,
                name: "grep".to_string(),
            },
            SemanticRecord::ToolResult {
                call_id: Some("c1".to_string()),
                content: "2 matches".to_string(),
                is_error: false,
            },
        ];
        assert_eq!(
            canonical_turn_bytes(&records),
            br#"[{"call_id":"c1","input":{"a":"x","b":2},"name":"grep","role":"tool_call"},{"call_id":"c1","content":"2 matches","is_error":false,"role":"tool_result"}]"#.to_vec()
        );
        assert_eq!(
            coverage_digest_hex(&records),
            "af085109b1584fd77bb3b752c1141c1c0f0f12b51a163e94b0f91ddd329460a4"
        );
    }

    /// coverage-v1.md §5 vector 3 — escapes and raw UTF-8.
    #[test]
    fn golden_vector_3_escapes_and_unicode() {
        let records = vec![SemanticRecord::User {
            text: "line1\nline2 \"quoted\" \\ 中文".to_string(),
        }];
        assert_eq!(
            canonical_turn_bytes(&records),
            "[{\"role\":\"user\",\"text\":\"line1\\nline2 \\\"quoted\\\" \\\\ 中文\"}]".as_bytes()
        );
        assert_eq!(
            coverage_digest_hex(&records),
            "f1e76bf75df5d6b0f67a46806abb256cd6de30eaea30e45254a8a66cf5183356"
        );
    }

    /// M4 DR-05a: Codex historical rollout and Claude live capture must feed
    /// the same coverage-v1 digest for the same semantic turn. Event mirrors
    /// and unknown provenance fields must not duplicate or contaminate it.
    #[test]
    fn codex_rollout_matches_claude_digest_and_drops_unknown_fields() {
        let rollout = concat!(
            r#"{"type":"event_msg","payload":{"type":"task_started","turn_id":"turn-1"}}"#,
            "\n",
            r#"{"type":"event_msg","payload":{"type":"user_message","message":"mirrored","private":"AKIAAAAAAAAAAAAAAAAA"}}"#,
            "\n",
            r#"{"type":"response_item","payload":{"type":"message","role":"user","content":[{"type":"input_text","text":"hi","unknown":"drop-me"}],"provider_only":"drop-me"}}"#,
            "\n",
            r#"{"type":"response_item","payload":{"type":"message","role":"assistant","content":[{"type":"output_text","text":"hello"}],"encrypted_content":"drop-me"}}"#,
        );
        let turns = normalize_codex_rollout(rollout.as_bytes());
        assert_eq!(turns.len(), 1);
        assert_eq!(turns[0].logical_turn_key, "turn-1");
        assert_eq!(turns[0].completeness, Completeness::Complete);
        assert_eq!(
            turns[0].digest_hex(),
            "df991cd9a1ac5c12c32b8cdf0254c3dfbbf26485b505a5afc83a90d1128ebc54"
        );

        let projection = safe_turn_projection("codex", &turns[0]).to_string();
        assert!(!projection.contains("unknown"));
        assert!(!projection.contains("provider_only"));
        assert!(!projection.contains("encrypted_content"));
        assert!(!projection.contains("AKIAAAAAAAAAAAAAAAAA"));
    }

    /// Wrong-typed Codex semantic fields poison only their own turn. This is
    /// the historical-import equivalent of the live Claude fail-closed guard.
    #[test]
    fn codex_rollout_wrong_typed_semantics_fail_turn_closed() {
        let rollout = concat!(
            r#"{"type":"response_item","payload":{"type":"message","role":"user","content":"clean"}}"#,
            "\n",
            r#"{"type":"response_item","payload":{"type":"message","role":"assistant","content":[{"type":"output_text","text":7}]}}"#,
        );
        let turns = normalize_codex_rollout(rollout.as_bytes());
        assert_eq!(turns.len(), 1);
        assert_eq!(turns[0].completeness, Completeness::Incomplete);
    }

    /// Codex M1 R2 blocker 1: wrong-typed SEMANTIC fields (assistant text
    /// block carrying a number, tool_result is_error as a string, wrong-typed
    /// user content) must mark the turn `incomplete`, never silently default
    /// to a complete digest.
    #[test]
    fn wrong_typed_semantic_fields_fail_the_turn_closed() {
        // Assistant text block whose payload is a float.
        let t = concat!(
            r#"{"type":"user","uuid":"u1","message":{"role":"user","content":"hi"}}"#,
            "\n",
            r#"{"type":"assistant","uuid":"a1","message":{"role":"assistant","content":[{"type":"text","text":1.5}]}}"#,
        );
        let turns = normalize_claude_transcript(t.as_bytes());
        assert_eq!(turns.len(), 1);
        assert_eq!(turns[0].completeness, Completeness::Incomplete);

        // tool_result is_error present but not a bool.
        let t = concat!(
            r#"{"type":"user","uuid":"u1","message":{"role":"user","content":"go"}}"#,
            "\n",
            r#"{"type":"assistant","uuid":"a1","message":{"role":"assistant","content":[{"type":"tool_use","id":"c1","name":"x","input":{}}]}}"#,
            "\n",
            r#"{"type":"user","uuid":"u2","message":{"role":"user","content":[{"type":"tool_result","tool_use_id":"c1","content":"ok","is_error":"yes"}]}}"#,
        );
        let turns = normalize_claude_transcript(t.as_bytes());
        assert_eq!(turns.len(), 1);
        assert_eq!(turns[0].completeness, Completeness::Incomplete);

        // Wrong-typed user content (number instead of text/blocks).
        let t = r#"{"type":"user","uuid":"u1","message":{"role":"user","content":42}}"#;
        let turns = normalize_claude_transcript(t.as_bytes());
        assert_eq!(turns.len(), 1);
        assert_eq!(turns[0].completeness, Completeness::Incomplete);

        // tool_use name wrong-typed.
        let t = concat!(
            r#"{"type":"user","uuid":"u1","message":{"role":"user","content":"go"}}"#,
            "\n",
            r#"{"type":"assistant","uuid":"a1","message":{"role":"assistant","content":[{"type":"tool_use","id":"c1","name":7,"input":{}}]}}"#,
        );
        let turns = normalize_claude_transcript(t.as_bytes());
        assert_eq!(turns[0].completeness, Completeness::Incomplete);
    }

    /// Codex M1 R2: redact-before-digest proof. Two transcripts identical
    /// except for DIFFERENT embedded secrets must redact to the same
    /// projection and therefore the same digest; and that digest must differ
    /// from the digest of the unredacted turns (the secret never reaches the
    /// digest input).
    #[test]
    fn different_secrets_redact_to_same_digest() {
        let make = |key: &str| {
            format!(
                concat!(
                    r#"{{"type":"user","uuid":"u1","message":{{"role":"user","content":"use {}"}}}}"#,
                    "\n",
                    r#"{{"type":"assistant","uuid":"a1","message":{{"role":"assistant","content":[{{"type":"tool_use","id":"{}","name":"{}","input":{{"key":"{}"}}}}]}}}}"#,
                    "\n",
                    r#"{{"type":"user","uuid":"u2","message":{{"role":"user","content":[{{"type":"tool_result","tool_use_id":"{}","content":"{}","is_error":false}}]}}}}"#,
                ),
                key, key, key, key, key, key
            )
        };
        // Two distinct AWS-style access key ids (redactor family AKIA…).
        let a = make("AKIAAAAAAAAAAAAAAAAA");
        let b = make("AKIABBBBBBBBBBBBBBBB");

        let mut turns_a = normalize_claude_transcript(a.as_bytes());
        let unredacted_digest = turns_a[0].digest_hex();
        redact_turns(&mut turns_a);
        let mut turns_b = normalize_claude_transcript(b.as_bytes());
        redact_turns(&mut turns_b);

        assert_eq!(
            turns_a[0].digest_hex(),
            turns_b[0].digest_hex(),
            "different secrets must redact to the same digest"
        );
        assert_ne!(
            turns_a[0].digest_hex(),
            unredacted_digest,
            "the secret must never reach the digest input"
        );
        // Every allowlisted string position is redacted: user content, tool
        // call id/name/input and tool-result id/content.
        let canonical = canonical_turn_bytes(&turns_a[0].records);
        let canonical = String::from_utf8(canonical).expect("canonical JSON is UTF-8");
        assert!(
            !canonical.contains("AKIAAAAAAAAAAAAAAAAA"),
            "got: {canonical}"
        );
    }

    /// DR-04b: the OpenCode export normalizer maps the probed 1.17.x shape
    /// (messages[].info.role/id + text/tool parts with state) into turns —
    /// and the SAME semantic content yields the SAME digest as the Claude
    /// path (shared_splitter cross-path contract, golden vector 1).
    #[test]
    fn opencode_export_normalizer_maps_turns_and_matches_cross_path_digest() {
        let export = br#"{
            "info": {"id": "ses_x", "directory": "/w"},
            "messages": [
                {"info": {"role": "user", "id": "msg_u1", "time": {"created": 1.5}},
                 "parts": [{"type": "text", "text": "hi"}]},
                {"info": {"role": "assistant", "id": "msg_a1", "model": "m"},
                 "parts": [{"type": "text", "text": "hello"}]}
            ]
        }"#;
        let turns = normalize_opencode_export(export);
        assert_eq!(turns.len(), 1);
        assert_eq!(turns[0].logical_turn_key, "msg_u1");
        assert_eq!(turns[0].completeness, Completeness::Complete);
        // Cross-path: identical semantic content == golden vector 1 digest,
        // which the Claude splitter also produces.
        assert_eq!(
            turns[0].digest_hex(),
            "df991cd9a1ac5c12c32b8cdf0254c3dfbbf26485b505a5afc83a90d1128ebc54"
        );

        // Tool part: completed tool yields call + result records; a running
        // tool marks the turn incomplete (upgradeable on the next export).
        let export = br#"{
            "info": {"id": "ses_x"},
            "messages": [
                {"info": {"role": "user", "id": "msg_u1"},
                 "parts": [{"type": "text", "text": "run"}]},
                {"info": {"role": "assistant", "id": "msg_a1"},
                 "parts": [{"type": "tool", "tool": "grep", "callID": "c1",
                            "state": {"status": "completed", "input": {"b": 2, "a": "x"},
                                      "output": "2 matches"}}]}
            ]
        }"#;
        let turns = normalize_opencode_export(export);
        assert_eq!(turns.len(), 1);
        assert_eq!(turns[0].records.len(), 3); // user, tool_call, tool_result
        assert_eq!(turns[0].completeness, Completeness::Complete);

        let running = br#"{
            "info": {"id": "ses_x"},
            "messages": [
                {"info": {"role": "user", "id": "msg_u1"},
                 "parts": [{"type": "text", "text": "run"}]},
                {"info": {"role": "assistant", "id": "msg_a1"},
                 "parts": [{"type": "tool", "tool": "grep", "callID": "c1",
                            "state": {"status": "running", "input": {}}}]}
            ]
        }"#;
        let turns = normalize_opencode_export(running);
        assert_eq!(turns[0].completeness, Completeness::Incomplete);

        // Truncated export document: one poisoned turn, never a panic.
        let turns = normalize_opencode_export(br#"{"info": {"id": "ses_x"}, "mess"#);
        assert_eq!(turns.len(), 1);
        assert_eq!(turns[0].completeness, Completeness::Incomplete);
    }

    /// Codex M3 R2 P1-3: a structurally malformed message poisons ITS OWN turn
    /// (never a healthy neighbor), and a malformed element inside an otherwise
    /// valid `parts` array poisons that turn too — no silent skip.
    #[test]
    fn opencode_normalizer_poisons_malformed_message_own_turn() {
        // A clean user turn followed by a user message with a non-string `id`:
        // the first turn stays Complete, the malformed message's own (new)
        // turn is Incomplete — not the reverse.
        let export = br#"{
            "info": {"id": "ses_x"},
            "messages": [
                {"info": {"role": "user", "id": "msg_ok"},
                 "parts": [{"type": "text", "text": "clean"}]},
                {"info": {"role": "user", "id": 7},
                 "parts": [{"type": "text", "text": "malformed id"}]}
            ]
        }"#;
        let turns = normalize_opencode_export(export);
        assert_eq!(turns.len(), 2);
        assert_eq!(turns[0].logical_turn_key, "msg_ok");
        assert_eq!(
            turns[0].completeness,
            Completeness::Complete,
            "healthy neighbor must stay complete"
        );
        assert_eq!(
            turns[1].completeness,
            Completeness::Incomplete,
            "malformed message's own turn must be poisoned"
        );

        // A non-object element inside a valid parts array poisons that turn.
        let export = br#"{
            "info": {"id": "ses_x"},
            "messages": [
                {"info": {"role": "user", "id": "msg_u"},
                 "parts": ["not-an-object", {"type": "text", "text": "ok"}]}
            ]
        }"#;
        let turns = normalize_opencode_export(export);
        assert_eq!(turns.len(), 1);
        assert_eq!(
            turns[0].completeness,
            Completeness::Incomplete,
            "malformed part entry must poison the turn"
        );
    }

    #[test]
    fn canon_parse_rejects_duplicate_keys() {
        assert!(parse_canon_value(br#"{"a":1,"a":2}"#).is_err());
    }

    #[test]
    fn canon_parse_tolerates_floats_as_provenance() {
        // Floats parse (they are legal in provenance positions) …
        let v = parse_canon_value(br#"{"a":1.5}"#).expect("floats parse");
        assert!(contains_float(&v));
        let big = parse_canon_value(br#"{"a":18446744073709551615}"#).expect("big ints parse");
        assert!(contains_float(&big));
    }

    /// A fractional number in a PROVENANCE position (line envelope, e.g. a
    /// summary timestamp) must NOT poison the turn; one in a SEMANTIC
    /// position (tool input) fails that turn closed to `incomplete`.
    #[test]
    fn float_positions_provenance_tolerated_semantic_fails_closed() {
        let transcript = [
            r#"{"type":"summary","timestamp":1.5,"summary":"ignored"}"#,
            r#"{"type":"user","uuid":"u1","message":{"role":"user","content":"hi"},"cost":0.25}"#,
            r#"{"type":"assistant","uuid":"a1","message":{"role":"assistant","content":[{"type":"text","text":"hello"}]},"usage":{"cost":1.5}}"#,
        ]
        .join("\n");
        let turns = normalize_claude_transcript(transcript.as_bytes());
        assert_eq!(turns.len(), 1);
        assert_eq!(
            turns[0].completeness,
            Completeness::Complete,
            "provenance floats must not poison the turn"
        );
        // Identical semantic content → golden vector 1 digest.
        assert_eq!(
            turns[0].digest_hex(),
            "df991cd9a1ac5c12c32b8cdf0254c3dfbbf26485b505a5afc83a90d1128ebc54"
        );

        let semantic_float = concat!(
            r#"{"type":"user","uuid":"u1","message":{"role":"user","content":"hi"}}"#,
            "\n",
            r#"{"type":"assistant","uuid":"a1","message":{"role":"assistant","content":[{"type":"tool_use","id":"c1","name":"calc","input":{"x":1.5}}]}}"#,
        );
        let turns = normalize_claude_transcript(semantic_float.as_bytes());
        assert_eq!(turns.len(), 1);
        assert_eq!(
            turns[0].completeness,
            Completeness::Incomplete,
            "semantic floats fail the turn closed"
        );
    }

    #[test]
    fn claude_splitter_groups_turns_and_attaches_tool_results() {
        let transcript = [
            r#"{"type":"user","uuid":"u1","message":{"role":"user","content":"run grep"}}"#,
            r#"{"type":"assistant","uuid":"a1","message":{"role":"assistant","content":[{"type":"text","text":"ok"},{"type":"tool_use","id":"c1","name":"grep","input":{"b":2,"a":"x"}}]}}"#,
            r#"{"type":"user","uuid":"u2","message":{"role":"user","content":[{"type":"tool_result","tool_use_id":"c1","content":"2 matches"}]}}"#,
            r#"{"type":"user","uuid":"u3","message":{"role":"user","content":"thanks"}}"#,
            r#"{"type":"assistant","uuid":"a2","message":{"role":"assistant","content":[{"type":"text","text":"any time"}]}}"#,
        ]
        .join("\n");
        let turns = normalize_claude_transcript(transcript.as_bytes());
        assert_eq!(turns.len(), 2);
        assert_eq!(turns[0].logical_turn_key, "u1");
        assert_eq!(turns[0].ordinal, 0);
        assert_eq!(turns[0].completeness, Completeness::Complete);
        assert_eq!(turns[0].records.len(), 4); // user, assistant text, tool call, tool result
        assert!(matches!(
            &turns[0].records[3],
            SemanticRecord::ToolResult { call_id: Some(id), content, is_error: false }
                if id == "c1" && content == "2 matches"
        ));
        assert_eq!(turns[1].logical_turn_key, "u3");
        assert_eq!(turns[1].ordinal, 1);
        assert_eq!(turns[1].records.len(), 2);
    }

    #[test]
    fn claude_splitter_same_content_same_digest_across_parses() {
        let transcript = concat!(
            r#"{"type":"user","uuid":"u1","message":{"role":"user","content":"hi"}}"#,
            "\n",
            r#"{"type":"assistant","uuid":"a1","message":{"role":"assistant","content":[{"type":"text","text":"hello"}]}}"#,
        );
        let a = normalize_claude_transcript(transcript.as_bytes());
        let b = normalize_claude_transcript(transcript.as_bytes());
        assert_eq!(a[0].digest_hex(), b[0].digest_hex());
        // And it matches golden vector 1 (same semantic content).
        assert_eq!(
            a[0].digest_hex(),
            "df991cd9a1ac5c12c32b8cdf0254c3dfbbf26485b505a5afc83a90d1128ebc54"
        );
    }

    #[test]
    fn claude_splitter_marks_corrupt_tail_incomplete() {
        let transcript = concat!(
            r#"{"type":"user","uuid":"u1","message":{"role":"user","content":"hi"}}"#,
            "\n",
            r#"{"type":"assistant","uuid":"a1","message":{"role":"assistant","content":[{"type":"te"#, // truncated
        );
        let turns = normalize_claude_transcript(transcript.as_bytes());
        assert_eq!(turns.len(), 1);
        assert_eq!(turns[0].completeness, Completeness::Incomplete);
    }

    #[test]
    fn claude_splitter_invalid_uuid_falls_back_to_ordinal() {
        let transcript =
            r#"{"type":"user","uuid":"has spaces !","message":{"role":"user","content":"hi"}}"#;
        let turns = normalize_claude_transcript(transcript.as_bytes());
        assert_eq!(turns[0].logical_turn_key, "ordinal:0");
    }

    #[test]
    fn truncated_and_complete_same_turn_share_logical_key_but_differ_in_digest() {
        // The whole point of ADR-DR-08: same logical key, different content
        // revision — never two separate turns.
        let truncated =
            r#"{"type":"user","uuid":"u1","message":{"role":"user","content":"hi"}}"#.to_string();
        let complete = format!(
            "{truncated}\n{}",
            r#"{"type":"assistant","uuid":"a1","message":{"role":"assistant","content":[{"type":"text","text":"hello"}]}}"#
        );
        let t = normalize_claude_transcript(truncated.as_bytes());
        let c = normalize_claude_transcript(complete.as_bytes());
        assert_eq!(t[0].logical_turn_key, c[0].logical_turn_key);
        assert_ne!(t[0].digest_hex(), c[0].digest_hex());
    }

    #[test]
    fn secret_shaped_provider_turn_ids_are_hashed_consistently_without_collision() {
        let secret_a = "AKIAIOSFODNN7EXAMPLE";
        let secret_b = "AKIAIOSFODNN7EXAMPLF";
        let claude = format!(
            "{}\n{}\n",
            serde_json::json!({
                "type": "user", "uuid": secret_a,
                "message": {"role": "user", "content": "hi"}
            }),
            serde_json::json!({
                "type": "assistant", "uuid": "answer",
                "message": {"role": "assistant", "content": "hello"}
            })
        );
        let codex = format!(
            "{}\n{}\n{}\n",
            serde_json::json!({
                "type": "event_msg",
                "payload": {"type": "turn_started", "turn_id": secret_a}
            }),
            serde_json::json!({
                "type": "response_item",
                "payload": {"type": "message", "role": "user", "content": "hi"}
            }),
            serde_json::json!({
                "type": "response_item",
                "payload": {"type": "message", "role": "assistant", "content": "hello"}
            })
        );
        let opencode = serde_json::json!({
            "messages": [
                {"info": {"role": "user", "id": secret_a},
                 "parts": [{"type": "text", "text": "hi"}]},
                {"info": {"role": "assistant", "id": "answer"},
                 "parts": [{"type": "text", "text": "hello"}]}
            ]
        })
        .to_string();
        let mut sources = [
            normalize_claude_transcript(claude.as_bytes()),
            normalize_codex_rollout(codex.as_bytes()),
            normalize_opencode_export(opencode.as_bytes()),
        ];
        for turns in &mut sources {
            let report = redact_turns_with_report(turns);
            assert!(report.bytes_redacted > 0);
            assert!(!report.matches.is_empty());
            assert!(!turns[0].logical_turn_key.contains(secret_a));
        }
        assert_eq!(
            sources[0][0].logical_turn_key,
            sources[1][0].logical_turn_key
        );
        assert_eq!(
            sources[0][0].logical_turn_key,
            sources[2][0].logical_turn_key
        );
        assert_eq!(sources[0][0].digest_hex(), sources[1][0].digest_hex());
        assert_eq!(sources[0][0].digest_hex(), sources[2][0].digest_hex());

        let other = format!(
            "{}\n",
            serde_json::json!({
                "type": "user", "uuid": secret_b,
                "message": {"role": "user", "content": "hi"}
            })
        );
        let mut other = normalize_claude_transcript(other.as_bytes());
        redact_turns(&mut other);
        assert_ne!(sources[0][0].logical_turn_key, other[0].logical_turn_key);
    }

    #[test]
    fn redaction_covers_secret_shaped_tool_input_keys_deterministically() {
        let secret = "AKIAIOSFODNN7EXAMPLE";
        let rollout = format!(
            "{}\n{}\n",
            serde_json::json!({
                "type": "response_item",
                "payload": {"type": "message", "role": "user", "content": "inspect"}
            }),
            serde_json::json!({
                "type": "response_item",
                "payload": {"type": "function_call", "name": "inspect",
                    "arguments": serde_json::json!({secret: "value"}).to_string()}
            })
        );
        let mut first = normalize_codex_rollout(rollout.as_bytes());
        let mut second = first.clone();
        let first_report = redact_turns_with_report(&mut first);
        let second_report = redact_turns_with_report(&mut second);
        let first_json = String::from_utf8(canonical_turn_bytes(&first[0].records))
            .expect("canonical coverage is UTF-8");

        assert!(!first_json.contains(secret));
        assert!(first_json.contains("redacted-key-sha256-"));
        assert!(first_report.bytes_redacted > 0);
        assert!(
            first_report
                .matches
                .iter()
                .any(|entry| entry.rule_id == "aws-access-key-id")
        );
        assert_eq!(first, second);
        assert_eq!(first_report, second_report);
    }

    #[test]
    fn aggregate_redaction_report_caps_samples_across_turns_and_keeps_exact_count() {
        const TOKEN: &str = "AKIAIOSFODNN7EXAMPLE";
        let per_field_overflow = 3;
        let aggregate_overflow = 5;
        let dense_secret_field = std::iter::repeat_n(
            TOKEN,
            crate::internal::ai::observed_agents::MAX_REDACTION_MATCH_SAMPLES
                .saturating_add(per_field_overflow),
        )
        .collect::<Vec<_>>()
        .join(" ");
        let mut turns = vec![NormalizedTurn {
            logical_turn_key: "dense-secret-turn".to_string(),
            ordinal: 0,
            completeness: Completeness::Complete,
            started_at: None,
            ended_at: None,
            records: vec![SemanticRecord::User {
                text: dense_secret_field,
            }],
        }];
        for ordinal in 1..=aggregate_overflow {
            turns.push(NormalizedTurn {
                logical_turn_key: format!("secret-turn-{ordinal}"),
                ordinal,
                completeness: Completeness::Complete,
                started_at: None,
                ended_at: None,
                records: vec![SemanticRecord::User {
                    text: format!("{TOKEN} record {ordinal}"),
                }],
            });
        }

        let report = redact_turns_with_report(&mut turns);
        assert_eq!(
            report.matches.len(),
            crate::internal::ai::observed_agents::MAX_REDACTION_MATCH_SAMPLES,
            "durable aggregate detail must remain globally bounded"
        );
        assert_eq!(
            report.dropped_matches,
            per_field_overflow + aggregate_overflow,
            "include both per-field and aggregate-level omitted samples"
        );
        assert_eq!(
            report.match_count(),
            crate::internal::ai::observed_agents::MAX_REDACTION_MATCH_SAMPLES
                .saturating_add(per_field_overflow)
                .saturating_add(aggregate_overflow),
            "the capped report must retain the exact redaction count"
        );
        assert!(report.matches_truncated());
        assert!(turns.iter().all(|turn| {
            turn.records.iter().all(|record| match record {
                SemanticRecord::User { text } => !text.contains(TOKEN),
                _ => true,
            })
        }));
    }

    #[test]
    fn codex_malformed_or_wrong_typed_tool_arguments_poison_the_turn() {
        for arguments in [
            serde_json::Value::String("{\"path\":\"truncated".to_string()),
            serde_json::json!({"path": "wrong wire type"}),
        ] {
            let rollout = format!(
                "{}\n{}\n",
                serde_json::json!({
                    "type": "response_item",
                    "payload": {"type": "message", "role": "user", "content": "inspect"}
                }),
                serde_json::json!({
                    "type": "response_item",
                    "payload": {"type": "function_call", "name": "inspect",
                        "arguments": arguments}
                })
            );
            let turns = normalize_codex_rollout(rollout.as_bytes());
            assert_eq!(turns.len(), 1);
            assert_eq!(turns[0].completeness, Completeness::Incomplete);
        }
    }

    #[test]
    fn codex_custom_tool_call_accepts_its_freeform_input_field() {
        let rollout = format!(
            "{}\n{}\n",
            serde_json::json!({
                "type": "response_item",
                "payload": {"type": "message", "role": "user", "content": "inspect"}
            }),
            serde_json::json!({
                "type": "response_item",
                "payload": {"type": "custom_tool_call", "name": "shell",
                    "call_id": "call-1", "input": "printf hello"}
            })
        );
        let turns = normalize_codex_rollout(rollout.as_bytes());
        assert_eq!(turns.len(), 1);
        assert_eq!(turns[0].completeness, Completeness::Complete);
        assert!(matches!(
            &turns[0].records[1],
            SemanticRecord::ToolCall {
                input: CanonValue::Str(input),
                ..
            } if input == "printf hello"
        ));
    }

    /// RG-04 readable-reasoning projection: canary in provider-visible
    /// reasoning text fires the anthropic rule in the report, the gated text
    /// is redacted in place, and the projection/canonical bytes stay clean.
    #[test]
    fn rg04_readable_reasoning_redaction_and_projection() {
        let (record, text) = reasoning::classify_provider_visible_reasoning(
            reasoning::ReasoningProvider::OpenCode,
            reasoning::ReasoningSourceKind::Reasoning,
            "readable reasoning with sk-ant-EXAMPLE0000000000000000 inside".to_string(),
        );
        assert_eq!(
            record.availability(),
            reasoning::ReasoningAvailability::ProviderVisible
        );
        assert!(record.warning().is_none());

        let mut turn = NormalizedTurn {
            logical_turn_key: "rg04-lib-turn".to_string(),
            ordinal: 0,
            completeness: Completeness::Complete,
            started_at: None,
            ended_at: None,
            records: vec![SemanticRecord::Reasoning {
                provider: reasoning::ReasoningProvider::OpenCode,
                source_kind: Some(reasoning::ReasoningSourceKind::Reasoning),
                text,
            }],
        };
        let report = redact_turns_with_report(std::slice::from_mut(&mut turn));
        assert!(
            report
                .matches
                .iter()
                .any(|hit| hit.rule_id == "anthropic-api-key"),
            "the anthropic rule must fire on reasoning text: {:?}",
            report.matches
        );
        let projection = safe_turn_projection("opencode", &turn);
        let rendered = serde_json::to_string(&projection).unwrap();
        assert!(!rendered.contains("sk-ant-"), "projection leak: {rendered}");
        assert_eq!(projection["records"][0]["type"], "reasoning");
        assert_eq!(projection["records"][0]["provider"], "opencode");

        let canonical = canonical_turn_bytes(&turn.records);
        assert!(
            !String::from_utf8_lossy(&canonical).contains("sk-ant-"),
            "canonical bytes must carry the redacted text only"
        );

        // AC 7 metadata sink: the serialized record tags (provider/source_kind)
        // must never carry the canary; the visible-reasoning warning is None.
        let metadata_tags = match &turn.records[0] {
            SemanticRecord::Reasoning {
                provider,
                source_kind,
                ..
            } => serde_json::to_string(&(provider, source_kind)).unwrap(),
            other => panic!("expected reasoning record, got {other:?}"),
        };
        assert!(
            !metadata_tags.contains("sk-ant-") && !metadata_tags.contains("CANARY"),
            "metadata tags must never carry reasoning text: {metadata_tags}"
        );
    }

    /// RG-04 fixture conformance (provider_visible half): the synthetic
    /// fixture's reasoning_text drives the classification path and the
    /// projected record matches the frozen canonical form.
    #[test]
    fn rg04_fixture_provider_visible_conformance() {
        let fixture = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/agent_transcripts/reasoning/provider_visible.json"
        ))
        .unwrap();
        let value: serde_json::Value = serde_json::from_str(&fixture).unwrap();
        let record_json = &value["records"][0];
        let text = record_json["reasoning_text"]
            .as_str()
            .expect("reasoning_text");
        let (record, gated) = reasoning::classify_provider_visible_reasoning(
            reasoning::ReasoningProvider::OpenCode,
            reasoning::ReasoningSourceKind::Reasoning,
            text.to_string(),
        );
        assert_eq!(
            record.availability(),
            reasoning::ReasoningAvailability::ProviderVisible
        );
        let mut turn = NormalizedTurn {
            logical_turn_key: "fixture-turn".to_string(),
            ordinal: 0,
            completeness: Completeness::Complete,
            started_at: None,
            ended_at: None,
            records: vec![SemanticRecord::Reasoning {
                provider: reasoning::ReasoningProvider::OpenCode,
                source_kind: Some(reasoning::ReasoningSourceKind::Reasoning),
                text: gated,
            }],
        };
        redact_turns_with_report(std::slice::from_mut(&mut turn));
        let projection = safe_turn_projection("opencode", &turn);
        assert_eq!(projection["records"][0]["source_kind"], "reasoning");
        assert_eq!(projection["records"][0]["provider"], "opencode");
        assert_eq!(projection["records"][0]["type"], "reasoning");
        // Codex re-review P1: the WHOLE projection must equal the frozen
        // contract.snap value (shape AND text), not just three fields.
        let snap = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/agent_transcripts/reasoning/contract.snap"
        ))
        .unwrap();
        let snap_value: serde_json::Value = serde_json::from_str(&snap).unwrap();
        assert_eq!(
            projection, snap_value["semantic_reasoning_projection"],
            "whole projection diverges from the frozen contract.snap"
        );
    }

    /// RG-04 AC 7 (tracing half): no tracing output emitted while a
    /// canary-bearing readable-reasoning record is redacted and projected may
    /// contain the canary — a regression that logs reasoning text would fail
    /// here (Codex re-review P1).
    #[test]
    fn rg04_tracing_never_emits_reasoning_text() {
        #[derive(Clone, Default)]
        struct Sink(std::sync::Arc<std::sync::Mutex<Vec<u8>>>);
        impl std::io::Write for Sink {
            fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
                self.0.lock().unwrap().extend_from_slice(buf);
                Ok(buf.len())
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }
        impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for Sink {
            type Writer = Sink;
            fn make_writer(&'a self) -> Self::Writer {
                self.clone()
            }
        }
        let sink = Sink::default();
        let subscriber = tracing_subscriber::fmt()
            .with_max_level(tracing::Level::TRACE)
            .with_ansi(false)
            .with_writer(sink.clone())
            .with_span_events(tracing_subscriber::fmt::format::FmtSpan::CLOSE)
            .finish();

        tracing::subscriber::with_default(subscriber, || {
            let (_, text) = reasoning::classify_provider_visible_reasoning(
                reasoning::ReasoningProvider::OpenCode,
                reasoning::ReasoningSourceKind::Reasoning,
                "reasoning with sk-ant-EXAMPLE0000000000000000 embedded".to_string(),
            );
            let mut turn = NormalizedTurn {
                logical_turn_key: "tracing-turn".to_string(),
                ordinal: 0,
                completeness: Completeness::Complete,
                started_at: None,
                ended_at: None,
                records: vec![SemanticRecord::Reasoning {
                    provider: reasoning::ReasoningProvider::OpenCode,
                    source_kind: Some(reasoning::ReasoningSourceKind::Reasoning),
                    text,
                }],
            };
            tracing::debug!(?turn, "before typed reasoning redaction");
            tracing::info!(logical_turn_key = %turn.logical_turn_key, "projecting turn");
            redact_turns_with_report(std::slice::from_mut(&mut turn));
            let _ = safe_turn_projection("opencode", &turn);
        });
        let captured_bytes = sink.0.lock().unwrap();
        let captured = String::from_utf8_lossy(&captured_bytes).to_string();
        assert!(captured.contains("before typed reasoning redaction"));
        assert!(captured.contains("projecting turn"));
        assert!(
            !captured.contains("sk-ant-"),
            "tracing must never carry reasoning text: {captured}"
        );
    }
}
