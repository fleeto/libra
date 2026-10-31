//! `libra repack` — consolidate objects into a single pack.
//!
//! Reachable objects are encoded into one new `pack-<checksum>.pack` (plus its
//! index) through the shared [`crate::internal::pack_writer`], the same writer
//! the `maintenance` tasks use — there is no separate pack encoder here.
//!
//! - default: pack the reachable objects that are currently **loose**; existing
//!   packs are left untouched (an incremental repack).
//! - `-a` / `--all`: pack **all** reachable objects, including ones already in a
//!   pack, into a single fresh pack.
//! - `-d` / `--delete`: after packing, remove the loose objects that now live in
//!   the new pack; combined with `-a` it also deletes old packs whose objects
//!   are all covered by the new pack (ADR-MX-08), never before the new pack and
//!   index are durable.
//! - `-l` / `--local`: skip objects that live only in an alternate object store
//!   (`pack-objects --local` semantics).
//! - `--window` / `--depth`: delta-compression tuning (ADR-MX-09). `--window` is
//!   forwarded to the shared encoder (0 disables deltas; defaults to
//!   `pack.window`, then Git's 10). `--depth` is validated and reported for Git
//!   CLI parity, but the upstream encoder applies a fixed chain cap of 50
//!   (ER-MX-03 upstream dependency), so packs stay valid for any accepted
//!   value.

use std::collections::HashSet;

use clap::Parser;
use git_internal::hash::ObjectHash;
use serde::Serialize;

use crate::{
    command::maintenance::{collect_reachable_objects, list_loose_objects, parse_object_hash},
    internal::{config::ConfigKv, pack_writer},
    utils::{
        client_storage::ClientStorage,
        error::{CliError, CliResult, StableErrorCode},
        output::{OutputConfig, emit_json_data},
        path,
        storage::{Storage, local::LocalStorage},
        util,
    },
};

/// Git's default delta window (pack-objects `--window`).
const DEFAULT_WINDOW: usize = pack_writer::DEFAULT_WINDOW;
/// Git's default delta chain depth (pack-objects `--depth`).
const DEFAULT_DEPTH: usize = pack_writer::DEFAULT_DEPTH;

pub const REPACK_EXAMPLES: &str = "\
EXAMPLES:
    libra repack               Pack loose reachable objects into a single pack
    libra repack -a            Pack all reachable objects (loose and already packed)
    libra repack -a -d         Repack everything, then drop redundant loose objects and old packs
    libra repack -a -d -l      Same, but skip objects that live only in an alternate
    libra repack -a -d --window=0      Consolidate without delta compression
    libra repack -a -d --window=50 --depth=100  Spend more time on delta search";

/// Combine repository objects into a single pack.
#[derive(Parser, Debug)]
#[command(after_help = REPACK_EXAMPLES)]
pub struct RepackArgs {
    /// Pack all reachable objects, including those already stored in a pack.
    #[arg(short = 'a', long = "all")]
    pub all: bool,
    /// Remove loose objects that end up in the new pack (and, with `--all`,
    /// fully-covered old packs).
    #[arg(short = 'd', long = "delete")]
    pub delete: bool,
    /// Suppress informational output.
    #[arg(short = 'q', long = "quiet")]
    pub quiet: bool,
    /// Skip objects that exist only in an alternate object store.
    #[arg(short = 'l', long = "local")]
    pub local: bool,
    /// Delta-compression window size (default: `pack.window`, then 10; 0
    /// disables delta compression). Forwarded to the shared encoder.
    #[arg(long, value_name = "N", value_parser = clap::value_parser!(usize))]
    pub window: Option<usize>,
    /// Maximum delta chain depth (default: `pack.depth`, then 50). Accepted and
    /// validated for Git CLI parity; the upstream encoder applies a fixed chain
    /// cap of 50 (ER-MX-03), so any accepted value yields a valid pack.
    #[arg(long, value_name = "N", value_parser = clap::value_parser!(usize))]
    pub depth: Option<usize>,
}

#[derive(Serialize)]
struct RepackOutput {
    pack: String,
    objects_packed: usize,
    loose_removed: usize,
    packs_removed: usize,
    window: usize,
    depth: usize,
}

pub async fn execute(args: RepackArgs) {
    if let Err(err) = execute_safe(args, &OutputConfig::default()).await {
        err.print_stderr();
        std::process::exit(err.exit_code());
    }
}

