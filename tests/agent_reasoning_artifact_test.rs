//! CC-05 ordinary Claude attached-field retention.
//!
//! The JSONL in this file is synthetic. It is not a Claude Code 2.1.303 capture
//! and it is not evidence that a live session wrote `thinking` or `signature`.

use libra::internal::ai::{
    capture::snapshot::{CaptureSnapshotPolicy, CaptureSnapshotService},
    observed_agents::claude_jsonl_session_matches,
};

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
