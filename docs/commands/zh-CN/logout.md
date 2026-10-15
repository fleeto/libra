# `libra logout`

撤销并清除已保存的 Libra host 会话。它没有 Git 对应命令。

默认先对所选 host 调用 `/api/cli/logout`，成功后才移除全局配置中的
`account.host.<host_sha256>` 会话；请求失败则报错并保留本地令牌。
`--local-only` 跳过远端撤销，远端令牌可能继续有效直到过期。
此会话与 `libra cloud` 的凭据独立。

## 选项

| 标志 | 说明 |
|---|---|
| `--host <url>` | 目标 host，默认 `https://libra.tools`。 |
| `--all` | 删除所有 host 的本地会话。非 `--local-only` 时，先向所选 host 的撤销接口传递 `all=true`；不遍历其它 host 逐一撤销。 |
| `--local-only` | 只清除本地会话，不请求 host 撤销。 |

## 示例

```bash
libra logout
libra logout --host https://libra.tools --local-only
libra logout --all
```
