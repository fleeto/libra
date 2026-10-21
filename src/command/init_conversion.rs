//! Interrupted `init --from-git-repository` detection, rollback, and retry.
//!
//! Issue #487: the conversion creates the repository layout, database, and
//! initial refs before importing, so a process that dies mid-import leaves a
//! valid-looking `.libra` with no history that can neither be retried (init
//! refuses an existing repository) nor distinguished from a genuinely empty one.
//!
//! The design (plan `issues/487.md`, ADR-IG-02..04) is a marker file written
//! **before** the first target write and removed as the **last** write on
//! success:
//!
//! * a normal error or a SIGINT/SIGTERM rolls the run back through the recorded
//!   `created` list, so the target is immediately retryable;
//! * a hard kill leaves the marker behind, so a retry can prove the previous
//!   owner is gone and the source still matches, move the residue aside
//!   (never delete it), and start over;
//! * every other command refuses to operate on a half-converted repository
//!   instead of presenting it as an empty one.
//!
//! Only the conversion path writes the marker; every command reads it.

use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use serde::{Deserialize, Serialize};

use crate::utils::error::StableErrorCode;

/// Marker file name at the storage root.
pub const CONVERSION_MARKER: &str = "CONVERSION_IN_PROGRESS";

/// Current marker schema version.
pub const MARKER_VERSION: u32 = 1;

/// The process that owns the in-progress conversion, in the same shape the
/// operation log records (`pid-<n>` plus the process start time when the
/// platform exposes it). The start time defeats pid reuse: a recycled pid with
/// a different start time is not the owner.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConversionOwner {
    pub pid: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_time: Option<u64>,
}

impl ConversionOwner {
    /// Capture this process as the conversion owner.
    pub fn current() -> Self {
        Self {
            pid: std::process::id(),
            // Older macOS clients cannot read process start times. Keep this
            // field empty for them; the new reader uses the marker's v2 field.
            start_time: if cfg!(target_os = "macos") {
                None
            } else {
                crate::command::control_lock::current_pid_starttime()
            },
        }
    }

    /// Whether the recorded owner process is still running.
    ///
    /// A cross-machine or pre-start-time marker (no `start_time`) can never be
    /// proven dead, so it is reported as live. A live PID whose start time
    /// cannot be read is also treated as unknown and blocks recovery.
    pub fn is_live(&self) -> bool {
        let Some(start_time) = self.start_time else {
            return true;
        };
        if !crate::command::control_lock::pid_is_live(self.pid) {
            return false;
        }
        match crate::command::control_lock::read_pid_starttime(self.pid) {
            Some(observed) => observed == start_time,
            None => true,
        }
    }
}

/// One path this conversion created, recorded **before** it is created so a
/// crash between the record and the write still rolls back cleanly.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreatedEntry {
    /// Path, absolute when the target directory itself was created by this run,
    /// otherwise relative to the storage root's parent.
    pub path: String,
    /// Whole-directory entry (the target worktree): rollback removes it
    /// recursively. File entries are removed individually.
    pub directory: bool,
}

/// The on-disk marker.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConversionMarker {
    pub version: u32,
    /// Canonical path of the source Git directory this conversion reads.
    pub source: String,
    /// Unix seconds when the conversion started.
    pub started_at: i64,
    pub owner: ConversionOwner,
    /// macOS-only start time in a field ignored by older readers. Those
    /// readers see `owner.start_time = None` and refuse a concurrent retry.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner_start_time_v2: Option<u64>,
    /// The macOS boot session that produced `owner_start_time_v2`. A different
    /// host or reboot cannot prove that the recorded process has exited.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner_boot_session: Option<String>,
    #[serde(default)]
    pub created: Vec<CreatedEntry>,
}

