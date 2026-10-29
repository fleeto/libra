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
    fs::File,
    io::Read,
    path::{Component, Path, PathBuf},
    time::SystemTime,
};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::super::adapter::{AgentKind, AgentSessionCtx, AgentStability, ObservedAgent};
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
    let source = session.get("source").and_then(Value::as_str);
    let created = session.get("created_source").and_then(Value::as_str);
    if !source.is_some_and(cli_source) || created.is_some_and(|value| !cli_source(value)) {
        return false;
    }
    if session
        .get("user_id")
        .and_then(Value::as_str)
        .is_some_and(|user| !user.is_empty())
    {
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

fn cli_source(source: &str) -> bool {
    source.eq_ignore_ascii_case("cli")
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
    if overflow {
        if let HermesSourceStatus::Read(mut transcript) = status {
            transcript.partial = true;
            return HermesSourceStatus::Read(transcript);
        }
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
