# Coverage schema v2 — typed reasoning encoding boundary

This document covers the codec introduced by COV2-01 and the unreleased
COV2-11 live/export and historical-import wiring. Stable availability and the
upgrade boundary are described under Coverage-v2 capture below.
The frozen v1 contract and its three byte/digest vectors remain in
[coverage-v1.md](coverage-v1.md). A stored claim's `coverage_schema_version`
is separate from the public `agent graph` and `agent list` envelope versions.

## Selection and validation

After the provider has produced typed `NormalizedTurn` records and typed-field
redaction has completed, the selector chooses coverage version 2 if any record
is `SemanticRecord::Reasoning`; otherwise it chooses version 1. A source key
named `reasoning`, `signature`, `redactedData`, or
`reasoningEncryptedContent` is not by itself a typed record or native-origin
proof. Unsupported versions are rejected before encoding. An explicit v1
request containing typed Reasoning is rejected rather than downgraded or
silently omitted. Readers may explicitly request version 2 for a turn without
Reasoning, but the automatic writer still selects v1 for that turn.

The new `encode_coverage_turn` and `encode_coverage_turn_for_schema` APIs
return the selected claim version, canonical bytes, and SHA-256 digest as one
unit. `safe_turn_projection_for_schema` uses the same closed version check
for allowlist-only JSON projections. Existing `NormalizedTurn::digest_hex`,
`canonical_turn_bytes`, and `safe_turn_projection` retain their legacy
behavior for frozen v1/RG-04 oracles. COV2-11 connects actual authorized
live/export and import turns to the versioned gate together. This codec does not change the database,
claim transactions, pending replay, graph, doctor, or public CLI defaults.

## Canonical bytes

Version 2 retains v1's ordered JSON-array envelope, UTF-8, minimal string
escaping, recursively sorted keys, integer-only semantic values, and
SHA-256 lowercase-hex digest. The four v1 record shapes are unchanged. Its
additional provider-visible Reasoning record has exactly these four keys in
canonical order:

```json
{"provider":"opencode","role":"reasoning","source_kind":"reasoning","text":"synthetic"}
```

`source_kind` may be `null`; no native proof, state, model, usage, path, or
other provenance field enters canonical bytes. A synthetic vector with user
`hi`, that Reasoning record, then assistant `hello` is 153 UTF-8 bytes and
has digest
`be52311c1b5daf77926d352c3b44eb93bb4fd57b78423774b3d0ac7ecbccc586`.
The independent fixture is
`tests/fixtures/agent_transcripts/reasoning/coverage_v2_golden.json`.
COV2-13 independently pins all three historical v1 byte and digest
vectors against the new versioned encoder. It also checks the original
`contract.snap` file's SHA-256 before the existing RG-04 legacy projection
test compares the complete JSON value. These checks are compatibility oracles;
they do not switch production writers to v2.

The RG-04 `contract.snap` remains unchanged and continues to exercise the
legacy projection; its synthetic Reasoning example is not a production v1
claim.

The codec assumes its input was redacted before encoding. COV2-10/11 own
proof that every production typed path enforces that order and that unproved
state/proof-bearing candidates stay partial with zero ordinary or typed
persistence. Version 2 does not confer OpenCode native issuer authority or
permit opaque artifact archival. Source A remains a separate hard gate.

## Claim arbitration preparation

COV2-02 adds a version-bound reservation entry point for local claim DB
arbitration. It uses the existing unique key
`(session_id, logical_turn_key, coverage_schema_version)`: equivalent rows
within one version are idempotent, an incomplete row can advance to complete,
and a different complete digest becomes a conflict in that version only. An
unsupported version or a version that disagrees with the typed turn selector
is rejected before any reservation write. The older reservation entry points
remain v1 and reject a typed Reasoning turn rather than claiming it as v1.

