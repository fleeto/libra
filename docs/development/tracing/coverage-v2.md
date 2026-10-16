# Coverage schema v2 — typed reasoning encoding boundary

> **Coverage-v2 source version:** The coverage-v2 descriptions below refer to the Libra 0.30.50 source line and later compatible versions. This is not a stable-release announcement; check the [official releases](https://github.com/libra-tools/libra/releases) and `libra --version` for installed support. Other sections retain their own version requirements. Coverage-v2 does not prove native origin or authorize native reasoning artifacts.

> 以下带 COV 编号的状态引文是各自时点的历史；当前交付和发布状态以实际计划/收据及上方 source-version 说明为准，不把旧 pending/FAIL 改记 PASS。

> **COV2-17 10/10＋真实visible-only已通过，单审/交付待：** 仅一次Haiku4.5实际观察；source当前24/26=92.31%，本卡signed交付闭合后才25/26=96.15%；formal0家族C/D、父formal59/71不变。下一COV2-REL-01。

> **历史快照（2026-10-15 21:06 UTC 真实观察前；已由上方结果覆盖）：COV2-17 实际10/10通过，真实观察待。** 前驱20f7787已signed交付，COV source24/26=92.31%；本卡AC12/VER6、单审/交付仍待，formal0家族C/D，父formal59/71不变。

> **COV2-16 本机12/12已通过，单审/交付待：** G09当前12选定名已在actual8a最终行为源码非skip通过，原legacy FAIL/defer保留；source当前23/26，本卡signed交付闭合后才24/26=92.31%，formal0/26，父formal59/71不变。下一COV2-17。

> **G09 前瞻范围修订（由本次签名提交采纳生效）：** legacy `info.status="stopped"`终态import样例实际FAIL，归DEFER-COV11-IMPORT-TERMINAL-01计划外延期；本轮import功能正例以已审当前格式替代。源码/测试不改，旧FAIL不变；仅实际soleClaude PASS、signed main采纳后新matrix生效；source23/26不增。


This document covers the codec introduced by COV2-01 and the COV2-11
live/export and historical-import wiring included in Libra 0.30.50. Stable availability and the
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


<!-- Compatibility anchor for earlier links; not a release-status assertion. -->
<a id="versioned-prepared-ordinary-import-cov2-05-unreleased"></a>

### Versioned prepared ordinary import (COV2-05, Libra 0.30.50)

The library entry `import_prepared_with_coverage_versions` accepts prepared ordinary turns, an index-aligned coverage version sidecar, a nonzero budget of at most 300 seconds and an optional absolute path to a caller-trusted Libra binary. It checks input size/depth, chronology, privacy-v2 ownership, timestamps and redaction before any lease. The capture scope must use the connected database repository ID, and the absolute storage root must contain that same SQLite database; a mismatched database/storage pair rejects before object writes. Newly matched sensitive content is rejected. This explicit entry alone may choose v2 for the four ordinary record kinds. COV2-11 separately activates typed runtime selection: actual capture/import chooses v2 only when a turn contains supported typed Reasoning, and ordinary turns still choose v1. It neither constructs typed/native reasoning nor proves OpenCode Source A.

Sidecars travel with their turns through sorting. One request rejects unknown versions, length mismatches, repeated bare keys (including across versions), repeated ordinals and missing chronology before writing. Separate requests may explicitly commit independent v1/v2 claims for the same ordinary key once each. An authenticated native association in the other schema still rejects; a version sidecar never supplies native provenance.

Import reservation and binding retain the exact `(coverage_schema_version, logical_turn_key)` through the opaque typed checkpoint adapter. Its prebound versioned branch is private to that adapter; marker generation, scope/deadline, claim owner/fence/revision, import lifecycle/identity and ref CAS remain transaction fences. Failed binding constructs no imported object; stale markers cannot complete the cursor. Committed-source lookup uses batches of at most 64 requested tuples rather than loading session history. Every requested tuple for an ordinal must be committed before the contiguous cursor can pass it.

All-v1 source digest and import identity preimages retain frozen legacy bytes. A source containing v2 uses the explicit `libra-agent-import-source-coverage-v2\0` hash domain with per-turn versions. Privacy identity version is independent of coverage version; mutable privacy-v1 writes stay rejected. One source retains one import identity row whose committed digest/cursor describe its latest request; exact versioned claims, rather than that scalar digest, provide idempotence when requests alternate versions. The library API and its lower gate/store seams add no CLI flags or website command behavior. Pure normalized-turn/temp-DB tests run without an OpenCode Session/exporter/model; actual account capture remains local to macOS, outside CI.

The explicit binary selects only the existing checkpoint object-I/O helper within this import task. Selection checks availability and Unix containment before acquiring any identity lease, restores on return/error/cancellation, and cannot affect other tasks, source readers, discovery, preparation or index repair. `None` uses the existing `helper_program()` (main registration in production; the established private override in unit tests); no host executable, PATH or environment fallback is inferred. This is caller-authorized execution configuration, not binary attestation or Source A. The shared fixed helper argument also serves VerifyReasoningArtifacts in cloud/doctor; those operations are unreachable and forbidden within this ordinary-only scope. Production VerifySnapshot uses the caller-trusted helper response; write OIDs are compared to their expected content, while snapshot verification is not an independent binary/source proof.

macOS real DB acceptance builds the same-source Libra binary before the four exact tests and records its SHA256 before and after. Lib tests locate that binary only in cfg(test) code; the external direct DB test supplies compile-time CARGO_BIN_EXE_libra, including production VerifySnapshot. Existing Unix containment limits these object-helper DB paths to Unix; other platforms reject this opt-in before lease mutation. Pure legacy gold remains portable. Windows compilation/security and four-platform release checks remain required; Linux functional acceptance follows separately.

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
<a id="inert-opencode-visible-reasoning-classification-cov2-06-unreleased"></a>

## Inert OpenCode visible reasoning classification (COV2-06, Libra 0.30.50)

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


<!-- Compatibility anchor for earlier links; not a release-status assertion. -->
<a id="redacted-in-memory-coverage-preview-cov2-10-unreleased"></a>

## Redacted in-memory coverage preview (COV2-10, Libra 0.30.50)

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
These changes add no command flags and require Libra 0.30.50 or later. OpenCode Session/exporter/model acceptance runs locally on
macOS; it is not a CI integration test.

<!-- Compatibility anchor for earlier links; not a release-status assertion. -->
<a id="macos-local-session-acceptance-cov2-12-unreleased"></a>

## macOS local Session acceptance (COV2-12, Libra 0.30.50)

Controlled-exporter Session acceptance is an operator-local, ignored gate under the existing `test-live-agent` target feature. The macOS harness uses the existing Required Seatbelt server/client protocol, a canonical isolated repository/store, and registered exporter provenance. A missing containment or store prerequisite fails the manual test; it must not return success before its assertions. The live and import gates compare persisted semantic canonical bytes and actual claim digests against the independently pinned COV2-11 gold, then check repeated-delivery idempotence. This does not expose the private runtime canonical buffer; the binding relies on digest collision resistance. Native idle closes the live turn, while the current import Session remains active: the fixture expects live Complete and import Incomplete without changing the canonical digest or claiming to fix deferred import-terminal handling. The failed-export gate checks its intended failure path and zero completed writes.

The scripted extraction fixture is named `opencode_extraction_no_consumer_keeps_pending_without_checkpoint`: both its original PARTIAL input and broken-input delivery must leave zero completed checkpoints and pending receipts in the real temporary database. The old `opencode_extraction_failure_continues_checkpoint` failure remains historical evidence; it does not authorize a consumer-policy change. COV2-12 covers its three new macOS Session cases and this repaired fixture. COV2-16/17 separately audit the other 21 inherited test bodies.

These controlled fixtures do not prove native Source A or the three native fields. They require neither an account nor a model call, and Session/Checkpoint integration is not selected by CI. Pure parser, cryptographic, sandbox-unit and static guards remain eligible for CI. The manual commands and exact-selector requirements are in `docs/development/integration/integration-test-plan.md`. No public command, flag, schema, release version or installed OpenCode changes in this card.


<!-- Compatibility anchor for earlier links; not a release-status assertion. -->
<a id="candidate-live-session-acceptance-cov2-15-unreleased"></a>

## Candidate live Session acceptance (COV2-15, Libra 0.30.50)

Exporter trust authenticates the local executable and export identity; it does not establish native origin for an OpenCode reasoning part. A proof/state-bearing part remains a candidate until the producer proof is verified. The live path must classify that source as partial, keep candidate content out of ordinary and typed text, persisted checkpoint objects, logs and public errors, and mint neither a native issuer nor an opaque reasoning artifact. A source without a ready consumer can leave its receipt pending and return the existing content-free hook error; that refusal is not a successful empty checkpoint. Safe retained ordinary content, where permitted by the existing partial consumer, does not promote the removed candidate.

The operator-local macOS case is `cov2_macos_fake_exporter_candidate_live_partial_no_plaintext`. It uses the registered controlled exporter through the actual Required sandbox client and live hook, with explicit assertions for each sink. It requires `test-live-agent`, `--ignored --exact --nocapture --test-threads=1`; a missing prerequisite or zero selected tests is UNRUN. Its static companion is `cov2_candidate_live_session_case_is_ignored_and_non_skippable`. These gates use synthetic candidate content and grant no native/Source A evidence. Session capture cases stay out of CI; parser, crypto, sandbox unit and static guards can still run there. Historical import candidate acceptance belongs to COV2-18.


<!-- Compatibility anchor for earlier links; not a release-status assertion. -->
<a id="candidate-historical-import-acceptance-cov2-18-unreleased"></a>

## Candidate historical import acceptance (COV2-18, Libra 0.30.50)

Historical OpenCode import applies its own source, cursor and object-write boundaries. Exporter trust does not prove native origin for a proof/state-bearing reasoning part. Import must retain the partial classification, exclude candidate content from ordinary and typed text, persisted import/checkpoint objects, logs and public errors, and grant neither a native issuer nor an opaque reasoning artifact. The existing consumer decides whether safe ordinary partial content can be retained or the source must be refused; live capture acceptance does not establish import acceptance.

The operator-local macOS case is `cov2_macos_fake_exporter_candidate_import_partial_no_plaintext`, with `test-live-agent` and `--ignored --exact --nocapture --test-threads=1`. Its static companion is `cov2_candidate_import_session_case_is_ignored_and_non_skippable`. The case must execute the trusted controlled exporter through the actual historical import path and inspect the actual import outputs and stores independently. A missing prerequisite, early success return or zero selected tests is UNRUN. Synthetic candidates provide no native/Source A proof. Session capture remains operator-local; CI can run parser, crypto, sandbox unit and static guards. COV2-15 remains the separate live-path acceptance.

At the current import source gate, proof/state candidates are refused before a Session, cursor or checkpoint is published. The private preparation helper clears its environment and discards stderr. Its internal partial warning is therefore not a public import diagnostic. Local acceptance couples a successful ordinary import control and a same-input data-only partial comparison to the actual content-free source refusal, zero published content and a static check of the pre-projection rejection route; it does not claim direct observation of a private partial field.


## Current macOS capture audit (COV2-16; effective through the signed G09 adoption)

The current selection of eight retained bridge cases, one current-format import replacement, two pipeline cases and the OG-26 candidate case must be audited locally on the final source. The excluded classic legacy case remains FAIL/deferred; across both audits the 22 selected names are 21 inherited plus one replacement, with 26 unique required names after deduplication against the five new cases. The extraction fixture now uses `opencode_extraction_no_consumer_keeps_pending_without_checkpoint`, the reviewed COV2-12 successor of the historical failing selector. Tests and production files remain read-only during this audit. A zero selection, ignored prerequisite or successful early skip is not acceptance. The legacy oversize test alone does not establish that the byte cap was reached if its exporter instead failed to fork inside the sandbox; that limitation stays explicit. No real native fields, Source A or Linux acceptance is implied.


## Legacy terminal-status import debt (effective through the signed G09 adoption)

The old `opencode_historical_import_e2e_uses_trusted_export_bytes` fixture supplies classic `info.status="stopped"`. It currently fails with `LBR-AGENT-020`; its failure is preserved under `DEFER-COV11-IMPORT-TERMINAL-01`. The G09-defined macOS audit requires the reviewed current-format `cov2_macos_fake_exporter_import_session_capture` for the import positive instead. This changes the acceptance inventory, not the old test result or production admission rules. The original test remains ignored/manual and unchanged. Required current audit names total 22 (21 inherited plus one replacement); the replacement already belongs to the five new cases, so the unique required Session set is 26. No Source A/native, full-suite or release gate is waived.


### COV2-16 legacy import 终态样例延期与当前格式替代（2026-10-15 19:53:41 UTC，由本次signed main采纳生效）

95a9400最终行为源码只读审计实际11/12通过，唯一失败 `opencode_historical_import_e2e_uses_trusted_export_bytes` 的fixture包含classic `info.status="stopped"`，来源投影closed-info不接纳该非当前格式字段而标unproven，实际import返回LBR-AGENT-020；原9组为8PASS/1FAIL/0ignored，rawSHA `b1ab0f5fe8a6637a25c7484753146bb89d4f391395357cc869f1579589a9e077`，不能记9/9或把失败改PASS。collect_facts虽仍认识legacy status终态，当前ordinary29源要求与该旧契约不一致；把status添加到来源准入会涉及legacy终态兼容，归既有 `DEFER-COV11-IMPORT-TERMINAL-01` 计划外延后，当前默认拒绝不改。用户已授权相似复杂外围问题延后、优先当前OpenCode基础采集与producer；本次不增加生产写权，不弱化candidate/native/Source A边界。

当前审计AC1改用已经COV12source-reviewed的 `cov2_macos_fake_exporter_import_session_capture`：真实trusted exporter经当前格式历史import写入独立checkpoint，绑定独立canonical gold7973604f...与live/import readback，保留actualimport Incomplete/active fence。这**替代一项本轮功能验收**，不是旧compatibility PASS，旧测试body/fixture全部保留不改。COV16当前12名=8个其余旧bridge+1当前格式import+2pipeline+1OG26；COV17仍10个旧名。当前两个audit共22个选定名=21继承+1当前替代；全部新增Sessioncase仍单独要求，不重复计算：原27名immutable/static CI inventory保留，release必需unique26=21继承+5新增，另1legacy名明确DEFER/FAIL。旧22 inherited allPASS历史判据由本段及当前卡/REL/DEP镜像明确覆盖，不倒改任何旧事实。

COV16仍13AC/5VER/S/0prod0landing，26卡和71父卡分母、DAG、SourceA/真实模型/四平台/安全CI/站点安装/最终bumped full要求均不变。此四文档G09协调仅source23/26=88.46%、formal0/26，父formal59/71=83.10%、source61/71=85.92%不增；COV16仍in-progress/空，原失败不能被批准后的新矩阵回写。实际单Claude PASS/root裁决、109reader/fmtstrict和signed+DCO main/Push/freshremote后才采纳；新8+1+1+1+1矩阵仍须实际运行，本提案不预授audit/source24。


**G09 协调写集（与 COV2-16 审计写集分开）：** 本次仅同步以下四条既有文档精确路径：`docs/development/plan/plan-20261013-coverage-schema.md`、`docs/development/plan/plan-status.md`、`docs/development/plan/plan-20260902.md`、`docs/development/tracing/coverage-v2.md`。这是验收范围/依赖镜像的前瞻协调，不是 COV2-16 的实现写权扩展；COV2-16 后续审计仍仅 I-audit 原三文档。四路径只由 root 串行写入，无新增任务/依赖边/生产或测试写权、无版本/安装/网站行为变更，恢复为 signed+DCO main 前滚。G-01–11 复核：单一 legacy 终态兼容延期与当前格式功能替代；COV16 保持 13 AC/5 VER、S、0 生产文件/0 落点，REL 保持 12 AC/12 VER，26/71 卡分母与既有 DAG 不变；原始失败、legacy fixture 和 CI 中仅静态库存检查全部保留。仅单 Claude Code 实际 PASS、root 裁决、文档门/fmt/strict 和签名 main 交付闭合后采纳；不授 source24。


**替代覆盖边界与遗留重启：** 当前格式import正例不覆盖旧classic `info`/`parts`端到端兼容，也没有查询session行来证明`started_at`/`last_event_at`由`info.time`映射，更不证明`stopped_at`终态；pure collect_facts timestamp单测不等于该DB端到端证明。这些缺口与原终态例的FAIL一并明确归计划外`DEFER-COV11-IMPORT-TERMINAL-01`，未修复、不记PASS；将来另有界兼容任务须保留来源安全边界、soleClaude审查并使原legacy用例实际非skip1/1、读回上述时间与终态后才可解除。原COV11的“不免除原AC/VER”不被追溯更改，本次只前瞻调整COV16和依赖/发布集合。

**R1评审事实：** soleClaude实际exit0但VERDICT FAIL，rawSHA `91ceab399575e8dd72f1832a06b1b960e45eebaae30f84cf9df621b0f0d70372`；1P2为五处当前规范未同步，另4P3为覆盖缺口、遗留登记/重启、采纳措辞与时区顺序。本版同步这些项；旧R1不是PASS、尚未采纳。当前修订须自己的实际R2PASS与签推门，不预授新matrix/source24。


### COV2-16 当前12例最终源码本机审计（2026-10-15 20:37:17 UTC；A通过，单审/交付待闭合）

前驱CFG窄修复已实际signed+DCO main `8a547afd0fd5a73247ba47ec979541b3aacba8c9`、Push0/remote同SHA、actualarchive2048同13QA+final109reader/fmtstrict后像；codeR2soleClaude PASS/raw `2e95bbe94161a611df30e4893da5de23084fa14cbaff5d5bd47bac659cf30f5e`。仅import/cfg/lint变化、全部runtime body不变，原CODE R1 FAIL1P1保留。Windows必要CI仍pending，不以Mac冒称通过。此处测试冻结actual8a，root审计仍只写I3文档，不改任何生产/测试。

本轮实际12个不重复功能用例全部PASS/0FAIL/0ignored：其余old bridge8/8（raw `29bd53356664efd2204229f29f3b23f4fd2aac7be6ef9df03adcf2aade56ad42`）＋当前格式import替代1/1（raw `e37324c9f50ea2931634034ec2841790b63c8219c248e8e1782f6ea8d78567c1`）＋no-consumer extraction1/1（raw `83be6e7d6387ccc1d60a9b854f26e230e16a4cf6b192c31dd5bfce33fc6082b4`）＋lease1/1（raw `bec70f4136de611348cc0f283ad3fe66bfe01fb14067bb74b818f5ccb9a20b5a`）＋OG26 zero-normal-sinks1/1（raw `919475e9442ea40514b2018a73cd77b05fc461d0daa9d785775bb92e6a262bb5`）。每个实际testname、匹配数、exit及start/end均与原始Cargo报告相符；静态non-skip/manual-only两门各1/1，64+32+1+12=109readers、fmt和strictalltargets/allfeatures-Dwarnings实际exit0。共12个QA步骤，2048文件前后SHA/OID/size/mode/kind同一，manifestSHA `4f0fde5edd16d4d9fdc21e017b3c6ef45e8e1ff90c655021177a517d4ad05208`。

- `src/internal/ai/capture/live_pipeline_tests.rs` SHA256 `3413f5059f1899c499e7c8e0067da5e1c5b1babd11502071c2ab2c4467ad9722` mode `0o644`。

- `tests/agent_checkpoint_redaction_test.rs` SHA256 `ef1896407d1a0db29be11dcb25892659fb96f5d3df1a524d72eecc247f3b7cb2` mode `0o644`。

- `tests/agent_opencode_bridge_test.rs` SHA256 `c04d3544f3e1dd492a6be269efaa9962f90855e46d507af7bf16c0c2c5b915c8` mode `0o644`。

Required macOS Seatbelt assert和每个选定body的执行路径已对照，未以缺sandbox/0selected/早return计PASS；static guard两门并不额外算功能用例。旧oversize body若先遇到sandboxfork拒绝，仅证明保守降级，不扩大成实际触达byte-cap正例。旧classifier私有字段仍只有此前审定的有界路由推断，不声称直接观测或SourceA。原parser postprocessing首次同一行ok正则漏识别nocapture插入日志，断言失败保留；只修私有收据解析，用实际name+exit+pass/fail/ignored结果校核，源码和测试从未改动。

本轮范围严格依actual signed G09 a4a863f：AC1为current-format功能替代。旧classic status用例在95a实际8PASS/1FAIL的原FAIL/raw b1ab0f5...保留计划外DEFER-COV11-IMPORT-TERMINAL-01，classic info/parts、session行时间映射/终态缺口未修复、不追认兼容PASS；原27静态inventory与所有测试body未改。两个audit当前22名=21继承+1替代，替代已在5新增中；required unique26。COV17十例/真实visible-only、SourceA/native三字段、家族bumped full/安全CI/四平台/站点安装门均不减。

原13AC/5VER按实际测试勾选，但Lifecycle仍in-progress/Acceptance空，须新增三文档相关QA、soleClaude审计PASS/root裁决及signed+DCO main/archive2048/Push/freshremote后才source24/26=92.31%；此前仍23/26=88.46%、formal0/26家族C-D。父formal59/71=83.10%、source61/71=85.92%不增；71/26分母不变。无新full、Linux功能/账号模型/nativeissuer/注册/网站/版本/安装/SessionCI操作。当前runtime已装official49原SHA保持，用户04/05及原7stash保留。下张COV2-17只在本卡源码交付真实闭合后开工。


### COV2-17 开工与10例实际审计（2026-10-15 21:06:04 UTC；真实响应观察仍UNRUN）

前驱COV16实际signed+DCO main `20f7787b8c8589e1af24c71d1835973adbce3d98`，parent8a547af；archiveSHA `ac25cbcfb3cfd6cbccaecd742c046f10e810d85b34a91a215e8e8d4ebd1924fc`，2048文件与最终QA后像相同，3个I文档外2045行为输入一致；实际Push0/freshremote同一，soleClaude PASS与root裁决已闭合。source当前24/26=92.31%，formal0/26仍家族C/D，父formal59/71=83.10%、source61/71=85.92%不变；Windows8a特定job实际success不预授20f新SHA的CI。

本卡在actual20f最终行为源码运行：lifecycle7/7＋crash1/1＋import确认1/1＋oracle1/1，共10uniquePASS/0FAIL/0ignored；两个no-skip/manual-only静态门各1/1，109reader（64+32+1+12）、fmt与strictalltargets/allfeatures-Dwarnings全部实际exit0；共10个QA步骤，2048前后SHA/OID/size/mode/kind同一。

- VER1-lifecycle7 actual 7/actual7，raw SHA256 `1a816571d12de02af202ed609f18dd5a55a24e2b6cb27e8f29543b48ecb7394a`，UTC 2026-10-15T20:56:24.558735+00:00→2026-10-15T20:56:53.387131+00:00。

- VER2-crash actual 1/actual1，raw SHA256 `2b78a5b2bc5029be01943873564f0134eb5c0f42ec755009a2f8655df0142abf`，UTC 2026-10-15T20:56:53.387309+00:00→2026-10-15T20:57:01.473408+00:00。

- VER3-consent actual 1/actual1，raw SHA256 `776dbbc2e3ad631876ae93c4f0dabf7b904efcc7c5583469059fcbecd1044c18`，UTC 2026-10-15T20:57:01.473599+00:00→2026-10-15T20:57:05.552888+00:00。

- VER4-oracle actual 1/actual1，raw SHA256 `493cd059e627aeb66a06a460e8363204d872f81f4d56eb95c3d5e8012e59a328`，UTC 2026-10-15T20:57:05.553099+00:00→2026-10-15T20:57:19.608628+00:00。

- `tests/agent_lifecycle_event_test.rs` SHA256 `c0bdb6ca4e4ce3f77c2e7e031f2af3551cfed58a2b3798b6143a7c6989facad4` mode `0o644`。

- `tests/agent_hook_crash_test.rs` SHA256 `5c16c45c9e4fc6929b4681a30b4a8f541357240c708d61d1199e68a18f87e977` mode `0o644`。

- `tests/agent_import_test.rs` SHA256 `9f165781a65971f9dbac5ed6442353e681be429ae73bef39987351371680dc74` mode `0o644`。

- `src/internal/ai/capture/live_oracle_tests.rs` SHA256 `2493e17afad7211842e046c41f5327ae5ed7e902c6fd64ce76871904c301d0b8` mode `0o644`。

原具名10项与实际Cargo所选name/positivecount/exit一一对应；lifecycle public-contract表包含OpenCode surface；crash固定8row中两条OpenCode逐一进入for循环，fault/replay/claim/receipt/session oracle无早退；metadata三provider包含OpenCode桥不可用row并与冻结整张gold逐字断言。所选body没有环境skip或return；两个静态guard不额外算功能用例。这些本机fixture并非真实native来源/正例；原COV16legacy FAIL/defer及SourceA仍保留。

AC1–11/VER1–5按实际通过勾选；AC12/VER6仍未运行、不勾选。当前已验证官方OpenCode2.0.29 SHA7f46ba1...、88个公开builtin plugin active、外部plugin0、官方Anthropic endpoint配置一致、Haiku4.5 enabled、thinking disabled；工具权限最后wildcard deny后仅有browser deny，属于更窄deny而非allow/ask。官方2.0.29 `packages/core/src/permission.ts:87–95` 使用 `findLast`（第91行），故最后匹配rule优先；追加browser deny不会重开工具。实际四轮私有preflight停止，均在任何模型请求前（attempt0）；首轮只记总阶段、未保留更细原因，后续修正owned路径canonical比较及Location.response wrapper，并将过严的最后一条恰为wildcard检查改为最后wildcard deny之后只能继续deny。第四轮内容无敏元数据证实追加browser deny。这些是collector准备，不是模型FAIL或模型重试；其原STOPPED记录保留。已修正私有collector，不改任何仓库源码/测试/原AC/VER。独立私有Rust classifier初次误用cfg(test)辅助方法编译失败保留，修正调用现有public with_deadline后实际编译exit0；仅依已审pure preview API，不授native能力。

本卡只写原I4文档，无生产、测试、CI、installer/版本或公开命令变化，网站N/A。Linux功能由用户后验；无新full，家族首次bumped T2全量/安全CI/四平台/站点安装仍必需。未登记issuer、不live-mint、不修改globalauth/config，owned credential以内存转移到临时store，完成后清理；不存响应正文或密钥。Source A/native三字段不因可读响应放行。未知后续SHA/结果不预填，soleClaude/root/signed交付待；source25/26只在本卡全部门实际闭合后登记。


### COV2-17 真实visible-only观察结果（2026-10-15 21:07:35 UTC；功能A通过，单审/交付待）

预登记 `COV17-OBSERVATION-PREREG-20f7787` 在实际启动前已写本文；gate UTC `2026-10-15T21:06:04.673195+00:00`，绑定prereg SHA `f9ac142572ba7f629669b1db78546e60fc7af69e8f2ffe338a30739fb41e4d63` 与当时plan SHA `4606c0a9a235511d421fd84a67552bd0ca6eb2cc9f184807559a903c0109b161`。实际唯一attempt UTC `2026-10-15T21:06:04.962167+00:00`，结果UTC `2026-10-15T21:06:07.317946+00:00`；同一owned服务上重核88 builtin/0external、配置/endpoint/model/权限后才发起，genuine provider run/原始export/当前classifier实际exit均0，elapsed `2.355785`秒，小于600秒，不是按elapsed估算成功。1个新Session、1个step_start/1个text事件；无工具事件，无附加模型尝试，也不把一次CLI启动说成只有一次HTTP请求。

当前pure classifier在actual20f行为源码实际编译的artifact上读同一未经sanitize/字段删除的export bytebuffer，实际classifier_ok=true、partial=false、1turn、coverage版本[1]。原始树全层有界检查state/signature/redactedData/reasoningEncryptedContent/originProof/thoughtSignature计数均0；结果HMAC-SHA256 `c1275413b2e9f8b84c87438d89182b61f51b26c258a18c6338c6844dd1d8f6ac`，新鲜HMAC key只在内存、未留存。只观察到普通可见文本；coverage-v1不证明typed Reasoning positive，native/SourceA credit=false。原始响应、stderr/credential内容不进证据；owned运行store已在进程终结后清理，保留的只是固定字段名、JSON类型/数量与以下无敏元数据。并未更改global auth/config、登记issuer、调用live mint或给native三字段放行。source-side `auth export anthropic` 使用现有用户环境且没有显式 `--server`，可能ensure/启动其现有managed服务（official29 cli auth/export.ts:12 与services/server-connection.ts:21–52）；未宣称该源端的服务或文件系统副作用为零，目标auth import仍仅owned store。

观察收据SHA256 `5eda73cd5b53e4354a3314d55e754441da808cafddfcd020a63f8befc2daf97a`；独立原始树计数/classifier无敏收据SHA `c601d95d0712ccf8d4820c0299de56193daa7b6ae4eb5d1275b7b83a54b5bbe1`。内容无敏字段清单如下（`<other>`合并未知字段名，未保留任意自由字段/值）：

```json
{
  "field_names_types_counts": {
    "info:object": 1,
    "id:string": 7,
    "projectID:string": 1,
    "agent:string": 3,
    "model:object": 2,
    "providerID:string": 2,
    "<other>:string": 6,
    "cost:number": 2,
    "tokens:object": 2,
    "input:number": 2,
    "output:number": 2,
    "reasoning:number": 2,
    "cache:object": 2,
    "read:number": 2,
    "write:number": 2,
    "time:object": 5,
    "created:number": 5,
    "updated:number": 1,
    "<other>:number": 2,
    "title:string": 1,
    "<other>:object": 1,
    "directory:string": 1,
    "messages:array": 1,
    "type:string": 5,
    "text:string": 2,
    "<other>:array": 1,
    "completed:number": 1,
    "content:array": 1,
    "finish:string": 1
  },
  "state_proof_candidate_counts": {
    "originProof": 0,
    "reasoningEncryptedContent": 0,
    "redactedData": 0,
    "signature": 0,
    "state": 0,
    "thoughtSignature": 0
  },
  "node_count": 72
}
```

AC12/VER6仅按上述实际观察勾选；原12AC/6VER现均有具名实证，Lifecycle仍in-progress/Acceptance空，不能在Claude/root/signed交付前称本卡已交付。后续只有I4文档变化，全部2044非I源码/测试/配置/fixtures与actual20f行为同SHA/mode；需新增I4文档109reader/fmtstrict、唯一Claude PASS/root裁决、签名+DCO main/archive2048/Push/freshremote，同一性闭合后才source25/26=96.15%并登记DEP-OG-13 source-ready。formal0/26仍家族C/D，父formal59/71=83.10%/source61/71=85.92%不变。下张COV2-REL-01首次bumped最终源码双env full、安全CI、四平台签名发布、站点/Homebrew/curl安装及原`.2`兼容门均保留；这次普通响应不关闭Source A或current`.3` SRC029/RUN029。
