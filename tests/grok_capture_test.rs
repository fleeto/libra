//! GK-02: synthetic Grok `updates.jsonl` keeps each thought chunk in order.
//!
//! Fixtures are written under a temp session directory. Nothing here opens
//! `GROK_HOME` or starts `grok`.

use std::{
    fs::{self, File, OpenOptions},
    io::Write,
    path::Path,
};

use libra::internal::ai::{
    hooks::providers::grok::{
        encode_grok_cwd, install_marked_grok_hooks, observe_grok_hook, uninstall_marked_grok_hooks,
    },
    observed_agents::{
        AgentKind, AgentStability, agent_for,
        grok_reasoning::{
            GrokMergedEvent, GrokReasoningError, export_grok_ordinary, import_grok_reasoning,
            sink_bundle,
        },
        grok_updates::{
            GROK_LINE_MAX_BYTES, GrokReadCursor, GrokTranscriptEvent, GrokUpdatesError,
            read_grok_updates, session_file,
        },
        live_capture_matches_hook_installable, registration_for,
        transcript_source::TRANSCRIPT_READ_HARD_CAP_BYTES,
    },
};
use sha2::{Digest, Sha256};

fn thought_line(event_id: &str, prompt_id: &str, text: &str, timestamp: &str) -> String {
    serde_json::json!({
        "timestamp": timestamp,
        "method": "session/update",
        "params": {
            "sessionId": "sess-synthetic",
            "update": {
                "sessionUpdate": "agent_thought_chunk",
                "promptId": prompt_id,
                "content": { "type": "text", "text": text }
            },
            "_meta": { "eventId": event_id }
        }
    })
    .to_string()
}

fn tool_line(event_id: &str, title: &str) -> String {
    serde_json::json!({
        "timestamp": "2026-10-18T00:00:00Z",
        "method": "session/update",
        "params": {
            "sessionId": "sess-synthetic",
            "update": {
                "sessionUpdate": "tool_call",
                "title": title,
                "toolCallId": "call-1"
            },
            "_meta": { "eventId": event_id }
        }
    })
    .to_string()
}

fn turn_line(event_id: &str, reasoning_tokens: u64) -> String {
    serde_json::json!({
        "timestamp": "2026-10-18T00:00:00Z",
        "method": "session/update",
        "params": {
            "sessionId": "sess-synthetic",
            "update": {
                "sessionUpdate": "turn_completed",
                "usage": { "reasoningTokens": reasoning_tokens }
            },
            "_meta": { "eventId": event_id }
        }
    })
    .to_string()
}

fn write_jsonl(dir: &Path, lines: &[String]) -> std::path::PathBuf {
    let path = session_file(dir, "updates.jsonl");
    let mut body = String::new();
    for line in lines {
        body.push_str(line);
        body.push('\n');
    }
    fs::write(&path, body).expect("write synthetic updates.jsonl");
    path
}

fn thought_texts(events: &[GrokTranscriptEvent]) -> Vec<&str> {
    events
        .iter()
        .filter_map(|event| match event {
            GrokTranscriptEvent::Thought { text, .. } => Some(text.as_str()),
            _ => None,
        })
        .collect()
}

#[test]
fn grok_updates_keeps_three_thoughts_in_order() {
    let session = tempfile::tempdir().expect("session dir");
    let path = write_jsonl(
        session.path(),
        &[
            thought_line("e1", "p1", "alpha", "2026-10-18T00:00:00Z"),
            tool_line("e2", "shell"),
            thought_line("e3", "p2", "beta", "2026-10-18T00:00:00Z"),
            turn_line("e4", 17),
            thought_line("e5", "p3", "gamma", "2026-10-18T00:00:00Z"),
        ],
    );
    let read = read_grok_updates(session.path(), &path, &GrokReadCursor::default())
        .expect("read synthetic updates");
    assert!(read.is_complete());
    assert_eq!(thought_texts(&read.events), ["alpha", "beta", "gamma"]);
    assert_eq!(
        read.events,
        vec![
            GrokTranscriptEvent::Thought {
                text: "alpha".to_string(),
                prompt_id: "p1".to_string(),
                event_id: "e1".to_string(),
            },
            GrokTranscriptEvent::Tool {
                name: "shell".to_string(),
                event_id: "e2".to_string(),
            },
            GrokTranscriptEvent::Thought {
                text: "beta".to_string(),
                prompt_id: "p2".to_string(),
                event_id: "e3".to_string(),
            },
            GrokTranscriptEvent::Turn {
                reasoning_tokens: 17,
                event_id: "e4".to_string(),
            },
            GrokTranscriptEvent::Thought {
                text: "gamma".to_string(),
                prompt_id: "p3".to_string(),
                event_id: "e5".to_string(),
            },
        ]
    );
    assert!(
        thought_texts(&read.events).iter().all(|text| *text != "17"),
        "reasoningTokens is a count, not thought text"
    );
}

