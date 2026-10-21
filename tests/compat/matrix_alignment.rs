mod matrix_alignment_support;

use std::{fs, process::Command};

use matrix_alignment_support::{
    assert_contains, cli_commands, command_development_public_commands,
    command_development_unpublished_docs, compatibility_commands, declared_cargo_targets,
    declared_features, plan_features, plan_test_targets, quarantine_tests, read_repo_file,
    repo_root,
};

#[test]
fn compatibility_matrix_matches_cli_commands() {
    let cli = cli_commands();
    let compat = compatibility_commands();
    let missing = cli.difference(&compat).cloned().collect::<Vec<_>>();
    let extra = compat.difference(&cli).cloned().collect::<Vec<_>>();

    assert!(
        missing.is_empty() && extra.is_empty(),
        "COMPATIBILITY.md top-level command matrix is out of sync with src/cli.rs::Commands.\nmissing from COMPATIBILITY.md: {missing:?}\nlisted in COMPATIBILITY.md but absent from src/cli.rs::Commands: {extra:?}"
    );
}

#[test]
fn send_email_policy_is_explicit_and_non_sending() {
    assert!(
        !cli_commands().contains("send-email"),
        "P2-04 / D19 requires send-email to remain absent until a reviewed transport RFC lands"
    );

    let outside_repo = tempfile::tempdir().expect("create no-repository send-email test directory");
    let output = Command::new(env!("CARGO_BIN_EXE_libra"))
        .args(["send-email", "--dry-run", "0001-example.patch"])
        .current_dir(outside_repo.path())
        .output()
        .expect("run libra send-email negative-path guard");
    assert_eq!(
        output.status.code(),
        Some(129),
        "an absent send-email command must use the stable CLI-error exit"
    );
    let diagnostic = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_contains(
        &diagnostic,
        "'send-email' is not a libra command",
        "send-email runtime rejection",
    );
    assert_contains(
        &diagnostic,
        "LBR-CLI-001",
        "send-email stable runtime error code",
    );

    let compat = read_repo_file("COMPATIBILITY.md");
    assert_contains(
        &compat,
        "| send-email | unsupported | P2-04 / D19 policy:",
        "COMPATIBILITY.md absent-command matrix",
    );
    assert_contains(
        &compat,
        "never reads `sendemail.*` configuration, SMTP credentials, or contacts a mail server",
        "COMPATIBILITY.md no-network boundary",
    );

    let user_doc = read_repo_file("docs/commands/send-email.md");
    for needle in [
        "Libra does not implement SMTP",
        "git send-email --dry-run",
        "LBR-CLI-001",
    ] {
        assert_contains(&user_doc, needle, "docs/commands/send-email.md");
    }

    let zh_user_doc = read_repo_file("docs/commands/zh-CN/send-email.md");
    for needle in [
        "Libra 不实现 SMTP 投递",
        "git send-email --dry-run",
        "LBR-CLI-001",
    ] {
        assert_contains(&zh_user_doc, needle, "docs/commands/zh-CN/send-email.md");
    }

    let development_doc = read_repo_file("docs/development/commands/send-email.md");
    assert_contains(
        &development_doc,
        "No `Commands::SendEmail` variant",
        "docs/development/commands/send-email.md",
    );
    assert_contains(
        &read_repo_file("docs/development/commands/_compatibility.md"),
        "### D19：`send-email` SMTP 传输",
        "send-email governance decision",
    );
}

