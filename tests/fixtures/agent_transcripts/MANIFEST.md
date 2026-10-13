# Agent transcript fixtures — provenance

Consumed by `tests/agent_transcript_intelligence_test.rs` (AG-21 / E6 / E7 /
A0-07). Assertion failures should be triaged against this table:
implementation regression vs upstream format drift.

**Provenance note (2026-07-15)**: the original fixture files were referenced
by the test suite but never committed to the repository, so a fresh checkout
could not run the target. These files are **synthetic reconstructions**
hand-authored against the frozen parser shapes in
`src/internal/ai/observed_agents/extract.rs` — they are *not* captures of
real agent sessions. Each file exercises exactly the values the test pins.

| File | Format modeled | Pinned expectations |
|---|---|---|
| `claude_code.jsonl` | Claude Code session JSONL (`type`/`uuid`/`timestamp` envelope, `message.content` string or block array, Claude-native `usage` keys `cache_creation_input_tokens`/`cache_read_input_tokens`, `tool_use` blocks with `input.file_path`) | 2 prompts; 1 `/review` skill event (`input_slash_command`); merged usage input=200 output=65 cached=40; model `claude-sonnet-5`; modified files `src/lib.rs`, `docs/readme.md` (in order); no `Task` tool so subagent-aware total equals session usage |
| `codex.jsonl` | Codex rollout JSONL (generic `message` wrapper records; E6 wire `usage` keys) | 2 prompts; 1 `/review` skill event; merged `total_tokens=260` (155+105); model `gpt-5.3-codex` |
| `opencode.json` | Synthetic OpenCode 2.0.26 native `{info,messages}` export; fixed official commit `9b4ec5714d481559990db0a816d5dec19541a814`, schema session-message.ts/session-transfer.ts, core session/transfer.ts and token-usage.ts; never live/native-origin proof | 2 user.text prompts; 1 `/review` skill event; assistant model.id `claude-sonnet-5`; tokens input=2 output=3 reasoning=4 cache.read=5 cache.write=6, cached=11 total=20 api_call_count=1; modified files `src/lib.rs`, `docs/readme.md`; snapshot files are project-relative |

Editing rules:

- Values are load-bearing: the test asserts exact prompt counts, token sums,
  model ids, file lists, and single-skill-event extraction. Update the test
  and this table together with any fixture change.
- The second user prompt in each fixture must NOT start with a curated skill
  command (`/review`, `/security-review`, `/simplify`) or contain a
  `<command-name>` tag, so exactly one skill event is extracted per fixture.
- Keep `claude_code.jsonl` free of `Task` tool_use blocks — introducing one
  flips `subagent_usage` to the session aggregate and marks the summary
  partial, breaking `total_token_usage_including_subagents` expectations.

## OG-04 current reasoning field audit applicability

opencode_pin: 2.0.26
opencode_commit: 9b4ec5714d481559990db0a816d5dec19541a814

The current field-shape contract is the plan-20260902 inline OG-04 source
record, read together with the bounded OG-18 native-origin audit. The three
ordinary candidate fields are reasoning `state.signature`, `state.redactedData`
and `state.reasoningEncryptedContent`; names and string shapes do not prove
native origin. Source A remains NO-GO for unproved opaque archive. Compaction
`providerContext.messages` is outside the ordinary field scope.

The RG-04 `reasoning/opencode.json` is an unchanged historical synthetic
open-state / UnsupportedShape counterexample, applicable to the current pin
only for that semantic decision. Its historical note is not a current native
three-field sample. All four RG-04 fixtures and the embedded contract/digest
expectations stay frozen in OG-04. OG-05 owns the later complete state matrix,
independent canaries and production wiring; this audit does not claim them
implemented. Current pin/commit declarations must agree with OG-00, OG-04 and
the nested `opencode/MANIFEST.md`; drift, absence or duplication must fail
visibly. JSON string value UTF-8 bytes define the downstream opaque candidate
boundary, without base64 decoding, trimming or normalization. No fixture or
synthetic label grants trusted native provenance.


