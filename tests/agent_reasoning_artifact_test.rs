//! CC-05 Claude and CX-06 local Codex ordinary attached-field retention.
//!
//! The JSONL in this file is synthetic. It is not a Claude Code 2.1.303 capture
//! and it is not evidence that a live session wrote `thinking` or `signature`.

#[cfg(feature = "test-live-agent")]
use libra::internal::ai::observed_agents::RedactedBytes;
use libra::internal::ai::{
    capture::snapshot::{CaptureSnapshotPolicy, CaptureSnapshotService},
    observed_agents::claude_jsonl_session_matches,
};

#[cfg(feature = "test-live-agent")]
fn codex_ordinary(source: &[u8]) -> RedactedBytes {
    CaptureSnapshotService::preview_redacted_coverage_v2(
        "codex",
        source.to_vec(),
        CaptureSnapshotPolicy::default(),
    )
    .expect("Codex ordinary preview")
    .0
}

fn synthetic_source() -> Vec<u8> {
    b"{\"type\":\"user\",\"uuid\":\"cc05-user\",\"timestamp\":\"2026-10-19T00:00:00Z\",\"message\":{\"role\":\"user\",\"content\":\"draw a snake\"}}\n\
{\"type\":\"assistant\",\"uuid\":\"cc05-turn\",\"timestamp\":\"2026-10-19T00:00:01Z\",\"message\":{\"role\":\"assistant\",\"model\":\"claude-sonnet-5\",\"content\":[{\"type\":\"thinking\",\"thinking\":\"CC05-THINK\",\"signature\":\"CC05-SIG\"},{\"type\":\"redacted_thinking\",\"data\":\"CC05-DATA\"}],\"usage\":{\"input_tokens\":1}}}\n"
        .to_vec()
}

/// Ordinary signature and redacted data keep a content index.
/// A duplicate key and a wrong session add no fields and do not erase a
/// projection that was already retained.
#[test]
fn claude_source_attached_fields_keep_content_index_without_native_artifact() {
    let session_id = "cc05-session";
    let file_name = format!("/isolated/projects/work/{session_id}.jsonl");
    let source = synthetic_source();

    let (ordinary, preview) = CaptureSnapshotService::preview_redacted_coverage_v2(
        "claude_code",
        source.clone(),
        CaptureSnapshotPolicy::default(),
    )
    .expect("synthetic preview");
    let ordinary_text = String::from_utf8(ordinary.bytes().to_vec()).expect("ordinary utf-8");
    assert!(ordinary_text.contains("\"uuid\""));
    assert!(ordinary_text.contains("CC05-SIG"));
    assert!(ordinary_text.contains("CC05-DATA"));
    assert!(ordinary_text.contains("CC05-THINK"));
    assert!(ordinary_text.contains("claude-sonnet-5"));
    assert!(!preview["redaction_report"].to_string().contains("CC05-SIG"));
    assert!(!preview.to_string().contains("native_artifact"));

    let retained = CaptureSnapshotService::authorize_claude_attached_fields(
        &file_name,
        session_id,
        ordinary.bytes(),
    );
    assert!(!retained.refused);
    assert!(retained.readable_obtained());
    assert_eq!(retained.readable, vec!["CC05-THINK".to_owned()]);
    assert_eq!(retained.fields.len(), 2);
    assert_eq!(retained.fields[0].provider, "claude_code");
    assert_eq!(retained.fields[0].session_id, session_id);
    assert_eq!(retained.fields[0].turn_key, "cc05-turn");
    assert_eq!(retained.fields[0].record_ordinal, 0);
    assert_eq!(retained.fields[0].content_index, 0);
    assert_eq!(retained.fields[0].field, "signature");
    assert_eq!(retained.fields[0].value, "CC05-SIG");
    assert_eq!(retained.fields[0].availability, "unproven");
    assert_eq!(retained.fields[1].record_ordinal, 1);
    assert_eq!(retained.fields[1].content_index, 1);
    assert_eq!(retained.fields[1].field, "data");
    assert_eq!(retained.fields[1].value, "CC05-DATA");
    assert_eq!(retained.fields[1].availability, "unproven");

    let replay = CaptureSnapshotService::authorize_claude_attached_fields(
        &file_name,
        session_id,
        ordinary.bytes(),
    );
    assert_eq!(replay, retained);

    let duplicate = b"{\"type\":\"assistant\",\"message\":{\"role\":\"assistant\",\"content\":[{\"type\":\"thinking\",\"thinking\":\"CC05-THINK\",\"thinking\":\"other\",\"signature\":\"CC05-SIG\"}]}}\n";
    let rejected = CaptureSnapshotService::claude_attached_projection(duplicate);
    assert!(rejected.duplicate_rejected >= 1);
    assert!(rejected.fields.is_empty());
    assert!(!rejected.data_obtained());
    assert_eq!(retained.fields[0].value, "CC05-SIG");
    assert_eq!(retained.fields[1].value, "CC05-DATA");

    let wrong = "other-session.jsonl";
    assert!(!claude_jsonl_session_matches(wrong, session_id));
    let refused =
        CaptureSnapshotService::authorize_claude_attached_fields(wrong, session_id, &source);
    assert!(refused.refused);
    assert!(refused.fields.is_empty());
    let refused_again =
        CaptureSnapshotService::authorize_claude_attached_fields(wrong, session_id, &source);
    assert!(refused_again.refused);
    assert!(refused_again.fields.is_empty());
    assert_eq!(retained.fields[0].value, "CC05-SIG");

    let secret = "AKIAABCDEFGHIJKLMNOP";
    let secret_line = format!(
        "{{\"type\":\"assistant\",\"uuid\":\"cc05-turn\",\"message\":{{\"role\":\"assistant\",\"content\":[{{\"type\":\"thinking\",\"thinking\":\"visible\",\"signature\":\"{secret}\"}}]}}}}\n"
    );
    let (secret_ordinary, _) = CaptureSnapshotService::preview_redacted_coverage_v2(
        "claude_code",
        secret_line.into_bytes(),
        CaptureSnapshotPolicy::default(),
    )
    .expect("credential preview");
    let secret_text =
        String::from_utf8(secret_ordinary.bytes().to_vec()).expect("secret ordinary utf-8");
    assert!(!secret_text.contains(secret));

    let empty = CaptureSnapshotService::claude_attached_projection(
        b"{\"type\":\"assistant\",\"message\":{\"role\":\"assistant\",\"content\":[{\"type\":\"thinking\",\"thinking\":\"\",\"signature\":\"\"}]}}\n",
    );
    assert!(!empty.data_obtained());
    assert!(empty.fields.is_empty());
}

