//! Explicit repository-local issuer enrollment. Neither wire data nor a
//! catalog match grants receipt, field or artifact authority.

use std::{collections::BTreeSet, path::Path, time::Duration};

use anyhow::{Context, Result, bail};
use sea_orm::{ConnectionTrait, Statement};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use sha2::{Digest, Sha256};
use thiserror::Error;

use super::{approved_build, attest, codec};
use crate::{
    internal::{
        ai::observed_agents::{coverage::parse_canon_value, trust},
        config::ConfigKv,
        db,
    },
    utils::util::{DATABASE, try_get_request_paths},
};

pub const REGISTRY_KEY: &str = "agent.native_origin.opencode.v1";
pub const ISSUER_CAP: usize = 128;
pub const REGISTRY_BYTE_CAP: usize = 65536;

/// All diagnostics are fixed strings; untrusted descriptors and child output
/// are never interpolated into an error or a log.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum NativeOriginError {
    #[error(
        "Invalid native-origin installation descriptor; use a new owned private installation with the reviewed producer"
    )]
    Descriptor,
    #[error(
        "OpenCode exporter trust is missing or changed; explicitly trust the approved private binary before registering an issuer"
    )]
    PriorTrust,
    #[error(
        "Native-origin observer binding differs from this Libra installation; enable the reviewed OpenCode observer and prepare a matching private installation"
    )]
    Observer,
    #[error(
        "Native-origin challenge failed; keep capture partial and check the approved private producer and its owned installation"
    )]
    Challenge,
    #[error(
        "Native-origin child exceeded its 30 second or output budget; keep capture partial and check the private producer"
    )]
    ChildBounds,
    #[error(
        "Invalid native-origin registry row, spelling, version or records; preserve the registry and repair its configuration before retrying"
    )]
    Registry,
    #[error(
        "Native-origin registry epoch changed; reread its epoch and retry the explicit registration with that expected epoch"
    )]
    Epoch,
    #[error(
        "Native-origin registry reached its 64 KiB or 128 issuer limit; preserve all revoked records and use a separately configured repository"
    )]
    Capacity,
    #[error(
        "Native-origin issuer already exists; do not reactivate an old key, prepare a new private installation for rotation"
    )]
    ExistingIssuer,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum IssuerState {
    Active,
    VerifyOnly,
    Revoked,
}

/// Persisted public key and local installation binding. These are still data:
/// later verification must check current trust, state and receipt coordinates.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct IssuerRecord {
    pub issuer: String,
    pub public_key: String,
    pub key_generation: u32,
    pub build_identity: String,
    pub producer_version: String,
    pub producer_binary_digest: String,
    pub producer_path: String,
    pub policy_descriptor: codec::Policy,
    pub policy_digest: String,
    pub state: IssuerState,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Registry {
    pub version: u32,
    pub epoch: u64,
    pub issuers: Vec<IssuerRecord>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Registration {
    pub epoch: u64,
    pub issuer: String,
    pub key_generation: u32,
    pub policy_digest: String,
    pub observer_executable_sha256: String,
}

/// Minimal JSON spelling, without imposing sorted object-member order.
/// The producer's owned-file parser requires JSON.stringify(parsed) == input;
/// every number here is an integer and every string uses JSON's minimal
/// escaping. Serde then applies the closed field/type grammar to original bytes.
pub(super) fn decode_owned<T: DeserializeOwned>(bytes: &[u8]) -> Result<T> {
    if bytes.len() > REGISTRY_BYTE_CAP || parse_canon_value(bytes).is_err() {
        return Err(NativeOriginError::Descriptor.into());
    }
    let mut pos = 0;
    while pos < bytes.len() {
        match bytes[pos] {
            b'"' => {
                let start = pos;
                pos += 1;
                loop {
                    let byte = *bytes.get(pos).ok_or(NativeOriginError::Descriptor)?;
                    pos += 1;
                    if byte == b'\\' {
                        pos += 1;
                    } else if byte == b'"' {
                        break;
                    }
                }
                let raw = bytes.get(start..pos).ok_or(NativeOriginError::Descriptor)?;
                let text: String =
                    serde_json::from_slice(raw).map_err(|_| NativeOriginError::Descriptor)?;
                if serde_json::to_vec(&text)? != raw {
                    return Err(NativeOriginError::Descriptor.into());
                }
            }
            byte if byte.is_ascii_whitespace() => {
                return Err(NativeOriginError::Descriptor.into());
            }
            b'-' | b'.' | b'+' => return Err(NativeOriginError::Descriptor.into()),
            b'0'..=b'9' => {
                let start = pos;
                while bytes.get(pos).is_some_and(u8::is_ascii_digit) {
                    pos += 1;
                }
                if bytes.get(pos).is_some_and(|b| matches!(b, b'e' | b'E'))
                    || (pos - start > 1 && bytes[start] == b'0')
                {
                    return Err(NativeOriginError::Descriptor.into());
                }
            }
            _ => pos += 1,
        }
    }
    serde_json::from_slice(bytes).map_err(|_| NativeOriginError::Descriptor.into())
}

pub(super) fn lower_hex(value: &str, bytes: usize) -> Result<Vec<u8>> {
    if value.len() != bytes * 2
        || !value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err(NativeOriginError::Descriptor.into());
    }
    hex::decode(value).map_err(|_| NativeOriginError::Descriptor.into())
}

