# `libra diff-files`

Show the differences between the index and the working tree — a plumbing entry
point that reuses the one `diff` engine (see [`diff`](diff.md)).

## Synopsis

```
libra diff-files [-p] [-s] [--raw] [--name-only] [--name-status] [-- <path>...]
```

## Description

`diff-files` is equivalent to `libra diff --no-renames`: it shows the unstaged
changes (index vs working tree). Path limiters go after `--`; as Git plumbing it
ignores porcelain `diff.renames`. Global output flags such as
`libra --json diff-files` still apply.

Like Git's plumbing, the **default output is raw**. `--name-status` prints Git's
`A`/`M`/`D`/`R` letters (for example `A\tnew.txt` for an intent-to-add entry).

## Options

| Option | Description | Example |
|--------|-------------|---------|
| `-p`, `--patch` | Emit a patch instead of raw records. | `libra diff-files -p` |
| `--raw` | Force raw records (the default). | `libra diff-files --raw` |
| `--name-only` | Print changed path names only. | `libra diff-files --name-only` |
| `--name-status` | Print the status letter for each changed path. | `libra diff-files --name-status` |
| `-s`, `--no-patch` | Suppress the diff body entirely. | `libra diff-files -s` |
| `-- <path>...` | Limit the diff to paths. | `libra diff-files -- src/` |
| `--json` / `--machine` | Structured diff output. | `libra --json diff-files` |

The remaining porcelain flag surface (`-1`/`-2`/`-3` stage selection) is not exposed.

## Exit codes

| Code | Meaning |
|------|---------|
| `0` | No differences. |
| `1` | There are differences — Git plumbing exit convention. |
| `128` | Not inside a repository. |

## Examples

```bash
# Raw unstaged records
libra diff-files

# Patch form
libra diff-files -p

# Names only
libra diff-files --name-only
```

## Comparison with Git

| Task | Libra | Git |
|------|-------|-----|
| Unstaged changes (raw) | `libra diff-files` | `git diff-files` |
| Patch form | `libra diff-files -p` | `git diff-files -p` |

Deferred: `-1`/`-2`/`-3` stage selection. Use `libra diff` for richer options.
