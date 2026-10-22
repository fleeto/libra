# `libra worktree`

Manage multiple working trees attached to this repository.

**Alias:** `wt`

## Synopsis

```
libra worktree add <NAME> [TARGET] [--path <PATH>] [--force]
libra worktree list [--porcelain]
libra worktree lock <NAME> [--reason <text>]
libra worktree lock --path <PATH> [--reason <text>]
libra worktree unlock (<NAME> | --path <PATH>)
libra worktree move <NAME> --path <DEST>
libra worktree move --from-path <SRC> --path <DEST>
libra worktree prune
libra worktree remove (<NAME> | --path <PATH>) [--delete-dir] [--force]
libra worktree umount <path> [--cleanup]
libra worktree repair [<NAME> | --path <PATH>] --confirm
libra worktree repair [<NAME> | --path <PATH>] --migrate-layout [--dry-run] [--confirm]
libra worktree repair (<NAME> | --path <PATH>) --resolve-identity --yes
libra worktree doctor [<workspace-id>] [--limit <n>] [--cursor <cursor>]
```

## Description

`libra worktree` manages multiple working trees that share a single repository database and object store. This allows you to have several checkouts of the same repository simultaneously, which is useful for working on multiple branches at once, running builds while editing code, or testing changes in isolation.

New worktrees have a stable **name**. By default, ordinary linked worktrees live at `<common-storage>/worktree/linked/<NAME>`: normally `.libra/worktree/linked/<NAME>` in the main repository. The common storage is shared by all linked worktrees, so invoking `add` from a linked worktree or a subdirectory uses the same default root. The main worktree stays in its original directory. `--path <PATH>` selects an explicit complete destination instead; relative paths are resolved from the caller's current directory.

Names are 1–64 lowercase ASCII characters matching `[a-z0-9][a-z0-9._-]{0,63}`. A name must not end in a dot or use a reserved Windows device name, including device names with an extension. Paths may contain spaces and Unicode. Names share one repository namespace across ordinary and FUSE worktrees, including detached entries and entries awaiting cleanup. An existing name always resolves to its stored path, even after a move; supplying a different `--path` is refused.

Each linked worktree contains its own real `.libra` gitdir — a local directory (not a symlink) holding its own `HEAD`, index and `HEAD` reflog, plus a `commondir` pointer to shared storage and a stable `worktree_id`. Worktrees share the database, object store, branch/tag/remote refs and configuration while retaining separate checkout and staging state. Main-worktree file operations exclude its `.libra` metadata, while commands inside a managed linked worktree operate on that worktree's files. A missing or invalid local identity inside the managed container is refused rather than resolving upward into the main worktree.

