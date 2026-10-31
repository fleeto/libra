//! Pi coding agent adapter (plan-20260911). Source pin: `earendil-works/pi`
//! `v1.4.0` (`4a08025`).
//!
//! PI-01 registered the identity; PI-02 adds restricted provider-root
//! discovery of session files; PI-15 keeps the discovered sessions of one
//! repository, by header `cwd`, with the session id from the header `id`;
//! PI-03 normalizes one v3 session file into its tree, disk leaf, active
//! branch, and user turns, without keeping content. Pi stays unsupported:
//! nothing here imports a session, installs hooks, or reads a transcript for
//! capture.
//!
//! Pi writes sessions under `<agent-dir>/sessions/--<encoded-cwd>--/` by
//! default (`session-manager.ts:585-601`, `<agent-dir>` defaulting to
//! `~/.pi/agent`) and directly into an explicit session directory otherwise
//! (`session-manager.ts:1752-1756`). Discovery only walks roots inside the
//! configured provider root (`~/.pi`); a project's `.pi/settings.json`
//! `sessionDir` is read by Pi before trust is decided, so it never grants a
//! root here, and neither does any path taken from a session context.

use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Component, Path, PathBuf},
};

use anyhow::{Context, Result};

use super::super::adapter::{AgentKind, AgentSessionCtx, AgentStability, ObservedAgent};

/// PI-01 identity stub. The db tag is accepted, but the registry row stays
/// unsupported: discovery is internal (PI-02), and capture, hook install, and
/// launch are closed until later cards.
pub struct PiObservedAgent;

impl ObservedAgent for PiObservedAgent {
    fn provider_kind(&self) -> AgentKind {
        AgentKind::Pi
    }

    fn provider_name(&self) -> &'static str {
        "pi"
    }

    fn stability(&self) -> AgentStability {
        AgentStability::Stable
    }

    fn read_transcript(&self, _session: &AgentSessionCtx) -> Result<Option<Vec<u8>>> {
        Err(anyhow::anyhow!(
            "pi capture is not available; the identity is registered but discovery, capture, hook install, and launch stay disabled"
        ))
    }

    fn protected_dirs(&self) -> &'static [&'static str] {
        &[".pi"]
    }
}

/// Default bound on directory entries one discovery walk inspects.
pub const PI_DISCOVERY_MAX_ENTRIES: usize = 4096;
/// Default bound on the bytes read from one session file's header line.
pub const PI_DISCOVERY_MAX_HEADER_BYTES: u64 = 64 * 1024;
/// Default bound on the bytes one discovery walk reads in total.
pub const PI_DISCOVERY_MAX_TOTAL_BYTES: u64 = 16 * 1024 * 1024;

/// Bounds for one discovery walk.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PiDiscoveryLimits {
    /// Directory entries inspected across every level of the walk.
    pub max_entries: usize,
    /// Bytes read from one session file looking for its header line.
    pub max_header_bytes: u64,
    /// Bytes read from all session files together.
    pub max_total_bytes: u64,
}

impl Default for PiDiscoveryLimits {
    fn default() -> Self {
        Self {
            max_entries: PI_DISCOVERY_MAX_ENTRIES,
            max_header_bytes: PI_DISCOVERY_MAX_HEADER_BYTES,
            max_total_bytes: PI_DISCOVERY_MAX_TOTAL_BYTES,
        }
    }
}

/// How the session files sit under the discovery root.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PiSessionLayout {
    /// `<root>/--<encoded-cwd>--/<file>.jsonl`, Pi's default sessions root.
    Default,
    /// `<root>/<file>.jsonl`, an explicit session directory.
    Flat,
}

/// Why a discovery root was refused before anything under it was read.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PiRootRefusal {
    /// The root is not an absolute path.
    NotAbsolute,
    /// The root contains a `..` component.
    ParentComponent,
    /// The root is not inside the configured Pi provider root (`~/.pi`).
    OutsideProviderRoot,
    /// The root, or a directory between it and the provider root, is a
    /// symlink.
    Symlink,
    /// The root exists but is not a directory.
    NotADirectory,
}

/// Why a discovery result is partial.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum PiPartialReason {
    /// More entries existed than [`PiDiscoveryLimits::max_entries`].
    EntryLimit,
    /// A header or the walk needed more bytes than the limits allow.
    ByteLimit,
    /// A symlinked entry was refused without being followed or read.
    SymlinkEntry,
    /// A FIFO, device, or other non-regular entry was refused unread.
    NonRegularEntry,
    /// A directory another user owns, that its group or others can write, or
    /// (macOS) that carries an extended ACL, was not walked: an entry there
    /// could be swapped between its type check and its open.
    UnsafeDirectory,
    /// An entry vanished or changed type between enumeration and its open,
    /// or vanished before its type could be read.
    ChangedEntry,
}

/// One session file found by discovery: its path and its first line.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PiSessionCandidate {
    pub path: PathBuf,
    /// The header line without its newline. [`attribute_pi_sessions`] takes
    /// the session identity and repository attribution from it.
    pub header: Vec<u8>,
}

/// The result of one discovery walk.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PiDiscovery {
    /// Session files found, sorted by path.
    pub candidates: Vec<PiSessionCandidate>,
    /// Set when the root was refused; nothing under it was read.
    pub refused: Option<PiRootRefusal>,
    pub partial_reasons: BTreeSet<PiPartialReason>,
    /// Directory entries inspected.
    pub entries_scanned: usize,
    /// Session files read: opened and confirmed to be regular files.
    pub source_reads: usize,
    /// Bytes read from session files.
    pub bytes_read: u64,
}

impl PiDiscovery {
    /// Whether some session files may be missing from [`Self::candidates`].
    pub fn partial(&self) -> bool {
        !self.partial_reasons.is_empty()
    }

    fn refuse(reason: PiRootRefusal) -> Self {
        Self {
            refused: Some(reason),
            ..Self::default()
        }
    }
}

/// Discover Pi session files under `root`, which must be an absolute path
/// inside the configured Pi provider root. Symlinked entries are never
/// followed and non-regular entries are never read; walks past a limit stop
/// and report [`PiPartialReason`]s. A missing root yields an empty result.
pub fn discover_pi_sessions(
    root: &Path,
    layout: PiSessionLayout,
    limits: PiDiscoveryLimits,
) -> Result<PiDiscovery> {
    discover_pi_sessions_observed(root, layout, limits, &mut |_| {})
}

/// [`discover_pi_sessions`] with `before_open` called on each session file
/// just before it is opened (tests use it to swap the entry).
fn discover_pi_sessions_observed(
    root: &Path,
    layout: PiSessionLayout,
    limits: PiDiscoveryLimits,
    before_open: &mut dyn FnMut(&Path),
) -> Result<PiDiscovery> {
    if !root.is_absolute() {
        return Ok(PiDiscovery::refuse(PiRootRefusal::NotAbsolute));
    }
    if root
        .components()
        .any(|component| matches!(component, Component::ParentDir))
    {
        return Ok(PiDiscovery::refuse(PiRootRefusal::ParentComponent));
    }
    let adapter = super::super::agent_for(AgentKind::Pi);
    let normalized = super::super::transcript_source::normalize_macos_system_directory_alias(root);
    let Some(provider_root) = super::super::transcript_source::configured_provider_roots(adapter)
        .into_iter()
        .find(|provider_root| normalized.starts_with(provider_root))
    else {
        return Ok(PiDiscovery::refuse(PiRootRefusal::OutsideProviderRoot));
    };
    discover_within_provider_root(
        adapter,
        &provider_root,
        &normalized,
        layout,
        limits,
        before_open,
    )
}

/// One session of the repository: the file, the session id from its header,
/// and the header `cwd` that placed it in the repository.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PiRepoSession {
    pub path: PathBuf,
    /// The header `id`. File names are never read for it: Pi names a file
    /// `<timestamp>_<id>.jsonl` (`session-manager.ts:1079-1080`), but the
    /// header is the record.
    pub session_id: String,
    pub cwd: PathBuf,
}

