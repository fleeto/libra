//! Integration tests for `repack` and the hidden `pack-objects` plumbing.
//!
//! **Layer:** L1 — deterministic, no external dependencies.
//!
//! The central correctness check is that a pack produced by the shared writer
//! round-trips through `index-pack` (i.e. it is a well-formed pack whose trailer
//! checksum matches its bytes) and that objects remain readable after `-d`
//! removes the loose copies. The pre-refactor maintenance writer produced packs
//! that failed exactly this check.
//!
//! M-REPACK R1-R5 (ADR-MX-08, pack lifecycle and `-l`) and M-DELTA D1-D6
//! (ADR-MX-09, delta window/depth) from `docs/development/plan/issues/481.md`
//! are covered by the `repack_matrix_*` / `gc_aggressive_*` tests below.

use std::{
    collections::HashMap,
    fs,
    io::{self, Read, Write},
    path::Path,
    process::{Output, Stdio},
};

use git_internal::{
    hash::{HashKind, ObjectHash},
    internal::pack::Pack,
};

use super::*;

/// Run the Libra binary feeding `stdin_bytes` on stdin (for `pack-objects`).
fn run_libra_with_stdin(args: &[&str], cwd: &std::path::Path, stdin_bytes: &[u8]) -> Output {
    let mut command = base_libra_command(args, cwd);
    command.stdin(Stdio::piped());
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());
    let mut child = command.spawn().expect("failed to spawn libra");
    child
        .stdin
        .take()
        .expect("stdin pipe")
        .write_all(stdin_bytes)
        .expect("failed to write stdin");
    child.wait_with_output().expect("failed to wait for libra")
}

/// Every loose object id currently on disk under `.libra/objects/`.
fn loose_object_ids(repo: &std::path::Path) -> Vec<String> {
    let objects = repo.join(".libra").join("objects");
    let mut ids = Vec::new();
    let Ok(dirs) = fs::read_dir(&objects) else {
        return ids;
    };
    for dir in dirs.flatten() {
        let name = dir.file_name().to_string_lossy().into_owned();
        // Object shards are the 2-hex-char directories; skip `pack`, `info`, …
        if name.len() != 2 || !name.chars().all(|c| c.is_ascii_hexdigit()) {
            continue;
        }
        let Ok(files) = fs::read_dir(dir.path()) else {
            continue;
        };
        for file in files.flatten() {
            ids.push(format!("{name}{}", file.file_name().to_string_lossy()));
        }
    }
    ids
}

/// Pack files (`*.pack`) currently in the pack directory.
fn pack_files(repo: &std::path::Path) -> Vec<std::path::PathBuf> {
    let pack_dir = repo.join(".libra").join("objects").join("pack");
    let Ok(entries) = fs::read_dir(&pack_dir) else {
        return Vec::new();
    };
    entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|ext| ext == "pack"))
        .collect()
}

/// Build a repo with a few commits so several loose objects exist.
fn repo_with_history() -> tempfile::TempDir {
    let repo = create_committed_repo_via_cli();
    for i in 0..3 {
        fs::write(
            repo.path().join(format!("file{i}.txt")),
            format!("contents number {i}\n"),
        )
        .expect("write file");
        let output = run_libra_command(&["add", &format!("file{i}.txt")], repo.path());
        assert_cli_success(&output, "add file");
        let output = run_libra_command(
            &["commit", "-m", &format!("commit {i}"), "--no-verify"],
            repo.path(),
        );
        assert_cli_success(&output, "commit file");
    }
    repo
}

