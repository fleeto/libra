# Agent Command Development

## Grok capture (GK-05)

Grok is supported and hook-installable with `support_wave=grok`, not `first_batch`, and is not a `review` or `investigate` launcher. **Automatic Thinking capture is paused.** Hook lifecycle events are recorded through the shared pipeline; decoding does not read session logs or write `GROK_HOME/libra-capture`. Existing readers for local `updates.jsonl` and `chat_history.jsonl` are not yet connected to the authorized snapshot/checkpoint writer. Offline observer JSON is scratch state, not a Libra checkpoint. The intended transcript retains Thinking text and unproven `encrypted_content`; Libra does not decrypt it. Current synthetic reader tests do not prove that live checkpoints contain it. Default `checkpoint show` does not print Thinking plaintext; an explicit transcript path returns only content actually stored.

## Claude Code hook registry (CC-01)

`src/internal/ai/hooks/providers/claude/events.rs` is the only Claude event taxonomy. The parser reads lifecycle kinds from that table and does not keep a second name list. Safely installable names match the CC-00 freeze. `WorktreeCreate`, `WorktreeRemove`, and `PreModelSwitch` are control-sensitive, stay off the current ingress, and are not installed. `SessionStop`, `ModelUpdate`, and `Compaction` remain parsable deprecated aliases and are excluded from the installable set. Unknown names and unknown extra keys are partial; diagnostics are static and do not echo the payload. `agent enable` writes every safely installable event. `SessionEnd` omits the host timeout. `FileChanged` omits the matcher and does not emit `watchPaths`. `agent doctor` lists known events as installable or excluded and whether a Libra-owned entry is configured. `agent hooks claude-code observe` feeds safely installable names through the existing capture coordinator, prints nothing, and exits 0 for controllable success and failure. MessageDisplay admits 128 batches per session. The in-process gate in `events.rs` bounds one process; `HookProvider::display_batch_limit` carries the session-wide budget, which `live_pipeline::admit_display_batch` enforces with a conditional UPSERT on `metadata_kv` scope `agent_display_batch_count` (`kept` / `dropped`) before any catalog revision. Session erase deletes those rows. Control-sensitive names stay off that ingress. `normalize_claude_transcript` is the live and import reader for Claude thinking: a non-empty decoded `thinking` string becomes one `SemanticRecord::Reasoning`, while `thinking.signature` and `redacted_thinking.data` remain in the ordinary transcript bytes and are Unproven without native proof. Empty and missing stay distinct. Duplicate keys reject that line. Coverage schema stays v1 unless a readable reasoning record selects the existing v2. Authorized `checkpoint show --json` and ordinary export return the redacted thinking text and signature. List and graph do not. Live capture, import, and snapshot recovery read those attached fields from the original JSONL, with provider, session, turn, record, and content index. A duplicate key rejects that record and leaves retained bytes in place. A mismatched session file is not associated. Ordinary retention does not create a native artifact. An explicit `libra agent doctor --repair` appends one coverage revision for an authorized legacy Claude checkpoint. The revision key binds that checkpoint and the ordinary transcript content. Repeating the repair with the same content leaves the revision and the stored objects in place. Erase and the cloud mirror keep that replacement together with the old checkpoint reference, and a tombstone blocks revival.

