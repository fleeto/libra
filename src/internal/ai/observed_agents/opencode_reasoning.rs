//! OpenCode's bounded, pre-redaction reasoning classifier.
//!
//! The official export is an ordinary source. A same-named SDK metadata key
//! never proves native origin. This stage removes the closed opaque window
//! before the shared Redactor and deliberately discards unverified candidates.

use std::{
    collections::{BTreeMap, BTreeSet},
    ops::{Deref, DerefMut},
};

use zeroize::{Zeroize, Zeroizing};

use super::{
    coverage::{CanonValue, SemanticRecord, parse_canon_value},
    native_origin::codec::{FieldName, RECEIPT_BYTE_CAP},
    reasoning::{
        OPENCODE_CANON_MAX_NODES, OpenCodeCandidatePart, OpenCodeCoverageClassification,
        OpenCodeCoverageSource, OpenCodeVisibleReasoningPart, PreparedReasoningSource,
        ReasoningFieldError, ReasoningProvider, ReasoningSourceKind, SealedOpenCodeCandidates,
        classify_provider_visible_reasoning, prepared_source_is_ordinary_projection,
    },
    transcript_source::TRANSCRIPT_READ_HARD_CAP_BYTES,
};

const MAX_OBSERVED_IDS: usize = 100_000;
const MAX_JSON_DEPTH: usize = 64;
const MAX_VISIBLE_REASONING_PARTS: usize = 4096;

/// Classify the original, authorized in-memory export before the scrubber
/// erases state/proof fields. This seam has no capture/live/import callers yet. It
/// retains the existing non-authoritative prepared source for the later
/// typed-redaction stage; nothing here writes a claim, artifact or cursor.
#[allow(dead_code)] // COV2-11 alone activates this private parser seam.
pub(crate) fn classify_opencode_coverage_source(source: Vec<u8>) -> OpenCodeCoverageClassification {
    fn classify(
        mut source: Zeroizing<Vec<u8>>,
    ) -> Result<OpenCodeCoverageSource, ReasoningFieldError> {
        if source.len() as u64 > TRANSCRIPT_READ_HARD_CAP_BYTES {
            return Err(ReasoningFieldError::SourceTooLarge);
        }
        let (_, document) = parse_bounded_canon_value(&source, parse_canon_value)?;
        let document = Zeroizing::new(document);
        let source_session = document
            .get("info")
            .and_then(|info| info.get("id"))
            .and_then(CanonValue::as_str)
            .filter(|id| !id.is_empty() && id.len() <= 1024)
            .ok_or(ReasoningFieldError::UnsupportedShape)?;
        let messages = document
            .get("messages")
            .and_then(CanonValue::as_array)
            .ok_or(ReasoningFieldError::UnsupportedShape)?;
        let mut message_ids = BTreeMap::<&str, usize>::new();
        let mut part_ids = BTreeMap::<&str, usize>::new();
        for message in messages {
            let (_, id, parts) = coverage_message(message)?;
            if let Some(id) = id {
                *message_ids.entry(id).or_default() += 1;
            }
            if let Some(parts) = parts {
                for part in parts {
                    if let Some(id) = part.get("id").and_then(CanonValue::as_str) {
                        *part_ids.entry(id).or_default() += 1;
                    }
                }
            }
        }

        let mut partial = false;
        let mut visible_parts = Vec::new();
        for (message_ordinal, message) in messages.iter().enumerate() {
            let (kind, id, parts) = coverage_message(message)?;
            if kind != "assistant" {
                continue;
            }
            let Some(id) = id.filter(|id| !id.is_empty() && id.len() <= 1024) else {
                partial = true;
                continue;
            };
            if message_ids.get(id) != Some(&1) {
                partial = true;
                continue;
            }
            let Some(parts) = parts else {
                partial = true;
                continue;
            };
            for (part_ordinal, part) in parts.iter().enumerate() {
                let CanonValue::Object(fields) = part else {
                    partial = true;
                    continue;
                };
                if fields.get("type").and_then(CanonValue::as_str) != Some("reasoning") {
                    continue;
                }
                let part_id = fields
                    .get("id")
                    .and_then(CanonValue::as_str)
                    .filter(|id| !id.is_empty() && id.len() <= 1024);
                let text = fields
                    .get("text")
                    .and_then(CanonValue::as_str)
                    .filter(|text| !text.trim().is_empty());
                let (Some(part_id), Some(text)) = (part_id, text) else {
                    partial = true;
                    continue;
                };
                if part_ids.get(part_id) != Some(&1)
                    || !matches!(
                        classify_reasoning_fields(fields),
                        ReasoningPartShape::Visible
                    )
                    || fields
                        .get("sessionID")
                        .is_some_and(|value| value.as_str() != Some(source_session))
                    || fields
                        .get("messageID")
                        .is_some_and(|value| value.as_str() != Some(id))
                {
                    partial = true;
                    continue;
                }
                if visible_parts.len() >= MAX_VISIBLE_REASONING_PARTS {
                    return Err(ReasoningFieldError::SourceTooLarge);
                }
                let (_, text) = classify_provider_visible_reasoning(
                    ReasoningProvider::OpenCode,
                    ReasoningSourceKind::Reasoning,
                    text.to_owned(),
                );
                visible_parts.push(OpenCodeVisibleReasoningPart {
                    source_message: Zeroizing::new(id.to_owned()),
                    message_ordinal,
                    source_part: Zeroizing::new(part_id.to_owned()),
                    part_ordinal,
                    record: SemanticRecord::Reasoning {
                        provider: ReasoningProvider::OpenCode,
                        source_kind: Some(ReasoningSourceKind::Reasoning),
                        text,
                    },
                });
            }
        }
        // Reuse the reviewed closed export projection rather than inventing
        // another envelope allowlist. Its own bounded parser runs once more;
        // both passes share the existing source/node/depth caps. Candidates
        // remain sealed and unproven, and unknown envelope data stays partial.
        let source_session = Zeroizing::new(source_session.to_owned());
        drop(message_ids);
        drop(part_ids);
        drop(document);
        let prepared = prepare_authorized_source(std::mem::take(&mut *source))?;
        partial |= !prepared_source_is_ordinary_projection(&prepared);
        Ok(OpenCodeCoverageSource {
            source_session,
            prepared,
            visible_parts,
            partial,
        })
    }

    match classify(Zeroizing::new(source)) {
        Ok(source) => OpenCodeCoverageClassification::Parsed(Box::new(source)),
        Err(reason) => OpenCodeCoverageClassification::Partial(reason),
    }
}

/// Accept exactly one of the two already supported export message envelopes.
/// All returned coordinates borrow the bounded, wiping parse tree.
type CoverageMessageView<'a> = (&'a str, Option<&'a str>, Option<&'a [CanonValue]>);

fn coverage_message(message: &CanonValue) -> Result<CoverageMessageView<'_>, ReasoningFieldError> {
    let CanonValue::Object(fields) = message else {
        return Err(ReasoningFieldError::UnsupportedShape);
    };
    if let Some(kind) = fields.get("type") {
        if fields.contains_key("info") || fields.contains_key("parts") {
            return Err(ReasoningFieldError::UnsupportedShape);
        }
        let kind = kind.as_str().ok_or(ReasoningFieldError::UnsupportedShape)?;
        let parts = match fields.get("content") {
            None => None,
            Some(CanonValue::Array(parts)) => Some(parts.as_slice()),
            Some(_) => return Err(ReasoningFieldError::UnsupportedShape),
        };
        Ok((kind, fields.get("id").and_then(CanonValue::as_str), parts))
    } else {
        if fields.contains_key("content") {
            return Err(ReasoningFieldError::UnsupportedShape);
        }
        let info = fields
            .get("info")
            .ok_or(ReasoningFieldError::UnsupportedShape)?;
        let role = info
            .get("role")
            .and_then(CanonValue::as_str)
            .ok_or(ReasoningFieldError::UnsupportedShape)?;
        let parts = match fields.get("parts") {
            Some(CanonValue::Array(parts)) => Some(parts.as_slice()),
            _ => return Err(ReasoningFieldError::UnsupportedShape),
        };
        Ok((role, info.get("id").and_then(CanonValue::as_str), parts))
    }
}

/// Offsets refer to the *authorized raw export*, before either JSON decoding
/// or ordinary projection. A receipt is copied only after the strict parse has
/// confirmed the corresponding assistant part and closed candidate state.
struct ReceiptSpan {
    message_ordinal: usize,
    part_ordinal: usize,
    start: usize,
    end: usize,
}

#[derive(Clone, Copy)]
enum ScanLocation {
    Root,
    Messages,
    Message(usize),
    ContentParts(usize),
    Part(usize, usize),
    Other,
}

/// A small lexical structural preflight. It bounds container depth and total
/// JSON values *before* `CanonValue` materializes an adversarial tree. The
/// strict CanonValue parser remains the grammar and duplicate-key authority.
struct StructureScanner<'a> {
    input: &'a [u8],
    offset: usize,
    nodes: usize,
    receipts: Vec<ReceiptSpan>,
}

impl<'a> StructureScanner<'a> {
    fn scan(input: &'a [u8]) -> Result<Vec<ReceiptSpan>, ReasoningFieldError> {
        let mut scanner = Self {
            input,
            offset: 0,
            nodes: 0,
            receipts: Vec::new(),
        };
        scanner.value(ScanLocation::Root, 0)?;
        scanner.space();
        if scanner.offset != input.len() {
            return Err(ReasoningFieldError::MalformedSource);
        }
        Ok(scanner.receipts)
    }

    fn space(&mut self) {
        while self
            .input
            .get(self.offset)
            .is_some_and(|byte| matches!(byte, b' ' | b'\n' | b'\r' | b'\t'))
        {
            self.offset += 1;
        }
    }

    fn consume(&mut self, expected: u8) -> Result<(), ReasoningFieldError> {
        if self.input.get(self.offset) != Some(&expected) {
            return Err(ReasoningFieldError::MalformedSource);
        }
        self.offset += 1;
        Ok(())
    }

    fn value(&mut self, location: ScanLocation, depth: usize) -> Result<(), ReasoningFieldError> {
        if depth > MAX_JSON_DEPTH {
            return Err(ReasoningFieldError::SourceTooLarge);
        }
        self.nodes = self
            .nodes
            .checked_add(1)
            .filter(|count| *count <= OPENCODE_CANON_MAX_NODES)
            .ok_or(ReasoningFieldError::SourceTooLarge)?;
        self.space();
        match self.input.get(self.offset).copied() {
            Some(b'{') => self.object(location, depth),
            Some(b'[') => self.array(location, depth),
            Some(b'"') => {
                self.string()?;
                Ok(())
            }
            Some(b't') => self.literal(b"true"),
            Some(b'f') => self.literal(b"false"),
            Some(b'n') => self.literal(b"null"),
            Some(b'-' | b'0'..=b'9') => self.number(),
            _ => Err(ReasoningFieldError::MalformedSource),
        }
    }

    fn object(&mut self, location: ScanLocation, depth: usize) -> Result<(), ReasoningFieldError> {
        self.consume(b'{')?;
        self.space();
        if self.input.get(self.offset) == Some(&b'}') {
            return self.consume(b'}');
        }
        loop {
            let key = self.string()?;
            self.space();
            self.consume(b':')?;
            self.space();
            let start = self.offset;
            let child = match location {
                ScanLocation::Root if key == b"messages" => ScanLocation::Messages,
                ScanLocation::Message(index) if key == b"content" => {
                    ScanLocation::ContentParts(index)
                }
                _ => ScanLocation::Other,
            };
            self.value(child, depth + 1)?;
            if let ScanLocation::Part(message_ordinal, part_ordinal) = location
                && key == b"originProof"
            {
                if self.offset - start > RECEIPT_BYTE_CAP
                    || self.receipts.len()
                        >= crate::internal::ai::traces::REASONING_ARTIFACT_MAX_ENTRIES
                {
                    return Err(ReasoningFieldError::SourceTooLarge);
                }
                self.receipts
                    .try_reserve(1)
                    .map_err(|_| ReasoningFieldError::SourceTooLarge)?;
                self.receipts.push(ReceiptSpan {
                    message_ordinal,
                    part_ordinal,
                    start,
                    end: self.offset,
                });
            }
            self.space();
            match self.input.get(self.offset).copied() {
                Some(b',') => {
                    self.offset += 1;
                    self.space();
                }
                Some(b'}') => return self.consume(b'}'),
                _ => return Err(ReasoningFieldError::MalformedSource),
            }
        }
    }

    fn array(&mut self, location: ScanLocation, depth: usize) -> Result<(), ReasoningFieldError> {
        self.consume(b'[')?;
        self.space();
        if self.input.get(self.offset) == Some(&b']') {
            return self.consume(b']');
        }
        let mut ordinal = 0_usize;
        loop {
            let child = match location {
                ScanLocation::Messages => ScanLocation::Message(ordinal),
                ScanLocation::ContentParts(message) => ScanLocation::Part(message, ordinal),
                _ => ScanLocation::Other,
            };
            self.value(child, depth + 1)?;
            ordinal = ordinal
                .checked_add(1)
                .ok_or(ReasoningFieldError::SourceTooLarge)?;
            self.space();
            match self.input.get(self.offset).copied() {
                Some(b',') => {
                    self.offset += 1;
                    self.space();
                }
                Some(b']') => return self.consume(b']'),
                _ => return Err(ReasoningFieldError::MalformedSource),
            }
        }
    }

    /// Return the unescaped key bytes only when the source spelling is direct.
    /// Escaped field names still parse safely but cannot yield a receipt span.
    fn string(&mut self) -> Result<&'a [u8], ReasoningFieldError> {
        self.consume(b'"')?;
        let start = self.offset;
        let mut direct = true;
        loop {
            match self.input.get(self.offset).copied() {
                Some(b'"') => {
                    let end = self.offset;
                    self.offset += 1;
                    return Ok(if direct { &self.input[start..end] } else { &[] });
                }
                Some(b'\\') => {
                    direct = false;
                    self.offset += 1;
                    match self.input.get(self.offset).copied() {
                        Some(b'"' | b'\\' | b'/' | b'b' | b'f' | b'n' | b'r' | b't') => {
                            self.offset += 1;
                        }
                        Some(b'u') => {
                            self.offset += 1;
                            let Some(hex) = self.input.get(self.offset..self.offset + 4) else {
                                return Err(ReasoningFieldError::MalformedSource);
                            };
                            if !hex.iter().all(u8::is_ascii_hexdigit) {
                                return Err(ReasoningFieldError::MalformedSource);
                            }
                            self.offset += 4;
                        }
                        _ => return Err(ReasoningFieldError::MalformedSource),
                    }
                }
                Some(0..=31) | None => return Err(ReasoningFieldError::MalformedSource),
                Some(_) => self.offset += 1,
            }
        }
    }

    fn literal(&mut self, literal: &[u8]) -> Result<(), ReasoningFieldError> {
        if self.input.get(self.offset..self.offset + literal.len()) != Some(literal) {
            return Err(ReasoningFieldError::MalformedSource);
        }
        self.offset += literal.len();
        Ok(())
    }

    fn number(&mut self) -> Result<(), ReasoningFieldError> {
        if self.input.get(self.offset) == Some(&b'-') {
            self.offset += 1;
        }
        match self.input.get(self.offset).copied() {
            Some(b'0') => self.offset += 1,
            Some(b'1'..=b'9') => {
                self.offset += 1;
                while self.input.get(self.offset).is_some_and(u8::is_ascii_digit) {
                    self.offset += 1;
                }
            }
            _ => return Err(ReasoningFieldError::MalformedSource),
        }
        if self.input.get(self.offset) == Some(&b'.') {
            self.offset += 1;
            let start = self.offset;
            while self.input.get(self.offset).is_some_and(u8::is_ascii_digit) {
                self.offset += 1;
            }
            if start == self.offset {
                return Err(ReasoningFieldError::MalformedSource);
            }
        }
        if self
            .input
            .get(self.offset)
            .is_some_and(|byte| matches!(byte, b'e' | b'E'))
        {
            self.offset += 1;
            if self
                .input
                .get(self.offset)
                .is_some_and(|byte| matches!(byte, b'+' | b'-'))
            {
                self.offset += 1;
            }
            let start = self.offset;
            while self.input.get(self.offset).is_some_and(u8::is_ascii_digit) {
                self.offset += 1;
            }
            if start == self.offset {
                return Err(ReasoningFieldError::MalformedSource);
            }
        }
        Ok(())
    }
}

impl Zeroize for CanonValue {
    fn zeroize(&mut self) {
        match self {
            Self::Str(value) => value.zeroize(),
            Self::Array(items) => {
                for item in items.iter_mut() {
                    item.zeroize();
                }
            }
            Self::Object(fields) => {
                for (mut key, mut value) in std::mem::take(fields) {
                    key.zeroize();
                    value.zeroize();
                }
            }
            Self::Null | Self::Bool(_) | Self::Int(_) | Self::Float(_) => {}
        }
        *self = Self::Null;
    }
}

/// Every branch removed from the owned parse tree is wiped before its
/// allocation is released. `Zeroizing<CanonValue>` on the root alone cannot
/// cover nodes dropped by `retain`, `remove` or replacement during scrub.
fn retain_wiping(
    fields: &mut BTreeMap<String, CanonValue>,
    mut keep: impl FnMut(&String, &mut CanonValue) -> bool,
) {
    for (mut key, mut value) in fields.extract_if(.., |key, value| !keep(key, value)) {
        key.zeroize();
        value.zeroize();
    }
}

fn remove_wiping(fields: &mut BTreeMap<String, CanonValue>, name: &str) -> bool {
    if let Some((mut key, mut value)) = fields.remove_entry(name) {
        key.zeroize();
        value.zeroize();
        true
    } else {
        false
    }
}

fn take_wiping(
    fields: &mut BTreeMap<String, CanonValue>,
    name: &str,
) -> Option<Zeroizing<CanonValue>> {
    fields.remove_entry(name).map(|(mut key, value)| {
        key.zeroize();
        Zeroizing::new(value)
    })
}

/// The duplicate-ID check owns copies of source coordinates while scanning.
/// They leave no unwiped heap strings on either the success or error path.
struct WipingIdSet(BTreeSet<String>);