/// M-OID O3: after `repack` (loose + pack coexistence), every short-OID
/// consumer resolves the same unique prefix to the same object.
#[test]
fn test_short_oid_consumers_after_repack() {
    let repo = repo_with_history();
    let head = run_libra_command(&["rev-parse", "HEAD"], repo.path());
    assert_cli_success(&head, "rev-parse HEAD");
    let full_hash = String::from_utf8_lossy(&head.stdout).trim().to_string();
    let prefix = &full_hash[..10];

    let repack = run_libra_command(&["repack"], repo.path());
    assert_cli_success(&repack, "repack (loose + pack)");

    // cat-file -t
    let out = run_libra_command(&["cat-file", "-t", prefix], repo.path());
    assert_cli_success(&out, "cat-file -t <prefix>");
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "commit");

    // log --oneline -1
    let out = run_libra_command(&["log", "--oneline", "-1", prefix], repo.path());
    assert_cli_success(&out, "log --oneline -1 <prefix>");
    assert!(
        String::from_utf8_lossy(&out.stdout).contains("commit 2"),
        "log must print the resolved commit, got: {}",
        String::from_utf8_lossy(&out.stdout)
    );

    // show -s --format=%s
    let out = run_libra_command(&["show", "-s", "--format=%s", prefix], repo.path());
    assert_cli_success(&out, "show -s --format=%s <prefix>");
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "commit 2");

    // merge-base <prefix> HEAD
    let out = run_libra_command(&["merge-base", prefix, "HEAD"], repo.path());
    assert_cli_success(&out, "merge-base <prefix> HEAD");
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), full_hash);

    // branch <name> <prefix>
    let out = run_libra_command(&["branch", "mx-consumer", prefix], repo.path());
    assert_cli_success(&out, "branch mx-consumer <prefix>");
    let out = run_libra_command(&["rev-parse", "mx-consumer"], repo.path());
    assert_cli_success(&out, "rev-parse mx-consumer");
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), full_hash);

    // rev-parse <prefix>^{tree}
    let out = run_libra_command(&["rev-parse", &format!("{prefix}^{{tree}}")], repo.path());
    assert_cli_success(&out, "rev-parse <prefix>^{tree}");
    let tree = String::from_utf8_lossy(&out.stdout).trim().to_string();
    assert_eq!(tree.len(), 40, "expected full tree hash, got: {tree}");
    assert!(tree.chars().all(|c| c.is_ascii_hexdigit()));

    // restore --source=<prefix> -- <path>
    let out = run_libra_command(
        &["restore", &format!("--source={prefix}"), "--", "file0.txt"],
        repo.path(),
    );
    assert_cli_success(&out, "restore --source=<prefix> -- file0.txt");

    // checkout <prefix> detaches to the commit.
    let out = run_libra_command(&["checkout", prefix], repo.path());
    assert_cli_success(&out, "checkout <prefix>");
    let out = run_libra_command(&["rev-parse", "HEAD"], repo.path());
    assert_cli_success(&out, "rev-parse HEAD after checkout");
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), full_hash);
}

#[test]
fn repack_all_delete_produces_valid_pack_and_prunes_loose() {
    let repo = repo_with_history();
    assert!(
        !loose_object_ids(repo.path()).is_empty(),
        "precondition: repo should have loose objects"
    );

    let output = run_libra_command(&["repack", "-a", "-d"], repo.path());
    assert_cli_success(&output, "repack -a -d");

    // A pack + its index now exist.
    let packs = pack_files(repo.path());
    assert_eq!(packs.len(), 1, "exactly one pack should be written");
    let pack = &packs[0];
    assert!(
        pack.with_extension("idx").exists(),
        "the pack's .idx must be written alongside it"
    );

    // The pack round-trips through index-pack — i.e. it is well-formed and its
    // trailer checksum matches its bytes. The old hand-rolled writer failed here.
    let output = run_libra_command(&["index-pack", pack.to_str().unwrap()], repo.path());
    assert_cli_success(&output, "index-pack must validate the repacked pack");

    // Loose objects were pruned...
    assert!(
        loose_object_ids(repo.path()).is_empty(),
        "-d should remove the loose objects now stored in the pack"
    );
    // ...yet history is still fully readable from the pack.
    let output = run_libra_command(&["log", "--oneline"], repo.path());
    assert_cli_success(&output, "log after repack -d");
    let output = run_libra_command(&["cat-file", "-p", "HEAD"], repo.path());
    assert_cli_success(&output, "cat-file HEAD after repack -d");
}

#[test]
fn repack_is_idempotent_when_nothing_is_loose() {
    let repo = repo_with_history();
    let output = run_libra_command(&["repack", "-a", "-d"], repo.path());
    assert_cli_success(&output, "first repack");

    // With every reachable object already packed and no loose objects left, a
    // default repack has nothing to do.
    let output = run_libra_command(&["repack"], repo.path());
    assert_cli_success(&output, "second repack");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Nothing new to pack"),
        "expected a no-op message, got: {stdout}"
    );
}

#[test]
fn repack_json_reports_pack_and_counts() {
    let repo = repo_with_history();
    let output = run_libra_command(&["--json", "repack", "-a", "-d"], repo.path());
    assert_cli_success(&output, "repack --json");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let value: Value = serde_json::from_str(&stdout).expect("repack --json must emit valid JSON");
    let data = &value["data"];
    assert!(
        data["pack"]
            .as_str()
            .unwrap_or_default()
            .starts_with("pack-"),
        "json should name the new pack, got: {stdout}"
    );
    assert!(
        data["objects_packed"].as_u64().unwrap_or(0) > 0,
        "json should report a positive object count, got: {stdout}"
    );
}

