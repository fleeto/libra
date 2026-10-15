# `libra login`

通过浏览器授权登录 Libra host，并保存按 host 隔离的会话令牌。它没有 Git 对应命令。

当前实现使用 PKCE 风格的浏览器跳转和 `/api/cli/*` 接口，将会话保存在全局配置的
`account.host.<host_sha256>` 中；与 `libra cloud` 的 D1/R2 凭据独立。
程序始终在本机 loopback 地址监听回调，需在同一机器完成浏览器授权；不支持远端/SSH 浏览器会话。

## 选项

| 标志 | 说明 |
|---|---|
| `--host <url>` | 目标 host，默认 `https://libra.tools`。仅接受不含路径/凭据/query/fragment 的 HTTPS origin；本机 loopback 测试可用 HTTP。 |
| `--no-browser` | 只打印授权 URL，不自动打开浏览器；仍须完成本机 loopback 回调。 |

等待浏览器授权的超时为15分钟。JSON 输出不包含 bearer session token。

## 示例

```bash
libra login
libra login --host https://libra.tools --no-browser
```
