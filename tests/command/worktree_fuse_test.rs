//! FUSE worktree tests (feature-gated).

use std::{
    fs,
    path::{Path, PathBuf},
};

use libra::{
    exec_async,
    utils::{fuse, test, util},
};
use serial_test::serial;
use tempfile::tempdir;

use super::*;

#[derive(Debug, serde::Deserialize)]
struct FuseState {
    #[serde(alias = "entries")]
    worktrees: Vec<FuseEntry>,
}

#[derive(Debug, serde::Deserialize)]
struct FuseEntry {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    mount_id: Option<String>,
    path: String,
    branch: String,
    upper_dir: String,
    lower_dirs: Vec<String>,
    locked: bool,
}

fn default_fuse_workspace(repo: &Path, name: &str) -> PathBuf {
    repo.join(".libra/worktree/fuse")
        .join(name)
        .join("workspace")
}

fn fuse_state_path() -> std::path::PathBuf {
    util::storage_path().join("worktrees-fuse.json")
}

fn read_fuse_state() -> FuseState {
    let data = fs::read_to_string(fuse_state_path()).expect("worktrees-fuse.json should exist");
    serde_json::from_str(&data).expect("fuse state should be valid json")
}

fn can_run_fuse_tests() -> bool {
    let requested = std::env::var("LIBRA_RUN_FUSE_TESTS")
        .ok()
        .map(|v| matches!(v.as_str(), "1" | "true" | "TRUE" | "yes" | "on"))
        .unwrap_or(false);
    if requested {
        assert!(
            Path::new("/dev/fuse").exists(),
            "LIBRA_RUN_FUSE_TESTS requires a host with /dev/fuse available"
        );
    }
    requested
}

fn try_write_probe(path: &Path) -> bool {
    let probe = path.join(".libra-fuse-probe");
    if fs::write(&probe, b"probe").is_err() {
        return false;
    }
    let _ = fs::remove_file(&probe);
    true
}

fn is_mounted(path: &Path) -> bool {
    let Ok(content) = fs::read_to_string("/proc/self/mountinfo") else {
        return false;
    };
    let target = path.to_string_lossy().to_string();
    content.lines().any(|line| {
        let fields: Vec<&str> = line.split_whitespace().collect();
        if fields.len() < 5 {
            return false;
        }
        fields[4].replace("\\040", " ") == target
    })
}

struct FuseMountCleanup {
    paths: Vec<PathBuf>,
}

impl FuseMountCleanup {
    fn new(paths: Vec<PathBuf>) -> Self {
        Self { paths }
    }
}

impl Drop for FuseMountCleanup {
    fn drop(&mut self) {
        for path in &self.paths {
            let _ = fuse::force_unmount_path(path);
        }
    }
}

/// The lower repository contains the FUSE mount itself. A real mount must
/// mask the entire common worktree container or it can reveal sibling
/// worktrees and recursively expose its own upper/mount directories.
#[test]
#[serial(cwd)]
#[ignore = "requires a Linux FUSE device and LIBRA_RUN_FUSE_TESTS=1"]
fn wd_fuse_live_mount_boundary() {
    std::thread::Builder::new()
        .name("wd-fuse-live".into())
        .stack_size(32 * 1024 * 1024)
        .spawn(|| {
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .enable_all()
                .build()
                .expect("create FUSE test runtime");
            runtime.block_on(async {
                assert!(
                    can_run_fuse_tests(),
                    "set LIBRA_RUN_FUSE_TESTS=1 for this live gate"
                );

                let repo_dir = tempdir().expect("create temp repo");
                test::setup_with_new_libra_in(repo_dir.path()).await;
                let _guard = test::ChangeDirGuard::new(repo_dir.path());
                let mount = default_fuse_workspace(repo_dir.path(), "wd-live");
                let _cleanup = FuseMountCleanup::new(vec![mount.clone()]);
                let sibling = repo_dir.path().join(".libra/worktree/linked/wd-private");
                fs::create_dir_all(&sibling).expect("create sibling worktree fixture");
                fs::write(sibling.join("secret.txt"), b"sibling data").expect("seed sibling data");

                exec_async(vec!["worktree", "add", "wd-live", "--fuse"])
                    .await
                    .expect("mount managed FUSE worktree");
                assert!(
                    is_mounted(&mount),
                    "managed workspace must be a real FUSE mount"
                );
                assert!(
                    try_write_probe(&mount),
                    "mounted workspace must support I/O"
                );
                assert!(
                    !mount
                        .join(".libra/worktree/linked/wd-private/secret.txt")
                        .exists(),
                    "FUSE lower must not reveal a sibling below the managed container"
                );
                assert!(
                    !mount.join(".libra/worktree/fuse/wd-live/upper").exists(),
                    "FUSE lower must not reveal its own upper directory"
                );
                assert_eq!(
                    fs::read_dir(
                        repo_dir
                            .path()
                            .join(".libra/worktree/fuse/wd-live/upper/.libra/worktree")
                    )
                    .expect("read opaque upper container")
                    .count(),
                    0,
                    "the opaque upper container must remain empty"
                );

                exec_async(vec!["worktree", "remove", "wd-live"])
                    .await
                    .expect("unmount and unregister managed FUSE worktree");
                assert!(!is_mounted(&mount), "managed workspace must be unmounted");
                assert_eq!(
                    fs::read(sibling.join("secret.txt")).unwrap(),
                    b"sibling data"
                );
            });
        })
        .expect("spawn FUSE test thread")
        .join()
        .expect("FUSE test thread panicked");
}