#[test]
fn grok_updates_string_chunks_are_not_concatenated() {
    let session = tempfile::tempdir().expect("session dir");
    let path = write_jsonl(
        session.path(),
        &[
            thought_line("c1", "same-prompt", "left-", "2026-10-18T00:00:01Z"),
            thought_line("c2", "same-prompt", "right", "2026-10-18T00:00:02Z"),
        ],
    );
    let read = read_grok_updates(session.path(), &path, &GrokReadCursor::default())
        .expect("read synthetic updates");
    assert!(read.is_complete());
    assert_eq!(read.events.len(), 2);
    match &read.events[0] {
        GrokTranscriptEvent::Thought {
            text, prompt_id, ..
        } => {
            assert_eq!(text, "left-");
            assert_eq!(prompt_id, "same-prompt");
        }
        other => panic!("first chunk must stay its own thought, got {other:?}"),
    }
    match &read.events[1] {
        GrokTranscriptEvent::Thought {
            text, prompt_id, ..
        } => {
            assert_eq!(text, "right");
            assert_eq!(prompt_id, "same-prompt");
        }
        other => panic!("second chunk must stay its own thought, got {other:?}"),
    }
    assert!(
        read.events.iter().all(|event| match event {
            GrokTranscriptEvent::Thought { text, .. } => text != "left-right",
            _ => true,
        }),
        "same promptId chunks must not be joined"
    );
}