#[cfg(feature = "test-live-agent")]
#[test]
#[ignore = "operator-local Codex acceptance; never run in CI"]
fn codex_source_attached_fields_bind_session_turn_and_history() {
    let id = "01a15fb5-4b7f-7951-8621-452cc45be036";
    let leaf = format!("rollout-2026-10-22T02-24-40-{id}.jsonl");
    let source = format!(
        "{{\"type\":\"session_meta\",\"payload\":{{\"id\":\"{id}\",\"cli_version\":\"0.163.0\"}}}}\n\
{{\"type\":\"event_msg\",\"payload\":{{\"type\":\"task_started\",\"turn_id\":\"turn-A\"}}}}\n\
{{\"type\":\"response_item\",\"payload\":{{\"type\":\"reasoning\",\"id\":\"r1\",\"summary\":[{{\"type\":\"summary_text\",\"text\":\"可读摘要\"}}],\"encrypted_content\":\"ENCRYPTED-A\\u00e9\"}}}}\n\
{{\"type\":\"compacted\",\"payload\":{{\"replacement_history\":[{{\"type\":\"reasoning\",\"id\":\"r2\",\"encrypted_content\":\"HISTORY\"}}]}}}}\n"
    );
    let (ordinary, report) = CaptureSnapshotService::preview_redacted_coverage_v2(
        "codex",
        source.into_bytes(),
        CaptureSnapshotPolicy::default(),
    )
    .expect("authorized ordinary preview");
    let indexed = CaptureSnapshotService::authorize_codex_attached_fields(&leaf, id, &ordinary);
    assert!(!indexed.refused);
    assert!(indexed.readable_obtained());
    assert_eq!(indexed.readable, ["可读摘要"]);
    assert_eq!(indexed.fields.len(), 2);
    assert_eq!(indexed.fields[0].turn_key.as_deref(), Some("turn-A"));
    assert_eq!(indexed.fields[0].record_id.as_deref(), Some("r1"));
    assert_eq!(indexed.fields[0].line_ordinal, 2);
    assert_eq!(indexed.fields[0].session_id, id);
    assert_eq!(indexed.fields[0].record_ordinal, 0);
    assert_eq!(indexed.fields[0].history_index, None);
    assert_eq!(indexed.fields[0].value, "ENCRYPTED-Aé");
    assert_eq!(indexed.fields[1].history_index, Some(0));
    assert_eq!(indexed.fields[1].record_ordinal, 0);
    assert_eq!(indexed.fields[1].turn_key, None);
    assert_eq!(indexed.fields[1].record_id.as_deref(), Some("r2"));
    assert_eq!(indexed.fields[1].value, "HISTORY");
    assert!(
        indexed
            .fields
            .iter()
            .all(|field| field.availability == "unproven")
    );
    assert!(!report.to_string().contains("ENCRYPTED-A"));
    assert!(!report.to_string().contains("HISTORY"));
    let again = CaptureSnapshotService::authorize_codex_attached_fields(&leaf, id, &ordinary);
    assert!(again == indexed);
}

