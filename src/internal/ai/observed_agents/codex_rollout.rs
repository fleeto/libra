//! Pure metadata projection for Codex 0.162.1, source 092d3acd6bec3e3a14bdc7e7a2810ab628ab759d.
//! See history/rollout_payload.rs and protocol/{protocol,items}.rs upstream.
//! This does not normalize coverage, mutate transcripts, or authorize storage.

use std::{
    collections::{BTreeMap, BTreeSet},
    time::Instant,
};

use serde::{
    Deserialize,
    de::{self, IgnoredAny, Visitor},
};
use serde_json::Value;
use sha2::{Digest, Sha256};

use super::{
    capability::{SkillEvent, SkillEventSignal, SkillEventSource, SkillEventType, SkillRef},
    coverage::parse_canon_value,
    extract::{CODEX_SKILL_REGISTRY, ExtractionCollectionLimits, ExtractionSummary},
};
use crate::internal::ai::completion::CompletionUsageSummary;

const MAX_INPUT: usize = 16 * 1024 * 1024;
const MAX_LINE: usize = 1024 * 1024;
const MAX_STATE: usize = 4096;
const MAX_STRING: usize = 64 * 1024;
const MAX_RETAINED: usize = 1024 * 1024;
const SHAPE: &str = "Codex rollout contained an unsupported or incomplete record";
const BUDGET: &str = "Codex rollout metadata exceeded its bounded collection budget";
const USAGE: &str = "Codex rollout usage was missing, inconsistent or not exactly representable";
const IDENTITY: &str = "Codex rollout item identity was absent or conflicting";
const RESET: &str = "Codex legacy usage reset at an explicit context boundary; totals are partial";
const PATCH: &str = "Codex patch modification was unsuccessful or lacked completion provenance";

const INPUT_BUDGET: &str = "Codex rollout exceeded the 16 MiB input limit";
const LINE_BUDGET: &str = "Codex rollout exceeded the 1 MiB JSONL record limit";
const STRING_BUDGET: &str = "Codex rollout exceeded its string budget";
const STATE_BUDGET: &str = "Codex rollout exceeded its bounded identity-state count";
const USAGE_DEADLINE: &str = "Codex thread usage snapshot exceeded its reserved metadata deadline";
const USAGE_REDACTION: &str = "Codex usage may omit records removed by credential redaction";

/// Producer content annotations, shared by metadata and typed turn projection.
/// Message text itself never decides whether a user-role item is human input.
pub(super) fn human_content_kind(kind: &str) -> bool {
    kind.is_empty()
        || kind == "unknown"
        || kind.starts_with("user.")
        || matches!(
            kind,
            "images.preparation_error" | "images.unsupported" | "audio.unsupported"
        )
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum OffsetProjection {
    Prompts,
    Skills,
    Files,
    Usage,
}

impl OffsetProjection {
    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::Prompts => "prompt",
            Self::Skills => "skill",
            Self::Files => "file",
            Self::Usage => "token usage",
        }
    }
}

pub(super) fn exceeded_limit(out: &ExtractionSummary) -> Option<&'static str> {
    [
        (INPUT_BUDGET, "the 16 MiB input limit"),
        (LINE_BUDGET, "the 1 MiB JSONL record limit"),
        (STRING_BUDGET, "the 64 KiB string limit"),
        (STATE_BUDGET, "the bounded identity-state count"),
        (BUDGET, "the bounded collection or retained-output budget"),
    ]
    .into_iter()
    .find_map(|(warning, limit)| {
        out.warnings
            .iter()
            .any(|value| value == warning)
            .then_some(limit)
    })
}

pub(super) fn usage_is_inconsistent(out: &ExtractionSummary) -> bool {
    // The plain-total capability cannot represent a redaction-limited subtotal.
    out.warnings
        .iter()
        .any(|warning| warning == USAGE || warning == USAGE_REDACTION)
}

pub(super) fn usage_is_unproven(out: &ExtractionSummary) -> bool {
    out.warnings
        .iter()
        .any(|warning| warning == USAGE || warning == RESET || warning == USAGE_REDACTION)
}

// Oversized bodies are never materialized. Only these bounded schema fields
// may decide whether the record is irrelevant to the requested projection.
struct ThinText(String);
impl<'de> Deserialize<'de> for ThinText {
    fn deserialize<D: de::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct TextVisitor;
        impl Visitor<'_> for TextVisitor {
            type Value = ThinText;
            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("a bounded Codex metadata string")
            }
            fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> {
                if value.len() > MAX_STRING {
                    return Err(E::custom("Codex metadata string exceeds its bound"));
                }
                Ok(ThinText(value.to_owned()))
            }
        }
        deserializer.deserialize_str(TextVisitor)
    }
}
fn thin_text(value: &Option<ThinText>) -> Option<&str> {
    value.as_ref().map(|value| value.0.as_str())
}
#[derive(Deserialize)]
struct ThinContext {
    turn_id: Option<ThinText>,
    cwd: Option<ThinText>,
}
#[derive(Deserialize)]
struct ThinItem {
    #[serde(rename = "type")]
    kind: Option<ThinText>,
}
#[derive(Deserialize)]
struct ThinPayload {
    #[serde(rename = "type")]
    kind: Option<ThinText>,
    role: Option<ThinText>,
    thread_id: Option<ThinText>,
    turn_id: Option<ThinText>,
    cwd: Option<ThinText>,
    item: Option<ThinItem>,
    internal_chat_message_metadata_passthrough: Option<ThinContext>,
}
#[derive(Deserialize)]
struct ThinFlat {
    role: Option<ThinText>,
    usage: Option<IgnoredAny>,
    token_usage: Option<IgnoredAny>,
}
#[derive(Deserialize)]
struct ThinRecordMetadata {
    inherited_user_message: Option<bool>,
}
#[derive(Deserialize)]
struct ThinRecord {
    #[serde(rename = "type")]
    kind: Option<ThinText>,
    payload: Option<ThinPayload>,
    role: Option<ThinText>,
    message: Option<ThinFlat>,
    metadata: Option<ThinRecordMetadata>,
    usage: Option<IgnoredAny>,
    token_usage: Option<IgnoredAny>,
}
impl ThinRecord {
    fn irrelevant(&self, projection: OffsetProjection) -> bool {
        use OffsetProjection::{Files, Prompts, Skills, Usage};
        if matches!(
            thin_text(&self.kind),
            Some("world_state" | "security_risk_score" | "inter_agent_communication_metadata")
        ) {
            return true;
        }
        if let Some(payload) = &self.payload {
            return match (thin_text(&self.kind), thin_text(&payload.kind)) {
                (Some("token_usage_record" | "compacted"), _) => projection != Usage,
                (Some("response_item"), Some("message")) => match thin_text(&payload.role) {
                    Some("user") => {
                        self.metadata
                            .as_ref()
                            .and_then(|metadata| metadata.inherited_user_message)
                            == Some(true)
                            || matches!(projection, Usage | Files)
                    }
                    _ => true,
                },
                (
                    Some("response_item"),
                    Some(
                        "reasoning"
                        | "function_call"
                        | "function_call_output"
                        | "custom_tool_call"
                        | "custom_tool_call_output"
                        | "web_search_call"
                        | "compaction"
                        | "agent_message"
                        | "local_shell_call"
                        | "tool_search_call"
                        | "tool_search_output"
                        | "image_generation_call"
                        | "additional_tools"
                        | "configuration_update"
                        | "context_compaction",
                    ),
                ) => true,
                (Some("event_msg"), Some("user_message")) => matches!(projection, Usage | Files),
                (Some("event_msg"), Some("token_count")) => projection != Usage,
                (Some("event_msg"), Some("patch_apply_end")) => projection != Files,
                (Some("event_msg"), Some("item_completed")) => {
                    match payload.item.as_ref().and_then(|item| thin_text(&item.kind)) {
                        Some("UserMessage") => matches!(projection, Usage | Files),
                        Some("FileChange") => matches!(projection, Usage | Prompts | Skills),
                        Some(
                            "HookPrompt"
                            | "AgentMessage"
                            | "Reasoning"
                            | "Plan"
                            | "CommandExecution"
                            | "FunctionCallOutput"
                            | "ContextCompaction"
                            | "WebSearch"
                            | "McpToolCall"
                            | "ImageGeneration"
                            | "ImageView"
                            | "DynamicToolCall"
                            | "CollabAgentToolCall"
                            | "SubAgentActivity"
                            | "Extension"
                            | "EnteredReviewMode"
                            | "ExitedReviewMode",
                        ) => true,
                        _ => false,
                    }
                }
                (
                    Some("event_msg"),
                    Some(
                        "task_started"
                        | "turn_started"
                        | "task_complete"
                        | "turn_complete"
                        | "turn_aborted"
                        | "agent_message"
                        | "agent_reasoning"
                        | "agent_reasoning_raw_content"
                        | "item_started"
                        | "patch_apply_begin"
                        | "patch_apply_updated"
                        | "context_compacted"
                        | "entered_review_mode"
                        | "exited_review_mode"
                        | "mcp_tool_call_end"
                        | "web_search_end"
                        | "image_generation_end"
                        | "sub_agent_activity"
                        | "thread_settings_applied",
                    ),
                ) => true,
                _ => false,
            };
        }
        let role = self
            .message
            .as_ref()
            .and_then(|message| thin_text(&message.role))
            .or_else(|| thin_text(&self.role));
        if !matches!(role, Some("user" | "assistant")) {
            return false;
        }
        match projection {
            Files => true,
            Prompts | Skills => role == Some("assistant"),
            Usage => {
                self.usage.is_none()
                    && self.token_usage.is_none()
                    && self.message.as_ref().is_none_or(|message| {
                        message.usage.is_none() && message.token_usage.is_none()
                    })
            }
        }
    }
}
#[derive(Deserialize)]
struct ThinOuter {
    #[serde(rename = "type")]
    kind: Option<ThinText>,
}
fn ignored_outer_record(line: &[u8]) -> bool {
    serde_json::from_slice::<ThinOuter>(line).is_ok_and(|record| {
        matches!(
            thin_text(&record.kind),
            Some("world_state" | "security_risk_score" | "inter_agent_communication_metadata")
        )
    })
}
fn ignorable_large_record(line: &[u8], projection: Option<OffsetProjection>) -> bool {
    line.iter().all(u8::is_ascii_whitespace)
        || projection.is_some_and(|projection| {
            serde_json::from_slice::<ThinRecord>(line).map_or_else(
                |_| ignored_outer_record(line),
                |record| record.irrelevant(projection),
            )
        })
}

type Key = [u8; 32];

fn key(parts: &[&str]) -> Key {
    let mut hash = Sha256::new();
    for part in parts {
        hash.update(part.len().to_le_bytes());
        hash.update(part.as_bytes());
    }
    hash.finalize().into()
}

fn text<'a>(value: &'a Value, field: &str) -> Option<&'a str> {
    value.get(field)?.as_str()
}

fn partial(out: &mut ExtractionSummary, warning: &'static str) {
    out.partial = true;
    if !out.warnings.iter().any(|old| old == warning) {
        out.warnings.push(warning.to_owned());
    }
}