The on-disk `worktrees.json` registry uses `schema_version: 4`, with optional `name` fields. Existing v2/v3 registries remain readable; a normal mutation writes the current format while preserving stored identities and v3 registration generations (`epoch_counter` and each entry's `epoch`). Existing unnamed entries and the main entry have no assigned name. This change does not move existing directories, assign names in bulk, or add a directory-migration command. Use `--path` to select an existing unnamed worktree; `worktree add <NAME> --path <registered-path>` assigns a name to an active unnamed entry, or assigns one and reattaches a detached unnamed entry. Registry files are written atomically. The public `worktree list` JSON schema is version 2, independently of the disk registry version.

Older shared-`.libra` symlink layouts retain their existing diagnosis and explicit `repair --migrate-layout` workflow; that operation repairs gitdir isolation and does not relocate a worktree into the new default directory. Registry schema 4 makes older v2/v3 registry readers refuse the file instead of silently dropping names. The capability marker separately makes Libra 0.30.66 and older refuse the whole repository; it does not prove that every intermediate build understands schema 4. Once v0.30.85 mutates the worktree registry, it writes schema 4 even when no entries have names. From that point, use v0.30.85 or a subsequent release containing its registry and identity-safety fixes for worktree commands. The earlier v0.31.0 tag understands schema 4 but predates those fixes; do not use it to mutate these worktrees.

**Resolving an ambiguous registry.** An older registry may contain two entries claiming one path-derived identity. Mutating worktree operations refuse this ambiguity. `libra worktree doctor` identifies the entries; `libra worktree repair --path <PATH> --resolve-identity --yes` detaches the selected entry while preserving its files and scoped state, so the remaining active entry can be used. The detached entry cannot be reattached while the survivor is active, and `remove --delete-dir` refuses while any other registered entry shares the identity: deleting it would erase the other's scoped state. Back up both directories, continue using the survivor, and keep the detached entry for manual recovery. Detaching the survivor does not make deletion safe. No automatic identity migration is provided.

A newly created worktree is populated from its selected commit (HEAD by default), not from staged index changes. Reattaching a detached worktree does not perform checkout or replace its files, HEAD or index.

## Options

### Subcommand: `add`

Create a named linked worktree in the managed directory, or at an explicit destination.

| Argument / Flag | Description |
|-----------------|-------------|
| `<NAME>` | Stable worktree name. By default creates `.libra/worktree/linked/<NAME>` under common storage. |
| `[TARGET]` | Optional branch or commit-ish. An existing branch is checked out attached, unless any worktree already has it checked out. Other commit-ish targets create a detached worktree. Unknown targets fail before creation; remote-branch guessing and `--track`/`--no-track` are not supported. |
| `--path <PATH>` | Complete external destination, absolute or relative to the caller. It must be absent or an empty directory unless `--force` is used in a standard build; it cannot be a symlink and may enter common storage only at this name's own managed default. It is not a parent directory to which the name is appended. |
| `--detach` | Detach HEAD even when `TARGET` names a branch. |
| `-b, --new-branch <NEW_BRANCH>` | Create and check out a new branch at `TARGET`, or the source HEAD when omitted. An existing branch is refused; a later creation failure rolls the new branch back. |
| `-f, --force` | In standard builds, reuse a non-empty destination directory. Check its contents before using this option. In `worktree-fuse` builds, `--force` is unavailable and `-f` means `--fuse`. |

`--create-branch` is an alias for `--new-branch`. Without a target the worktree is **detached at the source commit**. The name does not implicitly create a branch. `--lock`, `--orphan`, `--no-checkout`, remote guessing and `-B` are not supported; use `worktree lock` after creation.

Standard builds have supported `add --force` and `remove --force` since v0.30.69, including v0.30.85. The older published `v0.31.0` Linux/macOS binaries predate these flags. If you installed v0.31.0, use the [signed stable installer](upgrade.md) once the stable channel contains v0.30.85 or a subsequent release with its fixes; `libra upgrade` will not select a lower version number.

For a named detached registry entry, `add <NAME>` verifies the saved identity and common-storage pointer, then reattaches its existing files and staging state. For an unnamed detached entry, use `add <NAME> --path <registered-path>` to name and reattach it. `TARGET`, `-b` and `--detach` are refused during reattachment. If supplied for a named entry, `--path` must match the registered path. A tombstoned entry remains reserved until cleanup succeeds.

```bash
# Detached at the source commit in .libra/worktree/linked/my-feature
libra worktree add my-feature
libra --json worktree add my-feature

# Check out an existing branch
libra worktree add fix-1 hotfix

# Detached at a tag or commit
libra worktree add --detach probe v1.2.0

# Create a branch explicitly
libra worktree add -b topic topic main

# Use this exact external destination, including spaces or Unicode
libra worktree add experiment --path "../实验 workspace"
```

#### FUSE worktrees

On Unix builds with `worktree-fuse`, `add <NAME> --fuse` requires a working FUSE driver (`/dev/fuse` on Linux or macFUSE on macOS). Official release binaries are built without `worktree-fuse`; use a custom build to enable it. On these builds, `-f` is short for `--fuse`, not `--force`. Custom macOS `worktree-fuse` builds from v0.30.85 source include the managed-directory protection described here; the earlier `v0.31.0` tag predates this fix. It uses `.libra/worktree/fuse/<NAME>/workspace` as its mountpoint and the sibling `upper/` as its writable overlay. An explicit `--path` selects an external mountpoint; targets inside a lower tree, upper tree, common metadata or another managed workspace are refused before creation. The internal default is protected by an opaque `.libra/worktree` directory in the overlay, so it cannot recursively expose its own mountpoint, upper directory or sibling workspaces. Mount setup refuses a filesystem that cannot establish this mask.

FUSE and ordinary worktrees cannot claim the same name. The FUSE registry uses `{ "schema_version": 1, "entries": [...] }`; known older records remain readable without relocating their directories. FUSE worktrees do not support `move` or scoped identity repair (`repair <NAME>` / `repair --path <PATH>`). Use `worktree repair --confirm` for global registry repair; FUSE does not have the ordinary linked-worktree private identity layout. Their `remove` behavior remains unmounting and deleting the upper layer, rather than ordinary keep-dir detachment. For the managed default, it also removes the empty mountpoint and container; a nonempty directory or cleanup failure keeps the registration and name for retry. Existing legacy and external mountpoint directories are kept. `umount` continues to take an actual path rather than a worktree name.

The `repair --resolve-identity --yes` form is available only in standard builds. A `worktree-fuse` build does not expose these flags, even for ordinary linked worktrees.

```bash
libra worktree add overlay --fuse
libra worktree umount /repo/.libra/worktree/fuse/overlay/workspace
```

### Subcommand: `list`

List all registered worktrees and their state. `--porcelain` emits a stable,
machine-readable format: for each worktree a `worktree <path>` line, that
worktree's own `HEAD <sha>` line and either a `branch <ref>` or a `detached`
line (each worktree owns its HEAD), an optional `name <name>` line after the
`worktree` line, and a `locked [<reason>]` line when locked,
with a blank line between worktrees. A worktree whose HEAD cannot be resolved
(a legacy shared-`.libra` layout, or a missing/corrupt scope) omits the HEAD
lines rather than being mislabeled with another worktree's commit.

```bash
libra worktree list
libra worktree list --porcelain
libra --json worktree list
libra --machine worktree list
```

Structured output uses the `worktree.list` command envelope with
`data.schema_version: 2`. Each entry reports optional `name`, its actual absolute
`path`, `kind`, `is_main`, `locked`, `lock_reason`, whether the path currently
exists on disk, the persisted `worktree_id`, registration `epoch`, and the lifecycle `state`
(`active`, `detached_from_registry`, or `tombstone` — see `remove`), and the on-disk `layout` (`main`, `linked-v2`, `legacy-symlink`, `missing`, `corrupt`; porcelain adds a matching `layout` line per entry). In a `legacy-symlink` worktree (pre-isolation shared `.libra`), read-only commands keep working but state-mutating commands refuse with `LBR-REPO-003` — run `libra worktree repair --migrate-layout --confirm --path <PATH>` from the main worktree. Target-oriented lifecycle commands (`worktree remove --path <PATH>` in both modes and `worktree repair --path <PATH>`) also refuse a legacy-symlink target — the shared symlink would route their writes into MAIN storage — until the migration completes.

Unnamed entries, including the main worktree, omit `name` from JSON and omit the porcelain `name` line. Named human-readable entries use `worktree <name> <actual-path>`.

### Subcommand: `repair`

Repair stale registry rows, restore a worktree's gitdir identity, or migrate a
legacy layout. **Every mutating repair action requires `--confirm`**
(plan-20260714 Part C W0 §C.11, Codex R16/R17): without it the command is
refused with `LBR-CONFLICT-002` before any registry lock, database write or
filesystem touch (zero side effects); with it the action runs inside one
operation-log audit boundary, recording exactly one row per executed action
(inspect with `libra op log`). The read-only `--migrate-layout --dry-run`
preview never needs confirmation, and `repair --path <PATH> --resolve-identity` keeps
its own dedicated `--yes` confirmation.

Without a name or `--path`, the confirmed action removes duplicate registry entries (same canonical path), ensures exactly one main worktree entry exists, and runs the W3 lifecycle recovery engine: stale intent-journal rows (from an interrupted add/move/remove/prune) are rolled forward or back when their identity can be verified (recovery never deletes directories), tombstone entries get their scoped cleanup retried, and detached markers plus the SQL lifecycle mirror are reconciled with the registry. If another registered entry shares an identity, scoped cleanup is refused and the journal or tombstone remains for manual recovery. The state file is only rewritten when something actually changed.

With `--migrate-layout`, migrates legacy shared-`.libra` symlink worktrees to the isolated layout (run from the MAIN worktree; `--dry-run` reports without writing; without a path every legacy entry is migrated). The migration installs a fresh journaled gitdir by atomic renames (the legacy link is kept as a backup until verification passes), seeds a DETACHED HEAD at the shared snapshot, and rebuilds the private index from that commit: working files are never touched (they show as dirty/untracked afterwards) and shared STAGED state is never copied — commit or stash it in the main worktree first. An unmerged shared index or an in-progress main rebase/cherry-pick/bisect refuses before any rename; an interrupted migration is recovered by the next `worktree repair --confirm`.

With a name or `--path`, restores that **linked** worktree's gitdir identity from the registry (registry v2): rewrites a missing or corrupt `.libra/worktree_id` from the entry's persisted stable id and restores a missing or corrupt (empty/unreadable) `commondir` pointer to this repository's shared storage. The identity always comes from the registry — never from a guess — so the repaired worktree maps back to its own scoped state (HEAD, index, stash snapshots) instead of a fresh scope or the main worktree's. A `commondir` that validly points at a **different** storage is refused (repair never silently re-homes a worktree onto another repository), and the refusal is side-effect free — neither gitdir file is touched. Unregistered paths and the main worktree are refused, and so is a registry still in the legacy v1 format (it carries no persisted identities) — run the no-argument `libra worktree repair --confirm` once to upgrade it, then retry.

```bash
libra worktree repair --confirm
libra --json worktree repair --confirm
libra worktree repair experiment --confirm
libra --json worktree repair --path ../experiment --confirm
libra worktree repair --migrate-layout --dry-run   # preview, no --confirm needed
libra worktree repair --migrate-layout --confirm
```

### Subcommand: `doctor`

Report Agent workspace-scope diagnostics. The ordinary forms are **strictly
read-only**: the registry, database, lease state, and filesystem are
byte-identical before and after the call. A diagnostic must be safe to run on a
repository you do not yet understand, so a bare `doctor` never adopts,
reclaims, or repairs anything. (Libra-only; Git has no equivalent.)

A *workspace* is the association record an Agent runtime takes over a worktree (see `libra agent workspace list`). Human use of a linked worktree never needs one, so a repository with no Agent activity reports nothing.

| Argument / Flag | Description |
|-----------------|-------------|
| `<workspace-id>` | Diagnose exactly one workspace instead of paging over all of them. Cannot be combined with `--limit`/`--cursor` (a single scope is not a page) — the combination is a usage error, `LBR-CLI-002`. |
| `--limit <n>` | Maximum diagnostics per page. Default 50, capped at 500. |
| `--cursor <cursor>` | Resume after a previous page: pass the `next_cursor` value back verbatim. The cursor is opaque; one this command did not issue is refused with `LBR-WORKTREE-001` rather than silently restarting at page one. |
| `--adopt-capture-session <session-id>` | Explicitly attributes `legacy_unknown` capture rows to the named workspace. Requires `<workspace-id>` and `--confirm`; it cannot be combined with `--limit`/`--cursor`. All other doctor forms remain read-only. |
| `--adopt-info-to <worktree-path>` | Copies main's legacy common `info/exclude` + `info/attributes` into that ONE linked worktree's own gitdir (existing destination files are kept, never overwritten). Requires `--confirm`. |
| `--clear-common-info` | Deletes the legacy `info/exclude` + `info/attributes` from common storage (the rules stop applying anywhere). Requires `--confirm`. |
| `--adopt-approved-project <legacy-project-id>` | Re-homes Always approvals whose opaque `project_id` is not the current `libra.repoid` onto the canonical repository identity (plan-20260715 W4-07). Migrations never rewrite those rows. Requires `--confirm`. The read-only doctor lists discoverable legacy IDs. |
| `--clear-approved-project <legacy-project-id>` | Deletes Always approvals under a legacy (non-canonical) `project_id` without adopting them. Refuses the canonical `libra.repoid`. Requires `--confirm`. |
| `--confirm` | Confirms the irreversible doctor mutations — capture-scope adoption, info-file adopt/clear, and approved_permission adopt/clear. Every other doctor form is read-only and needs no confirmation. |

```bash
libra worktree doctor
libra --json worktree doctor
libra --json worktree doctor --limit 20
libra --json worktree doctor --cursor "$cursor"
libra worktree doctor <workspace-id>
```

Without an id, the machine response pages `data.diagnostics[]` by opaque
`next_cursor` (`workspace_id` ascending; default limit 50, maximum 500). With
`<workspace-id>`, it returns the singular `data.diagnostic`; combining that id
with `--limit` or `--cursor` is `LBR-CLI-002`. An invalid cursor fails closed
with `LBR-WORKTREE-001`; an unreadable scope fails closed with
`LBR-WORKTREE-002` rather than returning a partial page.

Each diagnostic reports the workspace's identity (`workspace_id`, `repo_id`, `path`, `worktree_id`), its `lease_state` (`none`, `held`, or `expired`), and a `scope_diagnostics` array of findings. Findings carry a stable `code` and a `severity` of `warning` or `error`:

| Code | Severity | Meaning |
|------|----------|---------|
| `foreign_repository_identity` | error | The record was written under a previous repository identity: it is invisible to the normal listings and blocks new workspace registrations. |
| `registry_path_mismatch` | error | The worktree registry and the workspace record disagree about where the scope lives. |
| `scope_layout_corrupt` | error | The gitdir layout at that path is unrecognizable. |
| `workspace_orphaned` | warning | Teardown failed or the owner vanished; the workspace still holds recovery state. |
| `lease_expired` | warning | The lease deadline passed. The lease still belongs to its owner until it is explicitly reclaimed. |
| `workspace_path_missing` | warning | No directory exists at the claimed path. |
| `registry_entry_missing` | warning | No worktree registry entry owns this scope. |
| `registry_entry_detached` | warning | The worktree was unregistered with `worktree remove` (keep-dir). |
| `registry_entry_tombstoned` | warning | The directory was deleted but its scoped rows are still pending cleanup; `libra worktree repair --confirm` retries it. |
| `scope_layout_legacy_symlink` | warning | The worktree still uses the pre-isolation shared-`.libra` symlink layout; migrate it with `libra worktree repair --migrate-layout --confirm`. |

Migration `2026080401` classifies capture rows written before workspace scope
existed as `legacy_unknown`. They remain unavailable to hooks and historical
imports rather than being guessed into main. Their only attribution path is a
separate, confirmed mutation:

```bash
libra worktree doctor <workspace-id> \
  --adopt-capture-session <session-id> --confirm
```

The target workspace must have a current live lease fence. The command assigns
all legacy capture rows for that provider session (session, export job, and
import identity) to that exact repository/worktree/workspace fence, refuses if
any scoped row already exists for the provider session, and appends one
immutable audit record. It cannot be undone; inspect the target workspace and
session before confirming. Supply the catalog session id; if an export/import
row outlived its catalog session, its provider-session id is accepted instead
so the orphan can be recovered. The adoption response uses the distinct
`worktree.doctor.adopt_capture` JSON command envelope, leaving the read-only
`worktree.doctor` page schema unchanged.

**Legacy common info files** (W0 §C.4.1.1): since `info/exclude` and
`info/attributes` became worktree-local, the files in COMMON storage (main's
`.libra/info/*`) apply only to the main worktree. When linked worktrees
exist, the read-only report says so on the main entry, and two explicit,
confirmed actions handle the legacy files — never an automatic copy:

```bash
# Copy main's info/exclude + info/attributes into ONE linked worktree's
# own gitdir (existing destination files are kept, never overwritten):
libra worktree doctor --adopt-info-to <worktree-path> --confirm

# Delete them from common storage (the rules stop applying anywhere):
libra worktree doctor --clear-common-info --confirm
```

Both refuse without `--confirm` before any side effect and run inside the
same one-row operation-log audit boundary as the other mutating repair
actions. In `--json`/`--machine` mode each uses its own envelope —
`worktree.doctor.adopt_info` (`data.target` + `data.report`) and
`worktree.doctor.clear_common_info` (`data.report`) — leaving the read-only
`worktree.doctor` page schema untouched. The read-only report also lists
each worktree's live local info sources (e.g. `.libra/info/exclude`, plus
`.git/info/exclude` in a dual-layout tree) on its text entry.

