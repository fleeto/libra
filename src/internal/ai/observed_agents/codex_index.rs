//! Bounded Codex source location and fixed readonly index queries. Filesystem
//! writes and SQLite connection creation belong to provider_index_io.
use std::{
    io::Write,
    path::{Component, Path, PathBuf},
    time::{Duration, Instant},
};

use anyhow::{Context, Result, ensure};
use sea_orm::sqlx::{Connection, Row, SqliteConnection};
use serde::{Deserialize, Serialize};

use super::{
    AgentKind, agent_for, live_capture::LiveCandidateUnavailable,
    transcript_source::configured_provider_roots,
};
use crate::internal::ai::authorized_read::{
    self, AUTHORIZED_READ_HELPER_ARG, AUTHORIZED_READ_HELPER_CAP_ENV,
    AUTHORIZED_READ_HELPER_MODE_ENV, RegisteredHelperOutput,
};

pub(crate) const LOCATE_MODE: &str = "codex-live-locate-v1";
const CONTROL_CAP: u64 = 8192;

// SQLx uses SHA-384 of the original migration bytes, not the SHA-256 source
// pins. Source: Codex 0.162.1 / 092d3acd, state/migrations/{0001,0005,0021}.
const REQUIRED_MIGRATIONS: [(i64, &str); 3] = [
    (
        1,
        "627ef19164c9bb298a0cd99945981c9b7bda3d9e6cf12eb35145e3b1d3bf7cf8740f0dbaa0b475185fc2993397078049",
    ),
    (
        5,
        "1ac64f85e083e5b49b974bf498fc39c630f0032057fb4bf4cb31708619fac5a80189f261cf3e38dafd92dc406e73c220",
    ),
    (
        21,
        "ea1aebe4f5a1d56c8effcc15f705c272275dd8adfeb01bc97d3f7b283474a3b8c2b0c8c69de55786e8bda17cae6f4de7",
    ),
];
const ROW_CAP: usize = 10_000;

#[cfg(all(test, feature = "test-live-agent"))]
tokio::task_local! {
    pub(crate) static FULL_SCOPE_ROW_FETCHES: std::sync::Arc<std::sync::atomic::AtomicUsize>;
}