#[test]
fn grok_updates_decoded_utf8_escape_and_unicode() {
    let session = tempfile::tempdir().expect("session dir");
    // Rust escapes below are the JSON spelling: \" and \u754c. `café` is raw UTF-8.
    let line = "{\"timestamp\":\"t\",\"method\":\"session/update\",\"params\":{\"sessionId\":\"s\",\"update\":{\"sessionUpdate\":\"agent_thought_chunk\",\"promptId\":\"p\",\"content\":{\"type\":\"text\",\"text\":\"say \\\"\\u754c\\\" café\"}},\"_meta\":{\"eventId\":\"e-utf\"}}}";
    assert!(line.contains(r#"\u754c"#));
    assert!(line.contains(r#"\""#));
    assert!(line.contains("café"));
    let path = session_file(session.path(), "updates.jsonl");
    fs::write(&path, format!("{line}\n")).expect("write escaped line");
    let read = read_grok_updates(session.path(), &path, &GrokReadCursor::default())
        .expect("read escaped line");
    assert!(read.is_complete());
    assert_eq!(
        thought_texts(&read.events),
        ["say \"界\" café"],
        "JSON escapes and non-ASCII must be decoded UTF-8"
    );
}

#[test]
fn grok_updates_repeat_event_id_is_idempotent() {
    let session = tempfile::tempdir().expect("session dir");
    let path = write_jsonl(
        session.path(),
        &[
            thought_line("e1", "p", "alpha", "2026-10-18T00:00:00Z"),
            thought_line("e1", "p", "alpha-duplicate", "2026-10-18T00:00:09Z"),
            thought_line("e2", "p", "beta", "2026-10-18T00:00:00Z"),
            thought_line("e3", "p", "gamma", "2026-10-18T00:00:00Z"),
        ],
    );
    let first =
        read_grok_updates(session.path(), &path, &GrokReadCursor::default()).expect("first read");
    assert!(first.is_complete());
    assert_eq!(
        thought_texts(&first.events),
        ["alpha", "beta", "gamma"],
        "duplicate eventId is one thought; equal timestamps stay separate"
    );
    let cursor = GrokReadCursor {
        offset: first.next_offset,
        seen_event_ids: first.seen_event_ids.clone(),
    };
    let second = read_grok_updates(session.path(), &path, &cursor).expect("reread");
    assert!(
        second.events.is_empty(),
        "reread from the recorded offset adds nothing"
    );

    let appended = thought_line("e4", "p", "delta", "2026-10-17T00:00:00Z");
    let mut file = OpenOptions::new().append(true).open(&path).expect("append");
    writeln!(file, "{appended}").expect("write appended thought");
    drop(file);
    let third = read_grok_updates(session.path(), &path, &cursor).expect("resume");
    assert_eq!(thought_texts(&third.events), ["delta"]);
    assert!(
        third
            .events
            .iter()
            .all(|event| !matches!(event, GrokTranscriptEvent::Thought { text, .. } if text == "alpha" || text == "beta" || text == "gamma")),
        "resume must not merge the new row into earlier rows by timestamp"
    );

    let from_start = GrokReadCursor {
        offset: 0,
        seen_event_ids: first.seen_event_ids,
    };
    let replay = read_grok_updates(session.path(), &path, &from_start).expect("replay");
    assert_eq!(
        thought_texts(&replay.events),
        ["delta"],
        "a saved eventId set does not add the same thought again"
    );
}

#[test]
fn grok_updates_non_string_text_marks_partial() {
    let session = tempfile::tempdir().expect("session dir");
    let non_string = serde_json::json!({
        "timestamp": "t",
        "method": "session/update",
        "params": {
            "sessionId": "s",
            "update": {
                "sessionUpdate": "agent_thought_chunk",
                "promptId": "p",
                "content": { "type": "text", "text": { "nested": true } }
            },
            "_meta": { "eventId": "bad" }
        }
    })
    .to_string();
    let wrong_type = serde_json::json!({
        "timestamp": "t",
        "method": "session/update",
        "params": {
            "sessionId": "s",
            "update": {
                "sessionUpdate": "agent_thought_chunk",
                "promptId": "p",
                "content": { "type": "image", "text": "hidden" }
            },
            "_meta": { "eventId": "img" }
        }
    })
    .to_string();
    let path = write_jsonl(
        session.path(),
        &[
            thought_line("ok1", "p", "kept-before", "t"),
            non_string,
            wrong_type,
            thought_line("ok2", "p", "kept-after", "t"),
        ],
    );
    let read =
        read_grok_updates(session.path(), &path, &GrokReadCursor::default()).expect("partial read");
    assert!(read.partial);
    assert!(!read.is_complete());
    assert_eq!(thought_texts(&read.events), ["kept-before", "kept-after"]);
}

#[test]
fn grok_updates_unknown_session_update_keeps_type_and_hash() {
    let session = tempfile::tempdir().expect("session dir");
    let unknown = serde_json::json!({
        "timestamp": "t",
        "method": "session/update",
        "params": {
            "sessionId": "s",
            "update": { "sessionUpdate": "agent_custom_widget", "payload": 1 },
            "_meta": { "eventId": "u1" }
        }
    })
    .to_string();
    let expected_hash = format!("{:x}", Sha256::digest(unknown.as_bytes()));
    let path = write_jsonl(
        session.path(),
        &[thought_line("ok", "p", "real-thought", "t"), unknown],
    );
    let read = read_grok_updates(session.path(), &path, &GrokReadCursor::default())
        .expect("read unknown update");
    assert!(read.is_complete());
    assert_eq!(thought_texts(&read.events), ["real-thought"]);
    assert!(read.events.iter().any(|event| matches!(
        event,
        GrokTranscriptEvent::Unknown {
            session_update,
            line_sha256,
        } if session_update == "agent_custom_widget" && line_sha256 == &expected_hash
    )));
    assert!(
        read.events
            .iter()
            .all(|event| !matches!(event, GrokTranscriptEvent::Thought { text, .. } if text.contains("agent_custom_widget")))
    );
}

#[test]
fn grok_updates_oversized_line_is_not_complete() {
    let session = tempfile::tempdir().expect("session dir");
    let path = session_file(session.path(), "updates.jsonl");
    let mut file = File::create(&path).expect("create");
    let huge = "x".repeat((GROK_LINE_MAX_BYTES as usize) + 1);
    writeln!(file, "{huge}").expect("write oversized line");
    let kept = thought_line("after", "p", "still-here", "t");
    writeln!(file, "{kept}").expect("write following thought");
    drop(file);
    let read = read_grok_updates(session.path(), &path, &GrokReadCursor::default())
        .expect("read oversized line");
    assert!(read.partial);
    assert!(!read.is_complete());
    assert_eq!(thought_texts(&read.events), ["still-here"]);
    assert!(
        read.events.iter().all(|event| match event {
            GrokTranscriptEvent::Thought { text, .. } => text.len() < 64,
            _ => true,
        }),
        "the oversized line must not be stored"
    );
}

#[test]
fn grok_updates_oversized_source_is_refused() {
    let session = tempfile::tempdir().expect("session dir");
    let path = session_file(session.path(), "updates.jsonl");
    let file = File::create(&path).expect("create");
    file.set_len(TRANSCRIPT_READ_HARD_CAP_BYTES + 1)
        .expect("set logical size");
    drop(file);
    let error = read_grok_updates(session.path(), &path, &GrokReadCursor::default())
        .expect_err("source above 16 MiB is refused");
    match error {
        GrokUpdatesError::Refused { reason, .. } => {
            assert!(
                reason.contains(&TRANSCRIPT_READ_HARD_CAP_BYTES.to_string()),
                "reason should name the cap, got {reason}"
            );
        }
        other => panic!("expected refusal, got {other}"),
    }
}

#[test]
fn grok_updates_symlink_directory_and_outside_path_are_refused() {
    let session = tempfile::tempdir().expect("session dir");
    let real = write_jsonl(session.path(), &[thought_line("e", "p", "inside", "t")]);
    let link = session.path().join("link.jsonl");
    std::os::unix::fs::symlink(&real, &link).expect("symlink");
    let link_error = read_grok_updates(session.path(), &link, &GrokReadCursor::default())
        .expect_err("symlink is refused");
    assert!(
        matches!(link_error, GrokUpdatesError::Refused { ref reason, .. } if reason.contains("symlink")),
        "got {link_error}"
    );

    let directory = session.path().join("not-a-file");
    fs::create_dir(&directory).expect("dir");
    let dir_error = read_grok_updates(session.path(), &directory, &GrokReadCursor::default())
        .expect_err("directory is refused");
    assert!(
        matches!(dir_error, GrokUpdatesError::Refused { ref reason, .. } if reason.contains("regular file")),
        "got {dir_error}"
    );

    let outside = tempfile::tempdir().expect("outside dir");
    let outside_file = write_jsonl(outside.path(), &[thought_line("e", "p", "outside", "t")]);
    let outside_error =
        read_grok_updates(session.path(), &outside_file, &GrokReadCursor::default())
            .expect_err("path outside the session dir is refused");
    assert!(
        matches!(outside_error, GrokUpdatesError::Refused { ref reason, .. } if reason.contains("outside")),
        "got {outside_error}"
    );
}

fn reasoning_line(id: &str, status: &str, summaries: &[&str], encrypted: &str) -> String {
    let summary = summaries
        .iter()
        .map(|text| serde_json::json!({"type": "summary_text", "text": text}))
        .collect::<Vec<_>>();
    serde_json::json!({
        "type": "reasoning",
        "id": id,
        "status": status,
        "summary": summary,
        "encrypted_content": encrypted,
    })
    .to_string()
}

fn thought(event_id: &str, prompt_id: &str, text: &str) -> GrokTranscriptEvent {
    GrokTranscriptEvent::Thought {
        text: text.to_string(),
        prompt_id: prompt_id.to_string(),
        event_id: event_id.to_string(),
    }
}

fn write_history(dir: &Path, lines: &[String]) -> std::path::PathBuf {
    let path = session_file(dir, "chat_history.jsonl");
    let mut body = String::new();
    for line in lines {
        body.push_str(line);
        body.push('\n');
    }
    fs::write(&path, body).expect("write chat history");
    path
}

#[test]
fn grok_unproven_ciphertext_is_in_transcript_and_absent_from_projection() {
    let session = tempfile::tempdir().expect("session dir");
    let secret = "sk-ant-api03-AAAAAAAAAAAAAAAAAA";
    let canary = "GROK-CANARY-7c1e";
    let exact_cipher = "CIPHER-EXACT-7c1e";
    let join_cipher = "CIPHER-JOIN-7c1e";
    let orphan_cipher = "CIPHER-ORPHAN-7c1e";
    let reasoning_id = "rid-7c1e";
    let updates = vec![
        thought("e1", "p-join", "W1"),
        thought("e2", "p-join", "W2"),
        thought("e3", "p-join", "W3"),
        thought("e4", "p-join", "W4"),
        GrokTranscriptEvent::Tool {
            name: "shell".to_string(),
            event_id: "e-tool".to_string(),
        },
        thought("e-exact", "p-exact", canary),
        thought("e-secret", "p-secret", secret),
    ];
    let history = write_history(
        session.path(),
        &[
            reasoning_line("rid-join", "completed", &["W1W2W3W4"], join_cipher),
            reasoning_line(reasoning_id, "completed", &[canary], exact_cipher),
            reasoning_line(
                "rid-orphan",
                "completed",
                &["orphan-summary"],
                orphan_cipher,
            ),
        ],
    );
    let import = import_grok_reasoning(session.path(), &history, &updates).expect("import");
    assert!(import.is_complete());
    let document = import.transcript_document();
    let transcript = document.to_string();
    assert!(transcript.contains(canary));
    assert!(transcript.contains(exact_cipher));
    assert!(transcript.contains(join_cipher));
    assert!(transcript.contains(orphan_cipher));
    assert!(transcript.contains(reasoning_id));
    assert!(transcript.contains("completed"));
    assert!(transcript.contains("orphan-summary"));
    assert!(transcript.contains("encrypted_content"));
    assert!(
        !transcript.contains("W1W2W3W4"),
        "a four-chunk join is not stored as a second body"
    );
    assert_eq!(
        transcript.matches(canary).count(),
        1,
        "exact summary is not duplicated"
    );
    assert!(transcript.contains("<REDACTED:anthropic-api-key>"));
    assert!(!transcript.contains(secret));

    let mut attached_join = 0;
    let mut attached_exact = 0;
    for event in &import.events {
        match event {
            GrokMergedEvent::Thought {
                event_id,
                text,
                reasoning,
                ..
            } => {
                if event_id == "e1" {
                    let reasoning = reasoning
                        .as_ref()
                        .expect("join ciphertext hangs on the first chunk");
                    assert_eq!(reasoning.encrypted_content, join_cipher);
                    attached_join += 1;
                    assert_eq!(text, "W1");
                }
                if event_id == "e-exact" {
                    let reasoning = reasoning
                        .as_ref()
                        .expect("exact ciphertext hangs on that thought");
                    assert_eq!(reasoning.encrypted_content, exact_cipher);
                    assert_eq!(reasoning.id, reasoning_id);
                    assert_eq!(reasoning.status, "completed");
                    attached_exact += 1;
                }
                assert_ne!(text, "W1W2W3W4");
            }
            GrokMergedEvent::Reasoning {
                summary_text,
                encrypted_content,
                ..
            } => {
                assert_eq!(summary_text.as_deref(), Some("orphan-summary"));
                assert_eq!(encrypted_content, orphan_cipher);
            }
            GrokMergedEvent::Tool { name, .. } => assert_eq!(name, "shell"),
            other => panic!("unexpected event {other:?}"),
        }
    }
    assert_eq!(attached_join, 1);
    assert_eq!(attached_exact, 1);
    let tool_at = import
        .events
        .iter()
        .position(|event| matches!(event, GrokMergedEvent::Tool { .. }))
        .expect("tool");
    assert!(matches!(
        &import.events[tool_at - 1],
        GrokMergedEvent::Thought { event_id, .. } if event_id == "e4"
    ));
    assert!(matches!(
        &import.events[tool_at + 1],
        GrokMergedEvent::Thought { text, .. } if text == canary
    ));

    let sinks = sink_bundle(&import);
    for (name, body) in [
        ("metadata", sinks.metadata_json.as_str()),
        ("coverage", sinks.typed_coverage_json.as_str()),
        ("debug", sinks.debug_text.as_str()),
        ("projection", sinks.safe_projection_json.as_str()),
        ("doctor", sinks.doctor_json.as_str()),
        ("digest", sinks.coverage_digest.as_str()),
        ("log", sinks.log_line.as_str()),
    ] {
        for secret_text in [
            canary,
            exact_cipher,
            join_cipher,
            orphan_cipher,
            reasoning_id,
            secret,
            "orphan-summary",
        ] {
            assert!(
                !body.contains(secret_text),
                "{name} contains {secret_text}: {body}"
            );
        }
    }
    assert!(sinks.doctor_json.contains("<REDACTED:anthropic-api-key>"));

    let mut with_metadata = document;
    with_metadata["metadata"] = serde_json::json!({
        "openai": {"reasoningEncryptedContent": "OPENAI-CANARY-7c1e", "itemId": "keep-id"},
        "anthropic": {"signature": "ANTHROPIC-CANARY-7c1e"}
    });
    let exported = export_grok_ordinary(&with_metadata).expect("export");
    let export_text = String::from_utf8(exported).expect("utf8 export");
    assert!(!export_text.contains("encrypted_content"));
    assert!(!export_text.contains("reasoningEncryptedContent"));
    assert!(!export_text.contains("\"signature\""));
    assert!(!export_text.contains(exact_cipher));
    assert!(export_text.contains("keep-id"));
    assert!(transcript.contains(exact_cipher));
}

#[test]
fn grok_reasoning_empty_summary_keeps_ciphertext() {
    let session = tempfile::tempdir().expect("session dir");
    let history = write_history(
        session.path(),
        &[reasoning_line(
            "rid-empty",
            "completed",
            &[],
            "CIPHER-EMPTY-7c1e",
        )],
    );
    let import = import_grok_reasoning(
        session.path(),
        &history,
        &[thought("e", "p", "visible-thought")],
    )
    .expect("import");
    assert!(import.is_complete());
    match &import.events[..] {
        [
            GrokMergedEvent::Thought {
                text, reasoning, ..
            },
            GrokMergedEvent::Reasoning {
                id,
                status,
                summary_text,
                encrypted_content,
            },
        ] => {
            assert_eq!(text, "visible-thought");
            assert!(reasoning.is_none());
            assert_eq!(id, "rid-empty");
            assert_eq!(status, "completed");
            assert_eq!(summary_text, &None);
            assert_eq!(encrypted_content, "CIPHER-EMPTY-7c1e");
        }
        other => panic!("expected thought plus ciphertext-only reasoning, got {other:?}"),
    }
    let transcript = import.transcript_document().to_string();
    assert!(transcript.contains("CIPHER-EMPTY-7c1e"));
    assert!(!transcript.contains("summary_text\":\""));
}

#[test]
fn grok_reasoning_non_string_ciphertext_is_partial() {
    let session = tempfile::tempdir().expect("session dir");
    let bad = serde_json::json!({
        "type": "reasoning",
        "id": "rid-bad",
        "status": "completed",
        "summary": [{"type": "summary_text", "text": "kept-out"}],
        "encrypted_content": {"blob": true}
    })
    .to_string();
    let history = write_history(
        session.path(),
        &[
            reasoning_line("rid-good", "completed", &["visible-thought"], "CIPHER-GOOD"),
            bad,
        ],
    );
    let import = import_grok_reasoning(
        session.path(),
        &history,
        &[thought("e", "p", "visible-thought")],
    )
    .expect("partial import");
    assert!(import.partial);
    assert!(!import.is_complete());
    let transcript = import.transcript_document().to_string();
    assert!(transcript.contains("CIPHER-GOOD"));
    assert!(transcript.contains("visible-thought"));
    assert!(!transcript.contains("rid-bad"));
    assert!(!transcript.contains("kept-out"));
    assert!(!transcript.contains("blob"));
}

#[test]
fn grok_reasoning_outside_matrix_is_refused() {
    let session = tempfile::tempdir().expect("session dir");
    let history = write_history(
        session.path(),
        &[
            reasoning_line("rid-good", "completed", &["visible-thought"], "CIPHER-GOOD"),
            r#"{"type":"reasoning","id":"rid-x","status":"completed","summary":[],"cipher":"dropped-thinking"}"#.to_string(),
        ],
    );
    let error = import_grok_reasoning(
        session.path(),
        &history,
        &[thought("e", "p", "visible-thought")],
    )
    .expect_err("shape outside the matrix is refused");
    assert!(
        matches!(error, GrokReasoningError::Malformed { ref reason } if reason.contains("encrypted_content")),
        "got {error}"
    );
    let rendered = error.to_string();
    assert!(rendered.contains("malformed transcript"));
    assert!(!rendered.contains("dropped-thinking"));
}

const LIBRA_HOOK_COMMAND: &str = "libra agent hooks grok";

fn hook_event(
    name: &str,
    session_id: &str,
    cwd: &str,
    transcript: Option<&str>,
) -> serde_json::Value {
    let mut event = serde_json::json!({
        "hook_event_name": name,
        "sessionId": session_id,
        "cwd": cwd,
    });
    if let Some(transcript) = transcript {
        event["transcript_path"] = serde_json::json!(transcript);
    }
    event
}

fn write_session_updates(
    root: &Path,
    cwd: &str,
    session_id: &str,
    body: &str,
) -> std::path::PathBuf {
    let path = root
        .join(encode_grok_cwd(cwd))
        .join(session_id)
        .join("updates.jsonl");
    fs::create_dir_all(path.parent().expect("session dir")).expect("mkdir");
    fs::write(&path, body).expect("write updates");
    path
}

fn stored_thoughts(state: &Path, session_id: &str) -> Vec<String> {
    let bytes = fs::read(state.join(session_id).join("store.json")).expect("store");
    let value: serde_json::Value = serde_json::from_slice(&bytes).expect("json");
    value["thoughts"]
        .as_array()
        .expect("thoughts")
        .iter()
        .map(|item| item.as_str().expect("text").to_string())
        .collect()
}

#[test]
fn grok_hook_marker_round_trip_keeps_user_bytes() {
    let hooks = tempfile::tempdir().expect("hooks");
    let user_path = hooks.path().join("user.json");
    let user_bytes = b"{\n  \"hooks\": {\n    \"Stop\": [{\"matcher\": \"user\", \"hooks\": [{\"type\": \"command\", \"command\": \"echo keep-me\"}]}]\n  }\n}\n";
    fs::write(&user_path, user_bytes).expect("user file");
    let mixed_path = hooks.path().join("mixed.json");
    fs::write(
        &mixed_path,
        serde_json::to_vec_pretty(&serde_json::json!({
            "note": "leave-me",
            "hooks": {
                "Stop": [
                    {"matcher": "user", "hooks": [{"type": "command", "command": "echo keep-user"}]},
                    {"statusMessage": "libra capture", "hooks": [{"type": "command", "command": "echo old-mark", "timeout": 10}]}
                ]
            }
        }))
        .expect("mixed"),
    )
    .expect("write mixed");
    install_marked_grok_hooks(hooks.path(), LIBRA_HOOK_COMMAND).expect("install");
    assert_eq!(fs::read(&user_path).expect("reread user"), user_bytes);
    let mixed: serde_json::Value =
        serde_json::from_slice(&fs::read(&mixed_path).expect("mixed")).expect("parse mixed");
    let groups = mixed["hooks"]["Stop"].as_array().expect("groups");
    assert_eq!(groups[0]["hooks"][0]["command"], "echo keep-user");
    assert_eq!(groups[1]["hooks"][0]["command"], LIBRA_HOOK_COMMAND);
    assert_eq!(mixed["note"], "leave-me");
    let owned = fs::read_to_string(hooks.path().join("libra-grok-capture.json")).expect("owned");
    assert!(owned.contains(LIBRA_HOOK_COMMAND));
    assert!(owned.contains("libra capture"));
    assert!(!owned.contains("\"command\": \"grok"));
    uninstall_marked_grok_hooks(hooks.path()).expect("uninstall");
    assert_eq!(fs::read(&user_path).expect("user after"), user_bytes);
    assert!(!hooks.path().join("libra-grok-capture.json").exists());
    let mixed: serde_json::Value =
        serde_json::from_slice(&fs::read(&mixed_path).expect("mixed after")).expect("parse");
    let groups = mixed["hooks"]["Stop"].as_array().expect("groups");
    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0]["hooks"][0]["command"], "echo keep-user");
    assert_eq!(mixed["note"], "leave-me");
    assert!(
        !serde_json::to_string(&mixed)
            .expect("text")
            .contains("libra capture")
    );
}

#[test]
fn grok_hook_does_not_spawn_grok() {
    let hooks = tempfile::tempdir().expect("hooks");
    let rejected = install_marked_grok_hooks(hooks.path(), "grok --serve");
    assert!(rejected.is_err(), "install must refuse a grok program");
    assert!(
        fs::read_dir(hooks.path())
            .map(|entries| entries.count())
            .unwrap_or(0)
            == 0
    );
    install_marked_grok_hooks(hooks.path(), LIBRA_HOOK_COMMAND).expect("install libra");
    for entry in fs::read_dir(hooks.path()).expect("list") {
        let path = entry.expect("entry").path();
        let text = fs::read_to_string(&path).expect("hook file");
        let value: serde_json::Value = serde_json::from_str(&text).expect("json");
        let hooks_obj = value.get("hooks").and_then(|item| item.as_object());
        let Some(hooks_obj) = hooks_obj else {
            continue;
        };
        for groups in hooks_obj.values() {
            for group in groups.as_array().into_iter().flatten() {
                for hook in group
                    .get("hooks")
                    .and_then(|item| item.as_array())
                    .into_iter()
                    .flatten()
                {
                    let command = hook
                        .get("command")
                        .and_then(|item| item.as_str())
                        .unwrap_or("");
                    let program = command.split_whitespace().next().unwrap_or("");
                    assert_ne!(
                        Path::new(program)
                            .file_name()
                            .and_then(|name| name.to_str()),
                        Some("grok"),
                        "installed command must not be grok: {command}"
                    );
                }
            }
        }
    }
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/internal/ai/hooks/providers/grok");
    for name in ["mod.rs", "parser.rs", "settings.rs"] {
        let text = fs::read_to_string(source.join(name)).expect("source");
        assert!(
            !text.contains("std::process") && !text.contains("Command::new"),
            "{name} must not spawn a process"
        );
    }
    assert!(agent_for(AgentKind::Grok).as_hooks().is_none());
    let sessions = tempfile::tempdir().expect("sessions");
    let state = tempfile::tempdir().expect("state");
    let cwd = sessions.path().join("work");
    fs::create_dir_all(&cwd).expect("cwd");
    let cwd_text = cwd.to_string_lossy().to_string();
    let encoded = encode_grok_cwd(&cwd_text);
    assert!(encoded.len() <= 255);
    assert!(encoded.contains("%2F") || encoded.contains('-'));
    let updates = write_session_updates(
        sessions.path(),
        &cwd_text,
        "sess-1",
        &format!(
            "{}\n",
            thought_line("e1", "p", "alpha", "2026-10-18T00:00:00Z")
        ),
    );
    let outcome = observe_grok_hook(
        sessions.path(),
        state.path(),
        &hook_event("Stop", "sess-1", &cwd_text, None),
        None,
    );
    assert_eq!(outcome.exit_code, 0);
    assert_eq!(outcome.stdout, "{}");
    assert!(!outcome.stdout.contains("deny"));
    assert!(!outcome.stdout.contains("block"));
    assert!(outcome.complete);
    assert_eq!(stored_thoughts(state.path(), "sess-1"), ["alpha"]);
    let link = updates.with_file_name("link.jsonl");
    std::os::unix::fs::symlink(&updates, &link).expect("symlink");
    let via_link = observe_grok_hook(
        sessions.path(),
        state.path(),
        &hook_event("Stop", "sess-1", &cwd_text, Some(&link.to_string_lossy())),
        None,
    );
    assert!(via_link.refused);
    assert_eq!(stored_thoughts(state.path(), "sess-1"), ["alpha"]);
    let outside = tempfile::tempdir().expect("outside");
    let outside_file = outside.path().join("updates.jsonl");
    fs::write(&outside_file, "{}\n").expect("outside");
    let outside_outcome = observe_grok_hook(
        sessions.path(),
        state.path(),
        &hook_event(
            "Stop",
            "sess-1",
            &cwd_text,
            Some(&outside_file.to_string_lossy()),
        ),
        None,
    );
    assert!(outside_outcome.refused);
    let missing = observe_grok_hook(
        sessions.path(),
        state.path(),
        &serde_json::json!({"hook_event_name": "Stop", "sessionId": "sess-1"}),
        None,
    );
    assert!(missing.lifecycle_only);
    assert!(!missing.complete);
    let short = observe_grok_hook(
        sessions.path(),
        state.path(),
        &hook_event("SessionEnd", "sess-1", &cwd_text, None),
        Some(1_500),
    );
    assert_eq!(short.exit_code, 0);
    assert!(short.pending);
    assert!(!short.complete);
    assert!(state.path().join("sess-1").join("pending.json").exists());
}

#[test]
fn grok_hook_precompact_and_shrink_keep_captured_thoughts() {
    let sessions = tempfile::tempdir().expect("sessions");
    let state = tempfile::tempdir().expect("state");
    let cwd = "/var/lib/grok-hook-synthetic";
    let updates = write_session_updates(
        sessions.path(),
        cwd,
        "sess-keep",
        &format!(
            "{}\n{}\n",
            thought_line("e1", "p", "alpha", "t"),
            thought_line("e2", "p", "gamma", "t")
        ),
    );
    let stop = observe_grok_hook(
        sessions.path(),
        state.path(),
        &hook_event("Stop", "sess-keep", cwd, None),
        None,
    );
    assert!(stop.complete);
    assert_eq!(
        stored_thoughts(state.path(), "sess-keep"),
        ["alpha", "gamma"]
    );
    let compact = observe_grok_hook(
        sessions.path(),
        state.path(),
        &hook_event("PreCompact", "sess-keep", cwd, None),
        None,
    );
    assert_eq!(compact.exit_code, 0);
    let checkpoint = fs::read_to_string(state.path().join("sess-keep").join("checkpoint.json"))
        .expect("checkpoint");
    assert!(checkpoint.contains("alpha"));
    assert!(checkpoint.contains("gamma"));
    fs::write(
        &updates,
        format!("{}\n", thought_line("e3", "p", "beta", "t")),
    )
    .expect("shorter file");
    let again = observe_grok_hook(
        sessions.path(),
        state.path(),
        &hook_event("StopFailure", "sess-keep", cwd, None),
        None,
    );
    assert!(again.complete);
    let thoughts = stored_thoughts(state.path(), "sess-keep");
    assert!(thoughts.contains(&"alpha".to_string()));
    assert!(thoughts.contains(&"gamma".to_string()));
    assert!(thoughts.contains(&"beta".to_string()));
    let before = thoughts.len();
    let mut repeated = fs::read_to_string(&updates).expect("current");
    repeated.push_str(&thought_line("e3", "p", "beta-again", "t"));
    repeated.push('\n');
    fs::write(&updates, repeated).expect("repeat id");
    observe_grok_hook(
        sessions.path(),
        state.path(),
        &hook_event("StopCancelled", "sess-keep", cwd, None),
        None,
    );
    let after = stored_thoughts(state.path(), "sess-keep");
    assert!(after.starts_with(&thoughts));
    assert_eq!(
        after.iter().filter(|text| text.as_str() == "beta").count(),
        1
    );
    assert!(!after.iter().any(|text| text.as_str() == "beta-again"));
    assert!(after.len() >= before);

    let child_cwd = "/var/lib/grok-hook-child";
    write_session_updates(
        sessions.path(),
        child_cwd,
        "child-1",
        &format!("{}\n", thought_line("c1", "p", "child-thought", "t")),
    );
    let mut child_event = hook_event("Stop", "child-1", child_cwd, None);
    child_event["parentSessionId"] = serde_json::json!("parent-1");
    observe_grok_hook(sessions.path(), state.path(), &child_event, None);
    assert_eq!(stored_thoughts(state.path(), "child-1"), ["child-thought"]);
    let parent =
        fs::read_to_string(state.path().join("parent-1").join("store.json")).expect("parent");
    assert!(parent.contains("child-1"));
    assert!(!parent.contains("child-thought"));

    let huge = "x".repeat((GROK_LINE_MAX_BYTES as usize) + 8);
    write_session_updates(sessions.path(), cwd, "sess-partial", &format!("{huge}\n"));
    let partial = observe_grok_hook(
        sessions.path(),
        state.path(),
        &hook_event("Stop", "sess-partial", cwd, None),
        None,
    );
    assert!(partial.pending);
    assert!(!partial.complete);
    assert!(
        state
            .path()
            .join("sess-partial")
            .join("pending.json")
            .exists()
    );
}

#[test]
fn grok_live_capture_matches_hook_installable() {
    let row = registration_for(AgentKind::Grok);
    assert!(live_capture_matches_hook_installable(AgentKind::Grok));
    assert!(row.supported);
    assert!(row.hook_installable);
    assert_eq!(row.support_wave, Some("grok"));
    assert!(!row.launchable_review);
    assert!(!row.launchable_investigate);
    let agent = agent_for(AgentKind::Grok);
    assert!(agent.as_hooks().is_some());
    assert_eq!(agent.stability(), AgentStability::Stable);
    for kind in AgentKind::all() {
        assert!(live_capture_matches_hook_installable(*kind));
    }
}