impl ConversionMarker {
    fn effective_owner(&self) -> ConversionOwner {
        #[cfg(target_os = "macos")]
        {
            let mut owner = self.owner.clone();
            // Older macOS writers never used this field. A set legacy stamp
            // comes from another platform and cannot be compared here.
            owner.start_time = None;
            if self
                .owner_boot_session
                .as_deref()
                .zip(current_boot_session().as_deref())
                .is_some_and(|(recorded, current)| recorded == current)
            {
                owner.start_time = self.owner_start_time_v2;
            }
            owner
        }
        #[cfg(not(target_os = "macos"))]
        {
            // Only macOS writers use the v2 field. A Linux or Windows reader
            // cannot compare that process stamp with its local clock/PID table.
            self.owner.clone()
        }
    }

    /// Whether this host can verify the recorded owner's process identity.
    pub fn has_verifiable_owner(&self) -> bool {
        self.effective_owner().start_time.is_some()
    }

    /// Check the owner using the compatibility start-time field when present.
    pub fn owner_is_live(&self) -> bool {
        self.effective_owner().is_live()
    }
}

#[cfg(target_os = "macos")]
fn current_boot_session() -> Option<String> {
    let mut buffer = [0_u8; 64];
    let mut size = buffer.len();
    // SAFETY: `sysctlbyname` receives a NUL-terminated constant key, a valid
    // writable buffer, and its mutable size. The returned length is checked.
    let result = unsafe {
        libc::sysctlbyname(
            c"kern.bootsessionuuid".as_ptr(),
            buffer.as_mut_ptr().cast(),
            &mut size,
            std::ptr::null_mut(),
            0,
        )
    };
    if result != 0 || size == 0 || size > buffer.len() {
        return None;
    }
    let raw = buffer[..size].split(|byte| *byte == 0).next()?;
    let value = std::str::from_utf8(raw).ok()?;
    uuid::Uuid::parse_str(value).ok().map(|id| id.to_string())
}

/// Why a marker could not be used.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MarkerError {
    /// The file exists but is not a readable marker document.
    Unparsable(String),
    /// A marker version this binary does not understand.
    UnsupportedVersion(u32),
    /// The storage root is a symlink; the retry refuses to rename through it.
    Symlink,
}

impl MarkerError {
    /// Human-readable diagnostic used by the CLI and the retry path.
    pub fn message(&self, storage_root: &Path) -> String {
        match self {
            Self::Unparsable(detail) => format!(
                "cannot read the interrupted Git conversion marker at '{}': {detail}",
                storage_root.join(CONVERSION_MARKER).display()
            ),
            Self::UnsupportedVersion(version) => format!(
                "the interrupted Git conversion marker at '{}' has unsupported version {version}",
                storage_root.join(CONVERSION_MARKER).display()
            ),
            Self::Symlink => format!(
                "refusing to recover the interrupted Git conversion: storage root '{}' is a symlink",
                storage_root.display()
            ),
        }
    }
}

/// Read the marker at `storage_root`, if any.
pub fn read_marker(storage_root: &Path) -> Option<Result<ConversionMarker, MarkerError>> {
    let path = storage_root.join(CONVERSION_MARKER);
    let metadata = match fs::symlink_metadata(&path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return None,
        Err(error) => return Some(Err(MarkerError::Unparsable(error.to_string()))),
    };
    if metadata.file_type().is_symlink() {
        return Some(Err(MarkerError::Symlink));
    }
    let text = match fs::read_to_string(&path) {
        Ok(text) => text,
        Err(error) => return Some(Err(MarkerError::Unparsable(error.to_string()))),
    };
    let marker: ConversionMarker = match serde_json::from_str(&text) {
        Ok(marker) => marker,
        Err(error) => return Some(Err(MarkerError::Unparsable(error.to_string()))),
    };
    if marker.version != MARKER_VERSION {
        return Some(Err(MarkerError::UnsupportedVersion(marker.version)));
    }
    Some(Ok(marker))
}

