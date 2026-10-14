# OpenCode Hook/export source contract — synthetic fixture

opencode_pin: 2.0.26
opencode_commit: 9b4ec5714d481559990db0a816d5dec19541a814
standalone_events: session.created, session.inbox.delivered, session.execution.succeeded, session.execution.failed, session.execution.interrupted, session.deleted, session.compaction.ended
plugin_hook_events: tool.execute.after
plugin_merged_events: session.inbox.enqueued
compatibility_events: session.status, message.updated, session.compacted, server.instance.disposed
deprecated_aliases: session.idle
terminal_interruption_reasons: user, superseded, inactivity
status_filtered: busy, retry

`hook_events.json` is hand-authored against the fixed public source. It contains eight minimal Libra observation envelopes: seven standalone events and one tool hook. The enqueue name is a merged prompt source, not an additional turn start. The event vocabulary, compatibility aliases, interruption reasons and filtered statuses bind to OG-00's inline source record; OG-01 owns the actual registry/parser conformance target.

No fixture is a capture of a real session, a real model response, or proof of native reasoning origin. Current source schema uses `{id,type,created,data}`; these envelopes are the minimized Libra ingress form after observation, not complete raw durable bus records. The cwd and identifiers are synthetic.

`session.status` and `session.idle` are schema-defined compatibility inputs, with no execution publisher in this pin. `message.updated`, `session.compacted`, and `server.instance.disposed` are compatibility inputs, not advertised current upstream events. Shutdown interruption preserves restart continuity. Capture is observe-only and does not change provider sessions.

CLI `opencode session export` without `--sanitize` encodes `PublicSessionTransfer={info:PublicSessionInfo,messages:PublicSessionMessage[]}` and filters unsettled messages. PublicInfo keeps SessionInfo fields but replaces location with `{directory}`; location-switched messages replace current and previous locations the same way, excluding their workspaceID. Other message variants retain their schema. App `SessionExportData={info:SessionInfo,messages:SessionMessageInfo[]}` is independently constructed from paginated APIs; the CLI settled filter is not an app-path guarantee. Neither source audit proves live delivery or macOS exporter isolation. The 2.0.26 policy-aware plugin loader may refuse setup even when the file exists. Real end-to-end capture, native Source A, typed sinks and final platform/release gates remain separate pending work.

Owner: OG-00. Current main originally lacked this fixture directory; this card restores these no-code synthetic artifacts. Historical missing baseline is handled by EX-OG-00-BASE current artifact-boundary audit, without reconstructing or claiming a historical diff PASS. Subsequent changes must keep pin/commit and ordered vocabulary equal to the inline source record, and pass current source, privacy and boundary checks.


NPR-18 keeps the historical official `2.0.26` / `9b4ec5714d481559990db0a816d5dec19541a814` source record and all existing JSON bytes frozen. The additional `opencode/native-origin-build.json` (named `native-origin-build.json` in this directory) is the separate current public identity descriptor for the reviewed private `2.0.26-libra-native.1` darwin-arm64 artifact, build `bac21cc4027896b8d1836133c347d20f`. Its official base commit is not the patched source identity. The source-ready audit is actual single Claude PASS; Source A remains UNPROVEN, archive/native GO is false. It contains no issuer, key or per-installation policy. Neither old synthetic fixtures nor this public descriptor is real native provider proof. macOS acceptance explicitly selects `LIBRA_OPENCODE_NATIVE_BINARY`; no global fallback or Linux functional claim.

`native-origin-build-anthropic.json` records the separate uninstalled, unregistered private `.2` candidate. Its four source/provenance hashes have the same full-source and complete official-to-private meanings as `.1`, including a 65-path changed-file manifest. It is catalog input, not a real native-field fixture or authority grant. The sibling `../native_origin_endpoint_anthropic_v2.json` pins only the synthetic exact Anthropic endpoint bytes and SHA-256. No account/model run, issuer registration, native Source A proof or archive GO follows from either fixture; those checks remain local/manual outside CI.


## Current official ordinary-export source extension

current_official_opencode_pin: 2.0.29
current_official_opencode_commit: 042d0832f5f3a73eaa039a58a7d565776f6d4290

These current-source fields are distinct from the frozen `opencode_pin` and
`opencode_commit` above. All existing JSON bytes and private `.1`/`.2`
descriptors remain historical; none is relabeled as a live 2.0.29 capture or
native Source A. The new inline deterministic `opencode29_idle_fixture` in
`opencode_reasoning.rs` follows the public 2.0.29 optional idle SessionError
shape: required string type/message, optional integer status 100..599 and
optional response body. Response/unknown/malformed/opaque error branches stay
partial after safe projection. These synthetic mechanism cases are not
provider or account evidence; real Session/model checks stay macOS local.
