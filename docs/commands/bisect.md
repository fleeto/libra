# `libra bisect`

Use binary search to find the commit that introduced a bug.

## Synopsis

```
libra bisect start [<bad> [<good>...]] [--term-old <term>] [--term-new <term>] [--first-parent]
libra bisect bad [<rev>]
libra bisect good [<rev>]
libra bisect new [<rev>]
libra bisect old [<rev>]
libra bisect reset [<rev>]
libra bisect skip [<rev>]
libra bisect terms [--term-good | --term-bad]
libra bisect log
libra bisect run <cmd> [<args>...]
libra bisect view
```

## Description

`libra bisect` performs a binary search through the commit history to find the specific commit that introduced a regression or bug. The user marks commits as "good" (working correctly) or "bad" (containing the bug), and bisect systematically checks out commits in between until the first bad commit is identified.

A bisect session begins with `bisect start`, which saves the current HEAD and branch so they can be restored later. The user then marks the boundaries: a "bad" commit (where the bug exists) and one or more "good" commits (where the bug does not exist). Bisect calculates the midpoint between good and bad in the commit graph using BFS traversal, checks out that commit, and waits for the user to test and mark it. This process repeats, halving the search space each time, until the culprit commit is found.

Bisect state is persisted in a `bisect_state` table in the SQLite database, making sessions survive process restarts. The state tracks the original HEAD, the bad commit, all good commits, skipped commits, the current test commit, estimated remaining steps, and whether the session has completed.

When bisect identifies the culprit, it prints the commit details and marks the session as completed. The user must then run `bisect reset` to end the session and restore HEAD to its original position. Marking `good` or `bad` again after convergence is not an error: the step exits 0 and re-prints the culprit (`<oid> is the first bad commit`, followed by the commit subject). The culprit subject is taken from the commit message body, so a vault signature's `gpgsig` / `gpgsig-sha256` header block is never displayed as the subject.

Libra does not manage submodule contents. If checking out a bisect candidate or reset target would remove or replace a tracked gitlink (mode `160000`) whose directory is nonempty, the command refuses with `LBR-CONFLICT-002` before changing HEAD, the index, or the working tree. Move the nested files safely aside before retrying; an empty Libra-owned directory placeholder can be removed successfully. Initial `bisect start` requires a clean tree for **tracked** files only (staged or unstaged); untracked and ignored files — including a root `.libraignore` — no longer block the start (ADR-MX-05 §2), and a candidate whose checkout would overwrite a colliding untracked/ignored path is refused by the per-step restore preflight before any state is written. In an existing session, a `good`, `bad`, or `skip` step is atomic: the restore preflight runs before the mark is persisted, so a step refused by this check leaves the saved bisect state untouched (`current` does not advance and `bisect log` is unchanged) — fix the conflict and retry the step. `reset` is different by design (ADR-MX-03): a refused restore never wedges the session. `reset` still ends the session — it clears the state row, warns with the original HEAD target, and leaves HEAD at the current candidate so the nested files can be moved aside and the checkout retried manually.

Custom good/bad terms (ADR-MX-11) are supported: `bisect start --term-old <term> --term-new <term>` renames the two sides (for example `fast`/`slow`), the terms are persisted for the session, and the custom names become mark subcommands (`libra bisect slow`) while `bad`/`good` are refused for that session. `new`/`old` are aliases that fix an unbound session to the `new`/`old` terms, and `bisect terms` reports the current terms.

### Session safety and unified errors

Every `good`/`bad`/`skip` step computes the next candidate and preflights its checkout before writing session state, so a failed step (for example a candidate that tracks a path currently present as an untracked directory) never advances `current` and never wedges the session. `bisect reset` always ends the session: it restores the original HEAD (branch or detached commit), and if that restore fails it still clears the state row and prints a warning naming the target so the checkout can be retried manually. Without a session, `bad`, `good`, `skip`, `log`, `view`/`visualize` and `run` all fail with the stable error `LBR-BISECT-001` (exit 128, `not in an active bisect`); `reset` without a session prints `We are not bisecting.` and exits 0.