#[cfg(all(test, feature = "test-live-agent"))]
pub(crate) fn required_migrations_for_local_test() -> [(i64, &'static str); 3] {
    REQUIRED_MIGRATIONS
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct IndexThread {
    pub id: String,
    // Data only. Never a capability to open the path or a native-source proof.
    pub rollout_path: String,
    pub cwd: String,
    pub archived: bool,
    pub cli_version: String,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct IndexSpawnEdge {
    pub parent_thread_id: String,
    pub child_thread_id: String,
    pub status: String,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct IndexSnapshot {
    pub max_migration: i64,
    pub threads: Vec<IndexThread>,
    pub spawn_edges: Vec<IndexSpawnEdge>,
}

fn index_refusal(reason: IndexUnavailableReason, message: &'static str) -> anyhow::Error {
    anyhow::Error::new(reason).context(message)
}

async fn begin_snapshot(
    connection: &mut SqliteConnection,
    end: Instant,
) -> Result<(sea_orm::sqlx::Transaction<'_, sea_orm::sqlx::Sqlite>, i64)> {
    let mut tx = connection.begin().await?;
    ensure!(Instant::now() < end, "index query deadline expired");
    let schema = sea_orm::sqlx::query("SELECT name, type FROM pragma_table_list WHERE schema='main' AND name IN ('threads','thread_spawn_edges','_sqlx_migrations')")
        .fetch_all(&mut *tx).await?;
    ensure!(
        schema.len() == 3
            && schema.iter().all(|row| row
                .try_get::<String, _>("type")
                .is_ok_and(|kind| kind == "table")),
        index_refusal(
            IndexUnavailableReason::SchemaUnsupported,
            "index schema is unavailable"
        )
    );
    // Projected columns must be ordinary: generated expressions can amplify
    // a tiny backup. Literal DEFAULTs also need the bounds/preflights below.
    for (sql, expected) in [
        "SELECT COUNT(*) FROM pragma_table_xinfo('threads') WHERE hidden=0 AND name COLLATE NOCASE IN ('id','rollout_path','cwd','archived','cli_version')",
        "SELECT COUNT(*) FROM pragma_table_xinfo('_sqlx_migrations') WHERE hidden=0 AND name COLLATE NOCASE IN ('version','success','checksum')",
        "SELECT COUNT(*) FROM pragma_table_xinfo('thread_spawn_edges') WHERE hidden=0 AND name COLLATE NOCASE IN ('parent_thread_id','child_thread_id','status')",
    ].into_iter().zip([5i64,3,3]) {
        let count: i64 = sea_orm::sqlx::query_scalar(sql).fetch_one(&mut *tx).await?;
        ensure!(count == expected, index_refusal(IndexUnavailableReason::SchemaUnsupported, "index schema is unavailable"));
    }
    // quick_check can evaluate generated-column NOT NULL checks elsewhere
    // in the schema. Reject them before integrity work; CHECK evaluation is
    // disabled separately by the IO owner's connection options.
    let generated: i64 = sea_orm::sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM pragma_table_list l, pragma_table_xinfo(l.name) x WHERE l.schema='main' AND x.hidden IN (2,3))",
    ).fetch_one(&mut *tx).await?;
    ensure!(generated == 0, IndexUnavailableReason::GeneratedSchema);
    let check: String = sea_orm::sqlx::query_scalar("PRAGMA quick_check(1)")
        .fetch_one(&mut *tx)
        .await?;
    ensure!(
        check == "ok",
        index_refusal(
            IndexUnavailableReason::IntegrityFailed,
            "index backup integrity check failed"
        )
    );
    let migrations = sea_orm::sqlx::query(
        "SELECT CASE WHEN typeof(version)='integer' THEN version END AS version,
            CASE WHEN typeof(success)='integer' THEN success END AS success,
            substr(checksum,1,49) AS checksum
            FROM _sqlx_migrations ORDER BY CASE WHEN typeof(version)='integer' THEN version ELSE 0 END LIMIT 60",
    )
    .fetch_all(&mut *tx)
    .await?;
    ensure!(
        !migrations.is_empty() && migrations.len() <= 59,
        index_refusal(
            IndexUnavailableReason::MigrationUnsupported,
            "index migration history is unsupported"
        )
    );
    let mut found = [false; 3];
    let mut max_migration = 0;
    for row in migrations {
        let version: i64 = row.try_get("version").map_err(|_| {
            index_refusal(
                IndexUnavailableReason::MigrationUnsupported,
                "index migration is unsupported or failed",
            )
        })?;
        let success: i64 = row.try_get("success").map_err(|_| {
            index_refusal(
                IndexUnavailableReason::MigrationUnsupported,
                "index migration is unsupported or failed",
            )
        })?;
        let checksum: Vec<u8> = row.try_get("checksum").map_err(|_| {
            index_refusal(
                IndexUnavailableReason::MigrationUnsupported,
                "index migration is unsupported or failed",
            )
        })?;
        ensure!(
            (1..=59).contains(&version) && success == 1 && checksum.len() == 48,
            index_refusal(
                IndexUnavailableReason::MigrationUnsupported,
                "index migration is unsupported or failed"
            )
        );
        for (slot, (required, digest)) in REQUIRED_MIGRATIONS.iter().enumerate() {
            if version == *required {
                ensure!(
                    !found[slot] && hex::encode(&checksum) == *digest,
                    index_refusal(
                        IndexUnavailableReason::MigrationUnsupported,
                        "index required migration checksum differs"
                    )
                );
                found[slot] = true;
            }
        }
        max_migration = version;
    }
    ensure!(
        found.into_iter().all(|present| present),
        index_refusal(
            IndexUnavailableReason::MigrationUnsupported,
            "index required migrations are missing"
        )
    );
    Ok((tx, max_migration))
}

/// One read transaction covers integrity, migration/schema checks and all rows.
/// SQL is fixed: provider fields never become SQL, filesystem or model input.
pub(crate) async fn query_snapshot(
    connection: &mut SqliteConnection,
    end: Instant,
) -> Result<IndexSnapshot> {
    let (mut tx, max_migration) = begin_snapshot(connection, end).await?;
    // Bound lazy DEFAULT values before SQLx copies any string rows. The
    // sentinel subquery has no ORDER BY: sorting could materialize large data.
    let metrics = sea_orm::sqlx::query(
        "SELECT COUNT(*) AS n,
            COALESCE(SUM(CASE WHEN typeof(id)='text' AND octet_length(id) BETWEEN 1 AND 64
                AND typeof(rollout_path)='text' AND octet_length(rollout_path) BETWEEN 1 AND 4096
                AND typeof(cwd)='text' AND octet_length(cwd) BETWEEN 1 AND 4096
                AND typeof(cli_version)='text' AND octet_length(cli_version)<=128
                AND typeof(archived)='integer' AND archived IN (0,1) THEN 0 ELSE 1 END),0) AS bad,
            COALESCE(SUM(octet_length(id)+octet_length(rollout_path)+octet_length(cwd)+octet_length(cli_version)),0) AS text_bytes
        FROM (SELECT id,rollout_path,cwd,archived,cli_version FROM threads LIMIT 10001)",
    ).fetch_one(&mut *tx).await?;
    ensure!(
        metrics.try_get::<i64, _>("n")? <= ROW_CAP as i64,
        index_refusal(
            IndexUnavailableReason::RowCap,
            "index thread limit exceeded"
        )
    );
    ensure!(
        metrics.try_get::<i64, _>("bad")? == 0,
        index_refusal(
            IndexUnavailableReason::RowInvalid,
            "invalid index thread fields"
        )
    );
    ensure!(
        metrics.try_get::<i64, _>("text_bytes")? <= 1024 * 1024,
        index_refusal(IndexUnavailableReason::TextCap, "index text limit exceeded")
    );
    #[cfg(all(test, feature = "test-live-agent"))]
    let _ = FULL_SCOPE_ROW_FETCHES.try_with(|counter| {
        counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    });
    let rows = sea_orm::sqlx::query(
        "SELECT id, rollout_path, cwd, archived, cli_version FROM threads LIMIT 10001",
    )
    .fetch_all(&mut *tx)
    .await?;
    ensure!(rows.len() <= ROW_CAP, "index thread limit exceeded");
    let mut threads = Vec::with_capacity(rows.len());
    let mut ids = std::collections::BTreeSet::new();
    let mut total_text = 0usize;
    for row in rows {
        ensure!(Instant::now() < end, "index thread deadline expired");
        let id: String = row.try_get("id")?;
        let rollout_path: String = row.try_get("rollout_path")?;
        let cwd: String = row.try_get("cwd")?;
        let cli_version: String = row.try_get("cli_version")?;
        let archived: i64 = row.try_get("archived")?;
        ensure!(
            valid_session(&id) && matches!(archived, 0 | 1),
            "invalid index thread fields"
        );
        ensure!(
            ids.insert(id.clone()),
            index_refusal(
                IndexUnavailableReason::RowInvalid,
                "index contains duplicate thread identifiers"
            )
        );
        for text in [&rollout_path, &cwd] {
            ensure!(
                !text.is_empty() && text.len() <= 4096 && !text.contains('\0'),
                "invalid index path data"
            );
            total_text += text.len();
        }
        ensure!(
            cli_version.len() <= 128 && !cli_version.contains('\0'),
            "invalid index CLI version"
        );
        total_text += id.len() + cli_version.len();
        ensure!(total_text <= 1024 * 1024, "index text limit exceeded");
        threads.push(IndexThread {
            id,
            rollout_path,
            cwd,
            archived: archived == 1,
            cli_version,
        });
    }
    let metrics = sea_orm::sqlx::query(
        "SELECT COUNT(*) AS n,
            COALESCE(SUM(CASE WHEN typeof(parent_thread_id)='text' AND octet_length(parent_thread_id) BETWEEN 1 AND 64
                AND typeof(child_thread_id)='text' AND octet_length(child_thread_id) BETWEEN 1 AND 64
                AND typeof(status)='text' AND octet_length(status) BETWEEN 1 AND 128 THEN 0 ELSE 1 END),0) AS bad,
            COALESCE(SUM(octet_length(parent_thread_id)+octet_length(child_thread_id)+octet_length(status)),0) AS text_bytes
        FROM (SELECT parent_thread_id,child_thread_id,status FROM thread_spawn_edges LIMIT 10001)",
    ).fetch_one(&mut *tx).await?;
    ensure!(
        metrics.try_get::<i64, _>("n")? <= ROW_CAP as i64,
        index_refusal(IndexUnavailableReason::EdgeCap, "index edge limit exceeded")
    );
    ensure!(
        metrics.try_get::<i64, _>("bad")? == 0,
        index_refusal(
            IndexUnavailableReason::RowInvalid,
            "invalid index edge fields"
        )
    );
    let thread_bytes =
        i64::try_from(total_text).context("index text size exceeds this platform")?;
    ensure!(
        metrics.try_get::<i64, _>("text_bytes")? + thread_bytes <= 1024 * 1024,
        index_refusal(IndexUnavailableReason::TextCap, "index text limit exceeded")
    );
    #[cfg(all(test, feature = "test-live-agent"))]
    let _ = FULL_SCOPE_ROW_FETCHES.try_with(|counter| {
        counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    });
    let rows = sea_orm::sqlx::query(
        "SELECT parent_thread_id, child_thread_id, status FROM thread_spawn_edges LIMIT 10001",
    )
    .fetch_all(&mut *tx)
    .await?;
    ensure!(rows.len() <= ROW_CAP, "index edge limit exceeded");
    let mut spawn_edges = Vec::with_capacity(rows.len());
    for row in rows {
        ensure!(Instant::now() < end, "index edge deadline expired");
        let parent_thread_id: String = row.try_get("parent_thread_id")?;
        let child_thread_id: String = row.try_get("child_thread_id")?;
        let status: String = row.try_get("status")?;
        ensure!(
            valid_session(&parent_thread_id)
                && valid_session(&child_thread_id)
                && !status.is_empty()
                && status.len() <= 128
                && !status.contains('\0'),
            "invalid index edge fields"
        );
        total_text += parent_thread_id.len() + child_thread_id.len() + status.len();
        ensure!(total_text <= 1024 * 1024, "index text limit exceeded");
        spawn_edges.push(IndexSpawnEdge {
            parent_thread_id,
            child_thread_id,
            status,
        });
    }
    ensure!(Instant::now() < end, "index snapshot deadline expired");
    tx.commit().await?;
    Ok(IndexSnapshot {
        max_migration,
        threads,
        spawn_edges,
    })
}
// Migration 0025 is deliberately separate: InspectAll remains compatible
// with its original three migrations and its original wire response.
const TIMESTAMP_MIGRATION: &str = "d6fb51d79941b93c3278fe2a8a0c6fa882407b7b716e32d72329f0b7b60c181a42f49e56cd552315efb0402a111ede12";
pub(crate) const RECENT_WINDOW_MS: i64 = 7 * 24 * 60 * 60 * 1000;
pub(crate) const CLOCK_SKEW_MS: i64 = 5 * 60 * 1000;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RecentScopeQuery {
    pub version: u8,
    pub now_ms: i64,
    pub root: PathBuf,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root_identity: Option<(u64, u64)>,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum IndexFreshness {
    WithinWindow,
    OutsideWindow,
    EmptySnapshot,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum RecentScopeStatus {
    Proven,
    LayoutUnknown,
    AliasAmbiguous,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum RecentEdgeStatus {
    Running,
    Completed,
    Failed,
    Cancelled,
    Other,
}
impl RecentEdgeStatus {
    pub(crate) fn from_provider(value: &str) -> Self {
        match value {
            "running" => Self::Running,
            "completed" => Self::Completed,
            "failed" => Self::Failed,
            "cancelled" => Self::Cancelled,
            _ => Self::Other,
        }
    }
}
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct RecentSpawnEdge {
    pub parent_thread_id: String,
    pub child_thread_id: String,
    pub status: RecentEdgeStatus,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct RecentIndexThread {
    pub id: String,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
    pub archived: bool,
    pub scope: RecentScopeStatus,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct RecentIndexSnapshot {
    pub max_migration: i64,
    pub window_anchor_ms: i64,
    pub cutoff_ms: i64,
    pub backup_max_updated_at_ms: Option<i64>,
    pub freshness: IndexFreshness,
    // Scope-wide index count. Status totals will cover the selected page only.
    pub candidate_count: usize,
    pub scope_ambiguous_count: usize,
    pub scope_unknown_count: usize,
    pub threads: Vec<RecentIndexThread>,
    pub spawn_edges: Vec<RecentSpawnEdge>,
}

/// Explicit full-history import projection. Paths remain untrusted locators;
/// only the existing descriptor/header import authorization can read a source.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ImportScopeQuery {
    pub version: u8,
    pub root: PathBuf,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root_identity: Option<(u64, u64)>,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ImportSourceState {
    Eligible,
    InvalidPath,
    ScopeUnproven,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct ImportIndexThread {
    pub id: String,
    pub archived: bool,
    pub scope: RecentScopeStatus,
    pub source_state: ImportSourceState,
    pub rollout_path: Option<PathBuf>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct ImportIndexSnapshot {
    pub max_migration: i64,
    pub candidate_count: usize,
    pub scope_ambiguous_count: usize,
    pub scope_unknown_count: usize,
    pub threads: Vec<ImportIndexThread>,
    // Projection only. Import commands do not pass these edges to a writer.
    pub spawn_edges: Vec<RecentSpawnEdge>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(tag = "status", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum ImportIndexOutcome {
    Available { snapshot: ImportIndexSnapshot },
    Unavailable { reason: IndexUnavailableReason },
}

/// Fixed diagnostics only; SQL errors, paths and source values never cross
/// the helper boundary as an error message.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, thiserror::Error, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum IndexUnavailableReason {
    #[error("index schema is unsupported")]
    SchemaUnsupported,
    #[error("index migration history is unsupported")]
    MigrationUnsupported,
    #[error("index backup integrity check failed")]
    IntegrityFailed,
    #[error("index scope root cannot be proved")]
    ScopeRootUnproven,
    #[error("index generated schema is unsupported")]
    GeneratedSchema,
    #[error("index millisecond migration is unavailable")]
    TimestampMigration,
    #[error("index millisecond schema is unavailable")]
    TimestampSchema,
    #[error("index millisecond timestamps are invalid")]
    TimestampInvalid,
    #[error("index timestamps exceed the clock skew allowance")]
    FutureSkew,
    #[error("recent index thread limit exceeded")]
    RowCap,
    #[error("recent index row validation failed")]
    RowInvalid,
    #[error("recent index text limit exceeded")]
    TextCap,
    #[error("recent index edge limit exceeded")]
    EdgeCap,
    #[error("recent index edge text limit exceeded")]
    EdgeTextCap,
}

impl IndexUnavailableReason {
    pub(crate) fn reason_key(self) -> &'static str {
        match self {
            Self::SchemaUnsupported => "schema_unsupported",
            Self::MigrationUnsupported => "migration_unsupported",
            Self::IntegrityFailed => "integrity_failed",
            Self::ScopeRootUnproven => "scope_root_unproven",
            Self::GeneratedSchema => "generated_schema",
            Self::TimestampMigration => "timestamp_migration",
            Self::TimestampSchema => "timestamp_schema",
            Self::TimestampInvalid => "timestamp_invalid",
            Self::FutureSkew => "future_skew",
            Self::RowCap => "row_cap",
            Self::RowInvalid => "row_invalid",
            Self::TextCap => "text_cap",
            Self::EdgeCap => "edge_cap",
            Self::EdgeTextCap => "edge_text_cap",
        }
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(tag = "status", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum RecentIndexOutcome {
    Available { snapshot: RecentIndexSnapshot },
    Unavailable { reason: IndexUnavailableReason },
}

pub(crate) struct RecentRawThread {
    pub id: String,
    pub cwd: String,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
    pub archived: bool,
}

pub(crate) struct RecentRawSnapshot {
    pub max_migration: i64,
    pub window_anchor_ms: i64,
    pub cutoff_ms: i64,
    pub backup_max_updated_at_ms: Option<i64>,
    pub freshness: IndexFreshness,
    pub threads: Vec<RecentRawThread>,
    pub spawn_edges: Vec<IndexSpawnEdge>,
}

/// Only SQL/data projection lives here; the IO owner proves directory scope
/// afterwards. Filtering happens before the row cap, never after truncation.
pub(crate) async fn query_recent_snapshot(
    connection: &mut SqliteConnection,
    query: &RecentScopeQuery,
    end: Instant,
) -> Result<RecentRawSnapshot> {
    ensure!(query.version == 1, "unsupported recent index query version");
    let cutoff = query
        .now_ms
        .checked_sub(RECENT_WINDOW_MS)
        .filter(|value| *value >= 0)
        .context("invalid recent index window")?;
    let latest = query
        .now_ms
        .checked_add(CLOCK_SKEW_MS)
        .context("invalid recent index clock")?;
    // Preserve IO, corruption and deadline errors instead of labelling every
    // failed integrity/transaction check as an unsupported schema.
    let (mut tx, max_migration) = begin_snapshot(connection, end).await?;
    let checksum: Option<Vec<u8>> = sea_orm::sqlx::query_scalar(
        "SELECT substr(checksum,1,49) FROM _sqlx_migrations WHERE version=25 AND success=1",
    )
    .fetch_optional(&mut *tx)
    .await?;
    ensure!(
        checksum.is_some_and(|bytes| hex::encode(bytes) == TIMESTAMP_MIGRATION),
        IndexUnavailableReason::TimestampMigration
    );
    let columns = sea_orm::sqlx::query(
        "SELECT name,type,hidden FROM pragma_table_xinfo('threads') WHERE name IN ('created_at_ms','updated_at_ms')",
    ).fetch_all(&mut *tx).await?;
    ensure!(
        columns.len() == 2
            && columns.iter().all(|row| row
                .try_get::<String, _>("type")
                .is_ok_and(|kind| kind.eq_ignore_ascii_case("INTEGER"))
                && row
                    .try_get::<i64, _>("hidden")
                    .is_ok_and(|hidden| hidden == 0)),
        IndexUnavailableReason::TimestampSchema
    );
    // Run across the entire table before the date predicate: NULL/TEXT/REAL
    // timestamps must never disappear into a false empty recent window.
    let invalid: i64 = sea_orm::sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM threads WHERE typeof(created_at_ms)!='integer' OR typeof(updated_at_ms)!='integer' OR created_at_ms<0 OR updated_at_ms<0)",
    ).fetch_one(&mut *tx).await?;
    ensure!(invalid == 0, IndexUnavailableReason::TimestampInvalid);
    let future: i64 = sea_orm::sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM threads WHERE created_at_ms>? OR updated_at_ms>?)",
    )
    .bind(latest)
    .bind(latest)
    .fetch_one(&mut *tx)
    .await?;
    ensure!(future == 0, IndexUnavailableReason::FutureSkew);
    let maximum: Option<i64> =
        sea_orm::sqlx::query_scalar("SELECT MAX(updated_at_ms) FROM threads")
            .fetch_one(&mut *tx)
            .await?;
    let freshness = match maximum {
        None => IndexFreshness::EmptySnapshot,
        Some(value) if value < cutoff => IndexFreshness::OutsideWindow,
        Some(_) => IndexFreshness::WithinWindow,
    };
    // SQLite can synthesize a large constant column DEFAULT for many small
    // physical rows. Check only lengths/types and bounded sort keys first;
    // a physical-file limit alone does not bound SQLx's decoded row allocation.
    let sizes = sea_orm::sqlx::query(
        "SELECT COUNT(*) AS n,COALESCE(SUM(bad_row),0) AS invalid,COALESCE(SUM(text_bytes),0) AS bytes FROM (
            SELECT CASE WHEN typeof(id)='text' AND octet_length(id) BETWEEN 1 AND 64
                AND typeof(cwd)='text' AND octet_length(cwd) BETWEEN 1 AND 4096
                AND typeof(archived)='integer' AND archived IN (0,1) THEN 0 ELSE 1 END AS bad_row,
                octet_length(id)+octet_length(cwd) AS text_bytes
            FROM threads WHERE updated_at_ms>=? AND updated_at_ms<=?
            ORDER BY updated_at_ms DESC,substr(id,1,65) ASC LIMIT 10001
        )",
    ).bind(cutoff).bind(latest).fetch_one(&mut *tx).await?;
    ensure!(
        sizes.try_get::<i64, _>("n")? <= ROW_CAP as i64,
        IndexUnavailableReason::RowCap
    );
    ensure!(
        sizes.try_get::<i64, _>("invalid")? == 0,
        IndexUnavailableReason::RowInvalid
    );
    ensure!(
        sizes.try_get::<i64, _>("bytes")? <= 1024 * 1024,
        IndexUnavailableReason::TextCap
    );
    let rows = sea_orm::sqlx::query(
        "SELECT id,cwd,archived,created_at_ms,updated_at_ms FROM threads WHERE updated_at_ms>=? AND updated_at_ms<=? ORDER BY updated_at_ms DESC,id ASC LIMIT 10001",
    ).bind(cutoff).bind(latest).fetch_all(&mut *tx).await?;
    ensure!(rows.len() <= ROW_CAP, IndexUnavailableReason::RowCap);
    let mut threads = Vec::with_capacity(rows.len());
    let mut text_size = 0usize;
    let mut seen_ids = std::collections::HashSet::new();
    for row in rows {
        ensure!(Instant::now() < end, "recent index deadline expired");
        let id: String = row
            .try_get("id")
            .context(IndexUnavailableReason::RowInvalid)?;
        ensure!(
            seen_ids.insert(id.clone()),
            IndexUnavailableReason::RowInvalid
        );
        let cwd: String = row
            .try_get("cwd")
            .context(IndexUnavailableReason::RowInvalid)?;
        let archived: i64 = row
            .try_get("archived")
            .context(IndexUnavailableReason::RowInvalid)?;
        ensure!(
            valid_session(&id) && matches!(archived, 0 | 1),
            IndexUnavailableReason::RowInvalid
        );
        ensure!(
            !cwd.is_empty() && cwd.len() <= 4096 && !cwd.contains('\0'),
            IndexUnavailableReason::RowInvalid
        );
        text_size += id.len() + cwd.len();
        ensure!(text_size <= 1024 * 1024, IndexUnavailableReason::TextCap);
        threads.push(RecentRawThread {
            id,
            cwd,
            archived: archived == 1,
            created_at_ms: row
                .try_get("created_at_ms")
                .context(IndexUnavailableReason::RowInvalid)?,
            updated_at_ms: row
                .try_get("updated_at_ms")
                .context(IndexUnavailableReason::RowInvalid)?,
        });
    }
    let sizes = sea_orm::sqlx::query(
        "SELECT COUNT(*) AS n,COALESCE(SUM(bad_row),0) AS invalid,COALESCE(SUM(text_bytes),0) AS bytes FROM (
            SELECT CASE WHEN typeof(e.parent_thread_id)='text' AND octet_length(e.parent_thread_id) BETWEEN 1 AND 64
                AND typeof(e.child_thread_id)='text' AND octet_length(e.child_thread_id) BETWEEN 1 AND 64
                AND typeof(e.status)='text' AND octet_length(e.status) BETWEEN 1 AND 128 THEN 0 ELSE 1 END AS bad_row,
                octet_length(e.parent_thread_id)+octet_length(e.child_thread_id)+octet_length(e.status) AS text_bytes
            FROM thread_spawn_edges e JOIN threads t ON t.id=e.child_thread_id
            WHERE t.updated_at_ms>=? AND t.updated_at_ms<=?
            ORDER BY substr(e.child_thread_id,1,65),substr(e.parent_thread_id,1,65) LIMIT 10001
        )",
    ).bind(cutoff).bind(latest).fetch_one(&mut *tx).await?;
    ensure!(
        sizes.try_get::<i64, _>("n")? <= ROW_CAP as i64,
        IndexUnavailableReason::EdgeCap
    );
    ensure!(
        sizes.try_get::<i64, _>("invalid")? == 0,
        IndexUnavailableReason::RowInvalid
    );
    ensure!(
        sizes.try_get::<i64, _>("bytes")? <= 1024 * 1024,
        IndexUnavailableReason::EdgeTextCap
    );
    let rows = sea_orm::sqlx::query(
        "SELECT e.parent_thread_id,e.child_thread_id,e.status FROM thread_spawn_edges e JOIN threads t ON t.id=e.child_thread_id WHERE t.updated_at_ms>=? AND t.updated_at_ms<=? ORDER BY e.child_thread_id,e.parent_thread_id LIMIT 10001",
    ).bind(cutoff).bind(latest).fetch_all(&mut *tx).await?;
    ensure!(rows.len() <= ROW_CAP, IndexUnavailableReason::EdgeCap);
    let mut spawn_edges = Vec::with_capacity(rows.len());
    let mut edge_text_size = 0usize;
    for row in rows {
        ensure!(Instant::now() < end, "recent index edge deadline expired");
        let parent_thread_id: String = row
            .try_get("parent_thread_id")
            .context(IndexUnavailableReason::RowInvalid)?;
        let child_thread_id: String = row
            .try_get("child_thread_id")
            .context(IndexUnavailableReason::RowInvalid)?;
        let status: String = row
            .try_get("status")
            .context(IndexUnavailableReason::RowInvalid)?;
        ensure!(
            valid_session(&parent_thread_id)
                && valid_session(&child_thread_id)
                && !status.is_empty()
                && status.len() <= 128
                && !status.contains('\0'),
            IndexUnavailableReason::RowInvalid
        );
        edge_text_size += parent_thread_id.len() + child_thread_id.len() + status.len();
        ensure!(
            edge_text_size <= 1024 * 1024,
            IndexUnavailableReason::EdgeTextCap
        );
        spawn_edges.push(IndexSpawnEdge {
            parent_thread_id,
            child_thread_id,
            status,
        });
    }
    ensure!(
        Instant::now() < end,
        "recent index snapshot deadline expired"
    );
    tx.commit().await?;
    Ok(RecentRawSnapshot {
        max_migration,
        window_anchor_ms: query.now_ms,
        cutoff_ms: cutoff,
        backup_max_updated_at_ms: maximum,
        freshness,
        threads,
        spawn_edges,
    })
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct LocateRequest {
    session_id: String,
    remaining_ms: u64,
}
#[derive(Deserialize, Serialize)]
#[serde(tag = "status", deny_unknown_fields)]
enum LocateResponse {
    Available { relative: String },
    Missing,
    Unavailable,
}
fn valid_session(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 64
        && id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}
fn rollout_leaf_matches_id(name: &str, id: &str) -> bool {
    name.starts_with("rollout-") && name.ends_with(&format!("-{id}.jsonl"))
}

/// Pure shape bound for a locator supplied by index data, before any source IO.
/// This shares only the leaf predicate with the live locator, not its layout.
pub(crate) fn valid_import_rollout_path(path: &Path, id: &str) -> bool {
    if !valid_session(id)
        || !path.is_absolute()
        || !path
            .to_str()
            .is_some_and(|text| text.len() <= 4096 && !text.contains('\0'))
    {
        return false;
    }
    let mut components = path.components();
    #[cfg(windows)]
    {
        use std::path::Prefix;
        if !matches!(components.next(), Some(Component::Prefix(prefix))
            if matches!(prefix.kind(), Prefix::Disk(_) | Prefix::VerbatimDisk(_)))
        {
            return false;
        }
    }
    if !matches!(components.next(), Some(Component::RootDir))
        || !components.all(|component| matches!(component, Component::Normal(_)))
    {
        return false;
    }
    path.file_name()
        .and_then(|leaf| leaf.to_str())
        .is_some_and(|leaf| rollout_leaf_matches_id(leaf, id))
}

fn valid_relative(relative: &str, id: &str) -> bool {
    let path = Path::new(relative);
    relative.len() <= 4096
        && !relative.is_empty()
        && !relative.contains('\\')
        && path.components().count() == 4
        && path
            .components()
            .all(|part| matches!(part, Component::Normal(_)))
        && relative
            .split('/')
            .all(|part| !part.is_empty() && part != "." && part != "..")
        && path
            .file_name()
            .and_then(|p| p.to_str())
            .is_some_and(|name| rollout_leaf_matches_id(name, id))
}
pub(crate) async fn locate_live(
    id: &str,
    deadline: Option<Instant>,
) -> Result<Option<PathBuf>, LiveCandidateUnavailable> {
    let deadline = deadline
        .filter(|d| *d > Instant::now())
        .ok_or(LiveCandidateUnavailable)?;
    if !valid_session(id) {
        return Err(LiveCandidateUnavailable);
    }
    #[cfg(all(test, feature = "test-live-agent"))]
    let _ = local_tests::LOCATOR_IO.try_with(|calls| {
        calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    });
    let root = configured_provider_roots(agent_for(AgentKind::Codex))
        .into_iter()
        .next()
        .ok_or(LiveCandidateUnavailable)?;
    let remaining_ms = deadline
        .saturating_duration_since(Instant::now())
        .as_millis()
        .min(60_000) as u64;
    if remaining_ms == 0 {
        return Err(LiveCandidateUnavailable);
    }
    let request = serde_json::to_vec(&LocateRequest {
        session_id: id.to_owned(),
        remaining_ms,
    })
    .map_err(|_| LiveCandidateUnavailable)?;
    let mut command = authorized_read::registered_helper_command(AUTHORIZED_READ_HELPER_ARG)
        .ok_or(LiveCandidateUnavailable)?;
    let home = std::env::var_os("LIBRA_TEST_HOME")
        .map(PathBuf::from)
        .or_else(dirs::home_dir)
        .unwrap_or_else(|| root.clone());
    command
        .env_clear()
        .current_dir(Path::new("/"))
        .env("CODEX_HOME", &root)
        .env("HOME", &home)
        .env("USERPROFILE", &home)
        .env(AUTHORIZED_READ_HELPER_CAP_ENV, CONTROL_CAP.to_string())
        .env(AUTHORIZED_READ_HELPER_MODE_ENV, LOCATE_MODE);
    let bytes = match authorized_read::run_registered_bounded_helper_until(
        command,
        &[&request],
        CONTROL_CAP,
        deadline,
    )
    .await
    {
        RegisteredHelperOutput::Output(bytes) => bytes,
        _ => return Err(LiveCandidateUnavailable),
    };
    match serde_json::from_slice(&bytes).map_err(|_| LiveCandidateUnavailable)? {
        LocateResponse::Available { relative } if valid_relative(&relative, id) => {
            Ok(Some(root.join("sessions").join(relative)))
        }
        LocateResponse::Missing => Ok(None),
        _ => Err(LiveCandidateUnavailable),
    }
}
/// Called only by the main binary's fixed private helper dispatch.
pub(crate) fn run_locator_helper(cap: u64) -> i32 {
    let response = (|| {
        if cap != CONTROL_CAP {
            return None;
        }
        let mut stdin = std::io::stdin().lock();
        let bytes = match authorized_read::read_strictly_bounded(&mut stdin, CONTROL_CAP) {
            authorized_read::StrictBoundedRead::Complete(bytes) => bytes,
            _ => return None,
        };
        let request: LocateRequest = serde_json::from_slice(&bytes).ok()?;
        if !valid_session(&request.session_id) || !(1..=60_000).contains(&request.remaining_ms) {
            return None;
        }
        let root = configured_provider_roots(agent_for(AgentKind::Codex))
            .into_iter()
            .next()?
            .join("sessions");
        let mut limits = super::builtin::stable_promoted::RolloutDiscoveryLimits::default();
        limits.deadline = limits
            .deadline
            .min(Duration::from_millis(request.remaining_ms));
        let path = super::builtin::stable_promoted::find_codex_rollout_bounded(
            &request.session_id,
            limits,
        )
        .ok()?;
        Some(match path {
            Some(path) => {
                let path = super::transcript_source::normalize_macos_system_directory_alias(&path);
                let relative = path
                    .strip_prefix(root)
                    .ok()?
                    .components()
                    .map(|part| match part {
                        Component::Normal(name) => name.to_str(),
                        _ => None,
                    })
                    .collect::<Option<Vec<_>>>()?
                    .join("/");
                if !valid_relative(&relative, &request.session_id) {
                    return None;
                }
                LocateResponse::Available { relative }
            }
            None => LocateResponse::Missing,
        })
    })()
    .unwrap_or(LocateResponse::Unavailable);
    match serde_json::to_vec(&response) {
        Ok(bytes) if bytes.len() as u64 <= CONTROL_CAP => {
            if std::io::stdout().lock().write_all(&bytes).is_ok() {
                0
            } else {
                1
            }
        }
        _ => 2,
    }
}

#[cfg(test)]
#[cfg(feature = "test-live-agent")]
mod local_tests {
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

    use super::*;
    tokio::task_local! { pub(super) static LOCATOR_IO: Arc<AtomicUsize>; }

    #[tokio::test]
    #[ignore = "operator-local Codex acceptance; never run in CI"]
    async fn codex_index_no_deadline_never_reads_parent() {
        use crate::internal::ai::{
            capture::snapshot::{
                CaptureSnapshotPartialReason, CaptureSnapshotPolicy, CaptureSnapshotService,
            },
            observed_agents::{
                AgentSessionCtx, ObservedAgent,
                live_capture::{LiveCaptureBinding, LiveCaptureContext},
            },
        };
        struct DenyParent(Arc<AtomicUsize>);
        impl ObservedAgent for DenyParent {
            fn provider_kind(&self) -> AgentKind {
                AgentKind::Codex
            }
            fn provider_name(&self) -> &'static str {
                "codex"
            }
            fn read_transcript(&self, _: &AgentSessionCtx) -> anyhow::Result<Option<Vec<u8>>> {
                self.0.fetch_add(1, Ordering::SeqCst);
                anyhow::bail!("parent source reads forbidden by test");
            }
            fn protected_dirs(&self) -> &'static [&'static str] {
                self.0.fetch_add(1, Ordering::SeqCst);
                &[]
            }
        }
        let io = Arc::new(AtomicUsize::new(0));
        let reads = Arc::new(AtomicUsize::new(0));
        let adapter = DenyParent(reads.clone());
        let binding = LiveCaptureBinding::resolve(agent_for(AgentKind::Codex).as_hooks().unwrap());
        let capability = binding.live_transcript_locator().unwrap();
        let context = LiveCaptureContext {
            libra_session_id: "codex__no-budget",
            provider_session_id: "no-budget",
            verified_cwd: Path::new("/work/repo"),
        };
        LOCATOR_IO
            .scope(io.clone(), async {
                // Production Codex policy stays compatible with the frozen None
                // host-budget contract. Managed hooks pass their explicit budget.
                assert_eq!(binding.missing_host_capture_budget(), None);
                assert!(capability.locate(&context, None).await.is_err());
                for path in [None, Some(PathBuf::from("/must-not-open/source.jsonl"))] {
                    let ctx = AgentSessionCtx {
                        session_id: "codex__no-budget".into(),
                        provider_session_id: "no-budget".into(),
                        working_dir: PathBuf::from("/work/repo"),
                        transcript_path: path.clone(),
                    };
                    let result = CaptureSnapshotService::capture_live_with_source_until(
                        &adapter,
                        &ctx,
                        CaptureSnapshotPolicy::with_deadline(None),
                        capability,
                    )
                    .await;
                    assert_eq!(
                        result.partial_reason(),
                        Some(if path.is_none() {
                            CaptureSnapshotPartialReason::SourceAbsent
                        } else {
                            CaptureSnapshotPartialReason::SourceAuthorizationMismatch
                        })
                    );
                }
            })
            .await;
        assert_eq!(
            io.load(Ordering::SeqCst),
            0,
            "locator root access or helper launch attempted"
        );
        assert_eq!(
            reads.load(Ordering::SeqCst),
            0,
            "resolver/parent raw fallback was reached"
        );
    }
}