**Legacy Always approvals** (plan-20260715 W4-07): `approved_permission`
rows whose opaque `project_id` is not the current `libra.repoid` stay
invisible to the runtime. Migrations backfill empty provenance and never
rewrite `project_id`. The read-only doctor lists discoverable legacy IDs;
recovery is explicit and confirmed:

```bash
# Re-home every Always approval under a legacy project_id onto libra.repoid:
libra worktree doctor --adopt-approved-project <legacy-project-id> --confirm

# Or delete that legacy bucket without adopting:
libra worktree doctor --clear-approved-project <legacy-project-id> --confirm
```

Both refuse without `--confirm` before any side effect and run inside the
same one-row operation-log audit boundary. `--json` envelopes are
`worktree.doctor.adopt_approved_project` and
`worktree.doctor.clear_approved_project` (`data.action`,
`data.legacy_project_id`, `data.rows_affected`).

### Subcommand: `lock`

Lock a worktree to prevent prune, move or removal. Select it with either a name or `--path`, never both. `--reason <text>` records an optional explanation shown by `list`.

```bash
libra worktree lock my-feature
libra worktree lock my-feature --reason "long-running experiment"
libra --json worktree lock --path ../existing-tree --reason "long-running experiment"
```

### Subcommand: `unlock`

Unlock a named worktree or one selected by `--path`. Unlocking an already-unlocked worktree is a no-op.