NPR-18 keeps the historical official `2.0.26` / `9b4ec5714d481559990db0a816d5dec19541a814` source record and all existing JSON bytes frozen. The additional `opencode/native-origin-build.json` (named `native-origin-build.json` in this directory) is the separate current public identity descriptor for the reviewed private `2.0.26-libra-native.1` darwin-arm64 artifact, build `bac21cc4027896b8d1836133c347d20f`. Its official base commit is not the patched source identity. The source-ready audit is actual single Claude PASS; Source A remains UNPROVEN, archive/native GO is false. It contains no issuer, key or per-installation policy. Neither old synthetic fixtures nor this public descriptor is real native provider proof. macOS acceptance explicitly selects `LIBRA_OPENCODE_NATIVE_BINARY`; no global fallback or Linux functional claim.

The separate `native_origin_endpoint_anthropic_v2.json` is an independently encoded, synthetic 111-byte endpoint descriptor vector for the exact first-party Anthropic Messages URL. `opencode/native-origin-build-anthropic.json` is the public, uninstalled and unregistered `2.0.26-libra-native.2` candidate identity. Its source archive and manifest describe the complete reviewed private `.2` tree; its patch and changed-path manifest describe the complete official 2.0.26-to-`.2` delta, not the incremental `.1`-to-`.2` review diff. Both fixtures contain only public hashes and synthetic protocol data. They do not contain account output, issuer keys or evidence that real `signature`/`redactedData` bytes are native; Source A and archive admission remain unproved.

## OG-17 frozen pre-v2 pending reader

`reasoning/pending_v1_decoder.rs` is the compiled pre-v2 pending reader oracle,
frozen before any v2 edits from Libra commit
`7efcf868d49a5183e7f0851a89abebee91357155`.
The original `src/internal/ai/capture/pending_payload.rs` SHA-256 was
`87f2e8879831dae5fa83e3ffac3e916e23159dfb6a08f33a865bd1ae9f5d93fb`;
the original `src/internal/ai/capture/pending.rs` SHA-256 was
`393c9c7877c15680246a2ebcd1ce8c560ed2f61881dd0586d8f7871e7488743a`.
The fixture SHA-256 at freeze is
`96d71660e8b75d61cff2dccaa59a410b90b499afd63e10c9dcc2cc678312e32c`.
The copied payload decoder/schema is production lines 1–674 (only its two
inner doc comments became ordinary comments for `include!`). The following
`pending.rs` source fragments from the same commit are compiled in the same
oracle: header lines 128–164 (SHA-256
`9435d9a6e32fc76a477cc00b5060f2f4611a8548760c9772ff8fb7663eed125a`,
only the type name changes to `FrozenPendingHeader`), closed envelope lines
166–173 (SHA-256
`38cc9f32fdf2350848036f72f9691512083d768e0ab09499d12b0a6f09dc30d5`),
capacity/depth validator lines 432–475 (SHA-256
`8450bf919640d460469705f40b8ecf3d9b29227c57f10626d966c648948f63cd`),
and the complete authenticated recovery dispatch lines 533–602 (SHA-256
`84edadb81ea26c0e429fefa4d20e5075c803cc433d5efef53c0ef84013f69335`).
It compiles through `capture::pending::frozen_pre_v2`; OG-17 VER11 and the
named route test persist a signed v2 envelope, call this old
`load_verified_payload` with the persisted header, and check that the old
reader rejects it without deleting pending bytes. Its unchanged
`load_chunks`, `PendingHeader` (the route parameter), `PendingBinding`,
`VerifiedPendingPayload`, deadline and sidecar checks, payload-binding checks,
and the scope/key-backed `AuthenticatedPendingEnvelope::verify` still come
from the live implementation. The copied payload reader also calls the live
`CaptureSnapshotProjection`, `CheckpointRedactedPayload`, `PreparedPendingAlias`,
`Redactor`, `parse_canon_value`, `LiveClaimCommitPlan`, `ReservedTurnClaim`,
`CaptureCatalogError` and `retryable_load_failure` APIs; their behavior is not
frozen here. The frozen header decoder is exercised separately against the
actual `metadata_kv` header text persisted by the signed v2 test. The oracle
therefore pins the exact
old payload decoder and old recovery dispatch, but is not an independent
snapshot of the old storage, MAC or catalog implementations. OG-17's named
route test runs the frozen dispatch against a signed v2 artifact. Do not
regenerate this oracle from a later decoder. This is
historical reader compatibility evidence, never native provider-origin
evidence.
