//! GK-02: synthetic Grok `updates.jsonl` keeps each thought chunk in order.
//!
//! Fixtures are written under a temp session directory. Nothing here opens
//! `GROK_HOME` or starts `grok`.

use std::{
    fs::{self, File, OpenOptions},
    io::Write,
    path::Path,
};

use libra::internal::ai::observed_agents::{
    grok_updates::{
        GROK_LINE_MAX_BYTES, GrokReadCursor, GrokTranscriptEvent, GrokUpdatesError,
        read_grok_updates, session_file,
    },
    transcript_source::TRANSCRIPT_READ_HARD_CAP_BYTES,
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