/// Why a discovered session file is not one of the repository's sessions.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum PiSkipReason {
    /// The first line is a JSON object but not a Pi session header (`type`
    /// `"session"` with a string `id`). Pi does not list the file either
    /// (`session-manager.ts:677-683`).
    NotASession,
    /// The first line is blank, not JSON, repeats a key, or is not an
    /// object. Pi skips such a line and keeps scanning for a header, so the
    /// file may still be a session.
    UnreadableHeader,
    /// The header `cwd` is missing, not a string, empty, relative, or has a
    /// `.` or `..` component, so the session cannot be placed.
    NoCwd,
    /// The header `cwd` is outside the repository.
    OutsideRepo,
    /// A session of this repository whose header `id` does not match Pi's
    /// session id grammar (`session-manager.ts:268-274`).
    InvalidSessionId,
    /// A session of this repository whose header `id` another file of this
    /// repository also has; none of them is returned.
    DuplicateSessionId,
}

impl PiSkipReason {
    /// Whether a file skipped for this reason may be a session of the
    /// repository that the result leaves out.
    pub fn may_hide_a_repo_session(self) -> bool {
        !matches!(self, Self::NotASession | Self::OutsideRepo)
    }
}

/// The sessions one discovery walk found for one repository.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PiRepoSessions {
    /// The repository's sessions, sorted by path.
    pub sessions: Vec<PiRepoSession>,
    /// Discovered files left out, counted by reason.
    pub skipped: BTreeMap<PiSkipReason, usize>,
    /// The discovery walk's root refusal, if any.
    pub refused: Option<PiRootRefusal>,
    /// The discovery walk's partial reasons.
    pub partial_reasons: BTreeSet<PiPartialReason>,
}

impl PiRepoSessions {
    /// Whether some of the repository's sessions may be missing from
    /// [`Self::sessions`]: the root was refused, the walk was partial, or a
    /// skipped file may be one.
    pub fn partial(&self) -> bool {
        self.refused.is_some()
            || !self.partial_reasons.is_empty()
            || self
                .skipped
                .keys()
                .any(|reason| reason.may_hide_a_repo_session())
    }

    fn skip(&mut self, reason: PiSkipReason) {
        *self.skipped.entry(reason).or_default() += 1;
    }
}

/// Keep the discovered sessions whose header `cwd` is `repo_root` or lies
/// beneath it, with the session id taken from the header `id`. This holds
/// for both layouts: the default layout's `--<encoded-cwd>--` directory name
/// is a lossy encoding of the cwd (`session-manager.ts:589-594`), so it never
/// places a session, and an explicit directory holds sessions of any cwd
/// (Pi filters those by header `cwd` too, `session-manager.ts:1874-1910`).
///
/// `repo_root` must be absolute. A header `cwd` matches when it equals, or
/// lies beneath, the canonical repository root or `repo_root` as given; it
/// is compared lexically, component by component, and never resolved, so no
/// path a header names is touched.
pub fn attribute_pi_sessions(discovery: PiDiscovery, repo_root: &Path) -> Result<PiRepoSessions> {
    if !repo_root.is_absolute()
        || repo_root
            .components()
            .any(|component| matches!(component, Component::ParentDir))
    {
        anyhow::bail!(
            "the repository root {} must be an absolute path without `..`",
            repo_root.display()
        );
    }
    let canonical = repo_root.canonicalize().with_context(|| {
        format!(
            "resolve the repository root {} for Pi session attribution",
            repo_root.display()
        )
    })?;
    let roots = [without_verbatim_prefix(canonical), repo_root.to_path_buf()];
    let mut result = PiRepoSessions {
        refused: discovery.refused,
        partial_reasons: discovery.partial_reasons,
        ..PiRepoSessions::default()
    };
    let mut attributed = Vec::new();
    for candidate in discovery.candidates {
        match attribute_candidate(&candidate.header, &roots) {
            Ok((session_id, cwd)) => attributed.push(PiRepoSession {
                path: candidate.path,
                session_id,
                cwd,
            }),
            Err(reason) => result.skip(reason),
        }
    }
    let mut claims = BTreeMap::<String, usize>::new();
    for session in &attributed {
        *claims.entry(session.session_id.clone()).or_default() += 1;
    }
    for session in attributed {
        if claims.get(&session.session_id).copied().unwrap_or_default() > 1 {
            result.skip(PiSkipReason::DuplicateSessionId);
        } else {
            result.sessions.push(session);
        }
    }
    Ok(result)
}

/// The session id and cwd of one header line, or why it is not one of the
/// repository's sessions.
fn attribute_candidate(
    header: &[u8],
    roots: &[PathBuf],
) -> std::result::Result<(String, PathBuf), PiSkipReason> {
    use super::super::coverage::{CanonValue, parse_canon_value};

    let value = match parse_canon_value(header) {
        Ok(value @ CanonValue::Object(_)) => value,
        _ => return Err(PiSkipReason::UnreadableHeader),
    };
    let (Some("session"), Some(session_id)) = (
        value.get("type").and_then(CanonValue::as_str),
        value.get("id").and_then(CanonValue::as_str),
    ) else {
        return Err(PiSkipReason::NotASession);
    };
    // The raw text is checked for `.` and `..` segments: `Path::components`
    // drops an interior `.`, which would let `/repo/./sub` through.
    let cwd = value
        .get("cwd")
        .and_then(CanonValue::as_str)
        .filter(|cwd| {
            !cwd.is_empty()
                && !cwd
                    .split(std::path::is_separator)
                    .any(|segment| segment == "." || segment == "..")
        })
        .map(PathBuf::from)
        .filter(|cwd| cwd.is_absolute())
        .ok_or(PiSkipReason::NoCwd)?;
    if !roots.iter().any(|root| cwd.starts_with(root)) {
        return Err(PiSkipReason::OutsideRepo);
    }
    if !is_pi_session_id(session_id) {
        return Err(PiSkipReason::InvalidSessionId);
    }
    Ok((session_id.to_owned(), cwd))
}

/// Pi's session id grammar, `^[A-Za-z0-9](?:[A-Za-z0-9._-]*[A-Za-z0-9])?$`
/// (`assertValidSessionId`, `session-manager.ts:268-274`). Pi's own ids are
/// UUIDv7; `--session-id` accepts any id of this form.
fn is_pi_session_id(id: &str) -> bool {
    let bytes = id.as_bytes();
    let (Some(first), Some(last)) = (bytes.first(), bytes.last()) else {
        return false;
    };
    first.is_ascii_alphanumeric()
        && last.is_ascii_alphanumeric()
        && bytes
            .iter()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
}

/// `path` in the form Pi records a cwd. On Windows `canonicalize` returns a
/// verbatim path (`\\?\C:\…`, `\\?\UNC\server\share\…`) while Pi records the
/// plain one (`C:\…`, `\\server\share\…`), so the prefix is rewritten; any
/// other verbatim form is kept and so matches no header.
#[cfg(windows)]
fn without_verbatim_prefix(path: PathBuf) -> PathBuf {
    use std::{ffi::OsString, path::Prefix};

    let mut components = path.components();
    let Some(Component::Prefix(prefix)) = components.next() else {
        return path;
    };
    let root = match prefix.kind() {
        Prefix::VerbatimDisk(drive) => OsString::from(format!("{}:\\", char::from(drive))),
        Prefix::VerbatimUNC(server, share) => {
            let mut plain = OsString::from(r"\\");
            plain.push(server);
            plain.push(r"\");
            plain.push(share);
            plain.push(r"\");
            plain
        }
        _ => return path,
    };
    let mut plain = PathBuf::from(root);
    for component in components {
        if let Component::Normal(part) = component {
            plain.push(part);
        }
    }
    plain
}

#[cfg(not(windows))]
fn without_verbatim_prefix(path: PathBuf) -> PathBuf {
    path
}

/// The session file version this normalizer reads (`CURRENT_SESSION_VERSION`,
/// `session-manager.ts:41`). Pi migrates v1/v2 files in place on open; Libra
/// never rewrites a file, so it refuses them.
pub const PI_SESSION_VERSION: i64 = 3;
/// Default bound on the session file bytes one normalization reads.
pub const PI_NORMALIZE_MAX_BYTES: u64 =
    super::super::transcript_source::TRANSCRIPT_READ_HARD_CAP_BYTES;
/// Default bound on the entries one normalization indexes.
pub const PI_NORMALIZE_MAX_ENTRIES: usize = 200_000;

/// Bounds for one normalization.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PiNormalizeLimits {
    pub max_bytes: u64,
    pub max_entries: usize,
}

impl Default for PiNormalizeLimits {
    fn default() -> Self {
        Self {
            max_bytes: PI_NORMALIZE_MAX_BYTES,
            max_entries: PI_NORMALIZE_MAX_ENTRIES,
        }
    }
}

/// The role of a `message` entry: the five roles Pi persists as `message`
/// entries (PI-21 frozen table; `custom`, `branchSummary`, and
/// `compactionSummary` are derived or stored as other entry types).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PiMessageRole {
    User,
    Assistant,
    ToolResult,
    System,
    BashExecution,
    /// A role outside the PI-21 frozen table, or none.
    Unknown,
}

