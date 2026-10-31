//! Implements restore flows to reset files or entire trees from commits or the index, respecting pathspecs and staged vs worktree targets.

use std::{
    collections::{BTreeSet, HashMap, HashSet},
    env, fs,
    io::{self, Write},
    path::{Path, PathBuf},
};

use clap::Parser;
use git_internal::{
    hash::ObjectHash,
    internal::{
        index::{Index, IndexEntry},
        object::{
            blob::Blob,
            commit::Commit,
            tree::{Tree, TreeItemMode},
            types::ObjectType,
        },
    },
};
use serde::Serialize;

use crate::{
    command::{calc_file_blob_hash, load_object},
    internal::{
        branch::{self, Branch, BranchStoreError},
        head::Head,
        protocol::lfs_client::LFSClient,
    },
    utils::{
        client_storage::ClientStorage,
        error::{CliError, CliResult, StableErrorCode},
        lfs,
        object_ext::{BlobExt, CommitExt, TreeExt},
        output::{OutputConfig, emit_json_data},
        path,
        pathspec::{PathspecError, PathspecSet},
        util,
        worktree_blob::{WorktreeFileWriter, write_worktree_blob},
    },
};

const RESTORE_EXAMPLES: &str = "\
EXAMPLES:
    libra restore file.txt                Restore file from index to worktree
    libra restore --staged file.txt       Unstage a file (restore index from HEAD)
    libra restore --source HEAD~1 .       Restore all files from a previous commit
    libra restore -S -W file.txt          Restore both worktree and index
    libra restore --ours file.txt         Take the 'our' side of a merge conflict
    libra restore --theirs file.txt       Take the 'their' side of a merge conflict
    libra restore --merge file.txt        Recreate the conflict markers from the index stages
    libra restore --json --source HEAD .  Structured JSON output for agents";

// ── Typed error ──────────────────────────────────────────────────────

/// Typed error for checkout / restore operations, providing enough detail for
/// callers (e.g. `clone`) to map each failure into a stable error code without
/// resorting to string matching on `io::Error` messages.
#[derive(thiserror::Error, Debug)]
pub enum RestoreError {
    #[error("failed to resolve checkout source")]
    ResolveSource,
    #[error("reference is not a commit")]
    ReferenceNotCommit,
    #[error("pathspec '{0}' did not match any files")]
    PathspecNotMatched(String),
    #[error("failed to read index")]
    ReadIndex,
    #[error("failed to read object")]
    ReadObject,
    #[error("failed to read worktree")]
    ReadWorktree,
    #[error("invalid path encoding")]
    InvalidPathEncoding,
    #[error("failed to write worktree file")]
    WriteWorktree,
    #[error("refusing to replace non-empty worktree directory '{0}'")]
    NonEmptyWorktreeDirectory(String),
    /// The working tree holds something at a `160000` submodule path that Libra
    /// did not write: either an untracked file or symlink where a pointer must
    /// be materialized as a directory placeholder, or an entry where the index
    /// already records a pointer and the restore would remove or overwrite it.
    /// Either way the content is not Libra's, so the whole restore is refused
    /// before any write (ADR-MG-01).
    #[error(
        "refusing to replace worktree path '{0}': Libra does not manage submodule content there"
    )]
    SubmodulePathNotOwned(String),
    #[error("failed to download LFS content")]
    LfsDownload,
    /// Refused to restore from a Libra-managed locked branch (`intent`,
    /// `traces`, …). These refs hold AI-agent state that the user
    /// should not be able to overwrite with `restore --source`.
    #[error("refusing to restore from locked branch '{0}'")]
    LockedSource(String),
    /// Refused to mutate the worktree while HEAD is attached to a
    /// Libra-managed AI branch.
    #[error("refusing to restore worktree while on locked branch '{0}'")]
    LockedCurrentBranch(String),
    #[error("failed to read pathspec file: {0}")]
    PathspecFileRead(String),
    /// A matched path is unmerged (conflict stages 1/2/3 present) and no
    /// conflict-resolution flag was given. Mirrors Git's
    /// `path '<file>' is unmerged`.
    #[error("path '{0}' is unmerged")]
    PathUnmerged(String),
    /// `--overlay --ours`/`--theirs` asked for a conflict stage that the path
    /// does not have (a modify/delete conflict). This is the OVERLAY-mode error:
    /// it mirrors Git's `path '<file>' does not have our/their version`, which
    /// Git also only emits under `--overlay`. In the default (no-overlay) mode
    /// `restore` instead removes the worktree file (the deleting side wins), so
    /// this variant is not reached there.
    #[error("path '{path}' does not have {} version", stage_side(*stage))]
    MissingStageVersion { path: String, stage: u8 },
    /// `--conflict=<style>` was given an unsupported value (only `merge` and
    /// `diff3` are accepted; `zdiff3` is not implemented).
    #[error("unsupported conflict style '{0}' (expected 'merge' or 'diff3')")]
    UnsupportedConflictStyle(String),
    #[error("symlink checkout is not supported on this platform: {0}")]
    SymlinkUnsupported(String),
    #[error("{0}")]
    InvalidPathspec(String),
    /// Object-store construction or a tree read failed. The string is the
    /// I/O error, which names the storage path.
    #[error("{0}")]
    Storage(String),
}

/// Human label for a conflict stage used by [`RestoreError::MissingStageVersion`].
const fn stage_side(stage: u8) -> &'static str {
    match stage {
        2 => "our",
        3 => "their",
        _ => "a",
    }
}

impl RestoreError {
    fn stable_code(&self) -> StableErrorCode {
        match self {
            Self::ResolveSource => StableErrorCode::CliInvalidTarget,
            Self::ReferenceNotCommit => StableErrorCode::CliInvalidTarget,
            Self::PathspecNotMatched(_) => StableErrorCode::CliInvalidTarget,
            Self::ReadIndex => StableErrorCode::IoReadFailed,
            Self::ReadObject => StableErrorCode::IoReadFailed,
            Self::ReadWorktree => StableErrorCode::IoReadFailed,
            Self::InvalidPathEncoding => StableErrorCode::CliInvalidArguments,
            Self::WriteWorktree => StableErrorCode::IoWriteFailed,
            Self::NonEmptyWorktreeDirectory(_) | Self::SubmodulePathNotOwned(_) => {
                StableErrorCode::ConflictOperationBlocked
            }
            Self::LfsDownload => StableErrorCode::NetworkUnavailable,
            Self::LockedSource(_) => StableErrorCode::CliInvalidTarget,
            Self::LockedCurrentBranch(_) => StableErrorCode::ConflictOperationBlocked,
            Self::PathspecFileRead(_) => StableErrorCode::IoReadFailed,
            Self::PathUnmerged(_) => StableErrorCode::ConflictUnresolved,
            Self::MissingStageVersion { .. } => StableErrorCode::ConflictUnresolved,
            Self::UnsupportedConflictStyle(_) => StableErrorCode::CliInvalidArguments,
            Self::SymlinkUnsupported(_) => StableErrorCode::Unsupported,
            Self::InvalidPathspec(_) => StableErrorCode::CliInvalidTarget,
            Self::Storage(_) => StableErrorCode::IoReadFailed,
        }
    }
}

impl From<std::io::Error> for RestoreError {
    fn from(error: std::io::Error) -> Self {
        RestoreError::Storage(error.to_string())
    }
}

impl From<RestoreError> for CliError {
    fn from(error: RestoreError) -> Self {
        let stable_code = error.stable_code();
        let message = error.to_string();
        match error {
            // Ref resolution keeps Git-compatible exit 128 semantics even though
            // the stable code stays target-oriented for machine classification.
            RestoreError::ResolveSource => CliError::fatal(message)
                .with_stable_code(stable_code)
                .with_exit_code(128)
                .with_hint("check that the source ref exists with 'libra log'"),
            RestoreError::ReferenceNotCommit => CliError::fatal(message)
                .with_stable_code(stable_code)
                .with_exit_code(128)
                .with_hint("only commit references can be used as restore source"),
            RestoreError::PathspecNotMatched(_) => CliError::fatal(message)
                .with_stable_code(stable_code)
                .with_hint("check the path and try again"),
            RestoreError::LfsDownload => CliError::fatal(message)
                .with_stable_code(stable_code)
                .with_hint("check LFS server availability"),
            RestoreError::LockedSource(_) => CliError::fatal(message)
                .with_stable_code(stable_code)
                .with_exit_code(128)
                .with_hint(
                    "Libra-managed branches like 'intent' and 'traces' cannot be used as restore sources",
                ),
            RestoreError::LockedCurrentBranch(_) => CliError::fatal(message)
                .with_stable_code(stable_code)
                .with_hint("switch to a user branch before modifying the worktree"),
            // Unresolved-conflict failures keep Git's exit 128 while exposing the
            // ConflictUnresolved stable code for machine classification.
            RestoreError::PathUnmerged(_) => CliError::fatal(message)
                .with_stable_code(stable_code)
                .with_exit_code(128)
                .with_hint("resolve the conflict, or pass --ours/--theirs/--merge/--ignore-unmerged"),
            RestoreError::MissingStageVersion { .. } => CliError::fatal(message)
                .with_stable_code(stable_code)
                .with_exit_code(128)
                .with_hint("the path has no version at that conflict stage"),
            RestoreError::UnsupportedConflictStyle(_) => CliError::command_usage(message)
                .with_stable_code(stable_code)
                .with_hint("use --conflict=merge (default) or --conflict=diff3"),
            RestoreError::InvalidPathspec(_) => CliError::fatal(message)
                .with_stable_code(stable_code)
                .with_hint("use supported pathspec magic: top, exclude, icase, literal, glob"),
            RestoreError::NonEmptyWorktreeDirectory(_) => CliError::fatal(message)
                .with_stable_code(stable_code)
                .with_hint("move or remove nested files before restoring across the gitlink"),
            RestoreError::SubmodulePathNotOwned(_) => CliError::fatal(message)
                .with_stable_code(stable_code)
                .with_hint("move that path aside — Libra never writes or deletes submodule content"),
            _ => CliError::fatal(message).with_stable_code(stable_code),
        }
    }
}

// ── Structured output ────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct RestoreOutput {
    pub source: Option<String>,
    pub worktree: bool,
    pub staged: bool,
    pub restored_files: Vec<String>,
    pub deleted_files: Vec<String>,
}

#[derive(Debug, Clone, Copy)]
struct RestoreTarget {
    hash: ObjectHash,
    mode: Option<TreeItemMode>,
}

impl RestoreTarget {
    const fn new(hash: ObjectHash, mode: Option<TreeItemMode>) -> Self {
        Self { hash, mode }
    }

    fn index_mode(self) -> u32 {
        self.mode
            .and_then(tree_item_mode_to_index_mode)
            .unwrap_or(0o100644)
    }
}

// ── Entry points ─────────────────────────────────────────────────────

#[derive(Parser, Debug, Default)]
#[command(about = "Restore working tree files")]
#[command(after_help = RESTORE_EXAMPLES)]
pub struct RestoreArgs {
    /// files or dir to restore
    #[clap(required_unless_present = "pathspec_from_file")]
    pub pathspec: Vec<String>,
    /// source
    #[clap(long, short)]
    pub source: Option<String>,
    /// worktree
    #[clap(long, short = 'W')]
    pub worktree: bool,
    /// staged
    #[clap(long, short = 'S')]
    pub staged: bool,
    /// Restore the "our" side (conflict stage 2) of unmerged paths to the
    /// working tree. Reads the conflict index stages and writes the worktree
    /// only; the index is left unmerged. Mutually exclusive with `--theirs`,
    /// `--source`, `--staged`, and `--ignore-unmerged`.
    #[clap(
        long,
        short = '2',
        conflicts_with_all = ["theirs", "source", "staged", "ignore_unmerged"],
    )]
    pub ours: bool,
    /// Restore the "their" side (conflict stage 3) of unmerged paths to the
    /// working tree. Mutually exclusive with `--ours`, `--source`, `--staged`,
    /// and `--ignore-unmerged`.
    #[clap(
        long,
        short = '3',
        conflicts_with_all = ["source", "staged", "ignore_unmerged"],
    )]
    pub theirs: bool,
    /// Skip unmerged paths instead of erroring. Without a conflict-resolution
    /// flag, `restore` refuses to touch unmerged paths; `--ignore-unmerged`
    /// silently skips them and restores the rest.
    #[clap(long)]
    pub ignore_unmerged: bool,
    /// Recreate the conflict in the working tree for unmerged paths: write the
    /// conflict markers from the index stages (ours from stage 2, theirs from
    /// stage 3), leaving the index unmerged. Mutually exclusive with
    /// `--ours`/`--theirs`/`--source`/`--staged`/`--ignore-unmerged`. (Implied by
    /// `--conflict`.) Restore independently rebuilds whole-file `ours`/`theirs`
    /// markers from the index stages (one `ours` block / one `theirs` block), with
    /// generic `ours`/`theirs` labels since the stages carry no commit names — not
    /// Git's line-level 3-way merge, and unlike `libra merge`/`cherry-pick`, which
    /// now emit line-level hunks for both-modified text conflicts.
    #[clap(
        long,
        conflicts_with_all = ["ours", "theirs", "source", "staged", "ignore_unmerged"],
    )]
    pub merge: bool,
    /// Conflict-marker style for `--merge` (implies `--merge`): `merge` (default —
    /// `ours`/`theirs` blocks) or `diff3` (also include the `base` from stage 1).
    /// `zdiff3` is not supported.
    #[clap(
        long,
        value_name = "STYLE",
        conflicts_with_all = ["ours", "theirs", "source", "staged", "ignore_unmerged"],
    )]
    pub conflict: Option<String>,
    /// Read pathspecs from the given file, one per line (`-` reads stdin).
    #[clap(long = "pathspec-from-file", value_name = "FILE")]
    pub pathspec_from_file: Option<String>,
    /// Pathspecs read via --pathspec-from-file are separated by NUL, not newlines.
    #[clap(long = "pathspec-file-nul", requires = "pathspec_from_file")]
    pub pathspec_file_nul: bool,
    /// Do not show a progress meter. Accepted for Git parity and is a no-op:
    /// Libra's restore never renders a progress meter, so there is nothing to
    /// suppress.
    #[clap(long = "no-progress")]
    pub no_progress: bool,
    /// Restore in overlay mode: only create or update the paths present in the
    /// source; tracked paths that are absent from the source are left alone
    /// rather than removed. Toggle pair with `--no-overlay`; the last one wins.
    #[clap(long = "overlay", overrides_with = "no_overlay")]
    pub overlay: bool,
    /// Do not restore in overlay mode (the default): paths missing from the
    /// source are removed from the target so it matches the source exactly.
    /// Toggle pair with `--overlay`; the last one wins.
    #[clap(long = "no-overlay", overrides_with = "overlay")]
    pub no_overlay: bool,
}

