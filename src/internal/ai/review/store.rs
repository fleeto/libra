//! Review run directory store (`.libra/sessions/agent-runs/<run_id>/`).
//!
//! Persists the E8-libra run wire (`docs/development/tracing/agent.md`
//! E8-libra + 落地执行补充规格 §5): per run one directory containing
//! `state.json`, `manifest.json`, `findings.md` and
//! `reviewers/<slug>.stdout.redacted.log` / `.stderr.redacted.log`.
//!
//! Modeled on [`crate::internal::ai::agent_run::event_store::AgentRunEventStore`]:
//! the store is stateless beyond its `sessions_root`, every write is a
//! whole-file JSON overwrite (`fs::write` of `serde_json::to_vec_pretty`)
//! and reads tolerate missing files. A run directory is written by exactly
//! one producer (the runner driving that run) plus the small cross-process
//! cancel/clean surface below; concurrent multi-writer coordination is out
//! of scope for AG-22.

use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, Instant, SystemTime},
};

use serde::{Deserialize, Serialize};

use crate::internal::ai::{
    checkpoint_input::{
        CheckpointInputIoError, RunMetadataBytes, ScopedIoBudget, cleanup_checkpoint_input,
        open_scoped_run_root, read_run_metadata,
    },
    observed_agents::RedactionReport,
    workspace_isolation::{
        TaskWorkspaceDescriptor, cleanup_task_workspaces_until, has_task_workspace_intents,
        validate_task_workspace_descriptor, verify_task_workspace,
    },
};

/// `object_index` o_type tag for objectized review/investigate findings +
/// manual attachments (A0-06). A distinguished tag (parallel to
/// `agent_transcript`) so `libra agent doctor` and retention can enumerate
/// findings artifacts; cloud sync does not filter by o_type either way.
pub const AGENT_FINDINGS_OTYPE: &str = "agent_findings";

/// `manifest.json` schema version (E8-libra manifest contract).
pub const REVIEW_MANIFEST_SCHEMA_VERSION: u32 = 2;

/// The `kind` value review runs stamp into `state.json` / `manifest.json`
/// (`review|investigate` per E8-libra; AG-23 owns `investigate`).
pub const REVIEW_RUN_KIND: &str = "review";

/// Directory under the sessions root holding one subdirectory per run.
const AGENT_RUNS_DIR: &str = "agent-runs";
/// Cross-process cancel-request marker file inside a run directory.
const CANCEL_REQUESTED_FILE: &str = "cancel.requested";

// ---------------------------------------------------------------------------
// Terminal states
// ---------------------------------------------------------------------------

/// The five terminal states every review run must end in exactly one of
/// (plan.md Task A7 验收; `agent.md` §6 `agent.review.run`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewTerminalState {
    /// Every reviewer completed with exit status 0.
    Success,
    /// Infrastructure failure (workspace/store), or no reviewer succeeded
    /// and none timed out.
    Error,
    /// The run was cancelled (`review cancel` / SIGINT / SIGTERM — one
    /// shared cleanup path).
    Cancelled,
    /// No reviewer succeeded and at least one hit its deadline.
    Timeout,
    /// Some — but not all — reviewers succeeded.
    Partial,
}

impl ReviewTerminalState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Success => "success",
            Self::Error => "error",
            Self::Cancelled => "cancelled",
            Self::Timeout => "timeout",
            Self::Partial => "partial",
        }
    }
}

impl std::fmt::Display for ReviewTerminalState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Per-reviewer outcome recorded in `state.json` and aggregated into the
/// run's terminal state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewerOutcome {
    /// Exit status 0.
    Ok,
    /// Spawn failure or non-zero exit status.
    Failed,
    /// Killed at the per-reviewer deadline.
    TimedOut,
    /// Killed by the run-level cancel path.
    Cancelled,
}

/// Deterministic terminal-state aggregation (unit-tested truth table):
///
/// - run-level cancel always wins → [`ReviewTerminalState::Cancelled`];
/// - no reviewer ever produced an outcome (infrastructure failed before
///   any reviewer ran) → [`ReviewTerminalState::Error`];
/// - all `Ok` → [`ReviewTerminalState::Success`];
/// - some (but not all) `Ok` → [`ReviewTerminalState::Partial`];
/// - none `Ok`, at least one `TimedOut` → [`ReviewTerminalState::Timeout`];
/// - none `Ok`, none `TimedOut` → [`ReviewTerminalState::Error`].
pub fn aggregate_terminal_state(
    cancelled: bool,
    outcomes: &[ReviewerOutcome],
) -> ReviewTerminalState {
    if cancelled {
        return ReviewTerminalState::Cancelled;
    }
    if outcomes.is_empty() {
        return ReviewTerminalState::Error;
    }
    let ok = outcomes
        .iter()
        .filter(|o| matches!(o, ReviewerOutcome::Ok))
        .count();
    if ok == outcomes.len() {
        ReviewTerminalState::Success
    } else if ok > 0 {
        ReviewTerminalState::Partial
    } else if outcomes
        .iter()
        .any(|o| matches!(o, ReviewerOutcome::TimedOut))
    {
        ReviewTerminalState::Timeout
    } else {
        ReviewTerminalState::Error
    }
}

// ---------------------------------------------------------------------------
// Manifest (E8 exact key set)
// ---------------------------------------------------------------------------

/// Deserializable summary of a redaction pass. The full
/// [`RedactionReport`] is `Serialize`-only (its match list carries rule
/// offsets the manifest does not need), so the manifest stores this
/// round-trippable aggregate instead.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RedactionReportSummary {
    /// Total redaction rule matches across all persisted reviewer text.
    pub matches: usize,
    pub bytes_scanned: usize,
    pub bytes_redacted: usize,
}

impl RedactionReportSummary {
    pub fn absorb(&mut self, report: &RedactionReport) {
        self.matches += report.matches.len();
        self.bytes_scanned += report.bytes_scanned;
        self.bytes_redacted += report.bytes_redacted;
    }

    pub fn merge(&mut self, other: &RedactionReportSummary) {
        self.matches += other.matches;
        self.bytes_scanned += other.bytes_scanned;
        self.bytes_redacted += other.bytes_redacted;
    }
}

/// `manifest.json` — the E8-libra manifest with a versioned workspace
/// descriptor in schema 2. Schema 1 remains readable without assigning a
/// managed path or rewriting old runs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReviewManifest {
    pub schema_version: u32,
    pub run_id: String,
    /// `"review"` (`REVIEW_RUN_KIND`); `"investigate"` is AG-23's.
    pub kind: String,
    /// Reviewer slugs in request order.
    pub agents: Vec<String>,
    /// The commit the reviewed worktree started from.
    pub starting_sha: String,
    /// Human-readable review scope (e.g. `--since <rev>` resolution).
    pub target_scope: String,
    /// `null` while the run is in flight; exactly one of the five
    /// terminal states once it ends.
    pub terminal_state: Option<ReviewTerminalState>,
    /// RFC 3339 UTC, microsecond precision (fixed width, so
    /// lexicographic order equals chronological order — the keyset
    /// pagination contract relies on this).
    pub created_at: String,
    pub updated_at: String,
    /// OID of `findings.md` in the object store. `null` at create time and
    /// for an empty findings set; populated at `finalize_run` (A0-06) with a
    /// content-addressed, `object_index`-visible, doctor-repairable blob.
    pub findings_oid: Option<String>,
    pub redaction_report: RedactionReportSummary,
    /// Manual attachments (A0-06): each entry is
    /// `{oid, path, provenance:"manual", size, attached_at}` for an external
    /// transcript/findings/context file attached via `libra review attach`.
    /// Empty until the first attach; the attached bytes are redacted and
    /// object_index-tagged like findings.
    pub manual_attach: Vec<serde_json::Value>,
    /// Managed workspace ownership is proved by the durable attempt intent;
    /// this descriptor lets readers identify the run's current attempt.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workspace_descriptor: Option<TaskWorkspaceDescriptor>,
}

// ---------------------------------------------------------------------------
// State (engine-internal, richer than the manifest)
// ---------------------------------------------------------------------------

/// Per-reviewer row in `state.json`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReviewerStateEntry {
    pub slug: String,
    /// `null` until the reviewer finishes.
    pub outcome: Option<ReviewerOutcome>,
    pub exit_code: Option<i32>,
    #[serde(default)]
    pub stdout_truncated: bool,
    #[serde(default)]
    pub stderr_truncated: bool,
    /// Redacted one-line spawn/launch error, when the reviewer never ran.
    #[serde(default)]
    pub launch_error: Option<String>,
    /// The spawned reviewer's process id, written through at spawn time
    /// so an orphaned-run cancel (runner crashed or was killed) can
    /// still verify and release the process. `null` before spawn and in
    /// terminal snapshots (finalize rewrites the rows without live
    /// pids).
    #[serde(default)]
    pub pid: Option<u32>,
    /// The reviewer's process-group id (== `pid`: reviewers are spawned
    /// as their own group leaders). The orphan cancel path group-kills
    /// this, taking reviewer descendants down too — but only after the
    /// `proc_start_ticks` provenance check verifies the pid still names
    /// OUR process incarnation.
    #[serde(default)]
    pub pgid: Option<u32>,
    /// Process provenance: the spawned reviewer's kernel start time
    /// (`/proc/<pid>/stat` field 22, clock ticks; Linux only). Pids are
    /// reused, so the orphaned-run cancel refuses to kill a recorded
    /// pgid whose current start time does not match this value (or when
    /// either side is unavailable).
    #[serde(default)]
    pub proc_start_ticks: Option<u64>,
}

