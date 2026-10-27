//! Shared index stat-refresh and executable-bit helpers (plan issues/479,
//! UI-02/UI-03).
//!
//! Both `add --refresh` and `update-index --refresh`/`--chmod` route through
//! this module so the stat-refresh decision and the mode flip have exactly one
//! implementation (GC-02).
//!
//! The refresh loop replicates git's `refresh_index` / `refresh_cache_ent`
//! semantics (`read-cache.c`, git 2.56): a candidate whose stat matches is
//! skipped (racy-clean entries are trusted, as in git); one whose stat changed
//! is re-hashed with the **correct blob hash** — unlike `git_internal`'s
//! `Index::refresh`, which stores the raw content hash — and either has its
//! stat fields refreshed (content still matches the staged blob) or is
//! reported as needing update (content/mode/type changed). Entries are never
//! given a hash that does not identify a real blob.

use std::{
    fs,
    path::{Path, PathBuf},
};

use git_internal::internal::index::{Index, IndexEntry};

use crate::utils::pathspec::PathspecSet;

/// Stat/refresh behavior switches shared by `add --refresh` (which renders its
/// own summary) and `update-index --refresh` (which renders git's
/// `needs update`/`needs merge` lines).
#[derive(Debug, Clone, Copy, Default)]
pub struct RefreshOptions {
    /// `-q`: suppress the `<path>: needs update` report and its exit code.
    /// Like git, `needs merge` is still reported.
    pub quiet: bool,
    /// `--ignore-missing`: skip entries whose worktree file is gone.
    pub ignore_missing: bool,
    /// `--unmerged`: skip unmerged entries instead of reporting them.
    pub allow_unmerged: bool,
    /// `core.fileMode` (`true` on Unix by default): when enabled, an
    /// executable-bit-only change counts as "needs update" (git never
    /// silently syncs the mode during a refresh).
    pub file_mode: bool,
}

impl RefreshOptions {
    /// The switches `add --refresh` uses: never print per-file lines (add
    /// renders its own summary) and skip unmerged entries (its candidates come
    /// from the status diff, which has no unmerged stage-0 handling).
    pub fn for_add_refresh(file_mode: bool) -> Self {
        Self {
            quiet: true,
            allow_unmerged: true,
            file_mode,
            ..Self::default()
        }
    }
}

/// What one pass over the candidates observed.
#[derive(Debug, Default)]
pub struct RefreshReport {
    /// Entries whose stat fields were refreshed (content matches the blob).
    pub refreshed: Vec<PathBuf>,
    /// Entries whose content, mode or type changed: `<path>: needs update`.
    pub needs_update: Vec<PathBuf>,
    /// Worktree files that are missing: reported as `needs update` too, but
    /// tracked separately so `--ignore-missing` handling stays explicit.
    pub missing: Vec<PathBuf>,
    /// Unmerged paths: `<path>: needs merge` (reported even under `-q`).
    pub needs_merge: Vec<PathBuf>,
    /// Every reportable path in candidate (index) order, so `update-index
    /// --refresh` can render `needs update`/`needs merge` lines interleaved
    /// exactly like git's `refresh_index`. `add --refresh` ignores it.
    pub events: Vec<(PathBuf, RefreshEventKind)>,
    /// Whether the pass should exit 1 (git's refresh contract).
    pub has_errors: bool,
}

/// The git-rendered kind of a refresh report line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefreshEventKind {
    /// `<path>: needs update` (content, mode, type change or missing file).
    NeedsUpdate,
    /// `<path>: needs merge` (unmerged entry).
    NeedsMerge,
    /// A missing worktree file (renders as `needs update`, tracked separately
    /// for `--ignore-missing` documentation).
    Missing,
}

/// Restrict `files` to entries that fall under at least one pathspec.
/// Extracted from `add` so `add --refresh` and `update-index --refresh` share
/// one candidate filter.
pub fn filter_refresh_candidates(files: &[PathBuf], pathspecs: &PathspecSet) -> Vec<PathBuf> {
    files
        .iter()
        .filter(|file| pathspecs.matches_path(file.as_path()))
        .cloned()
        .collect()
}