/// Write `marker` atomically (temp file + fsync + rename + directory fsync).
pub fn write_marker(storage_root: &Path, marker: &ConversionMarker) -> std::io::Result<()> {
    let target = storage_root.join(CONVERSION_MARKER);
    let temp = storage_root.join(format!("{CONVERSION_MARKER}.tmp-{}", std::process::id()));
    let body = serde_json::to_vec_pretty(marker)
        .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))?;
    {
        let mut file = fs::File::create(&temp)?;
        file.write_all(&body)?;
        file.sync_all()?;
    }
    fs::rename(&temp, &target)?;
    // Durability of the rename itself: a crash must not leave the marker
    // invisible after the conversion has already created target state.
    if let Ok(dir) = fs::File::open(storage_root) {
        let _ = dir.sync_all();
    }
    Ok(())
}

/// Removes the marker as the final write of a successful conversion.
pub fn clear_marker(storage_root: &Path) -> std::io::Result<()> {
    let path = storage_root.join(CONVERSION_MARKER);
    match fs::remove_file(&path) {
        Ok(()) => {
            if let Ok(dir) = fs::File::open(storage_root) {
                let _ = dir.sync_all();
            }
            Ok(())
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
}

/// What a retry must do when it finds an existing marker.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RetryDecision {
    /// No marker: an ordinary fresh conversion.
    Proceed,
    /// Move `storage_root` aside to this path, then convert from scratch.
    MoveAside(PathBuf),
    /// Refuse with a stable code and an actionable message.
    Refuse {
        code: StableErrorCode,
        message: String,
        hint: String,
    },
}

/// Decide how to handle a marker found by a retry of
/// `init --from-git-repository <source>` (ADR-IG-03 item 2).
pub fn decide_retry(storage_root: &Path, canonical_source: &Path) -> RetryDecision {
    let Some(marker) = read_marker(storage_root) else {
        return RetryDecision::Proceed;
    };
    let marker = match marker {
        Ok(marker) => marker,
        Err(error) => {
            return RetryDecision::Refuse {
                code: StableErrorCode::RepoStateInvalid,
                message: error.message(storage_root),
                hint: manual_recovery_hint(storage_root),
            };
        }
    };
    if fs::symlink_metadata(storage_root).is_ok_and(|meta| meta.file_type().is_symlink()) {
        return RetryDecision::Refuse {
            code: StableErrorCode::RepoStateInvalid,
            message: MarkerError::Symlink.message(storage_root),
            hint: manual_recovery_hint(storage_root),
        };
    }
    let owner = marker.effective_owner();
    if owner.start_time.is_none() {
        return RetryDecision::Refuse {
            code: StableErrorCode::RepoStateInvalid,
            message: format!(
                "the conversion marker in '{}' lacks a locally verifiable owner identity",
                storage_root.display()
            ),
            hint: format!(
                "confirm no conversion is running on any host, then {}",
                manual_recovery_hint(storage_root)
            ),
        };
    }
    if owner.is_live() {
        return RetryDecision::Refuse {
            code: StableErrorCode::ConflictOperationBlocked,
            message: format!(
                "a Git conversion of '{}' is already running (pid {}); wait for it to finish",
                marker.source, marker.owner.pid
            ),
            hint: "let the running conversion finish, or terminate it and retry".to_string(),
        };
    }
    let recorded_source = PathBuf::from(&marker.source);
    if recorded_source != canonical_source {
        return RetryDecision::Refuse {
            code: StableErrorCode::RepoStateInvalid,
            message: format!(
                "the interrupted conversion in '{}' imported from '{}', not '{}'",
                storage_root.display(),
                marker.source,
                canonical_source.display()
            ),
            hint: manual_recovery_hint(storage_root),
        };
    }
    RetryDecision::MoveAside(aside_path(storage_root))
}

/// The sibling directory an interrupted conversion is renamed to. Never
/// deleted automatically: the residue may contain user-owned data.
pub fn aside_path(storage_root: &Path) -> PathBuf {
    let parent = storage_root.parent().unwrap_or(storage_root);
    let name = storage_root
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| ".libra".to_string());
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs())
        .unwrap_or(0);
    parent.join(format!("{name}.incomplete-{stamp}"))
}