/// `state.json` — the run's E8 state record (terminal state, timestamps,
/// agents, scope). Review has no round-robin fields; those are
/// investigate's (AG-23).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReviewRunState {
    pub schema_version: u32,
    pub run_id: String,
    pub kind: String,
    pub agents: Vec<ReviewerStateEntry>,
    pub starting_sha: String,
    pub target_scope: String,
    /// `null` while running; exactly one of the five states at the end.
    pub terminal_state: Option<ReviewTerminalState>,
    pub created_at: String,
    pub updated_at: String,
    /// Set when a cross-process cancel request was observed.
    #[serde(default)]
    pub cancel_requested: bool,
    /// Root of the run's materialized isolated workspace, recorded
    /// before reviewers spawn so an orphaned-run cancel can release the
    /// directory even when no runner is alive. Stale after a normal
    /// finalize (the runner already removed the workspace) — the orphan
    /// path checks existence before acting.
    #[serde(default)]
    pub workspace_root: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workspace_descriptor: Option<TaskWorkspaceDescriptor>,
}

impl ReviewRunState {
    pub fn is_terminal(&self) -> bool {
        self.terminal_state.is_some()
    }
}

// ---------------------------------------------------------------------------
// Listing / keyset pagination
// ---------------------------------------------------------------------------

/// One row of `review list`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReviewRunSummary {
    pub run_id: String,
    pub kind: String,
    pub agents: Vec<String>,
    pub target_scope: String,
    pub terminal_state: Option<ReviewTerminalState>,
    pub created_at: String,
    pub updated_at: String,
}

/// Keyset cursor for `review list` pagination: strictly-after position in
/// (`created_at DESC`, `run_id DESC`) order (`agent.md` 强制补强项 #5).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReviewRunCursor {
    pub created_at: String,
    pub run_id: String,
}

/// One keyset page.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ReviewRunPage {
    pub items: Vec<ReviewRunSummary>,
    pub next_cursor: Option<ReviewRunCursor>,
    pub has_more: bool,
}

// ---------------------------------------------------------------------------
// Store
// ---------------------------------------------------------------------------

/// Filesystem store for review run directories, rooted at a
/// `.libra/sessions` directory (runs live under
/// `<sessions_root>/agent-runs/<run_id>/`).
#[derive(Clone, Debug)]
pub struct ReviewRunStore {
    sessions_root: PathBuf,
}

impl ReviewRunStore {
    /// Construct a store rooted at a `.libra/sessions` directory.
    pub fn new(sessions_root: impl Into<PathBuf>) -> Self {
        Self {
            sessions_root: sessions_root.into(),
        }
    }

    pub fn sessions_root(&self) -> &Path {
        &self.sessions_root
    }

    /// `<sessions_root>/agent-runs`.
    pub fn runs_root(&self) -> PathBuf {
        self.sessions_root.join(AGENT_RUNS_DIR)
    }

