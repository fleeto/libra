//! compat guard: every clap `dry_run` field under `src/command/**` must be
//! registered as either an explicit preview (classified `ReadOnly` by
//! `operation_class_for_command`) or an accepted exception.
//!
//! Issue #483 / ADR-CO-05: a new `--dry-run` flag must not silently start
//! publishing Operation Log snapshot objects. Adding an unregistered field
//! fails here with an actionable hint.

use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

/// Preview fields that ARE classified `ReadOnly` in `src/cli.rs`. The pair is
/// (command file stem, cli.rs arm marker).
const CLASSIFIED: &[(&str, &str)] = &[
    ("add", "command::add::AddArgs { dry_run: true"),
    ("remove", "command::remove::RemoveArgs { dry_run: true"),
    ("clean", "command::clean::CleanArgs { dry_run: true"),
    ("mv", "command::mv::MvArgs { dry_run: true"),
    ("reflog", "command::reflog::reflog_is_read_only_query"),
    (
        "alternates",
        "command::alternates::AlternatesCommand::Prune { dry_run: true",
    ),
    (
        "cache",
        "command::cache::CacheCommand::Evict { dry_run: true",
    ),
    // Pre-existing precedents (own preview classification, not part of #483).
    ("commit", "args.dry_run || args.porcelain"),
    ("merge", "Commands::Merge(args) if args.dry_run"),
];

/// Commands with a `dry_run` field that need no classification: they either
/// have their own durable boundary, are external/unknown writes, or the
/// preview path already writes nothing.
const ACCEPTED: &[&str] = &[
    "agent",
    "fast_import",
    "fetch",
    "file",
    "hydrate",
    "maintenance",
    "notes",
    "op",
    "push",
    "remote",
    "worktree",
    "worktree-fuse",
];

fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).expect("read src/command") {
        let path = entry.expect("dir entry").path();
        if path.is_dir() {
            walk(&path, out);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            out.push(path);
        }
    }
}

fn command_key(root: &Path, path: &Path) -> String {
    let command_dir = root.join("src/command");
    let rel = path.strip_prefix(&command_dir).expect("under src/command");
    let first = rel
        .components()
        .next()
        .expect("component")
        .as_os_str()
        .to_string_lossy()
        .into_owned();
    first.trim_end_matches(".rs").to_string()
}

#[test]
fn every_dry_run_field_is_registered() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files = Vec::new();
    let command_dir = root.join("src/command");
    walk(&command_dir, &mut files);

    let cli = fs::read_to_string(root.join("src/cli.rs")).expect("read cli.rs");

    let mut offenders = Vec::new();
    let mut seen: BTreeSet<String> = BTreeSet::new();
    for file in files {
        let text = fs::read_to_string(&file).expect("read command file");
        if !text.contains("dry_run: bool") && !text.contains("pub dry_run: bool") {
            continue;
        }
        let key = command_key(root, &file);
        seen.insert(key.clone());

        if let Some((_, marker)) = CLASSIFIED.iter().find(|(name, _)| *name == key) {
            assert!(
                cli.contains(marker),
                "classification arm missing for `{key}`: expected cli.rs to contain {marker:?}"
            );
        } else if !ACCEPTED.contains(&key.as_str()) {
            offenders.push(format!(
                "{}: command `{key}` has a `dry_run` field but is neither classified ReadOnly \
                 nor in ACCEPTED",
                file.display()
            ));
        }
    }

    assert!(
        offenders.is_empty(),
        "unregistered dry-run fields (add an ADR-CO-05 classification or an ACCEPTED reason):\n{}",
        offenders.join("\n")
    );

    // The registry itself must not rot: every CLASSIFIED name must still have
    // a field somewhere.
    for (name, _) in CLASSIFIED {
        assert!(
            seen.contains(*name),
            "CLASSIFIED entry `{name}` no longer has a `dry_run` field"
        );
    }
}
