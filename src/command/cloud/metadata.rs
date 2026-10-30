use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

use super::*;

fn calculate_metadata_hash(json: &[u8]) -> u64 {
    let mut hasher = DefaultHasher::new();
    json.hash(&mut hasher);
    hasher.finish()
}

/// Upload the refs projection frozen in this run's snapshot spool (PI-09);
/// refs written after the freeze wait for the next sync.
pub(super) async fn sync_metadata(
    r2_storage: &RemoteStorage,
    progress: &dyn CloudSyncProgress,
    frozen: &super::snapshot::FrozenRefs,
) -> CloudResult<MetadataSyncOutcome> {
    progress.on_metadata_starting();
    let json = &frozen.json;
    let current_hash = calculate_metadata_hash(json);

    // Check if hash matches last sync.
    if let Some(stored) = ConfigKv::get("cloud.metadata_hash")
        .await
        .ok()
        .flatten()
        .map(|e| e.value)
        && let Ok(stored_hash) = stored.parse::<u64>()
        && stored_hash == current_hash
    {
        progress.on_metadata_skipped();
        return Ok(MetadataSyncOutcome::Skipped);
    }

    r2_storage
        .put_metadata(json)
        .await
        .map_err(|e| CloudError::R2(format!("Failed to upload metadata: {}", e)))?;

    // Update stored hash.
    let _ = ConfigKv::set("cloud.metadata_hash", &current_hash.to_string(), false).await;

    progress.on_metadata_synced(frozen.references);
    Ok(MetadataSyncOutcome::Synced {
        references: frozen.references,
    })
}

/// Keep the cloud metadata contract limited to references. Repository-private
/// `metadata_kv` scopes never enter this projection.
pub(super) fn serialize_reference_projection(
    references: Vec<reference::Model>,
) -> CloudResult<(Vec<reference::Model>, Vec<u8>)> {
    let mut sorted_refs = references;
    sorted_refs.sort_by(|a, b| {
        let a_kind = format!("{:?}", a.kind);
        let b_kind = format!("{:?}", b.kind);
        let a_key = (&a.name, &a.remote, a_kind);
        let b_key = (&b.name, &b.remote, b_kind);
        a_key.cmp(&b_key)
    });

    let json = serde_json::to_vec(&sorted_refs)
        .map_err(|e| CloudError::Generic(format!("Failed to serialize metadata: {}", e)))?;
    Ok((sorted_refs, json))
}

#[cfg(test)]
mod tests {
    use sea_orm::{ActiveModelTrait, ActiveValue::Set, EntityTrait};

    use super::*;
    use crate::internal::{
        metadata::{MetadataKv, MetadataScope, MetadataValueType},
        model::reference::ConfigKind,
    };

    #[tokio::test]
    async fn reasoning_identity_is_not_in_cloud_projection() {
        let dir = tempfile::tempdir().unwrap();
        let conn =
            crate::internal::db::create_database(dir.path().join("libra.db").to_str().unwrap())
                .await
                .unwrap();
        crate::internal::model::reference::ActiveModel {
            name: Set(Some("refs/heads/main".to_owned())),
            kind: Set(ConfigKind::Branch),
            commit: Set(Some("a".repeat(40))),
            remote: Set(None),
            worktree_id: Set(None),
            ..Default::default()
        }
        .insert(&conn)
        .await
        .unwrap();
        MetadataKv::set_with_conn(
            &conn,
            MetadataScope::AgentReasoningIdentity,
            "session-private-target",
            "identity-v1",
            "private-identity-canary",
            MetadataValueType::Text,
        )
        .await
        .unwrap();

        let references = reference::Entity::find().all(&conn).await.unwrap();
        let (_, projected) = serialize_reference_projection(references).unwrap();
        let projection = String::from_utf8(projected).unwrap();
        assert!(projection.contains("refs/heads/main"));
        assert!(!projection.contains("private-identity-canary"));
        assert!(!projection.contains("agent_reasoning_identity"));
    }
}