    /// `<sessions_root>/agent-runs/<run_id>` (the `run_id` is validated
    /// against path traversal — see [`is_valid_run_id`]).
    pub fn run_dir(&self, run_id: &str) -> io::Result<PathBuf> {
        if !is_valid_run_id(run_id) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "invalid review run id '{run_id}': expected only ASCII letters, digits, \
                     '-' or '_' (run `libra review list` for known run ids)"
                ),
            ));
        }
        Ok(self.runs_root().join(run_id))
    }

    pub fn state_path(&self, run_id: &str) -> io::Result<PathBuf> {
        Ok(self.run_dir(run_id)?.join("state.json"))
    }

    pub fn manifest_path(&self, run_id: &str) -> io::Result<PathBuf> {
        Ok(self.run_dir(run_id)?.join("manifest.json"))
    }

    pub fn findings_path(&self, run_id: &str) -> io::Result<PathBuf> {
        Ok(self.run_dir(run_id)?.join("findings.md"))
    }

    pub fn reviewers_dir(&self, run_id: &str) -> io::Result<PathBuf> {
        Ok(self.run_dir(run_id)?.join("reviewers"))
    }

    /// `reviewers/<name>.stdout.redacted.log` (the caller supplies a
    /// filesystem-safe reviewer name — see [`sanitize_reviewer_name`]).
    pub fn reviewer_stdout_log_path(&self, run_id: &str, name: &str) -> io::Result<PathBuf> {
        Ok(self
            .reviewers_dir(run_id)?
            .join(format!("{name}.stdout.redacted.log")))
    }

    /// `reviewers/<name>.stderr.redacted.log`.
    pub fn reviewer_stderr_log_path(&self, run_id: &str, name: &str) -> io::Result<PathBuf> {
        Ok(self
            .reviewers_dir(run_id)?
            .join(format!("{name}.stderr.redacted.log")))
    }

    /// Create the run directory (`reviewers/` included) plus the initial
    /// `state.json`, `manifest.json` and an empty `findings.md`. Fails
    /// with [`io::ErrorKind::AlreadyExists`] if the run already exists.
    pub fn create_run(
        &self,
        run_id: &str,
        agents: &[String],
        starting_sha: &str,
        target_scope: &str,
    ) -> io::Result<ReviewRunState> {
        let dir = self.run_dir(run_id)?;
        if dir.exists() {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                format!("review run '{run_id}' already exists at {}", dir.display()),
            ));
        }
        fs::create_dir_all(dir.join("reviewers"))?;

        let now = utc_timestamp();
        let state = ReviewRunState {
            schema_version: REVIEW_MANIFEST_SCHEMA_VERSION,
            run_id: run_id.to_string(),
            kind: REVIEW_RUN_KIND.to_string(),
            agents: agents
                .iter()
                .map(|slug| ReviewerStateEntry {
                    slug: slug.clone(),
                    outcome: None,
                    exit_code: None,
                    stdout_truncated: false,
                    stderr_truncated: false,
                    launch_error: None,
                    pid: None,
                    pgid: None,
                    proc_start_ticks: None,
                })
                .collect(),
            starting_sha: starting_sha.to_string(),
            target_scope: target_scope.to_string(),
            terminal_state: None,
            created_at: now.clone(),
            updated_at: now.clone(),
            cancel_requested: false,
            workspace_root: None,
            workspace_descriptor: None,
        };
        let manifest = ReviewManifest {
            schema_version: REVIEW_MANIFEST_SCHEMA_VERSION,
            run_id: run_id.to_string(),
            kind: REVIEW_RUN_KIND.to_string(),
            agents: agents.to_vec(),
            starting_sha: starting_sha.to_string(),
            target_scope: target_scope.to_string(),
            terminal_state: None,
            created_at: now.clone(),
            updated_at: now,
            findings_oid: None,
            redaction_report: RedactionReportSummary::default(),
            manual_attach: Vec::new(),
            workspace_descriptor: None,
        };
        self.write_state(&state)?;
        self.write_manifest(&manifest)?;
        fs::write(dir.join("findings.md"), b"")?;
        Ok(state)
    }

    /// Whole-file overwrite of `state.json` (snapshot-writer pattern —
    /// the latest write supersedes earlier ones).
    pub fn write_state(&self, state: &ReviewRunState) -> io::Result<()> {
        validate_run_document(state.schema_version, &state.run_id, &state.kind)?;
        self.validate_recorded_descriptor(&state.run_id, state.workspace_descriptor.as_ref())?;
        self.validate_mutation(&state.run_id)?;
        let path = self.state_path(&state.run_id)?;
        let json = serde_json::to_vec_pretty(state).map_err(io::Error::other)?;
        fs::write(path, json)
    }

    /// Load-modify-write one run's `state.json`, bumping `updated_at`.
    /// Returns `Ok(false)` when the run does not exist. Single-writer
    /// contract: during a run only the serial sink (spawn write-through)
    /// and the runner's sequential phases call this — never two writers
    /// concurrently.
    pub fn update_state<F>(&self, run_id: &str, mutate: F) -> io::Result<bool>
    where
        F: FnOnce(&mut ReviewRunState),
    {
        let Some(mut state) = self.load_state(run_id)? else {
            return Ok(false);
        };
        mutate(&mut state);
        state.updated_at = utc_timestamp();
        self.write_state(&state)?;
        Ok(true)
    }

    /// Whole-file overwrite of `manifest.json`.
    pub fn write_manifest(&self, manifest: &ReviewManifest) -> io::Result<()> {
        validate_run_document(manifest.schema_version, &manifest.run_id, &manifest.kind)?;
        self.validate_recorded_descriptor(
            &manifest.run_id,
            manifest.workspace_descriptor.as_ref(),
        )?;
        self.validate_mutation(&manifest.run_id)?;
        let path = self.manifest_path(&manifest.run_id)?;
        let json = serde_json::to_vec_pretty(manifest).map_err(io::Error::other)?;
        fs::write(path, json)
    }

    /// Load `state.json`; `Ok(None)` when the run does not exist.
    pub fn load_state(&self, run_id: &str) -> io::Result<Option<ReviewRunState>> {
        read_json_opt(&self.state_path(run_id)?)
    }

    /// Load `manifest.json`; `Ok(None)` when the run does not exist.
    pub fn load_manifest(&self, run_id: &str) -> io::Result<Option<ReviewManifest>> {
        read_json_opt(&self.manifest_path(run_id)?)
    }

    /// Overwrite `findings.md` (already-redacted content only — the
    /// runner is the sole producer and passes text through the sink's
    /// redaction pipeline first).
    pub fn write_findings(&self, run_id: &str, content: &str) -> io::Result<()> {
        self.validate_mutation(run_id)?;
        fs::write(self.findings_path(run_id)?, content.as_bytes())
    }

    /// Read `findings.md`; `Ok(None)` when the run does not exist.
    pub fn read_findings(&self, run_id: &str) -> io::Result<Option<String>> {
        match fs::read_to_string(self.findings_path(run_id)?) {
            Ok(text) => Ok(Some(text)),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(error),
        }
    }

    /// Append one redacted block to a reviewer log (serial-sink writer
    /// only).
    pub fn append_reviewer_log(&self, run_id: &str, path: &Path, content: &str) -> io::Result<()> {
        self.validate_mutation(run_id)?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;
        file.write_all(content.as_bytes())
    }

    /// A0-06: the `.libra` dir (git object-store root) derived from the
    /// store's `<.libra>/sessions` root.
    fn libra_dir(&self) -> io::Result<PathBuf> {
        self.sessions_root
            .parent()
            .map(Path::to_path_buf)
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "review sessions root has no parent .libra directory",
                )
            })
    }

    /// Refuse writes before they can downgrade a newer schema or mutate a
    /// foreign run sharing the sessions directory.
    pub(crate) fn validate_mutation(&self, run_id: &str) -> io::Result<()> {
        if let Some(state) = self.load_state(run_id)? {
            validate_run_document(state.schema_version, &state.run_id, &state.kind)?;
            self.validate_recorded_descriptor(run_id, state.workspace_descriptor.as_ref())?;
            if state.run_id != run_id {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "review state run id does not match its directory; inspect the run before retrying",
                ));
            }
        }
        if let Some(manifest) = self.load_manifest(run_id)? {
            validate_run_document(manifest.schema_version, &manifest.run_id, &manifest.kind)?;
            self.validate_recorded_descriptor(run_id, manifest.workspace_descriptor.as_ref())?;
            if manifest.run_id != run_id {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "review manifest run id does not match its directory; inspect the run before retrying",
                ));
            }
        }
        Ok(())
    }

    fn validate_recorded_descriptor(
        &self,
        run_id: &str,
        descriptor: Option<&TaskWorkspaceDescriptor>,
    ) -> io::Result<()> {
        validate_workspace_descriptor(run_id, descriptor)?;
        if let Some(descriptor) = descriptor {
            validate_task_workspace_descriptor(&self.libra_dir()?, descriptor.run_id, descriptor)?;
        }
        Ok(())
    }

    /// Record the same current attempt in state and manifest. The intent has
    /// already been persisted by the allocator, so either write can be retried
    /// after a crash without losing the cleanup authority.
    pub fn record_workspace(
        &self,
        run_id: &str,
        descriptor: &TaskWorkspaceDescriptor,
    ) -> io::Result<()> {
        self.validate_mutation(run_id)?;
        if descriptor.run_id.to_string() != run_id {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "review workspace belongs to a different run; inspect its ownership descriptor",
            ));
        }
        verify_task_workspace(&self.libra_dir()?, &self.run_dir(run_id)?, descriptor)?;
        if !self.update_state(run_id, |state| {
            state.schema_version = REVIEW_MANIFEST_SCHEMA_VERSION;
            state.workspace_root = Some(descriptor.canonical_path.display().to_string());
            state.workspace_descriptor = Some(descriptor.clone());
        })? {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "review state is missing while recording its workspace; keep the run metadata and inspect it",
            ));
        }
        let mut manifest = self.load_manifest(run_id)?.ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                "review manifest is missing while recording its workspace",
            )
        })?;
        manifest.schema_version = REVIEW_MANIFEST_SCHEMA_VERSION;
        manifest.workspace_descriptor = Some(descriptor.clone());
        manifest.updated_at = utc_timestamp();
        self.write_manifest(&manifest)
    }

    /// Remove only attempts whose durable intents and allocation proofs belong
    /// to this run. Run metadata remains available if cleanup cannot finish.
    pub fn cleanup_workspaces(&self, run_id: &str) -> io::Result<()> {
        self.validate_mutation(run_id)?;
        let state = self.load_state(run_id)?;
        let manifest = self.load_manifest(run_id)?;
        self.cleanup_recorded_workspaces(
            run_id,
            state
                .as_ref()
                .and_then(|state| state.workspace_descriptor.as_ref())
                .or_else(|| {
                    manifest
                        .as_ref()
                        .and_then(|manifest| manifest.workspace_descriptor.as_ref())
                }),
            state
                .as_ref()
                .and_then(|state| state.workspace_root.as_deref()),
            Instant::now() + Duration::from_secs(30),
        )
    }

    fn cleanup_recorded_workspaces(
        &self,
        run_id: &str,
        descriptor: Option<&TaskWorkspaceDescriptor>,
        recorded_path: Option<&str>,
        deadline: Instant,
    ) -> io::Result<()> {
        if Instant::now() >= deadline {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "review workspace cleanup reached its deadline; keep the run and retry cleanup",
            ));
        }
        let run_dir = self.run_dir(run_id)?;
        let has_intents = has_task_workspace_intents(&run_dir)?;
        let parsed_id = match uuid::Uuid::parse_str(run_id) {
            Ok(id) => Some(id),
            Err(_) if !has_intents && descriptor.is_none() => None,
            Err(error) => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!(
                        "review workspace intents have an invalid run id '{run_id}': {error}; keep the run for inspection"
                    ),
                ));
            }
        };
        validate_workspace_descriptor(run_id, descriptor)?;
        if let Some(parsed_id) = parsed_id
            && (has_intents || descriptor.is_some())
        {
            if let Some(descriptor) = descriptor {
                validate_task_workspace_descriptor(&self.libra_dir()?, parsed_id, descriptor)?;
            }
            cleanup_task_workspaces_until(&self.libra_dir()?, &run_dir, parsed_id, deadline)?;
        }
        // The old path field never authorizes deletion. A surviving unowned
        // path must keep its run metadata, so later inspection remains possible.
        if let Some(path) = descriptor
            .map(|item| item.canonical_path.as_path())
            .or_else(|| recorded_path.map(Path::new))
        {
            // Legacy checkpoint input is independently confined by the existing
            // no-follow run-root cleanup; it needs no migration or new intent.
            if descriptor.is_none() {
                let canonical_input = fs::canonicalize(self.libra_dir()?)?
                    .join("sessions/agent-runs")
                    .join(run_id)
                    .join("checkpoint-input");
                if path == run_dir.join("checkpoint-input") || path == canonical_input {
                    return Ok(());
                }
            }
            match fs::symlink_metadata(path) {
                Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                Err(error) => {
                    return Err(io::Error::new(
                        error.kind(),
                        format!(
                            "cannot verify cleanup of review workspace '{}': {error}; keep the run and inspect filesystem access",
                            path.display()
                        ),
                    ));
                }
                Ok(_) => {
                    return Err(io::Error::new(
                        io::ErrorKind::PermissionDenied,
                        format!(
                            "review workspace '{}' has no removable ownership proof; keep the run and inspect the workspace before retrying cleanup",
                            path.display()
                        ),
                    ));
                }
            }
        }
        Ok(())
    }

    /// §C.4.3 writer-vs-deleter: objectize `bytes` and publish the manifest
    /// that roots the resulting oid, as ONE sequence under the repository
    /// maintenance lock.
    ///
    /// The two halves cannot be separated by a caller, which is the point.
    /// Findings and attachments are content-addressed, so objectizing can
    /// resolve to an oid that already exists — including one a `gc` has
    /// already quarantined. In the window between the objectize and the
    /// manifest write the blob has no root in this run, and the other
    /// protections do not cover it: the manifest EXISTS (so the
    /// missing-manifest fail-closed does not fire) and does not yet name the
    /// oid. A concurrent deletion phase would take the bytes and leave the
    /// manifest pointing at nothing.
    ///
    /// `libra review` / `libra investigate` deliberately do NOT take the
    /// command-level shared hold — an AI run lasts minutes and holding it
    /// would starve every deleter (§C.10) — so the hold lives here, around
    /// the publication itself, which is the only part that needs it.
    /// `publish` receives the oid and must be the call that writes the
    /// manifest naming it.
    pub(crate) fn objectize_and_publish<F>(&self, bytes: &[u8], publish: F) -> io::Result<String>
    where
        F: FnOnce(&str) -> io::Result<()>,
    {
        let libra_dir = self.libra_dir()?;
        let publication = crate::internal::maintenance_lock::MaintenanceLock::shared(&libra_dir)
            .map_err(|error| {
                io::Error::other(format!(
                    "failed to take the repository maintenance lock before publishing: {error}"
                ))
            })?;
        let oid = raw_object::write(&publication, &libra_dir, bytes)?;
        publish(&oid)?;
        Ok(oid)
    }

    /// Stamp the run terminal: updates `state.json` (outcome rows +
    /// terminal state) and `manifest.json` (terminal state + redaction
    /// summary + A0-06 `findings_oid`), bumping `updated_at` on both.
    pub fn finalize_run(
        &self,
        run_id: &str,
        terminal: ReviewTerminalState,
        agents: &[ReviewerStateEntry],
        redaction: RedactionReportSummary,
    ) -> io::Result<()> {
        self.validate_mutation(run_id)?;
        let now = utc_timestamp();
        let mut state = self.load_state(run_id)?.ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("review run '{run_id}' has no state.json to finalize"),
            )
        })?;
        state.agents = agents.to_vec();
        state.terminal_state = Some(terminal);
        state.updated_at = now.clone();
        self.write_state(&state)?;

        let mut manifest = self.load_manifest(run_id)?.ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("review run '{run_id}' has no manifest.json to finalize"),
            )
        })?;
        manifest.terminal_state = Some(terminal);
        manifest.redaction_report = redaction;
        manifest.updated_at = now;
        // A0-06: objectize the final findings.md into the object store so the
        // manifest's `findings_oid` points at a real, object_index-visible,
        // doctor-repairable blob instead of a placeholder null. The objectize
        // and the manifest write that roots it are ONE sequence under the
        // maintenance lock — `objectize_and_publish` is the only way to reach
        // the object writer, so this cannot be split by accident (§C.4.3).
        let findings = self.read_findings(run_id)?.unwrap_or_default();
        if findings.is_empty() {
            return self.write_manifest(&manifest);
        }
        self.objectize_and_publish(findings.as_bytes(), |oid| {
            manifest.findings_oid = Some(oid.to_string());
            self.write_manifest(&manifest)
        })?;
        Ok(())
    }

    /// Cross-process cancel request: drop a marker file the owning
    /// runner polls. Returns `false` when the run does not exist.
    pub fn mark_cancel_requested(&self, run_id: &str) -> io::Result<bool> {
        self.validate_mutation(run_id)?;
        let dir = self.run_dir(run_id)?;
        if !dir.is_dir() {
            return Ok(false);
        }
        fs::write(dir.join(CANCEL_REQUESTED_FILE), b"")?;
        Ok(true)
    }

    /// Whether a cross-process cancel request marker exists.
    pub fn cancel_requested(&self, run_id: &str) -> bool {
        self.run_dir(run_id)
            .map(|dir| dir.join(CANCEL_REQUESTED_FILE).exists())
            .unwrap_or(false)
    }

    /// Mark a non-running (orphaned) run `cancelled` directly in
    /// `state.json` + `manifest.json`. Returns `Ok(true)` when the run
    /// transitioned, `Ok(false)` when it was already terminal, and
    /// [`io::ErrorKind::NotFound`] when it does not exist. The in-process
    /// path for live runs is [`super::runner::ReviewCancelHandle`]; both
    /// converge on the same terminal bookkeeping.
    pub fn mark_cancelled(&self, run_id: &str) -> io::Result<bool> {
        let state = self.load_state(run_id)?.ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("review run '{run_id}' not found (run `libra review list`)"),
            )
        })?;
        if state.is_terminal() {
            return Ok(false);
        }
        let mut agents = state.agents.clone();
        for entry in &mut agents {
            if entry.outcome.is_none() {
                entry.outcome = Some(ReviewerOutcome::Cancelled);
            }
        }
        let redaction = self
            .load_manifest(run_id)?
            .map(|m| m.redaction_report)
            .unwrap_or_default();
        self.finalize_run(run_id, ReviewTerminalState::Cancelled, &agents, redaction)?;
        Ok(true)
    }

    /// Every run, sorted in keyset order (`created_at DESC, run_id DESC`)
    /// — the enumeration order the CLI pagination contract requires.
    /// Unreadable or foreign entries under `agent-runs/` are skipped, not
    /// fatal (tolerant-read pattern, like `list_all_snapshots`).
    pub fn list_runs(&self) -> io::Result<Vec<ReviewRunSummary>> {
        let mut runs = Vec::new();
        let entries = match fs::read_dir(self.runs_root()) {
            Ok(read_dir) => read_dir,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(error) => return Err(error),
        };
        for entry in entries {
            let entry = entry?;
            if !entry.path().is_dir() {
                continue;
            }
            let Ok(Some(state)) = read_json_opt::<ReviewRunState>(&entry.path().join("state.json"))
            else {
                continue;
            };
            runs.push(ReviewRunSummary {
                run_id: state.run_id,
                kind: state.kind,
                agents: state.agents.into_iter().map(|a| a.slug).collect(),
                target_scope: state.target_scope,
                terminal_state: state.terminal_state,
                created_at: state.created_at,
                updated_at: state.updated_at,
            });
        }
        runs.sort_by(|a, b| {
            b.created_at
                .cmp(&a.created_at)
                .then_with(|| b.run_id.cmp(&a.run_id))
        });
        Ok(runs)
    }

    /// One keyset page after `cursor` (exclusive), `limit` items max, in
    /// (`created_at DESC, run_id DESC`) order. The cursor is stable
    /// across inserts: newly created runs sort strictly before an
    /// existing cursor position and cannot duplicate rows into a later
    /// page.
    pub fn list_runs_page(
        &self,
        cursor: Option<&ReviewRunCursor>,
        limit: usize,
    ) -> io::Result<ReviewRunPage> {
        let all = self.list_runs()?;
        let after = |run: &ReviewRunSummary| match cursor {
            None => true,
            Some(cursor) => {
                // Strictly after the cursor in DESC order == strictly
                // smaller (created_at, run_id) tuple.
                (run.created_at.as_str(), run.run_id.as_str())
                    < (cursor.created_at.as_str(), cursor.run_id.as_str())
            }
        };
        let mut remaining = all.into_iter().filter(after);
        let items: Vec<ReviewRunSummary> = remaining.by_ref().take(limit).collect();
        let has_more = remaining.next().is_some();
        let next_cursor = if has_more {
            items.last().map(|run| ReviewRunCursor {
                created_at: run.created_at.clone(),
                run_id: run.run_id.clone(),
            })
        } else {
            None
        };
        Ok(ReviewRunPage {
            items,
            next_cursor,
            has_more,
        })
    }

    /// Remove an own-kind terminal run. `Ok(false)` means it is absent.
    pub fn clean_run(&self, run_id: &str) -> io::Result<bool> {
        self.clean_run_checked(run_id, false)
    }

    fn clean_run_checked(&self, run_id: &str, bulk: bool) -> io::Result<bool> {
        let budget = ScopedIoBudget {
            deadline: Instant::now() + Duration::from_secs(5),
            cancelled: Arc::new(|| false),
        };
        self.clean_run_with_budget(run_id, bulk, &budget)
    }

    fn clean_run_with_budget(
        &self,
        run_id: &str,
        bulk: bool,
        budget: &ScopedIoBudget,
    ) -> io::Result<bool> {
        let dir = self.run_dir(run_id)?;
        let runs_root = self.runs_root();
        let trusted_root = if runs_root.is_absolute() {
            runs_root
        } else {
            std::env::current_dir()?.join(runs_root)
        };
        let root = match open_scoped_run_root(&trusted_root, run_id, budget) {
            Ok(root) => root,
            // The immutable Unix anchor reports absence as this typed refusal;
            // Windows reports NotFound. Preserve the existing false/0 contract.
            Err(CheckpointInputIoError::Refused {
                reason: "a missing trusted root or run directory",
            }) => return Ok(false),
            Err(CheckpointInputIoError::Io { source, .. })
                if source.kind() == io::ErrorKind::NotFound =>
            {
                return Ok(false);
            }
            Err(error) => return Err(io::Error::other(error)),
        };
        let mut metadata = read_run_metadata(&root, budget).map_err(io::Error::other)?;
        if !cleanup_metadata_is_terminal(&metadata, run_id, bulk) {
            if bulk {
                return Ok(false);
            }
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                format!(
                    "review run '{run_id}' is not proven to be a finished review run; \
                     keep it, inspect state.json and manifest.json, and cancel an unfinished \
                     run with `libra review cancel {run_id}` before retrying"
                ),
            ));
        }
        let recorded_state = metadata
            .state
            .as_deref()
            .and_then(|bytes| serde_json::from_slice::<ReviewRunState>(bytes).ok());
        let recorded_manifest = metadata
            .manifest
            .as_deref()
            .and_then(|bytes| serde_json::from_slice::<ReviewManifest>(bytes).ok());
        self.cleanup_recorded_workspaces(
            run_id,
            recorded_state
                .as_ref()
                .and_then(|state| state.workspace_descriptor.as_ref())
                .or_else(|| {
                    recorded_manifest
                        .as_ref()
                        .and_then(|manifest| manifest.workspace_descriptor.as_ref())
                }),
            recorded_state
                .as_ref()
                .and_then(|state| state.workspace_root.as_deref()),
            budget.deadline,
        )?;
        // Drop metadata bodies before taking the second bounded snapshot.
        // The immutable helper owns all no-follow reads and input cleanup.
        metadata.state = None;
        metadata.manifest = None;
        cleanup_checkpoint_input(&root, budget).map_err(io::Error::other)?;
        let current = read_run_metadata(&root, budget).map_err(io::Error::other)?;
        if metadata.root_identity != current.root_identity
            || metadata.state_identity != current.state_identity
            || metadata.manifest_identity != current.manifest_identity
        {
            return Err(io::Error::other(
                "run lifecycle metadata changed during input cleanup; keep the run and inspect concurrent changes before retrying",
            ));
        }
        if Instant::now() >= budget.deadline {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "run cleanup reached its deadline; keep the run and inspect its bounded input before retrying",
            ));
        }
        fs::remove_dir_all(&dir).map_err(|error| {
            io::Error::new(
                error.kind(),
                format!("failed to remove finished review run '{run_id}': {error}; inspect filesystem access before retrying"),
            )
        })?;
        Ok(true)
    }

    /// Remove only readable own-kind terminal runs. Foreign, unfinished and
    /// corrupt bulk candidates survive; removed counts successful deletions.
    pub fn clean_all(&self) -> io::Result<usize> {
        let entries = match fs::read_dir(self.runs_root()) {
            Ok(read_dir) => read_dir,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(0),
            Err(error) => return Err(error),
        };
        let mut removed = 0usize;
        for entry in entries {
            let entry = entry?;
            let file_type = entry.file_type()?;
            if !file_type.is_dir() && !file_type.is_symlink() {
                continue;
            }
            let name = entry.file_name();
            let Some(run_id) = name.to_str().filter(|id| is_valid_run_id(id)) else {
                continue;
            };
            if self.clean_run_checked(run_id, true)? {
                removed += 1;
            }
        }
        Ok(removed)
    }
}