pub async fn execute(args: RestoreArgs) {
    if let Err(e) = execute_safe(args, &OutputConfig::default()).await {
        e.print_stderr();
    }
}

/// Safe entry point that returns structured [`CliResult`] instead of printing
/// errors and exiting.
///
/// # Side Effects
/// - Restores selected paths from the index or a commit tree.
/// - May rewrite index entries when `--staged` is set.
/// - May overwrite working-tree files when the worktree target is active.
/// - Renders human or JSON output for restored paths.
///
/// # Errors
/// Returns [`CliError`] when the repository is missing, the source revision or
/// pathspecs cannot be resolved, object reads fail, or index/worktree writes
/// fail.
pub async fn execute_safe(args: RestoreArgs, output: &OutputConfig) -> CliResult<()> {
    util::require_repo().map_err(|_| CliError::repo_not_found())?;
    let result = run_restore(args).await.map_err(CliError::from)?;
    render_restore_output(&result, output)
}

pub(crate) async fn execute_to_output(args: RestoreArgs) -> CliResult<RestoreOutput> {
    util::require_repo().map_err(|_| CliError::repo_not_found())?;
    run_restore(args).await.map_err(CliError::from)
}

// ── Core execution ───────────────────────────────────────────────────

/// Read pathspecs from a file for `--pathspec-from-file`. Entries are separated
/// by newlines, or by NUL when `--pathspec-file-nul` is set; `-` reads stdin.
/// Empty entries are dropped (and a trailing `\r` is stripped in newline mode).
fn read_restore_pathspec_file(path: &str, nul: bool) -> Result<Vec<String>, RestoreError> {
    let raw = if path == "-" {
        use std::io::Read;
        let mut buf = String::new();
        std::io::stdin()
            .read_to_string(&mut buf)
            .map_err(|error| RestoreError::PathspecFileRead(error.to_string()))?;
        buf
    } else {
        std::fs::read_to_string(path)
            .map_err(|error| RestoreError::PathspecFileRead(format!("{path}: {error}")))?
    };

    let separator = if nul { '\0' } else { '\n' };
    Ok(raw
        .split(separator)
        .map(|entry| {
            if nul {
                entry
            } else {
                entry.strip_suffix('\r').unwrap_or(entry)
            }
        })
        .filter(|entry| !entry.is_empty())
        .map(str::to_string)
        .collect())
}

async fn compile_restore_pathspecs(raw: &[String]) -> Result<PathspecSet, RestoreError> {
    let workdir = util::try_working_dir().map_err(|_| RestoreError::ReadWorktree)?;
    compile_restore_pathspecs_at(raw, &workdir).await
}

async fn compile_restore_pathspecs_at(
    raw: &[String],
    workdir: &Path,
) -> Result<PathspecSet, RestoreError> {
    if raw.is_empty() {
        return Err(RestoreError::InvalidPathspec(
            "no pathspec was given".to_string(),
        ));
    }
    let current_dir = env::current_dir().map_err(|_| RestoreError::ReadWorktree)?;
    let ignore_case = crate::utils::path_case::effective_ignore_case()
        .await
        .map_err(|_| RestoreError::ReadWorktree)?;
    PathspecSet::from_workdir_with_default_icase(raw, &current_dir, workdir, ignore_case)
        .map_err(invalid_pathspec)
}

fn invalid_pathspec(error: PathspecError) -> RestoreError {
    RestoreError::InvalidPathspec(error.to_string())
}

