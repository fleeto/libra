//! Tests for `count-objects`: plain/verbose/human output, JSON envelope,
//! repository forms, error exits, zero-write guarantee, pack fields,
//! prune-packable, garbage diagnostics, and alternates.

use std::{
    fs,
    path::{Path, PathBuf},
};

use super::*;

/// Every file under `.libra/objects`, sorted (relative paths).
fn object_files(repo: &Path) -> Vec<PathBuf> {
    fn walk(dir: &Path, base: &Path, out: &mut Vec<PathBuf>) {
        let Ok(entries) = fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(&path, base, out);
            } else {
                out.push(path.strip_prefix(base).expect("under base").to_path_buf());
            }
        }
    }
    let base = repo.join(".libra").join("objects");
    let mut out = Vec::new();
    walk(&base, &base, &mut out);
    out.sort();
    out
}

fn object_bytes(repo: &Path) -> u64 {
    object_files(repo)
        .iter()
        .map(|rel| {
            fs::metadata(repo.join(".libra").join("objects").join(rel))
                .unwrap()
                .len()
        })
        .sum()
}

fn count_output(repo: &Path, args: &[&str]) -> String {
    let mut argv = vec!["count-objects"];
    argv.extend_from_slice(args);
    let out = run_libra_command(&argv, repo);
    assert_cli_success(&out, "count-objects should succeed");
    String::from_utf8_lossy(&out.stdout).to_string()
}

fn count_verbose_field(output: &str, field: &str) -> u64 {
    output
        .lines()
        .find_map(|line| line.strip_prefix(&format!("{field}: ")))
        .unwrap_or_else(|| panic!("missing field {field} in:\n{output}"))
        .trim()
        .parse()
        .unwrap_or_else(|e| panic!("field {field} not numeric: {e}"))
}

fn committed_repo() -> tempfile::TempDir {
    let repo = tempdir().expect("temp repo");
    init_repo_via_cli(repo.path());
    configure_identity_via_cli(repo.path());
    fs::write(repo.path().join("tracked.txt"), "tracked\n").expect("write tracked");
    assert_cli_success(
        &run_libra_command(&["add", "tracked.txt"], repo.path()),
        "add tracked",
    );
    assert_cli_success(
        &run_libra_command(&["commit", "-m", "base", "--no-verify"], repo.path()),
        "commit base",
    );
    repo
}

#[test]
fn count_objects_plain_matches_loose_inventory() {
    let repo = committed_repo();
    let output = count_output(repo.path(), &[]);
    let count: u64 = output
        .split_whitespace()
        .next()
        .expect("count word")
        .parse()
        .expect("numeric count");
    let loose = object_files(repo.path()).len() as u64;
    assert_eq!(count, loose, "plain count must match the loose inventory");
    assert!(
        output.contains("kilobytes"),
        "plain size is in kilobytes: {output}"
    );
}

#[test]
fn count_objects_human_readable_renders_units() {
    let repo = committed_repo();
    let output = count_output(repo.path(), &["-H"]);
    let trimmed = output.trim_end();
    assert!(
        trimmed.ends_with("bytes") || trimmed.ends_with("KiB"),
        "human-readable size unit: {output}"
    );
    let verbose = count_output(repo.path(), &["-vH"]);
    assert!(
        verbose.contains("size: "),
        "verbose -H keeps the size field"
    );
}

#[test]
fn count_objects_is_read_only() {
    let repo = committed_repo();
    let before_files = object_files(repo.path());
    let before_bytes = object_bytes(repo.path());
    let before_count = count_output(repo.path(), &["-v"]);

    let _ = count_output(repo.path(), &[]);
    let after_count = count_output(repo.path(), &["-v"]);

    assert_eq!(
        before_files,
        object_files(repo.path()),
        "no new object files"
    );
    assert_eq!(
        before_bytes,
        object_bytes(repo.path()),
        "no object bytes added"
    );
    assert_eq!(before_count, after_count, "count-objects -v is idempotent");
}

