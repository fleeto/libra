//! plan issues/490 SW-03 compat guard: staging-class index replacements must
//! route through `utils::index_ext::update_preserving_flags`, so the index v3
//! extended flags (`skip_worktree`, `intent_to_add`) survive a replacement.
//!
//! The guard scans the production code of each listed module (pruning test-only
//! items and statements surgically) for direct `index.update(` / `index.add(`
//! calls and allows only the classified exceptions below, each of which carries
//! the extended flags itself.

use std::{fs, path::PathBuf};

use syn::{Item, spanned::Spanned, visit::Visit};

const STAGING_MODULES: &[&str] = &[
    "src/command/add.rs",
    "src/command/update_index.rs",
    "src/command/restore.rs",
    "src/command/stash.rs",
    "src/command/mv.rs",
    "src/command/remove.rs",
    "src/command/commit.rs",
];

/// Classified direct replacements. Each entry is a `(module, line fragment)`
/// pair and a reason:
/// - `mv.rs` copies `skip_worktree`/`intent_to_add` from the removed source
///   entry onto the destination entry, so calling the helper (which looks up
///   the *destination*) would be wrong.
/// - `update_index.rs` `--index-info` builds a FRESH entry from the record
///   (git's `add_index_entry` semantics: the record carries its own mode/oid/
///   stage and no extended flags survive) — the helper would wrongly keep the
///   replaced entry's `skip_worktree`/`intent_to_add` bits (issues/479 UI-01).
const ALLOWED: &[(&str, &str)] = &[
    ("src/command/mv.rs", "index.add(entry);"),
    ("src/command/update_index.rs", "index.add(entry);"),
];

/// SW-04 history-rewrite modules whose whole-index rebuild must carry
/// `skip_worktree` over from the previous index.
const REBUILD_MODULES: &[&str] = &[
    "src/command/reset.rs",
    "src/command/revert.rs",
    "src/command/rebase.rs",
];

/// `read-tree` rebuilds (no `-m`) and therefore CLEARS the flags by design;
/// its `-m` merge path goes through `merge_tree_into_index`, which preserves
/// them through the update helper.
const CLEAR_ON_REBUILD: &[&str] = &["src/command/read_tree.rs"];

