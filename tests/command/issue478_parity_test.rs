//! Issue #478 parity guards: log/show/diff/grep/blame/notes/reflog families.
//!
//! Each test pins one previously-divergent surface so a regression is caught:
//! `log --notes`/`%N`, `show ^{tree}`, `diff-tree` raw plumbing, `log`/`rev-list`
//! `--grep` dialect, `shortlog` ranges, bare `reflog` flags, `apply` writing,
//! `diff --dirstat`, `blame -M/-C/--date`, `rev-list --no-walk/--disk-usage`,
//! and `fast-import <file> --dry-run`.
//!
//! **Layer:** L1 — deterministic, no external dependencies.

use std::fs;

use tempfile::TempDir;

use super::{assert_cli_success, init_repo_via_cli, run_libra_command};

/// Two-commit repo with `f.txt` (`a` then `a\nb`).
fn repo_with_two_commits() -> TempDir {
    let repo = tempfile::tempdir().expect("tempdir");
    init_repo_via_cli(repo.path());
    assert_cli_success(
        &run_libra_command(&["config", "user.name", "Test User"], repo.path()),
        "user.name",
    );
    assert_cli_success(
        &run_libra_command(&["config", "user.email", "test@example.com"], repo.path()),
        "user.email",
    );
    fs::write(repo.path().join("f.txt"), "a\n").unwrap();
    assert_cli_success(&run_libra_command(&["add", "f.txt"], repo.path()), "add");
    assert_cli_success(
        &run_libra_command(&["commit", "-m", "c1", "--no-verify"], repo.path()),
        "commit c1",
    );
    fs::write(repo.path().join("f.txt"), "a\nb\n").unwrap();
    assert_cli_success(&run_libra_command(&["add", "f.txt"], repo.path()), "add2");
    assert_cli_success(
        &run_libra_command(&["commit", "-m", "c2", "--no-verify"], repo.path()),
        "commit c2",
    );
    repo
}

#[test]
fn show_accepts_tree_peel_suffix() {
    let repo = repo_with_two_commits();
    let out = run_libra_command(&["show", "HEAD^{tree}"], repo.path());
    assert_eq!(out.status.code(), Some(0), "show ^{{tree}} must succeed");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("f.txt"), "tree listing expected: {stdout}");
}

#[test]
fn log_notes_renders_block_and_percent_n() {
    let repo = repo_with_two_commits();
    assert_cli_success(
        &run_libra_command(&["notes", "add", "-m", "reviewed", "HEAD"], repo.path()),
        "notes add",
    );

    let out = run_libra_command(&["log", "-1", "--notes"], repo.path());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("Notes:"), "Notes block expected: {stdout}");
    assert!(stdout.contains("reviewed"));

    let out = run_libra_command(&["log", "-1", "--format=[%N]"], repo.path());
    assert!(String::from_utf8_lossy(&out.stdout).contains("reviewed"));

    // `--no-notes` countermands the block.
    let out = run_libra_command(&["log", "-1", "--notes", "--no-notes"], repo.path());
    assert!(!String::from_utf8_lossy(&out.stdout).contains("Notes:"));
}

#[test]
fn log_grep_defaults_to_literal_and_accepts_dialects() {
    let repo = tempfile::tempdir().expect("tempdir");
    init_repo_via_cli(repo.path());
    assert_cli_success(
        &run_libra_command(&["config", "user.name", "Test User"], repo.path()),
        "user.name",
    );
    assert_cli_success(
        &run_libra_command(&["config", "user.email", "test@example.com"], repo.path()),
        "user.email",
    );
    fs::write(repo.path().join("f.txt"), "x\n").unwrap();
    assert_cli_success(&run_libra_command(&["add", "f.txt"], repo.path()), "add");
    assert_cli_success(
        &run_libra_command(&["commit", "-m", "c++ (fix)", "--no-verify"], repo.path()),
        "commit",
    );

    // BRE default: `+` and parentheses are literal.
    let out = run_libra_command(&["log", "--oneline", "--grep", "c++ (fix)"], repo.path());
    assert_eq!(out.status.code(), Some(0));
    assert!(String::from_utf8_lossy(&out.stdout).contains("c++ (fix)"));

    // `--fixed-strings` makes `.` literal.
    let out = run_libra_command(
        &["log", "--oneline", "--fixed-strings", "--grep", "c++ (fix)"],
        repo.path(),
    );
    assert!(String::from_utf8_lossy(&out.stdout).contains("c++ (fix)"));

    // Invalid extended regex exits 129.
    let out = run_libra_command(
        &["log", "--extended-regexp", "--grep", "c++ (fix"],
        repo.path(),
    );
    assert_eq!(out.status.code(), Some(129));
}

