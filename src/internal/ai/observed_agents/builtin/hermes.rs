//! HM-03 Hermes source reader.
//!
//! The primary input is the official JSONL export shape frozen in HM-00:
//! one session object per line, messages as `dict(row)`. Rows with both
//! `active` and `compacted` false are revoked and stay out. Non-empty
//! `reasoning` or `reasoning_content` is readable thinking. The three JSON
//! columns stay raw strings. A read-only SQLite view is used only when that
//! export is unavailable or does not carry the thinking columns. This module
//! does not install hooks or launch Hermes.

use std::{
    collections::BTreeMap,
    fs::File,
    io::Read,
    path::{Component, Path, PathBuf},
    time::{Instant, SystemTime},
};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::super::{
    adapter::{AgentKind, AgentSessionCtx, AgentStability, ObservedAgent},
    coverage::{
        CanonValue, Completeness, NormalizedTurn, SemanticRecord, parse_canon_value,
        valid_provider_turn_id,
    },
    extract::{HermesSessionUsage, project_hermes_session_usage},
};
use crate::utils::object::{open_directory_tree_no_follow, open_regular_file_at_no_follow};

/// Hard cap for one Hermes source. Larger inputs are refused, not truncated
/// into a successful transcript.
pub const HERMES_SOURCE_MAX_BYTES: u64 = 16 * 1024 * 1024;
/// Row budget for one session. One extra row marks the read partial.
pub const HERMES_SOURCE_MAX_ROWS: usize = 4_096;
/// A selected session export is one object. Extra lines are not merged.
pub const HERMES_SOURCE_MAX_SESSIONS: usize = 1;

/// Caller scope the source must match before any message is returned.
#[derive(Debug, Clone)]
pub struct HermesScope {
    pub session_id: String,
    pub profile: String,
    pub repo_root: PathBuf,
}

/// Budgets for one read. Defaults match the module constants.
#[derive(Debug, Clone, Copy)]
pub struct HermesReadLimits {
    pub max_bytes: u64,
    pub max_rows: usize,
    pub max_sessions: usize,
}

impl Default for HermesReadLimits {
    fn default() -> Self {
        Self {
            max_bytes: HERMES_SOURCE_MAX_BYTES,
            max_rows: HERMES_SOURCE_MAX_ROWS,
            max_sessions: HERMES_SOURCE_MAX_SESSIONS,
        }
    }
}

/// One kept Hermes message. Revoked rows never appear here.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HermesMessage {
    pub reasoning: Option<String>,
    pub reasoning_content: Option<String>,
    pub reasoning_details: Option<String>,
    pub codex_reasoning_items: Option<String>,
    pub codex_message_items: Option<String>,
    pub active: bool,
    pub compacted: bool,
    pub partial: bool,
}

impl HermesMessage {
    /// Non-empty `reasoning` or `reasoning_content`. Null and empty are not
    /// readable thinking.
    pub fn readable_thinking(&self) -> Option<&str> {
        [self.reasoning.as_deref(), self.reasoning_content.as_deref()]
            .into_iter()
            .flatten()
            .find(|text| !text.is_empty())
    }
}

/// Filtered transcript. `thinking_fields_present` is false when the export
/// shape has no `reasoning` or `reasoning_content` key at all.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HermesTranscript {
    pub messages: Vec<HermesMessage>,
    pub partial: bool,
    pub thinking_fields_present: bool,
}

/// Fixed, content-free result. Display text never includes a path, SQL, or
/// message body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HermesSourceStatus {
    Read(HermesTranscript),
    Unavailable,
}

impl std::fmt::Display for HermesSourceStatus {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Read(transcript) if transcript.partial => {
                formatter.write_str("hermes source is partial")
            }
            Self::Read(_) => formatter.write_str("hermes source is ready"),
            Self::Unavailable => formatter.write_str("hermes source is unavailable"),
        }
    }
}

/// Where to read. JSONL is first. SQLite runs only when JSONL is missing,
/// unusable, or has no thinking columns.
#[derive(Debug, Clone)]
pub struct HermesSourceRequest {
    pub authorized_root: PathBuf,
    pub jsonl: Option<PathBuf>,
    pub sqlite: Option<PathBuf>,
    pub scope: HermesScope,
    pub limits: HermesReadLimits,
}

/// Parse official JSONL bytes. This does not open a path.
pub fn read_hermes_jsonl(
    bytes: &[u8],
    scope: &HermesScope,
    limits: &HermesReadLimits,
) -> HermesSourceStatus {
    if bytes.len() as u64 > limits.max_bytes || limits.max_sessions == 0 || limits.max_rows == 0 {
        return HermesSourceStatus::Unavailable;
    }
    let mut lines = bytes
        .split(|byte| *byte == b'\n')
        .filter(|line| !line.is_empty());
    let Some(first) = lines.next() else {
        return HermesSourceStatus::Unavailable;
    };
    if lines.take(limits.max_sessions).next().is_some() {
        return HermesSourceStatus::Unavailable;
    }
    let value: Value = match serde_json::from_slice(first) {
        Ok(value) => value,
        Err(_) => return HermesSourceStatus::Unavailable,
    };
    let Some(session) = value.as_object() else {
        return HermesSourceStatus::Unavailable;
    };
    if !session_scope_matches(session, scope) {
        return HermesSourceStatus::Unavailable;
    }
    let Some(messages) = session.get("messages").and_then(Value::as_array) else {
        return HermesSourceStatus::Unavailable;
    };
    project_messages(messages, limits)
}

