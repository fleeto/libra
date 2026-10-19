//! Implements `count-objects` — a read-only census of the object database.
//!
//! Git parity (issue #483): the command reports loose object count/size, and
//! with `-v` the full Git field set (`count`, `size`, `in-pack`, `packs`,
//! `size-pack`, `prune-packable`, `garbage`, `size-garbage`) followed by
//! `alternate:` lines. It never rebuilds pack indexes, never migrates the
//! schema, and never touches the network.
//!
//! Intentional differences (documented in the command docs and COMPATIBILITY):
//! garbage/alternate paths are absolute; an unreadable shard or pack directory
//! fails closed with `LBR-IO-001`; Operation Log snapshot objects written by
//! real mutations are counted; only the local tier is inspected.

use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use clap::Parser;
use git_internal::hash::get_hash_kind;
use serde::Serialize;

use crate::{
    internal::alternates,
    utils::{
        error::{CliError, CliResult, emit_warning},
        output::{OutputConfig, emit_json_data},
    },
};

const COUNT_OBJECTS_EXAMPLES: &str = "\
EXAMPLES:
    libra count-objects                      Count loose objects and their total size
    libra count-objects -H                   Print sizes in human-readable units
    libra count-objects -v                   Print the verbose Git field set
    libra count-objects -v --json            Structured object-database census for agents";

/// Upper bound on detailed garbage entries carried in structured output.
const GARBAGE_ENTRY_LIMIT: usize = 100;

/// Pack-directory extensions Git recognises; anything else is garbage.
const KNOWN_PACK_EXTENSIONS: [&str; 7] =
    ["idx", "rev", "pack", "bitmap", "keep", "promisor", "mtimes"];

#[derive(Parser, Debug)]
#[command(after_help = COUNT_OBJECTS_EXAMPLES)]
pub struct CountObjectsArgs {
    /// Print sizes in human-readable units
    #[arg(short = 'H', long = "human-readable")]
    pub human_readable: bool,

    /// Print the verbose Git field set (packs, garbage, alternates)
    #[arg(short, long)]
    pub verbose: bool,
}

#[derive(Debug, Clone, Serialize)]
struct GarbageEntry {
    path: String,
    reason: &'static str,
}

#[derive(Debug, Default, Serialize)]
struct Census {
    verbose: bool,
    count: u64,
    size_bytes: u64,
    size_kib: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    in_pack: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    packs: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    size_pack_bytes: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    size_pack_kib: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    prune_packable: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    garbage: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    size_garbage_bytes: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    size_garbage_kib: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    garbage_entries: Option<Vec<GarbageEntry>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    garbage_entries_truncated: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    alternates: Option<Vec<String>>,
}

impl Census {
    fn record_garbage(&mut self, path: &Path, reason: &'static str) {
        self.garbage = Some(self.garbage.unwrap_or(0) + 1);
        let size = fs::symlink_metadata(path).map(|m| m.len()).unwrap_or(0);
        self.size_garbage_bytes = Some(self.size_garbage_bytes.unwrap_or(0) + size);
        let entries = self.garbage_entries.get_or_insert_with(Vec::new);
        if entries.len() < GARBAGE_ENTRY_LIMIT {
            entries.push(GarbageEntry {
                path: path.display().to_string(),
                reason,
            });
        }
    }
}

pub async fn execute(args: CountObjectsArgs) -> Result<(), String> {
    execute_safe(args, &OutputConfig::default())
        .await
        .map_err(|err| err.render())
}

