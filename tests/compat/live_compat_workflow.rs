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
        "cancel-superseded-main.yml",
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

/// The Linux job retains the exporter sandbox's fail-closed unit gates. A
/// complete OpenCode CLI -> Session/Checkpoint bridge is operator-local.
/// This source guard is not Linux runtime or session-capture proof.
#[test]
fn opencode_export_linux_job_uses_verified_userns_bubblewrap() {
    let base = fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".github/workflows/base.yml"),
    )
    .expect("read base safety workflow");
    let job = base
        .split("  opencode-export-sandbox-linux:\n")
        .nth(1)
        .expect("the existing Linux safety job must remain present");
    assert!(
        !base.contains("opencode-export-linux")
            && !base.contains("--test agent_opencode_bridge_test"),
        "base workflow must name only the sandbox safety job and omit the OpenCode Session bridge"
    );
    for required in [
        "name: opencode-export-sandbox-linux",
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
        "--test agent_opencode_bridge_test",
        "Run OpenCode export bridge e2e",
    ] {
        assert!(
            !job.contains(forbidden),
            "Linux safety job must not introduce {forbidden}"
        );
    }
}

#[test]
fn opencode_session_capture_requires_local_opt_in() {
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let cargo = fs::read_to_string(repo.join("Cargo.toml")).expect("read Cargo.toml");
    assert!(cargo.contains(
        "name = \"agent_opencode_bridge_test\"\npath = \"tests/agent_opencode_bridge_test.rs\"\nrequired-features = [\"test-live-agent\"]"
    ));
    let bridge = fs::read_to_string(repo.join("tests/agent_opencode_bridge_test.rs"))
        .expect("read OpenCode bridge tests");

    let lifecycle = fs::read_to_string(repo.join("tests/agent_lifecycle_event_test.rs"))
        .expect("read lifecycle tests");
    let crash =
        fs::read_to_string(repo.join("tests/agent_hook_crash_test.rs")).expect("read crash tests");
    let live_pipeline =
        fs::read_to_string(repo.join("src/internal/ai/capture/live_pipeline_tests.rs"))
            .expect("read live pipeline tests");
    let live_oracle = fs::read_to_string(repo.join("src/internal/ai/capture/live_oracle_tests.rs"))
        .expect("read live checkpoint oracle tests");
    let import = fs::read_to_string(repo.join("tests/agent_import_test.rs"))
        .expect("read agent import tests");
    const MANUAL: &str =
        "#[ignore = \"OpenCode Session capture is an operator-local manual gate\"]";
    for name in [
        "opencode_historical_import_e2e_uses_trusted_export_bytes",
        "opencode_export_whole_session_idempotent",
        "opencode_export_durable_snapshot_uses_repository_keyed_hmac",
        "opencode_export_rejects_wrong_native_session_without_export_claim",
        "opencode_registered_failure_releases_dirty_job_claim_and_marker",
        "opencode_export_plaintext_never_in_persist_or_logs",
        "opencode_export_binary_trust_revalidates",
        "opencode_export_oversize_session_degrades",
        "opencode_export_macos_requires_trust_before_spawn",
    ] {
        let signature = format!("async fn {name}(");
        let at = bridge.find(&signature).expect("manual bridge test exists");
        let test_attribute = bridge[..at]
            .rfind("#[tokio::test]")
            .expect("manual bridge test has tokio test attribute");
        assert!(
            bridge[test_attribute..at].contains(MANUAL),
            "{name} must stay out of default CI execution"
        );
    }
    for (source, name) in [
        (&lifecycle, "hook_public_contract_byte_compat_matrix"),
        (&lifecycle, "opencode_status_idle_maps_turn_end"),
        (&lifecycle, "opencode_status_busy_retry_fails_closed"),
        (
            &lifecycle,
            "opencode_message_updated_is_single_envelope_turn_start",
        ),
        (&lifecycle, "opencode_legacy_alias_transition_forwarded"),
        (
            &lifecycle,
            "opencode_missing_legacy_prompt_fails_before_writes",
        ),
        (
            &lifecycle,
            "opencode_produced_frames_reach_real_ingress_and_survive_history_rollover",
        ),
        (&crash, "capture_coordinator_native_replay_matrix"),
    ] {
        let signature = format!("async fn {name}(");
        let at = source.find(&signature).expect("manual test exists");
        let test_attribute = source[..at]
            .rfind("#[tokio::test]")
            .expect("manual test has tokio test attribute");
        assert!(
            source[test_attribute..at].contains(MANUAL),
            "{name} must stay out of default CI execution"
        );
    }
    assert!(lifecycle.contains("async fn hook_public_contract_claude_codex_ci()"));
    assert!(crash.contains("async fn claude_codex_native_replay_stays_in_ci()"));
    let replay_ci = crash
        .split("async fn claude_codex_native_replay_stays_in_ci()")
        .nth(1)
        .and_then(|rest| {
            rest.split("async fn covered_turn_end_replay_keeps_cleanup_owned_marker_until_repair()")
                .next()
        })
        .expect("Claude/Codex-only replay matrix exists");
    assert!(
        !replay_ci.contains("provider: \"opencode\""),
        "default replay matrix must not run OpenCode"
    );
    assert!(live_oracle.contains("async fn live_checkpoint_metadata_shape_claude_codex_ci()"));
    assert!(live_oracle.contains("let cases: [(&str, &dyn HookProvider, [&str; 3]); 2]"));
    let oracle_ci = live_oracle
        .split("async fn live_checkpoint_metadata_shape_claude_codex_ci()")
        .nth(1)
        .expect("Claude/Codex-only live oracle exists");
    assert!(
        !oracle_ci.contains("opencode_provider()"),
        "default checkpoint metadata oracle must not run OpenCode"
    );
    assert!(
        !live_pipeline.contains("opencode__sample-reservation"),
        "default generic lease mapping must not reserve an OpenCode Session"
    );
    let oracle_at = live_oracle
        .find("async fn live_checkpoint_metadata_shape_is_stable()")
        .expect("three-provider live oracle exists");
    let oracle_test_attribute = live_oracle[..oracle_at]
        .rfind("#[tokio::test(flavor = \"current_thread\")]")
        .expect("three-provider live oracle has tokio test attribute");
    assert!(live_oracle[oracle_test_attribute..oracle_at].contains(MANUAL));
    let import_at = import
        .find("fn agent_import_confirmation_precedes_opencode_export()")
        .expect("OpenCode import preflight exists");
    let import_test_attribute = import[..import_at]
        .rfind("#[test]")
        .expect("OpenCode import preflight has test attribute");
    assert!(import[import_test_attribute..import_at].contains(MANUAL));
    for name in [
        "opencode_extraction_failure_continues_checkpoint",
        "export_stage_releases_runner_lease_on_every_exit",
    ] {
        let signature = format!("async fn {name}(");
        let at = live_pipeline
            .find(&signature)
            .expect("manual lib test exists");
        let test_attribute = live_pipeline[..at]
            .rfind("#[tokio::test(flavor = \"current_thread\")]")
            .expect("manual lib test has tokio test attribute");
        assert!(
            live_pipeline[test_attribute..at].contains(MANUAL),
            "{name} must stay out of default CI execution"
        );
    }
}