#[test]
fn test_fuse_worktree_umount_json_reports_cleanup_without_repo() {
    let temp = tempdir().expect("create temp dir");
    let cleanup_root = temp
        .path()
        .join("libra-task-worktree-fuse-29353-019ddec6-de60-7383");
    let workspace = cleanup_root.join("workspace");
    fs::create_dir_all(&workspace).expect("create task workspace");
    let canonical_cleanup_root = cleanup_root.canonicalize().expect("canonical cleanup root");
    let canonical_workspace = workspace.canonicalize().expect("canonical workspace");
    let cleanup_arg = cleanup_root.to_string_lossy().to_string();

    let output = run_libra_command(
        &[
            "--json",
            "worktree",
            "umount",
            cleanup_arg.as_str(),
            "--cleanup",
        ],
        temp.path(),
    );

    assert_cli_success(&output, "json fuse worktree umount --cleanup");
    assert!(
        output.stderr.is_empty(),
        "json fuse worktree umount should keep stderr clean: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let parsed = parse_json_stdout(&output);
    assert_eq!(parsed["command"], "worktree.umount");
    assert_eq!(
        parsed["data"]["mountpoint"],
        canonical_workspace.to_string_lossy().as_ref()
    );
    assert_eq!(parsed["data"]["unmounted"], true);
    assert_eq!(parsed["data"]["cleanup_requested"], true);
    assert_eq!(
        parsed["data"]["cleanup_root"],
        canonical_cleanup_root.to_string_lossy().as_ref()
    );
    assert_eq!(parsed["data"]["cleanup_root_removed"], true);
    assert!(
        !cleanup_root.exists(),
        "cleanup should remove the task worktree root"
    );
}

#[test]
fn managed_fuse_name_with_legacy_prefix_cannot_be_cleaned_as_a_task() {
    let temp = tempdir().expect("create temp dir");
    let managed = temp
        .path()
        .join(".libra/worktree/fuse/libra-task-worktree-fuse-collision");
    let workspace = managed.join("workspace");
    let upper = managed.join("upper");
    fs::create_dir_all(&workspace).expect("create workspace");
    fs::create_dir_all(&upper).expect("create upper");
    fs::write(upper.join("sentinel"), b"keep").expect("write upper sentinel");

    let output = run_libra_command(
        &[
            "worktree",
            "umount",
            workspace.to_str().expect("utf8 fixture"),
            "--cleanup",
        ],
        temp.path(),
    );
    assert!(!output.status.success(), "managed cleanup must be refused");
    assert_eq!(
        fs::read(upper.join("sentinel")).expect("upper kept"),
        b"keep"
    );
}

#[tokio::test]
#[serial(cwd)]
async fn test_fuse_worktree_metadata_management() {
    if !can_run_fuse_tests() {
        return;
    }

    let repo_dir = tempdir().expect("create temp repo");
    test::setup_with_new_libra_in(repo_dir.path()).await;
    let _guard = test::ChangeDirGuard::new(repo_dir.path());
    let _cleanup = FuseMountCleanup::new(vec![default_fuse_workspace(
        repo_dir.path(),
        "wt-fuse-meta",
    )]);

    if let Err(err) = exec_async(vec!["worktree", "add", "wt-fuse-meta", "--fuse"]).await {
        panic!("fuse add should succeed: {err}");
    }

    let mount_path = default_fuse_workspace(repo_dir.path(), "wt-fuse-meta");
    assert!(is_mounted(&mount_path), "FUSE workspace must be mounted");
    assert!(
        try_write_probe(&mount_path),
        "FUSE workspace must support I/O"
    );

    let state = read_fuse_state();
    assert_eq!(state.worktrees.len(), 1);
    let entry = &state.worktrees[0];
    assert!(entry.path.ends_with("wt-fuse-meta/workspace"));
    assert_eq!(entry.name.as_deref(), Some("wt-fuse-meta"));
    let mount_id = entry.mount_id.as_deref().expect("new mount binding");
    uuid::Uuid::parse_str(mount_id).expect("valid UUID mount binding");
    assert_eq!(
        fs::read_to_string(mount_path.join(".libra/worktree-mount-id"))
            .expect("mounted binding marker"),
        format!("{mount_id}\n")
    );
    assert!(!entry.upper_dir.is_empty());
    assert!(!entry.lower_dirs.is_empty());
    assert!(!entry.locked);
}

#[tokio::test]
#[serial(cwd)]
async fn test_fuse_worktree_add_list_remove_flow() {
    if !can_run_fuse_tests() {
        return;
    }

    let repo_dir = tempdir().expect("create temp repo");
    test::setup_with_new_libra_in(repo_dir.path()).await;
    let _guard = test::ChangeDirGuard::new(repo_dir.path());
    let _cleanup = FuseMountCleanup::new(vec![default_fuse_workspace(
        repo_dir.path(),
        "wt-fuse-flow",
    )]);

    if let Err(err) = exec_async(vec!["worktree", "add", "wt-fuse-flow", "--fuse"]).await {
        panic!("fuse add should succeed: {err}");
    }

    let mount_path = default_fuse_workspace(repo_dir.path(), "wt-fuse-flow");
    assert!(is_mounted(&mount_path), "FUSE workspace must be mounted");
    assert!(
        try_write_probe(&mount_path),
        "FUSE workspace must support I/O"
    );

    let list_output = run_libra_command(&["worktree", "list"], repo_dir.path());
    assert!(list_output.status.success(), "list should succeed");
    let stdout = String::from_utf8_lossy(&list_output.stdout);
    assert!(
        stdout.contains("wt-fuse-flow"),
        "list output should include fuse worktree: {stdout}"
    );

    if let Err(err) = exec_async(vec!["worktree", "remove", "wt-fuse-flow"]).await {
        panic!("fuse remove should succeed: {err}");
    }

    let state = read_fuse_state();
    assert!(state.worktrees.is_empty(), "fuse state should be cleaned");
}

#[tokio::test]
#[serial(cwd)]
async fn test_fuse_worktree_list_json_includes_fuse_entry() {
    if !can_run_fuse_tests() {
        return;
    }

    let repo_dir = tempdir().expect("create temp repo");
    test::setup_with_new_libra_in(repo_dir.path()).await;
    let _guard = test::ChangeDirGuard::new(repo_dir.path());
    let _cleanup = FuseMountCleanup::new(vec![default_fuse_workspace(
        repo_dir.path(),
        "wt-fuse-json",
    )]);

    if let Err(err) = exec_async(vec!["worktree", "add", "wt-fuse-json", "--fuse"]).await {
        panic!("fuse add should succeed: {err}");
    }

    let output = run_libra_command(&["--json", "worktree", "list"], repo_dir.path());
    assert_cli_success(&output, "json worktree list with fuse entry");
    assert!(
        output.stderr.is_empty(),
        "json worktree list should keep stderr clean: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let parsed = parse_json_stdout(&output);
    assert_eq!(parsed["command"], "worktree.list");
    let worktrees = parsed["data"]["worktrees"]
        .as_array()
        .expect("worktrees should be an array");
    let fuse_path = default_fuse_workspace(repo_dir.path(), "wt-fuse-json")
        .canonicalize()
        .unwrap();
    assert!(
        worktrees
            .iter()
            .any(|entry| entry["path"] == fuse_path.to_string_lossy().as_ref()),
        "json list should include FUSE worktree: {}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[tokio::test]
#[serial(cwd)]
async fn test_fuse_multiple_worktrees_mount_and_access() {
    if !can_run_fuse_tests() {
        return;
    }

    let repo_dir = tempdir().expect("create temp repo");
    test::setup_with_new_libra_in(repo_dir.path()).await;
    let _guard = test::ChangeDirGuard::new(repo_dir.path());
    let _cleanup = FuseMountCleanup::new(vec![
        default_fuse_workspace(repo_dir.path(), "wt-fuse-a"),
        default_fuse_workspace(repo_dir.path(), "wt-fuse-b"),
    ]);

    if let Err(err) = exec_async(vec!["worktree", "add", "wt-fuse-a", "--fuse"]).await {
        panic!("fuse add a should succeed: {err}");
    }
    if let Err(err) = exec_async(vec!["worktree", "add", "wt-fuse-b", "--fuse"]).await {
        panic!("fuse add b should succeed: {err}");
    }

    let a = default_fuse_workspace(repo_dir.path(), "wt-fuse-a");
    let b = default_fuse_workspace(repo_dir.path(), "wt-fuse-b");

    assert!(
        is_mounted(&a) && is_mounted(&b),
        "both workspaces must be mounted"
    );
    assert!(
        try_write_probe(&a) && try_write_probe(&b),
        "both workspaces must support I/O"
    );

    fs::write(a.join("only-a.txt"), b"a").expect("write a");
    fs::write(b.join("only-b.txt"), b"b").expect("write b");

    assert!(a.join("only-a.txt").exists());
    assert!(b.join("only-b.txt").exists());
    assert!(!a.join("only-b.txt").exists());
    assert!(!b.join("only-a.txt").exists());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[serial(cwd)]
async fn test_fuse_worktree_parallel_add_remove() {
    if !can_run_fuse_tests() {
        return;
    }

    let repo_dir = tempdir().expect("create temp repo");
    test::setup_with_new_libra_in(repo_dir.path()).await;
    let _guard = test::ChangeDirGuard::new(repo_dir.path());
    let _cleanup = FuseMountCleanup::new(vec![
        default_fuse_workspace(repo_dir.path(), "wt-par-1"),
        default_fuse_workspace(repo_dir.path(), "wt-par-2"),
    ]);

    let (r1, r2) = tokio::join!(
        exec_async(vec!["worktree", "add", "wt-par-1", "--fuse"]),
        exec_async(vec!["worktree", "add", "wt-par-2", "--fuse"])
    );
    if let Err(err) = r1 {
        panic!("parallel add 1 should succeed: {err}");
    }
    if let Err(err) = r2 {
        panic!("parallel add 2 should succeed: {err}");
    }

    let state = read_fuse_state();
    assert_eq!(
        state.worktrees.len(),
        2,
        "parallel adds must preserve both entries"
    );

    let (d1, d2) = tokio::join!(
        exec_async(vec!["worktree", "remove", "wt-par-1"]),
        exec_async(vec!["worktree", "remove", "wt-par-2"])
    );
    if let Err(err) = d1 {
        panic!("parallel remove 1 should succeed: {err}");
    }
    if let Err(err) = d2 {
        panic!("parallel remove 2 should succeed: {err}");
    }
}

#[tokio::test]
#[serial(cwd)]
async fn test_fuse_worktree_add_with_branch_and_create_branch() {
    if !can_run_fuse_tests() {
        return;
    }

    let repo_dir = tempdir().expect("create temp repo");
    test::setup_with_new_libra_in(repo_dir.path()).await;
    let _guard = test::ChangeDirGuard::new(repo_dir.path());
    let _cleanup = FuseMountCleanup::new(vec![default_fuse_workspace(
        repo_dir.path(),
        "wt-fuse-branch",
    )]);

    fs::write(repo_dir.path().join("seed.txt"), "seed\n").expect("write seed");
    let add_output = run_libra_command(&["add", "seed.txt"], repo_dir.path());
    assert!(add_output.status.success(), "add should succeed");
    let commit_output =
        run_libra_command(&["commit", "-m", "seed", "--no-verify"], repo_dir.path());
    assert!(commit_output.status.success(), "commit should succeed");
    let source_index = repo_dir.path().join(".libra/index");
    let index_before = fs::read(&source_index).expect("source index before overlay checkout");
    let source_root_before = util::working_dir();
    let source_storage_before = util::storage_path();
    fs::write(repo_dir.path().join("seed.txt"), "source-dirty\n").expect("dirty source worktree");

    if let Err(err) = exec_async(vec![
        "worktree",
        "add",
        "wt-fuse-branch",
        "--fuse",
        "--create-branch",
        "feature/fuse-wt",
    ])
    .await
    {
        panic!("fuse add with create-branch should succeed: {err}");
    }

    let mount_path = default_fuse_workspace(repo_dir.path(), "wt-fuse-branch");
    assert!(is_mounted(&mount_path), "FUSE workspace must be mounted");
    assert!(
        try_write_probe(&mount_path),
        "FUSE workspace must support I/O"
    );

    assert_eq!(
        fs::read(mount_path.join("seed.txt")).expect("materialized target branch"),
        b"seed\n"
    );
    assert_eq!(
        fs::read(repo_dir.path().join("seed.txt")).expect("source contents unchanged"),
        b"source-dirty\n"
    );
    assert_eq!(
        fs::read(&source_index).expect("source index after overlay checkout"),
        index_before
    );
    assert_eq!(
        util::working_dir(),
        source_root_before,
        "request destination scope must be restored"
    );
    assert_eq!(
        util::storage_path(),
        source_storage_before,
        "source common storage must remain pinned"
    );
    assert!(
        fs::read_dir(mount_path.join(".libra/worktree"))
            .expect("opaque container mask")
            .next()
            .is_none()
    );
    let state = read_fuse_state();
    assert_eq!(state.worktrees.len(), 1);
    assert_eq!(state.worktrees[0].branch, "feature/fuse-wt");
}

/// Regression (W3 §C.7): the fuse build REPLACES the worktree subcommand
/// surface, and its `Repair` variant must accept the same optional path as
/// the legacy command — `libra worktree repair <path>` restores a linked
/// worktree's gitdir identity in `--features worktree-fuse` builds too. No
/// FUSE mount is involved; this guards the CLI surface only.
#[test]
fn repair_path_restores_identity_in_fuse_build() {
    let dir = tempdir().expect("tempdir");
    let main = dir.path();
    assert_cli_success(&run_libra_command(&["init", "--vault=false"], main), "init");
    assert_cli_success(
        &run_libra_command(&["config", "user.name", "t"], main),
        "name",
    );
    assert_cli_success(
        &run_libra_command(&["config", "user.email", "t@t"], main),
        "email",
    );
    fs::write(main.join("a.txt"), "a").expect("seed file");
    assert_cli_success(&run_libra_command(&["add", "a.txt"], main), "add");
    assert_cli_success(
        &run_libra_command(&["commit", "-m", "c1", "--no-verify"], main),
        "commit",
    );

    let wt = main.join("wt-fuse-repair");
    assert_cli_success(
        &run_libra_command(
            &[
                "worktree",
                "add",
                "fuse-repair",
                "--path",
                wt.to_str().unwrap(),
            ],
            main,
        ),
        "worktree add (non-fuse) in fuse build",
    );
    let id_file = wt.join(".libra").join("worktree_id");
    let original_id = fs::read_to_string(&id_file)
        .expect("gitdir id")
        .trim()
        .to_string();
    fs::remove_file(&id_file).expect("drop id file");

    assert_cli_success(
        &run_libra_command(
            &[
                "worktree",
                "repair",
                "--path",
                wt.to_str().unwrap(),
                "--confirm",
            ],
            main,
        ),
        "worktree repair <path> in fuse build",
    );
    let restored = fs::read_to_string(&id_file)
        .expect("restored id")
        .trim()
        .to_string();
    assert_eq!(restored, original_id, "identity restored from the registry");
}

/// Read the `worktree repair` audit rows from a repository's operation log.
#[cfg(unix)]
async fn repair_audit_rows(repo: &Path) -> Vec<(String, String)> {
    use sea_orm::{ConnectOptions, ConnectionTrait, Database, DatabaseBackend, Statement};

    let url = format!("sqlite://{}", repo.join(".libra/libra.db").display());
    let mut opts = ConnectOptions::new(url);
    opts.sqlx_logging(false);
    let conn = Database::connect(opts).await.expect("open repo db");
    conn.query_all_raw(Statement::from_string(
        DatabaseBackend::Sqlite,
        "SELECT command_name, status FROM operation WHERE command_name = 'worktree' \
         ORDER BY op_id",
    ))
    .await
    .expect("read repair audit rows")
    .iter()
    .map(|row| {
        (
            row.try_get_by_index::<String>(0).expect("command_name"),
            row.try_get_by_index::<String>(1).expect("status"),
        )
    })
    .collect()
}

#[cfg(unix)]
fn init_repo_with_commit(main: &Path) {
    assert_cli_success(&run_libra_command(&["init", "--vault=false"], main), "init");
    assert_cli_success(
        &run_libra_command(&["config", "user.name", "t"], main),
        "name",
    );
    assert_cli_success(
        &run_libra_command(&["config", "user.email", "t@t"], main),
        "email",
    );
    fs::write(main.join("a.txt"), "a").expect("seed file");
    assert_cli_success(&run_libra_command(&["add", "a.txt"], main), "add");
    assert_cli_success(
        &run_libra_command(&["commit", "-m", "c1", "--no-verify"], main),
        "commit",
    );
}

/// W0 (§C.11, Codex R18): in a `--features worktree-fuse` build the no-arg
/// `worktree repair` runs the core registry repair AND the FUSE state repair
/// inside ONE operation-log audit boundary — exactly one row, closed with the
/// combined outcome. No FUSE mount is involved; this guards the boundary
/// composition only (the FUSE repair mutates `worktrees-fuse.json`).
#[cfg(unix)]
#[tokio::test]
#[serial(cwd, env, hash_kind)]
async fn fuse_repair_shares_one_audit_boundary_with_core_repair() {
    let dir = tempdir().expect("tempdir");
    let main = dir.path();
    init_repo_with_commit(main);

    // A duplicated FUSE state entry for the repair to drop.
    fs::write(
        main.join(".libra/worktrees-fuse.json"),
        r#"{"worktrees":[
            {"path":"/wt-a","branch":"b","upper_dir":"/u","lower_dirs":[],"locked":false,"lock_reason":null,"privileged":false,"allow_other":false},
            {"path":"/wt-a","branch":"b","upper_dir":"/u","lower_dirs":[],"locked":false,"lock_reason":null,"privileged":false,"allow_other":false}
        ]}"#,
    )
    .expect("seed fuse state");

    assert_cli_success(
        &run_libra_command(&["worktree", "repair", "--confirm"], main),
        "worktree repair --confirm (fuse build)",
    );

    let data =
        fs::read_to_string(main.join(".libra/worktrees-fuse.json")).expect("read fuse state");
    let state: FuseState = serde_json::from_str(&data).expect("fuse state is valid json");
    assert_eq!(state.worktrees.len(), 1, "fuse repair deduped the state");

    let rows = repair_audit_rows(main).await;
    assert_eq!(
        rows.len(),
        1,
        "exactly one audit row covers the combined repair: {rows:?}"
    );
    assert_eq!(rows[0].1, "success");
}

/// The same boundary, failing: a FUSE-state repair error must close the ONE
/// audit row as `failed` and fail the command — the pre-fix shape closed the
/// row as successful (core repair done) and THEN failed the command on the
/// FUSE mutation, leaving the failure unrecorded.
#[cfg(unix)]
#[tokio::test]
#[serial(cwd, env, hash_kind)]
async fn fuse_repair_failure_closes_the_shared_audit_row_failed() {
    let dir = tempdir().expect("tempdir");
    let main = dir.path();
    init_repo_with_commit(main);

    // Corrupt the FUSE state file so `repair_fuse_worktrees` fails to load it.
    fs::write(main.join(".libra/worktrees-fuse.json"), "{ not json").expect("corrupt fuse state");

    let out = run_libra_command(&["worktree", "repair", "--confirm"], main);
    assert!(
        !out.status.success(),
        "a FUSE-state repair failure must fail the command: {:?}",
        String::from_utf8_lossy(&out.stdout)
    );

    let rows = repair_audit_rows(main).await;
    assert_eq!(
        rows.len(),
        1,
        "exactly one audit row covers the combined repair: {rows:?}"
    );
    assert_eq!(
        rows[0].1, "failed",
        "the shared boundary closes with the combined failure"
    );
}

#[test]
fn fuse_explicit_path_preflight_precedes_branch_and_directory_creation() {
    let repo = tempdir().expect("temp repo");
    init_repo_with_commit(repo.path());
    let target = repo.path().join("recursive-target");
    let registry_before = fs::read(repo.path().join(".libra/worktrees.json")).ok();
    let output = run_libra_command(
        &[
            "worktree",
            "add",
            "rejected",
            "--fuse",
            "--path",
            target.to_str().expect("utf8 target"),
            "--new-branch",
            "feature/rejected",
        ],
        repo.path(),
    );
    assert!(
        !output.status.success(),
        "a target inside the lower tree must be rejected"
    );
    assert!(String::from_utf8_lossy(&output.stderr).contains("outside every source worktree"));
    assert!(!target.exists());
    assert!(!repo.path().join(".libra/worktree/fuse/rejected").exists());
    assert!(!repo.path().join(".libra/worktrees-fuse.json").exists());
    assert_eq!(
        fs::read(repo.path().join(".libra/worktrees.json")).ok(),
        registry_before
    );
    let branch = run_libra_command(
        &["show-ref", "--verify", "refs/heads/feature/rejected"],
        repo.path(),
    );
    assert!(
        !branch.status.success(),
        "invalid target must not create its requested branch"
    );
}

#[test]
fn fuse_explicit_target_rejects_final_symlink_and_resolved_parent_alias() {
    use std::os::unix::fs::symlink;

    let temp = tempdir().expect("tempdir");
    let repo = temp.path().join("repo");
    let outside = temp.path().join("outside");
    fs::create_dir(&repo).expect("repo");
    fs::create_dir(&outside).expect("outside");
    init_repo_with_commit(&repo);
    symlink(&repo, temp.path().join("repo-alias")).expect("source alias");
    symlink(&outside, temp.path().join("target-link")).expect("final target symlink");
    for target in [
        temp.path().join("repo-alias/recursive"),
        temp.path().join("target-link"),
    ] {
        let output = run_libra_command(
            &[
                "worktree",
                "add",
                "rejected",
                "--fuse",
                "--path",
                target.to_str().expect("target"),
            ],
            &repo,
        );
        assert!(
            !output.status.success(),
            "unsafe alias must fail: {}",
            target.display()
        );
    }
    assert!(!repo.join("recursive").exists());
    assert!(fs::read_dir(&outside).expect("outside").next().is_none());
    assert!(!repo.join(".libra/worktree/fuse/rejected").exists());
}

#[test]
fn fuse_named_controls_preserve_registration_when_cleanup_is_incomplete() {
    let repo = tempdir().expect("temp repo");
    init_repo_with_commit(repo.path());
    let root = repo.path().join(".libra/worktree/fuse/topic");
    let workspace = root.join("workspace");
    let upper = root.join("upper");
    fs::create_dir_all(&workspace).expect("workspace");
    fs::create_dir(&upper).expect("upper");
    let registry = repo.path().join(".libra/worktrees-fuse.json");
    let state = serde_json::json!({"schema_version":1,"entries":[{
        "name":"topic", "path":workspace.canonicalize().expect("workspace path"),
        "branch":"main", "upper_dir":upper.canonicalize().expect("upper path"),
        "lower_dirs":[repo.path().canonicalize().expect("repo path")],
        "locked":false,"lock_reason":null,"privileged":false,"allow_other":false
    }]});
    fs::write(&registry, serde_json::to_vec(&state).expect("json")).expect("registry");
    let repair = run_libra_command(&["worktree", "repair", "topic", "--confirm"], repo.path());
    assert!(
        !repair.status.success(),
        "FUSE identity repair is unsupported"
    );
    assert!(
        String::from_utf8_lossy(&repair.stderr)
            .contains("scoped identity repair is not supported for FUSE worktrees"),
        "the FUSE build must give an actionable repair limitation: {}",
        String::from_utf8_lossy(&repair.stderr)
    );
    assert_cli_success(
        &run_libra_command(
            &["worktree", "lock", "topic", "--reason", "retain"],
            repo.path(),
        ),
        "lock FUSE by name",
    );
    let locked: serde_json::Value =
        serde_json::from_slice(&fs::read(&registry).expect("state")).expect("json");
    assert_eq!(locked["entries"][0]["locked"], true);
    assert_cli_success(
        &run_libra_command(&["worktree", "unlock", "topic"], repo.path()),
        "unlock FUSE by name",
    );
    let move_output = run_libra_command(
        &[
            "worktree",
            "move",
            "topic",
            "--path",
            tempdir()
                .expect("temp move")
                .path()
                .to_str()
                .expect("move target"),
        ],
        repo.path(),
    );
    assert!(
        !move_output.status.success(),
        "FUSE move is explicitly unsupported"
    );
    fs::write(workspace.join("unexpected"), b"retain me").expect("unexpected post-unmount data");
    let remove_output = run_libra_command(&["worktree", "remove", "topic"], repo.path());
    assert!(
        !remove_output.status.success(),
        "non-empty workspace must retain registration"
    );
    let retained: serde_json::Value =
        serde_json::from_slice(&fs::read(&registry).expect("state")).expect("json");
    assert_eq!(retained["entries"][0]["name"], "topic");
    assert_eq!(
        fs::read(workspace.join("unexpected")).expect("preserved"),
        b"retain me"
    );
    assert!(
        upper.exists(),
        "failed removal must retain writable upper data"
    );
    fs::remove_file(workspace.join("unexpected")).expect("clear unexpected file");
    assert_cli_success(
        &run_libra_command(&["worktree", "remove", "topic"], repo.path()),
        "retry cleanup",
    );
    assert!(!root.exists());
    let removed: serde_json::Value =
        serde_json::from_slice(&fs::read(&registry).expect("state")).expect("json");
    assert!(removed["entries"].as_array().expect("entries").is_empty());
}

#[test]
fn fuse_legacy_unnamed_registry_still_supports_path_controls() {
    let repo = tempdir().expect("temp repo");
    init_repo_with_commit(repo.path());
    let storage = repo.path().join(".libra").canonicalize().expect("storage");
    let legacy_root = storage.join("worktrees-fuse/legacy-id");
    let upper = legacy_root.join("upper");
    fs::create_dir_all(&upper).expect("legacy upper");
    let external_dir = tempdir().expect("external mountpoint");
    let mountpoint = external_dir.path().canonicalize().expect("mountpoint");
    let registry = storage.join("worktrees-fuse.json");
    let entry = serde_json::json!({
        "path":mountpoint,
        "branch":"main",
        "upper_dir":upper,
        "lower_dirs":[repo.path().canonicalize().expect("lower")],
        "locked":false,
        "lock_reason":null,
        "privileged":false,
        "allow_other":false
    });
    fs::write(
        &registry,
        serde_json::to_vec(&serde_json::json!({"worktrees":[entry.clone()]})).expect("json"),
    )
    .expect("legacy registry");
    let path = mountpoint.to_str().expect("mountpoint UTF-8");

    let listed = run_libra_command(&["worktree", "list"], repo.path());
    assert_cli_success(&listed, "list legacy FUSE worktree");
    assert!(String::from_utf8_lossy(&listed.stdout).contains(path));
    assert_cli_success(
        &run_libra_command(&["worktree", "lock", "--path", path], repo.path()),
        "lock legacy FUSE worktree by path",
    );
    assert_cli_success(
        &run_libra_command(&["worktree", "unlock", "--path", path], repo.path()),
        "unlock legacy FUSE worktree by path",
    );
    assert_cli_success(
        &run_libra_command(&["worktree", "remove", "--path", path], repo.path()),
        "remove legacy FUSE worktree by path",
    );
    assert!(
        !upper.exists(),
        "legacy upper was cleaned after unmount proof"
    );
    let removed: serde_json::Value =
        serde_json::from_slice(&fs::read(&registry).expect("registry")).expect("updated registry");
    assert!(removed["entries"].as_array().expect("entries").is_empty());

    let missing = repo.path().join("missing-legacy-mount");
    let mut stale = entry;
    stale["path"] = serde_json::json!(missing);
    fs::write(
        &registry,
        serde_json::to_vec(&serde_json::json!({"worktrees":[stale]})).expect("json"),
    )
    .expect("stale registry");
    assert_cli_success(
        &run_libra_command(&["worktree", "prune"], repo.path()),
        "prune missing legacy FUSE mountpoint",
    );
    let pruned: serde_json::Value =
        serde_json::from_slice(&fs::read(&registry).expect("registry")).expect("updated registry");
    assert!(pruned["entries"].as_array().expect("entries").is_empty());
}