#[test]
fn repack_outside_repository_fails() {
    let dir = tempdir().expect("tempdir");
    let output = run_libra_command(&["repack"], dir.path());
    assert!(
        !output.status.success(),
        "repack outside a repository must fail"
    );
}

#[test]
fn pack_objects_reads_ids_from_stdin_and_writes_valid_pack() {
    let repo = repo_with_history();
    let ids = loose_object_ids(repo.path());
    assert!(!ids.is_empty(), "need loose objects to pack");
    let stdin = ids.join("\n");

    let output = run_libra_with_stdin(&["pack-objects"], repo.path(), stdin.as_bytes());
    assert!(
        output.status.success(),
        "pack-objects should succeed, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.trim().starts_with("pack-"),
        "pack-objects should print the new pack's name, got: {stdout}"
    );

    // The written pack validates through index-pack.
    let packs = pack_files(repo.path());
    assert!(!packs.is_empty(), "pack-objects should write a pack");
    let output = run_libra_command(&["index-pack", packs[0].to_str().unwrap()], repo.path());
    assert_cli_success(&output, "index-pack must validate the pack-objects output");
}

#[test]
fn pack_objects_empty_stdin_fails() {
    let repo = repo_with_history();
    let output = run_libra_with_stdin(&["pack-objects"], repo.path(), b"   \n");
    assert!(
        !output.status.success(),
        "pack-objects with no ids on stdin must fail"
    );
}

// ---------------------------------------------------------------------------
// M-REPACK (ADR-MX-08): pack lifecycle, `-l`, deletion safety.
// ---------------------------------------------------------------------------

/// Whether the loose file for `oid` still exists on disk.
fn loose_object_exists(repo: &Path, oid: &str) -> bool {
    repo.join(".libra")
        .join("objects")
        .join(&oid[..2])
        .join(&oid[2..])
        .exists()
}

/// M-REPACK R1 + R5: three consecutive `repack -a -d` runs each leave exactly
/// one pack; the final state round-trips through `fsck`, `verify-pack` and
/// short-SHA resolution.
#[test]
fn repack_matrix_r1_three_all_delete_runs_keep_single_pack() {
    let repo = repo_with_history();

    let head = run_libra_command(&["rev-parse", "HEAD"], repo.path());
    assert_cli_success(&head, "rev-parse HEAD");
    let full_hash = String::from_utf8_lossy(&head.stdout).trim().to_string();

    for run in 1..=3 {
        let output = run_libra_command(&["repack", "-a", "-d"], repo.path());
        assert_cli_success(&output, &format!("repack -a -d run {run}"));
        let packs = pack_files(repo.path());
        assert_eq!(packs.len(), 1, "run {run}: exactly one pack must remain");
        assert!(
            packs[0].with_extension("idx").exists(),
            "run {run}: the pack's .idx must sit alongside it"
        );
    }

    // R5: integrity after the full R1 sequence.
    let packs = pack_files(repo.path());
    let idx = packs[0].with_extension("idx");
    let output = run_libra_command(&["verify-pack", idx.to_str().unwrap()], repo.path());
    assert_cli_success(&output, "verify-pack after the R1 sequence");
    let output = run_libra_command(&["fsck"], repo.path());
    assert_cli_success(&output, "fsck after the R1 sequence");
    let prefix = &full_hash[..10];
    let output = run_libra_command(&["rev-parse", prefix], repo.path());
    assert_cli_success(&output, "short SHA resolves after the R1 sequence");
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), full_hash);
}

