//! plan-20260713 M4 historical transcript import contract.

use std::{
    io::{Read, Write},
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
    str::FromStr,
    sync::Arc,
    time::{Duration, Instant},
};

use git_internal::hash::ObjectHash;
use libra::{
    internal::{
        ai::{
            agent_import::restore_tombstone,
            history::{HistoryManager, TracesInflightMarker, write_traces_inflight_marker},
            observed_agents::{
                AgentKind, normalize_claude_transcript, redact_turns, safe_turn_projection,
            },
        },
        branch::TRACES_BRANCH,
    },
    utils::{client_storage::ClientStorage, object::read_git_object},
};
use sea_orm::{ConnectionTrait, Database, Statement};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use tempfile::TempDir;

struct ImportRepo {
    _tmp: TempDir,
    repo: PathBuf,
    home: PathBuf,
}

impl ImportRepo {
    fn init() -> Self {
        let tmp = TempDir::new().expect("create tempdir");
        let repo = tmp.path().join("repo");
        let home = tmp.path().join("home");
        std::fs::create_dir_all(&repo).expect("create repo dir");
        std::fs::create_dir_all(&home).expect("create home dir");
        // The binary runs with `current_dir(repo)` and therefore sees the
        // canonical path, which is what it hashes into the Claude project
        // slug and compares against for containment. `tempfile` hands back the
        // caller's spelling, and stock macOS reaches TMPDIR through
        // `/var -> private/var`, so an uncanonicalized fixture path makes
        // discovery look in a directory the binary never writes to.
        let repo = repo.canonicalize().expect("canonical repo dir");
        let home = home.canonicalize().expect("canonical home dir");
        let fixture = Self {
            _tmp: tmp,
            repo,
            home,
        };
        let output = fixture.run(&["init"]);
        assert!(output.status.success(), "init: {}", describe(&output));
        fixture
    }

    fn command(&self) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_libra"));
        command
            .current_dir(&self.repo)
            .env("HOME", &self.home)
            .env("LIBRA_TEST_HOME", &self.home)
            .env_remove("CODEX_HOME");
        command
    }

    fn run(&self, args: &[&str]) -> Output {
        self.command().args(args).output().expect("run libra")
    }

    fn session_end_hook(&self, session_id: &str, transcript_path: &Path) -> Output {
        self.session_end_hook_from_cwd(&self.repo, session_id, transcript_path)
    }

    fn session_end_hook_from_cwd(
        &self,
        cwd: &Path,
        session_id: &str,
        transcript_path: &Path,
    ) -> Output {
        let envelope = json!({
            "hook_event_name": "SessionEnd",
            "session_id": session_id,
            "cwd": cwd.to_string_lossy(),
            "transcript_path": transcript_path.to_string_lossy(),
        })
        .to_string();
        let mut child = self
            .command()
            .args(["agent", "hooks", "claude-code", "session-end"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("spawn live hook writer");
        child
            .stdin
            .as_mut()
            .expect("hook stdin is piped")
            .write_all(envelope.as_bytes())
            .expect("write hook envelope");
        child.wait_with_output().expect("wait for hook writer")
    }

    fn transcript_path(&self, session_id: &str) -> PathBuf {
        self.home
            .join(".claude")
            .join("projects")
            .join("fixture")
            .join(format!("{session_id}.jsonl"))
    }

    fn discoverable_transcript_path_for_cwd(&self, session_id: &str, cwd: &Path) -> PathBuf {
        let slug = cwd
            .to_string_lossy()
            .chars()
            .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '-' })
            .collect::<String>();
        self.home
            .join(".claude")
            .join("projects")
            .join(slug)
            .join(format!("{session_id}.jsonl"))
    }

    fn discoverable_transcript_path(&self, session_id: &str) -> PathBuf {
        self.discoverable_transcript_path_for_cwd(session_id, &self.repo)
    }

    fn write_transcript(&self, session_id: &str, cwd: &Path, complete: bool) -> PathBuf {
        let path = self.transcript_path(session_id);
        std::fs::create_dir_all(path.parent().expect("transcript parent"))
            .expect("create transcript dir");
        let mut lines = vec![json!({
            "type": "user",
            "uuid": "turn-1",
            "sessionId": session_id,
            "cwd": cwd,
            "timestamp": "2026-07-15T01:00:00Z",
            "unknown_private": "AKIAAAAAAAAAAAAAAAAA",
            "message": {"role": "user", "content": "inspect the repo"}
        })];
        if complete {
            lines.push(json!({
                "type": "assistant",
                "uuid": "assistant-1",
                "sessionId": session_id,
                "cwd": cwd,
                "timestamp": "2026-07-15T01:00:01Z",
                "provider_private": "drop-this-field",
                "message": {"role": "assistant", "content": [{"type": "text", "text": "done"}]}
            }));
            lines.push(json!({
                "type": "session_end",
                "sessionId": session_id,
                "cwd": cwd,
                "timestamp": "2026-07-15T01:00:02Z"
            }));
        }
        let mut body = lines
            .into_iter()
            .map(|line| line.to_string())
            .collect::<Vec<_>>()
            .join("\n");
        if !complete {
            body.push_str(
                "\n{\"type\":\"assistant\",\"message\":{\"role\":\"assistant\",\"content\":[{\"type\":\"te",
            );
        }
        std::fs::write(&path, format!("{body}\n")).expect("write transcript");
        path
    }

    fn write_discoverable_transcript(&self, session_id: &str, cwd: &Path) -> PathBuf {
        self.write_discoverable_transcript_for_cwd(session_id, cwd)
    }

    fn write_discoverable_transcript_for_cwd(&self, session_id: &str, cwd: &Path) -> PathBuf {
        let source = self.write_transcript(session_id, cwd, true);
        let destination = self.discoverable_transcript_path_for_cwd(session_id, cwd);
        std::fs::create_dir_all(destination.parent().expect("discovery parent"))
            .expect("create discovery dir");
        std::fs::rename(source, &destination).expect("move transcript into discovery dir");
        destination
    }

    async fn scalar(&self, sql: &str) -> i64 {
        let url = format!(
            "sqlite://{}?mode=ro",
            self.repo.join(".libra/libra.db").display()
        );
        let conn = Database::connect(url).await.expect("open repo db");
        conn.query_one_raw(Statement::from_string(
            conn.get_database_backend(),
            sql.to_string(),
        ))
        .await
        .expect("query db")
        .expect("one row")
        .try_get_by("n")
        .expect("integer result")
    }

    async fn text_rows(&self, sql: &str, column: &str) -> Vec<String> {
        let url = format!(
            "sqlite://{}?mode=ro",
            self.repo.join(".libra/libra.db").display()
        );
        let conn = Database::connect(url).await.expect("open repo db");
        conn.query_all_raw(Statement::from_string(
            conn.get_database_backend(),
            sql.to_string(),
        ))
        .await
        .expect("query db")
        .into_iter()
        .map(|row| row.try_get_by(column).expect("text result"))
        .collect()
    }

    async fn repo_id(&self) -> String {
        self.text_rows(
            "SELECT value FROM config_kv WHERE key = 'libra.repoid' ORDER BY id DESC LIMIT 1",
            "value",
        )
        .await
        .into_iter()
        .next()
        .expect("initialized repository identity")
    }
}