/// The type of one session entry (PI-20 frozen table,
/// `session-manager.ts:183-194`); `message` roles follow PI-21.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PiEntryKind {
    Message(PiMessageRole),
    CustomMessage,
    Custom,
    ModelChange,
    ThinkingLevelChange,
    Usage,
    Compaction,
    ContextEdit,
    BranchSummary,
    Label,
    SessionInfo,
    /// A type outside the PI-20 frozen table, or none.
    Unknown,
}

impl PiEntryKind {
    fn from_entry(entry: &serde_json::Value) -> Self {
        match entry.get("type").and_then(serde_json::Value::as_str) {
            Some("message") => Self::Message(
                match entry
                    .get("message")
                    .and_then(|message| message.get("role"))
                    .and_then(serde_json::Value::as_str)
                {
                    Some("user") => PiMessageRole::User,
                    Some("assistant") => PiMessageRole::Assistant,
                    Some("toolResult") => PiMessageRole::ToolResult,
                    Some("system") => PiMessageRole::System,
                    Some("bashExecution") => PiMessageRole::BashExecution,
                    _ => PiMessageRole::Unknown,
                },
            ),
            Some("custom_message") => Self::CustomMessage,
            Some("custom") => Self::Custom,
            Some("model_change") => Self::ModelChange,
            Some("thinking_level_change") => Self::ThinkingLevelChange,
            Some("usage") => Self::Usage,
            Some("compaction") => Self::Compaction,
            Some("context_edit") => Self::ContextEdit,
            Some("branch_summary") => Self::BranchSummary,
            Some("label") => Self::Label,
            Some("session_info") => Self::SessionInfo,
            _ => Self::Unknown,
        }
    }
}

/// Why a session file was refused as a whole.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PiNormalizeRefusal {
    /// The input is larger than [`PiNormalizeLimits`] allow.
    TooLarge,
    /// The first parsed line is not a session header (`type` `"session"`
    /// with a session id); Pi loads such a file as empty
    /// (`session-manager.ts:660-665`).
    NotASession,
    /// The header `version` is not [`PI_SESSION_VERSION`].
    UnsupportedVersion,
    /// Two entries have the same `id`.
    DuplicateEntryId,
    /// An entry's `parentId` names no entry in the file.
    MissingParent,
    /// Following `parentId` from some entry returns to it.
    Cycle,
    /// A parsed entry Pi would index under something other than a
    /// non-empty string `id`, a parsed line that is not an object, or a
    /// `parentId` that is neither absent, null, nor a non-empty string: its
    /// place in Pi's tree cannot be reproduced.
    UnrepresentableEntry,
}

/// Why a normalized branch may not show everything the file holds.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum PiBranchPartialReason {
    /// A line `JSON.parse` rejects, or a falsy JSON value, was skipped, as
    /// Pi's loader skips it (`session-manager.ts:616-648`).
    MalformedLine,
    /// A line repeats a key; like `JSON.parse`, the last value is used.
    DuplicateKey,
    /// A second session header was skipped.
    ExtraHeader,
    /// An entry type outside the PI-20 frozen table, or none.
    UnknownEntryType,
    /// A `message` role outside the PI-21 frozen table, or none.
    UnknownMessageRole,
    /// The latest compaction's `firstKeptEntryId` is missing, or names
    /// neither the compaction itself (retain-none) nor an entry earlier on
    /// the branch; the boundary is taken at the compaction, as Pi keeps
    /// nothing before it then (`session-manager.ts:497-509`).
    CompactionBoundaryNotOnBranch,
}

/// One entry on the active branch. No content is kept: `line` points back
/// at the entry's line (1-based) in the normalized bytes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PiBranchEntry {
    pub id: String,
    pub parent_id: Option<String>,
    pub line: usize,
    pub kind: PiEntryKind,
}

/// One user turn on the active branch: the entries from a user message up
/// to the next one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PiTurn {
    pub ordinal: usize,
    /// The `id` of the user message that opens the turn.
    pub opening_entry_id: String,
    /// The turn's entries, as indices into [`PiActiveBranch::entries`].
    pub entries: std::ops::Range<usize>,
    /// `Incomplete` when any entry lies before a compaction boundary: from
    /// there on Pi replaced that history with the compaction summary in the
    /// model context (`session-manager.ts:468-515`), although the raw
    /// entries stay on disk.
    pub completeness: super::super::coverage::Completeness,
}

/// The v3 tree of one session file and its active branch.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PiActiveBranch {
    /// The header `id`.
    pub session_id: String,
    /// The disk leaf: the last entry in file order (`_buildIndex`,
    /// `session-manager.ts:1103-1121`). `None` for a file with only a
    /// header.
    pub leaf_id: Option<String>,
    /// The branch from its root to the leaf.
    pub entries: Vec<PiBranchEntry>,
    /// Entries before the first user message, as indices into `entries`.
    /// They open no turn.
    pub preamble: std::ops::Range<usize>,
    pub turns: Vec<PiTurn>,
    /// Indices into `entries` before the latest compaction boundary.
    pub before_compaction: std::ops::Range<usize>,
    /// Entries in the file that are not on the active branch.
    pub off_branch_entries: usize,
    pub partial_reasons: BTreeSet<PiBranchPartialReason>,
}

impl PiActiveBranch {
    pub fn partial(&self) -> bool {
        !self.partial_reasons.is_empty()
    }
}

/// One indexed entry while the tree is built.
struct IndexedEntry {
    id: String,
    parent_id: Option<String>,
    line: usize,
    kind: PiEntryKind,
    first_kept_entry_id: Option<String>,
}