pub async fn execute_safe(args: RepackArgs, output: &OutputConfig) -> CliResult<()> {
    let repo_path = util::try_get_storage_path(None)
        .map_err(|e| CliError::repo_not_found().with_hint(e.to_string()))?;
    let storage = ClientStorage::try_init(path::objects())?;
    let hash_kind = git_internal::hash::get_hash_kind();

    // Resolve the delta window/depth before any write. `--window`/`--depth`
    // take precedence, then `pack.window`/`pack.depth`, then Git's 10/50
    // defaults. Invalid values fail at argument parsing (129) with zero writes.
    let window = resolve_window(&args).await?;
    let depth = resolve_depth(&args).await?;

    // §C.4.3 writer-vs-deleter: `repack` PUBLISHES (a new pack becomes the
    // only home of objects it consolidates) and, under `-d`, DELETES. It
    // takes the maintenance lock shared for the publication half, exactly
    // like every other publisher, so an obliteration cannot classify an
    // object as loose-only and then find it re-published inside a pack it
    // never looked at. `-d` upgrades to the exclusive hold below.
    let publish_lock = crate::internal::maintenance_lock::MaintenanceLock::shared(&repo_path)?;

    let reachable = collect_reachable_objects(&storage).await?;

    // Loose objects on disk, as (hash, path). Used to pick the default packing
    // set and to know which files `-d` may remove.
    let loose = list_loose_objects(&repo_path)
        .map_err(|e| CliError::fatal(format!("failed to list loose objects: {e}")))?;
    let loose_hashes: HashSet<ObjectHash> = loose
        .iter()
        .filter_map(|(hash_str, _)| parse_object_hash(hash_str))
        .collect();

    let to_pack: Vec<ObjectHash> = if args.all {
        let mut hashes: Vec<ObjectHash> = reachable.iter().copied().collect();
        if args.local {
            hashes = retain_local_objects(&hashes).await?;
        }
        hashes
    } else {
        reachable.intersection(&loose_hashes).copied().collect()
    };

    let pack_dir = path::objects().join("pack");
    // The resolved window reaches the shared encoder: 0 selects the no-delta
    // path, non-zero values are forwarded as the sliding-window size. Empty
    // payloads are routed around an upstream sorting panic inside
    // `encode_pack_bytes_with_window`. `depth` is validated and reported but
    // the encoder's chain cap is fixed at 50 (ER-MX-03).
    let publication = match pack_writer::write_pack_with_index_with_window(
        &storage, &to_pack, &pack_dir, hash_kind, window,
    )
    .await
    {
        Ok(Some(publication)) => publication,
        Ok(None) => {
            if !output.quiet && !output.is_json() {
                println!("Nothing new to pack.");
            }
            return Ok(());
        }
        Err(e) => return Err(CliError::fatal(format!("failed to write pack: {e}"))),
    };

    // `-d`: drop loose objects that are now in the new pack. Only files whose
    // hash is in `to_pack` are removed, so nothing that was left out of the pack
    // is ever deleted. Combined with `-a`, fully-covered old packs are deleted
    // too (ADR-MX-08), still only after the new pack + index are durable.
    let pack_path = publication.path.clone();
    let mut loose_removed = 0usize;
    let mut packs_removed = 0usize;
    if args.delete {
        // The pack is about to become these objects' only home, so its NAME
        // has to be durable, not just its bytes: a crash that loses the
        // directory entry after the loose copies are gone loses the objects.
        if !publication.durable {
            return Err(CliError::fatal(
                "refusing to delete loose objects: the new pack's directory entry could not be \
                 made durable, so a crash could leave the objects with no copy at all"
                    .to_string(),
            )
            .with_stable_code(crate::utils::error::StableErrorCode::IoWriteFailed)
            .with_hint("re-run 'libra repack' without -d to keep the loose objects"));
        }
        // lore.md 2.3 / plan-20260714 W0 deletion hard gate: `-d` is a
        // DELETION entry point, so it must refuse while another repository
        // borrows from this object store — this store's reachability does
        // not include the borrower's refs, so a loose object it still needs
        // could be dropped. The borrower must `alternates remove` (or
        // dissociate) first.
        crate::internal::alternates::ensure_no_live_borrowers(
            "delete loose objects after repacking",
            crate::utils::error::StableErrorCode::ConflictOperationBlocked,
        )
        .map_err(|error| {
            error.with_hint("or re-run 'libra repack' without -d to keep the loose objects")
        })?;
        // The publication is durable on disk now, so the shared hold is
        // released and the deletion phase takes the EXCLUSIVE one: no
        // publisher may be running while loose payloads are unlinked. A
        // shared hold cannot be upgraded in place (another process may hold
        // it too), which is why this is a release-then-acquire rather than a
        // nested acquisition.
        drop(publish_lock);
        let _deletion_lock =
            crate::internal::maintenance_lock::MaintenanceLock::exclusive_or_refuse(
                &repo_path,
                "delete loose objects after repacking",
            )
            .map_err(|error| {
                error.with_hint("or re-run 'libra repack' without -d to keep the loose objects")
            })?;
        // Re-checked UNDER the exclusive hold: the gate above ran before this
        // lock existed, and borrower registration is itself a publication, so
        // one that appeared in between is caught here (§C.4.3).
        crate::internal::alternates::ensure_no_live_borrowers(
            "delete loose objects after repacking",
            crate::utils::error::StableErrorCode::ConflictOperationBlocked,
        )
        .map_err(|error| {
            error.with_hint("or re-run 'libra repack' without -d to keep the loose objects")
        })?;
        let packed: HashSet<ObjectHash> = to_pack.iter().copied().collect();
        for (hash_str, obj_path) in &loose {
            if let Some(hash) = parse_object_hash(hash_str)
                && packed.contains(&hash)
            {
                match std::fs::remove_file(obj_path) {
                    Ok(()) => loose_removed += 1,
                    // Already gone. Two `repack -d` runs can each snapshot the
                    // same loose set under their shared holds; whichever
                    // reaches the exclusive phase second finds the files
                    // removed, and its own pack still contains every object,
                    // so the goal state is reached. Treating that as fatal
                    // failed a command that had done its job correctly.
                    Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                    Err(e) => {
                        return Err(CliError::fatal(format!(
                            "failed to remove loose object {hash_str}: {e}"
                        )));
                    }
                }
            }
        }
        // ADR-MX-08: `-a -d` also deletes old packs that are now fully covered
        // by the new pack. Every old-pack object must be in `to_pack` (the new
        // pack's object set); a single uncovered object keeps that pack. A
        // `.keep` sentinel pins a pack unconditionally.
        if args.all {
            packs_removed = delete_covered_packs(&pack_path, &to_pack, hash_kind)?;
        }
    }

    // Either the `-d` branch released it, or it is released when this
    // function returns — the publication window ends with the pack on disk.
    let pack_name = pack_path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default();

    if output.is_json() {
        emit_json_data(
            "repack",
            &RepackOutput {
                pack: pack_name,
                objects_packed: to_pack.len(),
                loose_removed,
                packs_removed,
                window,
                depth,
            },
            output,
        )?;
    } else if !output.quiet {
        println!("Packed {} objects into {pack_name}.", to_pack.len());
        if args.delete {
            println!("Removed {loose_removed} loose objects now stored in the pack.");
            if packs_removed > 0 {
                println!("Removed {packs_removed} fully-covered old pack(s).");
            }
        }
    }

    Ok(())
}