`ReservedTurnClaim` and `LiveClaimCommitPlan` keep their old field layouts.
A separate `VersionedReservedTurnClaim` carries each claim's version, and
`VersionedLiveClaimCommitPlan` binds an ordered collection of those claims to
the unchanged commit-plan literal. Its uniqueness check uses `(version,key)`,
so a pending artifact can later hold one claim for each version of the same
logical key without guessing from the key. COV2-03 binds version-aware marker
and checkpoint transactions; COV2-19/20/21 add the native companion and
versioned reservation/retain steps, COV2-04 adds the bounded pending codec,
COV2-24 adds in-memory sealing, COV2-22 adds persistence, and COV2-11
switches production live/import paths together.
This preparatory arbitration does not itself enable OpenCode Session capture
or native archival.

## Private identity authentication preparation

COV2-07 preserves the complete identity-v1 slot, descriptor, record and
whole-map bytes and MAC answers in a fixed-key local oracle. Existing v1
callers keep the same canonical body version, `reasoning-identity/hmac-v1/`
descriptor purpose and `reasoning-record/hmac-v1/` record/whole-map purpose.
The repository-private 32-byte key remains at its existing path; verification
never creates or rotates it.

Explicit v2 helpers choose the schema version before borrowing that key. The
v2 descriptor, record and whole-map each have a distinct visible prefix and
HMAC byte-domain tag; replacing a prefix cannot transplant a tag across
versions or purposes. Unknown versions and missing keys fail closed. These
are authentication primitives only. COV2-14 binds each v2 MAC to a closed
v2 record/map body and enforces the shared map capacity before persistence;
COV2-03 binds marker/ref fences; COV2-19/21 add native v2 identity writes,
COV2-04 adds the pending codec, COV2-24 adds in-memory sealing, COV2-22
adds persistence, and COV2-11 switches live/import.
A valid private MAC does not prove provider-native origin or authorize opaque
archival.

## Private identity map carrier (COV2-14)

A Session may retain `identity-v1` and `identity-v2` side by side in the
local `agent_reasoning_identity` metadata scope. The existing v1 envelope,
record bytes, key, and MAC purpose are unchanged. The v2 codec has its own
closed envelope and independent descriptor, record, and whole-map MAC
purposes. A reader enumerates the complete bounded private key set, admits
only those two keys, bounds the declared type before loading it, checks both
stored text types and their combined encoded length before loading values,
then authenticates both maps. The two maps share
a ceiling of 4096 records and 1 MiB of stored envelope bytes. A damaged or
unknown private map fails the entire reservation batch before any claim-state write.

The existing v1 claim companion updates only `identity-v1` and preserves the
stored `identity-v2` value byte-for-byte, while applying the combined capacity
check before its claim/catalog mutation. COV2-03 binds the versioned marker
and ref transaction; COV2-19/21 own the sealed native v2 identity/retain
producer, and COV2-11 owns the production Session switch. The COV2-14 v2
carrier tests use synthetic signed records and do not establish OpenCode
native provenance. An older binary's
v1-only reader may reject a Session containing both keys; upgrading the
binary restores access, with no silent v2-to-v1 downgrade.

## Versioned checkpoint marker and replay fence (COV2-03)

A marker-bound checkpoint now carries an immutable per-claim tuple of coverage
version, logical turn key, digest, completeness, next revision, reservation
owner, fence token, and reservation state. The registration transaction checks
each tuple against the current claim row and its previous revision before it
can publish a successful marker. Duplicate `(version,key)` tuples, an invalid
version, or a stale owner/fence/revision reject the whole attempt. A checkpoint
containing v2 claims is bounded by 4096 tuples; historical pure-v1 checkpoints
above that count remain replayable. The checkpoint, live
persisted-generation, and recovery constructors retain the complete tuple;
the typed checkpoint request must bind the same versioned claim plan before a
v2 write. Ref/catalog/revision/claim changes remain in one transaction, so a
failed later claim leaves the earlier state unchanged.