fn describe(output: &Output) -> String {
    format!(
        "status={:?}\nstdout={}\nstderr={}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

/// Captured from the v0.30.46 descriptor helper at source commit
/// 077e66f75a5bf4bb793b95f904c82017fa1f0e86 (before OG-20). Reproduce
/// with that release's `libra --libra-internal-agent-import-preparation-descriptor-helper`,
/// `LIBRA_INTERNAL_IMPORT_PREPARATION_DESCRIPTOR_CONTROL` from
/// `empty_preparation_control`, and `empty_preparation_source` on stdin after
/// `libra init` in `ImportRepo`. The pre-change binary returned status=ok,
/// raw_bytes=699 in the recorded local fixture and the pinned content digest
/// below. Only values derived from the temporary repo path and source bytes
/// are substituted; field names, order, punctuation and fixed values are pinned.
const LEGACY_EMPTY_PREPARATION_PROJECTION: &str = r#"{"content_digest":"284e1514b809c031303fd684e91a92715b906cb8c6619e1993a5d3dde2eb0473","started_at":1784077200,"ended_at":1784077202,"session_state":"stopped","stopped_at":1784077202,"storage_commitment":@STORAGE_COMMITMENT@,"redaction_report":{"bytes_redacted":0,"bytes_scanned":@TOTAL_SCANNED@,"matches":[],"pipeline":"typed_allowlist","raw_persisted":false,"snapshot_redaction":true},"transcript_snapshot":{"completeness":"complete","partial_reason":null,"source":{"kind":"trusted_export","identity":"not_retained:v1","digest_sha256":"sha256:@SOURCE_SHA256@","byte_len":@SOURCE_LEN@},"transcript_redacted_bytes":@SOURCE_LEN@,"redaction_match_count":0,"redaction_bytes_scanned":@SOURCE_LEN@,"redaction_bytes_redacted":0},"turn_boundaries":{"turn-1":{"started_at":1784077200,"ended_at":1784077201}},"turns":[{"logical_turn_key":"turn-1","ordinal":0,"completeness":"complete","started_at":1784077200,"ended_at":1784077201,"records":[{"kind":"user","text":"inspect the repo"},{"kind":"assistant","text":"done"}]}]}"#;

const EMPTY_PREPARATION_SESSION: &str = "og20-empty-wire";

fn empty_preparation_source(fixture: &ImportRepo) -> Vec<u8> {
    let lines = [
        json!({
            "type": "user",
            "uuid": "turn-1",
            "sessionId": EMPTY_PREPARATION_SESSION,
            "cwd": fixture.repo,
            "timestamp": "2026-07-15T01:00:00Z",
            "message": {"role": "user", "content": "inspect the repo"}
        }),
        json!({
            "type": "assistant",
            "uuid": "assistant-1",
            "sessionId": EMPTY_PREPARATION_SESSION,
            "cwd": fixture.repo,
            "timestamp": "2026-07-15T01:00:01Z",
            "message": {"role": "assistant", "content": [{"type": "text", "text": "done"}]}
        }),
        json!({
            "type": "session_end",
            "sessionId": EMPTY_PREPARATION_SESSION,
            "cwd": fixture.repo,
            "timestamp": "2026-07-15T01:00:02Z"
        }),
    ];
    format!(
        "{}\n",
        lines
            .iter()
            .map(Value::to_string)
            .collect::<Vec<_>>()
            .join("\n")
    )
    .into_bytes()
}

fn empty_preparation_control(source_len: usize) -> String {
    // Freeze the pre-OG-20 domain and framing independently of production's
    // private commitment builder so a changed request cannot self-confirm.
    let mut commitment = Sha256::new();
    commitment.update(b"libra-agent-import-provider-commitment-v2\0");
    commitment.update((EMPTY_PREPARATION_SESSION.len() as u64).to_be_bytes());
    commitment.update(EMPTY_PREPARATION_SESSION.as_bytes());
    let provider_commitment: [u8; 32] = commitment.finalize().into();
    json!({
        "agent_kind": "claude-code",
        "source_kind": "file",
        "provider_commitment": provider_commitment,
        "read_cap": source_len + 1,
        "remaining_ms": 10_000,
    })
    .to_string()
}

fn run_real_empty_preparation_helper(fixture: &ImportRepo, source: &[u8]) -> Output {
    let mut child = fixture
        .command()
        .arg(libra::command::agent::IMPORT_PREPARATION_DESCRIPTOR_HELPER_ARG)
        .env(
            libra::command::agent::IMPORT_PREPARATION_DESCRIPTOR_CONTROL_ENV,
            empty_preparation_control(source.len()),
        )
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn real Libra descriptor helper");
    child
        .stdin
        .as_mut()
        .expect("helper stdin")
        .write_all(source)
        .expect("write held descriptor bytes");
    child
        .wait_with_output()
        .expect("wait for real descriptor helper")
}

fn legacy_empty_projection_bytes(fixture: &ImportRepo, source: &[u8]) -> Vec<u8> {
    let storage = fixture
        .repo
        .join(".libra")
        .canonicalize()
        .expect("canonical fixture storage root");
    let storage_text = storage.to_str().expect("UTF-8 fixture storage root");
    let mut commitment = Sha256::new();
    commitment.update(b"libra-agent-import-storage-commitment-v2\0");
    commitment.update((storage_text.len() as u64).to_be_bytes());
    commitment.update(storage_text.as_bytes());
    let storage_commitment: [u8; 32] = commitment.finalize().into();
    let storage_bytes = format!(
        "[{}]",
        storage_commitment
            .iter()
            .map(u8::to_string)
            .collect::<Vec<_>>()
            .join(",")
    );
    let source_sha256 = hex::encode(Sha256::digest(source));
    // The pre-change typed pass scanned 26 ordinary semantic bytes in this
    // fixture in addition to the exact authorized source byte count.
    LEGACY_EMPTY_PREPARATION_PROJECTION
        .replace("@STORAGE_COMMITMENT@", &storage_bytes)
        .replace("@TOTAL_SCANNED@", &(source.len() + 26).to_string())
        .replace("@SOURCE_SHA256@", &source_sha256)
        .replace("@SOURCE_LEN@", &source.len().to_string())
        .into_bytes()
}

#[test]
fn reasoning_empty_preparation_wire_is_unchanged() {
    let fixture = ImportRepo::init();
    let source = empty_preparation_source(&fixture);
    let output = run_real_empty_preparation_helper(&fixture, &source);
    assert!(output.status.success(), "{}", describe(&output));
    let projection = legacy_empty_projection_bytes(&fixture, &source);
    let mut expected = b"{\"status\":\"ok\",\"projection\":".to_vec();
    expected.extend_from_slice(&projection);
    expected.extend_from_slice(format!(",\"raw_bytes\":{}}}", source.len()).as_bytes());
    assert_eq!(
        output.stdout, expected,
        "legacy helper response bytes drifted"
    );
}

#[test]
fn reasoning_empty_preparation_projection_is_unchanged() {
    let fixture = ImportRepo::init();
    let source = empty_preparation_source(&fixture);
    let output = run_real_empty_preparation_helper(&fixture, &source);
    assert!(output.status.success(), "{}", describe(&output));
    let prefix = b"{\"status\":\"ok\",\"projection\":";
    let suffix = format!(",\"raw_bytes\":{}}}", source.len());
    assert!(output.stdout.starts_with(prefix));
    assert!(output.stdout.ends_with(suffix.as_bytes()));
    let projection = &output.stdout[prefix.len()..output.stdout.len() - suffix.len()];
    assert_eq!(
        projection,
        legacy_empty_projection_bytes(&fixture, &source),
        "legacy source projection bytes drifted"
    );
}

#[test]
fn reasoning_private_helper_control_frame_is_bounded() {
    let fixture = ImportRepo::init();
    let source = empty_preparation_source(&fixture);
    let mut control = empty_preparation_control(source.len());
    control.extend(std::iter::repeat_n(' ', 4 * 1024 + 1 - control.len()));
    assert_eq!(control.len(), 4 * 1024 + 1);
    let mut child = fixture
        .command()
        .arg(libra::command::agent::IMPORT_PREPARATION_DESCRIPTOR_HELPER_ARG)
        .env(
            libra::command::agent::IMPORT_PREPARATION_DESCRIPTOR_CONTROL_ENV,
            control,
        )
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn real Libra descriptor helper");
    // Hold stdin open and send no bytes: an oversized control must be rejected
    // before the helper touches its descriptor, with no response frame.
    let held_stdin = child.stdin.take().expect("held helper stdin");
    let deadline = Instant::now() + Duration::from_secs(5);
    let status = loop {
        if let Some(status) = child.try_wait().expect("poll real descriptor helper") {
            break status;
        }
        if Instant::now() >= deadline {
            child.kill().expect("stop hung real descriptor helper");
            let _ = child.wait();
            panic!("oversized control reached the descriptor read");
        }
        std::thread::sleep(Duration::from_millis(10));
    };
    drop(held_stdin);
    let mut stdout = Vec::new();
    child
        .stdout
        .take()
        .expect("helper stdout")
        .read_to_end(&mut stdout)
        .expect("read helper stdout");
    assert_eq!(status.code(), Some(2));
    assert!(stdout.is_empty(), "rejected control emitted a response");
}

/// Auxiliary forward-recovery evidence for the production-disabled native
/// path: a real pending checkpoint artifact survives both an unsupported
/// private control version and a subsequent ordinary import attempt.
/// A nonempty authenticated DTO cannot be injected into the production
/// parent while no provider native issuer is registered, so this is not the
/// full OG-20 VER18 candidate/transport acceptance gate.
#[cfg(unix)]
#[tokio::test]
async fn reasoning_private_helper_old_control_preserves_pending_artifact_evidence() {
    let fixture = ImportRepo::init();
    let session = "og20-retained-artifact";
    let transcript = fixture.write_discoverable_transcript(session, &fixture.repo);
    let run_hook = |event: &str, verb: &str, budget: bool| {
        let envelope = json!({
            "hook_event_name": event,
            "session_id": session,
            "cwd": fixture.repo,
            "transcript_path": transcript,
            "event_id": format!("og20-{verb}-v1"),
        })
        .to_string();
        let mut command = fixture.command();
        command.args(["agent", "hooks"]);
        if budget {
            command.args(["--capture-budget-ms", "60000"]);
        }
        let mut child = command
            .args(["claude-code", verb])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("spawn real hook handler");
        child
            .stdin
            .as_mut()
            .expect("hook stdin")
            .write_all(envelope.as_bytes())
            .expect("write hook envelope");
        child.wait_with_output().expect("wait for real hook")
    };
    let start = run_hook("SessionStart", "session-start", false);
    assert!(
        start.status.success(),
        "session start: {}",
        describe(&start)
    );

    // Force the real terminal hook to seal its source into the local pending
    // namespace, then fail only the checkpoint object publication.
    let objects = fixture.repo.join(".libra/objects");
    let backup = fixture.repo.join(".libra/objects-og20-backup");
    std::fs::rename(&objects, &backup).expect("move object store aside");
    std::fs::write(&objects, b"not an object directory").expect("block object publication");
    let terminal = run_hook("SessionEnd", "session-end", true);
    std::fs::remove_file(&objects).expect("remove object-directory blocker");
    std::fs::rename(&backup, &objects).expect("restore object store");
    assert!(
        !terminal.status.success(),
        "checkpoint object publication must fail: {}",
        describe(&terminal)
    );

    let evidence_query = "SELECT scope || ':' || target || ':' || key || ':' || \
                          hex(CAST(value AS BLOB)) AS evidence FROM metadata_kv \
                          WHERE scope IN ('agent_capture_pending', 'agent_capture_pending_chunk', \
                          'agent_capture_quarantine') ORDER BY scope, target, key";
    let retained = fixture.text_rows(evidence_query, "evidence").await;
    assert!(
        retained
            .iter()
            .any(|row| row.starts_with("agent_capture_pending:")),
        "the real hook must leave an authenticated pending artifact: {} rows={retained:?}",
        describe(&terminal)
    );
    assert!(
        retained
            .iter()
            .any(|row| row.starts_with("agent_capture_pending_chunk:")),
        "the pending artifact must retain its original byte chunks"
    );

    let ordinary_source = empty_preparation_source(&fixture);
    let old_response = run_real_empty_preparation_helper(&fixture, &ordinary_source);
    assert!(
        old_response.status.success(),
        "legacy empty helper response: {}",
        describe(&old_response)
    );
    assert!(
        old_response
            .stdout
            .starts_with(b"{\"status\":\"ok\",\"projection\":")
    );
    assert!(
        !old_response
            .stdout
            .windows(9)
            .any(|bytes| bytes == b"transport")
    );
    assert_eq!(
        fixture.text_rows(evidence_query, "evidence").await,
        retained
    );

    // Unsupported transport versions are rejected by the real hidden helper
    // before its held descriptor is read or any response can be emitted.
    let mut control: Value =
        serde_json::from_str(&empty_preparation_control(1)).expect("closed legacy control");
    control["transport"] = json!({
        "version": 2,
        "challenge": vec![0_u8; 32],
        "source_preimage_commitment": vec![0_u8; 32],
        "scope_digest": vec![0_u8; 32],
        "key": vec![1_u8; 32]
    });
    let mut child = fixture
        .command()
        .arg(libra::command::agent::IMPORT_PREPARATION_DESCRIPTOR_HELPER_ARG)
        .env(
            libra::command::agent::IMPORT_PREPARATION_DESCRIPTOR_CONTROL_ENV,
            control.to_string(),
        )
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn real descriptor helper with old transport version");
    let held_stdin = child.stdin.take().expect("held helper descriptor");
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if child
            .try_wait()
            .expect("poll rejected transport helper")
            .is_some()
        {
            break;
        }
        if Instant::now() >= deadline {
            child.kill().expect("stop hung transport helper");
            let _ = child.wait();
            panic!("unsupported transport version reached descriptor read");
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    drop(held_stdin);
    let rejected = child
        .wait_with_output()
        .expect("collect rejected transport helper");
    assert_eq!(rejected.status.code(), Some(2));
    assert!(
        rejected.stdout.is_empty(),
        "rejected helper emitted a frame"
    );
    assert_eq!(
        fixture.text_rows(evidence_query, "evidence").await,
        retained
    );

    // A separate real parent import attempt cannot retire the live artifact.
    let path = path_arg(&transcript);
    let import = fixture.run(&[
        "agent",
        "import",
        "--path",
        &path,
        "--agent",
        "claude-code",
        "--yes",
        "--json",
    ]);
    assert!(
        !import.status.success(),
        "retained live artifact should hold its original claim: {}",
        describe(&import)
    );
    assert_eq!(
        fixture.text_rows(evidence_query, "evidence").await,
        retained,
        "an ordinary helper replay must preserve the original pending artifact bytes"
    );
}

fn loose_object_file_count(root: &Path) -> usize {
    if !root.exists() {
        return 0;
    }
    walkdir::WalkDir::new(root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .count()
}

fn path_arg(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

/// Create a directory whose name is deliberately not valid UTF-8.
///
/// Returns `None` when the filesystem refuses the name outright (`EILSEQ`),
/// which is what stock macOS APFS/HFS+ does: those volumes only accept valid
/// UTF-8 filenames, so the byte sequence the caller wants to exercise cannot
/// exist there at all. Callers skip in that case rather than fail — the
/// behaviour under test is the binary's handling of such a path, not the
/// platform's willingness to create one.
#[cfg(unix)]
fn create_non_utf8_dir(path: &std::path::Path) -> Option<()> {
    match std::fs::create_dir_all(path) {
        Ok(()) => Some(()),
        Err(error) if error.raw_os_error() == Some(92) => None,
        Err(error) => panic!("create non-UTF-8 dir {}: {error}", path.display()),
    }
}

#[cfg(unix)]
#[test]
fn non_utf8_repository_is_rejected_with_stable_io_code_before_import() {
    use std::{ffi::OsString, os::unix::ffi::OsStringExt};

    let tmp = TempDir::new().expect("create non-UTF-8 import tempdir");
    let repo = tmp.path().join(OsString::from_vec(b"repo-\xff".to_vec()));
    if create_non_utf8_dir(&repo).is_none() {
        eprintln!("skipped (filesystem rejects non-UTF-8 directory names)");
        return;
    }
    let init = Command::new(env!("CARGO_BIN_EXE_libra"))
        .current_dir(&repo)
        .arg("init")
        .output()
        .expect("attempt non-UTF-8 repo initialization");
    assert!(
        !init.status.success(),
        "non-UTF-8 repo unexpectedly initialized"
    );
    assert!(
        String::from_utf8_lossy(&init.stderr).contains("LBR-IO-001"),
        "non-UTF-8 repo rejection lost its stable code: {}",
        describe(&init)
    );
}

#[cfg(unix)]
#[test]
fn agent_import_accepts_non_utf8_provider_root_via_lossless_helper_wire() {
    use std::{ffi::OsString, os::unix::ffi::OsStringExt};

    let tmp = TempDir::new().expect("create non-UTF-8 provider tempdir");
    let repo = tmp.path().join("repo");
    let home = tmp.path().join(OsString::from_vec(b"home-\xfe".to_vec()));
    std::fs::create_dir_all(&repo).expect("create repo");
    if create_non_utf8_dir(&home).is_none() {
        eprintln!("skipped (filesystem rejects non-UTF-8 directory names)");
        return;
    }
    let command = || {
        let mut command = Command::new(env!("CARGO_BIN_EXE_libra"));
        command
            .current_dir(&repo)
            .env("HOME", &home)
            .env("LIBRA_TEST_HOME", &home)
            .env_remove("CODEX_HOME");
        command
    };
    let init = command().arg("init").output().expect("initialize repo");
    assert!(init.status.success(), "init: {}", describe(&init));
    let project_slug = repo
        .to_string_lossy()
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '-' })
        .collect::<String>();
    let transcript = home
        .join(".claude/projects")
        .join(project_slug)
        .join("abc123.jsonl");
    std::fs::create_dir_all(transcript.parent().expect("transcript parent"))
        .expect("create provider transcript directory");
    let body = [
        json!({
            "type": "user", "uuid": "turn-1", "sessionId": "abc123",
            "cwd": repo, "message": {"role": "user", "content": "inspect"}
        }),
        json!({
            "type": "assistant", "uuid": "answer-1", "sessionId": "abc123",
            "cwd": repo,
            "message": {"role": "assistant", "content": [{"type":"text", "text":"done"}]}
        }),
        json!({"type": "session_end", "sessionId": "abc123", "cwd": repo}),
    ]
    .into_iter()
    .map(|line| line.to_string())
    .collect::<Vec<_>>()
    .join("\n");
    std::fs::write(&transcript, format!("{body}\n")).expect("write transcript");
    let output = command()
        .args([
            "agent",
            "import",
            "--session",
            "abc123",
            "--agent",
            "claude-code",
            "--yes",
            "--json",
        ])
        .output()
        .expect("run non-UTF-8 import");
    assert!(
        output.status.success(),
        "lossless helper wire rejected non-UTF-8 provider root: {}",
        describe(&output)
    );
}

#[test]
fn agent_import_explicit_session_accepts_safe_legacy_claude_identifier() {
    let fixture = ImportRepo::init();
    let session_id = "Legacy.session_01";
    let transcript = fixture.discoverable_transcript_path(session_id);
    std::fs::create_dir_all(transcript.parent().expect("transcript parent"))
        .expect("create provider transcript directory");
    let body = [
        json!({
            "type": "user", "uuid": "turn-1", "sessionId": session_id,
            "cwd": fixture.repo, "message": {"role": "user", "content": "inspect"}
        }),
        json!({
            "type": "assistant", "uuid": "answer-1", "sessionId": session_id,
            "cwd": fixture.repo,
            "message": {"role": "assistant", "content": [{"type":"text", "text":"done"}]}
        }),
        json!({"type": "session_end", "sessionId": session_id, "cwd": fixture.repo}),
    ]
    .into_iter()
    .map(|line| line.to_string())
    .collect::<Vec<_>>()
    .join("\n");
    std::fs::write(&transcript, format!("{body}\n")).expect("write legacy transcript");
    let output = fixture.run(&[
        "agent",
        "import",
        "--session",
        session_id,
        "--yes",
        "--json",
    ]);
    assert!(
        output.status.success(),
        "safe legacy explicit session id was accepted by CLI but rejected by resolver: {}",
        describe(&output)
    );
}

#[test]
fn agent_import_deduplicates_equivalent_transcript_working_directories() {
    let fixture = ImportRepo::init();
    let session_id = "cwd-alias-import";
    let transcript = fixture.discoverable_transcript_path(session_id);
    std::fs::create_dir_all(transcript.parent().expect("transcript parent"))
        .expect("create provider transcript directory");
    let alias = fixture.repo.join(".");
    let body = [
        json!({
            "type": "user", "uuid": "turn-1", "sessionId": session_id,
            "cwd": fixture.repo, "message": {"role": "user", "content": "inspect"}
        }),
        json!({
            "type": "assistant", "uuid": "answer-1", "sessionId": session_id,
            "cwd": alias,
            "message": {"role": "assistant", "content": [{"type":"text", "text":"done"}]}
        }),
        json!({"type": "session_end", "sessionId": session_id, "cwd": fixture.repo}),
    ]
    .into_iter()
    .map(|line| line.to_string())
    .collect::<Vec<_>>()
    .join("\n");
    std::fs::write(&transcript, format!("{body}\n")).expect("write alias transcript");

    let output = fixture.run(&[
        "agent",
        "import",
        "--session",
        session_id,
        "--agent",
        "claude-code",
        "--yes",
        "--json",
    ]);
    assert!(
        output.status.success(),
        "equivalent canonical transcript cwd spellings must import: {}",
        describe(&output)
    );
}

fn tree_entry_oid(tree: &[u8], wanted: &str) -> String {
    let mut cursor = 0usize;
    while cursor < tree.len() {
        let mode_end = tree[cursor..]
            .iter()
            .position(|byte| *byte == b' ')
            .map(|offset| cursor + offset)
            .expect("tree entry mode delimiter");
        let name_start = mode_end + 1;
        let name_end = tree[name_start..]
            .iter()
            .position(|byte| *byte == 0)
            .map(|offset| name_start + offset)
            .expect("tree entry name delimiter");
        let oid_start = name_end + 1;
        let oid_end = oid_start + 20;
        assert!(oid_end <= tree.len(), "tree entry object id is truncated");
        if &tree[name_start..name_end] == wanted.as_bytes() {
            return hex::encode(&tree[oid_start..oid_end]);
        }
        cursor = oid_end;
    }
    panic!("tree entry '{wanted}' not found");
}

fn read_object_payload(storage_root: &Path, oid: &str) -> Vec<u8> {
    let oid = ObjectHash::from_str(oid).expect("valid test object id");
    read_git_object(storage_root, &oid).expect("read test object")
}

fn read_checkpoint_blob(
    storage_root: &Path,
    root_tree_oid: &str,
    checkpoint_id: &str,
    relative_path: &[&str],
) -> Vec<u8> {
    let mut oid = root_tree_oid.to_string();
    for component in ["checkpoint", &checkpoint_id[..2], &checkpoint_id[2..]]
        .into_iter()
        .chain(relative_path.iter().copied())
    {
        let tree = read_object_payload(storage_root, &oid);
        oid = tree_entry_oid(&tree, component);
    }
    read_object_payload(storage_root, &oid)
}

#[tokio::test]
async fn agent_import_derives_working_dir_and_is_idempotent() {
    let fixture = ImportRepo::init();
    let transcript = fixture.write_transcript("abc123", &fixture.repo, true);
    let transcript = path_arg(&transcript);
    let args = [
        "agent",
        "import",
        "--path",
        transcript.as_str(),
        "--agent",
        "claude-code",
        "--yes",
        "--json",
    ];

    let first = fixture.run(&args);
    assert!(first.status.success(), "first import: {}", describe(&first));
    let json: Value = serde_json::from_slice(&first.stdout).expect("import JSON");
    assert_eq!(json["data"]["results"][0]["status"], "imported");
    assert_eq!(json["data"]["results"][0]["checkpoints_written"], 1);
    assert_eq!(
        fixture
            .scalar("SELECT COUNT(*) AS n FROM agent_session")
            .await,
        1
    );
    assert_eq!(
        fixture
            .scalar("SELECT COUNT(*) AS n FROM agent_checkpoint")
            .await,
        1
    );
    assert_eq!(
        fixture
            .scalar("SELECT COUNT(*) AS n FROM agent_import_identity")
            .await,
        1
    );
    assert_eq!(
        fixture
            .scalar("SELECT COUNT(*) AS n FROM agent_coverage_claim")
            .await,
        1
    );
    assert_eq!(
        fixture
            .text_rows("SELECT working_dir FROM agent_session", "working_dir")
            .await,
        vec![
            fixture
                .repo
                .canonicalize()
                .expect("canonical repo")
                .to_string_lossy()
                .into_owned(),
        ]
    );

    let second = fixture.run(&args);
    assert!(
        second.status.success(),
        "replay import: {}",
        describe(&second)
    );
    let json: Value = serde_json::from_slice(&second.stdout).expect("replay JSON");
    assert_eq!(json["data"]["results"][0]["status"], "noop");
    assert_eq!(json["data"]["results"][0]["checkpoints_written"], 0);
    assert_eq!(
        fixture
            .scalar("SELECT COUNT(*) AS n FROM agent_checkpoint")
            .await,
        1
    );

    let db = std::fs::read(fixture.repo.join(".libra/libra.db")).expect("read sqlite file");
    assert!(
        !db.windows("AKIAAAAAAAAAAAAAAAAA".len())
            .any(|window| window == b"AKIAAAAAAAAAAAAAAAAA"),
        "unknown provider fields and raw secrets must not persist"
    );
}

/// ACF-08: the post-consent importer must cross the same bounded snapshot
/// boundary as live capture before it normalizes or persists any source
/// evidence. The durable checkpoint carries only the safe projection, not a
/// path or raw transcript.
#[tokio::test]
async fn import_uses_capture_foundation_services() {
    let fixture = ImportRepo::init();
    let session_id = "capture-foundation-import";
    let transcript = fixture.write_transcript(session_id, &fixture.repo, true);
    let imported = fixture.run(&[
        "agent",
        "import",
        "--path",
        path_arg(&transcript).as_str(),
        "--agent",
        "claude-code",
        "--yes",
        "--json",
    ]);
    assert!(imported.status.success(), "import: {}", describe(&imported));

    let db = Database::connect(format!(
        "sqlite://{}?mode=ro",
        fixture.repo.join(".libra/libra.db").display()
    ))
    .await
    .expect("open capture-foundation db");
    let row = db
        .query_one_raw(Statement::from_string(
            db.get_database_backend(),
            "SELECT checkpoint_id, tree_oid FROM agent_checkpoint \
             WHERE session_id = 'claude__capture-foundation-import'"
                .to_string(),
        ))
        .await
        .expect("query import checkpoint")
        .expect("import checkpoint");
    let checkpoint_id: String = row.try_get_by("checkpoint_id").expect("checkpoint id");
    let tree_oid: String = row.try_get_by("tree_oid").expect("checkpoint tree");
    db.close().await.expect("close capture-foundation db");

    let metadata: Value = serde_json::from_slice(&read_checkpoint_blob(
        &fixture.repo.join(".libra"),
        &tree_oid,
        &checkpoint_id,
        &["metadata.json"],
    ))
    .expect("parse import checkpoint metadata");
    let snapshot = &metadata["transcript_snapshot"];
    assert_eq!(snapshot["completeness"], "complete");
    assert_eq!(snapshot["source"]["kind"], "trusted_export");
    assert!(
        snapshot["source"]["digest_sha256"]
            .as_str()
            .is_some_and(|digest| {
                digest.strip_prefix("source/hmac-v2/").is_some_and(|hex| {
                    hex.len() == 64
                        && hex
                            .bytes()
                            .all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f'))
                })
            }),
        "imported snapshot must retain only a repository-keyed HMAC v2 commitment: {snapshot}"
    );
    assert_eq!(
        snapshot["source"]["identity"], "not_retained:v1",
        "snapshot identity must not retain a correlating provider source key"
    );
    assert_eq!(metadata["redaction_report"]["snapshot_redaction"], true);
    let metadata_text = metadata.to_string();
    assert!(
        !metadata_text.contains("AKIAAAAAAAAAAAAAAAAA"),
        "the snapshot projection or report retained a raw source secret"
    );
}

/// Load the single checkpoint row a parity phase expects for `session_id`.
async fn sole_checkpoint(fixture: &ImportRepo, session_id: &str) -> (String, String) {
    let rows = fixture
        .text_rows(
            &format!(
                "SELECT checkpoint_id || ' ' || tree_oid AS row FROM agent_checkpoint \
                 WHERE session_id = '{session_id}'"
            ),
            "row",
        )
        .await;
    assert_eq!(rows.len(), 1, "expected exactly one checkpoint: {rows:?}");
    let (checkpoint_id, tree_oid) = rows[0].split_once(' ').expect("checkpoint id and tree oid");
    (checkpoint_id.to_string(), tree_oid.to_string())
}

/// The content-bearing roles of one checkpoint tree.
struct CheckpointContent {
    metadata: Value,
    transcript: Vec<u8>,
    redaction_report: Value,
}

fn checkpoint_content(
    fixture: &ImportRepo,
    (checkpoint_id, tree_oid): &(String, String),
) -> CheckpointContent {
    let storage = fixture.repo.join(".libra");
    let read = |path: &[&str]| read_checkpoint_blob(&storage, tree_oid, checkpoint_id, path);
    CheckpointContent {
        metadata: serde_json::from_slice(&read(&["metadata.json"]))
            .expect("parse checkpoint metadata"),
        transcript: read(&["transcript", "claude_code.jsonl"]),
        redaction_report: serde_json::from_slice(&read(&["redaction_report.json"]))
            .expect("parse checkpoint redaction report"),
    }
}

/// ACF-08 parity pin (AC3): identical transcript bytes captured live and
/// through `agent import` commit the same exact-source digest, snapshot
/// redaction projection and checkpoint content.
///
/// Phase 1 replays the import over the live capture: coverage recognizes the
/// turn, writes no second checkpoint, and the session-level projection keeps
/// the live digest.
///
/// Phase 2 erases the live session and imports the same bytes through the
/// audited restore path, so both entries write a checkpoint tree under one
/// repository key. Entry metadata (source kind, ids, timestamps, channel, the
/// lifecycle events and therefore `content_hash.txt`, which covers them) is
/// excluded; every content role is compared. The transcript role is typed by
/// provenance: live persists the exact redacted snapshot, import persists the
/// allowlisted per-turn projection derived from that same snapshot
/// (`CheckpointRedactedPayload::from_derived_turn_projection`). The import
/// blob must therefore be byte-for-byte the projection of the live blob, and
/// both entries must commit the same coverage-v1 content digest.
#[tokio::test]
async fn live_and_import_snapshot_hashes_match() {
    const SESSION: &str = "claude__capture-foundation-parity";
    let fixture = ImportRepo::init();
    let session_id = "capture-foundation-parity";
    let transcript = fixture.write_discoverable_transcript(session_id, &fixture.repo);
    let live = fixture.session_end_hook(session_id, &transcript);
    assert!(live.status.success(), "live hook: {}", describe(&live));

    let live_checkpoint = sole_checkpoint(&fixture, SESSION).await;
    let live_content = checkpoint_content(&fixture, &live_checkpoint);
    let live_snapshot = &live_content.metadata["transcript_snapshot"];
    assert_eq!(live_snapshot["source"]["kind"], "provider_file");
    assert_eq!(live_snapshot["completeness"], "complete");
    let live_claims = fixture
        .text_rows(
            "SELECT logical_turn_key || ' ' || coverage_digest || ' ' || completeness AS claim \
             FROM agent_coverage_claim WHERE state = 'catalog_committed' \
             ORDER BY logical_turn_key",
            "claim",
        )
        .await;
    assert!(
        !live_claims.is_empty(),
        "live capture committed no coverage"
    );

    let import_args = |restore: bool| {
        let mut args = vec![
            "agent".to_string(),
            "import".to_string(),
            "--path".to_string(),
            path_arg(&transcript),
            "--agent".to_string(),
            "claude-code".to_string(),
            "--yes".to_string(),
            "--json".to_string(),
        ];
        if restore {
            args.push("--restore-erased".to_string());
        }
        args
    };
    let covered = fixture
        .command()
        .args(import_args(false))
        .output()
        .expect("run covered import");
    assert!(
        covered.status.success(),
        "covered import: {}",
        describe(&covered)
    );
    assert_eq!(
        fixture
            .scalar("SELECT COUNT(*) AS n FROM agent_checkpoint")
            .await,
        1,
        "the import must reuse the live coverage claim rather than write another checkpoint"
    );
    let session_metadata = fixture
        .text_rows(
            &format!("SELECT metadata_json FROM agent_session WHERE session_id = '{SESSION}'"),
            "metadata_json",
        )
        .await
        .into_iter()
        .next()
        .expect("imported session metadata");
    let session_metadata: Value =
        serde_json::from_str(&session_metadata).expect("parse imported session metadata");
    let session_snapshot = &session_metadata["transcript_snapshot"];
    assert_eq!(session_snapshot["source"]["kind"], "trusted_export");
    for field in [
        "completeness",
        "transcript_redacted_bytes",
        "redaction_match_count",
        "redaction_bytes_scanned",
        "redaction_bytes_redacted",
    ] {
        assert_eq!(
            session_snapshot[field], live_snapshot[field],
            "snapshot projection field {field} drifted between live and import"
        );
    }
    assert_eq!(
        session_snapshot["source"]["digest_sha256"], live_snapshot["source"]["digest_sha256"],
        "the shared snapshot service must commit the same exact-source digest"
    );

    // Phase 2: remove the live evidence (checkpoint, catalog, coverage) and
    // import the same bytes again so the import writes its own tree.
    let conn = Database::connect(format!(
        "sqlite://{}",
        fixture.repo.join(".libra/libra.db").display()
    ))
    .await
    .expect("open writable repo db");
    let libra_dir = fixture.repo.join(".libra");
    let history = HistoryManager::new_with_ref(
        Arc::new(ClientStorage::try_init(libra_dir.join("objects")).expect("storage")),
        libra_dir,
        Arc::new(conn.clone()),
        TRACES_BRANCH,
    );
    let erased = history
        .erase_session_local(SESSION)
        .await
        .expect("erase live session");
    assert!(erased.session_deleted);
    drop(history);
    conn.close().await.expect("close writable repo db");

    let restored = fixture
        .command()
        .args(import_args(true))
        .output()
        .expect("run restored import");
    assert!(
        restored.status.success(),
        "restored import: {}",
        describe(&restored)
    );
    let restored_json: Value = serde_json::from_slice(&restored.stdout).expect("import JSON");
    assert_eq!(restored_json["data"]["results"][0]["status"], "imported");
    let import_checkpoint = sole_checkpoint(&fixture, SESSION).await;
    assert_ne!(
        import_checkpoint.0, live_checkpoint.0,
        "the restored import must write its own checkpoint tree"
    );
    let import_content = checkpoint_content(&fixture, &import_checkpoint);

    // Source digest and snapshot projection: only the entry's source kind
    // differs (provider file versus the importer's trusted handoff).
    let mut live_projection = live_snapshot.clone();
    let mut import_projection = import_content.metadata["transcript_snapshot"].clone();
    assert_eq!(import_projection["source"]["kind"], "trusted_export");
    for projection in [&mut live_projection, &mut import_projection] {
        projection["source"]
            .as_object_mut()
            .expect("snapshot source object")
            .remove("kind");
    }
    assert_eq!(
        import_projection, live_projection,
        "checkpoint snapshot projections (digest, sizes, redaction counts) drifted"
    );

    // Transcript role: the import blob is exactly the allowlisted projection
    // of the live exact-snapshot blob, turn by turn.
    let mut derived_turns = normalize_claude_transcript(&live_content.transcript);
    redact_turns(&mut derived_turns);
    assert_eq!(derived_turns.len(), 1, "fixture holds one logical turn");
    let mut derived_blob =
        serde_json::to_vec(&safe_turn_projection("claude_code", &derived_turns[0]))
            .expect("serialize derived projection");
    derived_blob.push(b'\n');
    assert!(
        import_content.transcript == derived_blob,
        "import transcript blob is not the projection of the live snapshot blob: import={} derived={}",
        String::from_utf8_lossy(&import_content.transcript),
        String::from_utf8_lossy(&derived_blob),
    );

    // Coverage-v1 content digest: both entries commit the digest of the turn
    // derived from the live checkpoint's transcript bytes.
    let derived_claims = derived_turns
        .iter()
        .map(|turn| format!("{} {} complete", turn.logical_turn_key, turn.digest_hex()))
        .collect::<Vec<_>>();
    assert_eq!(live_claims, derived_claims, "live coverage digest drifted");
    let import_claims = fixture
        .text_rows(
            "SELECT logical_turn_key || ' ' || coverage_digest || ' ' || completeness AS claim \
             FROM agent_coverage_claim WHERE state = 'catalog_committed' \
             AND source_channel = 'import' ORDER BY logical_turn_key",
            "claim",
        )
        .await;
    assert_eq!(import_claims, live_claims, "import coverage digest drifted");

    // Redaction report: the same snapshot-stage matches and redacted byte
    // count. Import additionally records its typed-field pass and pipeline
    // markers, so its scanned-byte total may only grow.
    for key in ["matches", "bytes_redacted"] {
        assert_eq!(
            import_content.redaction_report[key], live_content.redaction_report[key],
            "redaction report {key} drifted between live and import"
        );
        assert_eq!(
            import_content.metadata["redaction_report"][key],
            live_content.metadata["redaction_report"][key],
            "metadata redaction report {key} drifted between live and import"
        );
    }
    assert!(
        live_content.redaction_report["matches"]
            .as_array()
            .is_some_and(|matches| !matches.is_empty()),
        "the fixture secret must produce a snapshot redaction match"
    );
    let scanned = |report: &Value| report["bytes_scanned"].as_u64().expect("bytes scanned");
    assert_eq!(
        scanned(&live_content.redaction_report),
        live_snapshot["redaction_bytes_scanned"]
            .as_u64()
            .expect("snapshot bytes scanned")
    );
    assert!(scanned(&import_content.redaction_report) >= scanned(&live_content.redaction_report));
    let extra_keys = import_content
        .redaction_report
        .as_object()
        .expect("import report object")
        .keys()
        .filter(|key| live_content.redaction_report.get(key.as_str()).is_none())
        .cloned()
        .collect::<Vec<_>>();
    assert_eq!(
        extra_keys,
        ["pipeline", "raw_persisted", "snapshot_redaction"],
        "import redaction report gained an undocumented divergence"
    );
    for content in [&live_content, &import_content] {
        assert!(
            !String::from_utf8_lossy(&content.transcript).contains("AKIAAAAAAAAAAAAAAAAA"),
            "a checkpoint transcript retained the raw fixture secret"
        );
    }
}

#[tokio::test]
async fn agent_import_skips_live_covered_turn_and_merges_terminal_lifecycle() {
    let fixture = ImportRepo::init();
    let session_id = "live-covered-import";
    // Live capture deliberately ignores the envelope transcript pointer and
    // resolves only the provider-root-derived path. Use that discoverable
    // source so this is a real live/import coverage-dedup contract.
    let transcript = fixture.write_discoverable_transcript(session_id, &fixture.repo);
    let live = fixture.session_end_hook(session_id, &transcript);
    assert!(live.status.success(), "live hook: {}", describe(&live));
    assert_eq!(
        fixture
            .scalar("SELECT COUNT(*) AS n FROM agent_coverage_claim")
            .await,
        1,
        "the live side of a cross-channel dedup test must have an authorized coverage claim"
    );
    let live_coverage_digest = fixture
        .text_rows(
            "SELECT coverage_digest FROM agent_coverage_claim ORDER BY logical_turn_key",
            "coverage_digest",
        )
        .await;
    let transcript_arg = path_arg(&transcript);

    let imported = fixture.run(&[
        "agent",
        "import",
        "--path",
        transcript_arg.as_str(),
        "--agent",
        "claude-code",
        "--yes",
        "--json",
    ]);
    assert!(
        imported.status.success(),
        "covered import: {}",
        describe(&imported)
    );
    let body: Value = serde_json::from_slice(&imported.stdout).expect("covered import JSON");
    assert_eq!(body["data"]["results"][0]["status"], "noop");
    assert_eq!(body["data"]["results"][0]["checkpoints_written"], 0);
    assert_eq!(
        fixture
            .scalar("SELECT COUNT(*) AS n FROM agent_checkpoint")
            .await,
        1,
        "import must not duplicate a live-covered turn"
    );
    assert_eq!(
        fixture
            .scalar("SELECT COUNT(*) AS n FROM agent_coverage_revision")
            .await,
        1,
        "covered no-op must not append a duplicate revision"
    );
    assert_eq!(
        fixture
            .text_rows(
                "SELECT coverage_digest FROM agent_coverage_claim ORDER BY logical_turn_key",
                "coverage_digest",
            )
            .await,
        live_coverage_digest,
        "the import must compare against the same canonical redacted coverage digest as live"
    );
    assert_eq!(
        fixture
            .text_rows(
                "SELECT state FROM agent_session WHERE provider_session_id = 'live-covered-import'",
                "state"
            )
            .await,
        vec!["stopped"],
        "terminal no-op import must still merge its lifecycle state"
    );
}

#[tokio::test]
async fn agent_import_commit_before_live_hook_has_one_defined_revision() {
    let fixture = ImportRepo::init();
    let session_id = "import-before-live";
    // The subsequent live hook must read the same provider-root-derived
    // source that the explicit import consumed; an envelope pointer alone is
    // intentionally never an authorized live transcript capability.
    let transcript = fixture.write_discoverable_transcript(session_id, &fixture.repo);
    let imported = fixture.run(&[
        "agent",
        "import",
        "--path",
        path_arg(&transcript).as_str(),
        "--agent",
        "claude-code",
        "--yes",
        "--json",
    ]);
    assert!(imported.status.success(), "import: {}", describe(&imported));
    let import_coverage_digest = fixture
        .text_rows(
            "SELECT coverage_digest FROM agent_coverage_claim ORDER BY logical_turn_key",
            "coverage_digest",
        )
        .await;
    let import_stopped_at = fixture
        .text_rows(
            "SELECT CAST(stopped_at AS TEXT) AS stopped_at FROM agent_session \
             WHERE provider_session_id = 'import-before-live'",
            "stopped_at",
        )
        .await;

    let live = fixture.session_end_hook(session_id, &transcript);
    assert!(
        live.status.success(),
        "live replay after import: {}",
        describe(&live)
    );
    assert_eq!(
        fixture
            .scalar("SELECT COUNT(*) AS n FROM agent_checkpoint")
            .await,
        1,
        "live hook must recognize the import-covered turn"
    );
    assert_eq!(
        fixture
            .scalar("SELECT COUNT(*) AS n FROM agent_coverage_revision")
            .await,
        1,
        "import-first/live-second ordering must have one defined revision"
    );
    assert_eq!(
        fixture
            .text_rows(
                "SELECT coverage_digest FROM agent_coverage_claim ORDER BY logical_turn_key",
                "coverage_digest",
            )
            .await,
        import_coverage_digest,
        "the live hook must compare against the same canonical redacted coverage digest as import"
    );
    assert_eq!(
        fixture
            .text_rows(
                "SELECT state FROM agent_session WHERE provider_session_id = 'import-before-live'",
                "state"
            )
            .await,
        vec!["stopped"],
        "a covered live replay must not regress terminal session state"
    );
    assert_eq!(
        fixture
            .text_rows(
                "SELECT CAST(stopped_at AS TEXT) AS stopped_at FROM agent_session \
                 WHERE provider_session_id = 'import-before-live'",
                "stopped_at",
            )
            .await,
        import_stopped_at,
        "a covered live replay must not re-publish the terminal timestamp"
    );
    let session_metadata = fixture
        .text_rows(
            "SELECT metadata_json FROM agent_session \
             WHERE provider_session_id = 'import-before-live'",
            "metadata_json",
        )
        .await
        .into_iter()
        .next()
        .expect("captured session metadata");
    let metadata: Value = serde_json::from_str(&session_metadata).expect("parse session metadata");
    let receipts = metadata["capture_catalog_receipts_v1"]["entries"]
        .as_array()
        .expect("capture receipt ledger entries");
    assert!(
        receipts
            .iter()
            .all(|receipt| receipt["status"] == "complete"),
        "covered live replay must settle its catalog receipt rather than leave finalizer work pending"
    );
}

#[tokio::test]
async fn agent_import_same_digest_terminal_upgrade_writes_complete_schema_valid_revision() {
    let fixture = ImportRepo::init();
    let session_id = "terminalupgrade";
    let secret = "AKIAIOSFODNN7EXAMPLE";
    let transcript = fixture.transcript_path(session_id);
    std::fs::create_dir_all(transcript.parent().expect("transcript parent"))
        .expect("create transcript directory");
    let semantic_lines = [
        json!({
            "type": "user",
            "uuid": "upgrade-turn",
            "sessionId": session_id,
            "cwd": fixture.repo,
            "timestamp": "2026-07-15T01:00:00Z",
            "message": {"role": "user", "content": format!("inspect with {secret}")}
        }),
        json!({
            "type": "assistant",
            "uuid": "upgrade-answer",
            "sessionId": session_id,
            "cwd": fixture.repo,
            "timestamp": "2026-07-15T01:00:01Z",
            "message": {"role": "assistant", "content": [
                {"type": "text", "text": "done"},
                {"type": "tool_use", "id": "tool-secret-key", "name": "inspect",
                 "input": {(secret): "value"}}
            ]}
        }),
    ];
    std::fs::write(
        &transcript,
        format!(
            "{}\n",
            semantic_lines
                .iter()
                .map(Value::to_string)
                .collect::<Vec<_>>()
                .join("\n")
        ),
    )
    .expect("write growing transcript");
    let transcript_arg = path_arg(&transcript);
    let args = [
        "agent",
        "import",
        "--path",
        transcript_arg.as_str(),
        "--agent",
        "claude-code",
        "--yes",
        "--json",
    ];
    let first = fixture.run(&args);
    assert!(first.status.success(), "{}", describe(&first));
    assert_eq!(
        fixture
            .text_rows(
                "SELECT state FROM agent_session WHERE provider_session_id = 'terminalupgrade'",
                "state"
            )
            .await,
        vec!["active"]
    );

    writeln!(
        std::fs::OpenOptions::new()
            .append(true)
            .open(&transcript)
            .expect("open transcript for terminal evidence"),
        "{}",
        json!({
            "type": "session_end",
            "sessionId": session_id,
            "cwd": fixture.repo,
            "timestamp": "2026-07-15T01:00:02Z"
        })
    )
    .expect("append terminal evidence");
    let second = fixture.run(&args);
    assert!(second.status.success(), "{}", describe(&second));
    let second_json: Value = serde_json::from_slice(&second.stdout).expect("upgrade JSON");
    assert_eq!(second_json["data"]["results"][0]["status"], "imported");
    assert_eq!(second_json["data"]["results"][0]["checkpoints_written"], 1);
    assert_eq!(
        fixture
            .scalar("SELECT COUNT(*) AS n FROM agent_checkpoint")
            .await,
        2
    );
    assert_eq!(
        fixture
            .scalar("SELECT COUNT(*) AS n FROM agent_coverage_revision")
            .await,
        2
    );

    let db = Database::connect(format!(
        "sqlite://{}?mode=ro",
        fixture.repo.join(".libra/libra.db").display()
    ))
    .await
    .expect("open result db");
    let lifecycle = db
        .query_one_raw(Statement::from_string(
            db.get_database_backend(),
            "SELECT state, last_event_at, stopped_at FROM agent_session
             WHERE provider_session_id = 'terminalupgrade'"
                .to_string(),
        ))
        .await
        .expect("query lifecycle")
        .expect("lifecycle row");
    assert_eq!(
        lifecycle.try_get_by::<String, _>("state").unwrap(),
        "stopped"
    );
    assert_eq!(
        lifecycle.try_get_by::<i64, _>("last_event_at").unwrap(),
        1_784_077_202
    );
    assert_eq!(
        lifecycle.try_get_by::<i64, _>("stopped_at").unwrap(),
        1_784_077_202
    );
    let checkpoint = db
        .query_one_raw(Statement::from_string(
            db.get_database_backend(),
            "SELECT r.checkpoint_id, c.tree_oid
             FROM agent_coverage_revision r
             JOIN agent_checkpoint c ON c.checkpoint_id = r.checkpoint_id
             WHERE r.session_id = 'claude__terminalupgrade'
             ORDER BY r.revision DESC LIMIT 1"
                .to_string(),
        ))
        .await
        .expect("query upgraded checkpoint")
        .expect("upgraded checkpoint");
    let checkpoint_id: String = checkpoint.try_get_by("checkpoint_id").unwrap();
    let tree_oid: String = checkpoint.try_get_by("tree_oid").unwrap();
    db.close().await.expect("close result db");

    let storage = fixture.repo.join(".libra");
    let metadata = read_checkpoint_blob(&storage, &tree_oid, &checkpoint_id, &["metadata.json"]);
    let metadata: Value = serde_json::from_slice(&metadata).expect("metadata schema JSON");
    assert_eq!(metadata["schema_version"], 2);
    assert_eq!(metadata["model"], "unknown");
    assert_eq!(metadata["redaction_report"]["raw_persisted"], false);
    assert!(
        metadata["redaction_report"]["bytes_redacted"]
            .as_u64()
            .is_some_and(|bytes| bytes > 0)
    );

    let transcript_blob = read_checkpoint_blob(
        &storage,
        &tree_oid,
        &checkpoint_id,
        &["transcript", "claude_code.jsonl"],
    );
    assert!(
        !transcript_blob
            .windows(secret.len())
            .any(|window| window == secret.as_bytes())
    );
    let transcript_lines = transcript_blob
        .split(|byte| *byte == b'\n')
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();
    assert_eq!(transcript_lines.len(), 1);
    for line in transcript_lines {
        serde_json::from_slice::<Value>(line).expect("transcript line is JSON");
    }

    let lifecycle_blob = read_checkpoint_blob(
        &storage,
        &tree_oid,
        &checkpoint_id,
        &["events", "lifecycle.jsonl"],
    );
    for line in lifecycle_blob
        .split(|byte| *byte == b'\n')
        .filter(|line| !line.is_empty())
    {
        let event: Value = serde_json::from_slice(line).expect("lifecycle line is JSON");
        for key in [
            "schema_version",
            "event_id",
            "identity_scheme",
            "kind",
            "agent_kind",
            "session_id",
            "provider_session_id",
            "timestamp",
            "source",
            "partial",
            "provenance",
        ] {
            assert!(event.get(key).is_some(), "missing lifecycle key {key}");
        }
        uuid::Uuid::parse_str(event["event_id"].as_str().expect("event id string"))
            .expect("event id UUID");
        assert_eq!(event["schema_version"], 2);
        assert_eq!(event["identity_scheme"], "import_uuid_v5");
        assert_eq!(event["partial"], false);
    }

    let report = read_checkpoint_blob(
        &storage,
        &tree_oid,
        &checkpoint_id,
        &["redaction_report.json"],
    );
    let report: Value = serde_json::from_slice(&report).expect("redaction report JSON");
    assert!(
        report["bytes_redacted"]
            .as_u64()
            .is_some_and(|bytes| bytes > 0)
    );
    assert!(
        report["matches"]
            .as_array()
            .is_some_and(|matches| !matches.is_empty())
    );
    assert!(
        report["matches"].as_array().is_some_and(|matches| matches
            .iter()
            .any(|entry| entry["rule_id"] == "aws-access-key-id")),
        "tool-input object keys must contribute redaction evidence"
    );

    let manifest = read_checkpoint_blob(&storage, &tree_oid, &checkpoint_id, &["manifest.json"]);
    let manifest: Value = serde_json::from_slice(&manifest).expect("manifest JSON");
    assert_eq!(manifest["schema_version"], 1);
    assert_eq!(
        manifest["entries"]["transcript"]["media_type"],
        "application/x-ndjson"
    );
}

#[tokio::test]
async fn agent_import_preserves_three_turn_checkpoint_and_lifecycle_chronology() {
    let fixture = ImportRepo::init();
    let session_id = "chronology123";
    let transcript = fixture.transcript_path(session_id);
    std::fs::create_dir_all(transcript.parent().expect("transcript parent"))
        .expect("create transcript directory");
    let mut lines = Vec::new();
    for (index, minute) in [0_u32, 10, 20].into_iter().enumerate() {
        lines.push(json!({
            "type": "user",
            "uuid": format!("chronology-turn-{index}"),
            "sessionId": session_id,
            "cwd": fixture.repo,
            "timestamp": format!("2026-07-15T01:{minute:02}:00Z"),
            "message": {"role": "user", "content": format!("question {index}")}
        }));
        lines.push(json!({
            "type": "assistant",
            "uuid": format!("chronology-answer-{index}"),
            "sessionId": session_id,
            "cwd": fixture.repo,
            "timestamp": format!("2026-07-15T01:{minute:02}:01Z"),
            "message": {"role": "assistant", "content": format!("answer {index}")}
        }));
    }
    lines.push(json!({
        "type": "session_end",
        "sessionId": session_id,
        "cwd": fixture.repo,
        "timestamp": "2026-07-15T01:20:02Z"
    }));
    std::fs::write(
        &transcript,
        format!(
            "{}\n",
            lines
                .iter()
                .map(Value::to_string)
                .collect::<Vec<_>>()
                .join("\n")
        ),
    )
    .expect("write chronology transcript");
    let transcript_arg = path_arg(&transcript);
    let output = fixture.run(&[
        "agent",
        "import",
        "--path",
        transcript_arg.as_str(),
        "--agent",
        "claude-code",
        "--yes",
        "--json",
    ]);
    assert!(output.status.success(), "{}", describe(&output));
    let payload: Value = serde_json::from_slice(&output.stdout).expect("import JSON");
    assert_eq!(payload["data"]["results"][0]["checkpoints_written"], 3);

    let db = Database::connect(format!(
        "sqlite://{}?mode=ro",
        fixture.repo.join(".libra/libra.db").display()
    ))
    .await
    .expect("open chronology db");
    let rows = db
        .query_all_raw(Statement::from_string(
            db.get_database_backend(),
            "SELECT checkpoint_id, tree_oid, created_at FROM agent_checkpoint
             WHERE session_id = 'claude__chronology123'
             ORDER BY created_at ASC, checkpoint_id ASC"
                .to_string(),
        ))
        .await
        .expect("query checkpoint chronology");
    let expected_times = [1_784_077_201_i64, 1_784_077_801, 1_784_078_401];
    assert_eq!(rows.len(), expected_times.len());
    let mut ascending_ids = Vec::new();
    for (row, expected_time) in rows.iter().zip(expected_times) {
        let checkpoint_id: String = row.try_get_by("checkpoint_id").unwrap();
        let tree_oid: String = row.try_get_by("tree_oid").unwrap();
        assert_eq!(
            row.try_get_by::<i64, _>("created_at").unwrap(),
            expected_time
        );
        ascending_ids.push(checkpoint_id.clone());
        let metadata = read_checkpoint_blob(
            &fixture.repo.join(".libra"),
            &tree_oid,
            &checkpoint_id,
            &["metadata.json"],
        );
        let metadata: Value = serde_json::from_slice(&metadata).expect("metadata JSON");
        assert_eq!(metadata["created_at"], expected_time);
        assert_eq!(metadata["turn_ended_at"], expected_time);
        let lifecycle = read_checkpoint_blob(
            &fixture.repo.join(".libra"),
            &tree_oid,
            &checkpoint_id,
            &["events", "lifecycle.jsonl"],
        );
        let event: Value = serde_json::from_slice(
            lifecycle
                .split(|byte| *byte == b'\n')
                .find(|line| !line.is_empty())
                .expect("lifecycle line"),
        )
        .expect("lifecycle JSON");
        let actual = chrono::DateTime::parse_from_rfc3339(
            event["timestamp"].as_str().expect("timestamp string"),
        )
        .expect("RFC3339 lifecycle timestamp")
        .timestamp();
        assert_eq!(actual, expected_time);
    }
    db.close().await.expect("close chronology db");

    let list = fixture.run(&["agent", "checkpoint", "list", "--json"]);
    assert!(list.status.success(), "{}", describe(&list));
    let list: Value = serde_json::from_slice(&list.stdout).expect("checkpoint list JSON");
    let listed_ids = list["data"]["checkpoints"]
        .as_array()
        .expect("checkpoint list")
        .iter()
        .map(|row| row["checkpoint_id"].as_str().unwrap().to_string())
        .collect::<Vec<_>>();
    ascending_ids.reverse();
    assert_eq!(listed_ids, ascending_ids);
}

#[tokio::test]
async fn agent_import_normalizes_same_second_turns_in_public_checkpoint_order() {
    let fixture = ImportRepo::init();
    let session_id = "same-second-chronology";
    let transcript = fixture.transcript_path(session_id);
    std::fs::create_dir_all(transcript.parent().expect("transcript parent"))
        .expect("create transcript directory");
    let mut lines = Vec::new();
    for index in 0..3 {
        lines.push(json!({
            "type": "user",
            "uuid": format!("same-second-turn-{index}"),
            "sessionId": session_id,
            "cwd": fixture.repo,
            "timestamp": "2026-07-15T01:00:00.100Z",
            "message": {"role": "user", "content": format!("question {index}")}
        }));
        lines.push(json!({
            "type": "assistant",
            "uuid": format!("same-second-answer-{index}"),
            "sessionId": session_id,
            "cwd": fixture.repo,
            "timestamp": "2026-07-15T01:00:00.900Z",
            "message": {"role": "assistant", "content": format!("answer {index}")}
        }));
    }
    lines.push(json!({
        "type": "session_end",
        "sessionId": session_id,
        "cwd": fixture.repo,
        "timestamp": "2026-07-15T01:00:00.999Z"
    }));
    std::fs::write(
        &transcript,
        format!(
            "{}\n",
            lines
                .iter()
                .map(Value::to_string)
                .collect::<Vec<_>>()
                .join("\n")
        ),
    )
    .expect("write same-second transcript");
    let imported = fixture.run(&[
        "agent",
        "import",
        "--path",
        path_arg(&transcript).as_str(),
        "--agent",
        "claude-code",
        "--yes",
        "--json",
    ]);
    assert!(imported.status.success(), "{}", describe(&imported));

    let db = Database::connect(format!(
        "sqlite://{}?mode=ro",
        fixture.repo.join(".libra/libra.db").display()
    ))
    .await
    .expect("open same-second chronology db");
    let rows = db
        .query_all_raw(Statement::from_string(
            db.get_database_backend(),
            "SELECT checkpoint_id, created_at FROM agent_checkpoint
             WHERE session_id = 'claude__same-second-chronology'
             ORDER BY created_at ASC"
                .to_string(),
        ))
        .await
        .expect("query normalized chronology");
    assert_eq!(rows.len(), 3);
    let expected_times = [1_784_077_200_i64, 1_784_077_201, 1_784_077_202];
    let mut expected_public_ids = Vec::new();
    for (row, expected_time) in rows.iter().zip(expected_times) {
        assert_eq!(
            row.try_get_by::<i64, _>("created_at").unwrap(),
            expected_time
        );
        expected_public_ids.push(row.try_get_by::<String, _>("checkpoint_id").unwrap());
    }
    let lifecycle = db
        .query_one_raw(Statement::from_string(
            db.get_database_backend(),
            "SELECT last_event_at, stopped_at FROM agent_session
             WHERE session_id = 'claude__same-second-chronology'"
                .to_string(),
        ))
        .await
        .expect("query normalized session lifecycle")
        .expect("normalized session row");
    assert_eq!(
        lifecycle.try_get_by::<i64, _>("last_event_at").unwrap(),
        expected_times[2]
    );
    assert_eq!(
        lifecycle.try_get_by::<i64, _>("stopped_at").unwrap(),
        expected_times[2]
    );
    db.close().await.expect("close same-second chronology db");

    let listed = fixture.run(&["agent", "checkpoint", "list", "--json"]);
    assert!(listed.status.success(), "{}", describe(&listed));
    let listed: Value = serde_json::from_slice(&listed.stdout).expect("checkpoint list JSON");
    let listed_ids = listed["data"]["checkpoints"]
        .as_array()
        .expect("checkpoint list")
        .iter()
        .map(|row| row["checkpoint_id"].as_str().unwrap().to_string())
        .collect::<Vec<_>>();
    expected_public_ids.reverse();
    assert_eq!(listed_ids, expected_public_ids);
}

#[tokio::test]
async fn agent_import_duplicate_provider_turn_ids_do_not_silently_drop_turns() {
    let fixture = ImportRepo::init();
    let session_id = "duplicate-turn-ids";
    let transcript = fixture.transcript_path(session_id);
    std::fs::create_dir_all(transcript.parent().expect("transcript parent"))
        .expect("create transcript directory");
    let lines = [
        json!({"type":"user","uuid":"duplicate","sessionId":session_id,"cwd":fixture.repo,
            "timestamp":"2026-07-15T01:00:00Z","message":{"role":"user","content":"first"}}),
        json!({"type":"assistant","uuid":"answer-1","sessionId":session_id,"cwd":fixture.repo,
            "timestamp":"2026-07-15T01:00:01Z","message":{"role":"assistant","content":"one"}}),
        json!({"type":"user","uuid":"duplicate","sessionId":session_id,"cwd":fixture.repo,
            "timestamp":"2026-07-15T01:00:02Z","message":{"role":"user","content":"second"}}),
        json!({"type":"assistant","uuid":"answer-2","sessionId":session_id,"cwd":fixture.repo,
            "timestamp":"2026-07-15T01:00:03Z","message":{"role":"assistant","content":"two"}}),
        json!({"type":"session_end","sessionId":session_id,"cwd":fixture.repo,
            "timestamp":"2026-07-15T01:00:04Z"}),
    ];
    std::fs::write(
        &transcript,
        format!(
            "{}\n",
            lines
                .iter()
                .map(Value::to_string)
                .collect::<Vec<_>>()
                .join("\n")
        ),
    )
    .expect("write duplicate-ID transcript");
    let transcript_arg = path_arg(&transcript);
    let args = [
        "agent",
        "import",
        "--path",
        transcript_arg.as_str(),
        "--agent",
        "claude-code",
        "--yes",
        "--json",
    ];
    let imported = fixture.run(&args);
    assert!(imported.status.success(), "{}", describe(&imported));
    let payload: Value = serde_json::from_slice(&imported.stdout).expect("import JSON");
    assert_eq!(payload["data"]["results"][0]["status"], "imported");
    assert_eq!(payload["data"]["results"][0]["checkpoints_written"], 2);
    assert_eq!(
        fixture
            .text_rows(
                "SELECT logical_turn_key FROM agent_coverage_claim ORDER BY created_at, logical_turn_key",
                "logical_turn_key"
            )
            .await,
        vec!["duplicate", "ordinal:1"],
        "the later duplicate must receive a deterministic collision-free key"
    );
    let replay = fixture.run(&args);
    assert!(replay.status.success(), "{}", describe(&replay));
    assert_eq!(
        fixture
            .scalar("SELECT COUNT(*) AS n FROM agent_checkpoint")
            .await,
        2,
        "duplicate-ID replay must remain idempotent"
    );
}

#[tokio::test]
async fn import_reactivation_consumer_contract_v1() {
    let fixture = ImportRepo::init();
    let session_id = "resumed-after-end";
    let transcript = fixture.transcript_path(session_id);
    std::fs::create_dir_all(transcript.parent().expect("transcript parent"))
        .expect("create transcript directory");
    let terminal_lines = [
        json!({"type":"user","uuid":"old-turn","sessionId":session_id,"cwd":fixture.repo,
            "timestamp":"2026-07-15T01:00:00Z","message":{"role":"user","content":"old"}}),
        json!({"type":"assistant","sessionId":session_id,"cwd":fixture.repo,
            "timestamp":"2026-07-15T01:00:01Z","message":{"role":"assistant","content":"done"}}),
        json!({"type":"session_end","sessionId":session_id,"cwd":fixture.repo,
            "timestamp":"2026-07-15T01:00:02Z"}),
    ];
    std::fs::write(
        &transcript,
        format!(
            "{}\n",
            terminal_lines
                .iter()
                .map(Value::to_string)
                .collect::<Vec<_>>()
                .join("\n")
        ),
    )
    .expect("write initially terminal transcript");
    let transcript_arg = path_arg(&transcript);
    let args = [
        "agent",
        "import",
        "--path",
        transcript_arg.as_str(),
        "--agent",
        "claude-code",
        "--yes",
        "--json",
    ];
    let initial = fixture.run(&args);
    assert!(initial.status.success(), "{}", describe(&initial));
    assert_eq!(
        fixture
            .text_rows(
                "SELECT state FROM agent_session WHERE provider_session_id = 'resumed-after-end'",
                "state"
            )
            .await,
        vec!["stopped"],
        "the first import must establish the terminal lifecycle being reopened"
    );
    let terminal_sync_revision = fixture
        .scalar(
            "SELECT sync_revision AS n FROM agent_session \
             WHERE provider_session_id = 'resumed-after-end'",
        )
        .await;

    let mut resumed = std::fs::OpenOptions::new()
        .append(true)
        .open(&transcript)
        .expect("open transcript for resumed activity");
    for line in [
        json!({"type":"user","uuid":"new-turn","sessionId":session_id,"cwd":fixture.repo,
            "timestamp":"2026-07-15T01:01:00Z","message":{"role":"user","content":"resume"}}),
        json!({"type":"assistant","sessionId":session_id,"cwd":fixture.repo,
            "timestamp":"2026-07-15T01:01:01Z","message":{"role":"assistant","content":"still working"}}),
    ] {
        writeln!(resumed, "{line}").expect("append resumed activity");
    }
    drop(resumed);
    let imported = fixture.run(&args);
    assert!(imported.status.success(), "{}", describe(&imported));
    assert_eq!(
        fixture
            .text_rows(
                "SELECT state FROM agent_session WHERE provider_session_id = 'resumed-after-end'",
                "state"
            )
            .await,
        vec!["active"],
        "later semantic activity must clear an older terminal record"
    );
    assert_eq!(
        fixture
            .scalar(
                "SELECT COUNT(*) AS n FROM agent_session
                 WHERE provider_session_id = 'resumed-after-end' AND stopped_at IS NULL"
            )
            .await,
        1,
        "a reopened active session must clear its obsolete stopped timestamp"
    );
    assert!(
        fixture
            .scalar(
                "SELECT sync_revision AS n FROM agent_session \
                 WHERE provider_session_id = 'resumed-after-end'",
            )
            .await
            > terminal_sync_revision,
        "a newer historical nonterminal tail must advance the consumer revision"
    );
    assert_eq!(
        fixture
            .text_rows(
                "SELECT completeness FROM agent_coverage_claim WHERE logical_turn_key = 'new-turn'",
                "completeness"
            )
            .await,
        vec!["incomplete"],
        "the resumed tail remains upgradeable until newer terminal evidence"
    );
}

#[tokio::test]
async fn agent_import_enforces_configured_transcript_read_cap() {
    let fixture = ImportRepo::init();
    let configured = fixture.run(&["config", "set", "agent.max_transcript_read_bytes", "128"]);
    assert!(configured.status.success(), "{}", describe(&configured));
    let transcript = fixture.write_transcript("configured-cap", &fixture.repo, true);
    let transcript_arg = path_arg(&transcript);
    let output = fixture.run(&[
        "agent",
        "import",
        "--path",
        transcript_arg.as_str(),
        "--agent",
        "claude-code",
        "--yes",
        "--json",
    ]);
    assert!(!output.status.success(), "oversized import passed");
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("LBR-AGENT-018"),
        "unexpected failure: {}",
        describe(&output)
    );
    assert_eq!(
        fixture
            .scalar("SELECT COUNT(*) AS n FROM agent_checkpoint")
            .await,
        0
    );
}

#[tokio::test]
async fn agent_import_diagnoses_config_above_adapter_hard_cap() {
    let fixture = ImportRepo::init();
    let configured = fixture.run(&[
        "config",
        "set",
        "agent.max_transcript_read_bytes",
        "33554432",
    ]);
    assert!(configured.status.success(), "{}", describe(&configured));
    let transcript = fixture.write_transcript("configured-hard-cap", &fixture.repo, true);
    let transcript_arg = path_arg(&transcript);
    let output = fixture.run(&[
        "agent",
        "import",
        "--path",
        transcript_arg.as_str(),
        "--agent",
        "claude-code",
        "--yes",
        "--json",
    ]);
    assert!(output.status.success(), "{}", describe(&output));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("effective per-source cap is 16777216 bytes"),
        "configured hard-cap clamp was silent: {stderr}"
    );
}

#[tokio::test]
async fn agent_import_codex_rollout_e2e() {
    let fixture = ImportRepo::init();
    let session_id = "123e4567-e89b-12d3-a456-426614174000";
    let path = fixture
        .home
        .join(".codex/sessions/2026/07/15")
        .join(format!("rollout-2026-07-15T01-00-00-{session_id}.jsonl"));
    std::fs::create_dir_all(path.parent().expect("rollout parent")).expect("create rollout dir");
    let lines = [
        json!({
            "type": "session_meta",
            "timestamp": "2026-07-15T01:00:00Z",
            "payload": {"id": session_id, "cwd": fixture.repo}
        }),
        json!({
            "type": "response_item",
            "timestamp": "2026-07-15T01:00:01Z",
            "payload": {"type": "message", "role": "user", "id": "turn-codex-1",
                "content": [{"type": "input_text", "text": "inspect"}]}
        }),
        json!({
            "type": "response_item",
            "timestamp": "2026-07-15T01:00:02Z",
            "payload": {"type": "message", "role": "assistant", "id": "reply-codex-1",
                "content": [{"type": "output_text", "text": "done"}]}
        }),
        json!({
            "type": "session_end",
            "timestamp": "2026-07-15T01:00:03Z",
            "payload": {"type": "session_end", "cwd": fixture.repo}
        }),
    ];
    std::fs::write(
        &path,
        format!(
            "{}\n",
            lines
                .iter()
                .map(Value::to_string)
                .collect::<Vec<_>>()
                .join("\n")
        ),
    )
    .expect("write rollout");

    let output = fixture.run(&[
        "agent",
        "import",
        "--path",
        path_arg(&path).as_str(),
        "--agent",
        "codex",
        "--yes",
        "--json",
    ]);
    assert!(
        output.status.success(),
        "codex import: {}",
        describe(&output)
    );
    assert_eq!(
        fixture
            .scalar("SELECT COUNT(*) AS n FROM agent_checkpoint")
            .await,
        1
    );
    assert_eq!(
        fixture
            .text_rows(
                "SELECT source_channel FROM agent_coverage_claim",
                "source_channel"
            )
            .await,
        vec!["import"]
    );
}

#[tokio::test]
async fn agent_import_codex_malformed_arguments_upgrade_without_conflict() {
    let fixture = ImportRepo::init();
    let session_id = "123e4567-e89b-12d3-a456-426614174111";
    let path = fixture
        .home
        .join(".codex/sessions/2026/07/15")
        .join(format!("rollout-2026-07-15T01-00-00-{session_id}.jsonl"));
    std::fs::create_dir_all(path.parent().expect("rollout parent")).expect("create rollout dir");
    let write_rollout = |arguments: Value| {
        let lines = [
            json!({
                "type": "session_meta", "timestamp": "2026-07-15T01:00:00Z",
                "payload": {"id": session_id, "cwd": fixture.repo}
            }),
            json!({
                "type": "response_item", "timestamp": "2026-07-15T01:00:01Z",
                "payload": {"type": "message", "role": "user", "content": "inspect"}
            }),
            json!({
                "type": "response_item", "timestamp": "2026-07-15T01:00:02Z",
                "payload": {"type": "function_call", "call_id": "call-1",
                    "name": "inspect", "arguments": arguments}
            }),
            json!({
                "type": "session_end", "timestamp": "2026-07-15T01:00:03Z",
                "payload": {"type": "session_end", "cwd": fixture.repo}
            }),
        ];
        std::fs::write(
            &path,
            format!(
                "{}\n",
                lines
                    .iter()
                    .map(Value::to_string)
                    .collect::<Vec<_>>()
                    .join("\n")
            ),
        )
        .expect("write rollout");
    };
    let path = path_arg(&path);
    let args = [
        "agent",
        "import",
        "--path",
        path.as_str(),
        "--agent",
        "codex",
        "--yes",
        "--json",
    ];

    write_rollout(Value::String("{\"path\":\"truncated".to_string()));
    let first = fixture.run(&args);
    assert!(
        first.status.success(),
        "malformed import: {}",
        describe(&first)
    );
    assert_eq!(
        fixture
            .text_rows(
                "SELECT completeness FROM agent_coverage_claim",
                "completeness"
            )
            .await,
        vec!["incomplete"]
    );

    write_rollout(Value::String("{\"path\":\"Cargo.toml\"}".to_string()));
    let second = fixture.run(&args);
    assert!(
        second.status.success(),
        "corrected import: {}",
        describe(&second)
    );
    let db = Database::connect(format!(
        "sqlite://{}?mode=ro",
        fixture.repo.join(".libra/libra.db").display()
    ))
    .await
    .expect("open result db");
    let row = db
        .query_one_raw(Statement::from_string(
            db.get_database_backend(),
            "SELECT state, completeness FROM agent_coverage_claim".to_string(),
        ))
        .await
        .expect("query corrected claim")
        .expect("corrected claim");
    assert_eq!(
        row.try_get_by::<String, _>("state").unwrap(),
        "catalog_committed"
    );
    assert_eq!(
        row.try_get_by::<String, _>("completeness").unwrap(),
        "complete"
    );
    assert_eq!(
        fixture
            .scalar("SELECT COUNT(*) AS n FROM agent_coverage_revision")
            .await,
        2
    );
}

#[tokio::test]
async fn agent_import_reuses_same_repository_live_session_from_subdirectory() {
    let fixture = ImportRepo::init();
    let subdir = fixture.repo.join("nested/work");
    std::fs::create_dir_all(&subdir).expect("create repo subdir");
    let subdir = subdir.canonicalize().expect("canonical subdir");
    let transcript = fixture.write_discoverable_transcript_for_cwd("subdir123", &subdir);
    let repo_id = fixture.repo_id().await;
    let db_url = format!(
        "sqlite://{}",
        fixture.repo.join(".libra/libra.db").display()
    );
    let conn = Database::connect(db_url).await.expect("open repo db");
    conn.execute_raw(Statement::from_sql_and_values(
        conn.get_database_backend(),
        "INSERT INTO agent_session (
            session_id, agent_kind, provider_session_id, state, working_dir,
            metadata_json, redaction_report, started_at, last_event_at, schema_version,
            repo_id, worktree_id, workspace_id, workspace_fence, scope_state
         ) VALUES ('claude__subdir123', 'claude_code', 'subdir123', 'active', ?,
                   '{}', '{}', 1, 1, 1, ?, '', NULL, NULL, 'scoped')",
        [subdir.to_string_lossy().into_owned().into(), repo_id.into()],
    ))
    .await
    .expect("seed live session");
    conn.close().await.expect("close db");

    let transcript_arg = path_arg(&transcript);
    let args = [
        "agent",
        "import",
        "--path",
        transcript_arg.as_str(),
        "--agent",
        "claude-code",
        "--yes",
        "--json",
    ];
    let output = fixture.run(&args);
    assert!(
        output.status.success(),
        "subdir import: {}",
        describe(&output)
    );
    assert_eq!(
        fixture
            .text_rows("SELECT working_dir FROM agent_session", "working_dir")
            .await,
        vec![subdir.to_string_lossy().into_owned()]
    );

    let replay = fixture.run(&args);
    assert!(
        replay.status.success(),
        "subdir import replay: {}",
        describe(&replay)
    );

    let live = fixture.session_end_hook_from_cwd(&subdir, "subdir123", &transcript);
    assert!(
        live.status.success(),
        "live hook after subdirectory adoption: {}",
        describe(&live)
    );
    assert_eq!(
        fixture
            .text_rows("SELECT working_dir FROM agent_session", "working_dir")
            .await,
        vec![subdir.to_string_lossy().into_owned()],
        "historical import must retain the verified live cwd for later hooks"
    );
}

