# `libra whoami`

查询已保存的 Libra host 会话身份。它没有 Git 对应命令。

命令调用 host 的 `/api/cli/whoami` 校验身份；没有本机会话时返回缺少凭据错误，
网络不可达或令牌被拒绝时失败。会话与 `libra cloud` 的凭据独立。

## 选项

| 标志 | 说明 |
|---|---|
| `--host <url>` | 目标 host，默认 `https://libra.tools`。 |
| `--refresh` | 为前向兼容接受；当前无额外效果，身份始终由 host 校验，不重写本地令牌。 |

## 示例

```bash
libra whoami
libra whoami --host https://libra.tools
```