/// Read one explicit export after the scope check. The bytes are the original
/// JSONL, not the thinking-only projection. A symlink or a scope miss returns
/// nothing. This does not scan a Hermes home.
pub fn authorized_hermes_jsonl_bytes(
    path: &Path,
    scope: &HermesScope,
    limits: &HermesReadLimits,
) -> Option<Vec<u8>> {
    let root = path.parent().filter(|root| !root.as_os_str().is_empty())?;
    let bytes = read_regular_under_root(root, path, limits.max_bytes)?;
    match read_hermes_jsonl(&bytes, scope, limits) {
        HermesSourceStatus::Read(_) => Some(bytes),
        HermesSourceStatus::Unavailable => None,
    }
}

/// Open one regular file under `authorized_root` without following a symlink,
/// then parse it as JSONL.
pub fn read_hermes_jsonl_file(
    path: &Path,
    authorized_root: &Path,
    scope: &HermesScope,
    limits: &HermesReadLimits,
) -> HermesSourceStatus {
    let bytes = match read_regular_under_root(authorized_root, path, limits.max_bytes) {
        Some(bytes) => bytes,
        None => return HermesSourceStatus::Unavailable,
    };
    read_hermes_jsonl(&bytes, scope, limits)
}

/// JSONL first. SQLite only when that export cannot supply the thinking
/// columns. Failures stay content-free.
pub async fn read_hermes_source(request: &HermesSourceRequest) -> HermesSourceStatus {
    if let Some(path) = request.jsonl.as_deref() {
        match read_hermes_jsonl_file(
            path,
            &request.authorized_root,
            &request.scope,
            &request.limits,
        ) {
            HermesSourceStatus::Read(transcript) if transcript.thinking_fields_present => {
                return HermesSourceStatus::Read(transcript);
            }
            HermesSourceStatus::Read(_) | HermesSourceStatus::Unavailable => {}
        }
    }
    if let Some(path) = request.sqlite.as_deref() {
        return read_hermes_sqlite(
            path,
            &request.authorized_root,
            &request.scope,
            &request.limits,
        )
        .await;
    }
    HermesSourceStatus::Unavailable
}

/// Read-only consistent view of an external Hermes `state.db`. Libra does not
/// migrate it. `immutable=1` is not used, so a live WAL remains visible. The
/// main database and its WAL must be unchanged afterwards. An `-shm` file in
/// the same directory may be created or updated by the engine.
pub async fn read_hermes_sqlite(
    db: &Path,
    authorized_root: &Path,
    scope: &HermesScope,
    limits: &HermesReadLimits,
) -> HermesSourceStatus {
    if !confirm_regular_under_root(authorized_root, db, limits.max_bytes) {
        return HermesSourceStatus::Unavailable;
    }
    let before_db = file_stamp(db);
    let wal = sibling(db, "-wal");
    let before_wal = file_stamp(&wal);
    let opened = open_readonly_sqlite(db).await;
    let status = match opened {
        Some(mut connection) => read_sqlite_transaction(&mut connection, scope, limits).await,
        None => HermesSourceStatus::Unavailable,
    };
    let after_db = file_stamp(db);
    let after_wal = file_stamp(&wal);
    if before_db != after_db || before_wal != after_wal {
        return HermesSourceStatus::Unavailable;
    }
    status
}

pub struct HermesObservedAgent;

impl ObservedAgent for HermesObservedAgent {
    fn provider_kind(&self) -> AgentKind {
        AgentKind::Hermes
    }

    fn provider_name(&self) -> &'static str {
        "hermes"
    }

    fn stability(&self) -> AgentStability {
        AgentStability::Stable
    }

    fn read_transcript(&self, session: &AgentSessionCtx) -> anyhow::Result<Option<Vec<u8>>> {
        let Some(path) = session.transcript_path.as_deref() else {
            return Err(anyhow::anyhow!("hermes source is not available"));
        };
        let Some(root) = path.parent().filter(|root| !root.as_os_str().is_empty()) else {
            return Err(anyhow::anyhow!("hermes source is not available"));
        };
        let scope = HermesScope {
            session_id: session.provider_session_id.clone(),
            profile: "default".to_string(),
            repo_root: session.working_dir.clone(),
        };
        match read_hermes_jsonl_file(path, root, &scope, &HermesReadLimits::default()) {
            HermesSourceStatus::Read(transcript) => {
                let projection = serde_json::json!({
                    "partial": transcript.partial,
                    "messages": transcript.messages,
                });
                let bytes = serde_json::to_vec(&projection)
                    .map_err(|_| anyhow::anyhow!("hermes source is not available"))?;
                let source =
                    super::super::transcript_source::TranscriptSource::from_hermes_projection(
                        &session.provider_session_id,
                        bytes,
                    );
                match source {
                    super::super::transcript_source::TranscriptSource::Bytes { bytes, auth }
                        if auth.matches("hermes", &session.provider_session_id, &bytes) =>
                    {
                        Ok(Some(bytes))
                    }
                    _ => Err(anyhow::anyhow!("hermes source is not available")),
                }
            }
            HermesSourceStatus::Unavailable => {
                Err(anyhow::anyhow!("hermes source is not available"))
            }
        }
    }

    fn protected_dirs(&self) -> &'static [&'static str] {
        &[".hermes"]
    }

    fn as_hooks(&self) -> Option<&dyn crate::internal::ai::hooks::provider::HookProvider> {
        Some(crate::internal::ai::hooks::providers::hermes_provider())
    }
}