/// A parseable foreign, incomplete or nonterminal document vetoes cleanup.
/// Only a specifically requested corrupt/missing state may use a valid
/// same-root own-kind terminal manifest; bulk never sweeps that fallback.
fn cleanup_metadata_is_terminal(metadata: &RunMetadataBytes, run_id: &str, bulk: bool) -> bool {
    let state = match metadata.state.as_deref() {
        Some(bytes) if serde_json::from_slice::<serde::de::IgnoredAny>(bytes).is_ok() => {
            match serde_json::from_slice::<ReviewRunState>(bytes) {
                Ok(state) => Some(state),
                Err(_) => return false,
            }
        }
        _ => None,
    };
    if bulk && state.is_none() {
        return false;
    }
    let manifest = match metadata.manifest.as_deref() {
        Some(bytes) => match serde_json::from_slice::<ReviewManifest>(bytes) {
            Ok(manifest) => Some(manifest),
            Err(_) => return false,
        },
        None => None,
    };
    if let Some(state) = &state
        && (!supported_run_schema(state.schema_version)
            || state.run_id != run_id
            || state.kind != REVIEW_RUN_KIND
            || validate_workspace_descriptor(run_id, state.workspace_descriptor.as_ref()).is_err()
            || !state.is_terminal())
    {
        return false;
    }
    if let Some(manifest) = &manifest
        && (!supported_run_schema(manifest.schema_version)
            || manifest.run_id != run_id
            || manifest.kind != REVIEW_RUN_KIND
            || validate_workspace_descriptor(run_id, manifest.workspace_descriptor.as_ref())
                .is_err())
    {
        return false;
    }
    match (state, manifest) {
        (Some(state), Some(manifest)) => {
            // Finalization writes terminal state before publishing the manifest.
            // An interrupted second write leaves None, which is not a conflict
            // with this already-proved own-kind terminal state.
            (manifest.terminal_state.is_none() || state.terminal_state == manifest.terminal_state)
                && (state.workspace_descriptor.is_none()
                    || manifest.workspace_descriptor.is_none()
                    || state.workspace_descriptor == manifest.workspace_descriptor)
                && state.starting_sha == manifest.starting_sha
                // Infrastructure finalization has no reviewer reports and
                // writes an empty state list, retaining the requested manifest
                // agents. Only present outcome rows can conflict with it.
                && (state.agents.is_empty()
                    || state
                        .agents
                        .iter()
                        .map(|entry| entry.slug.as_str())
                        .eq(manifest.agents.iter().map(String::as_str)))
                && state.target_scope == manifest.target_scope
        }
        (Some(_), None) => true,
        (None, Some(manifest)) => manifest.terminal_state.is_some(),
        (None, None) => false,
    }
}