/// A matching repository storage identity permits a live session rooted in a
/// subdirectory, but no import may adopt the same provider id from a different
/// durable capture scope.  This protects the catalog transaction-bound import
/// prepare seam from becoming a weaker replacement for `CaptureScope`.
#[tokio::test]
async fn agent_import_refuses_mismatched_catalog_scope_and_fence() {
    let provider_session_id = "foreign-scope-import";
    for label in ["repo_id", "worktree_id", "scope_state", "workspace_fence"] {
        let fixture = ImportRepo::init();
        let transcript = fixture.write_transcript(provider_session_id, &fixture.repo, true);
        let repo_id = fixture.repo_id().await;
        let (case_repo_id, worktree_id, workspace_id, workspace_fence, scope_state): (
            String,
            String,
            Option<String>,
            Option<i64>,
            &str,
        ) = match label {
            "repo_id" => (
                "foreign-repo".to_string(),
                String::new(),
                None,
                None,
                "scoped",
            ),
            "worktree_id" => (
                repo_id.clone(),
                "foreign-worktree".to_string(),
                None,
                None,
                "scoped",
            ),
            "scope_state" => (repo_id.clone(), String::new(), None, None, "legacy_unknown"),
            "workspace_fence" => (
                repo_id.clone(),
                String::new(),
                Some("import-scope-workspace".to_string()),
                Some(9_i64),
                "scoped",
            ),
            _ => unreachable!("fixed scope-mismatch matrix"),
        };
        let conn = Database::connect(format!(
            "sqlite://{}",
            fixture.repo.join(".libra/libra.db").display()
        ))
        .await
        .expect("open repo db");
        if label == "workspace_fence" {
            let workspace_path = fixture
                .repo
                .canonicalize()
                .expect("canonical workspace path")
                .to_string_lossy()
                .into_owned();
            conn.execute_raw(Statement::from_sql_and_values(
                conn.get_database_backend(),
                "INSERT INTO workspace_record (
                    workspace_id, repo_id, kind, worktree_id, path, owner_kind,
                    owner_id, task_id, session_id, base_commit, branch, state,
                    lease_owner, lease_fence, lease_expires_at, created_at, updated_at
                 ) VALUES (?, ?, 'task_copy', NULL, ?, 'agent', 'scope-test',
                           NULL, NULL, NULL, NULL, 'active', 'scope-test-owner',
                           8, 9999999999999, 1, 1)",
                [
                    workspace_id
                        .clone()
                        .expect("fence case has a matching workspace id")
                        .into(),
                    repo_id.clone().into(),
                    workspace_path.into(),
                ],
            ))
            .await
            .expect("seed current workspace fence");
        }
        conn.execute_raw(Statement::from_sql_and_values(
            conn.get_database_backend(),
            "INSERT INTO agent_session (
                session_id, agent_kind, provider_session_id, state, working_dir,
                metadata_json, redaction_report, started_at, last_event_at, schema_version,
                repo_id, worktree_id, workspace_id, workspace_fence, scope_state
             ) VALUES (?, 'claude_code', ?, 'active', ?, '{}', '{}', 1, 1, 1,
                       ?, ?, ?, ?, ?)",
            [
                "claude__foreign-scope-import".into(),
                provider_session_id.into(),
                fixture.repo.to_string_lossy().into_owned().into(),
                case_repo_id.into(),
                worktree_id.into(),
                workspace_id.into(),
                workspace_fence.into(),
                scope_state.into(),
            ],
        ))
        .await
        .expect("seed foreign catalog scope");
        conn.close().await.expect("close seeded db");

        let output = fixture.run(&[
            "agent",
            "import",
            "--path",
            path_arg(&transcript).as_str(),
            "--agent",
            "claude-code",
            "--yes",
            "--json",
        ]);
        assert!(
            !output.status.success(),
            "import unexpectedly adopted {label} mismatch: {}",
            describe(&output)
        );
        assert_eq!(
            fixture
                .scalar("SELECT COUNT(*) AS n FROM agent_checkpoint")
                .await,
            0,
            "{label} mismatch must fail before a checkpoint is durable"
        );
        assert_eq!(
            fixture
                .scalar("SELECT COUNT(*) AS n FROM agent_import_identity")
                .await,
            0,
            "{label} mismatch must fail before an import lease is durable"
        );
    }
}