fn project_messages(messages: &[Value], limits: &HermesReadLimits) -> HermesSourceStatus {
    let mut kept = Vec::new();
    let mut partial = false;
    let mut thinking_fields_present = false;
    for message in messages {
        let Some(object) = message.as_object() else {
            partial = true;
            continue;
        };
        if object.contains_key("reasoning") || object.contains_key("reasoning_content") {
            thinking_fields_present = true;
        }
        let active = flag(object.get("active"));
        let compacted = flag(object.get("compacted"));
        if !active && !compacted {
            continue;
        }
        if kept.len() >= limits.max_rows {
            partial = true;
            break;
        }
        let (reasoning_details, details_partial) = raw_json_string(object.get("reasoning_details"));
        let (codex_reasoning_items, items_partial) =
            raw_json_string(object.get("codex_reasoning_items"));
        let (codex_message_items, message_partial) =
            raw_json_string(object.get("codex_message_items"));
        kept.push(HermesMessage {
            reasoning: optional_text(object.get("reasoning")),
            reasoning_content: optional_text(object.get("reasoning_content")),
            reasoning_details,
            codex_reasoning_items,
            codex_message_items,
            active,
            compacted,
            partial: details_partial || items_partial || message_partial,
        });
        partial = partial || details_partial || items_partial || message_partial;
    }
    if messages.is_empty() {
        return HermesSourceStatus::Read(HermesTranscript {
            messages: Vec::new(),
            partial: false,
            thinking_fields_present: true,
        });
    }
    if !thinking_fields_present {
        return HermesSourceStatus::Read(HermesTranscript {
            messages: Vec::new(),
            partial: true,
            thinking_fields_present: false,
        });
    }
    HermesSourceStatus::Read(HermesTranscript {
        messages: kept,
        partial,
        thinking_fields_present: true,
    })
}

fn session_scope_matches(session: &serde_json::Map<String, Value>, scope: &HermesScope) -> bool {
    if session.get("id").and_then(Value::as_str) != Some(scope.session_id.as_str()) {
        return false;
    }
    if !crate::internal::ai::hooks::providers::hermes::scope::hermes_local_cli_transport(
        session.get("source").and_then(Value::as_str),
        session.get("created_source").and_then(Value::as_str),
    ) {
        return false;
    }
    if !crate::internal::ai::hooks::providers::hermes::scope::hermes_local_operator_user(
        session.get("user_id").and_then(Value::as_str),
    ) {
        return false;
    }
    let profile = session.get("profile_name").and_then(Value::as_str);
    let profile_ok = match profile {
        Some(name) if !name.is_empty() => name == scope.profile,
        _ => scope.profile == "default",
    };
    if !profile_ok {
        return false;
    }
    let Some(cwd) = session.get("cwd").and_then(Value::as_str) else {
        return false;
    };
    cwd_inside_repo(Path::new(cwd), &scope.repo_root)
}

fn cwd_inside_repo(cwd: &Path, repo_root: &Path) -> bool {
    if !cwd.is_absolute() || !repo_root.is_absolute() {
        return false;
    }
    if cwd.components().any(|component| {
        matches!(
            component,
            Component::ParentDir | Component::CurDir | Component::Prefix(_)
        )
    }) {
        return false;
    }
    cwd == repo_root || cwd.starts_with(repo_root)
}

fn flag(value: Option<&Value>) -> bool {
    match value {
        Some(Value::Bool(bit)) => *bit,
        Some(Value::Number(number)) => number.as_i64() == Some(1) || number.as_u64() == Some(1),
        _ => false,
    }
}

fn optional_text(value: Option<&Value>) -> Option<String> {
    match value {
        Some(Value::String(text)) => Some(text.clone()),
        _ => None,
    }
}

/// A JSON string stays that string. Null stays absent. Any other JSON type
/// is a partial row and is not rewritten into a new string.
fn raw_json_string(value: Option<&Value>) -> (Option<String>, bool) {
    match value {
        None | Some(Value::Null) => (None, false),
        Some(Value::String(text)) => (Some(text.clone()), false),
        Some(_) => (None, true),
    }
}

fn confirm_regular_under_root(root: &Path, file: &Path, cap: u64) -> bool {
    let Some(handle) = open_regular_under_root(root, file) else {
        return false;
    };
    handle
        .metadata()
        .ok()
        .is_some_and(|meta| meta.is_file() && meta.len() <= cap)
}

fn read_regular_under_root(root: &Path, file: &Path, cap: u64) -> Option<Vec<u8>> {
    let mut handle = open_regular_under_root(root, file)?;
    read_bounded(&mut handle, cap)
}

fn open_regular_under_root(root: &Path, file: &Path) -> Option<File> {
    let relative = file.strip_prefix(root).ok()?;
    if relative.as_os_str().is_empty()
        || relative
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return None;
    }
    let parent = open_directory_tree_no_follow(root).ok()?;
    let mut directory = parent;
    let mut parts = relative.components();
    let name = loop {
        let component = parts.next()?;
        let Component::Normal(name) = component else {
            return None;
        };
        if parts.clone().next().is_none() {
            break name.to_os_string();
        }
        directory = open_directory_at(&directory, name)?;
    };
    open_regular_file_at_no_follow(&directory, name.as_os_str()).ok()
}