#[test]
fn count_objects_repository_forms() {
    let repo = committed_repo();

    // Subdirectory gives the same census as the root.
    let sub = repo.path().join("sub");
    fs::create_dir_all(&sub).expect("create subdir");
    let root_out = count_output(repo.path(), &[]);
    let sub_out = count_output(&sub, &[]);
    assert_eq!(root_out, sub_out, "subdirectory census matches root");

    // Bare repository exits 0.
    let bare = tempdir().expect("bare repo");
    assert_cli_success(
        &run_libra_command(&["init", "--bare"], bare.path()),
        "init bare",
    );
    assert_cli_success(
        &run_libra_command(&["count-objects", "-v"], bare.path()),
        "count-objects in bare repo",
    );
}

#[test]
fn count_objects_error_exits() {
    let outside = tempdir().expect("outside repo");
    let out = run_libra_command(&["count-objects"], outside.path());
    assert_eq!(out.status.code(), Some(128), "outside repo exits 128");
    let (_human, report) = parse_cli_error_stderr(&out.stderr);
    assert_eq!(report.error_code, "LBR-REPO-001");

    let repo = committed_repo();
    let out = run_libra_command(&["count-objects", "foo"], repo.path());
    assert_eq!(out.status.code(), Some(129), "positional arg exits 129");
    let (_human, report) = parse_cli_error_stderr(&out.stderr);
    assert_eq!(report.error_code, "LBR-CLI-002");

    let out = run_libra_command(&["count-objects", "--nope"], repo.path());
    assert_eq!(out.status.code(), Some(129), "unknown flag exits 129");
}

#[test]
fn count_objects_json_envelope() {
    let repo = committed_repo();
    let out = run_libra_command(&["--json", "count-objects"], repo.path());
    assert_cli_success(&out, "json count-objects");
    let json = parse_json_stdout(&out);
    assert_eq!(json["command"], "count-objects");
    assert_eq!(json["data"]["verbose"], false);
    assert!(json["data"]["count"].is_u64());
    assert!(json["data"]["size_bytes"].is_u64());
    assert!(json["data"]["size_kib"].is_u64());
    assert!(
        json["data"].get("in_pack").is_none(),
        "plain JSON omits pack fields"
    );
}

#[test]
fn count_objects_sha256_repository() {
    let repo = create_committed_repo_with_format("sha256");
    let output = count_output(repo.path(), &["-v"]);
    let count = count_verbose_field(&output, "count");
    assert!(
        count >= 3,
        "sha256 loose objects (62-char names) counted: {count}"
    );
}

#[test]
fn count_objects_verbose_field_order() {
    let repo = committed_repo();
    let output = count_output(repo.path(), &["-v"]);
    let keys: Vec<&str> = output
        .lines()
        .filter_map(|line| line.split(':').next())
        .filter(|k| !k.starts_with("warning"))
        .collect();
    assert_eq!(
        keys,
        vec![
            "count",
            "size",
            "in-pack",
            "packs",
            "size-pack",
            "prune-packable",
            "garbage",
            "size-garbage"
        ],
        "exact Git field names and order"
    );
}

#[test]
fn count_objects_verbose_pack_fields_and_prune_packable() {
    let repo = committed_repo();
    assert_cli_success(&run_libra_command(&["repack"], repo.path()), "repack");
    let output = count_output(repo.path(), &["-v"]);
    assert!(count_verbose_field(&output, "packs") >= 1, "pack counted");
    assert!(
        count_verbose_field(&output, "in-pack") > 0,
        "packed objects counted"
    );
    assert!(
        count_verbose_field(&output, "prune-packable") > 0,
        "loose copies also in a pack are prune-packable"
    );
    // Plain form must not open packs.
    let plain = count_output(repo.path(), &[]);
    assert_eq!(plain.lines().count(), 1);
}

