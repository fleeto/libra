//! Guard the optional live compatibility workflow shape.
//!
//! The live-cloud gate requires external cloud secrets, so it must remain
//! outside the base required-check workflow. This test locks the local
//! contract that `compat-live-cloud` is manual/scheduled, secret-gated, and
//! absent from `base.yml`; the former Code-era live AI job and its deleted
//! targets (plan-20260920) must not silently return.
//!
//! plan-20260927 G-3a: the live run is wrapped in
//! `tests/cloud_live_no_skip.sh <expected-count> ...` (fail-closed on test
//! skip markers, run/pass count drift, or a missing Nextest summary) with
//! `cargo nextest run --success-output immediate`; the pinned selected count
//! and the wrapper invocation are locked here so drift needs a conscious
//! two-place update.

use std::{fs, path::PathBuf};

#[test]
fn live_compat_workflow_is_optional_and_secret_gated() {
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let live = fs::read_to_string(repo.join(".github/workflows/live-compat.yml"))
        .expect("read .github/workflows/live-compat.yml");
    let base = fs::read_to_string(repo.join(".github/workflows/base.yml"))
        .expect("read .github/workflows/base.yml");

    for required in [
        "workflow_dispatch: {}",
        "schedule:",
        "name: compat-live-cloud",
        "LIBRA_D1_ACCOUNT_ID",
        "LIBRA_STORAGE_SECRET_KEY",
        "skip=true",
        "--features test-live-cloud",
        "--test cloud_storage_backup_test",
        "--test agent_cloud_tombstone_test",
        // plan-20260927 G-3a: the no-skip wrapper replaces the bare libtest
        // run; the pinned count must match the nextest JSON list selection.
        "tests/cloud_live_no_skip.sh 22",
        "cargo nextest run --features test-live-cloud",
        "--success-output immediate",
        "Retain live gate log",
    ] {
        assert!(
            live.contains(required),
            "live compatibility workflow is missing expected marker: {required}"
        );
    }

    for forbidden in ["pull_request:", "push:"] {
        assert!(
            !live.contains(forbidden),
            "live compatibility workflow must not run as a required PR/push gate: {forbidden}"
        );
    }

    // plan-20260920: the Code-era live AI job was removed together with the
    // `ai_agent_test` / `ai_chat_agent_test` targets and the `test-live-ai`
    // feature's only consumers.
    for forbidden in [
        "compat-live-ai",
        "DEEPSEEK_API_KEY",
        "--features test-live-ai",
        "ai_agent_test",
        "ai_chat_agent_test",
    ] {
        assert!(
            !live.contains(forbidden),
            "live compatibility workflow must not keep the removed Code-era live AI surface: {forbidden}"
        );
    }

    for live_job in ["compat-live-ai", "compat-live-cloud"] {
        assert!(
            !base.contains(live_job),
            "base.yml required-check workflow must not include optional live job {live_job}"
        );
    }
}

/// OG-15 preserves the existing safety CI while Linux operator acceptance
/// is handed to the user. This is a source guard, not Linux runtime proof.
#[test]
fn opencode_export_linux_job_uses_verified_userns_bubblewrap() {
    let base = fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".github/workflows/base.yml"),
    )
    .expect("read base safety workflow");
    let job = base
        .split("  opencode-export-linux:\n")
        .nth(1)
        .expect("the existing Linux safety job must remain present");
    for required in [
        "runs-on: ubuntu-24.04",
        "version=0.13.0",
        "https://github.com/containers/bubblewrap/releases/download/v${version}/bubblewrap-${version}.tar.xz",
        "4734237473c0e5d695e4e9034a34e43b2dbf5164655bd13fa59ae376b2b7a765",
        "sha256sum --check --status",
        "sudo install -o root -g root -m 0755",
        "sudo sysctl -w kernel.apparmor_restrict_unprivileged_userns=0",
        "test ! -w",
        "--bind-fd",
        "--ro-bind-fd",
        "--as-pid-1",
        "Run Linux export sandbox gates",
        "--include-ignored",
    ] {
        assert!(
            job.contains(required),
            "Linux safety job lost required boundary: {required}"
        );
    }
    assert!(
        job.find("sha256sum --check --status")
            .expect("digest check exists")
            < job
                .find("tar -C /tmp -xf")
                .expect("archive extraction exists"),
        "the source archive must be authenticated before extraction"
    );
    for forbidden in [
        "chmod u+s",
        "chmod 4755",
        "apt-get install -y bubblewrap",
        "sudo cargo test",
    ] {
        assert!(
            !job.contains(forbidden),
            "Linux safety job must not introduce {forbidden}"
        );
    }
}
