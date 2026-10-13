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
logical key without guessing from the key. COV2-03 adds version-aware marker
and commit transactions; COV2-04 adds the authenticated pending wire; COV2-11
switches production live/import paths together. This preparatory arbitration
does not itself enable OpenCode Session capture or native archival.
