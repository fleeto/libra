//! Tests `libra bisect` for finding the commit that introduced a regression.
//!
//! **Layer:** L1 — deterministic, no external dependencies.
//!
//! Fixture convention: each test sets up a fresh repo via
//! `setup_with_new_libra_in()`, configures a stable identity, and uses
//! `create_linear_commits(n)` to lay down a straight chain of commits whose
//! hashes are returned newest-first. The sub-state machine `BisectState` is
//! inspected directly to verify that `start`/`bad`/`good`/`skip`/`reset`
//! transitions write the expected on-disk state. CLI-level smoke tests at
//! the bottom run the binary outside or inside an empty repo to confirm
//! the user-visible failure behaviour.

use std::{fs, process::Command};

use libra::{
    cli::Bisect,
    command::{
        add::{self, AddArgs},
        bisect::{BisectState, execute_safe},
        commit,
    },
    internal::{branch::Branch, config::ConfigKv, head::Head},
    utils::{
        output::OutputConfig,
        test::{self, ChangeDirGuard},
    },
};
use serial_test::serial;
use tempfile::tempdir;

mod gitlink_preflight;

/// Run the Libra binary with an isolated HOME so host config never leaks into tests.
fn run_libra_command(args: &[&str], cwd: &std::path::Path) -> std::process::Output {
    let home = cwd.join(".libra-test-home");
    let config_home = home.join(".config");
    std::fs::create_dir_all(&config_home).expect("failed to create isolated config directory");

    Command::new(env!("CARGO_BIN_EXE_libra"))
        .args(args)
        .current_dir(cwd)
        .env_clear()
        .env("PATH", "/usr/bin:/bin:/usr/sbin:/sbin")
        .env("HOME", &home)
        .env("USERPROFILE", &home)
        .env("XDG_CONFIG_HOME", &config_home)
        .env("LANG", "C")
        .env("LC_ALL", "C")
        .env("LIBRA_TEST_ENV", "1")
        .output()
        .expect("failed to execute libra binary")
}