#[test]
fn command_development_readme_matches_public_cli_surface() {
    let cli = cli_commands();
    let compat = compatibility_commands();
    let public_docs = command_development_public_commands();
    let unpublished_docs = command_development_unpublished_docs();

    let missing_from_docs = cli.difference(&public_docs).cloned().collect::<Vec<_>>();
    let extra_in_docs = public_docs.difference(&cli).cloned().collect::<Vec<_>>();
    let unpublished_but_public = unpublished_docs
        .intersection(&cli)
        .cloned()
        .collect::<Vec<_>>();
    let unpublished_but_compatible = unpublished_docs
        .intersection(&compat)
        .cloned()
        .collect::<Vec<_>>();

    assert!(
        missing_from_docs.is_empty()
            && extra_in_docs.is_empty()
            && unpublished_but_public.is_empty()
            && unpublished_but_compatible.is_empty(),
        "docs/development/commands/README.md command tables are out of sync with src/cli.rs::Commands and COMPATIBILITY.md.\nmissing public docs: {missing_from_docs:?}\nextra public docs: {extra_in_docs:?}\nunpublished docs exposed in CLI: {unpublished_but_public:?}\nunpublished docs exposed in COMPATIBILITY.md: {unpublished_but_compatible:?}"
    );

    for command in public_docs.union(&unpublished_docs) {
        // agent/code development docs live under docs/development/tracing/ since the
        // 932c3a0 reorganization; their README rows link there instead of this directory.
        let relative = if command == "agent" || command == "code" {
            format!("docs/development/tracing/{command}.md")
        } else {
            format!("docs/development/commands/{command}.md")
        };
        let path = repo_root().join(&relative);
        assert!(
            path.is_file(),
            "command development README links to missing document: {}",
            path.display()
        );
    }
}

#[test]
fn integration_test_plan_references_existing_targets_and_features() {
    let repo = repo_root();
    let cargo_targets = declared_cargo_targets();
    let missing_targets = plan_test_targets()
        .into_iter()
        .filter(|target| {
            !repo.join(format!("tests/{target}.rs")).is_file() && !cargo_targets.contains(target)
        })
        .collect::<Vec<_>>();
    let features = declared_features();
    let missing_features = plan_features()
        .into_iter()
        .filter(|feature| !features.contains(feature))
        .collect::<Vec<_>>();

    assert!(
        missing_targets.is_empty() && missing_features.is_empty(),
        "integration-test-plan.md references unknown targets or features\nunknown targets: {missing_targets:?}\nunknown features: {missing_features:?}"
    );

    for test in quarantine_tests() {
        let (target, test_fn) = test
            .split_once("::")
            .unwrap_or_else(|| panic!("quarantine test must use target::fn: {test}"));
        let path = repo.join(format!("tests/{target}.rs"));
        assert!(path.is_file(), "quarantine target file is missing: {test}");
        let body = fs::read_to_string(&path).unwrap_or_else(|error| {
            panic!("read {}: {error}", path.display());
        });
        assert!(
            body.contains(&format!("fn {test_fn}")),
            "quarantine test function is missing: {test}"
        );
    }
}

#[test]
fn docs_consistency_covers_code_command_router_contracts() {
    let code_doc = read_repo_file("docs/commands/code.md");
    let code_control_doc = read_repo_file("docs/commands/code-control.md");
    let zh_code_doc = read_repo_file("docs/commands/zh-CN/code.md");
    let workflow = read_repo_file(".github/workflows/base.yml");
    let compatibility = read_repo_file("COMPATIBILITY.md");
    let development_readme = read_repo_file("docs/development/commands/README.md");

    for (path, body) in [
        ("docs/commands/code.md", &code_doc),
        ("docs/commands/code-control.md", &code_control_doc),
        ("docs/commands/zh-CN/code.md", &zh_code_doc),
    ] {
        assert!(
            body.contains("## Examples") || body.contains("## Common Commands"),
            "{path} must keep an Examples / Common Commands heading"
        );
        assert!(
            !body.contains("/api/code"),
            "{path} must not document a live /api/code surface"
        );
    }
    assert_contains(&code_doc, "has been removed", "docs/commands/code.md");
    assert_contains(&code_doc, "libra agent", "docs/commands/code.md");
    assert_contains(
        &code_control_doc,
        "is now gone",
        "docs/commands/code-control.md",
    );

    assert!(
        !compatibility
            .lines()
            .any(|line| line.starts_with("| code |")),
        "COMPATIBILITY.md must not keep a live top-level `code` row"
    );
    assert!(
        !development_readme.contains("[`code`]("),
        "docs/development/commands/README.md must not list `code` as a public command"
    );

    assert_contains(
        &workflow,
        "Check Rustdoc intra-doc links",
        ".github/workflows/base.yml",
    );
    assert_contains(
        &workflow,
        "RUSTDOCFLAGS=\"-D rustdoc::broken_intra_doc_links\" cargo doc --no-deps --all-features",
        ".github/workflows/base.yml",
    );
    assert!(
        !workflow.contains("paths-ignore:"),
        ".github/workflows/base.yml must run the compatibility guards for workflow and command-documentation-only pull requests",
    );
}