#[cfg(feature = "test-live-agent")]
#[test]
#[ignore = "operator-local Codex acceptance; never run in CI"]
fn codex_source_attached_fields_reject_ambiguous_or_unbound_sources() {
    let id = "01a15fb5-4b7f-7951-8621-452cc45be036";
    let leaf = format!("rollout-2026-10-22T02-24-40-{id}.jsonl");
    let header = format!(
        "{{\"type\":\"session_meta\",\"payload\":{{\"id\":\"{id}\",\"cli_version\":\"0.163.0\"}}}}\n"
    );
    let turn = "{\"type\":\"turn_context\",\"payload\":{\"turn_id\":\"turn-B\"}}\n";
    let duplicate = "{\"type\":\"response_item\",\"payload\":{\"type\":\"reasoning\",\"encrypted_content\":\"A\",\"encrypted_content\":\"B\"}}\n";
    let wrong_type = "{\"type\":\"response_item\",\"payload\":{\"type\":\"reasoning\",\"encrypted_content\":42}}\n";
    let valid = "{\"type\":\"response_item\",\"payload\":{\"type\":\"reasoning\",\"encrypted_content\":\"SURVIVES\"}}\n";
    let source = format!("{header}{turn}{duplicate}{turn}{wrong_type}{valid}");
    let ordinary = codex_ordinary(source.as_bytes());
    let indexed = CaptureSnapshotService::authorize_codex_attached_fields(&leaf, id, &ordinary);
    assert!(!indexed.refused);
    assert_eq!(indexed.duplicate_rejected, 1);
    assert!(indexed.unsupported >= 1);
    assert_eq!(indexed.fields.len(), 1);
    assert_eq!(indexed.fields[0].value, "SURVIVES");
    let wrong_leaf =
        CaptureSnapshotService::authorize_codex_attached_fields("other.jsonl", id, &ordinary);
    assert!(wrong_leaf.refused);
    assert!(wrong_leaf.fields.is_empty());
    let wrong_version = source.replace("0.163.0", "0.999.0");
    let refused = CaptureSnapshotService::authorize_codex_attached_fields(
        &leaf,
        id,
        &codex_ordinary(wrong_version.as_bytes()),
    );
    assert!(refused.refused);
    assert!(refused.fields.is_empty());
    let wrong_session = source.replacen(id, "01a15fb5-4b7f-7951-8621-452cc45be037", 1);
    let refused = CaptureSnapshotService::authorize_codex_attached_fields(
        &leaf,
        id,
        &codex_ordinary(wrong_session.as_bytes()),
    );
    assert!(refused.refused);
    assert!(refused.fields.is_empty());
    let foreign_header = format!(
        "{header}{turn}{valid}{{\"type\":\"session_meta\",\"payload\":{{\"id\":\"01a15fb5-4b7f-7951-8621-452cc45be037\",\"cli_version\":\"0.163.0\"}}}}\n{turn}{valid}"
    );
    let refused = CaptureSnapshotService::authorize_codex_attached_fields(
        &leaf,
        id,
        &codex_ordinary(foreign_header.as_bytes()),
    );
    assert!(refused.refused);
    assert!(refused.fields.is_empty());
}

