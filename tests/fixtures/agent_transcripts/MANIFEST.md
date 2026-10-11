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
