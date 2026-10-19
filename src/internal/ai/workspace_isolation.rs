//! Isolated workspace types and materialization — plan-20260920 RC-03/RC-07/RC-23.
//!
//! KEEP `review` / `investigate` depend on this module. RC-23 deleted the
//! executor SCC (`orchestrator` / `workspace_snapshot`); this seam now
//! materializes a full copy only. Review/investigate already disable FUSE
//! before calling [`materialize_isolated_workspace`].

use std::{
    ffi::{OsStr, OsString},
    fs::{self, File},
    io::{self, Read, Write},
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    internal::ai::agent_run::{AgentRunId, WorkspaceStrategy},
    utils::worktree_layout,
};

/// Physical materialization backend. Copy-only after RC-23.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskWorkspaceBackend {
    Shared,
    Copy,
    Fuse,
}

impl TaskWorkspaceBackend {
    pub fn label(self) -> &'static str {
        match self {
            Self::Shared => "shared workspace",
            Self::Copy => "copy worktree",
            Self::Fuse => "FUSE worktree",
        }
    }
}

/// Per-session FUSE-provisioning flag. Review/investigate still construct
/// this and call [`Self::disable_first_time`]; materialization ignores FUSE.
#[derive(Clone, Debug)]
pub struct FuseProvisionState {
    disabled: Arc<AtomicBool>,
}

impl Default for FuseProvisionState {
    fn default() -> Self {
        Self {
            disabled: Arc::new(AtomicBool::new(true)),
        }
    }
}

impl FuseProvisionState {
    pub fn disable_first_time(&self) -> bool {
        self.disabled
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
    }

    pub fn is_disabled(&self) -> bool {
        self.disabled.load(Ordering::Acquire)
    }
}

/// Isolation settings keep repository storage separate from run metadata.
#[derive(Clone, Debug)]
pub struct WorkspaceIsolationConfig {
    pub fuse_state: FuseProvisionState,
    pub common_storage: PathBuf,
    pub run_dir: PathBuf,
    pub allow_full_copy: bool,
}

const TASK_LAYOUT_VERSION: u32 = 1;
const INTENTS_DIR: &str = "workspace-intents";
const MAX_INTENTS: usize = 4096;
const MAX_DESCRIPTOR_BYTES: u64 = 16 * 1024;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskWorkspaceKind {
    Workspace,
    CheckpointInput,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskWorkspaceState {
    Provisioning,
    Active,
    CleanupPending,
}

/// Private lifecycle record. Public run manifests do not embed this wire format.
/// Paths alone never authorize mutation: repository and attempt identities must match.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TaskWorkspaceDescriptor {
    pub layout_version: u32,
    pub run_id: Uuid,
    pub attempt_id: Uuid,
    pub kind: TaskWorkspaceKind,
    pub backend: TaskWorkspaceBackend,
    pub canonical_path: PathBuf,
    pub state: TaskWorkspaceState,
    common_storage: PathBuf,
    repo_identity: (u64, u64),
    run_identity: (u64, u64),
    attempt_identity: Option<(u64, u64)>,
    workspace_identity: Option<(u64, u64)>,
}

impl TaskWorkspaceDescriptor {
    pub fn attempt_root(&self) -> PathBuf {
        worktree_layout::task_attempt_root(&self.common_storage, self.run_id, self.attempt_id)
    }

    pub fn common_storage(&self) -> &Path {
        &self.common_storage
    }
}

fn refused(message: impl Into<String>) -> io::Error {
    io::Error::new(
        io::ErrorKind::PermissionDenied,
        format!(
            "task workspace ownership could not be verified: {}; keep its run metadata and inspect it before retrying",
            message.into()
        ),
    )
}

fn expected_path(common: &Path, run: Uuid, attempt: Uuid, kind: TaskWorkspaceKind) -> PathBuf {
    match kind {
        TaskWorkspaceKind::Workspace => worktree_layout::task_workspace(common, run, attempt),
        TaskWorkspaceKind::CheckpointInput => {
            worktree_layout::checkpoint_input(common, run, attempt)
        }
    }
}

fn record_name(attempt: Uuid, suffix: &str) -> OsString {
    OsString::from(format!("{attempt}.{suffix}.json"))
}

fn staging_record_name(attempt: Uuid, suffix: &str) -> OsString {
    OsString::from(format!(".{attempt}.{suffix}.tmp-{}", Uuid::new_v4()))
}

fn is_staging_record_name(name: &str) -> bool {
    let Some((prefix, nonce)) = name
        .strip_prefix('.')
        .and_then(|name| name.rsplit_once(".tmp-"))
    else {
        return false;
    };
    let Some((attempt, suffix)) = prefix.rsplit_once('.') else {
        return false;
    };
    Uuid::parse_str(attempt).is_ok()
        && Uuid::parse_str(nonce).is_ok()
        && matches!(suffix, "intent" | "owned" | "active" | "cleanup")
}

fn validate_run(common: &Path, run_dir: &Path, run_id: Uuid) -> io::Result<(PathBuf, File)> {
    let original_common = if common.is_absolute() {
        common.to_path_buf()
    } else {
        std::env::current_dir()?.join(common)
    };
    let common = fs::canonicalize(&original_common)?;
    let suffix = PathBuf::from("sessions")
        .join("agent-runs")
        .join(run_id.to_string());
    let expected = common.join(&suffix);
    // Normalize only the trusted repository anchor (/var is an alias on macOS).
    // Never canonicalize sessions/run components while reading persisted proof.
    let supplied = if run_dir.is_absolute() {
        run_dir.to_path_buf()
    } else {
        std::env::current_dir()?.join(run_dir)
    };
    let supplied_common = supplied
        .parent()
        .filter(|parent| parent.file_name() == Some(OsStr::new("agent-runs")))
        .and_then(Path::parent)
        .filter(|parent| parent.file_name() == Some(OsStr::new("sessions")))
        .and_then(Path::parent);
    // A caller may retain the system spelling /var while its descriptor
    // stores the physical /private/var path. Resolve only the repository
    // anchor; never follow sessions or the run leaf through a symlink.
    let same_anchor = supplied_common
        .and_then(|anchor| fs::canonicalize(anchor).ok())
        .is_some_and(|anchor| anchor == common);
    if supplied.file_name() != Some(OsStr::new(&run_id.to_string())) || !same_anchor {
        return Err(refused(
            "run metadata is not under the pinned repository's sessions/agent-runs",
        ));
    }
    let file = owned_fs::open_directory(&expected)?;
    Ok((common, file))
}

fn lifecycle_lock(run: &File) -> io::Result<File> {
    let file = match owned_fs::create_file(run, OsStr::new(".workspace.lock")) {
        Ok(file) => file,
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
            owned_fs::open_file(run, OsStr::new(".workspace.lock"))?
        }
        Err(error) => return Err(error),
    };
    file.try_lock().map_err(|error| match error {
        fs::TryLockError::WouldBlock => io::Error::new(
            io::ErrorKind::WouldBlock,
            "this run's workspace lifecycle is busy; retry after its current operation finishes",
        ),
        fs::TryLockError::Error(error) => error,
    })?;
    Ok(file)
}

fn open_intents(run: &File, create: bool) -> io::Result<Option<File>> {
    match owned_fs::open_child_directory(run, OsStr::new(INTENTS_DIR)) {
        Ok(file) => Ok(Some(file)),
        Err(error) if error.kind() == io::ErrorKind::NotFound && create => {
            let dir = owned_fs::create_directory(run, OsStr::new(INTENTS_DIR))?;
            owned_fs::sync_directory(run)?;
            Ok(Some(dir))
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error),
    }
}