fn open_directory_at(parent: &File, name: &std::ffi::OsStr) -> Option<File> {
    crate::utils::object::open_directory_at_no_follow(parent, name).ok()
}

fn read_bounded(file: &mut File, cap: u64) -> Option<Vec<u8>> {
    let mut buffer = Vec::new();
    let mut chunk = [0_u8; 8192];
    let mut total = 0_u64;
    loop {
        let read = file.read(&mut chunk).ok()?;
        if read == 0 {
            break;
        }
        total = total.saturating_add(read as u64);
        if total > cap {
            return None;
        }
        buffer.extend_from_slice(&chunk[..read]);
    }
    Some(buffer)
}

fn sibling(db: &Path, suffix: &str) -> PathBuf {
    let mut name = db.as_os_str().to_os_string();
    name.push(suffix);
    PathBuf::from(name)
}

fn file_stamp(path: &Path) -> Option<(u64, SystemTime)> {
    let meta = std::fs::metadata(path).ok()?;
    Some((meta.len(), meta.modified().ok()?))
}

async fn open_readonly_sqlite(db: &Path) -> Option<sea_orm::sqlx::SqliteConnection> {
    use std::time::Duration;

    use sea_orm::sqlx::{Connection, sqlite::SqliteConnectOptions};

    let options = SqliteConnectOptions::new()
        .filename(db)
        .read_only(true)
        .create_if_missing(false)
        .busy_timeout(Duration::from_millis(50))
        .pragma("query_only", "ON")
        .pragma("trusted_schema", "OFF");
    sea_orm::sqlx::SqliteConnection::connect_with(&options)
        .await
        .ok()
}

async fn read_sqlite_transaction(
    connection: &mut sea_orm::sqlx::SqliteConnection,
    scope: &HermesScope,
    limits: &HermesReadLimits,
) -> HermesSourceStatus {
    if sea_orm::sqlx::query("BEGIN")
        .execute(&mut *connection)
        .await
        .is_err()
    {
        return HermesSourceStatus::Unavailable;
    }
    let status = read_sqlite_snapshot(connection, scope, limits).await;
    let _ = sea_orm::sqlx::query("ROLLBACK")
        .execute(&mut *connection)
        .await;
    status
}

async fn read_sqlite_snapshot(
    connection: &mut sea_orm::sqlx::SqliteConnection,
    scope: &HermesScope,
    limits: &HermesReadLimits,
) -> HermesSourceStatus {
    use sea_orm::sqlx::Row;

    let columns = sea_orm::sqlx::query("SELECT name FROM pragma_table_info('messages')")
        .fetch_all(&mut *connection)
        .await;
    let Ok(columns) = columns else {
        return HermesSourceStatus::Unavailable;
    };
    let mut names = Vec::new();
    for column in columns {
        let Ok(name) = column.try_get::<String, _>(0) else {
            return HermesSourceStatus::Unavailable;
        };
        names.push(name);
    }
    for required in [
        "session_id",
        "reasoning",
        "reasoning_content",
        "reasoning_details",
        "codex_reasoning_items",
        "codex_message_items",
        "active",
        "compacted",
    ] {
        if !names.iter().any(|name| name == required) {
            return HermesSourceStatus::Unavailable;
        }
    }
    let session = sea_orm::sqlx::query(
        "SELECT id, source, created_source, user_id, cwd, profile_name FROM sessions WHERE id = ? LIMIT 2",
    )
    .bind(scope.session_id.as_str())
    .fetch_all(&mut *connection)
    .await;
    let Ok(session_rows) = session else {
        return HermesSourceStatus::Unavailable;
    };
    if session_rows.len() != 1 {
        return HermesSourceStatus::Unavailable;
    }
    let row = &session_rows[0];
    let session_value = serde_json::json!({
        "id": text_column(row, 0),
        "source": text_column(row, 1),
        "created_source": text_column(row, 2),
        "user_id": text_column(row, 3),
        "cwd": text_column(row, 4),
        "profile_name": text_column(row, 5),
    });
    let Some(session_object) = session_value.as_object() else {
        return HermesSourceStatus::Unavailable;
    };
    if !session_scope_matches(session_object, scope) {
        return HermesSourceStatus::Unavailable;
    }
    let limit = i64::try_from(limits.max_rows.saturating_add(1)).unwrap_or(i64::MAX);
    let messages = sea_orm::sqlx::query(
        "SELECT reasoning, reasoning_content, reasoning_details, codex_reasoning_items, codex_message_items, active, compacted FROM messages WHERE session_id = ? AND (active <> 0 OR compacted <> 0) ORDER BY id ASC LIMIT ?",
    )
    .bind(scope.session_id.as_str())
    .bind(limit)
    .fetch_all(&mut *connection)
    .await;
    let Ok(message_rows) = messages else {
        return HermesSourceStatus::Unavailable;
    };
    let mut values = Vec::with_capacity(message_rows.len());
    let overflow = message_rows.len() > limits.max_rows;
    for message in &message_rows {
        values.push(serde_json::json!({
            "reasoning": text_column(message, 0),
            "reasoning_content": text_column(message, 1),
            "reasoning_details": text_column(message, 2),
            "codex_reasoning_items": text_column(message, 3),
            "codex_message_items": text_column(message, 4),
            "active": integer_column(message, 5),
            "compacted": integer_column(message, 6),
        }));
    }
    let status = project_messages(&values, limits);
    if overflow && let HermesSourceStatus::Read(mut transcript) = status {
        transcript.partial = true;
        return HermesSourceStatus::Read(transcript);
    }
    status
}