A marker-bound replay of a committed checkpoint compares the complete
immutable `agent_coverage_revision` set with the expected tuples, including
when the expected set is empty. Missing, extra, or changed revisions reject
before `AlreadyExists`/`AlreadyCommitted`. If another writer commits a
revision between the optimistic replay probe and marker registration, the
registration returns a replay-payload conflict without publishing a marker or
ref. A pre-coverage v1 database with **both** coverage tables absent and no
coverage-or-later migration receipt can replay only an empty claim set;
nonempty expectations, a partial schema, or missing tables after the coverage
migration fail closed. This also applies to no-write settlement and completed
terminal receipts. A completed terminal receipt with an expected or durably committed v2
revision additionally requires its committed checkpoint in the same session
and scope and an exact revision set;
an empty or v1-only caller cannot downgrade that v2 receipt to an ACK. A
legacy no-write/prebound settlement without an authoritative set still accepts
a previously committed nonempty v1 revision set, but refuses a durable v2
revision. The frozen v1 terminal receipt without an action checkpoint remains
acknowledgeable. A rejected replay preserves any pre-existing repairable
cleanup marker for doctor/GC.

The persisted marker JSON remains `schema_version: 3`, independent of the
claim's coverage version. This step adds no SQL schema or public command
change. Pending v2 persistence/replay, OpenCode Session capture, native issuer
authority, and reasoning archival remain with later task cards and Source A.

## Sealed native v2 identity commit (COV2-19)

The private v2 identity plan is derived only from a `NativeProven` checkpoint
payload whose existing capture proof and binding match the request. Its typed
turns determine a canonical `(coverage version, logical key, digest,
completeness)` claim set; the submitted version sidecar must match exactly.
The concrete checkpoint store verifies those same claims against the marker
fences and registers the marker before it gives a private transaction adapter
to HistoryManager. The native companion itself cannot be passed as a low-level
`txn_extra`. The winning ref-CAS transaction authenticates retained maps,
checks claim ownership and the shared 4096-entry/1-MiB budget, then commits
sealed v1/v2 records and maps with catalog, revisions, claims, ref, and object
index. On a replay, matching committed revisions alone do not prove that this
native companion was committed: the store also authenticates the retained
private record and map for every bound version/key against the source's
checkpoint, revision, fence, kind, and descriptor. It applies this check to
the initial replay probe and to a concurrent writer winning marker registration.
If a later checkpoint has superseded the same version/key slot in the current
session map, an older replay returns a non-retryable replay-payload conflict:
the current map cannot prove its historical native identity. The earlier
checkpoint remains committed; operators can inspect it with `libra agent doctor`.
An invalid or unauthenticated private map instead reports a store failure and
logs only a fixed diagnostic stage with the doctor hint; it never becomes a
successful replay or exposes private bytes.
A failure leaves no successful checkpoint ACK. The frozen v1 writer and
v1 byte domains remain unchanged. This writer capability is internal;
OpenCode live/import activation, durable v2 pending replay, real provider
Source A proof, and native archival remain gated by later work. Synthetic
sealed fixtures exercise transaction behavior but confer no issuer authority.

## Versioned live-reservation verification (COV2-20)

An internal read-only helper accepts the sealed `VersionedLiveClaimCommitPlan`
and the caller's existing database connection or SQLite transaction. It reads
each actual `(session, coverage version, logical key)` claim row and requires
exact digest, completeness, owner, fence, previous revision and `reserved_live`
state. A still-owned expired lease remains valid; a changed ownership tuple
fails closed. Same-key v1/v2 reservations are checked independently. The
helper checks the current workspace scope, has a 4096-claim bound, and does not
update leases or open another connection. COV2-21 owns retain mutations and
COV2-22 owns durable pending-v2 persistence; this helper alone does not enable
pending replay or native-source admission.

## Caller-transaction reservation retention (COV2-21)

After the caller verifies that exact sealed versioned plan on its artifact
transaction, the retain helper updates each actual `(version, logical key)`
row in that same transaction. Its conditional UPDATE repeats digest,
completeness, `reserved_live` state, owner, fence and previous revision before
setting the lease to the durable-artifact sentinel. An expired lease alone
does not block retention. A stale tuple or SQL failure rejects the handoff;
the caller must roll back the whole transaction because an earlier row may
already have been updated. The helper neither opens nor commits a transaction.

The frozen v1 helper and claim bytes remain unchanged. COV2-04/24/22 own
bounded pending payloads, same-source sealing and real artifact persistence;
COV2-22 must roll back retained leases if a chunk, header or alias writer
fails. This internal retain seam does not establish provider Source A or permit
native reasoning archival.

## Same-source native pending borrow (COV2-23)