/// Credential rules may hide structure while preserving JSON, or consume a
/// closing quote and invalidate a usage-bearing row. A valid nonstructural
/// replacement alone does not invalidate the observed usage.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct UsageRedactionScan {
    has_marker: bool,
    may_hide_records: bool,
}

fn scan_usage_redaction(data: &[u8], deadline: Option<Instant>) -> Option<UsageRedactionScan> {
    const CHUNK: usize = 16 * 1024;
    const STRUCTURAL_MARKERS: &[&[u8]] = &[
        b"REDACTED:private-key-pem>",
        b"REDACTED:google-service-account-private-key>",
        b"REDACTED:credential-uri>",
    ];
    // Each suffix follows '<'; derive overlap from the whole marker set.
    const OVERLAP: usize = {
        let mut longest = b"REDACTED:".len();
        let mut index = 0;
        while index < STRUCTURAL_MARKERS.len() {
            if STRUCTURAL_MARKERS[index].len() > longest {
                longest = STRUCTURAL_MARKERS[index].len();
            }
            index += 1;
        }
        longest
    };
    if deadline.is_some_and(|end| Instant::now() >= end) {
        return None;
    }
    let mut observed = UsageRedactionScan::default();
    for start in (0..data.len()).step_by(CHUNK) {
        if deadline.is_some_and(|end| Instant::now() >= end) {
            return None;
        }
        let end = (start + CHUNK + OVERLAP).min(data.len());
        for suffix in data[start..end].split(|byte| *byte == b'<').skip(1) {
            observed.has_marker |= suffix.starts_with(b"REDACTED:");
            if STRUCTURAL_MARKERS
                .iter()
                .any(|marker| suffix.starts_with(marker))
            {
                observed.may_hide_records = true;
                return Some(observed);
            }
        }
    }
    Some(observed)
}

/// Each value is an upstream i64 counter. Cache/reasoning counters are retained
/// separately, never added to total. Missing required counters remain unknown.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Counters([u64; 5]);

impl Counters {
    fn read(value: &Value) -> Option<Self> {
        let get = |field| u64::try_from(value.get(field)?.as_i64()?).ok();
        let input = get("input_tokens")?;
        let output = get("output_tokens")?;
        let cache_write = match value.get("cache_write_input_tokens") {
            None => 0, // Upstream serde default, unlike the required counters.
            Some(value) => u64::try_from(value.as_i64()?).ok()?,
        };
        let cached = get("cached_input_tokens")?.checked_add(cache_write)?;
        let reasoning = get("reasoning_output_tokens")?;
        let total = get("total_tokens")?;
        if input.checked_add(output)? != total {
            return None;
        }
        Some(Self([input, output, cached, reasoning, total]))
    }

    fn checked(self, other: Self, subtract: bool) -> Option<Self> {
        let mut values = [0; 5];
        for (index, value) in values.iter_mut().enumerate() {
            *value = if subtract {
                self.0[index].checked_sub(other.0[index])?
            } else {
                self.0[index].checked_add(other.0[index])?
            };
        }
        Some(Self(values))
    }

    fn summary(self) -> CompletionUsageSummary {
        CompletionUsageSummary {
            input_tokens: self.0[0],
            output_tokens: self.0[1],
            cached_tokens: Some(self.0[2]),
            reasoning_tokens: Some(self.0[3]),
            total_tokens: Some(self.0[4]),
            cost_usd: None,
        }
    }
}

struct PromptCandidate {
    scope: Key,
    identity: Option<Key>,
    source: usize,
    new_data: bool,
    text: String,
    turn: String,
    timestamp: String,
}

struct Parser {
    out: ExtractionSummary,
    projection: Option<OffsetProjection>,
    usage_snapshot: bool,
    limits: ExtractionCollectionLimits,
    retained: usize,
    candidate_bytes: usize,
    legacy_warnings: BTreeSet<&'static str>,
    thread: String,
    foreign_partition: bool,
    turn: String,
    cwd: Option<String>,
    prompt_ids: BTreeMap<Key, Key>,
    prompts: Vec<PromptCandidate>,
    response_ids: BTreeMap<Key, Counters>,
    files: BTreeSet<String>,
    exact: Option<Counters>,
    exact_all: Option<Counters>,
    exact_all_valid: bool,
    legacy: Option<Counters>,
    legacy_last: Option<Counters>,
    legacy_prefix: Option<Counters>,
    first_usage_exact: Option<bool>,
    legacy_calls: u64,
    exact_calls: u64,
    emit: bool,
    patch_ids: BTreeMap<Key, Key>,
    reset_allowed: bool,
}

impl Parser {
    fn new(
        limits: Option<ExtractionCollectionLimits>,
        projection: Option<OffsetProjection>,
    ) -> Self {
        let limits = limits.unwrap_or(ExtractionCollectionLimits::new(
            MAX_STATE,
            MAX_STRING,
            MAX_RETAINED,
        ));
        Self {
            out: ExtractionSummary::default(),
            projection,
            usage_snapshot: false,
            limits: ExtractionCollectionLimits::new(
                limits.max_items_per_collection.min(MAX_STATE),
                limits.max_string_bytes.min(MAX_STRING),
                limits.max_retained_string_bytes.min(MAX_RETAINED),
            ),
            retained: 0,
            candidate_bytes: 0,
            legacy_warnings: BTreeSet::new(),
            thread: String::new(),
            foreign_partition: false,
            turn: String::new(),
            cwd: None,
            prompt_ids: BTreeMap::new(),
            prompts: Vec::new(),
            response_ids: BTreeMap::new(),
            files: BTreeSet::new(),
            exact: None,
            exact_all: None,
            exact_all_valid: true,
            legacy: None,
            legacy_last: None,
            legacy_prefix: None,
            first_usage_exact: None,
            legacy_calls: 0,
            exact_calls: 0,
            emit: true,
            patch_ids: BTreeMap::new(),
            reset_allowed: false,
        }
    }

    fn admit(&mut self, count: usize, bytes: usize) -> bool {
        if bytes > self.limits.max_string_bytes {
            partial(&mut self.out, STRING_BUDGET);
            return false;
        }
        if count >= self.limits.max_items_per_collection
            || self.retained.saturating_add(bytes) > self.limits.max_retained_string_bytes
        {
            partial(&mut self.out, BUDGET);
            return false;
        }
        self.retained += bytes;
        true
    }

    fn skip_large_record(&mut self, line: &[u8]) -> bool {
        if line.iter().all(u8::is_ascii_whitespace) {
            return true;
        }
        let Some(projection) = self.projection else {
            return false;
        };
        let Ok(record) = serde_json::from_slice::<ThinRecord>(line) else {
            let ignored = ignored_outer_record(line);
            if ignored {
                partial(&mut self.out, SHAPE);
            }
            return ignored;
        };
        // Only a fully decoded owner header can clear a foreign partition.
        if thin_text(&record.kind) == Some("session_meta") {
            return false;
        }
        if let Some(payload) = &record.payload {
            let thread = thin_text(&payload.thread_id);
            if (self.foreign_partition && thread.is_none())
                || thread.is_some_and(|thread| !self.thread.is_empty() && thread != self.thread)
            {
                partial(&mut self.out, SHAPE);
                return true;
            }
            if !record.irrelevant(projection) {
                return false;
            }
            if thin_text(&record.kind) == Some("event_msg")
                && matches!(
                    thin_text(&payload.kind),
                    Some("item_completed" | "task_started" | "turn_started")
                )
            {
                self.context(&serde_json::json!({"turn_id":thin_text(&payload.turn_id),"cwd":thin_text(&payload.cwd)}));
            }
            if thin_text(&record.kind) == Some("response_item")
                && thin_text(&payload.kind) == Some("message")
                && thin_text(&payload.role) == Some("user")
                && record
                    .metadata
                    .as_ref()
                    .and_then(|metadata| metadata.inherited_user_message)
                    != Some(true)
                && let Some(context) = &payload.internal_chat_message_metadata_passthrough
            {
                self.context(&serde_json::json!({"turn_id":thin_text(&context.turn_id),"cwd":thin_text(&context.cwd)}));
            }
        } else if !record.irrelevant(projection) {
            return false;
        }
        partial(&mut self.out, SHAPE); // Body omitted, never an attestation of complete metadata.
        true
    }

    fn context(&mut self, value: &Value) {
        if self.projection == Some(OffsetProjection::Usage) {
            return;
        }
        if let Some(turn) = text(value, "turn_id") {
            if turn.len() <= MAX_STRING {
                self.turn = turn.to_owned();
            } else {
                partial(&mut self.out, STRING_BUDGET);
            }
        }
        if (self.projection.is_none() || self.projection == Some(OffsetProjection::Files))
            && let Some(cwd) = text(value, "cwd")
        {
            if cwd.len() <= MAX_STRING {
                self.cwd = Some(cwd.to_owned());
            } else {
                partial(&mut self.out, STRING_BUDGET);
            }
        }
    }

    fn model(&mut self, value: &Value) {
        if self.projection.is_some() {
            return;
        }
        let Some(model) = text(value, "model").filter(|model| !model.is_empty()) else {
            partial(&mut self.out, SHAPE);
            return;
        };
        if self.out.model.as_deref() != Some(model) && self.admit(0, model.len()) {
            self.out.model = Some(model.to_owned());
        }
    }