## Options

### Subcommand: `start`

Begin a new bisect session. Saves the current HEAD and branch for later restoration.

| Argument / Flag | Description |
|-----------------|-------------|
| `<bad>` | Optional commit to immediately mark as bad. If omitted, use `bisect bad` later. |
| `<good>...` | Optional positional commits to immediately mark as good (Git-compatible). If omitted, use `bisect good` later. |
| `--good` / `-g` | Optional commit to immediately mark as good (compatibility alias for the first positional good). |
| `--term-old` / `--term-good` | Custom term for the old/good side (default `good`). |
| `--term-new` / `--term-bad` | Custom term for the new/bad side (default `bad`). |
| `--first-parent` | Follow only the first parent of merge commits, restricting the bisect to mainline history (a merged-in side branch contributes no testable commits). |

```bash
# Start with no initial markers
libra bisect start

# Start with known bad (current HEAD) and good commit (positional, Git-compatible)
libra bisect start HEAD v1.0

# Start with several known-good commits
libra bisect start HEAD v1.0 v1.1

# Start with a specific bad commit and the --good compatibility alias
libra bisect start abc1234 --good def5678

# Bisect only the first-parent (mainline) history, ignoring merged side branches
libra bisect start HEAD --good v1.0 --first-parent

# Use custom terms (fast = old/good, slow = new/bad)
libra bisect start --term-old fast --term-new slow HEAD v1.0
```

### Subcommand: `bad`

Mark the current or given commit as bad (contains the bug). If both good and bad commits are known, bisect immediately calculates the next midpoint and checks it out.

| Argument | Description |
|----------|-------------|
| `<rev>` | Commit to mark as bad. Defaults to the current HEAD. |

```bash
# Mark current commit as bad
libra bisect bad

# Mark a specific commit as bad
libra bisect bad abc1234
```

### Subcommand: `good`

Mark the current or given commit as good (does not contain the bug). If both good and bad commits are known, bisect calculates the next midpoint and checks it out.

| Argument | Description |
|----------|-------------|
| `<rev>` | Commit to mark as good. Defaults to the current HEAD. |

```bash
# Mark current commit as good
libra bisect good

# Mark a specific commit as good
libra bisect good def5678
```

### Subcommand: `terms`

Report the session's terms. With no option, prints git's two-line form naming the old/good and new/bad terms; `--term-good`/`--term-old` prints only the old/good term and `--term-bad`/`--term-new` prints only the new/bad term. Without a session it fails with `LBR-BISECT-001` (exit 128); a session whose terms are not yet fixed (an unbound `bisect start` with no mark) reports `no terms defined`.

```bash
# Show both terms
libra bisect terms

# Show only the old/good term
libra bisect terms --term-good

# Show only the new/bad term
libra bisect terms --term-bad
```

### Subcommand: `new` / `old` (and custom terms)

`new` and `old` are aliases for `bad` and `good`. After an unbound `bisect start`, the first `new`/`old` mark fixes the session to the `new`/`old` terms; in a session whose terms are fixed, `new`/`old` (and `bad`/`good`) are only accepted when they match the session's terms — a mismatching mark is a usage error. When a session started with `--term-old fast --term-new slow`, the custom names become mark subcommands (`libra bisect slow`, `libra bisect fast`) and `bisect log`/`bisect view` render them.

```bash
# Fix an unbound session to the new/old terms
libra bisect new HEAD
libra bisect old v1.0

# Custom-term session: mark with the custom names
libra bisect slow
libra bisect fast
```

### Subcommand: `reset`

End the bisect session and restore HEAD to its original position (the branch or commit that was checked out before `bisect start`). If a `<rev>` is provided, HEAD is restored to that commit instead of the original.

| Argument | Description |
|----------|-------------|
| `<rev>` | Optional commit to reset to instead of the original HEAD. |

```bash
# End bisect and restore original HEAD
libra bisect reset

# End bisect and go to a specific commit
libra bisect reset main
```

### Subcommand: `skip`

