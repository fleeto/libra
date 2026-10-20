# `libra repack`

Combine repository objects into a single pack.

## Synopsis

```
libra repack [-a|--all] [-d|--delete] [-l|--local] [-q|--quiet]
             [--window <N>] [--depth <N>]
```

## Description

`repack` encodes repository objects into one new `pack-<checksum>.pack` (with a
matching `pack-<checksum>.idx`) under `.libra/objects/pack/`. It uses the same
shared pack writer as the `maintenance` tasks, so every pack Libra writes on disk
goes through one well-formed encoder — the produced pack round-trips through
`libra index-pack` and `libra verify-pack`.

By default only the reachable objects that are currently **loose** are packed;
objects already stored in an existing pack are left where they are. `--all`
widens the set to every reachable object, producing a single consolidated pack.

Reachability is computed from refs, reflogs and the index — exactly like
`libra maintenance run --task gc` — so an object referenced only by a reflog is
never dropped.

When `--window`/`--depth` are not given, the values come from `pack.window` /
`pack.depth` configuration, defaulting to 10 / 50 (Git's defaults). The window
is forwarded to the shared pack encoder as its sliding delta-search window:
`0` disables delta compression, larger values spend more time looking for
smaller deltas. `--depth` is accepted and validated for Git CLI parity; the
upstream encoder applies a fixed chain cap of 50 (recorded upstream
dependency, ER-MX-03), so any accepted depth value yields a valid pack.

## Options

| Option | Description |
|--------|-------------|
| `-a`, `--all` | Pack all reachable objects, including those already stored in a pack, into a single fresh pack. |
| `-d`, `--delete` | After packing, remove the loose objects that now live in the new pack. Only files whose object id is in the new pack are removed. Combined with `-a`, old packs whose objects are ALL covered by the new pack are deleted too — never before the new pack and its index are written and made durable, and only after a final per-pack coverage re-check (any uncovered object keeps that pack). Deletion refuses while another repository borrows this object store (`LBR-CONFLICT-002`). |
| `-l`, `--local` | Skip objects that exist only in an alternate object store (`pack-objects --local` semantics); alternate-only objects stay where they are. |
| `-q`, `--quiet` | Suppress the informational summary. |
| `--window <N>` | Delta-compression window size (default: `pack.window`, then 10; `0` disables deltas). |
| `--depth <N>` | Maximum delta-chain depth (default: `pack.depth`, then 50). |

With `--json` / `--machine` the command emits an envelope whose `data` object
carries `pack` (the new pack's name), `objects_packed`, `loose_removed`,
`packs_removed`, and the resolved `window` / `depth`.

## Exit status

- `0` — the repack completed (including the no-op case where nothing needed
  packing).
- non-zero — the command was run outside a repository, or a pack could not be
  written.

## Compatibility

This is a focused subset of Git's `git repack`. Libra writes one new pack per
run via its shared writer, with `--window`/`--depth` delta tuning (and
redundant-*pack* removal for `-a -d`); geometric repacking (`--geometric`),
bitmap generation, and `.keep` handling are not implemented. See
[`COMPATIBILITY.md`](../../COMPATIBILITY.md).

## Examples

```
# Pack the loose reachable objects into a single pack.
libra repack

# Consolidate every reachable object into one pack.
libra repack -a

# Repack everything, then drop the loose copies and the fully-covered old packs.
libra repack -a -d

# Same, but leave objects that live only in an alternate untouched.
libra repack -a -d -l

# Consolidate without delta compression (smaller CPU cost, larger pack).
libra repack -a -d --window=0

# Spend more time on delta search for a smaller pack.
libra repack -a -d --window=50 --depth=100

# Machine-readable summary.
libra --json repack -a -d
```

## See also

- [`libra maintenance`](maintenance.md) — scheduled optimization tasks (its
  `loose-objects` and `incremental-repack` tasks share this writer).
- [`libra index-pack`](index-pack.md) — build the index for an existing pack.
- [`libra verify-pack`](verify-pack.md) — validate a pack and its index.