/// Normalize one Pi v3 session file: index its tree, recover the disk leaf,
/// and segment the active branch into user turns. The bytes are only read;
/// nothing is written, resolved, or logged.
pub fn normalize_pi_session(
    bytes: &[u8],
    limits: PiNormalizeLimits,
) -> std::result::Result<PiActiveBranch, PiNormalizeRefusal> {
    use serde_json::Value;

    use super::super::coverage::{Completeness, parse_canon_value};

    if bytes.len() as u64 > limits.max_bytes {
        return Err(PiNormalizeRefusal::TooLarge);
    }
    let mut partial_reasons = BTreeSet::new();
    let mut header: Option<String> = None;
    let mut entries: Vec<IndexedEntry> = Vec::new();
    for (index, line) in bytes.split(|byte| *byte == b'\n').enumerate() {
        if line.iter().all(u8::is_ascii_whitespace) {
            continue;
        }
        // Pi's loader keeps every line `JSON.parse` accepts except a falsy
        // value, and skips the rest, before the header too
        // (`session-manager.ts:616-648`). A repeated key keeps its last value,
        // as `JSON.parse` does.
        let value = match serde_json::from_slice::<Value>(line) {
            Ok(value) if !is_js_falsy(&value) => value,
            _ => {
                partial_reasons.insert(PiBranchPartialReason::MalformedLine);
                continue;
            }
        };
        if parse_canon_value(line).is_err() {
            partial_reasons.insert(PiBranchPartialReason::DuplicateKey);
        }
        let kind = value.get("type").and_then(Value::as_str);
        if header.is_none() {
            let Some(session_id) = value
                .get("id")
                .and_then(Value::as_str)
                .filter(|_| kind == Some("session"))
            else {
                return Err(PiNormalizeRefusal::NotASession);
            };
            if value.get("version").and_then(Value::as_i64) != Some(PI_SESSION_VERSION) {
                return Err(PiNormalizeRefusal::UnsupportedVersion);
            }
            header = Some(session_id.to_owned());
            continue;
        }
        // `_buildIndex` skips every session-typed entry.
        if kind == Some("session") {
            partial_reasons.insert(PiBranchPartialReason::ExtraHeader);
            continue;
        }
        // `_buildIndex` indexes every other entry under its `id` and makes it
        // the leaf. One whose `id` is not a non-empty string, or whose
        // `parentId` is neither absent, null, nor a non-empty string, has no
        // place in the tree that can be reproduced, so the file is refused
        // rather than given an earlier leaf.
        let id = value
            .get("id")
            .and_then(Value::as_str)
            .filter(|id| !id.is_empty())
            .ok_or(PiNormalizeRefusal::UnrepresentableEntry)?;
        let parent_id = match value.get("parentId") {
            None | Some(Value::Null) => None,
            Some(Value::String(parent)) if !parent.is_empty() => Some(parent.clone()),
            Some(_) => return Err(PiNormalizeRefusal::UnrepresentableEntry),
        };
        if entries.len() >= limits.max_entries {
            return Err(PiNormalizeRefusal::TooLarge);
        }
        let kind = PiEntryKind::from_entry(&value);
        match kind {
            PiEntryKind::Unknown => {
                partial_reasons.insert(PiBranchPartialReason::UnknownEntryType);
            }
            PiEntryKind::Message(PiMessageRole::Unknown) => {
                partial_reasons.insert(PiBranchPartialReason::UnknownMessageRole);
            }
            _ => {}
        }
        entries.push(IndexedEntry {
            id: id.to_owned(),
            parent_id,
            line: index + 1,
            kind,
            first_kept_entry_id: value
                .get("firstKeptEntryId")
                .and_then(Value::as_str)
                .map(str::to_owned),
        });
    }
    let Some(session_id) = header else {
        return Err(PiNormalizeRefusal::NotASession);
    };

    let mut by_id = std::collections::HashMap::with_capacity(entries.len());
    for (position, entry) in entries.iter().enumerate() {
        if by_id.insert(entry.id.as_str(), position).is_some() {
            return Err(PiNormalizeRefusal::DuplicateEntryId);
        }
    }
    let mut parents = Vec::with_capacity(entries.len());
    for entry in &entries {
        parents.push(match &entry.parent_id {
            None => None,
            Some(parent) => Some(
                *by_id
                    .get(parent.as_str())
                    .ok_or(PiNormalizeRefusal::MissingParent)?,
            ),
        });
    }
    reject_parent_cycles(&parents)?;

    let mut path = Vec::new();
    let mut current = entries.len().checked_sub(1);
    while let Some(position) = current {
        path.push(position);
        current = parents[position];
    }
    path.reverse();
    let off_branch_entries = entries.len() - path.len();

    // Pi's context follows the latest compaction on the branch only: the
    // history before its kept entry (earlier on the branch) is replaced by
    // the summary, and without such an entry (retain-none names the
    // compaction itself) everything before the compaction is
    // (`buildContextEntries`, `session-manager.ts:468-515`).
    let boundary = match path
        .iter()
        .rposition(|&position| entries[position].kind == PiEntryKind::Compaction)
    {
        None => 0,
        Some(compaction) => {
            let entry = &entries[path[compaction]];
            let kept = entry.first_kept_entry_id.as_deref();
            let found = kept.and_then(|kept| {
                path[..compaction]
                    .iter()
                    .position(|&earlier| entries[earlier].id == kept)
            });
            if found.is_none() && kept != Some(entry.id.as_str()) {
                partial_reasons.insert(PiBranchPartialReason::CompactionBoundaryNotOnBranch);
            }
            found.unwrap_or(compaction)
        }
    };

    let branch: Vec<PiBranchEntry> = path
        .iter()
        .map(|&position| {
            let entry = &entries[position];
            PiBranchEntry {
                id: entry.id.clone(),
                parent_id: entry.parent_id.clone(),
                line: entry.line,
                kind: entry.kind,
            }
        })
        .collect();
    let openings: Vec<usize> = branch
        .iter()
        .enumerate()
        .filter(|(_, entry)| entry.kind == PiEntryKind::Message(PiMessageRole::User))
        .map(|(index, _)| index)
        .collect();
    let preamble = 0..openings.first().copied().unwrap_or(branch.len());
    let turns = openings
        .iter()
        .enumerate()
        .map(|(ordinal, &start)| {
            let end = openings.get(ordinal + 1).copied().unwrap_or(branch.len());
            PiTurn {
                ordinal,
                opening_entry_id: branch[start].id.clone(),
                entries: start..end,
                completeness: if start < boundary {
                    Completeness::Incomplete
                } else {
                    Completeness::Complete
                },
            }
        })
        .collect();
    Ok(PiActiveBranch {
        session_id,
        leaf_id: branch.last().map(|entry| entry.id.clone()),
        entries: branch,
        preamble,
        turns,
        before_compaction: 0..boundary,
        off_branch_entries,
        partial_reasons,
    })
}

/// Whether `JSON.parse` would give a value Pi's loader drops (`if (entry)`).
fn is_js_falsy(value: &serde_json::Value) -> bool {
    use serde_json::Value;

    match value {
        Value::Null | Value::Bool(false) => true,
        Value::Number(number) => number.as_f64() == Some(0.0),
        Value::String(text) => text.is_empty(),
        _ => false,
    }
}

/// Refuse a parent graph with a cycle. Each entry is walked toward its root
/// once; meeting an entry still on the current walk is a cycle.
fn reject_parent_cycles(parents: &[Option<usize>]) -> std::result::Result<(), PiNormalizeRefusal> {
    #[derive(Clone, Copy, PartialEq, Eq)]
    enum State {
        Unvisited,
        OnWalk,
        Done,
    }
    let mut state = vec![State::Unvisited; parents.len()];
    let mut walk = Vec::new();
    for start in 0..parents.len() {
        let mut current = Some(start);
        while let Some(position) = current {
            match state[position] {
                State::Done => break,
                State::OnWalk => return Err(PiNormalizeRefusal::Cycle),
                State::Unvisited => {
                    state[position] = State::OnWalk;
                    walk.push(position);
                    current = parents[position];
                }
            }
        }
        for position in walk.drain(..) {
            state[position] = State::Done;
        }
    }
    Ok(())
}

#[cfg(any(unix, windows))]
fn discover_within_provider_root(
    adapter: &dyn ObservedAgent,
    provider_root: &Path,
    root: &Path,
    layout: PiSessionLayout,
    limits: PiDiscoveryLimits,
    before_open: &mut dyn FnMut(&Path),
) -> Result<PiDiscovery> {
    let directory =
        match super::super::transcript_source::open_provider_directory_for_discovery(adapter, root)
        {
            Ok(Some(directory)) => directory,
            Ok(None) => return Ok(PiDiscovery::default()),
            Err(error) => {
                return match root_refusal(provider_root, root) {
                    Some(reason) => Ok(PiDiscovery::refuse(reason)),
                    None => Err(error).context("open the Pi session root"),
                };
            }
        };
    let mut discovery = PiDiscovery::default();
    let mut walk = Walk {
        limits,
        discovery: &mut discovery,
        before_open,
    };
    walk.directory(&directory, root, layout)?;
    discovery
        .candidates
        .sort_by(|left, right| left.path.cmp(&right.path));
    Ok(discovery)
}

#[cfg(not(any(unix, windows)))]
fn discover_within_provider_root(
    _adapter: &dyn ObservedAgent,
    _provider_root: &Path,
    _root: &Path,
    _layout: PiSessionLayout,
    _limits: PiDiscoveryLimits,
    _before_open: &mut dyn FnMut(&Path),
) -> Result<PiDiscovery> {
    anyhow::bail!("Pi session discovery is unavailable on this platform; it fails closed")
}