/// M-REPACK R2 (fault injection): the deletion phase refuses while another
/// repository borrows this object store — a fault injected between the durable
/// new pack and the deletion — and everything stays intact: old pack, loose
/// objects, `fsck`.
#[test]
fn repack_matrix_r2_borrower_refusal_keeps_old_packs_and_loose() {
    let base = repo_with_history();
    // A pack that a later `-a -d` would be allowed to delete...
    let output = run_libra_command(&["repack", "-a"], base.path());
    assert_cli_success(&output, "repack -a");
    let packs_before = pack_files(base.path());
    assert_eq!(packs_before.len(), 1, "one pack before the refused run");
    let loose_before = loose_object_ids(base.path());
    assert!(
        !loose_before.is_empty(),
        "loose objects before the refused run"
    );

    // ...now a borrower appears, and the next `-d` must fail BEFORE deleting.
    let dest = tempdir().expect("borrower parent");
    let borrower_path = dest.path().join("borrower");
    let clone = run_libra_command(
        &[
            "clone",
            "--shared",
            base.path().to_str().expect("base path"),
            borrower_path.to_str().expect("borrower path"),
        ],
        dest.path(),
    );
    assert_cli_success(&clone, "clone --shared");

    let output = run_libra_command(&["repack", "-a", "-d"], base.path());
    assert!(
        !output.status.success(),
        "repack -a -d must refuse deletion while a borrower exists"
    );
    let diagnostic = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        diagnostic.contains("LBR-CONFLICT-002"),
        "the refusal carries the stable conflict code, got: {diagnostic}"
    );

    // The pre-existing pack and every loose object survive...
    for pack in &packs_before {
        assert!(
            pack.exists(),
            "old pack {} must survive the refused deletion",
            pack.display()
        );
    }
    for id in &loose_before {
        assert!(
            loose_object_exists(base.path(), id),
            "loose object {id} must survive the refused deletion"
        );
    }
    // ...and the store is fully healthy (R5).
    let output = run_libra_command(&["fsck"], base.path());
    assert_cli_success(&output, "fsck after the refused repack");

    // The borrower still reads every object through the alternate.
    let kind = run_libra_command(&["cat-file", "-t", "HEAD"], &borrower_path);
    assert_cli_success(&kind, "borrower reads HEAD after the refused repack");
    assert_eq!(String::from_utf8_lossy(&kind.stdout).trim(), "commit");
}

/// M-REPACK R2 (coverage side): an old pack holding an object the new pack
/// does not contain is NEVER deleted — only fully-covered packs go, and the
/// uncovered object stays readable.
#[test]
fn repack_matrix_r2_partially_covered_pack_is_retained() {
    let repo = repo_with_history();

    // A loose object nothing references, packed on its own.
    fs::write(repo.path().join("orphan.txt"), "orphan payload\n").expect("write orphan");
    let hash = run_libra_command(&["hash-object", "-w", "orphan.txt"], repo.path());
    assert_cli_success(&hash, "hash-object -w");
    let orphan = String::from_utf8_lossy(&hash.stdout).trim().to_string();
    fs::remove_file(repo.path().join("orphan.txt")).expect("remove orphan");

    let output = run_libra_with_stdin(&["pack-objects"], repo.path(), orphan.as_bytes());
    assert!(
        output.status.success(),
        "pack-objects packs the orphan: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let orphan_pack = {
        let packs = pack_files(repo.path());
        assert_eq!(packs.len(), 1, "pack-objects wrote exactly one pack");
        packs[0].clone()
    };

    // `-a` packs every reachable object into a fresh pack; the orphan pack is
    // NOT fully covered (the orphan is unreachable), so it must be retained.
    let output = run_libra_command(&["repack", "-a", "-d"], repo.path());
    assert_cli_success(&output, "repack -a -d with a partially-covered pack");
    let packs = pack_files(repo.path());
    assert_eq!(packs.len(), 2, "the partially-covered old pack is retained");
    assert!(
        packs.contains(&orphan_pack),
        "the orphan pack must still exist after repack -a -d"
    );

    // The orphan is still readable, and the store is healthy.
    let output = run_libra_command(&["cat-file", "-t", &orphan], repo.path());
    assert_cli_success(&output, "orphan readable after repack");
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "blob");
    let output = run_libra_command(&["fsck"], repo.path());
    assert_cli_success(&output, "fsck with the retained orphan pack");
}

/// M-REPACK R3: `-d` without `-a` removes the loose objects it packed but
/// keeps pre-existing packs (git 2.56 keeps them too — empirical baseline).
#[test]
fn repack_matrix_r3_delete_without_all_keeps_old_packs() {
    let repo = repo_with_history();

    let output = run_libra_command(&["repack"], repo.path());
    assert_cli_success(&output, "repack");
    let first_packs = pack_files(repo.path());
    assert_eq!(first_packs.len(), 1, "one pack after the plain repack");
    assert!(
        !loose_object_ids(repo.path()).is_empty(),
        "a plain repack keeps the loose objects on disk"
    );

    // New history adds new loose objects.
    fs::write(repo.path().join("new.txt"), "new\n").expect("write new file");
    let output = run_libra_command(&["add", "new.txt"], repo.path());
    assert_cli_success(&output, "add new.txt");
    let output = run_libra_command(&["commit", "-m", "new", "--no-verify"], repo.path());
    assert_cli_success(&output, "commit new");

    let output = run_libra_command(&["repack", "-d"], repo.path());
    assert_cli_success(&output, "repack -d");
    assert!(
        loose_object_ids(repo.path()).is_empty(),
        "-d removes the loose objects that are now packed"
    );
    assert_eq!(
        pack_files(repo.path()).len(),
        2,
        "the old pack is kept without -a"
    );
    assert!(
        first_packs[0].exists(),
        "the pre-existing pack must still exist"
    );

    // History remains fully readable, and every pack verifies (R5).
    let output = run_libra_command(&["log", "--oneline"], repo.path());
    assert_cli_success(&output, "log after repack -d");
    for pack in &pack_files(repo.path()) {
        let idx = pack.with_extension("idx");
        let output = run_libra_command(&["verify-pack", idx.to_str().unwrap()], repo.path());
        assert_cli_success(&output, "verify-pack after R3");
    }
}