// Deterministic filesystem race injection, absent from production builds.
#[cfg(test)]
type RestoreRaceHook = Box<dyn FnOnce(&Path)>;
#[cfg(test)]
thread_local! {
    static RESTORE_RACE_HOOK: std::cell::RefCell<Option<(&'static str, RestoreRaceHook)>> = const { std::cell::RefCell::new(None) };
}
#[cfg(test)]
fn run_restore_race_hook(phase: &str, path: &Path) {
    let hook = RESTORE_RACE_HOOK.with(|slot| {
        let mut slot = slot.borrow_mut();
        if slot
            .as_ref()
            .is_some_and(|(expected, _)| *expected == phase)
        {
            slot.take().map(|(_, hook)| hook)
        } else {
            None
        }
    });
    if let Some(hook) = hook {
        hook(path);
    }
}

async fn run_restore(mut args: RestoreArgs) -> Result<RestoreOutput, RestoreError> {
    // `--pathspec-from-file` populates the pathspec list from a file (or stdin
    // for `-`) before the normal restore logic runs.
    if let Some(file) = args.pathspec_from_file.take() {
        args.pathspec = read_restore_pathspec_file(&file, args.pathspec_file_nul)?;
    }
    let pathspecs = compile_restore_pathspecs(&args.pathspec).await?;
    let staged = args.staged;
    let mut worktree = args.worktree;
    if !staged {
        worktree = true;
    }

    const HEAD: &str = "HEAD";
    let mut source = args.source;
    if source.is_none() && staged {
        source = Some(HEAD.to_string());
    }

    // Refuse to use Libra-managed locked branches (`intent`, `traces`)
    // as a restore source. `is_locked_revision` also strips revision suffixes
    // (`~1`, `^`, `@{0}`) so users can't end-run the guard with `traces~1`.
    if let Some(src) = source.as_deref()
        && branch::is_locked_revision(src)
    {
        return Err(RestoreError::LockedSource(src.to_string()));
    }
    if worktree {
        reject_restore_on_ai_managed_current_branch().await?;
    }

    // Conflict-stage restore (`--ours`/`--theirs`): read the unmerged index
    // stages and write the working tree only, leaving the index unmerged. clap
    // guarantees `--source`/`--staged` are absent here, so this is purely a
    // worktree operation.
    if args.ours || args.theirs {
        let stage = if args.ours { 2 } else { 3 };
        let (restored, deleted) = restore_conflict_stage(&pathspecs, stage, args.overlay).await?;
        return Ok(RestoreOutput {
            source: None,
            worktree: true,
            staged: false,
            restored_files: restored,
            deleted_files: deleted,
        });
    }

    // Conflict re-materialization (`--merge` / `--conflict=<style>`): rewrite the
    // working tree for unmerged paths with the conflict markers rebuilt from the
    // index stages, leaving the index unmerged. clap guarantees
    // `--source`/`--staged` are absent here.
    if args.merge || args.conflict.is_some() {
        let diff3 = match args.conflict.as_deref() {
            None | Some("merge") => false,
            Some("diff3") => true,
            Some(other) => return Err(RestoreError::UnsupportedConflictStyle(other.to_string())),
        };
        let restored = restore_conflict_merge(&pathspecs, diff3).await?;
        return Ok(RestoreOutput {
            source: None,
            worktree: true,
            staged: false,
            restored_files: restored,
            deleted_files: Vec::new(),
        });
    }

    let storage = util::objects_storage()?;
    let mut target_blobs = resolve_target_blobs(source.as_deref(), staged, &storage).await?;

    // Unmerged guard: a plain restore must not silently act on a matched
    // unmerged path. Without an exemption this is a fatal error (Git's
    // `path '<file>' is unmerged`, exit 128); `--ignore-unmerged` instead drops
    // the unmerged paths from the restore set so the rest still restore.
    let mut skipped_unmerged_paths = Vec::new();
    let unmerged_matches = collect_matched_unmerged_paths(&pathspecs)?;
    if !unmerged_matches.is_empty() {
        if args.ignore_unmerged {
            // `skip` holds the index-relative unmerged paths matched by the
            // pathspecs. Drop them from the restore inputs so they are never
            // rewritten...
            skipped_unmerged_paths = unmerged_matches;
            target_blobs.retain(|(p, _)| !skipped_unmerged_paths.contains(p));
            // Paths matched only by skipped unmerged entries are treated as clean
            // no-ops below by including `skip` in the no-match allowance.
        } else {
            let first = path_to_utf8_typed(&unmerged_matches[0])?;
            return Err(RestoreError::PathUnmerged(first.to_string()));
        }
    }

    let mut restored_files = Vec::new();
    let mut deleted_files = Vec::new();

    // Overlay mode (`--overlay`) only creates/updates paths from the source and
    // never removes tracked paths absent from it; the default (`--no-overlay`)
    // removes them so the target matches the source exactly.
    let overlay = args.overlay;

    let stat_snapshots = if worktree {
        let (restored, deleted, snapshots) = restore_worktree_tracked(
            &pathspecs,
            &target_blobs,
            overlay,
            &skipped_unmerged_paths,
            None,
        )
        .await?;
        restored_files.extend(restored);
        deleted_files.extend(deleted);
        snapshots
    } else {
        HashMap::new()
    };
    if staged {
        let (restored, deleted) = restore_index_tracked(
            &pathspecs,
            &target_blobs,
            overlay,
            &skipped_unmerged_paths,
            worktree,
            None,
            Some(&stat_snapshots),
        )?;
        let mut restored_seen: HashSet<String> = restored_files.iter().cloned().collect();
        let mut deleted_seen: HashSet<String> = deleted_files.iter().cloned().collect();

        for f in restored {
            if restored_seen.insert(f.clone()) {
                restored_files.push(f);
            }
        }
        for f in deleted {
            if deleted_seen.insert(f.clone()) {
                deleted_files.push(f);
            }
        }
    }

    Ok(RestoreOutput {
        source: source.clone(),
        worktree,
        staged,
        restored_files,
        deleted_files,
    })
}

// ── Rendering ────────────────────────────────────────────────────────

fn render_restore_output(result: &RestoreOutput, output: &OutputConfig) -> CliResult<()> {
    if output.is_json() {
        return emit_json_data("restore", result, output);
    }

    if output.quiet {
        return Ok(());
    }

    let total = result.restored_files.len() + result.deleted_files.len();
    if total > 0 {
        let source_desc = result.source.as_deref().unwrap_or("the index");
        println!("Updated {total} path(s) from {source_desc}");
    }
    Ok(())
}

// ── Resolve target blobs ─────────────────────────────────────────────

async fn resolve_target_blobs(
    source: Option<&str>,
    staged: bool,
    storage: &ClientStorage,
) -> Result<Vec<(PathBuf, RestoreTarget)>, RestoreError> {
    const HEAD: &str = "HEAD";

    match source {
        None => {
            if staged {
                return Err(RestoreError::ResolveSource);
            }
            let index = Index::load(path::index()).map_err(|_| RestoreError::ReadIndex)?;
            Ok(index
                .tracked_entries(0)
                .into_iter()
                .map(|entry| {
                    (
                        PathBuf::from(&entry.name),
                        RestoreTarget::new(entry.hash, index_mode_to_tree_item_mode(entry.mode)),
                    )
                })
                .collect())
        }
        Some(src) => {
            let commit = if src == HEAD {
                Head::current_commit_result()
                    .await
                    .map_err(map_restore_branch_store_error)?
                    .ok_or(RestoreError::ResolveSource)?
            } else if src.contains('~') || src.contains('^') {
                util::get_commit_base_typed(src)
                    .await
                    .map_err(|_| RestoreError::ResolveSource)?
            } else {
                resolve_source_commit(src, storage).await?
            };

            let tree_id = load_object::<Commit>(&commit)
                .map_err(|_| RestoreError::ReadObject)?
                .tree_id;
            Ok(load_object::<Tree>(&tree_id)
                .map_err(|_| RestoreError::ReadObject)?
                .get_plain_items_with_mode()?
                .into_iter()
                .map(|(path, hash, mode)| (path, RestoreTarget::new(hash, Some(mode))))
                .collect())
        }
    }
}

// ── Worktree restore (unified typed path) ────────────────────────────

async fn restore_worktree_tracked(
    pathspecs: &PathspecSet,
    target_blobs: &[(PathBuf, RestoreTarget)],
    overlay: bool,
    allowed_unmatched: &[PathBuf],
    destination_root: Option<&Path>,
) -> Result<(Vec<String>, Vec<String>, HashMap<PathBuf, fs::Metadata>), RestoreError> {
    let target_map = preprocess_blobs(target_blobs);
    let index = Index::load(path::index()).map_err(|_| RestoreError::ReadIndex)?;
    let file_paths =
        collect_restore_worktree_paths(pathspecs, &target_map, &index, allowed_unmatched)?;
    if let Some(root) = destination_root {
        for path in &file_paths {
            validate_explicit_destination_path(root, path)?;
        }
    }
    preflight_worktree_directory_transitions(
        &file_paths,
        &target_map,
        &index,
        overlay,
        destination_root,
    )?;
    let mut restored = Vec::new();
    let mut deleted = Vec::new();
    let mut stat_snapshots = HashMap::new();
    let workdir = match destination_root {
        Some(root) => root.to_path_buf(),
        None => util::try_working_dir().map_err(|_| RestoreError::ReadWorktree)?,
    };

    for path_wd in &file_paths {
        if let Some(root) = destination_root {
            // Earlier entries may have materialized a symlink that is a
            // parent of this path, even if the initial preflight was safe.
            validate_explicit_destination_path(root, path_wd)?;
        }
        let path_abs = workdir.join(path_wd);
        let path_wd_str = path_to_utf8_typed(path_wd)?;
        let tracked = index.tracked(path_wd_str, 0);
        if !worktree_path_exists(&path_abs) {
            if let Some(target) = target_map.get(path_wd) {
                if let Some(snapshot) =
                    restore_target_to_file_typed(*target, path_wd, Some(&workdir)).await?
                {
                    stat_snapshots.insert(path_wd.clone(), snapshot);
                }
                restored.push(path_wd.display().to_string());
            } else if !tracked {
                return Err(pathspec_not_matched(path_wd));
            }
        } else if let Some(target) = target_map.get(path_wd) {
            let mode_matches = worktree_mode_matches(&path_abs, target.mode)?;
            let content_matches = if matches!(target.mode, Some(TreeItemMode::Commit)) {
                // Gitlink commits live in the nested repository, so there is no
                // parent-repository blob to hash or load here.
                true
            } else if !mode_matches {
                false
            } else {
                let pre_hash =
                    fs::symlink_metadata(&path_abs).map_err(|_| RestoreError::ReadWorktree)?;
                let matches = calc_file_blob_hash(&path_abs)
                    .map_err(|_| RestoreError::ReadObject)?
                    == target.hash;
                #[cfg(test)]
                run_restore_race_hook("matched-hash", &path_abs);
                if matches {
                    stat_snapshots.insert(path_wd.clone(), pre_hash);
                }
                matches
            };
            if !content_matches || !mode_matches {
                if let Some(snapshot) =
                    restore_target_to_file_typed(*target, path_wd, Some(&workdir)).await?
                {
                    stat_snapshots.insert(path_wd.clone(), snapshot);
                }
                restored.push(path_wd.display().to_string());
            }
        } else if !overlay && tracked {
            remove_worktree_path_for_restore(&path_abs)?;
            clear_empty_restore_parents(&path_abs, destination_root);
            deleted.push(path_wd.display().to_string());
        }
    }

    Ok((restored, deleted, stat_snapshots))
}

// ── Index restore (unified typed path) ───────────────────────────────

fn restore_index_tracked(
    pathspecs: &PathspecSet,
    target_blobs: &[(PathBuf, RestoreTarget)],
    overlay: bool,
    allowed_unmatched: &[PathBuf],
    materialized_worktree: bool,
    destination_root: Option<&Path>,
    stat_snapshots: Option<&HashMap<PathBuf, fs::Metadata>>,
) -> Result<(Vec<String>, Vec<String>), RestoreError> {
    let target_map = preprocess_blobs(target_blobs);

    let idx_file = path::index();
    let mut index = Index::load(&idx_file).map_err(|_| RestoreError::ReadIndex)?;
    // Source paths missing from the index — added in BOTH modes. Overlay only
    // suppresses *removal* of index entries absent from the source (gated below).
    let deleted_files_index =
        get_index_deleted_files_in_filters_typed(&index, pathspecs, &target_map)?;

    ensure_positive_pathspecs_match(
        pathspecs,
        restore_match_candidates(&target_map, &index, allowed_unmatched),
    )?;
    let mut file_paths = filter_paths(&index.tracked_files(), pathspecs);
    file_paths.extend(deleted_files_index);

    // Resolve the destination once for the whole index pass, not per entry.
    let workdir = match destination_root {
        Some(root) => root.to_path_buf(),
        None => util::try_working_dir().map_err(|_| RestoreError::ReadWorktree)?,
    };
    let mut verified_entries = HashMap::new();
    if materialized_worktree {
        for path in &file_paths {
            if let Some(target) = target_map.get(path)
                && let Some(entry) = restore_index_entry(
                    path,
                    *target,
                    &workdir,
                    stat_snapshots.and_then(|snapshots| snapshots.get(path)),
                )
            {
                verified_entries.insert(path.clone(), entry);
            }
        }
    }

    // Batch-resolve the blob lengths needed to author new or updated index entries.
    //
    // The helper below otherwise loads an entire blob just to read `data.len()`. On a
    // fresh worktree every path is new, and switching `--source` across commits rewrites
    // every changed entry, so a naive pass would do one full object load (and decompress)
    // per touched entry. Index sizes are the decoded blob length, not an on-disk byte
    // count or delta-chain memory budget; `decoded_object_sizes` answers them without
    // materializing any payload, in one storage-runtime round trip. Unique hashes are
    // deduplicated because one blob is commonly referenced by several paths.
    let mut size_cache: Option<HashMap<ObjectHash, u32>> = Some(HashMap::new());
    {
        let mut needed: Vec<ObjectHash> = Vec::new();
        let mut seen: HashSet<ObjectHash> = HashSet::new();
        for path in &file_paths {
            let path_str = path_to_utf8_typed(path)?;
            let Some(target) = target_map.get(path) else {
                continue;
            };
            if verified_entries.contains_key(path)
                || matches!(target.mode, Some(TreeItemMode::Commit))
            {
                continue;
            }
            let needs_size = match index.get(path_str, 0) {
                None => true,
                Some(entry) => {
                    !index.verify_hash(path_str, 0, &target.hash)
                        || entry.mode != target.index_mode()
                        || materialized_worktree
                }
            };
            if needs_size && seen.insert(target.hash) {
                needed.push(target.hash);
            }
        }
        if !needed.is_empty() {
            match util::objects_storage()?.decoded_object_sizes(&needed) {
                Ok(sizes) => {
                    let mut cache: HashMap<ObjectHash, u32> = HashMap::with_capacity(needed.len());
                    for (hash, size) in needed.into_iter().zip(sizes) {
                        if let Some(size) = size.and_then(|s| u32::try_from(s).ok()) {
                            cache.insert(hash, size);
                        }
                    }
                    size_cache = Some(cache);
                }
                // Fall back to per-object loads; the error surface is unchanged.
                Err(error) => tracing::warn!(
                    count = needed.len(),
                    %error,
                    "batch decoded-object-size lookup failed; falling back to per-object loads"
                ),
            }
        }
    }

    let mut restored = Vec::new();
    let mut deleted = Vec::new();

    for path in &file_paths {
        let path_str = path_to_utf8_typed(path)?;
        if !index.tracked(path_str, 0) {
            if let Some(target) = target_map.get(path) {
                crate::utils::index_ext::update_preserving_flags(
                    &mut index,
                    if let Some(entry) = verified_entries.remove(path) {
                        entry
                    } else {
                        index_entry_from_target(
                            path_str.to_string(),
                            *target,
                            restore_target_index_size(*target, size_cache.as_mut())?,
                        )
                    },
                );
                restored.push(path.display().to_string());
            } else {
                return Err(pathspec_not_matched(path));
            }
        } else if let Some(target) = target_map.get(path) {
            let mode_matches = index
                .get(path_str, 0)
                .map(|entry| entry.mode == target.index_mode())
                .unwrap_or(false);
            let index_changed = !index.verify_hash(path_str, 0, &target.hash) || !mode_matches;
            if index_changed
                || (materialized_worktree && !matches!(target.mode, Some(TreeItemMode::Commit)))
            {
                crate::utils::index_ext::update_preserving_flags(
                    &mut index,
                    if let Some(entry) = verified_entries.remove(path) {
                        entry
                    } else {
                        index_entry_from_target(
                            path_str.to_string(),
                            *target,
                            restore_target_index_size(*target, size_cache.as_mut())?,
                        )
                    },
                );
                // Refreshing internal stat data is not a user-visible restore.
                if index_changed {
                    restored.push(path.display().to_string());
                }
            }
        } else if !overlay {
            index.remove(path_str, 0);
            deleted.push(path.display().to_string());
        }
    }

    index
        .save(&idx_file)
        .map_err(|_| RestoreError::WriteWorktree)?;

    Ok((restored, deleted))
}

/// Build from verified worktree bytes, preserving observed size even when the
/// metadata must be smudged. Hash/read failures use the batched size fallback.
fn restore_index_entry(
    path: &Path,
    target: RestoreTarget,
    workdir: &Path,
    pre_hash: Option<&fs::Metadata>,
) -> Option<IndexEntry> {
    if matches!(target.mode, Some(TreeItemMode::Commit)) {
        return None;
    }
    let absolute = workdir.join(path);
    let owned_snapshot;
    let before_hash = if let Some(snapshot) = pre_hash {
        snapshot
    } else {
        owned_snapshot = fs::symlink_metadata(&absolute).ok()?;
        &owned_snapshot
    };
    if before_hash.len() > u64::from(u32::MAX)
        || (pre_hash.is_none() && restored_worktree_blob_hash(&absolute).ok() != Some(target.hash))
    {
        return None;
    }
    // Non-Unix metadata uses creation time rather than an edit-sensitive ctime.
    // Keep the observed size, but force content comparison on the next status.
    let verification_snapshot = cfg!(unix).then_some(before_hash);
    let mut entry =
        crate::command::verified_index_entry(path, target.hash, workdir, verification_snapshot)
            .ok()?;
    // Smudging must retain the payload size, rather than an LFS pointer length.
    // The shared stat comparator never grants trust to zero timestamps.
    entry.mode = target.index_mode();
    Some(entry)
}

fn restored_worktree_blob_hash(path: &Path) -> io::Result<ObjectHash> {
    #[cfg(test)]
    run_restore_race_hook("fallback-hash", path);
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() {
        return Ok(Blob::from_content_bytes(crate::command::read_symlink_blob_bytes(path)?).id);
    }
    if lfs::is_lfs_tracked(path) {
        let (pointer, _) = lfs::generate_pointer_file_result(path)?;
        return Ok(Blob::from_content(&pointer).id);
    }
    crate::command::stream_file_blob_hash_bounded(path, u64::from(u32::MAX))?
        .0
        .ok_or_else(|| io::Error::other("worktree file changed while hashing"))
}

// ── Legacy public API (used by worktree.rs and checkout) ─────────────

/// Low-level restore that skips the repository-existence check.
///
/// # Preconditions
///
/// The caller **must** ensure a valid libra repository is reachable from the
/// current working directory (e.g. by calling `util::require_repo()` or
/// `execute_safe()` first).  This function is `pub` because it is used by
/// `worktree.rs`, which performs its own repository validation.
pub async fn execute_checked(args: RestoreArgs) -> io::Result<()> {
    let staged = args.staged;
    let mut worktree = args.worktree;
    if !staged {
        worktree = true;
    }

    const HEAD: &str = "HEAD";
    let mut source = args.source;
    if source.is_none() && staged {
        source = Some(HEAD.to_string());
    }

    let storage = util::objects_storage()?;
    let target_commit: Option<ObjectHash> = match source {
        None => {
            assert!(!staged);
            None
        }
        Some(ref src) => {
            if src == HEAD {
                Some(
                    Head::current_commit_result()
                        .await
                        .map_err(|error| io::Error::other(error.to_string()))?
                        .ok_or_else(|| io::Error::other("could not resolve HEAD"))?,
                )
            } else if src.contains('~') || src.contains('^') {
                Some(
                    util::get_commit_base_typed(src)
                        .await
                        .map_err(|error| io::Error::other(error.to_string()))?,
                )
            } else {
                resolve_source_commit_io(src, &storage)
                    .await
                    .map(Some)
                    .map_err(|error| io::Error::other(error.to_string()))?
            }
        }
    };

    let target_blobs: Vec<(PathBuf, RestoreTarget)> = {
        match (source.as_ref(), target_commit) {
            (None, _) => {
                assert!(!staged);
                let index =
                    Index::load(path::index()).map_err(|e| io::Error::other(e.to_string()))?;
                index
                    .tracked_entries(0)
                    .into_iter()
                    .map(|entry| {
                        (
                            PathBuf::from(&entry.name),
                            RestoreTarget::new(
                                entry.hash,
                                index_mode_to_tree_item_mode(entry.mode),
                            ),
                        )
                    })
                    .collect()
            }
            (Some(_), Some(commit)) => {
                let tree_id = Commit::load(&commit)?.tree_id;
                let tree = Tree::load(&tree_id)?;
                tree.get_plain_items_with_mode()?
                    .into_iter()
                    .map(|(path, hash, mode)| (path, RestoreTarget::new(hash, Some(mode))))
                    .collect()
            }
            (Some(src), None) => {
                if storage
                    .search_result(src)
                    .await
                    .map_err(|error| io::Error::other(error.to_string()))?
                    .len()
                    != 1
                {
                    return Err(io::Error::other(format!("could not resolve {src}")));
                } else {
                    return Err(io::Error::other(format!(
                        "reference is not a commit: {src}"
                    )));
                }
            }
        }
    };

    let pathspecs = compile_restore_pathspecs(&args.pathspec)
        .await
        .map_err(|error| io::Error::other(error.to_string()))?;

    let stat_snapshots = if worktree {
        let (_, _, snapshots) =
            restore_worktree_tracked(&pathspecs, &target_blobs, false, &[], None)
                .await
                .map_err(|error| io::Error::other(error.to_string()))?;
        snapshots
    } else {
        HashMap::new()
    };
    if staged {
        restore_index_tracked(
            &pathspecs,
            &target_blobs,
            false,
            &[],
            worktree,
            None,
            Some(&stat_snapshots),
        )
        .map_err(|error| io::Error::other(error.to_string()))?;
    }
    Ok(())
}

/// Typed checkout entry point that returns [`RestoreError`] instead of
/// `io::Error`, allowing callers like `clone` to map each failure category
/// into a distinct stable error code.
pub async fn execute_checked_typed(args: RestoreArgs) -> Result<(), RestoreError> {
    execute_checked_typed_at(args, None).await
}

/// Restore tracked files into an explicit destination directory while keeping
/// the caller's repository objects, index, and branch resolution as the source.
/// Only `:/` is accepted because relative pathspecs would use the caller's cwd.
#[cfg(any(test, all(unix, feature = "worktree-fuse")))]
pub(crate) async fn execute_checked_typed_into(
    args: RestoreArgs,
    destination_root: &Path,
) -> Result<(), RestoreError> {
    if !destination_root.is_absolute() {
        return Err(RestoreError::ReadWorktree);
    }
    if args.staged || args.pathspec.as_slice() != [":/"] {
        return Err(RestoreError::InvalidPathspec(
            "explicit destination restore requires only the repository-root pathspec ':/' and cannot update the index".to_string(),
        ));
    }
    execute_checked_typed_at(args, Some(destination_root)).await
}

async fn execute_checked_typed_at(
    args: RestoreArgs,
    destination_root: Option<&Path>,
) -> Result<(), RestoreError> {
    let staged = args.staged;
    let mut worktree = args.worktree;
    if !staged {
        worktree = true;
    }

    const HEAD: &str = "HEAD";
    let mut source = args.source;
    if source.is_none() && staged {
        source = Some(HEAD.to_string());
    }

    let storage = util::objects_storage()?;
    let target_blobs: Vec<(PathBuf, RestoreTarget)> = match source.as_ref() {
        None => {
            if staged {
                return Err(RestoreError::ResolveSource);
            }
            let index = Index::load(path::index()).map_err(|_| RestoreError::ReadIndex)?;
            index
                .tracked_entries(0)
                .into_iter()
                .map(|entry| {
                    (
                        PathBuf::from(&entry.name),
                        RestoreTarget::new(entry.hash, index_mode_to_tree_item_mode(entry.mode)),
                    )
                })
                .collect()
        }
        Some(src) => {
            let commit = if src == HEAD {
                Head::current_commit_result()
                    .await
                    .map_err(map_restore_branch_store_error)?
                    .ok_or(RestoreError::ResolveSource)?
            } else {
                resolve_source_commit(src, &storage).await?
            };

            let tree_id = load_object::<Commit>(&commit)
                .map_err(|_| RestoreError::ReadObject)?
                .tree_id;
            load_object::<Tree>(&tree_id)
                .map_err(|_| RestoreError::ReadObject)?
                .get_plain_items_with_mode()?
                .into_iter()
                .map(|(path, hash, mode)| (path, RestoreTarget::new(hash, Some(mode))))
                .collect()
        }
    };

    let pathspecs = match destination_root {
        Some(root) => compile_restore_pathspecs_at(&args.pathspec, root).await?,
        None => compile_restore_pathspecs(&args.pathspec).await?,
    };
    let stat_snapshots = if worktree {
        let (_, _, snapshots) =
            restore_worktree_tracked(&pathspecs, &target_blobs, false, &[], destination_root)
                .await?;
        snapshots
    } else {
        HashMap::new()
    };
    if staged {
        restore_index_tracked(
            &pathspecs,
            &target_blobs,
            false,
            &[],
            worktree,
            destination_root,
            Some(&stat_snapshots),
        )?;
    }
    Ok(())
}

// ── Shared helpers ───────────────────────────────────────────────────

fn restore_absolute_path(path: &Path, destination_root: Option<&Path>) -> PathBuf {
    match destination_root {
        Some(root) => root.join(path),
        None => util::workdir_to_absolute(path),
    }
}

fn validate_explicit_destination_path(root: &Path, relative: &Path) -> Result<(), RestoreError> {
    let mut components = relative.components().peekable();
    if components.peek().is_none() {
        return Err(RestoreError::InvalidPathspec(
            "cannot restore an empty path into an explicit destination".to_string(),
        ));
    }
    let mut parent = root.to_path_buf();
    while let Some(component) = components.next() {
        let std::path::Component::Normal(name) = component else {
            return Err(RestoreError::InvalidPathspec(format!(
                "cannot restore unsafe path '{}' into explicit destination",
                relative.display()
            )));
        };
        if components.peek().is_none() {
            break;
        }
        parent.push(name);
        match fs::symlink_metadata(&parent) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(RestoreError::InvalidPathspec(format!(
                    "cannot restore '{}' through symbolic-link parent '{}'",
                    relative.display(),
                    parent.display()
                )));
            }
            Ok(_) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => break,
            Err(_) => return Err(RestoreError::ReadWorktree),
        }
    }
    Ok(())
}

