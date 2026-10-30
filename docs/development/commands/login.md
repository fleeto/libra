# `libra login`

**Compatibility:** `intentionally-different` — Libra host-scoped HTTP auth extension, not a Git command.

人读登录行对 `exchanged.host` 走 `redact_url_credentials`。该字段来自服务器交换响应，不是 `normalize_host` 的结果。无 userinfo 的 origin 逐字节不变。`--json` 仍输出原始 `exchanged.host`。

## Summary

`libra login` authenticates to a Libra host's `/api/cli/*` endpoints and
manages the resulting host-scoped session token (`internal::account`). It
is a Libra-only extension with no Git equivalent, and is independent of the
D1/R2/Cloudflare credentials used by `cloud`/`publish`.

Authenticates to a host and stores a session token via a PKCE-style flow. It always binds a same-machine loopback listener and waits for the browser redirect callback, so it requires a session on the local machine (remote/SSH sessions are unsupported). Flags: `--host <host>` (default host); `--no-browser` prints the login URL for you to open manually instead of launching a browser automatically — the loopback callback is still required. Times out after 15 minutes.

## Examples

```bash
libra login --host libra.tools
```