fn text_column(row: &sea_orm::sqlx::sqlite::SqliteRow, index: usize) -> Value {
    use sea_orm::sqlx::Row;

    match row.try_get::<Option<String>, _>(index) {
        Ok(Some(text)) => Value::String(text),
        _ => Value::Null,
    }
}

fn integer_column(row: &sea_orm::sqlx::sqlite::SqliteRow, index: usize) -> Value {
    use sea_orm::sqlx::Row;

    match row.try_get::<i64, _>(index) {
        Ok(value) => Value::from(value),
        Err(_) => Value::Null,
    }
}

/// One kept message's thinking fields, unchanged and not interpreted.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct HermesRetainedThinking {
    pub message_id: Option<i64>,
    pub turn_key: String,
    pub reasoning: Option<String>,
    pub reasoning_content: Option<String>,
    pub reasoning_details: Option<String>,
    pub codex_reasoning_items: Option<String>,
    pub codex_message_items: Option<String>,
}

impl HermesRetainedThinking {
    /// Non-empty `reasoning` or `reasoning_content`. Empty and null are not
    /// readable thinking.
    pub fn readable_thinking(&self) -> Option<&str> {
        [self.reasoning.as_deref(), self.reasoning_content.as_deref()]
            .into_iter()
            .flatten()
            .find(|text| !text.is_empty())
    }
}

/// Ordinary turns plus the source values HM-07 still has to store.
///
/// Thinking is not a coverage reasoning record and is not copied into the
/// assistant answer. `parent_associated` stays false: an unproven parent is
/// not linked. `source_state` stays `unproven` until a later card has a
/// native issuer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HermesNormalized {
    pub turns: Vec<NormalizedTurn>,
    pub retained: Vec<HermesRetainedThinking>,
    pub usage: Option<HermesSessionUsage>,
    pub partial: bool,
    pub parent_associated: bool,
    pub source_state: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HermesNormalizeStatus {
    Ready(HermesNormalized),
    Unavailable,
}

impl std::fmt::Display for HermesNormalizeStatus {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ready(normalized) if normalized.partial => {
                formatter.write_str("hermes normalize is partial")
            }
            Self::Ready(_) => formatter.write_str("hermes normalize is ready"),
            Self::Unavailable => formatter.write_str("hermes normalize is unavailable"),
        }
    }
}

/// Normalize one official JSONL export. No path, process, or repository write.
pub fn normalize_hermes_jsonl(
    bytes: &[u8],
    scope: &HermesScope,
    limits: &HermesReadLimits,
) -> HermesNormalizeStatus {
    normalize_hermes_jsonl_until(bytes, scope, limits, None)
        .unwrap_or(HermesNormalizeStatus::Unavailable)
}

/// `None` means the deadline elapsed. Unavailable is a status, not a timeout.
pub fn normalize_hermes_jsonl_until(
    bytes: &[u8],
    scope: &HermesScope,
    limits: &HermesReadLimits,
    deadline: Option<Instant>,
) -> Option<HermesNormalizeStatus> {
    if deadline_elapsed(deadline) {
        return None;
    }
    if bytes.len() as u64 > limits.max_bytes || limits.max_sessions == 0 || limits.max_rows == 0 {
        return Some(HermesNormalizeStatus::Unavailable);
    }
    let mut lines = bytes
        .split(|byte| *byte == b'\n')
        .filter(|line| !line.is_empty());
    let Some(first) = lines.next() else {
        return Some(HermesNormalizeStatus::Unavailable);
    };
    if lines.take(limits.max_sessions).next().is_some() {
        return Some(HermesNormalizeStatus::Unavailable);
    }
    let value: Value = match serde_json::from_slice(first) {
        Ok(value) => value,
        Err(_) => return Some(HermesNormalizeStatus::Unavailable),
    };
    let Some(session) = value.as_object() else {
        return Some(HermesNormalizeStatus::Unavailable);
    };
    if !session_scope_matches(session, scope) {
        return Some(HermesNormalizeStatus::Unavailable);
    }
    let Some(messages) = session.get("messages").and_then(Value::as_array) else {
        return Some(HermesNormalizeStatus::Unavailable);
    };
    Some(HermesNormalizeStatus::Ready(normalize_kept_messages(
        session, messages, limits, deadline,
    )?))
}

fn deadline_elapsed(deadline: Option<Instant>) -> bool {
    deadline.is_some_and(|deadline| Instant::now() >= deadline)
}

struct DraftMessage {
    id: Option<i64>,
    message_uid: Option<String>,
    platform_message_id: Option<String>,
    role: String,
    content: String,
    content_partial: bool,
    tool_call_id: Option<String>,
    tool_name: Option<String>,
    tool_calls: Vec<(Option<String>, String, CanonValue)>,
    tool_partial: bool,
    failed: bool,
    timestamp: Option<i64>,
    retained: HermesRetainedThinking,
}