#[test]
fn w203_revision_receipt_and_network_boundary_stay_aligned() {
    // RC-23: executor SCC (code.rs / web / runtime / workspace_snapshot)
    // is gone. Keep the surviving docs + session JSONL pins; do not
    // `read_repo_file` deleted sources.
    for gone in [
        "src/command/code.rs",
        "src/internal/ai/runtime/phase1.rs",
        "src/internal/ai/workspace_snapshot.rs",
        "src/internal/ai/web/sse_wire.rs",
        "src/internal/ai/web/web_admission.rs",
        "src/internal/ai/web/code_ui.rs",
        "src/internal/ai/web/headless.rs",
        "tests/ai_code_ui_wire_test.rs",
        "tests/ai_code_ui_headless_test.rs",
        "tests/code_ui_scenarios.rs",
    ] {
        assert!(
            !repo_root().join(gone).exists(),
            "{gone} must stay deleted after RC-23"
        );
    }

    let code_doc = read_repo_file("docs/commands/code.md");
    let zh_code_doc = read_repo_file("docs/commands/zh-CN/code.md");
    let error_codes = read_repo_file("docs/error-codes.md");
    let jsonl = read_repo_file("src/internal/ai/session/jsonl.rs");
    let compatibility = read_repo_file("COMPATIBILITY.md");

    for (path, body) in [
        ("docs/commands/code.md", &code_doc),
        ("docs/commands/zh-CN/code.md", &zh_code_doc),
    ] {
        assert!(
            body.contains("## Examples") || body.contains("## Common Commands"),
            "{path} must keep an Examples heading after the public Code CLI was removed"
        );
        assert!(
            !body.contains("/api/code")
                && !body.contains("PLAN_EXECUTION_NOT_AVAILABLE")
                && !body.contains("intent_revision"),
            "{path} must not keep a live Code product contract"
        );
    }
    assert_contains(&code_doc, "has been removed", "docs/commands/code.md");
    assert_contains(&zh_code_doc, "已移除", "docs/commands/zh-CN/code.md");

    assert_contains(
        &jsonl,
        "pub sidecar_digest: String",
        "digest-only IntentRevisionRecovery schema",
    );
    assert_contains(
        &jsonl,
        "intent_revision_consumption: Option<IntentRevisionConsumption>",
        "additive consumption receipt schema",
    );
    for needle in [
        "file.try_lock()",
        "options.custom_flags(libc::O_NOFOLLOW)",
        "code_workflow_append_lock_path_matches_file",
        "latest_recoverable_intent_revision_attempt_before_claim",
        "validated_intent_revision_consumption_receipts(replay)",
        "struct ValidatedIntentRevisionReceiptIndex",
        "fn source_terminals(",
        "fn exact_source_terminal(",
        "fn receipts(",
        "fn exact_receipt_for_source(",
        "fn committed_consumer_status(",
        "fn claimed_intent_revision_consumer_status(",
    ] {
        assert_contains(&jsonl, needle, "W2-03 crash-recovery authority");
    }

    assert!(
        !compatibility
            .lines()
            .any(|line| line.starts_with("| code |")),
        "COMPATIBILITY.md must not keep a public `| code |` row after the Code CLI was removed"
    );

    for (needle, description) in [
        (
            "| `PLAN_EXECUTION_NOT_AVAILABLE` | `409` |",
            "docs/error-codes.md public 409 row",
        ),
        (
            "| `PHASE1_WORKSPACE_CHANGED` | `409` |",
            "docs/error-codes.md drift row",
        ),
        (
            "| `PLAN_REVISION_NOTE_REQUIRED` | `400` |",
            "docs/error-codes.md revision-note row",
        ),
        (
            "| `SESSION_BUSY` | `409` |",
            "docs/error-codes.md revision admission row",
        ),
        (
            "| `INVALID_QUERY_PARAM` | `400` |",
            "docs/error-codes.md revision size row",
        ),
    ] {
        assert_contains(&error_codes, needle, description);
    }
}