The internal pending preparation entry borrows `NativeProven` from the same
sealed checkpoint payload as its versioned companion. It checks the capture
scope and source binding, then requires the companion's complete canonical
`(version, logical key, digest, completeness)` claim set before looking up
identities by `(version, logical key)` in claim order. Empty or over-capacity
claims fail before allocating associations; an unassociated pending key fails
closed. Because the current pending association still uses a bare logical key,
native claims sharing that key across versions are rejected here. This does
not prohibit the separate versioned wire codec from representing such values.

This is a checked in-memory borrow only. COV2-04 encodes the bounded wire
format, COV2-24 seals an in-memory artifact from this borrow, and COV2-22
persists and rolls back real artifact writes. The borrow does not grant native
issuer authority, establish provider Source A, or enable OpenCode Session
archival.

## Bounded authenticated pending-v2 codec (COV2-04)

The private pending-v2 codec carries an explicit coverage schema version on
each claim and its ordered identity association. Its wire representation can
contain v1 and v2 entries with the same logical key; duplicate `(version, key)`
tuples and unknown versions are rejected. The producer bounds the entry count
before collecting claims and checks encoded tuple bytes before allocating the
outer envelope. The decoder accepts only an `AuthenticatedPendingEnvelope`
issued after exact-byte MAC verification, checks raw entry and byte limits
before constructing the typed projection, and then validates the expected
binding, ordered claim/identity association and canonical envelope bytes.
The outer-v2 encoding is separate from the frozen v1 pending envelope and MAC.

This is a pure, in-memory codec. An envelope MAC does not establish native
provider origin. COV2-23 still rejects native pending associations that reuse
a bare logical key across versions; COV2-24 consumes its checked same-source
borrow to seal an in-memory artifact, and COV2-22 owns real persistence and
frozen old-reader behavior. No durable v2 pending artifact, replay, OpenCode
Session admission or provider Source A is established by this card.

## Source-bound in-memory pending-v2 seal (COV2-24)

The private v2 sealer accepts one complete redacted payload, its sealed
source-derived versioned claim companion, and a matching capture binding and
prepared alias. It invokes the COV2-23 checked borrow from that same payload;
the versioned inner projection then pairs each ordered `(version, logical key)`
claim with the checked slot and descriptor MAC. A missing or mismatched pair
fails before a v2 artifact can be returned. The resulting outer-v2 bytes use
the COV2-04 bounded codec and the existing exact-byte pending MAC domain with
the historical header version. The v1 sealer and its bytes remain unchanged.

This operation produces only an authenticated artifact in memory. It does not
persist pending rows, retain a claim lease, move a ref, or acknowledge a
checkpoint. COV2-22 owns the transaction and frozen old-reader behavior;
COV2-08 owns replay. Synthetic tests of this seal do not prove OpenCode
provider Source A or permit native cross-version aliases with the same bare
logical key.

## Original pending-v2 member integrity (COV2-25)

The private key-owner verifier checks the original whole-envelope MAC with the
existing repository key, capture scope, storage binding, authorized root and
deadline before using the COV2-04 bounded authenticated decoder. It compares the caller's binding,
`payload.redacted`, optional reasoning source, ordered associations and coverage
with the exact original canonical member slices, including the decoder's
ordered `(coverage_schema_version, logical_turn_key)` tuples. After parsing it
checks the deadline again before issuing the local `PendingIntegrityToken`.
Missing keys fail closed; verification does not initialize or replace a key.
Unknown envelope/payload fields and duplicate JSON keys fail closed.

An omitted source uses the domain-separated digest of
`libra/pending-v2/no-reasoning-source\0`; explicitly present `null` is rejected
rather than treated as canonical omission. Source-present production-seal tests
and absent-source codec/key-owner tests exercise local integrity mechanisms;
neither supplies real provider Source A. COV2-22 owns its separate private
durable loader consumer; no production Session path is connected here. This
key-owner entry grants no native proof, artifact persistence or checkpoint ACK.
COV2-08 owns subsequent replay checks. The v1 member API,
serialization and pending MAC domain remain unchanged.


## Private pending-v2 transaction and deferred loader (COV2-22)