/// M-REPACK R4: `repack -a -d -l` must not pack objects that exist only in
/// the alternate object store; without `-l` the same objects ARE packed.
#[test]
fn repack_matrix_r4_local_skips_alternate_objects() {
    // The alternate holds a commit (and its history) that the borrower
    // reaches only through `alternates`.
    let alt = repo_with_history();
    let alt_head = run_libra_command(&["rev-parse", "HEAD"], alt.path());
    assert_cli_success(&alt_head, "rev-parse HEAD in the alternate");
    let alt_head = String::from_utf8_lossy(&alt_head.stdout).trim().to_string();

    let build_borrower = || {
        let repo = create_committed_repo_via_cli();
        let output = run_libra_command(
            &["alternates", "add", alt.path().to_str().expect("alt path")],
            repo.path(),
        );
        assert_cli_success(&output, "alternates add");
        let output = run_libra_command(
            &["update-ref", "refs/heads/from-alt", &alt_head],
            repo.path(),
        );
        assert_cli_success(&output, "update-ref to the alternate commit");
        repo
    };

    // Control: without -l the alternate's objects land in the local pack.
    let control = build_borrower();
    let output = run_libra_command(&["repack", "-a", "-d"], control.path());
    assert_cli_success(&output, "repack -a -d (no -l)");
    let packs = pack_files(control.path());
    assert_eq!(packs.len(), 1, "control repack writes one pack");
    let idx = packs[0].with_extension("idx");
    let listed = run_libra_command(
        &["verify-pack", "-v", idx.to_str().unwrap()],
        control.path(),
    );
    assert_cli_success(&listed, "verify-pack -v control");
    assert!(
        String::from_utf8_lossy(&listed.stdout).contains(&alt_head),
        "without -l the alternate commit is packed locally"
    );

    // With -l the same repack must skip alternate-only objects entirely.
    let local = build_borrower();
    let output = run_libra_command(&["repack", "-a", "-d", "-l"], local.path());
    assert_cli_success(&output, "repack -a -d -l");
    let packs = pack_files(local.path());
    assert_eq!(packs.len(), 1, "local repack writes one pack");
    let idx = packs[0].with_extension("idx");
    let listed = run_libra_command(&["verify-pack", "-v", idx.to_str().unwrap()], local.path());
    assert_cli_success(&listed, "verify-pack -v local");
    assert!(
        !String::from_utf8_lossy(&listed.stdout).contains(&alt_head),
        "with -l the alternate commit must not be packed"
    );

    // The borrowed ref still resolves through the alternate (R5)...
    let output = run_libra_command(&["cat-file", "-t", &alt_head], local.path());
    assert_cli_success(&output, "borrowed commit readable through the alternate");
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "commit");
    // ...and nothing in the alternate's own store was written or deleted.
    assert_eq!(
        pack_files(alt.path()).len(),
        0,
        "the alternate's store must stay untouched"
    );
}

// ---------------------------------------------------------------------------
// M-DELTA (ADR-MX-09): delta window/depth forwarding and validation.
// ---------------------------------------------------------------------------

/// Build a repository whose history contains three near-duplicate large blob
/// versions (long shared prefix, small per-version tail) — the corpus for
/// M-DELTA D1/D2: `--window > 0` must delta-compress it, `--window=0` must not.
fn repo_with_duplicate_blob_versions() -> tempfile::TempDir {
    let repo = create_committed_repo_via_cli();
    let versions: [&[usize]; 3] = [&[], &[100, 1500], &[100, 1500, 2000, 2500]];
    for (version, changed) in versions.iter().enumerate() {
        let mut body = String::new();
        for line in 0..2000 {
            if changed.contains(&line) {
                body.push_str(&format!("line {line:06} CHANGED in version {version}\n"));
            } else {
                body.push_str("line 000000 shared filler content for delta compression\n");
            }
        }
        fs::write(repo.path().join("big.txt"), &body).expect("write big file");
        let output = run_libra_command(&["add", "big.txt"], repo.path());
        assert_cli_success(&output, "add big.txt");
        let output = run_libra_command(
            &["commit", "-m", &format!("version {version}"), "--no-verify"],
            repo.path(),
        );
        assert_cli_success(&output, "commit big.txt version");
    }
    repo
}