fn write_record(
    intents: &File,
    suffix: &str,
    descriptor: &TaskWorkspaceDescriptor,
) -> io::Result<()> {
    let bytes = serde_json::to_vec(descriptor).map_err(io::Error::other)?;
    let staging = staging_record_name(descriptor.attempt_id, suffix);
    let mut file = owned_fs::create_file(intents, &staging)?;
    let identity = owned_fs::identity(&file)?;
    let written = file.write_all(&bytes).and_then(|_| file.sync_all());
    drop(file);
    if let Err(error) = written {
        let _ = owned_fs::remove_entry(intents, &staging, identity, false);
        return Err(error);
    }
    if let Err(error) = owned_fs::publish_file(
        intents,
        &staging,
        &record_name(descriptor.attempt_id, suffix),
        identity,
    ) {
        let _ = owned_fs::remove_entry(intents, &staging, identity, false);
        return Err(error);
    }
    owned_fs::sync_directory(intents)
}

fn read_record(intents: &File, attempt: Uuid, suffix: &str) -> io::Result<TaskWorkspaceDescriptor> {
    let file = owned_fs::open_file(intents, &record_name(attempt, suffix))?;
    let mut bytes = Vec::new();
    file.take(MAX_DESCRIPTOR_BYTES + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MAX_DESCRIPTOR_BYTES {
        return Err(refused("workspace descriptor exceeds its size limit"));
    }
    serde_json::from_slice(&bytes)
        .map_err(|error| refused(format!("workspace descriptor is invalid: {error}")))
}

fn validate_descriptor(
    common: &Path,
    run_id: Uuid,
    descriptor: &TaskWorkspaceDescriptor,
) -> io::Result<()> {
    if descriptor.layout_version != TASK_LAYOUT_VERSION {
        return Err(refused(format!(
            "unsupported workspace layout version {}",
            descriptor.layout_version
        )));
    }
    if descriptor.common_storage != common
        || descriptor.run_id != run_id
        || descriptor.canonical_path
            != expected_path(common, run_id, descriptor.attempt_id, descriptor.kind)
        || descriptor.repo_identity != owned_fs::identity(&owned_fs::open_directory(common)?)?
    {
        return Err(refused(
            "workspace descriptor does not belong to this repository and run",
        ));
    }
    Ok(())
}

/// Validate persisted descriptors even after their physical workspace was cleaned.
pub fn validate_task_workspace_descriptor(
    common: &Path,
    run_id: Uuid,
    descriptor: &TaskWorkspaceDescriptor,
) -> io::Result<()> {
    validate_descriptor(&fs::canonicalize(common)?, run_id, descriptor)
}

/// Persist intent before exclusive allocation. An interrupted allocation without
/// an ownership proof is reported, never adopted or overwritten.
pub fn create_task_workspace(
    common_storage: &Path,
    run_dir: &Path,
    run_id: Uuid,
    kind: TaskWorkspaceKind,
) -> io::Result<TaskWorkspaceDescriptor> {
    create_task_workspace_inner(common_storage, run_dir, run_id, kind, None)
}

/// Preserve a caller's already-pinned metadata run identity across allocation.
pub fn create_task_workspace_for_run_handle(
    common_storage: &Path,
    run_dir: &Path,
    run_id: Uuid,
    kind: TaskWorkspaceKind,
    pinned_run: &File,
) -> io::Result<TaskWorkspaceDescriptor> {
    create_task_workspace_inner(common_storage, run_dir, run_id, kind, Some(pinned_run))
}

fn create_task_workspace_inner(
    common_storage: &Path,
    run_dir: &Path,
    run_id: Uuid,
    kind: TaskWorkspaceKind,
    pinned_run: Option<&File>,
) -> io::Result<TaskWorkspaceDescriptor> {
    let (common, run) = validate_run(common_storage, run_dir, run_id)?;
    if let Some(pinned) = pinned_run
        && owned_fs::identity(pinned)? != owned_fs::identity(&run)?
    {
        return Err(refused("metadata run changed before workspace allocation"));
    }
    let _lock = lifecycle_lock(&run)?;
    let intents =
        open_intents(&run, true)?.ok_or_else(|| refused("intent directory disappeared"))?;
    if owned_fs::names(&intents)?.len() >= MAX_INTENTS * 3 {
        return Err(refused(
            "too many retained workspace intents; clean completed attempts first",
        ));
    }
    let attempt_id = Uuid::new_v4();
    let mut descriptor = TaskWorkspaceDescriptor {
        layout_version: TASK_LAYOUT_VERSION,
        run_id,
        attempt_id,
        kind,
        backend: TaskWorkspaceBackend::Copy,
        canonical_path: expected_path(&common, run_id, attempt_id, kind),
        state: TaskWorkspaceState::Provisioning,
        repo_identity: owned_fs::identity(&owned_fs::open_directory(&common)?)?,
        run_identity: owned_fs::identity(&run)?,
        common_storage: common.clone(),
        attempt_identity: None,
        workspace_identity: None,
    };
    write_record(&intents, "intent", &descriptor)?;
    let attempt_path = descriptor.attempt_root();
    let parent_path = attempt_path
        .parent()
        .ok_or_else(|| refused("attempt has no parent"))?;
    worktree_layout::ensure_managed_directory(&common, parent_path)?;
    let parent = worktree_layout::open_managed_directory(&common, parent_path)?;
    let attempt = owned_fs::create_directory(&parent, OsStr::new(&attempt_id.to_string()))?;
    owned_fs::sync_directory(&parent)?;
    descriptor.attempt_identity = Some(owned_fs::identity(&attempt)?);
    // Publish the inode proof before touching children. A partial provision is
    // consequently recoverable even if materialization fails before Active.
    if let Err(error) = write_record(&intents, "owned", &descriptor) {
        let identity = descriptor
            .attempt_identity
            .ok_or_else(|| refused("new attempt lost its directory identity"))?;
        drop(attempt);
        owned_fs::remove_entry(&parent, OsStr::new(&attempt_id.to_string()), identity, true)
            .map_err(|cleanup| {
                io::Error::other(format!(
                    "cannot publish workspace ownership proof: {error}; cannot remove the empty new attempt: {cleanup}; retain its creation intent for doctor"
                ))
            })?;
        owned_fs::sync_directory(&parent)?;
        let intent_name = record_name(attempt_id, "intent");
        let intent = owned_fs::entry(&intents, &intent_name)?;
        owned_fs::remove_entry(&intents, &intent_name, intent.identity, false)?;
        owned_fs::sync_directory(&intents)?;
        return Err(error);
    }
    for name in [
        match kind {
            TaskWorkspaceKind::Workspace => "workspace",
            TaskWorkspaceKind::CheckpointInput => "checkpoint-input",
        },
        "home",
        "xdg-config",
        "xdg-cache",
        "cargo-home",
        "logs",
        "target",
    ] {
        owned_fs::create_directory(&attempt, OsStr::new(name))?;
    }
    owned_fs::sync_directory(&attempt)?;
    descriptor.workspace_identity = Some(owned_fs::identity(&owned_fs::open_child_directory(
        &attempt,
        descriptor
            .canonical_path
            .file_name()
            .ok_or_else(|| refused("workspace has no leaf name"))?,
    )?)?);
    descriptor.state = TaskWorkspaceState::Active;
    write_record(&intents, "active", &descriptor)?;
    Ok(descriptor)
}

/// Verify the authoritative private descriptor and its current directory identity.
pub fn verify_task_workspace(
    common: &Path,
    run_dir: &Path,
    descriptor: &TaskWorkspaceDescriptor,
) -> io::Result<()> {
    let (common, run) = validate_run(common, run_dir, descriptor.run_id)?;
    validate_descriptor(&common, descriptor.run_id, descriptor)?;
    if descriptor.run_identity != owned_fs::identity(&run)? {
        return Err(refused("metadata run directory was replaced"));
    }
    let intents =
        open_intents(&run, false)?.ok_or_else(|| refused("workspace has no creation intent"))?;
    let active = read_record(&intents, descriptor.attempt_id, "active")?;
    if &active != descriptor {
        return Err(refused(
            "workspace descriptor differs from its durable active record",
        ));
    }
    let attempt = verify_attempt(&common, &intents, descriptor)?;
    let leaf = owned_fs::open_child_directory(
        &attempt,
        descriptor
            .canonical_path
            .file_name()
            .ok_or_else(|| refused("workspace has no leaf name"))?,
    )?;
    if Some(owned_fs::identity(&leaf)?) != descriptor.workspace_identity {
        return Err(refused("workspace leaf was replaced after allocation"));
    }
    Ok(())
}

/// Bind a caller's already-open no-follow workspace handle to the durable proof.
pub fn verify_task_workspace_handle(
    common: &Path,
    run_dir: &Path,
    descriptor: &TaskWorkspaceDescriptor,
    workspace: &File,
) -> io::Result<()> {
    verify_task_workspace(common, run_dir, descriptor)?;
    if Some(owned_fs::identity(workspace)?) != descriptor.workspace_identity {
        return Err(refused(
            "opened workspace handle differs from its allocated identity",
        ));
    }
    Ok(())
}

fn verify_attempt(
    common: &Path,
    intents: &File,
    descriptor: &TaskWorkspaceDescriptor,
) -> io::Result<File> {
    let owned = read_record(intents, descriptor.attempt_id, "owned")?;
    validate_descriptor(common, descriptor.run_id, &owned)?;
    if owned.attempt_id != descriptor.attempt_id
        || owned.kind != descriptor.kind
        || owned.attempt_identity != descriptor.attempt_identity
        || owned.attempt_identity.is_none()
    {
        return Err(refused(
            "attempt inode proof does not match the requested workspace",
        ));
    }
    let attempt = worktree_layout::open_managed_directory(common, &descriptor.attempt_root())?;
    if Some(owned_fs::identity(&attempt)?) != descriptor.attempt_identity {
        return Err(refused("attempt directory was replaced after allocation"));
    }
    Ok(attempt)
}

fn intent_ids(intents: &File) -> io::Result<Vec<Uuid>> {
    let mut budget = MAX_INTENTS * 4;
    intent_ids_bounded(intents, &mut budget)
}

fn intent_ids_bounded(intents: &File, budget: &mut usize) -> io::Result<Vec<Uuid>> {
    if *budget == 0 {
        return Err(refused(
            "workspace intent scan has no remaining entry budget",
        ));
    }
    let names = owned_fs::names_bounded(intents, (*budget).min(MAX_INTENTS * 4))?;
    *budget -= names.len();
    let mut ids = Vec::new();
    let mut proof_ids = Vec::new();
    for name in names {
        let name = name
            .to_str()
            .ok_or_else(|| refused("non-UTF-8 workspace intent filename"))?;
        if is_staging_record_name(name) {
            // A crash may leave only a private staging file. It never grants
            // ownership and must not poison cleanup of other attempts.
            continue;
        }
        let (id, is_intent) = if let Some(id) = name.strip_suffix(".intent.json") {
            (id, true)
        } else if let Some(id) = [".owned.json", ".active.json", ".cleanup.json"]
            .iter()
            .find_map(|suffix| name.strip_suffix(suffix))
        {
            (id, false)
        } else {
            return Err(refused("unknown file in workspace intent directory"));
        };
        let id = Uuid::parse_str(id)
            .map_err(|_| refused("workspace intent or proof filename is not an attempt UUID"))?;
        if is_intent {
            ids.push(id);
        } else {
            proof_ids.push(id);
        }
    }
    if ids.len() > MAX_INTENTS {
        return Err(refused("too many retained workspace attempts"));
    }
    for id in proof_ids {
        if !ids.contains(&id) {
            return Err(refused(
                "workspace allocation proof has no durable creation intent",
            ));
        }
    }
    ids.sort_unstable();
    Ok(ids)
}

pub fn has_task_workspace_intents(run_dir: &Path) -> io::Result<bool> {
    let absolute = if run_dir.is_absolute() {
        run_dir.to_path_buf()
    } else {
        std::env::current_dir()?.join(run_dir)
    };
    let normalized = match absolute
        .parent()
        .and_then(Path::parent)
        .and_then(Path::parent)
    {
        Some(common)
            if absolute.parent().and_then(Path::file_name) == Some(OsStr::new("agent-runs"))
                && absolute
                    .parent()
                    .and_then(Path::parent)
                    .and_then(Path::file_name)
                    == Some(OsStr::new("sessions")) =>
        {
            let suffix = absolute
                .strip_prefix(common)
                .map_err(|_| refused("run path has an invalid metadata anchor"))?;
            fs::canonicalize(common)?.join(suffix)
        }
        _ => absolute,
    };
    let run = owned_fs::open_directory(&normalized)?;
    match open_intents(&run, false)? {
        Some(intents) => Ok(!intent_ids(&intents)?.is_empty()),
        None => Ok(false),
    }
}

/// Read-only inventory for doctor and lifecycle consumers. Incomplete allocation
/// remains an error with its intent preserved for operator inspection.
pub fn read_task_workspace_descriptors(
    common: &Path,
    run_dir: &Path,
    run_id: Uuid,
) -> io::Result<Vec<TaskWorkspaceDescriptor>> {
    let mut budget = MAX_INTENTS * 4;
    read_task_workspace_descriptors_bounded(common, run_dir, run_id, &mut budget)
}

/// The doctor path charges each intent-directory entry against its aggregate
/// scan budget before reading any proof; an oversized directory fails closed.
pub fn read_task_workspace_descriptors_bounded(
    common: &Path,
    run_dir: &Path,
    run_id: Uuid,
    budget: &mut usize,
) -> io::Result<Vec<TaskWorkspaceDescriptor>> {
    let (common, run) = validate_run(common, run_dir, run_id)?;
    let Some(intents) = open_intents(&run, false)? else {
        return Ok(Vec::new());
    };
    let mut descriptors = Vec::new();
    for id in intent_ids_bounded(&intents, budget)? {
        let intent = read_record(&intents, id, "intent")?;
        validate_descriptor(&common, run_id, &intent)?;
        if intent.run_identity != owned_fs::identity(&run)? {
            return Err(refused("metadata run directory was replaced"));
        }
        let descriptor = match read_record(&intents, id, "cleanup").or_else(|error| {
            if error.kind() == io::ErrorKind::NotFound {
                read_record(&intents, id, "active")
            } else {
                Err(error)
            }
        }) {
            Ok(active) => active,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                read_record(&intents, id, "owned")?
            }
            Err(error) => return Err(error),
        };
        validate_descriptor(&common, run_id, &descriptor)?;
        verify_attempt(&common, &intents, &descriptor)?;
        descriptors.push(descriptor);
    }
    Ok(descriptors)
}

