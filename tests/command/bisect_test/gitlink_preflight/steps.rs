//! Active marks/reset can encounter submodules materialized after bisect start.

use super::*;

#[test]
fn bisect_marks_preserve_head_index_and_session_when_gitlink_restore_is_refused() {
    for mark in ["bad", "good", "skip"] {
        // Given: an active session, then user materializes the current gitlink.
        let (repo, commits) = gitlink_history();
        let path = repo.path();
        let args = match mark {
            "bad" => {
                successful_stdout(path, &["bisect", "start"]);
                successful_stdout(path, &["bisect", "good", &commits[0]]);
                vec!["bisect", "bad", commits[4].as_str()]
            }
            "good" => {
                successful_stdout(path, &["bisect", "start", &commits[4]]);
                vec!["bisect", "good", commits[0].as_str()]
            }
            "skip" => {
                successful_stdout(
                    path,
                    &["bisect", "start", &commits[6], "--good", &commits[0]],
                );
                assert_eq!(successful_stdout(path, &["rev-parse", "HEAD"]), commits[3]);
                vec!["bisect", "skip"]
            }
            _ => unreachable!("test table has only bad/good/skip"),
        };
        fs::write(path.join("vendor/inner.txt"), "nested user bytes\n").unwrap();
        let before = snapshot(path);

        // When: marking a commit selects a candidate without the gitlink.
        let output = run_libra_command(&args, path);

        // Then: the rejected mark does not publish a new HEAD or session state.
        assert_gitlink_refusal(&output);
        assert_eq!(snapshot(path), before, "{mark}");
    }
}

#[test]
fn bisect_reset_clears_session_when_gitlink_restore_is_refused() {
    for reattach in [false, true] {
        // Given: an active session and a reset target without the materialized gitlink.
        let (repo, commits) = gitlink_history();
        let path = repo.path();
        successful_stdout(
            path,
            &["bisect", "start", &commits[6], "--good", &commits[0]],
        );
        let args = if reattach {
            successful_stdout(path, &["update-ref", "refs/heads/main", &commits[2]]);
            vec!["bisect", "reset"]
        } else {
            vec!["bisect", "reset", commits[2].as_str()]
        };
        fs::write(path.join("vendor/inner.txt"), "nested user bytes\n").unwrap();
        let before = snapshot(path);

        // When: reset attempts an attached or detached restore across the gitlink.
        let output = run_libra_command(&args, path);

        // Then: ADR-MX-03 §2 — a refused restore must NOT wedge the session.
        // `reset` exits 0, warns with the original target for a manual retry,
        // and clears the session row; HEAD, refs, index and worktree stay at
        // the pre-reset candidate untouched.
        assert_cli_success(&output, "refused reset still ends the session");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("failed to restore original HEAD"),
            "reset must warn about the failed restore: {stderr}"
        );
        assert!(
            stderr.contains("clearing the bisect session"),
            "reset must explain the cleared session: {stderr}"
        );
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("could not be restored"),
            "reset stdout must note the unrestored HEAD: {stdout}"
        );
        assert_eq!(successful_stdout(path, &["rev-parse", "HEAD"]), before.head);
        assert_eq!(successful_stdout(path, &["show-ref"]), before.refs);
        assert_eq!(
            successful_stdout(path, &["reflog", "show", "HEAD"]),
            before.reflog
        );
        assert_eq!(fs::read(path.join(".libra/index")).unwrap(), before.index);
        assert_eq!(fs::read(path.join("tracked.txt")).unwrap(), before.tracked);
        assert_eq!(
            fs::read(path.join("vendor/inner.txt")).unwrap(),
            before.nested
        );
        let view = run_libra_command(&["bisect", "view"], path);
        assert_eq!(view.status.code(), Some(128));
        let (_, report) = parse_cli_error_stderr(&view.stderr);
        assert_eq!(report.error_code, "LBR-BISECT-001");
    }
}

#[test]
fn bisect_reset_preserves_nested_content_when_target_retains_the_gitlink() {
    // Given: an active candidate and original branch both retain the gitlink.
    let (repo, commits) = gitlink_history();
    let path = repo.path();
    successful_stdout(
        path,
        &["bisect", "start", &commits[6], "--good", &commits[0]],
    );
    assert_eq!(successful_stdout(path, &["rev-parse", "HEAD"]), commits[3]);
    fs::write(path.join("vendor/inner.txt"), "nested user bytes\n").unwrap();

    // When: resetting to a different commit that still retains the pointer.
    let output = run_libra_command(&["bisect", "reset"], path);

    // Then: reset succeeds without touching nested user content.
    assert_cli_success(&output, "bisect reset retains materialized gitlink");
    assert_eq!(successful_stdout(path, &["rev-parse", "HEAD"]), commits[6]);
    assert_eq!(
        successful_stdout(path, &["symbolic-ref", "HEAD"]),
        "refs/heads/main"
    );
    assert_eq!(
        successful_stdout(path, &["ls-files", "--stage", "vendor"]),
        format!("160000 {GITLINK} 0\tvendor")
    );
    assert_eq!(
        fs::read(path.join("vendor/inner.txt")).unwrap(),
        b"nested user bytes\n"
    );
    assert!(
        !run_libra_command(&["bisect", "view"], path)
            .status
            .success()
    );
}