/// Per-entry delta statistics of a pack file, obtained by walking the raw
/// entries with git-internal's pack decoder (the same one the object store
/// uses to read packs).
struct PackDeltaStats {
    #[allow(dead_code)] // kept for diagnostics alongside the assertions
    objects: usize,
    deltas: usize,
    max_chain: usize,
}

/// Scan a pack's raw entries and count delta objects and the longest delta
/// chain. The matching index supplies object-id → pack-offset mapping so
/// REF_DELTA bases (which are named by hash) resolve to their entry's chain
/// length. Sha1 repositories only (every test repository above uses the
/// default object format).
fn scan_pack_delta_stats(pack_path: &Path, idx_path: &Path) -> PackDeltaStats {
    let file = fs::File::open(pack_path).expect("open pack file");
    let mut reader = io::BufReader::new(file);
    let mut header = [0u8; 12];
    reader.read_exact(&mut header).expect("read pack header");
    assert_eq!(&header[..4], b"PACK", "pack magic");
    let object_count = u32::from_be_bytes(header[8..12].try_into().expect("object count")) as usize;

    let idx_bytes = fs::read(idx_path).expect("read pack index");
    let idx = git_internal::internal::pack::pack_index::parse_idx_v2(&idx_bytes, HashKind::Sha1)
        .expect("parse pack index v2");
    let offset_by_hash: HashMap<ObjectHash, usize> = idx
        .entries
        .iter()
        .map(|entry| (entry.hash, entry.offset as usize))
        .collect();

    // Bases are decoded before the deltas that reference them, so walking the
    // entries in order lets us resolve every base's chain length.
    let mut chain_by_offset: HashMap<usize, usize> = HashMap::new();
    let mut offset = 12usize;
    let mut deltas = 0usize;
    let mut max_chain = 0usize;
    for _ in 0..object_count {
        let entry_offset = offset;
        let obj = Pack::decode_pack_object_with_kind(HashKind::Sha1, &mut reader, &mut offset)
            .expect("decode pack entry")
            .expect("object count matches entries");
        if obj.is_delta_in_pack {
            deltas += 1;
            let base_chain = if let Some(base_offset) = obj.offset_delta() {
                chain_by_offset.get(&base_offset).copied().unwrap_or(0)
            } else if let Some(base_hash) = obj.hash_delta() {
                offset_by_hash
                    .get(&base_hash)
                    .and_then(|base| chain_by_offset.get(base).copied())
                    .unwrap_or(0)
            } else {
                0
            };
            let chain = base_chain + 1;
            chain_by_offset.insert(entry_offset, chain);
            max_chain = max_chain.max(chain);
        } else {
            chain_by_offset.insert(entry_offset, 0);
        }
    }
    PackDeltaStats {
        objects: object_count,
        deltas,
        max_chain,
    }
}