Skip the current commit and move to the next candidate. Useful when the current commit cannot be tested (e.g., it does not compile). Skipped commits are excluded from future midpoint calculations. If too many commits are skipped, bisect may not be able to narrow down the culprit precisely.

| Argument | Description |
|----------|-------------|
| `<rev>` | Commit to skip. Defaults to the current HEAD. |

```bash
# Skip the current commit
libra bisect skip

# Skip a specific commit
libra bisect skip abc1234
```

### Subcommand: `log`

Show the bisect log, displaying all good, bad, and skipped marks made during the current session.

```bash
libra bisect log
```

### Subcommand: `run`

Run a command at each bisect step and dispatch `good` / `bad` / `skip` automatically based on its exit code. The command is invoked at each candidate commit and bisect advances until convergence (or until candidates are exhausted).

`bisect run` requires an active session that already has both a bad bound and at least one good bound, so start it with `libra bisect start <bad> --good <good>` or mark both bounds manually before invoking automation.

| Argument | Description |
|----------|-------------|
| `<cmd> [<args>...]` | The command to execute. The first token is the executable; everything after is forwarded verbatim. `--` is allowed and pass-through (e.g. `libra bisect run cargo test -- --ignored`). |

Exit-code semantics (aligned with stock `git bisect run`):

| Exit code | Mark / Action |
|-----------|---------------|
| `0` | `good` |
| `1`–`124`, `126`–`127` | `bad` |
| `125` | `skip` (cannot test this commit) |
| `128` and above | Terminate the bisect with a fatal `BISECT_RUN_FAILED` error |

Killed by signal also terminates the bisect with a fatal error.

```bash
# Drive bisect with a cargo test
libra bisect run cargo test --test foo

# Pass flags through to the underlying test command
libra bisect run cargo test -- --ignored

# Use a custom shell script
libra bisect run bash -c 'cargo build && ./target/debug/repro'
```

### Subcommand: `view` (alias: `visualize`)

Show the current bisect state — good / bad boundaries, current HEAD, remaining candidates, and any skipped commits. `visualize` is an alias for `view`: where Git's `bisect visualize` launches a GUI (gitk) or pager, Libra is terminal-native and prints the same text state summary.

```bash
libra bisect view
libra bisect visualize   # alias for view
```

If no bisect is in progress, returns the unified no-session fatal error `LBR-BISECT-001` (`NOT_IN_BISECT`, exit 128) — the same error `bad`/`good`/`skip`/`log`/`run` report without a session.

## JSON / Machine Output

`libra bisect` supports global `--json` and `--machine` for all subcommands.
Both modes emit a single `bisect` command envelope on success; `--machine`
uses the same envelope as one compact line and suppresses human progress.

Common fields:

| Field | Description |
|-------|-------------|
| `action` | One of `start`, `mark`, `skip`, `reset`, `log`, `view`, `run`. |
| `status` | Present for state transitions: `started`, `waiting_for_good`, `waiting_for_bad`, `testing`, `converged`, or `all_skipped`. |
| `bad` / `good` / `current` | Full commit IDs for the current bisect bounds and candidate. |
| `remaining` / `steps` | Candidate count and estimated remaining search steps when known. |
| `first_bad` | Full commit ID when the session converged. |
| `term_good` / `term_bad` | The session's old/good and new/bad terms when fixed. |

Example:

```json
{
  "ok": true,
  "command": "bisect",
  "data": {
    "action": "view",
    "head": "901abcd...",
    "good": ["abc1234..."],
    "bad": "def5678...",
    "current": "901abcd...",
    "remaining": 1,
    "completed": false
  }
}
```

## Common Commands

```bash
# Start a bisect session
libra bisect start

# Mark the current version as broken
libra bisect bad

# Mark a known-good version
libra bisect good v1.0

# Test the checked-out commit, then mark it
# (run your tests here)
libra bisect good    # if tests pass
libra bisect bad     # if tests fail

# Skip an untestable commit
libra bisect skip

# View the bisect log
libra bisect log

# End the session
libra bisect reset

# Quick start with known boundaries
libra bisect start HEAD --good abc1234
```