#[tokio::test]
async fn agent_import_preserves_newer_stopped_live_session_lifecycle() {
    let fixture = ImportRepo::init();
    let session_id = "stoppednewer";
    let transcript = fixture.transcript_path(session_id);
    std::fs::create_dir_all(transcript.parent().expect("transcript parent"))
        .expect("create transcript directory");
    let body = [
        json!({
            "type": "user",
            "uuid": "older-uncovered-turn",
            "sessionId": session_id,
            "cwd": fixture.repo,
            "timestamp": "2026-07-15T01:00:00Z",
            "message": {"role": "user", "content": "import older hole"}
        }),
        json!({
            "type": "assistant",
            "uuid": "older-uncovered-answer",
            "sessionId": session_id,
            "cwd": fixture.repo,
            "timestamp": "2026-07-15T01:00:01Z",
            "message": {"role": "assistant", "content": "done"}
        }),
    ]
    .into_iter()
    .map(|line| line.to_string())
    .collect::<Vec<_>>()
    .join("\n");
    std::fs::write(&transcript, format!("{body}\n")).expect("write nonterminal transcript");

    let newer_event_at = 9_999_999_999_000_i64;
    let newer_stopped_at = 9_999_999_999_500_i64;
    let repo_id = fixture.repo_id().await;
    let db_url = format!(
        "sqlite://{}",
        fixture.repo.join(".libra/libra.db").display()
    );
    let conn = Database::connect(db_url).await.expect("open repo db");
    conn.execute_raw(Statement::from_sql_and_values(
        conn.get_database_backend(),
        "INSERT INTO agent_session (
            session_id, agent_kind, provider_session_id, state, working_dir,
            metadata_json, redaction_report, started_at, last_event_at,
            stopped_at, schema_version, repo_id, worktree_id, workspace_id,
            workspace_fence, scope_state
         ) VALUES ('claude__stoppednewer', 'claude_code', ?, 'stopped', ?,
                   '{}', '{}', ?, ?, ?, 1, ?, '', NULL, NULL, 'scoped')",
        [
            session_id.into(),
            fixture.repo.to_string_lossy().into_owned().into(),
            newer_event_at.into(),
            newer_event_at.into(),
            newer_stopped_at.into(),
            repo_id.into(),
        ],
    ))
    .await
    .expect("seed newer stopped live session");
    conn.close().await.expect("close seed db");

    let output = fixture.run(&[
        "agent",
        "import",
        "--path",
        path_arg(&transcript).as_str(),
        "--agent",
        "claude-code",
        "--yes",
        "--json",
    ]);
    assert!(
        output.status.success(),
        "older-hole import: {}",
        describe(&output)
    );
    assert_eq!(
        fixture
            .scalar("SELECT COUNT(*) AS n FROM agent_checkpoint")
            .await,
        1,
        "uncovered historical turn must still be imported"
    );

    let conn = Database::connect(format!(
        "sqlite://{}?mode=ro",
        fixture.repo.join(".libra/libra.db").display()
    ))
    .await
    .expect("open result db");
    let row = conn
        .query_one_raw(Statement::from_string(
            conn.get_database_backend(),
            "SELECT state, started_at, last_event_at, stopped_at FROM agent_session
             WHERE session_id = 'claude__stoppednewer'"
                .to_string(),
        ))
        .await
        .expect("query imported session lifecycle")
        .expect("session row remains");
    assert_eq!(row.try_get_by::<String, _>("state").unwrap(), "stopped");
    assert_eq!(
        row.try_get_by::<i64, _>("started_at").unwrap(),
        1_784_077_200,
        "historical import must merge the earliest observed session start"
    );
    assert_eq!(
        row.try_get_by::<i64, _>("last_event_at").unwrap(),
        newer_event_at
    );
    assert_eq!(
        row.try_get_by::<i64, _>("stopped_at").unwrap(),
        newer_stopped_at
    );
}