/// M-DELTA D1/D2: `--window=10 --depth=50` produces delta objects and a pack
/// no larger than the window-0 pack of the same corpus; `--window=0` produces
/// no deltas at all.
#[test]
fn repack_matrix_d1_d2_window_controls_delta_compression() {
    let delta_repo = repo_with_duplicate_blob_versions();
    let output = run_libra_command(
        &["repack", "-a", "-d", "--window=10", "--depth=50"],
        delta_repo.path(),
    );
    assert_cli_success(&output, "repack --window=10 --depth=50");
    let delta_packs = pack_files(delta_repo.path());
    assert_eq!(delta_packs.len(), 1, "windowed repack writes one pack");
    let delta_stats = scan_pack_delta_stats(&delta_packs[0], &delta_packs[0].with_extension("idx"));
    assert!(
        delta_stats.deltas > 0,
        "D1: window > 0 must produce delta objects"
    );
    let delta_size = fs::metadata(&delta_packs[0]).expect("pack metadata").len();

    let no_delta_repo = repo_with_duplicate_blob_versions();
    let output = run_libra_command(&["repack", "-a", "-d", "--window=0"], no_delta_repo.path());
    assert_cli_success(&output, "repack --window=0");
    let no_delta_packs = pack_files(no_delta_repo.path());
    assert_eq!(no_delta_packs.len(), 1, "window-0 repack writes one pack");
    let no_delta_stats =
        scan_pack_delta_stats(&no_delta_packs[0], &no_delta_packs[0].with_extension("idx"));
    assert_eq!(
        no_delta_stats.deltas, 0,
        "D2: window=0 must produce no deltas"
    );
    let no_delta_size = fs::metadata(&no_delta_packs[0])
        .expect("pack metadata")
        .len();

    assert!(
        delta_size <= no_delta_size,
        "D1: delta pack ({delta_size}B) must not exceed the no-delta pack ({no_delta_size}B)"
    );

    // D4: both packs round-trip through verify-pack, index-pack and fsck.
    for repo in [&delta_repo, &no_delta_repo] {
        let packs = pack_files(repo.path());
        let idx = packs[0].with_extension("idx");
        let output = run_libra_command(
            &["verify-pack", "--stat-only", idx.to_str().unwrap()],
            repo.path(),
        );
        assert_cli_success(&output, "verify-pack --stat-only on the pack");
        let output = run_libra_command(&["verify-pack", idx.to_str().unwrap()], repo.path());
        assert_cli_success(&output, "verify-pack on the pack");
        let output = run_libra_command(&["index-pack", packs[0].to_str().unwrap()], repo.path());
        assert_cli_success(&output, "index-pack validates the pack");
        let output = run_libra_command(&["fsck"], repo.path());
        assert_cli_success(&output, "fsck after the windowed repack");
        let output = run_libra_command(&["log", "--oneline"], repo.path());
        assert_cli_success(&output, "history readable after the windowed repack");
    }
}

/// M-DELTA D3: `--depth=1` is accepted and yields a valid pack. Chain-depth
/// capping itself is an upstream dependency (ER-MX-03): git-internal's encoder
/// applies a fixed `MAX_CHAIN_LEN` of 50 and ignores the depth value, so this
/// test asserts the actual guarantee — every delta chain stays within the
/// encoder's 50 cap — and documents the gap instead of asserting chains ≤ 1.
#[test]
fn repack_matrix_d3_depth_chain_cap_is_upstream_dependency() {
    let repo = repo_with_duplicate_blob_versions();
    let output = run_libra_command(&["repack", "-a", "-d", "--depth=1"], repo.path());
    assert_cli_success(&output, "repack --depth=1");
    let packs = pack_files(repo.path());
    assert_eq!(packs.len(), 1, "depth=1 repack writes one pack");
    let stats = scan_pack_delta_stats(&packs[0], &packs[0].with_extension("idx"));
    assert!(
        stats.max_chain <= 50,
        "ER-MX-03: the upstream encoder caps chains at 50, observed {}",
        stats.max_chain
    );
    let idx = packs[0].with_extension("idx");
    let output = run_libra_command(&["verify-pack", idx.to_str().unwrap()], repo.path());
    assert_cli_success(&output, "verify-pack accepts the depth=1 pack");
    let output = run_libra_command(&["fsck"], repo.path());
    assert_cli_success(&output, "fsck accepts the depth=1 pack");
}

/// M-DELTA D5: without flags the window/depth come from `pack.window` /
/// `pack.depth`, defaulting to Git's 10/50; explicit flags win.
#[test]
fn repack_matrix_d5_defaults_and_config_resolution() {
    let repo = repo_with_history();

    // Git defaults when nothing is configured.
    let output = run_libra_command(&["--json", "repack", "-a", "-d"], repo.path());
    assert_cli_success(&output, "repack --json (defaults)");
    let value: Value = serde_json::from_slice(&output.stdout).expect("json envelope");
    assert_eq!(
        value["data"]["window"].as_u64(),
        Some(10),
        "default window is Git's 10"
    );
    assert_eq!(
        value["data"]["depth"].as_u64(),
        Some(50),
        "default depth is Git's 50"
    );

    // pack.window / pack.depth configuration is honored...
    let output = run_libra_command(&["config", "set", "pack.window", "0"], repo.path());
    assert_cli_success(&output, "config set pack.window 0");
    let output = run_libra_command(&["config", "set", "pack.depth", "17"], repo.path());
    assert_cli_success(&output, "config set pack.depth 17");
    let output = run_libra_command(&["--json", "repack", "-a", "-d"], repo.path());
    assert_cli_success(&output, "repack --json (configured)");
    let value: Value = serde_json::from_slice(&output.stdout).expect("json envelope");
    assert_eq!(
        value["data"]["window"].as_u64(),
        Some(0),
        "pack.window=0 is honored"
    );
    assert_eq!(
        value["data"]["depth"].as_u64(),
        Some(17),
        "pack.depth=17 is honored"
    );

    // ...and explicit flags override the configuration.
    let output = run_libra_command(
        &["--json", "repack", "-a", "-d", "--window=5", "--depth=7"],
        repo.path(),
    );
    assert_cli_success(&output, "repack --json (explicit flags)");
    let value: Value = serde_json::from_slice(&output.stdout).expect("json envelope");
    assert_eq!(value["data"]["window"].as_u64(), Some(5));
    assert_eq!(value["data"]["depth"].as_u64(), Some(7));
}