/// Resolve the delta window: `--window`, then `pack.window`, then Git's 10.
async fn resolve_window(args: &RepackArgs) -> CliResult<usize> {
    if let Some(window) = args.window {
        return Ok(window);
    }
    resolve_config_usize("pack.window", DEFAULT_WINDOW).await
}

/// Resolve the delta depth: `--depth`, then `pack.depth`, then Git's 50.
async fn resolve_depth(args: &RepackArgs) -> CliResult<usize> {
    if let Some(depth) = args.depth {
        return Ok(depth);
    }
    resolve_config_usize("pack.depth", DEFAULT_DEPTH).await
}

/// Read a non-negative integer config key, falling back to `default` on an
/// absent or unparseable value (matching Git's lenient config parsing).
async fn resolve_config_usize(key: &str, default: usize) -> CliResult<usize> {
    match ConfigKv::get(key).await {
        Ok(Some(entry)) => Ok(entry.value.trim().parse::<usize>().unwrap_or(default)),
        Ok(None) => Ok(default),
        Err(error) => Err(CliError::fatal(format!(
            "failed to read {key} config: {error}"
        ))),
    }
}

/// Keep only objects present in THIS repository's object store (loose or
/// packed), dropping objects reachable only through an alternate. `-l` /
/// `--local` semantics.
async fn retain_local_objects(hashes: &[ObjectHash]) -> CliResult<Vec<ObjectHash>> {
    // A local-only reader: no alternates chain, so `exist_checked` answers
    // "present in this store" rather than "present anywhere reachable".
    // A read error stays an error; only `Ok(false)` drops the object.
    let local = LocalStorage::open_no_create(path::objects());
    let mut out = Vec::with_capacity(hashes.len());
    for hash in hashes {
        match local.exist_checked(hash).await {
            Ok(true) => out.push(*hash),
            Ok(false) => {}
            Err(error) => {
                return Err(CliError::fatal(format!(
                    "failed to read object storage while checking {hash}: {error}"
                ))
                .with_stable_code(StableErrorCode::IoReadFailed));
            }
        }
    }
    Ok(out)
}