## Human Output

**`bisect start`**:

```text
Bisect session started.
```

**`bisect start <bad> --good <good>`** (with both markers):

```text
Bisect session started.
Bisecting: N revisions left to test (roughly M steps)
[abc1234] commit message here
```

**`bisect bad`** / **`bisect good`** (narrowing down):

```text
Bisecting: N revisions left to test (roughly M steps)
[abc1234] commit message here
```

**`bisect bad`** / **`bisect good`** (culprit found):

```text
Marked abc1234 as bad
abc1234 is the first bad commit
introduce the bug here
```

Marking `good` or `bad` again after convergence exits 0 and re-prints the same culprit block.

**`bisect skip`**:

```text
Bisecting: N revisions left to test (roughly M steps)
[def5678] next candidate commit message
```

**`bisect log`**:

```text
Bisect log:
  bad: abc1234
  good: def5678
  Current: 901abcd
  Skipped: 0 commits
  Steps remaining: Some(2)
```

**`bisect terms`**:

```text
Your current terms are 'good' for the old state
and 'bad' for the new state.
```

A custom-term session renders the custom names in `log`/`view` (for example
`slow:`/`fast:` instead of `bad:`/`good:`), and convergence prints
`<oid> is the first 'slow' commit`.

**`bisect reset`**:

```text
HEAD is now at 4f8789d (on branch main)
Bisect session ended, HEAD restored to 4f8789d
```

Without a session:

```text
We are not bisecting.
```

If the restore is refused (for example a nonempty gitlink directory), `reset` still clears the session and prints a stderr warning naming the original target, followed by:

```text
Bisect session ended; original HEAD 4f8789d could not be restored
```

**`bisect run`** (converging):

```text
Bisecting: 5 candidates remaining
Running cargo test --test foo at abc1234... PASS (good)
Bisecting: 2 candidates remaining
Running cargo test --test foo at def5678... FAIL (bad)
Bisecting: 1 candidate remaining
Running cargo test --test foo at 901abcd... FAIL (bad)
Converged: first bad commit is 901abcd
3 steps, 0 skipped
```

**`bisect view`**:

```text
Bisecting between abc1234 (good) and def5678 (bad)
HEAD: 901abcd
Remaining: 1 candidate
Skipped: (none)
```

## Design Rationale

### Why is bisect not hidden?

Despite being listed as a hidden command in some early designs, `libra bisect` is a fully visible subcommand. Binary search for regressions is a fundamental debugging workflow that benefits both human users and AI agents. Hiding it would reduce discoverability without meaningful benefit. The command is stable and follows the same patterns as other Libra commands.

### How does `bisect run` handle exit codes?

`bisect run` mirrors stock `git bisect run` to keep AI-agent and CI integration straightforward. The exit-code contract is:

- `0` → mark `good` and advance.
- `1`–`124` or `126`–`127` → mark `bad` and advance.
- `125` → `skip` (the commit cannot be tested — e.g. it does not compile) and advance.
- `128` and above → fatal: terminate the bisect and surface `BISECT_RUN_FAILED` so the caller can react. Killed by signal (e.g. SIGINT) is treated the same way.

The full command line is passed through verbatim, so `libra bisect run cargo test -- --ignored` forwards `--ignored` to the test command rather than parsing it as a `bisect` flag. This is enabled by `trailing_var_arg` + `allow_hyphen_values` on the `cmd` argument.

Manual marking (`bisect good` / `bisect bad`) remains the recommended path for AI agents that evaluate results in-process and prefer explicit control over each step.

### First-parent bisecting

By default Libra's bisect traverses the full commit graph using BFS, which is correct for all topologies. `bisect start --first-parent` mirrors `git bisect --first-parent`: it follows only the first parent of merge commits, so a merged-in side branch contributes no testable commits. This narrows the search to the mainline in workflows with many merge commits. The flag is recorded in the bisect session state, so subsequent `good`/`bad`/`skip` steps stay on the first-parent history until `bisect reset`.