fn clear_empty_restore_parents(path: &Path, destination_root: Option<&Path>) {
    let Some(root) = destination_root else {
        util::clear_empty_dir(path);
        return;
    };
    let Ok(relative) = path.strip_prefix(root) else {
        return;
    };
    if relative.as_os_str().is_empty()
        || relative
            .components()
            .any(|part| !matches!(part, std::path::Component::Normal(_)))
    {
        return;
    }
    let mut parent = path.parent();
    while let Some(dir) = parent {
        if dir == root || fs::remove_dir(dir).is_err() {
            break;
        }
        parent = dir.parent();
    }
}

async fn resolve_source_commit(
    src: &str,
    storage: &ClientStorage,
) -> Result<ObjectHash, RestoreError> {
    if let Some(branch) = Branch::find_branch_result(src, None)
        .await
        .map_err(map_restore_branch_store_error)?
    {
        return Ok(branch.commit);
    }

    if Branch::exists_result(src, None)
        .await
        .map_err(map_restore_branch_store_error)?
    {
        return Err(RestoreError::ResolveSource);
    }

    let objs = storage
        .search_result(src)
        .await
        .map_err(|_| RestoreError::ReadObject)?;
    if objs.len() != 1 {
        return Err(RestoreError::ResolveSource);
    }
    match storage.is_object_type_checked(&objs[0], ObjectType::Commit) {
        Ok(true) => {}
        Ok(false) => return Err(RestoreError::ReferenceNotCommit),
        Err(_) => return Err(RestoreError::ReadObject),
    }
    Ok(objs[0])
}

async fn resolve_source_commit_io(
    src: &str,
    storage: &ClientStorage,
) -> Result<ObjectHash, String> {
    if let Some(branch) = Branch::find_branch_result(src, None)
        .await
        .map_err(|e| e.to_string())?
    {
        return Ok(branch.commit);
    }

    if Branch::exists_result(src, None)
        .await
        .map_err(|e| e.to_string())?
    {
        return Err(format!("could not resolve {src}"));
    }

    let objs = storage
        .search_result(src)
        .await
        .map_err(|e| e.to_string())?;
    if objs.len() != 1 {
        return Err(format!("could not resolve {src}"));
    }
    match storage.is_object_type_checked(&objs[0], ObjectType::Commit) {
        Ok(true) => {}
        Ok(false) => return Err(format!("reference is not a commit: {src}")),
        Err(error) => return Err(error.to_string()),
    }
    Ok(objs[0])
}

fn map_restore_branch_store_error(error: BranchStoreError) -> RestoreError {
    match error {
        BranchStoreError::Query(_) | BranchStoreError::AlreadyExists(_) => RestoreError::ReadObject,
        BranchStoreError::Corrupt { .. } => RestoreError::ReadObject,
        BranchStoreError::NotFound(_) => RestoreError::ResolveSource,
        BranchStoreError::Delete { .. } => RestoreError::ReadObject,
        // `restore` only READS HEAD through this mapper, so a checkout
        // collision cannot originate here; classify it as a read fault
        // rather than silently borrowing another variant's meaning.
        BranchStoreError::CheckedOutElsewhere { .. } => RestoreError::ReadObject,
    }
}

async fn reject_restore_on_ai_managed_current_branch() -> Result<(), RestoreError> {
    match Head::current_result()
        .await
        .map_err(map_restore_branch_store_error)?
    {
        Head::Branch(name) if branch::is_ai_managed_branch(&name) => {
            Err(RestoreError::LockedCurrentBranch(name))
        }
        _ => Ok(()),
    }
}

fn preprocess_blobs(blobs: &[(PathBuf, RestoreTarget)]) -> HashMap<PathBuf, RestoreTarget> {
    blobs
        .iter()
        .map(|(path, target)| (path.clone(), *target))
        .collect()
}

fn legacy_targets(blobs: &[(PathBuf, ObjectHash)]) -> Vec<(PathBuf, RestoreTarget)> {
    blobs
        .iter()
        .map(|(path, hash)| (path.clone(), RestoreTarget::new(*hash, None)))
        .collect()
}

fn collect_restore_worktree_paths(
    pathspecs: &PathspecSet,
    target_map: &HashMap<PathBuf, RestoreTarget>,
    index: &Index,
    allowed_unmatched: &[PathBuf],
) -> Result<Vec<PathBuf>, RestoreError> {
    let tracked_paths = index.tracked_files();
    let target_paths = target_map.keys().cloned().collect::<Vec<_>>();
    let candidates = pathspec_candidates(&target_paths, &tracked_paths, allowed_unmatched);
    ensure_positive_pathspecs_match(pathspecs, candidates)?;
    let mut paths = BTreeSet::new();
    paths.extend(filter_paths(&target_paths, pathspecs));
    paths.extend(filter_paths(&tracked_paths, pathspecs));
    Ok(paths.into_iter().collect())
}

fn filter_paths(paths: &[PathBuf], pathspecs: &PathspecSet) -> Vec<PathBuf> {
    paths
        .iter()
        .filter(|path| pathspecs.matches_path(path))
        .cloned()
        .collect()
}

fn pathspec_candidates(
    target_paths: &[PathBuf],
    tracked_paths: &[PathBuf],
    allowed_unmatched: &[PathBuf],
) -> Vec<PathBuf> {
    let mut candidates =
        Vec::with_capacity(target_paths.len() + tracked_paths.len() + allowed_unmatched.len());
    candidates.extend(target_paths.iter().cloned());
    candidates.extend(tracked_paths.iter().cloned());
    candidates.extend(allowed_unmatched.iter().cloned());
    candidates
}

fn restore_match_candidates(
    target_map: &HashMap<PathBuf, RestoreTarget>,
    index: &Index,
    allowed_unmatched: &[PathBuf],
) -> Vec<PathBuf> {
    let target_paths = target_map.keys().cloned().collect::<Vec<_>>();
    let tracked_paths = index.tracked_files();
    pathspec_candidates(&target_paths, &tracked_paths, allowed_unmatched)
}

fn ensure_positive_pathspecs_match<I, P>(
    pathspecs: &PathspecSet,
    candidates: I,
) -> Result<(), RestoreError>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    // A full-tree restore is also valid when both the source tree and index
    // are empty. There is no individual pathname to satisfy the usual
    // positive-pathspec check, but switching between empty-tree commits is a
    // legitimate no-op for the worktree and index.
    if pathspecs.is_full_tree_match() {
        return Ok(());
    }
    if let Some(unmatched) = pathspecs.unmatched_positive(candidates) {
        return Err(RestoreError::PathspecNotMatched(unmatched.to_string()));
    }
    Ok(())
}

fn tree_item_mode_to_index_mode(mode: TreeItemMode) -> Option<u32> {
    match mode {
        TreeItemMode::Blob => Some(0o100644),
        TreeItemMode::BlobExecutable => Some(0o100755),
        TreeItemMode::Link => Some(0o120000),
        TreeItemMode::Commit => Some(0o160000),
        TreeItemMode::Tree => None,
    }
}

fn index_mode_to_tree_item_mode(mode: u32) -> Option<TreeItemMode> {
    match mode {
        0o100644 => Some(TreeItemMode::Blob),
        0o100755 => Some(TreeItemMode::BlobExecutable),
        0o120000 => Some(TreeItemMode::Link),
        0o160000 => Some(TreeItemMode::Commit),
        _ => None,
    }
}

fn index_entry_from_target(path: String, target: RestoreTarget, size: u32) -> IndexEntry {
    let mut entry = IndexEntry::new_from_blob(path, target.hash, size);
    entry.mode = target.index_mode();
    entry
}

fn worktree_path_exists(path: &Path) -> bool {
    fs::symlink_metadata(path).is_ok()
}