fn normalize_kept_messages(
    session: &serde_json::Map<String, Value>,
    messages: &[Value],
    limits: &HermesReadLimits,
    deadline: Option<Instant>,
) -> Option<HermesNormalized> {
    let usage = project_hermes_session_usage(session);
    let mut partial = usage.as_ref().is_some_and(|usage| usage.malformed);
    let mut drafts = Vec::new();
    for message in messages {
        if deadline_elapsed(deadline) {
            return None;
        }
        let Some(object) = message.as_object() else {
            partial = true;
            continue;
        };
        if !flag(object.get("active")) && !flag(object.get("compacted")) {
            continue;
        }
        if drafts.len() >= limits.max_rows {
            partial = true;
            break;
        }
        drafts.push(draft_message(object, &mut partial));
    }
    if drafts.iter().any(|draft| draft.id.is_none()) {
        partial = true;
    } else {
        drafts.sort_by_key(|draft| draft.id);
    }
    drafts = dedupe_drafts(drafts);
    let mut turns: Vec<NormalizedTurn> = Vec::new();
    let mut retained = Vec::new();
    let mut current: Option<usize> = None;
    for draft in drafts {
        if deadline_elapsed(deadline) {
            return None;
        }
        let opens_turn = draft.role == "user" && !draft.content.is_empty();
        if opens_turn || current.is_none() {
            let ordinal = turns.len();
            let logical_turn_key = draft
                .id
                .map(|id| id.to_string())
                .filter(|id| valid_provider_turn_id(id))
                .unwrap_or_else(|| format!("ordinal:{ordinal}"));
            turns.push(NormalizedTurn {
                logical_turn_key,
                ordinal,
                completeness: Completeness::Complete,
                started_at: None,
                ended_at: None,
                records: Vec::new(),
            });
            current = Some(turns.len() - 1);
        }
        // INVARIANT: the branch above pushes a turn whenever `current` is empty.
        let index = current?;
        let turn = turns.get_mut(index)?;
        if draft.content_partial || draft.tool_partial || draft.failed || draft.role.is_empty() {
            turn.completeness = Completeness::Incomplete;
            partial = true;
        }
        match draft.role.as_str() {
            "user" if !draft.content.is_empty() => turn.records.push(SemanticRecord::User {
                text: draft.content,
            }),
            "assistant" => {
                if !draft.content.is_empty() {
                    turn.records.push(SemanticRecord::Assistant {
                        text: draft.content,
                    });
                }
                for (call_id, name, input) in draft.tool_calls {
                    turn.records.push(SemanticRecord::ToolCall {
                        call_id,
                        input,
                        name,
                    });
                }
            }
            "tool" => turn.records.push(SemanticRecord::ToolResult {
                call_id: draft.tool_call_id.clone(),
                content: draft.content,
                is_error: draft.failed || draft.tool_name.as_deref() == Some("error"),
            }),
            _ => {
                turn.completeness = Completeness::Incomplete;
                partial = true;
            }
        }
        if let Some(timestamp) = draft.timestamp {
            turn.started_at = Some(
                turn.started_at
                    .map_or(timestamp, |current| current.min(timestamp)),
            );
            turn.ended_at = Some(
                turn.ended_at
                    .map_or(timestamp, |current| current.max(timestamp)),
            );
        }
        let mut kept = draft.retained;
        kept.turn_key = turn.logical_turn_key.clone();
        if kept.reasoning.is_some()
            || kept.reasoning_content.is_some()
            || kept.reasoning_details.is_some()
            || kept.codex_reasoning_items.is_some()
            || kept.codex_message_items.is_some()
        {
            retained.push(kept);
        }
    }
    turns.retain(|turn| !turn.records.is_empty() || turn.completeness == Completeness::Incomplete);
    for (ordinal, turn) in turns.iter_mut().enumerate() {
        if turn.logical_turn_key == format!("ordinal:{}", turn.ordinal) {
            turn.logical_turn_key = format!("ordinal:{ordinal}");
        }
        turn.ordinal = ordinal;
    }
    if partial && let Some(last) = turns.last_mut() {
        last.completeness = Completeness::Incomplete;
    }
    Some(HermesNormalized {
        turns,
        retained,
        usage,
        partial,
        parent_associated: false,
        source_state: "unproven",
    })
}

fn draft_message(object: &serde_json::Map<String, Value>, partial: &mut bool) -> DraftMessage {
    let mut content_partial = false;
    let mut tool_partial = false;
    let (reasoning_details, details_partial) = raw_json_string(object.get("reasoning_details"));
    let (codex_reasoning_items, items_partial) =
        raw_json_string(object.get("codex_reasoning_items"));
    let (codex_message_items, message_partial) = raw_json_string(object.get("codex_message_items"));
    *partial |= details_partial || items_partial || message_partial;
    DraftMessage {
        id: object
            .get("id")
            .and_then(Value::as_i64)
            .filter(|id| *id >= 0),
        message_uid: optional_nonempty(object.get("message_uid")),
        platform_message_id: optional_nonempty(object.get("platform_message_id")),
        role: object
            .get("role")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string(),
        content: ordinary_text(object.get("content"), &mut content_partial),
        content_partial,
        tool_call_id: optional_nonempty(object.get("tool_call_id")),
        tool_name: optional_nonempty(object.get("tool_name")),
        tool_calls: tool_calls_from(object.get("tool_calls"), &mut tool_partial),
        tool_partial,
        failed: failed_finish(object.get("finish_reason"))
            || matches!(
                object.get("effect_disposition").and_then(Value::as_str),
                Some("error" | "failed")
            ),
        timestamp: message_timestamp(object.get("timestamp")),
        retained: HermesRetainedThinking {
            message_id: object
                .get("id")
                .and_then(Value::as_i64)
                .filter(|id| *id >= 0),
            turn_key: String::new(),
            reasoning: optional_text(object.get("reasoning")),
            reasoning_content: optional_text(object.get("reasoning_content")),
            reasoning_details,
            codex_reasoning_items,
            codex_message_items,
        },
    }
}

