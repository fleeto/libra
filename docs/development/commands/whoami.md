# `libra whoami`

**Compatibility:** `intentionally-different` — Libra host-scoped HTTP auth extension, not a Git command.

人读身份行对 `body.host` 走 `redact_url_credentials`。服务端字符串可以带 scheme；含 userinfo 时 userinfo 消失。`--json` 仍输出原始 `body.host`。

## Summary

`libra whoami` reports the identity for a Libra host-scoped HTTP session
token (the `internal::account` surface authenticated via `/api/cli/*`). It
is a Libra-only extension with no Git equivalent, and is independent of the
D1/R2/Cloudflare credentials used by `cloud`/`publish`.

Reports the identity associated with the stored token for a host by querying the host's whoami endpoint. Flags: `--host <host>`; `--refresh` is accepted for forward compatibility but is currently a no-op (the identity is always validated against the host on each call and the stored token is not rewritten).

## Examples

```bash
libra whoami --host libra.tools
```
