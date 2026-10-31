//! SC-03: the promote subcommand stays deleted, so a dry-run cannot echo a prompt.
//!
//! Restoring the subcommand would regress RC-11. These tests drive the real
//! binary and assert that each fixture secret is absent from stdout and stderr.

use std::process::{Command, Output};

use tempfile::tempdir;

fn run(args: &[&str]) -> Output {
    let repo = tempdir().expect("tempdir");
    let home = repo.path().join("home");
    std::fs::create_dir_all(&home).expect("home");
    Command::new(env!("CARGO_BIN_EXE_libra"))
        .args(args)
        .current_dir(repo.path())
        .env_clear()
        .env("PATH", "/usr/bin:/bin:/usr/sbin:/sbin")
        .env("HOME", &home)
        .env("USERPROFILE", &home)
        .env("LIBRA_TEST", "1")
        .env("LANG", "C")
        .env("LC_ALL", "C")
        .output()
        .expect("run libra")
}

fn assert_secret_absent(output: &Output, secret: &str) {
    assert_ne!(
        output.status.code(),
        Some(0),
        "promote dry-run must be rejected, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stdout.contains(secret) && !stderr.contains(secret),
        "secret leaked.\nstdout={stdout}\nstderr={stderr}"
    );
    assert!(
        !stdout.contains("prompt     :") && !stderr.contains("prompt     :"),
        "dry-run echo format returned.\nstdout={stdout}\nstderr={stderr}"
    );
}

#[test]
fn default_human_len_only() {
    let secret = "ordinary prompt text for length check";
    let output = run(&["agent", "session", "promote", "--dry-run"]);
    assert_secret_absent(&output, secret);
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        !combined.contains("<len="),
        "rejected command must not format a length line: {combined}"
    );
}

#[test]
fn fixture_sk() {
    let secret = "sk-SC03FixtureValue";
    assert_secret_absent(&run(&["agent", "session", "promote", "--dry-run"]), secret);
}

#[test]
fn fixture_ghp() {
    let secret = "ghp_SC03FixtureValue";
    assert_secret_absent(&run(&["agent", "session", "promote", "--dry-run"]), secret);
}

#[test]
fn fixture_github_pat() {
    let secret = "github_pat_SC03FixtureValue";
    assert_secret_absent(&run(&["agent", "session", "promote", "--dry-run"]), secret);
}

#[test]
fn fixture_short_secret() {
    let secret = "s3cr3t";
    assert_eq!(secret.len(), 6);
    assert_secret_absent(&run(&["agent", "session", "promote", "--dry-run"]), secret);
}

#[test]
fn fixture_secret_at_start() {
    let secret = "sk-starts-the-prompt";
    assert_secret_absent(&run(&["agent", "session", "promote", "--dry-run"]), secret);
}

#[test]
fn fixture_unicode() {
    let secret = "密钥Δprompt";
    assert_secret_absent(&run(&["agent", "session", "promote", "--dry-run"]), secret);
}

#[test]
fn show_redacted_prompt() {
    let secret = "sk-shown-only-with-flag";
    let output = run(&[
        "agent",
        "session",
        "promote",
        "--dry-run",
        "--show-redacted-prompt",
    ]);
    assert_secret_absent(&output, secret);
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        !combined.contains("[REDACTED]"),
        "deleted command must not emit a redacted prompt body: {combined}"
    );
}

#[test]
fn json_default_len_string() {
    let secret = "json-default-prompt-body";
    let output = run(&["--json", "agent", "session", "promote", "--dry-run"]);
    assert_secret_absent(&output, secret);
}

#[test]
fn json_flag_redacted_string() {
    let secret = "json-flag-prompt-body";
    let output = run(&[
        "--json",
        "agent",
        "session",
        "promote",
        "--dry-run",
        "--show-redacted-prompt",
    ]);
    assert_secret_absent(&output, secret);
}