The closed versioned artifact's `persist(&DatabaseTransaction)` checks its
canonical sealed bytes, bounds, binding and ordered claim correspondence
before its first SQLite action. An all-v1 claim set and a native association
with the same bare logical key in two versions are rejected at this entry,
even though the normal native borrow already rejects the latter. Within the
one caller-owned writer transaction it verifies the exact versioned live
reservations, retains both claim versions, then writes bounded chunks, the
header and the authenticated private alias. The caller must roll back the
whole transaction on every error. Retention is not an ACK, and idempotent
delivery preserves retained quarantine/manual state and original chunks.

The separate versioned loader reads the committed original bytes and MAC.
After whole-envelope authentication and bounded decoding, the COV2-25 key
owner authenticates the exact original binding, nested `payload.redacted`,
source, ordered associations and coverage. Only then does the payload owner
rebuild redacted sidecars and verify their existing commitments. A source
must be present; the codec's valid absent-source integrity sentinel is not a
native pending-load capability. The loader checks its deadline again after
rehydration and after allocating the deferred result.

The non-Clone, non-Serde carrier retains the original bounded bytes/MAC,
source DTO, exact ordered `(coverage_schema_version, logical_turn_key)`
associations and integrity stamp. These remain local integrity data, not
NativeProven. COV2-08 must keep the deferred source attached and freshly
revalidate original bytes/MAC, current catalog/receipt/workspace/claim fences
and the lower fixed source proof on each replay attempt. No production
Session path, native restore, checkpoint/ref publication or success ACK is
connected by this card. The v1 wire, MAC domains and immutable old oracle
remain unchanged. Pure Rust storage/codec fixtures do not prove OpenCode
Source A; account/model capture stays outside CI.


### COV2-08 同版本 pending 恢复边界

外层 v2 恢复保留同一已提交 artifact 的原有序 `(coverage_schema_version, logical_turn_key)`、原 bytes/MAC 和 typed deferred source；不按 discovery header、裸 turn key 或错误文本猜测版本。V1 loader 的临时错误或总 deadline 到期直接稍后重试，不降为其它版成功。早期 redacted payload 的认证关联在 sidecar 被丢弃时仍阻止普通落库；完整 sidecar 也只代表本地完整性，不能生成 native 来源。现有 ordinary versioned direct writer 与 verified ordinary partial 的 direct-only 规则保持，未获 native 证明的保留 payload 不降成 ordinary checkpoint。

catalog 在同一 writer transaction 复查 alias、scope/workspace、tombstone、receipt/source/marker 与每个版本的 owner/fence/revision；manual repair 的 one-shot/audit 失败仍全回滚。marker 登记后由原 closed checkpoint owner 重新认证原 bytes/MAC 及 COV2-25 原始成员，lower issuer 仍独立判定 native；各版本 identity 从本次 lower 结果重新计算并精确对拍，再用 store-owned permit 与原 versioned claims/refCAS 一起提交。已提交后的中断重试只在原 receipt、source、exact versioned revision 和新鲜 native identity 核对通过后做 no-write settlement；不能只看 checkpoint 存在或把 committed claims 重新要求为 reserved。失败/deadline/cleanup_pending 保留可修复原证据，不 ACK。

这是本地机制路径，不增加 OpenCode Session/model CI 或证明真实 Source A；同裸 key 跨 schema 的 native pending 在父 OG-29 正例与真实 Source A 前仍 fail closed。COV2-08 的正式验收/家族 C/D 以计划实际收据为准。


### Versioned prepared ordinary import (COV2-05, unreleased)

The library entry `import_prepared_with_coverage_versions` accepts prepared ordinary turns, an index-aligned coverage version sidecar, a nonzero budget of at most 300 seconds and an optional absolute path to a caller-trusted Libra binary. It checks input size/depth, chronology, privacy-v2 ownership, timestamps and redaction before any lease. The capture scope must use the connected database repository ID, and the absolute storage root must contain that same SQLite database; a mismatched database/storage pair rejects before object writes. Newly matched sensitive content is rejected. This explicit entry alone may choose v2 for the four ordinary record kinds. COV2-11 separately activates typed runtime selection: actual capture/import chooses v2 only when a turn contains supported typed Reasoning, and ordinary turns still choose v1. It neither constructs typed/native reasoning nor proves OpenCode Source A.