#[tokio::test]
async fn agent_import_updates_digest_without_changing_structural_parent() {
    let fixture = ImportRepo::init();
    let transcript = fixture.write_transcript("revision123", &fixture.repo, false);
    let transcript_arg = path_arg(&transcript);
    let args = [
        "agent",
        "import",
        "--path",
        transcript_arg.as_str(),
        "--agent",
        "claude-code",
        "--yes",
        "--json",
    ];
    let first = fixture.run(&args);
    assert!(
        first.status.success(),
        "incomplete import: {}",
        describe(&first)
    );

    fixture.write_transcript("revision123", &fixture.repo, true);
    let second = fixture.run(&args);
    assert!(
        second.status.success(),
        "complete import: {}",
        describe(&second)
    );
    assert_eq!(
        fixture
            .scalar("SELECT COUNT(*) AS n FROM agent_coverage_revision")
            .await,
        2,
        "same logical turn keeps both source revisions"
    );
    let parents = fixture
        .text_rows(
            "SELECT COALESCE(parent_commit, '<root>') AS parent_commit \
             FROM agent_checkpoint ORDER BY created_at, checkpoint_id",
            "parent_commit",
        )
        .await;
    assert_eq!(parents.len(), 2);
    assert_eq!(
        parents[0], parents[1],
        "source revision must not alter the structural repository parent"
    );
}

#[tokio::test]
async fn agent_import_tombstone_blocks_resurrection_until_audited_restore() {
    let fixture = ImportRepo::init();
    let transcript = fixture.write_transcript("erase123", &fixture.repo, true);
    let transcript_arg = path_arg(&transcript);
    let base_args = [
        "agent",
        "import",
        "--path",
        transcript_arg.as_str(),
        "--agent",
        "claude-code",
        "--yes",
        "--json",
    ];
    let imported = fixture.run(&base_args);
    assert!(
        imported.status.success(),
        "initial import: {}",
        describe(&imported)
    );
    let imported_sync_revision = fixture
        .scalar(
            "SELECT sync_revision AS n FROM agent_session WHERE session_id = 'claude__erase123'",
        )
        .await;

    let db_url = format!(
        "sqlite://{}",
        fixture.repo.join(".libra/libra.db").display()
    );
    let conn = Database::connect(db_url)
        .await
        .expect("open writable repo db");
    let legacy_source_fingerprint = "a".repeat(64);
    conn.execute_raw(Statement::from_sql_and_values(
        conn.get_database_backend(),
        "UPDATE agent_session SET metadata_json = ? WHERE session_id = ?",
        [
            json!({"source_fingerprint": legacy_source_fingerprint})
                .to_string()
                .into(),
            "claude__erase123".into(),
        ],
    ))
    .await
    .expect("seed legacy unkeyed source fingerprint");
    let libra_dir = fixture.repo.join(".libra");
    let history = HistoryManager::new_with_ref(
        Arc::new(ClientStorage::try_init(libra_dir.join("objects")).expect("storage")),
        libra_dir,
        Arc::new(conn.clone()),
        TRACES_BRANCH,
    );
    let erased = history
        .erase_session_local("claude__erase123")
        .await
        .expect("erase imported session");
    assert!(erased.session_deleted);
    drop(history);

    assert_eq!(
        fixture
            .scalar("SELECT COUNT(*) AS n FROM agent_session")
            .await,
        0
    );
    assert_eq!(
        fixture
            .scalar("SELECT COUNT(*) AS n FROM agent_import_tombstone")
            .await,
        1
    );
    let tombstone = conn
        .query_one_raw(Statement::from_string(
            conn.get_database_backend(),
            "SELECT source_fingerprint FROM agent_import_tombstone \
             WHERE agent_kind = 'claude_code' AND provider_session_id = 'erase123'"
                .to_string(),
        ))
        .await
        .expect("query erased import tombstone")
        .expect("tombstone exists");
    let tombstone_fingerprint: Option<String> = tombstone
        .try_get_by("source_fingerprint")
        .expect("decode nullable tombstone fingerprint");
    assert_eq!(
        tombstone_fingerprint, None,
        "erasure must not copy a legacy raw/unkeyed source fingerprint from session metadata into a durable tombstone"
    );
    assert_eq!(
        fixture
            .scalar("SELECT COUNT(*) AS n FROM agent_import_identity")
            .await,
        0
    );
    assert_eq!(
        fixture
            .scalar(
                "SELECT next_session_sync_revision AS n FROM agent_capture_incarnation
                 WHERE agent_kind = 'claude_code' AND provider_session_id = 'erase123'",
            )
            .await,
        imported_sync_revision + 1,
        "erasure must preserve a strictly newer cloud replication epoch"
    );

    let blocked = fixture.run(&base_args);
    assert!(!blocked.status.success());
    assert!(
        String::from_utf8_lossy(&blocked.stderr).contains("LBR-AGENT-019"),
        "{}",
        describe(&blocked)
    );
    assert_eq!(
        fixture
            .scalar("SELECT COUNT(*) AS n FROM agent_session")
            .await,
        0
    );

    let restored = fixture.run(&[
        "agent",
        "import",
        "--path",
        transcript_arg.as_str(),
        "--agent",
        "claude-code",
        "--yes",
        "--restore-erased",
        "--json",
    ]);
    assert!(
        restored.status.success(),
        "restored import: {}",
        describe(&restored)
    );
    assert_eq!(
        fixture
            .scalar(
                "SELECT COUNT(*) AS n FROM agent_audit_log WHERE action = 'restore_erased_import'"
            )
            .await,
        1,
        "explicit restore is append-only audited"
    );
    assert!(
        fixture
            .scalar(
                "SELECT sync_revision AS n FROM agent_session
                 WHERE session_id = 'claude__erase123'",
            )
            .await
            > imported_sync_revision,
        "restored session must not reuse its erased cloud generation"
    );
    let incarnation = fixture
        .text_rows(
            "SELECT json_extract(metadata_json, '$.capture_incarnation') AS incarnation
             FROM agent_session WHERE session_id = 'claude__erase123'",
            "incarnation",
        )
        .await;
    assert_eq!(incarnation.len(), 1);
    assert_eq!(incarnation[0].len(), 32);
}

#[tokio::test]
async fn agent_import_tombstone_update_clears_legacy_source_fingerprint() {
    let fixture = ImportRepo::init();
    let db_url = format!(
        "sqlite://{}",
        fixture.repo.join(".libra/libra.db").display()
    );
    let conn = Database::connect(db_url).await.expect("open repo db");
    let legacy_source_fingerprint = "b".repeat(64);
    conn.execute_raw(Statement::from_sql_and_values(
        conn.get_database_backend(),
        "INSERT INTO agent_session (
            session_id, agent_kind, provider_session_id, state, working_dir,
            metadata_json, redaction_report, started_at, last_event_at, schema_version
         ) VALUES (?, 'claude_code', 'tombstone-update', 'stopped', ?, ?, '{}', 1, 1, 1)",
        [
            "claude__tombstone-update".into(),
            fixture.repo.to_string_lossy().into_owned().into(),
            json!({"source_fingerprint": legacy_source_fingerprint})
                .to_string()
                .into(),
        ],
    ))
    .await
    .expect("seed session with a legacy source fingerprint");
    conn.execute_raw(Statement::from_sql_and_values(
        conn.get_database_backend(),
        "INSERT INTO agent_import_tombstone (
            tombstone_id, agent_kind, provider_session_id, erased_session_id,
            source_fingerprint, erased_at
         ) VALUES (?, 'claude_code', 'tombstone-update', ?, ?, 1)",
        [
            "legacy-tombstone-update".into(),
            "claude__previous-erasure".into(),
            "c".repeat(64).into(),
        ],
    ))
    .await
    .expect("seed pre-existing legacy tombstone");

    let libra_dir = fixture.repo.join(".libra");
    let history = HistoryManager::new_with_ref(
        Arc::new(ClientStorage::try_init(libra_dir.join("objects")).expect("storage")),
        libra_dir,
        Arc::new(conn.clone()),
        TRACES_BRANCH,
    );
    let erased = history
        .erase_session_local("claude__tombstone-update")
        .await
        .expect("erase session through the existing tombstone");
    assert!(erased.session_deleted);
    drop(history);

    let row = conn
        .query_one_raw(Statement::from_string(
            conn.get_database_backend(),
            "SELECT erased_session_id, source_fingerprint
             FROM agent_import_tombstone
             WHERE agent_kind = 'claude_code' AND provider_session_id = 'tombstone-update'"
                .to_string(),
        ))
        .await
        .expect("query updated tombstone")
        .expect("updated tombstone exists");
    assert_eq!(
        row.try_get_by::<String, _>("erased_session_id")
            .expect("decode erased session id"),
        "claude__tombstone-update",
        "the existing anti-resurrection barrier must continue to track the newly erased session"
    );
    let fingerprint: Option<String> = row
        .try_get_by("source_fingerprint")
        .expect("decode nullable tombstone fingerprint");
    assert_eq!(
        fingerprint, None,
        "updating an existing tombstone must clear, not retain, a legacy unkeyed source fingerprint"
    );
}

#[tokio::test]
async fn agent_import_restore_refuses_while_erasure_is_unfinished() {
    let fixture = ImportRepo::init();
    let db_url = format!(
        "sqlite://{}",
        fixture.repo.join(".libra/libra.db").display()
    );
    let conn = Database::connect(db_url).await.expect("open repo db");
    conn.execute_raw(Statement::from_string(
        conn.get_database_backend(),
        "INSERT INTO agent_session (
            session_id, agent_kind, provider_session_id, state, working_dir,
            metadata_json, redaction_report, started_at, last_event_at, schema_version
         ) VALUES ('claude__erasing', 'claude_code', 'erasing', 'stopped',
                   '/tmp', '{}', '{}', 1, 1, 1)"
            .to_string(),
    ))
    .await
    .expect("seed erasing session");
    conn.execute_raw(Statement::from_string(
        conn.get_database_backend(),
        "INSERT INTO agent_import_tombstone (
            tombstone_id, agent_kind, provider_session_id, erased_session_id, erased_at
         ) VALUES ('t-erasing', 'claude_code', 'erasing', 'claude__erasing', 1)"
            .to_string(),
    ))
    .await
    .expect("seed tombstone");

    let error = restore_tombstone(&conn, AgentKind::ClaudeCode, "erasing")
        .await
        .expect_err("restore must wait for catalog deletion");
    assert!(error.to_string().contains("still being pruned"));
    let row = conn
        .query_one_raw(Statement::from_string(
            conn.get_database_backend(),
            "SELECT COUNT(*) AS n FROM agent_import_tombstone".to_string(),
        ))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(row.try_get_by::<i64, _>("n").unwrap(), 1);
}

#[tokio::test]
async fn import_of_a_shortened_source_releases_all_claim_leases() {
    let fixture = ImportRepo::init();
    let session_id = "shortened-source";
    let transcript = fixture.transcript_path(session_id);
    std::fs::create_dir_all(transcript.parent().expect("transcript parent"))
        .expect("create transcript directory");
    let source = [
        json!({
            "type": "user", "uuid": "shortened-turn",
            "sessionId": session_id, "cwd": fixture.repo,
            "message": {"role": "user", "content": "question"}
        }),
        json!({
            "type": "assistant", "uuid": "shortened-answer",
            "sessionId": session_id, "cwd": fixture.repo,
            "message": {"role": "assistant", "content": "answer"}
        }),
        json!({"type": "session_end", "sessionId": session_id, "cwd": fixture.repo}),
    ]
    .iter()
    .map(Value::to_string)
    .collect::<Vec<_>>()
    .join("\n");
    std::fs::write(&transcript, format!("{source}\n")).expect("write shortened transcript");
    let transcript_arg = path_arg(&transcript);
    let output = fixture.run(&[
        "agent",
        "import",
        "--path",
        transcript_arg.as_str(),
        "--agent",
        "claude-code",
        "--yes",
        "--json",
    ]);
    assert!(output.status.success(), "import: {}", describe(&output));
    assert_eq!(
        fixture
            .scalar(
                "SELECT COUNT(*) AS n FROM agent_coverage_claim \
                 WHERE state = 'reserved_import' OR owner IS NOT NULL"
            )
            .await,
        0,
        "a completed import must release every claim lease"
    );
}

#[tokio::test]
async fn doctor_retires_a_seeded_expired_object_writer_marker_without_deleting_objects() {
    let fixture = ImportRepo::init();
    let libra_dir = fixture.repo.join(".libra");
    let oid = libra::utils::object::write_git_object(&libra_dir, "blob", b"crash residue")
        .expect("write crash-residue fixture object")
        .to_string();
    let conn = Database::connect(format!("sqlite://{}", libra_dir.join("libra.db").display()))
        .await
        .expect("open writable repo db");
    let mut marker = TracesInflightMarker::new("claude__object-crash", "object-crash", 0);
    marker.ttl_ms = 0;
    marker.created_oids.push(oid.clone());
    marker.cleanup_pending = true;
    write_traces_inflight_marker(&conn, &marker)
        .await
        .expect("seed durable crash ownership marker");
    conn.close().await.expect("close seeded marker database");

    let repaired = fixture.run(&["agent", "doctor", "--repair", "--json"]);
    assert!(repaired.status.success(), "{}", describe(&repaired));
    assert_eq!(
        fixture
            .scalar("SELECT COUNT(*) AS n FROM metadata_kv WHERE scope = 'agent_traces_inflight'")
            .await,
        0,
        "doctor repair must retire stale ownership evidence"
    );
    let object_path = libra_dir.join("objects").join(&oid[..2]).join(&oid[2..]);
    assert!(
        object_path.exists(),
        "ownership retirement is non-destructive; repository GC owns loose object reclamation"
    );
}

#[cfg(unix)]
#[tokio::test]
async fn agent_doctor_retires_marker_without_reading_fifo_cleanup_root() {
    use std::{
        ffi::CString,
        os::unix::{ffi::OsStrExt as _, fs::FileTypeExt as _},
    };

    let fixture = ImportRepo::init();
    let db_url = format!(
        "sqlite://{}",
        fixture.repo.join(".libra/libra.db").display()
    );
    let conn = Database::connect(db_url)
        .await
        .expect("open writable FIFO cleanup database");
    let libra_dir = fixture.repo.join(".libra");
    let fifo_oid = ObjectHash::from_str("e69de29bb2d1d6434b8b29ae775ad8c2e48c5391")
        .expect("valid FIFO cleanup oid");
    let fifo_text = fifo_oid.to_string();
    let fifo_shard = libra_dir.join("objects").join(&fifo_text[..2]);
    std::fs::create_dir_all(&fifo_shard).expect("create FIFO cleanup shard");
    let fifo = fifo_shard.join(&fifo_text[2..]);
    let fifo_name = CString::new(fifo.as_os_str().as_bytes()).expect("FIFO path has no NUL");
    // SAFETY: fifo_name is NUL-terminated and lies in this test's temporary
    // repository.
    assert_eq!(unsafe { libc::mkfifo(fifo_name.as_ptr(), 0o600) }, 0);

    conn.execute_raw(Statement::from_sql_and_values(
        conn.get_database_backend(),
        "INSERT INTO reference (name, kind, `commit`, remote, worktree_id)
         VALUES ('fifo-cleanup-root', 'Branch', ?, NULL, NULL)",
        [fifo_text.clone().into()],
    ))
    .await
    .expect("register FIFO only as a cleanup graph root");
    let mut marker = TracesInflightMarker::new(
        "claude__fifo-cleanup",
        "fifo-cleanup-attempt",
        chrono::Utc::now().timestamp_millis(),
    );
    marker.cleanup_pending = true;
    marker.oids.push(fifo_text.clone());
    marker.created_oids.push(fifo_text);
    write_traces_inflight_marker(&conn, &marker)
        .await
        .expect("seed FIFO cleanup ownership");
    conn.close().await.expect("close FIFO cleanup database");

    let mut child = fixture
        .command()
        .args(["agent", "doctor", "--repair", "--json"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn FIFO cleanup doctor");
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        if child
            .try_wait()
            .expect("poll FIFO cleanup doctor")
            .is_some()
        {
            break;
        }
        if Instant::now() >= deadline {
            child.kill().expect("kill hung FIFO cleanup doctor");
            let output = child.wait_with_output().expect("reap hung FIFO doctor");
            panic!(
                "doctor hung on a special-file loose object: {}",
                describe(&output)
            );
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    let output = child
        .wait_with_output()
        .expect("collect FIFO cleanup doctor");
    assert!(output.status.success(), "{}", describe(&output));
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .contains("root-fenced ownership retirement; repository GC owns payload reachability"),
        "doctor did not report non-destructive ownership retirement: {}",
        describe(&output)
    );
    assert_eq!(
        fixture
            .scalar("SELECT COUNT(*) AS n FROM metadata_kv WHERE scope = 'agent_traces_inflight'")
            .await,
        0,
        "successful ownership retirement left the cleanup marker behind"
    );
    assert!(
        fifo.symlink_metadata()
            .expect("inspect FIFO after marker retirement")
            .file_type()
            .is_fifo(),
        "non-destructive marker retirement replaced or removed the FIFO payload"
    );
}

#[test]
fn agent_import_non_tty_requires_yes_before_content_read() {
    let fixture = ImportRepo::init();
    let nonexistent = fixture.transcript_path("missing");
    let output = fixture.run(&[
        "agent",
        "import",
        "--path",
        path_arg(&nonexistent).as_str(),
        "--agent",
        "claude-code",
        "--json",
    ]);
    assert!(!output.status.success(), "import unexpectedly succeeded");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("LBR-CLI-002"), "{}", describe(&output));
    assert!(stderr.contains("--yes"), "{}", describe(&output));
    assert!(
        !stderr.contains("source authorization"),
        "source must not be inspected before consent: {}",
        describe(&output)
    );
}

#[test]
#[ignore = "OpenCode Session capture is an operator-local manual gate"]
fn agent_import_confirmation_precedes_opencode_export() {
    let fixture = ImportRepo::init();
    let output = fixture.run(&[
        "agent",
        "import",
        "--session",
        "opencode123",
        "--agent",
        "opencode",
        "--json",
    ]);
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("LBR-CLI-002"), "{}", describe(&output));
    assert!(stderr.contains("--yes"), "{}", describe(&output));
    assert!(
        !stderr.contains("trusted") && !stderr.contains("exporter"),
        "export capability must not be probed before consent: {}",
        describe(&output)
    );
}

#[test]
fn agent_import_rejects_cross_repo_and_ambiguous_working_dir() {
    let fixture = ImportRepo::init();
    let other = fixture._tmp.path().join("other");
    std::fs::create_dir_all(&other).expect("create other repo");
    let mut init = Command::new(env!("CARGO_BIN_EXE_libra"));
    let init_output = init
        .current_dir(&other)
        .env("HOME", &fixture.home)
        .env("LIBRA_TEST_HOME", &fixture.home)
        .arg("init")
        .output()
        .expect("init other repo");
    assert!(
        init_output.status.success(),
        "other init: {}",
        describe(&init_output)
    );

    let cross = fixture.write_transcript("cross123", &other, true);
    let output = fixture.run(&[
        "agent",
        "import",
        "--path",
        path_arg(&cross).as_str(),
        "--agent",
        "claude-code",
        "--yes",
        "--json",
    ]);
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("LBR-AGENT-015"),
        "{}",
        describe(&output)
    );

    let ambiguous = fixture.write_transcript("ambiguous123", &fixture.repo, true);
    let second = json!({
        "type": "assistant",
        "uuid": "assistant-2",
        "sessionId": "ambiguous123",
        "cwd": other,
        "message": {"role": "assistant", "content": "different repo"}
    });
    writeln!(
        std::fs::OpenOptions::new()
            .append(true)
            .open(&ambiguous)
            .expect("open transcript"),
        "{second}"
    )
    .expect("append ambiguous cwd");
    let output = fixture.run(&[
        "agent",
        "import",
        "--path",
        path_arg(&ambiguous).as_str(),
        "--agent",
        "claude-code",
        "--yes",
        "--json",
    ]);
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("LBR-AGENT-016"),
        "{}",
        describe(&output)
    );
}

#[tokio::test]
async fn agent_import_accepts_sibling_linked_worktree_of_same_repository() {
    let fixture = ImportRepo::init();
    let sibling = fixture._tmp.path().join("sibling-worktree");
    let sibling_arg = path_arg(&sibling);
    let added = fixture.run(&[
        "worktree",
        "add",
        "sibling-worktree",
        "--path",
        sibling_arg.as_str(),
    ]);
    assert!(
        added.status.success(),
        "create sibling linked worktree: {}",
        describe(&added)
    );
    let transcript = fixture.write_transcript("siblingwt123", &sibling, true);
    let imported = fixture.run(&[
        "agent",
        "import",
        "--path",
        path_arg(&transcript).as_str(),
        "--agent",
        "claude-code",
        "--yes",
        "--json",
    ]);
    assert!(
        imported.status.success(),
        "same-storage sibling worktree was rejected: {}",
        describe(&imported)
    );
    assert_eq!(
        fixture
            .scalar("SELECT COUNT(*) AS n FROM agent_checkpoint")
            .await,
        1
    );
}