#[test]
fn web_build_job_enforces_ignored_static_export_lifecycle() {
    // plan-20260920 RC-30: the `web-check`/`compat-web-check` job was removed
    // with the Code UI surface, so the guard now pins the removal instead of
    // the retired static-export lifecycle.
    let workflow = read_repo_file(".github/workflows/base.yml");
    for forbidden in [
        "compat-web-check",
        "test -f web/out/index.html",
        "git ls-files --error-unmatch -- web/out",
        "git check-ignore -q web/out/index.html",
    ] {
        assert!(
            !workflow.contains(forbidden),
            ".github/workflows/base.yml must not keep the removed web-check step: {forbidden}"
        );
    }
    assert!(
        !repo_root().join("scripts").exists(),
        "scripts directory should be removed"
    );
}

#[test]
fn lfs_compatibility_docs_use_current_attributes_filename() {
    for path in [
        "COMPATIBILITY.md",
        "docs/development/commands/_compatibility.md",
        "docs/development/commands/_compatibility.md",
    ] {
        let body = read_repo_file(path);
        assert!(
            body.contains(".libra_attributes"),
            "{path} must mention the current Libra attributes filename"
        );
        assert!(
            !body.contains(".libraattributes"),
            "{path} must not mention the retired .libraattributes spelling"
        );
    }
}

#[test]
fn compatibility_governance_roadmap_marks_current_surfaces_without_batch_status() {
    let governance = read_repo_file("docs/development/commands/_compatibility.md");

    for row in [
        "| merge | partial | partial | fast-forward, single-head three-way, and multi-head octopus merge supported; `-s ours`, `-X ours/theirs`, unrelated-history opt-in, and CLI/config merge shortlogs supported; other strategies/options deferred |",
        "| pull | partial | partial | fetch + fast-forward/three-way merge supported; `pull.rebase`/`branch.<name>.rebase`/`pull.ff` defaults are config-aware with local/global decryption, system-scope skip, and explicit unsupported diagnostics for interactive/rebase-merges modes; advanced strategy flags still partial |",
        "| push | partial | partial | branch/tag update, multi-refspec, delete, `--tags`, and `--mirror` supported; local file remote rejected intentionally |",
        "| checkout | partial | partial | visible branch compatibility surface including worktree-scoped `checkout -` previous-target toggling shared with `switch -`, `-b`/`-B <branch> [<start-point>]` symbolic-HEAD branch creation, `--orphan <branch>` unborn root branch creation (start-point currently rejected), plus explicit `checkout -- <path>` restoration alias; prefer `switch` / `restore` |",
    ] {
        assert!(
            governance.contains(row),
            "compatibility governance roadmap must retain completed row: {row}"
        );
    }

    for removed in ["批次状态", "C7", "C8", "C9", "C7-C9 后续补录"] {
        assert!(
            !governance.contains(removed),
            "governance roadmap must not retain batch status marker: {removed}"
        );
    }
}
// A successful Cargo process can select zero tests. Exercise the workflow's
// actual shell step with a private stub rather than accepting its exit alone.
fn macos_workflow_step(workflow: &str, name: &str) -> String {
    let job = workflow
        .split_once("  owner-liveness-macos:\n")
        .expect("macOS safety job must remain registered")
        .1
        .split_once("\n\n  scoped-input-windows:")
        .expect("macOS job must end before the independent Windows job")
        .0;
    assert!(job.contains("name: compat-owner-liveness-macos"));
    assert!(job.contains("runs-on: macos-latest"));
    let step = job
        .split_once(&format!("      - name: {name}\n"))
        .expect("named macOS safety step")
        .1
        .split_once("        run: |\n")
        .expect("bounded exact test step")
        .1;
    let step = step
        .split_once("\n      - name: ")
        .map_or(step, |(script, _)| script)
        .trim_end_matches('\n');
    step.lines()
        .map(|line| {
            if line.is_empty() {
                return line;
            }
            line.strip_prefix("          ")
                .expect("shell step indentation")
        })
        .collect::<Vec<_>>()
        .join("\n")
        + "\n"
}

