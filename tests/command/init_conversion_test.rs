//! Issue #487: an interrupted `init --from-git-repository` must either roll
//! back or stay recoverable, and must never be presented as an ordinary empty
//! repository.
//!
//! Layer: L1 (deterministic; tempdir + isolated HOME, no network). The
//! interruption is injected through the conversion failpoints (plan
//! `issues/487.md`, ADR-IG-05), which are inert unless `LIBRA_TEST=1` — the
//! harness always sets it for child processes.

use std::{fs, path::Path, process::Command};

use super::{assert_cli_success, run_libra_command, run_libra_command_with_env};

/// Create a Git source repository with one commit; returns the source dir.
fn git_source(root: &Path) -> std::path::PathBuf {
    let source = root.join("src");
    fs::create_dir_all(&source).expect("create source");
    let run = |args: &[&str]| {
        let output = Command::new("git")
            .current_dir(&source)
            .args(args)
            .env("GIT_AUTHOR_NAME", "T")
            .env("GIT_AUTHOR_EMAIL", "t@example.com")
            .env("GIT_COMMITTER_NAME", "T")
            .env("GIT_COMMITTER_EMAIL", "t@example.com")
            .output()
            .expect("run git");
        assert!(
            output.status.success(),
            "git {args:?} failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    };
    run(&["init", "-q", "."]);
    fs::write(source.join("a.txt"), "a\n").expect("write a.txt");
    run(&["add", "a.txt"]);
    run(&["commit", "-q", "-m", "c1"]);
    source
}

/// A target directory with a user file and (optionally) a Git repository, i.e.
/// the realistic "convert in place" shape.
fn target_dir(root: &Path, with_git: bool) -> std::path::PathBuf {
    let target = root.join("target");
    fs::create_dir_all(&target).expect("create target");
    fs::write(target.join("keep.txt"), "keep\n").expect("write keep.txt");
    if with_git {
        let output = Command::new("git")
            .current_dir(&target)
            .args(["init", "-q", "."])
            .output()
            .expect("git init");
        assert!(output.status.success(), "git init target");
    }
    target
}

fn convert(target: &Path, source: &Path, extra_env: &[(&str, &str)]) -> std::process::Output {
    let source = source.to_string_lossy().to_string();
    run_libra_command_with_env(
        &[
            "init",
            "--vault",
            "false",
            "--from-git-repository",
            &source,
            ".",
        ],
        target,
        extra_env,
    )
}

/// M-FAIL: an ordinary conversion error rolls the run's target state back, so
/// the target is immediately retryable and the user file survives.
#[test]
fn init_conversion_failure_rolls_back_and_is_retryable() {
    let root = tempfile::tempdir().expect("tempdir");
    let source = git_source(root.path());
    let target = target_dir(root.path(), true);

    let failed = convert(
        &target,
        &source,
        &[("LIBRA_TEST_INIT_CONVERT_FAIL_AT", "after-database")],
    );
    assert!(
        !failed.status.success(),
        "the injected failure must fail the conversion: {}",
        String::from_utf8_lossy(&failed.stderr)
    );
    assert!(
        String::from_utf8_lossy(&failed.stderr).contains("injected Git conversion failure"),
        "the injected failure must be reported: {}",
        String::from_utf8_lossy(&failed.stderr)
    );
    assert!(
        !target.join(".libra").exists(),
        "a rolled-back conversion must not leave a storage root behind"
    );
    assert!(
        target.join("keep.txt").exists(),
        "the user file must survive the rollback"
    );
    assert!(
        target.join(".git").exists(),
        "the source history must survive the rollback"
    );

    // The target is retryable immediately.
    let retried = convert(&target, &source, &[]);
    assert_cli_success(&retried, "retry after rollback");
    let log = run_libra_command(&["log", "--oneline"], &target);
    assert!(
        log.status.success(),
        "the retry must import history: {}",
        String::from_utf8_lossy(&log.stderr)
    );
}

/// M-FAIL: a hard kill leaves the conversion marker; every other command
/// refuses to treat the residue as an ordinary empty repository, and a retry
/// moves the residue aside (never deleting it) and completes.
#[test]
fn init_conversion_hard_kill_is_detected_and_recovered() {
    let root = tempfile::tempdir().expect("tempdir");
    let source = git_source(root.path());
    let target = target_dir(root.path(), true);

    // The abort failpoint kills the process with SIGABRT, like an OOM kill.
    let killed = convert(
        &target,
        &source,
        &[("LIBRA_TEST_INIT_CONVERT_ABORT_AT", "after-database")],
    );
    assert!(
        !killed.status.success(),
        "the abort failpoint must kill the conversion"
    );
    let marker = target.join(".libra").join("CONVERSION_IN_PROGRESS");
    assert!(
        marker.exists(),
        "a hard kill must leave the conversion marker behind"
    );

    // Ordinary commands refuse with the recovery guidance.
    let status = run_libra_command(&["status"], &target);
    assert_eq!(
        status.status.code(),
        Some(128),
        "status on a half-converted repository must fail: {}",
        String::from_utf8_lossy(&status.stdout)
    );
    let stderr = String::from_utf8_lossy(&status.stderr);
    assert!(
        stderr.contains("incomplete Git conversion") || stderr.contains("LBR-REPO-003"),
        "the refusal must name the incomplete conversion: {stderr}"
    );

    // A retry moves the residue aside and converts.
    let retried = convert(&target, &source, &[]);
    assert_cli_success(&retried, "retry after a hard kill");
    assert!(!marker.exists(), "a completed retry must clear the marker");
    let residue: Vec<_> = fs::read_dir(&target)
        .expect("read target")
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .filter(|name| name.starts_with(".libra.incomplete-"))
        .collect();
    assert_eq!(
        residue.len(),
        1,
        "the interrupted storage root must be preserved under one aside name: {residue:?}"
    );
    let log = run_libra_command(&["log", "--oneline"], &target);
    assert!(
        log.status.success(),
        "the recovered repository must have history: {}",
        String::from_utf8_lossy(&log.stderr)
    );
}

/// M-FAIL: a marker owned by a LIVE process blocks the retry instead of
/// racing it (ADR-IG-03 item 2).
#[test]
fn init_conversion_retry_refuses_a_live_owner() {
    use libra::command::init_conversion::{CONVERSION_MARKER, ConversionGuard};

    let root = tempfile::tempdir().expect("tempdir");
    let source = git_source(root.path());
    let target = target_dir(root.path(), true);
    fs::create_dir_all(target.join(".libra")).expect("mkdir .libra");
    // Use the production writer so the current process is provably alive in
    // both the legacy Linux/Windows and compatible macOS marker formats.
    let canonical_source = source.canonicalize().expect("source path").join(".git");
    let _guard = ConversionGuard::begin(&target.join(".libra"), &canonical_source)
        .expect("write live marker");
    assert!(target.join(".libra").join(CONVERSION_MARKER).exists());

    let refused = convert(&target, &source, &[]);
    assert!(
        !refused.status.success(),
        "a live conversion must refuse the retry"
    );
    let stderr = String::from_utf8_lossy(&refused.stderr);
    assert!(
        stderr.contains("already running") || stderr.contains("LBR-CONFLICT-002"),
        "the refusal must name the live conversion: {stderr}"
    );
    assert!(
        target.join(".libra").join(CONVERSION_MARKER).exists(),
        "a refused retry must leave the marker untouched"
    );
}

#[test]
fn init_conversion_unstamped_marker_explains_manual_recovery() {
    use libra::command::init_conversion::{
        CONVERSION_MARKER, ConversionMarker, ConversionOwner, MARKER_VERSION, write_marker,
    };

    let root = tempfile::tempdir().expect("tempdir");
    let source = git_source(root.path());
    let target = target_dir(root.path(), false);
    let initialized = run_libra_command(&["init", "--vault", "false", "."], &target);
    assert_cli_success(&initialized, "initialize marker fixture");
    let storage = target.join(".libra");
    let marker = ConversionMarker {
        version: MARKER_VERSION,
        source: source
            .join(".git")
            .canonicalize()
            .expect("source .git")
            .display()
            .to_string(),
        started_at: 0,
        owner: ConversionOwner {
            pid: u32::MAX,
            start_time: None,
        },
        owner_start_time_v2: None,
        owner_boot_session: None,
        created: Vec::new(),
    };
    write_marker(&storage, &marker).expect("write marker");

    let status = run_libra_command(&["status"], &target);
    let status_error = String::from_utf8_lossy(&status.stderr);
    assert_eq!(status.status.code(), Some(128));
    assert!(status_error.contains("LBR-REPO-003"), "{status_error}");
    assert!(
        status_error.contains("confirm no conversion is running on any host"),
        "{status_error}"
    );

    let retry = convert(&target, &source, &[]);
    let retry_error = String::from_utf8_lossy(&retry.stderr);
    assert_eq!(retry.status.code(), Some(128));
    assert!(retry_error.contains("LBR-REPO-003"), "{retry_error}");
    assert!(
        retry_error.contains("confirm no conversion is running on any host"),
        "{retry_error}"
    );
    assert!(storage.join(CONVERSION_MARKER).exists());
}

/// M-FAIL: a marker recorded for a DIFFERENT source is refused rather than
/// silently re-using another conversion's residue.
#[test]
fn init_conversion_retry_refuses_a_source_mismatch() {
    use libra::command::init_conversion::{ConversionGuard, read_marker, write_marker};

    let root = tempfile::tempdir().expect("tempdir");
    let source = git_source(root.path());
    let target = target_dir(root.path(), true);
    fs::create_dir_all(target.join(".libra")).expect("mkdir .libra");
    let storage = target.join(".libra");
    let _guard = ConversionGuard::begin(&storage, Path::new("/somewhere/else/.git"))
        .expect("write source-mismatch marker");
    let mut marker = read_marker(&storage)
        .expect("marker exists")
        .expect("parse marker");
    marker.owner.pid = 999_999;
    write_marker(&storage, &marker).expect("write dead-owner marker");

    let refused = convert(&target, &source, &[]);
    assert!(
        !refused.status.success(),
        "a source mismatch must refuse the retry"
    );
    let stderr = String::from_utf8_lossy(&refused.stderr);
    assert!(
        stderr.contains("interrupted conversion") || stderr.contains("LBR-REPO-003"),
        "the refusal must explain the mismatch: {stderr}"
    );
}