    fn prompt(&mut self, value: &Value, id: Option<&str>, timestamp: &str, source: usize) {
        if matches!(
            self.projection,
            Some(OffsetProjection::Files | OffsetProjection::Usage)
        ) {
            return;
        }
        let bounded_output = self.emit && self.projection != Some(OffsetProjection::Skills);
        let mut prompt = String::new();
        if let Some(message) = text(value, "message") {
            if bounded_output && message.len() > self.limits.max_string_bytes {
                partial(&mut self.out, STRING_BUDGET);
                return;
            }
            prompt.push_str(message);
        } else if let Some(content) = value.get("content").and_then(Value::as_array) {
            let kinds = value
                .get("internal_chat_message_metadata_passthrough")
                .and_then(|meta| meta.get("content_item_kinds"))
                .and_then(Value::as_array);
            // Raw user-role response items also contain injected host context.
            // Only producer annotations or the event-side input identify a prompt.
            if source == 0
                && kinds.is_none_or(|kinds| {
                    kinds.len() != content.len() || kinds.iter().any(|kind| !kind.is_string())
                })
            {
                partial(&mut self.out, SHAPE);
                return;
            }
            for (index, block) in content.iter().enumerate() {
                if let Some(kind) = kinds
                    .filter(|kinds| kinds.len() == content.len())
                    .and_then(|kinds| kinds[index].as_str())
                    && !human_content_kind(kind)
                {
                    continue;
                }
                match text(block, "type") {
                    Some("input_text" | "text") => {
                        let Some(part) = text(block, "text") else {
                            partial(&mut self.out, SHAPE);
                            continue;
                        };
                        let size = prompt
                            .len()
                            .saturating_add(part.len())
                            .saturating_add(usize::from(!prompt.is_empty()));
                        if bounded_output && size > self.limits.max_string_bytes {
                            partial(&mut self.out, STRING_BUDGET);
                            return;
                        }
                        if !prompt.is_empty() {
                            prompt.push('\n');
                        }
                        prompt.push_str(part);
                    }
                    Some(
                        "input_image" | "image" | "local_image" | "audio" | "local_audio" | "skill"
                        | "mention",
                    ) => (),
                    _ => partial(&mut self.out, SHAPE),
                }
            }
        } else {
            partial(&mut self.out, SHAPE);
            return;
        }
        if prompt.is_empty() {
            return;
        }
        let digest = key(&[&prompt]);
        let identity = id
            .filter(|id| !id.is_empty())
            .map(|id| key(&[&self.thread, id]));
        if let Some(id) = id.filter(|id| !id.is_empty()) {
            // IDs are unique per producer namespace, not per mutable turn.
            let scoped = key(&[&self.thread, &source.to_string(), id]);
            if let Some(old) = self.prompt_ids.get(&scoped) {
                if *old != digest {
                    partial(&mut self.out, IDENTITY);
                }
                return;
            }
            if self.prompt_ids.len() >= MAX_STATE * 3 {
                partial(&mut self.out, STATE_BUDGET);
                return;
            }
            self.prompt_ids.insert(scoped, digest);
        } else {
            partial(&mut self.out, IDENTITY);
        }
        let scope = key(&[&self.thread, &self.turn, &prompt]);
        let retained_prompt = if !self.emit {
            String::new()
        } else if self.projection == Some(OffsetProjection::Skills) {
            prompt
                .split_whitespace()
                .next()
                .filter(|word| CODEX_SKILL_REGISTRY.contains(word))
                .unwrap_or("")
                .to_owned()
        } else {
            prompt
        };
        let retain_context = self.emit
            && !retained_prompt.is_empty()
            && self.projection != Some(OffsetProjection::Prompts);
        let turn = if retain_context {
            self.turn.clone()
        } else {
            String::new()
        };
        let timestamp = if retain_context {
            timestamp.to_owned()
        } else {
            String::new()
        };
        let bytes = retained_prompt
            .len()
            .saturating_add(turn.len())
            .saturating_add(timestamp.len());
        // Candidate copies have a separate 3-surface budget. Only surviving
        // prompts consume the public output budget in finish_prompts.
        if self.prompts.len() >= self.limits.max_items_per_collection.saturating_mul(3)
            || retained_prompt.len() > self.limits.max_string_bytes
            || turn.len() > self.limits.max_string_bytes
            || timestamp.len() > self.limits.max_string_bytes
            || self.candidate_bytes.saturating_add(bytes)
                > self.limits.max_retained_string_bytes.saturating_mul(3)
        {
            partial(&mut self.out, BUDGET);
            return;
        }
        self.candidate_bytes += bytes;
        self.prompts.push(PromptCandidate {
            scope,
            identity,
            source,
            new_data: self.emit,
            text: retained_prompt,
            turn,
            timestamp,
        });
    }

    fn finish_prompts(&mut self) {
        let mut eligible = self
            .prompts
            .iter()
            .map(|prompt| prompt.new_data)
            .collect::<Vec<_>>();
        let mut representatives: BTreeMap<Key, usize> = BTreeMap::new();
        let mut candidates: BTreeSet<usize> = (0..self.prompts.len()).collect();
        for (index, prompt) in self.prompts.iter().enumerate() {
            if let Some(identity) = prompt.identity {
                if let Some(previous) = representatives.get(&identity).copied() {
                    let old = &self.prompts[previous];
                    if old.scope != prompt.scope {
                        partial(&mut self.out, IDENTITY);
                    }
                    let new_data = eligible[previous] && eligible[index];
                    if prompt.source > old.source {
                        eligible[index] = new_data;
                        candidates.remove(&previous);
                        representatives.insert(identity, index);
                    } else {
                        eligible[previous] = new_data;
                        candidates.remove(&index);
                    }
                } else {
                    representatives.insert(identity, index);
                }
            }
        }
        let mut groups: BTreeMap<Key, [Vec<usize>; 3]> = BTreeMap::new();
        for index in candidates {
            let prompt = &self.prompts[index];
            groups.entry(prompt.scope).or_default()[prompt.source].push(index);
        }
        let mut selected = BTreeMap::new();
        for sources in groups.values() {
            // Pair occurrence-for-occurrence only across producer surfaces.
            // Repeated text on one surface is never silently deduplicated.
            let wanted = sources.iter().map(Vec::len).max().unwrap_or(0);
            let chosen = (0..wanted)
                .filter_map(|occurrence| {
                    sources
                        .iter()
                        .rev()
                        .find_map(|items| items.get(occurrence))
                        .copied()
                })
                .collect::<Vec<_>>();
            if sources.iter().filter(|items| !items.is_empty()).count() > 1 {
                for index in sources.iter().flatten() {
                    let identity = self.prompts[*index].identity;
                    if identity.is_none()
                        || !chosen
                            .iter()
                            .any(|chosen| self.prompts[*chosen].identity == identity)
                    {
                        // Without a shared ID this is a lower-bound projection;
                        // original source values are retained by a different owner.
                        partial(&mut self.out, IDENTITY);
                    }
                }
            }
            for (occurrence, index) in chosen.into_iter().enumerate() {
                let new_data = sources
                    .iter()
                    .filter_map(|items| items.get(occurrence))
                    .all(|item| eligible[*item]);
                eligible[index] &= new_data;
                selected.insert(index, occurrence);
            }
        }
        let prompts = std::mem::take(&mut self.prompts);
        for (index, prompt) in prompts.into_iter().enumerate() {
            let Some(occurrence) = selected.get(&index) else {
                continue;
            };
            let emit_prompts = self.projection != Some(OffsetProjection::Skills);
            if !eligible[index]
                || (emit_prompts && !self.admit(self.out.prompts.len(), prompt.text.len()))
            {
                continue;
            }
            if self.projection != Some(OffsetProjection::Prompts)
                && let Some(skill) = prompt
                    .text
                    .split_whitespace()
                    .next()
                    .filter(|skill| CODEX_SKILL_REGISTRY.contains(skill))
            {
                // Identity stays stable when an offset read suppresses earlier
                // prompts or a higher-priority mirror replaces a representative.
                let identity = match prompt.identity {
                    Some(identity) => hex::encode(identity),
                    None => format!("{}-{}", hex::encode(prompt.scope), occurrence),
                };
                let event_id = format!("codex:prompt-{identity}:{skill}");
                let bytes = event_id.len() + prompt.turn.len() + prompt.timestamp.len() + 32;
                if self.admit(self.out.skill_events.len(), bytes) {
                    self.out.skill_events.push(SkillEvent {
                        id: event_id,
                        event_type: SkillEventType::PromptInvocation,
                        skill: SkillRef {
                            name: skill.to_owned(),
                        },
                        source: SkillEventSource {
                            agent: "codex".to_owned(),
                            signal: SkillEventSignal::InputSlashCommand,
                            confidence: 1.0,
                        },
                        turn_id: prompt.turn,
                        timestamp: prompt.timestamp,
                        transcript_anchor: None,
                        native: false,
                        collapse: false,
                    });
                }
            }
            if emit_prompts {
                self.out.prompts.push(prompt.text);
            }
        }
    }

    fn observe_exact_total(&mut self, usage: Counters) {
        if !self.exact_all_valid {
            return;
        }
        self.exact_all = match self.exact_all {
            None => Some(usage),
            Some(old) => old.checked(usage, false),
        };
        if self.exact_all.is_none() {
            self.exact_all_valid = false;
            partial(&mut self.out, USAGE);
        }
    }

    fn exact_usage(&mut self, value: &Value) {
        if self
            .projection
            .is_some_and(|projection| projection != OffsetProjection::Usage)
        {
            return;
        }
        let identity_fields = ["thread_id", "session_id", "response_id"];
        let lossy_identity = identity_fields.iter().any(|field| {
            text(value, field).is_some_and(|identity| identity.contains("<REDACTED:"))
        });
        let missing_snapshot_identity = self.usage_snapshot
            && identity_fields
                .iter()
                .any(|field| text(value, field).is_none_or(str::is_empty));
        if lossy_identity || missing_snapshot_identity {
            // Different producer identities can collapse to the same
            // credential placeholder. Equal counters then cease to prove a
            // replay, for the ordinary parent reader as well as child usage.
            partial(&mut self.out, IDENTITY);
            partial(&mut self.out, USAGE);
            return;
        }
        if !self.accepts_thread(value) {
            return;
        }
        let Some(response) = text(value, "response_id").filter(|id| !id.is_empty()) else {
            partial(&mut self.out, IDENTITY);
            partial(&mut self.out, USAGE);
            return;
        };
        let Some(usage) = value.get("usage").and_then(Counters::read) else {
            partial(&mut self.out, USAGE);
            return;
        };
        let identity = key(&[
            text(value, "thread_id").unwrap_or(&self.thread),
            text(value, "session_id").unwrap_or(""),
            response,
        ]);
        if let Some(old) = self.response_ids.get(&identity) {
            if *old != usage {
                partial(&mut self.out, USAGE);
            }
            return;
        }
        if self.response_ids.len() >= MAX_STATE {
            partial(&mut self.out, STATE_BUDGET);
            return;
        }
        self.first_usage_exact.get_or_insert(true);
        if !self.emit {
            self.observe_exact_total(usage);
            self.response_ids.insert(identity, usage);
            return;
        }
        let sum = match self.exact {
            None => Some(usage),
            Some(old) => old.checked(usage, false),
        };
        if let Some(sum) = sum {
            self.observe_exact_total(usage);
            self.exact = Some(sum);
            self.exact_calls = self.exact_calls.saturating_add(1);
            self.response_ids.insert(identity, usage);
        } else {
            partial(&mut self.out, USAGE);
        }
    }

    fn legacy_usage(&mut self, value: &Value) {
        if self
            .projection
            .is_some_and(|projection| projection != OffsetProjection::Usage)
        {
            return;
        }
        if value.get("info").is_none_or(Value::is_null) && value.get("rate_limits").is_some() {
            return;
        }
        let Some(total) = value
            .get("info")
            .and_then(|info| info.get("total_token_usage"))
            .and_then(Counters::read)
        else {
            self.legacy_warnings.insert(USAGE);
            return;
        };
        self.first_usage_exact.get_or_insert(false);
        let delta = match self.legacy_last {
            None => Some(total),
            Some(last) => total.checked(last, true).or_else(|| {
                if self.reset_allowed {
                    self.legacy_warnings.insert(RESET);
                    Some(total)
                } else {
                    None
                }
            }),
        };
        let Some(delta) = delta else {
            self.legacy_warnings.insert(USAGE);
            return;
        };
        if !self.emit {
            self.legacy_prefix = Some(total);
            self.legacy_last = Some(total);
            self.reset_allowed = false;
            return;
        }
        let sum = match self.legacy {
            None => Some(delta),
            Some(old) => old.checked(delta, false),
        };
        let Some(sum) = sum else {
            self.legacy_warnings.insert(USAGE);
            return;
        };
        if delta.0 != [0; 5] {
            self.legacy_calls = self.legacy_calls.saturating_add(1);
        }
        self.legacy = Some(sum);
        self.legacy_last = Some(total);
        self.reset_allowed = false;
    }