/// # Side Effects
///
/// Read-only. Scans `objects/` (and, for `-v`, the pack directories of local
/// and alternate object stores) without creating, repairing, or migrating any
/// storage.
///
/// # Errors
///
/// `LBR-REPO-001` outside a repository; `LBR-IO-001` when a shard or pack
/// directory cannot be read (fail-closed); `LBR-CLI-002` for unknown flags or
/// positional arguments.
pub async fn execute_safe(args: CountObjectsArgs, output: &OutputConfig) -> CliResult<()> {
    let storage =
        crate::utils::util::try_get_storage_path(None).map_err(|_| CliError::repo_not_found())?;
    let objects_dir = storage.join("objects");

    let mut census = Census {
        verbose: args.verbose,
        ..Census::default()
    };
    if args.verbose {
        census.in_pack = Some(0);
        census.packs = Some(0);
        census.size_pack_bytes = Some(0);
        census.size_pack_kib = Some(0);
        census.prune_packable = Some(0);
        census.garbage = Some(0);
        census.size_garbage_bytes = Some(0);
        census.size_garbage_kib = Some(0);
        census.garbage_entries = Some(Vec::new());
        census.garbage_entries_truncated = Some(false);
        census.alternates = Some(Vec::new());
    }

    let hexsz = get_hash_kind().size() * 2;
    let alternates = if args.verbose {
        alternates::resolve_chain(&objects_dir)
    } else {
        Vec::new()
    };
    let alternate_pack_idxs = alternates
        .iter()
        .flat_map(|dir| list_idx_files(&dir.join("pack")))
        .collect::<Vec<_>>();
    let mut pack_idxs = list_idx_files(&objects_dir.join("pack"));
    pack_idxs.extend(alternate_pack_idxs.iter().cloned());

    // Loose census (primary source only; alternates contribute pack garbage).
    scan_loose(&objects_dir, hexsz, &pack_idxs, &mut census)?;

    if args.verbose {
        scan_pack_dir(&objects_dir.join("pack"), true, &mut census)?;
        for alternate in &alternates {
            scan_pack_dir(&alternate.join("pack"), false, &mut census)?;
        }
    }

    census.size_kib = census.size_bytes / 1024;
    if let Some(bytes) = census.size_pack_bytes {
        census.size_pack_kib = Some(bytes / 1024);
    }
    if let Some(bytes) = census.size_garbage_bytes {
        census.size_garbage_kib = Some(bytes / 1024);
    }
    if let Some(count) = census.garbage {
        census.garbage_entries_truncated = Some(count > GARBAGE_ENTRY_LIMIT as u64);
    }

    if output.is_json() {
        if args.verbose {
            census.alternates = Some(raw_alternate_paths(&alternates)?);
        }
        return emit_json_data("count-objects", &census, output);
    }

    if args.verbose {
        for entry in census.garbage_entries.as_deref().unwrap_or(&[]) {
            emit_warning(format!("{}: {}", entry.reason, entry.path));
        }
    }

    render_human(&census, args.human_readable, &alternates);
    Ok(())
}

fn raw_alternate_paths(paths: &[PathBuf]) -> CliResult<Vec<String>> {
    paths
        .iter()
        .map(|path| {
            path.to_str().map(str::to_owned).ok_or_else(|| {
                CliError::io("alternate path is not valid UTF-8; use text output to inspect its escaped bytes")
            })
        })
        .collect()
}

fn render_human(census: &Census, human_readable: bool, alternates: &[PathBuf]) {
    if census.verbose {
        println!("count: {}", census.count);
        println!("size: {}", render_size(census.size_bytes, human_readable));
        println!("in-pack: {}", census.in_pack.unwrap_or(0));
        println!("packs: {}", census.packs.unwrap_or(0));
        println!(
            "size-pack: {}",
            render_size(census.size_pack_bytes.unwrap_or(0), human_readable)
        );
        println!("prune-packable: {}", census.prune_packable.unwrap_or(0));
        println!("garbage: {}", census.garbage.unwrap_or(0));
        println!(
            "size-garbage: {}",
            render_size(census.size_garbage_bytes.unwrap_or(0), human_readable)
        );
        for alternate in alternates {
            println!("alternate: {}", quote_c_style(alternate));
        }
    } else if human_readable {
        println!(
            "{} objects, {}",
            census.count,
            humanise_bytes(census.size_bytes)
        );
    } else {
        println!("{} objects, {} kilobytes", census.count, census.size_kib);
    }
}

fn render_size(bytes: u64, human_readable: bool) -> String {
    if human_readable {
        humanise_bytes(bytes)
    } else {
        format!("{}", bytes / 1024)
    }
}

/// Match Git's strict unit thresholds and integer hundredths calculation.
/// Reference: Git 1630431f32, strbuf.c humanise_bytes and quote.c cq_lookup.
fn humanise_bytes(bytes: u64) -> String {
    if bytes > 1 << 30 {
        format!(
            "{}.{:02} GiB",
            bytes >> 30,
            (bytes & ((1 << 30) - 1)) / 10_737_419
        )
    } else if bytes > 1 << 20 {
        let rounded = bytes + 5_243;
        format!(
            "{}.{:02} MiB",
            rounded >> 20,
            ((rounded & ((1 << 20) - 1)) * 100) >> 20
        )
    } else if bytes > 1 << 10 {
        let rounded = bytes + 5;
        format!(
            "{}.{:02} KiB",
            rounded >> 10,
            ((rounded & ((1 << 10) - 1)) * 100) >> 10
        )
    } else if bytes == 1 {
        "1 byte".to_string()
    } else {
        format!("{bytes} bytes")
    }
}