> **Coverage-v2 source version:** The coverage-v2 descriptions below refer to the Libra 0.30.50 source line and later compatible versions. This is not a stable-release announcement; check the [official releases](https://github.com/libra-tools/libra/releases) and `libra --version` for installed support. Other sections retain their own version requirements. Coverage-v2 does not prove native origin or authorize native reasoning artifacts.

## Final OpenCode delivery scope (2026-10-18)

[plan-20260902 final C](../plan/plan-20260902.md#ordinary-final-c-20261017)
closes this wave around ordinary OpenCode capture and safe partial results.
The current ordinary source reference is official OpenCode **2.0.29**, commit
`042d0832f5f3a73eaa039a58a7d565776f6d4290`; the frozen 2.0.26 fixtures and
private `.1`/`.2` registration recipes retain their historical applicability.
Neither those fixtures nor one successful session establish support for every
export shape or model. The actual current CLI and
[ordinary source limitations](../../commands/agent.md#current-official-opencode-ordinary-source-baseline)
remain authoritative, including rejected unsupported-state historical imports
(`LBR-AGENT-020`) and bounded structural/deadline failures.

Ordinary events, prompt/model/tokens/files and supported provider-visible
reasoning use the existing typed redaction and coverage projection. An
unproven `signature`, `redactedData` or `reasoningEncryptedContent` is a private
candidate: retain partial/Incomplete classification, remove its content from
ordinary sinks and create **zero native artifacts**. A safe ordinary partial
checkpoint is possible only where the existing consumer permits it; a refused
source or pending receipt is not a successful empty checkpoint. Source A
remains UNPROVEN. No producer install, new issuer or real native-field archive
is promised by this plan. Previously published verifier/carrier and legacy
registration compatibility remain available under their existing constraints.

The native startup issue was **closed-by-scope-decision, unfixed and non-PASS**,
not rescheduled. See the [gap ledger](../gap/agent-trace.md#opencode-final-c-ledger)
for all six withdrawn cards and the original deferred inventory. Native work
requires a new explicit product decision; it has no automatic restart queue.
Linux functional acceptance remains with the user. Installed OpenCode,
exporter/hook-to-Session and account/model acceptance, including controlled
fake-exporter Session tests, run manually on macOS outside CI. Pure parser,
codec, cryptographic and sandbox/static safety tests remain eligible for CI.


`libra agent` is an intentionally different external-agent capture extension,
not a Git-compatible command.

OpenCode export on macOS requires a **Required Seatbelt server/export pair**.
Both deny `process-fork`, use private HOME/XDG roots and the exact pinned
OpenCode store, and share one authenticated loopback port. The same absolute
deadline covers trust, sealing, readiness, bounded export and clean server
shutdown; failure remains **metadata-only**, without an unsandboxed fallback.
The direct leader is disarmed after reap, preventing signals to a recycled PGID.
See [`../tracing/agent.md`](../tracing/agent.md) §5 for the platform boundary.
Linux content export is available only when the trusted system `bwrap` passes
Libra's descriptor-native `--bind-fd`/`--ro-bind-fd` safety probe; unsupported
or legacy bwrap is unavailable and remains metadata-only. Install or upgrade
the system bwrap package—there is no unsandboxed fallback.

The shared `run_bounded_exporter` Unix `pre_exec` sets both soft and hard
`RLIMIT_CORE` to zero next to the existing `RLIMIT_FSIZE`. That file-size
limit is a coarse 8 GiB backstop on every platform, not the 16 MiB export
cap: it is process-wide, so a cap-sized limit also failed OpenCode's own
SQLite writes once the store outgrew the cap (macOS FIX-SBX-01; on Linux the
`--standalone` server then exited before readiness and every capture was
metadata-only). The export cap stays enforced on stdout by the size poll and
the post-exit recheck.
Failure to set either limit fails spawn with context. Core limits propagate
to exporter descendants without changing the parent process; SIGXFSZ
disposition, byte caps, deadlines and sandbox controls are unchanged.
This also suppresses cores from unexpected exporter crashes. Piped core
handlers decide whether to honor the limit. systemd-coredump v259 honors
it when `core_pattern` passes the limit through `%c`; brief signal metadata
can remain. The regression
`opencode_export_core_limits_are_zero_in_child_and_descendants` exercises
the real runner and confirms child/descendant limits and unchanged parent
limits. It does not establish the cause of any historical linker SIGKILL.

The active development contract, backlog, and compatibility guardrails live in
[`../tracing/agent.md`](../tracing/agent.md). Keep this file as the command
development index entry so `docs/development/commands/README.md` can list every
public CLI command without duplicating the Agent planning document.

## Deferred / Non-goal parity

The following external-agent parity surfaces are decided **non-goals** for the
current wave. Each is recorded — with its handling and restart condition — in
the 「还未实现的功能」 table of [`../tracing/agent.md`](../tracing/agent.md)
(the canonical Agent contract); they are surfaced to users in
[`docs/commands/agent.md`](../../commands/agent.md) and in the `agent` row of
[`COMPATIBILITY.md`](../../../COMPATIBILITY.md):

1. **`agent add`/`remove` `--local-dev` / `--force`** — unpublished; canonical
   `status` / `enable` / `disable` (+ `add` / `remove` aliases) only. If
   implemented, each must hang on both the canonical verb and its alias.
2. **Provider-specific transcript compaction/reassemble trait** — deferred parity
   on top of the landed manifest-relative chunking (no provider-specific
   compactor yet).
3. **Optional capability traits** (`ProtectedFilesProvider`, `TranscriptCompactor`,
   `HookResponseWriter`, `RestoredSessionPathResolver`, …) beyond the landed
   `DeclaredAgentCaps` set — no public behavior yet.
4. **External-RPC method family beyond the v2 `info`/capability gate** —
   undeclared capabilities stay fail-closed.
5. **Non-first-batch supported roster** — `gemini` / `cursor` / `copilot` /
   `factory-ai` stay `supported=false` (unsupported, not hook-installable, not
   launchable) and are omitted from `agent list` entirely; the first batch is
   `claude-code` / `codex` / `opencode`. The omission is pinned by
   `tests/command/agent_roster_test.rs::agent_roster_surface`; the unsupported
   registry classification stays pinned by
   `tests/compat/agent_capability_matrix_pin.rs`.

## Private capture erasure

Doctor's terminal-finalizer cold scan uses keyset pagination in one read-only
snapshot: 32 sessions/page, 512 sessions and 128 receipt findings/run, with a
1 MiB SQL byte cap on each metadata ledger before hydration. Bad ledger rows
do not abort later valid rows. Fixed content-free notes disclose skipped keys
or bounded truncation; neither is evidence that unlisted receipts are clean.
`pending_source` survives later stop/resume as superseded manual-only evidence.
Session-quarantined receipts also remain visible; repair never overwrites a
newer session or calls quarantine successful terminal capture. Artifact-bound
budget exhaustion moves only the header, preserving the original receipt.
Eligible artifacts use the same bounded doctor executor: alias, receipt,
marker, and coverage fences are revalidated before local checkpoint replay.
Each `doctor --repair` invocation counts at most five replay attempts.
Automatic replays of eligible artifacts share one 2-second cooperative
deadline; each audited manual attempt gets its own fresh 2-second deadline.
Deferred findings (batch cap or retry-later) are not counted and require
another bounded invocation. Success requires both a durable checkpoint and strict
completion of the original receipt. An expired artifact gets one audited
manual attempt after quarantine; failures remain visible and do not reset the
original receipt policy.

Private recovery erasure uses the existing two-stage tombstone/catalog workflow:
tombstone commit and checkpoint prune precede one transaction deleting the
session plus attributable headers/chunks/aliases. Attribution is the local
canonical-repo catalog PK and capture incarnation; no key or receipt-ledger
proof is required, and old worktree/workspace scope does not block deletion.
Foreign/ambiguous corrupt evidence is retained; a content-free lost-capacity
warning is logged after commit. Without a consistent original DB backup the
capacity is lost, not repairable through a force-discard option. DB-copy backups
retain sensitive associations and can undo the local tombstone on restoration.
Receipt completion instead removes the exact artifact and only unreferenced
aliases in that same writer transaction. A trusted `SessionEnd` whose budget
expires after a complete redacted snapshot persists this artifact (ACF-13); a
later `SessionStart` may launch the bounded detached recovery worker (ACF-11)
and `libra agent doctor --repair` replays it (ACF-12).

Same-checkpoint pending/quarantine collisions are not broad deletion authority:
only owner-attributed headers are removed. Unknown/foreign headers and shared
chunks survive; explicit erase reports retained capacity, not complete removal
of every private copy.

Unknown or mismatched incarnation for a known alias refuses final deletion;
restore consistent session/alias metadata before retrying. The earlier
tombstone remains committed. One undecodable retained header blocks all
new/re-delivered recovery artifacts and destructive GC/root-walking
maintenance, and can conservatively retain completion aliases. Explicit
attributable erase still works. Out-of-band reads are capped at 17 headers and
17 associations; unexamined data may include the erased session's content,
so the warning does not certify removal of every private copy.

## Historical import contract

The active DR-05/M4 contract is implemented by `libra agent import` and is
specified canonically in [`../tracing/agent.md`](../tracing/agent.md): explicit
consent before content access/export, provider-root descriptor authorization,
typed redaction before persistence, current-repository ownership, coverage
claim + import identity fencing, and local erase tombstones. The default
`agent list --json` remains schema v1; callers opt into the method matrix with
`--schema-version 2`. Batch limits charge bytes actually read from the held
source even when a candidate later fails validation, and the absolute deadline
begins before discovery, bounds reservation/object/CAS work, and releases every
owned uncommitted import lease on expiry. Transaction commit awaits are not
cancelled: the deadline is checked immediately before commit and the resulting
success is authoritative even if observation finishes after the deadline. A
failed abandonment is chained into the surfaced error with a doctor-repair hint.
New V2 import records retain a source only as the repository-keyed,
domain-separated `source/hmac-v2/<64 lower-hex>` commitment. New V2 subagent
content records use their separately domain-separated
`source/subagent-hmac-v2/<64-lower-hex>` commitment. Raw locators and unkeyed
source SHA-256 values are not durable metadata, claims, markers, logs, errors,
or cloud fields. A V2 snapshot digest is a distinct-domain, repository-keyed
`source/hmac-v2/<64 lower-hex>` commitment over redacted snapshot content; the
helper's transient SHA-256 is never durable. A tagged unkeyed SHA-256 remains read-only immutable V1 proof; a bare unkeyed SHA-256 has the same legacy-only status. The migration path verifies the exact scoped legacy proof and atomically moves only committed, quiescent
identity/catalog/repair-marker state. Live, partial, or repair-pending V1
state remains V1 rather than creating parallel V2 state.
Repository ownership is the canonical shared Libra
storage identity, so sibling linked worktrees are accepted while cross-repo
sources remain rejected. Import attempt markers are created in the reservation
transaction; live, export, and subagent writers use the same fail-closed
pre-object registration. The effective per-source read cap is
`min(agent.max_transcript_read_bytes, 16 MiB)`; explicit larger settings emit
the actual effective value. Discovery and held-descriptor read helper phases
run in private, kill-on-timeout processes under the command's absolute
deadline; the reader receives the already-pinned descriptor and no locator.
The command process still synchronously classifies, canonicalizes, opens, and
rewinds a source before that handoff. Thus these are helper-phase bounds, not a
strict end-to-end wall-clock deadline when NFS/FUSE stalls. A strict deadline,
FD-only wire, and autonomous timeout replay together require a long-lived owner
or a provider ABI that supplies a pre-authorized descriptor; that remains the
unresolved blocker in [`plan-20260924.md`](../plan/plan-20260924.md). Provider roots are
opened component-by-component; Claude sources and each nested Codex date
directory are opened relative to pinned no-follow descriptors before consent. Each new OID is added
as a durable provisional preclaim before its loose-object write, but becomes
deletion-eligible only after this writer wins publication and records it in
`created_oids`; a crash between those steps leaks safely instead of claiming a
concurrent writer's object. The object is compressed to a unique file in the
shared private `objects/info/libra-tmp` directory and promoted without overwrite,
with any existing final object fully validated before reuse. Fsync is conditional
on `--sync-data`/`LIBRA_SYNC_DATA`. A 64-entry bounded, 24-hour scavenger removes
only exact `.<40-or-64-lowercase-hex-oid>.tmp-<decimal-pid>-<uuid>` regular files and retains
unrelated entries. Expired construction attempts and `cleanup_pending` jobs are
retired by explicit `agent doctor --repair`/GC maintenance, not append or erase;
cleanup-pending ownership ignores the ordinary writer TTL, blocks same-session
erasure until retired, and is immediately repairable by doctor. Malformed
markers are surfaced as manual-required rather than silently skipped.
Each marker has a random writer generation, and every ownership mutation,
final ref CAS, and clear operation compares it exactly so same-checkpoint
takeover cannot be confused with the expired writer.
Rejected-object diagnostic reachability covers loose, packed, and alternate
objects under a 64 MiB per-object load-cost cap, full OID verification, a
250,000-object traversal cap, and a 30-second per-read/total deadline. Its roots
include refs, reflogs, registered worktree indexes, and sequencer state; roots
are snapshotted outside the writer transaction and revalidated before ownership retirement.
Index snapshotting runs in a killable helper under the same aggregate deadline;
no-follow/nonblocking regular-file opens, held-descriptor `limit + 1` reads, and
checksum/parsing of those exact bytes close special-file, growth, and path-reopen races.
Index enumeration is capped at 256 files/64 MiB aggregate. Any refusal preserves
durable cleanup ownership and fails closed. Inline recovery never unlinks a
shared loose object or deletes its object-index row; physical reclamation is
delegated to repository GC because index writers do not share the SQLite lock.
Rejected-writer ownership registration is O(1), durable, and blocks empty-catalog erasure until
the ownership job is safely retired. Persisted provisional ownership,
not process-local "created" state, controls zero-progress session reaping.
Retention GC physically deletes terminal ownerless import identities once no
coverage claim remains, including zero-checkpoint rows. Dry-run obtains the
same identity count by simulating coverage removal in a rolled-back transaction.

## MCP server (plan-20261019 MC-01)

`src/internal/ai/mcp_server/` is a second front end beside the Harness bridge (ADR-MC-02). It does not change the bridge allowlist, protocol version, limits, error catalog, NDJSON transport, or `agent_bridge_*` tables. MCP protocol versions, JSON-RPC codes, the 256 KiB line cap, and the tool registry live only in `mcp_server` (ADR-MC-03, ADR-MC-08).

`libra agent mcp --stdio` speaks legacy MCP `2025-11-25` and `2025-06-18`: `initialize`, `notifications/initialized`, `ping`, `tools/list`, and `tools/call`. An unknown `protocolVersion` falls back to `2025-11-25`. Batches are rejected. Unknown tools are protocol errors. Arguments that fail a tool's input schema are tool error `LBR-AGENT-027`. stdout is JSON-RPC only.

Production tool calls are translated into an internal `BridgeRequest` built by the server. Client parameters enter only allowlisted `params` fields; the client does not choose repository scope. MC-01 registers no production tool. The debug-only `libra_test_echo` tool exists when `LIBRA_TEST=1` so the tool-error path and `outputSchema` fixture can be checked. Each tool's `outputSchema` is `{"type":"object","anyOf":[<success>, TOOL_ERROR_SCHEMA]}`.

Tool results pass through `mcp_server/output.rs` before they are written (MC-16, GC-MC-03). The envelope scrubs control characters in `structuredContent`, serializes that value, runs `review::redact_untrusted`, parses it again, and scrubs once more. The first scrub keeps a JSON `\u` escape from gluing to a following secret. It then rebuilds `content[0].text` from the scrubbed value so that copy cannot keep a pre-scrub encoding. A failure is tool error `LBR-AGENT-034` with no raw fallback. The same envelope measures the full JSON-RPC response, including both copies, and returns tool error `LBR-AGENT-028` above 256 KiB (MC-19). `tools/call` waits at most 30 s for a read tool (`output.rs` `READ_DEADLINE`); a deadline is tool error `LBR-AGENT-036` with `retryable: true`. Debug builds with `LIBRA_TEST=1` also register `libra_test_blob` and `libra_test_sleep` for those gates. Read tools run in `libra agent mcp --read-worker` (MC-29), at most four at a time. The parent kills the worker process group with `SIGKILL` (Windows: `TerminateProcess`) and waits for it to exit before releasing the slot. The worker does a read-only hash-kind preflight and does not migrate schema, repair the object index, or start an upgrade check.

Each MCP request emits one `agent.mcp.request` span (MC-22). `method` is the original name only when it is in the server's known set, otherwise `unknown`. `id_hash` is the first 16 hex characters of SHA-256 over the canonical JSON encoding of the JSON-RPC id; the raw id is not recorded. `repository_id` is `libra.repoid`. A `tools/call` span also has `tool`, the registered tool name or `unknown`. `clientInfo`, `params`, and result content are not span fields. This is not the bridge span, which records the raw method and id.

## Bridge (plan-20260818 LB-01)

`libra agent bridge --stdio` is the repository-scoped DeepSeek Harness ingress.

- **Transport:** JSON-RPC 2.0 over newline-delimited frames on stdin/stdout.
  stdout carries exactly one protocol frame per response; diagnostics go to
  stderr (GC-LB-04). The command requires `--stdio` and has no other transport.
- **Protocol authority:** the frame/method/limit/error contract lives only in
  `src/internal/ai/agent_bridge/{protocol,transport}.rs` (GC-LB-02). The
  TypeScript plugin consumes the fixture generated from it; it must not define
  a second schema. Protocol v1: 20-method allowlist, 256 KiB frame cap, 64
  in-flight requests, 64-event/256 KiB batch, 30 s default deadline.
- **Scope:** repository/worktree/workspace/actor scope is derived from the
  trusted context at handshake (GC-LB-06/07); self-reported identity is never
  a credential. `deepseek-harness` is NOT an `AgentKind` (ADR-LB-02).
- **Non-goals:** not `libra code --control stdio`; a plain JSON-RPC 2.0
  NDJSON bridge, not a tool-serving protocol (ADR-LB-01). Implemented: the CLI + protocol + transport (LB-01), the durable
  session/event/operation storage (LB-02), the session/event ingress (LB-03),
  the typed read methods (LB-04), mutation admission/approval/actor binding
  (LB-05) and workspace lease claim/renew/release over `WorkspaceStore`
  (LB-06). As of `v0.21.1` all 20 v1 methods are implemented:
  `diff.get`/`commit.create`/`review.run`/`checkpoint.restore` reach the real
  services through the typed `agent_bridge/vcs.rs` adapter instead of failing
  closed behind the admission/approval gate.
- **Preflight:** the bridge uses the standard repository preflight (it is
  repo-scoped). It must never start without `--stdio`.
- **Publish lock:** `commit.create` takes `MaintenanceLock::shared` itself, for
  the span of that one commit. `command_holds_shared_maintenance_lock`
  (`src/cli.rs`) excludes the whole `agent` surface because an agent's VCS
  mutations are supposed to spawn `libra` as a subprocess and let the child
  hold the lock — which stopped being true once LB-05 called `run_commit`
  in-process. The hold is per mutation, never for the session lifetime (§C.10),
  so a bridge session cannot starve a deletion phase while still being ordered
  against one (§C.4.3 writer-vs-deleter). Regression:
  `agent_bridge_vcs_test::commit_create_waits_for_a_deletion_phase_before_publishing`.

## FIX-RG-SCOPED-04 confined checkpoint-input API

`checkpoint_input.rs` owns `open_scoped_run_root`, `read_run_metadata`, and `cleanup_checkpoint_input`. The helper reads `state.json` and `manifest.json` as regular no-follow leaves (64 MiB each, 128 MiB together) and clears `checkpoint-input` under a trusted absolute runs root while the directory namespace remains stable. Budgets are 8192 entries, 4096 files, 4096 directories, depth 64, 4096 path bytes, and 8 MiB total path bytes. It does not decide run kind or terminal state.

On Unix, held descriptors pin objects, not their current ancestry. The helper rechecks identities but cannot atomically bind those checks to chmod/unlink. A concurrent directory move in that last gap can affect an object after it leaves the trusted root; detecting the move afterward does not undo the effect. This unresolved guarantee and the complete ownership/exit protocol are outside this plan as `DEFER-RG-SCOPED-04`, by the user's 2026-10-10 decision. Callers must keep the namespace stable; the API does not enforce that precondition. This qualification also applies to scoped resume and store cleanup consumers.

The original `fix_rg_scoped_04_preflight_budget` combined regression remains present and ignored/UNRUN. The active `fix_rg_scoped_04_preflight_limits` retains its size-boundary and detectable input-change assertions, without running the deferred relocation cases. Neither its PASS nor a default full-suite PASS qualifies concurrent relocation. Windows execution is the `compat-scoped-input-windows` job on the actual main push SHA or verified PR merge/head. Its four active fixtures must be present, nonignored and actually pass; the original deferred fixture must remain listed as ignored. A Windows release build is not that evidence. Linux operator review stays UNRUN.

## FIX-RG-SCOPED-02 scoped resume caller

`materialize_validated_checkpoint_input` is the production entry for investigate start/continue and review scoped setup. It passes the absolute storage path, saved spec and actual caller budget to the shared checkpoint reader. The dedicated read-only catalog pool closes before a joined blocking owner validates the exact manifest/tree roles, loads typed ordinary blobs, invokes SCOPED-04 cleanup and writes ordinary files. Legacy full-list specs may name closed `reasoning/encrypted/<64 hex>` entries only as exact manifest/tree artifact tuples; those entries are excluded without opening their payload. Wrapped-v1 checkpoints without a manifest remain supported. Unknown, altered, duplicate or aliased entries, an id that is not a blob, and a blob content-hash mismatch are refused before cleanup. Ordinary files retain the 64 MiB per-file and 256 MiB total caps. The shared traversal separately bounds ordinary leaf trees to 16 MiB and ordinary child trees to a charged 16 MiB aggregate; the combined 32 MiB does not include wrapper, manifest or reasoning-tree proofs, which have their own bounded reads. The query phase uses the smaller of the caller's absolute deadline and its initial 200 ms phase budget, with awaited pool closure before filesystem handoff. Investigate samples its persisted remaining run budget once, before deriving its absolute deadline. Review setup starts immediately before validation and uses `phase_started + request.reviewer_timeout`; later reviewers retain their full original timeout. Both callers use their actual cancellation handle and existing run cancel marker. A paused continue with `CatalogUnavailable` returns the original store error before changing state or input; deterministic proof failures fail closed. The filesystem owner is cooperatively bounded and joined on cancel/deadline. SCOPED-04's documented concurrent namespace relocation limitation remains `DEFER-RG-SCOPED-04`; this card does not strengthen that guarantee. No schema, serialized field or stable error code changes.

## Reasoning type contract (RG-01)

The provider-neutral type contract defines `provider_visible`,
`encrypted_unavailable`, `opaque_archived`, `not_present`, and
`unsupported_shape`. `encrypted_unavailable` is not a capture failure: it
means a provider-declared encrypted field lacks an authorized decryptor.
Unknown reasoning-like source shapes produce a payload-free warning; ordinary
unrecognized high-entropy or base64 content remains `not_present`. A block
selected for encrypted-field verification must itself be a declared reasoning
block: selected text/tool blocks fail as `unsupported_shape`, even if they
carry an undeclared signature-like key. Adapters classify ordinary text/tool
content before calling the encrypted-field verifier.
`ReasoningRecord` contains safe enum metadata only. The reasoning module
validates the record, block type and JSON-string field type before producing
an in-memory `OpaqueEncryptedBytes` for authorized Claude assistant
`thinking.signature` / `redacted_thinking.data` sources. Escaped UTF-8 wire
bytes are not JSON re-serialized or base64-decoded. A matching JSON shape
is not a provider credential; source authorization precedes verification.
OpenCode 2.0.24 `reasoning.state` is open-ended and is not a declared
ciphertext field. `OpaqueEncryptedBytes` cannot be serialized, displayed, or
converted to `RedactedBytes`; this type contract does not yet capture or store
encrypted reasoning. RG-04 owns readable reasoning projection and RG-02/03/05
own artifact storage, export, mirroring, and erasure. No live provider adapter
is connected by RG-01.

This type-contract verifier accepts only the minimal synthetic envelope; real session records and additional metadata are deferred to the reviewed provider adapter contract.

## Readable reasoning projection (RG-04)

RG-04 adds the readable-reasoning projection path:
`ProviderVisible` reasoning text is the only reasoning content that can enter
the coverage projection, and it always passes typed redaction first; canary
secrets inside reasoning text are counted in the redaction report and removed
before projection, checkpoint persistence, or any metadata/digest write.
Readable reasoning projects under its own `reasoning` record type with
`provider` and `source_kind` metadata; it never impersonates an assistant
answer and is never written to unredacted metadata or logs.

Direct serde serialization/deserialization of `ProviderVisibleText` is deliberately rejected. Callers must use classification and typed redaction followed by `canonical_turn_bytes` or `safe_turn_projection`. RG-04 does not wire a live provider adapter.


RG-02 adds the artifact archive: verified ciphertext is stored byte-exact under
`reasoning/encrypted/<sha256>` and declared in the manifest's `reasoning_artifacts[]`
array; dedup by sha256, duplicate-locator rejection, and 512/256 KiB/32 MiB
fan-out budgets are all fail-closed. `content_hash` coverage stays four-role.

Opaque artifact writes alone use standard zlib stored blocks (level 0).
Canonical blob bytes and Git OIDs are unchanged. High-entropy ciphertext uses
more disk and mirror bytes; ordinary objects keep default compression, and
valid existing objects are reused without rewriting.

### Controlled opaque reasoning export

Libra preserves provider-encrypted bytes; it neither owns nor guesses decryption keys. Readable reasoning text passes typed redaction; a provider signature authenticates provider output, redactedData represents a provider-redacted reasoning block, and reasoningEncryptedContent is opaque provider ciphertext. None of these metadata labels alone proves native origin. The existing native source requirement remains in force.

`checkpoint show` keeps its fixed human-readable safe summary. `checkpoint show --json` adds `reasoning_artifacts[]` only when a checkpoint declares artifacts. Each item exposes only `provider`, `source_kind`, `locator`, `byte_len`, `sha256`, `availability`, and `decrypt_capability`; it never exposes artifact paths, object ids, arbitrary metadata or bytes. Default show, transcript export, scoped input and skill readers do not read artifact bodies. An invalid checkpoint layout or manifest fails closed without reflecting its contents. A valid catalog root whose object is not yet available keeps the legacy safe summary, omits the optional artifact array and emits a fixed content-free diagnostic; present corrupt roots still refuse. This compatibility fallback never authorizes export.

```sh
libra agent checkpoint show <checkpoint-id> --json
libra agent checkpoint export <checkpoint-id> \
  --reasoning-artifact <lowercase-sha256> \
  --allow-raw --justification "operator recovery" -o <new-private-path>
```

Opaque export requires all three authorization/output options, a lowercase SHA-256 and a new destination. Do not combine it with `--raw`. Authorization is checked and an independent `reasoning_artifact_export` audit row is appended before checkpoint lookup, artifact body access or file creation. A grant records the authorized attempt; later lookup or file failure retains that audit row. Audit failure stops the operation before body access.

The selected artifact must belong to the selected checkpoint. Export verifies the closed manifest/tree role, complete blob object identity/type, SHA-256 and any declared byte length. The maximum is 32 MiB; truncated prefixes are refused. Bytes bypass redaction and are never decoded, sent to stdout, used as model input or decrypted. macOS/Unix writes a 0600 temporary file beside the destination and publishes it without overwriting an existing file, symlink or directory. Missing parents are not created; failed writes remove the private temporary file. Platforms without this owner-only permission primitive refuse with `LBR-AGENT-013` and a denial audit. Windows private ACL export is deferred; default metadata and ordinary transcript readers remain available.

The `ReasoningDecryptor` registry returns `None` for Codex, Claude Code and OpenCode. `decrypt_capability` stays `none` and encrypted bytes remain `encrypted_unavailable`; a future official, explicitly authorized provider capability must enforce checkpoint/provider scope. This release implements no decryptor and accesses no decryption keys.

Implementation owners: the CLI selector, fixed audit action, metadata-only helper, separately audited complete-body verifier, private no-clobber publication and provider-scoped empty decryptor registry. Shared role/proof budgets and existing body-read observer are reused; no SQL migration or key/config/environment/network discovery.

RG-06 adds artifact tree reachability; RG-03 adds controlled read and export.


## Shared checkpoint input proof (FIX-RG-SCOPED-01)

`internal::ai::checkpoint_reader` owns the typed header and content-addressed identity check, ordinary-role traversal, and manifest/tree closure. Command readers use this owner; the fresh scoped resolver queries `storage.join(util::DATABASE)` through a dedicated SQLx readonly connection with creation disabled. The query phase has one absolute deadline capped at 200 ms and the caller's remaining budget; it does not initialize schema or reuse the cwd-selected writer pool. Filesystem work receives authority only after the query lease and dedicated pool have closed. Failures refuse the input and cannot authorize materialization.

SQLite contention is refused immediately (`busy_timeout(0)`). Acquired connection and pool closure are always awaited; an authority that finishes after the deadline is refused. The 200 ms phase deadline does not promise a 200 ms bound on awaited cleanup or that a timed-out lazy connection establishment thread has already exited. Default reader budget refusals keep their existing store-inconsistent message; fresh scoped layout, integrity and payload-cap failures retain the checkpoint-specific fatal context and inferred error code.

Fresh scoped traversal-limit refusals keep the existing store-inconsistent message and `LBR-AGENT-009`, while an empty wrapped leaf keeps its original scoped fatal message. Shared role proof returns its complete map; both fresh and saved consumers refuse an empty map before granting materialization authority.

The saved-spec proof API reserves at most 4608 saved entries, 4096 UTF-8 bytes per path and 8431616 cumulative path bytes before path normalization or map allocation. It then requires the complete ordinary path-to-OID map of the named wrapped checkpoint. Ordinary files retain their separate 4096-file and 8 MiB path budgets. A legacy full list may exclude at most 512 entries only when each exact 84-byte `reasoning/encrypted/<sha256>` path and OID agrees with both the manifest and artifact tree. Prefixes, OIDs alone and ordinary subsets are insufficient. Proof metadata reads are recorded separately from ordinary-body reads, and proof never reads artifact bodies.

This card supplies the shared internal API. Persisted review/investigate recovery is connected by FIX-RG-SCOPED-02, and filesystem ownership is supplied by FIX-RG-SCOPED-04. Their acceptance and the final aggregation full suite remain separate requirements. Synthetic role fixtures do not establish a native producer origin or authorize OpenCode reasoning archival.

The ordinary `checkpoint show`/`list` summary remains a closed whitelist. RG-03 enables the optional artifact metadata projection for JSON show using bounded role proofs, without reading artifact bodies; human show and list retain their existing summaries. Skill projection now passes checkpoint id, tree oid and metadata oid to the shared role proof; corrupt or unbound metadata is skipped without exposing its body. Raw export authorization and audit are unchanged.

The macOS local content gate is `local_capture_smoke_opencode_macos` in
`agent_local_capture_smoke_test`. It requires an explicit local-agent opt-in,
pins OpenCode 2.0.26, selects `volcengine-agent-plan/glm-5.3-flash`, and checks
the disposable project before requesting a single fixed file write. The gate
checks captured prompt/model/tokens/files against the source projection,
persisted file metadata, stopped sessions, doctor and uninstall retention.
A failed or timed-out preflight is a failure, with no model request started.
The historical 2.0.26 run failed server readiness and remains recorded as
`DEFER-OG-OPENCODE-BOOT-01`, not a passing content gate. Current 2.0.29 ordinary
automatic-capture evidence has its own source/version boundaries in the plan;
it does not retroactively pass that old smoke or prove native reasoning.

### Scoped run cleanup consumers

FIX-RG-SCOPED-03 consumes the immutable checkpoint_input bounded metadata and cleanup APIs in both stores, at clean_run and direct clean_all. Type/schema/run-id/terminal proof happens before input mutation; parseable foreign, nonterminal or incomplete state vetoes manifest fallback. Only explicit corrupt/missing-state cleanup may use a valid own terminal manifest. Bulk skips corrupt/unknown/foreign/nonterminal candidates and counts actual removals. Investigate CLI retains its existing lock; library bulk acquires a per-run lock, while review makes no OS-lock claim. Each input cleanup uses one cooperative five-second deadline. Each of two fixed metadata snapshots is bounded to 64 MiB/file and 128 MiB/pair (at most 256 MiB read work); initial bodies are dropped before rereading, identities must match before deleting the run. Input/root metadata no-follow checks remain solely in the delivered helper. Preflight limits fail before chmod; I/O/deadline or changed metadata after cleanup can leave partial input, return an error and never count a removal. Whole-run filesystem deletion retains the existing remove_dir_all behavior; synchronous filesystem calls are not forcefully preempted. Namespace concurrency remains the user-deferred limitation, no new ownership guarantee. No schema, target, dependency or serial-lane changes.

A proved own-kind terminal state remains cleanable when its matching manifest has no terminal marker after an interrupted state-first finalization. Identity/schema/common-field conflicts and differing non-null terminal values remain refused. Bulk qualification I/O, link or size refusals return an error; already-removed earlier runs are not rolled back. A proved own-kind terminal state also remains cleanable when its manifest is missing. A terminal review infrastructure failure may have no reviewer outcome rows while its manifest retains the requested reviewers; that supported shape is cleanable. An unparseable manifest is still refused even beside a proved terminal state: preserve the run and repair its metadata before retrying.


### RG-05 reasoning artifact lifecycle

Cloud publication validates manifest role, canonical tree, `object_index`, decoded `byte_len` and full SHA-256 before beginning a capture generation; existing upload repair and bounded artifact readback preserve original bytes. Exact checkpoint-tree and artifact-index evidence is rechecked against the final fenced generation. The registered object helper carries metadata evidence only and shares the caller's absolute deadline.

Session erasure hides artifact references; `agent clean --gc` and repository GC reclaim unreachable local objects. Doctor emits the read-only `inconsistent_reasoning_artifact` finding with canonical `checkpoint_id`, `manual_required=true`, and no persistent status column. Physical R2 payload deletion remains deferred; tombstones remove mirror catalog rows and fence normal restore. Non-quiet `agent enable` and `agent import --yes` disclose archival of provider-encrypted reasoning original bytes and checkpoint cloud mirroring without decryption. Synthetic fixtures remain `encrypted_unavailable` while visible, with no references after erasure; native source proof and the unresolved OpenCode Source A gate remain unchanged.


### Authorized reasoning source carrier (OG-16)

The selected registered adapter has an optional pre-redaction source preprocessor, disabled by default. It separates only an already authorized, bounded in-memory source; it cannot reopen provider files or access the checkpoint store. With no capability or no candidate, ordinary snapshot, live and store bytes retain their existing behavior. When enabled, candidates are removed before the ordinary Redactor. Unproved or unmapped candidates retain a payload-free partial classification, including a partial with zero artifacts; they cannot be converted to ordinary complete content.

A sealed native carrier moves original opaque bytes and their source-bound proof together into the shared checkpoint payload. Native issuer, provider, source session, message, part, field, wire SHA-256 and authorized capture/turn association are validated before object or index writes. The metadata-only identity adapter cannot expose payload bytes or mint native authority. Debug output does not contain native coordinates or payload. Source, artifact-byte and artifact-count limits remain enforced.

The authenticated private import transport carries a native-proven typed collection from the authorized descriptor helper to the parent. A one-request key binds the helper response to the request, authorized source, repository scope and collection; the parent verifies that binding and retains the complete native proof before assigning owned items to reserved turns and the shared checkpoint store. An unauthenticated or unproved nonempty collection fails closed, without treating a wire DTO as native authority or returning candidate bytes to the ordinary transcript. Empty preparation keeps its existing response and source projection bytes. Pending v1 still refuses nonordinary payloads before writing an envelope; pending v2 accepts only a validated borrowed native carrier and its prepared identity plan under the existing receipt, fence and lease rules. For an enabled Claude deadline-FD route, preparation is rejected before source resolution, without an ordinary fallback. An existing Pending receipt prevents the wrapper failure diagnostic; NotTracked retains only its existing bounded diagnostic attempt and does not advance sync_revision.

Production provider capabilities remain disabled and no production native issuer is admitted. Synthetic positive fixtures prove the carrier and import transport mechanism only. Source A remains UNPROVEN and real native field admission is outside this plan’s final scope; unproved OpenCode fields are not archived. Durable reasoning replay still requires its later gate. This mechanism adds no decryption capability or public metadata enum value.


### Pending reasoning v2 codec (OG-17)

The private pending codec pairs its closed inner payload v2 with the live SessionEnd writer and authenticated reader. When an admitted producer supplies a bound native-proven collection, the writer seals its source record and explicit per-turn `verified_nonempty`/`verified_empty` identities. Ordinary pending payloads remain byte-for-byte v1; the outer header and envelope remain v1 with the existing MAC domain, 64 MiB envelope, 128 chunks and 512 KiB-per-chunk limits. A pre-v2 reader rejects v2 and retains its evidence. The current reader checks the original whole-envelope MAC, canonical member/claim association and current catalog, claim and marker fences before the lower reasoning issuer validates the fixed source proof. It restores only reserved-turn artifacts and commits their original stable identities with the checkpoint, claim and ref transaction. A failed append retains the pending artifact and claims; each retry reauthenticates the original bytes and current fences without reopening a provider source or minting a replacement key. Missing or damaged private-key/source evidence requires diagnosis and manual recovery. A decoded DTO or envelope MAC alone never proves native provider origin: unproved fields remain partial and are not archived. Production native issuers and real OpenCode Source A evidence remain disabled; synthetic tests exercise this transport without claiming real native archival.

### Private reasoning identity and garbage collection

Garbage collection excludes the reserved `agent_reasoning_identity` scope from generic metadata object-reference scanning. This applies to the entire stored value, including a damaged value containing a JSON object ID or a nested object ID; those values do not trigger object hydration. This exclusion requires no private reasoning key. Ordinary metadata object roots, private capture recovery exclusions, and capture pending parent roots retain their existing rules. Forward recovery must retain both the private rows and this exclusion while a reasoning identity writer is stopped or repaired. This guard does not create identity records, issue native provenance, or enable OpenCode reasoning archival.

<!-- Compatibility anchor for earlier links; not a release-status assertion. -->
<a id="versioned-graphdoctor-readers-cov2-09-unreleased"></a>

## Versioned graph/doctor readers (COV2-09, Libra 0.30.50)

The graph reader decodes each selected claim and revision through the existing
closed `CoverageSchemaVersion` enum before joining or skipping unmatched rows.
Both revision history and current claims use `(logical_turn_key, coverage_schema_version)`;
v1 and v2 rows sharing a key/checkpoint remain distinct. The public graph JSON
schema remains 1. Unknown stored versions return the existing
`LBR-AGENT-009` error with upgrade/doctor guidance, without an indexed-success
response or candidate content. No version coercion, catalog mutation or blob
read is added. An unmatched revision is still validated before it can be ignored.

Doctor already joins conflicts by both key and version; its production code is
unchanged. Each conflict remains a distinct content-free, manual-required finding
in JSON and human output, with or without `--repair`. Synthetic catalog fixtures
check version separation, envelope stability, unknown-version failure and raw
identity/content canaries. They do not run an OpenCode Session, exporter or model,
and confer no native-origin proof. These reader guarantees require Libra 0.30.50 or later; upgrading the installed
binary is necessary before relying on them.

<!-- Compatibility anchor for earlier links; not a release-status assertion. -->
<a id="coverage-v2-capture-unreleased"></a>

## Coverage-v2 capture (Libra 0.30.50)

From the first stable release containing coverage-v2, OpenCode export capture
and historical import retain supported provider-visible reasoning as typed,
redacted turn content. Turns containing that content use coverage schema 2;
ordinary turns keep schema 1 and their existing canonical bytes. Redaction runs
before either digest or checkpoint projection. This does not authenticate
`signature`, `redactedData`, or `reasoningEncryptedContent`: unproven native
fields remain partial and cannot be archived as trusted artifacts.

After upgrading, an interrupted action whose checkpoint was already committed
with v1 (or an empty legacy revision set) first settles that existing receipt
without adding claims or rewriting its checkpoint. A subsequent **new action**
may capture the visible reasoning once under v2; repeated capture then deduplicates.
The old v1 objects, digests, and claims remain readable and unchanged.

If source classification is rejected after that session already has v2 claims,
the entire new capture/import attempt stops, including new ordinary turns.
Its pending receipt/export work stays retryable; no fallback claims or success
acknowledgment are created. An unchanged over-limit or unsupported source can
keep that session blocked. Keep the recorded data, inspect `libra agent doctor`,
and start a new OpenCode session to continue supported capture. A candidate or
unverified reasoning source cannot turn this failure into complete coverage.
These changes add no command flags and require Libra 0.30.50 or later. OpenCode Session/exporter/model acceptance runs locally on
macOS; it is not a CI integration test.
