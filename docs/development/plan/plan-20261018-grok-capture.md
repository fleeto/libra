# Grok 本地会话采集计划（2026-10-18）

> **历史维护者指令（2026-10-19 10:45:31 UTC，提交/推送范围已由下条更新）：** Grok 功能采集任务暂停；授权先同步最新main，修复解析/存储架构并补齐文档，再交其他agent按后续卡继续开发。本指令覆盖旧卡中把旁路observer测试当正式checkpoint验收的勾选；保留历史事实但不追认生产采集成功。不提交、推送、发布；Codex新增/修订测试继续仅本机，禁止加入CI。

> **当前交付指令（2026-10-19 11:50:58 UTC）：** 用户已明确授权将Grok独立提交推送到origin/main及网站origin/cf，并验证部署。后续Grok开发由其他Agent负责，本会话只继续Codex；Codex WIP与本机测试不纳入本次提交/推送，不发布版本或tag。


> **起草时间：2026-10-18 15:45:00 UTC；模板 v2.14。** 本文只规划任务，不宣称实现完成。Grok 不进入 OpenCode 导出器或 Source A 状态机。采集读取 Grok 自己的会话日志。隐私策略采用 [USER-OG-THINKING-RETAIN-20261018](plan-20260902.md#user-og-thinking-retain-20261018)：普通 transcript 保存助手 Thinking 字符串，并写入未证明字段；来源保持 `Unproven` / `Partial`，不归档为 native artifact。四 provider 契约第 2 条里「未证明字段不进入 transcript」的句子，只在本计划由 ADR-GK-02 按该用户决定降级；契约的其余上限、路径、观察者、迁移和发布规则仍然有效。

> **终止链预登记（v2.14）：** 本计划纯文档记账的最终登记提交，其 D 组 CI 只能在推送后观察。SHA、run ID、结论与观察 UTC 由下一次 `plan-status.md` 例行同步补记，不在该提交自身内填写，也不为本计划专设记账提交。

## 文档职责

本文把本机 Grok CLI（`grok`，已安装 1.0.62）纳为 Libra 外部 agent 的采集对象。用户可见结果是：`libra agent` 能识别 `grok`，在用户级 hook 上只观察回合结束和压缩前的边界，并把该会话已经落盘的 Thinking 写入普通 checkpoint transcript。

### 适用范围

- 本地 Grok 会话目录 `GROK_HOME/sessions/<编码后的工作目录>/<session-id>/` 中的 `updates.jsonl` 与同目录 `chat_history.jsonl`。
- 用户级 `GROK_HOME/hooks/*.json` 里带 Libra 标记的观察 hook。
- Repository 库的 `agent_session.agent_kind` CHECK、`AgentKind`、preview 注册，以及最后一卡的 supported 提升。
- 命令文档 EN/zh、开发文档、`COMPATIBILITY.md`、能力矩阵测试，以及网站 `agent.en.md`。

### 非目标

- 不解密 `encrypted_content`，不注册 Grok issuer，不写 `reasoning/encrypted/<sha256>`，不把 Thinking 升为 Complete coverage。
- 不启动 `grok` 子进程，不调用会改会话的 ACP 方法（prompt、rewind、compact）。
- 不扫描用户主目录里的全部会话，不读取 `auth.json`、`system_prompt.txt` 或工具定义作为 transcript。
- 不把 Grok 放进 review / investigate 的可启动名单。
- 不修改 plan-20260902 的 OpenCode 行为，也不把本计划的卡插进该计划。

### 成功定义

- 一条 `agent_thought_chunk` 的字符串 `content.text` 按文件顺序进入普通 transcript，不把同一 `promptId` 下的 chunk 拼成一段。
- `chat_history.jsonl` 里 `type=reasoning` 的字符串 `id`、`status`、`summary_text` 和 `encrypted_content` 进入同一普通 transcript；metadata、日志、coverage digest、doctor JSON、类型化覆盖、Debug 和 safe projection 对这些明文零命中。
- `list/status --json` 在 GK-05 之前不把 `grok` 标成 `supported=true`。GK-05 之后 `supported=true`，`launchable_review=false`，`launchable_investigate=false`。
- 六张卡都 `Lifecycle=done` 且 `Acceptance=complete`，完成判据里的全量收口门全绿，Codex 对实现的 review 为 PASS。规划 PASS 只允许选定卡开工，不代替实现。

## 事实基线

> 行号在每张卡开工时按 ER-02 刷新。下表是 2026-10-18 的核对结果。

| 类别 | 当前事实 | 证据 |
|---|---|---|
| 身份 | `AgentKind` 闭集没有 Grok。注释写明新 agent 需要 v2 计划和 CHECK 迁移 | `src/internal/ai/observed_agents/adapter.rs:23-38` |
| 数据库 | `agent_session.agent_kind` CHECK 只有 `claude_code`、`cursor`、`codex`、`gemini`、`opencode`、`copilot`、`factory_ai` | `sql/migrations/2026050303_agent_capture.sql:16-19` |
| 迁移注册 | 内置迁移由 `builtin_migrations()` 登记；当前 tip 前缀是 `2026092902` | `src/internal/db/migration.rs:1235`、`sql/migrations/` |
| 支持名单 | 首批 supported 只有 `claude-code`、`codex`、`opencode`。首批之外不得标 `supported=true` | `docs/development/tracing/agent.md:108-116`；`src/internal/ai/observed_agents/registry.rs:338-339` |
| 实时采集 | live capture 只有 Claude、Codex、OpenCode | `src/internal/ai/observed_agents/mod.rs:217-220` |
| 防御性提取 | 非首批 kind 走 partial 空摘要 | `src/internal/ai/capture/extraction.rs:424-431` |
| Grok 版本 | 本机安装版本 `1.0.62`。外部 CLI 不要求、也不检查上游 commit SHA | `grok --version`，核对日 2026-10-18 |
| 会话布局 | `updates.jsonl` 是恢复会话的权威日志；`chat_history.jsonl` 是发给模型的原始消息 | Grok 1.0.62 user-guide `17-sessions.md` Storage Layout |
| Thinking 流 | ACP `sessionUpdate=agent_thought_chunk`，`content.type=text`，`content.text` 为字符串 | user-guide `15-agent-mode.md` Streaming updates；本机四个 `updates.jsonl` 的键统计 |
| Hook | 事件含 `Stop`、`StopCancelled`、`StopFailure`、`PreCompact`、`PostCompact`、`SessionEnd`。`transcript_path` 指向该会话的 `updates.jsonl`。daemon 路由可以不给该字段 | user-guide `10-hooks.md` Hook Events；`25-status-line.md` 对 `transcript_path` 的说明 |
| 结构统计 | 只统计键和长度，不记录正文。285 条 thought 都有字符串 `text` 和 `promptId`，没有 `chunkId`。相邻 chunk 不是前缀。126 条 `type=reasoning` 都有字符串 `id`、`encrypted_content` 和 `status=completed`；`summary` 是列表。114 条含 `summary_text` 字符串，12 条摘要为空但仍有密文。多数摘要精确等于一条 chunk；一次等于连续 4 条拼接；一条对不上。一个会话 180 条 chunk 里有 168 条不是任何摘要的子串 | 2026-10-18 本机结构统计，会话正文未写入本计划 |

### 当前缺口

| ID | 缺口 | 影响 | 证据 | 计划动作 |
|---|---|---|---|---|
| GAP-GK-01 | 外部 Grok 的上游 commit 不可控，不能当作开工条件 | 用安装版本和字段矩阵识别；形状超出矩阵则拒绝导入 | 事实基线 Grok 版本行 | GK-00 |
| GAP-GK-02 | CHECK 与 `AgentKind` 拒绝 `grok` | 不能写入 `agent_session` | `adapter.rs:30-38`、迁移 `:16-19` | GK-01 |
| GAP-GK-03 | 没有 `updates.jsonl` 读取器，Thought chunk 会丢 | 只保存摘要会丢掉未进入 `chat_history` 的 Thinking | 180 对 12 的统计 | GK-02 |
| GAP-GK-04 | 未证明密文没有普通 transcript 落点，也没有 sink 隔离 | 要么丢掉用户要求保存的字段，要么漏进 safe projection | plan-20260902 retain 节 | GK-03 |
| GAP-GK-05 | 没有只观察的 Grok hook | 回合结束不会形成 checkpoint | user-guide `10-hooks.md` | GK-04 |
| GAP-GK-06 | tracing 契约禁止把首批外 agent 标成 supported | 未改契约就提升会破坏现有守卫 | `tracing/agent.md:116` | GK-05 |

## 与其它计划的关系

| 计划/文档 | 关系 | 本计划处理 |
|---|---|---|
| `plan-long.md` | B 类外部 agent 采集，无新 LR 编号 | 日期索引加一行，不复制路线图 |
| `plan-20260902.md` | 隐私策略来源，不是代码前置 | 只消费 USER-OG-THINKING-RETAIN；不改 OpenCode 写集 |
| `plan-20260904.md` | 四 provider 执行契约与 D-CAP-STD | 除 ADR-GK-02 写明的 transcript 降级外，整份采用 |
| `plan-20260924.md` / `plan-20260925.md` | ACF-09、SCAP-01/02 已 done/complete | 复用 ingress、snapshot、finalizer；不新建 ledger |
| `plan-20261018-hermes-capture.md` 及 CX/CC/PI | 共享 capture 写集的在册计划 | DEP-GK-WRITE 单方面避让；不在本提交改那些计划正文 |
| `docs/development/tracing/agent.md` | 首批名单契约 | 只有 GK-05 可以加 `grok` 行并保持 review/investigate 关闭 |

## 评审结论与修订记录

| 维度 | 结论 | 修订动作 |
|---|---|---|
| 合理性 | 本机已有 Grok 会话和 Thinking 流，Libra 尚不能采集 | 单独立项，不塞进 OpenCode 计划 |
| 可行性 | 日志已在磁盘上，不需要导出子进程 | 不检查上游 commit SHA。形状超出 GK-00 矩阵则拒绝导入 |
| 任务卡粒度 | 六张卡各一个行为轴，生产文件按 M 上限拆开 | 共享文件只串行，不并发 |
| 依赖与顺序 | GK-00 到 GK-05 一条链，无环 | 见实施顺序 |
| 完整性 | 实现卡自带测试和命令文档；审计卡只写本计划 | 网站页只在改变用户可见行为的卡交付 |
| 安全性 | 路径监狱、不解密、sink 零明文、观察 hook | ADR-GK-02 |
| 功能正确性 | chunk 不拼接；摘要只在精确匹配或已观察的 4 条窗口内挂接 | GK-02/GK-03 分卡 |
| 接口兼容 | GK-05 之前 `grok` 不是 supported | preview 与 supported 分卡 |
| 数据流与控制流 | 沿用现有 checkpoint 全量快照和 finalizer | 不改 OpenCode 分支 |
| 性能与容量 | 沿用 16 MiB / 1 MiB 行；Thought 文本另有 8 MiB 截断标记 | 超限 partial，不报 complete |
| 可靠性与容错 | 读不完写 durable pending；rewind 和 compact 分段 | 不回写已有 checkpoint |
| 可维护性 | 新文件承载 Grok 形状，共享文件只加闭集臂 | 开工刷新锚点 |

### 修订历史

| 时间 | 触发 | 变更内容 | 原卡 → 新卡 | 受影响的引用 |
|---|---|---|---|---|
| 2026-10-18 15:45:00 UTC | 用户要求按模板新写 Grok 采集计划，隐私沿用 plan-20260902 的 Thinking 保留 | 初稿六卡 | 无 → GK-00..GK-05 | 实施顺序、依赖表、审计表、追溯表、plan-status、plan-long |
| 2026-10-18 16:05:00 UTC | Codex R1 FAIL | 补上 ER 与 G 规则节；Verification 改为完整函数名并要求非零；GK-04 落地改为 3；review log 去掉非法结果词 | 无新卡 | 执行检查、粒度规则、GK-01..05 Verification、审计表、review log |
| 2026-10-18 16:20:00 UTC | Codex R2 FAIL | ER-04 补上 PR、同一 head 的 base.yml/CodeQL、squash merge 后再对 merge SHA 发布；GK-01 只消费 GK-00 的 GO 正文；单元测试改到集成测试根函数；审计命令不再被「必须 1 passed」套住 | 无新卡 | ER-04、ER-14、Verification 判定口径、GK-00/GK-01 依赖、四张卡的测试命令 |
| 2026-10-19 | GK-00 审计 | 公开 grok-build HEAD `2bdd1d6a6369de0e8c68132ea4539e9abd9e14a8` 的 shell 是 1.0.45，对不上本机 1.0.62。DEP-GK-PIN 与 GK-01..GK-05 为 HOLD | 无新卡 | GK-00 审计结论、plan-status |
| 2026-10-19 | 版本修订号 | `grok --version` 为 `1.0.62 (1474f9cbdd91)`，该前缀在公开仓库不存在，二进制无完整 SHA | 无新卡 | GK-00 审计结论 |
| 2026-10-19 | 用户取消上游 commit 哈希限制 | 外部软件的 commit 不可控，不再做该检查。DEP-GK-PIN 不再因缺少 SHA 而 HOLD。GK-01..GK-05 按本机 1.0.62 字段矩阵 GO | 无新卡 | DEP-GK-PIN、GK-00 结论、Phase 0、故障矩阵、plan-status、plan-long |
| 2026-10-19 10:51:42 UTC | 用户暂停Grok功能并授权更新main/架构/文档再交其他agent | GK-06恢复纯解析和禁止生产observer，GK-07纯字节接续，GK-08接入仍暂停；明确旧scratch不等于checkpoint | GK-04/05历史结论 → GK-06/07/08 | ADR-GK-05、用户/开发/网站文档、守卫、状态 |

## 已决议设计决策

### ADR-GK-01: 以会话日志为唯一正文来源

- **Status:** Accepted
- **Context:** Grok 已把恢复会话用的 ACP 更新写在 `updates.jsonl`。Hook 正文不含 Thinking。`transcript_path` 指向该文件。
- **Decision:** Libra 只读这个文件和同目录的 `chat_history.jsonl`。不启动 `grok`，不走 OpenCode 式 export bridge。
- **Alternatives considered:** 沙箱里跑 `grok export`。拒绝，因为 1.0.62 的权威正文已经在 `updates.jsonl`，再起子进程只会撞上 2 秒预算和会话变更风险。
- **Consequences:** 读取器必须理解 ACP 行，而不是 OpenCode 的 `reasoning` part。
- **Revisit when:** 上游不再把 `updates.jsonl` 当作恢复日志。

### ADR-GK-02: Thinking 按 plan-20260902 降级保留

- **Status:** Accepted（用户 2026-10-18 决定）
- **Context:** 四 provider 契约第 2 条要求未证明字段不进入 transcript。plan-20260902 的 USER-OG-THINKING-RETAIN-20261018 已把普通副本改为保留助手 Thinking 和未证明字段，同时保持 `Unproven` / `Partial`，且不归档 native。用户要求本计划采用后者。
- **Decision:** 普通 checkpoint transcript 保留：每条 thought chunk 的字符串 `content.text`；reasoning 行的字符串 `id`、`status`、`summary` 里 `type=summary_text` 的字符串 `text`，以及这份密文。Grok 把密文放在 reasoning 行的 `encrypted_content`。它按 retain 节对 `reasoningEncryptedContent` 的保留规则进入 transcript，不按 OpenAI 导出键处理。非字符串丢弃并标 partial。provider metadata 不复制。来源保持 `Unproven`，快照保持 `Partial`。不制造 native artifact、Complete coverage 或 ACK。metadata、日志、coverage digest、doctor JSON、类型化覆盖、Debug、safe projection 对上述明文零命中。普通导出仍删除键名 `encrypted_content`、`metadata.openai.reasoningEncryptedContent` 和 `metadata.anthropic.signature`，与 retain 节第 182 行的导出规则相同；这个删除只作用于导出字节，不从 checkpoint transcript 里拿走 Grok 的密文。保留文本仍做密钥扫描。已经落盘的 checkpoint 不因新代码回写。Grok 源适配器采用 provider 解码后的 UTF-8 字符串，不保留 JSON 转义的词面拼写；转义与非 ASCII 用独立向量锁住。
- **Alternatives considered:** 只保留 thought 流、把密文放在默认不可读的 sidecar。拒绝，因为用户要求的隐私降级是把未证明字段写入普通 transcript，而不是再发明一套归档。
- **Consequences:** 本计划偏离四 provider 契约第 2 条的「不进入 transcript」一句。偏离只覆盖 Grok 普通 transcript 的这些具名字段。不放宽路径监狱、不解密、不从未知 blob 构造 `OpaqueEncryptedBytes`、不提高字节上限。
- **Revisit when:** 用户恢复「未证明字段不得进入 transcript」，或 Grok 提供可验证的来源证明并另立 Source A 计划。

### ADR-GK-03: chunk 是事件，摘要不是完整 Thinking

- **Status:** Accepted
- **Context:** 本机统计里，相邻 chunk 不是前缀；把一个 `promptId` 下全部 chunk 拼起来对不上摘要。多数摘要等于单独一条 chunk。168/180 条 chunk 没有进入摘要。
- **Decision:** 每条字符串 chunk 单独成为一条 thought 事件，按文件顺序保留。摘要文本只在两种情况下挂到已有事件而不再写第二份正文：与某一条 chunk 完全相同，或与同一 `promptId` 内连续最多 4 条 chunk 的顺序拼接完全相同。对不上的摘要单独成条。空摘要只留下密文字段。禁止按时间戳猜测对应关系。
- **Alternatives considered:** 总是拼接一个 prompt 的全部 chunk。拒绝，统计已经证明这样会制造会话里不存在的文本。
- **Consequences:** GK-02 不读 `chat_history.jsonl`。挂接只在 GK-03。
- **Revisit when:** 以后的已安装版本证明 chunk 是增量 delta。在该证明写入本计划之前，实现不得拼接。

### ADR-GK-04: 身份先 preview，支持最后提升

- **Status:** Accepted
- **Context:** tracing 契约禁止把未列入首批表的 slug 标成 supported。CHECK 与 enum 必须同时发布，否则新二进制写不出、旧二进制必须零写拒绝。
- **Decision:** GK-01 加入 `AgentKind::Grok` / db 值 `grok` / slug `grok`，`supported=false`，无 hook。GK-05 才把 supported 和 hook_installable 设为 true，并改 tracing 契约。全程 `launchable_review=false`、`launchable_investigate=false`。
- **Alternatives considered:** 第一张实现卡就对外宣称 supported。拒绝，因为读取器和 hook 尚未存在。
- **Consequences:** GK-01 的用户可见变化只是身份可以被识别为未支持的 preview，enable 仍返回 unsupported。
- **Revisit when:** 不提前。GK-05 的本机合成测试未绿时不得提升。

## 全局工程约束

本文全部任务采用 plan-template.md 的 GC-01..GC-13 和 ER-01..ER-14，并采用 [四 provider 执行契约](plan-20260904.md#capture-review-20261018)，只有 ADR-GK-02 写明的那一句例外。另加：

- 测试 fixture 只用合成 JSONL。不得打开真实 `GROK_HOME`、真实会话或凭据。
- 生产路径不新增 `unwrap()` / `expect()` / `panic!()`。
- 新 migration id 在开工时取严格大于当时 tip 的下一个空号。不得复用 `2026092902` 或更早的 id。
- 发布者是当时被选中执行该卡的 Agent。未指派个人姓名。同一时刻只有一张卡处于已 bump 未推送完。

## 执行检查必备需求（强制）

下列 ER-01..ER-14 与 `plan-template.md` v2.14 正文同义，适用于本计划每一张卡。这里只绑定本计划的取值，不删除模板里的门。

1. **ER-01：** 开工先跑 `libra status --short --branch`。分支是 `main`。目标文件已有别人的改动就停。
2. **ER-02：** 先刷新本卡锚点、测试名和 Grok pin，再写代码。
3. **ER-03：** 开工前核对审计表与该卡 `Granularity`。范围变大就先改计划。
4. **ER-04：** A 组是该卡 Verification 里的 focused 命令。B 组按 Task type。C 组由会发版的卡执行，顺序是：版本面 parity、`patch + 1`、工具链刷新 `Cargo.lock`、fmt、clippy、按 ER-13 的测试、`cargo build --release`、安装、精确 `libra add`、`libra commit -s`、推送 feature branch、`gh pr create`、同一 head SHA 上 `base.yml` 与 CodeQL 全绿、`gh pr merge --squash`、先 `libra fetch origin`，让 squash merge SHA 在本地可解析。再执行 `libra tag -s -m "Release v${VERSION}" "v${VERSION}" "$MERGE_SHA"`。`-m` 使它成为 annotated tag，`-s` 用仓库 vault 对它签名。`docs/commands/tag.md` 写明没有 `-a`/`-m`/`-F` 时只创建未签名的轻量 tag，且 `-s` 必须和 `-m` 一起用。然后 `libra tag -v "v${VERSION}"` 必须退出 0，再 `libra push origin "v${VERSION}"`，最后 `gh release create "v${VERSION}" --verify-tag`。不使用 GitHub API 创建的未签名轻量 tag。D 组是推送之后的远端证据：`release.yml` 四平台与安装脚本、CodeQL、Homebrew、manifest，按 D-CAP-STD 登记 workflow、job、event/ref 和 SHA。`locally-accepted` 不是完成。有 D 组时先停在 `remote-pending`。
5. **ER-05：** 实现和本地验收之后做 Codex review，直到字面 PASS。P0/P1 必须关闭。
6. **ER-06 / ER-06a：** 用户可见行为只在 GK-05 出现。该卡同批改 EN、zh-CN、开发文档、`COMPATIBILITY.md` 和网站 `agent.en.md`。网站写入前确认 `/Volumes/Sea/libra-backend` 的 VCS 与 `cf`。
7. **ER-07：** `libra add <相关路径>` 然后 `libra commit -s`。提交后用同一 SHA 检查 `gpgsig` 与 `Signed-off-by`。不用 `commit -a`，本仓库没有 `git commit -S`。
8. **ER-08：** GK-01..GK-05 各自 `PATCH + 1`，同步 `compat_version_surface_sync` 报告的三处版本面。不手改 `Cargo.lock`。
9. **ER-09：** push 不是 fast-forward 时先拉再验。认证失败不盲重试。
10. **ER-10：** 确定性错误不重试。暂时性错误最多 5 次、合计 30 分钟。发布动作不自动重试。
11. **ER-11：** 证据不写密钥、正文、绝对家目录或原始 tool payload。
12. **ER-12：** 发布串行。没有仓库级发布锁，本计划不并发发布。
13. **ER-13：** 执行阶段跑该卡 focused 测试。命中 `Full-suite trigger` 的卡在 bump 后跑 `source .env.test && source .env.live-test && cargo nextest run --all --no-fail-fast --retries 2`。全部卡完成后同一条命令再跑一次收口。
14. **ER-14：** 发布证据只用 nextest。`cargo test --all` 不能充当发布通过证据。`.env.test` 与 `.env.live-test` 必须能 `source`。C 组在 bump 后的树上，用 `cargo nextest run` 重跑该卡 Verification 里的同一完整测试名；全量触发的卡改为双环境 `cargo nextest run --all --no-fail-fast --retries 2`。

## 任务卡粒度规则（强制）

G-01..G-11 与模板相同。本计划的取值如下。

- **G-01：** 一卡一轴。GK-01 是 `forward-only`，其余是 `revert`。
- **G-02：** 实现、测试和该卡自己的文档在同一张卡。GK-00 只有审计正文。
- **G-03：** implementation / migration 的 AC 与 VER 上限都是 8。audit 上限是 20。计数的是独立谓词和独立命令。ER-04 的 fmt、clippy、全量和 ER-06a 的同卡文档门不计入。本计划每张卡的勾选数已经写进审计表。
- **G-04：** 开工只允许 S 或 M。落地按目录计，生产文件不含测试、命令文档和 README。超过 4 个落地或 12 个生产文件就先拆卡。
- **G-05：** 卡内写清锚点、命令和依赖，不写「见上卡」。
- **G-06：** 依赖只有 `GK-00 -> GK-01 -> GK-02 -> GK-03 -> GK-04 -> GK-05` 和已登记的 `DEP-GK-*`。
- **G-07 / G-07a：** 没有批量推迟 bump。GK-00 是 no-release。其余卡独立发布。
- **G-08：** 没有家族卡。
- **G-09：** 本轮没有拆已引用的卡。
- **G-10：** 写集相交的卡只串行，不并发。
- **G-11：** 类型就是各卡标题下的 Task type。

### Verification 判定口径

- 测试命令必须至少跑到 1 个测试。输出里出现 `0 passed`，或者过滤后没有匹配，就算失败。`--exact` 只和该测试二进制里的完整函数名一起用。`--lib` 不用裸函数名。审计卡的人工阅读和 `libra status` 不跑测试，按各自命令的退出码和计划里写下的矩阵判定。
- 零命中的 `rg` 使用模板：`rg` 退出码 0 是有命中，1 是零命中，大于 1 是命令失败。`! rg` 不能当证据。
- 全量 nextest 不写进 Verification 清单。它由 `Full-suite trigger` 或收口门覆盖。
- 新测试名在命令里标了 `(new)` 的，由该卡创建同名函数。

## 依赖登记表

| ID | direction | 类型 | 对象 | Owner | 产物与可用性判据 | 证据 | 超时与失败策略 |
|---|---|---|---|---|---|---|---|
| DEP-GK-PIN | incoming | 上游 release | 已安装 Grok 的版本字符串和本机 1.0.62 字段矩阵。不要求、也不检查 `xai-org/grok-build` 的 commit SHA | GK-00 审计者 | 计划内写出版本字符串和字段矩阵。缺少上游 SHA 不是 HOLD | 本机 `grok --version` 与 1.0.62 结构统计，2026-10-18 | 用户 2026-10-19 取消哈希限制。形状超出矩阵时由 GK-02 拒绝导入，不改用浮动 main 冒充另一份源码 |
| DEP-GK-BASE | incoming | 跨计划 | plan-20260924 ACF-09 与 plan-20260925 SCAP-01/02 | 选中卡的执行者 | 开工时两卡仍为 done/complete，且 `CaptureFinalizePolicy` 可注入 | plan-status 对应行 | 缺失则 GK-02 及之后 blocked；GK-00/GK-01 仍可做 |
| DEP-GK-DB | incoming | 跨计划 | plan-20260910 MIG-02/03 的 role manifest，以及 ADR-CAP-CHECK 的专用连接是否已在源码可用 | schema 执行者 | Repository-only、Global/System 零写可被现有测试证明 | `src/internal/db/migration.rs:615-653` 一带的 runner；开工重读 | 专用 CHECK 路径不存在则 GK-01 blocked，不在池内事务里 DROP 父表 |
| DEP-GK-WRITE | incoming | 跨计划 | plan-20260904、plan-20260905、plan-20260911、plan-20261018-hermes-capture 的共享 capture / registry / agent 文档写集 | 选中卡的执行者 | `libra status --short --branch` 显示目标文件没有别人的未交接改动 | 上述计划的写集节 | 脏或 in-progress 则停。对方计划正文未列出本计划时，本计划仍然避让 |
| DEP-GK-DOC | incoming | 外部仓库 | `/Volumes/Sea/libra-backend` 的 `cf` 与 `commands/agent.en.md` | 文档执行者 | 写入前按 ER-06a 记录 VCS 与 `cf`；typecheck/build 通过后才推 | 四 provider 契约第 10 条 | 分支或工作区不允许写入则该卡停在 locally-accepted，不假装网站已更新 |
| DEP-GK-RELEASE | incoming | 发布 | D-CAP-STD 的签名 tag、base.yml、codeql、release.yml | 单一发布者 | 固定 source SHA 的 job 全绿才把 Acceptance 标 complete | plan-20260904 D-CAP-STD | 失败前滚；未触发不算通过 |

## 实施顺序

计划内顺序不进依赖登记表：`GK-00 -> GK-01 -> GK-02 -> GK-03 -> GK-04 -> GK-05`。这条边表示前一张卡的交付物已经写入，不是要求前一张卡的 `Acceptance=complete`。GK-00 的 GO 矩阵写完后 GK-01 就可以开工。GK-00 自己的 `complete` 要等 GK-01 的 C/D，因此 GK-01 不得把 GK-00 的 `complete` 当作开工条件。

## 发布分组与并发窗口

无 `REL-*`。每张 implementation / migration 卡独立 `patch + 1` 并走 `gh` 发布。GK-00 是 no-release。

**并发声明:** 全串行。GK-02 起多张卡都会碰 `observed_agents/mod.rs` 和 agent 文档，G-10 不允许并发。

**发布者:** 当时被选中执行该卡的 Agent。

**发布窗口顺序:** GK-01，然后 GK-02，然后 GK-03，然后 GK-04，然后 GK-05。前一张的 tag 已触发且没有「已 bump 未推送」残留，才开始下一张的 bump。

### Phase 0: 来源钉死

**目标:** 把已安装 1.0.62 的字段矩阵写成后继卡的合同。不检查上游 commit SHA。

**进入条件:** 本计划规划审查 PASS，并且用户选中 GK-00。

**退出条件:** 版本字符串和字段矩阵已写入本计划，GK-01..GK-05 为 GO。

### Phase 1: 身份

**目标:** Repository 接受 `grok` 且对外仍 unsupported。

**进入条件:** GK-00 对身份字段 GO，DEP-GK-DB 可用。

**退出条件:** 新迁移后的库接受 `grok`，旧值仍在，Global/System/Derived 版本不变，supported 仍为 false。

### Phase 2: 正文

**目标:** 普通 transcript 含 Thought 和未证明字段，sink 不含明文。

**进入条件:** GK-01 已 source-qualified。formal complete 仍等该卡自己的 C/D。

**退出条件:** 合成 fixture 上的保留、挂接和零命中断言通过。

### Phase 3: 观察 hook 与支持提升

**目标:** 用户级 hook 在回合结束和压缩前读日志；契约允许 `grok` supported。

**进入条件:** GK-03 的保留规则已在合成数据上通过。

**退出条件:** enable/disable 只动带标记的用户级文件；`supported=true` 且 review/investigate 仍为 false。

## 任务卡粒度规则的字段默认

- **Release boundary 默认:** independent
- **Task type 默认:** implementation
- **Rollback mode 默认:** revert
- **Migration and rollback 默认:** N/A
- **Security and privacy 默认:** ADR-GK-02 与四 provider 契约
- **Performance budget 默认:** source 16 MiB，单行 1 MiB，Thought 文本 8 MiB，hook 内读取预算 2 秒

无 EX waiver。

### 默认覆盖

| 任务 | 偏离的字段 | 取值与理由 |
|---|---|---|
| GK-00 | Release boundary / Task type / Version increment | audit、no-release、N/A。产物只进本计划 |
| GK-01 | Task type / Rollback mode | migration、forward-only。CHECK 扩展不丢已有行 |

## 任务卡粒度审计表

| 任务 | type | axis | recovery | complete | self-contained | AC | VER | landing / prod-files | scope | deps | writeset | release | split-from | exception |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| GK-00 | audit | 记录已安装 Grok 1.0.62 的日志与 hook 形状，不检查上游 commit | revert 本卡文档 | yes | yes | 5/20 | 2/20 | 0/0 | M | DEP-GK-PIN | no-overlap | no-release | N/A | N/A |
| GK-01 | migration | Repository 接受 grok 身份且仍 unsupported | 前滚修复迁移，不删用户行 | yes | yes | 7/8 | 2/8 | 4/8 | M | GK-00, DEP-GK-DB, DEP-GK-WRITE | 序列化于共享 schema | independent | N/A | N/A |
| GK-02 | implementation | 按序保留 thought chunk | revert 本卡提交 | yes | yes | 8/8 | 4/8 | 1/2 | M | GK-01, DEP-GK-BASE, DEP-GK-WRITE | 序列化于 GK-01 | independent | N/A | N/A |
| GK-03 | implementation | 未证明字段进入普通 transcript 且离开 sink | revert 本卡提交 | yes | yes | 7/8 | 2/8 | 1/2 | M | GK-02, DEP-GK-WRITE | 序列化于 GK-02 | independent | N/A | N/A |
| GK-04 | implementation | 用户级只观察 hook | revert 本卡提交并卸标记 hook | yes | yes | 6/8 | 2/8 | 2/4 | M | GK-03, DEP-GK-PIN, DEP-GK-WRITE | 序列化于 GK-03 | independent | N/A | N/A |
| GK-05 | implementation | 将 grok 升为 supported 且不开放 launcher | revert 本卡提交回到 preview | yes | yes | 7/8 | 3/8 | 2/5 | M | GK-04, DEP-GK-DOC, DEP-GK-RELEASE, DEP-GK-WRITE | 序列化于 GK-04 | independent | N/A | N/A |

## 任务卡

### Task GK-00: Grok 1.0.62 来源形状审计

**Task type:** audit

**Lifecycle / Acceptance:** locally-accepted / 空

**Description:** 记录本机 1.0.62 的会话日志和 hook 字段，并给每个后继能力 GO 或 HOLD。2026-10-19 用户取消上游 commit 哈希检查。后继卡按字段矩阵 GO。Acceptance 仍空，因为 C/D 要等 GK-01。

**Out of scope:** 不改生产代码；不读真实会话正文；不安装新的 Grok；解密永久非目标。

**Current evidence:**

| 事实 | 证据 |
|---|---|
| 闭集没有 grok | `src/internal/ai/observed_agents/adapter.rs:30-38` |
| 本机版本与指南章节 | 事实基线 Grok 版本行、user-guide `17-sessions.md`、`15-agent-mode.md`、`10-hooks.md` |
| 结构统计不含正文 | 事实基线结构统计行 |

### GK-00 审计结论（2026-10-19）

公开仓库 `xai-org/grok-build` 最新提交是 `2bdd1d6a6369de0e8c68132ea4539e9abd9e14a8`（2026-09-29，message `Synced from monorepo`）。该树的 `crates/codegen/xai-grok-shell/Cargo.toml` 版本是 `1.0.45`。根上 `SOURCE_REV` 是 `559751fdcec02d413e4c57c8832ab275e4f44980`，这是 1.0.45 同步时的 monorepo SHA，不是 1.0.62。公开历史 51 个提交都更早，最新 crate 版本仍是 1.0.45。本机安装版本是 `1.0.62`（2026-10-18 核对）。同一路径的用户指南 `17-sessions.md` 不相等：本机 SHA-256 `95c1893520877457aabcf9fa7286dd6f22c83d2dad792c020dd579e7ca520728`，公开 HEAD SHA-256 `eab0fe715f74f2e379d3ee4f68e68d095aee64d78348f9ac25629619caceafc5`。本机 `grok --version` 打印 `grok 1.0.62 (1474f9cbdd91)`。二进制里只有这 12 位，没有 40 位完整 SHA。GitHub 对 `xai-org/grok-build` 查询 `1474f9cbdd91` 返回 “No commit found for SHA”。这些事实只说明公开仓库落后于已安装版本。2026-10-19 用户取消「必须钉住 `xai-org/grok-build` commit SHA」的限制：外部软件的提交哈希不可控，Libra 不检查它。不用浮动 `main` 冒充 1.0.62，也不再因为缺少 SHA 而 HOLD。

本机 1.0.62 的字段矩阵是 parser 和 hook 的合同：

| 表面 | 键 | 本机 1.0.62 观察 |
|---|---|---|
| `updates.jsonl` 行 | `timestamp`、`method`、`params` | `method` 为 `session/update` 或 `_x.ai/session/update` |
| `params` | `sessionId`、`update`、`_meta` | `_meta` 含 `eventId`；thought 行有 `promptId`，没有 `chunkId` |
| `update` | `sessionUpdate=agent_thought_chunk` | `content.type=text`，`content.text` 为字符串 |
| `turn_completed` | `usage.reasoningTokens` | 只是计数，不是 Thinking 正文 |
| compact | `auto_compact_started`、`compaction_checkpoint`、`auto_compact_completed` | 出现过；`compaction_checkpoint` 有 `checkpoint_id` 与 `checkpoint_file` |
| `chat_history.jsonl` | `type=reasoning` | 字符串 `id`、`status`、`summary` 列表、字符串 `encrypted_content` |
| `summary[]` | `type=summary_text` | 字符串 `text`；也见过空列表 |
| hook | `transcript_path` | 指向该会话的 `updates.jsonl`。daemon 可以不给这个字段。用户级 hook 不要求目录信任；项目级需要 |

相邻 chunk 不是前缀。一个 `promptId` 下全部拼接对不上摘要。结论保持 **不拼接**。公开 1.0.45 没有被用来推翻 ADR-GK-03。

Hook：`transcript_path` 是会话自己的 `updates.jsonl`。daemon 路由可以缺该字段。用户级 `GROK_HOME/hooks/*.json` 始终运行；项目级 `.grok/hooks` 还要目录信任。这些句子来自本机 1.0.62 指南。

| 后继卡 | 结论 | 原因 |
|---|---|---|
| GK-01 | GO | 身份迁移只消费字段矩阵里的 kind 名，不依赖上游 SHA |
| GK-02 | GO | parser 按本表实现。形状超出本表则拒绝导入 |
| GK-03 | GO | 同上 |
| GK-04 | GO | hook 事件名和 `transcript_path` 规则已写入本表 |
| GK-05 | GO | 依赖 hook 的 GO |

**Acceptance criteria:**

- [x] 上游 commit SHA 检查已取消。计划记录已安装版本字符串和字段矩阵，不因缺少 `xai-org/grok-build` SHA 而 HOLD。
- [x] 字段矩阵覆盖 `updates.jsonl` 行包装、`agent_thought_chunk`、`_meta.eventId`、`promptId`、`turn_completed.usage.reasoningTokens`、compact 事件，以及 `chat_history` 的 reasoning 键。
- [x] chunk 是否为 delta 有结论。在新证据推翻 ADR-GK-03 之前，结论保持「不拼接」。
- [x] hook 的 `transcript_path`、daemon 缺字段、用户级与项目级信任差异分开记录。
- [x] 每个后继卡的能力标 GO 或 HOLD。HOLD 的卡不得开工。

**Verification:**

- [x] 公开 HEAD `2bdd1d6a6369de0e8c68132ea4539e9abd9e14a8` 与本机 1.0.62 的差异，以及用户取消 SHA 检查后的 GO 矩阵，已写入本节。
- [ ] `libra status --short --branch` 证明本卡只改本计划与 `plan-status.md` 的本卡状态。推送后的证据写在审计记录里，不在提交前勾选。

**Full-suite trigger:** N/A（no-release 审计，不执行 C 组）

**Dependencies:** DEP-GK-PIN

**Deliverables:** 本计划中的版本字符串、字段矩阵、GO/HOLD 表。不交付上游 commit SHA。

**Implementation write set:** `docs/development/plan/plan-20261018-grok-capture.md`；`docs/development/plan/plan-status.md` 仅本卡状态。

**Release write set:** N/A

**Files likely touched:** 上列两个计划文件。

**Docs and compatibility impact:** N/A。本卡不改变用户可见命令、输出或错误。

**Rollback mode:** revert

**Migration and rollback:** 撤回本卡写入的审计段落，不删历史修订行。

**Security and privacy:** 审计不复制会话正文、绝对家目录路径或凭据。

**Performance budget:** N/A

**Estimated scope:** M（≤2 人日）

**Version increment:** N/A

**Release boundary:** no-release

**C/D coverage from:** GK-01

**Granularity:** `type=audit; axis=记录已安装 Grok 1.0.62 的日志与 hook 形状，不检查上游 commit; recovery=revert 本卡文档; complete=yes; self-contained=yes; AC=5/20; VER=2/20; landing=0; prod-files=0; scope=M; deps=DEP-GK-PIN; writeset=no-overlap; release=no-release; split-from=N/A; exception=N/A`

### Task GK-01: Repository grok 身份迁移

**Task type:** migration

**Lifecycle / Acceptance:** locally-accepted / 空

**Description:** 让 Repository 库接受 `agent_kind=grok`，CLI slug `grok` 能解析，但 list/enable 仍报告 unsupported。

**Out of scope:** 不读日志，不安装 hook，不把 supported 设为 true。由 GK-02、GK-04、GK-05 承接。云端 D1 白名单未覆盖新 kind 时，本卡不上传该 kind；永久不在本卡做 cloud mirror。

**Current evidence:**

| 事实 | 证据 |
|---|---|
| CHECK 七值 | `sql/migrations/2026050303_agent_capture.sql:16-19` |
| enum 与 live 臂 | `adapter.rs:30-38`、`mod.rs:217-220` |
| 首批断言 | `registry.rs:338-339` |
| 非首批提取 | `capture/extraction.rs:424-431`、`observed_agents/extract.rs` 中同一组防御臂 |

**Acceptance criteria:**

- [x] 新 migration id 大于开工时的 tip，只扩展 Repository 的 `agent_session.agent_kind` CHECK，保留全部旧值并加入 `grok`。
- [x] 迁移走 ADR-CAP-CHECK 的专用连接；失败丢弃该连接，不把 `foreign_keys=OFF` 还回池。
- [x] 隔离的 GlobalConfig、SystemConfig 与 Derived 临时库版本 receipt 和配置表不变，且这三类库零写。`grok_agent_kind_round_trip_and_check` 覆盖本条。
- [x] `AgentKind::Grok` 的 db 值是 `grok`，slug 是 `grok`。`agent_for` 返回放在 `mod.rs` 里的 `PreviewAgent`，不加入 `PREVIEW_SPECS`。因此 `is_preview(Grok)` 在本卡为 false，这是有意的，不得为此去改 `preview.rs`。该类型没有 hooks 字段，`as_hooks()` 默认是 `None`。`agent_for_returns_matching_kind_for_every_variant` 对 Grok 的稳定性断言在本卡改为 `Preview`，GK-05 换上专用 struct 后再恢复为全部 `Stable`。`live_capture_for` 返回 `None`，因为 `hook_installable` 仍为 false。`live_capture.rs` 里「live 臂存在当且仅当 hook_installable」的断言保持成立。两处提取臂给出 partial 空摘要。
- [x] `supported_slugs()` 仍正好是 `claude-code`、`codex`、`opencode`。`enable grok` 返回 unsupported，不写 hook 文件，不写 `agent_session`。
- [x] 已有七个 kind 的行在迁移后仍可读。库里已有 `grok` 行时，down 拒绝执行并保留行。
- [x] 不含 `grok` 枚举的旧读取器打开含 `grok` 行的库时零写拒绝。这条也由 `grok_agent_kind_round_trip_and_check` 断言。

**Verification:**

- [x] `source .env.test && cargo test --test grok_agent_kind_test grok_agent_kind_round_trip_and_check -- --exact`（new，函数写在 `tests/grok_agent_kind_test.rs` 根上）。输出含 `1 passed`，不含 `0 passed`。
- [x] `source .env.test && cargo test --test compat_agent_capability_matrix_pin known_agent_capability_matrix_matches_current_roster -- --exact`。输出含 `1 passed`，不含 `0 passed`。

**Full-suite trigger:** T-1: 本卡修改 `sql/migrations/**` 与共享 `AgentKind` 事实源。

**Dependencies:** GK-00 已经写入本计划的身份字段 GO。不依赖 GK-00 的 `Acceptance=complete`。另加 DEP-GK-DB、DEP-GK-WRITE。

**Deliverables:** N/A

**Implementation write set:** 开工时命名的 `sql/migrations/<新id>_agent_kind_grok.sql` 与配对 down 文件；`sql/migrations/README.md` 的 Current registry 表加一行；`src/internal/db/migration.rs`；`src/internal/ai/observed_agents/adapter.rs`；`src/internal/ai/observed_agents/mod.rs`（Grok 用 `PreviewAgent` 静态，`as_hooks()` 保持默认 `None`，`live_capture_for` 为 `None`）；`src/internal/ai/observed_agents/registry.rs`（新行 `supported=false`、`hook_installable=false`，不改首批三名断言）；`src/internal/ai/observed_agents/extract.rs`；`src/internal/ai/capture/extraction.rs`；`tests/grok_agent_kind_test.rs`；`tests/INDEX.md` 加 `grok_agent_kind_test` 一行。不改 `stable_promoted.rs`，不改命令行为文档。本计划与 `plan-status.md` 只更新本卡状态。

**Release write set:** Inherited

**Files likely touched:** 上列生产文件与对应单元测试。

**Docs and compatibility impact:** N/A。`grok` 仍 unsupported，不新增用户可见成功路径。enable 的既有 unsupported 文案不变。

**Rollback mode:** forward-only

**Migration and rollback:** 不变量是旧七值都还在，用户行不删。恢复验证是迁移测试重放后 COUNT 与 kind 集合一致。已发布的库只能再发一条前滚修复。down 仅在零条 `grok` 行时恢复旧 CHECK，有行则失败退出。

**Security and privacy:** 不读 Grok 家目录。db 值只是标签，不携带 transcript。

**Performance budget:** Inherited。迁移只重建 `agent_session` 这一张表的 CHECK，子表按 ADR-CAP-CHECK 预置并在故障注入后核对行数。

**Estimated scope:** M

**Version increment:** patch

**Release boundary:** independent

**C/D coverage from:** self

**Granularity:** `type=migration; axis=Repository 接受 grok 身份且仍 unsupported; recovery=前滚修复迁移，不删用户行; complete=yes; self-contained=yes; AC=7/8; VER=2/8; landing=4; prod-files=8; scope=M; deps=GK-00,DEP-GK-DB,DEP-GK-WRITE; writeset=序列化于共享 schema; release=independent; split-from=N/A; exception=N/A`

落地目录四个：`sql/migrations/`、`src/internal/db/`、`src/internal/ai/observed_agents/`、`src/internal/ai/capture/`。生产文件 8 个：两条 SQL、`migration.rs`、`adapter.rs`、`mod.rs`、`registry.rs`、`extract.rs`、`extraction.rs`。`README.md` 不计入。不碰 `builtin/`。

### Task GK-02: updates.jsonl 的 Thought 保留

**Task type:** implementation

**Lifecycle / Acceptance:** locally-accepted / 空

**Description:** 读取一份合成 `updates.jsonl`，把每条 `agent_thought_chunk` 的字符串正文按出现顺序放进普通 transcript。

**Out of scope:** 不读 `chat_history.jsonl`（GK-03）。不安装 hook（GK-04）。不拼接 chunk。`reasoningTokens` 只作为计数进入 turn 事件，不当成 Thinking 正文。

**Current evidence:**

| 事实 | 证据 |
|---|---|
| live 臂没有 Grok | `mod.rs:217-220` |
| thought 形状 | ADR-GK-01、ADR-GK-03 与事实基线结构统计 |

**Acceptance criteria:**

- [x] 合成文件里三条 thought 在 transcript 里保持三条，文本与顺序一致，中间的 tool 事件仍在原位。
- [x] 同一 `promptId` 的多条 chunk 不会被拼成一条。
- [x] 非字符串 `content.text` 丢弃，该回合 partial，其余字符串 thought 仍在。
- [x] 未知 `sessionUpdate` 保留类型名和原始行哈希，不解释成 thought。
- [x] 单行超过 1 MiB 或整份 source 超过 16 MiB 时 partial 或拒绝，不报 complete。
- [x] 路径不是普通文件、是符号链接，或规范化后不在调用方给出的会话目录内，则拒绝读取。
- [x] 含 `\"`、`\u754c` 与非 ASCII 的 thought 以解码后的 UTF-8 进入 transcript。`grok_updates_decoded_utf8_escape_and_unicode` 覆盖本条。
- [x] 同一 `eventId` 再读一次不新增 thought。进程重启后从已记录偏移继续，不按时间戳合并。

**Verification:**

- [x] `source .env.test && cargo test --test grok_capture_test grok_updates_keeps_three_thoughts_in_order -- --exact`（new）。输出含 `1 passed`，不含 `0 passed`。
- [x] `source .env.test && cargo test --test grok_capture_test grok_updates_string_chunks_are_not_concatenated -- --exact`（new，同一测试文件根上）。输出含 `1 passed`，不含 `0 passed`。
- [x] `source .env.test && cargo test --test grok_capture_test grok_updates_decoded_utf8_escape_and_unicode -- --exact`（new）。输出含 `1 passed`，不含 `0 passed`。
- [x] `source .env.test && cargo test --test grok_capture_test grok_updates_repeat_event_id_is_idempotent -- --exact`（new）。输出含 `1 passed`，不含 `0 passed`。

**Full-suite trigger:** T-1: 接入共享 `observed_agents` 模块树和 capture transcript。

**Dependencies:** GK-01；DEP-GK-BASE；DEP-GK-WRITE。GK-00 对 thought 形状必须是 GO。

**Deliverables:** N/A

**Implementation write set:** `src/internal/ai/observed_agents/grok_updates.rs`；`src/internal/ai/observed_agents/mod.rs` 只增加 `pub mod grok_updates`。不改 `live_capture_for`。测试直接调用读取函数。`tests/grok_capture_test.rs`；`tests/INDEX.md` 加 `grok_capture_test` 一行。本计划与 `plan-status.md` 只更新本卡状态。

**Release write set:** Inherited

**Files likely touched:** 上列文件。

**Docs and compatibility impact:** N/A。本卡没有新的 CLI 成功路径；`grok` 仍 unsupported。用户文档留到 GK-05，那时 supported 才变为真。

**Rollback mode:** revert

**Migration and rollback:** 一次 revert 去掉读取器。不回写已有 checkpoint。

**Security and privacy:** 只接受调用方打开的目录内的普通文件。测试数据是合成字符串。Thought 文本进入普通 transcript 是 ADR-GK-02 的要求；本卡还不处理密文。

**Performance budget:** Inherited。读取按字节偏移增量进行，不把同一文件读进无界 `Vec` 再复制多份。

**Estimated scope:** M

**Version increment:** patch

**Release boundary:** independent

**C/D coverage from:** self

**Granularity:** `type=implementation; axis=按序保留 thought chunk; recovery=revert 本卡提交; complete=yes; self-contained=yes; AC=8/8; VER=4/8; landing=1; prod-files=2; scope=M; deps=GK-01,DEP-GK-BASE,DEP-GK-WRITE; writeset=序列化于 GK-01; release=independent; split-from=N/A; exception=N/A`

落地目录只有 `src/internal/ai/observed_agents/`。生产文件是 `grok_updates.rs` 与 `mod.rs`。测试和 `tests/INDEX.md` 不计入。不注册 `live_capture_for`。

### Task GK-03: 未证明字段保留与 sink 隔离

**Task type:** implementation

**Lifecycle / Acceptance:** locally-accepted / 空

**Description:** 把合成 `chat_history.jsonl` 的 reasoning 字符串字段并入 GK-02 的 transcript，并保证 safe projection 等 sink 看不到这些明文。

**Out of scope:** 不解密。不写 native artifact。不改变 GK-02 的「chunk 不拼接」规则，除 ADR-GK-03 允许的精确挂接。历史 OpenCode import 仍由既有代码处理，本卡不放宽 `LBR-AGENT-020`。

**Current evidence:**

| 事实 | 证据 |
|---|---|
| retain 政策 | `plan-20260902.md` 锚 `user-og-thinking-retain-20261018` |
| reasoning 键 | 事实基线结构统计行 |
| 提取防御臂 | `capture/extraction.rs:424-431` |

**Acceptance criteria:**

- [x] 字符串 `encrypted_content`、`id`、`status` 和 `summary_text` 出现在普通 transcript。
- [x] 与某一条 thought 完全相同的摘要不重复写正文，密文挂在该条上。连续 4 条拼接的那一个合成样例同样只挂接一次。对不上的摘要单独成条。空摘要保留密文、不补正文。
- [x] 非字符串 `text` 或非字符串 `encrypted_content` 丢弃，回合为 partial。
- [x] metadata、类型化覆盖、Debug、safe projection、doctor JSON、coverage digest 和日志夹具对一份固定 canary 零命中。
- [x] `grok_reasoning.rs` 的普通导出函数生成的字节不含键 `encrypted_content`、`metadata.openai.reasoningEncryptedContent`、`metadata.anthropic.signature`。checkpoint transcript 仍含 Grok reasoning 密文。不改 `opencode_export.rs`。
- [x] 密钥形状的 thought 文本被现有 redaction 替换，不能原文进入 sink。
- [x] 形状不在 GK-00 矩阵内的文件拒绝导入，不写成一份删掉 Thinking 的成功 checkpoint。

**Verification:**

- [x] `source .env.test && cargo test --test grok_capture_test grok_unproven_ciphertext_is_in_transcript_and_absent_from_projection -- --exact`（new）。输出含 `1 passed`，不含 `0 passed`。
- [x] `source .env.test && cargo test --test grok_capture_test grok_reasoning_empty_summary_keeps_ciphertext -- --exact`（new，同一测试文件根上）。输出含 `1 passed`，不含 `0 passed`。

**Full-suite trigger:** T-1: 普通 transcript 与 safe projection 是共享 capture 事实源。

**Dependencies:** GK-02；DEP-GK-WRITE

**Deliverables:** N/A

**Implementation write set:** `src/internal/ai/observed_agents/grok_reasoning.rs`；`src/internal/ai/observed_agents/mod.rs` 只增加模块声明；`tests/grok_capture_test.rs` 只追加本卡用例。形状不符时返回现有 capture 路径已经使用的 malformed-transcript 错误，不新增 `StableErrorCode`。本计划与 `plan-status.md` 只更新本卡状态。

**Release write set:** Inherited

**Files likely touched:** 上列文件。

**Docs and compatibility impact:** N/A。本卡没有新的 CLI，也不新增错误码。用户可见说明在 GK-05。

**Rollback mode:** revert

**Migration and rollback:** revert 后旧 checkpoint 保持原字节。新导入停止写入未证明字段。

**Security and privacy:** ADR-GK-02。canary 只用合成标记，不用真实密钥。

**Performance budget:** Inherited。挂接只比较调用方已经保留的字符串，窗口最多 4 条，不做无界搜索。

**Estimated scope:** M

**Version increment:** patch

**Release boundary:** independent

**C/D coverage from:** self

**Granularity:** `type=implementation; axis=未证明字段进入普通 transcript 且离开 sink; recovery=revert 本卡提交; complete=yes; self-contained=yes; AC=7/8; VER=2/8; landing=1; prod-files=2; scope=M; deps=GK-02,DEP-GK-WRITE; writeset=序列化于 GK-02; release=independent; split-from=N/A; exception=N/A`

### Task GK-04: 用户级只观察 hook

**Task type:** implementation

**Lifecycle / Acceptance:** locally-accepted / 空

**Description:** 在 `GROK_HOME/hooks/` 安装带标记的观察 hook。测试直接调用该 provider 的处理函数。`Stop`、`StopCancelled`、`StopFailure` 和 `PreCompact` 是 hook 事件边界，所以压缩、截断和子会话分段放在本卡，而不是放在只解析字节的 GK-02。`live_capture_for` 保持 `None`，直到 GK-05 同时打开 `supported` 与 `hook_installable`。

**Out of scope:** 不写项目级 `.grok/hooks`。不注册会返回 `decision` 或 `updatedInput` 的 hook。不把 Cursor 的 `afterAgentThought` 当作 Thinking 来源。子会话不并入父会话正文。review/investigate 仍关闭。

**Current evidence:**

| 事实 | 证据 |
|---|---|
| hook 事件与 transcript_path | user-guide `10-hooks.md`、`25-status-line.md` |
| 现有 provider 目录 | `src/internal/ai/hooks/providers/mod.rs` 与 `opencode/`、`codex/` 同级 |

**Acceptance criteria:**

- [x] 直接调用的安装函数只原子更新带 `libra capture` 标记的用户级 matcher group；卸载函数只删除这些项；无标记文件保持原字节。CLI `enable` 仍由 GK-05 打开。
- [x] hook 进程退出码 0，stdout 不是 deny 或 block。读不完时写 durable pending，不把会话标成 complete。
- [x] `transcript_path` 规范化后必须是该 `sessionId` 目录下的 `updates.jsonl`。越界、符号链接和非普通文件拒绝。daemon 缺少 `transcript_path` 时只用事件里的 `cwd` 与 `sessionId` 按 GK-00 钉住的编码规则推导；缺一则只记生命周期。
- [x] `PreCompact` 先写 checkpoint。压缩后的文件是下一段。`updates.jsonl` 变短或 `eventId` 重复时封段，不删已捕获的 Thought。
- [x] 子会话目录单独成 session，父会话只保留链接。
- [x] 不存在调用 `grok` 二进制的代码路径。

**Verification:**

- [x] `source .env.test && cargo test --test grok_capture_test grok_hook_marker_round_trip_keeps_user_bytes -- --exact`（new）。输出含 `1 passed`，不含 `0 passed`。
- [x] `source .env.test && cargo test --test grok_capture_test grok_hook_does_not_spawn_grok -- --exact`（new，同一测试文件根上）。输出含 `1 passed`，不含 `0 passed`。

**Full-suite trigger:** T-1: hook provider 注册进共享 `hooks/providers/mod.rs`。

**Dependencies:** GK-03；DEP-GK-PIN 对 hook 形状 GO；DEP-GK-WRITE

**Deliverables:** N/A

**Implementation write set:** `src/internal/ai/hooks/providers/grok/mod.rs`；`src/internal/ai/hooks/providers/grok/parser.rs`；`src/internal/ai/hooks/providers/grok/settings.rs`；`src/internal/ai/hooks/providers/mod.rs`。不改 `command/agent/mod.rs` 的 enable 门（该门在 `mod.rs:485-540`，由 GK-05 打开）。不改 `live_capture_for`，不把 `as_hooks()` 设为 `Some`。测试直接调用 provider 函数，并追加到 `tests/grok_capture_test.rs`。本计划与 `plan-status.md` 只更新本卡状态。

**Release write set:** Inherited

**Files likely touched:** 上列文件。

**Docs and compatibility impact:** N/A。GK-01 已让 enable 返回 unsupported；本卡的安装函数只有在 GK-05 把 hook_installable 打开后才从 CLI 到达。本卡测试直接调用安装函数。若实现时 CLI 已经能写 hook，必须先停并修订，把命令文档移入本卡，不得悄悄变成 supported。

**Rollback mode:** revert

**Migration and rollback:** revert 代码后，已安装的标记 hook 用同一 uninstall 函数删除。用户自己的 hook 文件不删。

**Security and privacy:** 观察者。2 秒读取预算。SessionEnd 若宿主只给 1.5 秒，处理程序在剩余预算内只写 pending。

**Performance budget:** Inherited。一次 hook 只读上次偏移之后的新增字节。

**Estimated scope:** M

**Version increment:** patch

**Release boundary:** independent

**C/D coverage from:** self

**Granularity:** `type=implementation; axis=用户级只观察 hook; recovery=revert 本卡提交并卸标记 hook; complete=yes; self-contained=yes; AC=6/8; VER=2/8; landing=2; prod-files=4; scope=M; deps=GK-03,DEP-GK-PIN,DEP-GK-WRITE; writeset=序列化于 GK-03; release=independent; split-from=N/A; exception=N/A`

落地目录两个：`src/internal/ai/hooks/providers/` 与其子目录 `grok/`。按「一个具体目录」计为两个落地。生产文件 4 个：`providers/mod.rs` 与 `grok/` 下三个文件。

### Task GK-05: supported 提升与契约

**Task type:** implementation

**Lifecycle / Acceptance:** locally-accepted / 空

**Description:** 在 GK-04 的合成采集可用之后，把 `grok` 标为 supported 和 hook_installable，并更新首批之外的契约句，使它成为允许的例外而不是偷偷扩表。

**Out of scope:** 不打开 `launchable_review` 或 `launchable_investigate`。不把真实用户会话放进 CI。不解密。cloud mirror 仍拒绝尚未列入 D1 白名单的 kind。

**Current evidence:**

| 事实 | 证据 |
|---|---|
| 首批三名 | `registry.rs:338-339`；`tracing/agent.md:108-116` |
| 能力矩阵测试 | `tests/compat/agent_capability_matrix_pin.rs` |

**Acceptance criteria:**

- [x] `supported_slugs()` 包含 `grok`，且 `launchable_review` 与 `launchable_investigate` 的名单仍只有原来的首批三项。
- [x] `libra agent list --json` 里 `grok` 的 `supported=true`、`hook_installable=true`、`support_wave` 不是冒充 `first_batch` 的三名之一；使用新的 `grok` wave 值，并在 tracing 文档写明。
- [x] enable 后的用户级 hook 文件含标记和绝对命令路径；disable 后标记项消失。
- [x] 合成会话的 checkpoint transcript 含 thought 文本和未证明字段；默认 `checkpoint show` 不打印这些明文。显式 transcript 路径才输出脱敏后的正文。
- [x] tracing 第 116 行附近的「首批之外不得 supported」改为点名 `grok` 是 GK-05 之后的唯一追加，其余 slug 仍禁止。
- [x] EN、zh-CN、开发文档、`COMPATIBILITY.md` 和网站 `agent.en.md` 描述同一行为：本地日志、Thinking 保留、不解密、不作为 review/investigate 启动器。
- [x] `live_capture_for(Grok)` 为 `Some`，且与 `hook_installable`、`supported` 同为 true。GK-01 的 `PreviewAgent` 静态被 `mod.rs` 里的专用 struct 替换，该 struct 覆盖 `as_hooks()` 与 `stability()`。`command/agent/hooks.rs` 增加 grok 回调分发。enable 门仍在 `command/agent/mod.rs:485-540`，只放行 supported。不改 `preview.rs`。

**Verification:**

- [x] `source .env.test && cargo test --test compat_agent_capability_matrix_pin grok_supported_wave_is_not_first_batch -- --exact`（new）。输出含 `1 passed`，不含 `0 passed`。
- [x] `source .env.test && cargo test --test compat_agent_docs_contract grok_docs_contract_mentions_thinking_retain -- --exact`（new）。输出含 `1 passed`，不含 `0 passed`。
- [x] `source .env.test && cargo test --test grok_capture_test grok_live_capture_matches_hook_installable -- --exact`（new）。输出含 `1 passed`，不含 `0 passed`。

**Full-suite trigger:** T-1: 改变公开 agent 名单和 `registry.rs` 这一共享事实源。

**Dependencies:** GK-04；DEP-GK-DOC；DEP-GK-RELEASE；DEP-GK-WRITE

**Deliverables:** N/A

**Implementation write set:** `src/internal/ai/observed_agents/registry.rs`（同一行把 `supported`、`hook_installable` 设为 true，`support_wave=grok`，并改写「凡 supported 必为 first_batch」的断言，使 `grok` 除外、原来三名不变）；`src/internal/ai/observed_agents/mod.rs` 用专用 struct 替换 GK-01 的 `PreviewAgent` 静态，覆盖 `as_hooks()` 与 `stability()`，并把 `live_capture_for` 设为 `Some`；不改 `preview.rs`；`src/command/agent/list.rs`；`src/command/agent/mod.rs` 的 enable 门与写死三名的测试；`src/command/agent/hooks.rs` 的 grok 回调分发；`docs/commands/agent.md`；`docs/commands/zh-CN/agent.md`；`docs/development/commands/agent.md`；`docs/development/tracing/agent.md`；`COMPATIBILITY.md`；`/Volumes/Sea/libra-backend/apps/tanstack-app/content/docs/commands/agent.en.md`；`tests/compat/agent_capability_matrix_pin.rs`；`tests/compat/agent_docs_contract.rs`；`tests/grok_capture_test.rs` 只追加 live 臂用例。本计划与 `plan-status.md` 只更新本卡状态。

**Release write set:** Inherited

**Files likely touched:** 上列文件。

**Docs and compatibility impact:** `docs/commands/agent.md`；`docs/commands/zh-CN/agent.md`；`docs/development/commands/agent.md`；`docs/development/tracing/agent.md`；`COMPATIBILITY.md`；网站 `apps/tanstack-app/content/docs/commands/agent.en.md`。不改 `docs/error-codes.md`。网站写入前记录 VCS 与已验证的 `cf`。在该仓库执行 `python3 scripts/sync-libra-command-docs.py --libra-root <本仓库>`、`python3 scripts/test_sync_libra_command_docs.py`、`pnpm --filter @tinyship/tanstack-app typecheck` 和 `pnpm --filter @tinyship/tanstack-app build`。网站 diff 另需一次 Claude PASS 后才签推 `cf`。实现代码的 ER-05 仍是 Codex PASS。这两次审查对象不同。这些命令是 ER-06a 门，不计入 VER。

**Rollback mode:** revert

**Migration and rollback:** revert 后 slug 回到 GK-01 的 unsupported。已安装的标记 hook 仍可由 GK-04 的 uninstall 删除。已发布版本按 immutable-release 的用户指引是停用 hook，不回滚 artifact。

**Security and privacy:** ADR-GK-02。CI 只用合成 fixture。

**Performance budget:** Inherited。list 不读 transcript。

**Estimated scope:** M

**Version increment:** patch

**Release boundary:** independent

**C/D coverage from:** self

**Granularity:** `type=implementation; axis=将 grok 升为 supported 且不开放 launcher; recovery=revert 本卡提交回到 preview; complete=yes; self-contained=yes; AC=7/8; VER=3/8; landing=2; prod-files=5; scope=M; deps=GK-04,DEP-GK-DOC,DEP-GK-RELEASE,DEP-GK-WRITE; writeset=序列化于 GK-04; release=independent; split-from=N/A; exception=N/A`

落地目录：`src/internal/ai/observed_agents/`、`src/command/agent/`。生产文件 5 个：`registry.rs`、`mod.rs`、`list.rs`、`command/agent/mod.rs`、`command/agent/hooks.rs`。文档、网站和测试不计入。不改 `preview.rs`。

## 测试矩阵

| 类别 | 必须覆盖 | Target / command |
|---|---|---|
| 单元 | kind 往返、chunk 顺序、挂接、sink 零命中、hook 标记 | 与各卡 Verification 相同的 `--test <target> <完整函数名> -- --exact`，不使用 `--lib` 裸函数名 |
| 集成 | 合成 JSONL 到 transcript；enable/disable 文件字节 | `source .env.test && cargo test --test grok_capture_test grok_updates_keeps_three_thoughts_in_order -- --exact`，输出含 `1 passed` |
| 兼容 | 首批三名仍在；grok 的 supported 位只在 GK-05 改变 | `source .env.test && cargo test --test compat_agent_capability_matrix_pin known_agent_capability_matrix_matches_current_roster -- --exact` 与 `source .env.test && cargo test --test compat_agent_capability_matrix_pin grok_supported_wave_is_not_first_batch -- --exact` |
| 迁移 | 新 CHECK、旧行仍在、Global/System 零写、down 有行则拒绝 | `source .env.test && cargo test --test grok_agent_kind_test grok_agent_kind_round_trip_and_check -- --exact`，输出含 `1 passed` |
| 安全 | 路径越界、符号链接、canary 不进 sink、hook 不返回 deny | `source .env.test && cargo test --test grok_capture_test grok_hook_marker_round_trip_keeps_user_bytes -- --exact` 与 `source .env.test && cargo test --test grok_capture_test grok_unproven_ciphertext_is_in_transcript_and_absent_from_projection -- --exact` |
| 性能 | 16 MiB、1 MiB 行、8 MiB thought、2 秒预算 | 同集成测试里的超限用例 |
| live/gated | 不进 CI | 无。真实家目录不作为验收 |

## 追溯表

| 任务 | 来源/证据 | Libra 落点 | 文档/兼容动作 | 指定测试 |
|---|---|---|---|---|
| GK-00 | GAP-GK-01 | 本计划 | 无用户文档 | 人工矩阵 |
| GK-01 | GAP-GK-02 | CHECK、`AgentKind`、registry | 无，仍 unsupported | `grok_agent_kind_round_trip_and_check` |
| GK-02 | GAP-GK-03、ADR-GK-03 | `grok_updates.rs` | 无新 CLI | `grok_updates_keeps_three_thoughts_in_order` |
| GK-03 | GAP-GK-04、ADR-GK-02 | `grok_reasoning.rs` | 无新 CLI、无新错误码 | `grok_unproven_ciphertext_is_in_transcript_and_absent_from_projection` |
| GK-04 | GAP-GK-05 | `hooks/providers/grok/` | 无，直到 GK-05 | `grok_hook_marker_round_trip_keeps_user_bytes` |
| GK-05 | GAP-GK-06 | registry、tracing、命令文档 | EN/zh/开发/COMPATIBILITY/网站 | `grok_supported_wave_is_not_first_batch`、`grok_docs_contract_mentions_thinking_retain` |

## 里程碑验收与回滚

| 里程碑 | 完成条件 | 发布/证据 | 回滚或前滚 |
|---|---|---|---|
| M0 | GK-00 GO/HOLD 写入本计划 | 无版本 | revert 文档 |
| M1 | GK-01 库接受 `grok` 且 CLI 仍 unsupported | 该卡 patch 与 D-CAP-STD | 前滚 |
| M2 | 合成 transcript 含 Thought 与未证明字段，sink 零命中 | GK-02 与 GK-03 各自的 patch | revert 对应提交 |
| M3 | 用户级 hook 与 supported 提升 | GK-04 与 GK-05 各自的 patch | disable 标记 hook；已发布版本停用而不撤回 artifact |

### 故障恢复矩阵

| 故障点 | 可接受残留 | 恢复动作 | 禁止结果 |
|---|---|---|---|
| 上游 SHA 不可得 | 不 HOLD。按已安装版本的字段矩阵实现 | 形状超出矩阵则拒绝导入 | 用浮动 main 冒充 1.0.62，或把缺少 SHA 当成停工条件 |
| CHECK 迁移失败 | 旧库未改 | 丢弃专用连接 | 把 foreign_keys=OFF 还回池，或删用户行 |
| 日志超限或读超时 | durable pending，partial | 下一轮从偏移续读 | 报 complete，或删掉已写入的 Thought |
| rewind 或 compact | 旧段保留 | 新段另记 | 回写旧 checkpoint |
| 路径越界 | 零字节读取 | 拒绝并诊断 | 跟随符号链接读到会话目录外 |
| 网站或 release 失败 | locally-accepted 或 remote-pending | 前滚 | 把未部署的网站写成已发布 |

## 风险登记

| 风险 | 影响 | 缓解 | 任务 |
|---|---|---|---|
| chunk 在新版本变成 delta | 中。按事件保存会得到碎片 | GK-00 推翻 ADR-GK-03 之前禁止拼接 | GK-00、GK-02 |
| 摘要挂错密文 | 中。未证明字段贴到错误 thought | 只允许全等或 4 条窗口全等 | GK-03 |
| 与 Hermes/Codex 计划抢 `mod.rs` | 中。并发写坏闭集 | 全串行加 DEP-GK-WRITE | 全部实现卡 |
| 真实家目录被测试打开 | 高。隐私 | 合成 fixture；验收命令不含 `GROK_HOME` | GK-02..GK-05 |
| supported 早于读取器 | 中。用户启用后没有正文 | ADR-GK-04 把提升放在最后 | GK-01、GK-05 |

## 性能与容量摘要

| 操作 | 单次成本 | 累积成本 | 预算/上限 | 验证 |
|---|---|---|---|---|
| 读一条 JSONL | 一行 | 文件偏移单调增加 | 行 ≤1 MiB，source ≤16 MiB | 超限用例 |
| 保留 Thought | 一条事件 | checkpoint 全量快照 | 文本 8 MiB，超出写截断标记和偏移 | GK-02 |
| 挂接摘要 | 最多看 4 条 | 每条 reasoning 一次 | 不对整个会话做平方搜索 | GK-03 |
| hook | 读新增字节 | 每回合一次 | 2 秒；超时 pending | GK-04 |

## 兼容与文档收口

- [ ] `COMPATIBILITY.md` 在 GK-05 同步。此前 N/A。
- [ ] `docs/commands/agent.md` 在 GK-05 同步。
- [ ] `docs/commands/zh-CN/agent.md` 在 GK-05 同步。
- [ ] `docs/development/commands/agent.md` 与 `docs/development/tracing/agent.md` 在 GK-05 同步。
- [ ] `docs/error-codes.md` 为 N/A。本计划不新增 `StableErrorCode`。
- [ ] `tests/INDEX.md` 在 GK-01 增加 `grok_agent_kind_test`，在 GK-02 增加 `grok_capture_test`。
- [ ] 不新增 `tests/compat/*` 文件，故不改 `Cargo.toml` 的 `[[test]]`。既有 compat 测试在 GK-05 更新。
- [ ] `plan-long.md` 与 `plan-status.md` 在本计划建立时登记。

## Codex review log

Result 只允许 `PASS` 或 `FAIL`。

| Round | Scope | Result | P0/P1 | P2 处置 | Evidence |
|---|---|---|---|---|---|---|
| R1 | 初稿六卡 | FAIL | P1：`--exact` 配前缀会 0 测试；缺 ER 与粒度规则节。两项均已在下一稿关闭 | P2：GK-04 落地少计 `builtin/`；review log 写了 `未审`。同轮已改 | Codex session `01a14fa2-7da1-7243-b9cd-fd9db828671c`，2026-10-18 |
| R2 | R1 修订稿 | FAIL | P1：ER-04 漏了 PR 与 merge SHA；GK-00/GK-01 的 complete 循环；`--lib` 裸函数名；审计命令被 1 passed 规则套住。均在下一稿关闭 | 无新 P2 | Codex session `01a14fa7-f8f2-7943-91f9-606b3a9a3645`，2026-10-18 |
| R3 | R2 修订稿 | FAIL | P1：测试矩阵迁移行仍写 `--lib grok_agent_kind`。已改为完整 `--test grok_agent_kind_test` 函数名，追溯表前缀一并改成完整函数名 | 无 | Codex session `01a14fab-572d-7670-8867-f0ee5eb2d5b6`，2026-10-18 |
| R4 | R3 修订稿 | FAIL | P1：把 retain 第 182 行的导出删除理解成 transcript 不得保留密文。ADR-GK-02 现写明 transcript 保留 Grok 密文，普通导出仍删除该键 | P2：GK-05 追溯行已改成两个完整函数名 | Codex session `01a14fad-188a-7d52-b3b6-5a34973ccbaa`，2026-10-18 |
| R5 | R4 修订稿 | PASS | 无。R4 的导出与 transcript 区分已关闭 | 无 | Codex session `01a14faf-e652-73e1-a496-d9d10173a123`，2026-10-18 |
| Claude R1 | Codex R5 之后的全文 | FAIL | P1 六项：live 臂与 hook_installable 错卡；GK-01 落地把 `builtin/` 漏计后改为不碰该目录并维持 4 个落地；发布 tag 改为 vault 签名 annotated tag；sink 清单补全；旧读取器与 Derived 零写；UTF-8 解码向量 | P2：实施顺序标题、测试矩阵命令、INDEX 两处、GK-04 的 grok 子目录、幂等 AC、网站 Claude PASS、导出函数落在 grok_reasoning.rs | Claude Code 2.1.302，2026-10-18 |
| Claude R2 | Claude R1 修订稿 | FAIL | P1：`libra tag` 没有 `-s -m`，实际仍是未签名轻量 tag。已改为 `libra fetch origin` 后 `libra tag -s -m "Release v${VERSION}" "v${VERSION}" "$MERGE_SHA"`，再 `libra tag -v` | P2：PreviewAgent 没有 hooks 字段；GK-05 改为专用 struct 覆盖 `as_hooks`；旧读取器断言并入既有测试；去掉重复的零写句 | Claude Code 2.1.302，2026-10-18 |
| Claude R3 | Claude R2 修订稿 | PASS | 无 | 非阻断说明已落：Phase 1 退出条件含 Derived；GK-01 的 PreviewAgent 不进 `PREVIEW_SPECS`，`is_preview` 保持 false；稳定性单测在本卡对 Grok 断言 Preview | Claude Code 2.1.302，2026-10-18 |

## 非目标与延后项

| ID | 延后内容 | 原因 | 重启条件 | 承接位置 |
|---|---|---|---|---|
| DEFER-GK-01 | 解密 `encrypted_content` 或把它归档为 native artifact | 没有来源证明；用户要求的是保留而不是证明 | 另立 Source A 计划并得到可验证 issuer | 新计划，不在本文件加卡 |
| DEFER-GK-02 | D1 / cloud mirror 接受 `grok` | 白名单属于既有 cloud 计划的写集 | 该计划把 `grok` 列入允许集合并完成自己的卡 | plan-20260911 或后继 cloud 围栏卡 |
| DEFER-GK-03 | 项目级 `.grok/hooks` 与目录信任 | 用户级 hook 已能覆盖本机采集；项目级信任会扩大执行面 | 用户明确要求仓库内 hook，并有信任存储的测试 | 本计划新卡，不塞进 GK-04 |
| DEFER-GK-04 | review / investigate 启动 Grok | 采集不等于把外部 agent 变成受控执行器 | 单独的 launcher 计划 | 不在本计划 |

## 完成判据

计划只有在以下条件全部满足后才能标记完成：

- [ ] 所有任务卡满足粒度规则 `G-*`，审计表与各卡 `Granularity` 一致，无未登记的 L 例外。
- [ ] GK-00..GK-05 均为 `Lifecycle=done` 且 `Acceptance=complete`。
- [ ] 每张卡的 Verification 已运行并写入该卡。
- [ ] 全量收口门已绿：`cargo +nightly fmt --all --check`、`cargo clippy --all-targets --all-features -- -D warnings`、`source .env.test && source .env.live-test && cargo nextest run --all --no-fail-fast --retries 2`。
- [ ] 全量暴露的本计划失败已前滚并重跑到全绿；既有失败有基线复现证据并登记 `FIX-*` 或 `DEFER-*`。
- [ ] GK-05 的文档与网站已同步。
- [ ] GK-01 的迁移故障注入已做。其余卡的 `Rollback mode` 已核对。
- [ ] 实现阶段的 Codex review 最终为 PASS，P0/P1 已关闭。规划审查 PASS 不等于这一条。


## 架构修复与后续交接（GK-06 / GK-07 / GK-08）

本节是维护者最新指令下的执行边界；原GK-00..05勾选仅代表当时测试结果，不证明真实hook已将thinking写入正式checkpoint。没有关联GitHub Issue，不创建远端Issue；本次范围由用户直接授权。当前main通过fetch/ff更新至 `e270bfe`（0.30.65），CX-15旧工作通过备份/三方恢复保留，不发布旧0.30.63候选。autostash `de4a9b786046c94b1909842d765afef83c4a64f7` 保留，索引恢复失败不记成功；实际恢复保留Codex retention和上游Claude attached-fields两条路径。

### ADR-GK-05: 解析无I/O，正文只经授权snapshot和共享writer

- **Status:** Accepted（维护者授权架构修复）。
- **Context:** `e0b304c` 在 `GrokProvider::parse_hook_event` 内调用record/observer，解析尚未完成scope授权就读取日志并写 `GROK_HOME/libra-capture`，budget=None且结果忽略。旧文件级guard只扫描parser.rs，漏掉mod.rs新增调用。
- **Decision:** 删除生产record调用，trait parse仅委托纯parser；保留原store-free guard并新增trait纯委托/生产observer引用守卫。独立observer移至 `hooks/providers/grok/session_files.rs`，仅保留离线兼容入口，不接生产runtime。生命周期可以走共享管线；thinking接入保持暂停，后续按GK-07/08串行推进。
- **Compatibility:** `grok::encode_grok_cwd`、`grok::observe_grok_hook` 签名保持；`GrokObserveOutcome` 可从grok根或session_files模块取得。此前暴露的内部 `grok::parser::observe_grok_hook` / `GrokObserveOutcome` 路径迁至 `grok::session_files`；这是明确的Rust内部模块路径迁移，调用方更新import，不保留让纯parser重新暴露持久化handle的转发。CLI语法不变。现有scratch文件不删、不迁移、不导入为正式checkpoint。
- **Persistence:** observer的store.json/checkpoint.json/pending.json/lifecycle.json不是 `agent_checkpoint`、catalog或trace ref。不得把这些文件存在当获取成功。禁止将legacy observer重新接入生产。它仍有静默读写失败、非原子/无锁状态、累计大小和重复扫描预算问题，都是生产接入阻断项；通过共享owner接入时不复用该持久化实现。
- **Required chain:** validated ingress/repository scope → provider candidate（仅推导路径）→ source grant/snapshot授权、有界读取及脱敏 → 纯字节normalizer/reasoning合并 → shared checkpoint writer/catalog/ref。所有父子会话关联、pending和PreCompact遵循相同owner，不直接写独立JSON。
- **No regression:** 不启动grok、无OpenCode export桥、无native证明或解密；保留Thought各段及opaque值，不弱化现有字节、路径、deadline和repository/session绑定。

### Task GK-06: 恢复Grok纯解析架构

**Task type:** implementation

**Lifecycle / Acceptance:** locally-accepted / 空

**Description:** 删除解析阶段旁路采集，分离纯parser与离线observer，补运行时入口guard、暂停状态和兼容文档。

**Out of scope:** 不接真实Grok正文、不运行模型；不改CX-15功能、不将Codex测试加入CI；不提交、推送、发布或部署。

**Current evidence:** main `e270bfe` 中 `grok/mod.rs` 的record调用及 `parser.rs` 混合I/O；原GK-05 supported/hook位不能证明实际checkpoint存在thinking。

**Acceptance criteria:**

- [x] trait parse仅委托纯parser，五种事件/未知事件/不安全session均有回归；不依赖全局GROK_HOME变更。
- [x] parser无fs/path/store依赖，生产src不引用legacy observer（兼容导出除外）；原guard保留，新增guard防mod.rs/别名绕过。
- [x] 离线observer合成测试继续通过，其根导出保持兼容；Rust内部路径迁移、已知限制和禁止生产接入写明。
- [x] EN/zh/dev/tracing/COMPATIBILITY/网站说明暂停事实，名单表与registry一致；完整全量/格式/严格Clippy/只读复审如实登记。

**Verification:** `source .env.test && cargo test --locked --test grok_capture_test`；`source .env.test && cargo test --locked --test compat_agent_architecture_guard`；`cargo +nightly fmt --all --check`；`cargo clippy --locked --all-targets --all-features -- -D warnings`。使用共享target-dir可显式追加，实际命令和非零用例数记在本节。网站按既有sync/check/fixture/typecheck/build门。本卡共享provider契约和守卫触发全量：`source .env.test && source .env.live-test && cargo nextest run --locked --all --no-fail-fast --retries 2`。Codex仅本机专项另跑，不加入默认测试。

**Dependencies:** 用户本次架构修复授权；最新main已fetch并ff；GK功能实施暂停不阻本修复。

**Implementation write set:** `src/internal/ai/hooks/providers/grok/{mod,parser,session_files}.rs`、`src/internal/ai/hooks/provider.rs`、`tests/grok_capture_test.rs`、`tests/compat/agent_architecture_guard.rs`、`tests/compat/agent_docs_contract.rs`（暂停/scratch文案契约）；EN/zh/dev同名agent文档、tracing/agent、COMPATIBILITY、网站agent.en、本计划和plan-status；恢复CX-15冲突仅合并旧retention与新Claude字段，不扩行为。

**Release write set:** no-release，本次不bump、不签推/部署；保留上游0.30.65版本面。

**Rollback mode:** revert（只撤本卡变更；禁止恢复未经授权的parse旁路写入）。

**Docs and compatibility impact:** CLI不新增参数；声明thinking接入暂停，Rust旧parser helper路径迁移；旧正式checkpoint和scratch文件均不修改。

### Task GK-07: 纯字节Grok transcript normalizer（交接给其他agent）

**Task type:** implementation

**Lifecycle / Acceptance:** locally-accepted / 空

**Description:** 先建立不含I/O的Grok字节normalizer与合成契约，再由GK-08接授权读取。把已有updates/history字段映射整理为bytes→typed events→ordinary transcript/normalized turns，保留thought顺序、reasoning summary/id/status/opaque值，所有解码后正文继续凭据扫描。此卡不修改注册能力或启动器。

**Out of scope:** 禁止hook/provider文件I/O、禁止调用legacy observer、禁止接生产source/capture writer、禁止真实模型、提交、推送、发布；不改任何Codex测试或CI。

**Current evidence:** 现有 `grok_updates.rs` / `grok_reasoning.rs` 的文件读取和字段解释混合；必须保留原路径API的校验，并让新pure bytes API可在snapshot之后使用。

**Acceptance criteria:**

- [x] 为两份输入增加有界、deadline-aware的纯字节解析入口，复用同一字段解释；文件API保留且其路径/大小拒绝语义不弱化。
- [x] 合成测试覆盖三段thought不拼接、summary精确/四段匹配、不匹配独立、opaque-only、重复eventId、坏/截断行、凭据和Unicode；不把summary当完整thinking，不伪造user/assistant正文。
- [x] 没有真实可归属turn时明确partial/未归属，不从scratch文件制造完整coverage；输出供后续共享writer消费，无SQL/ref/FS副作用。
- [x] focused Grok reader测试、fmt、strict Clippy与只读复审通过；更新本节实际结果，Acceptance在未交付时保持空。

**Verification:** 既有 `cargo test --locked --test grok_capture_test`、`cargo test --locked --test compat_agent_architecture_guard grok_parser_delegates_without_observer_side_effects -- --exact`；新增pure parser用例加入既有target、在本节登记完整名称和非零计数；fmt/strict同GK-06。

**Dependencies:** GK-06本机架构门和文档交接通过后开工；同一工作树串行，不覆盖CX-15或架构修复改动。Owner为主agent交接后明确指派的agent，尚未指派前不能声称已接单。

**Implementation write set:** `src/internal/ai/observed_agents/grok_updates.rs`、`grok_reasoning.rs`、`tests/grok_capture_test.rs`（只追加pure parser cases）、本计划/plan-status。如必须增加共享coverage/注册/SQL/CLI文件，先停止该部分，提出下一卡，不自行扩大本卡。

**Full-suite trigger:** 无共享owner或公共CLI改变时只focused；不能以此卡局部通过宣称计划完成；GK-08集成和最终全量另验。

**Release write set:** no-release；本轮只本机开发，不bump/提交/推送。

**Rollback mode:** revert。旧文件API与fixture断言保留；新增byte API不影响当前hook处理。

### Task GK-08: 授权来源与共享checkpoint接入（仍暂停，后续选卡）

**Lifecycle / Acceptance:** planned / 空

接续方向而非本次已开工卡：在GK-07合格后，枚举snapshot候选/多源授权/预算、真实turn归属与typed coverage、共享writer、pending/compaction/fence/replay、正式checkpoint export正文验证。必须先形成满足粒度规则的具体I写集、AC和Verification，用户选卡后才接生产。禁止以legacy observer或文件式exporter绕过snapshot；禁止写 `GROK_HOME/libra-capture` 代替catalog。需要时拆成来源授权/turn normalizer/持久化三卡，保持依赖显式。真实模型测试仍需单独选择，不能从supported或合成测试推断已取到数据。

### GK-06 实施证据汇总

待实际运行后填入；旧失败保留为历史，不能预填通过。后续agent先读本节的最终交接状态，再执行GK-07。


**阶段实测（尚非最终门）：** 最新main整合后Grok专项20/20实际通过，含五事件纯解析和独立observer旧合成测试；原文件级parser guard和supported roster guard实际通过。初次完整architecture target为64 PASS/1 FAIL，唯一失败是上游e270bfe新增Claude legacy mirror后RG-05 serializer旧SHA pin未更新；对应cloud生产文件无本地改动。此失败保留，先进行专门Claude scope review再决定是否更新pin，不弱化guard。上游Claude legacy revision的本地幂等/缺checkpoint拒绝/erase/mirror合成回归实际1/1通过。初次strict Clippy实际exit0；后续guard宏扫描、文档和边界断言修订需最终重验。

Claude GK-06/网站只读审查实际exit0/is_error=false、36 turns、VERDICT PASS（仅架构/文档），raw stdout SHA-256 `ad2d0d8303e56d48150d9ca3a41cc151b43837a2372570c322a72cf891418c15`，无P0/P1/P2。非阻塞文案与安全session独立unknown-event断言已澄清。该审查读取到的guard无宏自检，独立agent另提出P2；已加递归TokenTree和5负/3正fixture，独立代码复核确认P2可关闭，最终实际运行仍随后登记。网站同步fixture7/7、typecheck/build实际exit0，文案澄清后已重跑同组全部exit0。未提交/推送/发布/部署；真实Grok模型仍UNRUN。


**RG-05 上游指纹整合审查：** 最新main e270bfe仅增加Claude legacy revision镜像元数据和校验，生产cloud serializer本地未修改。专门Claude只读scope审查实际exit0/is_error=false、24 turns、VERDICT PASS，允许remainder pin从 `06b56c63413da64355910602a1bfb109d1ee2a8d546832edf75276029c5ed469` 更新为实际 `ea0378334c06aa458c7e9acdd3a81c88c00d5261b8590245d1deeea08920ce73`；prefix pin `1b477e373f2554623a8e7121896e09ed5a806a2174845750b6ebf903d7b03912` 保持。raw SHA-256 `a2abeed2b34dd7b146c47da2a19a897c1fcdbf7b4dbe281e5040a067adc0c2b7`。复核结论是元数据只读、无源opener、无新ref/D1写入、同snapshot事务和重检，缺旧checkpoint或现存tombstone拒绝。专门1/1本机合成测试已通过。未把pin更新当新cloud功能验收；审查另留五项非阻塞P2观察给原Claude/cloud owner：残留session+revision+tombstone分支、完整cloud-sync E2E未直接覆盖，孤儿revision依赖FK，错误logical key行过滤、新公开只读helper API。它们不属于Grok后续agent写集，不声称已修复。此次GK-06写集补充仅该既有guard的精确指纹整合，不改cloud生产owner。

**CX-15整合复核：** 当前0.30.65合并树本机Codex集成23/23、私有11/11实际PASS；一次误选 `agent_reasoning_contract_test -- --ignored` 为0例，不能计foundation PASS，随后按真实 `agent_checkpoint_redaction_test::capture_foundation_codex_reasoning_boundaries_live_and_import` 单独重验。Codex目标仍required-feature+ignored，CI未改。


**最终focused文案边界：** 新架构65/65与capability5/5已通过；docs target首轮33/34，Grok例因新英文使用“without decryption”未匹配既有“does not decrypt”断言而失败。保留断言、改正文案显式“Libra does not decrypt”，并在同一既有Grok契约追加paused/scratch披露断言，不弱化旧要求。随后重新执行focused与静态门，旧失败不追认为PASS。


**最终focused/静态门（完整回归之前）：** 在0.30.65合并树执行 `source .env.test && cargo test --locked --target-dir /Volumes/Sea/validation/opencode-thinking-20261018-233523-3tfb_79x/build --test grok_capture_test --test grok_agent_kind_test --test compat_agent_architecture_guard --test compat_agent_docs_contract --test compat_agent_capability_matrix_pin` 实际exit0：20+1+65+34+5=125 PASS，无ignored/零匹配替代。新guard的宏扫描和5负/3正fixture已在这轮运行。随后的nightly fmt check与strict all-target/all-feature Clippy -D warnings均exit0。手动hook矩阵实际1/1 PASS，真实foundation用例在 `agent_checkpoint_redaction_test` 指定完整名称、feature+ignored+exact实际1/1 PASS；与23集成+11私有合计35本机Codex用例均PASS，不在默认CI执行。一次误选target的0例不计通过。模板要求的双环境nextest全量正在同一冻结非计划源码上运行，结果随后记录。


### GK-06 最终本机验收与交接

**记录时间：2026-10-19 11:11:43 UTC。** GK-06本机实现/测试/审查通过，Lifecycle改为locally-accepted，Acceptance仍空，不表示已提交、远端CI通过或发布。源码更新点为origin/main `e270bfe` / 0.30.65，没有安装或发布本地候选。

| 门 | 实际结果 |
|---|---|
| Grok/架构/docs/capability focused | 125/125 PASS（Grok20、kind1、architecture65、docs34、capability5）；保留原守卫，宏/别名负例实际运行 |
| CX-15恢复后本机用例 | 集成23/23、私有11/11、真实foundation1/1，共35 PASS；另手动hook字节矩阵1/1 PASS；feature+ignored，仅本机 |
| 上游Claude镜像整合 | 本地repair/replay/缺checkpoint拒绝/erase测试1/1 PASS；RG-05精确指纹经专门Claude PASS后更新，cloud生产源码无本地修改 |
| 网站 | cf本地sync/check：108命令、109导航、0 drift；fixture7/7、typecheck/build exit0；Claude架构/网站PASS；原14页未提交改动保留，只同步agent增量 |
| 架构/网站审查 | Claude36 turns PASS，无P0/P1/P2；宏遗漏P2由独立agent复核关闭；普通文案及unknown安全session边界已补；cloud审查24 turns PASS，五项非阻塞观察留原owner |
| 最终完整回归 | 双环境+子进程umask022：`cargo nextest run --locked --all --no-fail-fast --retries 2 --target-dir /Volumes/Sea/validation/opencode-thinking-20261018-233523-3tfb_79x/build` exit0，**10325/10325 PASS，13 slow、1 leaky、23 skipped，846.440s** |
| 冻结与Codex隔离 | 2008个非计划文件的内容/模式聚合SHA-256 `d8e5172d9e1cd4810819fb48e15d9a98bdb4585b35206422ed276f4311a621fb`，全量前后MATCH；JUnit新增Codex手动cases选入0，workflow无本地改动 |

完整回归stdout/stderr流SHA-256 `6c7e6d88ac2ecb930e7fead8700575eea0340a5d24ffdfcec6416b7d98db3d18`。本轮唯一LEAK为未修改的 `command::fetch::tests::fetch_prune_porcelain_and_fetch_head_layout`（0.214s）；测试断言PASS、全量exit0，保留诊断，不称零泄漏或已经修复。前述architecture pin旧FAIL、文案断言FAIL和误选0例均保留，不倒改。

现有根导出保持，旧Rust内部parser helper路径迁移已明示；不删除/重写历史checkpoint或scratch，不读取用户Grok日志。legacy observer仍有已登记的离线限制，禁止生产调用。恢复后的autostash和逐字节备份仅为恢复锚点，工作已手工三方整合，后续agent不要再次apply/pop该autostash。

后续受派agent `/root/grok_continue` 已完成只读准备：仅按GK-07两个reader文件/追加合成测试继续，不改生产hook、shared writer、SQL/CLI、Codex测试、CI或发布面。GK-08生产接入和真实Grok模型仍暂停，未授权开工。本机架构门通过后主agent发送正式开始指令；GK-07的代码变化须单独记录focused/审查结果，不能冒用以上GK-06冻结快照的全量PASS。当前无提交、推送、tag、release、网站推送或部署。


**2026-10-19 11:12:48 UTC 正式开发交接：** 主agent最终重复nightly fmt check、严格all-target/all-feature Clippy -D warnings、diff check均实际exit0。已向`/root/grok_continue`发送“GK-06全量门完成，允许GK-07实现”，该agent运行中；GK-07改为in-progress/Acceptance空。其写集严格限两份reader及现有target追加测试，计划记账由主agent协调，生产hook/GK-08和真实模型仍暂停。后续GK-07源码不属于上述GK-06全量冻结快照，不得将那次PASS挪作新代码的验收。仍无提交/推送/发布/部署。


**2026-10-19 11:34:48 UTC GK-07本机交接完成与Grok单独提交授权：** 用户明确要求仅提交Grok改动，后续Grok由另外的agent开发，本会话只继续Codex会话采集。GK-07仅两个reader+既有target追加11项：`parse_grok_updates_bytes`、`normalize_grok_reasoning_bytes`/`GrokNormalizedTranscript`；输入16MiB、行1MiB、正文8MiB、输出字符串32MiB，deadline协作检查，超预算显式Err，无静默截断/伪造turn。file wrapper保留jail/类型拒绝并加打开后cap+1读取；doctor只保留闭集脱敏标记，不含同句其它正文。执行agent实跑Grok31/31+architecture65/65、fmt、严格all-target/all-feature Clippy与diff check均exit0（Clippy42.41s）；独立只读复审PASS，P0/P1/P2均0。残余：单次有界JSON/脱敏操作不可抢占、旧updates wrapper全量有界读取后按cursor解释，不冒称增量成本；禁止把legacy observer接生产。GK-07 locally-accepted/Acceptance空，不借GK-06旧全量PASS验新源码。

单独提交使用HEAD加Grok相关hunk构造index，保留全部Codex WIP原字节；包括Grok架构/reader/tests、相应的迁移/名单/状态库存回归和文档，排除Codex retention/admission/oracle写集。网站同样只提交agent页面Grok段，保留其它页及Codex段。只授权本地签名DCO提交，不推送/发布/部署；最终实际commit与隔离验证结果随后登记。GK-08仍暂停，后续Grok任务不再由本会话派发实现。


**2026-10-19 11:41:37 UTC Grok-only提交快照隔离验证：** 使用HEAD加所选index内容的临时源码快照（不含codex_reasoning.rs、Codex本机target、admission转发或Claude oracle覆盖）验证：Grok31/31、kind1/1、docs34/34、capability5/5实际PASS；nightly fmt和严格all-target/all-feature Clippy exit0。架构target初次63/65：Grok引入第20个AgentKind分支的问题用既有provider-neutral partial fallback及下降至15的guard修复纳入本提交；另一项capture_modules_do_not_import_hook_entry_layers来自HEAD原有Claude admission直接引用，ingress/Claude源码逐字节等于HEAD，修复继续保留在Codex WIP，不夹带。先前联合工作区的10325全量PASS不是本次独立commit的全量PASS。仅本地提交，不推送；其它已有非Grok工作保留。


**2026-10-19 11:50:58 UTC 独立推送前远端整合：** 原本地Grok提交ddd06bd与网站58e900c在推送前发现远端main已至feafdb3、cf已至4fb6e7e。保留其它Agent的Claude admission/oracle及Grok架构修复，分别rebase独立提交；冲突仅合并同一Grok行为轴。离线helper采用远端已公布的`grok::session_files`路径，不另建重复observer模块，根导出兼容；去除重复allowlist项，保留严格纯解析及生产调用守卫。Codex与其它WIP已逐字节备份并暂时移出验证快照，提交后通过三方合并恢复。先前Grok-only Claude admission失败已由远端feafdb3修复，本次将重新验证，不沿用旧PASS。网站整合后sync检查108命令/109导航零漂移、fixture7/7、typecheck和build实际exit0；Claude复审与最终Rust focused/strict门随后登记。

**2026-10-19 11:53:09 UTC 推送前最终专项门：** 不含Codex WIP的整合快照实际执行Grok31、kind1、architecture65、docs34、capability5，共136/136 PASS；nightly fmt与严格all-target/all-feature Clippy -D warnings均exit0（59.02s，仅既有proc-macro-error2 future-incompat提示）。网站Claude Opus 5.5仅依据所提交差异及源文档复审PASS（无权限拒绝），cf提交经rebase为b77b548并已推送，Workers Builds自动部署运行中。首次Claude工具调用权限不足、未得到审查结果，不作为PASS依据。全量CI/网站上线属于后续远端门，不借之前联合工作区全量计数作本提交结果。


**2026-10-19 12:11:40 UTC 推送及全量回归：** main独立提交185378efd9054b4b81c6f6adde1e022d9ac05b30、网站cf独立提交b77b548138681d82dacfbb9e0c2a6cc82c082f7d均已推送并通过ls-remote核对。cf的Workers Builds a6af15c4-833a-4601-8c17-deb177388aa4实际success；部署93da9acd-4444-4c12-9246-c07879faa6a5为100%流量，线上/en/docs/commands/agent实测HTTP200且含暂停Thinking采集正文。main CI run39248072576的fmt/clippy/Windows scoped-input/OpenCode Linux sandbox已PASS，离线全量/网络/macOS任务当时仍在运行。

不含Codex WIP的本机双env、umask022完整nextest实际853.045s：10337运行，10332PASS（13slow、1flaky）、5FAIL、46skip；fetch mirror prune首轮失败重试后PASS。不将其标成全量通过。5项为config_ownership::every_code_agent_table_and_cache_is_classified（Claude events::COUNTS未登记）、mutable_state_ownership::table_inventory_ignores_ddl_diagnostics_but_keeps_real_statements（与远端table_name_after的严格括号检查叠加漏掉CREATE TABLE AS）、agent_doctor_repair_test::opencode_legacy_capture_is_visible（managed template标签断言）、agent_lifecycle_event_test::hooks_apply_pending_repository_migrations_before_capture（本次DDL指纹把重建后的表名双引号视为结构差异）、hook_public_contract_claude_codex_ci（既有公开字节矩阵漂移）。前述两项本次Grok验证整合问题在同一交付轴修正：DDL name parser统一接受括号列与AS查询、schema指纹仅归一已知agent_session表头引号并保留全部CHECK定义；不变更Codex测试注册/ignored策略或矩阵预期，不修改Claude cache/OpenCode功能。对应专项和strict结果随后登记。Codex与其它WIP仍隔离备份，提交后恢复。

**2026-10-19 12:16:00 UTC Grok验证整合修正：** mutable_state_ownership全部6项实测PASS；hooks_apply_pending_repository_migrations_before_capture实测1/1 PASS，回滚必须改变含CHECK的物理指纹、重放必须恢复当前约束的断言保留。初次仅统一表头引号仍因闭集agent_kind枚举列表的换行格式不同失败，最终只归一该闭集CHECK列表空白及表头引号，保留全部枚举值和其它DDL。nightly fmt check、diff check、严格all-target/all-feature Clippy实际exit0（25.53s）。此次修正限两个既有验证helper与本计划，无新增Codex测试/target或CI配置。完整10337回归结果仍以上述实际5FAIL登记，不能用7个专项PASS改写成全量PASS；其余Claude COUNTS库存、OpenCode legacy标签、公开hook矩阵3项未修，保留交给相应工作流。仅将本次Grok验证修正作签名DCO补充提交并推送；不发布版本，Codex WIP恢复后仍不提交。