/// Git's default C-style path escaping leaves ordinary spaces unquoted.
/// Work from native bytes so Unix names never pass through a lossy display.
fn quote_c_style(path: impl AsRef<std::ffi::OsStr>) -> String {
    let bytes = path.as_ref().as_encoded_bytes();
    if !bytes
        .iter()
        .any(|&b| b < 0x20 || b == 0x7f || b == b'"' || b == b'\\' || b >= 0x80)
    {
        return bytes.iter().map(|&b| char::from(b)).collect();
    }
    let mut out = String::from("\"");
    for &byte in bytes {
        match byte {
            b'"' => out.push_str("\\\""),
            b'\\' => out.push_str("\\\\"),
            b'\x07' => out.push_str("\\a"),
            b'\x08' => out.push_str("\\b"),
            b'\t' => out.push_str("\\t"),
            b'\n' => out.push_str("\\n"),
            b'\x0b' => out.push_str("\\v"),
            b'\x0c' => out.push_str("\\f"),
            b'\r' => out.push_str("\\r"),
            b if b < 0x20 || b == 0x7f || b >= 0x80 => out.push_str(&format!("\\{b:03o}")),
            b => out.push(char::from(b)),
        }
    }
    out.push('"');
    out
}

fn on_disk_bytes(meta: &fs::Metadata) -> u64 {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        meta.blocks() * 512
    }
    #[cfg(not(unix))]
    {
        meta.len()
    }
}

fn read_dir_fail_closed(dir: &Path) -> CliResult<Vec<fs::DirEntry>> {
    match fs::read_dir(dir) {
        Ok(entries) => {
            let mut out = Vec::new();
            for entry in entries {
                out.push(entry.map_err(|error| {
                    CliError::io(format!(
                        "failed to read entry in '{}': {error}",
                        dir.display()
                    ))
                })?);
            }
            Ok(out)
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(Vec::new()),
        Err(error) => Err(CliError::io(format!(
            "failed to read object directory '{}': {error}",
            dir.display()
        ))),
    }
}

fn list_idx_files(pack_dir: &Path) -> Vec<PathBuf> {
    let mut idxs = Vec::new();
    if let Ok(entries) = fs::read_dir(pack_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().is_some_and(|ext| ext == "idx") {
                idxs.push(path);
            }
        }
    }
    idxs
}

fn scan_loose(
    objects_dir: &Path,
    hexsz: usize,
    pack_idxs: &[PathBuf],
    census: &mut Census,
) -> CliResult<()> {
    for shard in read_dir_fail_closed(objects_dir)? {
        let shard_path = shard.path();
        if !shard_path.is_dir() {
            continue; // root-level crash leftovers (L6)
        }
        let shard_name = shard.file_name().to_string_lossy().into_owned();
        if shard_name.len() != 2 || !shard_name.chars().all(|c| c.is_ascii_hexdigit()) {
            continue; // pack/, info/, non-hex top-level dirs (L3)
        }

        for entry in read_dir_fail_closed(&shard_path)? {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().into_owned();
            let meta = fs::symlink_metadata(&path).map_err(|error| {
                CliError::io(format!("failed to stat '{}': {error}", path.display()))
            })?;

            let valid_name = name.len() == hexsz - 2 && name.chars().all(|c| c.is_ascii_hexdigit());
            if !valid_name || !meta.is_file() {
                if census.verbose {
                    census.record_garbage(&path, "garbage found");
                }
                continue;
            }

            census.count += 1;
            census.size_bytes += on_disk_bytes(&meta);

            if census.verbose {
                let Some(hash) = parse_oid(&shard_name, &name) else {
                    continue;
                };
                if pack_idxs.iter().any(|idx| {
                    crate::utils::storage::local::LocalStorage::pack_index_contains(idx, &hash)
                        .unwrap_or(false)
                }) {
                    census.prune_packable = Some(census.prune_packable.unwrap_or(0) + 1);
                }
            }
        }
    }
    Ok(())
}

fn parse_oid(shard: &str, rest: &str) -> Option<git_internal::hash::ObjectHash> {
    crate::internal::object_format::parse_repo_oid(&format!("{shard}{rest}")).ok()
}