fn dedupe_drafts(drafts: Vec<DraftMessage>) -> Vec<DraftMessage> {
    fn keep_last(
        drafts: Vec<DraftMessage>,
        key: impl Fn(&DraftMessage) -> Option<&str>,
    ) -> Vec<DraftMessage> {
        let mut last_for_key = BTreeMap::<String, usize>::new();
        for (index, draft) in drafts.iter().enumerate() {
            if let Some(value) = key(draft) {
                last_for_key.insert(value.to_string(), index);
            }
        }
        drafts
            .into_iter()
            .enumerate()
            .filter(|(index, draft)| match key(draft) {
                Some(value) => last_for_key.get(value).copied() == Some(*index),
                None => true,
            })
            .map(|(_, draft)| draft)
            .collect()
    }
    let drafts = keep_last(drafts, |draft| draft.message_uid.as_deref());
    keep_last(drafts, |draft| draft.platform_message_id.as_deref())
}

fn ordinary_text(value: Option<&Value>, partial: &mut bool) -> String {
    match value {
        None | Some(Value::Null) => String::new(),
        Some(Value::String(text)) => text.clone(),
        Some(Value::Array(items)) => {
            let mut text = String::new();
            for item in items {
                if let Some(text_item) = item.as_str() {
                    text.push_str(text_item);
                    continue;
                }
                let Some(object) = item.as_object() else {
                    *partial = true;
                    continue;
                };
                let kind = object.get("type").and_then(Value::as_str).unwrap_or("");
                if kind == "reasoning" || kind == "thinking" {
                    continue;
                }
                if let Some(part) = object.get("text").and_then(Value::as_str) {
                    text.push_str(part);
                } else {
                    *partial = true;
                }
            }
            text
        }
        Some(_) => {
            *partial = true;
            String::new()
        }
    }
}

fn tool_calls_from(
    value: Option<&Value>,
    partial: &mut bool,
) -> Vec<(Option<String>, String, CanonValue)> {
    let Some(Value::Array(items)) = value else {
        if value.is_some_and(|value| !value.is_null()) {
            *partial = true;
        }
        return Vec::new();
    };
    let mut calls = Vec::new();
    for item in items {
        let Some(object) = item.as_object() else {
            *partial = true;
            continue;
        };
        let call_id = optional_nonempty(object.get("id"));
        let (name, arguments) =
            if let Some(function) = object.get("function").and_then(Value::as_object) {
                (
                    optional_nonempty(function.get("name")).unwrap_or_default(),
                    function.get("arguments"),
                )
            } else {
                (
                    optional_nonempty(object.get("name")).unwrap_or_default(),
                    object.get("arguments").or_else(|| object.get("input")),
                )
            };
        if name.is_empty() {
            *partial = true;
        }
        calls.push((call_id, name, arguments_value(arguments, partial)));
    }
    calls
}

fn arguments_value(value: Option<&Value>, partial: &mut bool) -> CanonValue {
    match value {
        None | Some(Value::Null) => CanonValue::Null,
        Some(Value::String(text)) => match parse_canon_value(text.as_bytes()) {
            Ok(parsed) => parsed,
            Err(_) => {
                *partial = true;
                CanonValue::Null
            }
        },
        Some(other) => json_to_canon(other, partial),
    }
}

fn json_to_canon(value: &Value, partial: &mut bool) -> CanonValue {
    match value {
        Value::Null => CanonValue::Null,
        Value::Bool(bit) => CanonValue::Bool(*bit),
        Value::Number(number) => match number.as_i64() {
            Some(integer) => CanonValue::Int(integer),
            None => {
                *partial = true;
                CanonValue::Null
            }
        },
        Value::String(text) => CanonValue::Str(text.clone()),
        Value::Array(items) => CanonValue::Array(
            items
                .iter()
                .map(|item| json_to_canon(item, partial))
                .collect(),
        ),
        Value::Object(map) => CanonValue::Object(
            map.iter()
                .map(|(key, value)| (key.clone(), json_to_canon(value, partial)))
                .collect(),
        ),
    }
}

fn optional_nonempty(value: Option<&Value>) -> Option<String> {
    value
        .and_then(Value::as_str)
        .filter(|text| !text.is_empty())
        .map(str::to_string)
}

fn failed_finish(value: Option<&Value>) -> bool {
    matches!(
        value
            .and_then(Value::as_str)
            .map(|text| text.to_ascii_lowercase())
            .as_deref(),
        Some(
            "error" | "length" | "failed" | "failure" | "content_filter" | "cancelled" | "canceled"
        )
    )
}