#[test]
fn agent_import_rejects_missing_working_dir_and_root_escape() {
    let fixture = ImportRepo::init();
    let missing = fixture.transcript_path("missingcwd123");
    std::fs::create_dir_all(missing.parent().expect("transcript parent"))
        .expect("create transcript dir");
    std::fs::write(
        &missing,
        json!({
            "type": "user",
            "uuid": "turn-1",
            "sessionId": "missingcwd123",
            "message": {"role": "user", "content": "no cwd"}
        })
        .to_string(),
    )
    .expect("write missing-cwd transcript");
    let output = fixture.run(&[
        "agent",
        "import",
        "--path",
        path_arg(&missing).as_str(),
        "--agent",
        "claude-code",
        "--yes",
        "--json",
    ]);
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("LBR-AGENT-016"),
        "{}",
        describe(&output)
    );

    let outside = fixture._tmp.path().join("outside123.jsonl");
    std::fs::write(&outside, "not authorized").expect("write outside source");
    let output = fixture.run(&[
        "agent",
        "import",
        "--path",
        path_arg(&outside).as_str(),
        "--agent",
        "claude-code",
        "--yes",
        "--json",
    ]);
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("LBR-AGENT-020"),
        "{}",
        describe(&output)
    );
}

#[test]
fn agent_import_rejects_filename_only_provider_identity() {
    let fixture = ImportRepo::init();
    let path = fixture.transcript_path("filenameonly123");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(
        &path,
        format!(
            "{}\n",
            json!({
                "type": "user",
                "uuid": "turn-1",
                "cwd": fixture.repo,
                "message": {"role": "user", "content": "missing provider id"}
            })
        ),
    )
    .unwrap();
    let output = fixture.run(&[
        "agent",
        "import",
        "--path",
        path_arg(&path).as_str(),
        "--agent",
        "claude-code",
        "--yes",
        "--json",
    ]);
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("LBR-AGENT-015"),
        "{}",
        describe(&output)
    );
}

#[cfg(unix)]
#[test]
fn agent_import_codex_discovery_rejects_symlinked_sessions_root_pre_consent() {
    let fixture = ImportRepo::init();
    let codex_home = fixture._tmp.path().join("codex-home");
    let outside = fixture._tmp.path().join("outside-sessions");
    let session_id = "123e4567-e89b-12d3-a456-426614174099";
    let day = outside.join("2026/07/15");
    std::fs::create_dir_all(&day).expect("create outside rollout directory");
    std::fs::create_dir_all(&codex_home).expect("create Codex home");
    let rollout = day.join(format!("rollout-2026-07-15T01-00-00-{session_id}.jsonl"));
    std::fs::write(
        &rollout,
        format!(
            "{}\n",
            json!({
                "type": "session_meta",
                "timestamp": "2026-07-15T01:00:00Z",
                "payload": {"id": session_id, "cwd": fixture.repo}
            })
        ),
    )
    .expect("write outside rollout");
    std::os::unix::fs::symlink(&outside, codex_home.join("sessions"))
        .expect("symlink sessions root");

    for selector in [vec!["--session", session_id], vec!["--all"]] {
        let mut command = fixture.command();
        command
            .env("CODEX_HOME", &codex_home)
            .args(["agent", "import"])
            .args(selector)
            .args(["--agent", "codex", "--yes", "--json"]);
        let output = command.output().expect("run Codex discovery import");
        assert!(!output.status.success(), "{}", describe(&output));
        assert!(
            String::from_utf8_lossy(&output.stderr).contains("LBR-AGENT-020"),
            "unexpected failure: {}",
            describe(&output)
        );
    }
    assert!(rollout.exists(), "outside rollout must remain untouched");
}

#[cfg(unix)]
#[test]
fn agent_import_codex_discovery_rejects_symlinked_nested_directory_pre_consent() {
    let fixture = ImportRepo::init();
    let codex_home = fixture._tmp.path().join("codex-home-nested");
    let sessions = codex_home.join("sessions");
    let outside_month = fixture._tmp.path().join("outside-month");
    let session_id = "123e4567-e89b-12d3-a456-426614174098";
    std::fs::create_dir_all(sessions.join("2026")).expect("create sessions year");
    std::fs::create_dir_all(outside_month.join("15")).expect("create outside month");
    let victim = outside_month
        .join("15")
        .join(format!("rollout-2026-07-15T01-00-00-{session_id}.jsonl"));
    std::fs::write(&victim, b"outside").expect("write outside victim");
    std::os::unix::fs::symlink(&outside_month, sessions.join("2026/07"))
        .expect("symlink month component");

    for selector in [vec!["--all"], vec!["--session", session_id]] {
        let output = fixture
            .command()
            .env("CODEX_HOME", &codex_home)
            .args(["agent", "import"])
            .args(selector)
            .args(["--agent", "codex", "--yes", "--json"])
            .output()
            .expect("run nested Codex discovery");
        assert!(!output.status.success(), "{}", describe(&output));
        assert!(
            String::from_utf8_lossy(&output.stderr).contains("LBR-AGENT-020"),
            "{}",
            describe(&output)
        );
    }
    assert_eq!(std::fs::read(&victim).unwrap(), b"outside");
}

#[test]
fn agent_import_codex_discovery_fanout_limit_fails_loudly() {
    let fixture = ImportRepo::init();
    let codex_home = fixture._tmp.path().join("codex-home-fanout");
    let sessions = codex_home.join("sessions");
    std::fs::create_dir_all(&sessions).expect("create Codex sessions root");
    for index in 0..=20_000 {
        std::fs::create_dir(sessions.join(format!("junk-{index:05}")))
            .expect("create Codex fanout entry");
    }

    let output = fixture
        .command()
        .env("CODEX_HOME", &codex_home)
        .args([
            "agent", "import", "--all", "--agent", "codex", "--yes", "--json",
        ])
        .output()
        .expect("run bounded Codex discovery");
    assert!(!output.status.success(), "{}", describe(&output));
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("LBR-AGENT-018"),
        "the discovery safety bound must fail loudly: {}",
        describe(&output)
    );
}

#[cfg(unix)]
#[test]
fn agent_import_claude_discovery_rejects_symlinked_source_pre_consent() {
    let fixture = ImportRepo::init();
    let discovered = fixture.discoverable_transcript_path("symlink-source");
    std::fs::create_dir_all(discovered.parent().expect("discovery parent"))
        .expect("create Claude discovery directory");
    let victim = fixture._tmp.path().join("outside-claude.jsonl");
    std::fs::write(&victim, b"outside").expect("write outside Claude source");
    std::os::unix::fs::symlink(&victim, &discovered).expect("symlink Claude source");

    let output = fixture.run(&[
        "agent",
        "import",
        "--all",
        "--agent",
        "claude-code",
        "--yes",
        "--json",
    ]);
    assert!(!output.status.success(), "{}", describe(&output));
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("LBR-AGENT-020"),
        "{}",
        describe(&output)
    );
    assert_eq!(std::fs::read(&victim).unwrap(), b"outside");
}

#[tokio::test]
async fn agent_import_batch_limit_cursor_filter_and_partial_contract() {
    let fixture = ImportRepo::init();
    for session_id in ["batch001", "batch002", "batch003"] {
        fixture.write_discoverable_transcript(session_id, &fixture.repo);
    }
    let first = fixture.run(&[
        "agent",
        "import",
        "--since",
        "2026-07-01T00:00:00Z",
        "--agent",
        "claude-code",
        "--limit",
        "2",
        "--yes",
        "--json",
    ]);
    assert!(first.status.success(), "first page: {}", describe(&first));
    let first_json: Value = serde_json::from_slice(&first.stdout).expect("first page JSON");
    assert_eq!(
        first_json["data"]["results"].as_array().map(Vec::len),
        Some(2)
    );
    assert_eq!(first_json["data"]["next_cursor"], 2);

    let second = fixture.run(&[
        "agent",
        "import",
        "--all",
        "--agent",
        "claude-code",
        "--limit",
        "2",
        "--cursor",
        "2",
        "--yes",
        "--json",
    ]);
    assert!(
        second.status.success(),
        "second page: {}",
        describe(&second)
    );
    let second_json: Value = serde_json::from_slice(&second.stdout).expect("second page JSON");
    assert_eq!(
        second_json["data"]["results"].as_array().map(Vec::len),
        Some(1)
    );
    assert!(second_json["data"]["next_cursor"].is_null());
    assert_eq!(
        fixture
            .scalar("SELECT COUNT(*) AS n FROM agent_session")
            .await,
        3
    );

    let invalid = fixture.write_discoverable_transcript("batchbad", &fixture.repo);
    writeln!(
        std::fs::OpenOptions::new()
            .append(true)
            .open(&invalid)
            .expect("open invalid batch transcript"),
        "{}",
        json!({
            "type": "assistant",
            "sessionId": "batchbad",
            "cwd": fixture._tmp.path().join("not-a-repo"),
            "message": {"role": "assistant", "content": "ambiguous cwd"}
        })
    )
    .expect("append invalid batch cwd");
    fixture.write_discoverable_transcript("batchgood", &fixture.repo);
    let partial = fixture.run(&[
        "agent",
        "import",
        "--all",
        "--agent",
        "claude-code",
        "--limit",
        "50",
        "--yes",
        "--json",
    ]);
    assert!(!partial.status.success());
    let stderr = String::from_utf8_lossy(&partial.stderr);
    assert!(stderr.contains("LBR-AGENT-018"), "{}", describe(&partial));
    assert!(stderr.contains("LBR-AGENT-016"), "{}", describe(&partial));
    assert!(stderr.contains("\"results\""), "{}", describe(&partial));
    assert!(
        stderr.contains("\"schema_version\": 1"),
        "{}",
        describe(&partial)
    );
    assert!(
        stderr.contains("\"next_cursor\": null"),
        "{}",
        describe(&partial)
    );
    // SHA-256("batchbad")[..6]: the documented short hashed session id.
    assert!(
        stderr.contains("\"session_id\": \"sha256:33cec74cbb24\""),
        "{}",
        describe(&partial)
    );
    assert!(
        !stderr.contains("source/hmac-v2") && !stderr.contains("\"session_id\": \"batchbad\""),
        "batch failures must not expose a source commitment or raw session id: {}",
        describe(&partial)
    );
    assert!(!stderr.contains("\"session_id\": \"partial\""));
}

#[tokio::test]
async fn agent_import_codex_batch_reports_cross_repository_candidates_as_skipped() {
    let fixture = ImportRepo::init();
    let other = fixture._tmp.path().join("other-repo");
    std::fs::create_dir_all(&other).expect("create other repo");
    let init = fixture
        .command()
        .current_dir(&other)
        .arg("init")
        .output()
        .expect("init other repo");
    assert!(init.status.success(), "{}", describe(&init));

    let codex_home = fixture._tmp.path().join("codex-home");
    let day = codex_home.join("sessions/2026/07/15");
    std::fs::create_dir_all(&day).expect("create Codex day partition");
    for (session_id, cwd, minute) in [
        (
            "123e4567-e89b-12d3-a456-426614174010",
            fixture.repo.as_path(),
            "00",
        ),
        (
            "123e4567-e89b-12d3-a456-426614174011",
            other.as_path(),
            "01",
        ),
    ] {
        let lines = [
            json!({
                "type": "session_meta",
                "timestamp": format!("2026-07-15T01:{minute}:00Z"),
                "payload": {"id": session_id, "cwd": cwd}
            }),
            json!({
                "type": "response_item",
                "timestamp": format!("2026-07-15T01:{minute}:01Z"),
                "payload": {"type": "message", "role": "user", "id": format!("turn-{session_id}"),
                    "content": [{"type": "input_text", "text": "inspect"}]}
            }),
            json!({
                "type": "response_item",
                "timestamp": format!("2026-07-15T01:{minute}:02Z"),
                "payload": {"type": "message", "role": "assistant", "id": format!("reply-{session_id}"),
                    "content": [{"type": "output_text", "text": "done"}]}
            }),
            json!({
                "type": "session_end",
                "timestamp": format!("2026-07-15T01:{minute}:03Z"),
                "payload": {"type": "session_end", "cwd": cwd}
            }),
        ];
        std::fs::write(
            day.join(format!(
                "rollout-2026-07-15T01-{minute}-00-{session_id}.jsonl"
            )),
            format!(
                "{}\n",
                lines
                    .iter()
                    .map(Value::to_string)
                    .collect::<Vec<_>>()
                    .join("\n")
            ),
        )
        .expect("write Codex rollout");
    }

    let output = fixture
        .command()
        .env("CODEX_HOME", &codex_home)
        .args([
            "agent", "import", "--all", "--agent", "codex", "--yes", "--json",
        ])
        .output()
        .expect("run mixed Codex batch");
    assert!(output.status.success(), "{}", describe(&output));
    let payload: Value = serde_json::from_slice(&output.stdout).expect("batch JSON");
    assert_eq!(payload["data"]["results"].as_array().map(Vec::len), Some(1));
    assert_eq!(payload["data"]["results"][0]["status"], "imported");
    assert_eq!(payload["data"]["skipped"].as_array().map(Vec::len), Some(1));
    assert_eq!(payload["data"]["skipped"][0]["status"], "skipped");
    assert_eq!(
        payload["data"]["skipped"][0]["reason_code"],
        "LBR-AGENT-015"
    );
    // SHA-256("123e4567-e89b-12d3-a456-426614174011")[..6].
    assert_eq!(
        payload["data"]["skipped"][0]["session_id"],
        "sha256:278a7e727b81"
    );
    assert_eq!(
        payload["data"]["failures"].as_array().map(Vec::len),
        Some(0)
    );
}

/// Discovery runs in a private helper; argv errors are rejected by the parent
/// and provider errors cross the wire only as a closed reason, but each must
/// still render the shipped actionable message, stable code, and exit status.
#[test]
fn agent_import_discovery_errors_keep_actionable_messages() {
    let fixture = ImportRepo::init();
    // A real Claude project root, so a missing id is "not found" rather than
    // an absent provider.
    fixture.write_discoverable_transcript("present", &fixture.repo);
    for (args, message, code, exit) in [
        (
            &[
                "--all",
                "--agent",
                "claude-code",
                "--limit",
                "0",
                "--yes",
                "--json",
            ][..],
            "--limit must be between 1 and 100",
            "LBR-CLI-002",
            129,
        ),
        (
            &["--all", "--agent", "gemini", "--yes"][..],
            "agent import supports claude-code, codex, opencode, or hermes; got 'gemini'",
            "LBR-CLI-002",
            129,
        ),
        (
            &["--path", "x.jsonl", "--yes"][..],
            "--path requires --agent",
            "LBR-CLI-002",
            129,
        ),
        (
            &["--all", "--agent", "opencode", "--yes"][..],
            "OpenCode batch discovery is unavailable; select a session explicitly with --session",
            "LBR-CLI-002",
            129,
        ),
        (
            &[
                "--session",
                "missing123",
                "--agent",
                "claude-code",
                "--yes",
                "--json",
            ][..],
            "no authorized local transcript matched the session id; use --agent opencode for an export-only OpenCode session",
            "LBR-CLI-003",
            129,
        ),
        (
            &["--all", "--agent", "claude-code", "--cursor", "5", "--yes"][..],
            "--cursor is outside the discovery result set",
            "LBR-CLI-002",
            129,
        ),
    ] {
        let output = fixture
            .command()
            .args(["agent", "import"])
            .args(args)
            .output()
            .expect("run agent import");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert_eq!(
            output.status.code(),
            Some(exit),
            "{args:?}: {}",
            describe(&output)
        );
        assert!(stderr.contains(message), "{args:?}: {}", describe(&output));
        assert!(stderr.contains(code), "{args:?}: {}", describe(&output));
        assert!(
            !stderr.contains("rejected the supplied selector")
                && !stderr.contains("rejected the selected target"),
            "{args:?}: {}",
            describe(&output)
        );
    }

    // The same UUID as a Claude transcript and a Codex rollout is ambiguous
    // without --agent.
    let session_id = "123e4567-e89b-12d3-a456-426614174099";
    fixture.write_discoverable_transcript(session_id, &fixture.repo);
    let codex_home = fixture._tmp.path().join("codex-home");
    let day = codex_home.join("sessions/2026/07/15");
    std::fs::create_dir_all(&day).expect("create Codex day partition");
    std::fs::write(
        day.join(format!("rollout-2026-07-15T01-00-00-{session_id}.jsonl")),
        format!(
            "{}\n",
            json!({
                "type": "session_meta",
                "timestamp": "2026-07-15T01:00:00Z",
                "payload": {"id": session_id, "cwd": fixture.repo}
            })
        ),
    )
    .expect("write Codex rollout");
    let ambiguous = fixture
        .command()
        .env("CODEX_HOME", &codex_home)
        .args(["agent", "import", "--session", session_id, "--yes"])
        .output()
        .expect("run ambiguous agent import");
    let stderr = String::from_utf8_lossy(&ambiguous.stderr);
    assert_eq!(
        ambiguous.status.code(),
        Some(129),
        "{}",
        describe(&ambiguous)
    );
    assert!(
        stderr.contains("the session id matches multiple providers; add --agent")
            && stderr.contains("LBR-CLI-002"),
        "{}",
        describe(&ambiguous)
    );
}

#[tokio::test]
async fn agent_import_all_records_each_completed_candidate() {
    let fixture = ImportRepo::init();
    fixture.write_discoverable_transcript("a-index-first", &fixture.repo);
    fixture.write_discoverable_transcript("b-index-second", &fixture.repo);
    let output = fixture.run(&[
        "agent",
        "import",
        "--all",
        "--agent",
        "claude-code",
        "--yes",
        "--json",
    ]);
    assert!(output.status.success(), "{}", describe(&output));
    let payload: Value = serde_json::from_slice(&output.stdout).expect("batch JSON");
    assert_eq!(payload["data"]["results"].as_array().map(Vec::len), Some(2));
    assert!(
        payload["data"]["results"]
            .as_array()
            .expect("batch results")
            .iter()
            .all(|result| result["status"] == "imported"),
        "each discovered candidate should report imported: {}",
        describe(&output)
    );
    assert_eq!(
        payload["data"]["failures"].as_array().map(Vec::len),
        Some(0)
    );
    assert_eq!(
        fixture
            .scalar(
                "SELECT COUNT(*) AS n FROM agent_checkpoint cp
                 JOIN agent_session s ON s.session_id = cp.session_id
                 WHERE s.provider_session_id = 'a-index-first'"
            )
            .await,
        1
    );
    assert_eq!(
        fixture
            .scalar(
                "SELECT COUNT(*) AS n FROM agent_checkpoint cp
                 JOIN agent_session s ON s.session_id = cp.session_id
                 WHERE s.provider_session_id = 'b-index-second'"
            )
            .await,
        1,
        "each discovered candidate should receive its own checkpoint"
    );
    assert_eq!(
        fixture
            .scalar(
                "SELECT COUNT(*) AS n FROM metadata_kv
                 WHERE scope = 'agent_import_index_repair'
                   AND target = 'claude__a-index-first'"
            )
            .await,
        0,
        "a completed import should not retain an index-repair barrier"
    );
}

#[tokio::test]
async fn agent_import_completed_index_write_needs_no_repair_barrier() {
    let fixture = ImportRepo::init();
    let transcript = fixture.write_discoverable_transcript("index-error", &fixture.repo);
    let transcript_arg = path_arg(&transcript);
    let first = fixture.run(&[
        "agent",
        "import",
        "--path",
        transcript_arg.as_str(),
        "--agent",
        "claude-code",
        "--yes",
        "--json",
    ]);
    assert!(first.status.success(), "{}", describe(&first));
    assert_eq!(
        fixture
            .scalar(
                "SELECT COUNT(*) AS n FROM metadata_kv
                 WHERE scope = 'agent_import_index_repair'
                   AND target = 'claude__index-error'"
            )
            .await,
        0,
        "a completed import should not retain a repair barrier"
    );
    assert_eq!(
        fixture
            .scalar(
                "SELECT COUNT(*) AS n FROM agent_import_identity
                 WHERE provider_session_id = 'index-error' AND state = 'committed'"
            )
            .await,
        1
    );

    let replay = fixture.run(&[
        "agent",
        "import",
        "--path",
        transcript_arg.as_str(),
        "--agent",
        "claude-code",
        "--yes",
        "--json",
    ]);
    assert!(replay.status.success(), "{}", describe(&replay));
    let doctor = fixture.run(&["agent", "doctor", "--json"]);
    assert!(doctor.status.success(), "{}", describe(&doctor));
    assert!(
        !String::from_utf8_lossy(&doctor.stdout).contains("missing_object_index"),
        "foreground replay repair must restore every E4 index row: {}",
        describe(&doctor)
    );
}

#[tokio::test]
async fn agent_import_reports_child_only_replay_as_imported() {
    let fixture = ImportRepo::init();
    let session_id = "abcdef00-0000-0000-0000-000000000012";
    let parent = fixture.write_discoverable_transcript(session_id, &fixture.repo);
    let parent_arg = path_arg(&parent);
    let args = [
        "agent",
        "import",
        "--path",
        parent_arg.as_str(),
        "--agent",
        "claude-code",
        "--yes",
        "--json",
    ];
    let first = fixture.run(&args);
    assert!(first.status.success(), "first import: {}", describe(&first));

    let child_dir = parent
        .parent()
        .expect("Claude project directory")
        .join(session_id)
        .join("subagents");
    std::fs::create_dir_all(&child_dir).expect("subagent directory");
    std::fs::write(
        child_dir.join("child.jsonl"),
        format!(
            "{}\n",
            json!({
                "type": "assistant",
                "uuid": "child-assistant",
                "message": {"role": "assistant", "content": "late child"}
            })
        ),
    )
    .expect("late child transcript");

    let replay = fixture.run(&args);
    assert!(
        replay.status.success(),
        "child-only replay: {}",
        describe(&replay)
    );
    let payload: Value = serde_json::from_slice(&replay.stdout).expect("replay JSON");
    assert_eq!(payload["data"]["results"][0]["status"], "imported");
    assert_eq!(payload["data"]["results"][0]["checkpoints_written"], 0);
    assert_eq!(
        payload["data"]["results"][0]["subagent_checkpoints_written"],
        1
    );
}