Sidecars travel with their turns through sorting. One request rejects unknown versions, length mismatches, repeated bare keys (including across versions), repeated ordinals and missing chronology before writing. Separate requests may explicitly commit independent v1/v2 claims for the same ordinary key once each. An authenticated native association in the other schema still rejects; a version sidecar never supplies native provenance.

Import reservation and binding retain the exact `(coverage_schema_version, logical_turn_key)` through the opaque typed checkpoint adapter. Its prebound versioned branch is private to that adapter; marker generation, scope/deadline, claim owner/fence/revision, import lifecycle/identity and ref CAS remain transaction fences. Failed binding constructs no imported object; stale markers cannot complete the cursor. Committed-source lookup uses batches of at most 64 requested tuples rather than loading session history. Every requested tuple for an ordinal must be committed before the contiguous cursor can pass it.

All-v1 source digest and import identity preimages retain frozen legacy bytes. A source containing v2 uses the explicit `libra-agent-import-source-coverage-v2\0` hash domain with per-turn versions. Privacy identity version is independent of coverage version; mutable privacy-v1 writes stay rejected. One source retains one import identity row whose committed digest/cursor describe its latest request; exact versioned claims, rather than that scalar digest, provide idempotence when requests alternate versions. The library API and its lower gate/store seams add no CLI flags or website command behavior. Pure normalized-turn/temp-DB tests run without an OpenCode Session/exporter/model; actual account capture remains local to macOS, outside CI.

The explicit binary selects only the existing checkpoint object-I/O helper within this import task. Selection checks availability and Unix containment before acquiring any identity lease, restores on return/error/cancellation, and cannot affect other tasks, source readers, discovery, preparation or index repair. `None` uses the existing `helper_program()` (main registration in production; the established private override in unit tests); no host executable, PATH or environment fallback is inferred. This is caller-authorized execution configuration, not binary attestation or Source A. The shared fixed helper argument also serves VerifyReasoningArtifacts in cloud/doctor; those operations are unreachable and forbidden within this ordinary-only scope. Production VerifySnapshot uses the caller-trusted helper response; write OIDs are compared to their expected content, while snapshot verification is not an independent binary/source proof.

macOS real DB acceptance builds the same-source Libra binary before the four exact tests and records its SHA256 before and after. Lib tests locate that binary only in cfg(test) code; the external direct DB test supplies compile-time CARGO_BIN_EXE_libra, including production VerifySnapshot. Existing Unix containment limits these object-helper DB paths to Unix; other platforms reject this opt-in before lease mutation. Pure legacy gold remains portable. Windows compilation/security and four-platform release checks remain required; Linux functional acceptance follows separately.

## Versioned graph/doctor readers (COV2-09, unreleased)

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
and confer no native-origin proof. These reader guarantees apply to the first
stable release containing coverage-v2 and later; they do not assert that the
currently installed stable release contains this unreleased implementation.

## Inert OpenCode visible reasoning classification (COV2-06, unreleased)

`classify_opencode_coverage_source` is a crate-private, pure in-memory parser
seam with no capture/live/import callers before COV2-11. The explicit COV2-10
pure-memory preview may consume it. It consumes the original
bounded export before state/proof erasure. Closed native/classic assistant
reasoning parts with nonempty visible text become gated typed Reasoning
records, retaining source message/part IDs and ordinals. Duplicate or missing
coordinates, contradictory session/message references, empty/whitespace text,
state/proof-bearing or unsupported parts remain partial without a typed record
for that part. Malformed/duplicate-key/unknown export structure and budget
failures return payload-free `Partial(error)` when parsing cannot continue.
Safely retained parts with unsupported siblings instead return
`Parsed(source)` with `source.partial = true`; neither case permits cursor
completion from unproved content. A separate valid visible part never clears
a partial flag.