impl Deref for WipingIdSet {
    type Target = BTreeSet<String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for WipingIdSet {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Drop for WipingIdSet {
    fn drop(&mut self) {
        for mut id in std::mem::take(&mut self.0) {
            id.zeroize();
        }
    }
}

/// `serde_json::to_vec` may allocate well beyond the admitted source after
/// escapes expand. An output failure drops and wipes every partial write.
/// A payload-free classification; `Candidate` never grants native authority.
enum ReasoningPartShape {
    Visible,
    Candidate,
    Unsupported,
}

/// Only a native/classic assistant reasoning part may supply readable text.
/// An open `state`, even when it uses one of the three pinned names, cannot
/// become ordinary content or native proof by inspecting its shape alone.
fn classify_reasoning_fields(fields: &BTreeMap<String, CanonValue>) -> ReasoningPartShape {
    if fields.get("text").and_then(CanonValue::as_str).is_none() {
        return ReasoningPartShape::Unsupported;
    }
    if fields.keys().any(|name| {
        !matches!(
            name.as_str(),
            "id" | "type" | "text" | "time" | "state" | "originProof" | "sessionID" | "messageID"
        )
    }) {
        return ReasoningPartShape::Unsupported;
    }
    if ["sessionID", "messageID"].iter().any(|name| {
        fields
            .get(*name)
            .is_some_and(|value| value.as_str().is_none_or(str::is_empty))
    }) {
        return ReasoningPartShape::Unsupported;
    }
    if fields.get("time").is_some_and(|time| {
        !matches!(time, CanonValue::Object(values) if values.iter().all(|(name, value)| {
            matches!(name.as_str(), "start" | "end" | "created" | "completed")
                && matches!(value, CanonValue::Int(_) | CanonValue::Float(_))
        }))
    }) {
        // A nested hint in a nominally visible part's time would otherwise
        // disappear silently and misclassify the part as ordinary.
        return ReasoningPartShape::Unsupported;
    }
    let proof_present = fields.contains_key("originProof");
    match fields.get("state") {
        None if !proof_present => ReasoningPartShape::Visible,
        Some(CanonValue::Object(state)) if candidate_state_is_closed(state) => {
            ReasoningPartShape::Candidate
        }
        _ => ReasoningPartShape::Unsupported,
    }
}

fn candidate_state_is_closed(state: &BTreeMap<String, CanonValue>) -> bool {
    !state.is_empty()
        && state.iter().all(|(name, value)| {
            matches!(
                name.as_str(),
                "signature" | "redactedData" | "reasoningEncryptedContent"
            ) && value.as_str().is_some_and(|value| !value.is_empty())
        })
}

fn has_candidate_state_name(fields: &BTreeMap<String, CanonValue>) -> bool {
    matches!(fields.get("state"), Some(CanonValue::Object(state)) if state.keys().any(|name| {
        matches!(
            name.as_str(),
            "signature" | "redactedData" | "reasoningEncryptedContent" | "thoughtSignature" | "originProof"
        )
    }))
}

fn has_opaque_hint_on_nonreasoning(fields: &BTreeMap<String, CanonValue>) -> bool {
    fields.contains_key("originProof")
        || fields.keys().any(|name| {
            matches!(
                name.as_str(),
                "signature" | "redactedData" | "reasoningEncryptedContent" | "thoughtSignature"
            )
        })
        || has_candidate_state_name(fields)
}

fn contains_opaque_hint(value: &CanonValue) -> bool {
    match value {
        CanonValue::Object(fields) => {
            fields.keys().any(|name| {
                matches!(
                    name.as_str(),
                    "signature"
                        | "redactedData"
                        | "reasoningEncryptedContent"
                        | "thoughtSignature"
                        | "originProof"
                )
            }) || fields.values().any(contains_opaque_hint)
        }
        CanonValue::Array(items) => items.iter().any(contains_opaque_hint),
        _ => false,
    }
}

fn strip_provider_metadata(
    fields: &mut BTreeMap<String, CanonValue>,
    removed: &mut bool,
    unproven: &mut bool,
) {
    if let Some(value) = take_wiping(fields, "metadata") {
        *removed = true;
        *unproven |= contains_opaque_hint(&value);
    }
    strip_provider_metadata_except_tool_files(fields, removed, unproven);
}

fn strip_provider_metadata_except_tool_files(
    fields: &mut BTreeMap<String, CanonValue>,
    removed: &mut bool,
    unproven: &mut bool,
) {
    for name in [
        "providerMetadata",
        "callProviderMetadata",
        "providerState",
        "providerResultState",
        "providerOptions",
        "providerContext",
    ] {
        if let Some(value) = take_wiping(fields, name) {
            *removed = true;
            *unproven |= contains_opaque_hint(&value);
        }
    }
}

fn strip_named_opaque_fields(
    fields: &mut BTreeMap<String, CanonValue>,
    removed: &mut bool,
    unproven: &mut bool,
) {
    for name in [
        "signature",
        "redactedData",
        "reasoningEncryptedContent",
        "originProof",
        "state",
        "thoughtSignature",
    ] {
        if remove_wiping(fields, name) {
            *removed = true;
            *unproven = true;
        }
    }
}

/// An envelope may carry arbitrary provider data below a documented field
/// (for example `model.metadata`). Inspect every retained branch before the
/// Redactor sees it; tool input/output and visible text are handled separately.
fn scrub_nested_envelope_value(value: &mut CanonValue, removed: &mut bool, unproven: &mut bool) {
    if contains_opaque_hint(value) {
        value.zeroize();
        *removed = true;
        *unproven = true;
        return;
    }
    strip_nested_envelope_metadata(value, removed, unproven);
}

fn strip_nested_envelope_metadata(value: &mut CanonValue, removed: &mut bool, unproven: &mut bool) {
    match value {
        CanonValue::Object(fields) => {
            retain_wiping(fields, |name, child| {
                if matches!(
                    name.as_str(),
                    "metadata"
                        | "providerMetadata"
                        | "callProviderMetadata"
                        | "providerState"
                        | "providerResultState"
                        | "providerOptions"
                        | "providerContext"
                        | "state"
                ) {
                    *removed = true;
                    *unproven |= name == "state";
                    false
                } else {
                    strip_nested_envelope_metadata(child, removed, unproven);
                    true
                }
            });
        }
        CanonValue::Array(items) => {
            for item in items {
                strip_nested_envelope_metadata(item, removed, unproven);
            }
        }
        _ => {}
    }
}

fn scrub_retained_envelope_fields(
    fields: &mut BTreeMap<String, CanonValue>,
    part_key: &str,
    removed: &mut bool,
    unproven: &mut bool,
) {
    for (name, value) in fields.iter_mut() {
        if name == part_key {
            continue;
        }
        if name == "text" {
            if !matches!(value, CanonValue::Str(_)) {
                // The only ordinary text exception is a validated string.
                // An object here could carry provider ciphertext by name.
                value.zeroize();
                *removed = true;
                *unproven = true;
            }
            continue;
        }
        scrub_nested_envelope_value(value, removed, unproven);
    }
}

fn retain_native_nonassistant_envelope(
    fields: &mut BTreeMap<String, CanonValue>,
    kind: &str,
    removed: &mut bool,
    unproven: &mut bool,
) {
    let prior_len = fields.len();
    retain_wiping(fields, |name, _| {
        matches!(name.as_str(), "id" | "type" | "time")
            || match kind {
                "user" => matches!(name.as_str(), "text" | "files" | "agents" | "skills"),
                "location-switched" => matches!(
                    name.as_str(),
                    "location" | "projectID" | "subpath" | "previous"
                ),
                "agent-switched" => matches!(name.as_str(), "agent" | "previous"),
                "model-switched" => matches!(name.as_str(), "model" | "previous"),
                "synthetic" | "system" => matches!(name.as_str(), "text" | "description"),
                "skill" => matches!(name.as_str(), "skill" | "name" | "text"),
                "shell" => matches!(
                    name.as_str(),
                    "shellID" | "command" | "status" | "exit" | "output"
                ),
                "compaction" => matches!(
                    name.as_str(),
                    "status"
                        | "reason"
                        | "model"
                        | "summary"
                        | "recent"
                        | "cost"
                        | "tokens"
                        | "error"
                ),
                "idle" => matches!(name.as_str(), "outcome" | "error"),
                _ => false,
            }
    });
    if fields.len() != prior_len {
        *removed = true;
        *unproven = true;
    }
    project_scalar_fields(
        fields,
        &[
            ("id", ScalarShape::String),
            ("type", ScalarShape::String),
            ("text", ScalarShape::String),
            ("description", ScalarShape::String),
            ("projectID", ScalarShape::String),
            ("subpath", ScalarShape::String),
            ("agent", ScalarShape::String),
            ("skill", ScalarShape::String),
            ("name", ScalarShape::String),
            ("shellID", ScalarShape::String),
            ("command", ScalarShape::String),
            ("status", ScalarShape::String),
            ("exit", ScalarShape::Number),
            ("reason", ScalarShape::String),
            ("summary", ScalarShape::String),
            ("recent", ScalarShape::String),
            ("cost", ScalarShape::Number),
            ("outcome", ScalarShape::String),
        ],
        removed,
        unproven,
    );
    project_envelope_object(fields, "time", &["created", "completed"], removed, unproven);
    if kind == "agent-switched" {
        project_scalar_fields(
            fields,
            &[("previous", ScalarShape::String)],
            removed,
            unproven,
        );
    } else if kind == "location-switched" {
        project_envelope_object(fields, "location", &["directory"], removed, unproven);
        project_envelope_object(
            fields,
            "previous",
            &["location", "projectID", "subpath"],
            removed,
            unproven,
        );
        if let Some(CanonValue::Object(previous)) = fields.get_mut("previous") {
            project_envelope_object(previous, "location", &["directory"], removed, unproven);
        }
    } else if kind == "model-switched" {
        project_envelope_object(
            fields,
            "model",
            &["id", "providerID", "variant"],
            removed,
            unproven,
        );
        project_envelope_object(
            fields,
            "previous",
            &["id", "providerID", "variant"],
            removed,
            unproven,
        );
    } else if kind == "compaction" {
        project_envelope_object(
            fields,
            "model",
            &["id", "providerID", "variant"],
            removed,
            unproven,
        );
        project_envelope_object(
            fields,
            "tokens",
            &["input", "output", "reasoning", "cache"],
            removed,
            unproven,
        );
        if let Some(CanonValue::Object(tokens)) = fields.get_mut("tokens") {
            project_envelope_object(tokens, "cache", &["read", "write"], removed, unproven);
        }
        project_structured_error(fields, "error", removed, unproven);
    } else if kind == "idle" {
        project_native_idle_error(fields, removed, unproven);
    } else if kind == "shell" {
        project_shell_output(fields, removed, unproven);
    }
    if kind == "user" {
        if let Some(files) = fields.get("files")
            && !matches!(files, CanonValue::Array(entries) if entries.is_empty())
        {
            remove_wiping(fields, "files");
            *removed = true;
            *unproven = true;
        }
        for (name, allowed) in [
            ("agents", &["name", "mention"][..]),
            ("skills", &["id", "name", "mention"][..]),
        ] {
            if let Some(value) = fields.get_mut(name) {
                if let CanonValue::Array(entries) = value {
                    for entry in entries {
                        if contains_opaque_hint(entry) {
                            entry.zeroize();
                            *removed = true;
                            *unproven = true;
                            continue;
                        }
                        if let CanonValue::Object(item) = entry {
                            let prior_len = item.len();
                            retain_wiping(item, |key, _| allowed.contains(&key.as_str()));
                            if item.len() != prior_len {
                                *removed = true;
                                *unproven = true;
                            }
                            project_scalar_fields(
                                item,
                                &[("id", ScalarShape::String), ("name", ScalarShape::String)],
                                removed,
                                unproven,
                            );
                            project_envelope_object(
                                item,
                                "mention",
                                &["start", "end", "text"],
                                removed,
                                unproven,
                            );
                        } else {
                            entry.zeroize();
                            *removed = true;
                            *unproven = true;
                        }
                    }
                } else {
                    remove_wiping(fields, name);
                    *removed = true;
                    *unproven = true;
                }
            }
        }
    }
    scrub_retained_envelope_fields(fields, "content", removed, unproven);
}

fn project_shell_output(
    fields: &mut BTreeMap<String, CanonValue>,
    removed: &mut bool,
    unproven: &mut bool,
) {
    let Some(value) = fields.get_mut("output") else {
        return;
    };
    if contains_opaque_hint(value) {
        remove_wiping(fields, "output");
        *removed = true;
        *unproven = true;
        return;
    }
    let CanonValue::Object(output) = value else {
        remove_wiping(fields, "output");
        *removed = true;
        *unproven = true;
        return;
    };
    let prior_len = output.len();
    retain_wiping(output, |name, value| match name.as_str() {
        "output" => matches!(value, CanonValue::Str(_)),
        "cursor" | "size" => matches!(value, CanonValue::Int(_)),
        "truncated" => matches!(value, CanonValue::Bool(_)),
        _ => false,
    });
    if output.len() != prior_len {
        *removed = true;
        *unproven = true;
    }
}

fn project_envelope_object(
    fields: &mut BTreeMap<String, CanonValue>,
    name: &str,
    allowed: &[&str],
    removed: &mut bool,
    unproven: &mut bool,
) {
    let Some(value) = fields.get_mut(name) else {
        return;
    };
    if contains_opaque_hint(value) {
        remove_wiping(fields, name);
        *removed = true;
        *unproven = true;
        return;
    }
    let CanonValue::Object(projected) = value else {
        *removed |= remove_wiping(fields, name);
        *unproven = true;
        return;
    };
    let prior_len = projected.len();
    retain_wiping(projected, |key, value| {
        if !allowed.contains(&key.as_str()) {
            return false;
        }
        match name {
            "model" | "path" | "location" => {
                matches!(value, CanonValue::Str(_))
            }
            "time" => matches!(value, CanonValue::Int(_) | CanonValue::Float(_)),
            "tokens" | "cache" if key == "cache" => {
                matches!(value, CanonValue::Object(_))
            }
            "tokens" | "cache" => {
                matches!(value, CanonValue::Int(_) | CanonValue::Float(_))
            }
            "snapshot" if key == "files" => {
                matches!(value, CanonValue::Array(items) if items.iter().all(|item| matches!(item, CanonValue::Str(_))))
            }
            "snapshot" => matches!(value, CanonValue::Str(_)),
            "previous" if key == "location" => matches!(value, CanonValue::Object(_)),
            "previous" => matches!(value, CanonValue::Str(_)),
            "mention" if key == "text" => matches!(value, CanonValue::Str(_)),
            "mention" => matches!(value, CanonValue::Int(_) | CanonValue::Float(_)),
            "source" | "text" if key == "value" => matches!(value, CanonValue::Str(_)),
            "source" => matches!(value, CanonValue::Int(_)),
            "text" => matches!(value, CanonValue::Int(_) | CanonValue::Float(_)),
            "range" => matches!(value, CanonValue::Object(_)),
            "start" | "end" => matches!(value, CanonValue::Int(_)),
            _ => false,
        }
    });
    if projected.len() != prior_len {
        *removed = true;
        *unproven = true;
    }
}

#[derive(Clone, Copy)]
enum ScalarShape {
    String,
    Number,
    Integer,
    Boolean,
}

fn project_scalar_fields(
    fields: &mut BTreeMap<String, CanonValue>,
    shapes: &[(&str, ScalarShape)],
    removed: &mut bool,
    unproven: &mut bool,
) {
    for (name, shape) in shapes {
        let invalid = fields.get(*name).is_some_and(|value| match shape {
            ScalarShape::String => !matches!(value, CanonValue::Str(_)),
            ScalarShape::Number => !matches!(value, CanonValue::Int(_) | CanonValue::Float(_)),
            ScalarShape::Integer => !matches!(value, CanonValue::Int(_)),
            ScalarShape::Boolean => !matches!(value, CanonValue::Bool(_)),
        });
        if invalid {
            remove_wiping(fields, name);
            *removed = true;
            *unproven = true;
        }
    }
}

/// V1 timestamps use `NonNegativeInt`; native timestamps use finite numbers.
/// Keep this check at V1 call sites so native fractional milliseconds remain
/// compatible with the upstream schema.
fn project_classic_integer_time(
    fields: &mut BTreeMap<String, CanonValue>,
    names: &[&str],
    removed: &mut bool,
    unproven: &mut bool,
) {
    let Some(CanonValue::Object(time)) = fields.get_mut("time") else {
        return;
    };
    for name in names {
        if time
            .get(*name)
            .is_some_and(|value| !matches!(value, CanonValue::Int(_)))
        {
            remove_wiping(time, name);
            *removed = true;
            *unproven = true;
        }
    }
}

/// OpenCode 2.0.29 adds an optional SessionError to idle messages. Required
/// strings and the optional HTTP status use its exact public schema; response
/// bodies, unknown fields and opaque hints still follow the existing partial
/// projection policy. This does not authenticate provider/native provenance.
fn project_native_idle_error(
    fields: &mut BTreeMap<String, CanonValue>,
    removed: &mut bool,
    unproven: &mut bool,
) {
    let Some(value) = fields.get("error") else {
        return;
    };
    let valid_header = matches!(value, CanonValue::Object(error)
        if matches!(error.get("type"), Some(CanonValue::Str(_)))
            && matches!(error.get("message"), Some(CanonValue::Str(_)))
            && error.get("status").is_none_or(|status|
                matches!(status, CanonValue::Int(code) if (100..=599).contains(code))));
    if !valid_header {
        remove_wiping(fields, "error");
        *removed = true;
        *unproven = true;
        return;
    }
    project_structured_error(fields, "error", removed, unproven);
}

/// Provider error bodies can contain raw request or response material. Keep
/// only the typed diagnostic header; dropping a response is an explicit
/// partial result rather than silently treating it as an ordinary export.
fn project_structured_error(
    fields: &mut BTreeMap<String, CanonValue>,
    name: &str,
    removed: &mut bool,
    unproven: &mut bool,
) {
    let Some(value) = fields.get_mut(name) else {
        return;
    };
    if contains_opaque_hint(value) {
        remove_wiping(fields, name);
        *removed = true;
        *unproven = true;
        return;
    }
    let CanonValue::Object(error) = value else {
        remove_wiping(fields, name);
        *removed = true;
        *unproven = true;
        return;
    };
    let prior_len = error.len();
    retain_wiping(error, |key, value| match key.as_str() {
        "type" | "message" => matches!(value, CanonValue::Str(_)),
        "status" => matches!(value, CanonValue::Int(_)),
        _ => false,
    });
    if error.len() != prior_len {
        *removed = true;
        *unproven = true;
    }
}

fn project_native_retry(
    fields: &mut BTreeMap<String, CanonValue>,
    removed: &mut bool,
    unproven: &mut bool,
) {
    let Some(value) = fields.get_mut("retry") else {
        return;
    };
    if contains_opaque_hint(value) {
        remove_wiping(fields, "retry");
        *removed = true;
        *unproven = true;
        return;
    }
    let CanonValue::Object(retry) = value else {
        remove_wiping(fields, "retry");
        *removed = true;
        *unproven = true;
        return;
    };
    let prior_len = retry.len();
    retain_wiping(retry, |key, value| match key.as_str() {
        "attempt" => matches!(value, CanonValue::Int(_)),
        "at" => matches!(value, CanonValue::Int(_) | CanonValue::Float(_)),
        "error" => matches!(value, CanonValue::Object(_)),
        _ => false,
    });
    if retry.len() != prior_len {
        *removed = true;
        *unproven = true;
    }
    project_structured_error(retry, "error", removed, unproven);
}

/// Current native tool results carry a closed text/file content union. Older
/// OpenCode exports can carry a plain string, which remains ordinary output.
fn project_native_tool_content(
    state: &mut BTreeMap<String, CanonValue>,
    removed: &mut bool,
    unproven: &mut bool,
) {
    let Some(content) = state.get_mut("content") else {
        return;
    };
    scrub_nested_envelope_value(content, removed, unproven);
    let CanonValue::Array(entries) = content else {
        if !matches!(content, CanonValue::Str(_) | CanonValue::Null) {
            content.zeroize();
            *removed = true;
            *unproven = true;
        }
        return;
    };
    if entries.is_empty() {
        content.zeroize();
        *removed = true;
        *unproven = true;
        return;
    }
    for entry in entries {
        let CanonValue::Object(fields) = entry else {
            entry.zeroize();
            *removed = true;
            *unproven = true;
            continue;
        };
        let allowed: &[&str] = match fields.get("type").and_then(CanonValue::as_str) {
            Some("text") if fields.get("text").and_then(CanonValue::as_str).is_some() => {
                &["type", "text"]
            }
            Some("file")
                if fields.get("uri").and_then(CanonValue::as_str).is_some()
                    && fields.get("mime").and_then(CanonValue::as_str).is_some() =>
            {
                &["type", "uri", "mime", "name"]
            }
            _ => {
                entry.zeroize();
                *removed = true;
                *unproven = true;
                continue;
            }
        };
        let prior_len = fields.len();
        retain_wiping(fields, |name, value| {
            allowed.contains(&name.as_str())
                && (name != "name" || matches!(value, CanonValue::Str(_)))
        });
        if fields.len() != prior_len {
            *removed = true;
            *unproven = true;
        }
    }
}

/// V1 named errors have open response fields. The ordinary copy keeps only
/// stable scalar diagnostics, never response bodies, headers or metadata.
fn project_classic_error(
    fields: &mut BTreeMap<String, CanonValue>,
    name: &str,
    removed: &mut bool,
    unproven: &mut bool,
) {
    let Some(value) = fields.get_mut(name) else {
        return;
    };
    if contains_opaque_hint(value) {
        remove_wiping(fields, name);
        *removed = true;
        *unproven = true;
        return;
    }
    let CanonValue::Object(error) = value else {
        remove_wiping(fields, name);
        *removed = true;
        *unproven = true;
        return;
    };
    let prior_len = error.len();
    retain_wiping(error, |key, value| match key.as_str() {
        "name" => matches!(value, CanonValue::Str(_)),
        "data" => matches!(value, CanonValue::Object(_)),
        _ => false,
    });
    if error.len() != prior_len {
        *removed = true;
        *unproven = true;
    }
    let Some(CanonValue::Object(data)) = error.get_mut("data") else {
        return;
    };
    let prior_len = data.len();
    retain_wiping(data, |key, value| match key.as_str() {
        "providerID" | "message" | "ref" => matches!(value, CanonValue::Str(_)),
        "statusCode" | "retries" => matches!(value, CanonValue::Int(_)),
        "isRetryable" => matches!(value, CanonValue::Bool(_)),
        _ => false,
    });
    if data.len() != prior_len {
        *removed = true;
        *unproven = true;
    }
}

/// Tool execution metadata can contain the only modified-file provenance.
/// Keep just the extractor's `files[].file` string projection; every other
/// tool metadata value remains outside the ordinary copy.
fn project_tool_state_files(
    state: &mut BTreeMap<String, CanonValue>,
    removed: &mut bool,
    unproven: &mut bool,
) {
    let completed = state.get("status").and_then(CanonValue::as_str) == Some("completed");
    let Some(metadata) = take_wiping(state, "metadata") else {
        return;
    };
    *unproven |= contains_opaque_hint(&metadata);
    if !completed {
        *removed = true;
        return;
    }
    let projected = if let CanonValue::Object(fields) = &*metadata {
        // OpenCode's tool-output transform adds this Boolean to successful
        // tool results, including results that did not need truncation. The
        // value has no extractor use, so recognize its type and drop it.
        // Every other sibling, and a wrong-typed value, stays unproven.
        if fields.iter().any(|(name, value)| match name.as_str() {
            "files" => false,
            "truncated" => !matches!(value, CanonValue::Bool(_)),
            _ => true,
        }) {
            *unproven = true;
        }
        if let Some(CanonValue::Array(entries)) = fields.get("files") {
            let mut safe_entries = Vec::with_capacity(entries.len());
            for entry in entries {
                if let CanonValue::Object(entry_fields) = entry
                    && entry_fields
                        .keys()
                        .any(|name| name != "file" && name != "diff")
                {
                    *unproven = true;
                }
                let Some(CanonValue::Str(file)) = entry.get("file") else {
                    *unproven = true;
                    continue;
                };
                let mut safe_entry = BTreeMap::new();
                safe_entry.insert("file".to_owned(), CanonValue::Str(file.clone()));
                safe_entries.push(CanonValue::Object(safe_entry));
            }
            let mut safe_metadata = BTreeMap::new();
            safe_metadata.insert("files".to_owned(), CanonValue::Array(safe_entries));
            Some(CanonValue::Object(safe_metadata))
        } else {
            if fields.contains_key("files") {
                *unproven = true;
            }
            None
        }
    } else {
        *unproven = true;
        None
    };
    *removed |= projected.as_ref() != Some(&*metadata);
    if let Some(projected) = projected {
        state.insert("metadata".to_owned(), projected);
    }
}

fn scrub_assistant_message_hints(
    fields: &mut BTreeMap<String, CanonValue>,
    native: bool,
    removed: &mut bool,
    unproven: &mut bool,
) {
    strip_provider_metadata(fields, removed, unproven);
    strip_named_opaque_fields(fields, removed, unproven);
    let prior_len = fields.len();
    retain_wiping(fields, |name, _| {
        if native {
            matches!(
                name.as_str(),
                "id" | "type"
                    | "time"
                    | "agent"
                    | "model"
                    | "content"
                    | "snapshot"
                    | "finish"
                    | "rawFinish"
                    | "cost"
                    | "tokens"
                    | "error"
                    | "retry"
            )
        } else {
            matches!(name.as_str(), "info" | "parts")
        }
    });
    if fields.len() != prior_len {
        *removed = true;
        *unproven = true;
    }
    if native {
        project_scalar_fields(
            fields,
            &[
                ("id", ScalarShape::String),
                ("type", ScalarShape::String),
                ("agent", ScalarShape::String),
                ("finish", ScalarShape::String),
                ("rawFinish", ScalarShape::String),
                ("cost", ScalarShape::Number),
            ],
            removed,
            unproven,
        );
    }
    if !native && let Some(CanonValue::Object(info)) = fields.get_mut("info") {
        strip_provider_metadata(info, removed, unproven);
        let prior_len = info.len();
        retain_wiping(info, |name, _| {
            matches!(
                name.as_str(),
                "id" | "sessionID"
                    | "role"
                    | "time"
                    | "parentID"
                    | "modelID"
                    | "providerID"
                    | "model"
                    | "mode"
                    | "agent"
                    | "path"
                    | "cost"
                    | "tokens"
                    | "variant"
                    | "finish"
                    | "system"
                    | "summary"
                    | "tools"
                    | "format"
                    | "error"
                    | "structured"
            )
        });
        if info.len() != prior_len {
            *removed = true;
            *unproven = true;
        }
        project_scalar_fields(
            info,
            &[
                ("id", ScalarShape::String),
                ("sessionID", ScalarShape::String),
                ("role", ScalarShape::String),
                ("parentID", ScalarShape::String),
                ("modelID", ScalarShape::String),
                ("providerID", ScalarShape::String),
                ("mode", ScalarShape::String),
                ("agent", ScalarShape::String),
                ("variant", ScalarShape::String),
                ("finish", ScalarShape::String),
                ("system", ScalarShape::String),
                ("cost", ScalarShape::Number),
            ],
            removed,
            unproven,
        );
        project_envelope_object(
            info,
            "model",
            &["id", "modelID", "providerID", "variant"],
            removed,
            unproven,
        );
        project_envelope_object(info, "time", &["created", "completed"], removed, unproven);
        project_classic_integer_time(info, &["created", "completed"], removed, unproven);
        project_envelope_object(info, "path", &["cwd", "root"], removed, unproven);
        project_envelope_object(
            info,
            "tokens",
            &["total", "input", "output", "reasoning", "cache"],
            removed,
            unproven,
        );
        if let Some(CanonValue::Object(tokens)) = info.get_mut("tokens") {
            project_envelope_object(tokens, "cache", &["read", "write"], removed, unproven);
        }
        project_classic_error(info, "error", removed, unproven);
        // V1 schema and generated output can carry arbitrary nested JSON.
        // A future card can type-project it; this card leaves a safe partial.
        for name in ["summary", "format", "structured"] {
            if remove_wiping(info, name) {
                *removed = true;
                *unproven = true;
            }
        }
        if let Some(tools) = info.get_mut("tools") {
            if let CanonValue::Object(entries) = tools {
                let prior_len = entries.len();
                retain_wiping(entries, |_, value| matches!(value, CanonValue::Bool(_)));
                if entries.len() != prior_len {
                    *removed = true;
                    *unproven = true;
                }
            } else {
                remove_wiping(info, "tools");
                *removed = true;
                *unproven = true;
            }
        }
    }
    if native {
        project_envelope_object(
            fields,
            "model",
            &["id", "providerID", "variant"],
            removed,
            unproven,
        );
        project_structured_error(fields, "error", removed, unproven);
        project_native_retry(fields, removed, unproven);
        project_envelope_object(
            fields,
            "time",
            &["created", "streamed", "completed"],
            removed,
            unproven,
        );
        project_envelope_object(
            fields,
            "snapshot",
            &["start", "end", "files"],
            removed,
            unproven,
        );
        project_envelope_object(
            fields,
            "tokens",
            &["input", "output", "reasoning", "cache"],
            removed,
            unproven,
        );
        if let Some(CanonValue::Object(tokens)) = fields.get_mut("tokens") {
            project_envelope_object(tokens, "cache", &["read", "write"], removed, unproven);
        }
    }
    scrub_retained_envelope_fields(
        fields,
        if native { "content" } else { "parts" },
        removed,
        unproven,
    );
}

fn known_native_nonassistant(kind: &str) -> bool {
    matches!(
        kind,
        "user"
            | "synthetic"
            | "system"
            | "skill"
            | "shell"
            | "agent-switched"
            | "model-switched"
            | "location-switched"
            | "compaction"
            | "idle"
    )
}

fn known_nonreasoning_part(part_type: &str, classic: bool) -> bool {
    matches!(part_type, "text" | "tool")
        || (classic
            && matches!(
                part_type,
                "step-finish"
                    | "patch"
                    | "step-start"
                    | "snapshot"
                    | "retry"
                    | "compaction"
                    | "agent"
                    | "subtask"
                    | "file"
            ))
}

fn project_classic_file_source(
    fields: &mut BTreeMap<String, CanonValue>,
    removed: &mut bool,
    unproven: &mut bool,
) {
    let Some(value) = fields.get_mut("source") else {
        return;
    };
    if contains_opaque_hint(value) {
        remove_wiping(fields, "source");
        *removed = true;
        *unproven = true;
        return;
    }
    let CanonValue::Object(source) = value else {
        remove_wiping(fields, "source");
        *removed = true;
        *unproven = true;
        return;
    };
    let kind = source.get("type").and_then(CanonValue::as_str);
    let allowed: &[&str] = match kind {
        Some("file") => &["type", "text", "path"],
        Some("symbol") => &["type", "text", "path", "range", "name", "kind"],
        Some("resource") => &["type", "text", "clientName", "uri"],
        _ => {
            remove_wiping(fields, "source");
            *removed = true;
            *unproven = true;
            return;
        }
    };
    let prior_len = source.len();
    retain_wiping(source, |name, _| allowed.contains(&name.as_str()));
    if source.len() != prior_len {
        *removed = true;
        *unproven = true;
    }
    project_scalar_fields(
        source,
        &[
            ("type", ScalarShape::String),
            ("path", ScalarShape::String),
            ("name", ScalarShape::String),
            ("kind", ScalarShape::Integer),
            ("clientName", ScalarShape::String),
            ("uri", ScalarShape::String),
        ],
        removed,
        unproven,
    );
    project_envelope_object(
        source,
        "text",
        &["value", "start", "end"],
        removed,
        unproven,
    );
    project_envelope_object(source, "range", &["start", "end"], removed, unproven);
    if let Some(CanonValue::Object(range)) = source.get_mut("range") {
        for name in ["start", "end"] {
            project_envelope_object(range, name, &["line", "character"], removed, unproven);
        }
    }
}

/// Preserve the schema-known V1 part envelope. Open provider metadata is
/// removed separately; unknown part fields must not bypass that boundary.
fn project_classic_part(
    fields: &mut BTreeMap<String, CanonValue>,
    part_type: &str,
    removed: &mut bool,
    unproven: &mut bool,
) {
    let allowed: &[&str] = match part_type {
        "step-start" => &["snapshot"],
        "step-finish" => &["reason", "snapshot", "cost", "tokens"],
        "snapshot" => &["snapshot"],
        "patch" => &["hash", "files"],
        "agent" => &["name", "source"],
        "subtask" => &["prompt", "description", "agent", "model", "command"],
        "file" => &["mime", "filename", "url", "source"],
        "retry" => &["attempt", "error", "time"],
        "compaction" => &["auto", "overflow", "tail_start_id"],
        _ => &[],
    };
    let prior_len = fields.len();
    retain_wiping(fields, |name, _| {
        matches!(name.as_str(), "id" | "type" | "sessionID" | "messageID")
            || allowed.contains(&name.as_str())
    });
    if fields.len() != prior_len {
        *removed = true;
        *unproven = true;
    }
    project_scalar_fields(
        fields,
        &[
            ("id", ScalarShape::String),
            ("type", ScalarShape::String),
            ("sessionID", ScalarShape::String),
            ("messageID", ScalarShape::String),
            ("snapshot", ScalarShape::String),
            ("reason", ScalarShape::String),
            ("cost", ScalarShape::Number),
            ("hash", ScalarShape::String),
            ("name", ScalarShape::String),
            ("prompt", ScalarShape::String),
            ("description", ScalarShape::String),
            ("agent", ScalarShape::String),
            ("command", ScalarShape::String),
            ("mime", ScalarShape::String),
            ("filename", ScalarShape::String),
            ("url", ScalarShape::String),
            ("attempt", ScalarShape::Integer),
            ("auto", ScalarShape::Boolean),
            ("overflow", ScalarShape::Boolean),
            ("tail_start_id", ScalarShape::String),
        ],
        removed,
        unproven,
    );
    if part_type == "step-finish" {
        project_envelope_object(
            fields,
            "tokens",
            &["total", "input", "output", "reasoning", "cache"],
            removed,
            unproven,
        );
        if let Some(CanonValue::Object(tokens)) = fields.get_mut("tokens") {
            project_envelope_object(tokens, "cache", &["read", "write"], removed, unproven);
        }
    } else if part_type == "agent" {
        project_envelope_object(
            fields,
            "source",
            &["value", "start", "end"],
            removed,
            unproven,
        );
    } else if part_type == "subtask" {
        project_envelope_object(
            fields,
            "model",
            &["providerID", "modelID"],
            removed,
            unproven,
        );
    } else if part_type == "file" {
        project_classic_file_source(fields, removed, unproven);
    } else if part_type == "retry" {
        project_classic_error(fields, "error", removed, unproven);
        project_envelope_object(fields, "time", &["created"], removed, unproven);
        project_classic_integer_time(fields, &["created"], removed, unproven);
    }
    if part_type == "patch"
        && let Some(files) = fields.get("files")
        && !matches!(files, CanonValue::Array(entries) if entries.iter().all(|entry| matches!(entry, CanonValue::Str(_))))
    {
        remove_wiping(fields, "files");
        *removed = true;
        *unproven = true;
    }
    scrub_retained_envelope_fields(fields, "", removed, unproven);
}

fn project_classic_attachments(
    state: &mut BTreeMap<String, CanonValue>,
    removed: &mut bool,
    unproven: &mut bool,
) {
    let Some(attachments) = state.get_mut("attachments") else {
        return;
    };
    let CanonValue::Array(parts) = attachments else {
        remove_wiping(state, "attachments");
        *removed = true;
        *unproven = true;
        return;
    };
    for part in parts {
        let CanonValue::Object(fields) = part else {
            part.zeroize();
            *removed = true;
            *unproven = true;
            continue;
        };
        if fields.get("type").and_then(CanonValue::as_str) != Some("file")
            || has_opaque_hint_on_nonreasoning(fields)
            || fields.values().any(contains_opaque_hint)
        {
            part.zeroize();
            *removed = true;
            *unproven = true;
            continue;
        }
        strip_provider_metadata(fields, removed, unproven);
        project_classic_part(fields, "file", removed, unproven);
    }
}

fn inspect_id_field(
    fields: &mut BTreeMap<String, CanonValue>,
    seen: &mut BTreeSet<String>,
    removed: &mut bool,
    unproven: &mut bool,
) {
    let invalid = match fields.get("id") {
        None | Some(CanonValue::Null) => false,
        Some(CanonValue::Str(id)) if !id.is_empty() => {
            if seen.len() >= MAX_OBSERVED_IDS || seen.contains(id) {
                *unproven = true;
            } else {
                let _ = seen.insert(id.clone());
            }
            false
        }
        _ => true,
    };
    if invalid {
        *unproven = true;
        *removed |= remove_wiping(fields, "id");
    }
}

fn inspect_nonunique_tool_id(
    fields: &mut BTreeMap<String, CanonValue>,
    removed: &mut bool,
    unproven: &mut bool,
) {
    if !matches!(fields.get("id"), None | Some(CanonValue::Str(_))) {
        *removed |= remove_wiping(fields, "id");
        *unproven = true;
    }
}

fn scrub_parts(
    parts: &mut [CanonValue],
    classic: bool,
    seen_parts: &mut BTreeSet<String>,
    removed: &mut bool,
    unproven: &mut bool,
) -> Result<(), ReasoningFieldError> {
    for part in parts {
        let CanonValue::Object(fields) = part else {
            part.zeroize();
            *removed = true;
            *unproven = true;
            continue;
        };
        let Some(part_type) = fields
            .get("type")
            .and_then(CanonValue::as_str)
            .map(|value| Zeroizing::new(value.to_owned()))
        else {
            // An untyped part cannot identify a safe visible field. Discard
            // its complete payload while preserving the rest of the export.
            part.zeroize();
            *removed = true;
            *unproven = true;
            continue;
        };
        if part_type.as_str() == "tool" && !classic {
            // Native tool ids are provider call ids and can repeat across turns.
            inspect_nonunique_tool_id(fields, removed, unproven);
        } else {
            inspect_id_field(fields, seen_parts, removed, unproven);
        }
        if part_type.as_str() != "reasoning" {
            if part_type.as_str() == "text" {
                for name in [
                    "signature",
                    "redactedData",
                    "reasoningEncryptedContent",
                    "thoughtSignature",
                    "originProof",
                ] {
                    if remove_wiping(fields, name) {
                        *removed = true;
                        *unproven = true;
                    }
                }
                strip_provider_metadata(fields, removed, unproven);
                if let Some(state) = take_wiping(fields, "state") {
                    *removed = true;
                    *unproven |= contains_opaque_hint(&state);
                }
                let visible_text = matches!(fields.get("text"), Some(CanonValue::Str(_)));
                let prior_len = fields.len();
                retain_wiping(fields, |name, _| {
                    matches!(
                        name.as_str(),
                        "id" | "type"
                            | "sessionID"
                            | "messageID"
                            | "time"
                            | "synthetic"
                            | "ignored"
                    ) || (name == "text" && visible_text)
                });
                if !visible_text || fields.len() != prior_len {
                    *removed |= fields.len() != prior_len;
                    *unproven = true;
                }
                for name in ["sessionID", "messageID"] {
                    if fields
                        .get(name)
                        .is_some_and(|value| !matches!(value, CanonValue::Str(_)))
                    {
                        remove_wiping(fields, name);
                        *removed = true;
                        *unproven = true;
                    }
                }
                for name in ["synthetic", "ignored"] {
                    if fields
                        .get(name)
                        .is_some_and(|value| !matches!(value, CanonValue::Bool(_)))
                    {
                        remove_wiping(fields, name);
                        *removed = true;
                        *unproven = true;
                    }
                }
                if classic {
                    project_envelope_object(fields, "time", &["start", "end"], removed, unproven);
                    project_classic_integer_time(fields, &["start", "end"], removed, unproven);
                } else if remove_wiping(fields, "time") {
                    *removed = true;
                    *unproven = true;
                }
                scrub_retained_envelope_fields(fields, "", removed, unproven);
            } else if has_opaque_hint_on_nonreasoning(fields) {
                part.zeroize();
                *removed = true;
                *unproven = true;
            } else if !known_nonreasoning_part(part_type.as_str(), classic) {
                let prior_len = fields.len();
                retain_wiping(fields, |name, _| matches!(name.as_str(), "id" | "type"));
                *removed |= fields.len() != prior_len;
                *unproven = true;
            } else {
                strip_provider_metadata(fields, removed, unproven);
                if part_type.as_str() == "tool" {
                    let prior_len = fields.len();
                    retain_wiping(fields, |name, _| {
                        matches!(
                            name.as_str(),
                            "id" | "type"
                                | "name"
                                | "tool"
                                | "callID"
                                | "state"
                                | "time"
                                | "executed"
                                | "sessionID"
                                | "messageID"
                        )
                    });
                    if fields.len() != prior_len {
                        *removed = true;
                        *unproven = true;
                    }
                    project_scalar_fields(
                        fields,
                        &[
                            ("id", ScalarShape::String),
                            ("type", ScalarShape::String),
                            ("name", ScalarShape::String),
                            ("tool", ScalarShape::String),
                            ("callID", ScalarShape::String),
                            ("sessionID", ScalarShape::String),
                            ("messageID", ScalarShape::String),
                            ("executed", ScalarShape::Boolean),
                        ],
                        removed,
                        unproven,
                    );
                    if classic {
                        if remove_wiping(fields, "time") {
                            *removed = true;
                            *unproven = true;
                        }
                    } else {
                        project_envelope_object(
                            fields,
                            "time",
                            &["created", "ran", "completed"],
                            removed,
                            unproven,
                        );
                    }
                    scrub_retained_envelope_fields(fields, "state", removed, unproven);
                    match fields.get_mut("state") {
                        Some(CanonValue::Object(state)) => {
                            strip_provider_metadata_except_tool_files(state, removed, unproven);
                            project_tool_state_files(state, removed, unproven);
                            let prior_len = state.len();
                            let allowed: &[&str] =
                                match (classic, state.get("status").and_then(CanonValue::as_str)) {
                                    (true, Some("pending")) => &["status", "input", "raw"],
                                    (true, Some("running")) => {
                                        &["status", "input", "title", "metadata", "time"]
                                    }
                                    (true, Some("completed")) => &[
                                        "status",
                                        "input",
                                        "output",
                                        "title",
                                        "metadata",
                                        "time",
                                        "attachments",
                                    ],
                                    (true, Some("error")) => {
                                        &["status", "input", "error", "metadata", "time"]
                                    }
                                    (false, Some("streaming")) => &["status", "input"],
                                    (false, Some("running")) => &["status", "input", "metadata"],
                                    (false, Some("completed")) => {
                                        &["status", "input", "content", "metadata"]
                                    }
                                    (false, Some("error")) => {
                                        &["status", "input", "error", "content", "metadata"]
                                    }
                                    _ => &["status", "input"],
                                };
                            retain_wiping(state, |name, _| allowed.contains(&name.as_str()));
                            if state.len() != prior_len {
                                *removed = true;
                                *unproven = true;
                            }
                            for (name, value) in state.iter_mut() {
                                if name != "input" && !(classic && name == "output") {
                                    // Native input and classic input/output
                                    // are ordinary content. Other retained
                                    // fields have typed or projected shapes.
                                    scrub_nested_envelope_value(value, removed, unproven);
                                }
                            }
                            let status = state.get("status").and_then(CanonValue::as_str);
                            let valid_status = status.is_some_and(|value| {
                                if classic {
                                    matches!(value, "pending" | "running" | "completed" | "error")
                                } else {
                                    matches!(value, "streaming" | "running" | "completed" | "error")
                                }
                            });
                            if !valid_status {
                                *removed |= remove_wiping(state, "status");
                                *unproven = true;
                            }
                            if classic {
                                for name in ["raw", "title", "output"] {
                                    if state
                                        .get(name)
                                        .is_some_and(|value| !matches!(value, CanonValue::Str(_)))
                                    {
                                        remove_wiping(state, name);
                                        *removed = true;
                                        *unproven = true;
                                    }
                                }
                                // V1 tool state has no content field. Do not
                                // mistake an injected value for tool output.
                                if remove_wiping(state, "content") {
                                    *removed = true;
                                    *unproven = true;
                                }
                            } else {
                                // Native tool content is a structured visible
                                // result, not the raw input/output exception.
                                project_native_tool_content(state, removed, unproven);
                            }
                            if classic {
                                project_envelope_object(
                                    state,
                                    "time",
                                    &["start", "end", "compacted"],
                                    removed,
                                    unproven,
                                );
                                project_classic_integer_time(
                                    state,
                                    &["start", "end", "compacted"],
                                    removed,
                                    unproven,
                                );
                                project_classic_attachments(state, removed, unproven);
                                if state
                                    .get("error")
                                    .is_some_and(|value| !matches!(value, CanonValue::Str(_)))
                                {
                                    remove_wiping(state, "error");
                                    *removed = true;
                                    *unproven = true;
                                }
                            } else {
                                project_structured_error(state, "error", removed, unproven);
                            }
                        }
                        Some(_) => {
                            part.zeroize();
                            *removed = true;
                            *unproven = true;
                        }
                        None => {
                            *unproven = true;
                        }
                    }
                } else {
                    project_classic_part(fields, part_type.as_str(), removed, unproven);
                }
            }
            continue;
        }
        let visible_only = matches!(
            classify_reasoning_fields(fields),
            ReasoningPartShape::Visible
        );
        if !visible_only {
            *unproven = true;
        }
        let visible_text = visible_only && matches!(fields.get("text"), Some(CanonValue::Str(_)));
        // The owning carrier has already copied the original decoded values.
        // Erase the parsed private branch before the ordinary projection drops
        // it; neither a denied nor a malformed proof is kept in that copy.
        for name in ["state", "originProof"] {
            if take_wiping(fields, name).is_some() {
                *removed = true;
            }
        }
        let prior_len = fields.len();
        retain_wiping(fields, |name, _| {
            matches!(name.as_str(), "id" | "type") || (name == "text" && visible_text)
        });
        *removed |= fields.len() != prior_len;
    }
    Ok(())
}

/// Copy only the closed assistant state that has an exact raw receipt span.
/// The carrier is deliberately non-authoritative; a missing/malformed span
/// leaves the part on OG-25's safe partial path without an issuer witness.
fn collect_sealed_candidates(
    export: &CanonValue,
    source: &[u8],
    receipts: &[ReceiptSpan],
) -> Result<Option<SealedOpenCodeCandidates>, ReasoningFieldError> {
    let CanonValue::Object(export) = export else {
        return Ok(None);
    };
    let Some(CanonValue::Object(document)) = export.get("info") else {
        return Ok(None);
    };
    let Some(source_session) = document.get("id").and_then(CanonValue::as_str) else {
        return Ok(None);
    };
    let Some(messages) = export.get("messages").and_then(CanonValue::as_array) else {
        return Ok(None);
    };
    // IDs are source coordinates, not merely display text. Ambiguous IDs must
    // never be bound to a receipt from a different message or part.
    let mut message_ids = BTreeMap::<&str, usize>::new();
    let mut part_ids = BTreeMap::<&str, usize>::new();
    for message in messages {
        let CanonValue::Object(message) = message else {
            continue;
        };
        if let Some(id) = message.get("id").and_then(CanonValue::as_str) {
            *message_ids.entry(id).or_default() += 1;
        }
        if let Some(parts) = message.get("content").and_then(CanonValue::as_array) {
            for part in parts {
                if let CanonValue::Object(part) = part
                    && let Some(id) = part.get("id").and_then(CanonValue::as_str)
                {
                    *part_ids.entry(id).or_default() += 1;
                }
            }
        }
    }
    let mut candidates = None;
    for (message_ordinal, message) in messages.iter().enumerate() {
        let CanonValue::Object(message) = message else {
            continue;
        };
        if message.get("type").and_then(CanonValue::as_str) != Some("assistant")
            || message.contains_key("info")
            || message.contains_key("parts")
        {
            continue;
        }
        let Some(source_message) = message.get("id").and_then(CanonValue::as_str) else {
            continue;
        };
        if message_ids.get(source_message) != Some(&1) {
            continue;
        }
        let Some(parts) = message.get("content").and_then(CanonValue::as_array) else {
            continue;
        };
        for (part_ordinal, part) in parts.iter().enumerate() {
            let CanonValue::Object(part) = part else {
                continue;
            };
            if part.get("type").and_then(CanonValue::as_str) != Some("reasoning")
                || !matches!(
                    classify_reasoning_fields(part),
                    ReasoningPartShape::Candidate
                )
            {
                continue;
            }
            let Some(source_part) = part.get("id").and_then(CanonValue::as_str) else {
                continue;
            };
            if part_ids.get(source_part) != Some(&1)
                || part
                    .get("sessionID")
                    .is_some_and(|value| value.as_str() != Some(source_session))
                || part
                    .get("messageID")
                    .is_some_and(|value| value.as_str() != Some(source_message))
            {
                continue;
            }
            let Some(CanonValue::Object(state)) = part.get("state") else {
                continue;
            };
            let Some(CanonValue::Object(_)) = part.get("originProof") else {
                continue;
            };
            let mut matching_receipts = receipts.iter().filter(|receipt| {
                receipt.message_ordinal == message_ordinal && receipt.part_ordinal == part_ordinal
            });
            let Some(receipt) = matching_receipts.next() else {
                continue;
            };
            if matching_receipts.next().is_some() {
                return Err(ReasoningFieldError::MalformedSource);
            }
            let Some(raw_receipt) = source.get(receipt.start..receipt.end) else {
                return Err(ReasoningFieldError::MalformedSource);
            };
            let mut fields = Vec::new();
            fields
                .try_reserve(state.len())
                .map_err(|_| ReasoningFieldError::SourceTooLarge)?;
            for (name, value) in state {
                let field = match name.as_str() {
                    "signature" => FieldName::Signature,
                    "redactedData" => FieldName::RedactedData,
                    "reasoningEncryptedContent" => FieldName::ReasoningEncryptedContent,
                    // A closed state is checked above. If that invariant
                    // changes, keep the whole export on the safe Unproven
                    // path rather than suppressing its ordinary projection.
                    _ => return Ok(None),
                };
                let Some(value) = value.as_str() else {
                    return Ok(None);
                };
                fields.push((field, Zeroizing::new(value.to_owned())));
            }
            fields.sort_by_key(|(name, _)| *name);
            let part = match OpenCodeCandidatePart::checked(
                Zeroizing::new(source_message.to_owned()),
                message_ordinal,
                Zeroizing::new(source_part.to_owned()),
                part_ordinal,
                Zeroizing::new(raw_receipt.to_vec()),
                fields,
                source.len(),
            ) {
                Ok(part) => part,
                // A malformed candidate is not evidence. Drop any earlier
                // candidates too; scrub_parts will still remove every
                // opaque field from the ordinary copy and mark it Unproven.
                Err(ReasoningFieldError::UnsupportedShape) => return Ok(None),
                Err(error) => return Err(error),
            };
            if candidates.is_none() {
                candidates = Some(
                    match SealedOpenCodeCandidates::checked(
                        Zeroizing::new(source_session.to_owned()),
                        source.len(),
                    ) {
                        Ok(batch) => batch,
                        Err(ReasoningFieldError::UnsupportedShape) => return Ok(None),
                        Err(error) => return Err(error),
                    },
                );
            }
            let Some(candidates) = candidates.as_mut() else {
                return Ok(None);
            };
            match candidates.append(part) {
                Ok(()) => {}
                Err(ReasoningFieldError::UnsupportedShape) => return Ok(None),
                Err(error) => return Err(error),
            }
        }
    }
    Ok(candidates)
}

/// The parser is invoked only after the structural preflight succeeds.
/// Keeping this seam explicit lets a test prove oversized malformed input
/// never materializes a JSON tree.
fn parse_bounded_canon_value(
    source: &[u8],
    parse: impl FnOnce(&[u8]) -> Result<CanonValue, serde_json::Error>,
) -> Result<(Vec<ReceiptSpan>, CanonValue), ReasoningFieldError> {
    let receipts = StructureScanner::scan(source)?;
    let export = parse(source).map_err(|_| ReasoningFieldError::MalformedSource)?;
    Ok((receipts, export))
}

/// Consume the one authorized export buffer. No ciphertext, receipt, or
/// providerContext.messages bytes are returned to ordinary capture sinks.
fn has_original_origin_proof(value: &CanonValue) -> bool {
    match value {
        CanonValue::Object(fields) => {
            fields.contains_key("originProof") || fields.values().any(has_original_origin_proof)
        }
        CanonValue::Array(items) => items.iter().any(has_original_origin_proof),
        _ => false,
    }
}

/// A skeleton left after reasoning erasure is not useful ordinary capture.
fn retained_meaningful_ordinary_message(document: &BTreeMap<String, CanonValue>) -> bool {
    let Some(messages) = document.get("messages").and_then(CanonValue::as_array) else {
        return false;
    };
    messages.iter().any(|message| {
        let CanonValue::Object(fields) = message else {
            return false;
        };
        match fields.get("type").and_then(CanonValue::as_str) {
            Some("user" | "system") => fields
                .get("text")
                .and_then(CanonValue::as_str)
                .is_some_and(|text| !text.trim().is_empty()),
            Some("assistant") => fields
                .get("content")
                .and_then(CanonValue::as_array)
                .is_some_and(|parts| {
                    parts.iter().any(|part| {
                        part.get("type").and_then(CanonValue::as_str) == Some("text")
                            && part
                                .get("text")
                                .and_then(CanonValue::as_str)
                                .is_some_and(|text| !text.trim().is_empty())
                    })
                }),
            _ => false,
        }
    })
}

pub(crate) fn prepare_authorized_source(
    source: Vec<u8>,
) -> Result<PreparedReasoningSource, ReasoningFieldError> {
    let mut source = Zeroizing::new(source);
    if source.len() as u64 > TRANSCRIPT_READ_HARD_CAP_BYTES {
        return Err(ReasoningFieldError::SourceTooLarge);
    }
    let (receipts, export) = parse_bounded_canon_value(&source, parse_canon_value)?;
    let mut export = Zeroizing::new(export);
    let original_origin_proof = has_original_origin_proof(&export);
    let candidates = collect_sealed_candidates(&export, &source, &receipts)?;
    let CanonValue::Object(document) = &mut *export else {
        return Err(ReasoningFieldError::MalformedSource);
    };
    let original_source_session = document
        .get("info")
        .and_then(|info| info.get("id"))
        .and_then(CanonValue::as_str)
        .filter(|id| !id.is_empty())
        .map(str::to_owned);
    let classic_root = matches!(document.get("messages"), Some(CanonValue::Array(messages)) if messages.iter().any(|message| {
        matches!(message, CanonValue::Object(fields) if fields.contains_key("info") && !fields.contains_key("type"))
    }));
    let mut removed = false;
    let mut unproven = false;
    strip_provider_metadata(document, &mut removed, &mut unproven);
    strip_named_opaque_fields(document, &mut removed, &mut unproven);
    let root_len = document.len();
    retain_wiping(document, |name, _| {
        matches!(name.as_str(), "info" | "messages")
    });
    if document.len() != root_len {
        removed = true;
        unproven = true;
    }
    if let Some(info) = document.get_mut("info") {
        if let CanonValue::Object(fields) = info {
            strip_provider_metadata(fields, &mut removed, &mut unproven);
            strip_named_opaque_fields(fields, &mut removed, &mut unproven);
            let prior_len = fields.len();
            retain_wiping(fields, |name, _| {
                matches!(
                    name.as_str(),
                    "id" | "slug"
                        | "parentID"
                        | "projectID"
                        | "workspaceID"
                        | "agent"
                        | "model"
                        | "cost"
                        | "tokens"
                        | "outcome"
                        | "time"
                        | "title"
                        | "location"
                        | "directory"
                        | "path"
                        | "subpath"
                        | "version"
                )
            });
            if fields.len() != prior_len {
                removed = true;
                unproven = true;
            }
            project_scalar_fields(
                fields,
                &[
                    ("id", ScalarShape::String),
                    ("slug", ScalarShape::String),
                    ("parentID", ScalarShape::String),
                    ("projectID", ScalarShape::String),
                    ("workspaceID", ScalarShape::String),
                    ("agent", ScalarShape::String),
                    ("cost", ScalarShape::Number),
                    ("outcome", ScalarShape::String),
                    ("title", ScalarShape::String),
                    ("directory", ScalarShape::String),
                    ("path", ScalarShape::String),
                    ("subpath", ScalarShape::String),
                    ("version", ScalarShape::String),
                ],
                &mut removed,
                &mut unproven,
            );
            project_envelope_object(
                fields,
                "model",
                &["id", "providerID", "variant"],
                &mut removed,
                &mut unproven,
            );
            project_envelope_object(
                fields,
                "location",
                &["directory"],
                &mut removed,
                &mut unproven,
            );
            project_envelope_object(
                fields,
                "time",
                &[
                    "created",
                    "updated",
                    "compacting",
                    "idle",
                    "viewed",
                    "archived",
                ],
                &mut removed,
                &mut unproven,
            );
            if classic_root {
                project_classic_integer_time(
                    fields,
                    &["created", "updated", "compacting"],
                    &mut removed,
                    &mut unproven,
                );
            }
            project_envelope_object(
                fields,
                "tokens",
                &["input", "output", "reasoning", "cache"],
                &mut removed,
                &mut unproven,
            );
            if let Some(CanonValue::Object(tokens)) = fields.get_mut("tokens") {
                project_envelope_object(
                    tokens,
                    "cache",
                    &["read", "write"],
                    &mut removed,
                    &mut unproven,
                );
            }
            let prior_len = fields.len();
            retain_wiping(fields, |_, value| !contains_opaque_hint(value));
            if fields.len() != prior_len {
                removed = true;
                unproven = true;
            }
            for value in fields.values_mut() {
                scrub_nested_envelope_value(value, &mut removed, &mut unproven);
            }
        } else {
            remove_wiping(document, "info");
            removed = true;
            unproven = true;
        }
    }
    let Some(CanonValue::Array(messages)) = document.get_mut("messages") else {
        return Err(ReasoningFieldError::MalformedSource);
    };
    let mut seen_messages = WipingIdSet(BTreeSet::new());
    let mut seen_parts = WipingIdSet(BTreeSet::new());
    for message in messages {
        let CanonValue::Object(fields) = message else {
            // The remaining messages can still contribute safe coverage.
            message.zeroize();
            removed = true;
            unproven = true;
            continue;
        };
        if let Some(provider_context) = take_wiping(fields, "providerContext") {
            removed = true;
            unproven |= contains_opaque_hint(&provider_context);
            if !matches!(&*provider_context, CanonValue::Object(_) | CanonValue::Null) {
                unproven = true;
            }
        }
        let kind = match fields.get("type") {
            None => None,
            Some(CanonValue::Str(kind)) => Some(Zeroizing::new(kind.clone())),
            Some(_) => {
                // Without a valid type, no envelope field is safe to retain.
                message.zeroize();
                removed = true;
                unproven = true;
                continue;
            }
        };
        let native = kind.is_some();
        if (native && (fields.contains_key("info") || fields.contains_key("parts")))
            || (!native && fields.contains_key("content"))
        {
            message.zeroize();
            removed = true;
            unproven = true;
            continue;
        }
        if native {
            inspect_id_field(fields, &mut seen_messages, &mut removed, &mut unproven);
        } else if let Some(CanonValue::Object(info)) = fields.get_mut("info") {
            inspect_id_field(info, &mut seen_messages, &mut removed, &mut unproven);
        }
        let assistant = if native {
            kind.as_ref().map(|kind| kind.as_str()) == Some("assistant")
        } else {
            let role = fields
                .get("info")
                .and_then(|info| info.get("role"))
                .and_then(CanonValue::as_str);
            let Some(role) = role else {
                // The classic role is the only proof that its payload is
                // ordinary user text rather than assistant provider state.
                message.zeroize();
                removed = true;
                unproven = true;
                continue;
            };
            if !matches!(role, "user" | "assistant") {
                message.zeroize();
                removed = true;
                unproven = true;
                continue;
            }
            role == "assistant"
        };
        if !assistant {
            if native
                && !kind
                    .as_ref()
                    .is_some_and(|kind| known_native_nonassistant(kind.as_str()))
            {
                let prior_len = fields.len();
                retain_wiping(fields, |name, _| matches!(name.as_str(), "id" | "type"));
                removed |= fields.len() != prior_len;
                unproven = true;
                continue;
            }
            if native {
                strip_provider_metadata(fields, &mut removed, &mut unproven);
                strip_named_opaque_fields(fields, &mut removed, &mut unproven);
                retain_native_nonassistant_envelope(
                    fields,
                    kind.as_ref().map(|kind| kind.as_str()).unwrap_or_default(),
                    &mut removed,
                    &mut unproven,
                );
            } else {
                scrub_assistant_message_hints(fields, false, &mut removed, &mut unproven);
            }
            let part_key = if native { "content" } else { "parts" };
            match fields.get_mut(part_key) {
                Some(CanonValue::Array(parts)) => {
                    for part in parts {
                        let CanonValue::Object(part_fields) = part else {
                            part.zeroize();
                            removed = true;
                            unproven = true;
                            continue;
                        };
                        if part_fields
                            .get("type")
                            .and_then(CanonValue::as_str)
                            .is_none()
                        {
                            part.zeroize();
                            removed = true;
                            unproven = true;
                            continue;
                        }
                        if part_fields.get("type").and_then(CanonValue::as_str) == Some("reasoning")
                            || has_opaque_hint_on_nonreasoning(part_fields)
                            || part_fields.values().any(contains_opaque_hint)
                        {
                            part.zeroize();
                            removed = true;
                            unproven = true;
                            continue;
                        }
                        strip_provider_metadata(part_fields, &mut removed, &mut unproven);
                        let removed_context = remove_wiping(part_fields, "providerContext");
                        let removed_state = remove_wiping(part_fields, "state");
                        if removed_context || removed_state {
                            removed = true;
                            unproven = true;
                        }
                        let text_part =
                            part_fields.get("type").and_then(CanonValue::as_str) == Some("text");
                        if text_part && !matches!(part_fields.get("text"), Some(CanonValue::Str(_)))
                        {
                            part.zeroize();
                            removed = true;
                            unproven = true;
                            continue;
                        }
                        if !native
                            && !text_part
                            && known_nonreasoning_part(
                                part_fields
                                    .get("type")
                                    .and_then(CanonValue::as_str)
                                    .unwrap_or_default(),
                                true,
                            )
                        {
                            let part_type = Zeroizing::new(
                                part_fields
                                    .get("type")
                                    .and_then(CanonValue::as_str)
                                    .unwrap_or_default()
                                    .to_owned(),
                            );
                            if part_type.as_str() == "tool" {
                                part.zeroize();
                                removed = true;
                                unproven = true;
                                continue;
                            }
                            project_classic_part(
                                part_fields,
                                part_type.as_str(),
                                &mut removed,
                                &mut unproven,
                            );
                        } else {
                            let prior_len = part_fields.len();
                            retain_wiping(part_fields, |name, _| {
                                matches!(name.as_str(), "id" | "type" | "time")
                                    || (text_part
                                        && matches!(
                                            name.as_str(),
                                            "text"
                                                | "synthetic"
                                                | "ignored"
                                                | "sessionID"
                                                | "messageID"
                                        ))
                            });
                            if part_fields.len() != prior_len {
                                removed = true;
                                unproven = true;
                            }
                            project_scalar_fields(
                                part_fields,
                                &[
                                    ("id", ScalarShape::String),
                                    ("type", ScalarShape::String),
                                    ("sessionID", ScalarShape::String),
                                    ("messageID", ScalarShape::String),
                                    ("text", ScalarShape::String),
                                    ("synthetic", ScalarShape::Boolean),
                                    ("ignored", ScalarShape::Boolean),
                                ],
                                &mut removed,
                                &mut unproven,
                            );
                            if !native {
                                project_envelope_object(
                                    part_fields,
                                    "time",
                                    &["start", "end"],
                                    &mut removed,
                                    &mut unproven,
                                );
                                project_classic_integer_time(
                                    part_fields,
                                    &["start", "end"],
                                    &mut removed,
                                    &mut unproven,
                                );
                            } else if remove_wiping(part_fields, "time") {
                                removed = true;
                                unproven = true;
                            }
                            scrub_retained_envelope_fields(
                                part_fields,
                                "",
                                &mut removed,
                                &mut unproven,
                            );
                        }
                    }
                }
                Some(_) => {
                    removed |= remove_wiping(fields, part_key);
                    unproven = true;
                }
                None => {}
            }
            continue;
        }
        scrub_assistant_message_hints(fields, native, &mut removed, &mut unproven);
        let part_key = if native { "content" } else { "parts" };
        match fields.get_mut(part_key) {
            Some(CanonValue::Array(parts)) => {
                scrub_parts(parts, !native, &mut seen_parts, &mut removed, &mut unproven)?;
            }
            Some(_) => {
                removed |= remove_wiping(fields, part_key);
                unproven = true;
            }
            None => unproven = true,
        }
    }
    if unproven {
        tracing::warn!(
            provider = "opencode",
            reason = "unsupported_reasoning_shape",
            "reasoning source is partial"
        );
    }
    let ordinary_partial_eligible = unproven
        && !classic_root
        && !original_origin_proof
        && candidates.is_none()
        && original_source_session.is_some()
        && retained_meaningful_ordinary_message(document);
    let ordinary = if removed {
        let cap = usize::try_from(TRANSCRIPT_READ_HARD_CAP_BYTES)
            .map_err(|_| ReasoningFieldError::SourceTooLarge)?;
        super::coverage::encode_bounded_json(&*export, cap)?
    } else {
        std::mem::take(&mut *source)
    };
    if let Some(candidates) = candidates {
        Ok(PreparedReasoningSource::with_opencode_candidates(
            ordinary, candidates,
        ))
    } else if ordinary_partial_eligible {
        let source_session = original_source_session.ok_or(ReasoningFieldError::MalformedSource)?;
        Ok(PreparedReasoningSource::with_verified_opencode_partial(
            ordinary,
            source_session,
        ))
    } else if unproven {
        Ok(PreparedReasoningSource::unproven_candidates(ordinary))
    } else {
        Ok(PreparedReasoningSource::without_candidates(ordinary))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::internal::ai::observed_agents::{
        coverage::normalize_opencode_export,
        extract::extract_opencode,
        reasoning::{ReasoningSourceDisposition, ReasoningSourceState},
    };

    fn cov2_classify_part(
        part: serde_json::Value,
        classic: bool,
    ) -> OpenCodeCoverageClassification {
        let message = if classic {
            serde_json::json!({"info":{"id":"msg_cov2","role":"assistant"},"parts":[part]})
        } else {
            serde_json::json!({"id":"msg_cov2","type":"assistant","content":[part]})
        };
        classify_opencode_coverage_source(
            serde_json::to_vec(&serde_json::json!({"info":{"id":"ses_cov2"},"messages":[message]}))
                .expect("synthetic export"),
        )
    }

    #[test]
    fn cov2_visible_only_classification() {
        for classic in [false, true] {
            let result = cov2_classify_part(
                serde_json::json!({"id":"prt_cov2","type":"reasoning","text":"visible 中文"}),
                classic,
            );
            let OpenCodeCoverageClassification::Parsed(source) = result else {
                panic!("closed visible source must classify");
            };
            assert!(!source.partial);
            assert!(prepared_source_is_ordinary_projection(&source.prepared));
            assert_eq!(source.source_session.as_str(), "ses_cov2");
            assert_eq!(source.visible_parts.len(), 1);
            let part = &source.visible_parts[0];
            assert_eq!(part.source_message.as_str(), "msg_cov2");
            assert_eq!(part.message_ordinal, 0);
            assert_eq!(part.source_part.as_str(), "prt_cov2");
            assert_eq!(part.part_ordinal, 0);
            assert!(matches!(&part.record, SemanticRecord::Reasoning {
                provider: ReasoningProvider::OpenCode,
                source_kind: Some(ReasoningSourceKind::Reasoning), text,
            } if text.as_str() == "visible 中文"));
            assert!(
                serde_json::to_string(&part.record).is_err(),
                "unredacted typed text must retain its serialization barrier"
            );
            assert!(!format!("{:?}", part.record).contains("visible 中文"));

            for text in ["", " \n\t"] {
                let result = cov2_classify_part(
                    serde_json::json!({"id":"prt_empty","type":"reasoning","text":text}),
                    classic,
                );
                let OpenCodeCoverageClassification::Parsed(source) = result else {
                    panic!("empty readable source remains explicit partial");
                };
                assert!(source.partial);
                assert!(source.visible_parts.is_empty());
            }
        }
    }

    #[test]
    fn cov2_partial_source_keeps_independent_visible_part() {
        for unknown_root in [false, true] {
            let mut source = serde_json::json!({"info":{"id":"ses_mixed"},"messages":[
                {"id":"m","type":"assistant","content":[
                    {"id":"good","type":"reasoning","text":"readable"},
                    {"id":"bad","type":"reasoning","text":"PRIVATE_CANDIDATE","state":{"signature":"PRIVATE_CANDIDATE"}}]}]});
            if unknown_root {
                source["unknown_root"] = serde_json::json!("PRIVATE_METADATA");
            }
            let OpenCodeCoverageClassification::Parsed(result) = classify_opencode_coverage_source(
                serde_json::to_vec(&source).expect("synthetic mixed source"),
            ) else {
                panic!("known structure retains safe visible data with a partial flag");
            };
            assert!(result.partial);
            assert_eq!(result.visible_parts.len(), 1);
            assert_eq!(result.visible_parts[0].source_part.as_str(), "good");
            assert!(
                matches!(&result.visible_parts[0].record, SemanticRecord::Reasoning { text, .. }
                if text.as_str() == "readable")
            );
        }
    }

    #[test]
    fn cov2_candidate_and_unsupported_partial() {
        for classic in [false, true] {
            for extra in [
                serde_json::json!({"state":{"signature":"CANDIDATE_CANARY"}}),
                serde_json::json!({"state":{"redactedData":"CANDIDATE_CANARY"}}),
                serde_json::json!({"state":{"reasoningEncryptedContent":"CANDIDATE_CANARY"}}),
                serde_json::json!({"originProof":{"nonce":"CANDIDATE_CANARY"}}),
                serde_json::json!({"state":{}}),
                serde_json::json!({"state":null}),
                serde_json::json!({"providerMetadata":{"signature":"CANDIDATE_CANARY"}}),
                serde_json::json!({"extra":"CANDIDATE_CANARY"}),
                serde_json::json!({"time":{"unexpected":"CANDIDATE_CANARY"}}),
                serde_json::json!({"sessionID":"foreign_session"}),
                serde_json::json!({"messageID":"foreign_message"}),
            ] {
                let mut part = serde_json::json!({"id":"prt_cov2","type":"reasoning","text":"CANDIDATE_CANARY"});
                part.as_object_mut()
                    .expect("object")
                    .extend(extra.as_object().expect("extra object").clone());
                match cov2_classify_part(part, classic) {
                    OpenCodeCoverageClassification::Parsed(source) => {
                        assert!(source.partial);
                        assert!(
                            source.visible_parts.is_empty(),
                            "candidate cannot become typed text"
                        );
                    }
                    OpenCodeCoverageClassification::Partial(_) => {}
                }
            }
        }
    }

    #[test]
    fn cov2_malformed_duplicate_unknown_partial() {
        for bytes in [
            b"{".as_slice(),
            br#"{"info":{"id":"ses_cov2"},"messages":[],"messages":[]}"#,
            br#"{"info":{"id":"ses_cov2"},"messages":[{"id":"m","type":"assistant","content":[{"id":"p","type":"reasoning","text":"a","text":"b"}]}]}"#,
            br#"{"info":{"id":"ses_cov2"},"messages":{}}"#,
            br#"{"messages":[]}"#,
            br#"{"info":{"id":"ses_cov2"},"messages":[{"id":"m","type":"assistant","parts":[],"content":[]}]}"#,
            br#"{"info":{"id":"ses_cov2"},"messages":[{"id":"m","type":"assistant","content":[{"id":"p","type":"reasoning","text":"a"},{"id":"p","type":"reasoning","text":"b"}]}]}"#,
            br#"{"info":{"id":"ses_cov2"},"messages":[{"id":"m","type":"assistant","content":[{"id":"p","type":"reasoning","text":"a"}]},{"id":"m","type":"assistant","content":[{"id":"q","type":"reasoning","text":"b"}]}]}"#,
            br#"{"info":{"id":"ses_cov2"},"messages":[{"id":"m","type":"unknown","content":[]}]}"#,
        ] {
            match classify_opencode_coverage_source(bytes.to_vec()) {
                OpenCodeCoverageClassification::Partial(_) => {}
                OpenCodeCoverageClassification::Parsed(source) => {
                    assert!(source.partial);
                    assert!(source.visible_parts.is_empty());
                }
            }
        }
        for bytes in [
            vec![b' '; TRANSCRIPT_READ_HARD_CAP_BYTES as usize + 1],
            "[".repeat(MAX_JSON_DEPTH + 2).into_bytes(),
            format!("[{}]", "0,".repeat(OPENCODE_CANON_MAX_NODES)).into_bytes(),
        ] {
            assert!(matches!(
                classify_opencode_coverage_source(bytes),
                OpenCodeCoverageClassification::Partial(ReasoningFieldError::SourceTooLarge)
            ));
        }
        let parts = (0..=MAX_VISIBLE_REASONING_PARTS).map(|index| {
            serde_json::json!({"id":format!("p{index}"),"type":"reasoning","text":"visible"})
        }).collect::<Vec<_>>();
        let source = serde_json::to_vec(&serde_json::json!({
            "info":{"id":"ses_cap"},"messages":[{"id":"msg_cap","type":"assistant","content":parts}]
        }))
        .expect("synthetic bounded part-count input");
        assert!(matches!(
            classify_opencode_coverage_source(source),
            OpenCodeCoverageClassification::Partial(ReasoningFieldError::SourceTooLarge)
        ));
    }

    #[test]
    fn og26_structural_preflight_never_calls_parser_over_budget() {
        use std::cell::Cell;

        let mut bytes = Vec::with_capacity(2 * (OPENCODE_CANON_MAX_NODES + 1) + 10);
        bytes.push(b'[');
        for index in 0..=OPENCODE_CANON_MAX_NODES {
            if index != 0 {
                bytes.push(b',');
            }
            bytes.push(b'0');
        }
        bytes.extend_from_slice(b",INVALID]");
        let called = Cell::new(false);
        assert!(matches!(
            parse_bounded_canon_value(&bytes, |_| {
                called.set(true);
                parse_canon_value(b"null")
            }),
            Err(ReasoningFieldError::SourceTooLarge)
        ));
        assert!(!called.get());
        let deep = "[".repeat(200);
        assert!(matches!(
            parse_bounded_canon_value(deep.as_bytes(), |_| {
                called.set(true);
                parse_canon_value(b"null")
            }),
            Err(ReasoningFieldError::SourceTooLarge)
        ));
        assert!(!called.get());
    }

    #[test]
    fn og26_candidate_receipt_coordinates_are_unambiguous() {
        let candidate = r#"{"id":"npart_0123456789abcdef0123456789abcdef","type":"reasoning","text":"safe","state":{"signature":"private"},"originProof":{"nonce":"content"}}"#;
        let source = format!(
            r#"{{"info":{{"id":"ses_og26_fixture"}},"messages":[{{"id":"msg_og26","type":"assistant","parts":[{{"originProof":{{"nonce":"parts"}}}}],"content":[{candidate}]}}]}}"#
        );
        let (receipts, export) =
            parse_bounded_canon_value(source.as_bytes(), parse_canon_value).expect("strict source");
        assert_eq!(receipts.len(), 1);
        assert_eq!(
            &source.as_bytes()[receipts[0].start..receipts[0].end],
            br#"{"nonce":"content"}"#
        );
        assert!(
            collect_sealed_candidates(&export, source.as_bytes(), &receipts)
                .expect("invalid mixed native/classic cannot create a candidate")
                .is_none()
        );

        let source = source.replace(r#","parts":[{"originProof":{"nonce":"parts"}}]"#, "");
        let (receipts, export) = parse_bounded_canon_value(source.as_bytes(), parse_canon_value)
            .expect("strict native source");
        assert!(
            collect_sealed_candidates(&export, source.as_bytes(), &receipts)
                .expect("unique native coordinates")
                .is_some()
        );
        let duplicate_receipts = [
            ReceiptSpan {
                message_ordinal: 0,
                part_ordinal: 0,
                start: receipts[0].start,
                end: receipts[0].end,
            },
            ReceiptSpan {
                message_ordinal: 0,
                part_ordinal: 0,
                start: receipts[0].start,
                end: receipts[0].end,
            },
        ];
        assert!(matches!(
            collect_sealed_candidates(&export, source.as_bytes(), &duplicate_receipts),
            Err(ReasoningFieldError::MalformedSource)
        ));

        let duplicate_part = format!(
            r#"{{"info":{{"id":"ses_og26_fixture"}},"messages":[{{"id":"msg_og26","type":"assistant","content":[{candidate},{candidate}]}}]}}"#
        );
        let (receipts, export) =
            parse_bounded_canon_value(duplicate_part.as_bytes(), parse_canon_value)
                .expect("strict duplicate-ID source");
        assert!(
            collect_sealed_candidates(&export, duplicate_part.as_bytes(), &receipts)
                .expect("ambiguous IDs stay unproved")
                .is_none()
        );

        let duplicate_message = format!(
            r#"{{"info":{{"id":"ses_og26_fixture"}},"messages":[{{"id":"msg_og26","type":"assistant","content":[{candidate}]}},{{"id":"msg_og26","type":"user","text":"safe"}}]}}"#
        );
        let (receipts, export) =
            parse_bounded_canon_value(duplicate_message.as_bytes(), parse_canon_value)
                .expect("strict duplicate-message source");
        assert!(
            collect_sealed_candidates(&export, duplicate_message.as_bytes(), &receipts)
                .expect("ambiguous message coordinates stay unproved")
                .is_none()
        );

        let wrong_ref = candidate.replace(
            r#""type":"reasoning""#,
            r#""type":"reasoning","sessionID":"ses_other","messageID":"msg_og26""#,
        );
        let wrong_ref = format!(
            r#"{{"info":{{"id":"ses_og26_fixture"}},"messages":[{{"id":"msg_og26","type":"assistant","content":[{wrong_ref}]}}]}}"#
        );
        let (receipts, export) = parse_bounded_canon_value(wrong_ref.as_bytes(), parse_canon_value)
            .expect("strict inconsistent-reference source");
        assert!(
            collect_sealed_candidates(&export, wrong_ref.as_bytes(), &receipts)
                .expect("mismatched source coordinates stay unproved")
                .is_none()
        );
    }

    #[test]
    fn og26_malformed_candidate_coordinates_keep_safe_ordinary_copy() {
        let valid = r#"{"id":"npart_0123456789abcdef0123456789abcdef","type":"reasoning","text":"visible","state":{"signature":"private-signature"},"originProof":{"nonce":"private-proof"}}"#;
        let malformed = valid.replace("npart_", "prt_");
        for source in [
            export(&malformed),
            export(&format!("{valid},{malformed}")),
            export(valid).replace("ses_og25", &"s".repeat(1025)),
        ] {
            let (ordinary, state) = prepare(&source).expect("malformed candidate is partial");
            assert!(matches!(state, ReasoningSourceState::Unproven));
            let ordinary = String::from_utf8(ordinary).expect("ordinary copy is UTF-8");
            assert!(ordinary.contains("prompt"));
            assert!(!ordinary.contains("private-signature"));
            assert!(!ordinary.contains("private-proof"));
            assert!(!ordinary.contains("originProof"));
            assert!(!ordinary.contains("\"state\""));
        }
    }

    fn prepare(source: &str) -> Result<(Vec<u8>, ReasoningSourceState), ReasoningFieldError> {
        let prepared = prepare_authorized_source(source.as_bytes().to_vec())?;
        let (ordinary, disposition) = ReasoningSourceDisposition::from_prepared_authorized_source(
            prepared,
            true,
            16 * 1024 * 1024,
            "opencode",
            "ses_og25",
            Some("ses_og25"),
        )?;
        Ok((ordinary, disposition.state()))
    }

    const OG25_SCHEMA_SHAPED_NATIVE: &str = r#"{"info":{"id":"ses_native","projectID":"prj","model":{"id":"model","providerID":"provider","variant":"fast"},"cost":0,"tokens":{"input":1,"output":2,"reasoning":0,"cache":{"read":0,"write":0}},"time":{"created":1,"updated":2},"location":{"directory":"/project/src"},"parentID":"ses_parent","agent":"build","outcome":"succeeded","title":"safe session","subpath":"src"},"messages":[{"id":"u1","type":"user","time":{"created":1},"text":"go","files":[],"agents":[],"skills":[]},{"id":"m1","type":"model-switched","time":{"created":2},"model":{"id":"model","providerID":"provider","variant":"fast"},"previous":{"id":"old_model","providerID":"provider","variant":"slow"}},{"id":"c1","type":"compaction","time":{"created":3},"status":"completed","reason":"manual","model":{"id":"model","providerID":"provider","variant":"fast"},"summary":"safe","recent":"safe","cost":0,"tokens":{"input":1,"output":2,"reasoning":0,"cache":{"read":0,"write":0}}},{"id":"a1","type":"assistant","agent":"build","model":{"id":"model","providerID":"provider","variant":"fast"},"time":{"created":4,"streamed":5,"completed":6},"retry":{"attempt":1,"at":3,"error":{"type":"rate_limit","message":"retry","status":429}},"tokens":{"input":1,"output":2,"reasoning":0,"cache":{"read":0,"write":0}},"content":[{"id":"t1","type":"tool","name":"write","time":{"created":4,"ran":5,"completed":6},"state":{"status":"completed","input":{},"content":[{"type":"text","text":"ok"}],"metadata":{"truncated":false,"files":[{"file":"native-real.rs","diff":"ordinary tool output"}]}}},{"id":"v1","type":"text","text":"done"}]}]}"#;

    const OG25_SCHEMA_SHAPED_CLASSIC: &str = r#"{"info":{"id":"ses_classic","slug":"classic","projectID":"prj","workspaceID":"wrk","directory":"/project","path":"/project","version":"2.0.26","time":{"created":1,"updated":2,"compacting":3},"model":{"id":"model","providerID":"provider","variant":"fast"},"parentID":"ses_parent","agent":"build","title":"safe session","cost":0,"tokens":{"input":1,"output":2,"reasoning":0,"cache":{"read":0,"write":0}}},"messages":[{"info":{"id":"u1","sessionID":"ses_classic","role":"user","time":{"created":1},"agent":"build","model":{"providerID":"provider","modelID":"model","variant":"fast"},"system":"safe system","tools":{"write":true}},"parts":[{"id":"p1","sessionID":"ses_classic","messageID":"u1","type":"text","text":"go"},{"id":"f1","sessionID":"ses_classic","messageID":"u1","type":"file","mime":"text/plain","filename":"input.txt","url":"file:///tmp/input.txt","source":{"type":"file","path":"input.txt","text":{"value":"input","start":0,"end":5}}}]},{"info":{"id":"a1","sessionID":"ses_classic","role":"assistant","time":{"created":2,"completed":3},"parentID":"u1","modelID":"model","providerID":"provider","mode":"build","agent":"build","path":{"cwd":"/project","root":"/project"},"cost":0,"tokens":{"total":3,"input":1,"output":2,"reasoning":0,"cache":{"read":0,"write":0}},"variant":"fast","finish":"stop","error":{"name":"ProviderAuthError","data":{"providerID":"provider","message":"safe error"}}},"parts":[{"id":"s1","sessionID":"ses_classic","messageID":"a1","type":"step-start","snapshot":"start"},{"id":"t1","sessionID":"ses_classic","messageID":"a1","type":"tool","callID":"call_1","tool":"write","state":{"status":"completed","input":{},"output":"ok","title":"write file","time":{"start":2,"end":3},"metadata":{"truncated":false,"files":[{"file":"src/classic-real.rs","diff":"ordinary tool output"}]},"attachments":[{"id":"af1","sessionID":"ses_classic","messageID":"a1","type":"file","mime":"text/plain","url":"file:///tmp/output.txt"}]}},{"id":"v1","sessionID":"ses_classic","messageID":"a1","type":"text","text":"done"},{"id":"sf1","sessionID":"ses_classic","messageID":"a1","type":"step-finish","reason":"stop","snapshot":"end","cost":0,"tokens":{"total":3,"input":1,"output":2,"reasoning":0,"cache":{"read":0,"write":0}}}]}]}"#;
    const OG25_NATIVE_FILE_CONTENT: &str = r#"{"info":{"id":"ses_native_file"},"messages":[{"id":"msg_a","type":"assistant","content":[{"id":"call_0","type":"tool","name":"read","state":{"status":"completed","input":{},"content":[{"type":"text","text":"safe result"},{"type":"file","uri":"file:///tmp/result.txt","mime":"text/plain","name":"result.txt"}]}}]}]}"#;
    const OG25_CLASSIC_PENDING_RAW: &str = r#"{"info":{"id":"ses_classic_pending"},"messages":[{"info":{"id":"msg_a","role":"assistant"},"parts":[{"id":"part_t","type":"tool","tool":"read","callID":"call_0","state":{"status":"pending","input":{},"raw":"safe pending command"}}]}]}"#;
    const OG25_CLASSIC_ERROR_DATA: &str = r#"{"info":{"id":"ses_classic_error"},"messages":[{"info":{"id":"msg_u","role":"user"},"parts":[{"id":"part_u","type":"text","text":"safe prompt"}]},{"info":{"id":"msg_a","role":"assistant","error":{"name":"APIError","data":{"providerID":"provider","message":"rate limited","statusCode":429,"retries":1,"isRetryable":true}}},"parts":[{"id":"part_a","type":"text","text":"safe answer"}]}]}"#;
    const OG25_NATIVE_NONASSISTANT_BRANCHES: &str = r#"{"info":{"id":"ses_native_more"},"messages":[{"id":"u1","type":"user","text":"safe prompt","files":[],"agents":[{"name":"build","mention":{"start":0,"end":5,"text":"@build"}}],"skills":[{"id":"skill_1","name":"review","mention":{"start":6,"end":13,"text":"@review"}}],"time":{"created":1}},{"id":"ag1","type":"agent-switched","agent":"build","previous":"plan","time":{"created":2}},{"id":"loc1","type":"location-switched","location":{"directory":"/project"},"projectID":"prj","subpath":"src","previous":{"location":{"directory":"/old"},"projectID":"old","subpath":"oldsrc"},"time":{"created":3}},{"id":"sy1","type":"synthetic","text":"safe context","description":"safe summary","time":{"created":4}},{"id":"sys1","type":"system","text":"safe instruction","description":"safe note","time":{"created":5}},{"id":"sk1","type":"skill","skill":"skill_1","name":"review","text":"safe skill","time":{"created":6}},{"id":"sh1","type":"shell","shellID":"shell_1","command":"pwd","status":"exited","exit":0,"output":{"output":"/project","cursor":8,"size":8,"truncated":false},"time":{"created":7,"completed":8}},{"id":"idle1","type":"idle","outcome":"succeeded","time":{"created":9}}]}"#;
    const OG25_CLASSIC_PART_BRANCHES: &str = r#"{"info":{"id":"ses_classic_more"},"messages":[{"info":{"id":"u1","role":"user"},"parts":[{"id":"p1","type":"text","text":"safe prompt","synthetic":false,"ignored":false,"time":{"start":1,"end":2}},{"id":"f1","type":"file","mime":"text/plain","filename":"note.txt","url":"file:///tmp/note.txt","source":{"type":"symbol","text":{"value":"safe selected text","start":0,"end":4},"path":"/project/note.txt","range":{"start":{"line":1,"character":2},"end":{"line":3,"character":4}},"name":"safeSymbol","kind":1}},{"id":"f2","type":"file","mime":"text/plain","url":"file:///tmp/resource.txt","source":{"type":"resource","text":{"value":"safe uri","start":0,"end":8},"clientName":"local","uri":"file:///tmp/resource.txt"}}]},{"info":{"id":"a1","role":"assistant"},"parts":[{"id":"pa1","type":"patch","hash":"hash","files":["src/a.rs"]},{"id":"sn1","type":"snapshot","snapshot":"snapshot_1"},{"id":"ag1","type":"agent","name":"build","source":{"value":"@build","start":0,"end":6}},{"id":"su1","type":"subtask","prompt":"safe prompt","description":"safe task","agent":"build","model":{"providerID":"provider","modelID":"model"},"command":"safe command"},{"id":"co1","type":"compaction","auto":true,"overflow":false,"tail_start_id":"msg_a"},{"id":"re1","type":"retry","attempt":1,"error":{"name":"APIError","data":{"providerID":"provider","message":"retry","statusCode":429,"retries":1,"isRetryable":true}},"time":{"created":3}},{"id":"v1","type":"text","text":"safe answer"}]}]}"#;
    const OG25_NATIVE_OPTIONAL_BRANCHES: &str = r#"{"info":{"id":"ses_native_optional"},"messages":[{"id":"u1","type":"user","text":"safe prompt"},{"id":"cr1","type":"compaction","status":"running","reason":"auto","summary":"safe","recent":"safe","time":{"created":1}},{"id":"cf1","type":"compaction","status":"failed","reason":"auto","error":{"type":"provider","message":"safe error","status":429},"tokens":{"input":1,"output":2,"reasoning":0,"cache":{"read":0,"write":0}},"cost":0,"time":{"created":2}},{"id":"a1","type":"assistant","agent":"build","model":{"id":"model","providerID":"provider","variant":"fast"},"snapshot":{"start":"snap_start","end":"snap_end","files":["src/a.rs"]},"finish":"stop","rawFinish":"stop","cost":0,"tokens":{"input":1,"output":2,"reasoning":0,"cache":{"read":0,"write":0}},"error":{"type":"provider","message":"safe diagnostic","status":429},"time":{"created":3,"streamed":4,"completed":5},"content":[{"id":"tr1","type":"tool","name":"read","executed":true,"time":{"created":3,"ran":4},"state":{"status":"running","input":{},"metadata":{}}},{"id":"ts1","type":"tool","name":"read","executed":false,"time":{"created":4},"state":{"status":"streaming","input":"safe input"}},{"id":"te1","type":"tool","name":"read","time":{"created":4,"ran":5,"completed":6},"state":{"status":"error","input":{},"error":{"type":"provider","message":"safe tool error","status":500},"content":[{"type":"text","text":"safe error output"}]}}]}]}"#;
    const OG25_CLASSIC_TOOL_STATES: &str = r#"{"info":{"id":"ses_classic_states"},"messages":[{"info":{"id":"u1","role":"user"},"parts":[{"id":"p1","type":"text","text":"safe prompt"}]},{"info":{"id":"a1","role":"assistant"},"parts":[{"id":"tr1","type":"tool","tool":"read","callID":"call_r","state":{"status":"running","input":{},"title":"running","metadata":{},"time":{"start":1}}},{"id":"te1","type":"tool","tool":"read","callID":"call_e","state":{"status":"error","input":{},"error":"safe tool error","metadata":{},"time":{"start":2,"end":3}}},{"id":"v1","type":"text","text":"safe answer"}]}]}"#;
    const OG25_CLASSIC_FINITE_FILE_OFFSETS: &str = r#"{"info":{"id":"ses_classic_finite"},"messages":[{"info":{"id":"u1","role":"user"},"parts":[{"id":"f1","type":"file","mime":"text/plain","url":"file:///tmp/x","source":{"type":"file","path":"x","text":{"value":"safe selected","start":0.5,"end":4.5}}}]}]}"#;

    fn export(part: &str) -> String {
        format!(
            r#"{{"info":{{"id":"ses_og25"}},"messages":[{{"id":"msg_u","type":"user","text":"prompt"}},{{"id":"msg_a","type":"assistant","content":[{part}]}}]}}"#
        )
    }

    #[derive(Clone, Copy, Debug)]
    enum ClosedProjectionProbe {
        Object,
        Array,
        Leaf,
    }

    fn collect_closed_projection_paths(
        value: &serde_json::Value,
        pointer: String,
        paths: &mut Vec<(String, ClosedProjectionProbe)>,
    ) {
        match value {
            serde_json::Value::Object(fields) => {
                paths.push((pointer.clone(), ClosedProjectionProbe::Object));
                for (name, child) in fields {
                    let escaped = name.replace('~', "~0").replace('/', "~1");
                    collect_closed_projection_paths(child, format!("{pointer}/{escaped}"), paths);
                }
            }
            serde_json::Value::Array(items) => {
                paths.push((pointer.clone(), ClosedProjectionProbe::Array));
                for (index, child) in items.iter().enumerate() {
                    collect_closed_projection_paths(child, format!("{pointer}/{index}"), paths);
                }
            }
            _ => paths.push((pointer, ClosedProjectionProbe::Leaf)),
        }
    }

    fn is_closed_projection_content_exception(pointer: &str) -> bool {
        // This is the entire ordinary-content exception allowlist. Visible
        // text, native tool content and classic raw/output are schema-typed
        // strings; replacing them with an object must still fail closed.
        // Tool input alone is free-form content.
        let segments: Vec<_> = pointer.split('/').collect();
        segments
            .windows(2)
            .any(|pair| pair[0] == "state" && pair[1] == "input")
    }

    fn is_closed_projection_integer_slot(pointer: &str, classic: bool) -> bool {
        let leaf = pointer.rsplit('/').next().unwrap_or_default();
        matches!(leaf, "statusCode" | "retries" | "kind" | "cursor" | "size")
            || (classic && leaf == "attempt")
            || (leaf == "status" && pointer.contains("/error/"))
            || (!classic && pointer.ends_with("/retry/attempt"))
            || (classic
                && pointer.contains("/time/")
                && matches!(
                    leaf,
                    "created"
                        | "updated"
                        | "compacting"
                        | "completed"
                        | "start"
                        | "end"
                        | "compacted"
                ))
            || (classic
                && pointer.contains("/source/")
                && !pointer.contains("/source/text/")
                && matches!(leaf, "start" | "end" | "line" | "character"))
    }

    fn check_closed_projection_probe(
        injected: &serde_json::Value,
        label: &str,
        canary: &str,
        source_session: &str,
        allow_malformed: bool,
        failures: &mut Vec<String>,
    ) {
        use crate::internal::ai::{
            capture::snapshot::{
                AuthorizedReasoningSourceContext, CaptureSnapshotCompleteness,
                CaptureSnapshotPolicy, CaptureSnapshotService,
            },
            observed_agents::{
                AgentKind, agent_for,
                transcript_source::{ExportAuthorized, TranscriptSource},
            },
        };

        let bytes = serde_json::to_vec(injected).expect("probe JSON serializes");
        let source = std::str::from_utf8(&bytes).expect("JSON is UTF-8");
        match prepare(source) {
            Ok((ordinary, state)) => {
                if !matches!(state, ReasoningSourceState::Unproven) {
                    failures.push(format!("{label}: classifier stayed Ordinary"));
                }
                if String::from_utf8_lossy(&ordinary).contains(canary) {
                    failures.push(format!("{label}: ordinary retained canary"));
                }
            }
            Err(ReasoningFieldError::MalformedSource) if allow_malformed => {}
            Err(ReasoningFieldError::MalformedSource) => {
                failures.push(format!("{label}: parseable nested probe became malformed"));
            }
            Err(error) => failures.push(format!("{label}: unexpected {error:?}")),
        }
        let auth = ExportAuthorized::issue("opencode", "ses_capture", &bytes);
        let snapshot = CaptureSnapshotService::capture_authorized_with_reasoning_context(
            TranscriptSource::Bytes { bytes, auth },
            agent_for(AgentKind::OpenCode),
            AuthorizedReasoningSourceContext::authorized_export(
                "opencode",
                "ses_capture",
                source_session,
            ),
            CaptureSnapshotPolicy::default(),
        );
        if snapshot.completeness() != CaptureSnapshotCompleteness::Partial {
            failures.push(format!("{label}: snapshot stayed Complete"));
        }
        let transcript = snapshot
            .transcript()
            .map(|value| String::from_utf8_lossy(value.bytes()).into_owned())
            .unwrap_or_default();
        let projection =
            serde_json::to_string(&snapshot.safe_projection()).expect("safe projection serializes");
        let debug = format!("{snapshot:?}");
        for (sink, rendered) in [
            ("transcript", transcript.as_str()),
            ("projection", projection.as_str()),
            ("Debug", debug.as_str()),
        ] {
            if rendered.contains(canary) {
                failures.push(format!("{label}: {sink} retained canary"));
            }
        }
        if snapshot.try_into_ordinary().is_ok() {
            failures.push(format!("{label}: ordinary conversion succeeded"));
        }
    }

    #[test]
    fn og25_reasoning_state_truth_table_is_closed() {
        for (part, expected_ordinary) in [
            (
                r#"{"id":"part_1","type":"reasoning","text":"visible"}"#,
                true,
            ),
            (
                r#"{"id":"part_1","type":"reasoning","text":"visible","time":{"created":1,"completed":2}}"#,
                true,
            ),
            (r#"{"id":"part_1","type":"reasoning","text":""}"#, true),
            (
                r#"{"id":"part_1","type":"reasoning","text":"visible","time":{"start":1,"signature":"CIPHER_TIME_HINT"}}"#,
                false,
            ),
            (
                r#"{"id":"part_1","type":"reasoning","text":"visible","time":{"start":1,"unknown":"UNPROVEN_TIME"}}"#,
                false,
            ),
            (
                r#"{"id":"part_1","type":"reasoning","text":"visible","state":{}}"#,
                false,
            ),
            (
                r#"{"id":"part_1","type":"reasoning","text":"visible","state":{"signature":"CIPHER_SIG"}}"#,
                false,
            ),
            (
                r#"{"id":"part_1","type":"reasoning","text":"visible","state":{"redactedData":"CIPHER_REDACTED"}}"#,
                false,
            ),
            (
                r#"{"id":"part_1","type":"reasoning","text":"visible","state":{"reasoningEncryptedContent":"CIPHER_ENCRYPTED"}}"#,
                false,
            ),
            (
                r#"{"id":"part_1","type":"reasoning","text":"visible","state":{"signature":"CIPHER_SIG","redactedData":"CIPHER_REDACTED","reasoningEncryptedContent":"CIPHER_ENCRYPTED"}}"#,
                false,
            ),
            (
                r#"{"id":"part_1","type":"reasoning","text":"visible","state":{"signature":""}}"#,
                false,
            ),
            (
                r#"{"id":"part_1","type":"reasoning","text":"visible","state":{"signature":null}}"#,
                false,
            ),
            (
                r#"{"id":"part_1","type":"reasoning","text":"visible","state":{"signature":"CIPHER_SIG","unexpected":"CIPHER_EXTRA"}}"#,
                false,
            ),
            (
                r#"{"id":"part_1","type":"reasoning","text":"visible","state":"CIPHER_STATE"}"#,
                false,
            ),
            (
                r#"{"id":"part_1","type":"reasoning","state":{"signature":"CIPHER_SIG"}}"#,
                false,
            ),
            (
                r#"{"id":"part_1","type":"reasoning","text":"visible","originProof":null}"#,
                false,
            ),
            (
                r#"{"id":"part_1","type":"reasoning","text":{"signature":"CIPHER_TEXT_OBJECT"}}"#,
                false,
            ),
        ] {
            let input = export(part);
            let (ordinary, state) = prepare(&input).expect("synthetic export is well formed");
            assert_eq!(
                matches!(state, ReasoningSourceState::Ordinary),
                expected_ordinary
            );
            if !expected_ordinary {
                let ordinary = String::from_utf8(ordinary).expect("JSON is UTF-8");
                assert!(
                    !ordinary.contains(r#""text":"visible""#),
                    "candidate or unsupported reasoning retained its text"
                );
                for canary in [
                    "CIPHER_SIG",
                    "CIPHER_REDACTED",
                    "CIPHER_ENCRYPTED",
                    "CIPHER_EXTRA",
                    "CIPHER_STATE",
                    "CIPHER_TEXT_OBJECT",
                    "CIPHER_TIME_HINT",
                    "UNPROVEN_TIME",
                    "originProof",
                ] {
                    assert!(!ordinary.contains(canary), "normal copy retained {canary}");
                }
            }
        }
    }

    #[test]
    fn og25_provider_context_and_sdk_metadata_are_not_authority() {
        let input = r#"{"info":{"id":"ses_og25"},"messages":[{"id":"msg_u","type":"user","text":"prompt","signature":"USER_SIGNATURE","providerContext":{"messages":[{"private":"OPAQUE_WINDOW"}],"keep":"ordinary"}},{"id":"msg_a","type":"assistant","content":[{"id":"part_1","type":"reasoning","text":"visible","providerMetadata":{"signature":"SDK_ONLY"}}]}]}"#;
        let (ordinary, state) = prepare(input).expect("valid OpenCode export");
        assert!(matches!(state, ReasoningSourceState::Unproven));
        let ordinary = String::from_utf8(ordinary).expect("JSON is UTF-8");
        assert!(!ordinary.contains("OPAQUE_WINDOW"));
        let value: serde_json::Value = serde_json::from_str(&ordinary).expect("JSON is valid");
        assert!(
            value["messages"][0]["providerContext"]
                .get("messages")
                .is_none()
        );
        assert!(!ordinary.contains("ordinary"));
        assert!(ordinary.contains("prompt"));
        assert!(!ordinary.contains("USER_SIGNATURE"));
        assert!(!ordinary.contains("SDK_ONLY"));
    }

    #[test]
    fn og25_invalid_document_and_duplicate_keys_fail_closed() {
        let duplicate_key = export(
            r#"{"id":"part_1","type":"reasoning","text":"visible","state":{"signature":"one","signature":"two"}}"#,
        );
        assert!(matches!(
            prepare(&duplicate_key),
            Err(ReasoningFieldError::MalformedSource)
        ));
        for malformed in [r#"{"messages":{}}"#, r#"{"messages":["#] {
            assert!(prepare(malformed).is_err(), "accepted malformed source");
        }
        // The standalone metadata extractor still accepts these legacy
        // best-effort shapes, but capture cannot authorize them as an export.
        for non_export in [
            r#"{"role":"user","content":"CIPHER_FLAT"}"#,
            "{\"role\":\"user\",\"content\":\"CIPHER_JSONL_1\"}\n{\"role\":\"assistant\",\"content\":\"CIPHER_JSONL_2\"}",
            r#"{"info":{"id":"ses_og25"},"parts":[]}"#,
        ] {
            assert!(matches!(
                prepare(non_export),
                Err(ReasoningFieldError::MalformedSource)
            ));
        }
    }

    #[test]
    fn og25_ambiguous_parts_are_dropped_with_safe_partial() {
        for (ambiguous, canary) in [
            (
                r#"{"messages":[{"id":"msg","type":"assistant","content":[{"id":"part","type":"text","text":"visible","state":{"signature":"CIPHER_SIG"}}]}]}"#,
                "CIPHER_SIG",
            ),
            (
                r#"{"messages":[{"id":"msg","type":"assistant","content":[{"id":"part","type":"text","text":"visible","originProof":{"signature":"CIPHER_SIG"}}]}]}"#,
                "CIPHER_SIG",
            ),
            (
                r#"{"messages":[{"id":"msg","type":"user","content":[{"id":"part","type":"reasoning","text":"visible","state":{"signature":"CIPHER_SIG"}}]}]}"#,
                "CIPHER_SIG",
            ),
            (
                r#"{"messages":[{"id":"msg","type":"assistant","parts":[{"type":"reasoning","state":{"signature":"CIPHER_SIG"}}],"content":[]}]}"#,
                "CIPHER_SIG",
            ),
            (
                r#"{"messages":[{"id":"msg","type":"assistant","content":[{"id":"part","type":"thinking","text":"CIPHER_SIG"}]}]}"#,
                "CIPHER_SIG",
            ),
            (
                r#"{"messages":[{"id":"msg","type":"assistant","signature":"CIPHER_SIG","content":[]}]}"#,
                "CIPHER_SIG",
            ),
            (
                r#"{"messages":[{"id":"msg","type":"assistant","content":[{"id":"part","type":"text","text":"visible","unknown":{"signature":"CIPHER_SIG"}}]}]}"#,
                "CIPHER_SIG",
            ),
        ] {
            let (ordinary, state) = prepare(ambiguous).expect("ambiguous part is safe partial");
            assert!(matches!(state, ReasoningSourceState::Unproven));
            assert!(!String::from_utf8_lossy(&ordinary).contains(canary));
        }
    }

    #[test]
    fn og25_real_classic_text_shape_keeps_visible_capture() {
        let input = r#"{"info":{"id":"ses_og25"},"messages":[{"info":{"id":"msg_a","sessionID":"ses_og25","role":"assistant","time":{"created":1},"parentID":"msg_u","modelID":"model","providerID":"provider","mode":"build","agent":"build","path":{"cwd":"/tmp","root":"/tmp"},"cost":0,"tokens":{"input":1,"output":1,"reasoning":0,"cache":{"read":0,"write":0}}},"parts":[{"id":"part_text","sessionID":"ses_og25","messageID":"msg_a","type":"text","text":"visible","synthetic":false,"ignored":false,"time":{"start":1,"end":2},"metadata":{"custom":"benign"}}]}]}"#;
        let (ordinary, state) = prepare(input).expect("classic text shape is accepted");
        assert!(matches!(state, ReasoningSourceState::Ordinary));
        let ordinary = String::from_utf8(ordinary).expect("JSON is UTF-8");
        assert!(ordinary.contains("visible"));
        assert!(ordinary.contains("sessionID"));
        assert!(!ordinary.contains("custom"));
    }

    #[test]
    fn og25_real_classic_reasoning_shape_retains_visible_only() {
        let bare = r#"{"messages":[{"info":{"id":"msg_a","sessionID":"ses_og25","role":"assistant"},"parts":[{"id":"part_r","sessionID":"ses_og25","messageID":"msg_a","type":"reasoning","text":"visible","time":{"start":1,"end":2}}]}]}"#;
        let (ordinary, state) = prepare(bare).expect("classic reasoning is accepted");
        assert!(matches!(state, ReasoningSourceState::Ordinary));
        let value: serde_json::Value = serde_json::from_slice(&ordinary).expect("valid JSON");
        assert_eq!(value["messages"][0]["parts"][0]["text"], "visible");
        assert!(value["messages"][0]["parts"][0].get("sessionID").is_none());
        assert!(value["messages"][0]["parts"][0].get("messageID").is_none());

        let with_metadata = bare.replace(
            r#""time":{"start":1,"end":2}"#,
            r#""time":{"start":1,"end":2},"metadata":{"anthropic":{"signature":"CLASSIC_CIPHER"}}"#,
        );
        let (ordinary, state) = prepare(&with_metadata).expect("classic SDK metadata is partial");
        assert!(matches!(state, ReasoningSourceState::Unproven));
        assert!(!String::from_utf8_lossy(&ordinary).contains("visible"));
        assert!(!String::from_utf8_lossy(&ordinary).contains("CLASSIC_CIPHER"));
    }

    #[test]
    fn og25_text_state_candidate_keeps_visible_text() {
        let input = export(
            r#"{"id":"part_text","type":"text","text":"visible answer","state":{"signature":"CIPHER_SIG"}}"#,
        );
        let (ordinary, state) = prepare(&input).expect("text state candidate is partial");
        assert!(matches!(state, ReasoningSourceState::Unproven));
        let ordinary = String::from_utf8(ordinary).expect("JSON is UTF-8");
        assert!(ordinary.contains("visible answer"));
        assert!(!ordinary.contains("CIPHER_SIG"));
    }

    #[test]
    fn og25_nonassistant_provider_metadata_is_removed() {
        let input = r#"{"messages":[{"id":"msg_compact","type":"compaction","status":"completed","summary":"safe","metadata":{"anthropic":{"redactedData":"CIPHER_BASE"}},"providerState":{"reasoningEncryptedContent":"CIPHER_COMPACTION"}},{"id":"msg_user","type":"user","text":"prompt"}]}"#;
        let (ordinary, state) = prepare(input).expect("compaction metadata is partial");
        assert!(matches!(state, ReasoningSourceState::Unproven));
        let ordinary = String::from_utf8(ordinary).expect("JSON is UTF-8");
        assert!(!ordinary.contains("CIPHER_BASE"));
        assert!(!ordinary.contains("CIPHER_COMPACTION"));
        assert!(ordinary.contains("safe"));
        assert!(ordinary.contains("prompt"));
    }

    #[test]
    fn og25_document_and_nonassistant_part_hints_have_zero_snapshot_sink_hits() {
        use crate::internal::ai::{
            capture::snapshot::{
                AuthorizedReasoningSourceContext, CaptureSnapshotCompleteness,
                CaptureSnapshotPolicy, CaptureSnapshotService,
            },
            observed_agents::{
                AgentKind, agent_for,
                transcript_source::{ExportAuthorized, TranscriptSource},
            },
        };

        for source in [
            r#"{"metadata":{"anthropic":{"signature":"CIPHER_ROOT"}},"debug":{"reasoningEncryptedContent":"CIPHER_EXTRA_ROOT"},"info":{"id":"ses_og25","directory":"/project","metadata":{"anthropic":{"redactedData":"CIPHER_INFO"}}},"messages":[{"info":{"id":"u1","role":"user","signature":"CIPHER_MESSAGE_INFO"},"parts":[{"id":"p1","type":"text","text":"CIPHER_USER","metadata":{"anthropic":{"signature":"CIPHER_USER"}}}]},{"info":{"id":"u2","role":"user"},"parts":[{"id":"p2","type":"text","text":"safe prompt","metadata":{"note":"benign"}}]}]}"#,
            r#"{"info":{"id":"ses_og25","metadata":{"providerState":{"signature":"CIPHER_INFO"}}},"messages":[{"id":"u1","type":"user","redactedData":"CIPHER_MESSAGE_INFO","content":[{"id":"p1","type":"text","text":"CIPHER_USER","providerMetadata":{"signature":"CIPHER_USER"}}]},{"id":"u2","type":"user","text":"safe prompt"}]}"#,
            r#"{"messages":[{"id":"u1","type":"user","text":"safe prompt","files":[{"file":"src/a.rs","metadata":{"anthropic":{"signature":"CIPHER_NESTED_USER_FILE"}}}]}]}"#,
            r#"{"messages":[{"info":{"id":"u1","role":"user","model":{"id":"CIPHER_NESTED_CLASSIC_MODEL","metadata":{"anthropic":{"signature":"CIPHER_NESTED_CLASSIC_MODEL"}}}},"parts":[{"id":"p1","type":"text","text":"safe prompt"}]}]}"#,
            r#"{"messages":[{"id":"u1","type":"user","text":"safe prompt"},{"id":"a1","type":"assistant","model":{"id":"CIPHER_NESTED_ASSISTANT_MODEL","providerID":"p","metadata":{"anthropic":{"signature":"CIPHER_NESTED_ASSISTANT_MODEL"}}},"error":{"type":"provider","metadata":{"redactedData":"CIPHER_ASSISTANT_ERROR"}},"content":[{"id":"p1","type":"text","text":"safe answer"}]}]}"#,
            r#"{"messages":[{"id":"u1","type":"user","text":"safe prompt"},{"id":"a1","type":"assistant","error":{"type":"provider","message":"safe diagnostic","status":500,"response":{"body":"CIPHER_NATIVE_RESPONSE"}},"content":[{"id":"p1","type":"text","text":"safe answer"}]}]}"#,
            r#"{"messages":[{"info":{"id":"u1","role":"user"},"parts":[{"id":"p1","type":"text","text":"safe prompt"}]},{"info":{"id":"a1","role":"assistant","error":{"name":"APIError","data":{"message":"safe diagnostic","isRetryable":true,"responseBody":"CIPHER_CLASSIC_RESPONSE"}}},"parts":[{"id":"p2","type":"text","text":"safe answer"}]}]}"#,
            r#"{"messages":[{"id":"u1","type":"user","text":"safe prompt"},{"id":"a1","type":"assistant","content":[{"id":"p1","type":"text","text":"safe answer","time":{"signature":"CIPHER_NATIVE_TEXT_TIME"},"ignored":{"redactedData":"CIPHER_NATIVE_TEXT_FLAG"}}]}]}"#,
            r#"{"messages":[{"info":{"id":"u1","role":"user"},"parts":[{"id":"p1","type":"text","text":"safe prompt"}]},{"info":{"id":"a1","role":"assistant"},"parts":[{"id":"p2","type":"text","text":"safe answer","time":{"start":1,"anthropic":{"signature":"CIPHER_CLASSIC_TEXT_TIME"}},"sessionID":{"reasoningEncryptedContent":"CIPHER_CLASSIC_TEXT_ID"},"synthetic":{"redactedData":"CIPHER_CLASSIC_TEXT_FLAG"}}]}]}"#,
            r#"{"messages":[{"id":"u1","type":"user","text":"safe prompt"},{"id":"u2","type":"user","text":{"signature":"CIPHER_USER_TEXT"}},{"id":"s1","type":"synthetic","text":{"redactedData":"CIPHER_SYNTHETIC_TEXT"}},{"id":"s2","type":"system","text":{"reasoningEncryptedContent":"CIPHER_SYSTEM_TEXT"}},{"id":"s3","type":"skill","text":{"signature":"CIPHER_SKILL_TEXT"}}]}"#,
            r#"{"messages":[{"id":"u1","type":"user","text":"safe prompt"},{"id":"a1","type":"assistant","content":[{"id":"t1","type":"tool","name":"read","state":{"status":"completed","input":{},"content":[{"type":"text","text":"safe result","providerMetadata":{"signature":"CIPHER_NATIVE_TOOL_CONTENT"}}]}}]}]}"#,
            r#"{"messages":[{"id":"u1","type":"user","text":"safe prompt"},{"id":"a1","type":"assistant","content":[{"id":"t1","type":"tool","name":"read","state":{"status":"completed","input":{},"raw":"CIPHER_NATIVE_TOOL_RAW","title":"CIPHER_NATIVE_TOOL_TITLE","attachments":[{"secret":"CIPHER_NATIVE_TOOL_ATTACH"}],"time":{"start":1,"private":"CIPHER_NATIVE_TOOL_TIME"},"output":"CIPHER_NATIVE_TOOL_UNKNOWN_OUTPUT","content":[{"type":"text","text":"safe result","private":"CIPHER_NATIVE_TOOL_UNKNOWN"},{"type":"text","text":"safe result","state":{"payload":"CIPHER_NATIVE_TOOL_STATE"}}]}}]}]}"#,
            r#"{"messages":[{"info":{"id":"u1","role":"user"},"parts":[{"id":"p1","type":"text","text":"safe prompt"}]},{"info":{"id":"a1","role":"assistant"},"parts":[{"id":"p2","type":"tool","tool":"read","callID":"c1","state":{"status":"completed","input":{},"output":"safe result","content":[{"state":{"redactedData":"CIPHER_CLASSIC_TOOL_CONTENT"}}]}}]}]}"#,
            r#"{"messages":[{"info":{"id":"u1","role":"user"},"parts":[{"id":"p1","type":"text","text":"safe prompt"}]},{"info":{"id":"a1","role":"assistant"},"parts":[{"id":"p2","type":"tool","tool":"read","callID":"c1","state":{"status":"completed","input":{},"output":{"signature":"CIPHER_CLASSIC_TOOL_OUTPUT","state":{"payload":"CIPHER_CLASSIC_TOOL_OUTPUT"}}}}]}]}"#,
            r#"{"messages":[{"id":"u1","type":"user","text":"safe prompt"},{"id":"a1","type":"assistant","content":[{"id":"t1","type":"tool","name":"write","state":{"status":"completed","input":{},"content":[{"type":"text","text":"safe result"}],"metadata":{"files":[{"file":"safe.rs","diff":"safe diff"}],"extra":{"payload":"CIPHER_TOOL_METADATA_SIBLING"}}}}]}]}"#,
            r#"{"messages":[{"info":{"id":"u1","role":"user"},"parts":[{"id":"p1","type":"text","text":"safe prompt"}]},{"info":{"id":"a1","role":"assistant"},"parts":[{"id":"p2","type":"tool","tool":"read","callID":"c1","state":{"status":"completed","input":{},"output":"safe result","title":{"providerMetadata":{"signature":"CIPHER_CLASSIC_TOOL_TITLE"}}}}]}]}"#,
            r#"{"messages":[{"info":{"id":"u1","role":"user"},"parts":[{"id":"p1","type":"text","text":"safe prompt"}]},{"info":{"id":"a1","role":"assistant"},"parts":[{"id":"p2","type":"tool","tool":"read","callID":"c1","state":{"status":"pending","input":{},"raw":{"redactedData":"CIPHER_CLASSIC_TOOL_RAW"}}}]}]}"#,
            r#"{"messages":[{"info":{"id":"u1","role":"user"},"parts":[{"id":"p1","type":"text","text":"safe prompt"}]},{"info":{"id":"a1","role":"assistant"},"parts":[{"id":"p2","type":"tool","tool":"read","callID":"c1","state":{"status":{"reasoningEncryptedContent":"CIPHER_CLASSIC_TOOL_STATUS"},"input":{},"output":"safe result"}}]}]}"#,
            r#"{"messages":[{"id":"u1","type":"user","text":"safe prompt"},{"id":"a1","type":"assistant","content":[{"id":"t1","type":"tool","name":"read","state":{"status":{"signature":"CIPHER_NATIVE_TOOL_STATUS"},"input":{},"content":[{"type":"text","text":"safe result"}]}}]}]}"#,
            r#"{"messages":[{"id":"u1","type":"user","text":"safe prompt"},{"id":"a1","type":"assistant","content":[{"id":"t1","type":"tool","name":"read","state":{"status":"running","input":{},"metadata":{},"content":[{"type":"text","text":"CIPHER_NATIVE_RUNNING_CONTENT"}]}}]}]}"#,
            r#"{"messages":[{"id":"u1","type":"user","text":"safe prompt"},{"id":"a1","type":"assistant","content":[{"id":"t1","type":"tool","name":"read","state":{"status":"streaming","input":"safe input","error":{"type":"provider","message":"CIPHER_NATIVE_STREAMING_ERROR","status":500}}}]}]}"#,
            r#"{"messages":[{"info":{"id":"u1","role":"user"},"parts":[{"id":"p1","type":"text","text":"safe prompt"}]},{"info":{"id":"a1","role":"assistant"},"parts":[{"id":"t1","type":"tool","tool":"read","callID":"c1","state":{"status":"pending","input":{},"raw":"safe command","output":"CIPHER_CLASSIC_PENDING_OUTPUT"}}]}]}"#,
            r#"{"messages":[{"info":{"id":"u1","role":"user"},"parts":[{"id":"p1","type":"text","text":"safe prompt"}]},{"info":{"id":"a1","role":"assistant"},"parts":[{"id":"t1","type":"tool","tool":"read","callID":"c1","state":{"status":"error","input":{},"error":"safe error","time":{"start":1,"end":2},"title":"CIPHER_CLASSIC_ERROR_TITLE"}}]}]}"#,
            r#"{"messages":[{"id":"u1","type":"user","text":"safe prompt"},{"id":"a1","type":"assistant","content":[{"id":"r1","type":"reasoning","text":"visible","time":{"signature":"CIPHER_REASONING_TIME"}}]}]}"#,
        ] {
            let (ordinary, state) = prepare(source).expect("metadata is a safe partial");
            assert!(matches!(state, ReasoningSourceState::Unproven));
            let ordinary = String::from_utf8(ordinary).expect("JSON is UTF-8");
            assert!(ordinary.contains("safe prompt"));
            for canary in [
                "CIPHER_ROOT",
                "CIPHER_EXTRA_ROOT",
                "CIPHER_INFO",
                "CIPHER_MESSAGE_INFO",
                "CIPHER_USER",
                "CIPHER_NESTED_USER_FILE",
                "CIPHER_NESTED_CLASSIC_MODEL",
                "CIPHER_NESTED_ASSISTANT_MODEL",
                "CIPHER_ASSISTANT_ERROR",
                "CIPHER_NATIVE_RESPONSE",
                "CIPHER_CLASSIC_RESPONSE",
                "CIPHER_NATIVE_TEXT_TIME",
                "CIPHER_NATIVE_TEXT_FLAG",
                "CIPHER_CLASSIC_TEXT_TIME",
                "CIPHER_CLASSIC_TEXT_ID",
                "CIPHER_CLASSIC_TEXT_FLAG",
                "CIPHER_USER_TEXT",
                "CIPHER_SYNTHETIC_TEXT",
                "CIPHER_SYSTEM_TEXT",
                "CIPHER_SKILL_TEXT",
                "CIPHER_NATIVE_TOOL_CONTENT",
                "CIPHER_NATIVE_TOOL_UNKNOWN",
                "CIPHER_NATIVE_TOOL_UNKNOWN_OUTPUT",
                "CIPHER_NATIVE_TOOL_RAW",
                "CIPHER_NATIVE_TOOL_TITLE",
                "CIPHER_NATIVE_TOOL_ATTACH",
                "CIPHER_NATIVE_TOOL_TIME",
                "CIPHER_NATIVE_TOOL_STATE",
                "CIPHER_CLASSIC_TOOL_CONTENT",
                "CIPHER_CLASSIC_TOOL_OUTPUT",
                "CIPHER_TOOL_METADATA_SIBLING",
                "CIPHER_CLASSIC_TOOL_TITLE",
                "CIPHER_CLASSIC_TOOL_RAW",
                "CIPHER_CLASSIC_TOOL_STATUS",
                "CIPHER_NATIVE_TOOL_STATUS",
                "CIPHER_NATIVE_RUNNING_CONTENT",
                "CIPHER_NATIVE_STREAMING_ERROR",
                "CIPHER_CLASSIC_PENDING_OUTPUT",
                "CIPHER_CLASSIC_ERROR_TITLE",
                "CIPHER_REASONING_TIME",
            ] {
                assert!(
                    !ordinary.contains(canary),
                    "ordinary copy retained {canary}"
                );
            }

            let bytes = source.as_bytes().to_vec();
            let auth = ExportAuthorized::issue("opencode", "ses_capture", &bytes);
            let snapshot = CaptureSnapshotService::capture_authorized_with_reasoning_context(
                TranscriptSource::Bytes { bytes, auth },
                agent_for(AgentKind::OpenCode),
                AuthorizedReasoningSourceContext::authorized_export(
                    "opencode",
                    "ses_capture",
                    "ses_og25",
                ),
                CaptureSnapshotPolicy::default(),
            );
            assert_eq!(
                snapshot.completeness(),
                CaptureSnapshotCompleteness::Partial
            );
            let visible = String::from_utf8_lossy(
                snapshot
                    .transcript()
                    .expect("safe partial transcript")
                    .bytes(),
            )
            .into_owned();
            let projection = serde_json::to_string(&snapshot.safe_projection())
                .expect("safe projection serializes");
            let debug = format!("{snapshot:?}");
            for canary in [
                "CIPHER_ROOT",
                "CIPHER_EXTRA_ROOT",
                "CIPHER_INFO",
                "CIPHER_MESSAGE_INFO",
                "CIPHER_USER",
                "CIPHER_NESTED_USER_FILE",
                "CIPHER_NESTED_CLASSIC_MODEL",
                "CIPHER_NESTED_ASSISTANT_MODEL",
                "CIPHER_ASSISTANT_ERROR",
                "CIPHER_NATIVE_RESPONSE",
                "CIPHER_CLASSIC_RESPONSE",
                "CIPHER_NATIVE_TEXT_TIME",
                "CIPHER_NATIVE_TEXT_FLAG",
                "CIPHER_CLASSIC_TEXT_TIME",
                "CIPHER_CLASSIC_TEXT_ID",
                "CIPHER_CLASSIC_TEXT_FLAG",
                "CIPHER_USER_TEXT",
                "CIPHER_SYNTHETIC_TEXT",
                "CIPHER_SYSTEM_TEXT",
                "CIPHER_SKILL_TEXT",
                "CIPHER_NATIVE_TOOL_CONTENT",
                "CIPHER_NATIVE_TOOL_UNKNOWN",
                "CIPHER_NATIVE_TOOL_UNKNOWN_OUTPUT",
                "CIPHER_NATIVE_TOOL_RAW",
                "CIPHER_NATIVE_TOOL_TITLE",
                "CIPHER_NATIVE_TOOL_ATTACH",
                "CIPHER_NATIVE_TOOL_TIME",
                "CIPHER_NATIVE_TOOL_STATE",
                "CIPHER_CLASSIC_TOOL_CONTENT",
                "CIPHER_CLASSIC_TOOL_OUTPUT",
                "CIPHER_TOOL_METADATA_SIBLING",
                "CIPHER_CLASSIC_TOOL_TITLE",
                "CIPHER_CLASSIC_TOOL_RAW",
                "CIPHER_CLASSIC_TOOL_STATUS",
                "CIPHER_NATIVE_TOOL_STATUS",
                "CIPHER_NATIVE_RUNNING_CONTENT",
                "CIPHER_NATIVE_STREAMING_ERROR",
                "CIPHER_CLASSIC_PENDING_OUTPUT",
                "CIPHER_CLASSIC_ERROR_TITLE",
                "CIPHER_REASONING_TIME",
            ] {
                assert!(!visible.contains(canary), "transcript retained {canary}");
                assert!(!projection.contains(canary), "projection retained {canary}");
                assert!(!debug.contains(canary), "debug retained {canary}");
            }
            assert!(visible.contains("safe prompt"));
            assert!(snapshot.try_into_ordinary().is_err());
        }

        let clean = r#"{"info":{"id":"ses_og25","metadata":{"note":"benign"}},"messages":[{"info":{"id":"u1","role":"user"},"parts":[{"id":"p1","type":"text","text":"safe prompt","metadata":{"note":"benign"}}]}]}"#;
        let (ordinary, state) = prepare(clean).expect("benign user metadata is stripped");
        assert!(matches!(state, ReasoningSourceState::Ordinary));
        let ordinary = String::from_utf8(ordinary).expect("JSON is UTF-8");
        assert!(ordinary.contains("safe prompt"));
        assert!(!ordinary.contains("benign"));

        // Enumerate the actual retained ordinary tree, not the raw fixture:
        // benign provider metadata and tool diff are already intentionally
        // omitted. Every retained non-free-content container gets an unknown
        // child, a wrong scalar, and an object/array shape swap. Every typed
        // leaf becomes an object and receives a wrong-scalar probe; integer
        // slots also receive a fractional-number probe. One canary per
        // location and probe kind makes missed branches diagnosable.
        let mut total_probes = 0;
        let mut scalar_probes = 0;
        let mut container_type_probes = 0;
        let mut integer_float_probes = 0;
        let mut failures = Vec::new();
        for (dialect, clean, expected_content_exceptions) in [
            ("native", OG25_SCHEMA_SHAPED_NATIVE, 1),
            ("classic", OG25_SCHEMA_SHAPED_CLASSIC, 1),
            ("native_file", OG25_NATIVE_FILE_CONTENT, 1),
            ("classic_pending", OG25_CLASSIC_PENDING_RAW, 1),
            ("classic_error", OG25_CLASSIC_ERROR_DATA, 0),
            ("native_more", OG25_NATIVE_NONASSISTANT_BRANCHES, 0),
            ("classic_more", OG25_CLASSIC_PART_BRANCHES, 0),
            ("native_optional", OG25_NATIVE_OPTIONAL_BRANCHES, 3),
            ("classic_states", OG25_CLASSIC_TOOL_STATES, 2),
            (
                "classic_finite_offsets",
                OG25_CLASSIC_FINITE_FILE_OFFSETS,
                0,
            ),
        ] {
            let (baseline, baseline_state) = prepare(clean).expect("clean export parses");
            assert!(
                matches!(baseline_state, ReasoningSourceState::Ordinary),
                "{dialect}: clean export must be Ordinary"
            );
            let baseline: serde_json::Value =
                serde_json::from_slice(&baseline).expect("ordinary copy is JSON");
            let required_retained: &[&str] = match dialect {
                "native" => &[
                    "/info/parentID",
                    "/info/agent",
                    "/info/outcome",
                    "/info/title",
                    "/info/subpath",
                    "/messages/1/previous/id",
                    "/messages/2/tokens/cache/read",
                    "/messages/3/content/0/state/metadata/files/0/file",
                ],
                "classic" => &[
                    "/info/parentID",
                    "/info/agent",
                    "/info/title",
                    "/info/cost",
                    "/info/tokens/cache/read",
                    "/messages/0/parts/1/source/text/value",
                    "/messages/1/info/error/data/message",
                    "/messages/1/parts/1/state/attachments/0/url",
                ],
                "native_file" => &["/messages/0/content/0/state/content/1/name"],
                "classic_pending" => &["/messages/0/parts/0/state/raw"],
                "classic_error" => &["/messages/1/info/error/data/statusCode"],
                "native_more" => &[
                    "/messages/0/agents/0/mention/text",
                    "/messages/2/previous/location/directory",
                    "/messages/6/output/truncated",
                ],
                "classic_more" => &[
                    "/messages/0/parts/1/source/range/start/line",
                    "/messages/0/parts/2/source/uri",
                    "/messages/1/parts/5/error/data/statusCode",
                ],
                "native_optional" => &[
                    "/messages/1/status",
                    "/messages/2/tokens/cache/read",
                    "/messages/2/error/type",
                    "/messages/3/snapshot/files/0",
                    "/messages/3/finish",
                    "/messages/3/rawFinish",
                    "/messages/3/cost",
                    "/messages/3/error/type",
                    "/messages/3/content/0/executed",
                    "/messages/3/content/0/state/status",
                    "/messages/3/content/1/state/status",
                    "/messages/3/content/2/state/error/type",
                    "/messages/3/content/2/state/content/0/text",
                ],
                "classic_states" => &[
                    "/messages/1/parts/0/state/status",
                    "/messages/1/parts/0/state/title",
                    "/messages/1/parts/1/state/status",
                    "/messages/1/parts/1/state/error",
                ],
                "classic_finite_offsets" => &[
                    "/messages/0/parts/0/source/text/start",
                    "/messages/0/parts/0/source/text/end",
                ],
                _ => unreachable!("fixed fixture table"),
            };
            for pointer in required_retained {
                assert!(
                    baseline.pointer(pointer).is_some(),
                    "{dialect}: required safe schema field {pointer} was not retained"
                );
            }
            if dialect == "classic_finite_offsets" {
                assert_eq!(
                    baseline
                        .pointer("/messages/0/parts/0/source/text/start")
                        .and_then(serde_json::Value::as_f64),
                    Some(0.5)
                );
                assert_eq!(
                    baseline
                        .pointer("/messages/0/parts/0/source/text/end")
                        .and_then(serde_json::Value::as_f64),
                    Some(4.5)
                );
            }
            let source_session = baseline
                .pointer("/info/id")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("ses_capture");
            if matches!(dialect, "native" | "classic") {
                let metadata_pointer = if dialect == "native" {
                    "/messages/3/content/0/state/metadata"
                } else {
                    "/messages/1/parts/1/state/metadata"
                };
                let truncated_pointer = format!("{metadata_pointer}/truncated");
                assert!(
                    baseline.pointer(&truncated_pointer).is_none(),
                    "{dialect}: recognized truncation flag entered ordinary copy"
                );
                let clean_raw: serde_json::Value =
                    serde_json::from_str(clean).expect("schema-shaped export parses");
                assert_eq!(
                    clean_raw
                        .pointer(&truncated_pointer)
                        .and_then(serde_json::Value::as_bool),
                    Some(false),
                    "{dialect}: fixture lost the routine Boolean truncation flag"
                );
                let mut only_flag = clean_raw.clone();
                only_flag
                    .pointer_mut(metadata_pointer)
                    .and_then(serde_json::Value::as_object_mut)
                    .expect("schema-shaped completed tool metadata")
                    .remove("files");
                let only_flag_bytes =
                    serde_json::to_vec(&only_flag).expect("Boolean-only fixture serializes");
                let (only_ordinary, only_state) =
                    prepare(std::str::from_utf8(&only_flag_bytes).expect("JSON is UTF-8"))
                        .expect("Boolean-only metadata is supported");
                assert!(
                    matches!(only_state, ReasoningSourceState::Ordinary),
                    "{dialect}: Boolean-only metadata made capture partial"
                );
                assert!(
                    serde_json::from_slice::<serde_json::Value>(&only_ordinary)
                        .expect("Boolean-only ordinary copy parses")
                        .pointer(metadata_pointer)
                        .is_none(),
                    "{dialect}: Boolean-only metadata entered ordinary copy"
                );
                let auth = ExportAuthorized::issue("opencode", "ses_capture", &only_flag_bytes);
                let only_snapshot =
                    CaptureSnapshotService::capture_authorized_with_reasoning_context(
                        TranscriptSource::Bytes {
                            bytes: only_flag_bytes,
                            auth,
                        },
                        agent_for(AgentKind::OpenCode),
                        AuthorizedReasoningSourceContext::authorized_export(
                            "opencode",
                            "ses_capture",
                            source_session,
                        ),
                        CaptureSnapshotPolicy::default(),
                    );
                assert_eq!(
                    only_snapshot.completeness(),
                    CaptureSnapshotCompleteness::Complete,
                    "{dialect}: Boolean-only metadata snapshot stayed partial"
                );
                assert!(only_snapshot.try_into_ordinary().is_ok());

                let mut true_flag = clean_raw.clone();
                *true_flag
                    .pointer_mut(&truncated_pointer)
                    .expect("schema-shaped truncation flag") = serde_json::json!(true);
                let true_flag_json =
                    serde_json::to_string(&true_flag).expect("Boolean fixture serializes");
                let (true_ordinary, true_state) =
                    prepare(&true_flag_json).expect("Boolean true is supported");
                assert!(
                    matches!(true_state, ReasoningSourceState::Ordinary),
                    "{dialect}: Boolean true made capture partial"
                );
                assert_eq!(
                    serde_json::from_slice::<serde_json::Value>(&true_ordinary)
                        .expect("Boolean ordinary copy parses"),
                    baseline,
                    "{dialect}: truncation flag changed ordinary projection"
                );

                for (shape, wrong) in [
                    (
                        "string",
                        serde_json::json!(format!("OG25_R20_{dialect}_TRUNCATED_STRING")),
                    ),
                    (
                        "object",
                        serde_json::json!({"payload": format!("OG25_R20_{dialect}_TRUNCATED_OBJECT")}),
                    ),
                    (
                        "array",
                        serde_json::json!([format!("OG25_R20_{dialect}_TRUNCATED_ARRAY")]),
                    ),
                    ("integer", serde_json::json!(17)),
                ] {
                    let mut injected = clean_raw.clone();
                    *injected
                        .pointer_mut(&truncated_pointer)
                        .expect("schema-shaped truncation flag") = wrong;
                    let canary = format!("OG25_R20_{dialect}_TRUNCATED_{}", shape.to_uppercase());
                    check_closed_projection_probe(
                        &injected,
                        &format!("{dialect}:truncated:{shape}"),
                        &canary,
                        source_session,
                        false,
                        &mut failures,
                    );
                }

                let output_path = format!("OG25_R20_{dialect}_OUTPUT_PATH");
                let unknown_sibling = format!("OG25_R20_{dialect}_UNKNOWN_SIBLING");
                let mut unknowns = clean_raw;
                let metadata = unknowns
                    .pointer_mut(metadata_pointer)
                    .and_then(serde_json::Value::as_object_mut)
                    .expect("schema-shaped completed tool metadata");
                metadata.insert("outputPath".to_owned(), serde_json::json!(output_path));
                metadata.insert(
                    "unknownSibling".to_owned(),
                    serde_json::json!({"payload": unknown_sibling}),
                );
                for canary in [&output_path, &unknown_sibling] {
                    check_closed_projection_probe(
                        &unknowns,
                        &format!("{dialect}:metadata:combined_unknowns:{canary}"),
                        canary,
                        source_session,
                        false,
                        &mut failures,
                    );
                }
            }
            let classic = dialect.starts_with("classic");
            let mut paths = Vec::new();
            collect_closed_projection_paths(&baseline, String::new(), &mut paths);
            let skipped = paths
                .iter()
                .filter(|(pointer, _)| is_closed_projection_content_exception(pointer))
                .count();
            assert_eq!(
                skipped, expected_content_exceptions,
                "{dialect}: ordinary-content allowlist changed"
            );
            assert!(
                paths.len() > skipped + 15,
                "{dialect}: fixture is too small"
            );

            for (index, (pointer, kind)) in paths.iter().enumerate() {
                if is_closed_projection_content_exception(pointer) {
                    continue;
                }
                // Structural root replacements may be malformed, but adding
                // one unknown root field or one bad message element must
                // still produce a safe Partial result.
                let allow_malformed_swap = pointer.is_empty() || pointer == "/messages";
                total_probes += 1;
                let canary = format!("OG25_R17_{dialect}_{index:04}_closed");
                let mut injected = baseline.clone();
                let target = injected
                    .pointer_mut(pointer)
                    .expect("enumerated JSON pointer");
                match kind {
                    ClosedProjectionProbe::Object => {
                        target.as_object_mut().expect("enumerated object").insert(
                            "__og25_unallowlisted".to_owned(),
                            serde_json::Value::String(canary.clone()),
                        );
                    }
                    ClosedProjectionProbe::Array => {
                        target
                            .as_array_mut()
                            .expect("enumerated array")
                            .push(serde_json::json!({"__og25_unallowlisted": canary.clone()}));
                    }
                    ClosedProjectionProbe::Leaf => {
                        *target = serde_json::json!({"__og25_unallowlisted": canary.clone()});
                    }
                }
                check_closed_projection_probe(
                    &injected,
                    &format!("{dialect}:{pointer}:{kind:?}:closed"),
                    &canary,
                    source_session,
                    false,
                    &mut failures,
                );
                if matches!(
                    kind,
                    ClosedProjectionProbe::Object | ClosedProjectionProbe::Array
                ) {
                    for swap in ["scalar", "cross_shape"] {
                        total_probes += 1;
                        container_type_probes += 1;
                        let wrong_type_canary =
                            if matches!(kind, ClosedProjectionProbe::Array) && swap == "scalar" {
                                // A string is a valid legacy native tool content
                                // shape, so a number is invalid for every array
                                // in this matrix.
                                format!("{}", 900_000 + index)
                            } else {
                                format!("OG25_R17_{dialect}_{index:04}_{swap}")
                            };
                        let mut wrong_type_injected = baseline.clone();
                        let target = wrong_type_injected
                            .pointer_mut(pointer)
                            .expect("enumerated container JSON pointer");
                        *target = match (kind, swap) {
                            (ClosedProjectionProbe::Array, "scalar") => {
                                serde_json::json!(900_000 + index)
                            }
                            (ClosedProjectionProbe::Object, "scalar") => {
                                serde_json::Value::String(wrong_type_canary.clone())
                            }
                            (ClosedProjectionProbe::Object, "cross_shape") => {
                                serde_json::json!([wrong_type_canary])
                            }
                            (ClosedProjectionProbe::Array, "cross_shape") => {
                                serde_json::json!({"__og25_wrong_array": wrong_type_canary})
                            }
                            _ => unreachable!("fixed container shape table"),
                        };
                        check_closed_projection_probe(
                            &wrong_type_injected,
                            &format!("{dialect}:{pointer}:{kind:?}:{swap}"),
                            &wrong_type_canary,
                            source_session,
                            allow_malformed_swap,
                            &mut failures,
                        );
                    }
                }
                if matches!(kind, ClosedProjectionProbe::Leaf) {
                    total_probes += 1;
                    scalar_probes += 1;
                    let wrong_scalar_canary = format!("OG25_R17_{dialect}_{index:04}_scalar");
                    let mut scalar_injected = baseline.clone();
                    let target = scalar_injected
                        .pointer_mut(pointer)
                        .expect("enumerated scalar JSON pointer");
                    *target = if target.is_string() {
                        // An integer is a wrong scalar for every retained
                        // schema string; the object probe above supplies its
                        // unique canary for zero-sink assertions.
                        serde_json::json!(index + 1000)
                    } else {
                        // A unique string canary tests number/bool slots as
                        // well as the existing wrong-object probe.
                        serde_json::Value::String(wrong_scalar_canary.clone())
                    };
                    check_closed_projection_probe(
                        &scalar_injected,
                        &format!("{dialect}:{pointer}:{kind:?}:wrong_scalar"),
                        &wrong_scalar_canary,
                        source_session,
                        false,
                        &mut failures,
                    );
                    if is_closed_projection_integer_slot(pointer, classic) {
                        total_probes += 1;
                        integer_float_probes += 1;
                        let fraction = (index + 1000) as f64 + 0.5;
                        let fraction_canary = format!("{}.5", index + 1000);
                        let mut fraction_injected = baseline.clone();
                        *fraction_injected
                            .pointer_mut(pointer)
                            .expect("enumerated integer JSON pointer") =
                            serde_json::json!(fraction);
                        check_closed_projection_probe(
                            &fraction_injected,
                            &format!("{dialect}:{pointer}:{kind:?}:fraction"),
                            &fraction_canary,
                            source_session,
                            false,
                            &mut failures,
                        );
                    }
                }
            }
        }
        assert!(
            total_probes > 100,
            "closed-projection fixture traversal shrank"
        );
        assert!(scalar_probes > 100, "wrong-scalar fixture traversal shrank");
        assert!(
            container_type_probes > 100,
            "container type traversal shrank"
        );
        assert!(integer_float_probes >= 20, "integer float traversal shrank");
        assert!(
            failures.is_empty(),
            "{} closed-projection leaks:\n{}",
            failures.len(),
            failures.join("\n")
        );
    }

    #[test]
    fn og25_reused_native_tool_call_id_is_not_part_identity() {
        let input = r#"{"messages":[{"id":"msg_a1","type":"assistant","content":[{"id":"call_0","type":"tool","name":"read","state":{"status":"completed","input":{},"content":[{"type":"text","text":"one"}]}}]},{"id":"msg_a2","type":"assistant","content":[{"id":"call_0","type":"tool","name":"read","state":{"status":"completed","input":{},"content":[{"type":"text","text":"two"}]}}]}]}"#;
        let (ordinary, state) = prepare(input).expect("reused tool call id is allowed");
        assert!(matches!(state, ReasoningSourceState::Ordinary));
        assert_eq!(ordinary, input.as_bytes());
    }

    #[test]
    fn og25_clean_native_tool_file_content_remains_ordinary() {
        let input = OG25_NATIVE_FILE_CONTENT;
        let (ordinary, state) = prepare(input).expect("schema-shaped tool content is valid");
        assert!(matches!(state, ReasoningSourceState::Ordinary));
        assert_eq!(ordinary, input.as_bytes());
    }

    #[test]
    fn og25_clean_classic_pending_tool_raw_remains_ordinary() {
        let input = OG25_CLASSIC_PENDING_RAW;
        let (ordinary, state) = prepare(input).expect("schema-shaped pending tool is valid");
        assert!(matches!(state, ReasoningSourceState::Ordinary));
        assert_eq!(ordinary, input.as_bytes());
    }

    #[test]
    fn og25_tool_and_envelope_metadata_are_scrubbed_before_redaction() {
        let input = r#"{"messages":[{"id":"msg_a","type":"assistant","metadata":{"provider":{"signature":"CIPHER_MESSAGE"}},"content":[{"id":"part_tool","type":"tool","name":"read","state":{"status":"completed","input":{"signature":"USER_TOOL_INPUT"},"metadata":{"files":[{"file":"src/safe.rs","diff":{"signature":"CIPHER_TOOL_DIFF"}}],"anthropic":{"signature":"CIPHER_TOOL"}}},"providerResultState":{"reasoningEncryptedContent":"CIPHER_RESULT"}},{"id":"part_text","type":"text","text":"visible","metadata":{"anthropic":{"redactedData":"CIPHER_TEXT"}}}]}]}"#;
        let (ordinary, state) = prepare(input).expect("provider metadata is discarded");
        assert!(matches!(state, ReasoningSourceState::Unproven));
        let ordinary = String::from_utf8(ordinary).expect("JSON is UTF-8");
        for canary in [
            "CIPHER_MESSAGE",
            "CIPHER_TOOL",
            "CIPHER_TOOL_DIFF",
            "CIPHER_RESULT",
            "CIPHER_TEXT",
        ] {
            assert!(!ordinary.contains(canary), "normal copy retained {canary}");
        }
        assert!(ordinary.contains("USER_TOOL_INPUT"));
        assert!(ordinary.contains("visible"));
        assert!(ordinary.contains(r#""file":"src/safe.rs""#));
    }

    #[test]
    fn og25_classifier_output_preserves_coverage_and_extraction_on_clean_exports() {
        let existing_native = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/agent_transcripts/opencode.json"
        ));
        let metadata_only_native = r#"{"info":{"id":"ses_native","location":{"directory":"/project"}},"messages":[{"id":"u1","type":"user","text":"go"},{"id":"a1","type":"assistant","model":{"id":"model","providerID":"provider"},"tokens":{"input":1,"output":2,"reasoning":0,"cache":{"read":0,"write":0}},"content":[{"id":"t1","type":"tool","name":"write","state":{"status":"completed","input":{},"content":"ok","metadata":{"files":[{"file":"src/native.rs","diff":"ordinary tool output"}]}}}]}]}"#;
        let metadata_only_classic = r#"{"info":{"id":"ses_classic","directory":"/project"},"messages":[{"info":{"id":"u1","role":"user"},"parts":[{"id":"p1","type":"text","text":"go"}]},{"info":{"id":"a1","role":"assistant","modelID":"model","tokens":{"input":1,"output":2,"reasoning":0,"cache":{"read":0,"write":0}}},"parts":[{"id":"p2","type":"tool","tool":"write","callID":"c1","state":{"status":"completed","input":{},"output":"ok","metadata":{"files":[{"file":"src/classic.rs","diff":"ordinary tool output"}]}}}]}]}"#;
        let metadata_only_classic_model_fallback = metadata_only_classic.replace(
            r#""modelID":"model""#,
            r#""model":{"modelID":"model","providerID":"provider"}"#,
        );
        // Schema-shaped exports pin common fields that the small compatibility
        // fixtures omit. They are deterministic fixtures, not live model proof.
        let schema_shaped_native = OG25_SCHEMA_SHAPED_NATIVE;
        let schema_shaped_classic = OG25_SCHEMA_SHAPED_CLASSIC;
        for (name, source, expected_file) in [
            ("existing_native", existing_native, "src/lib.rs"),
            (
                "metadata_only_native",
                metadata_only_native,
                "src/native.rs",
            ),
            (
                "metadata_only_classic",
                metadata_only_classic,
                "src/classic.rs",
            ),
            (
                "metadata_only_classic_model_fallback",
                metadata_only_classic_model_fallback.as_str(),
                "src/classic.rs",
            ),
            (
                "schema_shaped_native",
                schema_shaped_native,
                "src/native-real.rs",
            ),
            (
                "schema_shaped_classic",
                schema_shaped_classic,
                "src/classic-real.rs",
            ),
        ] {
            let before_turns = normalize_opencode_export(source.as_bytes());
            let before_extract = extract_opencode(source.as_bytes());
            assert!(!before_turns.is_empty(), "{name}: no baseline turns");
            assert!(
                before_extract
                    .modified_files
                    .iter()
                    .any(|file| file == expected_file),
                "{name}: baseline lacks metadata file"
            );

            let (ordinary, state) = prepare(source).expect("clean export is supported");
            assert!(matches!(state, ReasoningSourceState::Ordinary), "{name}");
            let after_turns = normalize_opencode_export(&ordinary);
            assert_eq!(
                after_turns, before_turns,
                "{name}: coverage or digest changed"
            );
            let after_extract = extract_opencode(&ordinary);
            assert_eq!(
                after_extract.partial, before_extract.partial,
                "{name}: partial"
            );
            assert_eq!(
                after_extract.prompts, before_extract.prompts,
                "{name}: prompts"
            );
            assert_eq!(after_extract.model, before_extract.model, "{name}: model");
            assert_eq!(after_extract.usage, before_extract.usage, "{name}: usage");
            assert_eq!(
                after_extract.api_call_count, before_extract.api_call_count,
                "{name}: api calls"
            );
            assert_eq!(
                after_extract.modified_files, before_extract.modified_files,
                "{name}: modified files"
            );
            if name != "existing_native" {
                assert!(
                    !String::from_utf8_lossy(&ordinary).contains("ordinary tool output"),
                    "{name}: non-projected tool metadata retained"
                );
            }
        }

        let classic_timed_text = r#"{"messages":[{"info":{"id":"u1","role":"user"},"parts":[{"id":"p1","sessionID":"ses_1","messageID":"u1","type":"text","text":"go","synthetic":false,"ignored":false,"time":{"start":1,"end":2}}]}]}"#;
        let (ordinary, state) = prepare(classic_timed_text).expect("valid timed text part");
        assert!(matches!(state, ReasoningSourceState::Ordinary));
        let ordinary = String::from_utf8(ordinary).expect("ordinary JSON is UTF-8");
        assert!(ordinary.contains(r#""sessionID":"ses_1""#));
        let projected: serde_json::Value =
            serde_json::from_str(&ordinary).expect("ordinary JSON parses");
        assert_eq!(
            projected
                .pointer("/messages/0/parts/0/time/start")
                .and_then(serde_json::Value::as_i64),
            Some(1)
        );
        assert_eq!(
            projected
                .pointer("/messages/0/parts/0/time/end")
                .and_then(serde_json::Value::as_i64),
            Some(2)
        );
    }

    #[test]
    fn og25_unknown_native_message_type_discards_opaque_envelope() {
        let input = r#"{"messages":[{"id":"msg_unknown","type":"Assistant","metadata":{"signature":"CIPHER_UNKNOWN"},"text":"CIPHER_TEXT"},{"id":"msg_u","type":"user","text":"prompt"}]}"#;
        let (ordinary, state) = prepare(input).expect("unknown message is safe partial");
        assert!(matches!(state, ReasoningSourceState::Unproven));
        let ordinary = String::from_utf8(ordinary).expect("JSON is UTF-8");
        assert!(!ordinary.contains("CIPHER_UNKNOWN"));
        assert!(!ordinary.contains("CIPHER_TEXT"));
        assert!(ordinary.contains("prompt"));
    }

    #[test]
    fn og25_classic_reasoning_and_malformed_parts_are_scrubbed() {
        let classic = r#"{"info":{"id":"ses_og25"},"messages":[{"info":{"id":"u1","role":"user"},"parts":[{"type":"text","text":"prompt"}]},{"info":{"id":"a1","role":"assistant"},"parts":[{"id":"r1","type":"reasoning","text":"visible","state":{"signature":"CLASSIC_CIPHER"}}]}]}"#;
        let (ordinary, state) = prepare(classic).expect("classic export remains safe partial");
        assert!(matches!(state, ReasoningSourceState::Unproven));
        let ordinary = String::from_utf8(ordinary).expect("JSON is UTF-8");
        assert!(!ordinary.contains("visible"));
        assert!(!ordinary.contains("CLASSIC_CIPHER"));

        for (part, canary) in [
            (r#""CIPHER_PART_STRING""#, "CIPHER_PART_STRING"),
            (
                r#"{"id":"r1","text":"CIPHER_UNTYPED","state":{"signature":"CIPHER_UNTYPED"}}"#,
                "CIPHER_UNTYPED",
            ),
            (
                r#"{"id":"r1","type":42,"text":"CIPHER_BAD_TYPE"}"#,
                "CIPHER_BAD_TYPE",
            ),
        ] {
            let input = export(part);
            let (ordinary, state) = prepare(&input).expect("malformed part is discarded");
            assert!(matches!(state, ReasoningSourceState::Unproven));
            assert!(!String::from_utf8_lossy(&ordinary).contains(canary));
        }
    }

    #[test]
    fn og25_malformed_envelopes_preserve_safe_partial_transcript() {
        for (input, canary) in [
            (
                r#"{"messages":[null,{"id":"u1","type":"user","text":"prompt"}]}"#,
                None,
            ),
            (
                r#"{"messages":[{"id":"a1","type":{"signature":"CIPHER_TYPE"},"content":[{"state":{"signature":"CIPHER_BODY"}}]}]}"#,
                Some("CIPHER_TYPE"),
            ),
            (
                r#"{"messages":[{"id":"a1","type":5,"metadata":{"anthropic":{"signature":"CIPHER_BAD_TYPE"}}},{"id":"u1","type":"user","text":"prompt"}]}"#,
                Some("CIPHER_BAD_TYPE"),
            ),
            (
                r#"{"messages":[{"info":{"id":"a1","metadata":{"anthropic":{"redactedData":"CIPHER_MISSING_ROLE"}}},"parts":[{"type":"text","text":"unsafe"}]},{"id":"u1","type":"user","text":"prompt"}]}"#,
                Some("CIPHER_MISSING_ROLE"),
            ),
            (
                r#"{"messages":[{"info":{"id":"a1","role":"Assistant"},"parts":[{"type":"text","text":"unsafe","metadata":{"anthropic":{"signature":"CIPHER_ROLE"}}}]},{"id":"u1","type":"user","text":"prompt"}]}"#,
                Some("CIPHER_ROLE"),
            ),
            (
                r#"{"messages":[{"id":"a1","type":"assistant","content":{"signature":"CIPHER_CONTENT"}}]}"#,
                Some("CIPHER_CONTENT"),
            ),
            (
                r#"{"messages":[{"id":"a1","type":"assistant","content":[{"id":{"signature":"CIPHER_PART_ID"},"type":"reasoning","text":"visible"}]}]}"#,
                Some("CIPHER_PART_ID"),
            ),
            (
                r#"{"messages":[{"id":"a1","type":"assistant","content":[{"id":"r1","type":"reasoning","text":"visible"}],"providerContext":"CIPHER_PROVIDER"}]}"#,
                Some("CIPHER_PROVIDER"),
            ),
            (
                r#"{"debug":"CIPHER_UNKNOWN_ROOT","messages":[{"id":"u1","type":"user","text":"prompt"}]}"#,
                Some("CIPHER_UNKNOWN_ROOT"),
            ),
            (
                r#"{"messages":[{"id":"u1","type":"user","text":"prompt","futureField":"CIPHER_UNKNOWN_MESSAGE"}]}"#,
                Some("CIPHER_UNKNOWN_MESSAGE"),
            ),
            (
                r#"{"messages":[{"info":{"id":"u1","role":"user","futureField":"CIPHER_UNKNOWN_INFO"},"parts":[{"id":"p1","type":"text","text":"prompt"}]}]}"#,
                Some("CIPHER_UNKNOWN_INFO"),
            ),
        ] {
            let (ordinary, state) = prepare(input).expect("safe partial transcript");
            assert!(matches!(state, ReasoningSourceState::Unproven));
            if let Some(canary) = canary {
                assert!(!String::from_utf8_lossy(&ordinary).contains(canary));
            }
            if input.contains("prompt") {
                assert!(String::from_utf8_lossy(&ordinary).contains("prompt"));
            }
            if input.contains("unsafe") {
                assert!(!String::from_utf8_lossy(&ordinary).contains("unsafe"));
            }
        }
        let missing = r#"{"messages":[{"id":"a1","type":"assistant"}]}"#;
        let (ordinary, state) = prepare(missing).expect("missing content is safe partial");
        assert!(matches!(state, ReasoningSourceState::Unproven));
        assert_eq!(ordinary, missing.as_bytes()); // The unproven/no-scrub branch has no candidate.

        let null_context = r#"{"messages":[{"id":"a1","type":"assistant","providerContext":null,"content":[{"id":"r1","type":"reasoning","text":"visible"}]}]}"#;
        let (ordinary, state) = prepare(null_context).expect("null providerContext is absent");
        assert!(matches!(state, ReasoningSourceState::Ordinary));
        assert!(!String::from_utf8_lossy(&ordinary).contains("providerContext"));
        assert!(String::from_utf8_lossy(&ordinary).contains("visible"));
    }

    #[test]
    fn og25_ambiguous_ids_keep_safe_partial_and_obey_cap() {
        for input in [
            r#"{"messages":[{"id":"same","type":"user","text":"prompt"},{"id":"same","type":"assistant","content":[]}]}"#,
            r#"{"messages":[{"id":"a1","type":"assistant","content":[{"id":"same","type":"reasoning","text":"one"},{"id":"same","type":"reasoning","text":"two"}]}]}"#,
        ] {
            let (_, state) = prepare(input).expect("duplicate ID remains a partial transcript");
            assert!(matches!(state, ReasoningSourceState::Unproven));
        }
        let mut seen = (0..MAX_OBSERVED_IDS)
            .map(|index| format!("id_{index}"))
            .collect();
        let mut fields =
            BTreeMap::from([("id".to_owned(), CanonValue::Str("overflow".to_owned()))]);
        let (mut removed, mut unproven) = (false, false);
        inspect_id_field(&mut fields, &mut seen, &mut removed, &mut unproven);
        assert!(unproven);
        assert!(!removed);
        assert_eq!(seen.len(), MAX_OBSERVED_IDS);
    }

    #[test]
    fn og25_candidate_has_zero_normal_sink_hits() {
        use crate::internal::ai::{
            capture::snapshot::{
                AuthorizedReasoningSourceContext, CaptureSnapshotCompleteness,
                CaptureSnapshotPolicy, CaptureSnapshotService,
            },
            observed_agents::{
                AgentKind, agent_for,
                transcript_source::{ExportAuthorized, TranscriptSource},
            },
        };

        let input = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/agent_transcripts/reasoning/opencode_og25.json"
        ));
        let bytes = input.as_bytes().to_vec();
        let auth = ExportAuthorized::issue("opencode", "ses_capture", &bytes);
        let source = TranscriptSource::Bytes { bytes, auth };
        let snapshot = CaptureSnapshotService::capture_authorized_with_reasoning_context(
            source,
            agent_for(AgentKind::OpenCode),
            AuthorizedReasoningSourceContext::authorized_export(
                "opencode",
                "ses_capture",
                "ses_og25_synthetic",
            ),
            CaptureSnapshotPolicy::default(),
        );
        assert_eq!(
            snapshot.completeness(),
            CaptureSnapshotCompleteness::Partial
        );
        let ordinary = snapshot.transcript().expect("safe partial transcript");
        let visible = String::from_utf8_lossy(ordinary.bytes());
        let projection =
            serde_json::to_string(&snapshot.safe_projection()).expect("safe projection serializes");
        let debug = format!("{snapshot:?}");
        for canary in [
            "CIPHER_WINDOW",
            "CIPHER_SIG",
            "CIPHER_REDACTED",
            "CIPHER_ENCRYPTED",
            "CIPHER_PROOF",
        ] {
            assert!(!visible.contains(canary), "transcript retained {canary}");
            assert!(!projection.contains(canary), "metadata retained {canary}");
            assert!(!debug.contains(canary), "debug retained {canary}");
        }
        assert!(snapshot.try_into_ordinary().is_err());
        for kind in [
            AgentKind::ClaudeCode,
            AgentKind::Codex,
            AgentKind::Cursor,
            AgentKind::FactoryAi,
        ] {
            assert!(agent_for(kind).as_reasoning_source_preprocessor().is_none());
        }
        assert_non_export_capture_is_content_free_partial();
    }

    #[test]
    fn og25_identical_candidate_text_has_zero_snapshot_sink_hits() {
        use crate::internal::ai::{
            capture::snapshot::{
                AuthorizedReasoningSourceContext, CaptureSnapshotCompleteness,
                CaptureSnapshotPolicy, CaptureSnapshotService,
            },
            observed_agents::{
                AgentKind, agent_for,
                transcript_source::{ExportAuthorized, TranscriptSource},
            },
        };

        for part in [
            r#"{"id":"part_1","type":"reasoning","text":"CIPHER_IDENTICAL","state":{"signature":"CIPHER_IDENTICAL"}}"#,
            r#"{"id":"part_1","type":"reasoning","text":"visible CIPHER_FRAGMENT tail","state":{"signature":"CIPHER_FRAGMENT"}}"#,
            r#"{"id":"part_1","type":"reasoning","text":"CIPHER_UNKNOWN","state":{"future":"CIPHER_UNKNOWN"}}"#,
        ] {
            let bytes = export(part).into_bytes();
            let auth = ExportAuthorized::issue("opencode", "ses_capture", &bytes);
            let snapshot = CaptureSnapshotService::capture_authorized_with_reasoning_context(
                TranscriptSource::Bytes { bytes, auth },
                agent_for(AgentKind::OpenCode),
                AuthorizedReasoningSourceContext::authorized_export(
                    "opencode",
                    "ses_capture",
                    "ses_og25",
                ),
                CaptureSnapshotPolicy::default(),
            );
            assert_eq!(
                snapshot.completeness(),
                CaptureSnapshotCompleteness::Partial
            );
            let visible = String::from_utf8_lossy(
                snapshot
                    .transcript()
                    .expect("safe partial transcript")
                    .bytes(),
            )
            .into_owned();
            let projection = serde_json::to_string(&snapshot.safe_projection())
                .expect("safe projection serializes");
            let debug = format!("{snapshot:?}");
            for canary in ["CIPHER_IDENTICAL", "CIPHER_FRAGMENT", "CIPHER_UNKNOWN"] {
                assert!(!visible.contains(canary), "transcript retained {canary}");
                assert!(!projection.contains(canary), "projection retained {canary}");
                assert!(!debug.contains(canary), "debug retained {canary}");
            }
            assert!(snapshot.try_into_ordinary().is_err());
        }
    }

    #[test]
    fn og25_visible_reasoning_is_redacted_in_ordinary_snapshot() {
        use crate::internal::ai::{
            capture::snapshot::{
                AuthorizedReasoningSourceContext, CaptureSnapshotCompleteness,
                CaptureSnapshotPolicy, CaptureSnapshotService,
            },
            observed_agents::{
                AgentKind, agent_for,
                transcript_source::{ExportAuthorized, TranscriptSource},
            },
        };

        let bytes = br#"{"info":{"id":"ses_og25"},"messages":[{"id":"u1","type":"user","text":"prompt"},{"id":"a1","type":"assistant","content":[{"id":"r1","type":"reasoning","text":"thought sk-ant-EXAMPLE0000000000000000"}]}]}"#.to_vec();
        let auth = ExportAuthorized::issue("opencode", "ses_capture", &bytes);
        let snapshot = CaptureSnapshotService::capture_authorized_with_reasoning_context(
            TranscriptSource::Bytes { bytes, auth },
            agent_for(AgentKind::OpenCode),
            AuthorizedReasoningSourceContext::authorized_export(
                "opencode",
                "ses_capture",
                "ses_og25",
            ),
            CaptureSnapshotPolicy::default(),
        );
        assert_eq!(
            snapshot.completeness(),
            CaptureSnapshotCompleteness::Complete
        );
        let ordinary = snapshot.transcript().expect("redacted ordinary transcript");
        let visible = String::from_utf8_lossy(ordinary.bytes());
        assert!(visible.contains("reasoning"));
        assert!(!visible.contains("sk-ant-EXAMPLE0000000000000000"));
        assert!(snapshot.redaction_report().bytes_redacted > 0);
    }

    fn assert_non_export_capture_is_content_free_partial() {
        use crate::internal::ai::{
            capture::snapshot::{
                AuthorizedReasoningSourceContext, CaptureSnapshotCompleteness,
                CaptureSnapshotPartialReason, CaptureSnapshotPolicy, CaptureSnapshotService,
            },
            observed_agents::{
                AgentKind, agent_for,
                transcript_source::{ExportAuthorized, TranscriptSource},
            },
        };

        for (source, canary) in [
            (
                r#"{"messages":[{"type":"assistant","content":[{"state":{"signature":"CIPHER_MALFORMED"}}"#,
                "CIPHER_MALFORMED",
            ),
            (r#"{"role":"user","content":"CIPHER_FLAT"}"#, "CIPHER_FLAT"),
            (
                "{\"role\":\"user\",\"content\":\"CIPHER_JSONL\"}\n{\"role\":\"assistant\",\"content\":\"reply\"}",
                "CIPHER_JSONL",
            ),
            (
                r#"{"info":{"id":"ses_og25"},"parts":["CIPHER_PARTS"]}"#,
                "CIPHER_PARTS",
            ),
            (
                r#"{"messages":{"signature":"CIPHER_WRONG_MESSAGES"}}"#,
                "CIPHER_WRONG_MESSAGES",
            ),
            (
                r#"{"messages":[],"messages":[{"id":"u1","type":"user","text":"CIPHER_DUPLICATE"}]}"#,
                "CIPHER_DUPLICATE",
            ),
        ] {
            let bytes = source.as_bytes().to_vec();
            let auth = ExportAuthorized::issue("opencode", "ses_capture", &bytes);
            let snapshot = CaptureSnapshotService::capture_authorized_with_reasoning_context(
                TranscriptSource::Bytes { bytes, auth },
                agent_for(AgentKind::OpenCode),
                AuthorizedReasoningSourceContext::authorized_export(
                    "opencode",
                    "ses_capture",
                    "ses_og25",
                ),
                CaptureSnapshotPolicy::default(),
            );
            assert_eq!(
                snapshot.completeness(),
                CaptureSnapshotCompleteness::Partial
            );
            assert_eq!(
                snapshot.partial_reason(),
                Some(CaptureSnapshotPartialReason::SourceAuthorizationMismatch)
            );
            assert!(snapshot.transcript().is_none());
            assert!(!format!("{snapshot:?}").contains(canary));
            assert!(
                !serde_json::to_string(&snapshot.safe_projection())
                    .expect("safe projection serializes")
                    .contains(canary)
            );
            assert!(snapshot.try_into_ordinary().is_err());
        }
    }

    #[test]
    fn og25_malformed_export_is_content_free_partial_at_snapshot_seam() {
        assert_non_export_capture_is_content_free_partial();
    }

    #[test]
    fn og25_unverified_contexts_cannot_promote_candidates() {
        use crate::internal::ai::{
            capture::snapshot::{
                AuthorizedReasoningSourceContext, CaptureSnapshotPolicy, CaptureSnapshotService,
            },
            observed_agents::{
                AgentKind, agent_for,
                transcript_source::{ExportAuthorized, TranscriptSource},
            },
        };

        let bytes = export(
            r#"{"id":"r1","type":"reasoning","text":"visible","state":{"signature":"CIPHER_SIG"}}"#,
        )
        .into_bytes();
        let capture = |context: Option<AuthorizedReasoningSourceContext>| {
            let auth = ExportAuthorized::issue("opencode", "ses_capture", &bytes);
            let source = TranscriptSource::Bytes {
                bytes: bytes.clone(),
                auth,
            };
            match context {
                Some(context) => CaptureSnapshotService::capture_authorized_with_reasoning_context(
                    source,
                    agent_for(AgentKind::OpenCode),
                    context,
                    CaptureSnapshotPolicy::default(),
                ),
                None => CaptureSnapshotService::capture_authorized(
                    source,
                    "opencode",
                    "ses_capture",
                    CaptureSnapshotPolicy::default(),
                ),
            }
        };
        let legacy = capture(None);
        assert!(legacy.rejects_capture_source_consumer());
        assert!(matches!(
            legacy.into_checkpoint_parts().2.state(),
            ReasoningSourceState::Unmapped
        ));
        let parent = capture(Some(AuthorizedReasoningSourceContext::parent_only_child(
            "opencode",
            "ses_capture",
        )));
        assert!(parent.rejects_capture_source_consumer());
        assert!(matches!(
            parent.into_checkpoint_parts().2.state(),
            ReasoningSourceState::Unmapped
        ));
        let imported = capture(Some(AuthorizedReasoningSourceContext::verified_import(
            "opencode",
            "ses_capture",
            "ses_capture",
            "ses_og25",
        )));
        assert!(imported.rejects_capture_source_consumer());
        assert!(matches!(
            imported.into_checkpoint_parts().2.state(),
            ReasoningSourceState::Unproven
        ));
    }

    #[test]
    fn og25_only_opencode_adapter_exposes_classifier() {
        use crate::internal::ai::observed_agents::{AgentKind, agent_for};
        assert!(
            agent_for(AgentKind::OpenCode)
                .as_reasoning_source_preprocessor()
                .is_some()
        );
        for kind in [
            AgentKind::ClaudeCode,
            AgentKind::Codex,
            AgentKind::Cursor,
            AgentKind::FactoryAi,
        ] {
            assert!(agent_for(kind).as_reasoning_source_preprocessor().is_none());
        }
    }
    fn opencode29_idle_fixture(error: Option<serde_json::Value>) -> String {
        let mut idle = serde_json::json!({
            "id": "msg_idle29", "type": "idle", "outcome": "failed",
            "time": { "created": 2 }
        });
        if let Some(error) = error {
            idle["error"] = error;
        }
        serde_json::json!({
            "info": { "id": "ses_ordinary29" },
            "messages": [
                { "id": "msg_user29", "type": "user", "text": "safe prompt", "time": { "created": 1 } },
                idle
            ]
        }).to_string()
    }

    #[test]
    fn opencode29_idle_error_accepts_exact_typed_header_and_absence() {
        for error in [
            None,
            Some(serde_json::json!({ "type": "output-limit", "message": "limit reached" })),
            Some(
                serde_json::json!({ "type": "tool.input-incomplete", "message": "incomplete input", "status": 100 }),
            ),
            Some(serde_json::json!({ "type": "", "message": "", "status": 599 })),
        ] {
            let input = opencode29_idle_fixture(error.clone());
            let (ordinary, state) = prepare(&input).expect("synthetic official29 idle parses");
            assert!(matches!(state, ReasoningSourceState::Ordinary));
            let value: serde_json::Value = serde_json::from_slice(&ordinary).expect("safe JSON");
            assert_eq!(value["messages"][1].get("error"), error.as_ref());
        }
    }

    #[test]
    fn opencode29_idle_error_unknown_response_or_opaque_stays_partial() {
        for extra in [
            serde_json::json!({ "unknown": "ERROR29_CANARY" }),
            serde_json::json!({ "response": { "body": "ERROR29_CANARY" } }),
            serde_json::json!({ "signature": "ERROR29_CANARY" }),
            serde_json::json!({ "redactedData": "ERROR29_CANARY" }),
            serde_json::json!({ "reasoningEncryptedContent": "ERROR29_CANARY" }),
        ] {
            let mut error = serde_json::json!({ "type": "output-limit", "message": "safe header" });
            error
                .as_object_mut()
                .expect("object")
                .extend(extra.as_object().expect("object").clone());
            let (ordinary, state) =
                prepare(&opencode29_idle_fixture(Some(error))).expect("synthetic29 error parses");
            assert!(matches!(state, ReasoningSourceState::Unproven));
            assert!(!String::from_utf8_lossy(&ordinary).contains("ERROR29_CANARY"));
            let safe: serde_json::Value = serde_json::from_slice(&ordinary).expect("safe JSON");
            if ["signature", "redactedData", "reasoningEncryptedContent"]
                .iter()
                .any(|key| extra.get(key).is_some())
            {
                assert!(safe["messages"][1].get("error").is_none());
            } else {
                assert_eq!(
                    safe["messages"][1]["error"],
                    serde_json::json!({
                        "type": "output-limit", "message": "safe header"
                    })
                );
            }
        }
    }

    #[test]
    fn opencode29_idle_error_malformed_required_or_status_stays_partial() {
        for error in [
            serde_json::json!(null),
            serde_json::json!("invalid"),
            serde_json::json!({}),
            serde_json::json!({ "type": "missing-message" }),
            serde_json::json!({ "message": "missing-type" }),
            serde_json::json!({ "type": 1, "message": "safe" }),
            serde_json::json!({ "type": "safe", "message": 1 }),
            serde_json::json!({ "type": "safe", "message": "safe", "status": 99 }),
            serde_json::json!({ "type": "safe", "message": "safe", "status": 600 }),
            serde_json::json!({ "type": "safe", "message": "safe", "status": 400.5 }),
            serde_json::json!({ "type": "safe", "message": "safe", "status": "400" }),
            serde_json::json!({ "type": "safe", "message": "safe", "status": null }),
            serde_json::json!({ "type": "safe", "message": "safe", "status": false }),
            serde_json::json!([]),
            serde_json::json!(1),
        ] {
            let (ordinary, state) =
                prepare(&opencode29_idle_fixture(Some(error))).expect("synthetic29 error parses");
            assert!(matches!(state, ReasoningSourceState::Unproven));
            let value: serde_json::Value = serde_json::from_slice(&ordinary).expect("safe JSON");
            assert!(value["messages"][1].get("error").is_none());
        }
    }
    #[test]
    fn verified_partial_single_json_permit_is_closed() {
        use crate::internal::ai::observed_agents::reasoning::ReasoningSourceDisposition;
        fn source(state: serde_json::Value) -> serde_json::Value {
            serde_json::json!({"info":{"id":"ses_verified_partial","location":{"directory":"/fixture"}},
                "messages":[{"id":"msg_user","type":"user","text":"safe request"},
                {"id":"msg_assistant","type":"assistant","content":[
                    {"type":"reasoning","text":"PRIVATE_REASONING_CANARY","time":{"start":1},"state":state},
                    {"type":"text","text":"safe reply"}]}]})
        }
        fn has_permit(value: serde_json::Value) -> bool {
            let bytes = serde_json::to_vec(&value).expect("source fixture");
            let prepared = prepare_authorized_source(bytes).expect("strict valid fixture");
            let (ordinary, disposition, permit) =
                ReasoningSourceDisposition::from_prepared_authorized_source_with_partial_permit(
                    prepared,
                    true,
                    1024 * 1024,
                    "opencode",
                    "capture_partial",
                    Some("ses_verified_partial"),
                )
                .expect("verified matching mapping");
            assert!(!disposition.has_capture_consumer());
            assert!(!String::from_utf8_lossy(&ordinary).contains("PRIVATE_REASONING_CANARY"));
            assert!(!String::from_utf8_lossy(&ordinary).contains("PRIVATE_STATE_CANARY"));
            let safe: serde_json::Value =
                serde_json::from_slice(&ordinary).expect("safe projection");
            if let Some(parts) = safe["messages"][1]["content"].as_array() {
                for part in parts {
                    if part["type"] == "reasoning" {
                        for erased in [
                            "text",
                            "time",
                            "state",
                            "signature",
                            "redactedData",
                            "reasoningEncryptedContent",
                        ] {
                            assert!(
                                part.get(erased).is_none(),
                                "associated {erased} was retained"
                            );
                        }
                    }
                }
            }
            permit.is_some()
        }
        for state in [
            serde_json::json!({"reasoningField":"PRIVATE_STATE_CANARY"}),
            serde_json::json!({"signature":"PRIVATE_STATE_CANARY"}),
            serde_json::json!({"redactedData":"PRIVATE_STATE_CANARY"}),
            serde_json::json!({"reasoningEncryptedContent":"PRIVATE_STATE_CANARY"}),
            serde_json::json!({"anthropic":{"signature":"PRIVATE_STATE_CANARY"}}),
            serde_json::json!({"signature":"PRIVATE_STATE_CANARY", "redactedData":"PRIVATE_STATE_CANARY", "reasoningEncryptedContent":"PRIVATE_STATE_CANARY"}),
        ] {
            assert!(has_permit(source(state)));
        }
        for field in ["signature", "redactedData", "reasoningEncryptedContent"] {
            let mut value = source(serde_json::json!({"reasoningField":"PRIVATE_STATE_CANARY"}));
            value["messages"][1]["content"][0][field] = serde_json::json!("PRIVATE_STATE_CANARY");
            assert!(
                has_permit(value),
                "top-level {field} must be erased before eligibility"
            );
        }
        for path in ["root", "info", "nested", "part"] {
            let mut value = source(serde_json::json!({"signature":"PRIVATE_STATE_CANARY"}));
            match path {
                "root" => value["originProof"] = serde_json::json!(null),
                "info" => value["info"]["originProof"] = serde_json::json!(17),
                "nested" => {
                    value["info"]["metadata"] = serde_json::json!({"outer":{"originProof":false}})
                }
                _ => {
                    value["messages"][1]["content"][0]["originProof"] =
                        serde_json::json!({"nonce":"invalid"})
                }
            }
            assert!(!has_permit(value), "originProof in {path}");
        }
        let escaped_origin = serde_json::to_vec(&source(
            serde_json::json!({"reasoningField":"PRIVATE_STATE_CANARY", "originProof":null}),
        ))
        .unwrap();
        let escaped_origin = String::from_utf8(escaped_origin)
            .unwrap()
            .replace("originProof", r"origin\u0050roof");
        let escaped = prepare_authorized_source(escaped_origin.into_bytes()).unwrap();
        let (_, _, permit) =
            ReasoningSourceDisposition::from_prepared_authorized_source_with_partial_permit(
                escaped,
                true,
                1024 * 1024,
                "opencode",
                "capture_partial",
                Some("ses_verified_partial"),
            )
            .unwrap();
        assert!(
            permit.is_none(),
            "canonical escaped originProof must remain blocked"
        );
        let escaped_duplicate = br#"{"info":{"id":"ses_verified_partial"},"messages":[],"originProof":null,"origin\u0050roof":false}"#;
        assert!(
            matches!(
                prepare_authorized_source(escaped_duplicate.to_vec()),
                Err(ReasoningFieldError::MalformedSource)
            ),
            "decoded duplicate originProof keys must reject the whole source"
        );
        let classic = serde_json::json!({
            "info":{"id":"ses_verified_partial"},
            "messages":[
                {"info":{"id":"msg_user","role":"user"},"parts":[{"type":"text","text":"safe request"}]},
                {"info":{"id":"msg_assistant","role":"assistant"},"parts":[
                    {"type":"reasoning","text":"PRIVATE_REASONING_CANARY","state":{"signature":"PRIVATE_STATE_CANARY"}},
                    {"type":"text","text":"safe reply"}
                ]}
            ]
        });
        assert!(
            !has_permit(classic),
            "classic roots cannot mint a native-format partial permit"
        );
        let mut candidate = source(serde_json::json!({"signature":"PRIVATE_STATE_CANARY"}));
        candidate["messages"][1]["content"][0]["id"] =
            serde_json::json!("npart_0123456789abcdef0123456789abcdef");
        candidate["messages"][1]["content"][0]["originProof"] =
            serde_json::json!({"receipt":"fixture", "nonce":3});
        let candidate = prepare_authorized_source(serde_json::to_vec(&candidate).unwrap()).unwrap();
        let (_, disposition, permit) =
            ReasoningSourceDisposition::from_prepared_authorized_source_with_partial_permit(
                candidate,
                true,
                1024 * 1024,
                "opencode",
                "capture_partial",
                Some("ses_verified_partial"),
            )
            .unwrap();
        assert!(
            matches!(
                disposition,
                ReasoningSourceDisposition::OpenCodeCandidate(_)
            ),
            "fixture must exercise a present sealed candidate"
        );
        assert!(!disposition.has_capture_consumer());
        assert!(
            permit.is_none(),
            "present candidate cannot take the ordinary partial lane"
        );
        let unrecognized_only = serde_json::json!({
            "info":{"id":"ses_verified_partial"},
            "messages":[{"id":"msg_unknown","type":"Assistant","text":"PRIVATE_REASONING_CANARY",
                "metadata":{"signature":"PRIVATE_STATE_CANARY"}}]
        });
        assert!(
            !has_permit(unrecognized_only),
            "unrecognized-only source has no retained meaningful ordinary content"
        );
        let mut skeleton = source(serde_json::json!({"reasoningField":"PRIVATE_STATE_CANARY"}));
        skeleton["messages"]
            .as_array_mut()
            .expect("messages")
            .remove(0);
        skeleton["messages"][0]["content"]
            .as_array_mut()
            .expect("content")
            .pop();
        assert!(!has_permit(skeleton));
        let value = source(serde_json::json!({"reasoningField":"PRIVATE_STATE_CANARY"}));
        let prepared = prepare_authorized_source(serde_json::to_vec(&value).unwrap()).unwrap();
        assert!(
            ReasoningSourceDisposition::from_prepared_authorized_source_with_partial_permit(
                prepared,
                true,
                1024 * 1024,
                "opencode",
                "capture_partial",
                Some("wrong_session")
            )
            .is_err()
        );
        for bytes in [
            b"{\"info\":{},\"info\":{},\"messages\":[]}".to_vec(),
            [serde_json::to_vec(&value).unwrap(), b" {}".to_vec()].concat(),
            b"{broken".to_vec(),
        ] {
            assert!(prepare_authorized_source(bytes).is_err());
        }
        let cap = usize::try_from(TRANSCRIPT_READ_HARD_CAP_BYTES).expect("fixture byte cap");
        let mut dense = Vec::with_capacity(2 * (OPENCODE_CANON_MAX_NODES + 1) + 2);
        dense.push(b'[');
        for ordinal in 0..=OPENCODE_CANON_MAX_NODES {
            if ordinal > 0 {
                dense.push(b',');
            }
            dense.push(b'0');
        }
        dense.push(b']');
        for (budget, bytes) in [
            ("byte", vec![b' '; cap + 1]),
            ("node", dense),
            (
                "depth",
                format!(
                    "{}0{}",
                    "[".repeat(MAX_JSON_DEPTH + 2),
                    "]".repeat(MAX_JSON_DEPTH + 2)
                )
                .into_bytes(),
            ),
        ] {
            assert!(
                matches!(
                    prepare_authorized_source(bytes),
                    Err(ReasoningFieldError::SourceTooLarge)
                ),
                "{budget} limit cannot mint a permit"
            );
        }
        let unmapped = prepare_authorized_source(serde_json::to_vec(&value).unwrap()).unwrap();
        let (_, disposition, permit) =
            ReasoningSourceDisposition::from_prepared_authorized_source_with_partial_permit(
                unmapped,
                false,
                1024 * 1024,
                "opencode",
                "capture_partial",
                Some("ses_verified_partial"),
            )
            .unwrap();
        assert!(matches!(disposition, ReasoningSourceDisposition::Unmapped));
        assert!(permit.is_none());
        for (provider, capture, source_session) in [
            ("codex", "capture_partial", Some("ses_verified_partial")),
            ("opencode", "", Some("ses_verified_partial")),
            ("opencode", "capture_partial", None),
        ] {
            let prepared = prepare_authorized_source(serde_json::to_vec(&value).unwrap()).unwrap();
            assert!(
                ReasoningSourceDisposition::from_prepared_authorized_source_with_partial_permit(
                    prepared,
                    true,
                    1024 * 1024,
                    provider,
                    capture,
                    source_session
                )
                .is_err()
            );
        }
        let generic = PreparedReasoningSource::unproven_candidates(b"safe request".to_vec());
        let (_, _, permit) =
            ReasoningSourceDisposition::from_prepared_authorized_source_with_partial_permit(
                generic,
                true,
                1024 * 1024,
                "opencode",
                "capture_partial",
                Some("ses_verified_partial"),
            )
            .unwrap();
        assert!(permit.is_none());
    }
}