fn worktree_mode_matches(path: &Path, mode: Option<TreeItemMode>) -> Result<bool, RestoreError> {
    let Some(mode) = mode else {
        return Ok(true);
    };
    let metadata = fs::symlink_metadata(path).map_err(|_| RestoreError::ReadWorktree)?;
    let file_type = metadata.file_type();
    Ok(match mode {
        TreeItemMode::Link => file_type.is_symlink(),
        TreeItemMode::Blob => file_type.is_file() && blob_exec_matches(&metadata, false),
        TreeItemMode::BlobExecutable => file_type.is_file() && blob_exec_matches(&metadata, true),
        // A gitlink is represented by a directory in the working tree. Its
        // commit belongs to the nested repository and is intentionally not a
        // blob in the parent repository's object store.
        TreeItemMode::Commit => file_type.is_dir() && !file_type.is_symlink(),
        TreeItemMode::Tree => true,
    })
}

/// Compare a regular file's owner-execute bit against the entry mode (ADR-FM-02):
/// a mode-only difference must force a rewrite, not a silent no-op.
fn blob_exec_matches(metadata: &fs::Metadata, want_executable: bool) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        (metadata.permissions().mode() & 0o111 != 0) == want_executable
    }
    #[cfg(not(unix))]
    {
        // No POSIX permission bits: the executable distinction is not observable.
        let _ = (metadata, want_executable);
        true
    }
}

fn path_to_utf8(path: &Path) -> io::Result<&str> {
    path.to_str().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("non-UTF8 path: {}", path.display()),
        )
    })
}

fn path_to_utf8_typed(path: &Path) -> Result<&str, RestoreError> {
    path.to_str().ok_or(RestoreError::InvalidPathEncoding)
}

fn pathspec_not_matched(path: &Path) -> RestoreError {
    RestoreError::PathspecNotMatched(path.display().to_string())
}

/// Collect every path that is unmerged in `index` (i.e. has an entry at conflict
/// stage 1, 2, or 3), de-duplicated and in first-seen order. Read-only.
fn collect_unmerged_paths(index: &Index) -> Vec<PathBuf> {
    let mut seen: HashSet<String> = HashSet::new();
    let mut out = Vec::new();
    for stage in 1u8..=3 {
        for entry in index.tracked_entries(stage) {
            if seen.insert(entry.name.clone()) {
                out.push(PathBuf::from(&entry.name));
            }
        }
    }
    out
}

/// Unmerged paths (stages 1/2/3) that match the requested `filter` pathspecs.
/// Empty when nothing is unmerged, so the common conflict-free path stays cheap.
fn collect_matched_unmerged_paths(pathspecs: &PathspecSet) -> Result<Vec<PathBuf>, RestoreError> {
    let index = Index::load(path::index()).map_err(|_| RestoreError::ReadIndex)?;
    let unmerged = collect_unmerged_paths(&index);
    if unmerged.is_empty() {
        return Ok(Vec::new());
    }
    Ok(filter_paths(&unmerged, pathspecs))
}

/// Restore target for `path` at conflict `stage` (2 = ours, 3 = theirs).
/// Returns `None` when the path has no such stage.
fn stage_target(index: &Index, path: &str, stage: u8) -> Option<RestoreTarget> {
    index
        .tracked_entries(stage)
        .into_iter()
        .find(|entry| entry.name == path)
        .map(|entry| RestoreTarget::new(entry.hash, index_mode_to_tree_item_mode(entry.mode)))
}

fn preflight_conflict_stage_worktree(
    paths: &[PathBuf],
    index: &Index,
    stage: u8,
    overlay: bool,
) -> Result<(), RestoreError> {
    for path in paths {
        let path_str = path_to_utf8_typed(path)?;
        let target = stage_target(index, path_str, stage);
        if target.is_none() && overlay {
            return Err(RestoreError::MissingStageVersion {
                path: path_str.to_string(),
                stage,
            });
        }

        let absolute = util::workdir_to_absolute(path);
        let metadata = match fs::symlink_metadata(&absolute) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
            Err(_) => return Err(RestoreError::ReadWorktree),
        };
        if target.is_some_and(|target| matches!(target.mode, Some(TreeItemMode::Commit))) {
            ensure_submodule_path_is_replaceable(path, &metadata, index)?;
            continue;
        }
        if !metadata.is_dir() || metadata.file_type().is_symlink() {
            continue;
        }
        if fs::read_dir(&absolute)
            .map_err(|_| RestoreError::ReadWorktree)?
            .next()
            .is_some()
        {
            return Err(RestoreError::NonEmptyWorktreeDirectory(
                path.display().to_string(),
            ));
        }
    }
    Ok(())
}

/// Blob OID for `path` at conflict `stage` (2 = ours, 3 = theirs).
fn stage_blob(index: &Index, path: &str, stage: u8) -> Option<ObjectHash> {
    stage_target(index, path, stage).map(|target| target.hash)
}

/// Restore the `stage` side (2 = ours, 3 = theirs) of each matched unmerged path
/// to the working tree. Reads conflict stages only and writes the worktree; the
/// index is intentionally left unmerged so `libra status` still shows the
/// conflict until the user stages a resolution.
///
/// When the requested stage is ABSENT for a matched path (a modify/delete
/// conflict — that side deleted the file), the behavior follows Git's overlay
/// mode, exactly like the rest of `restore`:
/// - default (no-overlay): the deleting side "wins", so the worktree file is
///   removed (idempotently) — restoring a deletion means deleting. Exit 0.
/// - `--overlay`: overlay never removes paths, so this is an error
///   ([`RestoreError::MissingStageVersion`], matching Git's overlay message).
///
/// Returns `(restored, deleted)` working-tree paths.
async fn restore_conflict_stage(
    pathspecs: &PathspecSet,
    stage: u8,
    overlay: bool,
) -> Result<(Vec<String>, Vec<String>), RestoreError> {
    let index = Index::load(path::index()).map_err(|_| RestoreError::ReadIndex)?;
    let unmerged = collect_unmerged_paths(&index);
    let matched = filter_paths(&unmerged, pathspecs);
    if matched.is_empty() {
        if let Some(unmatched) = pathspecs.unmatched_positive(&unmerged) {
            return Err(RestoreError::PathspecNotMatched(unmatched.to_string()));
        }
        return Err(RestoreError::PathspecNotMatched(String::new()));
    }
    preflight_conflict_stage_worktree(&matched, &index, stage, overlay)?;

    let mut restored = Vec::new();
    let mut deleted = Vec::new();
    for path in &matched {
        let path_str = path_to_utf8_typed(path)?;
        match stage_target(&index, path_str, stage) {
            Some(target) => {
                restore_target_to_file_typed(target, path, None).await?;
                restored.push(path.display().to_string());
            }
            None if overlay => {
                // Overlay mode never removes paths — Git errors here.
                return Err(RestoreError::MissingStageVersion {
                    path: path_str.to_string(),
                    stage,
                });
            }
            None => {
                // Default (no-overlay): the requested side deleted this file, so
                // restore it by removing it from the worktree. Idempotent — an
                // already-absent file is fine (Git exits 0 either way).
                let path_abs = util::workdir_to_absolute(path);
                if worktree_path_exists(&path_abs) {
                    remove_worktree_path_for_restore(&path_abs)?;
                    util::clear_empty_dir(&path_abs);
                }
                deleted.push(path.display().to_string());
            }
        }
    }
    Ok((restored, deleted))
}

/// `restore --merge` / `--conflict=<style>`: for each matched unmerged path,
/// rebuild the conflict markers from the index stages (ours = stage 2, theirs =
/// stage 3, base = stage 1) and write them to the working tree, leaving the index
/// unmerged. `diff3` additionally emits the base block. Restore rebuilds the
/// markers independently from the index stages as a single whole-file `ours`
/// block / `theirs` block, with generic `ours`/`theirs` labels (the index stages
/// carry
/// no commit names) — not Git's line-level 3-way merge.
async fn restore_conflict_merge(
    pathspecs: &PathspecSet,
    diff3: bool,
) -> Result<Vec<String>, RestoreError> {
    let index = Index::load(path::index()).map_err(|_| RestoreError::ReadIndex)?;
    let unmerged = collect_unmerged_paths(&index);
    let matched = filter_paths(&unmerged, pathspecs);
    if matched.is_empty() {
        if let Some(unmatched) = pathspecs.unmatched_positive(&unmerged) {
            return Err(RestoreError::PathspecNotMatched(unmatched.to_string()));
        }
        return Err(RestoreError::PathspecNotMatched(String::new()));
    }
    preflight_conflict_merge_worktree(&matched)?;

    let eol = if cfg!(windows) { "\r\n" } else { "\n" };
    let mut restored = Vec::new();
    for path in &matched {
        let path_str = path_to_utf8_typed(path)?;
        let ours = stage_payload(&index, path_str, 2)?;
        let theirs = stage_payload(&index, path_str, 3)?;
        let base = if diff3 {
            stage_payload(&index, path_str, 1)?
        } else {
            None
        };
        let content =
            build_conflict_markers(ours.as_deref(), theirs.as_deref(), base.as_deref(), eol);
        let path_abs = util::workdir_to_absolute(path);
        if let Some(parent) = path_abs.parent() {
            fs::create_dir_all(parent).map_err(|_| RestoreError::WriteWorktree)?;
        }
        remove_existing_empty_directory(&path_abs)?;
        let executable = conflict_payload_is_executable(&index, path_str);
        write_worktree_blob(&path_abs, content.as_bytes(), executable)
            .map_err(|_| RestoreError::WriteWorktree)?;
        restored.push(path.display().to_string());
    }
    Ok(restored)
}

fn preflight_conflict_merge_worktree(paths: &[PathBuf]) -> Result<(), RestoreError> {
    for path in paths {
        let absolute = util::workdir_to_absolute(path);
        let metadata = match fs::symlink_metadata(&absolute) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
            Err(_) => return Err(RestoreError::ReadWorktree),
        };
        if !metadata.is_dir() || metadata.file_type().is_symlink() {
            continue;
        }
        if fs::read_dir(&absolute)
            .map_err(|_| RestoreError::ReadWorktree)?
            .next()
            .is_some()
        {
            return Err(RestoreError::NonEmptyWorktreeDirectory(
                path.display().to_string(),
            ));
        }
    }
    Ok(())
}

/// Load the blob content for a conflict stage, rendered for inclusion in a
/// conflict marker (UTF-8 text, or a `[binary content, N bytes]` placeholder —
/// matching `libra merge`'s `conflict_payload`). `None` when the stage is absent
/// (e.g. an add/add conflict has no base; a modify/delete conflict lacks a side).
fn stage_payload(index: &Index, path: &str, stage: u8) -> Result<Option<String>, RestoreError> {
    let Some(hash) = stage_blob(index, path, stage) else {
        return Ok(None);
    };
    let blob = load_object::<Blob>(&hash).map_err(|_| RestoreError::ReadObject)?;
    let rendered = match std::str::from_utf8(&blob.data) {
        Ok(text) => text.to_string(),
        Err(_) => format!("[binary content, {} bytes]", blob.data.len()),
    };
    Ok(Some(rendered))
}

/// Build conflict-marker content from the present stages: a single `<<<<<<<
/// ours` block, `=======`, a single `>>>>>>> theirs` block, with an optional
/// `||||||| base` block for `diff3`. A missing ours/theirs side is rendered as
/// `(deleted)`. This is an independent whole-file rebuild from the index stages
/// (one `ours` block / one `theirs` block, not Git's line-level 3-way hunks —
/// and unlike `libra merge`/`cherry-pick`, which now emit line-level hunks for
/// both-modified text conflicts), with the generic `ours`/`theirs`
/// rather than merge's `HEAD` / commit-abbrev — the index stages do not carry the
/// original commit names. (If the labels are ever unified, update both sites.)
fn build_conflict_markers(
    ours: Option<&str>,
    theirs: Option<&str>,
    base: Option<&str>,
    eol: &str,
) -> String {
    let mut out = String::new();
    match ours {
        Some(text) => {
            out.push_str(&format!("<<<<<<< ours{eol}{text}{eol}"));
        }
        None => out.push_str(&format!("<<<<<<< ours (deleted){eol}")),
    }
    if let Some(base_text) = base {
        out.push_str(&format!("||||||| base{eol}{base_text}{eol}"));
    }
    out.push_str(&format!("======={eol}"));
    match theirs {
        Some(text) => {
            out.push_str(&format!("{text}{eol}>>>>>>> theirs{eol}"));
        }
        None => out.push_str(&format!(">>>>>>> theirs (deleted){eol}")),
    }
    out
}