/// Clean every owned attempt before the caller removes run metadata. Failures
/// retain all outstanding intent/proof records, allowing a later retry.
pub fn cleanup_task_workspaces(common: &Path, run_dir: &Path, run_id: Uuid) -> io::Result<()> {
    cleanup_task_workspaces_until(
        common,
        run_dir,
        run_id,
        Instant::now() + Duration::from_secs(30),
    )
}

/// The caller's deadline covers the whole cleanup, including every attempt.
pub fn cleanup_task_workspaces_until(
    common: &Path,
    run_dir: &Path,
    run_id: Uuid,
    deadline: Instant,
) -> io::Result<()> {
    if Instant::now() >= deadline {
        return Err(io::Error::new(
            io::ErrorKind::TimedOut,
            "workspace cleanup deadline already expired; keep run metadata and retry",
        ));
    }
    let (common, run) = validate_run(common, run_dir, run_id)?;
    let _lock = lifecycle_lock(&run)?;
    let Some(intents) = open_intents(&run, false)? else {
        return Ok(());
    };
    let mut budget = CleanupBudget {
        deadline,
        remaining: 1_000_000,
    };
    let run_identity = owned_fs::identity(&run)?;
    let mut first_error = None;
    let mut failures = 0usize;
    for id in intent_ids(&intents)? {
        if let Err(error) =
            cleanup_attempt(&common, &intents, run_id, id, run_identity, &mut budget)
        {
            failures += 1;
            if first_error.is_none() {
                first_error = Some(error);
            }
        }
    }
    if let Some(error) = first_error {
        return Err(io::Error::new(
            error.kind(),
            format!(
                "{failures} task workspace attempt(s) could not be cleaned; first refusal: {error}"
            ),
        ));
    }
    Ok(())
}