/// Initialize a repository through the CLI to exercise the real process entrypoint.
fn init_repo_via_cli(repo: &std::path::Path) {
    std::fs::create_dir_all(repo).expect("failed to create repository directory");
    let output = run_libra_command(&["init"], repo);
    assert!(
        output.status.success(),
        "failed to initialize repository: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

/// Configure test identity directly through the in-process config layer.
/// Required before any commit because Libra refuses to author without it.
async fn configure_identity() {
    ConfigKv::set("user.name", "Bisect Test", false)
        .await
        .unwrap();
    ConfigKv::set("user.email", "bisect@test.com", false)
        .await
        .unwrap();
}

/// Create a linear chain of `count` commits, each modifying `file.txt`.
///
/// Returns the commit hashes ordered newest-first: `hashes[0]` is HEAD and
/// `hashes[count - 1]` is the root commit. Assumes the caller already holds a
/// `ChangeDirGuard` rooted in a fresh repo.
async fn create_linear_commits(count: usize) -> Vec<String> {
    let mut hashes = Vec::new();

    for i in 0..count {
        test::ensure_file("file.txt", Some(&format!("content_{i}\n")));
        let pathspec = vec![String::from("file.txt")];

        add::execute(AddArgs {
            intent_to_add: false,
            sparse: false,
            pathspec,
            all: false,
            update: false,
            refresh: false,
            force: false,
            verbose: false,
            dry_run: false,
            ignore_errors: false,
            pathspec_from_file: None,
            pathspec_file_nul: false,
            chmod: None,
            renormalize: false,
            ignore_missing: false,
            resolved: false,
            patch: false,
            auto_advance: false,
            no_auto_advance: false,
        })
        .await;
        commit::execute(commit::CommitArgs {
            message: Some(format!("Commit {i}").to_string()),
            file: None,
            allow_empty: false,
            conventional: false,
            no_edit: false,
            amend: false,
            signoff: false,
            disable_pre: true,
            all: false,
            no_verify: false,
            author: None,
            ..Default::default()
        })
        .await;

        let hash = Head::current_commit().await.unwrap().to_string();
        hashes.push(hash);
    }

    // Reverse so newest is first (hashes[0] = latest, hashes[n-1] = oldest)
    hashes.reverse();
    hashes
}

/// Scenario: `bisect start` (no bounds) must transition the repo into the
/// `in_progress` state with empty `bad` and `good` slots. Pins the initial
/// state shape.
#[tokio::test]
#[serial(cwd)]
async fn test_bisect_start_creates_state() {
    let temp_path = tempdir().unwrap();
    test::setup_with_new_libra_in(temp_path.path()).await;
    let _guard = ChangeDirGuard::new(temp_path.path());

    configure_identity().await;

    // Create at least one commit
    create_linear_commits(1).await;

    // Start bisect
    let args = Bisect::Start {
        bad: None,
        good: vec![],
        good_opt: None,
        term_old: vec![],
        term_new: vec![],
        first_parent: false,
    };
    execute_safe(args, &OutputConfig::default()).await.unwrap();

    // Verify state was created
    assert!(BisectState::is_in_progress().await.unwrap());

    let state = BisectState::load().await.unwrap();
    assert!(state.bad.is_none());
    assert!(state.good.is_empty());
}

/// Scenario: `bisect start <bad> <good>` must record both bounds and
/// immediately check out a midpoint commit (`state.current` populated).
/// Confirms the binary search seeding behaviour.
#[tokio::test]
#[serial(cwd)]
async fn test_bisect_start_with_bad_and_good() {
    let temp_path = tempdir().unwrap();
    test::setup_with_new_libra_in(temp_path.path()).await;
    let _guard = ChangeDirGuard::new(temp_path.path());

    configure_identity().await;

    // Create 5 commits: hashes[0] = latest (Commit 4), hashes[4] = oldest (Commit 0)
    let hashes = create_linear_commits(5).await;

    // Start bisect with bad (latest) and good (oldest)
    let bad = hashes[0].clone(); // latest
    let good = hashes[4].clone(); // oldest

    let args = Bisect::Start {
        bad: Some(bad.clone()),
        good: vec![good.clone()],
        good_opt: None,
        term_old: vec![],
        term_new: vec![],
        first_parent: false,
    };
    execute_safe(args, &OutputConfig::default()).await.unwrap();

    let state = BisectState::load().await.unwrap();
    assert_eq!(state.bad.unwrap().to_string(), bad);
    assert_eq!(state.good[0].to_string(), good);

    // Should have checked out to a middle commit
    assert!(state.current.is_some());
}

/// Scenario: marking `bad` followed by `good` on a 3-commit chain narrows
/// the search to the single middle commit, which becomes `state.current`.
/// Locks in the bisection convergence path.
#[tokio::test]
#[serial(cwd)]
async fn test_bisect_mark_bad_then_good() {
    let temp_path = tempdir().unwrap();
    test::setup_with_new_libra_in(temp_path.path()).await;
    let _guard = ChangeDirGuard::new(temp_path.path());

    configure_identity().await;

    // Create 3 commits: hashes[0] = latest, hashes[2] = oldest
    let hashes = create_linear_commits(3).await;

    // Start bisect
    let args = Bisect::Start {
        bad: None,
        good: vec![],
        good_opt: None,
        term_old: vec![],
        term_new: vec![],
        first_parent: false,
    };
    execute_safe(args, &OutputConfig::default()).await.unwrap();

    // Mark bad (latest)
    let bad = hashes[0].clone();
    let args = Bisect::Bad {
        rev: Some(bad.clone()),
    };
    execute_safe(args, &OutputConfig::default()).await.unwrap();

    let state = BisectState::load().await.unwrap();
    assert_eq!(state.bad.unwrap().to_string(), bad);

    // Mark good (oldest)
    let good = hashes[2].clone();
    let args = Bisect::Good { rev: Some(good) };
    execute_safe(args, &OutputConfig::default()).await.unwrap();

    // Should now be on the middle commit (hashes[1])
    let state = BisectState::load().await.unwrap();
    assert_eq!(state.current.unwrap().to_string(), hashes[1]);
}

/// Scenario: end-to-end bisection over 7 commits where commits 4-6 are
/// "bad". The loop drives the algorithm to termination using the index of
/// the current commit as ground truth. Confirms the algorithm terminates
/// and exits the bisect session cleanly.
#[tokio::test]
#[serial(cwd)]
async fn test_bisect_find_first_bad_commit() {
    let temp_path = tempdir().unwrap();
    test::setup_with_new_libra_in(temp_path.path()).await;
    let _guard = ChangeDirGuard::new(temp_path.path());

    configure_identity().await;

    // Create 7 commits: hashes[0] = latest (Commit 6), hashes[6] = oldest (Commit 0)
    let hashes = create_linear_commits(7).await;

    // Start bisect with bad at Commit 6 (latest), good at Commit 3 (hashes[3])
    // So Commit 4, 5, 6 are bad, Commit 0, 1, 2, 3 are good
    // First bad commit should be hashes[3] (Commit 4 from user perspective, but index 3 in our array)
    let bad = hashes[0].clone(); // latest = Commit 6
    let good = hashes[3].clone(); // Commit 3

    let args = Bisect::Start {
        bad: Some(bad),
        good: vec![good],
        good_opt: None,
        term_old: vec![],
        term_new: vec![],
        first_parent: false,
    };
    execute_safe(args, &OutputConfig::default()).await.unwrap();

    // Continue bisect until we find the first bad commit
    // The first bad commit should be hashes[2] (Commit 4 in sequence, which is index 2 from newest)
    loop {
        if !BisectState::is_in_progress().await.unwrap() {
            break;
        }

        let state = BisectState::load().await.unwrap();
        let current = state.current.unwrap().to_string();

        // For this test, commits 4, 5, 6 (hashes[0], [1], [2]) are bad
        // commits 0, 1, 2, 3 (hashes[3], [4], [5], [6]) are good
        let current_idx = hashes.iter().position(|h| h == &current).unwrap();

        if current_idx <= 2 {
            // This commit is bad (indices 0, 1, 2 are commits 6, 5, 4)
            let args = Bisect::Bad { rev: None };
            execute_safe(args, &OutputConfig::default()).await.unwrap();
        } else {
            // This commit is good
            let args = Bisect::Good { rev: None };
            execute_safe(args, &OutputConfig::default()).await.unwrap();
        }
    }

    // Bisect should have ended
    assert!(!BisectState::is_in_progress().await.unwrap());
}

/// MX-01 smoke (M-BSAFE S1, packed state): after `repack` leaves every
/// reachable object both as a loose object and in a pack index, a full bisect
/// session must still converge and terminate instead of failing with
/// `failed to resolve checkout source`.
#[tokio::test]
#[serial(cwd)]
async fn test_bisect_session_after_repack_smoke() {
    let temp_path = tempdir().unwrap();
    test::setup_with_new_libra_in(temp_path.path()).await;
    let _guard = ChangeDirGuard::new(temp_path.path());

    configure_identity().await;

    // 7 commits: hashes[0] = latest (Commit 6), hashes[6] = oldest (Commit 0).
    let hashes = create_linear_commits(7).await;

    // `repack` without `-d` keeps the loose objects alongside the new pack, so
    // every OID is now duplicated between the two object sources (issue #481).
    let output = run_libra_command(&["repack"], temp_path.path());
    assert!(
        output.status.success(),
        "repack should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // bad = Commit 6, good = Commit 3, so the first bad commit is hashes[2].
    let bad = hashes[0].clone();
    let good = hashes[3].clone();
    let args = Bisect::Start {
        bad: Some(bad),
        good: vec![good],
        good_opt: None,
        term_old: vec![],
        term_new: vec![],
        first_parent: false,
    };
    execute_safe(args, &OutputConfig::default()).await.unwrap();

    loop {
        if !BisectState::is_in_progress().await.unwrap() {
            break;
        }
        let state = BisectState::load().await.unwrap();
        let current = state.current.unwrap().to_string();
        let current_idx = hashes.iter().position(|h| h == &current).unwrap();

        if current_idx <= 2 {
            execute_safe(Bisect::Bad { rev: None }, &OutputConfig::default())
                .await
                .unwrap();
        } else {
            execute_safe(Bisect::Good { rev: None }, &OutputConfig::default())
                .await
                .unwrap();
        }
    }

    assert!(!BisectState::is_in_progress().await.unwrap());
}

/// Drive the standard 7-commit scenario (bad = hashes[0], good = hashes[3]) to
/// convergence by marking any commit at index `<= 2` bad and the rest good.
/// Returns the converged [`BisectState`].
async fn converge_7_commit_session(hashes: &[String]) -> BisectState {
    let args = Bisect::Start {
        bad: Some(hashes[0].clone()),
        good: vec![hashes[3].clone()],
        good_opt: None,
        term_old: vec![],
        term_new: vec![],
        first_parent: false,
    };
    execute_safe(args, &OutputConfig::default()).await.unwrap();

    loop {
        if !BisectState::is_in_progress().await.unwrap() {
            break;
        }
        let state = BisectState::load().await.unwrap();
        let current = state.current.unwrap().to_string();
        let current_idx = hashes.iter().position(|h| h == &current).unwrap();

        if current_idx <= 2 {
            execute_safe(Bisect::Bad { rev: None }, &OutputConfig::default())
                .await
                .unwrap();
        } else {
            execute_safe(Bisect::Good { rev: None }, &OutputConfig::default())
                .await
                .unwrap();
        }
    }

    BisectState::load().await.unwrap()
}

/// M-BSAFE S1: a full bad -> good -> converge -> reset cycle must complete on
/// all three object-store shapes — loose only, loose + pack (`repack`), and a
/// single pack with loose objects deleted (`repack -a -d`).
#[tokio::test]
#[serial(cwd)]
async fn test_bisect_full_cycle_object_states_matrix() {
    let shapes: &[(&str, &[&str])] = &[
        ("loose", &[]),
        ("repack", &["repack"]),
        ("repack-a-d", &["repack", "-a", "-d"]),
    ];

    for (name, repack_args) in shapes {
        let temp_path = tempdir().unwrap();
        test::setup_with_new_libra_in(temp_path.path()).await;
        let _guard = ChangeDirGuard::new(temp_path.path());

        configure_identity().await;
        let hashes = create_linear_commits(7).await;

        if !repack_args.is_empty() {
            let output = run_libra_command(repack_args, temp_path.path());
            assert!(
                output.status.success(),
                "{name}: repack failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        let state = converge_7_commit_session(&hashes).await;
        assert!(state.completed, "{name}: session should converge");
        assert_eq!(
            state.bad.as_ref().unwrap().to_string(),
            hashes[2],
            "{name}: first bad commit should be Commit 4"
        );

        execute_safe(Bisect::Reset { rev: None }, &OutputConfig::default())
            .await
            .unwrap();
        assert_eq!(
            Head::current_commit().await.unwrap().to_string(),
            hashes[0],
            "{name}: reset should restore the original HEAD"
        );
        assert!(
            !BisectState::has_state().await.unwrap(),
            "{name}: state should be cleared"
        );
    }
}

/// M-BSAFE S4: a step whose restore preflight is refused (a candidate tracks a
/// path that currently exists as a non-empty untracked directory) must leave
/// the saved bisect state untouched — `current` does not advance, `bisect log`
/// is byte-identical to the pre-failure state — and a subsequent `bisect reset`
/// still restores the original HEAD and clears the session.
#[tokio::test]
#[serial(cwd)]
async fn test_bisect_failed_step_keeps_state_and_reset_recovers() {
    let temp_path = tempdir().unwrap();
    test::setup_with_new_libra_in(temp_path.path()).await;
    let _guard = ChangeDirGuard::new(temp_path.path());

    configure_identity().await;

    // 4 commits, newest first after reverse:
    //   hashes[3] C0 (good): file.txt only
    //   hashes[2] C1:        file.txt + `blocked` (file)
    //   hashes[1] C2:        file.txt + `blocked` (file)
    //   hashes[0] C3 (HEAD): file.txt only (removed `blocked`)
    let mut hashes = Vec::new();
    for i in 0..4 {
        test::ensure_file("file.txt", Some(&format!("content_{i}\n")));
        if i == 1 || i == 2 {
            test::ensure_file("blocked", Some(&format!("blocked_{i}\n")));
        } else {
            let p = std::path::Path::new("blocked");
            if p.exists() {
                std::fs::remove_file(p).unwrap();
            }
        }
        add::execute(AddArgs {
            all: true,
            ..Default::default()
        })
        .await;
        commit::execute(commit::CommitArgs {
            message: Some(format!("Commit {i}")),
            disable_pre: true,
            ..Default::default()
        })
        .await;
        hashes.push(Head::current_commit().await.unwrap().to_string());
    }
    hashes.reverse();

    // Start with no bounds: HEAD stays at C3, which does not track `blocked`.
    execute_safe(
        Bisect::Start {
            bad: None,
            good: vec![],
            good_opt: None,
            term_old: vec![],
            term_new: vec![],
            first_parent: false,
        },
        &OutputConfig::default(),
    )
    .await
    .unwrap();

    // Introduce the conflict AFTER start (start refuses a dirty tree): a
    // non-empty untracked directory at the path the midpoint tracks as a file.
    std::fs::create_dir_all("blocked/sub").unwrap();
    std::fs::write("blocked/sub/keep.txt", "x").unwrap();

    // Mark bad = HEAD (C3); no checkout, just records the bound.
    execute_safe(Bisect::Bad { rev: None }, &OutputConfig::default())
        .await
        .unwrap();

    let log_before = run_libra_command(&["bisect", "log"], temp_path.path());
    assert!(
        log_before.status.success(),
        "log before the failed step should succeed: {}",
        String::from_utf8_lossy(&log_before.stderr)
    );

    // Mark good = C0. The midpoint is C2, which tracks `blocked` as a file;
    // the untracked `blocked/` directory makes the restore preflight refuse.
    let result = execute_safe(
        Bisect::Good {
            rev: Some(hashes[3].clone()),
        },
        &OutputConfig::default(),
    )
    .await;
    assert!(
        result.is_err(),
        "good step must fail on the blocked restore"
    );

    let state = BisectState::load().await.unwrap();
    assert!(
        state.good.is_empty(),
        "a failed step must not persist the good mark"
    );
    assert_eq!(state.bad.as_ref().unwrap().to_string(), hashes[0]);
    assert!(
        state.current.is_none(),
        "current must not advance on a failed step"
    );

    let log_after = run_libra_command(&["bisect", "log"], temp_path.path());
    assert!(
        log_after.status.success(),
        "log after the failed step should succeed: {}",
        String::from_utf8_lossy(&log_after.stderr)
    );
    assert_eq!(
        log_before.stdout, log_after.stdout,
        "bisect log must be identical before and after a failed step"
    );

    // The session is not wedged: reset restores the original HEAD.
    execute_safe(Bisect::Reset { rev: None }, &OutputConfig::default())
        .await
        .unwrap();
    assert_eq!(Head::current_commit().await.unwrap().to_string(), hashes[0]);
    assert!(!BisectState::has_state().await.unwrap());
}

/// M-BSAFE S2/S3: after the search converges, marking again (`good` or `bad`)
/// must exit 0 and re-print the culprit instead of surfacing an internal
/// "already found" error; the subject must be the commit body (no `gpgsig`).
#[tokio::test]
#[serial(cwd)]
async fn test_bisect_converged_mark_reprints_culprit() {
    let temp_path = tempdir().unwrap();
    test::setup_with_new_libra_in(temp_path.path()).await;
    let _guard = ChangeDirGuard::new(temp_path.path());

    configure_identity().await;
    let hashes = create_linear_commits(7).await;

    let state = converge_7_commit_session(&hashes).await;
    assert!(state.completed);
    assert_eq!(state.bad.as_ref().unwrap().to_string(), hashes[2]);

    for subcommand in ["good", "bad"] {
        let output = run_libra_command(&["bisect", subcommand], temp_path.path());
        assert!(
            output.status.success(),
            "bisect {subcommand} after convergence must exit 0: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("is the first bad commit"),
            "bisect {subcommand} must re-print the culprit: {stdout}"
        );
        assert!(
            stdout.contains("Commit 4"),
            "bisect {subcommand} must print the culprit subject: {stdout}"
        );
        assert!(
            !stdout.contains("gpgsig") && !stdout.contains("already found"),
            "bisect {subcommand} must not leak the signature header or an internal error: {stdout}"
        );
    }

    execute_safe(Bisect::Reset { rev: None }, &OutputConfig::default())
        .await
        .unwrap();
}

/// M-BSAFE S5: every session-requiring subcommand reports `LBR-BISECT-001`
/// (exit 128) when no bisect session exists; `reset` stays a no-op that exits
/// 0 with `We are not bisecting.`
#[::std::prelude::rust_2024::test]
fn test_bisect_no_session_uses_stable_bisect_not_active() {
    let repo = tempdir().unwrap();
    init_repo_via_cli(repo.path());

    for subcommand in ["bad", "good", "skip", "log", "view", "run"] {
        let mut args: Vec<&str> = vec!["bisect", subcommand];
        if subcommand == "run" {
            args.push("true");
        }
        let output = run_libra_command(&args, repo.path());
        assert_eq!(
            output.status.code(),
            Some(128),
            "bisect {subcommand} without a session must exit 128"
        );
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("LBR-BISECT-001"),
            "bisect {subcommand} without a session must use LBR-BISECT-001: {stderr}"
        );
    }

    let output = run_libra_command(&["bisect", "reset"], repo.path());
    assert_eq!(
        output.status.code(),
        Some(0),
        "bisect reset without a session must exit 0"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("We are not bisecting."),
        "bisect reset without a session must print 'We are not bisecting.': {stdout}"
    );
}

/// Scenario: `bisect reset` must clear the in-progress state and return
/// HEAD to its pre-bisect commit. Pins both the state-cleanup and the
/// HEAD-restore behaviour after a session is started and `bad`/`good`
/// have moved HEAD off the original tip.
#[tokio::test]
#[serial(cwd)]
async fn test_bisect_reset() {
    let temp_path = tempdir().unwrap();
    test::setup_with_new_libra_in(temp_path.path()).await;
    let _guard = ChangeDirGuard::new(temp_path.path());

    configure_identity().await;

    let hashes = create_linear_commits(3).await;
    let orig_head = hashes[0].clone(); // latest

    // Start bisect
    let args = Bisect::Start {
        bad: None,
        good: vec![],
        good_opt: None,
        term_old: vec![],
        term_new: vec![],
        first_parent: false,
    };
    execute_safe(args, &OutputConfig::default()).await.unwrap();

    // Mark commits
    let args = Bisect::Bad {
        rev: Some(hashes[0].clone()),
    }; // latest
    execute_safe(args, &OutputConfig::default()).await.unwrap();

    let args = Bisect::Good {
        rev: Some(hashes[2].clone()),
    }; // oldest
    execute_safe(args, &OutputConfig::default()).await.unwrap();

    // Should be on middle commit
    let _state = BisectState::load().await.unwrap();
    assert_ne!(Head::current_commit().await.unwrap().to_string(), orig_head);

    // Reset
    let args = Bisect::Reset { rev: None };
    execute_safe(args, &OutputConfig::default()).await.unwrap();

    // State should be cleared
    assert!(!BisectState::is_in_progress().await.unwrap());

    // Should be back to original HEAD
    assert_eq!(Head::current_commit().await.unwrap().to_string(), orig_head);
}

/// `bisect reset` must surface corrupt storage for the original branch instead
/// of silently treating it as a deleted branch and falling back to detached
/// checkout. Otherwise a storage-corruption bug can be hidden behind a
/// successful reset.
#[tokio::test]
#[serial(cwd)]
async fn test_bisect_reset_surfaces_corrupt_original_branch_storage() {
    let temp_path = tempdir().unwrap();
    test::setup_with_new_libra_in(temp_path.path()).await;
    let _guard = ChangeDirGuard::new(temp_path.path());

    configure_identity().await;
    create_linear_commits(3).await;

    execute_safe(
        Bisect::Start {
            bad: None,
            good: vec![],
            good_opt: None,
            term_old: vec![],
            term_new: vec![],
            first_parent: false,
        },
        &OutputConfig::default(),
    )
    .await
    .unwrap();
    Branch::update_branch("main", "not-a-valid-hash", None)
        .await
        .unwrap();

    let error = execute_safe(Bisect::Reset { rev: None }, &OutputConfig::default())
        .await
        .expect_err("bisect reset must fail when the original branch row is corrupt");
    assert_eq!(error.stable_code().as_str(), "LBR-REPO-002");
}

/// Scenario: `bisect skip` must record the current commit in
/// `state.skipped` and advance to a different commit. Locks in the skip
/// behaviour for untestable commits.
#[tokio::test]
#[serial(cwd)]
async fn test_bisect_skip() {
    let temp_path = tempdir().unwrap();
    test::setup_with_new_libra_in(temp_path.path()).await;
    let _guard = ChangeDirGuard::new(temp_path.path());

    configure_identity().await;

    // Create 5 commits
    let hashes = create_linear_commits(5).await;

    // Start bisect
    let bad = hashes[0].clone(); // latest
    let good = hashes[4].clone(); // oldest

    let args = Bisect::Start {
        bad: Some(bad),
        good: vec![good],
        good_opt: None,
        term_old: vec![],
        term_new: vec![],
        first_parent: false,
    };
    execute_safe(args, &OutputConfig::default()).await.unwrap();

    let state = BisectState::load().await.unwrap();
    let current = state.current.unwrap().to_string();

    // Skip current commit
    let args = Bisect::Skip { rev: None };
    execute_safe(args, &OutputConfig::default()).await.unwrap();

    let state = BisectState::load().await.unwrap();

    // Current should be skipped
    assert!(state.skipped.iter().any(|h| h.to_string() == current));

    // Should have moved to a different commit
    assert_ne!(state.current.unwrap().to_string(), current);
}

/// Scenario: `bisect log` must execute without error during an active
/// session. Smoke-tests the log subcommand path (the actual log content is
/// not asserted here).
#[tokio::test]
#[serial(cwd)]
async fn test_bisect_log() {
    let temp_path = tempdir().unwrap();
    test::setup_with_new_libra_in(temp_path.path()).await;
    let _guard = ChangeDirGuard::new(temp_path.path());

    configure_identity().await;

    let hashes = create_linear_commits(3).await;

    // Start bisect and mark some commits
    let args = Bisect::Start {
        bad: Some(hashes[0].clone()),
        good: vec![hashes[2].clone()],
        good_opt: None,
        term_old: vec![],
        term_new: vec![],
        first_parent: false,
    };
    execute_safe(args, &OutputConfig::default()).await.unwrap();

    // Log should work
    let args = Bisect::Log;
    execute_safe(args, &OutputConfig::default()).await.unwrap();
}

/// Scenario: starting a second bisect session while one is active must
/// return an error. Pins the "single active session" invariant.
#[tokio::test]
#[serial(cwd)]
async fn test_bisect_start_already_in_progress_fails() {
    let temp_path = tempdir().unwrap();
    test::setup_with_new_libra_in(temp_path.path()).await;
    let _guard = ChangeDirGuard::new(temp_path.path());

    configure_identity().await;

    create_linear_commits(3).await;

    // Start first bisect
    let args = Bisect::Start {
        bad: None,
        good: vec![],
        good_opt: None,
        term_old: vec![],
        term_new: vec![],
        first_parent: false,
    };
    execute_safe(args, &OutputConfig::default()).await.unwrap();

    // Try to start again - should fail
    let args = Bisect::Start {
        bad: None,
        good: vec![],
        good_opt: None,
        term_old: vec![],
        term_new: vec![],
        first_parent: false,
    };
    let result = execute_safe(args, &OutputConfig::default()).await;
    assert!(result.is_err());
}

/// Scenario: `bad`, `good`, and `skip` must all return errors when no
/// bisect session has been started. Pins the no-implicit-session contract.
#[tokio::test]
#[serial(cwd)]
async fn test_bisect_operations_without_session_fails() {
    let temp_path = tempdir().unwrap();
    test::setup_with_new_libra_in(temp_path.path()).await;
    let _guard = ChangeDirGuard::new(temp_path.path());

    configure_identity().await;

    create_linear_commits(3).await;

    // Try bad without session
    let args = Bisect::Bad { rev: None };
    let result = execute_safe(args, &OutputConfig::default()).await;
    assert!(result.is_err());

    // Try good without session
    let args = Bisect::Good { rev: None };
    let result = execute_safe(args, &OutputConfig::default()).await;
    assert!(result.is_err());

    // Try skip without session
    let args = Bisect::Skip { rev: None };
    let result = execute_safe(args, &OutputConfig::default()).await;
    assert!(result.is_err());
}

/// Scenario: invoking `libra bisect start` outside any repo through the
/// real binary must exit 128 and emit a "fatal" message on stderr. Note
/// the explicit `#[::std::prelude::rust_2024::test]` path because the
/// surrounding async tests pull `tokio::test` into scope.
#[::std::prelude::rust_2024::test]
fn test_bisect_cli_outside_repository_returns_fatal() {
    let temp = tempdir().unwrap();

    let output = run_libra_command(&["bisect", "start"], temp.path());
    assert_eq!(output.status.code(), Some(128));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("fatal"),
        "expected fatal error, got: {stderr}"
    );
}

/// Scenario: `libra bisect start` against a repo with no commits must
/// fail (no objects to walk). Captures the "empty history" error path.
#[::std::prelude::rust_2024::test]
fn test_bisect_cli_empty_repository_returns_fatal() {
    let repo = tempdir().unwrap();
    init_repo_via_cli(repo.path());

    let output = run_libra_command(&["bisect", "start"], repo.path());
    // Should fail because there are no commits
    assert!(!output.status.success());
}

// ── C4 surface tests: `bisect run` / `bisect view` ────────────────────────────────────────

/// `libra bisect --help` lists the new `run` and `view` subcommands plus
/// the EXAMPLES banner produced by `BISECT_EXAMPLES`.
#[::std::prelude::rust_2024::test]
fn test_bisect_help_lists_run_and_view() {
    let repo = tempdir().unwrap();
    init_repo_via_cli(repo.path());

    let output = run_libra_command(&["bisect", "--help"], repo.path());
    assert!(
        output.status.success(),
        "bisect --help should succeed, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("run"),
        "bisect --help should list 'run', stdout: {stdout}"
    );
    assert!(
        stdout.contains("view"),
        "bisect --help should list 'view', stdout: {stdout}"
    );
    assert!(
        stdout.contains("EXAMPLES:"),
        "bisect --help should include EXAMPLES, stdout: {stdout}"
    );
}

/// `bisect view` outside an active session must return `BisectNotActive`
/// (LBR-BISECT-001) so callers can distinguish "no bisect" from a transient
/// failure.
#[tokio::test]
#[serial(cwd)]
async fn test_bisect_view_without_session_errors() {
    let temp_path = tempdir().unwrap();
    test::setup_with_new_libra_in(temp_path.path()).await;
    let _guard = ChangeDirGuard::new(temp_path.path());
    configure_identity().await;
    let _hashes = create_linear_commits(3).await;

    let result = execute_safe(Bisect::View, &OutputConfig::default()).await;
    assert!(result.is_err(), "view without session must error");
    let err = result.unwrap_err();
    let stable = err.stable_code().as_str();
    assert_eq!(
        stable, "LBR-BISECT-001",
        "view without session must use BisectNotActive, got {stable}"
    );
}

/// `bisect view` during an active session prints state without erroring.
#[tokio::test]
#[serial(cwd)]
async fn test_bisect_view_inside_active_session() {
    let temp_path = tempdir().unwrap();
    test::setup_with_new_libra_in(temp_path.path()).await;
    let _guard = ChangeDirGuard::new(temp_path.path());
    configure_identity().await;
    let hashes = create_linear_commits(5).await;

    execute_safe(
        Bisect::Start {
            bad: Some(hashes[0].clone()),
            good: vec![hashes[4].clone()],
            good_opt: None,
            term_old: vec![],
            term_new: vec![],
            first_parent: false,
        },
        &OutputConfig::default(),
    )
    .await
    .unwrap();

    execute_safe(Bisect::View, &OutputConfig::default())
        .await
        .expect("view inside an active session must succeed");

    // Clean up.
    execute_safe(Bisect::Reset { rev: None }, &OutputConfig::default())
        .await
        .unwrap();
}

/// `--json bisect view` must emit a single clean command envelope without
/// human progress lines on stdout.
#[tokio::test]
#[serial(cwd)]
async fn test_bisect_json_view_outputs_clean_envelope() {
    let temp_path = tempdir().unwrap();
    test::setup_with_new_libra_in(temp_path.path()).await;
    let _guard = ChangeDirGuard::new(temp_path.path());
    configure_identity().await;
    let hashes = create_linear_commits(5).await;

    execute_safe(
        Bisect::Start {
            bad: Some(hashes[0].clone()),
            good: vec![hashes[4].clone()],
            good_opt: None,
            term_old: vec![],
            term_new: vec![],
            first_parent: false,
        },
        &OutputConfig::default(),
    )
    .await
    .unwrap();

    let output = run_libra_command(&["--json", "bisect", "view"], temp_path.path());
    assert!(
        output.status.success(),
        "json view failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains("Bisecting between"),
        "json stdout must not contain human text: {stdout}"
    );
    let json: serde_json::Value =
        serde_json::from_str(&stdout).expect("bisect json view should parse");
    assert_eq!(json["command"], "bisect");
    assert_eq!(json["data"]["action"], "view");
    assert_eq!(json["data"]["remaining"].as_u64(), Some(4));

    execute_safe(Bisect::Reset { rev: None }, &OutputConfig::default())
        .await
        .unwrap();
}

/// `bisect run` without an active session must reject with `BisectNotActive`.
/// The user must `bisect start` (with bounds) before automation kicks in.
#[tokio::test]
#[serial(cwd)]
async fn test_bisect_run_without_session_errors() {
    let temp_path = tempdir().unwrap();
    test::setup_with_new_libra_in(temp_path.path()).await;
    let _guard = ChangeDirGuard::new(temp_path.path());
    configure_identity().await;
    let _hashes = create_linear_commits(3).await;

    let result = execute_safe(
        Bisect::Run {
            cmd: vec!["true".to_string()],
        },
        &OutputConfig::default(),
    )
    .await;
    assert!(result.is_err(), "run without session must error");
    let err = result.unwrap_err();
    let stable = err.stable_code().as_str();
    assert_eq!(stable, "LBR-BISECT-001");
}

/// `bisect run` must fail before spawning the command when the session exists
/// but good/bad bounds have not selected a candidate yet.
#[cfg(unix)]
#[tokio::test]
#[serial(cwd)]
async fn test_bisect_run_without_bounds_does_not_spawn_command() {
    let temp_path = tempdir().unwrap();
    test::setup_with_new_libra_in(temp_path.path()).await;
    let _guard = ChangeDirGuard::new(temp_path.path());
    configure_identity().await;
    let _hashes = create_linear_commits(3).await;

    execute_safe(
        Bisect::Start {
            bad: None,
            good: vec![],
            good_opt: None,
            term_old: vec![],
            term_new: vec![],
            first_parent: false,
        },
        &OutputConfig::default(),
    )
    .await
    .unwrap();

    let marker = temp_path.path().join("spawned-marker");
    let result = execute_safe(
        Bisect::Run {
            cmd: vec![
                "sh".to_string(),
                "-c".to_string(),
                format!("touch {}", marker.display()),
            ],
        },
        &OutputConfig::default(),
    )
    .await;

    assert!(result.is_err(), "run without bounds must error");
    let err = result.unwrap_err();
    assert_eq!(err.stable_code().as_str(), "LBR-REPO-003");
    assert!(
        fs::metadata(&marker).is_err(),
        "bisect run spawned the command before validating bounds"
    );
}

/// `bisect run` with a script that always returns 128 must surface the
/// non-recoverable exit code through `BisectRunFailed` (LBR-BISECT-002).
#[cfg(unix)]
#[tokio::test]
#[serial(cwd)]
async fn test_bisect_run_propagates_fatal_exit_code() {
    let temp_path = tempdir().unwrap();
    test::setup_with_new_libra_in(temp_path.path()).await;
    let _guard = ChangeDirGuard::new(temp_path.path());
    configure_identity().await;
    let hashes = create_linear_commits(5).await;

    execute_safe(
        Bisect::Start {
            bad: Some(hashes[0].clone()),
            good: vec![hashes[4].clone()],
            good_opt: None,
            term_old: vec![],
            term_new: vec![],
            first_parent: false,
        },
        &OutputConfig::default(),
    )
    .await
    .unwrap();

    let result = execute_safe(
        Bisect::Run {
            cmd: vec!["sh".to_string(), "-c".to_string(), "exit 128".to_string()],
        },
        &OutputConfig::default(),
    )
    .await;
    assert!(result.is_err(), "exit 128 must abort bisect run");
    let err = result.unwrap_err();
    let stable = err.stable_code().as_str();
    assert_eq!(stable, "LBR-BISECT-002");

    // Clean up so the next test in the suite starts fresh.
    let _ = execute_safe(Bisect::Reset { rev: None }, &OutputConfig::default()).await;
}

/// `--machine bisect run` must emit exactly one JSON line for automation even
/// though the run internally marks multiple commits.
#[cfg(unix)]
#[tokio::test]
#[serial(cwd)]
async fn test_bisect_machine_run_outputs_single_json_line() {
    let temp_path = tempdir().unwrap();
    test::setup_with_new_libra_in(temp_path.path()).await;
    let _guard = ChangeDirGuard::new(temp_path.path());
    configure_identity().await;
    let hashes = create_linear_commits(5).await;

    execute_safe(
        Bisect::Start {
            bad: Some(hashes[0].clone()),
            good: vec![hashes[4].clone()],
            good_opt: None,
            term_old: vec![],
            term_new: vec![],
            first_parent: false,
        },
        &OutputConfig::default(),
    )
    .await
    .unwrap();

    let output = run_libra_command(
        &["--machine", "bisect", "run", "sh", "-c", "exit 1"],
        temp_path.path(),
    );
    assert!(
        output.status.success(),
        "machine run failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(
        stdout.lines().count(),
        1,
        "machine output should be exactly one JSON line: {stdout}"
    );
    assert!(
        !stdout.contains("Marked ") && !stdout.contains("HEAD is now"),
        "machine stdout must not contain human progress: {stdout}"
    );
    let json: serde_json::Value =
        serde_json::from_str(&stdout).expect("bisect machine run should parse");
    assert_eq!(json["command"], "bisect");
    assert_eq!(json["data"]["action"], "run");
    assert_eq!(json["data"]["steps"].as_u64(), Some(2));
    assert!(json["data"]["first_bad"].as_str().is_some());
}

/// `bisect start --first-parent` must restrict the candidate set to the
/// first-parent (mainline) history, so a merged-in side branch contributes no
/// testable commits. Verified by comparing the reported `remaining` candidate
/// count against a normal (all-parents) bisect over the same merge history.
#[test]
fn bisect_first_parent_shrinks_candidate_set() {
    let repo = tempdir().unwrap();
    let p = repo.path();
    // Keep the isolated HOME OUTSIDE the repo: anything under `p` would be an
    // untracked file that bisect's clean-tree guard rejects.
    let home_dir = tempdir().unwrap();
    let home = home_dir.path().to_path_buf();
    std::fs::create_dir_all(home.join(".config")).unwrap();

    let run = |args: &[&str]| -> std::process::Output {
        Command::new(env!("CARGO_BIN_EXE_libra"))
            .args(args)
            .current_dir(p)
            .env_clear()
            .env("PATH", "/usr/bin:/bin:/usr/sbin:/sbin")
            .env("HOME", &home)
            .env("XDG_CONFIG_HOME", home.join(".config"))
            .env("LANG", "C")
            .env("LC_ALL", "C")
            .env("LIBRA_COMMITTER_NAME", "Tester")
            .env("LIBRA_COMMITTER_EMAIL", "t@t.test")
            .output()
            .expect("run libra")
    };
    let commit_file = |name: &str, msg: &str| {
        std::fs::write(p.join(name), format!("{name}\n")).unwrap();
        assert!(run(&["add", name]).status.success(), "add {name}");
        let out = run(&["commit", "-m", msg, "--no-verify"]);
        assert!(
            out.status.success(),
            "commit {msg}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    };
    let head = || -> String {
        String::from_utf8_lossy(&run(&["rev-parse", "HEAD"]).stdout)
            .trim()
            .to_string()
    };

    assert!(run(&["init"]).status.success(), "init");
    assert!(
        run(&["config", "user.name", "Tester"]).status.success(),
        "config name"
    );
    assert!(
        run(&["config", "user.email", "t@t.test"]).status.success(),
        "config email"
    );

    std::fs::write(p.join("base.txt"), "base\n").unwrap();
    assert!(run(&["add", "base.txt"]).status.success(), "add base");
    assert!(
        run(&["commit", "-m", "c0", "--no-verify"]).status.success(),
        "commit c0"
    );
    let good = head();

    // Side branch with several commits (these become extra candidates only in
    // the all-parents walk).
    assert!(run(&["branch", "side"]).status.success(), "branch side");
    assert!(run(&["switch", "side"]).status.success(), "switch side");
    for i in 0..5 {
        commit_file(&format!("side{i}.txt"), &format!("s{i}"));
    }

    // Mainline advances, then merges the side branch (real merge commit).
    assert!(run(&["switch", "main"]).status.success(), "switch main");
    commit_file("main1.txt", "m1");
    let merge = run(&["merge", "side", "--no-ff", "-m", "merge side"]);
    assert!(
        merge.status.success(),
        "merge: {}",
        String::from_utf8_lossy(&merge.stderr)
    );
    commit_file("main2.txt", "m2");
    let bad = head();

    let remaining = |first_parent: bool| -> i64 {
        let mut args = vec![
            "--json",
            "bisect",
            "start",
            bad.as_str(),
            "-g",
            good.as_str(),
        ];
        if first_parent {
            args.push("--first-parent");
        }
        let out = run(&args);
        assert!(
            out.status.success(),
            "bisect start: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        let json: serde_json::Value =
            serde_json::from_slice(&out.stdout).expect("bisect start json");
        let rem = json["data"]["remaining"].as_i64().expect("remaining field");
        assert!(run(&["bisect", "reset"]).status.success(), "bisect reset");
        rem
    };

    let full = remaining(false);
    let first_parent = remaining(true);
    assert!(
        first_parent < full,
        "first-parent must shrink the candidate set: first_parent={first_parent}, full={full}"
    );
}

/// `bisect visualize` is Git's alias for `bisect view`; both must dispatch to
/// the same handler (identical exit code and output). Exercised through the
/// real clap parser via the binary.
#[test]
fn bisect_visualize_aliases_view() {
    let temp = tempdir().unwrap();
    init_repo_via_cli(temp.path());

    // With no active bisect, both forms hit the same handler and fail the same.
    let view = run_libra_command(&["bisect", "view"], temp.path());
    let visualize = run_libra_command(&["bisect", "visualize"], temp.path());

    assert_eq!(
        view.status.code(),
        visualize.status.code(),
        "view and visualize must share an exit code"
    );
    assert_eq!(
        view.stdout, visualize.stdout,
        "view and visualize must produce identical stdout"
    );
    assert_eq!(
        view.stderr, visualize.stderr,
        "view and visualize must produce identical stderr"
    );

    // `--help` advertises the alias.
    let help = run_libra_command(&["bisect", "--help"], temp.path());
    assert!(
        String::from_utf8_lossy(&help.stdout).contains("visualize"),
        "bisect --help lists the visualize alias"
    );
}

/// M-BSTART B1/B6: `bisect start <bad> <good1> <good2>` must accept Git's
/// positional form and record every good bound, while `--good` remains a
/// compatible alias for a single good. Exercised through the real clap parser
/// so the positional/option grammar is what is under test.
#[tokio::test]
#[serial(cwd)]
async fn test_bisect_start_positional_goods() {
    let temp_path = tempdir().unwrap();
    test::setup_with_new_libra_in(temp_path.path()).await;
    let _guard = ChangeDirGuard::new(temp_path.path());

    configure_identity().await;
    let hashes = create_linear_commits(5).await;
    let bad = hashes[0].clone(); // newest
    let good1 = hashes[4].clone(); // oldest
    let good2 = hashes[2].clone(); // middle

    // B1: two positional goods.
    let output = run_libra_command(
        &[
            "bisect",
            "start",
            bad.as_str(),
            good1.as_str(),
            good2.as_str(),
        ],
        temp_path.path(),
    );
    assert!(
        output.status.success(),
        "positional start failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let state = BisectState::load().await.unwrap();
    assert_eq!(state.bad.unwrap().to_string(), bad);
    assert_eq!(
        state.good.len(),
        2,
        "both positional goods must be recorded"
    );
    assert!(state.good.iter().any(|h| h.to_string() == good1));
    assert!(state.good.iter().any(|h| h.to_string() == good2));
    execute_safe(Bisect::Reset { rev: None }, &OutputConfig::default())
        .await
        .unwrap();

    // B6: `--good` stays a compatibility alias for the first positional good.
    let output = run_libra_command(
        &["bisect", "start", bad.as_str(), "--good", good1.as_str()],
        temp_path.path(),
    );
    assert!(
        output.status.success(),
        "compat --good start failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let state = BisectState::load().await.unwrap();
    assert_eq!(state.bad.unwrap().to_string(), bad);
    assert_eq!(state.good.len(), 1);
    assert_eq!(state.good[0].to_string(), good1);
    execute_safe(Bisect::Reset { rev: None }, &OutputConfig::default())
        .await
        .unwrap();

    // B6: `--first-parent` composes with both the positional and the `--good`
    // compatibility forms (t6030's option/rev order tolerance).
    let output = run_libra_command(
        &[
            "bisect",
            "start",
            "--first-parent",
            bad.as_str(),
            good1.as_str(),
        ],
        temp_path.path(),
    );
    assert!(
        output.status.success(),
        "--first-parent positional start failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let state = BisectState::load().await.unwrap();
    assert_eq!(state.bad.unwrap().to_string(), bad);
    assert_eq!(state.good.len(), 1);
    assert_eq!(state.good[0].to_string(), good1);
    execute_safe(Bisect::Reset { rev: None }, &OutputConfig::default())
        .await
        .unwrap();

    let output = run_libra_command(
        &[
            "bisect",
            "start",
            "--first-parent",
            bad.as_str(),
            "--good",
            good1.as_str(),
        ],
        temp_path.path(),
    );
    assert!(
        output.status.success(),
        "--first-parent with --good start failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let state = BisectState::load().await.unwrap();
    assert_eq!(state.bad.unwrap().to_string(), bad);
    assert_eq!(state.good.len(), 1);
    assert_eq!(state.good[0].to_string(), good1);
    execute_safe(Bisect::Reset { rev: None }, &OutputConfig::default())
        .await
        .unwrap();
}

/// M-BSTART B2/B4: untracked and ignored files (including a root
/// `.libraignore`) must not block `bisect start` (ADR-MX-05 §2); only tracked
/// staged/unstaged modifications refuse the start.
#[tokio::test]
#[serial(cwd)]
async fn test_bisect_start_allows_untracked_files() {
    let temp_path = tempdir().unwrap();
    test::setup_with_new_libra_in(temp_path.path()).await;
    let _guard = ChangeDirGuard::new(temp_path.path());

    configure_identity().await;
    let hashes = create_linear_commits(5).await;
    let bad = hashes[0].clone();
    let good = hashes[4].clone();

    // B2: untracked + ignored files are present, none of which the candidate
    // commits track — the start must succeed.
    fs::write(temp_path.path().join(".libraignore"), "*.log\n").unwrap();
    fs::write(temp_path.path().join("notes.txt"), "untracked\n").unwrap();
    fs::write(temp_path.path().join("debug.log"), "ignored\n").unwrap();

    let output = run_libra_command(
        &["bisect", "start", bad.as_str(), good.as_str()],
        temp_path.path(),
    );
    assert!(
        output.status.success(),
        "untracked files must not block start: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let state = BisectState::load().await.unwrap();
    assert!(state.bad.is_some());
    assert!(!state.good.is_empty());
    execute_safe(Bisect::Reset { rev: None }, &OutputConfig::default())
        .await
        .unwrap();

    // B4: a tracked modification still refuses the start (intentionally
    // stricter than Git, ADR-MX-05 §3), and leaves no session behind.
    fs::write(temp_path.path().join("file.txt"), "modified tracked\n").unwrap();
    let output = run_libra_command(
        &["bisect", "start", bad.as_str(), good.as_str()],
        temp_path.path(),
    );
    assert!(
        !output.status.success(),
        "a tracked modification must block start"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("uncommitted changes"),
        "start refusal must name the reason: {stderr}"
    );
    assert!(
        !BisectState::has_state().await.unwrap(),
        "a refused start must not leave a session"
    );
}

/// M-BSTART B5: `bisect start` with an unresolvable revision (git t6030:136's
/// "junk instead of revs" shape) fails without creating a session — for the
/// bad and the good side — and a valid start still works afterwards.
#[tokio::test]
#[serial(cwd)]
async fn test_bisect_start_rejects_invalid_revs() {
    let temp_path = tempdir().unwrap();
    test::setup_with_new_libra_in(temp_path.path()).await;
    let _guard = ChangeDirGuard::new(temp_path.path());

    configure_identity().await;
    let hashes = create_linear_commits(5).await;
    let bad = hashes[0].clone();
    let good = hashes[4].clone();

    // Junk bad rev with the explicit `--` separator (t6030:136 shape).
    let output = run_libra_command(
        &["bisect", "start", "foo", good.as_str(), "--"],
        temp_path.path(),
    );
    assert!(
        !output.status.success(),
        "a junk bad rev must fail the start"
    );
    assert!(
        !BisectState::has_state().await.unwrap(),
        "a refused start must not create a session"
    );

    // Junk good rev.
    let output = run_libra_command(&["bisect", "start", bad.as_str(), "bar"], temp_path.path());
    assert!(
        !output.status.success(),
        "a junk good rev must fail the start"
    );
    assert!(
        !BisectState::has_state().await.unwrap(),
        "a refused start must not create a session"
    );

    // The repo is not wedged: a valid start still succeeds.
    let output = run_libra_command(
        &["bisect", "start", bad.as_str(), good.as_str()],
        temp_path.path(),
    );
    assert!(
        output.status.success(),
        "a valid start after junk revs must succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let state = BisectState::load().await.unwrap();
    assert_eq!(state.bad.unwrap().to_string(), bad);
    assert_eq!(state.good.len(), 1);
    assert_eq!(state.good[0].to_string(), good);
    execute_safe(Bisect::Reset { rev: None }, &OutputConfig::default())
        .await
        .unwrap();
}

/// M-BSTART B3: an untracked directory that collides with a path a later
/// candidate tracks as a file must not block the positional start (ADR-MX-05
/// §2 — untracked files are the per-step preflight's job); the step that
/// reaches the conflicting candidate is refused and leaves the saved state
/// untouched (same S4 semantics), and `reset` still restores the original
/// HEAD.
#[tokio::test]
#[serial(cwd)]
async fn test_bisect_untracked_collision_refused_at_step() {
    let temp_path = tempdir().unwrap();
    test::setup_with_new_libra_in(temp_path.path()).await;
    let _guard = ChangeDirGuard::new(temp_path.path());

    configure_identity().await;

    // 8 commits C0..C7 (newest first): C5 and C6 track `blocked` as a file;
    // C7 (HEAD, bad) removed it again. C0 is the good bound.
    let mut hashes = Vec::new();
    for i in 0..7 {
        test::ensure_file("file.txt", Some(&format!("content_{i}\n")));
        if i == 5 {
            test::ensure_file("blocked", Some("blocked_content\n"));
        }
        add::execute(AddArgs {
            all: true,
            ..Default::default()
        })
        .await;
        commit::execute(commit::CommitArgs {
            message: Some(format!("Commit {i}")),
            disable_pre: true,
            ..Default::default()
        })
        .await;
        hashes.push(Head::current_commit().await.unwrap().to_string());
    }
    std::fs::remove_file("blocked").unwrap();
    add::execute(AddArgs {
        all: true,
        ..Default::default()
    })
    .await;
    commit::execute(commit::CommitArgs {
        message: Some("Commit 7".to_string()),
        disable_pre: true,
        ..Default::default()
    })
    .await;
    hashes.push(Head::current_commit().await.unwrap().to_string());
    hashes.reverse();

    // The collision exists BEFORE start: a non-empty untracked directory at
    // the path C5/C6 track as a file.
    std::fs::create_dir_all("blocked/sub").unwrap();
    std::fs::write("blocked/sub/keep.txt", "x").unwrap();

    // The midpoint of C0..C7 is in the clean half, so the positional start
    // succeeds despite the collision (ADR-MX-05 §2).
    let output = run_libra_command(
        &["bisect", "start", hashes[0].as_str(), hashes[7].as_str()],
        temp_path.path(),
    );
    assert!(
        output.status.success(),
        "start with a colliding untracked dir must succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let midpoint = Head::current_commit().await.unwrap().to_string();

    let log_before = run_libra_command(&["bisect", "log"], temp_path.path());
    assert!(
        log_before.status.success(),
        "log before the refused step should succeed"
    );

    // Marking the clean midpoint good moves the search toward C5/C6, whose
    // checkout would replace the untracked `blocked/` directory — the restore
    // preflight refuses and nothing is persisted (S4 semantics).
    let result = execute_safe(Bisect::Good { rev: None }, &OutputConfig::default()).await;
    assert!(
        result.is_err(),
        "the step to the colliding candidate must be refused"
    );
    assert!(
        std::fs::metadata("blocked/sub/keep.txt").is_ok(),
        "the refused step must leave the untracked file in place"
    );

    let state = BisectState::load().await.unwrap();
    assert_eq!(
        state.good.len(),
        1,
        "the refused good mark must not be persisted"
    );
    assert_eq!(state.good[0].to_string(), hashes[7]);
    assert_eq!(
        state.current.as_ref().unwrap().to_string(),
        midpoint,
        "current must not advance on a refused step"
    );

    let log_after = run_libra_command(&["bisect", "log"], temp_path.path());
    assert!(
        log_after.status.success(),
        "log after the refused step should succeed"
    );
    assert_eq!(
        log_before.stdout, log_after.stdout,
        "bisect log must be identical before and after the refused step"
    );

    // The session is not wedged: reset restores the original HEAD.
    execute_safe(Bisect::Reset { rev: None }, &OutputConfig::default())
        .await
        .unwrap();
    assert_eq!(Head::current_commit().await.unwrap().to_string(), hashes[0]);
    assert!(!BisectState::has_state().await.unwrap());
}

/// M-BTERMS T1-T7: custom terms, `old`/`new` aliases, `bisect terms` output,
/// invalid-term rejection with zero writes, and custom-term rendering in
/// `log`/`view`.
#[tokio::test]
#[serial(cwd)]
async fn test_bisect_terms_matrix() {
    let temp_path = tempdir().unwrap();
    test::setup_with_new_libra_in(temp_path.path()).await;
    let _guard = ChangeDirGuard::new(temp_path.path());

    configure_identity().await;
    let hashes = create_linear_commits(7).await;
    let bad = hashes[0].clone();
    let good = hashes[3].clone();

    // T3: no session — `bisect terms` follows S5 (LBR-BISECT-001, exit 128),
    // and `bisect terms a b` is a usage error (129).
    let output = run_libra_command(&["bisect", "terms"], temp_path.path());
    assert_eq!(
        output.status.code(),
        Some(128),
        "no-session terms must exit 128"
    );
    assert!(String::from_utf8_lossy(&output.stderr).contains("LBR-BISECT-001"));
    let output = run_libra_command(&["bisect", "terms", "a", "b"], temp_path.path());
    assert_eq!(
        output.status.code(),
        Some(129),
        "terms with two positional args must be a usage error"
    );

    // T1 + T6: custom-term start with the options AFTER the positional bounds
    // (git accepts options and revisions in any order, t6030:1290).
    let output = run_libra_command(
        &[
            "bisect",
            "start",
            bad.as_str(),
            good.as_str(),
            "--term-old=fast",
            "--term-new=slow",
        ],
        temp_path.path(),
    );
    assert!(
        output.status.success(),
        "custom-term start failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // T2: `bisect terms` output text matches git's two-line form.
    let output = run_libra_command(&["bisect", "terms"], temp_path.path());
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Your current terms are 'fast' for the old state"),
        "terms output mismatch: {stdout}"
    );
    assert!(
        stdout.contains("and 'slow' for the new state."),
        "terms output mismatch: {stdout}"
    );

    let output = run_libra_command(&["bisect", "terms", "--term-good"], temp_path.path());
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "fast");
    let output = run_libra_command(&["bisect", "terms", "--term-old"], temp_path.path());
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "fast");
    let output = run_libra_command(&["bisect", "terms", "--term-bad"], temp_path.path());
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "slow");
    let output = run_libra_command(&["bisect", "terms", "--term-new"], temp_path.path());
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "slow");

    // T1/T7: custom term marks are accepted and drive the session; `log` and
    // `view` render the custom terms.
    let output = run_libra_command(&["bisect", "slow"], temp_path.path());
    assert!(
        output.status.success(),
        "bisect slow must be accepted: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let output = run_libra_command(&["bisect", "fast"], temp_path.path());
    assert!(
        output.status.success(),
        "bisect fast must be accepted: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let output = run_libra_command(&["bisect", "log"], temp_path.path());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("slow:"),
        "log must use the new term: {stdout}"
    );
    assert!(
        stdout.contains("fast:"),
        "log must use the old term: {stdout}"
    );
    let output = run_libra_command(&["bisect", "view"], temp_path.path());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("(fast)") && stdout.contains("(slow)"),
        "view must use custom terms: {stdout}"
    );

    // A fixed custom-term session refuses the default marks.
    let output = run_libra_command(&["bisect", "bad"], temp_path.path());
    assert!(!output.status.success(), "bisect bad must be refused");
    assert!(String::from_utf8_lossy(&output.stderr).contains("Invalid command"));

    execute_safe(Bisect::Reset { rev: None }, &OutputConfig::default())
        .await
        .unwrap();

    // T4: a bound-less start leaves terms unfixed; the first `new`/`old` mark
    // fixes them to new/old (git's check_and_set_terms semantics).
    execute_safe(
        Bisect::Start {
            bad: None,
            good: vec![],
            good_opt: None,
            term_old: vec![],
            term_new: vec![],
            first_parent: false,
        },
        &OutputConfig::default(),
    )
    .await
    .unwrap();
    let output = run_libra_command(&["bisect", "new", bad.as_str()], temp_path.path());
    assert!(
        output.status.success(),
        "bisect new must be accepted after a bound-less start: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let output = run_libra_command(&["bisect", "terms"], temp_path.path());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("'old'") && stdout.contains("'new'"),
        "new/old must fix the terms: {stdout}"
    );
    let output = run_libra_command(&["bisect", "old", good.as_str()], temp_path.path());
    assert!(
        output.status.success(),
        "bisect old must be accepted: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    execute_safe(Bisect::Reset { rev: None }, &OutputConfig::default())
        .await
        .unwrap();

    // T5: invalid terms are rejected with zero writes (no session, no terms).
    let output = run_libra_command(
        &[
            "bisect",
            "start",
            "--term-old=start",
            "--term-new=slow",
            bad.as_str(),
            good.as_str(),
        ],
        temp_path.path(),
    );
    assert!(
        !output.status.success(),
        "a builtin subcommand name must be refused as a term"
    );
    assert!(!BisectState::has_state().await.unwrap());

    let output = run_libra_command(
        &[
            "bisect",
            "start",
            "--term-old=slow",
            "--term-new=slow",
            bad.as_str(),
            good.as_str(),
        ],
        temp_path.path(),
    );
    assert!(!output.status.success(), "identical terms must be refused");
    assert!(!BisectState::has_state().await.unwrap());
}