impl Registry {
    fn validate(&self) -> Result<()> {
        if self.version != 1 || self.epoch == 0 || self.issuers.is_empty() {
            return Err(NativeOriginError::Registry.into());
        }
        if self.issuers.len() > ISSUER_CAP {
            return Err(NativeOriginError::Capacity.into());
        }
        let mut seen = BTreeSet::new();
        let mut active = 0;
        for record in &self.issuers {
            let key = lower_hex(&record.public_key, 32)?;
            if record.issuer != hex::encode(Sha256::digest(&key))
                || !seen.insert(&record.issuer)
                || record.key_generation == 0
                || !Path::new(&record.producer_path).is_absolute()
                || record.producer_path.len() > 4096
                || record.producer_path.chars().any(char::is_control)
            {
                return Err(NativeOriginError::Registry.into());
            }
            let build = approved_build::lookup(
                &record.producer_version,
                &record.build_identity,
                &record.producer_binary_digest,
            )?;
            let rendered = build.render_observer(&record.policy_descriptor.libra_command_path)?;
            let expected = build.installation_policy(
                &record.policy_descriptor.libra_command_path,
                rendered.as_bytes(),
            )?;
            if record.policy_descriptor != expected
                || record.policy_digest
                    != hex::encode(Sha256::digest(codec::policy_bytes(&expected)?))
            {
                return Err(NativeOriginError::Registry.into());
            }
            if record.state == IssuerState::Active {
                active += 1;
            }
        }
        if active > 1 {
            return Err(NativeOriginError::Registry.into());
        }
        Ok(())
    }
}

/// Metadata is fetched before allocating a value. ASCII key aliases and
/// duplicate physical rows are refused, including a lone noncanonical alias.
/// Bound equality avoids the generic LIKE wildcard pitfall fixed by NPR-19.
pub async fn read_with_conn<C: ConnectionTrait>(conn: &C) -> Result<Option<Registry>> {
    let backend = conn.get_database_backend();
    let rows = conn
        .query_all_raw(Statement::from_sql_and_values(
            backend,
            "SELECT id,key,encrypted,length(CAST(value AS BLOB)) AS value_bytes FROM config_kv WHERE lower(key)=lower(?) ORDER BY id LIMIT 2",
            [REGISTRY_KEY.into()],
        ))
        .await
        .context("inspect reserved native-origin registry rows")?;
    if rows.is_empty() {
        return Ok(None);
    }
    if rows.len() != 1 {
        return Err(NativeOriginError::Registry.into());
    }
    let row = &rows[0];
    let id: i64 = row.try_get("", "id")?;
    let key: String = row.try_get("", "key")?;
    let encrypted: i32 = row.try_get("", "encrypted")?;
    let bytes: i64 = row.try_get("", "value_bytes")?;
    if key != REGISTRY_KEY || encrypted != 0 || bytes < 0 {
        return Err(NativeOriginError::Registry.into());
    }
    if bytes > REGISTRY_BYTE_CAP as i64 {
        return Err(NativeOriginError::Capacity.into());
    }
    let row = conn
        .query_one_raw(Statement::from_sql_and_values(
            backend,
            "SELECT value FROM config_kv WHERE id=? AND key=? AND encrypted=0 AND length(CAST(value AS BLOB))<=?",
            [id.into(), REGISTRY_KEY.into(), (REGISTRY_BYTE_CAP as i64).into()],
        ))
        .await
        .context("read bounded native-origin registry value")?
        .ok_or(NativeOriginError::Registry)?;
    let value: String = row.try_get("", "value")?;
    let registry: Registry =
        decode_owned(value.as_bytes()).map_err(|_| NativeOriginError::Registry)?;
    registry.validate()?;
    Ok(Some(registry))
}