/// Delete old packs whose every object is covered by the new pack.
///
/// Runs only after the new pack and its index are durable and under the
/// exclusive maintenance hold. Each old pack's `.idx` is parsed to enumerate
/// its objects; if any object is missing from `new_pack_hashes`, or the pack is
/// pinned by a `.keep` sentinel, it is retained. The new pack itself is never a
/// candidate. Deletion removes the `.pack` and its `.idx` together.
fn delete_covered_packs(
    new_pack_path: &std::path::Path,
    new_pack_hashes: &[ObjectHash],
    hash_kind: git_internal::hash::HashKind,
) -> CliResult<usize> {
    let pack_dir = path::objects().join("pack");
    let covered: HashSet<ObjectHash> = new_pack_hashes.iter().copied().collect();

    let old_packs: Vec<std::path::PathBuf> = match std::fs::read_dir(&pack_dir) {
        Ok(entries) => entries
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|ext| ext == "pack"))
            .filter(|p| p != new_pack_path)
            .collect(),
        Err(e) => {
            return Err(CliError::fatal(format!(
                "failed to read pack directory: {e}"
            )));
        }
    };

    let mut removed = 0usize;
    for old_pack in old_packs {
        // A `.keep` sentinel pins the pack unconditionally (an in-flight or
        // crashed fetch may still be about to publish a ref that needs it).
        if old_pack.with_extension("keep").exists() {
            continue;
        }
        let idx_path = old_pack.with_extension("idx");
        let idx_bytes = match std::fs::read(&idx_path) {
            Ok(bytes) => bytes,
            Err(_) => continue, // unreadable index: never delete what can't be verified
        };
        // Every object in the old pack must be covered by the new pack.
        if !idx_entries_fully_covered(&idx_bytes, &covered, hash_kind) {
            continue;
        }
        let _ = std::fs::remove_file(&old_pack);
        let _ = std::fs::remove_file(&idx_path);
        removed += 1;
    }
    Ok(removed)
}

/// Parse a pack index's object list (v2 for any hash kind, v1 only for sha1).
/// Returns `None` for an unreadable/mismatched index so the caller keeps the
/// pack rather than deleting what cannot be verified.
fn parse_idx_entries(
    idx_bytes: &[u8],
    hash_kind: git_internal::hash::HashKind,
) -> Option<Vec<ObjectHash>> {
    if idx_bytes.starts_with(&[0xff, b't', b'O', b'c']) {
        git_internal::internal::pack::pack_index::parse_idx_v2(idx_bytes, hash_kind)
            .ok()
            .map(|index| index.entries.into_iter().map(|entry| entry.hash).collect())
    } else if hash_kind == git_internal::hash::HashKind::Sha1 {
        git_internal::internal::pack::pack_index::parse_idx_v1(idx_bytes)
            .ok()
            .map(|index| index.entries.into_iter().map(|entry| entry.hash).collect())
    } else {
        None
    }
}