    fn patch(&mut self, value: &Value, legacy: bool) {
        if self
            .projection
            .is_some_and(|projection| projection != OffsetProjection::Files)
        {
            return;
        }
        if text(value, if legacy { "call_id" } else { "id" }).is_none_or(str::is_empty)
            || text(value, "status") != Some("completed")
            || (legacy && value.get("success").and_then(Value::as_bool) != Some(true))
        {
            partial(&mut self.out, PATCH);
            return;
        }
        let Some(changes) = value.get("changes").and_then(Value::as_object) else {
            partial(&mut self.out, PATCH);
            return;
        };
        let Some(id) = text(value, if legacy { "call_id" } else { "id" }) else {
            return;
        };
        let identity = key(&[
            &self.thread,
            text(value, "turn_id").unwrap_or(&self.turn),
            id,
        ]);
        let Ok(encoded) = serde_json::to_string(changes) else {
            partial(&mut self.out, PATCH);
            return;
        };
        let digest = key(&[&encoded]);
        if let Some(previous) = self.patch_ids.get(&identity) {
            if *previous != digest {
                partial(&mut self.out, PATCH);
            }
            return;
        }
        if self.patch_ids.len() >= MAX_STATE {
            partial(&mut self.out, STATE_BUDGET);
            return;
        }
        self.patch_ids.insert(identity, digest);
        if !self.emit {
            return;
        }
        for (path, change) in changes {
            if !matches!(text(change, "type"), Some("add" | "delete" | "update")) {
                partial(&mut self.out, PATCH);
                continue;
            }
            self.file(path);
            if let Some(path) = text(change, "move_path") {
                self.file(path);
            }
        }
    }

    fn file(&mut self, path: &str) {
        let Some(path) = relative_path(path, self.cwd.as_deref()) else {
            partial(&mut self.out, PATCH);
            return;
        };
        if !self.files.contains(&path) && self.admit(self.files.len(), path.len()) {
            self.files.insert(path.clone());
            self.out.modified_files.push(path);
        }
    }

    fn event(&mut self, value: &Value, timestamp: &str) {
        if !self.accepts_thread(value) {
            return;
        }
        match text(value, "type") {
            Some("task_started" | "turn_started") => self.context(value),
            Some("user_message") => {
                if value
                    .get("kind")
                    .is_some_and(|kind| !kind.is_null() && kind.as_str() != Some("plain"))
                {
                    partial(&mut self.out, SHAPE);
                } else {
                    self.prompt(value, text(value, "client_id"), timestamp, 1);
                }
            }
            Some("token_count") => self.legacy_usage(value),
            Some("collab_agent_spawn_end") => {
                let sender = text(value, "sender_thread_id");
                let child = value.get("new_thread_id").filter(|child| !child.is_null());
                if self.thread.is_empty()
                    || sender != Some(self.thread.as_str())
                    || child.is_some_and(|child| {
                        child.as_str().is_none_or(|id| {
                            id.is_empty()
                                || id.len() > 64
                                || Some(id) == sender
                                || id.contains("<REDACTED:")
                        })
                    })
                {
                    partial(&mut self.out, SHAPE);
                }
            }
            Some("item_completed") => {
                self.context(value);
                if let Some(item) = value.get("item") {
                    match text(item, "type") {
                        Some("UserMessage") => self.prompt(item, text(item, "id"), timestamp, 2),
                        Some("FileChange") => self.patch(item, false),
                        Some(
                            "HookPrompt"
                            | "AgentMessage"
                            | "Reasoning"
                            | "Plan"
                            | "CommandExecution"
                            | "FunctionCallOutput"
                            | "ContextCompaction"
                            | "WebSearch"
                            | "McpToolCall"
                            | "ImageGeneration"
                            | "ImageView"
                            | "DynamicToolCall"
                            | "CollabAgentToolCall"
                            | "SubAgentActivity"
                            | "Extension"
                            | "EnteredReviewMode"
                            | "ExitedReviewMode",
                        ) => (),
                        _ => partial(&mut self.out, SHAPE),
                    }
                } else {
                    partial(&mut self.out, SHAPE);
                }
            }
            Some("patch_apply_end") => self.patch(value, true),
            Some(
                "task_complete"
                | "turn_complete"
                | "turn_aborted"
                | "agent_message"
                | "agent_reasoning"
                | "agent_reasoning_raw_content"
                | "item_started"
                | "patch_apply_begin"
                | "patch_apply_updated"
                | "context_compacted"
                | "entered_review_mode"
                | "exited_review_mode"
                | "mcp_tool_call_end"
                | "web_search_end"
                | "image_generation_end"
                | "sub_agent_activity"
                | "thread_settings_applied",
            ) => (),
            _ => partial(&mut self.out, SHAPE),
        }
    }

    fn accepts_thread(&mut self, value: &Value) -> bool {
        if value.get("thread_id").is_some_and(|thread| {
            thread
                .as_str()
                .is_none_or(|thread| !self.thread.is_empty() && thread != self.thread)
        }) {
            partial(&mut self.out, SHAPE);
            false
        } else {
            true
        }
    }
}

fn relative_path(path: &str, cwd: Option<&str>) -> Option<String> {
    if path.is_empty() || path.len() > MAX_STRING || path.contains(['\0', '\n', '\r']) {
        return None;
    }
    let path = path.replace('\\', "/");
    let root = cwd.map(|cwd| cwd.replace('\\', "/"));
    let absolute = path.starts_with('/') || path.as_bytes().get(1) == Some(&b':');
    let relative = if absolute {
        path.strip_prefix(root.as_ref()?.trim_end_matches('/'))?
            .strip_prefix('/')?
    } else {
        path.as_str()
    };
    if relative
        .split('/')
        .any(|part| part == ".." || part.contains(':'))
    {
        return None;
    }
    let clean = relative
        .split('/')
        .filter(|part| !part.is_empty() && *part != ".")
        .collect::<Vec<_>>()
        .join("/");
    (!clean.is_empty()).then_some(clean)
}

pub(super) fn extract(
    data: &[u8],
    limits: Option<ExtractionCollectionLimits>,
) -> ExtractionSummary {
    extract_from_offset(data, limits, 0, None)
}

// A record completed after a previous read is new, but adding only its final
// newline is not. Byte offsets never slice a UTF-8 String or discard context.
fn record_reaches_offset(line: &[u8], start: usize, from: usize) -> bool {
    if from <= start {
        return true;
    }
    let relative = from - start;
    relative < line.len() && parse_canon_value(&line[..relative]).is_err()
}

pub(super) fn extract_from_offset(
    data: &[u8],
    limits: Option<ExtractionCollectionLimits>,
    from: usize,
    projection: Option<OffsetProjection>,
) -> ExtractionSummary {
    extract_with_usage_scope(data, limits, from, projection, None)
}

#[derive(Clone, Copy)]
struct UsageSnapshotScope<'a> {
    thread: &'a str,
    deadline: Instant,
}

/// Whole-thread observations from the same bounded record engine. Cumulative
/// mirrors can expose incomplete exact history, but never contribute tokens.
/// This is ordinary metadata; it neither authorizes source reads nor changes
/// the original extractor/offset contracts.
pub(crate) fn extract_thread_usage_snapshot(
    data: &[u8],
    thread: &str,
    deadline: Instant,
) -> ExtractionSummary {
    extract_with_usage_scope(
        data,
        Some(ExtractionCollectionLimits::new(
            MAX_STATE,
            MAX_STRING,
            MAX_RETAINED,
        )),
        0,
        Some(OffsetProjection::Usage),
        Some(UsageSnapshotScope { thread, deadline }),
    )
}

