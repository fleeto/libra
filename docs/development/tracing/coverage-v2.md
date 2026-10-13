# Coverage schema v2 — typed reasoning encoding boundary

This document specifies the codec introduced by COV2-01. It does not activate
the live or historical-import writers; COV2-11 owns that production switch.
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
behavior for frozen v1/RG-04 oracles; production live/import call sites must
switch together under COV2-11. This codec does not change the database,
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
versioned reservation/retain steps, COV2-04/22 add pending sealing/persistence,
and COV2-11 switches production live/import paths together.
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
COV2-04/22 add pending seal/persistence, and COV2-11 switches live/import.
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
change. It does not activate production v2 identity writing, pending v2
persistence/replay, OpenCode Session capture, native issuer authority, or
reasoning archival; those remain with their later task cards and Source A.