#[tokio::test]
async fn agent_import_marks_malformed_subagent_content_partial() {
    let fixture = ImportRepo::init();
    let session_id = "abcdef00-0000-0000-0000-000000000011";
    let parent = fixture.write_discoverable_transcript(session_id, &fixture.repo);
    let child_dir = parent
        .parent()
        .expect("Claude project directory")
        .join(session_id)
        .join("subagents");
    std::fs::create_dir_all(&child_dir).expect("subagent directory");
    std::fs::write(
        child_dir.join("child.jsonl"),
        format!(
            "not-json\n{}\n",
            json!({
                "type": "assistant",
                "uuid": "child-assistant",
                "message": {"role": "assistant", "content": "partial child"}
            })
        ),
    )
    .expect("malformed child transcript");

    let output = fixture.run(&[
        "agent",
        "import",
        "--path",
        path_arg(&parent).as_str(),
        "--agent",
        "claude-code",
        "--yes",
        "--json",
    ]);
    assert!(
        !output.status.success(),
        "malformed child content must make the import partial: {}",
        describe(&output)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("LBR-AGENT-018"),
        "{}",
        describe(&output)
    );
    assert_eq!(
        fixture
            .scalar(
                "SELECT COUNT(*) AS n FROM agent_import_identity
                 WHERE state = 'partial' AND last_error_code = 'LBR-AGENT-018'",
            )
            .await,
        1
    );
    assert_eq!(
        fixture
            .scalar("SELECT COUNT(*) AS n FROM agent_subagent_content_revision WHERE partial = 1",)
            .await,
        1
    );
    assert_eq!(
        fixture
            .scalar(
                "SELECT COUNT(*) AS n FROM agent_checkpoint cp
                 WHERE cp.scope = 'subagent'
                   AND (
                     NOT EXISTS (SELECT 1 FROM object_index oi WHERE oi.o_id = cp.traces_commit)
                     OR NOT EXISTS (SELECT 1 FROM object_index oi WHERE oi.o_id = cp.tree_oid)
                     OR NOT EXISTS (
                         SELECT 1 FROM object_index oi WHERE oi.o_id = cp.metadata_blob_oid
                     )
                   )",
            )
            .await,
        0,
        "partial import must drain every durable checkpoint object-index write before error exit"
    );
}

#[tokio::test]
async fn agent_import_marks_empty_subagent_content_partial() {
    let fixture = ImportRepo::init();
    let session_id = "abcdef00-0000-0000-0000-000000000012";
    let parent = fixture.write_discoverable_transcript(session_id, &fixture.repo);
    let child_dir = parent
        .parent()
        .expect("Claude project directory")
        .join(session_id)
        .join("subagents");
    std::fs::create_dir_all(&child_dir).expect("subagent directory");
    std::fs::write(child_dir.join("empty.jsonl"), b"").expect("empty child transcript");

    let output = fixture.run(&[
        "agent",
        "import",
        "--path",
        path_arg(&parent).as_str(),
        "--agent",
        "claude-code",
        "--yes",
        "--json",
    ]);
    assert!(
        !output.status.success(),
        "empty child evidence must make the import partial: {}",
        describe(&output)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("LBR-AGENT-018"),
        "{}",
        describe(&output)
    );
    assert_eq!(
        fixture
            .scalar("SELECT COUNT(*) AS n FROM agent_subagent_content_revision WHERE partial = 1")
            .await,
        1
    );
}

#[tokio::test]
async fn agent_import_marker_failure_is_fail_closed_and_preserves_existing_session() {
    let fixture = ImportRepo::init();
    let db_url = format!(
        "sqlite://{}",
        fixture.repo.join(".libra/libra.db").display()
    );
    let conn = Database::connect(db_url)
        .await
        .expect("open writable repo db");
    conn.execute_raw(Statement::from_string(
        conn.get_database_backend(),
        "CREATE TRIGGER reject_import_marker
         BEFORE INSERT ON metadata_kv
         WHEN NEW.scope = 'agent_traces_inflight'
         BEGIN
             SELECT RAISE(ABORT, 'test marker write failure');
         END"
        .to_string(),
    ))
    .await
    .expect("install marker rejection trigger");

    let fresh = fixture.write_transcript("markerfail-new", &fixture.repo, true);
    let objects_before = loose_object_file_count(&fixture.repo.join(".libra/objects"));
    let fresh_output = fixture.run(&[
        "agent",
        "import",
        "--path",
        path_arg(&fresh).as_str(),
        "--agent",
        "claude-code",
        "--yes",
        "--json",
    ]);
    assert!(!fresh_output.status.success(), "marker failure passed");
    assert_eq!(
        fixture
            .scalar("SELECT COUNT(*) AS n FROM agent_session")
            .await,
        0,
        "marker failure left a provisional session"
    );
    assert_eq!(
        loose_object_file_count(&fixture.repo.join(".libra/objects")),
        objects_before,
        "marker failure must happen before object construction"
    );

    conn.execute_raw(Statement::from_sql_and_values(
        conn.get_database_backend(),
        "INSERT INTO agent_session (
            session_id, agent_kind, provider_session_id, state, working_dir,
            metadata_json, redaction_report, started_at, last_event_at,
            stopped_at, schema_version
         ) VALUES (?, 'claude_code', ?, 'active', ?, ?, '{}', 11, 22, NULL, 1)",
        [
            "claude__markerfail-existing".into(),
            "markerfail-existing".into(),
            fixture.repo.to_string_lossy().into_owned().into(),
            serde_json::json!({"sentinel":"keep"}).to_string().into(),
        ],
    ))
    .await
    .expect("seed existing live session");
    let existing = fixture.write_transcript("markerfail-existing", &fixture.repo, true);
    let existing_output = fixture.run(&[
        "agent",
        "import",
        "--path",
        path_arg(&existing).as_str(),
        "--agent",
        "claude-code",
        "--yes",
        "--json",
    ]);
    assert!(!existing_output.status.success(), "marker failure passed");
    let row = conn
        .query_one_raw(Statement::from_string(
            conn.get_database_backend(),
            "SELECT state, working_dir, metadata_json, last_event_at, stopped_at
             FROM agent_session WHERE session_id = 'claude__markerfail-existing'"
                .to_string(),
        ))
        .await
        .expect("query preserved session")
        .expect("existing session remains");
    assert_eq!(
        row.try_get_by::<String, _>("state").expect("state"),
        "active"
    );
    assert_eq!(row.try_get_by::<i64, _>("last_event_at").expect("time"), 22);
    assert_eq!(
        row.try_get_by::<Option<i64>, _>("stopped_at")
            .expect("stopped"),
        None
    );
    assert_eq!(
        row.try_get_by::<String, _>("metadata_json")
            .expect("metadata"),
        serde_json::json!({"sentinel":"keep"}).to_string(),
        "failed import mutated existing session ownership metadata"
    );
}

#[cfg(unix)]
#[test]
fn agent_import_rejects_symlinked_source_fail_closed() {
    use std::os::unix::fs::symlink;

    let fixture = ImportRepo::init();
    let real = fixture.write_transcript("real123", &fixture.repo, true);
    let link = fixture.transcript_path("link123");
    symlink(&real, &link).expect("create transcript symlink");
    let output = fixture.run(&[
        "agent",
        "import",
        "--path",
        path_arg(&link).as_str(),
        "--agent",
        "claude-code",
        "--yes",
        "--json",
    ]);
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("LBR-AGENT-020"),
        "{}",
        describe(&output)
    );
}

#[test]
fn agent_list_schema_v1_stays_frozen_and_v2_is_explicit() {
    let fixture = ImportRepo::init();
    let v1 = fixture.run(&["agent", "list", "--json"]);
    assert!(v1.status.success(), "v1 list: {}", describe(&v1));
    let v1: Value = serde_json::from_slice(&v1.stdout).expect("v1 JSON");
    assert_eq!(v1["data"]["schema_version"], 1);
    let v1_row = v1["data"]["agents"][0].as_object().expect("v1 agent row");
    let v1_keys = v1_row.keys().map(String::as_str).collect::<Vec<_>>();
    assert_eq!(
        v1_keys,
        vec![
            "agent_kind",
            "capabilities",
            "config_paths",
            "db_value",
            "external_binary",
            "hook_installable",
            "installed",
            "launchable_investigate",
            "launchable_review",
            "protected_dirs",
            "provider_name",
            "registered",
            "slug",
            "stability",
            "support_wave",
            "supported",
            "transcript_readable",
        ],
        "v1 row key set/order is the frozen compatibility fixture"
    );

    let v2 = fixture.run(&["agent", "list", "--schema-version", "2", "--json"]);
    assert!(v2.status.success(), "v2 list: {}", describe(&v2));
    let v2: Value = serde_json::from_slice(&v2.stdout).expect("v2 JSON");
    assert_eq!(v2["data"]["schema_version"], 2);
    for method in v2["data"]["agents"][0]["methods"]
        .as_array()
        .expect("v2 methods")
    {
        assert_eq!(
            method
                .as_object()
                .expect("method object")
                .keys()
                .map(String::as_str)
                .collect::<Vec<_>>(),
            vec!["available", "name", "supported", "unavailable_reason"]
        );
    }
    let opencode = v2["data"]["agents"]
        .as_array()
        .expect("agents")
        .iter()
        .find(|row| row["slug"] == "opencode")
        .expect("opencode row");
    let discover = opencode["methods"]
        .as_array()
        .expect("methods")
        .iter()
        .find(|method| method["name"] == "transcript_discoverable")
        .expect("discovery method");
    assert_eq!(discover["supported"], false);
    assert_eq!(discover["available"], false);

    let unsupported = fixture.run(&["agent", "list", "--schema-version", "3", "--json"]);
    assert!(!unsupported.status.success());
    assert_eq!(unsupported.status.code(), Some(129));
    assert!(
        String::from_utf8_lossy(&unsupported.stderr).contains("LBR-AGENT-017"),
        "{}",
        describe(&unsupported)
    );
}