### Why SQLite state persistence?

Bisect sessions can span hours or days as the user tests each candidate. Storing state in the SQLite `bisect_state` table ensures the session survives process restarts, editor closes, and system reboots. Git uses flat files in `.git/BISECT_*`, which achieves the same persistence but with less structure. SQLite provides transactional writes and the ability to query state programmatically, which is valuable for AI agent integration.

### Why does `reset` accept an optional `<rev>`?

Sometimes the user wants to end the bisect session but go to a different commit than where they started. For example, after finding the culprit, they might want to reset to the commit just before the bug was introduced. The optional `<rev>` parameter provides this flexibility without requiring a separate `checkout` after reset.

## Parameter Comparison: Libra vs Git vs jj

| 参数 | Libra | Git | jj |
|-----------|-------|-----|----|
| Start session | `bisect start [<bad> [<good>...]]` | `bisect start [<bad> [<good>...]]` | N/A |
| Mark bad | `bisect bad [<rev>]` | `bisect bad [<rev>]` | N/A |
| Mark good | `bisect good [<rev>]` | `bisect good [<rev>]` | N/A |
| Reset | `bisect reset [<rev>]` | `bisect reset [<commit>]` | N/A |
| Skip | `bisect skip [<rev>]` | `bisect skip [<rev>...]` | N/A |
| Show log | `bisect log` | `bisect log` | N/A |
| Automated run | `bisect run <cmd> [<args>...]` | `bisect run <script>` | N/A |
| Show current state | `bisect view` / `bisect visualize` | `bisect visualize` (GUI / log) | N/A |
| Custom terms | `bisect terms` / `--term-old` / `--term-new` / `new` / `old` | `bisect terms` / `--term-old` / `--term-new` | N/A |
| Replay session | Not supported (deferred — see compatibility/declined.md D6) | `bisect replay <logfile>` | N/A |
| Visualize (GUI) | `bisect visualize` (alias for `view`; prints the text state, no GUI) | `bisect visualize` | N/A |
| First-parent only | `bisect start --first-parent` | `--first-parent` | N/A |
| Multiple good commits | Via positional args to `start` or repeated `bisect good` | Positional args to `start` | N/A |
| State storage | SQLite (`bisect_state` table) | Flat files (`.git/BISECT_*`) | N/A |

Note: jj does not have a bisect command. Users who need binary search debugging with jj must use external tooling or manually check out commits. This is a gap in jj's feature set that Libra addresses.

## Error Handling

| Code | Condition |
|------|-----------|
| `LBR-REPO-001` | Not a libra repository |
| `LBR-REPO-003` | No commits in repository |
| `LBR-REPO-003` | `bisect run` invoked before both good/bad bounds select a candidate |
| `LBR-CLI-002` | Bisect session already in progress (for `start`) |
| `LBR-CLI-003` | Commit not found (invalid rev argument) |
| `LBR-CLI-003` | Bad commit is an ancestor of good commit (invalid range) |
| `LBR-CONFLICT-001` | Uncommitted changes would be overwritten by checkout |
| `LBR-CONFLICT-002` | A nonempty tracked gitlink directory would be removed or replaced (exit 128). Move nested files safely aside before retrying; an empty Libra-owned directory placeholder can be removed. |
| `LBR-IO-001` | Failed to read bisect state from database |
| `LBR-IO-002` | Failed to save bisect state to database |
| `LBR-IO-002` | Failed to create bisect_state table |
| `LBR-BISECT-001` | `bad`, `good`, `skip`, `log`, `view`, `run` or `terms` invoked without a bisect session (`NOT_IN_BISECT`); exit 128. `reset` without a session is NOT an error: it prints `We are not bisecting.` and exits 0. |
| `LBR-BISECT-002` | `bisect run` command exited with code ≥ 128 or was killed by a signal (`BISECT_RUN_FAILED`) |
| `LBR-BISECT-003` | `bisect run` cannot advance because no candidate commits remain (`BISECT_NO_CANDIDATES`) |
