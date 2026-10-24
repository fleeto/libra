//! Shared index-entry stat comparison for `status` and Operation snapshots.
//!
//! Mirrors git-internal's private `Index::is_modified` triple comparison
//! (ctime / mtime / size) plus the racily-clean guard against the index file
//! mtime. Callers adapt `CapturedStat` or `fs::Metadata` into the SystemTime
//! triple before invoking these helpers so `internal::operation` never depends
//! on `command`.

use std::time::SystemTime;

use git_internal::internal::index::IndexEntry;

/// Serializable copy of the stat fields only. Time's fields are private in
/// git-internal; its stable Display representation preserves the exact values.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct IndexStatCache {
    ctime: String,
    mtime: String,
    size: u32,
}
impl IndexStatCache {
    pub(crate) fn from_entry(entry: &IndexEntry) -> Self {
        Self {
            ctime: entry.ctime.to_string(),
            mtime: entry.mtime.to_string(),
            size: entry.size,
        }
    }
    fn matches(&self, ctime: SystemTime, mtime: SystemTime, size: u64) -> bool {
        fn encoded(time: SystemTime) -> Option<String> {
            let duration = time.duration_since(std::time::UNIX_EPOCH).ok()?;
            let seconds = u32::try_from(duration.as_secs()).ok()?;
            Some(format!("{seconds}:{}", duration.subsec_nanos()))
        }
        size == u64::from(self.size)
            && encoded(ctime).as_deref() == Some(self.ctime.as_str())
            && encoded(mtime).as_deref() == Some(self.mtime.as_str())
    }
    pub(crate) fn differs(
        &self,
        ctime: SystemTime,
        mtime: SystemTime,
        size: u64,
        index_mtime: Option<SystemTime>,
    ) -> bool {
        !self.matches(ctime, mtime, size) || !index_mtime.is_some_and(|snapshot| mtime < snapshot)
    }
}

/// Whether `entry`'s cached ctime/mtime/size triple equals the observed values.
///
/// A size that does not fit in `u32` never matches (content-compare instead).
/// This is the non-racy half of the status / snapshot trust check.
pub fn entry_stat_matches(
    entry: &IndexEntry,
    ctime: SystemTime,
    mtime: SystemTime,
    size: u64,
) -> bool {
    IndexStatCache::from_entry(entry).matches(ctime, mtime, size)
}

/// Whether the index entry's cached stat disagrees with the observed triple,
/// including the racily-clean guard against `index_file_mtime`.
///
/// A matching triple is trustworthy only when the worktree mtime is strictly
/// older than the index snapshot itself (`mtime < index_file_mtime`). An
/// unknown index mtime never earns trust.
pub fn index_entry_stat_differs(
    entry: &IndexEntry,
    ctime: SystemTime,
    mtime: SystemTime,
    size: u64,
    index_file_mtime: Option<SystemTime>,
) -> bool {
    if !entry_stat_matches(entry, ctime, mtime, size) {
        return true;
    }
    let trustworthy = index_file_mtime.is_some_and(|snapshot| mtime < snapshot);
    !trustworthy
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    use git_internal::{hash::ObjectHash, internal::index::IndexEntry};

    use super::{entry_stat_matches, index_entry_stat_differs};

    fn sample_entry(ctime: SystemTime, mtime: SystemTime, size: u32) -> IndexEntry {
        let hash = ObjectHash::from_type_and_data(
            git_internal::internal::object::types::ObjectType::Blob,
            b"stat-diff-helper",
        );
        let mut entry = IndexEntry::new_from_blob("sample.txt".into(), hash, size);
        entry.ctime = git_internal::internal::index::Time::from_system_time(ctime);
        entry.mtime = git_internal::internal::index::Time::from_system_time(mtime);
        entry
    }

    /// G18: status and snapshot both call this module's helpers; the former
    /// `index_stat_differs` / `entry_stat_matches_metadata` mirrors are gone.
    #[test]
    fn status_and_snapshot_share_helper() {
        let ctime = UNIX_EPOCH + Duration::from_secs(1_700_000_000);
        let mtime = UNIX_EPOCH + Duration::from_secs(1_700_000_010);
        let index_mtime = UNIX_EPOCH + Duration::from_secs(1_700_000_020);
        let entry = sample_entry(ctime, mtime, 11);

        assert!(entry_stat_matches(&entry, ctime, mtime, 11));
        assert!(!entry_stat_matches(&entry, ctime, mtime, 12));
        assert!(!index_entry_stat_differs(
            &entry,
            ctime,
            mtime,
            11,
            Some(index_mtime)
        ));
        // Same-second / not-older-than-index is racily clean → differs.
        assert!(index_entry_stat_differs(
            &entry,
            ctime,
            mtime,
            11,
            Some(mtime)
        ));
        assert!(index_entry_stat_differs(&entry, ctime, mtime, 11, None));

        // The production call sites import these two symbols from
        // `crate::utils::stat_diff` (status_untracked + command::mod +
        // operation::snapshot). Keeping the names stable is the G18 contract.
        let _status_api: fn(&IndexEntry, SystemTime, SystemTime, u64, Option<SystemTime>) -> bool =
            index_entry_stat_differs;
        let _match_api: fn(&IndexEntry, SystemTime, SystemTime, u64) -> bool = entry_stat_matches;
        let _ = (_status_api, _match_api);
    }
}