fn extract_with_usage_scope(
    data: &[u8],
    limits: Option<ExtractionCollectionLimits>,
    from: usize,
    projection: Option<OffsetProjection>,
    usage_scope: Option<UsageSnapshotScope<'_>>,
) -> ExtractionSummary {
    if let Some(scope) = usage_scope
        && (scope.thread.is_empty() || scope.thread.len() > 64 || Instant::now() >= scope.deadline)
    {
        let mut out = ExtractionSummary::default();
        partial(
            &mut out,
            if scope.thread.is_empty() || scope.thread.len() > 64 {
                IDENTITY
            } else {
                USAGE_DEADLINE
            },
        );
        return out;
    }
    let from = from.min(data.len());
    let bounded = &data[..data.len().min(MAX_INPUT)];
    // Historical flat exports have no rollout envelope. Keep their established
    // compatibility behavior without applying generic heuristics to new roots.
    let rollout = bounded.split(|byte| *byte == b'\n').any(|line| {
        if usage_scope.is_some_and(|scope| Instant::now() >= scope.deadline) {
            return true;
        }
        line.len() <= MAX_LINE
            && serde_json::from_slice::<Value>(line).is_ok_and(|value| {
                value.get("payload").is_some_and(Value::is_object)
                    && value.get("type").is_some_and(Value::is_string)
            })
    });
    if usage_scope.is_some_and(|scope| Instant::now() >= scope.deadline) {
        let mut out = ExtractionSummary::default();
        partial(&mut out, USAGE_DEADLINE);
        return out;
    }
    if usage_scope.is_some() && !rollout {
        let mut out = ExtractionSummary::default();
        partial(&mut out, IDENTITY);
        return out;
    }
    if !rollout {
        let mut usable = Vec::new();
        let mut oversized = false;
        let mut invalid = false;
        let mut invalid_usage = false;
        let mut usage_redaction = false;
        let mut api_calls = 0u64;
        let mut usage_totals = [0u64; 5];
        let mut offset = 0usize;
        for segment in bounded.split_inclusive(|byte| *byte == b'\n') {
            let start = offset;
            offset += segment.len();
            let line = segment.strip_suffix(b"\n").unwrap_or(segment);
            let redaction = if matches!(projection, None | Some(OffsetProjection::Usage)) {
                scan_usage_redaction(line, None).unwrap_or_default()
            } else {
                UsageRedactionScan::default()
            };
            usage_redaction |= redaction.may_hide_records;
            usable.push(b'\n'); // Placeholder preserves the original record ordinal on skips.
            if line.len() > MAX_LINE {
                if redaction.has_marker && serde_json::from_slice::<IgnoredAny>(line).is_err() {
                    usage_redaction = true;
                }
                if !ignorable_large_record(line, projection) {
                    oversized = true;
                } else if !line.iter().all(u8::is_ascii_whitespace) {
                    invalid = true;
                }
                continue;
            }
            if line.iter().all(u8::is_ascii_whitespace) || !record_reaches_offset(line, start, from)
            {
                continue;
            }
            let Ok(mut entry) = serde_json::from_slice::<Value>(line) else {
                invalid = true;
                usage_redaction |= redaction.has_marker;
                continue;
            };
            if parse_canon_value(line).is_err() {
                invalid = true;
                usage_redaction |= redaction.has_marker;
                continue;
            }
            let record = entry.get("message").unwrap_or(&entry);
            let classifiable = matches!(text(record, "role"), Some("user" | "assistant"))
                || text(record, "model").is_some()
                || text(&entry, "model").is_some()
                || record.get("usage").is_some()
                || entry.get("usage").is_some()
                || entry.get("token_usage").is_some();
            if !classifiable {
                invalid = true;
                continue;
            }
            if text(&entry, "type")
                .is_some_and(|kind| !matches!(kind, "message" | "user" | "assistant"))
            {
                invalid = true;
            }
            match projection {
                Some(OffsetProjection::Files) => continue, // Flat rows never prove a completed patch.
                Some(OffsetProjection::Usage) => {
                    let record = entry.get("message").unwrap_or(&entry);
                    let usage = record
                        .get("usage")
                        .or_else(|| entry.get("usage"))
                        .or_else(|| entry.get("token_usage"))
                        .cloned();
                    entry = serde_json::json!({"usage": usage});
                }
                Some(OffsetProjection::Prompts | OffsetProjection::Skills) => {
                    if let Some(root) = entry.as_object_mut() {
                        for field in ["model", "usage", "token_usage"] {
                            root.remove(field);
                        }
                        if let Some(message) =
                            root.get_mut("message").and_then(Value::as_object_mut)
                        {
                            for field in ["model", "usage", "token_usage"] {
                                message.remove(field);
                            }
                        }
                    }
                }
                None => (),
            }
            // The historical E6 mapper assumes its arithmetic is representable.
            // Refuse malformed usage before entering that compatibility reader.
            let record = entry.get("message").unwrap_or(&entry);
            let usage = record
                .get("usage")
                .or_else(|| entry.get("usage"))
                .or_else(|| entry.get("token_usage"));
            if usage.is_some_and(|value| !value.is_null() && !value.is_object()) {
                invalid = true;
                invalid_usage = true;
            }
            if let Some(usage) = usage.filter(|value| value.is_object()) {
                let count = |field| {
                    usage
                        .get(field)
                        .filter(|value| !value.is_null())
                        .map_or(Some(0), Value::as_u64)
                };
                let valid = count("input_tokens")
                    .and_then(|a| count("output_tokens").and_then(|b| a.checked_add(b)))
                    .is_some()
                    && count("cache_creation_tokens")
                        .and_then(|a| count("cache_read_tokens").and_then(|b| a.checked_add(b)))
                        .is_some();
                if !valid {
                    invalid = true;
                    invalid_usage = true;
                    continue;
                }
                let calls = match usage.get("api_call_count").filter(|value| !value.is_null()) {
                    None => 1,
                    Some(value) => match value.as_u64() {
                        Some(0) => 1,
                        Some(calls) => calls,
                        None => {
                            invalid = true;
                            invalid_usage = true;
                            continue;
                        }
                    },
                };
                let Some(next) = api_calls.checked_add(calls) else {
                    invalid = true;
                    invalid_usage = true;
                    continue;
                };
                let values = [
                    count("input_tokens"),
                    count("output_tokens"),
                    count("cache_creation_tokens")
                        .and_then(|a| count("cache_read_tokens").and_then(|b| a.checked_add(b))),
                    count("input_tokens")
                        .and_then(|a| count("output_tokens").and_then(|b| a.checked_add(b))),
                    count("subagent_tokens"),
                ];
                let mut totals = usage_totals;
                let mut representable = true;
                for (index, value) in values.iter().enumerate() {
                    if let Some(sum) =
                        value.and_then(|value| usage_totals[index].checked_add(value))
                    {
                        totals[index] = sum;
                    } else {
                        representable = false;
                        break;
                    }
                }
                if !representable {
                    invalid = true;
                    invalid_usage = true;
                    continue;
                }
                usage_totals = totals;
                api_calls = next;
            }
            // Bare legacy ResponseItem uses input_text rather than the flat
            // reader's text blocks. Adapt a metadata-only copy, never source.
            let mut adapted = false;
            if text(&entry, "type") == Some("message")
                && text(&entry, "role") == Some("user")
                && let Some(content) = entry.get_mut("content").and_then(Value::as_array_mut)
            {
                for block in content {
                    if text(block, "type") == Some("input_text")
                        && let Some(block) = block.as_object_mut()
                    {
                        block.insert("type".to_owned(), Value::String("text".to_owned()));
                        adapted = true;
                    }
                }
            }
            if adapted {
                invalid = true;
            } // Historical role alone does not prove human input.
            if adapted || projection.is_some() {
                let Ok(encoded) = serde_json::to_vec(&entry) else {
                    continue;
                };
                usable.pop();
                usable.extend_from_slice(&encoded);
                usable.push(b'\n');
                continue;
            }
            usable.pop();
            usable.extend_from_slice(line);
            usable.push(b'\n');
        }
        let mut out = super::extract::extract_generic_jsonl_projected(
            &usable,
            "codex",
            CODEX_SKILL_REGISTRY,
            Some(limits.unwrap_or(ExtractionCollectionLimits::new(
                MAX_STATE,
                MAX_STRING,
                MAX_RETAINED,
            ))),
            projection,
        );
        if oversized {
            partial(&mut out, LINE_BUDGET);
        }
        if data.len() > MAX_INPUT {
            partial(&mut out, INPUT_BUDGET);
        }
        if invalid {
            partial(&mut out, SHAPE);
        }
        if invalid_usage {
            partial(&mut out, USAGE);
        }
        if usage_redaction {
            partial(&mut out, USAGE_REDACTION);
        }
        return out;
    }
    let mut parser = Parser::new(limits, projection);
    if let Some(scope) = usage_scope {
        parser.thread = scope.thread.to_owned();
        parser.usage_snapshot = true;
    }
    let mut usage_header_seen = false;
    let mut usage_scope_rejected = false;
    if data.len() > MAX_INPUT {
        partial(&mut parser.out, INPUT_BUDGET);
    }
    let mut offset = 0usize;
    for segment in bounded.split_inclusive(|byte| *byte == b'\n') {
        if usage_scope.is_some_and(|scope| Instant::now() >= scope.deadline) {
            partial(&mut parser.out, USAGE_DEADLINE);
            break;
        }
        let start = offset;
        offset += segment.len();
        let line = segment.strip_suffix(b"\n").unwrap_or(segment);
        let redaction = if matches!(projection, None | Some(OffsetProjection::Usage)) {
            match scan_usage_redaction(line, usage_scope.map(|scope| scope.deadline)) {
                Some(observed) => observed,
                None => {
                    partial(&mut parser.out, USAGE_DEADLINE);
                    break;
                }
            }
        } else {
            UsageRedactionScan::default()
        };
        if redaction.may_hide_records {
            partial(&mut parser.out, USAGE_REDACTION);
        }
        if line.len() > MAX_LINE {
            // Validate marker-bearing skipped rows without materializing their
            // bodies. A quote consumed by redaction must taint plain totals too.
            if redaction.has_marker && serde_json::from_slice::<IgnoredAny>(line).is_err() {
                partial(&mut parser.out, USAGE_REDACTION);
            }
            if usage_scope.is_some_and(|scope| Instant::now() >= scope.deadline) {
                partial(&mut parser.out, USAGE_DEADLINE);
                break;
            }
            if usage_scope.is_some() {
                partial(&mut parser.out, LINE_BUDGET);
            }
            if !parser.skip_large_record(line) {
                partial(&mut parser.out, LINE_BUDGET);
            }
            continue;
        }
        if line.iter().all(u8::is_ascii_whitespace) {
            continue;
        }
        parser.emit = record_reaches_offset(line, start, from);
        let Ok(canonical) = parse_canon_value(line) else {
            partial(&mut parser.out, SHAPE);
            if redaction.has_marker {
                partial(&mut parser.out, USAGE_REDACTION);
            }
            continue;
        };
        let Ok(entry) = serde_json::from_slice::<Value>(line) else {
            partial(&mut parser.out, SHAPE);
            if redaction.has_marker {
                partial(&mut parser.out, USAGE_REDACTION);
            }
            continue;
        };
        let Some(payload) = entry.get("payload").filter(|value| value.is_object()) else {
            partial(&mut parser.out, SHAPE);
            continue;
        };
        if let Some(scope) = usage_scope
            && !usage_header_seen
        {
            usage_header_seen = true;
            if text(&entry, "type") != Some("session_meta")
                || text(payload, "id") != Some(scope.thread)
            {
                usage_scope_rejected = true;
                partial(&mut parser.out, IDENTITY);
            }
        }
        if parser.foreign_partition
            && text(&entry, "type") != Some("session_meta")
            && text(payload, "thread_id").is_none()
        {
            partial(&mut parser.out, SHAPE);
            continue;
        }
        // Exact usage validates all three identity fields before applying
        // its thread filter. Keep other projections' existing early filter.
        let exact_usage_owns_identity = text(&entry, "type") == Some("token_usage_record")
            && matches!(projection, None | Some(OffsetProjection::Usage));
        if text(&entry, "type") != Some("session_meta")
            && !exact_usage_owns_identity
            && !parser.accepts_thread(payload)
        {
            continue;
        }
        match text(&entry, "type") {
            Some("session_meta") => {
                // Upstream recorder.rs::load_rollout_items binds the FIRST
                // SessionMeta as canonical. A copied parent header is not a new
                // session whose cumulative usage may be added to this file.
                let Some(thread) =
                    text(payload, "id").filter(|id| !id.is_empty() && id.len() <= MAX_STRING)
                else {
                    partial(&mut parser.out, SHAPE);
                    continue;
                };
                if !parser.thread.is_empty() && parser.thread != thread {
                    parser.foreign_partition = true;
                    partial(&mut parser.out, SHAPE);
                    continue;
                }
                if parser.thread.is_empty() {
                    parser.thread = thread.to_owned();
                }
                parser.foreign_partition = false;
                parser.context(payload);
            }
            Some("turn_context") => {
                parser.context(payload);
                parser.model(payload);
            }
            Some("event_msg") => parser.event(payload, text(&entry, "timestamp").unwrap_or("")),
            Some("token_usage_record") => parser.exact_usage(payload),
            Some("compacted") => {
                if text(payload, "window_id").is_some() {
                    parser.reset_allowed = true;
                }
                if let Some(record) = payload
                    .get("latest_token_usage_record")
                    .filter(|record| !record.is_null())
                {
                    parser.exact_usage(record);
                }
            }
            Some("response_item") => match text(payload, "type") {
                Some("message") if text(payload, "role") == Some("user") => {
                    if entry
                        .get("metadata")
                        .and_then(|meta| meta.get("inherited_user_message"))
                        .and_then(Value::as_bool)
                        != Some(true)
                    {
                        if let Some(meta) =
                            payload.get("internal_chat_message_metadata_passthrough")
                        {
                            parser.context(meta);
                        }
                        parser.prompt(
                            payload,
                            text(payload, "id"),
                            text(&entry, "timestamp").unwrap_or(""),
                            0,
                        );
                    }
                }
                Some("reasoning") => {
                    if let Some(classified) = canonical
                        .get("payload")
                        .and_then(super::codex_reasoning::classify_payload)
                        && classified.unsupported()
                    {
                        partial(&mut parser.out, SHAPE);
                    }
                }
                Some(
                    "message"
                    | "function_call"
                    | "function_call_output"
                    | "custom_tool_call"
                    | "custom_tool_call_output"
                    | "web_search_call"
                    | "compaction"
                    | "agent_message"
                    | "local_shell_call"
                    | "tool_search_call"
                    | "tool_search_output"
                    | "image_generation_call"
                    | "additional_tools"
                    | "configuration_update"
                    | "context_compaction",
                ) => (),
                _ => partial(&mut parser.out, SHAPE),
            },
            Some("world_state" | "security_risk_score" | "inter_agent_communication_metadata") => {}
            _ => partial(&mut parser.out, SHAPE),
        }
    }
    parser.finish_prompts();
    if from > 0 && parser.first_usage_exact == Some(false) && parser.legacy_prefix.is_some() {
        // A legacy-first prefix may already have reported a response whose
        // exact mirror arrives later. Keep cumulative deltas for subsequent
        // windows so that changing producer modes cannot count it again.
        for warning in &parser.legacy_warnings {
            partial(&mut parser.out, warning);
        }
        parser.out.usage = parser.legacy.map(Counters::summary);
        parser.out.api_call_count = parser.legacy_calls;
        if parser.legacy.is_none() && parser.exact.is_some() {
            partial(&mut parser.out, USAGE);
        }
    } else if !parser.response_ids.is_empty() {
        // Exact records remain authoritative even when all belong to the
        // prefix. A later cumulative mirror must not become new usage.
        parser.out.usage = parser.exact.map(Counters::summary);
        parser.out.api_call_count = parser.exact_calls;
        // Cumulative observations can mirror exact responses. Only an excess
        // beyond exact coverage demonstrates incomplete response history.
        let window_covered = parser.legacy.is_none_or(|legacy| {
            legacy.0 == [0; 5]
                || parser
                    .exact
                    .is_some_and(|exact| exact.checked(legacy, true).is_some())
        });
        let absolute_covered = parser.legacy_last.is_some_and(|legacy| {
            parser
                .exact_all
                .is_some_and(|exact| exact.checked(legacy, true).is_some())
        });
        if !window_covered && !absolute_covered {
            partial(&mut parser.out, USAGE);
            for warning in &parser.legacy_warnings {
                partial(&mut parser.out, warning);
            }
        }
    } else {
        for warning in &parser.legacy_warnings {
            partial(&mut parser.out, warning);
        }
        parser.out.usage = parser.legacy.map(Counters::summary);
        parser.out.api_call_count = parser.legacy_calls;
    }
    if usage_scope.is_some() {
        if !usage_header_seen || usage_scope_rejected || parser.response_ids.is_empty() {
            parser.out.usage = None;
            parser.out.api_call_count = 0;
            partial(&mut parser.out, USAGE);
        } else if parser.first_usage_exact == Some(false) {
            // Legacy-first history may include responses absent from the
            // exact subset even when the final cumulative mirror matches.
            partial(&mut parser.out, USAGE);
        }
        if usage_scope.is_some_and(|scope| Instant::now() >= scope.deadline) {
            partial(&mut parser.out, USAGE_DEADLINE);
        }
    }
    parser.out
}