/// Move an interrupted conversion's storage root aside, returning the new path.
pub fn move_aside(storage_root: &Path) -> std::io::Result<PathBuf> {
    let target = aside_path(storage_root);
    fs::rename(storage_root, &target)?;
    Ok(target)
}

/// The recovery command shown to users when automatic recovery is impossible.
pub fn manual_recovery_hint(storage_root: &Path) -> String {
    format!(
        "move the incomplete storage root aside and retry: 'mv {} {}' then \
         'libra init --from-git-repository <source>'",
        storage_root.display(),
        aside_path(storage_root).display()
    )
}

/// Deletes the paths a conversion created, in reverse order (ADR-IG-03 item 1).
/// Returns the paths that could not be removed.
pub fn rollback_created(entries: &[CreatedEntry], parent: &Path) -> Vec<(PathBuf, String)> {
    let mut failures = Vec::new();
    for entry in entries.iter().rev() {
        let path = PathBuf::from(&entry.path);
        let path = if path.is_absolute() {
            path
        } else {
            parent.join(path)
        };
        let result = if entry.directory {
            match fs::remove_dir_all(&path) {
                Ok(()) => Ok(()),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
                Err(error) => Err(error),
            }
        } else {
            match fs::remove_file(&path) {
                Ok(()) => Ok(()),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
                Err(error) => Err(error),
            }
        };
        if let Err(error) = result {
            failures.push((path, error.to_string()));
        }
    }
    failures
}

/// Signal-aware rollback state shared by the conversion path.
///
/// Records `created` entries and rewrites the marker before each reported
/// creation, so a hard kill at any point still describes what must be undone.
pub struct ConversionGuard {
    storage_root: PathBuf,
    marker: ConversionMarker,
    /// `Some(())` once the marker has been written for this run.
    active: bool,
}

impl ConversionGuard {
    /// Start a conversion guard, writing the marker immediately.
    pub fn begin(storage_root: &Path, canonical_source: &Path) -> std::io::Result<Self> {
        #[cfg(target_os = "macos")]
        let macos_identity =
            crate::command::control_lock::current_pid_starttime().zip(current_boot_session());
        let mut guard = Self {
            storage_root: storage_root.to_path_buf(),
            marker: ConversionMarker {
                version: MARKER_VERSION,
                source: canonical_source.display().to_string(),
                started_at: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map(|elapsed| elapsed.as_secs() as i64)
                    .unwrap_or(0),
                owner: ConversionOwner::current(),
                owner_start_time_v2: {
                    #[cfg(target_os = "macos")]
                    {
                        macos_identity.as_ref().map(|(start_time, _)| *start_time)
                    }
                    #[cfg(not(target_os = "macos"))]
                    {
                        None
                    }
                },
                owner_boot_session: {
                    #[cfg(target_os = "macos")]
                    {
                        macos_identity.map(|(_, boot_session)| boot_session)
                    }
                    #[cfg(not(target_os = "macos"))]
                    {
                        None
                    }
                },
                created: Vec::new(),
            },
            active: false,
        };
        write_marker(storage_root, &guard.marker)?;
        guard.active = true;
        Ok(guard)
    }