pub fn cleanup_task_workspace(
    common: &Path,
    run_dir: &Path,
    descriptor: &TaskWorkspaceDescriptor,
) -> io::Result<()> {
    let (common, run) = validate_run(common, run_dir, descriptor.run_id)?;
    validate_descriptor(&common, descriptor.run_id, descriptor)?;
    let _lock = lifecycle_lock(&run)?;
    let Some(intents) = open_intents(&run, false)? else {
        return Ok(());
    };
    cleanup_attempt(
        &common,
        &intents,
        descriptor.run_id,
        descriptor.attempt_id,
        owned_fs::identity(&run)?,
        &mut CleanupBudget {
            deadline: Instant::now() + Duration::from_secs(30),
            remaining: 1_000_000,
        },
    )
}

fn cleanup_attempt(
    common: &Path,
    intents: &File,
    run_id: Uuid,
    id: Uuid,
    run_identity: (u64, u64),
    budget: &mut CleanupBudget,
) -> io::Result<()> {
    if Instant::now() >= budget.deadline {
        return Err(io::Error::new(
            io::ErrorKind::TimedOut,
            "workspace cleanup deadline reached; keep run metadata and retry",
        ));
    }
    let intent = match read_record(intents, id, "intent") {
        Ok(intent) => intent,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error),
    };
    validate_descriptor(common, run_id, &intent)?;
    if intent.run_identity != run_identity {
        return Err(refused("metadata run directory was replaced"));
    }
    if intent.attempt_id != id {
        return Err(refused("intent filename and attempt identity disagree"));
    }
    for suffix in ["owned", "active", "cleanup"] {
        match read_record(intents, id, suffix) {
            Ok(record) => {
                validate_descriptor(common, run_id, &record)?;
                if record.attempt_id != id || record.kind != intent.kind {
                    return Err(refused("workspace lifecycle records disagree"));
                }
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(error),
        }
    }
    let attempt_path = intent.attempt_root();
    match worktree_layout::open_managed_directory(common, &attempt_path) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(error),
        Ok(attempt) => {
            let mut owned = read_record(intents, id, "owned").map_err(|error| {
                refused(format!(
                    "attempt {id} exists without a readable allocation proof: {error}"
                ))
            })?;
            validate_descriptor(common, run_id, &owned)?;
            if owned.attempt_id != id
                || owned.kind != intent.kind
                || owned.attempt_identity != Some(owned_fs::identity(&attempt)?)
            {
                return Err(refused(
                    "attempt directory is not the inode allocated by this run",
                ));
            }
            let cleanup_started = read_record(intents, id, "cleanup").is_ok();
            match read_record(intents, id, "active") {
                Ok(active) => {
                    let leaf_name = active
                        .canonical_path
                        .file_name()
                        .ok_or_else(|| refused("workspace has no leaf name"))?;
                    match owned_fs::entry(&attempt, leaf_name) {
                        Ok(leaf)
                            if Some(leaf.identity) == active.workspace_identity
                                && leaf.directory
                                && !leaf.symlink => {}
                        Err(error)
                            if error.kind() == io::ErrorKind::NotFound && cleanup_started => {}
                        _ => return Err(refused("workspace leaf identity changed before cleanup")),
                    }
                }
                Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                Err(error) => return Err(error),
            }
            owned.state = TaskWorkspaceState::CleanupPending;
            match write_record(intents, "cleanup", &owned) {
                Ok(()) => {}
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
                    if read_record(intents, id, "cleanup")? != owned {
                        return Err(refused("cleanup intent has changed"));
                    }
                }
                Err(error) => return Err(error),
            }
            let parent_path = attempt_path
                .parent()
                .ok_or_else(|| refused("attempt has no parent"))?;
            let parent = worktree_layout::open_managed_directory(common, parent_path)?;
            let identity = owned_fs::identity(&attempt)?;
            let mut verify =
                || -> io::Result<()> { verify_attempt(common, intents, &owned).map(|_| ()) };
            clear_directory(&attempt, budget, 0, &mut verify)?;
            verify()?;
            drop(attempt);
            owned_fs::remove_entry(&parent, OsStr::new(&id.to_string()), identity, true)?;
            owned_fs::sync_directory(&parent)?;
        }
    }
    // Intent is removed last: interruption at any earlier point is retryable.
    for suffix in ["active", "cleanup", "owned", "intent"] {
        let name = record_name(id, suffix);
        match owned_fs::entry(intents, &name) {
            Ok(entry) => owned_fs::remove_entry(intents, &name, entry.identity, false)?,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(error),
        }
    }
    owned_fs::sync_directory(intents)
}

struct CleanupBudget {
    deadline: Instant,
    remaining: usize,
}

fn clear_directory(
    directory: &File,
    budget: &mut CleanupBudget,
    depth: usize,
    verify: &mut dyn FnMut() -> io::Result<()>,
) -> io::Result<()> {
    if depth > 128 || budget.remaining == 0 || Instant::now() >= budget.deadline {
        return Err(io::Error::new(
            io::ErrorKind::TimedOut,
            "workspace cleanup reached its traversal budget; retained intent permits a bounded retry",
        ));
    }
    verify()?;
    owned_fs::make_writable(directory)?;
    for name in owned_fs::names(directory)? {
        if budget.remaining == 0 || Instant::now() >= budget.deadline {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "workspace cleanup deadline reached; retry cleanup",
            ));
        }
        budget.remaining -= 1;
        let entry = owned_fs::entry(directory, &name)?;
        if entry.directory && !entry.symlink {
            let child = owned_fs::open_child_directory(directory, &name)?;
            if owned_fs::identity(&child)? != entry.identity {
                return Err(refused("workspace child changed while opening it"));
            }
            clear_directory(&child, budget, depth + 1, verify)?;
            drop(child);
        }
        verify()?;
        owned_fs::remove_entry(directory, &name, entry.identity, entry.directory)?;
    }
    Ok(())
}

