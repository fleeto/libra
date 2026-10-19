# `libra diff-index`

Show the differences between a tree and the working tree (or the index with
`--cached`) — a plumbing entry point that reuses the one `diff` engine (see
[`diff`](diff.md)).

## Synopsis

```
libra diff-index [--cached] [-p] [-s] [--raw] [--name-only] [--name-status]
                 <tree> [-- <path>...]
```

## Description

`diff-index <tree>` is equivalent to `libra diff --old <tree> --no-renames`: it
diffs the given tree-ish against the current working tree. With `--cached` it
compares the tree against the index instead. Path limiters go after `--`; as Git
plumbing it ignores porcelain `diff.renames`. Global output flags such as
`libra --json diff-index ...` still apply.

Like Git's plumbing, the **default output is raw**.

## Options

| Option | Description | Example |
|--------|-------------|---------|
| `<tree>` | The tree-ish to compare. | `libra diff-index HEAD` |
| `--cached` | Compare against the index instead of the working tree. | `libra diff-index --cached HEAD` |
| `-p`, `--patch` | Emit a patch instead of raw records. | `libra diff-index -p HEAD` |
| `--raw` | Force raw records (the default). | `libra diff-index --raw HEAD` |
| `--name-only` | Print changed path names only. | `libra diff-index --name-only HEAD` |
| `--name-status` | Print the status letter for each changed path. | `libra diff-index --name-status HEAD` |
| `-s`, `--no-patch` | Suppress the diff body entirely. | `libra diff-index -s HEAD` |
| `-- <path>...` | Limit the diff to paths. | `libra diff-index HEAD -- src/` |
| `--json` / `--machine` | Structured diff output. | `libra --json diff-index HEAD` |

The remaining porcelain flag surface (`-m`) is not exposed.

## Exit codes

| Code | Meaning |
|------|---------|
| `0` | No differences. |
| `1` | There are differences — Git plumbing exit convention. |
| `128` | The tree-ish could not be resolved, or not inside a repository. |

## Examples

```bash
# Raw tree-vs-worktree records
libra diff-index HEAD

# Tree vs index
libra diff-index --cached HEAD

# Patch form
libra diff-index -p HEAD
```

## Comparison with Git

| Task | Libra | Git |
|------|-------|-----|
| Tree vs working tree | `libra diff-index <tree>` | `git diff-index <tree>` |
| Tree vs index | `libra diff-index --cached <tree>` | `git diff-index --cached <tree>` |
| raw output format | `libra diff-index --raw <tree>` | `git diff-index --raw <tree>` |

Deferred: `-m`. Use `libra diff` for richer options.