/// Whether every object in an index is present in `covered` (an old pack is
/// only deleted when the new pack holds every one of its objects).
fn idx_entries_fully_covered(
    idx_bytes: &[u8],
    covered: &HashSet<ObjectHash>,
    hash_kind: git_internal::hash::HashKind,
) -> bool {
    let Some(hashes) = parse_idx_entries(idx_bytes, hash_kind) else {
        return false;
    };
    hashes.iter().all(|hash| covered.contains(hash))
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use clap::Parser;
    use git_internal::{
        hash::{HashKind, ObjectHash},
        internal::{object::blob::Blob, pack::entry::Entry},
    };
    use serial_test::serial;

    use super::{RepackArgs, idx_entries_fully_covered, retain_local_objects};
    use crate::{
        internal::pack_writer::encode_pack_bytes,
        utils::{
            error::StableErrorCode,
            test::{ChangeDirGuard, setup_with_new_libra_in},
        },
    };

    #[tokio::test]
    #[serial(cwd, env)]
    async fn repack_storage_read_error_is_not_absent() {
        let tmp = tempfile::tempdir().expect("tmp");
        let _guard = ChangeDirGuard::new(tmp.path());
        setup_with_new_libra_in(tmp.path()).await;
        let pack_dir = tmp.path().join(".libra/objects/pack");
        std::fs::create_dir_all(&pack_dir).expect("pack dir");
        std::fs::write(pack_dir.join("orphan.pack"), b"not-a-pack").expect("unindexed pack");
        let hash = ObjectHash::new_for_kind(git_internal::hash::get_hash_kind(), &[3; 20]);
        let error = retain_local_objects(&[hash])
            .await
            .expect_err("an unindexed pack is a read failure");
        let rendered = error.to_string();
        assert_eq!(error.stable_code(), StableErrorCode::IoReadFailed);
        assert!(
            rendered.contains("no complete index"),
            "the pack read error must stay visible: {rendered}"
        );
        assert!(
            !rendered.contains("missing") && !rendered.contains("absent"),
            "a pack read error is not absence: {rendered}"
        );
    }

    #[test]
    fn repack_args_parse_flags_and_reject_invalid_window_depth() {
        let bare = RepackArgs::try_parse_from(["repack"]).expect("bare repack parses");
        assert!(!bare.all && !bare.delete && !bare.quiet && !bare.local);
        assert!(bare.window.is_none() && bare.depth.is_none());

        let full =
            RepackArgs::try_parse_from(["repack", "-a", "-d", "-l", "--window=50", "--depth=100"])
                .expect("full repack parse");
        assert!(full.all && full.delete && full.local);
        assert_eq!(full.window, Some(50));
        assert_eq!(full.depth, Some(100));

        // `--window=0` (no deltas) is a valid value.
        let zero = RepackArgs::try_parse_from(["repack", "--window=0"]).expect("window 0 parses");
        assert_eq!(zero.window, Some(0));

        // Non-numeric and negative values are usage errors (129, D6) — the
        // usize value parser rejects them before any write.
        assert!(RepackArgs::try_parse_from(["repack", "--window=abc"]).is_err());
        assert!(RepackArgs::try_parse_from(["repack", "--window=-1"]).is_err());
        assert!(RepackArgs::try_parse_from(["repack", "--depth=nope"]).is_err());
        assert!(RepackArgs::try_parse_from(["repack", "--bogus"]).is_err());
    }

    #[test]
    fn idx_coverage_requires_every_object_and_fails_closed_on_garbage() {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("test runtime");
        let entries = vec![
            Entry::from(Blob::from_content("coverage probe one")),
            Entry::from(Blob::from_content("coverage probe two")),
        ];
        let pack = runtime
            .block_on(encode_pack_bytes(entries, HashKind::Sha1))
            .expect("encode pack");
        let dir = tempfile::tempdir().expect("tempdir");
        let pack_path = dir.path().join("pack-test.pack");
        let idx_path = dir.path().join("pack-test.idx");
        std::fs::write(&pack_path, &pack).expect("write pack");
        crate::command::index_pack::build_index_v2(
            pack_path.to_str().expect("pack path"),
            idx_path.to_str().expect("idx path"),
        )
        .expect("build idx");
        let idx_bytes = std::fs::read(&idx_path).expect("read idx");

        let parsed =
            git_internal::internal::pack::pack_index::parse_idx_v2(&idx_bytes, HashKind::Sha1)
                .expect("parse idx");
        let covered: HashSet<ObjectHash> = parsed.entries.iter().map(|entry| entry.hash).collect();
        assert!(
            idx_entries_fully_covered(&idx_bytes, &covered, HashKind::Sha1),
            "an index whose objects are all covered is fully covered"
        );

        // Drop one object from the new pack's set → the old pack is retained.
        let mut partial = covered.clone();
        partial.remove(&parsed.entries[0].hash);
        assert!(
            !idx_entries_fully_covered(&idx_bytes, &partial, HashKind::Sha1),
            "a single uncovered object keeps the old pack"
        );

        // Unreadable/garbage index bytes never count as covered (fail closed).
        assert!(!idx_entries_fully_covered(
            b"garbage",
            &covered,
            HashKind::Sha1
        ));
    }
}