fn item_attrs(item: &Item) -> &[syn::Attribute] {
    match item {
        Item::Const(i) => &i.attrs,
        Item::Enum(i) => &i.attrs,
        Item::ExternCrate(i) => &i.attrs,
        Item::Fn(i) => &i.attrs,
        Item::ForeignMod(i) => &i.attrs,
        Item::Impl(i) => &i.attrs,
        Item::Macro(i) => &i.attrs,
        Item::Mod(i) => &i.attrs,
        Item::Static(i) => &i.attrs,
        Item::Struct(i) => &i.attrs,
        Item::Trait(i) => &i.attrs,
        Item::TraitAlias(i) => &i.attrs,
        Item::Type(i) => &i.attrs,
        Item::Union(i) => &i.attrs,
        Item::Use(i) => &i.attrs,
        _ => &[],
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum CfgValue {
    Off,
    On,
    Unknown,
}

fn evaluate_cfg(meta: &syn::Meta) -> CfgValue {
    match meta {
        syn::Meta::Path(path) if path.is_ident("test") => CfgValue::Off,
        syn::Meta::Path(_) | syn::Meta::NameValue(_) => CfgValue::Unknown,
        syn::Meta::List(list) => {
            let Ok(inner) = list.parse_args_with(
                syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated,
            ) else {
                return CfgValue::Unknown;
            };
            if list.path.is_ident("not") {
                return match inner.first().map(evaluate_cfg) {
                    Some(CfgValue::Off) => CfgValue::On,
                    Some(CfgValue::On) => CfgValue::Off,
                    _ => CfgValue::Unknown,
                };
            }
            if list.path.is_ident("all") {
                let mut result = CfgValue::On;
                for predicate in &inner {
                    match evaluate_cfg(predicate) {
                        CfgValue::Off => return CfgValue::Off,
                        CfgValue::Unknown => result = CfgValue::Unknown,
                        CfgValue::On => {}
                    }
                }
                return result;
            }
            if list.path.is_ident("any") {
                let mut result = CfgValue::Off;
                for predicate in &inner {
                    match evaluate_cfg(predicate) {
                        CfgValue::On => return CfgValue::On,
                        CfgValue::Unknown => result = CfgValue::Unknown,
                        CfgValue::Off => {}
                    }
                }
                return result;
            }
            CfgValue::Unknown
        }
    }
}

fn is_test_only(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attr| {
        attr.path().is_ident("cfg")
            && attr
                .parse_args::<syn::Meta>()
                .is_ok_and(|meta| evaluate_cfg(&meta) == CfgValue::Off)
    })
}

fn expr_attrs(expr: &syn::Expr) -> Option<&[syn::Attribute]> {
    Some(match expr {
        syn::Expr::Array(e) => &e.attrs,
        syn::Expr::Assign(e) => &e.attrs,
        syn::Expr::Async(e) => &e.attrs,
        syn::Expr::Await(e) => &e.attrs,
        syn::Expr::Binary(e) => &e.attrs,
        syn::Expr::Block(e) => &e.attrs,
        syn::Expr::Break(e) => &e.attrs,
        syn::Expr::Call(e) => &e.attrs,
        syn::Expr::Cast(e) => &e.attrs,
        syn::Expr::Closure(e) => &e.attrs,
        syn::Expr::Const(e) => &e.attrs,
        syn::Expr::Continue(e) => &e.attrs,
        syn::Expr::Field(e) => &e.attrs,
        syn::Expr::ForLoop(e) => &e.attrs,
        syn::Expr::Group(e) => &e.attrs,
        syn::Expr::If(e) => &e.attrs,
        syn::Expr::Index(e) => &e.attrs,
        syn::Expr::Infer(e) => &e.attrs,
        syn::Expr::Let(e) => &e.attrs,
        syn::Expr::Lit(e) => &e.attrs,
        syn::Expr::Loop(e) => &e.attrs,
        syn::Expr::Macro(e) => &e.attrs,
        syn::Expr::Match(e) => &e.attrs,
        syn::Expr::MethodCall(e) => &e.attrs,
        syn::Expr::Paren(e) => &e.attrs,
        syn::Expr::Path(e) => &e.attrs,
        syn::Expr::Range(e) => &e.attrs,
        syn::Expr::Reference(e) => &e.attrs,
        syn::Expr::Repeat(e) => &e.attrs,
        syn::Expr::Return(e) => &e.attrs,
        syn::Expr::Struct(e) => &e.attrs,
        syn::Expr::Try(e) => &e.attrs,
        syn::Expr::TryBlock(e) => &e.attrs,
        syn::Expr::Tuple(e) => &e.attrs,
        syn::Expr::Unary(e) => &e.attrs,
        syn::Expr::Unsafe(e) => &e.attrs,
        syn::Expr::While(e) => &e.attrs,
        syn::Expr::Yield(e) => &e.attrs,
        _ => return None,
    })
}

fn stmt_attrs(stmt: &syn::Stmt) -> Option<&[syn::Attribute]> {
    match stmt {
        syn::Stmt::Local(local) => Some(local.attrs.as_slice()),
        syn::Stmt::Macro(mac) => Some(mac.attrs.as_slice()),
        syn::Stmt::Expr(expr, _) => expr_attrs(expr),
        syn::Stmt::Item(item) => Some(item_attrs(item)),
    }
}

fn stmt_is_test_only(stmt: &syn::Stmt) -> bool {
    stmt_attrs(stmt).is_some_and(is_test_only)
}

struct TestRangeCollector {
    ranges: Vec<(usize, usize)>,
    in_test: bool,
}

impl<'ast> Visit<'ast> for TestRangeCollector {
    fn visit_item(&mut self, item: &'ast Item) {
        let is_test = !self.in_test && is_test_only(item_attrs(item));
        let prev = self.in_test;
        if is_test {
            self.in_test = true;
            let span = item.span();
            let start = span.byte_range().start;
            let end = span.byte_range().end;
            if end > start {
                self.ranges.push((start, end));
            }
        }
        if !self.in_test {
            syn::visit::visit_item(self, item);
        }
        self.in_test = prev;
    }

    fn visit_stmt(&mut self, stmt: &'ast syn::Stmt) {
        let is_test = !self.in_test && stmt_is_test_only(stmt);
        let prev = self.in_test;
        if is_test {
            self.in_test = true;
            let span = stmt.span();
            let start = span.byte_range().start;
            let end = span.byte_range().end;
            if end > start {
                self.ranges.push((start, end));
            }
        }
        if !self.in_test {
            syn::visit::visit_stmt(self, stmt);
        }
        self.in_test = prev;
    }
}

/// Strip test-only items and statements while preserving the exact byte length
/// and line count of the source file. Replaces non-newline bytes of test-only
/// ranges with spaces.
fn extract_production_source(label: &str, source: &str) -> String {
    let file = syn::parse_file(source)
        .unwrap_or_else(|error| panic!("`{label}` must parse as Rust: {error}"));

    let mut collector = TestRangeCollector {
        ranges: Vec::new(),
        in_test: false,
    };
    collector.visit_file(&file);

    let mut bytes = source.as_bytes().to_vec();
    for (start, end) in collector.ranges {
        let clamped_end = end.min(bytes.len());
        for byte in &mut bytes[start..clamped_end] {
            if *byte != b'\n' {
                *byte = b' ';
            }
        }
    }
    String::from_utf8(bytes).expect("valid utf-8")
}

#[test]
fn staging_commands_route_replacements_through_the_flag_helper() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut problems = Vec::new();
    let mut helper_users = 0usize;

    for module in STAGING_MODULES {
        let source = fs::read_to_string(root.join(module))
            .unwrap_or_else(|error| panic!("read {module}: {error}"));
        let production = extract_production_source(module, &source);
        if production.contains("update_preserving_flags")
            || production.contains("update_preserving_file_mode")
        {
            helper_users += 1;
        }
        for (index, line) in production.lines().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with("//") {
                continue;
            }
            if !(line.contains("index.update(") || line.contains("index.add(")) {
                continue;
            }
            if line.contains("update_preserving_flags") {
                continue;
            }
            let allowed = ALLOWED.iter().any(|(allowed_module, fragment)| {
                *allowed_module == *module && line.contains(fragment)
            });
            if !allowed {
                problems.push(format!("{module}:{}: {}", index + 1, trimmed));
            }
        }
    }

    assert!(
        problems.is_empty(),
        "unclassified direct index replacement(s) bypass update_preserving_flags:\n{}",
        problems.join("\n")
    );
    assert!(
        helper_users >= 5,
        "expected the staging modules to use the helper (found {helper_users})"
    );

    for module in REBUILD_MODULES {
        let source = fs::read_to_string(root.join(module))
            .unwrap_or_else(|error| panic!("read {module}: {error}"));
        assert!(
            source.contains("preserve_skip_worktree_from"),
            "{module} rebuilds the index and must carry skip-worktree with preserve_skip_worktree_from"
        );
    }
    for module in CLEAR_ON_REBUILD {
        let source = fs::read_to_string(root.join(module))
            .unwrap_or_else(|error| panic!("read {module}: {error}"));
        assert!(
            source.contains("merge_tree_into_index"),
            "{module} must route its -m merge path through merge_tree_into_index"
        );
    }
}

