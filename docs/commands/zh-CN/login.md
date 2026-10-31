# `libra login`

向 Libra host 认证，并保存按 host 划分的会话 token。

`libra login` 是 Libra 自己的扩展（没有 Git 对应命令）。它对本机回环监听做 PKCE 式浏览器回调，把会话 token 存在全局配置的 `account.host.<host_sha256>`。它和 `cloud` / `publish` 使用的 D1/R2/Cloudflare 凭据无关。

## 选项

| 标志 | 说明 |
|------|------|
| `--host <url>` | 目标 host（默认内置 host） |
| `--no-browser` | 打印登录 URL，由人自行打开；仍然需要本机回环回调 |

流程在 15 分钟后超时。

人读输出会先对服务器返回的 host 做 URL 脱敏。没有 userinfo 的 origin（包括没有尾部斜杠的 `https://libra.tools`）逐字节保持不变。`--json` 仍输出服务器原始 host。

## 示例

```bash
libra login --host libra.tools
```