    /// Record a path this conversion is about to create, rewriting the marker
    /// first so a crash between the record and the write still rolls it back.
    pub fn record_created(&mut self, path: &Path, directory: bool) -> std::io::Result<()> {
        let parent = self
            .storage_root
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| self.storage_root.clone());
        let recorded = if directory || path.starts_with(&parent) {
            path.display().to_string()
        } else {
            path.strip_prefix(&parent)
                .map(|relative| relative.display().to_string())
                .unwrap_or_else(|_| path.display().to_string())
        };
        self.marker.created.push(CreatedEntry {
            path: recorded,
            directory,
        });
        write_marker(&self.storage_root, &self.marker)
    }

    /// The source recorded in the marker.
    pub fn source(&self) -> &str {
        &self.marker.source
    }

    /// Finish successfully: the marker is the last thing to disappear.
    pub fn finish(mut self) -> std::io::Result<()> {
        clear_marker(&self.storage_root)?;
        self.active = false;
        Ok(())
    }

    /// Undo everything this run created, keeping the marker when a deletion
    /// fails so the next retry can still recognise the residue.
    pub fn rollback(&mut self) -> Vec<(PathBuf, String)> {
        let parent = self
            .storage_root
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| self.storage_root.clone());
        let failures = rollback_created(&self.marker.created, &parent);
        if failures.is_empty() {
            let _ = clear_marker(&self.storage_root);
            self.active = false;
        }
        failures
    }

    /// Whether the marker is still on disk for this run.
    pub fn is_active(&self) -> bool {
        self.active
    }
}

/// Exit status for an interrupted conversion (SIGINT).
pub const SIGNAL_EXIT_INT: i32 = 130;
/// Exit status for an interrupted conversion (SIGTERM).
pub const SIGNAL_EXIT_TERM: i32 = 143;

/// Install the SIGINT/SIGTERM rollback listener for this conversion.
///
/// A signal is a cancellation, not a crash: it must take the same rollback path
/// as an ordinary error so the target is immediately retryable (ADR-IG-03 item
/// 1). The listener owns only this process's residue — it re-reads the marker
/// and refuses to touch a marker whose owner pid is not ours.
///
/// A hard kill (SIGKILL, abort, OOM) cannot run this; the marker left on disk
/// is what [`decide_retry`] handles for that case.
#[cfg(unix)]
pub fn install_signal_rollback(storage_root: PathBuf) {
    use tokio::signal::unix::{SignalKind, signal};

    let Ok(mut interrupt) = signal(SignalKind::interrupt()) else {
        return;
    };
    let Ok(mut terminate) = signal(SignalKind::terminate()) else {
        return;
    };
    let own_pid = std::process::id();
    tokio::spawn(async move {
        let code = tokio::select! {
            _ = interrupt.recv() => SIGNAL_EXIT_INT,
            _ = terminate.recv() => SIGNAL_EXIT_TERM,
        };
        roll_back_own_conversion(&storage_root, own_pid);
        std::process::exit(code);
    });
}

/// Roll back the marker owned by `pid`, if it is ours. Best effort: an
/// unreadable marker is left in place rather than guessed at.
pub fn roll_back_own_conversion(storage_root: &Path, pid: u32) {
    let Some(Ok(marker)) = read_marker(storage_root) else {
        return;
    };
    if marker.owner.pid != pid {
        return;
    }
    let parent = storage_root
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| storage_root.to_path_buf());
    if rollback_created(&marker.created, &parent).is_empty() {
        let _ = clear_marker(storage_root);
    }
}

/// Non-Unix builds keep the marker/retry path and skip the signal hook.
#[cfg(not(unix))]
pub fn install_signal_rollback(_storage_root: PathBuf) {}

/// Test-only conversion failpoints (ADR-IG-05). Every one requires
/// `LIBRA_TEST=1`; production runs never consult these variables.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConversionStage {
    AfterLayout,
    AfterDatabase,
    AfterFetch,
    AfterSetup,
    AfterVault,
}

impl ConversionStage {
    /// The env-var spelling of this stage.
    pub fn key(self) -> &'static str {
        match self {
            Self::AfterLayout => "after-layout",
            Self::AfterDatabase => "after-database",
            Self::AfterFetch => "after-fetch",
            Self::AfterSetup => "after-setup",
            Self::AfterVault => "after-vault",
        }
    }
}