/// M-DELTA D6: invalid `--window`/`--depth` values are usage errors (exit
/// 129, LBR-CLI-002) with zero writes — no pack appears and no loose object
/// is removed.
#[test]
fn repack_matrix_d6_invalid_window_depth_zero_writes() {
    let repo = repo_with_history();
    let loose_before = loose_object_ids(repo.path());
    assert!(
        !loose_before.is_empty(),
        "precondition: loose objects exist"
    );

    for bad in [
        &["repack", "-a", "-d", "--window=abc"][..],
        &["repack", "-a", "-d", "--window=-3"][..],
        &["repack", "-a", "-d", "--depth=nope"][..],
        &["repack", "-a", "-d", "--depth=-1"][..],
    ] {
        let output = run_libra_command(bad, repo.path());
        assert_eq!(
            output.status.code(),
            Some(129),
            "{bad:?} must exit 129, stderr: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    // Zero writes: no packs appeared and every loose object is untouched.
    assert!(
        pack_files(repo.path()).is_empty(),
        "no pack may be written for usage errors"
    );
    let loose_after = loose_object_ids(repo.path());
    assert_eq!(
        loose_before, loose_after,
        "loose objects unchanged after usage errors"
    );
}

/// ADR-MX-09: `gc --aggressive` forwards Git's `gc.aggressiveWindow` default
/// (250) into the repack step. With `pack.window=0` configured, a plain `gc`
/// honors the configuration and writes no deltas; `gc --aggressive` must
/// override it with its hard-coded enlarged window and delta-compress the
/// corpus (the only way to beat the configured window is the explicit value
/// gc passes).
#[test]
fn gc_aggressive_forwards_enlarged_delta_window() {
    let build = || {
        let repo = repo_with_duplicate_blob_versions();
        let output = run_libra_command(&["config", "set", "pack.window", "0"], repo.path());
        assert_cli_success(&output, "config set pack.window 0");
        repo
    };

    // Control: the default gc honors pack.window=0 and produces no deltas.
    let default_repo = build();
    let output = run_libra_command(&["gc"], default_repo.path());
    assert_cli_success(&output, "gc with pack.window=0");
    let default_packs = pack_files(default_repo.path());
    assert_eq!(default_packs.len(), 1, "default gc leaves one pack");
    let default_stats =
        scan_pack_delta_stats(&default_packs[0], &default_packs[0].with_extension("idx"));
    assert_eq!(
        default_stats.deltas, 0,
        "default gc honors pack.window=0 (no deltas)"
    );

    // --aggressive overrides the configured window with the 250 aggressive
    // window, so the same corpus delta-compresses.
    let aggressive_repo = build();
    let output = run_libra_command(&["gc", "--aggressive"], aggressive_repo.path());
    assert_cli_success(&output, "gc --aggressive with pack.window=0");
    let aggressive_packs = pack_files(aggressive_repo.path());
    assert_eq!(aggressive_packs.len(), 1, "gc --aggressive leaves one pack");
    let aggressive_stats = scan_pack_delta_stats(
        &aggressive_packs[0],
        &aggressive_packs[0].with_extension("idx"),
    );
    assert!(
        aggressive_stats.deltas > 0,
        "gc --aggressive overrides pack.window=0 with the aggressive window"
    );

    // Both packs verify and the history stays readable.
    for repo in [&aggressive_repo, &default_repo] {
        let packs = pack_files(repo.path());
        let idx = packs[0].with_extension("idx");
        let output = run_libra_command(&["verify-pack", idx.to_str().unwrap()], repo.path());
        assert_cli_success(&output, "verify-pack after gc");
        let output = run_libra_command(&["log", "--oneline"], repo.path());
        assert_cli_success(&output, "log after gc");
    }
}