#[cfg(test)]
#[cfg(feature = "test-live-agent")]
mod codex_child_usage_local_tests {
    use std::time::Duration;

    use serde_json::json;

    use super::*;

    const CHILD: &str = "123e4567-e89b-12d3-a456-426614174099";

    #[test]
    #[ignore = "operator-local Codex acceptance; never run in CI"]
    fn codex_child_usage_spawn_records_keep_complete_usage_and_bad_sender_is_partial() {
        let usage = json!({"input_tokens":3,"output_tokens":2,"cached_input_tokens":1,
            "cache_write_input_tokens":0,"reasoning_output_tokens":1,"total_tokens":5});
        let rows = [
            json!({"type":"session_meta","payload":{"id":CHILD}}),
            json!({"type":"token_usage_record","payload":{"thread_id":CHILD,"session_id":CHILD,"response_id":"spawn-parent-response","usage":usage}}),
            json!({"type":"event_msg","payload":{"type":"collab_agent_spawn_end","sender_thread_id":CHILD,"new_thread_id":"123e4567-e89b-12d3-a456-426614174098"}}),
        ];
        let encode = |rows: &[serde_json::Value]| {
            (rows
                .iter()
                .map(serde_json::Value::to_string)
                .collect::<Vec<_>>()
                .join("\n")
                + "\n")
                .into_bytes()
        };
        for projection in [None, Some(OffsetProjection::Usage)] {
            let out = extract_from_offset(&encode(&rows), None, 0, projection);
            assert!(!out.partial, "{:?}", out.warnings);
            assert_eq!(out.usage.unwrap().input_tokens, 3);
        }
        assert!(
            !extract_thread_usage_snapshot(
                &encode(&rows),
                CHILD,
                Instant::now() + Duration::from_secs(1)
            )
            .partial
        );
        let mut bad = rows.to_vec();
        bad[2]["payload"]["sender_thread_id"] = "foreign-thread".into();
        assert!(extract(&encode(&bad), None).partial);
        let mut failed = rows.to_vec();
        failed[2]["payload"]["new_thread_id"] = serde_json::Value::Null;
        assert!(!extract(&encode(&failed), None).partial);
    }

    const ROOT: &str = "123e4567-e89b-12d3-a456-426614174031";

    fn message(text: &str) -> Value {
        json!({"type":"response_item","payload":{"type":"message","role":"assistant",
            "content":[{"type":"output_text","text":text}]}})
    }

    fn ordinary_usage_readers(data: &[u8]) -> [ExtractionSummary; 2] {
        [
            extract(data, None),
            extract_from_offset(data, None, 0, Some(OffsetProjection::Usage)),
        ]
    }

    #[test]
    #[ignore = "operator-local Codex acceptance; never run in CI"]
    fn codex_child_usage_parent_multiline_redaction_keeps_known_subtotal_partial() {
        let mut hidden = exact(CHILD, 3, 2);
        hidden["payload"]["response_id"] = json!("hidden-response");
        let raw = source(
            CHILD,
            &[
                exact(CHILD, 3, 2),
                message("-----BEGIN PRIVATE KEY-----"),
                hidden,
                message("-----END PRIVATE KEY-----"),
            ],
        );
        for original in ordinary_usage_readers(&raw) {
            assert!(!original.partial, "{:?}", original.warnings);
            assert_eq!(original.usage.unwrap().input_tokens, 6);
        }
        let (redacted, report) = super::super::Redactor::new_default().redact(&raw);
        assert!(report.match_count() > 0);
        assert!(
            redacted.as_ref().iter().filter(|b| **b == b'\n').count()
                < raw.iter().filter(|b| **b == b'\n').count()
        );
        for observed in ordinary_usage_readers(redacted.as_ref()) {
            assert_eq!(observed.usage.as_ref().unwrap().input_tokens, 3);
            assert!(
                observed.partial,
                "redaction hid an exact response but summary claims completeness"
            );
        }
    }

    #[test]
    #[ignore = "operator-local Codex acceptance; never run in CI"]
    fn codex_child_usage_compacted_redaction_can_hide_usage_without_losing_newline() {
        let mut hidden = exact(CHILD, 3, 2);
        hidden["payload"]["response_id"] = json!("hidden-compact-response");
        let compacted = format!(
            "{{\"type\":\"compacted\",\"payload\":{{\"message\":\"-----BEGIN PRIVATE KEY-----\",\"latest_token_usage_record\":{},\"window_id\":\"-----END PRIVATE KEY-----\"}}}}\n",
            hidden["payload"],
        );
        let mut raw = source(CHILD, &[exact(CHILD, 3, 2)]);
        raw.extend_from_slice(compacted.as_bytes());
        for original in ordinary_usage_readers(&raw)
            .into_iter()
            .chain([snapshot(&raw)])
        {
            assert!(!original.partial, "{:?}", original.warnings);
            assert_eq!(original.usage.unwrap().input_tokens, 6);
        }
        let (redacted, report) = super::super::Redactor::new_default().redact(&raw);
        assert!(report.match_count() > 0);
        assert_eq!(
            redacted.as_ref().iter().filter(|b| **b == b'\n').count(),
            raw.iter().filter(|b| **b == b'\n').count()
        );
        for line in redacted
            .as_ref()
            .split(|b| *b == b'\n')
            .filter(|line| !line.is_empty())
        {
            serde_json::from_slice::<Value>(line)
                .expect("redacted fixture must remain valid JSONL");
        }
        for observed in ordinary_usage_readers(redacted.as_ref())
            .into_iter()
            .chain([snapshot(redacted.as_ref())])
        {
            assert_eq!(observed.usage.as_ref().unwrap().input_tokens, 3);
            assert!(
                observed.partial,
                "valid JSONL and retained newline count cannot prove retained compacted usage"
            );
        }
    }

    #[test]
    #[ignore = "operator-local Codex acceptance; never run in CI"]
    fn codex_child_usage_parent_inline_credentials_preserve_complete_known_usage() {
        let secret = format!("sk_live_{}", "A".repeat(30));
        let raw = source(CHILD, &[exact(CHILD, 3, 2), message(&secret)]);
        let (redacted, report) = super::super::Redactor::new_default().redact(&raw);
        assert!(report.match_count() > 0);
        assert!(!String::from_utf8_lossy(redacted.as_ref()).contains(&secret));
        assert!(String::from_utf8_lossy(redacted.as_ref()).contains("<REDACTED:stripe-key>"));
        for observed in ordinary_usage_readers(redacted.as_ref())
            .into_iter()
            .chain([snapshot(redacted.as_ref())])
        {
            assert!(!observed.partial, "{:?}", observed.warnings);
            assert_eq!(observed.usage.unwrap().input_tokens, 3);
        }
    }

    #[test]
    #[ignore = "operator-local Codex acceptance; never run in CI"]
    fn codex_child_usage_redaction_marker_scan_is_bounded_and_covers_chunk_boundaries() {
        for marker in [
            "<REDACTED:private-key-pem>",
            "<REDACTED:google-service-account-private-key>",
            "<REDACTED:credential-uri>",
        ] {
            for start in [0, 16 * 1024 - 1, 16 * 1024 - 15, 32 * 1024 - 1] {
                let mut data = vec![b'x'; start];
                data.extend_from_slice(marker.as_bytes());
                assert_eq!(
                    scan_usage_redaction(&data, Some(Instant::now() + Duration::from_secs(1))),
                    Some(UsageRedactionScan {
                        has_marker: true,
                        may_hide_records: true
                    })
                );
            }
        }
        assert_eq!(
            scan_usage_redaction(
                b"<REDACTED:stripe-key>",
                Some(Instant::now() + Duration::from_secs(1))
            ),
            Some(UsageRedactionScan {
                has_marker: true,
                may_hide_records: false
            })
        );
        assert_eq!(
            scan_usage_redaction(b"REDACTED:private-key-pem>", None),
            Some(UsageRedactionScan::default())
        );
        assert_eq!(
            scan_usage_redaction(
                b"<REDACTED:private-key-pem>suffix",
                Some(Instant::now() - Duration::from_millis(1))
            ),
            None
        );
    }

    #[test]
    #[ignore = "operator-local Codex acceptance; never run in CI"]
    fn codex_child_usage_possible_record_loss_does_not_change_other_projections() {
        let raw = source(
            CHILD,
            &[exact(CHILD, 3, 2), message("<REDACTED:private-key-pem>")],
        );
        for projection in [
            OffsetProjection::Files,
            OffsetProjection::Prompts,
            OffsetProjection::Skills,
        ] {
            let out = extract_from_offset(&raw, None, 0, Some(projection));
            assert!(
                !out.warnings
                    .iter()
                    .any(|warning| warning == USAGE_REDACTION)
            );
        }
        for out in ordinary_usage_readers(&raw) {
            assert!(out.partial);
            assert!(
                out.warnings
                    .iter()
                    .any(|warning| warning == USAGE_REDACTION)
            );
            assert_eq!(out.usage.unwrap().input_tokens, 3);
        }
    }