#[test]
fn diff_tree_defaults_to_raw_and_supports_modes() {
    let repo = repo_with_two_commits();
    let out = run_libra_command(&["diff-tree", "HEAD~1", "HEAD"], repo.path());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.starts_with(":100644"),
        "diff-tree default must be raw: {stdout}"
    );

    let out = run_libra_command(&["diff-tree", "-p", "HEAD~1", "HEAD"], repo.path());
    assert!(String::from_utf8_lossy(&out.stdout).starts_with("diff --git"));

    let out = run_libra_command(&["diff-tree", "--name-only", "HEAD~1", "HEAD"], repo.path());
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "f.txt");

    let out = run_libra_command(&["diff-tree", "-s", "HEAD~1", "HEAD"], repo.path());
    assert!(out.stdout.is_empty());
}

#[test]
fn shortlog_accepts_range() {
    let repo = repo_with_two_commits();
    let out = run_libra_command(&["shortlog", "HEAD~1..HEAD"], repo.path());
    assert_eq!(out.status.code(), Some(0), "range must resolve");
    assert!(String::from_utf8_lossy(&out.stdout).contains("c2"));
}

#[test]
fn reflog_bare_flags_and_subcommands_work() {
    let repo = repo_with_two_commits();
    let out = run_libra_command(&["reflog", "-1"], repo.path());
    assert_eq!(out.status.code(), Some(0), "bare -1 must parse");
    assert!(String::from_utf8_lossy(&out.stdout).contains("HEAD@{0}"));

    let out = run_libra_command(&["reflog", "--date=iso", "-n", "1"], repo.path());
    assert_eq!(out.status.code(), Some(0));

    let out = run_libra_command(&["reflog", "--grep", "c2"], repo.path());
    assert_eq!(out.status.code(), Some(0));

    assert_eq!(
        run_libra_command(&["reflog", "exists", "HEAD"], repo.path())
            .status
            .code(),
        Some(0)
    );
}

#[test]
fn apply_writes_and_check_does_not() {
    let repo = repo_with_two_commits();
    let patch = "--- a/f.txt\n+++ b/f.txt\n@@ -1,2 +1,3 @@\n a\n b\n+c\n";
    fs::write(repo.path().join("p.diff"), patch).unwrap();

    let check = run_libra_command(&["apply", "--check", "p.diff"], repo.path());
    assert_eq!(check.status.code(), Some(0));
    assert_eq!(
        fs::read_to_string(repo.path().join("f.txt")).unwrap(),
        "a\nb\n"
    );

    let applied = run_libra_command(&["apply", "p.diff"], repo.path());
    assert_eq!(applied.status.code(), Some(0));
    assert_eq!(
        fs::read_to_string(repo.path().join("f.txt")).unwrap(),
        "a\nb\nc\n"
    );

    // Reverse restores.
    let reversed = run_libra_command(&["apply", "-R", "p.diff"], repo.path());
    assert_eq!(reversed.status.code(), Some(0));
    assert_eq!(
        fs::read_to_string(repo.path().join("f.txt")).unwrap(),
        "a\nb\n"
    );
}

#[test]
fn apply_stat_lists_without_writing() {
    let repo = repo_with_two_commits();
    let patch = "--- a/f.txt\n+++ b/f.txt\n@@ -1,2 +1,3 @@\n a\n b\n+c\n";
    fs::write(repo.path().join("p.diff"), patch).unwrap();
    let out = run_libra_command(&["apply", "--stat", "p.diff"], repo.path());
    assert_eq!(out.status.code(), Some(0));
    assert!(String::from_utf8_lossy(&out.stdout).contains("f.txt"));
    assert_eq!(
        fs::read_to_string(repo.path().join("f.txt")).unwrap(),
        "a\nb\n"
    );
}