fn macos_owner_safety_step(workflow: &str) -> String {
    macos_workflow_step(workflow, "Run current exact macOS owner safety tests")
}

fn macos_conversion_step(workflow: &str) -> String {
    macos_workflow_step(
        workflow,
        "Verify macOS conversion recovery and service compatibility",
    )
}

#[test]
fn macos_owner_liveness_workflow_selects_current_exact_tests() {
    let workflow = read_repo_file(".github/workflows/base.yml");
    let step = macos_owner_safety_step(&workflow);
    assert!(!step.contains("claim_owner_tests"));
    let fixtures = step
        .lines()
        .filter_map(|line| {
            line.trim()
                .strip_prefix('\'')
                .and_then(|name| name.strip_suffix('\''))
        })
        .collect::<Vec<_>>();
    assert_eq!(fixtures.len(), 13);
    assert_eq!(
        fixtures
            .iter()
            .copied()
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        13
    );
    let lease_source = read_repo_file("src/internal/operation/middleware/lease_tests.rs");
    let append_source = read_repo_file("src/internal/ai/session/jsonl.rs");
    for fixture in fixtures {
        let (module, name) = fixture.rsplit_once("::").expect("exact qualified case");
        let source = match module {
            "internal::operation::middleware::lease_tests" => &lease_source,
            "internal::ai::session::jsonl::tests" => &append_source,
            other => panic!("unreviewed macOS safety test module: {other}"),
        };
        assert!(
            source.contains(&format!("fn {name}(")),
            "missing real test {fixture}"
        );
        assert_ne!(
            name, "supervised_lease_child",
            "helper is not safety acceptance"
        );
    }
    assert!(step.contains("set -euo pipefail"));
    assert!(step.contains(
        "cargo test --locked --lib \"$fixture\" -- --exact --test-threads=1 --color never"
    ));
}

#[cfg(unix)]
#[test]
fn macos_owner_liveness_rejects_zero_wrong_ignored_and_failed_test_runs() {
    use std::{os::unix::fs::PermissionsExt, process::Command};

    let step = macos_owner_safety_step(&read_repo_file(".github/workflows/base.yml"));
    let dir = tempfile::tempdir().expect("private workflow fixture");
    let cargo = dir.path().join("cargo");
    fs::write(&cargo, r#"#!/bin/bash
set -eu
fixture="$4"
case "$CASE" in
  zero)
    printf 'running 0 tests\ntest result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 4089 filtered out; finished in 0.00s\n'
    exit 0 ;;
  wrong) fixture='some_other_test' ;;
  ignored)
    printf 'running 1 test\ntest %s ... ignored\ntest result: ok. 0 passed; 0 failed; 1 ignored; 0 measured; 4088 filtered out; finished in 0.00s\n' "$fixture"
    exit 0 ;;