/// Explicit direct prepared turns exercise the actual local DB writer, with no
/// provider process, exported Session, credentials or model response.
#[cfg(unix)]
#[tokio::test]
async fn cov2_mixed_v1_v2_direct_import_idempotent() {
    use std::collections::BTreeMap;

    use libra::internal::ai::{
        agent_import::{ImportRequest, TurnBoundary, import_prepared_with_coverage_versions},
        capture::snapshot::{
            CaptureSnapshotCompleteness, CaptureSnapshotProjection, CaptureSnapshotSource,
            CaptureSnapshotSourceKind,
        },
        capture_scope::CaptureScope,
        observed_agents::{Completeness, NormalizedTurn, SemanticRecord},
    };

    let fixture = ImportRepo::init();
    let storage = fixture.repo.join(".libra");
    let conn = Database::connect(format!("sqlite://{}", storage.join("libra.db").display()))
        .await
        .unwrap();
    let scope = CaptureScope::main_for_connection(&conn).await.unwrap();
    let turn = |key: &str, ordinal: usize| NormalizedTurn {
        logical_turn_key: key.to_owned(),
        ordinal,
        completeness: Completeness::Complete,
        started_at: None,
        ended_at: None,
        records: vec![SemanticRecord::User {
            text: "fixed ordinary direct import".to_owned(),
        }],
    };
    let request = |turns: Vec<NormalizedTurn>| {
        let commitment = format!("source/hmac-v2/{}", "a".repeat(64));
        ImportRequest {
            agent_kind: AgentKind::Codex,
            provider_session_id: "cov2-direct-integration".to_owned(),
            session_id: "codex__cov2-direct-integration".to_owned(),
            source_kind: "file".to_owned(),
            source_id: commitment.clone(),
            identity_schema_version: 2,
            content_digest: "caller digest must not be trusted".to_owned(),
            started_at: 1,
            ended_at: 2,
            session_state: "stopped".to_owned(),
            stopped_at: Some(2),
            working_dir: fixture.repo.clone(),
            repository_identity: "not_retained:v1".to_owned(),
            capture_scope: Some(scope.clone()),
            source_fingerprint: commitment.clone(),
            existing_session_fingerprint: None,
            redaction_report: json!({"pipeline":"typed_allowlist","raw_persisted":false,"matches":[],"bytes_scanned":32,"bytes_redacted":0}),
            transcript_snapshot: Some(CaptureSnapshotProjection {
                completeness: CaptureSnapshotCompleteness::Complete,
                partial_reason: None,
                source: Some(CaptureSnapshotSource {
                    kind: CaptureSnapshotSourceKind::ProviderFile,
                    identity: "not_retained:v1".to_owned(),
                    digest_sha256: Some(commitment),
                    byte_len: Some(32),
                }),
                transcript_redacted_bytes: 32,
                redaction_match_count: 0,
                redaction_bytes_scanned: 32,
                redaction_bytes_redacted: 0,
            }),
            turn_boundaries: turns
                .iter()
                .map(|turn| {
                    (
                        turn.logical_turn_key.clone(),
                        TurnBoundary {
                            started_at: 1,
                            ended_at: 2,
                        },
                    )
                })
                .collect::<BTreeMap<_, _>>(),
            turns,
        }
    };
    // Reverse lexical/source order catches a sidecar left behind by sorting.
    let v1 = vec![turn("z-first", 0)];
    let mixed = vec![turn("z-first", 0), turn("a-second", 1)];
    for (turns, versions, written, skipped) in [
        (v1.clone(), vec![1], 1, 0),
        (mixed.clone(), vec![1, 2], 1, 1),
        (v1.clone(), vec![1], 0, 1),
        (v1.clone(), vec![2], 1, 0),
        (v1.clone(), vec![2], 0, 1),
        (mixed.clone(), vec![1, 2], 0, 2),
        (vec![mixed[1].clone(), mixed[0].clone()], vec![2, 1], 0, 2),
    ] {
        let summary = import_prepared_with_coverage_versions(
            &conn,
            &storage,
            request(turns),
            &versions,
            Duration::from_secs(30),
            Some(Path::new(env!("CARGO_BIN_EXE_libra"))),
        )
        .await
        .unwrap();
        assert_eq!(summary.checkpoints_written, written);
        assert_eq!(summary.skipped_covered, skipped);
        assert!(!summary.partial);
    }
    let rows = conn.query_all_raw(Statement::from_string(conn.get_database_backend(),
        "SELECT logical_turn_key, coverage_schema_version, revision FROM agent_coverage_claim ORDER BY logical_turn_key, coverage_schema_version".to_owned())).await.unwrap();
    let actual = rows
        .iter()
        .map(|row| {
            (
                row.try_get_by::<String, _>("logical_turn_key").unwrap(),
                row.try_get_by::<i64, _>("coverage_schema_version").unwrap(),
                row.try_get_by::<i64, _>("revision").unwrap(),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        actual,
        vec![
            ("a-second".to_owned(), 2, 1),
            ("z-first".to_owned(), 1, 1),
            ("z-first".to_owned(), 2, 1)
        ]
    );
    let checkpoints = conn
        .query_all_raw(Statement::from_string(
            conn.get_database_backend(),
            "SELECT r.logical_turn_key, r.coverage_schema_version, c.checkpoint_id, c.tree_oid
         FROM agent_coverage_revision r JOIN agent_checkpoint c ON c.checkpoint_id=r.checkpoint_id"
                .to_owned(),
        ))
        .await
        .unwrap();
    assert_eq!(checkpoints.len(), 3);
    for row in checkpoints {
        let key: String = row.try_get_by("logical_turn_key").unwrap();
        let version: i64 = row.try_get_by("coverage_schema_version").unwrap();
        let id: String = row.try_get_by("checkpoint_id").unwrap();
        let tree: String = row.try_get_by("tree_oid").unwrap();
        let transcript = read_checkpoint_blob(&storage, &tree, &id, &["transcript", "codex.jsonl"]);
        let projection: Value = serde_json::from_slice(&transcript).unwrap();
        assert_eq!(projection["schema_version"], version);
        assert_eq!(projection["logical_turn_key"], key);
    }
    let count = |table: &str| {
        let query = format!("SELECT COUNT(*) AS n FROM {table}");
        let conn = &conn;
        async move {
            conn.query_one_raw(Statement::from_string(conn.get_database_backend(), query))
                .await
                .unwrap()
                .unwrap()
                .try_get_by::<i64, _>("n")
                .unwrap()
        }
    };
    assert_eq!(count("agent_checkpoint").await, 3);
    assert_eq!(count("agent_coverage_revision").await, 3);
    let before = conn
        .query_one_raw(Statement::from_string(
            conn.get_database_backend(),
            "SELECT fence_token, committed_digest FROM agent_import_identity".to_owned(),
        ))
        .await
        .unwrap()
        .unwrap();
    let fence: i64 = before.try_get_by("fence_token").unwrap();
    let digest: String = before.try_get_by("committed_digest").unwrap();
    assert_ne!(digest, "caller digest must not be trusted");
    for (turns, versions) in [
        (vec![turn("new", 0)], vec![3]),
        (vec![turn("new", 0)], vec![]),
        (vec![turn("same", 0), turn("same", 1)], vec![1, 2]),
    ] {
        assert!(
            import_prepared_with_coverage_versions(
                &conn,
                &storage,
                request(turns),
                &versions,
                Duration::from_secs(30),
                Some(Path::new(env!("CARGO_BIN_EXE_libra"))),
            )
            .await
            .is_err()
        );
    }
    let canary = format!("AKIA{}", "A".repeat(16));
    let mut secret = turn("private", 0);
    secret.records = vec![SemanticRecord::User {
        text: canary.clone(),
    }];
    let error = import_prepared_with_coverage_versions(
        &conn,
        &storage,
        request(vec![secret]),
        &[2],
        Duration::from_secs(30),
        Some(Path::new(env!("CARGO_BIN_EXE_libra"))),
    )
    .await
    .unwrap_err();
    assert!(!format!("{error:#}").contains(&canary));
    for budget in [Duration::ZERO, Duration::from_secs(301)] {
        assert!(
            import_prepared_with_coverage_versions(
                &conn,
                &storage,
                request(v1.clone()),
                &[1],
                budget,
                Some(Path::new(env!("CARGO_BIN_EXE_libra"))),
            )
            .await
            .is_err()
        );
    }
    let mut foreign_scope = request(v1.clone());
    foreign_scope.capture_scope.as_mut().unwrap().repo_id = "different-repository".to_owned();
    assert!(
        import_prepared_with_coverage_versions(
            &conn,
            &storage,
            foreign_scope,
            &[1],
            Duration::from_secs(30),
            Some(Path::new(env!("CARGO_BIN_EXE_libra")))
        )
        .await
        .is_err()
    );
    let foreign_storage = fixture.repo.join("foreign-storage");
    assert!(
        import_prepared_with_coverage_versions(
            &conn,
            &foreign_storage,
            request(v1.clone()),
            &[1],
            Duration::from_secs(30),
            Some(Path::new(env!("CARGO_BIN_EXE_libra")))
        )
        .await
        .is_err()
    );
    assert!(
        !foreign_storage.exists(),
        "storage mismatch must reject before any filesystem writes"
    );
    let absent_helper = fixture.repo.join("missing-libra-helper");
    for helper in [
        None,
        Some(Path::new("relative-libra")),
        Some(absent_helper.as_path()),
    ] {
        let error = import_prepared_with_coverage_versions(
            &conn,
            &storage,
            request(v1.clone()),
            &[1],
            Duration::from_secs(30),
            helper,
        )
        .await
        .unwrap_err();
        assert!(format!("{error:#}").contains("checkpoint helper"));
    }
    assert_eq!(count("agent_checkpoint").await, 3);
    assert_eq!(count("agent_coverage_claim").await, 3);
    let after = conn
        .query_one_raw(Statement::from_string(
            conn.get_database_backend(),
            "SELECT fence_token FROM agent_import_identity".to_owned(),
        ))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        after.try_get_by::<i64, _>("fence_token").unwrap(),
        fence,
        "invalid input must reject before leasing"
    );
}

fn hermes_source_scope(
    repo: &std::path::Path,
) -> libra::internal::ai::observed_agents::HermesScope {
    libra::internal::ai::observed_agents::HermesScope {
        session_id: "sess-hermes".to_string(),
        profile: "default".to_string(),
        repo_root: repo.to_path_buf(),
    }
}

fn hermes_source_session(repo: &std::path::Path, messages: serde_json::Value) -> String {
    serde_json::json!({
        "id": "sess-hermes",
        "source": "cli",
        "created_source": "cli",
        "user_id": null,
        "cwd": repo,
        "profile_name": "default",
        "messages": messages,
    })
    .to_string()
}

#[test]
fn hermes_source_jsonl_keeps_thinking_and_drops_revoked_rows() {
    let repo = std::path::PathBuf::from("/work/repo");
    let messages = serde_json::json!([
        {"active": 0, "compacted": 0, "reasoning": "revoked-thought", "reasoning_content": null},
        {"active": 1, "compacted": 0, "reasoning": null, "reasoning_content": "", "reasoning_details": "{\"raw\":1}", "codex_reasoning_items": "[]", "codex_message_items": null},
        {"active": 0, "compacted": 1, "reasoning": "kept-thought", "reasoning_content": "also-kept", "reasoning_details": "{\"keep\":true}"}
    ]);
    let bytes = hermes_source_session(&repo, messages);
    let status = libra::internal::ai::observed_agents::read_hermes_jsonl(
        bytes.as_bytes(),
        &hermes_source_scope(&repo),
        &libra::internal::ai::observed_agents::HermesReadLimits::default(),
    );
    let libra::internal::ai::observed_agents::HermesSourceStatus::Read(transcript) = status else {
        panic!("jsonl read failed: {status}");
    };
    assert!(transcript.thinking_fields_present);
    assert!(!transcript.partial);
    assert_eq!(transcript.messages.len(), 2, "revoked row must stay out");
    assert!(
        transcript
            .messages
            .iter()
            .all(|message| message.readable_thinking() != Some("revoked-thought"))
    );
    assert!(transcript.messages[0].readable_thinking().is_none());
    assert_eq!(transcript.messages[0].reasoning, None);
    assert_eq!(
        transcript.messages[0].reasoning_content.as_deref(),
        Some("")
    );
    assert_eq!(
        transcript.messages[0].reasoning_details.as_deref(),
        Some("{\"raw\":1}")
    );
    assert_eq!(
        transcript.messages[0].codex_reasoning_items.as_deref(),
        Some("[]")
    );
    assert_eq!(transcript.messages[0].codex_message_items, None);
    assert_eq!(
        transcript.messages[1].readable_thinking(),
        Some("kept-thought")
    );
    assert_eq!(
        transcript.messages[1].reasoning_content.as_deref(),
        Some("also-kept")
    );
    assert_eq!(
        transcript.messages[1].reasoning_details.as_deref(),
        Some("{\"keep\":true}")
    );
    assert!(!status_text_leaks(&transcript.messages[1]));

    let limits = libra::internal::ai::observed_agents::HermesReadLimits {
        max_rows: 1,
        ..libra::internal::ai::observed_agents::HermesReadLimits::default()
    };
    let status = libra::internal::ai::observed_agents::read_hermes_jsonl(
        bytes.as_bytes(),
        &hermes_source_scope(&repo),
        &limits,
    );
    let libra::internal::ai::observed_agents::HermesSourceStatus::Read(truncated) = status else {
        panic!("row budget read failed: {status}");
    };
    assert!(
        truncated.partial,
        "a kept row past the budget must stay partial"
    );
    assert_eq!(truncated.messages.len(), 1);
}

#[test]
fn hermes_source_jsonl_accepts_the_local_transport_label() {
    let repo = std::path::PathBuf::from("/work/repo");
    let mut local = serde_json::from_str::<serde_json::Value>(&hermes_source_session(
        &repo,
        serde_json::json!([
            {"active": 1, "compacted": 0, "reasoning": "snake-thought", "reasoning_content": "snake-thought"}
        ]),
    ))
    .expect("json");
    local["source"] = serde_json::json!("local");
    local["created_source"] = serde_json::json!("cli");
    let status = libra::internal::ai::observed_agents::read_hermes_jsonl(
        local.to_string().as_bytes(),
        &hermes_source_scope(&repo),
        &libra::internal::ai::observed_agents::HermesReadLimits::default(),
    );
    let libra::internal::ai::observed_agents::HermesSourceStatus::Read(transcript) = status else {
        panic!("source=local created_source=cli must stay readable: {status}");
    };
    assert_eq!(
        transcript.messages[0].readable_thinking(),
        Some("snake-thought")
    );
    local["created_source"] = serde_json::json!("telegram");
    let status = libra::internal::ai::observed_agents::read_hermes_jsonl(
        local.to_string().as_bytes(),
        &hermes_source_scope(&repo),
        &libra::internal::ai::observed_agents::HermesReadLimits::default(),
    );
    assert!(matches!(
        status,
        libra::internal::ai::observed_agents::HermesSourceStatus::Unavailable
    ));
}

#[cfg(unix)]
#[test]
fn hermes_source_jsonl_accepts_only_the_local_uid_subject() {
    let repo = std::path::PathBuf::from("/work/repo");
    let mut local = serde_json::from_str::<serde_json::Value>(&hermes_source_session(
        &repo,
        serde_json::json!([
            {"active": 1, "compacted": 0, "reasoning": "local-thought", "reasoning_content": "local-thought"}
        ]),
    ))
    .expect("json");
    // SAFETY: geteuid only reads the process credential.
    let uid = unsafe { libc::geteuid() };
    local["user_id"] = serde_json::json!(format!("uid:{uid}"));
    let status = libra::internal::ai::observed_agents::read_hermes_jsonl(
        local.to_string().as_bytes(),
        &hermes_source_scope(&repo),
        &libra::internal::ai::observed_agents::HermesReadLimits::default(),
    );
    let libra::internal::ai::observed_agents::HermesSourceStatus::Read(transcript) = status else {
        panic!("local uid must stay readable: {status}");
    };
    assert_eq!(
        transcript.messages[0].readable_thinking(),
        Some("local-thought")
    );

    local["user_id"] = serde_json::json!("uid:0");
    if uid == 0 {
        local["user_id"] = serde_json::json!("uid:1");
    }
    let status = libra::internal::ai::observed_agents::read_hermes_jsonl(
        local.to_string().as_bytes(),
        &hermes_source_scope(&repo),
        &libra::internal::ai::observed_agents::HermesReadLimits::default(),
    );
    assert!(matches!(
        status,
        libra::internal::ai::observed_agents::HermesSourceStatus::Unavailable
    ));
    local["user_id"] = serde_json::json!("telegram-user");
    let status = libra::internal::ai::observed_agents::read_hermes_jsonl(
        local.to_string().as_bytes(),
        &hermes_source_scope(&repo),
        &libra::internal::ai::observed_agents::HermesReadLimits::default(),
    );
    assert!(matches!(
        status,
        libra::internal::ai::observed_agents::HermesSourceStatus::Unavailable
    ));
    assert!(!status.to_string().contains("local-thought"));
}

#[test]
fn hermes_source_jsonl_rejects_gateway_outside_cwd_and_symlink() {
    let root = tempfile::tempdir().expect("root");
    let repo = root.path().join("repo");
    std::fs::create_dir(&repo).expect("repo");
    let gateway = hermes_source_session(
        &repo,
        serde_json::json!([
            {"active": 1, "compacted": 0, "reasoning": "nope"}
        ]),
    );
    let mut gateway_value: serde_json::Value = serde_json::from_str(&gateway).expect("json");
    gateway_value["source"] = serde_json::json!("gateway");
    let status = libra::internal::ai::observed_agents::read_hermes_jsonl(
        gateway_value.to_string().as_bytes(),
        &hermes_source_scope(&repo),
        &libra::internal::ai::observed_agents::HermesReadLimits::default(),
    );
    assert!(matches!(
        status,
        libra::internal::ai::observed_agents::HermesSourceStatus::Unavailable
    ));
    assert!(!status.to_string().contains("nope"));
    gateway_value["source"] = serde_json::json!("matrix");
    let status = libra::internal::ai::observed_agents::read_hermes_jsonl(
        gateway_value.to_string().as_bytes(),
        &hermes_source_scope(&repo),
        &libra::internal::ai::observed_agents::HermesReadLimits::default(),
    );
    assert!(matches!(
        status,
        libra::internal::ai::observed_agents::HermesSourceStatus::Unavailable
    ));

    let outside = hermes_source_session(
        std::path::Path::new("/other/repo"),
        serde_json::json!([
            {"active": 1, "compacted": 0, "reasoning": "outside-thought"}
        ]),
    );
    let status = libra::internal::ai::observed_agents::read_hermes_jsonl(
        outside.as_bytes(),
        &hermes_source_scope(&repo),
        &libra::internal::ai::observed_agents::HermesReadLimits::default(),
    );
    assert!(matches!(
        status,
        libra::internal::ai::observed_agents::HermesSourceStatus::Unavailable
    ));

    #[cfg(unix)]
    {
        let export_dir = root.path().join("export");
        std::fs::create_dir(&export_dir).expect("export dir");
        let real = export_dir.join("real.jsonl");
        std::fs::write(&real, format!("{gateway}\n")).expect("real");
        let link = export_dir.join("link.jsonl");
        std::os::unix::fs::symlink(&real, &link).expect("symlink");
        let status = libra::internal::ai::observed_agents::read_hermes_jsonl_file(
            &link,
            &export_dir,
            &hermes_source_scope(&repo),
            &libra::internal::ai::observed_agents::HermesReadLimits::default(),
        );
        assert!(matches!(
            status,
            libra::internal::ai::observed_agents::HermesSourceStatus::Unavailable
        ));
        assert!(!status.to_string().contains(&real.display().to_string()));
    }
}

#[cfg(unix)]
#[tokio::test]
async fn hermes_source_sqlite_reads_wal_and_refuses_a_read_only_directory() {
    use std::os::unix::fs::PermissionsExt;

    let root = tempfile::tempdir().expect("root");
    let root = root.path().canonicalize().expect("canonical root");
    let repo = root.join("repo");
    std::fs::create_dir(&repo).expect("repo");
    let db = root.join("state.db");
    let script = format!(
        r#"
import os
import sqlite3
conn = sqlite3.connect({db:?})
conn.execute("PRAGMA journal_mode=WAL")
conn.execute("CREATE TABLE sessions (id TEXT PRIMARY KEY, source TEXT NOT NULL, created_source TEXT, user_id TEXT, cwd TEXT, profile_name TEXT)")
conn.execute("CREATE TABLE messages (id INTEGER PRIMARY KEY AUTOINCREMENT, session_id TEXT NOT NULL, reasoning TEXT, reasoning_content TEXT, reasoning_details TEXT, codex_reasoning_items TEXT, codex_message_items TEXT, active INTEGER NOT NULL, compacted INTEGER NOT NULL)")
conn.execute("INSERT INTO sessions VALUES ('sess-hermes', 'cli', 'cli', NULL, ?, 'default')", ({repo:?},))
conn.execute("INSERT INTO messages (session_id, reasoning, reasoning_content, reasoning_details, codex_reasoning_items, codex_message_items, active, compacted) VALUES ('sess-hermes', 'revoked-thought', NULL, NULL, NULL, NULL, 0, 0)")
conn.execute("INSERT INTO messages (session_id, reasoning, reasoning_content, reasoning_details, codex_reasoning_items, codex_message_items, active, compacted) VALUES ('sess-hermes', NULL, '', '{{\"raw\":1}}', '[]', NULL, 1, 0)")
conn.execute("INSERT INTO messages (session_id, reasoning, reasoning_content, reasoning_details, codex_reasoning_items, codex_message_items, active, compacted) VALUES ('sess-hermes', 'kept-thought', NULL, NULL, NULL, NULL, 0, 1)")
conn.commit()
# Exit without closing the connection: a clean close checkpoints and deletes
# the WAL, and whether interpreter shutdown closes it depends on the Python
# version.
os._exit(0)
"#,
        db = db.display().to_string(),
        repo = repo.display().to_string(),
    );
    let planted = std::process::Command::new("python3")
        .arg("-c")
        .arg(&script)
        .status()
        .expect("python");
    assert!(planted.success(), "sqlite fixture");
    let wal = PathBuf::from(format!("{}-wal", db.display()));
    assert!(wal.exists(), "wal must hold the uncheckpointed rows");
    let db_before = std::fs::read(&db).expect("db bytes");
    let wal_before = std::fs::read(&wal).expect("wal bytes");
    let request = libra::internal::ai::observed_agents::HermesSourceRequest {
        authorized_root: root.clone(),
        jsonl: None,
        sqlite: Some(db.clone()),
        scope: hermes_source_scope(&repo),
        limits: libra::internal::ai::observed_agents::HermesReadLimits::default(),
    };
    let status = libra::internal::ai::observed_agents::read_hermes_source(&request).await;
    let libra::internal::ai::observed_agents::HermesSourceStatus::Read(transcript) = &status else {
        panic!("wal read failed: {status}");
    };
    assert_eq!(transcript.messages.len(), 2);
    assert_eq!(
        transcript.messages[1].readable_thinking(),
        Some("kept-thought")
    );
    let tight = libra::internal::ai::observed_agents::HermesReadLimits {
        max_rows: 1,
        ..libra::internal::ai::observed_agents::HermesReadLimits::default()
    };
    let tight_status = libra::internal::ai::observed_agents::read_hermes_sqlite(
        &db,
        &root,
        &hermes_source_scope(&repo),
        &tight,
    )
    .await;
    let libra::internal::ai::observed_agents::HermesSourceStatus::Read(tight_transcript) =
        &tight_status
    else {
        panic!("sqlite row budget failed: {tight_status}");
    };
    assert!(
        tight_transcript.partial,
        "revoked rows must not hide a kept row past the budget"
    );
    assert_eq!(tight_transcript.messages.len(), 1);
    assert!(
        transcript
            .messages
            .iter()
            .all(|message| message.readable_thinking() != Some("revoked-thought"))
    );
    assert_eq!(std::fs::read(&db).expect("db after"), db_before);
    assert_eq!(std::fs::read(&wal).expect("wal after"), wal_before);
    assert!(!status.to_string().contains("kept-thought"));
    assert!(!status.to_string().contains("SELECT"));

    let shm = PathBuf::from(format!("{}-shm", db.display()));
    let _ = std::fs::remove_file(&shm);
    let status = libra::internal::ai::observed_agents::read_hermes_sqlite(
        &db,
        &root,
        &hermes_source_scope(&repo),
        &libra::internal::ai::observed_agents::HermesReadLimits::default(),
    )
    .await;
    assert!(matches!(
        status,
        libra::internal::ai::observed_agents::HermesSourceStatus::Read(_)
    ));
    assert_eq!(std::fs::read(&db).expect("db after shm"), db_before);
    assert_eq!(std::fs::read(&wal).expect("wal after shm"), wal_before);

    let _ = std::fs::remove_file(&shm);
    let _ = std::fs::remove_file(PathBuf::from(format!("{}-shm", db.display())));
    let mut locked = std::fs::metadata(&root).expect("root meta").permissions();
    locked.set_mode(0o555);
    std::fs::set_permissions(&root, locked).expect("readonly dir");
    let status = libra::internal::ai::observed_agents::read_hermes_sqlite(
        &db,
        &root,
        &hermes_source_scope(&repo),
        &libra::internal::ai::observed_agents::HermesReadLimits::default(),
    )
    .await;
    let mut restore = std::fs::metadata(&root).expect("root meta").permissions();
    restore.set_mode(0o755);
    std::fs::set_permissions(&root, restore).expect("restore dir");
    assert!(matches!(
        status,
        libra::internal::ai::observed_agents::HermesSourceStatus::Unavailable
    ));
    assert_eq!(std::fs::read(&db).expect("db readonly"), db_before);
    assert_eq!(std::fs::read(&wal).expect("wal readonly"), wal_before);
    assert!(!status.to_string().contains(&db.display().to_string()));
}

fn status_text_leaks(message: &libra::internal::ai::observed_agents::HermesMessage) -> bool {
    let rendered = format!("{message:?}");
    rendered.contains("/secret")
}

fn hermes_normalize_session(repo: &std::path::Path, messages: serde_json::Value) -> String {
    json!({
        "id": "0123456789abcdef0123456789abcdef",
        "source": "cli",
        "cwd": repo,
        "started_at": 1_700_000_000,
        "ended_at": 1_700_000_100,
        "parent_session_id": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "input_tokens": 5,
        "output_tokens": 7,
        "session_model_usage": [{"input_tokens": 50, "output_tokens": 70}],
        "messages": messages,
    })
    .to_string()
}

fn hermes_normalize_scope(
    repo: &std::path::Path,
) -> libra::internal::ai::observed_agents::builtin::hermes::HermesScope {
    libra::internal::ai::observed_agents::builtin::hermes::HermesScope {
        session_id: "0123456789abcdef0123456789abcdef".to_string(),
        profile: "default".to_string(),
        repo_root: repo.to_path_buf(),
    }
}

#[test]
fn hermes_normalize_sorts_by_id_keeps_thinking_and_ignores_mirrored_usage() {
    use libra::internal::ai::observed_agents::{
        Completeness, HermesReadLimits, SemanticRecord,
        builtin::hermes::{HermesNormalizeStatus, normalize_hermes_jsonl},
    };

    let repo = std::env::temp_dir().join("hermes-normalize-scope");
    let bytes = hermes_normalize_session(
        &repo,
        json!([
            {"id": 3, "role": "assistant", "content": "answer", "active": 1, "compacted": 0,
             "timestamp": 10, "reasoning": "kept-thinking", "reasoning_content": "",
             "reasoning_details": "{\"raw\":true}", "token_count": 99, "finish_reason": "stop",
             "tool_calls": [{"id": "call-1", "function": {"name": "read", "arguments": "{\"path\":\"a\"}"}}]},
            {"id": 1, "role": "user", "content": "prompt", "active": 1, "compacted": 0,
             "timestamp": 99, "reasoning": null},
            {"id": 2, "role": "user", "content": "revoked", "active": 0, "compacted": 0,
             "timestamp": 1, "token_count": 99},
            {"id": 4, "role": "tool", "content": "tool-out", "tool_call_id": "call-1",
             "active": 1, "compacted": 0, "timestamp": 11, "token_count": 5},
            {"id": 5, "role": "assistant", "content": "failed-copy", "active": 1, "compacted": 0,
             "timestamp": 12, "finish_reason": "error", "token_count": 40, "message_uid": "dup"},
            {"id": 6, "role": "assistant", "content": "retried-answer", "active": 1, "compacted": 0,
             "timestamp": 13, "finish_reason": "stop", "token_count": 40, "message_uid": "dup"},
            {"id": 7, "role": "assistant", "content": "truncated", "active": 1, "compacted": 0,
             "timestamp": 14, "finish_reason": "length", "token_count": 3}
        ]),
    );
    let status = normalize_hermes_jsonl(
        bytes.as_bytes(),
        &hermes_normalize_scope(&repo),
        &HermesReadLimits::default(),
    );
    let HermesNormalizeStatus::Ready(normalized) = status else {
        panic!("normalize unavailable");
    };
    assert!(!normalized.parent_associated);
    assert_eq!(normalized.source_state, "unproven");
    let usage = normalized.usage.expect("known session usage");
    assert_eq!(usage.input_tokens, Some(5));
    assert_eq!(usage.output_tokens, Some(7));
    assert_eq!(usage.cache_read_tokens, None);
    assert!(!usage.malformed);
    assert_eq!(normalized.turns.len(), 1);
    let turn = &normalized.turns[0];
    assert_eq!(turn.logical_turn_key, "1");
    assert_eq!(turn.ordinal, 0);
    assert_eq!(turn.completeness, Completeness::Incomplete);
    assert!(matches!(turn.records[0], SemanticRecord::User { ref text } if text == "prompt"));
    assert!(matches!(turn.records[1], SemanticRecord::Assistant { ref text } if text == "answer"));
    assert!(
        turn.records.iter().any(
            |record| matches!(record, SemanticRecord::ToolCall { name, .. } if name == "read")
        )
    );
    assert!(turn.records.iter().any(|record| matches!(record, SemanticRecord::ToolResult { content, .. } if content == "tool-out")));
    assert!(turn.records.iter().any(
        |record| matches!(record, SemanticRecord::Assistant { text } if text == "retried-answer")
    ));
    assert!(!format!("{turn:?}").contains("kept-thinking"));
    assert!(!format!("{turn:?}").contains("failed-copy"));
    assert!(!format!("{turn:?}").contains("revoked"));
    let retained = normalized
        .retained
        .iter()
        .find(|item| item.message_id == Some(3))
        .expect("retained thinking");
    assert_eq!(retained.reasoning.as_deref(), Some("kept-thinking"));
    assert_eq!(retained.reasoning_content.as_deref(), Some(""));
    assert_eq!(
        retained.reasoning_details.as_deref(),
        Some("{\"raw\":true}")
    );
    assert_eq!(retained.turn_key, "1");

    let unknown = hermes_normalize_session(
        &repo,
        json!([
            {"id": 1, "role": "user", "content": "prompt", "active": 1, "compacted": 0, "timestamp": 10}
        ]),
    );
    let unknown = unknown
        .replace("\"input_tokens\":5,", "")
        .replace("\"output_tokens\":7,", "");
    let status = normalize_hermes_jsonl(
        unknown.as_bytes(),
        &hermes_normalize_scope(&repo),
        &HermesReadLimits::default(),
    );
    let HermesNormalizeStatus::Ready(normalized) = status else {
        panic!("unknown usage unavailable");
    };
    assert!(normalized.usage.is_none());
}

#[test]
fn hermes_normalize_import_replays_one_checkpoint_without_enabling_hermes() {
    use libra::internal::ai::observed_agents::registration_for;

    let fixture = ImportRepo::init();
    let session_id = "0123456789abcdef0123456789abcdef";
    let path = fixture.repo.join(format!("{session_id}.jsonl"));
    std::fs::write(
        &path,
        hermes_normalize_session(
            &fixture.repo,
            json!([
                {"id": 1, "role": "user", "content": "prompt", "active": 1, "compacted": 0, "timestamp": 1700000001},
                {"id": 2, "role": "assistant", "content": "answer", "active": 1, "compacted": 0,
                 "timestamp": 1700000002, "reasoning": "kept-thinking", "finish_reason": "stop"}
            ]),
        ),
    )
    .expect("write export");
    let scanned = fixture.run(&["agent", "import", "--agent", "hermes", "--all", "--yes"]);
    assert!(!scanned.status.success(), "{}", describe(&scanned));
    assert_eq!(checkpoint_count(&fixture), 0);
    let first = fixture.run(&[
        "agent",
        "import",
        "--agent",
        "hermes",
        "--path",
        path.to_str().expect("utf8 path"),
        "--yes",
    ]);
    assert!(first.status.success(), "{}", describe(&first));
    let shown = format!(
        "{}{}",
        String::from_utf8_lossy(&first.stdout),
        String::from_utf8_lossy(&first.stderr)
    );
    assert!(!shown.contains("kept-thinking"), "{shown}");
    assert!(!shown.contains("prompt"), "{shown}");
    assert_eq!(checkpoint_count(&fixture), 1);
    let state = identity_state(&fixture);
    assert_eq!(state, "committed");
    let second = fixture.run(&[
        "agent",
        "import",
        "--agent",
        "hermes",
        "--path",
        path.to_str().expect("utf8 path"),
        "--yes",
    ]);
    assert!(second.status.success(), "{}", describe(&second));
    assert_eq!(
        checkpoint_count(&fixture),
        1,
        "replay wrote another checkpoint"
    );
    let registration = registration_for(AgentKind::Hermes);
    assert!(registration.supported);
    assert!(registration.hook_installable);
    assert!(!registration.launchable_review);
    assert!(!registration.launchable_investigate);
    assert!(registration.transcript_readable);
    let listed = fixture.run(&["agent", "list", "--json"]);
    assert!(listed.status.success(), "{}", describe(&listed));
    assert!(String::from_utf8_lossy(&listed.stdout).contains("hermes"));

    let wrong = fixture.repo.join("fedcba9876543210fedcba9876543210.jsonl");
    let mut body = hermes_normalize_session(
        &fixture.repo,
        json!([
            {"id": 1, "role": "user", "content": "other", "active": 1, "compacted": 0, "timestamp": 1700000001}
        ]),
    );
    body = body.replace(
        "\"id\":\"0123456789abcdef0123456789abcdef\"",
        "\"id\":\"fedcba9876543210fedcba9876543210\"",
    );
    body = body.replace(
        "\"source\":\"cli\"",
        "\"source\":\"cli\",\"profile_name\":\"other\"",
    );
    std::fs::write(&wrong, body).expect("write other profile");
    let rejected = fixture.run(&[
        "agent",
        "import",
        "--agent",
        "hermes",
        "--path",
        wrong.to_str().expect("utf8 path"),
        "--yes",
    ]);
    assert!(!rejected.status.success(), "{}", describe(&rejected));
    assert_eq!(checkpoint_count(&fixture), 1);
}

fn checkpoint_count(fixture: &ImportRepo) -> i64 {
    scalar(fixture, "SELECT COUNT(*) FROM agent_checkpoint")
}

fn identity_state(fixture: &ImportRepo) -> String {
    scalar_text(fixture, "SELECT state FROM agent_import_identity")
}

fn scalar(fixture: &ImportRepo, sql: &str) -> i64 {
    let text = scalar_text(fixture, sql);
    text.trim()
        .parse()
        .unwrap_or_else(|_| panic!("scalar {sql}: {text}"))
}

fn scalar_text(fixture: &ImportRepo, sql: &str) -> String {
    let db = fixture.repo.join(".libra/libra.db");
    let mut command = std::process::Command::new("python3");
    let output = command
        .arg("-c")
        .arg("import sqlite3,sys; print(sqlite3.connect(sys.argv[1]).execute(sys.argv[2]).fetchone()[0])")
        .arg(&db)
        .arg(sql)
        .output()
        .expect("query sqlite");
    assert!(output.status.success(), "{}", describe(&output));
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}