    #[test]
    #[ignore = "operator-local Codex acceptance; never run in CI"]
    fn codex_child_usage_google_private_key_redaction_is_explicit_for_legacy_usage() {
        let raw = (json!({"role":"assistant","usage":{"input_tokens":3,"output_tokens":2}})
            .to_string()
            + "\n"
            + &json!({"role":"assistant","private_key":"-----BEGIN PRIVATE KEY-----"}).to_string()
            + "\n"
            + &json!({"role":"assistant","usage":{"input_tokens":30,"output_tokens":20},
                "content":"-----END PRIVATE KEY-----"})
            .to_string()
            + "\n")
            .into_bytes();
        let (redacted, report) = super::super::Redactor::new_default().redact(&raw);
        assert!(
            report
                .matches
                .iter()
                .any(|hit| hit.rule_id == "google-service-account-private-key")
        );
        assert!(
            String::from_utf8_lossy(redacted.as_ref())
                .contains("<REDACTED:google-service-account-private-key>")
        );
        for out in ordinary_usage_readers(redacted.as_ref()) {
            assert!(out.partial);
            assert!(
                out.warnings
                    .iter()
                    .any(|warning| warning == USAGE_REDACTION)
            );
            assert_eq!(out.usage.unwrap().input_tokens, 3);
        }
    }

    #[test]
    #[ignore = "operator-local Codex acceptance; never run in CI"]
    fn codex_child_usage_uri_redaction_can_hide_nested_counts_without_losing_newline() {
        for (message, tail) in [("postgres://u:p@db", "w 1"), ("postgres://u", "x@db tail")] {
            let mut hidden = exact(CHILD, 3, 2);
            hidden["payload"]["response_id"] = json!("uri-hidden-response");
            let compact = format!(
                "{{\"type\":\"compacted\",\"payload\":{{\"message\":{},\"latest_token_usage_record\":{},\"window_id\":{}}}}}\n",
                json!(message),
                hidden["payload"],
                json!(tail),
            );
            let mut raw = source(CHILD, &[exact(CHILD, 3, 2)]);
            raw.extend_from_slice(compact.as_bytes());
            for before in ordinary_usage_readers(&raw)
                .into_iter()
                .chain([snapshot(&raw)])
            {
                assert!(!before.partial, "{:?}", before.warnings);
                assert_eq!(before.usage.unwrap().input_tokens, 6);
            }
            let (redacted, report) = super::super::Redactor::new_default().redact(&raw);
            assert!(
                report
                    .matches
                    .iter()
                    .any(|hit| hit.rule_id == "credential-uri")
            );
            assert_eq!(
                redacted.as_ref().iter().filter(|b| **b == b'\n').count(),
                raw.iter().filter(|b| **b == b'\n').count()
            );
            for line in redacted
                .as_ref()
                .split(|b| *b == b'\n')
                .filter(|l| !l.is_empty())
            {
                serde_json::from_slice::<Value>(line).unwrap();
            }
            for observed in ordinary_usage_readers(redacted.as_ref())
                .into_iter()
                .chain([snapshot(redacted.as_ref())])
            {
                assert_eq!(observed.usage.as_ref().unwrap().input_tokens, 3);
                assert!(
                    observed.partial,
                    "credential URI swallowed a nested usage field"
                );
            }
        }
    }

    #[test]
    #[ignore = "operator-local Codex acceptance; never run in CI"]
    fn codex_child_usage_flat_uri_redaction_keeps_known_subtotal_partial() {
        let raw=format!(
            "{}\n{{\"role\":\"assistant\",\"content\":\"postgres://u\",\"usage\":{{\"input_tokens\":30,\"output_tokens\":20}},\"z\":\"x@db tail\"}}\n",
            json!({"role":"assistant","usage":{"input_tokens":3,"output_tokens":2}}),
        ).into_bytes();
        for before in ordinary_usage_readers(&raw) {
            assert!(!before.partial, "{:?}", before.warnings);
            assert_eq!(before.usage.unwrap().input_tokens, 33);
        }
        let (redacted, report) = super::super::Redactor::new_default().redact(&raw);
        assert!(
            report
                .matches
                .iter()
                .any(|hit| hit.rule_id == "credential-uri")
        );
        for observed in ordinary_usage_readers(redacted.as_ref()) {
            assert_eq!(observed.usage.as_ref().unwrap().input_tokens, 3);
            assert!(observed.partial);
        }
    }

    #[test]
    #[ignore = "operator-local Codex acceptance; never run in CI"]
    fn codex_child_usage_loss_reaches_token_calculator_and_session_summary() {
        let mut hidden = exact(CHILD, 30, 20);
        hidden["payload"]["response_id"] = json!("hidden-classification-response");
        let raw = source(
            CHILD,
            &[
                json!({"type":"turn_context","payload":{"turn_id":"known-turn","model":"redacted-source-model"}}),
                exact(CHILD, 3, 2),
                message("-----BEGIN PRIVATE KEY-----"),
                hidden,
                message("-----END PRIVATE KEY-----"),
            ],
        );
        for original in ordinary_usage_readers(&raw) {
            assert!(!original.partial);
            assert_eq!(original.usage.unwrap().input_tokens, 33);
        }
        let (redacted, _) = super::super::Redactor::new_default().redact(&raw);
        let observed = extract(redacted.as_ref(), None);
        assert!(observed.partial);
        assert_eq!(observed.usage.as_ref().unwrap().input_tokens, 3);
        assert!(
            usage_is_inconsistent(&observed),
            "strong usage API cannot return a partial total as complete"
        );
        assert!(
            usage_is_unproven(&observed),
            "session summary cannot retain a partial total as proven"
        );
        let agent = super::super::agent_for(super::super::AgentKind::Codex);
        assert!(
            agent
                .as_token_calculator()
                .unwrap()
                .calculate_token_usage(redacted.as_ref(), 0)
                .is_err()
        );
        let facts = super::super::session_summary::from_redacted_codex_snapshot(
            &redacted,
            Instant::now() + Duration::from_secs(10),
        )
        .unwrap();
        assert_eq!(
            facts.extraction_metadata()["model"],
            "redacted-source-model"
        );
        assert!(facts.extraction_metadata()["token_usage"].is_null());
    }

    #[test]
    #[ignore = "operator-local Codex acceptance; never run in CI"]
    fn codex_child_usage_marker_in_oversized_message_is_seen_before_projection_skip() {
        let raw = source(
            CHILD,
            &[
                exact(CHILD, 3, 2),
                message(
                    &(String::from_utf8(vec![b'x'; MAX_LINE + 32]).unwrap()
                        + "<REDACTED:private-key-pem>"),
                ),
            ],
        );
        let observed = extract_from_offset(&raw, None, 0, Some(OffsetProjection::Usage));
        assert!(observed.partial);
        assert_eq!(observed.usage.as_ref().unwrap().input_tokens, 3);
        assert!(
            observed
                .warnings
                .iter()
                .any(|warning| warning == USAGE_REDACTION)
        );
    }

    #[test]
    #[ignore = "operator-local Codex acceptance; never run in CI"]
    fn codex_child_usage_invalid_redacted_records_do_not_become_plain_totals() {
        let secret = format!("DB_PASSWORD={}", "A".repeat(34));
        let mut hidden = exact(CHILD, 3, 2);
        hidden["payload"]["response_id"] = json!("env-hidden-response");
        let compact = format!(
            "{{\"type\":\"compacted\",\"payload\":{{\"message\":{},\"latest_token_usage_record\":{}}}}}\n",
            json!(secret),
            hidden["payload"]
        );
        let mut rollout = source(CHILD, &[exact(CHILD, 3, 2)]);
        rollout.extend_from_slice(compact.as_bytes());
        let flat=format!("{}\n{{\"role\":\"assistant\",\"content\":{},\"usage\":{{\"input_tokens\":3,\"output_tokens\":2}}}}\n",
            json!({"role":"assistant","usage":{"input_tokens":3,"output_tokens":2}}),json!(secret)).into_bytes();
        for raw in [rollout, flat] {
            for before in ordinary_usage_readers(&raw) {
                assert!(!before.partial, "{:?}", before.warnings);
                assert_eq!(before.usage.unwrap().input_tokens, 6);
            }
            let (redacted, report) = super::super::Redactor::new_default().redact(&raw);
            assert!(
                report
                    .matches
                    .iter()
                    .any(|hit| hit.rule_id == "env-password-assignment")
            );
            assert!(
                redacted
                    .as_ref()
                    .split(|b| *b == b'\n')
                    .filter(|line| !line.is_empty())
                    .any(|line| serde_json::from_slice::<Value>(line).is_err())
            );
            for observed in ordinary_usage_readers(redacted.as_ref()) {
                assert!(observed.partial);
                assert_eq!(observed.usage.as_ref().unwrap().input_tokens, 3);
                assert!(
                    usage_is_inconsistent(&observed),
                    "redaction broke a usage-bearing row"
                );
                assert!(usage_is_unproven(&observed));
            }
            let agent = super::super::agent_for(super::super::AgentKind::Codex);
            assert!(
                agent
                    .as_token_calculator()
                    .unwrap()
                    .calculate_token_usage(redacted.as_ref(), 0)
                    .is_err()
            );
        }
    }

    #[test]
    #[ignore = "operator-local Codex acceptance; never run in CI"]
    fn codex_child_usage_valid_inline_password_marker_keeps_complete_observation() {
        let (masked, report) = super::super::Redactor::new_default()
            .redact(format!("DB_PASSWORD={}", "A".repeat(34)).as_bytes());
        assert!(
            report
                .matches
                .iter()
                .any(|hit| hit.rule_id == "env-password-assignment")
        );
        let raw = source(
            CHILD,
            &[
                exact(CHILD, 3, 2),
                message(std::str::from_utf8(masked.as_ref()).unwrap()),
            ],
        );
        for observed in ordinary_usage_readers(&raw)
            .into_iter()
            .chain([snapshot(&raw)])
        {
            assert!(!observed.partial, "{:?}", observed.warnings);
            assert_eq!(observed.usage.as_ref().unwrap().input_tokens, 3);
            assert!(!usage_is_inconsistent(&observed));
        }
    }

    #[test]
    #[ignore = "operator-local Codex acceptance; never run in CI"]
    fn codex_child_usage_oversized_redacted_syntax_loss_is_not_a_proven_subtotal() {
        let secret = String::from_utf8(vec![b'x'; MAX_LINE + 32]).unwrap()
            + " DB_PASSWORD="
            + &"A".repeat(34);
        let raw = source(CHILD, &[exact(CHILD, 3, 2), message(&secret)]);
        let (broken, report) = super::super::Redactor::new_default().redact(&raw);
        assert!(
            report
                .matches
                .iter()
                .any(|hit| hit.rule_id == "env-password-assignment")
        );
        let out = extract_from_offset(broken.as_ref(), None, 0, Some(OffsetProjection::Usage));
        assert!(out.partial);
        assert_eq!(out.usage.as_ref().unwrap().input_tokens, 3);
        assert!(
            out.warnings
                .iter()
                .any(|warning| warning == USAGE_REDACTION)
        );
        assert!(usage_is_unproven(&out));
        let valid = source(
            CHILD,
            &[
                exact(CHILD, 3, 2),
                message(
                    &(String::from_utf8(vec![b'x'; MAX_LINE + 32]).unwrap()
                        + "<REDACTED:env-password-assignment>"),
                ),
            ],
        );
        let ordinary = extract_from_offset(&valid, None, 0, Some(OffsetProjection::Usage));
        assert_eq!(ordinary.usage.as_ref().unwrap().input_tokens, 3);
        assert!(
            !ordinary
                .warnings
                .iter()
                .any(|warning| warning == USAGE_REDACTION),
            "valid large inline replacement must retain the existing projection policy"
        );
    }