#[cfg(feature = "test-live-agent")]
#[test]
#[ignore = "operator-local Codex acceptance; never run in CI"]
fn codex_source_attached_turn_version_and_redaction_vectors() {
    let id = "01a15fb5-4b7f-7951-8621-452cc45be036";
    let leaf = format!("rollout-2026-10-22T02-24-40-{id}.jsonl");
    let header = format!(
        "{{\"type\":\"session_meta\",\"payload\":{{\"id\":\"{id}\",\"cli_version\":\"0.163.0\"}}}}\n"
    );
    let turn = "{\"type\":\"turn_context\",\"payload\":{\"turn_id\":\"turn-A\"}}\n";
    let reasoning = "{\"type\":\"response_item\",\"payload\":{\"type\":\"reasoning\",\"id\":\"reason-A\",\"summary\":[{\"type\":\"summary_text\",\"text\":\"  \"}],\"encrypted_content\":\"OPAQUE-A\"}}\n";
    let project = |body: &str| {
        CaptureSnapshotService::authorize_codex_attached_fields(
            &leaf,
            id,
            &codex_ordinary(format!("{header}{body}").as_bytes()),
        )
    };
    let whitespace = project(&format!("{turn}{reasoning}"));
    assert!(whitespace.data_obtained());
    assert!(!whitespace.readable_obtained());
    assert_eq!(whitespace.fields.len(), 1);

    let foreign = format!(
        "{turn}{{\"type\":\"event_msg\",\"payload\":{{\"type\":\"task_started\",\"turn_id\":\"foreign\",\"thread_id\":\"foreign-session\"}}}}\n{{\"type\":\"response_item\",\"payload\":{{\"type\":\"reasoning\",\"id\":\"foreign-reason\",\"thread_id\":\"foreign-session\",\"encrypted_content\":\"FOREIGN\"}}}}\n{reasoning}"
    );
    let result = project(&foreign);
    assert_eq!(result.fields.len(), 1);
    assert_eq!(result.fields[0].turn_key.as_deref(), Some("turn-A"));

    let late_completion = format!(
        "{turn}{{\"type\":\"event_msg\",\"payload\":{{\"type\":\"task_complete\",\"turn_id\":\"older-turn\"}}}}\n{reasoning}"
    );
    assert_eq!(project(&late_completion).fields.len(), 1);
    let rejected_boundary = format!(
        "{turn}{{\"type\":\"turn_context\",\"payload\":{{\"turn_id\":\"turn-B\",\"turn_id\":\"turn-C\"}}}}\n{reasoning}"
    );
    assert!(project(&rejected_boundary).fields.is_empty());
    let old_version = format!("{header}{turn}{reasoning}").replace("0.163.0", "0.162.1");
    assert_eq!(
        CaptureSnapshotService::authorize_codex_attached_fields(
            &leaf,
            id,
            &codex_ordinary(old_version.as_bytes()),
        )
        .fields
        .len(),
        1
    );
    for bad in ["null", "42", "\"0.999.0\""] {
        let source = format!("{header}{turn}{reasoning}").replace("\"0.163.0\"", bad);
        let refused = CaptureSnapshotService::authorize_codex_attached_fields(
            &leaf,
            id,
            &codex_ordinary(source.as_bytes()),
        );
        assert!(refused.refused);
        assert!(refused.fields.is_empty());
    }
    let later_bad_version = format!(
        "{turn}{reasoning}{{\"type\":\"session_meta\",\"payload\":{{\"id\":\"{id}\",\"cli_version\":\"0.999.0\"}}}}\n"
    );
    assert!(project(&later_bad_version).refused);
    let secret = "AKIAABCDEFGHIJKLMNOP";
    let source = format!(
        "{header}{turn}{{\"type\":\"response_item\",\"payload\":{{\"type\":\"reasoning\",\"id\":\"secret\",\"encrypted_content\":\"{secret}\"}}}}\n"
    );
    let ordinary = codex_ordinary(source.as_bytes());
    let redacted = CaptureSnapshotService::authorize_codex_attached_fields(&leaf, id, &ordinary);
    assert!(
        !redacted
            .fields
            .iter()
            .any(|field| field.value.contains(secret))
    );
    assert!(
        !ordinary
            .bytes()
            .windows(secret.len())
            .any(|window| window == secret.as_bytes())
    );
    let invalid_escape = format!(
        "{turn}{{\"type\":\"response_item\",\"payload\":{{\"type\":\"reasoning\",\"id\":\"invalid-escape\",\"encrypted_content\":\"bad\\uD800\"}}}}\n"
    );
    assert!(project(&invalid_escape).fields.is_empty());
}