/// Classify a root the no-follow open refused: a symlink at or above it
/// (below the provider root), or a non-directory. `None` leaves the open
/// error as it was.
#[cfg(any(unix, windows))]
fn root_refusal(provider_root: &Path, root: &Path) -> Option<PiRootRefusal> {
    let relative = root.strip_prefix(provider_root).ok()?;
    let mut current = provider_root.to_path_buf();
    for path in std::iter::once(current.clone()).chain(relative.components().map(|component| {
        current.push(component);
        current.clone()
    })) {
        match std::fs::symlink_metadata(&path) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Some(PiRootRefusal::Symlink);
            }
            Ok(metadata) if path == root && !metadata.is_dir() => {
                return Some(PiRootRefusal::NotADirectory);
            }
            Ok(_) => {}
            Err(_) => return None,
        }
    }
    None
}

/// Bytes requested per read while looking for a header's newline. Up to
/// `PI_HEADER_READ_CHUNK - 1` bytes after the newline may be read into the
/// transient read buffer; they are dropped at once, never kept, parsed, or
/// logged, and are charged to the byte limits.
#[cfg(any(unix, windows))]
const PI_HEADER_READ_CHUNK: usize = 512;

/// One bounded walk. Returns `false` from a step once a limit stopped it.
#[cfg(any(unix, windows))]
struct Walk<'a> {
    limits: PiDiscoveryLimits,
    discovery: &'a mut PiDiscovery,
    before_open: &'a mut dyn FnMut(&Path),
}

#[cfg(any(unix, windows))]
impl Walk<'_> {
    fn directory(
        &mut self,
        directory: &std::fs::File,
        logical: &Path,
        layout: PiSessionLayout,
    ) -> Result<bool> {
        use super::super::transcript_source::PinnedEntryType;

        if !directory_is_private(directory)
            .with_context(|| format!("inspect Pi session directory {}", logical.display()))?
        {
            self.discovery
                .partial_reasons
                .insert(PiPartialReason::UnsafeDirectory);
            return Ok(true);
        }
        let mut entries =
            super::super::transcript_source::read_dir_pinned_provider_directory_reporting_vanished(
                directory,
            )
            .with_context(|| format!("read Pi session directory {}", logical.display()))?;
        loop {
            // Checked before the next entry is fetched, so the walk never
            // inspects more than `max_entries`; at the limit it is partial
            // whether or not more entries exist.
            if self.discovery.entries_scanned >= self.limits.max_entries {
                self.discovery
                    .partial_reasons
                    .insert(PiPartialReason::EntryLimit);
                return Ok(false);
            }
            let Some(entry) = entries.next() else {
                return Ok(true);
            };
            self.discovery.entries_scanned += 1;
            let entry = entry.with_context(|| {
                format!("read Pi session directory entry in {}", logical.display())
            })?;
            let name = entry.file_name;
            match entry.file_type {
                PinnedEntryType::Symlink => {
                    self.discovery
                        .partial_reasons
                        .insert(PiPartialReason::SymlinkEntry);
                }
                PinnedEntryType::Other => {
                    self.discovery
                        .partial_reasons
                        .insert(PiPartialReason::NonRegularEntry);
                }
                // Listed, then gone before its type could be read; counted
                // against `max_entries` like any other entry.
                PinnedEntryType::Vanished => {
                    self.discovery
                        .partial_reasons
                        .insert(PiPartialReason::ChangedEntry);
                }
                PinnedEntryType::Directory => {
                    if layout == PiSessionLayout::Default && is_encoded_cwd_dir(&name) {
                        (self.before_open)(&logical.join(&name));
                        let opened =
                            super::super::transcript_source::open_directory_beneath_pinned_directory(
                                directory,
                                Path::new(&name),
                            );
                        let subdirectory = match opened {
                            Ok(subdirectory) => subdirectory,
                            Err(error) => match changed_entry_reason(&error) {
                                Some(reason) => {
                                    self.discovery.partial_reasons.insert(reason);
                                    continue;
                                }
                                None => {
                                    return Err(error).with_context(|| {
                                        format!(
                                            "open Pi session directory {}",
                                            logical.join(&name).display()
                                        )
                                    });
                                }
                            },
                        };
                        if !self.directory(
                            &subdirectory,
                            &logical.join(&name),
                            PiSessionLayout::Flat,
                        )? {
                            return Ok(false);
                        }
                    }
                }
                PinnedEntryType::File => {
                    if layout == PiSessionLayout::Flat
                        && Path::new(&name)
                            .extension()
                            .is_some_and(|extension| extension == "jsonl")
                        && !self.session_file(directory, logical, &name)?
                    {
                        return Ok(false);
                    }
                }
            }
        }
    }

    /// Read one session file's header line within the limits.
    fn session_file(
        &mut self,
        directory: &std::fs::File,
        logical: &Path,
        name: &std::ffi::OsStr,
    ) -> Result<bool> {
        use std::io::Read;

        let remaining = self
            .limits
            .max_total_bytes
            .saturating_sub(self.discovery.bytes_read);
        if remaining == 0 {
            self.discovery
                .partial_reasons
                .insert(PiPartialReason::ByteLimit);
            return Ok(false);
        }
        let path = logical.join(name);
        (self.before_open)(&path);
        let opened = match open_regular_session_file(directory, name) {
            Ok(opened) => opened,
            Err(error) => match changed_entry_reason(&error) {
                Some(reason) => {
                    self.discovery.partial_reasons.insert(reason);
                    return Ok(true);
                }
                None => {
                    return Err(error)
                        .with_context(|| format!("open Pi session file {}", path.display()));
                }
            },
        };
        let Some(file) = opened else {
            // Replaced by a non-regular entry after its type was read.
            self.discovery
                .partial_reasons
                .insert(PiPartialReason::NonRegularEntry);
            return Ok(true);
        };
        self.discovery.source_reads += 1;
        let cap = self.limits.max_header_bytes.min(remaining);
        let mut header = Vec::new();
        let mut reader = file.take(cap);
        let mut chunk = [0u8; PI_HEADER_READ_CHUNK];
        let mut ended_line = false;
        loop {
            let read = reader
                .read(&mut chunk)
                .with_context(|| format!("read Pi session file {}", path.display()))?;
            if read == 0 {
                break;
            }
            self.discovery.bytes_read += read as u64;
            if let Some(index) = chunk[..read].iter().position(|byte| *byte == b'\n') {
                header.extend_from_slice(&chunk[..index]);
                ended_line = true;
                break;
            }
            header.extend_from_slice(&chunk[..read]);
        }
        let truncated = !ended_line && header.len() as u64 == cap;
        if truncated {
            self.discovery
                .partial_reasons
                .insert(PiPartialReason::ByteLimit);
            // A header longer than one file's cap drops that file only; a
            // walk out of total budget stops.
            return Ok(cap == self.limits.max_header_bytes);
        }
        self.discovery
            .candidates
            .push(PiSessionCandidate { path, header });
        Ok(true)
    }
}

/// Whether only the current user can change `directory`'s entries: it is
/// theirs and neither its group nor others can write it. Then no other user
/// can swap an entry between its type check and its open.
#[cfg(unix)]
fn directory_is_private(directory: &std::fs::File) -> Result<bool> {
    use std::os::unix::fs::MetadataExt;

    let metadata = directory.metadata()?;
    // SAFETY: geteuid has no preconditions and cannot fail.
    let euid = unsafe { libc::geteuid() };
    // On Linux a POSIX ACL entry that grants write is capped by the ACL mask,
    // which the group mode bits report, so the mode check covers it.
    Ok(metadata.uid() == euid
        && metadata.mode() & 0o022 == 0
        && !directory_has_extended_acl(directory)?)
}