/// Sandbox lookup proves the candidate against its private run record and inode,
/// rather than trusting a directory prefix supplied by the command's cwd.
pub fn validated_task_attempt_root(cwd: &Path) -> io::Result<Option<PathBuf>> {
    let canonical = fs::canonicalize(cwd)?;
    for leaf in canonical.ancestors() {
        let kind = match leaf.file_name().and_then(OsStr::to_str) {
            Some("workspace") => TaskWorkspaceKind::Workspace,
            Some("checkpoint-input") => TaskWorkspaceKind::CheckpointInput,
            _ => continue,
        };
        let Some(attempt) = leaf.parent() else {
            continue;
        };
        let Some(run) = attempt.parent() else {
            continue;
        };
        let Some(tasks) = run.parent() else {
            continue;
        };
        let Some(container) = tasks.parent() else {
            continue;
        };
        let Some(common) = container.parent() else {
            continue;
        };
        if tasks.file_name() != Some(OsStr::new("tasks"))
            || container.file_name() != Some(OsStr::new("worktree"))
        {
            continue;
        }
        let (Some(run_name), Some(attempt_name)) = (
            run.file_name().and_then(OsStr::to_str),
            attempt.file_name().and_then(OsStr::to_str),
        ) else {
            continue;
        };
        let (Ok(run_id), Ok(attempt_id)) =
            (Uuid::parse_str(run_name), Uuid::parse_str(attempt_name))
        else {
            return Err(refused("task path has invalid run or attempt identity"));
        };
        let metadata = common
            .join("sessions")
            .join("agent-runs")
            .join(run_id.to_string());
        let (_, metadata_handle) = validate_run(common, &metadata, run_id)?;
        let intents = open_intents(&metadata_handle, false)?
            .ok_or_else(|| refused("task path has no durable intent"))?;
        let descriptor = read_record(&intents, attempt_id, "active")?;
        if descriptor.kind != kind || descriptor.canonical_path != leaf {
            return Err(refused(
                "sandbox workspace differs from its owned descriptor",
            ));
        }
        verify_task_workspace(common, &metadata, &descriptor)?;
        return Ok(Some(attempt.to_path_buf()));
    }
    Ok(None)
}

pub struct SubAgentWorkspace {
    descriptor: TaskWorkspaceDescriptor,
    run_dir: PathBuf,
    strategy: WorkspaceStrategy,
}

impl SubAgentWorkspace {
    pub fn root(&self) -> &Path {
        &self.descriptor.canonical_path
    }
    pub fn descriptor(&self) -> &TaskWorkspaceDescriptor {
        &self.descriptor
    }
    pub fn strategy(&self) -> WorkspaceStrategy {
        self.strategy
    }
    pub fn backend(&self) -> TaskWorkspaceBackend {
        TaskWorkspaceBackend::Copy
    }
    pub fn cleanup(self) -> io::Result<()> {
        cleanup_task_workspace(
            &self.descriptor.common_storage,
            &self.run_dir,
            &self.descriptor,
        )
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SubAgentWorkspaceError {
    #[error("full-copy workspace materialization is not permitted")]
    Unavailable,
    #[error("failed to materialize isolated workspace: {0}")]
    Io(#[from] io::Error),
}

pub fn materialize_isolated_workspace(
    main_working_dir: &Path,
    _thread_id: Uuid,
    agent_run_id: AgentRunId,
    isolation: &WorkspaceIsolationConfig,
) -> Result<SubAgentWorkspace, SubAgentWorkspaceError> {
    let _ = isolation.fuse_state.is_disabled();
    if !isolation.allow_full_copy {
        return Err(SubAgentWorkspaceError::Unavailable);
    }
    let descriptor = create_task_workspace(
        &isolation.common_storage,
        &isolation.run_dir,
        agent_run_id.0,
        TaskWorkspaceKind::Workspace,
    )?;
    if let Err(error) = copy_tree(
        main_working_dir,
        &descriptor.canonical_path,
        main_working_dir,
        &descriptor.canonical_path,
    ) {
        // Keep the durable intent if cleanup itself fails; it is the recovery root.
        if let Err(cleanup_error) =
            cleanup_task_workspace(&isolation.common_storage, &isolation.run_dir, &descriptor)
        {
            return Err(io::Error::other(format!("workspace copy failed: {error}; cleanup failed and retained recovery intent: {cleanup_error}")).into());
        }
        return Err(error.into());
    }
    verify_task_workspace(&isolation.common_storage, &isolation.run_dir, &descriptor)?;
    Ok(SubAgentWorkspace {
        descriptor,
        run_dir: isolation.run_dir.clone(),
        strategy: WorkspaceStrategy::FullCopy,
    })
}
fn copy_tree(src: &Path, dest: &Path, _src_root: &Path, dest_root: &Path) -> io::Result<()> {
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let name = entry.file_name();
        if name == ".libra" {
            continue;
        }
        let from = entry.path();
        if from.starts_with(dest_root) {
            continue;
        }
        let to = dest.join(&name);
        let file_type = entry.file_type()?;
        #[cfg(windows)]
        {
            use std::os::windows::fs::MetadataExt;

            // Junctions and other reparse points are not all reported as
            // symlinks. Never follow one while copying into an agent workspace.
            const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;
            if fs::symlink_metadata(&from)?.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
                return Err(io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    format!(
                        "cannot copy Windows reparse point '{}' into a task workspace; remove or replace the link before retrying",
                        from.display()
                    ),
                ));
            }
        }
        if file_type.is_dir() {
            fs::create_dir_all(&to)?;
            copy_tree(&from, &to, _src_root, dest_root)?;
        } else if file_type.is_symlink() {
            #[cfg(unix)]
            {
                let target = fs::read_link(&from)?;
                std::os::unix::fs::symlink(target, &to)?;
            }
            #[cfg(not(unix))]
            {
                if from.is_dir() {
                    fs::create_dir_all(&to)?;
                    copy_tree(&from, &to, _src_root, dest_root)?;
                } else {
                    fs::copy(&from, &to)?;
                }
            }
        } else if file_type.is_file() {
            fs::copy(&from, &to)?;
        }
    }
    Ok(())
}

/// Filesystem capability operations shared by creation, proof reads and cleanup.
mod owned_fs {
    use super::*;

    pub struct Entry {
        pub identity: (u64, u64),
        pub directory: bool,
        pub symlink: bool,
    }

    #[cfg(unix)]
    mod platform {
        use std::{
            ffi::{CStr, CString},
            os::{
                fd::{AsRawFd, FromRawFd},
                unix::{
                    ffi::{OsStrExt, OsStringExt},
                    fs::MetadataExt,
                },
            },
        };

        use super::*;

        fn name(value: &OsStr) -> io::Result<CString> {
            if value.as_bytes().is_empty()
                || value == OsStr::new(".")
                || value == OsStr::new("..")
                || value.as_bytes().contains(&b'/')
            {
                return Err(refused(
                    "filesystem child name must be one normal component",
                ));
            }
            CString::new(value.as_bytes())
                .map_err(|_| refused("filesystem child name contains NUL"))
        }