async fn commit_registration<C: ConnectionTrait>(
    conn: &C,
    prepared: &attest::PreparedRegistration,
    expected_epoch: u64,
) -> Result<Registration> {
    let current = trust::read_trust_with_conn(conn, trust::OPENCODE_EXPORTER_SLUG).await?;
    let dirs = trust::read_trusted_dirs_with_conn(conn).await?;
    if current.as_ref() != Some(&prepared.exporter_trust)
        || !trust::path_within_trusted_dirs(&prepared.exporter_trust.path, &dirs)
    {
        return Err(NativeOriginError::PriorTrust.into());
    }
    let previous = read_with_conn(conn).await?;
    let epoch = previous.as_ref().map_or(0, |registry| registry.epoch);
    if epoch != expected_epoch {
        return Err(NativeOriginError::Epoch.into());
    }
    let mut registry = previous.unwrap_or(Registry {
        version: 1,
        epoch: 0,
        issuers: Vec::new(),
    });
    if registry
        .issuers
        .iter()
        .any(|record| record.issuer == prepared.record.issuer)
    {
        return Err(NativeOriginError::ExistingIssuer.into());
    }
    if registry.issuers.len() >= ISSUER_CAP {
        return Err(NativeOriginError::Capacity.into());
    }
    for record in &mut registry.issuers {
        if record.state == IssuerState::Active {
            record.state = IssuerState::VerifyOnly;
        }
    }
    registry.epoch = epoch.checked_add(1).ok_or(NativeOriginError::Epoch)?;
    registry.issuers.push(prepared.record.clone());
    registry.validate()?;
    let value = serde_json::to_vec(&registry).context("encode complete native-origin registry")?;
    if value.len() > REGISTRY_BYTE_CAP {
        return Err(NativeOriginError::Capacity.into());
    }
    let text = std::str::from_utf8(&value).context("encode native-origin registry text")?;
    ConfigKv::set_with_conn(conn, REGISTRY_KEY, text, false)
        .await
        .context("persist native-origin registry on registration connection")?;
    Ok(Registration {
        epoch: registry.epoch,
        issuer: prepared.record.issuer.clone(),
        key_generation: prepared.record.key_generation,
        policy_digest: prepared.record.policy_digest.clone(),
        observer_executable_sha256: prepared.observer_executable_sha256.clone(),
    })
}

/// Explicit CLI owner. Handshake does not hold a database writer lock. The
/// final same-connection trust reread, epoch CAS and one row write linearize
/// only after the existing write-lock-first transaction has been acquired.
pub async fn register(manifest: &Path, expected_epoch: u64) -> Result<Registration> {
    let (storage, worktree, _) = try_get_request_paths(None)
        .context("native-origin registration requires a Libra repository")?;
    let conn = db::get_db_conn_instance_for_path(&storage.join(DATABASE))
        .await
        .context("open repository for native-origin registration")?;
    // A preflight refusal avoids spawning a producer for corrupt state. This
    // read never substitutes for the transaction's authoritative reread.
    let before = read_with_conn(&conn).await?;
    if before.as_ref().map_or(0, |registry| registry.epoch) != expected_epoch {
        return Err(NativeOriginError::Epoch.into());
    }
    let prepared = attest::prepare(&conn, &worktree, manifest).await?;
    let txn = tokio::time::timeout(Duration::from_secs(5), db::begin_write_transaction(&conn))
        .await
        .context("native-origin registry writer is busy; reread the epoch and retry")?
        .context("acquire native-origin registry writer lock; reread the epoch and retry")?;
    match commit_registration(&txn, &prepared, expected_epoch).await {
        Ok(result) => {
            txn.commit().await.context(
                "commit native-origin registration; inspect the registry epoch before retrying",
            )?;
            Ok(result)
        }
        Err(primary) => {
            if let Err(rollback) = txn.rollback().await {
                bail!(
                    "native-origin registration refused: {primary}; rollback failed: {rollback}; inspect the repository before retrying"
                );
            }
            Err(primary)
        }
    }
}

