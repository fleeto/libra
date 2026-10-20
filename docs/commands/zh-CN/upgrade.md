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
从 Libra 0.31.0 起，新版 binary 首次在任意仓库执行命令时会写入不可回退的 worktree 布局能力标记，
即使该仓库没有额外的 worktree 也一样。此后 0.30.x 在打开该仓库数据库时会拒绝所有命令；
要继续使用该仓库，须使用稳定版 0.31.1 或更新版本，重新安装 0.30.x 不会撤销此标记。
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
