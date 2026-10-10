//! Task A6.5 — Agent 第一期本地三 Agent 采集 smoke (plan.md §0.3, first-batch
//! hard gate).
//!
//! Drives the **real**, locally installed `codex`, `claude` and `opencode`
//! CLIs (one minimal paid session each) in throwaway Libra repositories
//! and asserts the whole capture chain end to end: `libra agent add` hook
//! install (pinned-binary provenance + user-config preservation) → real
//! non-interactive session → `agent session/checkpoint list`,
//! safe-summary `checkpoint show`, `session show`, `refs/libra/traces`,
//! `agent doctor` → §0.3.5 uninstall smoke (semantic config restore,
//! idempotent remove, captured data retained). Deterministic fixtures
//! cannot stand in for this target (plan.md A6.5 acceptance).
//!
//! CI-safe by construction: every test is `#[ignore]` **and** skips unless
//! `LIBRA_RUN_LOCAL_AGENTS=1`, so neither `cargo test --all` nor a stray
//! `--ignored` run can start a paid session. Tests are `#[serial]` and the
//! canonical invocation adds `--test-threads=1` (§0.3.6):
//!
//! ```bash
//! LIBRA_RUN_LOCAL_AGENTS=1 \
//! LIBRA_LOCAL_AGENT_SET=codex,claude-code,opencode \
//! cargo test --test agent_local_capture_smoke_test -- --ignored --test-threads=1
//! ```
//!
//! Knobs (plan.md §0.3.6): `LIBRA_LOCAL_AGENT_SET` (default all three),
//! `LIBRA_LOCAL_AGENT_TIMEOUT_SECS` (default 180; expiry kills the
//! child's whole **process group**), `LIBRA_KEEP_LOCAL_AGENT_SMOKE=1`
//! (keep evidence — sensitive, never commit),
//! `LIBRA_LOCAL_AGENT_EVIDENCE_DIR` (custom 0700 evidence root).
//!
//! A missing binary or a failed read-only login probe marks that agent's
//! run **BLOCKED** without starting a paid session (§0 blocked rule) and
//! **fails the test with a `BLOCKED` panic**: with the gate explicitly
//! open, blocked stays machine-distinguishable (exit code / pass count)
//! from a real green run and can never be faked green. The shared driver — and
//! the documented per-CLI isolation decisions (isolated `CODEX_HOME` for
//! codex; real `HOME` + project-local capture configs for claude and
//! opencode) — lives in `tests/harness/agent_local_capture.rs`.

#![cfg(unix)]

mod harness;

use harness::agent_local_capture as smoke;
use serial_test::serial;

/// Guard against `LIBRA_LOCAL_AGENT_SET` typos silently skipping an agent
/// (checked in the gated path only — unknown slugs are a hard error).
fn check_agent_set() {
    if std::env::var(smoke::GATE_ENV)
        .map(|v| v == "1")
        .unwrap_or(false)
    {
        let unknown = smoke::unknown_requested_agents();
        assert!(
            unknown.is_empty(),
            "unknown agent slug(s) in {}: {unknown:?} (known: codex, claude-code, opencode)",
            smoke::SET_ENV
        );
    }
}

#[test]
#[ignore = "drives a real paid codex session; set LIBRA_RUN_LOCAL_AGENTS=1 and run with \
            --ignored --test-threads=1 (plan.md §0.3.6)"]
#[serial(cwd, env, hash_kind)]
fn local_capture_smoke_codex() {
    check_agent_set();
    smoke::run_slug("codex");
}

#[test]
#[ignore = "drives a real paid claude session; set LIBRA_RUN_LOCAL_AGENTS=1 and run with \
            --ignored --test-threads=1 (plan.md §0.3.6)"]
#[serial(cwd, env, hash_kind)]
fn local_capture_smoke_claude_code() {
    check_agent_set();
    smoke::run_slug("claude-code");
}

#[test]
#[ignore = "drives a real paid opencode session; set LIBRA_RUN_LOCAL_AGENTS=1 and run with \
            --ignored --test-threads=1 (plan.md §0.3.6)"]
#[serial(cwd, env, hash_kind)]
fn local_capture_smoke_opencode() {
    check_agent_set();
    smoke::run_slug("opencode");
}