#[cfg(test)]
mod tests {
    use sea_orm::{Database, DatabaseConnection, DbBackend};

    use super::*;

    async fn storage(url: &str) -> DatabaseConnection {
        let conn = Database::connect(url).await.unwrap();
        conn.execute_unprepared("PRAGMA busy_timeout=2000")
            .await
            .unwrap();
        conn.execute_unprepared("CREATE TABLE IF NOT EXISTS config_kv (id INTEGER PRIMARY KEY AUTOINCREMENT,key TEXT NOT NULL,value TEXT NOT NULL,encrypted INTEGER NOT NULL DEFAULT 0)").await.unwrap();
        conn
    }

    fn prepared() -> attest::PreparedRegistration {
        // Synthetic storage input only. It cannot pass the public registration
        // path, which measures an approved executable and verifies its challenge.
        let key = [7u8; 32];
        let build = approved_build::lookup(
            codec::PRODUCER_VERSION,
            "bac21cc4027896b8d1836133c347d20f",
            "547c36eb928da163ef61d019bad669860d519f2da0b67737c9a3920e831c9196",
        )
        .unwrap();
        let path = "/Applications/Libra.app/Contents/MacOS/libra";
        let rendered = build.render_observer(path).unwrap();
        let policy = build
            .installation_policy(path, rendered.as_bytes())
            .unwrap();
        attest::PreparedRegistration {
            exporter_trust: trust::TrustRecord {
                path: "/synthetic/producer/opencode".into(),
                sha256: "547c36eb928da163ef61d019bad669860d519f2da0b67737c9a3920e831c9196".into(),
                device: 1,
                inode: 1,
                mtime: 1,
            },
            record: IssuerRecord {
                issuer: hex::encode(Sha256::digest(key)),
                public_key: hex::encode(key),
                key_generation: 1,
                build_identity: "bac21cc4027896b8d1836133c347d20f".into(),
                producer_version: codec::PRODUCER_VERSION.into(),
                producer_binary_digest:
                    "547c36eb928da163ef61d019bad669860d519f2da0b67737c9a3920e831c9196".into(),
                producer_path: "/synthetic/producer/opencode".into(),
                policy_digest: hex::encode(Sha256::digest(codec::policy_bytes(&policy).unwrap())),
                policy_descriptor: policy,
                state: IssuerState::Active,
            },
            observer_executable_sha256: "a".repeat(64),
        }
    }

    async fn seed_trust(conn: &DatabaseConnection, input: &attest::PreparedRegistration) {
        ConfigKv::set_with_conn(
            conn,
            "agent.trust.opencode",
            &serde_json::to_string(&input.exporter_trust).unwrap(),
            false,
        )
        .await
        .unwrap();
        ConfigKv::set_with_conn(
            conn,
            trust::TRUSTED_DIRS_KEY,
            r#"["/synthetic/producer"]"#,
            false,
        )
        .await
        .unwrap();
    }

