//! Git-compat regression for issue #497: deleting the LAST tracked file must
//! produce a deletion commit, not "nothing to commit".
//!
//! The Git upstream scenario is `git rm <last-file> && git commit`, which
//! succeeds even though the worktree becomes empty. Libra must behave the same
//! for both `commit -a` and `add -A && commit`.

use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

use tempfile::{TempDir, tempdir};

struct CliFixture {
    _temp: TempDir,
    root: PathBuf,
    home: PathBuf,
    repo: PathBuf,
}

impl CliFixture {
    fn new() -> Self {
        let temp = tempdir().expect("create tempdir");
        let root = temp.path().to_path_buf();
        let home = root.join("home");
        let repo = root.join("repo");
        fs::create_dir_all(&home).expect("create isolated home");
        Self {
            _temp: temp,
            root,
            home,
            repo,
        }
    }

    fn command(&self, cwd: &Path, args: &[&str]) -> Command {
        let config_home = self.home.join(".config");
        let global_db = self.home.join(".libra").join("config.db");
        fs::create_dir_all(&config_home).expect("create isolated config dir");

        let mut command = Command::new(env!("CARGO_BIN_EXE_libra"));
        command
            .args(args)
            .current_dir(cwd)
            .env_clear()
            .env("PATH", "/usr/bin:/bin:/usr/sbin:/sbin")
            .env("HOME", &self.home)
            .env("USERPROFILE", &self.home)
            .env("XDG_CONFIG_HOME", &config_home)
            .env("LIBRA_CONFIG_GLOBAL_DB", &global_db)
            .env("LIBRA_TEST", "1")
            .env("LANG", "C")
            .env("LC_ALL", "C");
        if let Some(profile_file) = std::env::var_os("LLVM_PROFILE_FILE") {
            command.env("LLVM_PROFILE_FILE", profile_file);
        }
        command
    }

    fn run(&self, cwd: &Path, args: &[&str]) -> Output {
        self.command(cwd, args).output().expect("spawn libra")
    }

    fn success(&self, cwd: &Path, args: &[&str]) -> Output {
        let output = self.run(cwd, args);
        assert_success(args, &output);
        output
    }

    fn init_repo(&self) {
        fs::create_dir_all(&self.repo).expect("create repo dir");
        self.success(
            &self.root,
            &[
                "init",
                "--vault",
                "false",
                self.repo.to_str().expect("utf8 repo"),
            ],
        );
        self.success(&self.repo, &["config", "set", "user.name", "Config User"]);
        self.success(
            &self.repo,
            &["config", "set", "user.email", "config@example.com"],
        );
    }
}