        fn open_at(parent: &File, value: &OsStr, flags: libc::c_int) -> io::Result<File> {
            let name = name(value)?;
            // SAFETY: parent and the terminated component remain live. O_NOFOLLOW
            // applies to every component because callers descend one at a time.
            let fd = unsafe {
                libc::openat(
                    parent.as_raw_fd(),
                    name.as_ptr(),
                    flags | libc::O_NOFOLLOW | libc::O_CLOEXEC,
                    0o600,
                )
            };
            if fd < 0 {
                return Err(io::Error::last_os_error());
            }
            // SAFETY: openat returned a fresh owned descriptor.
            Ok(unsafe { File::from_raw_fd(fd) })
        }

        pub fn open_directory(path: &Path) -> io::Result<File> {
            if !path.is_absolute() {
                return Err(refused("filesystem anchor must be absolute"));
            }
            let mut dir = File::open("/")?;
            for component in path.components() {
                match component {
                    std::path::Component::RootDir => {}
                    std::path::Component::Normal(name) => dir = open_child_directory(&dir, name)?,
                    _ => return Err(refused("filesystem anchor contains a non-normal component")),
                }
            }
            Ok(dir)
        }

        pub fn open_child_directory(parent: &File, value: &OsStr) -> io::Result<File> {
            open_at(parent, value, libc::O_RDONLY | libc::O_DIRECTORY)
        }
        pub fn create_directory(parent: &File, value: &OsStr) -> io::Result<File> {
            let component = name(value)?;
            // SAFETY: the directory capability and single component are live.
            if unsafe { libc::mkdirat(parent.as_raw_fd(), component.as_ptr(), 0o700) } != 0 {
                return Err(io::Error::last_os_error());
            }
            open_child_directory(parent, value)
        }
        pub fn create_file(parent: &File, value: &OsStr) -> io::Result<File> {
            open_at(parent, value, libc::O_RDWR | libc::O_CREAT | libc::O_EXCL)
        }
        pub fn publish_file(
            parent: &File,
            staged: &OsStr,
            final_name: &OsStr,
            expected: (u64, u64),
        ) -> io::Result<()> {
            if entry(parent, staged)?.identity != expected {
                return Err(refused(
                    "workspace proof staging file changed before publication",
                ));
            }
            let source = name(staged)?;
            let destination = name(final_name)?;
            #[cfg(any(target_os = "linux", target_os = "android"))]
            // SAFETY: the parent descriptor and both terminated leaf names are
            // live. RENAME_NOREPLACE atomically publishes only a new name.
            let result = unsafe {
                libc::renameat2(
                    parent.as_raw_fd(),
                    source.as_ptr(),
                    parent.as_raw_fd(),
                    destination.as_ptr(),
                    libc::RENAME_NOREPLACE,
                )
            };
            #[cfg(any(target_os = "macos", target_os = "ios"))]
            // SAFETY: same held-parent contract; RENAME_EXCL forbids replacement.
            let result = unsafe {
                libc::renameatx_np(
                    parent.as_raw_fd(),
                    source.as_ptr(),
                    parent.as_raw_fd(),
                    destination.as_ptr(),
                    libc::RENAME_EXCL,
                )
            };
            #[cfg(not(any(
                target_os = "linux",
                target_os = "android",
                target_os = "macos",
                target_os = "ios"
            )))]
            {
                // linkat is no-replace on Unix targets without an exclusive
                // rename primitive. A stale staging link remains harmless and
                // is ignored by the bounded intent inventory after a crash.
                // SAFETY: the held parent and leaf components stay live.
                if unsafe {
                    libc::linkat(
                        parent.as_raw_fd(),
                        source.as_ptr(),
                        parent.as_raw_fd(),
                        destination.as_ptr(),
                        0,
                    )
                } != 0
                {
                    return Err(io::Error::last_os_error());
                }
                // SAFETY: the held parent and staged component stay live.
                if unsafe { libc::unlinkat(parent.as_raw_fd(), source.as_ptr(), 0) } != 0 {
                    let error = io::Error::last_os_error();
                    // SAFETY: best-effort reversal affects only the newly
                    // linked leaf under the same pinned parent.
                    unsafe { libc::unlinkat(parent.as_raw_fd(), destination.as_ptr(), 0) };
                    return Err(error);
                }
                return Ok(());
            }
            #[cfg(any(
                target_os = "linux",
                target_os = "android",
                target_os = "macos",
                target_os = "ios"
            ))]
            {
                if result != 0 {
                    return Err(io::Error::last_os_error());
                }
                Ok(())
            }
        }
        pub fn open_file(parent: &File, value: &OsStr) -> io::Result<File> {
            let file = open_at(parent, value, libc::O_RDONLY | libc::O_NONBLOCK)?;
            let metadata = file.metadata()?;
            if !metadata.is_file() || metadata.nlink() != 1 {
                return Err(refused(
                    "workspace proof must be an ordinary file with one link",
                ));
            }
            Ok(file)
        }
        pub fn identity(file: &File) -> io::Result<(u64, u64)> {
            let metadata = file.metadata()?;
            Ok((metadata.dev(), metadata.ino()))
        }
        pub fn entry(parent: &File, value: &OsStr) -> io::Result<Entry> {
            let component = name(value)?;
            // SAFETY: fstatat initializes the provided stat on success and never
            // follows the inspected leaf. The parent capability stays open.
            let mut stat: libc::stat = unsafe { std::mem::zeroed() };
            if unsafe {
                libc::fstatat(
                    parent.as_raw_fd(),
                    component.as_ptr(),
                    &mut stat,
                    libc::AT_SYMLINK_NOFOLLOW,
                )
            } != 0
            {
                return Err(io::Error::last_os_error());
            }
            Ok(Entry {
                identity: (stat.st_dev as u64, stat.st_ino),
                directory: stat.st_mode & libc::S_IFMT == libc::S_IFDIR,
                symlink: stat.st_mode & libc::S_IFMT == libc::S_IFLNK,
            })
        }
        pub fn names(directory: &File) -> io::Result<Vec<OsString>> {
            names_bounded(directory, 1_000_000)
        }
        pub fn names_bounded(directory: &File, max_entries: usize) -> io::Result<Vec<OsString>> {
            // An independent open file description avoids sharing a directory
            // cursor between scans (dup alone shares the underlying offset).
            let dot = CString::new(".").map_err(io::Error::other)?;
            // SAFETY: directory is live and dot is a terminated fixed component.
            let fd = unsafe {
                libc::openat(
                    directory.as_raw_fd(),
                    dot.as_ptr(),
                    libc::O_RDONLY | libc::O_DIRECTORY | libc::O_CLOEXEC,
                )
            };
            if fd < 0 {
                return Err(io::Error::last_os_error());
            }
            // SAFETY: fdopendir takes ownership of fd on success.
            let raw = unsafe { libc::fdopendir(fd) };
            if raw.is_null() {
                let error = io::Error::last_os_error();
                // SAFETY: failed fdopendir left ownership with this function.
                unsafe {
                    libc::close(fd);
                }
                return Err(error);
            }
            struct Directory(*mut libc::DIR);
            impl Drop for Directory {
                fn drop(&mut self) {
                    // SAFETY: fdopendir transferred the live stream to this
                    // guard, and Drop runs exactly once.
                    unsafe {
                        libc::closedir(self.0);
                    }
                }
            }
            let stream = Directory(raw);
            let mut result = Vec::new();
            loop {
                errno::set_errno(errno::Errno(0));
                // SAFETY: stream is live; readdir's buffer is copied before next call.
                let item = unsafe { libc::readdir(stream.0) };
                if item.is_null() {
                    let error = errno::errno();
                    if error.0 != 0 {
                        return Err(io::Error::from_raw_os_error(error.0));
                    }
                    break;
                }
                // SAFETY: the non-null readdir result remains owned by this
                // live stream until its next read; copy the name immediately.
                let bytes = unsafe { CStr::from_ptr((*item).d_name.as_ptr()) }.to_bytes();
                if bytes == b"." || bytes == b".." {
                    continue;
                }
                if result.len() >= max_entries {
                    return Err(refused("directory exceeds the remaining entry limit"));
                }
                result.push(OsString::from_vec(bytes.to_vec()));
            }
            Ok(result)
        }
        pub fn remove_entry(
            parent: &File,
            value: &OsStr,
            expected: (u64, u64),
            directory: bool,
        ) -> io::Result<()> {
            if entry(parent, value)?.identity != expected {
                return Err(refused("directory entry changed before removal"));
            }
            let component = name(value)?;
            // SAFETY: mutation is relative to the held parent and does not follow
            // the leaf. A replaced nonempty directory cannot be removed as a file.
            if unsafe {
                libc::unlinkat(
                    parent.as_raw_fd(),
                    component.as_ptr(),
                    if directory { libc::AT_REMOVEDIR } else { 0 },
                )
            } != 0
            {
                return Err(io::Error::last_os_error());
            }
            Ok(())
        }
        pub fn make_writable(directory: &File) -> io::Result<()> {
            let mode = libc::mode_t::try_from(directory.metadata()?.mode() | 0o700)
                .map_err(|_| refused("directory permission mode exceeds the platform limit"))?;
            // SAFETY: fchmod affects only the held owned directory inode.
            if unsafe { libc::fchmod(directory.as_raw_fd(), mode) } != 0 {
                return Err(io::Error::last_os_error());
            }
            Ok(())
        }
        pub fn sync_directory(directory: &File) -> io::Result<()> {
            directory.sync_all()
        }
    }

    #[cfg(windows)]
    mod platform {
        use super::*;
        use crate::utils::windows_fs;
        pub fn open_directory(path: &Path) -> io::Result<File> {
            windows_fs::open_mutable_tree(path)
        }
        pub fn open_child_directory(parent: &File, name: &OsStr) -> io::Result<File> {
            windows_fs::open_mutable_at(parent, name)
        }
        pub fn create_directory(parent: &File, name: &OsStr) -> io::Result<File> {
            windows_fs::exclusive_create_mutable_directory_at(parent, name)
        }
        pub fn create_file(parent: &File, name: &OsStr) -> io::Result<File> {
            windows_fs::create_private_file_at(parent, name)
        }
        pub fn publish_file(
            parent: &File,
            staged: &OsStr,
            final_name: &OsStr,
            expected: (u64, u64),
        ) -> io::Result<()> {
            windows_fs::rename_file_at(parent, staged, expected, parent, final_name)
        }
        pub fn open_file(parent: &File, name: &OsStr) -> io::Result<File> {
            windows_fs::open_at(parent, name, false)
        }
        pub fn identity(file: &File) -> io::Result<(u64, u64)> {
            windows_fs::identity(file)
        }
        pub fn entry(parent: &File, name: &OsStr) -> io::Result<Entry> {
            let entry = windows_fs::inspect_entry_at(parent, name)?;
            Ok(Entry {
                identity: entry.identity,
                directory: entry.directory,
                symlink: entry.reparse_point,
            })
        }
        pub fn names(directory: &File) -> io::Result<Vec<OsString>> {
            names_bounded(directory, 1_000_000)
        }
        pub fn names_bounded(directory: &File, max_entries: usize) -> io::Result<Vec<OsString>> {
            windows_fs::read_directory_names_bounded(directory, max_entries)
        }
        pub fn remove_entry(
            parent: &File,
            name: &OsStr,
            identity: (u64, u64),
            directory: bool,
        ) -> io::Result<()> {
            windows_fs::remove_entry_at(parent, name, identity, directory)
        }
        pub fn make_writable(_directory: &File) -> io::Result<()> {
            Ok(())
        }
        pub fn sync_directory(_directory: &File) -> io::Result<()> {
            // Windows namespace calls are synchronous; the regular-file intent
            // is flushed before allocation and retained until cleanup ends.
            Ok(())
        }
    }
    pub use platform::*;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> (tempfile::TempDir, PathBuf, PathBuf, Uuid) {
        let temp = tempfile::tempdir().unwrap();
        let common = temp.path().canonicalize().unwrap().join(".libra");
        let id = Uuid::new_v4();
        let run = common.join("sessions/agent-runs").join(id.to_string());
        fs::create_dir_all(&run).unwrap();
        (temp, common, run, id)
    }

    #[test]
    fn wd_doctor_inventory_refuses_intents_over_aggregate_budget() {
        let (_temp, common, run, id) = fixture();
        let workspace =
            create_task_workspace(&common, &run, id, TaskWorkspaceKind::Workspace).unwrap();
        let mut budget = 2;
        assert!(
            read_task_workspace_descriptors_bounded(&common, &run, id, &mut budget).is_err(),
            "three ownership records must not be read through a two-entry budget"
        );
        assert!(workspace.canonical_path.is_dir());
        assert!(run.join(INTENTS_DIR).is_dir());
        let mut budget = 3;
        let descriptors =
            read_task_workspace_descriptors_bounded(&common, &run, id, &mut budget).unwrap();
        assert_eq!(budget, 0);
        assert_eq!(descriptors.len(), 1);
        assert_eq!(descriptors[0].attempt_id, workspace.attempt_id);
    }

    #[test]
    fn wd_attempt_intent() {
        let (_temp, common, run, id) = fixture();
        let first = create_task_workspace(&common, &run, id, TaskWorkspaceKind::Workspace).unwrap();
        let second =
            create_task_workspace(&common, &run, id, TaskWorkspaceKind::Workspace).unwrap();
        assert_ne!(first.attempt_id, second.attempt_id);
        assert_eq!(
            first.canonical_path,
            worktree_layout::task_workspace(&common, id, first.attempt_id)
        );
        assert!(has_task_workspace_intents(&run).unwrap());
        verify_task_workspace(&common, &run, &first).unwrap();
        assert_eq!(
            validated_task_attempt_root(&first.canonical_path).unwrap(),
            Some(first.attempt_root())
        );
        assert!(
            run.join(INTENTS_DIR)
                .join(record_name(first.attempt_id, "intent"))
                .exists()
        );
        cleanup_task_workspaces(&common, &run, id).unwrap();
        assert!(!first.attempt_root().exists());
        assert!(!second.attempt_root().exists());
        assert!(run.exists());
        assert!(!has_task_workspace_intents(&run).unwrap());
    }

    #[test]
    fn failed_attempt_does_not_block_other_owned_attempt_cleanup() {
        let (_temp, common, run, id) = fixture();
        let first = create_task_workspace(&common, &run, id, TaskWorkspaceKind::Workspace).unwrap();
        let second =
            create_task_workspace(&common, &run, id, TaskWorkspaceKind::Workspace).unwrap();
        let first_attempt = first.attempt_root();
        let retained = first_attempt.with_extension("retained");
        fs::rename(&first_attempt, &retained).unwrap();
        fs::create_dir(&first_attempt).unwrap();

        assert!(cleanup_task_workspaces(&common, &run, id).is_err());
        assert!(
            !second.attempt_root().exists(),
            "healthy attempt must be reclaimed"
        );
        assert!(
            first_attempt.exists(),
            "unverified replacement must be retained"
        );
        assert!(has_task_workspace_intents(&run).unwrap());

        fs::remove_dir(&first_attempt).unwrap();
        fs::rename(&retained, &first_attempt).unwrap();
        cleanup_task_workspaces(&common, &run, id).unwrap();
        assert!(!has_task_workspace_intents(&run).unwrap());
    }

    #[test]
    fn staged_proof_failure_cannot_poison_retry_cleanup() {
        let (_temp, common, run, id) = fixture();
        let descriptor =
            create_task_workspace(&common, &run, id, TaskWorkspaceKind::Workspace).unwrap();
        let (_, run_handle) = validate_run(&common, &run, id).unwrap();
        let intents = open_intents(&run_handle, false).unwrap().unwrap();
        let active = record_name(descriptor.attempt_id, "active");
        let original = fs::read(run.join(INTENTS_DIR).join(&active)).unwrap();
        assert_eq!(
            write_record(&intents, "active", &descriptor)
                .unwrap_err()
                .kind(),
            io::ErrorKind::AlreadyExists
        );
        assert_eq!(
            fs::read(run.join(INTENTS_DIR).join(active)).unwrap(),
            original
        );
        let staged = staging_record_name(descriptor.attempt_id, "cleanup");
        fs::write(run.join(INTENTS_DIR).join(staged), b"{truncated")
            .expect("simulate an interrupted staging write");
        cleanup_task_workspaces(&common, &run, id).unwrap();
        assert!(!descriptor.attempt_root().exists());
        assert!(!has_task_workspace_intents(&run).unwrap());
    }

    #[test]
    fn wd_cleanup_ownership() {
        let (_temp, common, run, id) = fixture();
        let descriptor =
            create_task_workspace(&common, &run, id, TaskWorkspaceKind::Workspace).unwrap();
        let original = descriptor.attempt_root().join("original");
        fs::rename(&descriptor.canonical_path, &original).unwrap();
        fs::create_dir(&descriptor.canonical_path).unwrap();
        fs::write(
            descriptor.canonical_path.join("keep"),
            "not the allocated inode",
        )
        .unwrap();
        assert!(cleanup_task_workspaces(&common, &run, id).is_err());
        assert!(descriptor.canonical_path.join("keep").exists());
        assert!(has_task_workspace_intents(&run).unwrap());
        fs::remove_dir_all(&descriptor.canonical_path).unwrap();
        fs::rename(original, &descriptor.canonical_path).unwrap();
        cleanup_task_workspaces(&common, &run, id).unwrap();
    }

    #[test]
    fn wd_unknown_descriptor_keeps_files_and_retry_intent() {
        let (_temp, common, run, id) = fixture();
        let descriptor =
            create_task_workspace(&common, &run, id, TaskWorkspaceKind::Workspace).unwrap();
        let proof = run
            .join(INTENTS_DIR)
            .join(record_name(descriptor.attempt_id, "active"));
        let original = fs::read(&proof).unwrap();
        let mut changed = descriptor.clone();
        changed.layout_version += 1;
        fs::write(&proof, serde_json::to_vec(&changed).unwrap()).unwrap();
        assert!(cleanup_task_workspaces(&common, &run, id).is_err());
        assert!(descriptor.canonical_path.exists());
        fs::write(proof, original).unwrap();
        cleanup_task_workspaces(&common, &run, id).unwrap();
    }

    #[test]
    fn wd_incomplete_provision_is_never_adopted() {
        let (_temp, common, run, id) = fixture();
        let descriptor =
            create_task_workspace(&common, &run, id, TaskWorkspaceKind::Workspace).unwrap();
        fs::remove_file(
            run.join(INTENTS_DIR)
                .join(record_name(descriptor.attempt_id, "owned")),
        )
        .unwrap();
        assert!(cleanup_task_workspaces(&common, &run, id).is_err());
        assert!(descriptor.canonical_path.exists());
        assert!(has_task_workspace_intents(&run).unwrap());
    }

    #[test]
    fn wd_expired_cleanup_preserves_metadata_and_workspace() {
        let (_temp, common, run, id) = fixture();
        let descriptor =
            create_task_workspace(&common, &run, id, TaskWorkspaceKind::Workspace).unwrap();
        assert_eq!(
            cleanup_task_workspaces_until(&common, &run, id, Instant::now())
                .unwrap_err()
                .kind(),
            io::ErrorKind::TimedOut
        );
        assert!(descriptor.canonical_path.exists());
        assert!(has_task_workspace_intents(&run).unwrap());
        cleanup_task_workspaces(&common, &run, id).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn wd_cleanup_unlinks_symlinks_without_touching_targets() {
        use std::os::unix::fs::{PermissionsExt, symlink};
        let (temp, common, run, id) = fixture();
        let descriptor =
            create_task_workspace(&common, &run, id, TaskWorkspaceKind::CheckpointInput).unwrap();
        let outside = temp.path().join("outside");
        fs::create_dir(&outside).unwrap();
        fs::write(outside.join("keep"), b"keep").unwrap();
        symlink(&outside, descriptor.canonical_path.join("link")).unwrap();
        fs::set_permissions(
            &descriptor.canonical_path,
            fs::Permissions::from_mode(0o555),
        )
        .unwrap();
        cleanup_task_workspaces(&common, &run, id).unwrap();
        assert_eq!(fs::read(outside.join("keep")).unwrap(), b"keep");
    }

    #[cfg(unix)]
    #[test]
    fn wd_symlinked_metadata_is_rejected_before_intent() {
        use std::os::unix::fs::symlink;
        let (_temp, common, run, id) = fixture();
        let moved = run.with_extension("moved");
        fs::rename(&run, &moved).unwrap();
        symlink(&moved, &run).unwrap();
        assert!(create_task_workspace(&common, &run, id, TaskWorkspaceKind::Workspace).is_err());
        assert!(!moved.join(INTENTS_DIR).exists());
    }

    #[test]
    fn wd_full_copy_failure_keeps_source_and_no_owned_workspace() {
        let (_temp, common, run, id) = fixture();
        let config = WorkspaceIsolationConfig {
            fuse_state: FuseProvisionState::default(),
            common_storage: common.clone(),
            run_dir: run.clone(),
            allow_full_copy: true,
        };
        assert!(
            materialize_isolated_workspace(
                &common.join("missing-source"),
                id,
                AgentRunId(id),
                &config
            )
            .is_err()
        );
        assert!(!has_task_workspace_intents(&run).unwrap());
    }

    #[cfg(windows)]
    #[test]
    fn wd_full_copy_refuses_junction_to_outside_directory() {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("source");
        let destination = temp.path().join("destination");
        let outside = temp.path().join("outside");
        fs::create_dir(&source).unwrap();
        fs::create_dir(&destination).unwrap();
        fs::create_dir(&outside).unwrap();
        fs::write(outside.join("secret"), b"outside only").unwrap();
        let junction = source.join("outside-link");
        let output = std::process::Command::new("cmd")
            .args(["/C", "mklink", "/J"])
            .arg(&junction)
            .arg(&outside)
            .output()
            .unwrap();
        assert!(output.status.success(), "junction fixture: {output:?}");

        let error = copy_tree(&source, &destination, &source, &destination).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::PermissionDenied);
        assert!(!destination.join("outside-link").exists());
        assert_eq!(fs::read(outside.join("secret")).unwrap(), b"outside only");
    }
}