fn scan_pack_dir(pack_dir: &Path, local: bool, census: &mut Census) -> CliResult<()> {
    let entries = read_dir_fail_closed(pack_dir)?;
    if entries.is_empty() {
        return Ok(());
    }

    // Group files by basename (extension stripped) so a pack+idx pair is one unit.
    let mut groups: BTreeMap<String, BTreeMap<String, PathBuf>> = BTreeMap::new();
    for entry in entries {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.starts_with("multi-pack-index") {
            continue;
        }
        let Some(dot) = name.rfind('.') else {
            census.record_garbage(&path, "garbage found");
            continue;
        };
        let (base, ext) = name.split_at(dot);
        let ext = ext.trim_start_matches('.').to_string();
        if !KNOWN_PACK_EXTENSIONS.contains(&ext.as_str()) {
            census.record_garbage(&path, "garbage found");
            continue;
        }
        groups
            .entry(base.to_string())
            .or_default()
            .insert(ext, path);
    }

    for files in groups.values() {
        let has_pack = files.contains_key("pack");
        let has_idx = files.contains_key("idx");
        match (has_pack, has_idx) {
            (true, true) => {
                if local {
                    let idx = &files["idx"];
                    if let Ok(count) =
                        crate::utils::storage::local::LocalStorage::pack_index_object_count(idx)
                    {
                        census.packs = Some(census.packs.unwrap_or(0) + 1);
                        census.in_pack = Some(census.in_pack.unwrap_or(0) + count);
                        let pack_len = fs::metadata(&files["pack"]).map(|m| m.len()).unwrap_or(0);
                        let idx_len = fs::metadata(idx).map(|m| m.len()).unwrap_or(0);
                        census.size_pack_bytes =
                            Some(census.size_pack_bytes.unwrap_or(0) + pack_len + idx_len);
                    }
                }
            }
            (true, false) => {
                for path in files.values() {
                    census.record_garbage(path, "no corresponding .idx");
                }
            }
            (false, true) => {
                for path in files.values() {
                    census.record_garbage(path, "no corresponding .pack");
                }
            }
            (false, false) => {
                for path in files.values() {
                    census.record_garbage(path, "no corresponding .idx or .pack");
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{humanise_bytes, quote_c_style};

    #[test]
    fn humanise_boundaries_match_git() {
        assert_eq!(humanise_bytes(0), "0 bytes");
        assert_eq!(humanise_bytes(1), "1 byte");
        assert_eq!(humanise_bytes(1024), "1024 bytes");
        assert_eq!(humanise_bytes(1025), "1.00 KiB");
        assert_eq!(humanise_bytes(1048577), "1.00 MiB");
        assert_eq!(humanise_bytes(1073741825), "1.00 GiB");
        assert_eq!(humanise_bytes(1536), "1.50 KiB");
    }

    #[test]
    fn quote_c_style_quotes_specials_and_plain_paths() {
        assert_eq!(quote_c_style("/tmp/plain"), "/tmp/plain");
        assert_eq!(quote_c_style("/tmp/a b"), "/tmp/a b");
        assert_eq!(quote_c_style("/tmp/a\"b"), "\"/tmp/a\\\"b\"");
    }

    #[test]
    fn humanise_regressions_match_git_rounding() {
        for (bytes, expected) in [
            (1536, "1.50 KiB"),
            (2047, "2.00 KiB"),
            ((1 << 20) - 1, "1024.00 KiB"),
            (1 << 20, "1024.00 KiB"),
            ((1 << 30) - 1, "1024.00 MiB"),
            (1 << 30, "1024.00 MiB"),
            ((2 << 30) - 1, "1.99 GiB"),
        ] {
            assert_eq!(
                humanise_bytes(bytes),
                expected,
                "Git byte-size input {bytes}"
            );
        }
    }

    #[test]
    fn humanise_large_unsigned_input_is_overflow_safe() {
        assert_eq!(humanise_bytes(u64::MAX), "17179869183.99 GiB");
    }

    #[test]
    fn quote_c_style_uses_git_control_escapes() {
        assert_eq!(
            quote_c_style("/tmp/\x07\x08\t\n\x0b\x0c\r\x1b\x7f"),
            r#""/tmp/\a\b\t\n\v\f\r\033\177""#
        );
        assert_eq!(quote_c_style("/tmp/é"), r#""/tmp/\303\251""#);
        assert_eq!(quote_c_style("/tmp/a\\b"), r#""/tmp/a\\b""#);
    }

    #[cfg(unix)]
    #[test]
    fn alternate_non_utf8_path_is_lossless_in_text_and_rejected_in_json() {
        use std::{ffi::OsString, os::unix::ffi::OsStringExt, path::PathBuf};

        let path = PathBuf::from(OsString::from_vec(vec![b'/', 0xff]));
        assert_eq!(quote_c_style(&path), r#""/\377""#);
        let error = super::raw_alternate_paths(&[path]).expect_err("no lossy JSON path");
        assert!(error.to_string().contains(
            "alternate path is not valid UTF-8; use text output to inspect its escaped bytes"
        ));
    }
}