fn assert_success(args: &[&str], output: &Output) {
    assert!(
        output.status.success(),
        "{} failed\nstdout:\n{}\nstderr:\n{}",
        args.join(" "),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn assert_failure(args: &[&str], output: &Output) {
    assert!(
        !output.status.success(),
        "{} unexpectedly succeeded\nstdout:\n{}\nstderr:\n{}",
        args.join(" "),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

/// Mirror the Git upstream scenario: deleting the LAST tracked file via
/// `rm` (staged) then `commit -a` must produce a deletion commit.
#[test]
fn git_deletes_last_tracked_file_via_git_rm_and_commit() {
    let temp = tempdir().expect("tempdir");
    let root = temp.path();
    let git_repo = root.join("git_repo");
    fs::create_dir_all(&git_repo).expect("create git repo");

    let git = |args: &[&str]| {
        Command::new("git")
            .args(args)
            .current_dir(&git_repo)
            .output()
            .expect("spawn git")
    };

    assert_success(&["git init"], &git(&["init", "-q"]));
    assert_success(&["git config"], &git(&["config", "user.email", "t@t.com"]));
    assert_success(&["git config"], &git(&["config", "user.name", "t"]));

    fs::write(git_repo.join("only.txt"), "only\n").expect("write only.txt");
    assert_success(&["git add"], &git(&["add", "only.txt"]));
    assert_success(&["git commit"], &git(&["commit", "-q", "-m", "baseline"]));

    assert_success(&["git rm"], &git(&["rm", "-q", "only.txt"]));
    // Deleting the last tracked file must still create a deletion commit.
    assert_success(
        &["git commit"],
        &git(&["commit", "-q", "-m", "delete last"]),
    );

    let ls_tree = git(&["ls-tree", "-r", "--name-only", "HEAD"]);
    assert_success(&["git ls-tree"], &ls_tree);
    let names = String::from_utf8_lossy(&ls_tree.stdout);
    assert!(
        names.trim().is_empty(),
        "git HEAD tree should be empty after deleting the last tracked file: {names}"
    );
}

/// Libra must match the above Git behavior for `libra rm` + `commit -a`.
#[test]
fn libra_deletes_last_tracked_file_via_rm_and_commit_all() {
    let fixture = CliFixture::new();
    fixture.init_repo();

    // Track exactly one file so removing it empties the index (issue #497).
    fs::write(fixture.repo.join("only.txt"), "only\n").expect("write only.txt");
    fixture.success(&fixture.repo, &["add", "only.txt"]);
    fixture.success(&fixture.repo, &["commit", "--no-verify", "-m", "baseline"]);

    fixture.success(&fixture.repo, &["rm", "only.txt"]);
    // Must succeed — not "nothing to commit".
    fixture.success(
        &fixture.repo,
        &["commit", "-a", "--no-verify", "-m", "delete last"],
    );

    let ls_tree = fixture.success(&fixture.repo, &["ls-tree", "-r", "--name-only", "HEAD"]);
    let names = String::from_utf8_lossy(&ls_tree.stdout);
    assert!(
        names.trim().is_empty(),
        "libra HEAD tree should be empty after deleting the last tracked file: {names}"
    );
}

/// Mirror the documented issue #497 path without `-a`: `libra rm` already
/// stages the deletion, so a plain `libra commit` must produce the deletion
/// commit too.
#[test]
fn libra_deletes_last_tracked_file_via_rm_and_plain_commit() {
    let fixture = CliFixture::new();
    fixture.init_repo();

    fs::write(fixture.repo.join("only.txt"), "only\n").expect("write only.txt");
    fixture.success(&fixture.repo, &["add", "only.txt"]);
    fixture.success(&fixture.repo, &["commit", "--no-verify", "-m", "baseline"]);

    fixture.success(&fixture.repo, &["rm", "only.txt"]);
    fixture.success(
        &fixture.repo,
        &["commit", "--no-verify", "-m", "delete last"],
    );

    let ls_tree = fixture.success(&fixture.repo, &["ls-tree", "-r", "--name-only", "HEAD"]);
    let names = String::from_utf8_lossy(&ls_tree.stdout);
    assert!(
        names.trim().is_empty(),
        "libra HEAD tree should be empty after deleting the last tracked file: {names}"
    );
}

/// `status --porcelain` and `ls-files` must agree with Git once the deletion
/// of the last tracked file is staged: `D  only.txt` and an empty file list.
#[test]
fn libra_status_and_ls_files_match_git_after_staged_deletion() {
    let fixture = CliFixture::new();
    fixture.init_repo();

    fs::write(fixture.repo.join("only.txt"), "only\n").expect("write only.txt");
    fixture.success(&fixture.repo, &["add", "only.txt"]);
    fixture.success(&fixture.repo, &["commit", "--no-verify", "-m", "baseline"]);
    fixture.success(&fixture.repo, &["rm", "only.txt"]);

    let status = fixture.success(&fixture.repo, &["status", "--porcelain"]);
    let status_text = String::from_utf8_lossy(&status.stdout);
    assert!(
        status_text.lines().any(|line| line == "D  only.txt"),
        "libra status --porcelain should report the staged deletion: {status_text}"
    );

    let ls_files = fixture.success(&fixture.repo, &["ls-files"]);
    let ls_files_text = String::from_utf8_lossy(&ls_files.stdout);
    assert!(
        ls_files_text.trim().is_empty(),
        "libra ls-files should be empty once the index is empty: {ls_files_text}"
    );

    // Git upstream duel: same scenario must yield the same porcelain entry and
    // an empty `git ls-files`.
    let temp = tempdir().expect("tempdir");
    let git_repo = temp.path().join("git_repo");
    fs::create_dir_all(&git_repo).expect("create git repo");
    let git = |args: &[&str]| {
        Command::new("git")
            .args(args)
            .current_dir(&git_repo)
            .output()
            .expect("spawn git")
    };
    assert_success(&["git init"], &git(&["init", "-q"]));
    assert_success(&["git config"], &git(&["config", "user.email", "t@t.com"]));
    assert_success(&["git config"], &git(&["config", "user.name", "t"]));
    fs::write(git_repo.join("only.txt"), "only\n").expect("write only.txt");
    assert_success(&["git add"], &git(&["add", "only.txt"]));
    assert_success(&["git commit"], &git(&["commit", "-q", "-m", "baseline"]));
    assert_success(&["git rm"], &git(&["rm", "-q", "only.txt"]));

    let git_status = git(&["status", "--porcelain=v1"]);
    assert_success(&["git status"], &git_status);
    let git_status_text = String::from_utf8_lossy(&git_status.stdout);
    let git_entries: Vec<&str> = git_status_text.lines().collect();
    assert_eq!(
        git_entries,
        vec!["D  only.txt"],
        "git status --porcelain=v1 should report exactly the staged deletion: {git_status_text}"
    );

    let git_ls_files = git(&["ls-files"]);
    assert_success(&["git ls-files"], &git_ls_files);
    assert!(
        String::from_utf8_lossy(&git_ls_files.stdout)
            .trim()
            .is_empty(),
        "git ls-files should be empty once the index is empty"
    );
}

/// Libra must match Git for the `add -A && commit` path.
#[test]
fn libra_deletes_last_tracked_file_via_add_all_and_commit() {
    let fixture = CliFixture::new();
    fixture.init_repo();

    fs::write(fixture.repo.join("only.txt"), "only\n").expect("write only.txt");
    fixture.success(&fixture.repo, &["add", "only.txt"]);
    fixture.success(&fixture.repo, &["commit", "--no-verify", "-m", "baseline"]);

    fs::remove_file(fixture.repo.join("only.txt")).expect("remove only.txt");
    fixture.success(&fixture.repo, &["add", "-A"]);
    // `add -A` may also pick up untracked files (the init-generated
    // `.libraignore` lands in the commit, matching `git add -A`); the
    // deletion itself must still land, so only.txt must be gone from HEAD.
    fixture.success(
        &fixture.repo,
        &["commit", "--no-verify", "-m", "delete last"],
    );

    let ls_tree = fixture.success(&fixture.repo, &["ls-tree", "-r", "--name-only", "HEAD"]);
    let names = String::from_utf8_lossy(&ls_tree.stdout);
    assert!(
        !names.contains("only.txt"),
        "libra HEAD tree should no longer contain only.txt: {names}"
    );
}

/// A genuinely clean repository (with HEAD content and no staged, unstaged or
/// untracked changes) must still refuse to commit with Git's exact
/// classification, `LBR-REPO-003`, and the documented exit-code contract
/// (128 by default, repo-category code 3 under `LIBRA_FINE_EXIT_CODES=1`).
#[test]
fn libra_clean_repo_still_reports_nothing_to_commit() {
    let fixture = CliFixture::new();
    fixture.init_repo();

    fs::write(fixture.repo.join("kept.txt"), "kept\n").expect("write kept.txt");
    // Track `.libraignore` too so the repo is genuinely clean (no untracked files).
    fixture.success(&fixture.repo, &["add", "kept.txt", ".libraignore"]);
    fixture.success(&fixture.repo, &["commit", "--no-verify", "-m", "baseline"]);

    // No changes staged and no untracked leftovers: must be refused.
    let output = fixture.run(&fixture.repo, &["commit", "--no-verify", "-m", "nothing"]);
    assert_failure(&["commit (clean)"], &output);
    assert_eq!(
        output.status.code(),
        Some(128),
        "clean commit must exit 128 by default"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("nothing to commit, working tree clean"),
        "clean repo should report the exact Git classification: {stderr}"
    );
    assert!(
        stderr.contains("LBR-REPO-003"),
        "clean commit failure must carry the stable error code: {stderr}"
    );

    // Fine-grained exit codes: the repo category code replaces 128.
    let fine = fixture
        .command(&fixture.repo, &["commit", "--no-verify", "-m", "nothing"])
        .env("LIBRA_FINE_EXIT_CODES", "1")
        .output()
        .expect("spawn libra with fine exit codes");
    assert_failure(&["commit (clean, fine exit codes)"], &fine);
    assert_eq!(
        fine.status.code(),
        Some(3),
        "clean commit must exit with the repo category code under LIBRA_FINE_EXIT_CODES=1"
    );
}

/// An unstaged filesystem deletion of a tracked file must NOT be picked up by
/// a plain `libra commit`: staging through `libra rm` / `libra add -A` (or
/// `commit -a`) is the prerequisite, matching Git's refusal.
#[test]
fn libra_unstaged_deletion_is_refused_by_plain_commit() {
    let fixture = CliFixture::new();
    fixture.init_repo();

    fs::write(fixture.repo.join("only.txt"), "only\n").expect("write only.txt");
    fixture.success(&fixture.repo, &["add", "only.txt", ".libraignore"]);
    fixture.success(&fixture.repo, &["commit", "--no-verify", "-m", "baseline"]);

    // Filesystem-only deletion; nothing staged.
    fs::remove_file(fixture.repo.join("only.txt")).expect("remove only.txt");

    let output = fixture.run(&fixture.repo, &["commit", "--no-verify", "-m", "unstaged"]);
    assert_failure(&["commit (unstaged deletion)"], &output);
    assert_eq!(
        output.status.code(),
        Some(128),
        "unstaged deletion must be refused with exit 128"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("no changes added to commit"),
        "unstaged deletion should report the unstaged classification: {stderr}"
    );
    assert!(
        stderr.contains("LBR-REPO-003"),
        "unstaged-deletion refusal must carry the stable error code: {stderr}"
    );
}