#[test]
fn scan_preserves_production_code_after_inline_cfg_test_item() {
    let fixture = r#"
// Initial production code
pub fn early_prod() {}

#[cfg(test)]
fn test_race_hook() {
    // A direct index call in test code must NOT trigger an unclassified bypass problem:
    index.add(entry);
}

// Subsequent production code containing helper call
pub fn late_prod() {
    update_preserving_flags();
}

pub fn unclassified_prod() {
    index.add(entry);
}
"#;

    let production = extract_production_source("fixture.rs", fixture);

    // 1. It must NOT truncate at test_race_hook: late_prod and its helper call must survive.
    assert!(
        production.contains("update_preserving_flags"),
        "production code after inline cfg(test) item must be preserved"
    );

    // 2. Direct index operations inside #[cfg(test)] must be stripped.
    assert!(
        !production.contains("test_race_hook"),
        "test item should be stripped"
    );

    // 3. Scan for problems: only line 17 (unclassified_prod index.add(entry);) should be flagged,
    // not line 8 (inside test_race_hook).
    let mut problems = Vec::new();
    for (index, line) in production.lines().enumerate() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("//") {
            continue;
        }
        if !(line.contains("index.update(") || line.contains("index.add(")) {
            continue;
        }
        if line.contains("update_preserving_flags") {
            continue;
        }
        problems.push(format!("fixture.rs:{}: {}", index + 1, trimmed));
    }

    assert_eq!(
        problems,
        vec!["fixture.rs:17: index.add(entry);"],
        "only the production callsite on line 17 should be flagged, not the test callsite on line 8"
    );
}