#[cfg(target_os = "macos")]
#[test]
#[ignore = "drives a real paid opencode session; set LIBRA_RUN_LOCAL_AGENTS=1 and run with \
            --ignored --test-threads=1 (plan.md §0.3.6)"]
#[serial(cwd, env, hash_kind)]
fn local_capture_smoke_opencode_macos() {
    check_agent_set();
    smoke::run_slug("opencode");
}

#[cfg(target_os = "macos")]
#[test]
fn opencode_project_preflight_requires_disposable_directory() {
    use std::{ffi::OsStr, path::Path};

    assert_eq!(
        smoke::OPENCODE_SMOKE_MODEL,
        "volcengine-agent-plan/glm-5.3-flash"
    );
    let args = smoke::opencode_smoke_args();
    assert_eq!(args.first().map(String::as_str), Some("run"));
    assert!(args.iter().any(|arg| arg == "--standalone"));
    assert!(!args.iter().any(|arg| arg == "--dir"));
    assert!(
        args.windows(2)
            .any(|pair| { pair[0] == "--model" && pair[1] == smoke::OPENCODE_SMOKE_MODEL })
    );
    assert_eq!(
        args[9..].join(" "),
        smoke::OPENCODE_SMOKE_PROMPT,
        "the pinned CLI must receive the exact prompt without added quotes"
    );
    assert!(args[9..].iter().all(|part| !part.contains(' ')));
    let root = tempfile::tempdir().expect("create smoke fixture");
    let repo = root.path().join("repo");
    std::fs::create_dir(&repo).expect("create disposable project");
    let mut command = std::process::Command::new("opencode");
    let project = smoke::configure_opencode_project(&mut command, &repo);
    assert_eq!(command.get_current_dir(), Some(project.as_path()));
    assert_eq!(
        command
            .get_envs()
            .find(|(key, _)| *key == OsStr::new("PWD"))
            .and_then(|(_, value)| value),
        Some(project.as_os_str()),
        "the OpenCode 2.0.26 run command prefers PWD over cwd"
    );
    let response = serde_json::json!({ "directory": project });
    assert!(smoke::opencode_location_matches_project(
        &serde_json::to_vec(&response).expect("serialize location"),
        &project
    ));
    let foreign = serde_json::json!({ "directory": root.path() });
    assert!(!smoke::opencode_location_matches_project(
        &serde_json::to_vec(&foreign).expect("serialize foreign location"),
        &project
    ));
    assert!(!smoke::opencode_location_matches_project(
        br#"{"project":{"directory":"/"}}"#,
        &project
    ));
    assert!(!smoke::opencode_location_matches_project(
        b"not JSON",
        Path::new(&project)
    ));
}

/// Keep Linux's original A6.5 invocation while the separate macOS gate
/// requires content. This is a source guard, never Linux operator evidence.
#[test]
fn linux_a65_criteria_unchanged() {
    let harness = include_str!("harness/agent_local_capture.rs");
    assert!(harness.contains("Linux A6.5 remains"));
    assert!(
        harness.contains("#[cfg(not(target_os = \"macos\"))]\n            \"opencode\" => vec![")
    );
    let smoke = include_str!("agent_local_capture_smoke_test.rs");
    let original = [
        "fn ",
        &["local_capture", "smoke", "opencode"].join("_"),
        "(",
    ]
    .concat();
    assert!(smoke.contains(&original));
    let call = ["smoke::run_slug(", r#""opencode")"#].concat();
    assert!(smoke.contains(&call));
    assert!(harness.contains("assert_opencode_content_capture("));
}

#[cfg(target_os = "macos")]
#[test]
fn opencode_login_probe_timeout_is_not_success() {
    use std::{os::unix::fs::PermissionsExt, process::Command, time::Duration};

    let root = tempfile::tempdir().expect("create probe fixture");
    let stdout = root.path().join("stdout");
    let stderr = root.path().join("stderr");
    let mut command = Command::new("/bin/sh");
    command.args(["-c", "sleep 30"]);
    let result = smoke::run_child_with_timeout(
        command,
        &stdout,
        &stderr,
        Duration::from_millis(200),
        "synthetic login probe",
    );
    assert!(result.timed_out);
    assert!(!result.exit_ok);
    assert_eq!(result.exit_code, None);
    for path in [stdout, stderr] {
        assert_eq!(
            std::fs::metadata(path)
                .expect("private probe output")
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
    }
}
