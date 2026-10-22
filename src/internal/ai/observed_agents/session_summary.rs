//! Bounded ordinary metadata from one redacted Codex source snapshot.
//! These values confer no source/native authority and never change turn digests.

use std::time::Instant;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{RedactedBytes, Redactor, codex_rollout, extract::ExtractionCollectionLimits};

const TEXT_CAP: usize = 4096;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SummaryTokenUsage {
    pub(crate) input_tokens: u64,
    pub(crate) output_tokens: u64,
    pub(crate) cached_tokens: Option<u64>,
    pub(crate) reasoning_tokens: Option<u64>,
    pub(crate) total_tokens: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SessionSummaryFacts {
    pub(crate) schema_version: u8,
    pub(crate) partial: bool,
    pub(crate) model: Option<String>,
    pub(crate) title: Option<String>,
    pub(crate) prompt_count: Option<u64>,
    pub(crate) token_usage: Option<SummaryTokenUsage>,
    pub(crate) modified_file_count: Option<u64>,
}

fn text(value: Option<&Value>, partial: &mut bool, redactor: &Redactor) -> Option<String> {
    let value = match value {
        None | Some(Value::Null) => return None,
        Some(Value::String(value)) if value.len() <= TEXT_CAP => value,
        _ => {
            *partial = true;
            return None;
        }
    };
    if value.is_empty() || value == "unknown" {
        return None;
    }
    // Normalize display controls before scanning so stripping cannot join
    // previously split token fragments into a newly exposed credential.
    let display: String = value.chars()
        .filter(|character| {
            !character.is_control()
                && !matches!(*character, '\u{ad}' | '\u{61c}' | '\u{180e}' | '\u{200b}'..='\u{200f}' | '\u{2028}'..='\u{202e}' | '\u{2060}'..='\u{206f}' | '\u{feff}' | '\u{fff9}'..='\u{fffb}' | '\u{e0000}'..='\u{e007f}')
        }).collect();
    let (redacted, _) = redactor.redact(display.as_bytes());
    let clean = String::from_utf8_lossy(redacted.bytes()).into_owned();
    *partial |= clean != *value;
    if clean.len() > TEXT_CAP {
        *partial = true;
        return None;
    }
    (!clean.is_empty()).then_some(clean)
}

fn number(value: Option<&Value>, partial: &mut bool) -> Option<u64> {
    match value {
        None | Some(Value::Null) => None,
        Some(value) => match value.as_u64() {
            Some(number) => Some(number),
            None => {
                *partial = true;
                None
            }
        },
    }
}

impl SessionSummaryFacts {
    /// The existing live metadata extraction slot; no raw-transcript fallback.
    pub(crate) fn from_extraction(value: &Value) -> Option<Self> {
        if value.get("schema_version").and_then(Value::as_u64) != Some(1)
            || value.get("present").and_then(Value::as_bool) != Some(true)
        {
            return None;
        }
        let mut partial = value.get("partial").and_then(Value::as_bool)?;
        let redactor = Redactor::new_default();
        let model = text(value.get("model"), &mut partial, &redactor);
        let title = text(value.get("title"), &mut partial, &redactor);
        let prompt_count = number(value.get("prompt_count"), &mut partial);
        let modified_file_count = match value.get("modified_file_count") {
            Some(value) => number(Some(value), &mut partial),
            None => match value.get("modified_files") {
                Some(Value::Array(files))
                    if files.len() <= 4096 && files.iter().all(Value::is_string) =>
                {
                    Some(files.len() as u64)
                }
                None | Some(Value::Null) => None,
                _ => {
                    partial = true;
                    None
                }
            },
        };
        let token_usage = match value.get("token_usage") {
            None | Some(Value::Null) => None,
            Some(Value::Object(usage)) => {
                let input = number(usage.get("input_tokens"), &mut partial);
                let output = number(usage.get("output_tokens"), &mut partial);
                let cached = number(usage.get("cached_tokens"), &mut partial);
                let reasoning = number(usage.get("reasoning_tokens"), &mut partial);
                let total = number(usage.get("total_tokens"), &mut partial);
                match (input, output) {
                    (Some(input_tokens), Some(output_tokens))
                        if !total.is_some_and(|n| {
                            input_tokens.checked_add(output_tokens) != Some(n)
                        }) =>
                    {
                        Some(SummaryTokenUsage {
                            input_tokens,
                            output_tokens,
                            cached_tokens: cached,
                            reasoning_tokens: reasoning,
                            total_tokens: total,
                        })
                    }
                    _ => {
                        partial = true;
                        None
                    }
                }
            }
            Some(_) => {
                partial = true;
                None
            }
        };
        if model.is_none()
            && title.is_none()
            && prompt_count.is_none()
            && token_usage.is_none()
            && modified_file_count.is_none()
        {
            return None;
        }
        Some(Self {
            schema_version: 1,
            partial,
            model,
            title,
            prompt_count,
            token_usage,
            modified_file_count,
        })
    }

    pub(crate) fn extraction_metadata(&self) -> Value {
        serde_json::json!({"schema_version":self.schema_version,"present":true,
            "partial":self.partial,"model":self.model,"title":self.title,
            "prompt_count":self.prompt_count,"token_usage":self.token_usage,
            "modified_file_count":self.modified_file_count})
    }

    /// Revalidate a data-only helper response without granting it any authority.
    /// Invalid optional facts are omitted while the ordinary transcript survives.
    pub(crate) fn checked(self) -> Option<Self> {
        if self.schema_version != 1
            || self
                .model
                .as_ref()
                .is_some_and(|value| value.len() > TEXT_CAP)
            || self
                .title
                .as_ref()
                .is_some_and(|value| value.len() > TEXT_CAP)
        {
            return None;
        }
        Self::from_extraction(&self.extraction_metadata())
    }
}

/// One bounded pure parse over the helper's existing redacted snapshot. It
/// neither opens provider files nor changes normalization/coverage identity.
pub(crate) fn from_redacted_codex_snapshot(
    source: &RedactedBytes,
    deadline: Instant,
) -> Option<SessionSummaryFacts> {
    if Instant::now() >= deadline {
        return None;
    }
    let parsed = codex_rollout::extract(
        source.bytes(),
        Some(ExtractionCollectionLimits::new(
            4096,
            64 * 1024,
            1024 * 1024,
        )),
    );
    if Instant::now() >= deadline {
        return None;
    }
    let usage = if codex_rollout::usage_is_unproven(&parsed) {
        None
    } else {
        parsed.usage.as_ref()
    };
    SessionSummaryFacts::from_extraction(&serde_json::json!({
        "schema_version":1,"present":true,"partial":parsed.partial,
        "model":parsed.model,"prompt_count":if parsed.partial && parsed.prompts.is_empty() { None } else { Some(parsed.prompts.len()) },
        "modified_file_count":if parsed.partial && parsed.modified_files.is_empty() { None } else { Some(parsed.modified_files.len()) },
        "token_usage":usage,
    }))
}

#[cfg(test)]
#[cfg(feature = "test-live-agent")]
mod local_tests {
    use super::*;

    #[test]
    #[ignore = "operator-local Codex acceptance; never run in CI"]
    fn codex_session_summary_controls_cannot_reassemble_displayed_credentials() {
        for split in ["\u{200b}", "\u{202e}", "\u{e0041}", "\u{fff9}"] {
            let raw = format!("AKIAIOSF{split}ODNN7EXAMPLE");
            let facts = SessionSummaryFacts::from_extraction(&serde_json::json!({
                "schema_version":1,"present":true,"partial":false,"model":raw,
            }))
            .unwrap();
            assert!(facts.partial);
            let serialized = serde_json::to_string(&facts).unwrap();
            assert!(!serialized.contains("ODNN7EXAMPLE"), "{serialized}");
            assert!(!serialized.contains("AKIAIOSFODNN7EXAMPLE"), "{serialized}");
        }
    }

    #[test]
    #[ignore = "operator-local Codex acceptance; never run in CI"]
    fn codex_session_summary_display_controls_are_stripped() {
        let value = serde_json::json!({"schema_version":1,"present":true,"partial":false,
            "model":"safe\u{202e}model", "title":"a\u{2066}b\u{200b}c\u{2028}d\n"});
        let facts = SessionSummaryFacts::from_extraction(&value).unwrap();
        assert!(facts.partial);
        assert_eq!(facts.model.as_deref(), Some("safemodel"));
        assert_eq!(facts.title.as_deref(), Some("abcd"));
    }

    #[test]
    #[ignore = "operator-local Codex acceptance; never run in CI"]
    fn codex_session_summary_decoder_never_invents_missing_or_invalid_usage() {
        for usage in [
            Value::Null,
            serde_json::json!({"input_tokens":-1,"output_tokens":2}),
            serde_json::json!({"input_tokens":1.5,"output_tokens":2}),
            serde_json::json!({"input_tokens":u64::MAX,"output_tokens":2,"total_tokens":1}),
        ] {
            let value = serde_json::json!({"schema_version":1,"present":true,"partial":false,"prompt_count":1,"token_usage":usage});
            let parsed = SessionSummaryFacts::from_extraction(&value).unwrap();
            assert!(parsed.token_usage.is_none(), "{value}");
        }
        let measured = serde_json::json!({"schema_version":1,"present":true,"partial":false,
            "token_usage":{"input_tokens":0,"output_tokens":0,"total_tokens":0}});
        assert_eq!(
            SessionSummaryFacts::from_extraction(&measured)
                .unwrap()
                .token_usage
                .unwrap()
                .total_tokens,
            Some(0)
        );
        // The Codex projection adds cache-read and cache-write counters. Keep
        // independently reported cache/reasoning values; its existing parser
        // imposes no subset constraint on those fields.
        let independent = serde_json::json!({"schema_version":1,"present":true,"partial":false,
            "token_usage":{"input_tokens":1,"output_tokens":2,"cached_tokens":3,"reasoning_tokens":3,"total_tokens":3}});
        let usage = SessionSummaryFacts::from_extraction(&independent)
            .unwrap()
            .token_usage
            .unwrap();
        assert_eq!(usage.cached_tokens, Some(3));
        assert_eq!(usage.reasoning_tokens, Some(3));
    }

    #[test]
    #[ignore = "operator-local Codex acceptance; never run in CI"]
    fn codex_session_summary_decoder_bounds_text_and_preserves_legacy_unknowns() {
        for value in [
            Value::Null,
            serde_json::json!({}),
            serde_json::json!({"schema_version":1,"present":true,"partial":false}),
            serde_json::json!({"schema_version":2,"present":true,"partial":false}),
            serde_json::json!({"schema_version":1,"present":false,"partial":true}),
        ] {
            assert!(SessionSummaryFacts::from_extraction(&value).is_none());
        }
        let value = serde_json::json!({"schema_version":1,"present":true,"partial":false,
            "model":"x".repeat(TEXT_CAP+1),"title":"\u{1b}[2Jhello\nworld",
            "prompt_count":-1,"modified_file_count":0});
        let parsed = SessionSummaryFacts::from_extraction(&value).unwrap();
        assert!(parsed.partial);
        assert!(parsed.model.is_none());
        assert!(parsed.prompt_count.is_none());
        assert_eq!(parsed.modified_file_count, Some(0));
        assert_eq!(parsed.title.as_deref(), Some("[2Jhelloworld"));
        let expanded = SessionSummaryFacts::from_extraction(&serde_json::json!({
            "schema_version":1,"present":true,"partial":false,
            "model":"AKIAIOSFODNN7EXAMPLE ".repeat(190),
        }));
        assert!(
            expanded.is_none(),
            "a masked field over the cap leaves no facts"
        );
    }

    #[test]
    #[ignore = "operator-local Codex acceptance; never run in CI"]
    fn codex_session_summary_expired_optional_snapshot_never_creates_zero_facts() {
        let source = super::super::Redactor::new_default().redact(b"{}").0;
        assert!(from_redacted_codex_snapshot(&source, Instant::now()).is_none());
    }

    #[test]
    #[ignore = "operator-local Codex acceptance; never run in CI"]
    fn codex_session_summary_bad_optional_helper_metadata_preserves_ordinary_body() {
        let source = concat!(
            "{\"type\":\"session_meta\",\"payload\":{\"id\":\"thread-a\"}}\n",
            "{\"type\":\"response_item\",\"payload\":{\"type\":\"message\",\"role\":\"user\",\"content\":[{\"type\":\"input_text\",\"text\":\"question\"}]}}\n",
            "{\"type\":\"response_item\",\"payload\":{\"type\":\"reasoning\",\"summary\":[{\"type\":\"summary_text\",\"text\":\"CX07-RETAIN-OPTIONAL-FAIL\"}]}}\n",
            "{\"type\":\"response_item\",\"payload\":{\"type\":\"message\",\"role\":\"assistant\",\"content\":[{\"type\":\"output_text\",\"text\":\"answer\"}]}}\n",
        );
        let turns = super::super::normalize_codex_rollout(source.as_bytes());
        assert_eq!(turns.len(), 1);
        let key = &turns[0].logical_turn_key;
        for summary in [
            serde_json::json!({"schema_version":"invalid"}),
            serde_json::json!({"schema_version":2,"partial":false,"model":null,"title":null,"prompt_count":1,"token_usage":null,"modified_file_count":0}),
            serde_json::json!({"schema_version":1,"partial":false,"model":null,"title":null,"prompt_count":-1,"token_usage":null,"modified_file_count":0}),
        ] {
            let dto: super::super::codex_reasoning::CodexRetentionDto = serde_json::from_value(
                serde_json::json!({"version":1,"turns":{key:source},"summary":summary}),
            )
            .unwrap();
            let retained = dto
                .decode(
                    &turns,
                    source.len(),
                    Instant::now() + std::time::Duration::from_secs(1),
                )
                .unwrap();
            assert_eq!(retained.for_turn(key).unwrap().bytes(), source.as_bytes());
            assert!(!retained.is_unavailable());
            assert!(retained.summary_metadata().is_none());
        }
    }

    #[test]
    #[ignore = "operator-local Codex acceptance; never run in CI"]
    fn codex_session_summary_sensitive_metadata_is_redacted_before_display() {
        let secret = "sk-ant-EXAMPLE0000000000000000";
        let facts = SessionSummaryFacts::from_extraction(&serde_json::json!({
            "schema_version":1,"present":true,"partial":false,"model":secret,
        }))
        .unwrap();
        let serialized = serde_json::to_string(&facts).unwrap();
        assert!(!serialized.contains("EXAMPLE0000000000000000"));
        assert!(serialized.contains("REDACTED"));
        assert!(facts.partial);
    }
}