/// Bound for one Hermes worker frame. A larger retained body is refused.
pub const HERMES_WORKER_FRAME_MAX_BYTES: u64 = 8 * 1024 * 1024;

/// Closed readable tag for the Hermes ordinary worker. It is not a native
/// reasoning issuer and it does not change the Claude, Codex, or OpenCode tags.
pub fn hermes_readable_tag() -> &'static str {
    "hermes"
}

/// One bounded ordinary frame. Thinking is carried beside the answer.
/// `acquisition_gate_passed` stays false until a real session comparison.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HermesWorkerFrame {
    pub tag: &'static str,
    pub source_state: &'static str,
    pub partial: bool,
    pub acquisition_gate_passed: bool,
    pub ordinary: Vec<NormalizedTurn>,
    pub thinking: Vec<HermesRetainedThinking>,
}

impl HermesWorkerFrame {
    /// User text only. Thinking is not an ordinary prompt.
    pub fn ordinary_prompts(&self) -> Vec<String> {
        let mut prompts = Vec::new();
        for turn in &self.ordinary {
            for record in &turn.records {
                if let SemanticRecord::User { text } = record
                    && !text.is_empty()
                {
                    prompts.push(text.clone());
                }
            }
        }
        prompts
    }
}

/// Content-free worker failure. The body is not copied into the message.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HermesWorkerError {
    Budget { limit: u64 },
    Cancelled,
    BadFrame,
}

impl std::fmt::Display for HermesWorkerError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Budget { limit } => {
                write!(formatter, "hermes worker frame exceeded {limit} bytes")
            }
            Self::Cancelled => formatter.write_str("hermes worker frame was cancelled"),
            Self::BadFrame => formatter.write_str("hermes worker frame is unusable"),
        }
    }
}

/// Import path. The scope is the same one the JSONL reader uses.
pub fn hermes_worker_frame(
    bytes: &[u8],
    scope: &HermesScope,
    deadline: Option<Instant>,
) -> Result<HermesWorkerFrame, HermesWorkerError> {
    let status = normalize_hermes_jsonl_until(bytes, scope, &HermesReadLimits::default(), deadline)
        .ok_or(HermesWorkerError::Cancelled)?;
    let HermesNormalizeStatus::Ready(normalized) = status else {
        return Err(HermesWorkerError::BadFrame);
    };
    finish_hermes_worker_frame(normalized)
}

/// Snapshot and live path for bytes that were already authorized.
/// A non-cli source is still refused. The acquisition gate stays closed.
pub fn hermes_worker_frame_authorized(
    bytes: &[u8],
    deadline: Option<Instant>,
) -> Result<HermesWorkerFrame, HermesWorkerError> {
    if deadline.is_some_and(|deadline| Instant::now() >= deadline) {
        return Err(HermesWorkerError::Cancelled);
    }
    let mut lines = bytes
        .split(|byte| *byte == b'\n')
        .filter(|line| !line.is_empty());
    let Some(first) = lines.next() else {
        return Err(HermesWorkerError::BadFrame);
    };
    if lines.next().is_some() {
        return Err(HermesWorkerError::BadFrame);
    }
    let value: Value = serde_json::from_slice(first).map_err(|_| HermesWorkerError::BadFrame)?;
    let session = value.as_object().ok_or(HermesWorkerError::BadFrame)?;
    let session_id = session
        .get("id")
        .and_then(Value::as_str)
        .filter(|id| !id.is_empty())
        .ok_or(HermesWorkerError::BadFrame)?;
    let cwd = session
        .get("cwd")
        .and_then(Value::as_str)
        .ok_or(HermesWorkerError::BadFrame)?;
    let scope = HermesScope {
        session_id: session_id.to_string(),
        profile: "default".to_string(),
        repo_root: PathBuf::from(cwd),
    };
    hermes_worker_frame(bytes, &scope, deadline)
}

fn finish_hermes_worker_frame(
    normalized: HermesNormalized,
) -> Result<HermesWorkerFrame, HermesWorkerError> {
    let mut retained_bytes = 0_u64;
    for item in &normalized.retained {
        for text in [
            item.reasoning.as_deref(),
            item.reasoning_content.as_deref(),
            item.reasoning_details.as_deref(),
            item.codex_reasoning_items.as_deref(),
            item.codex_message_items.as_deref(),
        ]
        .into_iter()
        .flatten()
        {
            retained_bytes = retained_bytes.saturating_add(text.len() as u64);
        }
    }
    if retained_bytes > HERMES_WORKER_FRAME_MAX_BYTES {
        return Err(HermesWorkerError::Budget {
            limit: HERMES_WORKER_FRAME_MAX_BYTES,
        });
    }
    Ok(HermesWorkerFrame {
        tag: hermes_readable_tag(),
        source_state: "unproven",
        partial: true,
        acquisition_gate_passed: false,
        ordinary: normalized.turns,
        thinking: normalized.retained,
    })
}

fn message_timestamp(value: Option<&Value>) -> Option<i64> {
    let number = value?.as_f64()?;
    if !number.is_finite() || number < 0.0 {
        return None;
    }
    let whole = number.trunc();
    if whole > 10_000_000_000.0 {
        Some((whole / 1000.0) as i64)
    } else {
        Some(whole as i64)
    }
}