fn supported_run_schema(version: u32) -> bool {
    matches!(version, 1 | REVIEW_MANIFEST_SCHEMA_VERSION)
}

fn validate_run_document(version: u32, run_id: &str, kind: &str) -> io::Result<()> {
    if !supported_run_schema(version) || kind != REVIEW_RUN_KIND {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "review run '{run_id}' has unsupported schema {version} or kind '{kind}'; use a compatible Libra version and keep its metadata unchanged"
            ),
        ));
    }
    Ok(())
}

fn validate_workspace_descriptor(
    run_id: &str,
    descriptor: Option<&TaskWorkspaceDescriptor>,
) -> io::Result<()> {
    if let Some(descriptor) = descriptor
        && (descriptor.run_id.to_string() != run_id || descriptor.layout_version != 1)
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "review workspace descriptor has an unsupported layout or belongs to a different run; keep the run for inspection",
        ));
    }
    Ok(())
}

/// Run ids are uuid-shaped in production; the guard exists so a
/// CLI-supplied run id can never escape `agent-runs/` (path traversal).
pub fn is_valid_run_id(run_id: &str) -> bool {
    !run_id.is_empty()
        && run_id.len() <= 128
        && run_id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

/// Filesystem-safe reviewer log name: lowercased, every char outside
/// `[a-z0-9_-]` mapped to `-`, empty input mapped to `reviewer`. `.` is
/// deliberately excluded so `..` can never appear.
pub fn sanitize_reviewer_name(slug: &str) -> String {
    let name: String = slug
        .chars()
        .map(|c| {
            let c = c.to_ascii_lowercase();
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '-'
            }
        })
        .collect();
    if name.is_empty() {
        "reviewer".to_string()
    } else {
        name
    }
}

/// RFC 3339 UTC with fixed microsecond precision — fixed width, so
/// lexicographic string order equals chronological order (the keyset
/// contract).
pub(crate) fn utc_timestamp() -> String {
    chrono::DateTime::<chrono::Utc>::from(SystemTime::now())
        .to_rfc3339_opts(chrono::SecondsFormat::Micros, true)
}

pub(crate) fn read_json_opt<T: serde::de::DeserializeOwned>(path: &Path) -> io::Result<Option<T>> {
    match fs::read(path) {
        Ok(bytes) => {
            let value = serde_json::from_slice(&bytes).map_err(io::Error::other)?;
            Ok(Some(value))
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error),
    }
}

/// Raw object publication — the object writer, behind a lock the caller must
/// already hold.
///
/// The boundary is enforced by the module system and the signature, not by
/// convention: [`write`] is private to this file, and it cannot be called
/// without a [`MaintenanceLock`] guard, which only
/// `review` run publication takes. A future site that wanted to objectize
/// bytes without publishing them would have to acquire the lock to do it,
/// which is the invariant (plan-20260714 §C.4.3).
mod raw_object {
    use std::{io, path::Path};

    use super::AGENT_FINDINGS_OTYPE;
    use crate::internal::maintenance_lock::MaintenanceLock;