#[test]
fn diff_dirstat_reports_directory_share() {
    let repo = tempfile::tempdir().expect("tempdir");
    init_repo_via_cli(repo.path());
    assert_cli_success(
        &run_libra_command(&["config", "user.name", "Test User"], repo.path()),
        "user.name",
    );
    assert_cli_success(
        &run_libra_command(&["config", "user.email", "test@example.com"], repo.path()),
        "user.email",
    );
    fs::create_dir_all(repo.path().join("sub")).unwrap();
    fs::write(repo.path().join("sub/f.txt"), "a\n").unwrap();
    assert_cli_success(&run_libra_command(&["add", "-A"], repo.path()), "add");
    assert_cli_success(
        &run_libra_command(&["commit", "-m", "c1", "--no-verify"], repo.path()),
        "commit",
    );
    fs::write(repo.path().join("sub/f.txt"), "a\nb\n").unwrap();
    assert_cli_success(&run_libra_command(&["add", "-A"], repo.path()), "add2");
    assert_cli_success(
        &run_libra_command(&["commit", "-m", "c2", "--no-verify"], repo.path()),
        "commit2",
    );

    let out = run_libra_command(&["diff", "--dirstat", "HEAD~1", "HEAD"], repo.path());
    assert_eq!(out.status.code(), Some(0));
    assert!(
        String::from_utf8_lossy(&out.stdout).contains("% sub/"),
        "dirstat expected: {}",
        String::from_utf8_lossy(&out.stdout)
    );
}

#[test]
fn blame_accepts_move_copy_and_date_flags() {
    let repo = repo_with_two_commits();
    let out = run_libra_command(&["blame", "-C", "--date=iso", "f.txt"], repo.path());
    assert_eq!(
        out.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        String::from_utf8_lossy(&out.stdout).contains("Test User"),
        "blame attribution expected"
    );

    // `--no-abbrev` prints the full hash; a negative `-L` end is accepted.
    let out = run_libra_command(&["blame", "--no-abbrev", "f.txt"], repo.path());
    let first_line = String::from_utf8_lossy(&out.stdout)
        .lines()
        .next()
        .unwrap_or("")
        .to_string();
    assert_eq!(
        first_line.split(' ').next().unwrap_or("").len(),
        40,
        "full hash expected: {first_line}"
    );

    let out = run_libra_command(&["blame", "-L", "1,-1", "f.txt"], repo.path());
    assert_eq!(out.status.code(), Some(0));
}

#[test]
fn rev_list_no_walk_and_disk_usage() {
    let repo = repo_with_two_commits();
    let out = run_libra_command(&["rev-list", "--no-walk", "HEAD~1", "HEAD"], repo.path());
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(String::from_utf8_lossy(&out.stdout).lines().count(), 2);

    let out = run_libra_command(&["rev-list", "--disk-usage=raw", "HEAD"], repo.path());
    assert_eq!(out.status.code(), Some(0));
    assert!(
        String::from_utf8_lossy(&out.stdout)
            .trim()
            .parse::<u64>()
            .is_ok()
    );
}

#[test]
fn fast_import_accepts_positional_file_and_dry_run() {
    let repo = repo_with_two_commits();
    let stream = repo.path().join("stream.fi");
    let out = run_libra_command(&["fast-export", "HEAD"], repo.path());
    assert_cli_success(&out, "fast-export");
    fs::write(&stream, &out.stdout).unwrap();

    let out = run_libra_command(
        &["fast-import", "--dry-run", stream.to_str().unwrap()],
        repo.path(),
    );
    assert_eq!(
        out.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );

    let out = run_libra_command(&["fast-import", stream.to_str().unwrap()], repo.path());
    assert_eq!(out.status.code(), Some(0));
}

#[test]
fn log_no_patch_suppresses_diff() {
    let repo = repo_with_two_commits();
    let out = run_libra_command(&["log", "-1", "-p", "--no-patch"], repo.path());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        !stdout.contains("diff --git"),
        "patch must be suppressed: {stdout}"
    );
}
