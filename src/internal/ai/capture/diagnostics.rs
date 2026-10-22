//! Content-free, best-effort diagnostics for admitted Codex hook callbacks.
//!
//! This is an observation journal, never a capture ledger. Only the validated
//! runtime can supply its repository scope. Missing records, `runtime_ok`, and
//! retained-window counts prove neither successful checkpoints nor thinking.

use std::{
    fs::File,
    io::{self, BufRead, BufReader, Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use serde::{Deserialize, Serialize};

use super::{ingress::CaptureDeadline, runtime_scope::CaptureRuntimeScope};
use crate::internal::ai::{
    hooks::provider::ProviderHookCommand,
    observed_agents::{AgentKind, live_capture::LiveCaptureBinding},
};

const DIRECTORY: &str = "capture-diagnostics";
const SLOTS: [&str; 2] = ["0.jsonl", "1.jsonl"];
const SLOT_CAP: u64 = 5 * 1024 * 1024;
const LINE_CAP: usize = 256;

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
enum Event {
    SessionStart,
    Prompt,
    ToolUse,
    PermissionRequest,
    ModelUpdate,
    Compaction,
    Stop,
    SessionEnd,
    SubagentStart,
    SubagentEnd,
    Observe,
}

impl From<ProviderHookCommand> for Event {
    fn from(value: ProviderHookCommand) -> Self {
        match value {
            ProviderHookCommand::SessionStart => Self::SessionStart,
            ProviderHookCommand::Prompt => Self::Prompt,
            ProviderHookCommand::ToolUse => Self::ToolUse,
            ProviderHookCommand::PermissionRequest => Self::PermissionRequest,
            ProviderHookCommand::ModelUpdate => Self::ModelUpdate,
            ProviderHookCommand::Compaction => Self::Compaction,
            ProviderHookCommand::Stop => Self::Stop,
            ProviderHookCommand::SessionEnd => Self::SessionEnd,
            ProviderHookCommand::SubagentStart => Self::SubagentStart,
            ProviderHookCommand::SubagentEnd => Self::SubagentEnd,
            ProviderHookCommand::Observe => Self::Observe,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
enum Provider {
    Codex,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
enum Outcome {
    RuntimeOk,
    RuntimeFailed,
    TerminalFailed,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Record {
    schema: u8,
    sequence: u64,
    provider: Provider,
    event: Event,
    result: Outcome,
    count: u8,
}

/// A deliberately narrow input capability: no frame, session identity, source
/// path, model, or rendered error can enter the journal.
pub(crate) struct HookObservation {
    storage: PathBuf,
    event: Event,
    deadline: Option<Instant>,
}

impl HookObservation {
    pub(crate) fn admitted(
        binding: LiveCaptureBinding,
        scope: &CaptureRuntimeScope,
        command: ProviderHookCommand,
        deadline: Option<CaptureDeadline>,
    ) -> Option<Self> {
        (binding.observed().provider_kind() == AgentKind::Codex).then(|| Self {
            storage: scope.storage_path.clone(),
            event: command.into(),
            deadline: deadline.map(|value| value.monotonic()),
        })
    }

    /// Preserve the original result and host exit policy even if logging fails.
    /// Never log a journal error through tracing (that could recurse or expose
    /// OS paths). No waits or fresh capture budget are introduced here.
    pub(crate) fn finish(self, failed: bool, terminal: bool) {
        let result = match (failed, terminal) {
            (false, _) => Outcome::RuntimeOk,
            (true, false) => Outcome::RuntimeFailed,
            (true, true) => Outcome::TerminalFailed,
        };
        let soft_limit = Instant::now() + Duration::from_millis(20);
        let until = self
            .deadline
            .map_or(soft_limit, |limit| limit.min(soft_limit));
        let _ = append(&self.storage, self.event, result, until);
    }
}

/// Counts describe only valid records in the two retained slots. `may_have_gaps`
/// stays true: lock contention, process death, budgets, and IO errors are not
/// recoverable from an observation log. Canonical status remains in the catalog.
#[derive(Debug, Serialize)]
pub(crate) struct CaptureDiagnostics {
    state: &'static str,
    window: &'static str,
    may_have_gaps: bool,
    retained_events: u64,
    runtime_ok: u64,
    runtime_failed: u64,
    terminal_failed: u64,
    first_sequence: Option<u64>,
    last_sequence: Option<u64>,
}

impl CaptureDiagnostics {
    fn empty(state: &'static str) -> Self {
        Self {
            state,
            window: "retained_records",
            may_have_gaps: true,
            retained_events: 0,
            runtime_ok: 0,
            runtime_failed: 0,
            terminal_failed: 0,
            first_sequence: None,
            last_sequence: None,
        }
    }

    pub(crate) fn unavailable() -> Self {
        Self::empty("unavailable")
    }

    pub(crate) fn describe(&self) -> String {
        format!(
            "{}; retained={} runtime_ok={} runtime_failed={} terminal_failed={} (best-effort window; not capture proof)",
            self.state,
            self.retained_events,
            self.runtime_ok,
            self.runtime_failed,
            self.terminal_failed,
        )
    }
}

/// Read-only: never mkdir, create a lock, repair permissions, or truncate data.
pub(crate) fn summarize(storage: &Path) -> CaptureDiagnostics {
    match read_summary(storage) {
        Ok(summary) => summary,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            CaptureDiagnostics::empty("absent")
        }
        Err(_) => CaptureDiagnostics::unavailable(),
    }
}

fn invalid() -> io::Error {
    io::Error::other("capture diagnostics are unavailable")
}

fn check_budget(until: Instant) -> io::Result<()> {
    if Instant::now() >= until {
        return Err(io::Error::new(
            io::ErrorKind::TimedOut,
            "diagnostic budget exhausted",
        ));
    }
    Ok(())
}

fn decode(line: &[u8]) -> io::Result<Record> {
    if line.len() > LINE_CAP || line.last() != Some(&b'\n') {
        return Err(invalid());
    }
    let record: Record = serde_json::from_slice(line).map_err(|_| invalid())?;
    if record.schema != 1 || record.count != 1 || record.sequence == 0 {
        return Err(invalid());
    }
    Ok(record)
}

fn length(file: &File) -> io::Result<u64> {
    let size = file.metadata()?.len();
    if size > SLOT_CAP {
        return Err(invalid());
    }
    Ok(size)
}

/// A bounded tail read chooses the newest slot without scanning 10 MiB during
/// every hook. Readers additionally validate every retained line.
fn tail(file: &mut File) -> io::Result<u64> {
    let size = length(file)?;
    if size == 0 {
        return Ok(0);
    }
    let start = size.saturating_sub(LINE_CAP as u64);
    file.seek(SeekFrom::Start(start))?;
    let mut bytes = Vec::with_capacity(LINE_CAP);
    file.take(LINE_CAP as u64).read_to_end(&mut bytes)?;
    if bytes.last() != Some(&b'\n') {
        return Err(invalid());
    }
    let previous = bytes[..bytes.len() - 1]
        .iter()
        .rposition(|byte| *byte == b'\n');
    let offset = previous.map_or(0, |index| index + 1);
    if start != 0 && previous.is_none() {
        return Err(invalid());
    }
    Ok(decode(&bytes[offset..])?.sequence)
}

fn append(storage: &Path, event: Event, result: Outcome, until: Instant) -> io::Result<()> {
    check_budget(until)?;
    let directory = open_directory(storage, true)?;
    let _lock = lock(&directory, true)?;
    check_budget(until)?;
    let mut files = [
        open_file(&directory, SLOTS[0], true)?,
        open_file(&directory, SLOTS[1], true)?,
    ];
    let sequences = [tail(&mut files[0])?, tail(&mut files[1])?];
    if sequences[0] != 0 && sequences[0] == sequences[1] {
        return Err(invalid());
    }
    let mut slot = usize::from(sequences[1] > sequences[0]);
    let sequence = sequences[slot].checked_add(1).ok_or_else(invalid)?;
    let mut bytes = serde_json::to_vec(&Record {
        schema: 1,
        sequence,
        provider: Provider::Codex,
        event,
        result,
        count: 1,
    })
    .map_err(|_| invalid())?;
    bytes.push(b'\n');
    if bytes.len() > LINE_CAP {
        return Err(invalid());
    }
    let rotate = length(&files[slot])? + bytes.len() as u64 > SLOT_CAP;
    check_budget(until)?;
    if rotate {
        slot = 1 - slot;
        files[slot].set_len(0)?;
    }
    files[slot].seek(SeekFrom::End(0))?;
    check_budget(until)?;
    files[slot].write_all(&bytes)
}

fn read_summary(storage: &Path) -> io::Result<CaptureDiagnostics> {
    let directory = open_directory(storage, false)?;
    // A partial initialization is unavailable, not an empty successful log.
    let _lock = lock(&directory, false).map_err(|_| invalid())?;
    let mut summaries = Vec::with_capacity(2);
    for name in SLOTS {
        let file = open_file(&directory, name, false).map_err(|_| invalid())?;
        let size = length(&file)?;
        let mut summary = CaptureDiagnostics::empty("available");
        let mut reader = BufReader::new(file.take(size));
        let mut line = Vec::with_capacity(LINE_CAP);
        loop {
            line.clear();
            // `take` bounds even a malicious file with no line terminator.
            let count = reader
                .by_ref()
                .take((LINE_CAP + 1) as u64)
                .read_until(b'\n', &mut line)?;
            if count == 0 {
                break;
            }
            let record = decode(&line)?;
            if summary
                .last_sequence
                .is_some_and(|previous| record.sequence <= previous)
            {
                return Err(invalid());
            }
            summary.first_sequence.get_or_insert(record.sequence);
            summary.last_sequence = Some(record.sequence);
            summary.retained_events += 1;
            match record.result {
                Outcome::RuntimeOk => summary.runtime_ok += 1,
                Outcome::RuntimeFailed => summary.runtime_failed += 1,
                Outcome::TerminalFailed => summary.terminal_failed += 1,
            }
        }
        summaries.push(summary);
    }
    summaries.sort_by_key(|value| value.first_sequence);
    let mut result = CaptureDiagnostics::empty("available");
    for summary in summaries {
        if let Some(first) = summary.first_sequence {
            if result.last_sequence.is_some_and(|last| first <= last) {
                return Err(invalid());
            }
            result.first_sequence.get_or_insert(first);
            result.last_sequence = summary.last_sequence;
        }
        result.retained_events += summary.retained_events;
        result.runtime_ok += summary.runtime_ok;
        result.runtime_failed += summary.runtime_failed;
        result.terminal_failed += summary.terminal_failed;
    }
    Ok(result)
}

#[cfg(unix)]
fn validate(file: &File, directory: bool) -> io::Result<()> {
    use std::os::unix::fs::MetadataExt;
    let metadata = file.metadata()?;
    // SAFETY: geteuid has no arguments or memory side effects.
    let owner = unsafe { libc::geteuid() };
    if metadata.uid() != owner
        || metadata.mode() & 0o777 != if directory { 0o700 } else { 0o600 }
        || if directory {
            !metadata.is_dir()
        } else {
            !metadata.is_file() || metadata.nlink() != 1
        }
    {
        return Err(invalid());
    }
    Ok(())
}

#[cfg(unix)]
fn open_directory(storage: &Path, create: bool) -> io::Result<File> {
    use std::{
        ffi::CString,
        os::fd::{AsRawFd, FromRawFd},
    };
    let mut parent = crate::utils::beneath::open_root(storage)?;
    for name in ["private", DIRECTORY] {
        let name = CString::new(name).map_err(|_| invalid())?;
        if create {
            // SAFETY: parent is pinned and name is one fixed, NUL-terminated component.
            let result = unsafe { libc::mkdirat(parent.as_raw_fd(), name.as_ptr(), 0o700) };
            if result != 0 && io::Error::last_os_error().kind() != io::ErrorKind::AlreadyExists {
                return Err(io::Error::last_os_error());
            }
        }
        // SAFETY: pinned parent and fixed component; links are rejected by the kernel.
        let descriptor = unsafe {
            libc::openat(
                parent.as_raw_fd(),
                name.as_ptr(),
                libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
            )
        };
        if descriptor < 0 {
            return Err(io::Error::last_os_error());
        }
        // SAFETY: successful openat transferred a new owned descriptor.
        parent = unsafe { File::from_raw_fd(descriptor) };
        validate(&parent, true)?;
    }
    Ok(parent)
}

#[cfg(unix)]
fn open_file(directory: &File, name: &str, create: bool) -> io::Result<File> {
    use std::{
        ffi::CString,
        os::fd::{AsRawFd, FromRawFd},
    };
    let name = CString::new(name).map_err(|_| invalid())?;
    let flags = libc::O_NOFOLLOW
        | libc::O_CLOEXEC
        | libc::O_NONBLOCK
        | if create {
            libc::O_RDWR | libc::O_CREAT
        } else {
            libc::O_RDONLY
        };
    // SAFETY: all callers use fixed file names under the pinned private directory.
    let descriptor = unsafe { libc::openat(directory.as_raw_fd(), name.as_ptr(), flags, 0o600) };
    if descriptor < 0 {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: successful openat transferred a new owned descriptor.
    let file = unsafe { File::from_raw_fd(descriptor) };
    validate(&file, false)?;
    Ok(file)
}

#[cfg(windows)]
fn open_directory(storage: &Path, create: bool) -> io::Result<File> {
    use std::ffi::OsStr;

    use crate::utils::windows_fs;
    let mut parent = windows_fs::open_tree(storage)?;
    for name in ["private", DIRECTORY] {
        parent = if create {
            windows_fs::create_directory_at(&parent, OsStr::new(name), true)?
        } else {
            windows_fs::open_at(&parent, OsStr::new(name), true)?
        };
        windows_fs::validate_private(&parent)?;
    }
    Ok(parent)
}

#[cfg(windows)]
fn open_file(directory: &File, name: &str, create: bool) -> io::Result<File> {
    use std::ffi::OsStr;

    use crate::utils::windows_fs;
    let file = if create {
        windows_fs::open_private_rw_at(directory, OsStr::new(name))?
    } else {
        windows_fs::open_at(directory, OsStr::new(name), false)?
    };
    windows_fs::validate_private(&file)?;
    if windows_fs::information(&file)?.nNumberOfLinks != 1 {
        return Err(invalid());
    }
    Ok(file)
}

#[cfg(not(any(unix, windows)))]
fn open_directory(_storage: &Path, _create: bool) -> io::Result<File> {
    Err(invalid())
}

#[cfg(not(any(unix, windows)))]
fn open_file(_directory: &File, _name: &str, _create: bool) -> io::Result<File> {
    Err(invalid())
}

fn lock(directory: &File, create: bool) -> io::Result<File> {
    let file = open_file(directory, "lock", create)?;
    if file.metadata()?.len() != 0 {
        return Err(invalid());
    }
    if create {
        file.try_lock().map_err(|_| invalid())?;
    } else {
        file.try_lock_shared().map_err(|_| invalid())?;
    }
    Ok(file)
}