#[test]
fn count_objects_verbose_does_not_build_idx() {
    let repo = committed_repo();
    assert_cli_success(&run_libra_command(&["repack"], repo.path()), "repack");
    let pack_dir = repo.path().join(".libra").join("objects").join("pack");
    let idx: Vec<PathBuf> = fs::read_dir(&pack_dir)
        .expect("pack dir")
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|ext| ext == "idx"))
        .collect();
    assert!(!idx.is_empty(), "repack produced an idx");
    for path in &idx {
        fs::remove_file(path).expect("remove idx");
    }

    let output = count_output(repo.path(), &["-v"]);
    assert!(
        count_verbose_field(&output, "packs") == 0,
        "idx-less pack is skipped, not rebuilt"
    );
    for path in &idx {
        assert!(
            !path.exists(),
            "count-objects must not recreate {}",
            path.display()
        );
    }
    assert!(
        count_verbose_field(&output, "garbage") >= 1,
        "idx-less pack is garbage"
    );
}

#[test]
fn count_objects_garbage_matrix() {
    let repo = committed_repo();
    let objects = repo.path().join(".libra").join("objects");
    fs::create_dir_all(objects.join("ab")).expect("shard");
    fs::write(objects.join("ab").join("badname"), b"x").expect("loose garbage");
    let pack_dir = objects.join("pack");
    fs::create_dir_all(&pack_dir).expect("pack dir");
    fs::write(pack_dir.join("junkfile"), b"x").expect("pack garbage");
    fs::write(pack_dir.join("pack-aaa.idx"), b"x").expect("idx-only group");

    let output = count_output(repo.path(), &["-v"]);
    assert!(
        count_verbose_field(&output, "garbage") >= 3,
        "garbage counted"
    );
    let raw = run_libra_command(&["count-objects", "-v"], repo.path());
    let stderr = String::from_utf8_lossy(&raw.stderr);
    assert!(
        stderr.contains("no corresponding .pack"),
        "idx-only reason: {stderr}"
    );

    let out = run_libra_command(&["--json", "count-objects", "-v"], repo.path());
    assert_cli_success(&out, "verbose json");
    let json = parse_json_stdout(&out);
    let reasons: Vec<&str> = json["data"]["garbage_entries"]
        .as_array()
        .expect("entries array")
        .iter()
        .map(|e| e["reason"].as_str().expect("reason"))
        .collect();
    assert!(reasons.contains(&"garbage found"));
    assert!(reasons.contains(&"no corresponding .pack"));
}

#[test]
fn count_objects_alternates_matrix() {
    let repo = committed_repo();
    let alt = committed_repo();
    let alt_objects = alt.path().join(".libra").join("objects");
    fs::create_dir_all(alt_objects.join("pack")).expect("alt pack dir");
    fs::write(alt_objects.join("pack").join("junkfile"), b"x").expect("alt garbage");

    let info = repo.path().join(".libra").join("objects").join("info");
    fs::create_dir_all(&info).expect("info dir");
    fs::write(info.join("alternates"), alt_objects.display().to_string()).expect("alternates");

    let output = count_output(repo.path(), &["-v"]);
    assert!(output.contains("alternate: "), "alternate line: {output}");
    assert!(
        count_verbose_field(&output, "garbage") >= 1,
        "alternate pack garbage is counted"
    );
    // Local pack fields stay local: the alternate has no packs.
    assert_eq!(count_verbose_field(&output, "in-pack"), 0);
}

