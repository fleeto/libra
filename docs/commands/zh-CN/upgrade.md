# `libra upgrade`

检查 Ed25519 签名的稳定发布通道，并在确认后更新当前官方安装的 Libra binary。
它没有 Git 对应命令；与自动更新共用已验证的下载和安装流程。

## 概要

```bash
libra upgrade [--check | -y|--yes]
```

命令通过固定 HTTPS origin 获取不超过1 MiB的 `stable/manifest-v1.json`，
校验签名和 key generation/control revision/trusted time 的防回退下限，再比较版本。
交互模式提示当前/新版本和下载大小；提示为 `[Y/n]`，在真实终端中按 Enter 表示同意。
确认后会重新获取并验证 manifest，通道暂停、撤销或目标版本变化时不继续安装。

下载时校验 SHA256 和大小（最多256 MiB），在运行 executable 旁暂存并执行安装前自检，
然后原子替换并执行安装后探测；探测失败恢复原 binary。新版本在下次命令生效。
较早打标的 `v0.31.0` 有可下载的 Linux/macOS arm64 二进制，但未进入稳定通道，
且缺少 v0.30.85 包含的修复。因为 0.31.0 的版本号更高，`libra upgrade` 不会将它
替换为 v0.30.85。签名稳定通道提供 v0.30.85 或包含其修复的后续发行版后，清除 `LIBRA_VERSION` / `LIBRA_BASE_URL`
覆盖值并重新运行不带 `-v` 的默认安装脚本：

```sh
curl -fsSL https://download.libra.tools/install.sh | sh
libra --version
```

macOS 系统自带的 `/usr/bin/openssl` 可能不支持 Ed25519。请安装支持 Ed25519 的
OpenSSL，并在安装脚本的 `PATH` 中将它放在最前面。例如使用 Homebrew：

```sh
brew install openssl@3
curl -fsSL https://download.libra.tools/install.sh | PATH="$(brew --prefix openssl@3)/bin:$PATH" sh
libra --version
```

确认安装器显示 `stable manifest verified`。若稳定清单或签名验证程序不可用，
拒绝未验证回退；安装兼容的验证程序或待清单可用时重试。不要设置
`LIBRA_ALLOW_FALLBACK=1`。指定 `-v` 的安装路径
也不使用签名清单验证。
从 Libra 0.30.67 起，新版 binary 首次在任意仓库执行命令时会写入不可回退的 worktree 布局能力标记，
即使该仓库没有额外的 worktree 也一样。此后 0.30.66 及更早版本在打开该仓库数据库时会拒绝所有命令；
要继续使用该仓库，须使用支持 2026101902 能力标记的构建：已发布二进制在 Linux 上始于 v0.30.67，
在 macOS/Windows 上始于 v0.30.68；重新安装旧版本不会撤销此标记。该标记本身不能保证所有中间版本都理解命名 worktree 的 registry schema 4。v0.30.85 对 registry 的任何写入都会使用 schema 4，即使没有命名条目。此后应使用 v0.30.85 或包含其 registry 与身份安全修复的后续发行版执行 worktree 命令；较早的 v0.31.0 tag 虽能解析 schema 4，却缺少这些修复，不应修改这些 worktree。
公开命令 `worktree add` 的位置参数现为 NAME；原来传目标路径的脚本须改为
`worktree add NAME --path PATH`，按路径管理 worktree 时同样使用 `--path` 选择器；
`worktree move` 用 `--from-path` 选择源，用 `--path` 指定目标。
现有 worktree 目录不会移动。
`worktree move` 也要求目标位于同一文件系统；跨文件系统请求会保留源目录和注册记录，
不再尝试可能丢失符号链接语义的复制。需要使用另一文件系统时，请新建独立 worktree 并显式转移修改。
只管理有官方 marker 的安装；dev build、改名副本或缺 marker 的文件会被拒绝并提示使用安装脚本。
此命令可在仓库外运行，不读取或修改仓库状态。

## 选项

| 标志 | 说明 |
|---|---|
| `--check` | 只报告是否有新签名版本，绝不安装；信息状态退出码为0。与 `--yes` 互斥。 |
| `-y`, `--yes` | 不询问，安装可用的新版本。 |

`--json` / `--machine` / `--quiet` 不会询问；没有 `--check` 或 `--yes` 的可更新场景会拒绝，
并提示明确选择。非交互 stdin 也不能用默认 Enter 自动同意。
标准 JSON 信封的 `data.status` 包括 `up_to_date`、`available`、`installed`、`declined`、
`paused`、`latest_revoked`、`not_official_install`、`unsupported_platform`。

已接受 manifest 的单调下限会耐久保存，即使随后拒绝安装也不会丢失；保存失败会报错。
manifest 请求预算为30秒，binary 下载预算为300秒。并发更新锁被占用或另一进程已先更新时，
命令报告没有应用并要求重跑；发布暂停、最新版本撤销或自检失败时保留/恢复原安装。

## 示例

```bash
libra upgrade
libra upgrade --check
libra upgrade --yes
libra --json upgrade --check
libra config set --global upgrade.mode manual
```

自动更新模式可设为 `auto`、`manual`、`off`；显式 `libra upgrade` 仍使用同一签名验证流程。