    #[test]
    fn npr_09_owned_json_grammar() {
        #[derive(Debug, Deserialize, PartialEq)]
        #[serde(deny_unknown_fields)]
        struct Fields {
            version: u32,
            text: String,
        }
        let reordered: Fields = decode_owned(br#"{"text":"unchanged","version":1}"#).unwrap();
        assert_eq!(
            reordered,
            Fields {
                version: 1,
                text: "unchanged".into()
            }
        );
        for wire in [
            br#"{"version":1,"text":"x","text":"y"}"#.as_slice(),
            br#"{"version":1,"te\u0078t":"x"}"#,
            br#"{"version":1.0,"text":"x"}"#,
            br#"{"version":1e0,"text":"x"}"#,
            br#"{"version":-0,"text":"x"}"#,
            br#"{"version":1, "text":"x"}"#,
            br#"{"version":1,"text":"\u0078"}"#,
            br#"{"version":1,"text":"x","extra":0}"#,
            b"{\"version\":1,\"text\":\"x\"}\n",
        ] {
            assert!(
                decode_owned::<Fields>(wire).is_err(),
                "invalid owned spelling must fail"
            );
        }
        assert!(decode_owned::<Fields>(&vec![b' '; 65537]).is_err());
    }

    #[tokio::test]
    async fn npr_09_registry_storage_guards() {
        let conn = storage("sqlite::memory:").await;
        assert!(read_with_conn(&conn).await.unwrap().is_none());
        let input = prepared();
        let registry = Registry {
            version: 1,
            epoch: 1,
            issuers: vec![input.record],
        };
        let valid = serde_json::to_string(&registry).unwrap();
        for (keys, value, encrypted) in [
            (vec![REGISTRY_KEY, REGISTRY_KEY], valid.as_str(), 0),
            (
                vec![REGISTRY_KEY, "AGENT.NATIVE_ORIGIN.OPENCODE.V1"],
                valid.as_str(),
                0,
            ),
            (vec!["Agent.Native_Origin.Opencode.V1"], valid.as_str(), 0),
            (vec![REGISTRY_KEY], valid.as_str(), 1),
            (vec![REGISTRY_KEY], "{}", 0),
            (vec![REGISTRY_KEY], "not json", 0),
        ] {
            conn.execute_unprepared("DELETE FROM config_kv")
                .await
                .unwrap();
            for key in &keys {
                conn.execute_raw(Statement::from_sql_and_values(
                    DbBackend::Sqlite,
                    "INSERT INTO config_kv(key,value,encrypted) VALUES (?,?,?)",
                    [(*key).into(), value.into(), encrypted.into()],
                ))
                .await
                .unwrap();
            }
            assert!(read_with_conn(&conn).await.is_err());
            let rows = conn
                .query_all_raw(Statement::from_string(
                    DbBackend::Sqlite,
                    "SELECT key,value,encrypted FROM config_kv ORDER BY id",
                ))
                .await
                .unwrap();
            assert_eq!(rows.len(), keys.len());
            for (row, key) in rows.iter().zip(keys) {
                assert_eq!(row.try_get::<String>("", "key").unwrap(), key);
                assert_eq!(row.try_get::<String>("", "value").unwrap(), value);
                assert_eq!(row.try_get::<i32>("", "encrypted").unwrap(), encrypted);
            }
        }
        conn.execute_unprepared("DELETE FROM config_kv")
            .await
            .unwrap();
        ConfigKv::set_with_conn(&conn, REGISTRY_KEY, &valid, false)
            .await
            .unwrap();
        assert_eq!(read_with_conn(&conn).await.unwrap(), Some(registry));
        ConfigKv::set_with_conn(&conn, REGISTRY_KEY, &"x".repeat(65537), false)
            .await
            .unwrap();
        assert_eq!(
            read_with_conn(&conn).await.unwrap_err().downcast_ref(),
            Some(&NativeOriginError::Capacity)
        );
        ConfigKv::set_with_conn(
            &conn,
            "agent.trust.opencode",
            r#"{"mtime":"sensitive-test-canary"}"#,
            false,
        )
        .await
        .unwrap();
        let error = trust::read_trust_with_conn(&conn, "opencode")
            .await
            .unwrap_err()
            .to_string();
        assert!(error.contains("corrupt"));
        assert!(
            !error.contains("sensitive-test-canary"),
            "invalid trust JSON must not echo a wire value"
        );
    }

    #[tokio::test]
    async fn npr_09_linearized_admission() {
        let root = tempfile::tempdir().unwrap();
        let url = format!(
            "sqlite:{}?mode=rwc",
            root.path().join("registry.db").display()
        );
        let conn = storage(&url).await;
        let other = storage(&url).await;
        let input = prepared();
        seed_trust(&conn, &input).await;
        ConfigKv::set_with_conn(&conn, "canary.untouched", "preserve", false)
            .await
            .unwrap();
        // Separate real connections contend for the same writer lock. The
        // second expected-0 writer sees the first committed epoch, not a stale
        // outside-transaction read or a nested pooled connection.
        let winner = db::begin_write_transaction(&conn).await.unwrap();
        let (started_tx, started_rx) = tokio::sync::oneshot::channel();
        let loser = tokio::spawn(async move {
            let input = prepared();
            started_tx.send(()).unwrap();
            let txn = db::begin_write_transaction(&other).await.unwrap();
            let result = commit_registration(&txn, &input, 0).await;
            txn.rollback().await.unwrap();
            result
        });
        started_rx.await.unwrap();
        assert_eq!(
            commit_registration(&winner, &input, 0).await.unwrap().epoch,
            1
        );
        winner.commit().await.unwrap();
        let error = tokio::time::timeout(Duration::from_secs(5), loser)
            .await
            .unwrap()
            .unwrap()
            .unwrap_err();
        assert_eq!(error.downcast_ref(), Some(&NativeOriginError::Epoch));
        let before = ConfigKv::get_with_conn(&conn, REGISTRY_KEY)
            .await
            .unwrap()
            .unwrap()
            .value;
        let txn = db::begin_write_transaction(&conn).await.unwrap();
        trust::revoke_trust_with_conn(&txn, trust::OPENCODE_EXPORTER_SLUG)
            .await
            .unwrap();
        txn.commit().await.unwrap();
        let txn = db::begin_write_transaction(&conn).await.unwrap();
        assert_eq!(
            commit_registration(&txn, &input, 1)
                .await
                .unwrap_err()
                .downcast_ref(),
            Some(&NativeOriginError::PriorTrust)
        );
        txn.rollback().await.unwrap();
        assert_eq!(
            ConfigKv::get_with_conn(&conn, REGISTRY_KEY)
                .await
                .unwrap()
                .unwrap()
                .value,
            before
        );
        assert_eq!(
            ConfigKv::get_with_conn(&conn, "canary.untouched")
                .await
                .unwrap()
                .unwrap()
                .value,
            "preserve"
        );
        seed_trust(&conn, &input).await;
        conn.execute_unprepared("CREATE TRIGGER reject_native_update BEFORE UPDATE ON config_kv WHEN NEW.key='agent.native_origin.opencode.v1' BEGIN SELECT RAISE(ABORT,'synthetic write fault'); END").await.unwrap();
        let mut new = prepared();
        new.record.public_key = hex::encode([8u8; 32]);
        new.record.issuer = hex::encode(Sha256::digest([8u8; 32]));
        let txn = db::begin_write_transaction(&conn).await.unwrap();
        assert!(commit_registration(&txn, &new, 1).await.is_err());
        txn.rollback().await.unwrap();
        assert_eq!(
            ConfigKv::get_with_conn(&conn, REGISTRY_KEY)
                .await
                .unwrap()
                .unwrap()
                .value,
            before
        );
    }

    #[test]
    fn npr_09_error_display() {
        assert_eq!(
            NativeOriginError::Epoch.to_string(),
            "Native-origin registry epoch changed; reread its epoch and retry the explicit registration with that expected epoch"
        );
        assert_eq!(
            NativeOriginError::PriorTrust.to_string(),
            "OpenCode exporter trust is missing or changed; explicitly trust the approved private binary before registering an issuer"
        );
        assert_eq!(
            NativeOriginError::ChildBounds.to_string(),
            "Native-origin child exceeded its 30 second or output budget; keep capture partial and check the private producer"
        );
        assert_eq!(
            NativeOriginError::Registry.to_string(),
            "Invalid native-origin registry row, spelling, version or records; preserve the registry and repair its configuration before retrying"
        );
        assert_eq!(
            NativeOriginError::Descriptor.to_string(),
            "Invalid native-origin installation descriptor; use a new owned private installation with the reviewed producer"
        );
        assert_eq!(
            NativeOriginError::Observer.to_string(),
            "Native-origin observer binding differs from this Libra installation; enable the reviewed OpenCode observer and prepare a matching private installation"
        );
        assert_eq!(
            NativeOriginError::Challenge.to_string(),
            "Native-origin challenge failed; keep capture partial and check the approved private producer and its owned installation"
        );
        assert_eq!(
            NativeOriginError::Capacity.to_string(),
            "Native-origin registry reached its 64 KiB or 128 issuer limit; preserve all revoked records and use a separately configured repository"
        );
        assert_eq!(
            NativeOriginError::ExistingIssuer.to_string(),
            "Native-origin issuer already exists; do not reactivate an old key, prepare a new private installation for rotation"
        );
    }
}