/// Refresh the stat fields of every candidate whose working-tree content still
/// matches its staged blob, and classify the rest. The caller renders
/// `needs_update` / `missing` / `needs_merge` and decides the exit code.
///
/// Boundary conditions:
/// - Unmerged paths (any stage-1/2/3 entry) are reported once and never
///   refreshed; `--unmerged` skips them silently.
/// - Skip-worktree entries are skipped silently, as git's
///   `refresh_cache_ent` does.
/// - A missing worktree file is reported unless `ignore_missing` is set.
/// - Entry-type changes (file ↔ symlink) always count as needs-update; an
///   executable-bit change counts only under `file_mode` (git's trust-mode).
/// - The refreshed entry keeps its staged mode, hash and extended flags; only
///   the stat fields follow the worktree.
/// - Every stage-0 candidate loses a stale assume-unchanged (`CE_VALID`) bit,
///   whatever the outcome (issues/637). Libra has no assume-unchanged
///   semantics, so the bit can only be a leftover from git-internal <= 0.10.3;
///   Git would keep it here and keep skipping the worktree stat.
pub fn refresh_candidates(
    index: &mut Index,
    candidates: &[PathBuf],
    workdir: &Path,
    options: &RefreshOptions,
) -> RefreshReport {
    let mut report = RefreshReport::default();
    for candidate in candidates {
        let Some(name) = candidate.to_str() else {
            continue;
        };
        // Unmerged: stages 1-3 exist. Report once, never refresh (git's
        // refresh_index skips the whole path's entries after the stage-1
        // report).
        if [1u8, 2, 3]
            .iter()
            .any(|stage| index.get(name, *stage).is_some())
        {
            if options.allow_unmerged {
                continue;
            }
            report.needs_merge.push(candidate.clone());
            report
                .events
                .push((candidate.clone(), RefreshEventKind::NeedsMerge));
            report.has_errors = true;
            continue;
        }
        // Snapshot the stage-0 fields; `IndexEntry` is not `Clone`, and the
        // refresh decision must not hold the index borrow while hashing.
        let Some((mode, hash, skip_worktree)) = index
            .get(name, 0)
            .map(|entry| (entry.mode, entry.hash, entry.flags.skip_worktree))
        else {
            continue;
        };
        crate::utils::index_ext::clear_assume_valid(index, name, 0);
        if skip_worktree {
            // CE_SKIP_WORKTREE: the user promised the worktree copy is good.
            continue;
        }

        let absolute = workdir.join(candidate);
        let metadata = match fs::symlink_metadata(&absolute) {
            Ok(metadata) => metadata,
            Err(_) => {
                if options.ignore_missing {
                    continue;
                }
                if !options.quiet {
                    report.missing.push(candidate.clone());
                    report
                        .events
                        .push((candidate.clone(), RefreshEventKind::Missing));
                    report.has_errors = true;
                }
                continue;
            }
        };

        // Entry type: regular (0o100000) vs symlink (0o120000) — a change is
        // always a needs-update, independent of `core.fileMode` (FM-03).
        let disk_type = if metadata.file_type().is_symlink() {
            0o120000
        } else {
            0o100000
        };
        if disk_type != mode & 0o170000 {
            if !options.quiet {
                report.needs_update.push(candidate.clone());
                report
                    .events
                    .push((candidate.clone(), RefreshEventKind::NeedsUpdate));
                report.has_errors = true;
            }
            continue;
        }
        // Executable bit (regular files only; trust mode mirrors core.fileMode).
        if options.file_mode && disk_type == 0o100000 {
            let disk_exec = is_executable(&metadata);
            if disk_exec != (mode & 0o111 != 0) {
                if !options.quiet {
                    report.needs_update.push(candidate.clone());
                    report
                        .events
                        .push((candidate.clone(), RefreshEventKind::NeedsUpdate));
                    report.has_errors = true;
                }
                continue;
            }
        }

        // Stat comparison: ctime + mtime + size. A stat-equal entry is trusted
        // (git's racy-clean heuristic — content changed within the same
        // timestamp granularity is invisible to refresh, exactly as in git).
        if stat_matches(&metadata, index, name) {
            continue;
        }
        // Stat changed: re-hash with the REAL blob hash (LFS/symlink aware).
        // Only a matching hash refreshes the stat; a differing one reports
        // needs-update without touching the entry.
        match crate::command::calc_file_blob_hash(&absolute) {
            Ok(actual) if actual == hash => {
                // Refresh: keep mode/hash/flags, take the fresh stat fields.
                let mut refreshed = IndexEntry::new(&metadata, hash, name.to_string());
                refreshed.mode = mode;
                crate::utils::index_ext::update_preserving_flags(index, refreshed);
                report.refreshed.push(candidate.clone());
            }
            _ => {
                if !options.quiet {
                    report.needs_update.push(candidate.clone());
                    report
                        .events
                        .push((candidate.clone(), RefreshEventKind::NeedsUpdate));
                    report.has_errors = true;
                }
            }
        }
    }
    report
}

