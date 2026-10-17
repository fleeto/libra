# libra count-objects

Count unpacked objects and their disk consumption, mirroring `git count-objects`.

## Usage

```
libra count-objects [-H | --human-readable] [-v | --verbose]
```

With no flags the command prints `"<count> objects, <size> kilobytes"` for the
loose object store. `-H` renders sizes in human-readable units (bytes / KiB /
MiB / GiB with Git's exact thresholds). `-v` prints the verbose Git field set:

```
count: <loose object count>
size: <loose KiB>
in-pack: <packed object count>
packs: <pack file count>
size-pack: <pack KiB>
prune-packable: <loose objects that also exist in a pack>
garbage: <garbage entry count>
size-garbage: <garbage KiB>
alternate: <alternate object directory>      (0..n lines)
```

`--json` / `--machine` emit a structured census (raw byte and KiB fields, plus
`garbage_entries` with `path`/`reason` on `-v`, capped at 100 with a
`garbage_entries_truncated` flag). Structured mode suppresses the human warning
lines and puts garbage detail into `data.garbage_entries`.

## Examples

```
libra count-objects                   Count loose objects and total size
libra count-objects -H                Human-readable sizes
libra count-objects -v                Verbose Git field set
libra count-objects -v --json         Structured census for agents
```

## Intentional differences from Git

- Garbage and `alternate:` paths are printed as absolute paths (Libra resolves
  from any subdirectory and the storage directory name differs).
- An unreadable shard or pack directory fails closed with `LBR-IO-001` instead
  of Git's warn-and-continue (which under-counts and can make a before/after
  comparison pass falsely).
- Operation Log snapshot objects written by real mutations are counted, so
  Libra's count is usually larger than Git's after a mutation.
- Only the local storage tier is inspected; tiered remote objects are not
  counted and no network request is made.

## Guarantees

- Read-only: no `.idx` rebuild, no schema migration, no maintenance lock, no
  operation record. `count-objects -v` output is byte-stable across runs.
- Works in bare repositories, subdirectories, and linked worktrees; exits 0.

## Errors

- `LBR-REPO-001` (128) outside a repository.
- `LBR-CLI-002` (129) for unknown flags or positional arguments.
- `LBR-IO-001` (128) when the object store cannot be read.