async fn restore_target_to_file_typed(
    target: RestoreTarget,
    path: &Path,
    destination_root: Option<&Path>,
) -> Result<Option<fs::Metadata>, RestoreError> {
    let path_abs = restore_absolute_path(path, destination_root);

    if matches!(target.mode, Some(TreeItemMode::Commit)) {
        match fs::symlink_metadata(&path_abs) {
            Ok(metadata) if metadata.is_dir() && !metadata.file_type().is_symlink() => {
                return Ok(None);
            }
            Ok(_) => fs::remove_file(&path_abs).map_err(|_| RestoreError::WriteWorktree)?,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(_) => return Err(RestoreError::ReadWorktree),
        }
        fs::create_dir_all(&path_abs).map_err(|_| RestoreError::WriteWorktree)?;
        return Ok(None);
    }

    let blob = load_object::<Blob>(&target.hash).map_err(|_| RestoreError::ReadObject)?;
    if let Some(parent) = path_abs.parent() {
        fs::create_dir_all(parent).map_err(|_| RestoreError::WriteWorktree)?;
    }

    remove_existing_empty_directory(&path_abs)?;

    if matches!(target.mode, Some(TreeItemMode::Link)) {
        write_restore_symlink(&path_abs, &blob.data)?;
        return Ok(None);
    }

    let executable = matches!(target.mode, Some(TreeItemMode::BlobExecutable));

    match lfs::parse_pointer_data(&blob.data) {
        Some((oid, size)) => {
            let lfs_obj_path = lfs::lfs_object_path(&oid);
            if lfs_obj_path.exists() {
                let mut source =
                    fs::File::open(&lfs_obj_path).map_err(|_| RestoreError::WriteWorktree)?;
                let mut writer = WorktreeFileWriter::create(&path_abs, executable)
                    .map_err(|_| RestoreError::WriteWorktree)?;
                io::copy(&mut source, &mut writer).map_err(|_| RestoreError::WriteWorktree)?;
                writer.finish().map_err(|_| RestoreError::WriteWorktree)?;
            } else {
                let writer = WorktreeFileWriter::create(&path_abs, executable)
                    .map_err(|_| RestoreError::WriteWorktree)?;
                let temp_path = writer.temp_path().to_path_buf();
                LFSClient::get()
                    .await
                    .map_err(|_| RestoreError::LfsDownload)?
                    .download_object(&oid, size, &temp_path, None)
                    .await
                    .map_err(|_| RestoreError::LfsDownload)?;
                writer.finish().map_err(|_| RestoreError::WriteWorktree)?;
            }
            // LFS payloads are external to the Git blob and must be re-hashed
            // against the pointer before their worktree stat can be trusted.
        }
        None => {
            let mut writer = WorktreeFileWriter::create(&path_abs, executable)
                .map_err(|_| RestoreError::WriteWorktree)?;
            writer
                .write_all(&blob.data)
                .map_err(|_| RestoreError::WriteWorktree)?;
            writer.flush().map_err(|_| RestoreError::WriteWorktree)?;
            let snapshot = fs::symlink_metadata(writer.temp_path()).ok();
            writer.finish().map_err(|_| RestoreError::WriteWorktree)?;
            #[cfg(test)]
            run_restore_race_hook("published", &path_abs);
            return Ok(snapshot
                .and_then(|before| stable_published_snapshot(&before, &path_abs, target.hash)));
        }
    }

    Ok(None)
}

/// Verify the published bytes, returning the snapshot taken BEFORE that read.
/// Rename may change ctime, so publication metadata alone cannot prove content
/// identity. restore_index_entry re-stats against this pre-read snapshot and
/// smudges any concurrent change rather than pairing new stat with an old hash.
fn stable_published_snapshot(
    before: &fs::Metadata,
    path: &Path,
    expected: ObjectHash,
) -> Option<fs::Metadata> {
    let after = fs::symlink_metadata(path).ok()?;
    if before.len() != after.len() || before.modified().ok()? != after.modified().ok()? {
        return None;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if before.dev() != after.dev() || before.ino() != after.ino() {
            return None;
        }
    }
    let (hash, _) =
        crate::command::stream_file_blob_hash_bounded(path, u64::from(u32::MAX)).ok()?;
    if hash != Some(expected) {
        return None;
    }
    Some(after)
}

/// Refuse now exactly what a `restore --source <commit> --staged --worktree .`
/// would refuse later — without writing anything.
///
/// `merge`'s fast-forward path restores AFTER moving the ref (a deliberate
/// ordering: the updated pointers are what the restore reads). A refusal raised
/// by the restore itself would therefore leave the ref ahead of the index and
/// working tree. Running the same preflight before the ref moves keeps the
/// fast-forward all-or-nothing — most visibly for a materialized submodule
/// directory the target tree no longer declares (ADR-MG-01), which restore
/// refuses to replace when it is non-empty.
pub(crate) async fn preflight_worktree_restore_to_commit(
    commit: &ObjectHash,
) -> Result<(), RestoreError> {
    let tree_id = load_object::<Commit>(commit)
        .map_err(|_| RestoreError::ReadObject)?
        .tree_id;
    let target_blobs: Vec<(PathBuf, RestoreTarget)> = load_object::<Tree>(&tree_id)
        .map_err(|_| RestoreError::ReadObject)?
        .get_plain_items_with_mode()?
        .into_iter()
        .map(|(path, hash, mode)| (path, RestoreTarget::new(hash, Some(mode))))
        .collect();
    let target_map = preprocess_blobs(&target_blobs);
    let index = Index::load(path::index()).map_err(|_| RestoreError::ReadIndex)?;
    let pathspecs = compile_restore_pathspecs(&[util::working_dir_string()]).await?;
    let file_paths = collect_restore_worktree_paths(&pathspecs, &target_map, &index, &[])?;
    preflight_worktree_directory_transitions(&file_paths, &target_map, &index, false, None)
}

fn preflight_worktree_directory_transitions(
    paths: &[PathBuf],
    targets: &HashMap<PathBuf, RestoreTarget>,
    index: &Index,
    overlay: bool,
    destination_root: Option<&Path>,
) -> Result<(), RestoreError> {
    for path in paths {
        let absolute = restore_absolute_path(path, destination_root);
        let metadata = match fs::symlink_metadata(&absolute) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
            Err(_) => return Err(RestoreError::ReadWorktree),
        };
        let is_directory = metadata.is_dir() && !metadata.file_type().is_symlink();
        let target_is_gitlink = targets
            .get(path)
            .is_some_and(|target| matches!(target.mode, Some(TreeItemMode::Commit)));
        let path_str = path_to_utf8_typed(path)?;
        let index_is_gitlink = index
            .get(path_str, 0)
            .is_some_and(|entry| entry.mode & 0o170000 == 0o160000);
        // Either direction across a submodule path: materializing a pointer
        // here, or replacing/dropping one the index already records. `--overlay`
        // only suppresses DELETION of paths the source omits — a source-present
        // replacement still writes, so it stays guarded.
        if target_is_gitlink || (index_is_gitlink && (targets.contains_key(path) || !overlay)) {
            ensure_submodule_path_is_replaceable(path, &metadata, index)?;
        }
        if target_is_gitlink {
            continue;
        }
        if !is_directory {
            continue;
        }
        let will_replace = targets.contains_key(path) || (!overlay && index.tracked(path_str, 0));
        if will_replace
            && fs::read_dir(&absolute)
                .map_err(|_| RestoreError::ReadWorktree)?
                .next()
                .is_some()
        {
            return Err(RestoreError::NonEmptyWorktreeDirectory(
                path.display().to_string(),
            ));
        }
    }
    Ok(())
}

/// Refuse a transition that would delete or overwrite something Libra did not
/// write at a `160000` submodule path (ADR-MG-01). Two directions, one rule:
///
/// * **materializing** a pointer replaces the path with a DIRECTORY
///   placeholder, deleting a plain file or symlink sitting exactly there; and
/// * **dropping or replacing** a pointer the index already records would remove
///   or overwrite whatever the user has at that path.
///
/// Only content Libra can put back is replaceable: an ordinary tracked BLOB
/// (recoverable from the object store), or a directory it may own — an empty
/// placeholder is removable, and a non-empty one is caught by the
/// `NonEmptyWorktreeDirectory` rule further down. Everything else is the
/// user's.
fn ensure_submodule_path_is_replaceable(
    path: &Path,
    metadata: &fs::Metadata,
    index: &Index,
) -> Result<(), RestoreError> {
    if metadata.is_dir() && !metadata.file_type().is_symlink() {
        return Ok(());
    }
    let recoverable = index
        .get(path_to_utf8_typed(path)?, 0)
        .is_some_and(|entry| entry.mode & 0o170000 != 0o160000);
    if recoverable {
        return Ok(());
    }
    Err(RestoreError::SubmodulePathNotOwned(
        path.display().to_string(),
    ))
}

fn remove_existing_empty_directory(path: &Path) -> Result<(), RestoreError> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.is_dir() && !metadata.file_type().is_symlink() => {
            fs::remove_dir(path).map_err(|_| RestoreError::WriteWorktree)
        }
        Ok(_) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(_) => Err(RestoreError::ReadWorktree),
    }
}

fn remove_worktree_path_for_restore(path: &Path) -> Result<(), RestoreError> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(_) => return Err(RestoreError::ReadWorktree),
    };
    if metadata.is_dir() && !metadata.file_type().is_symlink() {
        fs::remove_dir(path).map_err(|_| RestoreError::WriteWorktree)
    } else {
        fs::remove_file(path).map_err(|_| RestoreError::WriteWorktree)
    }
}

/// Resolve the index `size` for `target`, preferring `size_cache` over a full object
/// load.
///
/// The naive form needs `blob.data.len()`. That is the DECODED length, so it cannot be
/// obtained from an on-disk byte count without materializing the blob. Callers that
/// author many entries at once (a fresh worktree makes every path new) prime a
/// batch-resolved cache instead, because one full load per entry dominates
/// `restore --staged` on a large tree. On a miss the length is obtained the original
/// way and written back, so repeated references to one blob cost at most one load and
/// the error surface is unchanged.
fn restore_target_index_size(
    target: RestoreTarget,
    size_cache: Option<&mut HashMap<ObjectHash, u32>>,
) -> Result<u32, RestoreError> {
    if matches!(target.mode, Some(TreeItemMode::Commit)) {
        return Ok(0);
    }
    if let Some(cache) = size_cache.as_deref()
        && let Some(size) = cache.get(&target.hash)
    {
        return Ok(*size);
    }
    #[cfg(test)]
    run_restore_race_hook("fallback-size", Path::new("blob"));
    let blob = load_object::<Blob>(&target.hash).map_err(|_| RestoreError::ReadObject)?;
    let size = u32::try_from(blob.data.len()).map_err(|_| RestoreError::ReadObject)?;
    if let Some(cache) = size_cache {
        cache.insert(target.hash, size);
    }
    Ok(size)
}

fn conflict_payload_is_executable(index: &Index, path: &str) -> bool {
    [2u8, 3, 1].iter().any(|stage| {
        index
            .get(path, *stage)
            .is_some_and(|entry| entry.mode & 0o100000 == 0o100000 && entry.mode & 0o111 != 0)
    })
}

#[cfg(unix)]
fn write_restore_symlink(path: &Path, target: &[u8]) -> Result<(), RestoreError> {
    match crate::utils::worktree_blob::write_worktree_symlink(path, target) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::Unsupported => {
            Err(RestoreError::SymlinkUnsupported(path.display().to_string()))
        }
        Err(_) => Err(RestoreError::WriteWorktree),
    }
}

#[cfg(not(unix))]
fn write_restore_symlink(path: &Path, _target: &[u8]) -> Result<(), RestoreError> {
    Err(RestoreError::SymlinkUnsupported(path.display().to_string()))
}

/// Restore a blob to file.
/// If blob is an LFS pointer, download the actual file from LFS server.
/// - `path` : to workdir
pub async fn restore_to_file(hash: &ObjectHash, path: &PathBuf) -> io::Result<()> {
    let blob = Blob::load(hash)?;
    let path_abs = util::workdir_to_absolute(path);
    if let Some(parent) = path_abs.parent() {
        fs::create_dir_all(parent)?;
    }
    // Legacy entry point: it receives no explicit mode, so keep the index's
    // executable bit when the path is tracked (ADR-FM-02).
    let executable = Index::load(path::index())
        .ok()
        .and_then(|index| {
            path.to_str().and_then(|name| {
                index
                    .get(name, 0)
                    .map(|entry| entry.mode & 0o100000 == 0o100000 && entry.mode & 0o111 != 0)
            })
        })
        .unwrap_or(false);
    match lfs::parse_pointer_data(&blob.data) {
        Some((oid, size)) => {
            let lfs_obj_path = lfs::lfs_object_path(&oid);
            if lfs_obj_path.exists() {
                let mut source = fs::File::open(&lfs_obj_path)?;
                let mut writer = WorktreeFileWriter::create(&path_abs, executable)?;
                io::copy(&mut source, &mut writer)?;
                writer.finish()?;
            } else {
                let writer = WorktreeFileWriter::create(&path_abs, executable)?;
                let temp_path = writer.temp_path().to_path_buf();
                let client = LFSClient::get()
                    .await
                    .map_err(|e| io::Error::other(e.to_string()))?;
                if let Err(e) = client.download_object(&oid, size, &temp_path, None).await {
                    return Err(io::Error::other(e.to_string()));
                }
                writer.finish()?;
            }
        }
        None => {
            write_worktree_blob(&path_abs, &blob.data, executable)?;
        }
    }
    Ok(())
}

// ── Legacy worktree/index restore (kept for execute_checked) ─────────

pub async fn restore_worktree(
    filter: &[PathBuf],
    target_blobs: &[(PathBuf, ObjectHash)],
) -> io::Result<()> {
    let target_blobs = legacy_targets(target_blobs);
    let target_blobs = preprocess_blobs(&target_blobs);
    let index = Index::load(path::index()).map_err(|e| io::Error::other(e.to_string()))?;
    let raw_pathspecs = filter
        .iter()
        .map(|path| util::path_to_string(path))
        .collect::<Vec<_>>();
    let pathspecs = compile_restore_pathspecs(&raw_pathspecs)
        .await
        .map_err(|error| io::Error::other(error.to_string()))?;
    let file_paths = collect_restore_worktree_paths(&pathspecs, &target_blobs, &index, &[])
        .map_err(|error| io::Error::other(error.to_string()))?;
    for path_wd in &file_paths {
        let path_abs = util::workdir_to_absolute(path_wd);
        let path_wd_str = path_to_utf8(path_wd)?;
        let tracked = index.tracked(path_wd_str, 0);
        if !worktree_path_exists(&path_abs) {
            if let Some(target) = target_blobs.get(path_wd) {
                restore_target_to_file_typed(*target, path_wd, None)
                    .await
                    .map_err(|error| io::Error::other(error.to_string()))?;
            } else if !tracked {
                return Err(io::Error::other(format!(
                    "pathspec '{}' did not match any files",
                    path_wd.display()
                )));
            }
        } else if let Some(target) = target_blobs.get(path_wd) {
            let hash =
                calc_file_blob_hash(&path_abs).map_err(|e| io::Error::other(e.to_string()))?;
            let mode_matches = worktree_mode_matches(&path_abs, target.mode)
                .map_err(|error| io::Error::other(error.to_string()))?;
            if hash != target.hash || !mode_matches {
                restore_target_to_file_typed(*target, path_wd, None)
                    .await
                    .map_err(|error| io::Error::other(error.to_string()))?;
            }
        } else if tracked {
            fs::remove_file(&path_abs)?;
            util::clear_empty_dir(&path_abs);
        }
    }
    Ok(())
}

