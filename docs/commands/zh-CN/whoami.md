# `libra whoami`

报告已保存的 Libra host 会话对应的身份。

`libra whoami` 是 Libra 自己的扩展（没有 Git 对应命令）。它查询 host 的 `/api/cli/whoami`。没有会话时返回缺少凭据的 `LBR-AUTH`；host 不可达或拒绝 token 时命令失败。它和 `cloud` / `publish` 使用的凭据无关。

## 选项

| 标志 | 说明 |
|------|------|
| `--host <url>` | 目标 host（默认内置 host） |
| `--refresh` | 为前向兼容保留；目前是空操作（每次都会向 host 校验身份，不改写已保存的 token） |

人读输出会对服务器返回的 host（`body.host`）做 URL 脱敏。`--json` 仍输出该 host 的原文。

## 示例

```bash
libra whoami --host libra.tools
```