/// Whether a macOS directory carries an extended ACL. Such an ACL can let
/// another user add or delete entries whatever the mode bits say, so any
/// extended ACL object, whatever its entries, makes the directory unsafe.
#[cfg(target_os = "macos")]
fn directory_has_extended_acl(directory: &std::fs::File) -> Result<bool> {
    use std::os::fd::AsRawFd;

    // <sys/acl.h>: `acl_type_t` is a C enum (int).
    const ACL_TYPE_EXTENDED: libc::c_int = 0x0000_0100;
    unsafe extern "C" {
        fn acl_get_fd_np(fd: libc::c_int, kind: libc::c_int) -> *mut libc::c_void;
        fn acl_free(object: *mut libc::c_void) -> libc::c_int;
    }

    // SAFETY: the descriptor stays open for the call; a non-null result is
    // an ACL this function owns and frees at once.
    let acl = unsafe { acl_get_fd_np(directory.as_raw_fd(), ACL_TYPE_EXTENDED) };
    if !acl.is_null() {
        // SAFETY: `acl` is the live ACL obtained above, freed exactly once.
        unsafe { acl_free(acl) };
        return Ok(true);
    }
    extended_acl_lookup_failure(std::io::Error::last_os_error())
}

/// The verdict for an extended-ACL lookup that returned no ACL: none
/// (`ENOENT`) or none possible on this filesystem (`ENOTSUP`, `EOPNOTSUPP`)
/// is safe; any other failure could hide an ACL and is an error, so the walk
/// fails closed.
#[cfg(target_os = "macos")]
fn extended_acl_lookup_failure(error: std::io::Error) -> Result<bool> {
    match error.raw_os_error() {
        Some(libc::ENOENT) | Some(libc::ENOTSUP) | Some(libc::EOPNOTSUPP) => Ok(false),
        _ => Err(error).context("read the directory's extended ACL"),
    }
}

#[cfg(all(unix, not(target_os = "macos")))]
fn directory_has_extended_acl(_directory: &std::fs::File) -> Result<bool> {
    Ok(false)
}

/// The partial reason for an open that failed because the entry vanished
/// (`ChangedEntry`), became a symlink (`SymlinkEntry`), or changed type
/// (`ChangedEntry`) after enumeration. `None` for any other failure.
#[cfg(any(unix, windows))]
fn changed_entry_reason(error: &anyhow::Error) -> Option<PiPartialReason> {
    let io = error.downcast_ref::<std::io::Error>()?;
    if io.kind() == std::io::ErrorKind::NotFound {
        return Some(PiPartialReason::ChangedEntry);
    }
    #[cfg(unix)]
    if io.raw_os_error() == Some(libc::ELOOP) {
        return Some(PiPartialReason::SymlinkEntry);
    }
    #[cfg(unix)]
    if io.raw_os_error() == Some(libc::ENOTDIR) {
        return Some(PiPartialReason::ChangedEntry);
    }
    // The Windows openers report an entry that is now a reparse point, the
    // other kind (file or directory), or not a disk object as `InvalidData`.
    #[cfg(windows)]
    if io.kind() == std::io::ErrorKind::InvalidData {
        return Some(PiPartialReason::ChangedEntry);
    }
    None
}

/// Whether only the current user (with SYSTEM and Administrators) can change
/// `directory`'s entries: it is owned by one of them and no allowed ACE that
/// applies to it gives write rights to anyone else. Windows directories hold
/// no FIFOs or device nodes, and reparse points are refused by the no-follow
/// walk. A security lookup that fails is an error.
#[cfg(windows)]
fn directory_is_private(directory: &std::fs::File) -> Result<bool> {
    Ok(crate::utils::windows_fs::directory_entries_private(
        directory,
    )?)
}

/// Open `name` beneath `directory` without following a symlink and without
/// blocking, and return it only if the opened object is a regular file. A
/// FIFO or device swapped in after the entry's type was read is closed
/// unread (`None`).
#[cfg(unix)]
fn open_regular_session_file(
    directory: &std::fs::File,
    name: &std::ffi::OsStr,
) -> Result<Option<std::fs::File>> {
    use std::{
        ffi::CString,
        os::{
            fd::{AsRawFd, FromRawFd},
            unix::ffi::OsStrExt,
        },
    };

    let name = CString::new(name.as_bytes()).context("session file name contains NUL")?;
    // SAFETY: the pinned directory descriptor and the NUL-terminated basename
    // stay alive for the call; O_NOFOLLOW refuses a symlink and O_NONBLOCK
    // keeps a FIFO from blocking the open.
    let fd = unsafe {
        libc::openat(
            directory.as_raw_fd(),
            name.as_ptr(),
            libc::O_RDONLY | libc::O_NOFOLLOW | libc::O_NONBLOCK | libc::O_CLOEXEC,
        )
    };
    if fd < 0 {
        return Err(std::io::Error::last_os_error()).context("open the session file (no-follow)");
    }
    // SAFETY: `fd` is a fresh descriptor from `openat`, owned from here on.
    let file = unsafe { std::fs::File::from_raw_fd(fd) };
    if !file
        .metadata()
        .context("inspect the session file")?
        .is_file()
    {
        return Ok(None);
    }
    Ok(Some(file))
}

#[cfg(windows)]
fn open_regular_session_file(
    directory: &std::fs::File,
    name: &std::ffi::OsStr,
) -> Result<Option<std::fs::File>> {
    super::super::open_file_beneath_pinned_provider_directory(directory, Path::new(name)).map(Some)
}