pub fn restore_index(filter: &[PathBuf], target_blobs: &[(PathBuf, ObjectHash)]) -> io::Result<()> {
    let target_blobs = legacy_targets(target_blobs);
    let target_blobs = preprocess_blobs(&target_blobs);

    let idx_file = path::index();
    let mut index = Index::load(&idx_file).map_err(|e| io::Error::other(e.to_string()))?;
    let deleted_files_index = get_index_deleted_files_in_filters(&index, filter, &target_blobs)?;

    let filter_vec = filter.to_vec();
    let mut file_paths = util::filter_to_fit_paths(&index.tracked_files(), &filter_vec);
    file_paths.extend(deleted_files_index);

    for path in &file_paths {
        let path_str = path_to_utf8(path)?;
        if !index.tracked(path_str, 0) {
            if let Some(target) = target_blobs.get(path) {
                let blob = Blob::load(&target.hash)?;
                crate::utils::index_ext::update_preserving_flags(
                    &mut index,
                    index_entry_from_target(path_str.to_string(), *target, blob.data.len() as u32),
                );
            } else {
                return Err(io::Error::other(format!(
                    "pathspec '{}' did not match any files",
                    path.display()
                )));
            }
        } else if let Some(target) = target_blobs.get(path) {
            if !index.verify_hash(path_str, 0, &target.hash) {
                let blob = Blob::load(&target.hash)?;
                crate::utils::index_ext::update_preserving_flags(
                    &mut index,
                    index_entry_from_target(path_str.to_string(), *target, blob.data.len() as u32),
                );
            }
        } else {
            index.remove(path_str, 0);
        }
    }
    index
        .save(&idx_file)
        .map_err(|e| io::Error::other(e.to_string()))?;
    Ok(())
}

fn get_index_deleted_files_in_filters(
    index: &Index,
    filters: &[PathBuf],
    target_blobs: &HashMap<PathBuf, RestoreTarget>,
) -> io::Result<HashSet<PathBuf>> {
    let mut deleted = HashSet::new();
    for path_wd in target_blobs.keys() {
        let path_wd_str = path_to_utf8(path_wd)?;
        let path_abs = util::workdir_to_absolute(path_wd);
        if !index.tracked(path_wd_str, 0) && util::is_sub_of_paths(path_abs, filters) {
            deleted.insert(path_wd.clone());
        }
    }
    Ok(deleted)
}