The source/tree byte, node and depth budgets run before materialization; at
most 4096 visible parts are emitted. The existing reviewed preprocessor runs
as a second bounded pass to preserve its closed envelope classification and
owning candidate separation. Both parse passes are bounded; this is not a new
unbounded scan. IDs and the original parse tree wipe on drop. The carrier has
no Clone/Serialize/Debug, and ProviderVisibleText retains its serialization
barrier and content-free Debug. It grants neither native provenance, an issuer,
an opaque artifact, nor capture authority. COV2-10 performs typed redaction in
the explicit preview before COV2-11 connects any persistence path; the retained
ordinary projection is unredacted input until that bounded redaction stage.

Actual production normalization is unchanged: reasoning keeps the turn
incomplete, contains no typed Reasoning record and selects v1. The COV2-06
contract test uses a complete ordinary control to distinguish that decision
from unrelated malformed input, and pins the absence of the new parser/preview from
live/import and the actual authorized capture entry. The separate COV2-05 ordinary versioned library
API remains available. These tests use synthetic buffers only, with no Session,
exporter or model; no user-visible command or website behavior changes here.


## Redacted in-memory coverage preview (COV2-10, unreleased)

`CaptureSnapshotService::preview_redacted_coverage_v2(agent_kind, bytes, policy)`
accepts caller-owned memory and returns `(RedactedBytes, serde_json::Value)`.
It opens no source, runs no exporter or model, and writes no checkpoint, claim,
artifact or cursor. The JSON envelope remains 1, with each turn's explicit
coverage version, digest and safe projection. This library data API grants no
source/capture authorization, native issuer or opaque-artifact capability.
Production live/import activation is a separate COV2-11 change; this preview
must not replace its authorized source boundary.

For OpenCode, classification precedes state/proof erasure. Original session,
message and part coordinates are checked before redaction; text equality only
confirms the located part. Ordinary bytes and decoded visible reasoning are
redacted before any v2 canonical encoding, digest or safe projection. Decoded
JSON strings are also redacted before reserialization, so JSON escape spellings
cannot reveal a secret during canonicalization. Additional matches outside the
visible reasoning field conservatively make the result partial. Readable parts
retain source order alongside text and tools; they do not settle a turn without
the normal supported completion event. Unsupported siblings keep it partial.
Unproved state/proof candidates supply no ordinary or typed plaintext: the
preview consumes the separated ordinary projection and destroys the owning
sealed candidate, native and partial-permit authority. Its only retained signal
is a boolean that makes the result partial. Candidate absence is checked using a non-secret-shaped
canary, independently of the redactor's matching rules.

Source bytes, parse depth/nodes and visible-part count remain bounded. Original
parse trees are dropped before the preprocessor's second bounded parse. The
redactor retains its output/working-buffer limits; those limits are not a claim
about total process RSS. Canonical/JSON output also has a checked byte budget,
and the optional absolute deadline is checked between stages and turns. Errors
are payload-free. COV2-10 initially retained an ordinary Rust String.
COV2-11 explicitly wipes `ProviderVisibleText` on drop and before redaction
replacement, and the prepared ordinary owner uses `Zeroizing<Vec<u8>>`.
This does not guarantee erasure of Serde scratch, allocator copies or all
process memory. Existing Claude Code/Codex
adapter preprocessing, ordinary bytes, v1 digest and safe projection are checked
against their established pure-memory path. All these checks run without an
OpenCode Session, exporter, account or model; account capture remains local to
macOS and outside CI. No command, flag or website command behavior changes.

## Coverage-v2 capture (unreleased)

From the first stable release containing coverage-v2, OpenCode export capture
and historical import retain supported provider-visible reasoning as typed,
redacted turn content. Turns containing that content use coverage schema 2;
ordinary turns keep schema 1 and their existing canonical bytes. Redaction runs
before either digest or checkpoint projection. This does not authenticate
`signature`, `redactedData`, or `reasoningEncryptedContent`: unproven native
fields remain partial and cannot be archived as trusted artifacts.

For a typed snapshot, the durable `redaction_bytes_scanned` projection records
the original authorized source byte length. The detailed in-memory redaction
report also counts the bounded decoded-text passes, so its aggregate
`bytes_scanned` can be larger. Taking the private runtime turn companion does
not change the durable source metric or its source commitment.

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
These changes add no command flags and are not included in the installed
0.30.49 release. OpenCode Session/exporter/model acceptance runs locally on
macOS; it is not a CI integration test.