/// Failpoint outcome at a stage.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConversionFailpoint {
    Proceed,
    /// Return an ordinary conversion error.
    Fail,
    /// Abort the process, simulating a hard kill.
    Abort,
    /// Block until the resume file appears (the test kills us meanwhile).
    Pause,
}

/// Consult the conversion failpoints. Inert unless `LIBRA_TEST=1`.
pub fn conversion_failpoint(stage: ConversionStage) -> ConversionFailpoint {
    if std::env::var("LIBRA_TEST").as_deref() != Ok("1") {
        return ConversionFailpoint::Proceed;
    }
    let key = stage.key();
    if std::env::var("LIBRA_TEST_INIT_CONVERT_FAIL_AT").as_deref() == Ok(key) {
        return ConversionFailpoint::Fail;
    }
    if std::env::var("LIBRA_TEST_INIT_CONVERT_ABORT_AT").as_deref() == Ok(key) {
        return ConversionFailpoint::Abort;
    }
    if std::env::var("LIBRA_TEST_INIT_CONVERT_PAUSE_AT").as_deref() == Ok(key) {
        return ConversionFailpoint::Pause;
    }
    ConversionFailpoint::Proceed
}

/// Block until `LIBRA_TEST_INIT_CONVERT_RESUME_FILE` exists (pause failpoint).
pub fn wait_for_resume_file() {
    let Ok(path) = std::env::var("LIBRA_TEST_INIT_CONVERT_RESUME_FILE") else {
        return;
    };
    let path = PathBuf::from(path);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
    while std::time::Instant::now() < deadline {
        if path.exists() {
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn marker(root: &Path, source: &str, owner: ConversionOwner) -> ConversionMarker {
        let mut marker = ConversionMarker {
            version: MARKER_VERSION,
            source: source.to_string(),
            started_at: 0,
            owner,
            owner_start_time_v2: None,
            owner_boot_session: None,
            created: Vec::new(),
        };
        write_marker(root, &marker).expect("write marker");
        marker.created.push(CreatedEntry {
            path: "sentinel".to_string(),
            directory: false,
        });
        marker
    }

    fn same_host_marker(root: &Path, source: &str, owner: ConversionOwner) -> ConversionMarker {
        let written = marker(root, source, owner);
        #[cfg(target_os = "macos")]
        {
            let mut written = written;
            written.owner_start_time_v2 = written.owner.start_time.take();
            written.owner_boot_session = current_boot_session();
            write_marker(root, &written).expect("write same-boot marker");
            written
        }
        #[cfg(not(target_os = "macos"))]
        {
            written
        }
    }

    #[test]
    fn marker_round_trips_and_clears() {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path().join(".libra");
        fs::create_dir_all(&root).expect("mkdir");
        let dead = ConversionOwner {
            pid: 999_999,
            start_time: Some(1),
        };
        let written = marker(&root, "/src/.git", dead.clone());
        let read = read_marker(&root).expect("marker present").expect("parses");
        assert_eq!(read.source, "/src/.git");
        assert_eq!(read.owner, dead);
        assert!(read.created.is_empty());
        assert_eq!(written.created.len(), 1, "test fixture mutation is local");

        clear_marker(&root).expect("clear");
        assert!(read_marker(&root).is_none(), "cleared marker is gone");
        // Clearing twice is idempotent.
        clear_marker(&root).expect("clear again");
    }

    #[test]
    fn unparsable_marker_is_reported() {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path().join(".libra");
        fs::create_dir_all(&root).expect("mkdir");
        fs::write(root.join(CONVERSION_MARKER), b"not json").expect("write");
        let error = read_marker(&root).expect("present").expect_err("must fail");
        assert!(matches!(error, MarkerError::Unparsable(_)));
    }

    #[test]
    fn unsupported_version_is_reported() {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path().join(".libra");
        fs::create_dir_all(&root).expect("mkdir");
        let mut marker = marker(
            &root,
            "/src/.git",
            ConversionOwner {
                pid: 999_999,
                start_time: Some(1),
            },
        );
        marker.version = 99;
        write_marker(&root, &marker).expect("write");
        let error = read_marker(&root).expect("present").expect_err("must fail");
        assert_eq!(error, MarkerError::UnsupportedVersion(99));
    }

    #[test]
    fn live_owner_blocks_the_retry() {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path().join(".libra");
        fs::create_dir_all(&root).expect("mkdir");
        same_host_marker(
            &root,
            "/src/.git",
            ConversionOwner {
                pid: std::process::id(),
                start_time: crate::command::control_lock::current_pid_starttime(),
            },
        );
        match decide_retry(&root, Path::new("/src/.git")) {
            RetryDecision::Refuse { code, .. } => {
                assert_eq!(code, StableErrorCode::ConflictOperationBlocked);
            }
            other => panic!("a live owner must refuse: {other:?}"),
        }
    }

    #[test]
    fn unstamped_owner_stays_conservative_even_if_pid_is_absent() {
        let owner = ConversionOwner {
            pid: u32::MAX,
            start_time: None,
        };
        assert!(owner.is_live());
    }

    #[test]
    fn unstamped_owner_refusal_explains_safe_manual_recovery() {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path().join(".libra");
        fs::create_dir_all(&root).expect("mkdir");
        marker(
            &root,
            "/src/.git",
            ConversionOwner {
                pid: u32::MAX,
                start_time: None,
            },
        );
        match decide_retry(&root, Path::new("/src/.git")) {
            RetryDecision::Refuse {
                code: StableErrorCode::RepoStateInvalid,
                message,
                hint,
            } => {
                assert!(message.contains("lacks a locally verifiable owner identity"));
                assert!(hint.contains("confirm no conversion is running on any host"));
                assert!(hint.contains(".libra.incomplete-"));
            }
            other => panic!("unstamped owner must require manual recovery: {other:?}"),
        }
    }

    #[cfg(not(target_os = "macos"))]
    #[test]
    fn foreign_macos_v2_owner_requires_manual_recovery() {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path().join(".libra");
        fs::create_dir_all(&root).expect("mkdir");
        let mut written = marker(
            &root,
            "/src/.git",
            ConversionOwner {
                pid: u32::MAX,
                start_time: None,
            },
        );
        written.owner_start_time_v2 = Some(1);
        written.owner_boot_session = Some(uuid::Uuid::new_v4().to_string());
        write_marker(&root, &written).expect("write foreign Mac marker");
        assert!(!written.has_verifiable_owner());
        assert!(matches!(
            decide_retry(&root, Path::new("/src/.git")),
            RetryDecision::Refuse {
                code: StableErrorCode::RepoStateInvalid,
                ..
            }
        ));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn macos_marker_preserves_legacy_reader_safety_and_new_recovery() {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path().join(".libra");
        fs::create_dir_all(&root).expect("mkdir");
        let guard = ConversionGuard::begin(&root, Path::new("/src/.git")).expect("begin");
        let mut written = read_marker(&root).expect("present").expect("parse");
        assert_eq!(written.owner.start_time, None);
        assert!(written.owner_start_time_v2.is_some());
        assert_eq!(written.owner_boot_session, current_boot_session());
        assert!(
            written.owner.is_live(),
            "old macOS readers must refuse takeover"
        );
        assert!(written.owner_is_live(), "new reader sees the live owner");
        assert!(matches!(
            decide_retry(&root, Path::new("/src/.git")),
            RetryDecision::Refuse {
                code: StableErrorCode::ConflictOperationBlocked,
                ..
            }
        ));

        written.owner.pid = u32::MAX;
        assert!(!written.owner_is_live(), "new reader detects hard kill");
        write_marker(&root, &written).expect("write dead-owner fixture");
        assert!(matches!(
            decide_retry(&root, Path::new("/src/.git")),
            RetryDecision::MoveAside(_)
        ));
        written.owner_boot_session = Some(uuid::Uuid::new_v4().to_string());
        write_marker(&root, &written).expect("write foreign-boot fixture");
        assert!(!written.has_verifiable_owner());
        assert!(matches!(
            decide_retry(&root, Path::new("/src/.git")),
            RetryDecision::Refuse {
                code: StableErrorCode::RepoStateInvalid,
                ..
            }
        ));
        guard.finish().expect("finish");
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn foreign_legacy_start_time_requires_manual_recovery() {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path().join(".libra");
        fs::create_dir_all(&root).expect("mkdir");
        let written = marker(
            &root,
            "/src/.git",
            ConversionOwner {
                pid: u32::MAX,
                start_time: Some(1),
            },
        );
        assert!(!written.has_verifiable_owner());
        assert!(matches!(
            decide_retry(&root, Path::new("/src/.git")),
            RetryDecision::Refuse {
                code: StableErrorCode::RepoStateInvalid,
                ..
            }
        ));
    }

    #[test]
    fn dead_owner_with_matching_source_moves_aside() {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path().join(".libra");
        fs::create_dir_all(&root).expect("mkdir");
        same_host_marker(
            &root,
            "/src/.git",
            ConversionOwner {
                pid: 999_999,
                start_time: Some(1),
            },
        );
        match decide_retry(&root, Path::new("/src/.git")) {
            RetryDecision::MoveAside(path) => {
                assert!(path.to_string_lossy().contains(".libra.incomplete-"));
            }
            other => panic!("a dead owner with a matching source must move aside: {other:?}"),
        }
    }

    #[test]
    fn dead_owner_with_a_different_source_is_refused() {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path().join(".libra");
        fs::create_dir_all(&root).expect("mkdir");
        same_host_marker(
            &root,
            "/other/.git",
            ConversionOwner {
                pid: 999_999,
                start_time: Some(1),
            },
        );
        match decide_retry(&root, Path::new("/src/.git")) {
            RetryDecision::Refuse { code, .. } => {
                assert_eq!(code, StableErrorCode::RepoStateInvalid);
            }
            other => panic!("a source mismatch must refuse: {other:?}"),
        }
    }

    #[test]
    fn no_marker_proceeds() {
        let dir = tempfile::tempdir().expect("tempdir");
        assert_eq!(
            decide_retry(dir.path(), Path::new("/src/.git")),
            RetryDecision::Proceed
        );
    }

    #[test]
    fn rollback_deletes_in_reverse_order_and_keeps_the_marker_on_failure() {
        let dir = tempfile::tempdir().expect("tempdir");
        let parent = dir.path();
        let root = parent.join(".libra");
        fs::create_dir_all(&root).expect("mkdir");
        let mut guard = ConversionGuard::begin(&root, Path::new("/src/.git")).expect("begin");
        let ignore = parent.join(".libraignore");
        fs::write(&ignore, "").expect("write ignore file");
        guard
            .record_created(&ignore, false)
            .expect("record ignore file");
        guard
            .record_created(&parent.join("worktree"), true)
            .expect("record worktree");
        fs::create_dir_all(parent.join("worktree")).expect("mkdir worktree");

        let failures = guard.rollback();
        assert!(failures.is_empty(), "rollback must succeed: {failures:?}");
        assert!(!ignore.exists(), "the created ignore file is removed");
        assert!(
            !parent.join("worktree").exists(),
            "the created worktree is removed"
        );
        assert!(!guard.is_active(), "a clean rollback clears the marker");
    }

    #[test]
    fn failpoints_are_inert_without_the_test_guard() {
        // The harness runs with LIBRA_TEST set globally for child processes;
        // assert the guard's own contract instead of mutating global state.
        let stage = ConversionStage::AfterLayout;
        if std::env::var("LIBRA_TEST").as_deref() != Ok("1") {
            assert_eq!(conversion_failpoint(stage), ConversionFailpoint::Proceed);
        }
    }
}