fn get_index_deleted_files_in_filters_typed(
    index: &Index,
    pathspecs: &PathspecSet,
    target_blobs: &HashMap<PathBuf, RestoreTarget>,
) -> Result<HashSet<PathBuf>, RestoreError> {
    let mut deleted = HashSet::new();
    for path_wd in target_blobs.keys() {
        let path_wd_str = path_to_utf8_typed(path_wd)?;
        if !index.tracked(path_wd_str, 0) && pathspecs.matches_path(path_wd) {
            deleted.insert(path_wd.clone());
        }
    }
    Ok(deleted)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use clap::Parser;
    use git_internal::{errors::GitError, hash::ObjectHash, internal::object::types::ObjectType};
    use serial_test::serial;
    use tempfile::tempdir;

    use super::*;
    use crate::{
        command::{add, commit, init},
        internal::config::ConfigKv,
        utils::{
            client_storage::ClientStorage,
            storage::Storage,
            test::{ChangeDirGuard, setup_with_new_libra_in},
        },
    };

    #[tokio::test]
    #[serial(cwd, env)]
    async fn restore_races_do_not_certify_modified_content_as_clean() {
        for phase in ["published", "matched-hash"] {
            let repo = tempdir().expect("isolated repository");
            let _guard = ChangeDirGuard::new(repo.path());
            setup_with_new_libra_in(repo.path()).await;
            ConfigKv::set("user.name", "Restore Race", false)
                .await
                .expect("author");
            ConfigKv::set("user.email", "race@example.invalid", false)
                .await
                .expect("email");
            let output = OutputConfig {
                quiet: true,
                ..OutputConfig::default()
            };
            fs::write("tracked.txt", b"original\n").expect("original content");
            add::execute_safe(add::AddArgs::parse_from(["add", "tracked.txt"]), &output)
                .await
                .expect("add");
            commit::execute_safe(
                commit::CommitArgs::parse_from([
                    "commit",
                    "--no-verify",
                    "--no-gpg-sign",
                    "-m",
                    "race fixture",
                ]),
                &output,
            )
            .await
            .expect("commit");
            if phase == "published" {
                fs::remove_file("tracked.txt").expect("force materialization");
            }
            RESTORE_RACE_HOOK.with(|slot| {
                *slot.borrow_mut() = Some((
                    phase,
                    Box::new(|path| {
                        let modified = fs::symlink_metadata(path)
                            .expect("pre-edit metadata")
                            .modified()
                            .expect("mtime");
                        fs::write(path, b"modified\n").expect("equal-length concurrent edit");
                        fs::File::options()
                            .write(true)
                            .open(path)
                            .expect("file")
                            .set_times(fs::FileTimes::new().set_modified(modified))
                            .expect("preserve mtime");
                    }),
                ))
            });
            execute_checked_typed(RestoreArgs {
                source: Some("HEAD".into()),
                staged: true,
                worktree: true,
                pathspec: vec!["tracked.txt".into()],
                ..Default::default()
            })
            .await
            .expect("restore");
            assert!(
                RESTORE_RACE_HOOK.with(|slot| slot.borrow().is_none()),
                "hook must run"
            );
            let index = Index::load(path::index()).expect("index");
            let entry = index.get("tracked.txt", 0).expect("entry");
            assert_eq!(
                entry.mtime.to_string(),
                "0:0",
                "{phase}: modified bytes must not earn trusted stat"
            );
            let mut status = Vec::new();
            crate::command::status::execute_to(
                crate::command::status::StatusArgs::parse_from(["status", "--porcelain"]),
                &mut status,
            )
            .await
            .expect("status");
            assert_eq!(
                String::from_utf8(status).expect("UTF-8"),
                " M tracked.txt\n",
                "{phase}"
            );
        }
    }

    #[tokio::test]
    #[serial(cwd, env)]
    async fn restore_trusted_stat_does_not_load_fallback_blob() {
        let repo = tempdir().expect("isolated repository");
        let _guard = ChangeDirGuard::new(repo.path());
        setup_with_new_libra_in(repo.path()).await;
        ConfigKv::set("user.name", "Restore Race", false)
            .await
            .expect("author");
        ConfigKv::set("user.email", "race@example.invalid", false)
            .await
            .expect("email");
        let output = OutputConfig {
            quiet: true,
            ..OutputConfig::default()
        };
        fs::write("tracked.txt", b"original\n").expect("original content");
        add::execute_safe(add::AddArgs::parse_from(["add", "tracked.txt"]), &output)
            .await
            .expect("add");
        commit::execute_safe(
            commit::CommitArgs::parse_from([
                "commit",
                "--no-verify",
                "--no-gpg-sign",
                "-m",
                "race fixture",
            ]),
            &output,
        )
        .await
        .expect("commit");
        RESTORE_RACE_HOOK.with(|slot| {
            *slot.borrow_mut() = Some((
                "fallback-size",
                Box::new(|_| panic!("trusted stat must not load fallback blob")),
            ))
        });
        execute_checked_typed(RestoreArgs {
            source: Some("HEAD".into()),
            staged: true,
            worktree: true,
            pathspec: vec!["tracked.txt".into()],
            ..Default::default()
        })
        .await
        .expect("restore");
        RESTORE_RACE_HOOK.with(|slot| {
            slot.borrow_mut().take();
        });
        let index = Index::load(path::index()).expect("index");
        assert_eq!(
            index
                .get("tracked.txt", 0)
                .expect("entry")
                .mtime
                .to_string()
                == "0:0",
            !cfg!(unix),
            "metadata trust follows the platform's ctime evidence"
        );
    }

    #[tokio::test]
    #[serial(cwd, env)]
    async fn restore_legacy_api_reuses_published_snapshot() {
        let repo = tempdir().expect("isolated repository");
        let _guard = ChangeDirGuard::new(repo.path());
        setup_with_new_libra_in(repo.path()).await;
        ConfigKv::set("user.name", "Restore Race", false)
            .await
            .expect("author");
        ConfigKv::set("user.email", "race@example.invalid", false)
            .await
            .expect("email");
        let output = OutputConfig {
            quiet: true,
            ..OutputConfig::default()
        };
        fs::write("tracked.txt", b"original\n").expect("original content");
        add::execute_safe(add::AddArgs::parse_from(["add", "tracked.txt"]), &output)
            .await
            .expect("add");
        commit::execute_safe(
            commit::CommitArgs::parse_from([
                "commit",
                "--no-verify",
                "--no-gpg-sign",
                "-m",
                "race fixture",
            ]),
            &output,
        )
        .await
        .expect("commit");
        fs::remove_file("tracked.txt").expect("force new materialization");
        RESTORE_RACE_HOOK.with(|slot| {
            *slot.borrow_mut() = Some((
                "fallback-hash",
                Box::new(|_| panic!("legacy API discarded verified publication snapshot")),
            ))
        });
        execute_checked(RestoreArgs {
            source: Some("HEAD".into()),
            staged: true,
            worktree: true,
            pathspec: vec!["tracked.txt".into()],
            ..Default::default()
        })
        .await
        .expect("legacy restore");
        RESTORE_RACE_HOOK.with(|slot| {
            slot.borrow_mut().take();
        });
        assert_eq!(
            fs::read("tracked.txt").expect("restored bytes"),
            b"original\n"
        );
        let index = Index::load(path::index()).expect("index");
        assert_eq!(
            index
                .get("tracked.txt", 0)
                .expect("entry")
                .mtime
                .to_string()
                == "0:0",
            !cfg!(unix),
            "metadata trust follows the platform's ctime evidence"
        );
    }

    struct UnreadableCommit(ObjectHash);

    #[async_trait::async_trait]
    impl Storage for UnreadableCommit {
        async fn get(&self, _hash: &ObjectHash) -> Result<(Vec<u8>, ObjectType), GitError> {
            Err(GitError::IOError(std::io::Error::other(
                "simulated type read failure",
            )))
        }

        async fn put(
            &self,
            _hash: &ObjectHash,
            _data: &[u8],
            _obj_type: ObjectType,
        ) -> Result<String, GitError> {
            unreachable!("restore probe test never stores objects")
        }

        async fn exist(&self, _hash: &ObjectHash) -> bool {
            false
        }

        async fn search(&self, prefix: &str) -> Vec<ObjectHash> {
            if self.0.to_string().starts_with(prefix) {
                vec![self.0]
            } else {
                Vec::new()
            }
        }
    }

    #[tokio::test]
    #[serial(cwd, env)]
    async fn restore_storage_read_error_is_not_non_commit() {
        let tmp = tempdir().expect("tmp");
        let _guard = ChangeDirGuard::new(tmp.path());
        setup_with_new_libra_in(tmp.path()).await;
        let hash = ObjectHash::new_for_kind(git_internal::hash::get_hash_kind(), &[9; 20]);
        let storage = ClientStorage::from_test_storage(
            Arc::new(UnreadableCommit(hash)),
            std::path::PathBuf::new(),
        );
        let error = resolve_source_commit(&hash.to_string(), &storage)
            .await
            .expect_err("a type read failure must not resolve");
        assert!(
            matches!(error, RestoreError::ReadObject),
            "read failure must not be reported as a non-commit: {error}"
        );
    }

    #[test]
    fn explicit_destination_pruning_stops_at_root() {
        let parent = tempdir().expect("create destination parent");
        let root = parent.path().join("workspace");
        fs::create_dir_all(root.join("nested")).expect("create empty nested directory");

        clear_empty_restore_parents(&root.join("nested/deleted.txt"), Some(&root));

        assert!(!root.join("nested").exists());
        assert!(root.is_dir(), "the destination root must survive pruning");
        assert!(
            parent.path().is_dir(),
            "the destination parent must survive pruning"
        );

        clear_empty_restore_parents(&root, Some(&root));
        assert!(
            root.is_dir(),
            "an empty relative path must not remove the root"
        );
        assert!(parent.path().is_dir());
    }

    #[test]
    #[cfg(unix)]
    fn explicit_destination_rejects_escape_and_symlink_parent() {
        let destination = tempdir().expect("create destination root");
        let outside = tempdir().expect("create outside directory");
        assert!(matches!(
            validate_explicit_destination_path(destination.path(), Path::new("../outside.txt")),
            Err(RestoreError::InvalidPathspec(_))
        ));
        assert!(matches!(
            validate_explicit_destination_path(destination.path(), outside.path()),
            Err(RestoreError::InvalidPathspec(_))
        ));
        std::os::unix::fs::symlink(outside.path(), destination.path().join("link"))
            .expect("create parent symlink");
        assert!(matches!(
            validate_explicit_destination_path(destination.path(), Path::new("link/file.txt")),
            Err(RestoreError::InvalidPathspec(_))
        ));
        assert!(!outside.path().join("file.txt").exists());
    }

    #[tokio::test]
    async fn explicit_destination_rejects_relative_pathspec_and_index_update() {
        let destination = tempdir().expect("create explicit destination");
        for args in [
            RestoreArgs {
                pathspec: vec!["tracked.txt".to_string()],
                ..Default::default()
            },
            RestoreArgs {
                pathspec: vec![":/".to_string()],
                staged: true,
                ..Default::default()
            },
        ] {
            assert!(matches!(
                execute_checked_typed_into(args, destination.path()).await,
                Err(RestoreError::InvalidPathspec(_))
            ));
        }
    }

    #[tokio::test]
    #[serial(cwd, env)]
    async fn explicit_destination_restores_older_commit_without_touching_source() {
        let repo = tempdir().expect("create source repository");
        let _guard = ChangeDirGuard::new(repo.path());
        init::init(init::InitArgs::parse_from(["init", "-q", "."]))
            .await
            .expect("initialize source repository");
        ConfigKv::set("user.name", "Restore Test User", false)
            .await
            .expect("configure author name");
        ConfigKv::set("user.email", "restore-test@example.com", false)
            .await
            .expect("configure author email");
        let output = OutputConfig {
            quiet: true,
            ..OutputConfig::default()
        };

        fs::write(repo.path().join("tracked.txt"), b"older\n").expect("write older file");
        add::execute_safe(add::AddArgs::parse_from(["add", "tracked.txt"]), &output)
            .await
            .expect("stage older file");
        commit::execute_safe(
            commit::CommitArgs::parse_from(["commit", "--no-verify", "-m", "older"]),
            &output,
        )
        .await
        .expect("commit older revision");
        let older = Head::current_commit_result()
            .await
            .expect("read older revision")
            .expect("older revision exists");

        fs::write(repo.path().join("tracked.txt"), b"newer\n").expect("write newer file");
        fs::write(repo.path().join("new-only.txt"), b"new only\n").expect("write source-only file");
        add::execute_safe(
            add::AddArgs::parse_from(["add", "tracked.txt", "new-only.txt"]),
            &output,
        )
        .await
        .expect("stage newer files");
        commit::execute_safe(
            commit::CommitArgs::parse_from(["commit", "--no-verify", "-m", "newer"]),
            &output,
        )
        .await
        .expect("commit newer revision");
        let source_index = repo.path().join(".libra/index");
        let index_before = fs::read(&source_index).expect("snapshot source index");
        fs::write(repo.path().join("tracked.txt"), b"source dirty\n").expect("dirty source file");

        let destination = tempdir().expect("create explicit destination");
        fs::write(
            destination.path().join("tracked.txt"),
            b"destination dirty\n",
        )
        .expect("dirty destination file");
        fs::write(
            destination.path().join("new-only.txt"),
            b"destination newer\n",
        )
        .expect("seed path absent from older commit");
        execute_checked_typed_into(
            RestoreArgs {
                source: Some(older.to_string()),
                pathspec: vec![":/".to_string()],
                worktree: true,
                ..Default::default()
            },
            destination.path(),
        )
        .await
        .expect("restore older revision into explicit destination");

        assert_eq!(
            fs::read(destination.path().join("tracked.txt")).expect("destination tracked file"),
            b"older\n"
        );
        assert!(!destination.path().join("new-only.txt").exists());
        assert_eq!(
            fs::read(repo.path().join("tracked.txt")).expect("source tracked file"),
            b"source dirty\n"
        );
        assert_eq!(
            fs::read(repo.path().join("new-only.txt")).expect("source-only file"),
            b"new only\n"
        );
        assert_eq!(fs::read(&source_index).expect("source index"), index_before);
    }

    /// Pin the `Display` format for every variant of [`RestoreError`].
    /// These strings are used as the `CliError` message via the
    /// `From<RestoreError> for CliError` mapping and surface in both
    /// human and `--json` envelopes for `restore` and the
    /// checkout-from-commit phase of `clone` / `switch`.
    ///
    /// Every variant carries either a static message or an explicit
    /// `{0}` field interpolation; none wrap an upstream source error,
    /// so all variants are pinned.
    #[test]
    fn restore_error_display_pins_each_variant() {
        assert_eq!(
            RestoreError::ResolveSource.to_string(),
            "failed to resolve checkout source",
        );
        assert_eq!(
            RestoreError::ReferenceNotCommit.to_string(),
            "reference is not a commit",
        );
        assert_eq!(
            RestoreError::PathspecNotMatched("src/missing.rs".to_string()).to_string(),
            "pathspec 'src/missing.rs' did not match any files",
        );
        assert_eq!(RestoreError::ReadIndex.to_string(), "failed to read index");
        assert_eq!(
            RestoreError::ReadObject.to_string(),
            "failed to read object",
        );
        assert_eq!(
            RestoreError::ReadWorktree.to_string(),
            "failed to read worktree",
        );
        assert_eq!(
            RestoreError::InvalidPathEncoding.to_string(),
            "invalid path encoding",
        );
        assert_eq!(
            RestoreError::WriteWorktree.to_string(),
            "failed to write worktree file",
        );
        assert_eq!(
            RestoreError::LfsDownload.to_string(),
            "failed to download LFS content",
        );
        assert_eq!(
            RestoreError::SubmodulePathNotOwned("vendor/sub".to_string()).to_string(),
            "refusing to replace worktree path 'vendor/sub': Libra does not manage submodule content there",
        );
        assert_eq!(
            RestoreError::LockedSource("intent".to_string()).to_string(),
            "refusing to restore from locked branch 'intent'",
        );
        assert_eq!(
            RestoreError::LockedCurrentBranch("traces".to_string()).to_string(),
            "refusing to restore worktree while on locked branch 'traces'",
        );
        assert_eq!(
            RestoreError::PathUnmerged("src/conflict.rs".to_string()).to_string(),
            "path 'src/conflict.rs' is unmerged",
        );
        assert_eq!(
            RestoreError::MissingStageVersion {
                path: "src/conflict.rs".to_string(),
                stage: 2,
            }
            .to_string(),
            "path 'src/conflict.rs' does not have our version",
        );
        assert_eq!(
            RestoreError::MissingStageVersion {
                path: "src/conflict.rs".to_string(),
                stage: 3,
            }
            .to_string(),
            "path 'src/conflict.rs' does not have their version",
        );
        assert_eq!(
            RestoreError::UnsupportedConflictStyle("zdiff3".to_string()).to_string(),
            "unsupported conflict style 'zdiff3' (expected 'merge' or 'diff3')",
        );
        assert_eq!(
            RestoreError::SymlinkUnsupported("src/link".to_string()).to_string(),
            "symlink checkout is not supported on this platform: src/link",
        );
        assert_eq!(
            RestoreError::InvalidPathspec("unsupported pathspec magic".to_string()).to_string(),
            "unsupported pathspec magic",
        );
    }

    /// Pin the `stable_code()` mapping for every variant of
    /// [`RestoreError`]. JSON consumers branch on the
    /// [`StableErrorCode`] in the error envelope — three of the read
    /// variants share `IoReadFailed`, three of the target variants
    /// share `CliInvalidTarget`, and `LfsDownload` is the only
    /// network-coded variant in the family. A future refactor that
    /// reroutes any of them (for example flipping `LfsDownload` from
    /// `NetworkUnavailable` to `IoReadFailed`) silently changes
    /// client retry classification unless every variant has its own
    /// guard. Enumerate every variant so a new variant trips both
    /// this exhaustive list and the `stable_code()` impl's match.
    #[test]
    fn restore_error_stable_code_pins_each_variant() {
        assert_eq!(
            RestoreError::ResolveSource.stable_code(),
            StableErrorCode::CliInvalidTarget,
        );
        assert_eq!(
            RestoreError::ReferenceNotCommit.stable_code(),
            StableErrorCode::CliInvalidTarget,
        );
        assert_eq!(
            RestoreError::PathspecNotMatched("ignored".to_string()).stable_code(),
            StableErrorCode::CliInvalidTarget,
        );
        assert_eq!(
            RestoreError::ReadIndex.stable_code(),
            StableErrorCode::IoReadFailed,
        );
        assert_eq!(
            RestoreError::ReadObject.stable_code(),
            StableErrorCode::IoReadFailed,
        );
        assert_eq!(
            RestoreError::ReadWorktree.stable_code(),
            StableErrorCode::IoReadFailed,
        );
        assert_eq!(
            RestoreError::InvalidPathEncoding.stable_code(),
            StableErrorCode::CliInvalidArguments,
        );
        assert_eq!(
            RestoreError::WriteWorktree.stable_code(),
            StableErrorCode::IoWriteFailed,
        );
        assert_eq!(
            RestoreError::LfsDownload.stable_code(),
            StableErrorCode::NetworkUnavailable,
        );
        assert_eq!(
            RestoreError::LockedSource("ignored".to_string()).stable_code(),
            StableErrorCode::CliInvalidTarget,
        );
        assert_eq!(
            RestoreError::LockedCurrentBranch("ignored".to_string()).stable_code(),
            StableErrorCode::ConflictOperationBlocked,
        );
        assert_eq!(
            RestoreError::PathUnmerged("ignored".to_string()).stable_code(),
            StableErrorCode::ConflictUnresolved,
        );
        assert_eq!(
            RestoreError::SubmodulePathNotOwned("ignored".to_string()).stable_code(),
            StableErrorCode::ConflictOperationBlocked,
        );
        assert_eq!(
            RestoreError::MissingStageVersion {
                path: "ignored".to_string(),
                stage: 2,
            }
            .stable_code(),
            StableErrorCode::ConflictUnresolved,
        );
        assert_eq!(
            RestoreError::UnsupportedConflictStyle("zdiff3".to_string()).stable_code(),
            StableErrorCode::CliInvalidArguments,
        );
        assert_eq!(
            RestoreError::SymlinkUnsupported("ignored".to_string()).stable_code(),
            StableErrorCode::Unsupported,
        );
        assert_eq!(
            RestoreError::InvalidPathspec("ignored".to_string()).stable_code(),
            StableErrorCode::CliInvalidTarget,
        );
    }

    /// Pin the externally-visible `--merge` / `--conflict` marker contract,
    /// including the deleted-side (modify/delete) and `diff3` base-block cases,
    /// independently of how a particular merge stages a conflict.
    #[test]
    fn build_conflict_markers_covers_present_deleted_and_diff3() {
        assert_eq!(
            build_conflict_markers(Some("A"), Some("B"), None, "\n"),
            "<<<<<<< ours\nA\n=======\nB\n>>>>>>> theirs\n",
        );
        assert_eq!(
            build_conflict_markers(Some("A"), Some("B"), Some("BASE"), "\n"),
            "<<<<<<< ours\nA\n||||||| base\nBASE\n=======\nB\n>>>>>>> theirs\n",
        );
        // Stage 2 (ours) missing — theirs modified, ours deleted.
        assert_eq!(
            build_conflict_markers(None, Some("B"), None, "\n"),
            "<<<<<<< ours (deleted)\n=======\nB\n>>>>>>> theirs\n",
        );
        // Stage 3 (theirs) missing — ours modified, theirs deleted.
        assert_eq!(
            build_conflict_markers(Some("A"), None, None, "\n"),
            "<<<<<<< ours\nA\n=======\n>>>>>>> theirs (deleted)\n",
        );
    }

    #[tokio::test]
    #[serial(cwd, env)]
    async fn restore_target_index_size_handles_cache_hit_miss_and_error() {
        let tmp = tempdir().expect("tmp");
        let _guard = ChangeDirGuard::new(tmp.path());
        setup_with_new_libra_in(tmp.path()).await;

        let blob = Blob::from_content("decoded fallback payload\n");
        crate::command::save_object(&blob, &blob.id).expect("save fallback blob");

        let commit_target = RestoreTarget {
            hash: blob.id,
            mode: Some(TreeItemMode::Commit),
        };
        assert_eq!(
            restore_target_index_size(commit_target, None).expect("gitlink size"),
            0
        );

        let mut cache = HashMap::new();
        let blob_target = RestoreTarget {
            hash: blob.id,
            mode: Some(TreeItemMode::Blob),
        };
        let resolved = restore_target_index_size(blob_target, Some(&mut cache))
            .expect("cache miss must fall back to loading the blob");
        assert_eq!(resolved, blob.data.len() as u32);
        assert_eq!(
            cache.get(&blob.id).copied(),
            Some(blob.data.len() as u32),
            "fallback load must populate size_cache for subsequent paths"
        );

        let missing_target = RestoreTarget {
            hash: ObjectHash::new_for_kind(git_internal::hash::get_hash_kind(), &[0xab; 20]),
            mode: Some(TreeItemMode::Blob),
        };
        let error = restore_target_index_size(missing_target, Some(&mut cache))
            .expect_err("missing blob on cache miss must return ReadObject");
        assert!(matches!(error, RestoreError::ReadObject));
    }
}