/// Pi names a default-layout directory `--<encoded-cwd>--`.
#[cfg(any(unix, windows))]
fn is_encoded_cwd_dir(name: &std::ffi::OsStr) -> bool {
    name.to_str()
        .is_some_and(|name| name.len() > 4 && name.starts_with("--") && name.ends_with("--"))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// PI-02 AC4: a session file replaced by a FIFO after its directory entry
    /// was read as a regular file is closed unread, without blocking.
    #[cfg(unix)]
    #[test]
    #[serial_test::serial(env)]
    fn pi_discovery_refuses_a_fifo_swapped_in_before_open() {
        let home = tempfile::tempdir().expect("pi home");
        let _home = crate::utils::test::ScopedEnvVar::set("LIBRA_TEST_HOME", home.path());
        let sessions = home.path().join(".pi").join("agent").join("sessions");
        let cwd_dir = sessions.join("--work--");
        std::fs::create_dir_all(&cwd_dir).expect("session dir");
        std::fs::write(cwd_dir.join("swapped.jsonl"), "{\"type\":\"session\"}\n")
            .expect("session file");
        std::fs::write(cwd_dir.join("kept.jsonl"), "{\"type\":\"session\"}\n")
            .expect("session file");
        let mut swapped = 0;
        let discovery = discover_pi_sessions_observed(
            &sessions,
            PiSessionLayout::Default,
            PiDiscoveryLimits::default(),
            &mut |path| {
                if path.file_name().is_some_and(|name| name == "swapped.jsonl") {
                    std::fs::remove_file(path).expect("remove the regular file");
                    let made = std::process::Command::new("mkfifo")
                        .arg(path)
                        .status()
                        .expect("run mkfifo");
                    assert!(made.success(), "mkfifo");
                    swapped += 1;
                }
            },
        )
        .expect("discovery");
        assert_eq!(swapped, 1, "the hook ran before the swapped file's open");
        assert!(
            discovery
                .partial_reasons
                .contains(&PiPartialReason::NonRegularEntry)
        );
        assert_eq!(discovery.source_reads, 1, "only the kept file was read");
        assert_eq!(
            discovery
                .candidates
                .iter()
                .map(|candidate| candidate.path.file_name().expect("name").to_owned())
                .collect::<Vec<_>>(),
            vec![std::ffi::OsString::from("kept.jsonl")]
        );
    }
    /// PI-02 AC3/AC4: an entry that vanishes, becomes a symlink, or (for a
    /// cwd directory) disappears between enumeration and its open is recorded
    /// as partial; the walk goes on and reads the other sessions.
    #[cfg(unix)]
    #[test]
    #[serial_test::serial(env)]
    fn pi_discovery_survives_a_removed_or_swapped_entry() {
        let home = tempfile::tempdir().expect("pi home");
        let _home = crate::utils::test::ScopedEnvVar::set("LIBRA_TEST_HOME", home.path());
        let sessions = home.path().join(".pi").join("agent").join("sessions");
        let cwd_dir = sessions.join("--work--");
        let gone_dir = sessions.join("--gone--");
        std::fs::create_dir_all(&cwd_dir).expect("session dir");
        std::fs::create_dir_all(&gone_dir).expect("vanishing dir");
        for name in ["kept.jsonl", "gone.jsonl", "linked.jsonl"] {
            std::fs::write(cwd_dir.join(name), "{\"type\":\"session\"}\n").expect("session file");
        }
        std::fs::write(gone_dir.join("inside.jsonl"), "{\"type\":\"session\"}\n")
            .expect("session file");
        let target = home.path().join("target.jsonl");
        std::fs::write(&target, "{\"type\":\"session\"}\n").expect("symlink target");
        let discovery = discover_pi_sessions_observed(
            &sessions,
            PiSessionLayout::Default,
            PiDiscoveryLimits::default(),
            &mut |path| match path.file_name().and_then(|name| name.to_str()) {
                Some("gone.jsonl") => std::fs::remove_file(path).expect("remove the file"),
                Some("linked.jsonl") => {
                    std::fs::remove_file(path).expect("remove the file");
                    std::os::unix::fs::symlink(&target, path).expect("swap in a symlink");
                }
                Some("--gone--") => std::fs::remove_dir_all(path).expect("remove the dir"),
                _ => {}
            },
        )
        .expect("a changed entry does not abort the walk");
        assert!(
            discovery
                .partial_reasons
                .contains(&PiPartialReason::ChangedEntry),
            "{:?}",
            discovery.partial_reasons
        );
        assert!(
            discovery
                .partial_reasons
                .contains(&PiPartialReason::SymlinkEntry),
            "{:?}",
            discovery.partial_reasons
        );
        assert_eq!(discovery.source_reads, 1);
        assert_eq!(
            discovery
                .candidates
                .iter()
                .map(|candidate| candidate.path.file_name().expect("name").to_owned())
                .collect::<Vec<_>>(),
            vec![std::ffi::OsString::from("kept.jsonl")]
        );
    }

    /// PI-02 AC3 on Windows: a symlinked (reparse-point) root refuses the
    /// walk, and a symlinked entry is refused unread. Creating a symlink
    /// needs Developer Mode or elevation; without it the test prints
    /// `skipped`, unless `LIBRA_REQUIRE_WINDOWS_SYMLINKS=1` (set by the
    /// native Windows CI job) makes that a failure.
    #[cfg(windows)]
    #[test]
    #[serial_test::serial(env)]
    fn pi_discovery_refuses_reparse_points_on_windows() {
        let home = tempfile::tempdir().expect("pi home");
        let _home = crate::utils::test::ScopedEnvVar::set("LIBRA_TEST_HOME", home.path());
        let agent = home.path().join(".pi").join("agent");
        let sessions = agent.join("sessions");
        let cwd_dir = sessions.join("--work--");
        std::fs::create_dir_all(&cwd_dir).expect("session dir");
        std::fs::write(cwd_dir.join("real.jsonl"), "{\"type\":\"session\"}\n")
            .expect("session file");
        let elsewhere = tempfile::tempdir().expect("elsewhere");
        std::fs::write(
            elsewhere.path().join("target.jsonl"),
            "{\"type\":\"session\"}\n",
        )
        .expect("symlink target");
        if let Err(error) =
            std::os::windows::fs::symlink_dir(elsewhere.path(), agent.join("linked-sessions"))
        {
            assert!(
                std::env::var_os("LIBRA_REQUIRE_WINDOWS_SYMLINKS").is_none(),
                "LIBRA_REQUIRE_WINDOWS_SYMLINKS is set but a symlink cannot be created: {error}"
            );
            eprintln!("skipped (creating a symlink needs Developer Mode or elevation: {error})");
            return;
        }
        let discovery = discover_pi_sessions(
            &agent.join("linked-sessions"),
            PiSessionLayout::Flat,
            PiDiscoveryLimits::default(),
        )
        .expect("discovery");
        assert_eq!(discovery.refused, Some(PiRootRefusal::Symlink));
        assert_eq!(discovery.source_reads, 0);
        std::os::windows::fs::symlink_file(
            elsewhere.path().join("target.jsonl"),
            cwd_dir.join("linked.jsonl"),
        )
        .expect("file symlink");
        let discovery = discover_pi_sessions(
            &sessions,
            PiSessionLayout::Default,
            PiDiscoveryLimits::default(),
        )
        .expect("discovery");
        assert!(
            discovery
                .partial_reasons
                .contains(&PiPartialReason::SymlinkEntry)
        );
        assert_eq!(discovery.source_reads, 1);
        assert_eq!(discovery.candidates.len(), 1);

        // A regular file swapped for a reparse point after enumeration is
        // recorded as changed; the walk still reads the other session.
        std::fs::remove_file(cwd_dir.join("linked.jsonl")).expect("remove the file symlink");
        std::fs::write(cwd_dir.join("swapped.jsonl"), "{\"type\":\"session\"}\n")
            .expect("session file");
        let target = elsewhere.path().join("target.jsonl");
        let discovery = discover_pi_sessions_observed(
            &sessions,
            PiSessionLayout::Default,
            PiDiscoveryLimits::default(),
            &mut |path| {
                if path.file_name().is_some_and(|name| name == "swapped.jsonl") {
                    std::fs::remove_file(path).expect("remove the regular file");
                    std::os::windows::fs::symlink_file(&target, path)
                        .expect("swap in a file symlink");
                }
            },
        )
        .expect("a swapped entry does not abort the walk");
        assert!(
            discovery
                .partial_reasons
                .contains(&PiPartialReason::ChangedEntry),
            "{:?}",
            discovery.partial_reasons
        );
        assert_eq!(discovery.source_reads, 1);
        assert_eq!(discovery.candidates.len(), 1);
    }

    /// PI-02 AC4 on Windows: a session directory whose DACL lets another
    /// principal (here Everyone) write is not walked, so nothing in it is read.
    #[cfg(windows)]
    #[test]
    #[serial_test::serial(env)]
    fn pi_discovery_refuses_a_directory_others_can_write_on_windows() {
        let home = tempfile::tempdir().expect("pi home");
        let _home = crate::utils::test::ScopedEnvVar::set("LIBRA_TEST_HOME", home.path());
        let sessions = home.path().join(".pi").join("agent").join("sessions");
        let private_dir = sessions.join("--work--");
        let shared_dir = sessions.join("--shared--");
        for dir in [&private_dir, &shared_dir] {
            std::fs::create_dir_all(dir).expect("session dir");
            std::fs::write(dir.join("s.jsonl"), "{\"type\":\"session\"}\n").expect("session file");
        }
        let granted = std::process::Command::new("icacls")
            .arg(&shared_dir)
            .args(["/grant", "*S-1-1-0:(W)"])
            .output()
            .expect("run icacls");
        assert!(
            granted.status.success(),
            "icacls: {}",
            String::from_utf8_lossy(&granted.stderr)
        );
        let discovery = discover_pi_sessions(
            &sessions,
            PiSessionLayout::Default,
            PiDiscoveryLimits::default(),
        )
        .expect("discovery");
        assert!(
            discovery
                .partial_reasons
                .contains(&PiPartialReason::UnsafeDirectory),
            "{:?}",
            discovery.partial_reasons
        );
        assert_eq!(
            discovery.source_reads, 1,
            "only the private directory was read"
        );
        assert!(
            discovery
                .candidates
                .iter()
                .all(|candidate| !candidate.path.starts_with(&shared_dir))
        );
    }

    /// PI-02 AC3/AC4 on macOS: an extended-ACL lookup that fails for any
    /// reason but "no ACL" or "unsupported here" is an error, so a directory
    /// whose ACL cannot be read is never walked.
    #[cfg(target_os = "macos")]
    #[test]
    fn pi_discovery_treats_an_unreadable_acl_as_an_error() {
        for errno in [libc::ENOENT, libc::ENOTSUP, libc::EOPNOTSUPP] {
            assert!(
                !extended_acl_lookup_failure(std::io::Error::from_raw_os_error(errno))
                    .expect("no ACL is safe"),
                "{errno}"
            );
        }
        for errno in [libc::EACCES, libc::EBADF, libc::EIO, libc::ENOMEM] {
            assert!(
                extended_acl_lookup_failure(std::io::Error::from_raw_os_error(errno)).is_err(),
                "{errno}"
            );
        }
    }
    /// The shared pinned iterator, given an entry that vanishes between
    /// enumeration and its type lookup: the default mode skips it (Unix) or
    /// fails (Windows) as before, and the reporting mode yields it as
    /// `Vanished`.
    #[test]
    fn pi_discovery_pinned_iterator_reports_an_entry_vanishing_before_type_lookup() {
        use super::super::super::transcript_source::{
            PinnedEntryType, open_absolute_directory_no_follow, read_dir_pinned_provider_directory,
            read_dir_pinned_provider_directory_reporting_vanished, type_lookup_hook,
        };

        let names = |reporting: bool| -> Vec<Result<(String, PinnedEntryType), String>> {
            let dir = tempfile::tempdir().expect("dir");
            for name in ["a.jsonl", "gone.jsonl"] {
                std::fs::write(dir.path().join(name), "x").expect("file");
            }
            let directory = open_absolute_directory_no_follow(
                &super::super::super::transcript_source::normalize_macos_system_directory_alias(
                    dir.path(),
                ),
            )
            .expect("pin the directory");
            let gone = dir.path().join("gone.jsonl");
            type_lookup_hook::with(
                move |name| {
                    if name == "gone.jsonl" {
                        let _ = std::fs::remove_file(&gone);
                    }
                },
                || {
                    let entries = if reporting {
                        read_dir_pinned_provider_directory_reporting_vanished(&directory)
                    } else {
                        read_dir_pinned_provider_directory(&directory)
                    }
                    .expect("read the directory");
                    let mut seen: Vec<_> = entries
                        .map(|entry| {
                            entry
                                .map(|entry| {
                                    (
                                        entry.file_name.to_string_lossy().into_owned(),
                                        entry.file_type,
                                    )
                                })
                                .map_err(|error| error.to_string())
                        })
                        .collect();
                    seen.sort_by(|left, right| format!("{left:?}").cmp(&format!("{right:?}")));
                    seen
                },
            )
        };
        let reported = names(true);
        assert!(
            reported.contains(&Ok(("gone.jsonl".to_string(), PinnedEntryType::Vanished))),
            "{reported:?}"
        );
        assert!(
            reported.contains(&Ok(("a.jsonl".to_string(), PinnedEntryType::File))),
            "{reported:?}"
        );
        let default = names(false);
        assert!(
            default.contains(&Ok(("a.jsonl".to_string(), PinnedEntryType::File))),
            "{default:?}"
        );
        assert!(
            !default
                .iter()
                .any(|entry| matches!(entry, Ok((_, PinnedEntryType::Vanished)))),
            "the default mode never yields Vanished: {default:?}"
        );
        #[cfg(unix)]
        assert_eq!(
            default.len(),
            1,
            "Unix skips the vanished entry: {default:?}"
        );
        #[cfg(windows)]
        assert!(
            default.iter().any(Result::is_err),
            "Windows fails on the vanished entry: {default:?}"
        );
    }

    /// PI-02: an entry that vanishes before its type is read is recorded as
    /// `ChangedEntry` and counted against `max_entries`, and the walk reads
    /// the remaining session.
    #[test]
    #[serial_test::serial(env)]
    fn pi_discovery_counts_an_entry_vanishing_before_type_lookup() {
        use super::super::super::transcript_source::type_lookup_hook;

        let home = tempfile::tempdir().expect("pi home");
        let _home = crate::utils::test::ScopedEnvVar::set("LIBRA_TEST_HOME", home.path());
        let sessions = home.path().join(".pi").join("agent").join("sessions");
        let cwd_dir = sessions.join("--work--");
        std::fs::create_dir_all(&cwd_dir).expect("session dir");
        for name in ["kept.jsonl", "gone.jsonl"] {
            std::fs::write(cwd_dir.join(name), "{\"type\":\"session\"}\n").expect("session file");
        }
        let gone = cwd_dir.join("gone.jsonl");
        let discover = |limits: PiDiscoveryLimits| {
            let gone = gone.clone();
            let _ = std::fs::write(&gone, "{\"type\":\"session\"}\n");
            type_lookup_hook::with(
                move |name| {
                    if name == "gone.jsonl" {
                        let _ = std::fs::remove_file(&gone);
                    }
                },
                || discover_pi_sessions(&sessions, PiSessionLayout::Default, limits),
            )
            .expect("discovery")
        };
        let discovery = discover(PiDiscoveryLimits::default());
        assert!(
            discovery
                .partial_reasons
                .contains(&PiPartialReason::ChangedEntry),
            "{:?}",
            discovery.partial_reasons
        );
        // `--work--`, `kept.jsonl`, and the vanished `gone.jsonl`.
        assert_eq!(discovery.entries_scanned, 3);
        assert_eq!(discovery.source_reads, 1);
        assert_eq!(discovery.candidates.len(), 1);
        // The vanished entry uses up the bound like any other entry. Only
        // the cwd directory and the vanishing file exist here, so the walk
        // cannot reach the bound without visiting it.
        std::fs::remove_file(cwd_dir.join("kept.jsonl")).expect("remove the kept session");
        std::fs::write(&gone, "{\"type\":\"session\"}\n").expect("session file");
        let visited = std::rc::Rc::new(std::cell::Cell::new(false));
        let bounded = {
            let gone = gone.clone();
            let visited = visited.clone();
            type_lookup_hook::with(
                move |name| {
                    if name == "gone.jsonl" {
                        visited.set(true);
                        let _ = std::fs::remove_file(&gone);
                    }
                },
                || {
                    discover_pi_sessions(
                        &sessions,
                        PiSessionLayout::Default,
                        PiDiscoveryLimits {
                            max_entries: 2,
                            ..PiDiscoveryLimits::default()
                        },
                    )
                },
            )
            .expect("discovery")
        };
        assert!(visited.get(), "the vanishing entry was visited");
        assert_eq!(bounded.entries_scanned, 2);
        assert!(
            bounded
                .partial_reasons
                .contains(&PiPartialReason::EntryLimit)
        );
        assert!(
            bounded
                .partial_reasons
                .contains(&PiPartialReason::ChangedEntry)
        );
        assert!(bounded.candidates.is_empty());
    }

    /// PI-15 AC2/AC3: on Windows `canonicalize` returns a verbatim path while
    /// Pi records a plain cwd; a plain header `cwd` beneath the repository
    /// still matches when the root is given in verbatim form.
    #[cfg(windows)]
    #[test]
    fn pi_attribution_matches_a_plain_windows_cwd_against_a_verbatim_root() {
        let repo = tempfile::tempdir().expect("repo");
        let canonical = repo.path().canonicalize().expect("canonical repo root");
        let verbatim = canonical.to_str().expect("utf-8 root");
        assert!(verbatim.starts_with(r"\\?\"), "{verbatim}");
        let plain = without_verbatim_prefix(canonical.clone());
        assert_eq!(plain.to_str().expect("utf-8 root"), &verbatim[4..]);
        assert_eq!(
            without_verbatim_prefix(PathBuf::from(r"\\?\UNC\server\share\repo")),
            PathBuf::from(r"\\server\share\repo")
        );
        assert_eq!(
            without_verbatim_prefix(PathBuf::from(r"\\?\GLOBALROOT\repo")),
            PathBuf::from(r"\\?\GLOBALROOT\repo"),
            "other verbatim forms are kept"
        );

        let cwd = plain.join("src");
        let header = serde_json::json!({
            "type": "session",
            "id": "w1",
            "cwd": cwd.to_str().expect("utf-8 cwd"),
        })
        .to_string()
        .into_bytes();
        let discovery = PiDiscovery {
            candidates: vec![PiSessionCandidate {
                path: PathBuf::from(r"C:\sessions\w1.jsonl"),
                header,
            }],
            ..PiDiscovery::default()
        };
        let sessions = attribute_pi_sessions(discovery, &canonical).expect("attribution");
        assert!(!sessions.partial(), "{sessions:?}");
        assert_eq!(sessions.sessions.len(), 1, "{sessions:?}");
        assert_eq!(sessions.sessions[0].session_id, "w1");
        assert_eq!(sessions.sessions[0].cwd, cwd);
    }
}