    /// Write `bytes` as a content-addressed git blob and enqueue it into
    /// `object_index`. Content-addressed, so re-writing identical bytes is
    /// idempotent.
    pub(super) fn write(
        _held: &MaintenanceLock,
        libra_dir: &Path,
        bytes: &[u8],
    ) -> io::Result<String> {
        let oid = crate::utils::object::write_git_object(libra_dir, "blob", bytes)
            .map_err(|e| io::Error::other(format!("failed to write findings object: {e}")))?;
        crate::utils::client_storage::enqueue_agent_blob_object_index_update(
            libra_dir,
            &oid.to_string(),
            AGENT_FINDINGS_OTYPE,
            bytes.len() as i64,
        )?;
        Ok(oid.to_string())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;

    fn store() -> (tempfile::TempDir, ReviewRunStore) {
        let dir = tempfile::tempdir().expect("tempdir");
        // macOS tempdir may spell /var through a system symlink. The
        // fixture uses its physical root; production no-follow is unchanged.
        let physical = fs::canonicalize(dir.path()).expect("physical temp root");
        let store = ReviewRunStore::new(physical.join(".libra").join("sessions"));
        (dir, store)
    }

    #[test]
    fn legacy_schema_is_preserved_and_future_schema_refuses_mutation() {
        let (_dir, store) = store();
        store.create_run("legacy", &[], "sha", "scope").unwrap();
        for path in [
            store.state_path("legacy").unwrap(),
            store.manifest_path("legacy").unwrap(),
        ] {
            let mut value: serde_json::Value =
                serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
            value["schema_version"] = 1.into();
            fs::write(&path, serde_json::to_vec(&value).unwrap()).unwrap();
        }
        assert!(store.mark_cancelled("legacy").unwrap());
        assert_eq!(
            store.load_state("legacy").unwrap().unwrap().schema_version,
            1
        );
        assert_eq!(
            store
                .load_manifest("legacy")
                .unwrap()
                .unwrap()
                .schema_version,
            1
        );
        assert!(store.clean_run("legacy").unwrap());

        store.create_run("future", &[], "sha", "scope").unwrap();
        let manifest_path = store.manifest_path("future").unwrap();
        let mut value: serde_json::Value =
            serde_json::from_slice(&fs::read(&manifest_path).unwrap()).unwrap();
        value["schema_version"] = 99.into();
        fs::write(&manifest_path, serde_json::to_vec(&value).unwrap()).unwrap();
        let state_before = fs::read(store.state_path("future").unwrap()).unwrap();
        let manifest_before = fs::read(&manifest_path).unwrap();
        assert!(
            store.load_manifest("future").unwrap().is_some(),
            "future metadata can still be inspected"
        );
        assert!(store.mark_cancel_requested("future").is_err());
        assert!(store.mark_cancelled("future").is_err());
        assert!(
            store
                .write_findings("future", "must not be written")
                .is_err()
        );
        assert!(store.clean_run("future").is_err());
        assert!(!store.cancel_requested("future"));
        assert_eq!(
            fs::read(store.state_path("future").unwrap()).unwrap(),
            state_before
        );
        assert_eq!(fs::read(&manifest_path).unwrap(), manifest_before);
        assert_eq!(store.read_findings("future").unwrap().unwrap(), "");
    }

    #[test]
    fn managed_cleanup_preserves_metadata_until_ownership_is_proven() {
        use crate::internal::ai::workspace_isolation::{TaskWorkspaceKind, create_task_workspace};

        let (_dir, store) = store();
        let run_id = uuid::Uuid::now_v7();
        let id = run_id.to_string();
        store.create_run(&id, &[], "sha", "scope").unwrap();
        let descriptor = create_task_workspace(
            &store.libra_dir().unwrap(),
            &store.run_dir(&id).unwrap(),
            run_id,
            TaskWorkspaceKind::Workspace,
        )
        .unwrap();
        store.record_workspace(&id, &descriptor).unwrap();
        store.mark_cancelled(&id).unwrap();
        fs::write(descriptor.canonical_path.join("sentinel"), "owned").unwrap();

        // A future manifest layout must veto cleanup even if the state carries
        // a valid descriptor. Neither workspace nor metadata may be removed.
        let manifest_path = store.manifest_path(&id).unwrap();
        let before = fs::read(&manifest_path).unwrap();
        let mut future: serde_json::Value = serde_json::from_slice(&before).unwrap();
        future["workspace_descriptor"]["layout_version"] = 99.into();
        fs::write(&manifest_path, serde_json::to_vec(&future).unwrap()).unwrap();
        assert!(store.clean_run(&id).is_err());
        assert_eq!(
            fs::read_to_string(descriptor.canonical_path.join("sentinel")).unwrap(),
            "owned"
        );
        assert!(store.run_dir(&id).unwrap().exists());
        fs::write(&manifest_path, before).unwrap();

        // Replacing an owned directory with another one must not turn a valid
        // path string into authority to delete the replacement.
        let saved = descriptor.canonical_path.with_extension("saved");
        fs::rename(&descriptor.canonical_path, &saved).unwrap();
        fs::create_dir(&descriptor.canonical_path).unwrap();
        fs::write(descriptor.canonical_path.join("sentinel"), "replacement").unwrap();
        assert!(store.clean_run(&id).is_err());
        assert_eq!(
            fs::read_to_string(descriptor.canonical_path.join("sentinel")).unwrap(),
            "replacement"
        );
        assert!(store.run_dir(&id).unwrap().exists());
        fs::remove_dir_all(&descriptor.canonical_path).unwrap();
        fs::rename(&saved, &descriptor.canonical_path).unwrap();
        assert!(store.clean_run(&id).unwrap());
        assert!(!descriptor.canonical_path.exists());
        assert!(!store.run_dir(&id).unwrap().exists());
    }

    /// E8-libra manifest contract: exactly these keys, no more, no less
    /// (`agent.md:876` / `agent.md:1321`).
    #[test]
    fn manifest_serializes_exactly_the_e8_key_set() {
        let manifest = ReviewManifest {
            schema_version: REVIEW_MANIFEST_SCHEMA_VERSION,
            run_id: "r1".into(),
            kind: REVIEW_RUN_KIND.into(),
            agents: vec!["codex".into()],
            starting_sha: "abc123".into(),
            target_scope: "HEAD~1..HEAD".into(),
            terminal_state: Some(ReviewTerminalState::Success),
            created_at: utc_timestamp(),
            updated_at: utc_timestamp(),
            findings_oid: None,
            redaction_report: RedactionReportSummary::default(),
            manual_attach: Vec::new(),
            workspace_descriptor: None,
        };
        let value = serde_json::to_value(&manifest).expect("manifest to json");
        let keys: BTreeSet<String> = value
            .as_object()
            .expect("manifest is a json object")
            .keys()
            .cloned()
            .collect();
        let expected: BTreeSet<String> = [
            "schema_version",
            "run_id",
            "kind",
            "agents",
            "starting_sha",
            "target_scope",
            "terminal_state",
            "created_at",
            "updated_at",
            "findings_oid",
            "redaction_report",
            "manual_attach",
        ]
        .into_iter()
        .map(str::to_string)
        .collect();
        assert_eq!(
            keys, expected,
            "manifest.json must carry exactly the E8 keys"
        );
        // A create-time / attachment-free manifest serializes findings_oid as
        // null and manual_attach as []; A0-06 populates them at finalize /
        // attach without changing the 12-key set (proven in the workflow
        // tests). This case pins the empty-state serialization only.
        assert_eq!(value["manual_attach"], serde_json::json!([]));
        assert_eq!(value["findings_oid"], serde_json::Value::Null);
    }

    #[test]
    fn terminal_state_aggregation_truth_table() {
        use ReviewerOutcome::*;
        // Cancel wins over everything.
        assert_eq!(
            aggregate_terminal_state(true, &[Ok, Ok]),
            ReviewTerminalState::Cancelled
        );
        assert_eq!(
            aggregate_terminal_state(true, &[]),
            ReviewTerminalState::Cancelled
        );
        // Nothing ever ran → infrastructure error.
        assert_eq!(
            aggregate_terminal_state(false, &[]),
            ReviewTerminalState::Error
        );
        // All ok → success.
        assert_eq!(
            aggregate_terminal_state(false, &[Ok, Ok]),
            ReviewTerminalState::Success
        );
        // Mixed → partial.
        assert_eq!(
            aggregate_terminal_state(false, &[Ok, Failed]),
            ReviewTerminalState::Partial
        );
        assert_eq!(
            aggregate_terminal_state(false, &[Ok, TimedOut]),
            ReviewTerminalState::Partial
        );
        // None ok, any timed out → timeout.
        assert_eq!(
            aggregate_terminal_state(false, &[Failed, TimedOut]),
            ReviewTerminalState::Timeout
        );
        // None ok, none timed out → error.
        assert_eq!(
            aggregate_terminal_state(false, &[Failed, Failed]),
            ReviewTerminalState::Error
        );
        assert_eq!(
            aggregate_terminal_state(false, &[Cancelled]),
            ReviewTerminalState::Error
        );
    }

    /// §C.4.3: publishing an objectized blob holds the repository
    /// maintenance lock across the manifest write that roots it.
    ///
    /// The invariant is STRUCTURAL, not merely tested: `objectize_bytes` is
    /// private, so the only route to the object writer is
    /// `objectize_and_publish`, which takes the lock, objectizes, and calls
    /// the publisher while still holding it. Deleting the lock acquisition
    /// would not "still pass this test" — it would remove the only place the
    /// lock is taken for these two commands.
    ///
    /// What the test adds is the ordering: the manifest naming the oid is on
    /// disk BEFORE the guard is released. It proves that by observing, from
    /// inside the publish callback, that the lock file is held — and by
    /// requiring the manifest write to have happened by the time the call
    /// returns. What the lock then guarantees against another process is
    /// pinned by `internal::maintenance_lock`'s tests and, end to end, by
    /// `gc_defers_deletion_while_a_publisher_holds_the_maintenance_lock`.
    #[test]
    fn publication_holds_the_lock_across_the_manifest_write() {
        let (dir, store) = store();
        store
            .create_run("run-lock", &["codex".to_string()], "sha-1", "worktree")
            .expect("create");

        let lock_file = dir.path().join(".libra").join("maintenance.lock");
        let mut observed_lock_held = false;
        let mut manifest = store.load_manifest("run-lock").expect("load").expect("run");
        let oid = store
            .objectize_and_publish(b"findings body", |oid| {
                // Inside the publication window: an outside acquirer must not
                // be able to take the lock exclusively. This process holds it
                // shared, and a shared hold is never upgradable, so the
                // refusal here is the same one another process would get.
                observed_lock_held =
                    crate::internal::maintenance_lock::MaintenanceLock::try_exclusive(
                        &dir.path().join(".libra"),
                        std::time::Duration::from_millis(50),
                    )
                    .expect("probe")
                    .is_none();
                manifest.findings_oid = Some(oid.to_string());
                store.write_manifest(&manifest)
            })
            .expect("publish");

        assert!(lock_file.exists(), "the publication took the lock");
        assert!(
            observed_lock_held,
            "the lock must still be held while the manifest is being written"
        );
        // The manifest that roots the blob is on disk when the call returns.
        let published = store.load_manifest("run-lock").expect("load").expect("run");
        assert_eq!(published.findings_oid.as_deref(), Some(oid.as_str()));
    }

    /// Tripwire for the structural boundary.
    ///
    /// The boundary itself is the module system: the object writer lives in
    /// the private `raw_object` module and its signature demands a held
    /// `MaintenanceLock`, so no site can objectize bytes without taking the
    /// lock — that is not something a test enforces. This only catches the
    /// cheaper regression of someone adding a second call inside the one
    /// function that already holds a guard and then publishing later.
    #[test]
    fn the_object_writer_has_exactly_one_caller() {
        let source = include_str!("store.rs");
        // Split so this test's own text is not a match.
        let needle = concat!("raw_object::", "write(");
        let calls = source.matches(needle).count();
        assert_eq!(
            calls, 1,
            "the object writer must be called only from objectize_and_publish, which holds the \
             maintenance lock across the publication"
        );
    }

    #[test]
    fn create_load_finalize_roundtrip() {
        let (_dir, store) = store();
        let agents = vec!["codex".to_string(), "opencode".to_string()];
        let state = store
            .create_run("run-a", &agents, "sha-1", "worktree")
            .expect("create");
        assert!(!state.is_terminal());
        assert!(store.load_manifest("run-a").expect("load").is_some());
        assert_eq!(
            store.read_findings("run-a").expect("findings"),
            Some(String::new())
        );

        // Double-create must fail loudly.
        let err = store
            .create_run("run-a", &agents, "sha-1", "worktree")
            .expect_err("duplicate create");
        assert_eq!(err.kind(), io::ErrorKind::AlreadyExists);

        let rows = vec![
            ReviewerStateEntry {
                slug: "codex".into(),
                outcome: Some(ReviewerOutcome::Ok),
                exit_code: Some(0),
                stdout_truncated: false,
                stderr_truncated: false,
                launch_error: None,
                pid: None,
                pgid: None,
                proc_start_ticks: None,
            },
            ReviewerStateEntry {
                slug: "opencode".into(),
                outcome: Some(ReviewerOutcome::Failed),
                exit_code: Some(2),
                stdout_truncated: true,
                stderr_truncated: false,
                launch_error: None,
                pid: None,
                pgid: None,
                proc_start_ticks: None,
            },
        ];
        store
            .finalize_run(
                "run-a",
                ReviewTerminalState::Partial,
                &rows,
                RedactionReportSummary {
                    matches: 1,
                    bytes_scanned: 10,
                    bytes_redacted: 4,
                },
            )
            .expect("finalize");
        let state = store.load_state("run-a").expect("load").expect("state");
        assert_eq!(state.terminal_state, Some(ReviewTerminalState::Partial));
        assert!(state.updated_at >= state.created_at);
        let manifest = store
            .load_manifest("run-a")
            .expect("load")
            .expect("manifest");
        assert_eq!(manifest.terminal_state, Some(ReviewTerminalState::Partial));
        assert_eq!(manifest.redaction_report.matches, 1);
    }

    #[test]
    fn list_is_keyset_ordered_and_paginates_without_dup_or_loss() {
        let (_dir, store) = store();
        // Force controlled created_at values (same timestamp for b/c to
        // exercise the run_id DESC tiebreak).
        for (run_id, created_at) in [
            ("run-a", "2026-07-01T00:00:00.000000Z"),
            ("run-b", "2026-07-02T00:00:00.000000Z"),
            ("run-c", "2026-07-02T00:00:00.000000Z"),
            ("run-d", "2026-07-03T00:00:00.000000Z"),
        ] {
            let mut state = store
                .create_run(run_id, &["codex".to_string()], "sha", "scope")
                .expect("create");
            state.created_at = created_at.to_string();
            state.updated_at = created_at.to_string();
            store.write_state(&state).expect("write");
        }

        let all = store.list_runs().expect("list");
        let ids: Vec<&str> = all.iter().map(|r| r.run_id.as_str()).collect();
        assert_eq!(
            ids,
            ["run-d", "run-c", "run-b", "run-a"],
            "created_at DESC then run_id DESC"
        );

        // Walk pages of 3 then the remainder: no duplicates, no loss.
        let page1 = store.list_runs_page(None, 3).expect("page1");
        assert_eq!(page1.items.len(), 3);
        assert!(page1.has_more);
        let cursor = page1.next_cursor.clone().expect("cursor");
        assert_eq!(cursor.run_id, "run-b");
        let page2 = store.list_runs_page(Some(&cursor), 3).expect("page2");
        assert_eq!(page2.items.len(), 1);
        assert!(!page2.has_more);
        assert!(page2.next_cursor.is_none());
        let mut walked: Vec<String> = page1
            .items
            .iter()
            .chain(page2.items.iter())
            .map(|r| r.run_id.clone())
            .collect();
        walked.sort();
        let mut expected: Vec<String> = ids.iter().map(|s| s.to_string()).collect();
        expected.sort();
        assert_eq!(walked, expected);
    }

    #[test]
    fn cancel_mark_and_clean() {
        let (_dir, store) = store();
        store
            .create_run("run-x", &["codex".to_string()], "sha", "scope")
            .expect("create");

        assert!(!store.cancel_requested("run-x"));
        assert!(store.mark_cancel_requested("run-x").expect("mark"));
        assert!(store.cancel_requested("run-x"));
        assert!(
            !store
                .mark_cancel_requested("missing")
                .expect("mark missing")
        );

        assert!(store.mark_cancelled("run-x").expect("cancel"));
        let state = store.load_state("run-x").expect("load").expect("state");
        assert_eq!(state.terminal_state, Some(ReviewTerminalState::Cancelled));
        assert_eq!(
            state.agents[0].outcome,
            Some(ReviewerOutcome::Cancelled),
            "pending reviewers are marked cancelled"
        );
        // Idempotent: already terminal.
        assert!(!store.mark_cancelled("run-x").expect("cancel again"));

        assert!(store.clean_run("run-x").expect("clean"));
        assert!(!store.clean_run("run-x").expect("clean missing"));
        store
            .create_run("run-y", &["codex".to_string()], "sha", "scope")
            .expect("create");
        assert!(store.mark_cancelled("run-y").expect("finish run-y"));
        store
            .create_run("keep-paused", &["codex".into()], "sha", "scope")
            .expect("create nonterminal survivor");
        let paused_path = store.state_path("keep-paused").unwrap();
        let paused_bytes = fs::read(&paused_path).unwrap();
        assert_eq!(store.clean_all().expect("clean all"), 1);
        assert_eq!(fs::read(&paused_path).unwrap(), paused_bytes);
        assert_eq!(store.clean_all().expect("only nonterminal survivors"), 0);
        assert_eq!(fs::read(&paused_path).unwrap(), paused_bytes);
    }

    #[cfg(unix)]
    #[test]
    fn fix_rg_scoped_03_cleanup_boundary() {
        use std::os::unix::fs::{PermissionsExt, symlink};

        fn make_terminal(s: &ReviewRunStore, id: &str) {
            s.create_run(id, &["codex".into()], "sha", "scope").unwrap();
            assert!(s.mark_cancelled(id).unwrap());
        }
        fn readonly_input(root: &Path) -> PathBuf {
            let input = root.join("checkpoint-input");
            fs::create_dir_all(input.join("nested")).unwrap();
            fs::write(input.join("nested/payload"), b"captured ordinary").unwrap();
            fs::set_permissions(
                input.join("nested/payload"),
                fs::Permissions::from_mode(0o444),
            )
            .unwrap();
            fs::set_permissions(input.join("nested"), fs::Permissions::from_mode(0o555)).unwrap();
            fs::set_permissions(&input, fs::Permissions::from_mode(0o555)).unwrap();
            input
        }
        let (temp, store) = store();
        store
            .create_run("unfinished", &["codex".into()], "sha", "scope")
            .unwrap();
        let paused = store.state_path("unfinished").unwrap();
        let bytes = fs::read(&paused).unwrap();
        assert_eq!(
            store.clean_run("unfinished").unwrap_err().kind(),
            io::ErrorKind::PermissionDenied
        );
        assert_eq!(store.clean_all().unwrap(), 0);
        assert_eq!(fs::read(&paused).unwrap(), bytes);

        let other =
            crate::internal::ai::investigate::InvestigateRunStore::new(store.sessions_root());
        other
            .create_run("foreign", "t", &["codex".into()], 4, 1, "sha")
            .unwrap();
        assert!(other.mark_cancelled("foreign").unwrap());
        let foreign_state = fs::read(other.state_path("foreign").unwrap()).unwrap();
        assert!(store.clean_run("foreign").is_err());
        assert_eq!(store.clean_all().unwrap(), 0);
        assert_eq!(
            fs::read(other.state_path("foreign").unwrap()).unwrap(),
            foreign_state
        );

        let outside = temp.path().join("outside");
        fs::write(&outside, b"outside sentinel").unwrap();
        fs::set_permissions(&outside, fs::Permissions::from_mode(0o444)).unwrap();
        make_terminal(&store, "finished");
        let root = store.run_dir("finished").unwrap();
        let input = readonly_input(&root);
        // Fixture construction only; production starts with real readonly input.
        fs::set_permissions(input.join("nested"), fs::Permissions::from_mode(0o755)).unwrap();
        symlink(&outside, input.join("nested/outside-link")).unwrap();
        fs::set_permissions(input.join("nested"), fs::Permissions::from_mode(0o555)).unwrap();
        assert!(store.clean_run("finished").unwrap());
        assert!(!root.exists());
        assert_eq!(fs::read(&outside).unwrap(), b"outside sentinel");
        assert_eq!(
            fs::metadata(&outside).unwrap().permissions().mode() & 0o777,
            0o444
        );
        assert!(!store.clean_run("finished").unwrap());

        make_terminal(&store, "corrupt");
        fs::write(store.state_path("corrupt").unwrap(), b"{ broken state").unwrap();
        let corrupt_input = readonly_input(&store.run_dir("corrupt").unwrap());
        assert_eq!(
            store.clean_all().unwrap(),
            0,
            "bulk cannot use corrupt-state fallback"
        );
        assert_eq!(
            fs::metadata(&corrupt_input).unwrap().permissions().mode() & 0o777,
            0o555
        );
        assert!(
            store.clean_run("corrupt").unwrap(),
            "specific valid terminal manifest recovery"
        );

        // Exercise the real first state-write API with the original unfinished
        // manifest: both direct and library bulk cleanup must remain possible.
        store
            .create_run("state-first", &["codex".into()], "sha", "scope")
            .unwrap();
        assert!(
            store
                .update_state("state-first", |state| {
                    state.terminal_state = Some(ReviewTerminalState::Cancelled);
                })
                .unwrap()
        );
        assert!(
            store
                .load_manifest("state-first")
                .unwrap()
                .unwrap()
                .terminal_state
                .is_none()
        );
        readonly_input(&store.run_dir("state-first").unwrap());
        assert_eq!(store.clean_all().unwrap(), 1);
        store
            .create_run("state-first-direct", &["codex".into()], "sha", "scope")
            .unwrap();
        assert!(
            store
                .update_state("state-first-direct", |state| {
                    state.terminal_state = Some(ReviewTerminalState::Cancelled);
                })
                .unwrap()
        );
        readonly_input(&store.run_dir("state-first-direct").unwrap());
        assert!(store.clean_run("state-first-direct").unwrap());

        // Real infrastructure finalization replaces state rows with the empty
        // report slice while the manifest still names requested reviewers.
        for (id, bulk) in [("infra-empty-bulk", true), ("infra-empty-direct", false)] {
            store
                .create_run(id, &["codex".into()], "sha", "scope")
                .unwrap();
            store
                .finalize_run(
                    id,
                    ReviewTerminalState::Error,
                    &[],
                    RedactionReportSummary::default(),
                )
                .unwrap();
            assert!(store.load_state(id).unwrap().unwrap().agents.is_empty());
            assert_eq!(store.load_manifest(id).unwrap().unwrap().agents, ["codex"]);
            readonly_input(&store.run_dir(id).unwrap());
            if bulk {
                assert_eq!(store.clean_all().unwrap(), 1);
            } else {
                assert!(store.clean_run(id).unwrap());
            }
            assert!(!store.run_dir(id).unwrap().exists());
        }

        for (id, field, value) in [
            ("bad-kind", "kind", serde_json::json!("foreign")),
            ("bad-id", "run_id", serde_json::json!("different-run")),
            ("bad-schema", "schema_version", serde_json::json!(99)),
            (
                "bad-starting",
                "starting_sha",
                serde_json::json!("different-start"),
            ),
            (
                "bad-scope",
                "target_scope",
                serde_json::json!("different-scope"),
            ),
            (
                "bad-terminal",
                "terminal_state",
                serde_json::json!("success"),
            ),
        ] {
            make_terminal(&store, id);
            let manifest = store.manifest_path(id).unwrap();
            let mut doc: serde_json::Value =
                serde_json::from_slice(&fs::read(&manifest).unwrap()).unwrap();
            doc[field] = value;
            fs::write(&manifest, serde_json::to_vec(&doc).unwrap()).unwrap();
            let state_before = fs::read(store.state_path(id).unwrap()).unwrap();
            let input = readonly_input(&store.run_dir(id).unwrap());
            assert!(store.clean_run(id).is_err());
            assert_eq!(store.clean_all().unwrap(), 0);
            assert_eq!(
                fs::read(store.state_path(id).unwrap()).unwrap(),
                state_before
            );
            assert_eq!(
                fs::metadata(&input).unwrap().permissions().mode() & 0o777,
                0o555
            );
            // Test teardown after the refusal has been proved, never a bypass.
            fs::set_permissions(&input, fs::Permissions::from_mode(0o755)).unwrap();
            fs::set_permissions(input.join("nested"), fs::Permissions::from_mode(0o755)).unwrap();
        }
        make_terminal(&store, "parseable-foreign");
        fs::write(
            store.state_path("parseable-foreign").unwrap(),
            br#"{"kind":"foreign"}"#,
        )
        .unwrap();
        assert!(
            store.clean_run("parseable-foreign").is_err(),
            "valid foreign JSON cannot use the own terminal manifest"
        );
        assert_eq!(store.clean_all().unwrap(), 0);
        make_terminal(&store, "deadline");
        let root = store.run_dir("deadline").unwrap();
        let input = readonly_input(&root);
        let expired = ScopedIoBudget {
            deadline: Instant::now(),
            cancelled: Arc::new(|| false),
        };
        assert!(
            store
                .clean_run_with_budget("deadline", false, &expired)
                .is_err()
        );
        assert_eq!(
            fs::metadata(&input).unwrap().permissions().mode() & 0o777,
            0o555
        );
        assert_eq!(
            fs::read(input.join("nested/payload")).unwrap(),
            b"captured ordinary"
        );
        assert!(store.clean_run("deadline").unwrap());

        // Root, ancestor and metadata symlinks are refused before input chmod.
        make_terminal(&store, "linked-target");
        let target = store.run_dir("linked-target").unwrap();
        let input = readonly_input(&target);
        let link = store.run_dir("linked-root").unwrap();
        symlink(&target, &link).unwrap();
        assert!(store.clean_run("linked-root").is_err());
        fs::remove_file(&link).unwrap();
        let sessions_link = temp.path().join("linked-sessions");
        symlink(store.sessions_root(), &sessions_link).unwrap();
        let through_link = ReviewRunStore::new(&sessions_link);
        assert!(through_link.clean_run("linked-target").is_err());
        fs::remove_file(&sessions_link).unwrap();
        for name in ["state.json", "manifest.json"] {
            let path = target.join(name);
            let saved = fs::read(&path).unwrap();
            fs::remove_file(&path).unwrap();
            symlink(&outside, &path).unwrap();
            assert!(store.clean_run("linked-target").is_err());
            assert_eq!(
                fs::metadata(&input).unwrap().permissions().mode() & 0o777,
                0o555
            );
            fs::remove_file(&path).unwrap();
            fs::write(&path, saved).unwrap();
        }
        assert!(store.clean_run("linked-target").unwrap());

        make_terminal(&store, "over-metadata");
        let root = store.run_dir("over-metadata").unwrap();
        let input = readonly_input(&root);
        let state_path = store.state_path("over-metadata").unwrap();
        let state = fs::read(&state_path).unwrap();
        fs::File::options()
            .write(true)
            .open(&state_path)
            .unwrap()
            .set_len(64 * 1024 * 1024 + 1)
            .unwrap();
        assert!(store.clean_run("over-metadata").is_err());
        assert_eq!(
            fs::metadata(&input).unwrap().permissions().mode() & 0o777,
            0o555
        );
        fs::write(&state_path, state).unwrap();
        assert!(store.clean_run("over-metadata").unwrap());

        make_terminal(&store, "over-input");
        let root = store.run_dir("over-input").unwrap();
        let input = root.join("checkpoint-input");
        fs::create_dir(&input).unwrap();
        for n in 0..4097 {
            fs::write(input.join(format!("f-{n:04}")), b"bounded").unwrap();
        }
        fs::set_permissions(&input, fs::Permissions::from_mode(0o555)).unwrap();
        assert!(store.clean_run("over-input").is_err());
        assert_eq!(
            fs::metadata(&input).unwrap().permissions().mode() & 0o777,
            0o555
        );
        assert_eq!(fs::read_dir(&input).unwrap().count(), 4097);
        fs::set_permissions(&input, fs::Permissions::from_mode(0o755)).unwrap();
        fs::remove_dir_all(&input).unwrap();
        assert!(store.clean_run("over-input").unwrap());
        make_terminal(&store, "bulk-finished");
        readonly_input(&store.run_dir("bulk-finished").unwrap());
        assert_eq!(store.clean_all().unwrap(), 1);
        assert!(!store.run_dir("bulk-finished").unwrap().exists());
        assert_eq!(fs::read(&outside).unwrap(), b"outside sentinel");
        assert_eq!(
            fs::metadata(&outside).unwrap().permissions().mode() & 0o777,
            0o444
        );
    }

    #[test]
    fn legacy_checkpoint_workspace_path_is_cleaned_with_run_metadata() {
        let (_temp, store) = store();
        let run_id = "legacy-checkpoint-workspace";
        store
            .create_run(run_id, &["codex".into()], "sha", "scope")
            .unwrap();
        assert!(store.mark_cancelled(run_id).unwrap());
        let input = store.run_dir(run_id).unwrap().join("checkpoint-input");
        fs::create_dir(&input).unwrap();
        fs::write(input.join("ordinary.txt"), b"checkpoint").unwrap();
        let mut state = store.load_state(run_id).unwrap().unwrap();
        state.workspace_root = Some(input.display().to_string());
        store.write_state(&state).unwrap();
        assert!(store.clean_run(run_id).unwrap());
        assert!(!input.exists());
    }

    #[test]
    fn run_id_validation_blocks_traversal() {
        let (_dir, store) = store();
        for bad in ["", "../evil", "a/b", "a\\b", "..", "run id"] {
            assert!(
                store.run_dir(bad).is_err(),
                "run id '{bad}' must be rejected"
            );
        }
        assert!(store.run_dir("0e6f0a1c-run_1").is_ok());
        assert_eq!(sanitize_reviewer_name("Claude Code!"), "claude-code-");
        assert_eq!(sanitize_reviewer_name("../x"), "---x");
        assert_eq!(sanitize_reviewer_name(""), "reviewer");
    }
}
