[English](README.md) | 中文

![Libra](docs/image/libra-banner.png)

<div align="center">

# Libra — 面向 AI Agent 的 AI 原生扩展版本控制系统

**版本化整个软件创造生命周期，而非仅仅是代码。**

</div>

Libra 是 AI 原生版本控制系统，将兼容 Git 的代码历史与外部 AI 编程 Agent 捕获结合起来。它记录受支持 Agent 的会话、检查点元数据，并在会话内容可捕获时将脱敏快照存入 `refs/libra/traces`，让开发者可以查阅变更背景，并随仓库保存这些历史。

Libra 的对象格式和传输协议兼容 Git，可融入现有开发流程。捕获 hook 支持 Claude Code、Codex 和 OpenCode；这些工具负责运行编程 Agent，Libra 负责记录其工作。

<div align="center">

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![CI](https://github.com/libra-tools/libra/actions/workflows/base.yml/badge.svg)](https://github.com/libra-tools/libra/actions/workflows/base.yml)
[![Discord](https://img.shields.io/badge/Discord-加入社区-%235865F2?logo=discord&logoColor=white)](https://discord.gg/MTbb5rDYsC)
[![X](https://img.shields.io/badge/X-%40git_mono_AI-%231DA1F2?logo=x&logoColor=white)](https://x.com/git_mono_AI)
[![文档](https://img.shields.io/badge/文档-docs.libra.tools-29B1FF)](https://docs.libra.tools)

</div>

---


coverage-v2 采集尚未发布。首个包含该能力的稳定版本将保留脱敏后的 OpenCode 可读
reasoning typed turn，普通 v1 字节保持不变。中断的旧动作先结算原回执，之后的新动作
可一次性重采 reasoning。如果已有 v2 历史的 Session 来源被拒绝，采集会停止；请使用
`libra agent doctor` 检查并新建 Session。native 证明字段仍需独立来源验证，详见
[采集命令文档](docs/commands/zh-CN/agent.md#coverage-v2-采集未发布)。


## 核心差异

| 能力 | 传统版本控制（Git） | Libra |
|-----------|----------------------|-------|
| **版本化内容** | 代码历史 | 代码历史 + 捕获的 Agent 检查点和脱敏会话记录 |
| **AI 协作** | 手动提交信息 | 外部 Agent 会话与检查点历史 |
| **知识复用** | 代码快照 | 可随仓库历史查阅的捕获上下文 |
| **安全** | 外部 GPG/SSH 配置 | 内置 Vault，每个仓库独立隔离密钥 |
| **Agent 集成** | 不适用 | Claude Code、Codex 和 OpenCode 捕获 hook |
| **自动化** | 外部 CI/CD | 仓库级 Cron 与 VCS 事件规则 |

---

## 快速开始

### 安装

```bash
# macOS（Apple Silicon）/ Linux（amd64、arm64，推荐）
# Intel macOS 无预编译二进制，请按下方步骤从源码构建。
# 安装器先验证 Ed25519 签名的 release manifest 及二进制 sha256/size 再安装；
# 信任模型见 docs/installation.zh-CN.md。
curl -fsSL https://download.libra.tools/install.sh | sh

# Homebrew（macOS）
brew install libra-tools/libra/libra

# 从源码编译（需要 Rust）
git clone https://github.com/libra-tools/libra.git
cd libra
cargo build --release --locked
```

源码构建生成 `target/release/libra`；可直接运行该二进制，或运行
`cargo install --path . --locked` 将其安装到 Cargo 的二进制目录。

脚本安装器还会创建可选短命令 `~/.libra/bin/lba -> libra`（相对 symlink）。
重复安装同一版本会修复缺失的 alias，不替换二进制。使用 `--no-alias` 或
`LIBRA_NO_ALIAS=1` 可关闭；已存在的用户自有 `lba` 绝不会被覆盖。详见
[安装器行为与选项](docs/installation.zh-CN.md)。

### 初始化你的第一个仓库

```bash
# 创建新的 Libra 仓库
libra init my-project
cd my-project

# 或从现有 Git 仓库转换
libra init --from-git-repository /path/to/existing/git/repo
```

### 使用 Agent 捕获

```bash
# 登记 OpenCode exporter 信任（请按实际安装位置调整目录）
libra agent rpc trust --dir "$HOME/.opencode/bin"
libra agent rpc trust opencode

# 安装当前仓库的捕获插件，然后检查安装状态
libra agent add opencode
libra agent list
libra agent doctor

# 在当前仓库中启动 OpenCode，完成一轮对话后退出
opencode

# 查看已捕获的会话和检查点
libra agent session list
libra agent checkpoint list
```

如果实际安装位置与上面的路径不同，请使用 OpenCode 可执行文件所在的目录。
安装插件后再启动 OpenCode；`--pure` / `OPENCODE_PURE=1` 会禁用捕获。

> 查看 [Agent 命令指南](docs/commands/zh-CN/agent.md)，了解支持的 Agent、提供商配置和会话管理。

---

## 核心特性

### 🧠 外部 Agent 捕获与 traces 历史

`libra agent` 记录受支持的外部编程 Agent 的会话和检查点。检查点元数据保存在本地索引中；会话内容可捕获时，脱敏快照作为仓库对象存储，可从 `refs/libra/traces` 到达。可用 `libra agent session`、`checkpoint` 和 `doctor` 查阅捕获状态，用 `libra agent push` 将 traces 推送到远端。

`libra review` 和 `libra investigate` 在隔离工作空间中调用受支持的外部 Agent CLI，执行只读审查与调查。`libra agent bridge --stdio` 通过 JSON-RPC 提供仓库级 DeepSeek Harness bridge。详见 [review](docs/commands/zh-CN/review.md)、[investigate](docs/commands/zh-CN/investigate.md) 和 [agent](docs/commands/zh-CN/agent.md) 指南。

原有 `libra code` UI/执行器和 `libra publish` 站点宿主已在 0.23.x 移除。外部 Agent 捕获使用 `libra agent`；仓库备份使用 `libra cloud`。

```
.libra/
├── libra.db              # SQLite：VCS 状态 + Agent 捕获元数据
├── vault.db              # 加密的签名密钥与认证凭据
├── objects/              # 对象存储（loose + pack，与 Git 兼容）
└── sessions/agent/       # 本地 Agent 捕获事件日志
```

### 🔄 Git 兼容基础

Libra 使用兼容 Git 的对象、索引和 pack 格式，支持与 GitHub、GitLab、Gitea 等服务器进行 Git 远程协作。命令和参数的支持程度各不相同；具体行为与限制见[兼容性矩阵](COMPATIBILITY.md)。

关键区别：Git 管理文件。Libra 管理**创造**。

### 🔐 Vault 安全

默认情况下，`libra init` 会创建仓库级加密密钥库并生成 **GPG 签名密钥**。**SSH 认证密钥**需另行运行 `libra config generate-ssh-key --remote origin` 生成。签名和认证密钥按仓库隔离，详见[配置指南](docs/commands/zh-CN/config.md)。

### 🛡️ 沙箱诊断

`libra sandbox status` 报告申请使用 Libra 内部沙箱的操作可用的后端与执行策略。外部编程 Agent 仍使用各自的执行与安全策略。

### ☁️ 分层云存储与备份

- **分层存储**：将大对象卸载到 S3/R2/RustFS，本地 LRU 缓存
- **云端备份**：将仓库对象、引用和已捕获的 Agent 历史同步到 Cloudflare D1 + R2
- **可移植**：在另一台机器上恢复已备份的仓库状态和捕获上下文

---

### 远端 Mega2 浏览器

`libra mega2 browser --server http://127.0.0.1:9000` 打开左侧目录树、右侧只读文件
内容的终端界面。Enter/右方向键展开目录，选中文件自动预览，Tab 切换左右焦点。
预览按代码/配置文件类型进行语法高亮，`--no-color` 可关闭颜色。
文本预览上限为 256 KiB，无需 clone 或本地仓库；脚本使用的 `--list`/`--json`
仍只返回一层目录。按键和限制见[浏览器指南](docs/commands/zh-CN/mega2.md)。
按 `a` 新建目录、`m` 移动文件或目录、`e` 重命名目录，或先按 `t` 再按 `a` 新建 tag；均无需 Shift。
名称和 tag 说明通过居中弹出框输入。移动使用独立的目标目录选择器：右方向键浏览、
第一个 Enter 选择目标、第二个 Enter 确认；Esc 返回或取消。

## 支持的外部 Agent

当前支持 **Claude Code**、**Codex** 和 **OpenCode** 捕获。运行 `libra agent list --schema-version 2 --json` 可查看安装状态和导入/导出能力。

当前 macOS 普通 OpenCode 捕获基线为官方 **OpenCode 2.0.29** 与 **Libra 0.30.49**。请按 [exporter 信任与捕获插件配置](docs/commands/zh-CN/agent.md)完成准备；`opencode --pure` 会禁用捕获。安全的普通检查点会丢弃不支持的 reasoning state 及其关联文本/时间，状态仍为 `Unproven` / `Partial`。含不支持 reasoning state 的历史导入仍返回 `LBR-AGENT-020`。该普通路径不准入原生 `signature`、`redactedData` 或 `reasoningEncryptedContent` artifact，也不证明完整 reasoning 覆盖；原生来源证明仍未通过。

> 查看 [Agent 命令指南](docs/commands/zh-CN/agent.md)，了解提供商配置、捕获边界和会话管理。

---

## 社区与资源

| 资源 | 链接 |
|----------|------|
| **官网** | [libra.tools](https://www.libra.tools) |
| **文档** | [docs.libra.tools](https://docs.libra.tools) |
| **Discord** | [加入社区](https://discord.gg/MTbb5rDYsC) |
| **X / Twitter** | [@git_mono_AI](https://x.com/git_mono_AI) |
---

## 贡献指南

我们欢迎来自开发者、AI 研究人员和所有热爱软件创造未来的人的贡献。在提交 Pull Request 之前，请确保你的代码通过我们的质量检查：

```bash
# 运行 clippy，所有警告视为错误
cargo clippy --all-targets --all-features -- -D warnings

# 检查代码格式（需要 nightly 工具链）
cargo +nightly fmt --all --check

# 如需要自动修复格式
cargo +nightly fmt --all

# 运行默认验收测试
cargo test --all
```

Windows 安装方法见 [Windows 安装说明](docs/installation.zh-CN.md#windows)。

详细贡献指南请参见 [贡献指南](docs/contributing.md)。

---

## 许可证

MIT 许可证 — 详情见 [LICENSE](LICENSE)。

Copyright (c) 2025-2026 Web3 Infrastructure Foundation.

Copyright (c) 2026 GitMono Limited.

---

<div align="center">

**[开始使用](https://docs.libra.tools/en/docs/getting-started) · [加入 Discord](https://discord.gg/MTbb5rDYsC) · [关注 X](https://x.com/git_mono_AI)**

</div>