    #[test]
    #[ignore = "operator-local Codex acceptance; never run in CI"]
    fn codex_child_usage_redaction_growth_keeps_capability_rejection_and_partial_facts() {
        let mut hidden = exact(CHILD, 3, 2);
        hidden["payload"]["response_id"] = json!("growth-hidden-response");
        let keys = " ".to_string() + &vec![format!("AKIA{}", "A".repeat(16)); 512].join(" ");
        let mut row = json!({"type":"compacted","payload":{"message":keys,
            "latest_token_usage_record":hidden["payload"]}});
        let padding = MAX_LINE - 64 - row.to_string().len();
        row["payload"]["message"] = json!(
            String::from_utf8(vec![b'x'; padding]).unwrap()
                + row["payload"]["message"].as_str().unwrap()
        );
        assert_eq!(row.to_string().len(), MAX_LINE - 64);
        let raw = source(CHILD, &[exact(CHILD, 3, 2), row]);
        for before in ordinary_usage_readers(&raw) {
            assert!(!before.partial, "{:?}", before.warnings);
            assert_eq!(before.usage.unwrap().input_tokens, 6);
        }
        let (redacted, report) = super::super::Redactor::new_default().redact(&raw);
        assert!(report.match_count() >= 512);
        assert!(
            redacted
                .as_ref()
                .split(|b| *b == b'\n')
                .any(|line| line.len() > MAX_LINE)
        );
        for observed in ordinary_usage_readers(redacted.as_ref()) {
            assert!(observed.partial);
            assert_eq!(observed.usage.as_ref().unwrap().input_tokens, 3);
            assert!(
                observed
                    .warnings
                    .iter()
                    .any(|warning| warning == LINE_BUDGET)
            );
        }
        // The actual capability wrapper rejects the parser's bound before
        // the total-only helper runs; no error-policy change is needed.
        let agent = super::super::agent_for(super::super::AgentKind::Codex);
        let error = agent
            .as_token_calculator()
            .unwrap()
            .calculate_token_usage(redacted.as_ref(), 0)
            .unwrap_err();
        assert!(
            error.to_string().contains("1 MiB JSONL record limit"),
            "{error}"
        );
        let facts = super::super::session_summary::from_redacted_codex_snapshot(
            &redacted,
            Instant::now() + Duration::from_secs(10),
        )
        .unwrap();
        let metadata = facts.extraction_metadata();
        assert_eq!(metadata["partial"], true);
        assert_eq!(metadata["token_usage"]["input_tokens"], 3);
        assert!(
            metadata.get("aggregate_token_usage").is_none(),
            "partial facts do not advertise a complete aggregate"
        );
    }

    fn counters(input: u64, output: u64) -> Value {
        json!({"input_tokens":input,"output_tokens":output,"cached_input_tokens":1,
            "cache_write_input_tokens":2,"reasoning_output_tokens":1,"total_tokens":input+output})
    }

    fn exact(thread: &str, input: u64, output: u64) -> Value {
        let usage = counters(input, output);
        json!({"type":"token_usage_record","payload":{
            "thread_id":thread,"session_id":ROOT,"turn_id":"child-turn","root_turn_id":"root-turn",
            "response_id":"same-response","usage":usage,"turn_token_usage":usage,"thread_token_usage":usage}})
    }

    fn legacy(input: u64, output: u64) -> Value {
        json!({"type":"event_msg","payload":{"type":"token_count",
            "info":{"total_token_usage":counters(input,output),"last_token_usage":counters(input,output)}}})
    }

    fn source(header: &str, records: &[Value]) -> Vec<u8> {
        let mut rows = vec![json!({"type":"session_meta","payload":{"id":header}})];
        rows.extend_from_slice(records);
        (rows
            .iter()
            .map(Value::to_string)
            .collect::<Vec<_>>()
            .join("\n")
            + "\n")
            .into_bytes()
    }

    fn snapshot(data: &[u8]) -> ExtractionSummary {
        extract_thread_usage_snapshot(data, CHILD, Instant::now() + Duration::from_secs(10))
    }

    #[test]
    #[ignore = "operator-local Codex acceptance; never run in CI"]
    fn codex_child_usage_exact_mirrors_count_response_once_and_keep_cache_writes() {
        let record = exact(CHILD, 3, 2);
        let data = source(CHILD, &[record.clone(), record, legacy(3, 2)]);
        let out = snapshot(&data);
        assert!(!out.partial, "{:?}", out.warnings);
        let usage = out.usage.unwrap();
        assert_eq!((usage.input_tokens, usage.output_tokens), (3, 2));
        assert_eq!(usage.cached_tokens, Some(3));
        assert_eq!(usage.reasoning_tokens, Some(1));
        assert_eq!(out.api_call_count, 1);
    }

    #[test]
    #[ignore = "operator-local Codex acceptance; never run in CI"]
    fn codex_child_usage_legacy_first_and_excess_are_partial_not_complete_exact_history() {
        for records in [
            vec![legacy(3, 2), exact(CHILD, 3, 2)],
            vec![exact(CHILD, 3, 2), legacy(30, 20)],
        ] {
            let data = source(CHILD, &records);
            let out = snapshot(&data);
            assert!(out.partial, "{:?}", out.warnings);
            assert_eq!(out.usage.unwrap().input_tokens, 3);
        }
        // New strict metadata must not change the original complete-source
        // reader's established exact-over-mirror behaviour.
        let data = source(CHILD, &[legacy(3, 2), exact(CHILD, 3, 2)]);
        let original = extract(&data, None);
        assert_eq!(original.usage.unwrap().input_tokens, 3);
    }

    #[test]
    #[ignore = "operator-local Codex acceptance; never run in CI"]
    fn codex_child_usage_legacy_only_is_unknown_and_wrong_first_thread_cannot_rebind() {
        for data in [
            source(CHILD, &[legacy(3, 2)]),
            source(ROOT, &[exact(CHILD, 3, 2)]),
        ] {
            let out = snapshot(&data);
            assert!(out.partial, "{:?}", out.warnings);
            assert!(out.usage.is_none());
            assert_eq!(out.api_call_count, 0);
        }
    }

    #[test]
    #[ignore = "operator-local Codex acceptance; never run in CI"]
    fn codex_child_usage_foreign_same_response_is_not_added_to_the_child() {
        let data = source(CHILD, &[exact(ROOT, 30, 20), exact(CHILD, 3, 2)]);
        let out = snapshot(&data);
        assert!(out.partial, "foreign history must remain explicit");
        assert_eq!(out.usage.unwrap().input_tokens, 3);
        assert_eq!(out.api_call_count, 1);
    }

    #[test]
    #[ignore = "operator-local Codex acceptance; never run in CI"]
    fn codex_child_usage_conflicting_response_counts_remain_partial() {
        let data = source(CHILD, &[exact(CHILD, 3, 2), exact(CHILD, 4, 2)]);
        let out = snapshot(&data);
        assert!(out.partial);
        assert!(out.warnings.iter().any(|warning| warning == USAGE));
        assert_eq!(out.usage.unwrap().input_tokens, 3);
        assert_eq!(out.api_call_count, 1);
    }

    #[test]
    #[ignore = "operator-local Codex acceptance; never run in CI"]
    fn codex_child_usage_redacted_response_identity_cannot_be_a_complete_mirror() {
        let mut first = exact(CHILD, 3, 2);
        let mut second = exact(CHILD, 3, 2);
        first["payload"]["response_id"] = json!(format!("sk-{}", "A".repeat(24)));
        second["payload"]["response_id"] = json!(format!("sk-{}", "B".repeat(24)));
        let raw = source(CHILD, &[first, second]);
        let original = snapshot(&raw);
        assert!(!original.partial, "{:?}", original.warnings);
        assert_eq!(original.usage.unwrap().input_tokens, 6);
        let (redacted, report) = super::super::Redactor::new_default().redact(&raw);
        assert!(
            report.match_count() >= 2,
            "must exercise actual default redaction"
        );
        assert!(String::from_utf8_lossy(redacted.as_ref()).contains("<REDACTED:"));
        let out = snapshot(redacted.as_ref());
        assert!(
            out.partial,
            "redaction must not merge two responses into complete usage"
        );
        assert!(out.usage.is_none());
        let ordinary_parent = extract(redacted.as_ref(), None);
        assert!(
            ordinary_parent.partial,
            "parent summaries must not trust a lossy dedup key"
        );
        assert!(
            ordinary_parent
                .warnings
                .iter()
                .any(|warning| warning == USAGE)
        );
    }

    #[test]
    #[ignore = "operator-local Codex acceptance; never run in CI"]
    fn codex_child_usage_malformed_exact_identity_is_partial_not_an_unscoped_counter() {
        for field in ["thread_id", "session_id", "response_id"] {
            for value in [Value::Null, json!(42), json!("")] {
                let mut invalid = exact(CHILD, 3, 2);
                invalid["payload"][field] = value;
                let out = snapshot(&source(CHILD, &[invalid]));
                assert!(out.partial, "{field}: {:?}", out.warnings);
                assert!(out.usage.is_none(), "{field}");
                assert!(out.warnings.iter().any(|warning| warning == IDENTITY));
            }
        }
    }

    #[test]
    #[ignore = "operator-local Codex acceptance; never run in CI"]
    fn codex_child_usage_expired_budget_does_not_parse_or_report_zero() {
        let out = extract_thread_usage_snapshot(
            &source(CHILD, &[exact(CHILD, 3, 2)]),
            CHILD,
            Instant::now() - Duration::from_millis(1),
        );
        assert!(out.partial);
        assert!(out.usage.is_none());
        assert!(out.warnings.iter().any(|warning| warning == USAGE_DEADLINE));
    }

    #[test]
    #[ignore = "operator-local Codex acceptance; never run in CI"]
    fn codex_child_usage_internal_input_and_line_caps_cannot_claim_completeness() {
        let mut input = source(CHILD, &[]);
        input.resize(MAX_INPUT + 1, b' ');
        let out = snapshot(&input);
        assert!(out.partial);
        assert!(out.usage.is_none());
        assert!(out.warnings.iter().any(|warning| warning == INPUT_BUDGET));

        let mut lines = source(CHILD, &[]);
        lines.extend(vec![b' '; MAX_LINE + 1]);
        lines.push(b'\n');
        lines.extend(exact(CHILD, 3, 2).to_string().bytes());
        lines.push(b'\n');
        let out = snapshot(&lines);
        assert!(out.partial);
        assert!(out.warnings.iter().any(|warning| warning == LINE_BUDGET));
        assert_eq!(out.usage.unwrap().input_tokens, 3);
    }
}
