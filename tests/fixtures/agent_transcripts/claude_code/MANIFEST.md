# Claude ordinary reasoning fixtures

Synthetic JSONL cases for CC-07. They are not a capture of Claude Code 2.1.303.
The on-disk line schema stays UNPROVEN. Each case is one JSON object the
ordinary reader must accept without deleting unknown outer keys.

`reasoning.json` maps a case name to one JSONL line. The test feeds those
bytes to `assess_claude_ordinary_transcript` and
`normalize_claude_transcript`.