#[test]
fn count_objects_alternate_json_keeps_raw_space_path() {
    let repo = committed_repo();
    let root = tempdir().expect("alternate fixture root");
    let alternate = root.path().join("alternate repo");
    fs::create_dir_all(&alternate).expect("alternate repo dir");
    init_repo_via_cli(&alternate);
    let alternate_objects = alternate
        .join(".libra/objects")
        .canonicalize()
        .expect("alternate objects");
    let info = repo.path().join(".libra/objects/info");
    fs::create_dir_all(&info).expect("info dir");
    fs::write(
        info.join("alternates"),
        format!("{}\n", alternate_objects.display()),
    )
    .expect("alternates");
    let before = object_files(repo.path());
    let out = run_libra_command(&["--json", "count-objects", "-v"], repo.path());
    assert_cli_success(&out, "raw alternate JSON");
    let body = parse_json_stdout(&out);
    assert_eq!(
        body["data"]["alternates"],
        serde_json::json!([alternate_objects.to_str().expect("UTF-8 fixture")])
    );
    #[cfg(unix)]
    {
        let text = count_output(repo.path(), &["-v"]);
        assert!(
            text.lines()
                .any(|line| line == format!("alternate: {}", alternate_objects.display())),
            "Git leaves ordinary spaces unquoted: {text}"
        );
    }
    assert_eq!(
        object_files(repo.path()),
        before,
        "census must remain read-only"
    );
}

#[test]
fn count_objects_garbage_truncation_starts_after_one_hundred() {
    let repo = committed_repo();
    let pack = repo.path().join(".libra/objects/pack");
    fs::create_dir_all(&pack).expect("pack fixture");
    for n in 0..100 {
        fs::write(pack.join(format!("garbage-{n}")), b"x").expect("garbage fixture");
    }
    let first = run_libra_command(&["--json", "count-objects", "-v"], repo.path());
    assert_cli_success(&first, "exactly one hundred details");
    let first = parse_json_stdout(&first);
    assert_eq!(first["data"]["garbage"], 100);
    assert_eq!(
        first["data"]["garbage_entries"].as_array().unwrap().len(),
        100
    );
    assert_eq!(first["data"]["garbage_entries_truncated"], false);
    fs::write(pack.join("garbage-100"), b"x").expect("one more fixture");
    let second = run_libra_command(&["--json", "count-objects", "-v"], repo.path());
    assert_cli_success(&second, "one hundred and one details");
    let second = parse_json_stdout(&second);
    assert_eq!(second["data"]["garbage"], 101);
    assert_eq!(
        second["data"]["garbage_entries"].as_array().unwrap().len(),
        100
    );
    assert_eq!(second["data"]["garbage_entries_truncated"], true);
}

#[test]
fn count_objects_skips_corrupt_index_pack_totals() {
    let repo = committed_repo();
    let pack = repo.path().join(".libra/objects/pack");
    fs::create_dir_all(&pack).expect("pack fixture");
    fs::write(pack.join("pack-corrupt.pack"), b"not a pack").expect("pack fixture");
    fs::write(pack.join("pack-corrupt.idx"), b"not an index").expect("index fixture");
    let out = run_libra_command(&["--json", "count-objects", "-v"], repo.path());
    assert_cli_success(&out, "bad index is skipped as documented");
    let body = parse_json_stdout(&out);
    assert_eq!(body["data"]["packs"], 0);
    assert_eq!(body["data"]["in_pack"], 0);
    assert_eq!(body["data"]["size_pack_bytes"], 0);
}

#[test]
fn count_objects_orphan_pack_sidecar_keeps_git_reason() {
    let repo = committed_repo();
    let pack = repo.path().join(".libra/objects/pack");
    fs::create_dir_all(&pack).expect("pack fixture");
    fs::write(pack.join("pack-orphan.keep"), b"fixture").expect("orphan sidecar");
    let out = run_libra_command(&["--json", "count-objects", "-v"], repo.path());
    assert_cli_success(&out, "orphan sidecar JSON");
    let body = parse_json_stdout(&out);
    assert_eq!(body["data"]["garbage"], 1);
    assert_eq!(
        body["data"]["garbage_entries"][0]["reason"],
        "no corresponding .idx or .pack"
    );
    let human = run_libra_command(&["count-objects", "-v"], repo.path());
    assert_cli_success(&human, "orphan sidecar warning");
    let stderr = String::from_utf8_lossy(&human.stderr);
    assert!(
        stderr.contains("warning: no corresponding .idx or .pack:")
            && stderr.contains("pack-orphan.keep"),
        "Git-compatible orphan warning: {stderr}"
    );
}