/// Whether the worktree stat matches the stage-0 entry's stat fields.
///
/// Compares ctime + mtime + size against the recorded fields. Entries built by
/// `new_from_blob` (cacheinfo / index-info) carry zeroed times, so they are
/// always considered stale until a refresh records real stat data — the
/// intended plumbing behavior.
fn stat_matches(metadata: &fs::Metadata, index: &Index, name: &str) -> bool {
    let Some(entry) = index.get(name, 0) else {
        return false;
    };
    if metadata.len() != u64::from(entry.size) {
        return false;
    }
    let (disk_ctime, disk_mtime) = stat_times(metadata);
    let entry_ctime = git_internal::internal::index::Time::from_system_time(disk_ctime);
    let entry_mtime = git_internal::internal::index::Time::from_system_time(disk_mtime);
    entry.ctime == entry_ctime && entry.mtime == entry_mtime
}

/// The (ctime, mtime) pair of a metadata handle, following git-internal's
/// `index_ctime`/`index_mtime` mapping on every platform.
fn stat_times(metadata: &fs::Metadata) -> (std::time::SystemTime, std::time::SystemTime) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        let ctime = std::time::SystemTime::UNIX_EPOCH
            + std::time::Duration::new(
                metadata.ctime().max(0) as u64,
                metadata.ctime_nsec().max(0) as u32,
            );
        let mtime = std::time::SystemTime::UNIX_EPOCH
            + std::time::Duration::new(
                metadata.mtime().max(0) as u64,
                metadata.mtime_nsec().max(0) as u32,
            );
        (ctime, mtime)
    }
    #[cfg(not(unix))]
    {
        let created = metadata
            .created()
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
        let modified = metadata.modified().unwrap_or(created);
        (created, modified)
    }
}

/// Whether a worktree file carries the executable bit (Unix only; elsewhere
/// regular files are never executable).
fn is_executable(metadata: &fs::Metadata) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        metadata.mode() & 0o111 != 0
    }
    #[cfg(not(unix))]
    {
        let _ = metadata;
        false
    }
}

/// Force the executable bit on a tracked stage-0 entry — the shared core of
/// `add --chmod` and `update-index --chmod`.
///
/// Returns `Err(message)` when the path has no stage-0 entry or the entry is
/// not a regular file (symlinks `120000` and gitlinks `160000` carry no
/// executable bit); callers choose how to surface that (`add` records a
/// rejection, `update-index` dies with `cannot chmod <flip> '<path>'`).
/// Returns `Ok(true)` when the mode flipped, `Ok(false)` when it already
/// matched (a no-op). The entry's blob, size and extended flags are kept; the
/// stat fields are zeroed so the next status content-compares this entry
/// (2026-08-06 R0-8 review).
pub fn chmod_entry(index: &mut Index, path: &str, target_mode: u32) -> Result<bool, String> {
    let Some(existing) = index.get(path, 0) else {
        return Err(format!("cannot chmod '{path}': not in the index"));
    };
    let (mode, hash, size) = (existing.mode, existing.hash, existing.size);
    if mode & 0o170000 != 0o100000 {
        return Err(format!(
            "cannot chmod '{path}': entry is not a regular file (mode {mode:o})"
        ));
    }
    if mode == target_mode {
        return Ok(false);
    }
    let mut updated = IndexEntry::new_from_blob(path.to_string(), hash, size);
    updated.mode = target_mode;
    crate::utils::index_ext::update_preserving_flags(index, updated);
    Ok(true)
}