```bash
libra worktree unlock my-feature
libra --machine worktree unlock --path ../existing-tree
```

### Subcommand: `move`

Move an ordinary linked worktree to an explicit destination on the same filesystem. Its name and stable identity remain unchanged; later name-based commands resolve to the new stored path. A cross-filesystem move is refused before copying any files, and the source and its registration remain in place. Choose a destination on the same filesystem, or create a separate worktree on the other filesystem and transfer changes explicitly. The main worktree, locked worktrees and FUSE worktrees cannot be moved.

| Argument / Flag | Description |
|-----------------|-------------|
| `<NAME>` | Registered source name; mutually exclusive with `--from-path`. |
| `--from-path <SRC>` | Select a source by its actual path, including existing unnamed worktrees. |
| `--path <DEST>` | Complete destination path. Must not already exist or be registered. Inside common storage, only this name's own managed default is allowed. |

```bash
libra worktree move my-feature --path ../my-feature-v2
libra --json worktree move --from-path ../existing-tree --path ../existing-tree-v2
```

### Subcommand: `prune`

Remove worktrees from the registry whose directories no longer exist on disk.
Only a path whose stat fails with NotFound counts as missing — a permission
error or an unmounted volume never classifies a worktree as missing. The main
worktree, locked worktrees, tombstone entries (repair's job), and scopes with
an in-progress rebase/cherry-pick/bisect are never pruned. If a pruned
entry's scoped-state cleanup fails, the entry is kept as a `tombstone` for
`libra worktree repair --confirm` to retry (reported in the `tombstoned` field).

```bash
libra worktree prune
libra --machine worktree prune
```

### Subcommand: `remove`

Remove an ordinary worktree by name or `--path`. By default the directory remains on disk and the registry entry moves to `detached_from_registry`. The **name stays reserved**, and its private HEAD, index, working files and scoped database state are preserved. Commands inside the detached directory refuse until it is reattached or removed completely.

Reattach a named entry with `libra worktree add <NAME>`; for an unnamed detached entry use `libra worktree add <NAME> --path <registered-path>` to assign a name and reattach it. The command verifies the directory identity and common-storage pointer; it does not check out a commit or overwrite staged and unstaged work. Do not supply `TARGET`, `-b` or `--detach`; when supplied for a named entry, `--path` must equal the stored path.

Use `--delete-dir` to complete removal. It checks for uncommitted changes and verifies that the directory still belongs to the registered worktree before deletion. In standard builds, `--force` bypasses the dirty-worktree refusal; **`--delete-dir --force` deletes uncommitted changes**. A replacement directory, symlink or invalid identity is refused. Only a confirmed missing path is treated as absent; inspection errors do not authorize deletion. Scoped cleanup follows directory deletion. If cleanup fails, the `tombstone` entry and its name remain reserved for `libra worktree repair --confirm`; the name is released only when cleanup finishes. Main, locked and active-sequencer worktrees cannot be removed.

| Argument / Flag | Description |
|-----------------|-------------|
| `<NAME>` | Registered worktree name; mutually exclusive with `--path`. |
| `--path <PATH>` | Select an existing worktree by its actual path. |
| `--delete-dir` | Delete a verified directory and its scoped state, then release the registration and name. Without `--force`, the worktree must be clean. |
| `-f, --force` | Standard builds only: bypass the uncommitted-changes check. Combined with `--delete-dir`, this deletes unsaved work. Unavailable in `worktree-fuse` builds. |

```bash
# Keep files, staging state and the reserved name
libra worktree remove my-feature
libra worktree add my-feature

# Delete a clean worktree and release its name
libra worktree remove my-feature --delete-dir
libra --machine worktree remove --path ../existing-tree --delete-dir
```

Behavior intentionally differs from Git: Git's default deletes the directory.
Libra keeps it by default (as a frozen, re-attachable detached worktree) to
prevent accidental data loss; `--delete-dir` restores Git-like semantics
opt-in. See
[`COMPATIBILITY.md`](../../COMPATIBILITY.md) and
[`compatibility/worktree-surface.md`](../development/commands/worktree.md)
for the rationale.

### Subcommand: `umount`

Unmount a FUSE worktree mountpoint. This is primarily useful for cleaning up
stale Agent task worktrees when the operating system reports a path as busy.
The command also accepts a historical Libra task FUSE root and resolves its
`workspace` mountpoint automatically. `--cleanup` supports only those historical
`libra-task-worktree-fuse-*` roots; it does not delete new named FUSE containers or
new task attempts. Use `worktree remove <NAME>` for a named FUSE worktree and the
run cleanup commands for review/investigation attempts.
The cleanup target is checked before unmounting; a managed named worktree is refused even if its name begins with `libra-task-worktree-fuse-`.

Alias: `unmount`

| Argument / Flag | Description |
|-----------------|-------------|
| `<path>` | FUSE mountpoint path, or a Libra task worktree root containing a `workspace` mountpoint. |
| `--cleanup` | After unmounting, remove the Libra task worktree root. Only task FUSE worktree paths are accepted. |

```bash
libra worktree umount /repo/.libra/worktrees/tasks/libra-task-worktree-fuse-29353-id/workspace --cleanup
libra --json worktree umount /repo/.libra/worktrees/tasks/libra-task-worktree-fuse-29353-id --cleanup
```

JSON / machine output envelope:

```json
{
  "ok": true,
  "command": "worktree.umount",
  "data": {
    "mountpoint": "/repo/.libra/worktrees/tasks/libra-task-worktree-fuse-29353-id/workspace",
    "unmounted": true,
    "cleanup_requested": true,
    "cleanup_root": "/repo/.libra/worktrees/tasks/libra-task-worktree-fuse-29353-id",
    "cleanup_root_removed": true
  }
}
```

## Common Commands

```bash
# Create a new worktree
libra worktree add experiment

# List all worktrees
libra wt list

# Lock a worktree to protect it
libra wt lock experiment --reason "production hotfix in progress"

# Unlock when done
libra wt unlock experiment

# Move a worktree to a new location
libra wt move experiment --path ../experiment-v2

# Clean up worktrees whose directories were deleted
libra wt prune

# Unregister a worktree (keeps files on disk)
libra wt remove experiment

# Fix inconsistent worktree metadata
libra wt repair --confirm

# Restore a linked worktree's gitdir identity from the registry
libra wt repair experiment --confirm

# Read-only diagnosis of every Agent workspace scope
libra wt doctor
```

## Human Output

**`worktree add`**:

```text
/Users/alice/projects/my-repo/.libra/worktree/linked/my-feature
```

**`worktree list`**:

```text
main /Users/alice/projects/my-repo
worktree my-feature /Users/alice/projects/my-repo/.libra/worktree/linked/my-feature
worktree hotfix /Users/alice/projects/my-repo/.libra/worktree/linked/hotfix [locked: production hotfix in progress]
```

**`worktree remove`**:

```text
Detached worktree '/Users/alice/projects/my-repo/.libra/worktree/linked/my-feature' from the registry. Directory and its state kept on disk (frozen); re-add it with `libra worktree add` or delete it with `--delete-dir`.
Removed worktree '/Users/alice/projects/my-repo/.libra/worktree/linked/my-feature' from registry and deleted directory.
```

**`worktree prune`** (with stale entries):

```text
Will prune 2 worktrees:
  /Users/alice/projects/old-experiment
  /Users/alice/projects/deleted-branch
Pruned 2 worktrees
```

**`worktree prune`** (nothing to prune):

```text
No worktrees to prune
```

## JSON Output

`worktree add`, `lock`, `unlock`, `move`, `prune`, `remove`, and `repair`
use command-specific envelopes. `--machine` emits the same schemas as compact
single-line JSON.

**`worktree.add`**:

```json
{
  "ok": true,
  "command": "worktree.add",
  "data": {
    "path": "/Users/alice/projects/my-repo/.libra/worktree/linked/my-feature",
    "already_exists": false,
    "reattached": false
  }
}
```

**`worktree.list`**:

```json
{
  "ok": true,
  "command": "worktree.list",
  "data": {
    "schema_version": 2,
    "worktrees": [
      {
        "kind": "main",
        "path": "/Users/alice/projects/my-repo",
        "is_main": true,
        "locked": false,
        "lock_reason": null,
        "exists": true,
        "worktree_id": null,
        "state": "active",
        "epoch": 0,
        "layout": "main"
      },
      {
        "name": "my-feature",
        "kind": "worktree",
        "path": "/Users/alice/projects/my-repo/.libra/worktree/linked/my-feature",
        "is_main": false,
        "locked": false,
        "lock_reason": null,
        "exists": true,
        "worktree_id": "1f0c…",
        "state": "active",
        "epoch": 1,
        "layout": "linked-v2"
      }
    ]
  }
}
```

**`worktree.lock`**:

```json
{
  "ok": true,
  "command": "worktree.lock",
  "data": {
    "path": "/Users/alice/projects/my-repo/.libra/worktree/linked/my-feature",
    "locked": true,
    "lock_reason": "long-running experiment",
    "changed": true
  }
}
```

**`worktree.unlock`**:

```json
{
  "ok": true,
  "command": "worktree.unlock",
  "data": {
    "path": "/Users/alice/projects/my-repo/.libra/worktree/linked/my-feature",
    "locked": false,
    "changed": true
  }
}
```

**`worktree.move`**:

```json
{
  "ok": true,
  "command": "worktree.move",
  "data": {
    "source": "/Users/alice/projects/my-repo/.libra/worktree/linked/my-feature",
    "destination": "/Users/alice/projects/my-feature-v2",
    "registry_updated": true,
    "disk_directory_moved": true
  }
}
```

**`worktree.prune`**:

```json
{
  "ok": true,
  "command": "worktree.prune",
  "data": {
    "pruned": ["/Users/alice/projects/old-experiment"],
    "pruned_count": 1
  }
}
```

**`worktree.remove`**:

```json
{
  "ok": true,
  "command": "worktree.remove",
  "data": {
    "path": "/Users/alice/projects/my-repo/.libra/worktree/linked/my-feature",
    "registry_removed": false,
    "disk_directory_deleted": false,
    "detached": true,
    "tombstone": false
  }
}
```

**`worktree.repair`**:

```json
{
  "ok": true,
  "command": "worktree.repair",
  "data": {
    "changed": true,
    "journal_recovered": 1,
    "tombstones_cleaned": 1,
    "tombstones_pending": 0,
    "notes": ["completed interrupted remove of '/abs/path/wt'"]
  }
}
```

**`worktree.repair` with a path**:

```json
{
  "ok": true,
  "command": "worktree.repair",
  "data": {
    "path": "/abs/path/to/experiment",
    "worktree_id": "1f0c…",
    "worktree_id_restored": true,
    "commondir_restored": true
  }
}
```

**`worktree.doctor`** (paginated view — `data.diagnostics[]` plus
`data.next_cursor`, which is `null` on the last page):

```json
{
  "ok": true,
  "command": "worktree.doctor",
  "data": {
    "schema_version": 1,
    "diagnostics": [
      {
        "workspace_id": "9f1c2f1e-4a0e-4c0e-9a71-6a2f4a2e0b13",
        "repo_id": "8f5b…",
        "kind": "linked",
        "state": "active",
        "path": "/Users/alice/projects/my-repo/.libra/worktree/linked/my-feature",
        "worktree_id": "1f0c…",
        "lease_state": "expired",
        "lease_owner": "agent-review-3",
        "lease_fence": 2,
        "lease_expires_at": 1785304730095,
        "scope_diagnostics": [
          {
            "code": "lease_expired",
            "severity": "warning",
            "detail": "the lease deadline passed (expires_at 1785304730095, now 1785305330095); the lease still belongs to its owner until it is explicitly reclaimed"
          }
        ]
      }
    ],
    "next_cursor": null
  }
}
```

**`worktree.doctor <workspace-id>`** (single-scope view — the singular
`data.diagnostic`, with no pagination keys):

```json
{
  "ok": true,
  "command": "worktree.doctor",
  "data": {
    "schema_version": 1,
    "diagnostic": {
      "workspace_id": "9f1c2f1e-4a0e-4c0e-9a71-6a2f4a2e0b13",
      "repo_id": "8f5b…",
      "kind": "linked",
      "state": "active",
      "path": "/Users/alice/projects/my-repo/.libra/worktree/linked/my-feature",
      "worktree_id": "1f0c…",
      "lease_state": "held",
      "lease_owner": "agent-review-3",
      "lease_fence": 2,
      "lease_expires_at": 1785305930095,
      "scope_diagnostics": []
    }
  }
}
```

## Design Rationale

### Why JSON-file persistence instead of filesystem links like Git?

Git tracks worktrees through a combination of filesystem structure: the main `.git/worktrees/` directory contains per-worktree directories with `gitdir`, `HEAD`, and `commondir` files, and each linked worktree has a `.git` file (not directory) pointing back. This approach is tightly coupled to Git's file-based architecture and requires careful cross-referencing between multiple locations.

Libra uses a single `worktrees.json` file in the shared storage directory. This provides several advantages: all worktree metadata is in one queryable location, state is written atomically (via temp-file rename), and the format is trivially inspectable by both humans and AI agents. Each linked worktree's real `.libra` gitdir holds a one-way `commondir` pointer back to the shared storage (plus its stable `worktree_id`), which is simpler than Git's bidirectional pointer system. The trade-off is that the JSON file is a single point of truth that must be kept consistent, which is why `repair` exists.

### Why `--reason` on lock?

Git's `git worktree lock` also supports `--reason`, and Libra preserves this. Lock reasons are valuable in team environments and when AI agents manage worktrees: they provide context about why a worktree should not be pruned or removed. Without a reason, a locked worktree is opaque, and another user (or agent) cannot determine whether the lock is still relevant. The reason is displayed in `list` output, making lock status self-documenting.

### Why does `remove` not delete directories on disk?

Deleting files is a destructive operation that cannot be undone. Libra's default `remove` DETACHES the worktree instead: the directory, its scoped database state, and its identity stay intact while the entry moves to `detached_from_registry` and the directory is frozen (every command inside it fails closed with a re-add/delete hint). This is a deliberate safety choice: the user can re-attach with `worktree add`, or finish the removal with `--delete-dir` once confident nothing is needed. It also prevents accidental data loss if a worktree contains uncommitted work. Git's `git worktree remove` deletes the directory by default, which has been a source of lost work.

### Why does `move` reject locked worktrees?

A locked worktree signals that it should not be modified. Moving it would change its filesystem path, which could break references to that path in other tools, scripts, or agent configurations. The user must explicitly unlock the worktree before moving it, ensuring the action is intentional.

### Why does `add` populate from HEAD instead of the index?

When creating a linked worktree, Libra restores content from the HEAD commit rather than the current index state. This ensures the new worktree reflects the last committed state, not any staged-but-uncommitted changes that exist only in the original worktree's context. This matches user expectations: a new worktree starts from a known good state.

## Parameter Comparison: Libra vs Git vs jj

| Operation | Libra | Git | jj |
|-----------|-------|-----|----|
| Create worktree | `worktree add <NAME> [TARGET] [--path <PATH>]` (no target: detached at source commit) | `worktree add <path> [<branch>]` | `workspace add <path>` |
| Create on branch | `worktree add <NAME> <branch>` (attached; refused if checked out anywhere; no DWIM) | `worktree add <path> <branch>` | `workspace add <path>` (then `jj edit`) |
| Create detached | `worktree add --detach <NAME> [<commit>]` (also `add <NAME> <commit>`) | `worktree add --detach <path> <commit>` | N/A |
| List worktrees | `worktree list` | `worktree list [--porcelain]` | `workspace list` |
| Lock | `worktree lock <NAME> [--reason]` | `worktree lock [--reason] <worktree>` | N/A |
| Unlock | `worktree unlock <NAME>` | `worktree unlock <worktree>` | N/A |
| Move | `worktree move <NAME> --path <DEST>` | `worktree move <worktree> <new-path>` | N/A |
| Prune | `worktree prune` | `worktree prune [--dry-run]` | N/A (automatic) |
| Remove | `worktree remove <NAME>` (detaches — directory + state kept, frozen) | `worktree remove [--force] <worktree>` (deletes dir) | `workspace forget <name>` |
| Repair | `worktree repair [<NAME>] --confirm` | `worktree repair [<path>...]` | N/A |
| Alias | `wt` | N/A | N/A |
| Branch per worktree | `-b <new> [<start>]` explicit (full rollback; no basename default) | Automatic (new branch or existing) | Automatic (new working copy commit) |
| Storage | JSON file (`worktrees.json`) | Filesystem structure (`.git/worktrees/`) | Operation log |
| Worktree link | Real local `.libra` gitdir with a `commondir` pointer to shared storage (legacy layouts: symlink) | `.git` file pointing to `gitdir` | Symlink to shared `.jj` |

Note: jj uses the term "workspace" instead of "worktree". Each workspace automatically gets its own working copy commit, and workspaces are tracked in the operation log. jj workspaces are simpler than Git worktrees because jj's change-based model does not require separate branch management per workspace.

## Error Handling

| Code | Condition |
|------|-----------|
| `LBR-REPO-001` | Not a libra repository |
| `LBR-REPO-002` | `worktrees.json` is corrupt |
| `LBR-CLI-003` | Invalid name or explicit destination enters protected metadata or another managed workspace |
| `LBR-CLI-003` | Reattaching a detached worktree with `TARGET`, `-b`, or `--detach` (its saved HEAD must resume unchanged) |
| `LBR-CLI-003` | Target exists and is not a directory |
| `LBR-CLI-003` | No such worktree (for lock, unlock, move, remove) |
| `LBR-CLI-003` | Cannot move or remove main worktree |
| `LBR-CLI-003` | Cannot move or remove locked worktree |
| `LBR-CLI-003` | `worktree umount --cleanup` was requested for a non-task FUSE worktree path |
| `LBR-CONFLICT-002` | Target directory exists and is not empty |
| `LBR-CONFLICT-002` | Target already contains a `.libra` entry |
| `LBR-CONFLICT-002` | Destination already exists (for move) |
| `LBR-CONFLICT-002` | Destination already registered as worktree (for move) |
| `LBR-CONFLICT-002` | Name belongs to another registered path/backend, or `--path` disagrees with the name's stored path |
| `LBR-CONFLICT-002` | Cross-filesystem move cannot preserve the worktree through an atomic rename; source and registration stay in place |
| `LBR-CONFLICT-002` | `--delete-dir` refused because the worktree is dirty |
| `LBR-IO-001` | Failed to read or inspect worktree paths/state/status |
| `LBR-IO-002` | Failed to write worktrees.json |
| `LBR-IO-002` | Failed to populate worktree from HEAD |
| `LBR-CLI-002` | `worktree doctor <workspace-id>` combined with `--limit`/`--cursor` |
| `LBR-CLI-003` | `worktree doctor <workspace-id>` names an unknown workspace |
| `LBR-WORKTREE-001` | `worktree doctor --cursor` was given a cursor this command did not issue (or one that expired) |
| `LBR-WORKTREE-002` | `worktree doctor` cannot read a scope (unparseable registry, unreadable record, missing repository identity) and refuses to report a partial diagnosis |

A build that supports managed worktrees records the `2026101902` worktree-layout capability marker when it first opens a repository. Libra 0.30.66 and older then refuse every command in that repository, including commands unrelated to worktrees. Published binaries first support this marker in v0.30.67 on Linux and v0.30.68 on macOS/Windows, but the marker alone does not make intermediate builds safe for named worktrees. Any registry mutation by v0.30.85 writes schema 4, even without named entries. After such a mutation, use v0.30.85 or a subsequent release containing its registry and identity-safety fixes for worktree commands; v0.31.0 predates those fixes. Existing worktree directories are not moved.