esac
printf 'running 1 test\ntest %s ... ok\ntest result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 4088 filtered out; finished in 0.00s\n' "$fixture"
if [ "$CASE" = failed ]; then exit 9; fi
"#).expect("write private Cargo stub");
    fs::set_permissions(&cargo, fs::Permissions::from_mode(0o700)).expect("executable stub");
    let script = dir.path().join("step.sh");
    fs::write(&script, step).expect("workflow step fixture");
    let path = format!(
        "{}:{}",
        dir.path().display(),
        std::env::var("PATH").expect("host PATH")
    );
    for case in ["pass", "zero", "wrong", "ignored", "failed"] {
        let output = Command::new("/bin/bash")
            .arg(&script)
            .env_clear()
            .env("PATH", &path)
            .env("RUNNER_TEMP", dir.path())
            .env("CASE", case)
            .output()
            .expect("execute private workflow step");
        assert_eq!(
            output.status.success(),
            case == "pass",
            "workflow incorrectly accepted {case}: {}",
            String::from_utf8_lossy(&output.stdout)
        );
        if case == "pass" {
            assert_eq!(
                String::from_utf8_lossy(&output.stdout)
                    .lines()
                    .filter(|line| line.starts_with("MACOS_SAFETY_EXACT_PASS "))
                    .count(),
                13
            );
        }
    }
}

#[cfg(unix)]
#[test]
fn macos_conversion_step_rejects_zero_wrong_ignored_and_failed_test_runs() {
    use std::os::unix::fs::PermissionsExt;

    let step = macos_conversion_step(&read_repo_file(".github/workflows/base.yml"));
    for fixture in [
        "command::control_lock::tests::macos_pid_starttime_is_available_for_conversion_recovery",
        "command::init_conversion::tests::macos_marker_preserves_legacy_reader_safety_and_new_recovery",
        "command::init_conversion::tests::foreign_legacy_start_time_requires_manual_recovery",
        "command::service::tests::macos_service_sidecar_keeps_legacy_pid_only_liveness",
        "command::init_conversion_test::init_conversion_hard_kill_is_detected_and_recovered",
    ] {
        assert!(
            step.contains(fixture),
            "missing conversion fixture: {fixture}"
        );
    }

    let dir = tempfile::tempdir().expect("private conversion workflow fixture");
    let cargo = dir.path().join("cargo");
    fs::write(
        &cargo,
        r#"#!/bin/bash
set -eu
fixture=''
for arg in "$@"; do
  case "$arg" in command::*) fixture="$arg" ;; esac
done
case "$CASE" in
  zero)
    printf 'running 0 tests\ntest result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 4089 filtered out; finished in 0.00s\n'
    exit 0 ;;
  wrong) fixture='some_other_test' ;;
  ignored)
    printf 'running 1 test\ntest %s ... ignored\ntest result: ok. 0 passed; 0 failed; 1 ignored; 0 measured; 4088 filtered out; finished in 0.00s\n' "$fixture"
    exit 0 ;;
esac
printf 'running 1 test\ntest %s ... ok\ntest result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 4088 filtered out; finished in 0.00s\n' "$fixture"
if [ "$CASE" = failed ]; then exit 9; fi
"#,
    )
    .expect("write private Cargo stub");
    fs::set_permissions(&cargo, fs::Permissions::from_mode(0o700)).expect("executable stub");
    let script = dir.path().join("step.sh");
    fs::write(&script, step).expect("workflow step fixture");
    let path = format!(
        "{}:{}",
        dir.path().display(),
        std::env::var("PATH").expect("host PATH")
    );
    for case in ["pass", "zero", "wrong", "ignored", "failed"] {
        let output = Command::new("/bin/bash")
            .arg(&script)
            .env_clear()
            .env("PATH", &path)
            .env("CASE", case)
            .output()
            .expect("execute private conversion step");
        assert_eq!(
            output.status.success(),
            case == "pass",
            "conversion workflow incorrectly accepted {case}: {}",
            String::from_utf8_lossy(&output.stdout)
        );
        if case == "pass" {
            assert_eq!(
                String::from_utf8_lossy(&output.stdout)
                    .lines()
                    .filter(|line| line.starts_with("MACOS_CONVERSION_EXACT_PASS "))
                    .count(),
                5
            );
        }
    }
}
