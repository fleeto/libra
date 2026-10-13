# OpenCode coverage schema-v2 前置实施计划（2026-10-13）

**状态：** 实施中；本次 G-09 将初始 19 卡细化为 23 卡（20 implementation+2 audit+1 release）。COV2-01/13/02/07/14/03 六卡已完成本机 A/B、唯一 Claude PASS 与 signed+DCO main 交接，COV2-14 实际 signed main `4558af64c09e63c4716fa68595776a8044f66cc7`；其最终双 env full 10069/10069 exit 0（raw SHA-256 `2849dcf7e2105ed1da5e5d3b0333374e2590ac909c1f478ef8ba68966e86aec4`），Claude PASS raw SHA-256 `2d1e5fb7425abc9aa2802db241213811dfcd6c891410e9853e554c5fcb806966`。当前正式 **0/23**，源码就绪 **6/23=26.09%**；家族 C/D 与父计划 Source A 尚未闭合，父计划正式 **55/67=82.09%**。COV2-03 五项 VER、五项 mandatory ER-04 与两项旧库缺表回归均各 1/1；最终双 env full **Summary [1618.814s] 10078 tests run: 10078 passed (14 slow), 23 skipped**/raw SHA-256 `c49dd3c33652d55541a0e4c0269c817460b9109cf261eab5271d90760a49e8a9`、exit0；最终十文件唯一 Claude Code PASS（P0/P1/P2=0、P3=5；raw SHA-256 `9ef25f92615eefcf2019212a306cfa7542ef416b693286bb99111006b3ea6df7`）；signed+DCO 源码/文档 main `d717e42e0d1ec75e6ae576b21242fd1519040abd` 已 Push 且 remote 验证相同；COV2-19..22 新 AC/VER 亦 UNRUN。G-09 r4 单 Claude 审查 FAIL（2×P2）保留；r5 最终字节 Claude PASS（P0/P1/P2=0，raw SHA-256 `77d4820e329ade99f5fba7b43821d072d4d7068ee5ba639ada4704851686f4fd`）已由 signed+DCO main `eaa43ec13f6c50212f0b3c2fa8a51799ec74bff5` 交付。本次三文件 G-09 规划协调独立于 COV2-14、COV2-03 的实现提交，已作为 COV2-03 执行基线。OG-26 signed+DCO main `734b6f03884d3aa38400acbbfd446b557e642d94` 已交接；原19卡采纳最终字节经单 Claude Code PASS（P0/P1/P2=0），并由签名+DCO main `c94fd8dd60d8d9fa670b2682bf7b9979bae91548` 交付；本文件按计划模板 v2.14 成稿，初始 0/19（历史采纳分母；拆卡后的当前分母见修订历史）。2026-10-13 09:15:00 UTC 的旧只读基线为本地 `main` `dcc468bd31c88a7e8d8c0d2957d6e80b65908990`；2026-10-13 10:59 UTC 的采纳前预核发现 OG-26 新增一个本机 ignored OpenCode Session 案例，使继承清单从旧21例变为22例。2026-10-13 11:16:08 UTC 已只读核 `HEAD=origin/main=734b6f03884d3aa38400acbbfd446b557e642d94`；开工前仍按 ER-01/02 重核逐路径源码/网站状态。来源 R4 草案 SHA-256 `18c34afa261f89100f2d6f0e9ce30953dbc97f7c9c1d7cf8661845d0733f7c2f`。R4 的拟议文件名 `plan-20261013-coverage-v2.md` 已改为符合模板小写字母后缀规则的 `docs/development/plan/plan-20261013-coverage-schema.md`；卡 ID 保持 COV2。

## 文档职责

本文解决 `DEFER-OG25-COVERAGE-V2`：现有 coverage-v1 不能把 OpenCode 可见 reasoning 当第五种生产 `SemanticRecord::Reasoning`。目标是在不重写 v1 历史字节的条件下，以 schema-v2 完成 writer/reader/digest/恢复、live/import、隐私及兼容交付，并在稳定 patch 发布后把已审源码交给父计划 OG-27。本文只规划任务，不宣称已实现、已通过模型或 native 来源证明。

### 适用范围

- Libra Repository 数据库中的 versioned coverage claim/identity map、checkpoint marker、pending/replay、import、graph/doctor。
- OpenCode visible-only reasoning 的 typed v2 路由；proof/state-bearing 候选保持 partial 与零明文 sink；真实 native 来源单独由父计划 Source A 判定。
- 20 张 implementation、2 张 audit、1 张 release（23 张；原采纳 19 张和隔离 r7 的 21 张均为历史口径）；macOS 本机 Session 验收，必要安全 CI 与四平台稳定发布。

### 非目标

- 不修改 `sql/**`、`src/cli.rs`、`history.rs`、native issuer/locator admission，也不修改冻结 `tests/fixtures/agent_transcripts/reasoning/contract.snap` 或 `pending_v1_decoder.rs`。
- 不把 synthetic/fake exporter/普通可读 text 当作真实 `signature`、`redactedData` 或 `reasoningEncryptedContent` 的 Source A；`reasoningEncryptedContent` 正例仍待父计划后续真实路径。
- Linux 功能验收由用户在 Linux 机器后验；本计划仍要求必要安全 CI 和四平台发布构建，不以本机功能范围取消发布质量门。
- 父计划 67 卡分母不变；本计划 G-09 保守拆分后 23 卡单独计数。

### 成功定义

- 带 typed Reasoning 的生产 turn 只走 coverage schema v2；不带该 record 的 v1 turn 原字节/MAC/digest 不变，旧读端对 v2 非破坏性拒绝。
- 同一逻辑 key 可跨 v1/v2 共存，但同版本重复、失效 owner/fence/revision、unknown version、损坏私有 map 均 fail closed，无假成功/假 ack/普通明文泄漏。
- 22 个继承 ignored OpenCode Session 用例（旧21例加 OG-26 新增1例）与本计划新增 ignored Session 用例在最终行为源码的 macOS 手工执行中非跳过通过；真实 visible-only 观测有脱敏记录，不冒称 native 来源。
- 全部 23 卡达到 `Lifecycle=done` 且 `Acceptance=complete`，唯一 patch 发布点完成精确源码全量 Nextest、单 Claude Code PASS、安全 CI、四平台、网站/安装及指定 curl 安装，随后才标父计划 `DEP-OG-13=complete`。

## 事实基线

| 类别 | 当前事实 | 证据（开工刷新） |
|---|---|---|
| 版本契约 | v1 只允许既有四种 semantic record；Reasoning enum 形状已存在但生产 v1 不可直接新增第五种 | `docs/development/tracing/coverage-v1.md`；`src/internal/ai/observed_agents/coverage.rs:276,352` |
| claim/DB | 现有 claim 键包含 `coverage_schema_version`；`read_reasoning_map` 当前限制单 row/identity-v1 | `src/internal/ai/coverage_gate.rs:160,307-362,422,2015-2105` |
| 私有容量 | 现有 identity map 上限 4096 条、1 MiB；v2 必须与 v1 共用 aggregate 容量 | `src/internal/metadata.rs:46,50-51` |
| live 封闭 | 无 ready consumer 时拒绝 source preparation | `src/internal/ai/observed_agents/reasoning.rs:126-129`; `src/internal/ai/capture/live_checkpoint.rs:973-987` |
| 历史 fixture | RG-04 snapshot SHA-256 `1e83e39d582c76d3b297ab8252c109a11eadaed40f2e6d299d221ae750b1b665`; pending-v1 decoder SHA-256 `96d71660e8b75d61cff2dccaa59a410b90b499afd63e10c9dcc2cc678312e32c` | 只读冻结文件；COV2-13/04 重核哈希 |
| 本机 Session | `dcc468b` 时继承21个 ignored OpenCode Session 案例；OG-26 signed main `734b6f03884d3aa38400acbbfd446b557e642d94` 在 `tests/agent_checkpoint_redaction_test.rs` 新增 `og26_candidate_has_zero_normal_sink_hits`，故采纳时继承清单为22个。历史 extraction ignored 测试在 `5cafe322` exit 101 | `tests/agent_checkpoint_redaction_test.rs:1326-1331` 的 macOS/ignored/Required Seatbelt 前提与源码断言；旧21例源文件见 COV2-16/17；`/private/tmp/libra-opencode-ci-handoff-20261013/local-opencode-extraction-fail.log` SHA-256 `7c2db01299c553ae028475db111d521f00a5ba6af46fece709a7ab5b024a19d4`（历史失败，不算 PASS） |
| 网站文档 | 邻仓为 Libra `cf` checkout（只能用 `libra` 作版本控制），目前落后 `origin/cf` 31 提交且 `agent.en.md` 已脏 | 2026-10-13 只读 `libra status --short --branch`；不得覆盖原 checkout |
| parent 进度 | 父计划现有正式 55/67=82.09%，OG-26 为本计划前驱，OG-27 被 coverage-v2 与 Source A 双门阻塞 | `docs/development/plan/plan-20260902.md`；`docs/development/plan/plan-status.md` |
| DEP-OG-12 独立重现 | 2026-10-13 10:49 UTC 独立 r2 仅使私有 `.2` selected binary-to-reviewed-source transformed-output 子门 PASS：selected SHA-256 `28174e893a2b63b75f58af4c77212747f6e1eebce5dd0252cfe64847b342615c`，raw Bun 产物不同，受限变换后逐字节相等；八门退出0。单 Claude 对此**有限子门** PASS，同时列出5个 P2 后续条件；issuer/runtime/transport、Source A、归档均 NO-GO | 独立索引 SHA-256 `b9071d9e5c22ef6eeecb2fa447137719a9a3189282a0adf496e37f07eec57bf1`；Claude raw SHA-256 `a4f933f6875c2c41422b183d3081769c424d279949f306d444c96958140e85eb`；`/private/tmp/og12-independent-r2-review.i5JQq6/HANDOFF.md`。索引创建时仍标 `CLAUDE_REVIEW_PENDING`，审查结论在后续独立 raw/handoff，不得只看索引状态作 GO |

### 当前缺口

| ID | 缺口 | 影响 | 计划动作 |
|---|---|---|---|
| GAP-COV2-01 | v1 不能承载新增 Reasoning 生产 record | typed 归档/完整 claim 不能放行 | COV2-01/13/11 |
| GAP-COV2-02 | 单版本 identity map/read、marker、pending/replay/import 仍有 v1 常量/域 | 双版本同键与故障恢复不能安全成立 | COV2-02/07/14/03/19/20/21/04/22/08/05/09 |
| GAP-COV2-03 | OpenCode Session macOS fake exporter 仍有 bwrap 前置跳过路径；旧 extraction 场景失败 | 本机端到端证据不可算绿 | COV2-12/15/18/16/17 |
| GAP-COV2-04 | 原生 producer/issuer Source A 未闭合；独立 r2 只通过 binary-to-source 有限子门，5个 P2 强化条件仍须归各自 owner；2026-10-13 旧安装版 0.30.48 拒绝私有 `.2` 注册 | OG-27 原生字段 production checkpoint 仍 NO-GO | 父计划 DEP-OG-12；先完成并安装 COV2-REL-01 stable patch，再重新验证原始 issuer epoch0，实测 selected runtime/plugin/transport、proof/receipt、凭据 epoch；不能以 r2 子门或旧版拒绝代替 |

## 与其它计划的关系

| 计划/文档 | 关系 | 本计划处理 |
|---|---|---|
| `docs/development/plan/plan-20260902.md` | `DEFER-OG25-COVERAGE-V2` 的外部承接；父计划固定 67 卡 | OG-26 source handoff 后开工；COV2-17 写 `source-ready`，COV2-REL-01 C/D 完成后写 `complete`，OG-27 main push 才可继续；不冒称 Source A |
| `docs/development/plan/plan-template.md` | 新计划 v2.14 | 遵守 G-03/G-07/G-08/G-10 与 ER-13/14；用户既有直推 main/单 Claude 约束见下 |
| `docs/development/tracing/coverage-v1.md` | 冻结 v1 契约 | COV2-13 只增加必要的 v1/v2 区分说明，保持原历史 fixture 字节 |
| `docs/development/plan/plan-long.md` | 长期能力索引 | 暂无新 LR 编号，本计划仅作为父计划已登记遗留的执行细化；若确需新增 LR 须另行同步并修写集 |

## 评审结论与修订记录

| 维度 | 自审结论 | 计划动作 |
|---|---|---|
| 合理性/接口兼容 | v1 冻结且 typed v2 独立，公开 graph/list envelope 仍为 1 | ADR-COV2-01；COV2-13/11 双 oracle |
| 粒度/恢复 | 20 个单轴实现、两个不写代码的 audit、唯一 release；无 EX 豁免 | 下方 G 审计表；任一实际 AC/VER 超限先按 G-03 拆卡 |
| 依赖/写集 | 全链串行、OG-26 后排他单写者；网站另有已脏 checkout | DEP-COV2-OG26 / SITE；COV2-09/11 仅隔离 `cf` owner 写 |
| 功能/数据流 | marker、map、pending、import 都须按 version/domain 认证并原子恢复 | COV2-02..09、COV2-14、19/20/21/22；故障矩阵 |
| 安全/隐私 | fake exporter 只证明管道；候选状态/证明材料不入明文 | COV2-06/10/12/15/18；Source A 独立 |
| 测试/可靠性 | 22 个继承 ignored cases（原21+OG-26新1）全部需 final-source 本机非 skip PASS | COV2-16/17；无本机 Session CI |
| 性能/容量 | 4096 条/1 MiB 对 v1/v2 共享，不倍增预算 | COV2-14 aggregate cap 回归 |
| 发布/后验 | 单一 patch 点；本机 macOS 功能与四平台发布区分 | COV2-REL-01；Linux 用户后验 |

### 修订历史

| 时间 | 触发 | 变更内容 | 原卡 → 新卡 | 受影响引用 |
|---|---|---|---|---|
| 2026-10-13 09:15:00 UTC | R4 草案转模板 v2.14 | 将 16+2+1 拆分、两阶段 `DEP-OG-13`、macOS 21-case 手工审计、单一 patch 发布点写入独立计划；将 R4 非法数字后缀改为合规文件名 | 草案 COV2-01..18 → 同 ID | 依赖表、REL、矩阵、父计划修订提案 |
| 2026-10-13 10:59 UTC（采纳前预核） | OG-26 待签工作树、DEP-OG-12 独立 r2 | 继承本机 ignored Session 清单从旧21例增至22例；COV2-16 从11例/12 AC/4 VER 调整为12例/13 AC/5 VER。独立 r2 只确认 selected `.2` binary-to-source 有限子门并保留5个 P2 后续，旧 clean r4/0.30.48 仅作历史时点，不推断 Source A | COV2-16 同 ID；19卡分母不变 | 成功定义、事实/缺口、GC-08/09、COV2-12/16、REL-01、矩阵/风险/完成判据；待 OG-26 signed main SHA 与最终 Claude 审查 |
| 2026-10-13 11:16:08 UTC | OG-26 signed main handoff | 实际 `HEAD=origin/main=734b6f03884d3aa38400acbbfd446b557e642d94`；21→22 继承案例计数在签名源码上成立；仅采纳 COV2 计划，未实施任何 COV2 卡，DEP-OG-12/Source A 仍 NO-GO | COV2-01..18 与 COV2-REL-01 保持 0/19（历史） | 逐路径 signed SHA/mode 清单及三路径采纳 patch 在隔离提案中，最终采纳字节仍需单 Claude PASS |
| 2026-10-13 15:52:39 UTC（隔离候选，尚未采纳） | COV2-03 G-03 独立谓词超限及 COV2-04 真实持久化前置 | 原 COV2-03 收窄为 versioned marker/ref 围栏；新增 native identity-v2 companion 与 pending versioned verify/retain 事务 API，COV2-04 在真实持久化 artifact 上验 frozen 旧读端；原始 19 卡采纳分母改为候选当前 21 卡，历史 0/19/1/19/2/19/3/19 保持原时点事实 | COV2-03 → COV2-03/19/20 | 依赖表、REL 成员与逆序、DAG、任务审计表、04/08/REL 卡、测试矩阵、追溯表、里程碑、风险表、状态索引；候选未验收、未签名 |
| 2026-10-13 17:02:33 UTC（历史隔离草案，未采纳） | COV2-03 r7 静态差异与两处独立事务/持久化行为 | 原 COV2-03→19/20；COV2-20 验领/保留拆为只读 verify→COV2-21 retain；原 COV2-04 codec/真实 artifact 拆为 codec/seal→COV2-22 persist/旧读端/故障。新增严格边 20→21→04→22→08；20 implementation+2 audit+1 release，23 卡分母。COV2-03 AC8 的冻结 v1 bytes 移至 mandatory ER-04，保留原 golden 1/1；父计划 COV2 单写者表须显式 I 同步。此行为待 root 在 COV2-14 signed main 后重基、最终单 Claude 审查和签名采纳，所有新 AC/VER/CI 未运行。 | COV2-20→21；COV2-04→22；COV2-03→19/20 保留 | 依赖、REL、粒度、phase、测试/追溯/风险/容量、plan-status 与父计划单写者表 |
| 2026-10-13 17:43:33 UTC | COV2 G-09 23卡规范性修订及COV2-14 signed handoff | 保留原19卡历史收据；COV2-03拆出19/20/21、COV2-04拆出22，按23卡DAG/REL/G-03/写集/矩阵同步。COV2-14 signed main `4558af64c09e63c4716fa68595776a8044f66cc7`，8 AC/7 VER、双env full 10069/10069 exit0/raw `2849dcf7e2105ed1da5e5d3b0333374e2590ac909c1f478ef8ba68966e86aec4`、唯一Claude PASS/raw `2d1e5fb7425abc9aa2802db241213811dfcd6c891410e9853e554c5fcb806966`；COV2-14 交付及 G-09 签署时0/23 formal、5/23=21.74% source-ready；当时 COV2-03新增VER5、旧catalog receipt ER-04回归与COV2-19..22新AC/VER均UNRUN | COV2-03→19/20/21，COV2-04→22；父67卡不变 | 三文件精确I和父单写者表；规划协调另由最终字节单Claude与signed main交付，Source A/C-D仍待 |

## 已决议设计决策

### ADR-COV2-01：生产 v2 与冻结 v1 并存

- **Status:** Accepted for this draft；执行前随计划单 Claude 审查复核。
- **Context:** v1 record 集合和 RG-04 fixture 已冻结。
- **Decision:** 生产 `SemanticRecord::Reasoning` 只选 coverage schema v2；无该 record 的 turn 维持 v1 全字节。显式 legacy projection 只供旧 synthetic oracle。graph/list 公共 envelope `schema_version:1` 不随 claim 的 `coverage_schema_version` 变化。
- **Alternatives considered:** 直接扩展 v1 会破坏 reader/digest 契约；停用 typed reasoning 不能满足父计划完成定义。
- **Consequences/Revisit when:** 每个 writer/reader/replay/import 都需 version-aware；若发现 SQL DDL 必需，先停工并加 GC-13 迁移卡/精确写集。

### ADR-COV2-02：组合键与私有 identity 域

- **Status:** Accepted for this draft。
- **Decision:** pending 唯一键是 `(coverage_schema_version, logical_turn_key)`，同 key 跨版本合法、同版本重复拒绝；identity-v1 byte/MAC 不动，v2 使用独立 key-owner MAC 前缀与 byte-domain、独立私有 map key/codec，v1/v2 共用 4096 条/1 MiB aggregate cap。旧读端可对 v2 fail closed，无透明读取承诺。
- **Revisit when:** frozen v1 oracle 或私有 cap 证明无法在现有 schema 中实现；先修计划，不隐性编辑 DDL。

### ADR-COV2-03：Session 验收与 native 权限分离

- **Status:** Accepted by user's macOS/CI/source-A instructions。
- **Decision:** 任何启动/模拟 OpenCode exporter 并产生或检查 Session/Checkpoint 的测试（含 fail-closed 零写、假 exporter）只在 macOS 显式 `#[ignore]` 本机运行；纯 parser/codec/静态/非 Session DB 仍在 CI。每个本机案例必须实际选择并执行、非提前 return、非 skip。真实可读响应不授予 native issuer/opaque artifact；真实 `signature` 的 Source A 由父计划单独验证。
- **Revisit when:** 用户修改验收策略，且安全与来源门经新的计划修订。

## 全局工程约束

- **GC-COV2-01 基线与单写者：** 每卡先 `libra status --short --branch`，核 `origin/main`、签名+DCO 前驱 SHA、逐路径 hash/mode、脏文件归属；保留现有 `plan-20260904.md`/`plan-20260905.md` 等无关改动。COV2 串行，主仓所有 VCS 操作用 `libra`，每卡 A/B 与唯一 Claude Code PASS 后精确暂存、签名+DCO commit 并直推 `main`；完成一张卡须报告下一张卡及父计划正式百分比（另列COV2百分比）；推送新 commit 后按用户指示取消旧 commit Actions，把 runner 留给新 commit。任何远端/本地旧 SHA 证据不能当新源码 PASS。
- **GC-COV2-02 安全：** fail closed；不在普通、typed、metadata、doctor/graph 输出或日志留候选原始字节；native issuer、locator 与 Source A 不由本计划铸造；脱敏在 digest/projection/persist 之前。
- **GC-COV2-03 兼容：** v1 fixture、pending envelope/MAC 与旧 struct literal 均保持；COV2-03/04/05/07/14/19/20/21/22 的 v1 goldens 均从冻结基线 `dcc468bd31c88a7e8d8c0d2957d6e80b65908990` 的原始字节取值，绝不由修改后代码重生成预期值。旧读端只需非破坏性拒绝 v2。受影响的命令 EN/zh、开发文档、网站 `cf`、`COMPATIBILITY.md` 同卡交付。
- **GC-COV2-04 数据库角色：** 当前无 DDL。受影响角色仅 Repository；GlobalConfig/SystemConfig/Derived 的 schema writer、receipt 和表均不变。若需 migration/bootstrap/top-up/compat 变更，按 GC-13 新增按角色隔离与旧/新 binary 门，修订写集/粒度后才能实施。
- **GC-COV2-05 资源：** v1/v2 两个 map 的条数和编码总字节共享原 4096/1 MiB；解析前限额、有界认证、事务 CAS 与 owner/fence/revision 不得削弱。无新增无界扫描/网络调用。
- **GC-COV2-06 证据：** 测试计数、SHA、run ID、评审结论和发布/站点证据写回本计划和 `docs/development/plan/plan-status.md`，不在仓库新建证据目录/文件。禁止泄露模型 transcript、密钥、token、私有路径或候选明文。终止链末端纯文档记账提交的 D 组结果按模板 v2.14 预登记：由下一次 `docs/development/plan/plan-status.md` 例行同步补记 SHA、run ID、结果、UTC；若至次日无例行同步，做一次明示状态维护提交，不建立无限自引用链。
- **GC-COV2-07 用户单评审授权：** 本对话用户原文“把所有评审改为单评审，只进行 Claude Code 评审，PASS 即可通过”；`plan-status.md:3` 亦记录现行“单 Claude Code PASS”。因此 ER-05 的 Codex 评审循环在本计划由用户较高优先级指令取代；每卡仍必须在最终源码字节上获得一名独立 Claude Code 审查者明确 PASS，处理其所有 material findings，再签名交付。草案的暂行审查不能代替 OG-26 后采纳到 main 的最终字节审查。
- **GC-COV2-08 被动随版源码：** COV2 从 OG-26 签名 main 之后开工，最终稳定 patch 会携带上次稳定 tag 至冻结 main 的**全部**祖先提交，不仅 OG-24/25/26。发布前执行 `libra log <last-stable-tag>..<freeze-main-sha>`，逐提交登记 SHA、归属计划/卡、已审与 C/D 状态；COV2 家族子卡的 C/D 由本次唯一发布点结清。OG-24/25/26 是明确登记的父计划乘载例：其 A/B、单 Claude、signed SHA/hash/mode 与本次安全 CI/安装门必须核实，但 `REL-FAST-OG-04` C/D 是本次发布的**下游**，不得反向要求其先 complete 制造循环；本发布不替它结清 ledger。任何其他计划乘载提交有应先满足的独立安全/验收门则先满足或阻断冻结，不能由 COV2 测试代替。已审的 `src/internal/ai/observed_agents/native_origin/approved_build.rs` 私有 `.2` 许可随新版二进制进入安装面，可能改变 **2026-10-13 当时已安装 0.30.48** 的 `LBR-AGENT-005` 拒绝；必须实装 COV2-REL stable patch 并重验原 issuer epoch0，不能从源码推断注册 GO。OG-27 源码不得混入冻结树。
- **GC-COV2-09 Source A 证据边界：** 旧 clean r4 与 2026-10-13 10:49 UTC 独立 r2 均只处理 selected `.2` binary-to-source；r2 的 raw Bun binary 与 selected 不同，285 个受限 nonce/hash 重建与 45,455 个 code-signature page hash 重算后才逐字节相等，不宣称 raw build determinism、notarization、独立供应链或真实 endpoint transport。独立 r2 Claude 对此有限子门 PASS，但5个 P2 后续须按 DEP-OG-12 owner 在相应强声明前处理：执行时脚本 SHA 绑定、manifest 原字节与 canonical JSON 区分、plugin/policy digest 与 selected binary 直接绑定、clean lockfile 依赖来源、tarball/member 等价复核。COV2 实施不凭此授予 issuer；真实注册、Source A、三字段归档仍受父计划原门。

## 执行检查必备需求

本计划继承 `docs/development/plan/plan-template.md` 的 ER-01..ER-14、GC-01..GC-13、G-01..G-11。逐卡具名 AC/VER 是 ER-04 A/B 的补充：所有会推送的卡仍必须在最终源码上通过 `cargo +nightly fmt --all --check`、`cargo clippy --all-targets --all-features -- -D warnings`；触发 ER-13 T-1/T-4/T-6 的卡须再跑全量；唯一发布点无条件 T-2，在已 bump 的精确源码上用 `source .env.test && source .env.live-test && cargo nextest run --all --no-fail-fast --retries 2`，不把 `cargo test --all` 当 release 通过证据。卡内 exact 测试必须实际报告 1 passed/0 failed（审核组要求组计数），0 test、skip、早退、失败、synthetic/different-source 均不能当 PASS。子卡 `family child` 继承 COV2-REL-01 的 C/D，故 A/B + Claude PASS + signed main 后仅 `locally-accepted`；release 实际 C/D 全绿才逐卡 `complete/done`。

## 实施顺序

### 依赖登记表

| ID | direction | 类型 | 对象 | Owner | 产物与可用性判据 | 证据 | 超时与失败策略 |
|---|---|---|---|---|---|---|---|
| DEP-COV2-OG26 | incoming | 跨计划 | `plan-20260902.md#OG-26` | OG-26 单写者；COV2-01 只读核验接收 | OG-26 最终 A/B、适用 focused/full、fmt/strict、唯一 Claude PASS；signed+DCO main SHA 与 `reasoning.rs`、`opencode_reasoning.rs`、`snapshot.rs` 及相关路径逐文件 post SHA/mode；共享写集已让出。 | 父计划 OG-26 执行记录及实际 main SHA/hash/mode；COV2-01 首笔写前刷新 | 任一缺口则 COV2-01 保持 pending，只可只读准备；当日交接无法证明即停止该卡，不覆盖 OG-26 修改。 |
| DEP-COV2-SITE | incoming | 外部仓库 | libra-backend `origin/cf` 的 `apps/tanstack-app/content/docs/commands/agent.en.md` | 经仓库 VCS 核实的独立 `cf` owner；09/11 互斥 | 记录远端 cf pre-SHA、干净隔离 checkout 根、目标页 pre-hash/mode；同卡 typecheck/build、签名+DCO 推送，发布点另核部署/HTTP。现有原 checkout 落后31且页已脏，不可覆盖。 | 隔离 owner 的 VCS/status/pre-hash 与 build/推送/部署收据，开工时刷新 | 无干净 owner 则 COV2-09 保持 pending 并停止依赖链；绝不以 locally-accepted 假过，同日无法交接即登记 blocked；发布 D 不得声称网站已部署。 |
| DEP-COV2-MODEL | incoming | 本机外部模型 | 用户已配置的 OpenCode Anthropic 可读响应路径（非 native 来源证明） | COV2-17 macOS 本机执行者 | 最终行为源码上观察 genuine visible-only 响应，必须由已审 classifier 证实该样本无 state/proof-bearing 候选；记录无敏字段名、JSON 类型/数量、版本与 keyed digest；如实际返回 `signature`/`originProof`，该样本不算 visible-only 正例。不登记 issuer、不触发 live mint、不把它当 Source A。 | 本机 sanitized 1/1 观察记录、实际 build/model/plugin/transport ID、最终源码 SHA；不得记录正文/密钥 | 每个不同最终行为源码 SHA 最多一次本机尝试、10 分钟；COV2-REL bumped SHA 的复测另计一次。无样本或模型失败则 COV2-17/REL 保持 blocked，明确 UNRUN，不用 synthetic 代替。 |
| DEP-COV2-OG27 | outgoing | 跨计划 | `plan-20260902.md#OG-27` / `DEP-OG-13` | 父计划 OG-27 接收者 | COV2-17 在全部22例本机非 skip PASS 且20张 implementation 源卡签名 main 交接后仅登记 `source-ready`，只允许隔离准备；COV2-REL-01 stable patch、四平台、安全 CI、站点/安装 C/D 全绿后才登记 `complete` 并许可 OG-27 main Push。接收者不重做 coverage-v2 家族实现，仍独立证明 `DEP-OG-12` Source A。 | 新计划 COV2-17/COV2-REL-01 报告、release run/manifest/site/install SHA/UTC；OG-27 开工时刷新 | 若接收者下一工作日未接手或门未绿，本计划保持 complete/source-ready 事实、父计划 OG-27 blocked；不替接收者做 native 注册、不以 synthetic 或旧版拒绝充当 GO。 |

**G-09 计划内新增顺序边：** `COV2-03→COV2-19→COV2-20→COV2-21→COV2-04→COV2-22→COV2-08`。前驱必须以最终 A/B、适用 focused/full、fmt/strict、唯一 Claude PASS 及 signed+DCO main SHA/hash/mode 交接；后继不能把其继承的家族 C/D `locally-accepted` 冒称 complete。外部 `DEP-*` 不变。

网站卡写集中的 `apps/tanstack-app/content/docs/commands/agent.en.md` 是 libra-backend 仓库的精确相对路径；执行前把实际 `origin/cf` pre-SHA、隔离 checkout 的绝对根路径和该文件的 pre-hash/mode 登记在 DEP-COV2-SITE。只在核验过的干净隔离 `cf` checkout 编辑该相对路径；`cf` 是分支名，不属于文件写集。原脏 checkout 不可写。

### 发布分组与并发窗口

| ID | 成员 | 唯一发布点 | 窗口规则 | 失败回滚顺序 | 理由 |
|---|---|---|---|---|---|
| REL-COV2-01 | `COV2-01→13→02→07→14→03→19→20→21→04→22→08→05→09→06→10→11→12→15→18→16→17`；20 implementation + 2 no-release audit | COV2-REL-01，仅此卡写 R | OG-26 签名交接后开窗；每张子卡本机 A/B、Claude PASS、signed+DCO main Push，但不 bump/tag/release；不得让其它计划在半成品 COV2 main 上发布稳定版。COV2-11 前无生产入口可激活 v2 writer，并以静态/运行时守卫证明。冻结/bump/tag 时禁止其它 COV2/OG-27 源码及 R 写入；若其它发布无法停，先证明其中不携带可触达的半成品 v2，否则 COV2 链暂停。 | `17→16→18→15→12→11→10→06→09→05→08→22→04→21→20→19→03→14→07→02→13→01` 逆序定位受影响 owner，签名前滚，重跑下游门并重新冻结；已发版 tag/asset 不改写，只出新 patch。 | schema/reader/持久化不能分开用户可见发布；G-08 唯一版本点。 |

**并发声明:** 全串行；libra-backend `cf` 网站 owner 与主仓写者分仓，09/11 的同一网站页仍互斥。

**发布者:** COV2-REL-01 单一发布者；开工时登记实际执行者，不预设姓名。

**发布窗口顺序:** 22 张子卡（20 implementation+2 audit）按上表顺序 signed+DCO main 交接；COV2-17 source-ready；COV2-REL-01 固定同一 bumped tree 的全量与版本面、单 Claude PASS、唯一 tag/release 和 D 组；之后方可 OG-27 main Push。同一时刻只允许一张卡持有 R 写锁。

**家族发布解释：** schema 与 reader 不能独立对外发布，故 G-08 例外采用唯一发布点。用户已授权每卡直推 `main`；与模板原 `family child` 本地提交流程的差异限于直接 main 源码交付，不赋予子卡 R/版本/发布权限。任何新 source 修复之后，审计和 release freeze 必须按最终 SHA 重做；不使用 `EX-OG-02` 或新 G-03 waiver。

**严格 DAG：** `OG-26 → COV2-01 → COV2-13 → COV2-02 → COV2-07 → COV2-14 → COV2-03 → COV2-19 → COV2-20 → COV2-21 → COV2-04 → COV2-22 → COV2-08 → COV2-05 → COV2-09 → COV2-06 → COV2-10 → COV2-11 → COV2-12 → COV2-15 → COV2-18 → COV2-16 → COV2-17 → COV2-REL-01 → OG-27 main Push → OG-28 → OG-29 → OG-05`。COV2-09/11 各另等 DEP-COV2-SITE；OG-27 另等父计划 Source A/DEP-OG-12。后继只消费前驱最终 A/B+Claude+signed main 的 source-qualified deliverables，不把前驱继承 C/D 假记 complete；发布点不能以 source-ready 替代 C/D。

### 字段全局默认与例外

- 所有 implementation：`Task type=implementation`、`Lifecycle=pending / Acceptance=空`、`Rollback mode=forward-only`、`Migration and rollback=无 DDL；签名前滚且历史 v1 不重写`、`Version increment=N/A`、`Release boundary=family child of REL-COV2-01`、`Release write set=N/A`、`C/D coverage from=COV2-REL-01`、`Estimated scope=M`。
- COV2-16/17：`Task type=audit`、`Lifecycle=pending / Acceptance=空`、`Rollback mode=forward-only`（审计失败返回实现 owner 重做，不撤销生产数据）、`Release boundary=no-release`、`Version increment=N/A`、`Release write set=N/A`、`C/D coverage from=COV2-REL-01`、`Estimated scope=S`。
- COV2-REL-01：`Task type=release`、`family release point of REL-COV2-01`、`Rollback mode=immutable-release`、`Version increment=patch`；版本面由 `compat_version_surface_sync` 定义，当前 2/2 tests 覆盖 `Cargo.toml`、`install.sh`、`install.ps1`，另有工具链 `Cargo.lock`。
- `I-core` 精确集合：`docs/development/plan/plan-20261013-coverage-schema.md`; `docs/development/plan/plan-status.md`; `docs/development/tracing/coverage-v2.md`; `docs/development/tracing/agent.md`; `COMPATIBILITY.md`。`I-audit` 精确集合：`docs/development/plan/plan-20261013-coverage-schema.md`; `docs/development/plan/plan-status.md`; `docs/development/tracing/coverage-v2.md`。卡内 I=相应集合加卡内显列路径；无 glob 授权。父 `docs/development/plan/plan-20260902.md` 在采纳本计划时由独立 parent/status amendment 登记依赖；本次 COV2-03 前 G-09 拆卡规划协调独立于 COV2-14 和 COV2-03 两张实现卡，以**单独 signed+DCO main commit**交付，精确 I 仅为 `docs/development/plan/plan-20261013-coverage-schema.md`、`docs/development/plan/plan-status.md`、`docs/development/plan/plan-20260902.md` 三文件。额外父计划路径仅同步外部 COV2 单写者表、COV2 数量/追溯，不作 COV2-03 行为实施权。之后实施阶段仅 COV2-17（source-ready）和 COV2-REL-01（complete/release）可写父计划。规划提交仍需最终字节单 Claude PASS、fmt/strict、signed+DCO main；不凭隔离补丁预记。所有重叠 I 由 DAG 串行。
- 每卡即使只改文档仍要遵守 GC-12 精确暂存、fmt/strict Clippy、单 Claude Code PASS；与用户当前单评审决策一致。无额外 waiver。

### 任务卡粒度审计表

| 任务 | type | axis | recovery | complete | self-contained | AC | VER | landing/prod-files | scope | deps | writeset | release | split-from | exception |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| COV2-01 | implementation | 生产 v2 选择与规范编码 | forward-only | yes | yes | 8/8 | 3/8 | 1/2 | M | DEP-COV2-OG26 | 严格串行 | family child | N/A | N/A |
| COV2-13 | implementation | 冻结 v1/RG-04 合同校验 | forward-only | yes | yes | 8/8 | 4/8 | 1/1 | M | COV2-01 | 严格串行 | family child | COV2-01 | N/A |
| COV2-02 | implementation | 版本化 DB claim 仲裁 | forward-only | yes | yes | 8/8 | 3/8 | 1/1 | M | COV2-13 | 严格串行 | family child | N/A | N/A |
| COV2-07 | implementation | 密钥所有者 identity-v1/v2 MAC 域 | forward-only | yes | yes | 8/8 | 4/8 | 2/3 | M | COV2-02 | 严格串行 | family child | COV2-02 | N/A |
| COV2-14 | implementation | 双版本私有 identity map 与共同容量 | forward-only | yes | yes | 8/8 | 7/8 | 2/2 | M | COV2-07 | 严格串行 | family child | COV2-07 | N/A |
| COV2-03 | implementation | 版本化 marker 与事务围栏 | forward-only | yes | yes | 8/8 | 5/8 | 3/5 | M | COV2-14；先行 G-09 精确 I 协调 | 严格串行 | family child | N/A | N/A |
| COV2-19 | implementation | sealed-native v2 identity 生产提交 | forward-only | yes | yes | 8/8 | 5/8 | 2/2 | M | COV2-03 | 严格串行 | family child | COV2-03 | N/A |
| COV2-20 | implementation | 版本化 live reservation 只读验领 | forward-only | yes | yes | 7/8 | 3/8 | 1/1 | M | COV2-19 | 严格串行 | family child | COV2-03 | N/A |
| COV2-21 | implementation | 调用方事务内版本化 reservation retain | forward-only | yes | yes | 7/8 | 4/8 | 1/1 | M | COV2-20 | 严格串行 | family child | COV2-20 | N/A |
| COV2-04 | implementation | 有界认证 pending-v2 codec 与内存 seal | forward-only | yes | yes | 8/8 | 4/8 | 1/2 | M | COV2-21 | 严格串行 | family child | N/A | N/A |
| COV2-22 | implementation | 真实 pending artifact 事务与冻结旧读端 | forward-only | yes | yes | 8/8 | 6/8 | 1/1 | M | COV2-04 | 严格串行 | family child | COV2-04 | N/A |
| COV2-08 | implementation | pending 同版本恢复 | forward-only | yes | yes | 8/8 | 4/8 | 2/5 | M | COV2-22 | 严格串行 | family child | COV2-04 | N/A |
| COV2-05 | implementation | 版本化 import cursor/digest/故障 | forward-only | yes | yes | 7/8 | 4/8 | 1/1 | M | COV2-08 | 严格串行 | family child | N/A | N/A |
| COV2-09 | implementation | graph/doctor 双版本只读契约 | forward-only | yes | yes | 8/8 | 4/8 | 1/1 | M | COV2-05；DEP-COV2-SITE | 严格串行 | family child | COV2-05 | N/A |
| COV2-06 | implementation | OpenCode typed 来源分类的未激活交付 | forward-only | yes | yes | 8/8 | 4/8 | 1/3 | M | COV2-09；DEP-COV2-OG26 | 严格串行 | family child | N/A | N/A |
| COV2-10 | implementation | typed 脱敏与零明文 sink | forward-only | yes | yes | 8/8 | 4/8 | 2/4 | M | COV2-06 | 严格串行 | family child | COV2-06 | N/A |
| COV2-11 | implementation | live/import v2 生产接线与字节对齐 | forward-only | yes | yes | 8/8 | 6/8 | 3/5 | M | COV2-10；DEP-COV2-SITE | 严格串行 | family child | COV2-06 | N/A |
| COV2-12 | implementation | macOS 本地 Session 入口与既有 fixture 修复 | forward-only | yes | yes | 8/8 | 6/8 | 1/1 | M | COV2-11 | 严格串行 | family child | COV2-06 | N/A |
| COV2-15 | implementation | 候选 live Session 权限防火墙 | forward-only | yes | yes | 8/8 | 2/8 | 2/2 | M | COV2-12 | 严格串行 | family child | COV2-12 | N/A |
| COV2-18 | implementation | 候选 import Session 权限防火墙 | forward-only | yes | yes | 8/8 | 2/8 | 2/2 | M | COV2-15 | 严格串行 | family child | COV2-15 | N/A |
| COV2-16 | audit | 既有 bridge/pipeline/OG-26 12 例本机审计 | forward-only | yes | yes | 13/20 | 5/20 | 0/0 | S | COV2-18 | 严格串行 | no-release | N/A | N/A |
| COV2-17 | audit | 既有 lifecycle/crash/import/oracle 10 例及真实可读响应本机审计 | forward-only | yes | yes | 12/20 | 6/20 | 0/0 | S | COV2-16；DEP-COV2-MODEL | 严格串行 | no-release | N/A | N/A |
| COV2-REL-01 | release | reviewed family patch | immutable-release | yes | yes | 12/12 | 12/12 | 0/0 | M | COV2-17,DEP-COV2-SITE,DEP-COV2-MODEL | 唯一 R 写者 | family release point | N/A | N/A |

### Phase 0：OG-26 交接与 v1/v2 基础

OG-26 的最终 signed main 源码和共享路径逐字节交接完成后，依顺序执行 COV2-01/13/02/07/14/03/19/20/21/04/22/08。每卡先刷新事实锚点与 G-03/G-04/G-10；不在脏 shared path 上双写。

### Phase 1：import/reader/OpenCode typed 接线

严格接 COV2-05/09/06/10/11；COV2-09 与 COV2-11 的网站文档只在干净隔离 `cf` owner 中实施；同卡门是编辑、typecheck/build 与签名 `cf` 推送，部署/HTTP 由 COV2-REL-01 D 门验收，不把本地页修改当上线。

### Phase 2：macOS Session 与完整审计

COV2-12/15/18 新增 ignored 本机 Session 正负例；COV2-16/17 只读核对 22 个继承 ignored 测试实际执行。最终报告只记录所选测试、pass/fail/skip、版本/源码 SHA、必要的无敏字段类型与哈希。Linux 功能留给用户。

### Phase 3：唯一 patch 发布与交接

COV2-REL-01 冻结不含 OG-27 源码的最终 COV2 树，完成 bumped exact-tree full、安全 CI、四平台 stable、网站、Homebrew、manifest 与 `curl --proto '=https' --tlsv1.2 -sSf https://download.libra.tools/install.sh | sh` 本机安装。发布后 `DEP-OG-13=complete`，OG-27 还必须独立解决 Source A。2026-10-13 当时已安装的 0.30.48 拒绝私有 `.2`（LBR-AGENT-005），只读负面收据 SHA-256 `81c9d1ec7142785357c9b121bc3edf5c0700ed27f974874d1a7ff86e1a7bacb1`；这是**版本及时点限定的历史事实**，未来稳定 patch 安装后必须重测，不能把新版本注册结果从该负面事实或独立 r2 binary-to-source 子门推断出来。

## 任务卡

每张卡都以本节具名判据和精确写集为执行边界。`(new)` 命令标识计划新增测试；既有守卫明确标记 existing。执行时 `source .env.test`/`.env.live-test` 只加载本机配置，不回显凭据。任何 exact selector 报 0 tests 都是失败。卡内 `Implementation write set` 与上方 `I-core/I-audit` 定义合起来是精确路径全集，网站绝对路径须由另一个已验证 `cf` owner 完成。

### Task COV2-01：生产 v2 选择与规范编码

**Task type:** implementation

**Lifecycle / Acceptance:** in-progress / locally-accepted（具名本机门、ER-13 全量与唯一 Claude Code 最终 PASS；本卡 signed main Push 随本提交交付，家族 C/D 仍待完成）

**Description:** 新增显式 coverage-v2 选择/规范编码 API，同时保留现有 `safe_turn_projection` 的 legacy-v1 语义；RG-04 `reasoning.rs::tests::contract_serde_snapshot` 继续调用旧 API 比对冻结 `contract.snap`。本卡只定义版本化编码边界，COV2-11 才把生产 live/import 调用点切到新 API；不得以 synthetic/native 互换验收。

**Out of scope:** v1 冻结向量由 COV2-13 验证；claim/事务由 COV2-02/03 承接；无本卡 OpenCode Session。不得在本卡悄然改变 `NormalizedTurn::digest_hex`、DB claim 版本常量或生产调用点；若确需改 `reasoning.rs` 的既有测试，先修 COV2-01 精确 I 集与 G 审计再编辑。

**Current evidence:** src/internal/ai/observed_agents/coverage.rs:276,352; docs/development/tracing/coverage-v1.md（基线 `dcc468b`；ER-02 开工刷新行号与实际源码）。

**Acceptance criteria:**

- [x] AC1 typed Reasoning selects v2
- [x] AC2 a turn without typed Reasoning selects v1
- [x] AC3 an unsupported coverage version fails closed
- [x] AC4 v2 Reasoning uses the closed four-field canonical shape
- [x] AC5 its bytes equal the new golden
- [x] AC6 its SHA-256 digest equals the new golden
- [x] AC7 the existing `NormalizedTurn` wire form is unchanged
- [x] AC8 the default graph/list public envelope schema version remains unchanged.

**Verification:**

- [x] VER1→AC1/2/3: `source .env.test && cargo test --lib internal::ai::observed_agents::coverage::tests::cov2_selects_from_typed_turn -- --exact` (1/1；root log SHA-256 `93d2487931c6f699c27fd6fe17813b4a69d05fdcefa1c22725e25f39615022b3`).
- [x] VER2→AC4/5/6: `source .env.test && cargo test --lib internal::ai::observed_agents::coverage::tests::cov2_closed_reasoning_golden -- --exact` (1/1；root log SHA-256 `7b48c9097c93a4da1ef894b72ed4d38c116dfb8379539e8648726b6003b7b6eb`).
- [x] VER3→AC7/8: `source .env.test && cargo test --test agent_reasoning_contract_test cov2_turn_wire_and_public_envelope_are_stable -- --exact` (1/1；root log SHA-256 `04b91714e2fb2c879de2eda0539eb44e5b33ed37f94a8a8416acbac96c2ac43b`; asserts default graph/list=1 and explicit list-v2 remains accepted). No OpenCode Session was invoked. `reasoning.rs::contract_serde_snapshot` exact 1/1 (root log SHA-256 `48f04e23bcdc426acfdbd7e6212371e5e44bcde5baa77ceb91be8741c390ae0d`), still calls legacy `safe_turn_projection`; frozen `contract.snap` SHA-256 `1e83e39d582c76d3b297ab8252c109a11eadaed40f2e6d299d221ae750b1b665` unchanged. New v2 API does not overwrite that legacy function.

**Final A/B local evidence:** `cargo +nightly fmt --all --check` exit 0；`cargo clippy --all-targets --all-features -- -D warnings` exit 0（raw SHA-256 `ce387dd8e31968c025fdd8ba63e0b5902fd0341d3d409b50665a660039d4d334`）；双 env `cargo nextest run --all --no-fail-fast --retries 2` exit 0，**10053/10053 passed、0 failed、23 skipped**（16 slow、3 leaky；raw SHA-256 `fa3327517528a07abd8ece858e56a519d70d010b7a1ac56e80890c39665cc7c0`）。23 skipped 不算 OpenCode Session/macOS 实验验收；本卡没有模型调用。首轮在最终 test-only Display 断言增补后由 root 中止，exit 130/raw SHA-256 `354f6c3488b391a1d27ac2f77b0cdff595c01bbe14fac4579596be25523ca1a4`，不计通过。最终源码八个非状态文件均为 mode `100644`，SHA-256 依次为 `coverage.rs` `91c0232fe09b299a02523c9b3fe53ee44408511c519d65c3015675cb0cd114a3`、`mod.rs` `e703995210ddc6932b40a269c139c15600d46072e7bffc56e61623f99e9153ec`、`agent_reasoning_contract_test.rs` `da73b9b354d76ebc2c8b0479a05e1824d5e7b894424537f574dece852333de57`、`coverage_v2_golden.json` `77d480e93846df7cb17854fbe934293c0416c57bf486e257c85874b9cb87c78b`、`tests/INDEX.md` `4a1b8629a7a872386e9a4f1381dcfdbc9808201c39fdca1730513fcf26000fc8`、`coverage-v2.md` `69aaf9ab74fa0c9accb080fe80199a695f2a042dc2a8f7829e84dca0172c115c`、`agent.md` `2a26ff5c2c1bc9b45114a7363c76f7a3d1fbd4ff9c94b0f05e04930ec8e9035e`、`COMPATIBILITY.md` `d8dcc4b2cca00e31e459f9d9dc964315d98a9702ac3c898606930e0c6fa4f2be`。计划/状态自描述文件以本卡 signed main tree 为源锚。唯一 Claude Code 最终 PASS（P0/P1/P2=0，raw SHA-256 `5e5ad9af5b9b917054ba5a94d6c17629d9e9b6b959edc8033b6ee72cfaa9c64c`），只复核源码/文档；对实测日志 SHA 的最终记账由 root 核验。正式 C/D 继承发布卡。

**Full-suite trigger:** `coverage.rs` 是 live/import/export 共用规范 turn/digest 源，本卡客观命中 ER-13 T-1；在 COV2-01 最终源码的 A/B 阶段必须运行 `source .env.test && source .env.live-test && cargo nextest run --all --no-fail-fast --retries 2`。COV2-REL-01 T-2 仍须在 bumped release 树上独立重跑，不能复用本卡全量。

**Dependencies:** DEP-COV2-OG26；消费前驱实际最终 A/B、适用 focused/full、fmt/strict、单 Claude PASS 与 signed+DCO main 的 source-qualified 文件 SHA/mode；不以其尚待 C/D 的 `locally-accepted` 冒称 complete。

**Deliverables:** 无独立文件；本卡实现/测试/文档随唯一发布点交付；每卡的计数、源码 SHA、hash/mode、review/门结果写在本计划正文，不建证据文件。

**Implementation write set:** I-core = `docs/development/plan/plan-20261013-coverage-schema.md`; `docs/development/plan/plan-status.md`; `docs/development/tracing/coverage-v2.md`; `docs/development/tracing/agent.md`; `COMPATIBILITY.md`；本卡新增精确路径 = `src/internal/ai/observed_agents/coverage.rs`; `src/internal/ai/observed_agents/mod.rs`; `tests/agent_reasoning_contract_test.rs`; `tests/fixtures/agent_transcripts/reasoning/coverage_v2_golden.json`; `tests/INDEX.md`。无 glob 默示写权。

**Release write set:** N/A；C/D 继承 COV2-REL-01。

**Files likely touched:** `docs/development/plan/plan-20261013-coverage-schema.md`; `docs/development/plan/plan-status.md`; `docs/development/tracing/coverage-v2.md`; `docs/development/tracing/agent.md`; `COMPATIBILITY.md`; `src/internal/ai/observed_agents/coverage.rs`; `src/internal/ai/observed_agents/mod.rs`; `tests/agent_reasoning_contract_test.rs`; `tests/fixtures/agent_transcripts/reasoning/coverage_v2_golden.json`; `tests/INDEX.md`。

**Docs and compatibility impact:** `docs/development/tracing/coverage-v2.md`; `docs/development/tracing/agent.md`; `COMPATIBILITY.md`；逐文件核内容与源码行为对应，`tests/INDEX.md` 只在本卡精确写集确有索引变化时同步。无新增网站命令行为；若实施引入公开差异，先按 ER-06a 修本卡写集再编辑。

**Rollback mode:** forward-only；恢复时签名+DCO 新 main 修复，既有 v1 claim/artifact/digest 不回写或重解释。

**Migration and rollback:** 无 DDL；Repository claim 仅用既有 versioned 键。若实现证明 DDL 必需，停止、加 GC-13 角色隔离迁移卡与旧/新读端门，再修本卡写集和 G-03/G-04。

**Security and privacy:** 继承 GC-COV2-02/03；错误或不可信版本必须 zero-success、不得向 ordinary/typed/log/error 导出候选内容；Session 例只在本机 ignored 执行。

**Performance budget:** 继承 GC-COV2-05；identity map 的 v1+v2 总量仍限 4096 条/1 MiB，任何新增解析在分配前有界；变更 hot path 时以具名门证明无无界扫描。

**Estimated scope:** M（落点/生产文件计数：8/8, 3/8, 1, 2; M）。

**Version increment:** N/A；此卡不发版。

**Release boundary:** family child of REL-COV2-01（1/20）；用户已授权逐卡 reviewed signed+DCO 直推 main，但不写 R、不 tag、不调用 release。

**C/D coverage from:** COV2-REL-01；发布点的 exact-source C/D 绿后才能 `done/complete`。

**Granularity:** `type=implementation; axis=生产 v2 选择与规范编码; recovery=forward-only; complete=yes; self-contained=yes; AC=8/8; VER=3/8; landing=1; prod-files=2; scope=M; deps=DEP-COV2-OG26; writeset=串行于前驱; release=family child; split-from=N/A; exception=N/A`

### Task COV2-13：冻结 v1/RG-04 合同校验

**Task type:** implementation

**Lifecycle / Acceptance:** in-progress / locally-accepted（8 AC、4 VER、fmt、严格 Clippy 与唯一 Claude Code PASS；signed+DCO main `cd698ade182a935c4d91b0f84cb3ad13e8325e0b` 已交付，家族 C/D 仍待）

**Description:** frozen v1/RG-04 legacy oracle。本卡唯一行为轴、恢复和粒度见下方 Granularity；不得以 synthetic/native 互换验收。

**Out of scope:** 不让历史 synthetic v1 fixture 授权生产 typed v1；生产接线由 COV2-11 承接。

**Current evidence:** 冻结 `contract.snap` SHA-256 `1e83e39d582c76d3b297ab8252c109a11eadaed40f2e6d299d221ae750b1b665` 未变；当前 `coverage.rs:2134,2163,2908` 与 `tests/compat/agent_architecture_guard.rs:10653` 是本卡具名 oracle，起点为 signed main `a3b51fd103fc2aa95f64c3395339a00937060356`。

**Frozen vector authority:** `docs/development/tracing/coverage-v1.md` §5 与基线 `dcc468bd31c88a7e8d8c0d2957d6e80b65908990:src/internal/ai/observed_agents/coverage.rs` 的既有 `golden_vector_1_user_assistant` / `_2_tool_call_and_result` / `_3_escapes_and_unicode` 测试字面量是预变更 oracle；基线源码 blob SHA-256 `796aef2cf5717d6ef133c5c7f727c08aede2861af5eab3691ce858ef32b00d85`。三个 digest 固定为 `df991cd9a1ac5c12c32b8cdf0254c3dfbbf26485b505a5afc83a90d1128ebc54`、`af085109b1584fd77bb3b752c1141c1c0f0f12b51a163e94b0f91ddd329460a4`、`f1e76bf75df5d6b0f67a46806abb256cd6de30eaea30e45254a8a66cf5183356`，不可由修改后的 codec 重新生成预期值。`contract.snap` 原字节 SHA-256 `1e83e39d582c76d3b297ab8252c109a11eadaed40f2e6d299d221ae750b1b665`，只读。

**Acceptance criteria:**

- [x] AC1：v1 向量 1 的 canonical bytes 与冻结值相同。
- [x] AC2：v1 向量 1 的 SHA-256 digest 与冻结值相同。
- [x] AC3：v1 向量 2 的 canonical bytes 与冻结值相同。
- [x] AC4：v1 向量 2 的 SHA-256 digest 与冻结值相同。
- [x] AC5：v1 向量 3 的 canonical bytes 与冻结值相同。
- [x] AC6：v1 向量 3 的 SHA-256 digest 与冻结值相同。
- [x] AC7：RG-04 的 contract.snap blob 与冻结原件字节完全相同。
- [x] AC8：既有 RG-04 synthetic 投影经显式 legacy-v1 oracle 精确匹配该 fixture。

**Verification:**

- [x] VER1→AC1/3/5: `source .env.test && cargo test --lib internal::ai::observed_agents::coverage::tests::cov2_v1_vector_bytes_frozen -- --exact` (1/1；root log SHA-256 `0f5d3812009e80cdd45feb9426e39df4ff59271c19838d102b2e9a9d88279219`).
- [x] VER2→AC2/4/6: `source .env.test && cargo test --lib internal::ai::observed_agents::coverage::tests::cov2_v1_vector_digests_frozen -- --exact` (1/1；root log SHA-256 `becd6b4664b2a05e8f0df2bd3a74bfcdf735da8dc5906d659a598d16c2c125b1`).
- [x] VER3→AC7: `source .env.test && cargo test --test compat_agent_architecture_guard cov2_rg04_contract_snap_sha_is_frozen -- --exact` (1/1；root log SHA-256 `3c6238f008c2c98ce3607dcad876d8a086b738550d98f3b9e0b7460df352da5c`；pinned historical SHA，file read-only).
- [x] VER4→AC8: `source .env.test && cargo test --lib internal::ai::observed_agents::coverage::tests::rg04_fixture_provider_visible_conformance -- --exact` (existing 1/1；root log SHA-256 `38b7aa8bdca07f17b2b901c54b6e527bfda9a4832cbb2f4362a8851cbeb20c7c`；只比对 legacy-v1 投影与原 fixture，未改 fixture). The two paths cannot be conflated.

**Final A/B local evidence:** 四项具名 VER 在根工作区最终源码各 1/1、0 skipped；`cargo +nightly fmt --all --check` exit 0；`cargo clippy --all-targets --all-features -- -D warnings` exit 0（raw SHA-256 `c390c26cd50bf4f19aaef41fdef60b34bd766cfd7f8a506e323a3bc38b679302`）；`libra diff --check` exit 0。四个非状态文件均为 mode `100644`，SHA-256：`coverage.rs` `f2ad98262d69332bec0ba9b7a91861006203ea56a828cbbe16f77b1b07c55ccd`、`agent_architecture_guard.rs` `46d67ec907e09bd8982ef5713d03cdf0cf4419ddc1d0346511fe6d85c73bbae3`、`coverage-v1.md` `d1c8bd683a40eb9a02bc106ee5c61715e4c13cac2234cbcccb33c9873b7efd4a`、`coverage-v2.md` `5c61bf28a1689423d0f20f9baf69b36a015914be15914a789ec09d3edfad4086`。`contract.snap` 仍为原 SHA-256；这四项是 synthetic/冻结兼容证据，不证明真实 native Source A，也不运行 OpenCode Session。唯一 Claude Code 最终 **PASS**（P0/P1/P2=0；raw SHA-256 `f900a64d58bc2b125145cec3e608d9356bb613bb8a0aca7e6d0338efb34b3afd`）；后审仅补本行和状态事实，signed main 随本提交交付。

**Full-suite trigger:** 本卡只在现有文件新增 `#[cfg(test)]` oracle、既有 compat 测试和文档说明，不改 production helper、测试基建、公开 CLI/错误码、SQL 或 CI；按 ER-13 复核未命中 T-1/T-4/T-6，四项 exact VER 已覆盖 8 AC，因此本卡不独立重跑全量。COV2-01 的 10053/10053 仅是前驱源码证据，不冒称本卡全量；COV2-REL-01 T-2 仍须在 bumped 树上重跑。

**Dependencies:** COV2-01；消费前驱实际最终 A/B、适用 focused/full、fmt/strict、单 Claude PASS 与 signed+DCO main 的 source-qualified 文件 SHA/mode；不以其尚待 C/D 的 `locally-accepted` 冒称 complete。

**Deliverables:** 无独立文件；本卡实现/测试/文档随唯一发布点交付；每卡的计数、源码 SHA、hash/mode、review/门结果写在本计划正文，不建证据文件。

**Implementation write set:** I-core = `docs/development/plan/plan-20261013-coverage-schema.md`; `docs/development/plan/plan-status.md`; `docs/development/tracing/coverage-v2.md`; `docs/development/tracing/agent.md`; `COMPATIBILITY.md`；本卡新增精确路径 = `src/internal/ai/observed_agents/coverage.rs`; `tests/agent_reasoning_contract_test.rs`; `tests/compat/agent_architecture_guard.rs`; `docs/development/tracing/coverage-v1.md`。无 glob 默示写权。

**Release write set:** N/A；C/D 继承 COV2-REL-01。

**Files likely touched:** `docs/development/plan/plan-20261013-coverage-schema.md`; `docs/development/plan/plan-status.md`; `docs/development/tracing/coverage-v2.md`; `docs/development/tracing/agent.md`; `COMPATIBILITY.md`; `src/internal/ai/observed_agents/coverage.rs`; `tests/agent_reasoning_contract_test.rs`; `tests/compat/agent_architecture_guard.rs`; `docs/development/tracing/coverage-v1.md`。

**Docs and compatibility impact:** `docs/development/tracing/coverage-v2.md`; `docs/development/tracing/agent.md`; `COMPATIBILITY.md`; `docs/development/tracing/coverage-v1.md`；逐文件核内容与源码行为对应，`tests/INDEX.md` 只在本卡精确写集确有索引变化时同步。无新增网站命令行为；若实施引入公开差异，先按 ER-06a 修本卡写集再编辑。

**Rollback mode:** forward-only；恢复时签名+DCO 新 main 修复，既有 v1 claim/artifact/digest 不回写或重解释。

**Migration and rollback:** 无 DDL；Repository claim 仅用既有 versioned 键。若实现证明 DDL 必需，停止、加 GC-13 角色隔离迁移卡与旧/新读端门，再修本卡写集和 G-03/G-04。

**Security and privacy:** 继承 GC-COV2-02/03；错误或不可信版本必须 zero-success、不得向 ordinary/typed/log/error 导出候选内容；Session 例只在本机 ignored 执行。

**Performance budget:** 继承 GC-COV2-05；identity map 的 v1+v2 总量仍限 4096 条/1 MiB，任何新增解析在分配前有界；变更 hot path 时以具名门证明无无界扫描。

**Estimated scope:** M（落点/生产文件计数：8/8, 4/8, 1, 1; M）。

**Version increment:** N/A；此卡不发版。

**Release boundary:** family child of REL-COV2-01（2/20）；用户已授权逐卡 reviewed signed+DCO 直推 main，但不写 R、不 tag、不调用 release。

**C/D coverage from:** COV2-REL-01；发布点的 exact-source C/D 绿后才能 `done/complete`。

**Granularity:** `type=implementation; axis=冻结 v1/RG-04 合同校验; recovery=forward-only; complete=yes; self-contained=yes; AC=8/8; VER=4/8; landing=1; prod-files=1; scope=M; deps=COV2-01; writeset=串行于前驱; release=family child; split-from=COV2-01; exception=N/A`

### Task COV2-02：版本化 DB claim 仲裁

**Task type:** implementation

**Lifecycle / Acceptance:** in-progress / locally-accepted（8 AC、3 VER、fmt、严格 Clippy、双 env 全量与唯一 Claude Code PASS；signed+DCO main `d4f159fd17676c7b591c1f80858f2de17094260f` 已交付，家族 C/D 仍待）

**Description:** versioned DB claim arbitration。本卡唯一行为轴、恢复和粒度见下方 Granularity；不得以 synthetic/native 互换验收。

**Out of scope:** 不负责 identity MAC/私有 map（COV2-07/14）或 marker（COV2-03）；不新增 SQL DDL。

**Current evidence:** src/internal/ai/coverage_gate.rs:160,199,422（基线 `dcc468b`；ER-02 开工刷新行号与实际源码）。

**Acceptance criteria:**

- [x] AC1 same `(session,key)` may have v1 and v2 rows
- [x] AC2 equivalent same-version duplicate is idempotent
- [x] AC3 same-version incomplete→complete advances revision
- [x] AC4 conflicting complete digest records conflict without overwrite
- [x] AC5 a production v1 claim contains no typed Reasoning
- [x] AC6 frozen `ReservedTurnClaim` and `LiveClaimCommitPlan` literals still compile while a separate per-claim `(coverage_schema_version,logical_turn_key)` carrier remains available to COV2-03/04/08
- [x] AC7 unsupported version rejects before reservation
- [x] AC8 injected DB error yields no visible claim success.

**Verification:**

- [x] VER1→AC1/2/3/4: `source .env.test && cargo test --lib internal::ai::coverage_gate::tests::cov2_claim_arbitration_by_version -- --exact` (root 1/1, 0 skipped; v1/v2 同键独立行、幂等、revision 和冲突).
- [x] VER2→AC6: `source .env.test && cargo test --test agent_coverage_gate_test cov2_v1_claim_and_legacy_literals_stay_valid -- --exact` (root 1/1, 0 skipped; pure struct-literal/sidecar contract；无 provider Session，含同键 v1/v2 collection).
- [x] VER3→AC5/7/8: `source .env.test && cargo test --lib internal::ai::coverage_gate::tests::cov2_invalid_version_or_db_error_zero_success -- --exact` (root 1/1, 0 skipped；旧生产 v1 入口拒 typed Reasoning，版本错误预写拒绝，第二条 INSERT fault 后整批 0 行).

**Final A/B local evidence:** 三项具名 VER 在根工作区最终源码各 1/1、0 skipped；`cargo +nightly fmt --all --check` exit 0；`cargo clippy --all-targets --all-features -- -D warnings` exit 0；`libra diff --check` exit 0。双 env `CARGO_BUILD_JOBS=2 cargo nextest run --all --no-fail-fast --retries 2` exit 0，**10059/10059 passed、0 failed、23 skipped**（14 slow；raw SHA-256 `afbadf5ca0b5ce4b3265d6810ff022efd054d9d7344f7f2c3737162eed5a6d1e`）。首次全量编译未完成即被终止，exit 143，不计 PASS；23 skipped 不算 OpenCode Session/macOS 真实模型验收。本卡五个非状态文件均为 mode `100644`，SHA-256：`coverage_gate.rs` `832646084a8a0b17d47f89cd10d1851c8a6d5c093bf9a266a6402745255a0267`、`agent_coverage_gate_test.rs` `e815afd756a32468cdbdd2bc5f674da471d18b29c0674042f2798a8b844ecb06`、`coverage-v2.md` `1289cb48664527868d1f3cf7411f101d404d1a213e6eb93d37537025e3bcf5ff`、`agent.md` `5aef28fbf4424827fe4723eabb9d78aea2095c345148731ff4a920e59706350b`、`COMPATIBILITY.md` `5021bcce9da0d450aa6a8ca97d50d4f118f5e8763c4c01f5bd4d147501004673`。唯一完成的 Claude Code Sonnet 源码审查 **PASS**（P0/P1/P2=0，raw SHA-256 `02db003513b2729c9b24fc82fe5a70bcabd0eaf9de11b065f83e5078ce88b244`）；首次 Fable 请求因用量上限 `is_error=true`，无评审结论，不计第二评审。Claude 审查时全量尚在运行，最终 exit 0/日志摘要由 root 后续核验。v2 Session/commit 接线属于后继 COV2-03/04/11；此卡不声明生产 v2 writer 或 Source A 已 GO。

**Full-suite trigger:** 本卡改动 shared `coverage_gate.rs`，命中 ER-13 T-1；已在最终源码的 A/B 阶段执行双 env 全量 Nextest 10059/10059。COV2-REL-01 T-2 仍须在 bumped 树上独立重跑，不能复用本卡结果。

**Dependencies:** COV2-13；消费前驱实际最终 A/B、适用 focused/full、fmt/strict、单 Claude PASS 与 signed+DCO main 的 source-qualified 文件 SHA/mode；不以其尚待 C/D 的 `locally-accepted` 冒称 complete。

**Deliverables:** 无独立文件；本卡实现/测试/文档随唯一发布点交付；每卡的计数、源码 SHA、hash/mode、review/门结果写在本计划正文，不建证据文件。

**Implementation write set:** I-core = `docs/development/plan/plan-20261013-coverage-schema.md`; `docs/development/plan/plan-status.md`; `docs/development/tracing/coverage-v2.md`; `docs/development/tracing/agent.md`; `COMPATIBILITY.md`；本卡新增精确路径 = `src/internal/ai/coverage_gate.rs`; `tests/agent_coverage_gate_test.rs`; `tests/INDEX.md`。无 glob 默示写权。

**Release write set:** N/A；C/D 继承 COV2-REL-01。

**Files likely touched:** `docs/development/plan/plan-20261013-coverage-schema.md`; `docs/development/plan/plan-status.md`; `docs/development/tracing/coverage-v2.md`; `docs/development/tracing/agent.md`; `COMPATIBILITY.md`; `src/internal/ai/coverage_gate.rs`; `tests/agent_coverage_gate_test.rs`; `tests/INDEX.md`。

**Docs and compatibility impact:** `docs/development/tracing/coverage-v2.md`; `docs/development/tracing/agent.md`; `COMPATIBILITY.md`；逐文件核内容与源码行为对应，`tests/INDEX.md` 只在本卡精确写集确有索引变化时同步。无新增网站命令行为；若实施引入公开差异，先按 ER-06a 修本卡写集再编辑。

**Rollback mode:** forward-only；恢复时签名+DCO 新 main 修复，既有 v1 claim/artifact/digest 不回写或重解释。

**Migration and rollback:** 无 DDL；Repository claim 仅用既有 versioned 键。若实现证明 DDL 必需，停止、加 GC-13 角色隔离迁移卡与旧/新读端门，再修本卡写集和 G-03/G-04。

**Security and privacy:** 继承 GC-COV2-02/03；错误或不可信版本必须 zero-success、不得向 ordinary/typed/log/error 导出候选内容；Session 例只在本机 ignored 执行。

**Performance budget:** 继承 GC-COV2-05；identity map 的 v1+v2 总量仍限 4096 条/1 MiB，任何新增解析在分配前有界；变更 hot path 时以具名门证明无无界扫描。

**Estimated scope:** M（落点/生产文件计数：8/8, 3/8, 1, 1; M）。

**Version increment:** N/A；此卡不发版。

**Release boundary:** family child of REL-COV2-01（3/20）；用户已授权逐卡 reviewed signed+DCO 直推 main，但不写 R、不 tag、不调用 release。

**C/D coverage from:** COV2-REL-01；发布点的 exact-source C/D 绿后才能 `done/complete`。

**Granularity:** `type=implementation; axis=版本化 DB claim 仲裁; recovery=forward-only; complete=yes; self-contained=yes; AC=8/8; VER=3/8; landing=1; prod-files=1; scope=M; deps=COV2-13; writeset=串行于前驱; release=family child; split-from=N/A; exception=N/A`

### Task COV2-07：密钥所有者 identity-v1/v2 MAC 域

**Task type:** implementation

**Lifecycle / Acceptance:** in-progress / locally-accepted（8 AC、4 VER、fmt、严格 Clippy、双 env 全量与唯一 Claude Code PASS；signed+DCO main `14fc516d94955fadb867c16868e7f12792c4b52e` 已交付，家族 C/D 仍待）

**Description:** key-owner v1/v2 identity MAC domains。本卡唯一行为轴、恢复和粒度见下方 Granularity；不得以 synthetic/native 互换验收。

**Out of scope:** 不写 dual-map 持久化（COV2-14），不铸造公开 claim；只建版本隔离密码学原语。

**Current evidence:** src/internal/ai/capture/key.rs:99-174,1705-1885; src/internal/ai/capture/checkpoint.rs:630-665（基线 `dcc468b`；ER-02 开工刷新行号与实际源码）。

**Acceptance criteria:**

- [x] AC1 the complete frozen identity-v1 canonical byte bundle equals its pinned golden
- [x] AC2 the complete frozen identity-v1 MAC known-answer bundle equals its pinned golden
- [x] AC3 identity-v2 Descriptor uses a distinct prefix and HMAC byte domain
- [x] AC4 identity-v2 Record does likewise
- [x] AC5 identity-v2 whole-map MAC uses its own distinct prefix and byte domain (not the v1 Record purpose)
- [x] AC6 cross-version or cross-purpose tag transplants fail verification
- [x] AC7 unknown identity/coverage version fails before a MAC can be accepted
- [x] AC8 authentication leaves repository-private key material unchanged.

**Verification:**

- [x] VER1→AC1/2: `source .env.test && cargo test --lib internal::metadata::tests::cov2_v1_identity_bytes_and_mac_frozen -- --exact`（root 1/1、0 skipped；六组规范字节 SHA-256 与四组固定 MAC）。
- [x] VER2→AC3/4/5: `source .env.test && cargo test --lib internal::ai::capture::key::tests::cov2_identity_v2_domains_are_disjoint -- --exact`（root 1/1、0 skipped；五个 purpose 的前缀及 HMAC 消息域互异）。
- [x] VER3→AC6: `source .env.test && cargo test --lib internal::ai::capture::key::tests::cov2_identity_cross_domain_transplant_rejects -- --exact`（root 1/1、0 skipped；逐对移植和 body 改写均拒绝）。
- [x] VER4→AC7/8: `source .env.test && cargo test --lib internal::ai::capture::key::tests::cov2_identity_unknown_version_and_missing_key_fail_closed -- --exact`（root 1/1、0 skipped；未知版本与缺 key 零成功，签验前后 key 原字节不变）。`key.rs` owns MAC prefixes/tags; `checkpoint.rs` owns purpose selection. Existing v1 APIs and bytes remain the default; this primitive-only card emits no public v2 claim.

**Final A/B local evidence:** 四项具名 VER 在根工作区最终源码各 1/1、0 skipped；固定合成密钥的独立 stdlib oracle 脚本 SHA-256 `465b022eac1d3036816c1308596957892c295630eb034d21389839439c71c30d` 实际重算六组完整 v1 规范字节摘要和四组 MAC，逐项与冻结断言一致，不接触仓库私钥。`cargo +nightly fmt --all --check` exit 0；`cargo clippy --all-targets --all-features -- -D warnings` exit 0；`libra diff --check` exit 0。双 env `CARGO_BUILD_JOBS=2 cargo nextest run --all --no-fail-fast --retries 2` exit 0，**10063/10063 passed、0 failed、23 skipped**（14 slow、1 leaky；raw SHA-256 `553bfa450a3efb7fecdf1907fc7e5bf4d14e51c8cfef40bd2d6f2a62754d05b8`）；四个新增用例均在该全量中再次 PASS。23 skipped 不算 OpenCode Session 或真实模型验收。本卡六个非状态文件均为 mode `100644`，SHA-256：`metadata.rs` `d92da0ff39bbd676561a6da3158230b33d556009864b91aed7892f069f9ce983`、`key.rs` `cfb505f061883e6331a80de350774658b541dad6158fb146d518d304c4667e7a`、`checkpoint.rs` `5e2609af811bb94f94047e80f3f8b9c69d646f70be013064f67be48985d9d789`、`coverage-v2.md` `c50cb282678f98ed85d9638f8972cc1fb644b2d0564a5a2e5a6a161330b0e203`、`agent.md` `1799c4d1ef191236750a5b02ee2704bc97133eba7143bf83aae47b05275b7a7f`、`COMPATIBILITY.md` `8ebe067ff16e9f4e390c821042571cd9e14cf73118bb3d55ae1e88e46fb84829`。唯一 Claude Code Sonnet 源码/文档审查 **PASS**（P0/P1/P2=0，raw SHA-256 `c275fa3872348db329daaa71591307daff4bf154483a4332687bb129b946bd9c`），审查者未运行测试，最终门由 root 核验。v2 record/map 封闭编码与共同容量属 COV2-14；此卡不持久化 v2 identity，不铸造 Source A 或 native 权限。

**Full-suite trigger:** 本卡修改共享 `src/internal/metadata.rs` 身份 codec，命中 ER-13 T-1；已在最终源码 A/B 阶段实际运行双 env Nextest 10063/10063。COV2-REL-01 T-2 仍须在 bumped 树上独立重跑。

**Dependencies:** COV2-02；消费前驱实际最终 A/B、适用 focused/full、fmt/strict、单 Claude PASS 与 signed+DCO main 的 source-qualified 文件 SHA/mode；不以其尚待 C/D 的 `locally-accepted` 冒称 complete。

**Deliverables:** 无独立文件；本卡实现/测试/文档随唯一发布点交付；每卡的计数、源码 SHA、hash/mode、review/门结果写在本计划正文，不建证据文件。

**Implementation write set:** I-core = `docs/development/plan/plan-20261013-coverage-schema.md`; `docs/development/plan/plan-status.md`; `docs/development/tracing/coverage-v2.md`; `docs/development/tracing/agent.md`; `COMPATIBILITY.md`；本卡新增精确路径 = `src/internal/metadata.rs`; `src/internal/ai/capture/key.rs`; `src/internal/ai/capture/checkpoint.rs`。无 glob 默示写权。

**Release write set:** N/A；C/D 继承 COV2-REL-01。

**Files likely touched:** `docs/development/plan/plan-20261013-coverage-schema.md`; `docs/development/plan/plan-status.md`; `docs/development/tracing/coverage-v2.md`; `docs/development/tracing/agent.md`; `COMPATIBILITY.md`; `src/internal/metadata.rs`; `src/internal/ai/capture/key.rs`; `src/internal/ai/capture/checkpoint.rs`。

**Docs and compatibility impact:** `docs/development/tracing/coverage-v2.md`; `docs/development/tracing/agent.md`; `COMPATIBILITY.md`；逐文件核内容与源码行为对应，`tests/INDEX.md` 只在本卡精确写集确有索引变化时同步。无新增网站命令行为；若实施引入公开差异，先按 ER-06a 修本卡写集再编辑。

**Rollback mode:** forward-only；恢复时签名+DCO 新 main 修复，既有 v1 claim/artifact/digest 不回写或重解释。

**Migration and rollback:** 无 DDL；Repository claim 仅用既有 versioned 键。若实现证明 DDL 必需，停止、加 GC-13 角色隔离迁移卡与旧/新读端门，再修本卡写集和 G-03/G-04。

**Security and privacy:** 继承 GC-COV2-02/03；错误或不可信版本必须 zero-success、不得向 ordinary/typed/log/error 导出候选内容；Session 例只在本机 ignored 执行。

**Performance budget:** 继承 GC-COV2-05；identity map 的 v1+v2 总量仍限 4096 条/1 MiB，任何新增解析在分配前有界；变更 hot path 时以具名门证明无无界扫描。

**Estimated scope:** M（落点/生产文件计数：8/8, 4/8, 2, 3; M）。

**Version increment:** N/A；此卡不发版。

**Release boundary:** family child of REL-COV2-01（4/20）；用户已授权逐卡 reviewed signed+DCO 直推 main，但不写 R、不 tag、不调用 release。

**C/D coverage from:** COV2-REL-01；发布点的 exact-source C/D 绿后才能 `done/complete`。

**Granularity:** `type=implementation; axis=密钥所有者 identity-v1/v2 MAC 域; recovery=forward-only; complete=yes; self-contained=yes; AC=8/8; VER=4/8; landing=2; prod-files=3; scope=M; deps=COV2-02; writeset=串行于前驱; release=family child; split-from=COV2-02; exception=N/A`

### Task COV2-14：双版本私有 identity map 与共同容量

**Task type:** implementation

**Lifecycle / Acceptance:** in-progress / locally-accepted（8 AC、7 VER、公开文档守卫与旧行为回归均已具名验收；fmt、严格 Clippy、双 env 全量与唯一 Claude Code PASS；本卡 signed+DCO main `4558af64c09e63c4716fa68595776a8044f66cc7` 已 Push，家族 C/D 仍待）

**Description:** dual-version private-map storage and aggregate bounds。本卡唯一行为轴、恢复和粒度见下方 Granularity；不得以 synthetic/native 互换验收。

**Out of scope:** 不改 v1 旧读端，不降级 v2；持久 marker 的版本事务由 COV2-03 承接。

**Current evidence:** src/internal/ai/coverage_gate.rs:307-362,2015-2105; src/internal/metadata.rs:50-51（基线 `dcc468b`；ER-02 开工刷新行号与实际源码）。

**Acceptance criteria:**

- [x] AC1 an existing v1-only map remains byte-identical under `identity-v1`
- [x] AC2 a v2 map uses a closed versioned codec
- [x] AC3 v1 and v2 maps for one Session can coexist for the same logical turn key
- [x] AC4 their combined entry count obeys one aggregate cap
- [x] AC5 their combined encoded-byte count obeys one aggregate cap
- [x] AC6 noncanonical private-map key sets reject rather than appearing legacy-absent
- [x] AC7 a rejected map leaves the visible claim state unchanged
- [x] AC8 identity-v2 is an additional key in the existing session-scoped `AgentReasoningIdentity` metadata namespace.

**Regression scope:** VER6/7 check that the additional key inherits the existing namespace's Session erasure, clean/GC and closed cloud-snapshot behavior. These are mandatory ER-04 regressions of AC8's scope placement, not new lifecycle or cloud semantics for this card; no separate product behavior is inferred from the synthetic signed fixture.

**Verification:**

- [x] VER1→AC1/2: `source .env.test && cargo test --lib internal::metadata::tests::cov2_v1_v2_private_map_codecs -- --exact` (1/1; frozen v1 raw bytes and explicit rejection of v2 context by the old v1 record codec).
- [x] VER2→AC3: `source .env.test && cargo test --lib internal::ai::coverage_gate::tests::cov2_dual_private_map_same_key_coexists -- --exact` (1/1).
- [x] VER3→AC4/5: `source .env.test && cargo test --lib internal::ai::coverage_gate::tests::cov2_private_map_aggregate_entry_and_byte_caps -- --exact` (1/1; include a real v1 claim companion transaction where an individually valid retained v2 map makes the new combined encoded size cross 1 MiB, then verify the claim/catalog/map and same-transaction ref probe roll back).
- [x] VER4→AC6: `source .env.test && cargo test --lib internal::metadata::tests::cov2_private_map_duplicate_and_unknown_reject -- --exact` (1/1).
- [x] VER5→AC7: `source .env.test && cargo test --lib internal::ai::coverage_gate::tests::cov2_private_map_failure_zero_claim -- --exact` (1/1; include non-text value, non-text and overlong declared `value_type`, oversized value, wrong MAC, unknown key, injected claim-write SQL failure and private-map SQL read failure). `coverage_gate.rs:315-362,2015-2105` currently assumes one `identity-v1` row and must read/authenticate only the two known keys with correct version-bound authentication; the existing v1 writer preserves retained v2 bytes and checks the aggregate bound before claim/catalog mutation. COV2-03 owns the versioned marker/ref fence; COV2-19 owns the sealed-native identity-v2 production companion. Synthetic signed v2 map seeding here proves only the carrier. Map decode remains bounded before allocation, including the declared type. A frozen old reader must retain exact v1-only behavior; when both keys exist it may fail closed because its current `LIMIT 2` reader rejects multiple rows (`coverage_gate.rs:315-335`). COV2-14 does not promise mixed-session compatibility with old binaries or silently downgrade v2. If dual-map commit needs another producer path, amend this precise I set before editing, not COV2-07's crypto card.

- [x] VER6→AC8: `source .env.test && cargo test --test agent_coverage_gate_test cov2_identity_v2_lifecycle_erasure_and_gc_exclusion -- --exact` (1/1; create both private keys and assert byte-identical retention through `agent clean` and two GC passes, then both deleted by Session erasure). ER-02 先只读核 `src/internal/ai/history.rs:4906-4917` 按 scope+target 删除、`src/command/agent/clean.rs:334` 委托 history prune、`src/command/maintenance.rs:3587-3591` 按 scope 排除；这些路径本卡 **verification-only，不在 I**。如实际测试证明需要修改其中任一路径，先新建精确写集的 FIX 卡并通过 G 审计、前滚后重跑，不在 COV2-14 默示扩权。
- [x] VER7→AC8: `source .env.test && cargo test --lib command::cloud::agent_capture::tests::private_capture_scopes_do_not_enter_closed_snapshot -- --exact` (1/1; extend the existing before/after closed catalog wire test with both private keys and distinct canaries under a catalog Session; no D1/R2 or model invocation). This proves the local cloud snapshot excludes identity-v1/v2; live D1/R2 transport remains a separate release gate.

**Final A/B local evidence:** 八项 AC、七项具名 VER 在最终行为源码上各 1/1、0 skipped；旧 partial/无效 map 回归 `reasoning_legacy_partial_and_invalid_identity_error_preserve_no_writes` 1/1，公开命令文档守卫 `reasoning_identity_reservation_errors_and_session_limits_are_documented` 1/1。`cargo +nightly fmt --all --check`、`CARGO_BUILD_JOBS=2 cargo clippy --all-targets --all-features -- -D warnings`、`libra diff --check` 均 exit 0。双 env `CARGO_BUILD_JOBS=2 cargo nextest run --all --no-fail-fast --retries 2` 最终 exit 0，**10069/10069 passed、0 failed、23 skipped、18 slow**，raw SHA-256 `2849dcf7e2105ed1da5e5d3b0333374e2590ac909c1f478ef8ba68966e86aec4`；七 VER 与公开文档守卫在本次全量再次 PASS。前两次全量 exit 100、各 10068 passed/1 failed/23 skipped，分别发现旧回归仍期待无效 map partial（raw SHA `181c3f5769df879680c1464af5e0b223195b1a806a372c6bbee0aece9ab9bc32`）及既有文档守卫仍钉旧措辞（raw SHA `00b651c9ddf653c2761aabe8a4aaad9832d6a6362f42249a12a1e374147ed652`）；均已修复且不计 PASS。最终唯一 Claude Code Sonnet 审查 **PASS**（P0/P1/P2=0，raw SHA-256 `2d1e5fb7425abc9aa2802db241213811dfcd6c891410e9853e554c5fcb806966`）；早轮公开文档 P2 FAIL（raw `b1d3235da90b481c71b309592c441f303dd4cfaddb5abf029feb0448114e4285`）已修正。十个非状态文件均 mode `100644`，SHA-256：`COMPATIBILITY.md` `aa2842d67b37911c5e1ad5a6fa4cda961c84d27208021798ffa1eb47ab7063f8`；`docs/commands/agent.md` `ca6ce7251c7ac0eed749e0a02836739522519155d0fc29a32499ec80861c31a5`；`docs/commands/zh-CN/agent.md` `b823b92ec6ed4e6800e90f0b179b1dafa401ea2f6ab8715148bf734a12cabd4f`；`docs/development/tracing/agent.md` `f40b14d3998a09a352c600d1bd2e99611388eb906b5e351f3616f16d9c3d8006`；`docs/development/tracing/coverage-v2.md` `4bbaab29a17b8c709f775fbc428064490683e7364234a5b2ce4b0bc7c8369d79`；`src/command/cloud/agent_capture.rs` `8cb0180b31bc2feed1ef74e315591995d3f5b77f98ac529ec458cc9aefca9413`；`src/internal/ai/coverage_gate.rs` `69fbf2a1c89e49b7fa4cb0ff4f43d6c3bae859b5b8a5b7281b2910cff4f0a15e`；`src/internal/metadata.rs` `5646a4bd1b7559f7b8feadc3ddc09e6a9e8ebc2611b515cc0a19ce191e0c738a`；`tests/agent_coverage_gate_test.rs` `b1edacc56d33a9943ae3fa05c8a538a8801852c023abc9846be47d663165e22c`；`tests/compat/agent_docs_contract.rs` `063c6d0bab59375cae6e2ca736018bb2dc3359af2836fecdacb49b6e0add848a`。COV2-14 只用合成签名 v2 map 证明双载体/容量与现有生命周期，不声明生产 v2 writer、真实 native 来源 A 或 Session 模型验收通过；23 skipped 亦不计这些门。

**Signed source handoff:** signed+DCO main `4558af64c09e63c4716fa68595776a8044f66cc7` 已直接 Push；本卡最终 full 10069/10069 exit 0（raw SHA-256 `2849dcf7e2105ed1da5e5d3b0333374e2590ac909c1f478ef8ba68966e86aec4`）、唯一 Claude Code PASS（raw SHA-256 `2d1e5fb7425abc9aa2802db241213811dfcd6c891410e9853e554c5fcb806966`）已在上方验收正文核实；继承 COV2-REL-01 C/D 仍待，故保持 `in-progress / locally-accepted`。

**Full-suite trigger:** family child 不自行发布；按 ER-13 对最终改动重新判定 T-1/T-4/T-6。若命中 shared single-source helper、测试基建或 focused 信号不足，则在本卡 A/B 阶段运行双 env 全量 Nextest；无论是否命中，COV2-REL-01 T-2 均须在 bumped 树上重跑。

**Dependencies:** COV2-07；消费前驱实际最终 A/B、适用 focused/full、fmt/strict、单 Claude PASS 与 signed+DCO main 的 source-qualified 文件 SHA/mode；不以其尚待 C/D 的 `locally-accepted` 冒称 complete。

**Deliverables:** 无独立文件；本卡实现/测试/文档随唯一发布点交付；每卡的计数、源码 SHA、hash/mode、review/门结果写在本计划正文，不建证据文件。

**Implementation write set:** I-core = `docs/development/plan/plan-20261013-coverage-schema.md`; `docs/development/plan/plan-status.md`; `docs/development/tracing/coverage-v2.md`; `docs/development/tracing/agent.md`; `COMPATIBILITY.md`；本卡新增精确路径 = `src/internal/ai/coverage_gate.rs`; `src/internal/metadata.rs`; `src/command/cloud/agent_capture.rs`（只扩现有 closed-snapshot 单元测试）；`tests/agent_coverage_gate_test.rs`。Claude 评审发现公开命令文档仍承诺无效 map 逐 turn partial；按 ER-06a 在修改前将 `docs/commands/agent.md` 与 `docs/commands/zh-CN/agent.md` 加入本卡精确 I，仅修正该行为契约。最终全量发现既有 `compat_agent_docs_contract` 仍断言旧 partial 文案；在修改前再将 `tests/compat/agent_docs_contract.rs` 加入本卡精确 I，仅更新该公开文档守卫并具名重验 1/1。这三处随附同步不扩大生产写集或 G-04 计数；文档守卫属 ER-06a 强制门，不计本卡 G-03 特有 VER。无 glob 默示写权。

**Release write set:** N/A；C/D 继承 COV2-REL-01。

**Files likely touched:** `docs/development/plan/plan-20261013-coverage-schema.md`; `docs/development/plan/plan-status.md`; `docs/development/tracing/coverage-v2.md`; `docs/development/tracing/agent.md`; `docs/commands/agent.md`; `docs/commands/zh-CN/agent.md`; `COMPATIBILITY.md`; `src/internal/ai/coverage_gate.rs`; `src/internal/metadata.rs`; `src/command/cloud/agent_capture.rs`; `tests/agent_coverage_gate_test.rs`; `tests/compat/agent_docs_contract.rs`。

**Docs and compatibility impact:** `docs/development/tracing/coverage-v2.md`; `docs/development/tracing/agent.md`; `docs/commands/agent.md`; `docs/commands/zh-CN/agent.md`; `COMPATIBILITY.md`; `tests/compat/agent_docs_contract.rs`（明确旧 binary 在双 map Session 上 fail closed、可能连 v1 turn 也不可读，须升级后恢复；无效 map 从逐 turn partial 改为整批 hard reject，import receipt 因而记录 failed；相应公开文档断言同步；不承诺透明降级）；逐文件核内容与源码行为对应，`tests/INDEX.md` 只在本卡精确写集确有索引变化时同步。

**Rollback mode:** forward-only；恢复时签名+DCO 新 main 修复，既有 v1 claim/artifact/digest 不回写或重解释。

**Migration and rollback:** 无 DDL；Repository claim 仅用既有 versioned 键。若实现证明 DDL 必需，停止、加 GC-13 角色隔离迁移卡与旧/新读端门，再修本卡写集和 G-03/G-04。

**Security and privacy:** 继承 GC-COV2-02/03；错误或不可信版本必须 zero-success、不得向 ordinary/typed/log/error 导出候选内容；Session 例只在本机 ignored 执行。

**Performance budget:** 继承 GC-COV2-05；identity map 的 v1+v2 总量仍限 4096 条/1 MiB，任何新增解析在分配前有界；变更 hot path 时以具名门证明无无界扫描。

**Estimated scope:** M（落点/生产文件计数：8/8, 7/8, 2, 2; M；`src/command/cloud/agent_capture.rs` 仅扩 `#[cfg(test)]`，不计行为生产文件）。

**Version increment:** N/A；此卡不发版。

**Release boundary:** family child of REL-COV2-01（5/20）；用户已授权逐卡 reviewed signed+DCO 直推 main，但不写 R、不 tag、不调用 release。

**C/D coverage from:** COV2-REL-01；发布点的 exact-source C/D 绿后才能 `done/complete`。

**Granularity:** `type=implementation; axis=双版本私有 identity map 与共同容量; recovery=forward-only; complete=yes; self-contained=yes; AC=8/8; VER=7/8; landing=2; prod-files=2; scope=M; deps=COV2-07; writeset=串行于前驱; release=family child; split-from=COV2-07; exception=N/A`

### Task COV2-03：版本化 marker 与事务围栏

**Task type:** implementation

**Lifecycle / Acceptance:** in-progress / locally-accepted（AC1–8 的五项 VER、五项旧 v1 mandatory ER-04 与两项缺表回归均逐项 1/1；最终双 env full **Summary [1618.814s] 10078 tests run: 10078 passed (14 slow), 23 skipped**/raw SHA-256 `c49dd3c33652d55541a0e4c0269c817460b9109cf261eab5271d90760a49e8a9`、exit0；唯一最终 Claude Code PASS，P0/P1/P2=0、P3=5/raw SHA-256 `9ef25f92615eefcf2019212a306cfa7542ef416b693286bb99111006b3ea6df7`；signed+DCO 源码/文档 main `d717e42e0d1ec75e6ae576b21242fd1519040abd` 已 Push 且 remote 同 SHA；COV2 source-ready 6/23=26.09%。家族 C/D 仍待；0 matched、只有 helper 或 skip 均不算 PASS）

**Description:** versioned marker and transactional fence。本卡唯一行为轴、恢复和粒度见下方 Granularity；不得以 synthetic/native 互换验收。

**Out of scope:** 不改 SQL schema；本卡不签发 native 来源、不写 identity-v2、不负责 pending verify/retain 或 wire/version。由本卡拆出 COV2-19（封闭 native identity-v2 companion）、COV2-20（只读版本化 reservation verify）与 COV2-21（同事务 retain）；COV2-04 只做有界 authenticated codec/seal，COV2-22 负责真实 pending persist/冻结旧读端，COV2-08 接恢复。`src/internal/ai/capture/catalog.rs` 对本卡仅只读；completed receipt 的精确 revision-set 比较必须落在本卡 I 内的 `traces.rs` 或 `capture/checkpoint.rs`。若最终行为确需修改 catalog，先按 ER-06a 修订精确 I、粒度与单写者表并重审，不隐式扩权。

**Current evidence:** `src/internal/ai/traces.rs:1192–1219,1479–1550`（现有 marker/terminal receipt 注册）；`src/internal/ai/coverage_gate.rs:167–203,2320–2367,2563–2578`（版本 sidecar 与 claim companion）；`src/internal/ai/capture/checkpoint.rs:576–602,3197–3215`（现有 plan/registration）；`src/internal/ai/capture/live_checkpoint.rs:2040–2062`（live claim 与 marker fences）；`src/internal/ai/capture/recovery.rs:287–334`（recovery fences）；`src/internal/ai/capture/catalog.rs:5735–5744`（completed receipt 判定，本卡只读）。这些是 G-09 signed main 的历史 ER-02 定位锚点，部分行号已过时，不是当前源码定位或新 VER 的通过证据；本卡最终八个非状态路径 SHA/mode 见下方清单，COV2-19 开工须重新刷新行号。r12 五文件候选加 owned-key 生命周期修复、注册竞态分类修复、同 ID 修复后重试、纯 v1 超额兼容及旧库缺表与迁移收据判据，实现 versioned marker/ref 围栏和精确 committed revision-set replay；当前五项 VER、五项 ER-04 与两项缺表回归均各 1/1。最终双 env full **Summary [1618.814s] 10078 tests run: 10078 passed (14 slow), 23 skipped**/raw SHA-256 `c49dd3c33652d55541a0e4c0269c817460b9109cf261eab5271d90760a49e8a9`、exit0；唯一最终十文件 Claude Code PASS、P0/P1/P2=0、P3=5/raw SHA-256 `9ef25f92615eefcf2019212a306cfa7542ef416b693286bb99111006b3ea6df7`；signed+DCO 源码/文档 main `d717e42e0d1ec75e6ae576b21242fd1519040abd` 已 Push 且 remote 同 SHA；这些测试只证明当前源码，不授予 native Source A。

**Acceptance criteria:**

- [x] AC1 marker SQL binds every claim's actual `(version,key,digest,completeness,owner,fence,previous_revision,state)` from one immutable sidecar.
- [x] AC2 checkpoint constructor retains every claim's complete versioned fence tuple.
- [x] AC3 live persisted-generation constructor retains that tuple.
- [x] AC4 recovery constructor retains that tuple; production replay routing remains later work.
- [x] AC5 committed-checkpoint replay compares the exact immutable revision set before acknowledging `AlreadyExists`/`AlreadyCommitted`.
- [x] AC6 any marker fence mismatch rejects before marker success.
- [x] AC7 a second-claim fault leaves the actual traces ref, catalog, revisions, claims and object index at their pre-transaction state. A clean retry and early replay are evidence for AC5/7, not separate relaxed success paths.
- [x] AC8 marker JSON `schema_version=3` is independent of the claim's coverage version.

**Verification:**

- [x] VER1→AC1/5/6: `source .env.test && cargo test --lib internal::ai::traces::tests::cov2_marker_fence_binds_version -- --exact` (1/1; wrong version/tuple, exact committed revision-set replay including nonempty committed set versus empty expectation, zero ACK/write on mismatch).
- [x] VER2→AC2/3/4: `source .env.test && cargo test --lib internal::ai::capture::checkpoint::tests::cov2_all_marker_constructors_keep_version -- --exact` (1/1; complete sidecar and same key across versions).
- [x] VER3→AC5/7: `source .env.test && cargo test --lib internal::ai::coverage_gate::tests::cov2_owner_and_companion_fault_atomicity -- --exact` (1/1; actual `TracesCheckpointStore::write`/HistoryManager fault, rollback, retry and early replay; ordinary/visible typed source only, no native map assertion).
- [x] VER4→AC8 + mandatory ER-04 v1 freeze: `source .env.test && cargo test --lib internal::ai::traces::tests::cov2_v1_marker_golden -- --exact` (1/1; marker JSON schema_version independence plus unchanged frozen v1 historical bytes from pre-change oracle and old empty-fence lookup). The v1 byte freeze remains a hard gate outside the eight *new* AC, not a waived requirement.
- [x] VER5→AC5: `source .env.test && cargo test --lib internal::ai::traces::tests::cov2_completed_terminal_receipt_runtime_branch -- --exact` (1/1; independent `#[tokio::test]` must actually select and run the completed catalog receipt → traces registration → checkpoint outcome path; exact v2 revision set may ACK, missing/extra/mismatched revision or absent v2 checkpoint must fail before ACK/write; G-09 新增门，当前本机源码已精确 1/1；必须存在独立 `#[tokio::test]` 并实际精确选中 1/1。只有 async helper、0 matched 或 skipped 均不算 PASS；同一用例须证明失败后预注册 marker 仍可恢复、零假 ACK).

**Mandatory ER-04 old v1 regressions（outside G-03 specific VER count）:**

- [x] 旧 catalog terminal receipt：`source .env.test && cargo test --lib internal::ai::capture::catalog::tests::completed_terminal_receipt_acknowledges_stale_marker_without_checkpoint_row -- --exact`（1/1；保留无 action checkpoint row 的 v1 空围栏 ACK）。
- [x] 旧 v1 有 revision 的 live no-op：`source .env.test && cargo test --lib internal::ai::capture::live_checkpoint::tests::legacy_v1_covered_noop_settles_nonempty_prior_revision -- --exact`（1/1）。
- [x] 旧 v1 有 revision 的 recovery store settlement：`source .env.test && cargo test --lib internal::ai::capture::recovery::tests::legacy_v1_recovery_settles_nonempty_prior_revision -- --exact`（1/1）。
- [x] 真实 checkpoint 的无写入结算/空 authoritative set 区分：`source .env.test && cargo test --lib internal::ai::capture::checkpoint::tests::committed_replay_without_write_retires_only_this_actions_ordinary_marker -- --exact`（1/1）。
- [x] 历史纯 v1 超过 4096 条仍可重放：`source .env.test && cargo test --lib internal::ai::traces::tests::cov2_large_pure_v1_checkpoint_replay_preserves_legacy_claim_count -- --exact`（1/1；真实注册对 4097 条返回 `AlreadyCommitted`，第 4098 条额外 revision 拒绝，含 v2 的混合集仍守 4096 上限）。

五项均在当前本机源码分别精确选中并执行，实际各 1/1；0 matched、skip 或仅 helper 不计通过。

**当前本机 A/B 证据：** AC1–8 由五项具名 VER 各 1/1 覆盖；五项 mandatory ER-04 旧行为回归和两项旧库缺表正反例各 1/1；十二条当前源码精确日志索引 SHA-256 `7df3c3c43e8ee41caa7138d8ec7bee0c660938473fe67d6229446ad24700cb76`。旧纯 v1 超额场景源于单 Claude 预审 P3 兼容提示，已补生产限额区分与真实注册回归；原单审 PASS/raw `9baf792d0ecbc88fae9bea7567d8d06a1580ba723afdeb5d9875b9fa5fb1c946` 只对应修复前字节，不追认最终评审。首轮 Cargo 编译 E0597 失败已由 owned-key 修复，失败不计 PASS。ER-13 T-1 双 env full-r1 在测试发现阶段因预审修正主动终止（exit -15，raw SHA-256 `f3e5fef2e055ef7571fe3b1718c065343c566ddd78ad8299fdea8aed17b40246`）；full-r2 在测试发现阶段因本次 v1 兼容修正主动终止（exit 143，raw SHA-256 `3f35371f8fed9edc13c86844c91c2cb4f5c1d8a1fc19c2dd6b835f4cb43ad599`），均不计 PASS。full-r3 运行至 4588/10077 时因旧 committed checkpoint 无 coverage 表失败，实际 4587 passed、1 failed、23 skipped、5489 未执行、exit100/raw SHA-256 `cfb4b2ebf34c04e3a1c5e5b7dbbb7bb957f4922b32f93007e0c800f1115f2df3`；已修并具名重验，原失败不计 PASS。最终源码 fmt/strict Clippy exit0；单 Claude Code r4 预全量审查 PASS、P0/P1/P2=0、P3=3（raw SHA-256 `db9c944a5d71d8d43f6de99a6d0ba24dfbecbbbba8cb784286098774ea5cb694`），其意见登记为版本常量维护、极端双表与 ledger 丢失和 terminal 查询冗余，不把预审追认为最终字节；最终双 env full **Summary [1618.814s] 10078 tests run: 10078 passed (14 slow), 23 skipped**、exit0、raw SHA-256 `c49dd3c33652d55541a0e4c0269c817460b9109cf261eab5271d90760a49e8a9`；唯一最终 Claude Code PASS，P0/P1/P2=0、P3=5/raw SHA-256 `9ef25f92615eefcf2019212a306cfa7542ef416b693286bb99111006b3ea6df7`；signed+DCO 源码/文档 main `d717e42e0d1ec75e6ae576b21242fd1519040abd` 已 Push、remote 同 SHA。23 skipped 不替代本机 OpenCode Session 门；helper、synthetic 或 CI 状态不能替代未运行门。

**当前八个非状态路径 SHA-256/mode（审查前逐项重核）：** `src/internal/ai/traces.rs` `49b86e0e2f86ec275880439e9c5e93ac934ee54a8a9e4b30bedebecba2d97d5b`/`100644`；`src/internal/ai/coverage_gate.rs` `19770a4cec752945522ff6742dc4648dc5d597a8e37b349799c9f1015ec138d2`/`100644`；`src/internal/ai/capture/checkpoint.rs` `33988510731611fe42b740537a59f300ba3761756f9b17bfd449f83e660b6421`/`100644`；`src/internal/ai/capture/live_checkpoint.rs` `da06fee5c96695385edab7d6fac4f3100883c06b5f0d4a930fa7b26325479b36`/`100644`；`src/internal/ai/capture/recovery.rs` `74f3c53971e4f5b4a46369619156c631e44cba57851bccab5752e94e06e704a7`/`100644`；`docs/development/tracing/coverage-v2.md` `0ce8e1123b95a3231081536d9e17b82854944d35b94efc46b51c386af89a7ab0`/`100644`；`docs/development/tracing/agent.md` `5facd2b457a864dbd4e76f523a15bc960d700053f58fa1733ae62cb102116cb7`/`100644`；`COMPATIBILITY.md` `c18f97ad5b40b3da8102331d9d27902c9f58293c189d9ac15171af00db48b523`/`100644`。

**最终 Claude Code 五项非阻断 P3 与承接：** （1）无 authoritative expected set 的 no-write v2 已按设计 fail closed；COV2-08/11 启用 v2 前须提供权威围栏，不将当前拒绝当 v2 正例。（2）`traces.rs` 的 `2026071301` 迁移号为维护风险，后续若改迁移 registry 应加常量或对应 guard；本卡不改 SQL。（3）极端的 coverage 双表及 `schema_versions` 同时丢失可被判为旧 v1 空集合，风险已在公开文档披露，后续恢复/doctor 卡继续审计。（4）审查指出本卡 ER-02 行号锚点陈旧；本次状态收据已更正其历史性质，COV2-19 开工刷新当前行号。（5）`with_versioned_claims` 当前尚无生产调用，COV2-11 接入后移除 `allow(dead_code)`。五项均为评审 P3，未据此宣称 COV2-19..22、OpenCode Session、native Source A、家族 C/D 已完成。

**Full-suite trigger:** family child 不自行发布；按 ER-13 对最终改动重新判定 T-1/T-4/T-6。若命中 shared single-source helper、测试基建或 focused 信号不足，则在本卡 A/B 阶段运行双 env 全量 Nextest；无论是否命中，COV2-REL-01 T-2 均须在 bumped 树上重跑。旧隔离草案不替代本卡已验收的最终源码；`verify_checkpoint_claim_set_with_conn` 不得以空 expected fence 提前返回而漏掉非空 committed set，`TerminalReceiptAlreadyComplete` 不得未经精确集合比较给成功 ACK；最终源码须按 VER1/VER5 分别真实实测或 fail closed，另跑旧 v1 回执 ER-04 回归，保留 v1 冻结合约。

**Dependencies:** COV2-14。ER-02 开工前置条件已满足：本计划、plan-status 与父 plan-20260902 的 G-09 三文件协调经最终字节单 Claude PASS（raw SHA `77d4820e329ade99f5fba7b43821d072d4d7068ee5ba639ada4704851686f4fd`），独立 signed+DCO main `eaa43ec13f6c50212f0b3c2fa8a51799ec74bff5` 已 Push。这是可核的执行前置条件，不是另一个未登记的 COV2 任务 ID。消费 COV2-14 的最终 A/B、适用 focused/full、fmt/strict、单 Claude PASS、signed main 与逐路径 SHA/mode；不以其继承 C/D 的 `locally-accepted` 冒称 complete。

**Deliverables:** 无独立文件；本卡实现/测试/文档随唯一发布点交付；每卡的计数、源码 SHA、hash/mode、review/门结果写在本计划正文，不建证据文件。

**Implementation write set:** I-core = `docs/development/plan/plan-20261013-coverage-schema.md`; `docs/development/plan/plan-status.md`; `docs/development/tracing/coverage-v2.md`; `docs/development/tracing/agent.md`; `COMPATIBILITY.md`；本卡新增精确路径 = `src/internal/ai/traces.rs`; `src/internal/ai/coverage_gate.rs`; `src/internal/ai/capture/checkpoint.rs`; `src/internal/ai/capture/live_checkpoint.rs`; `src/internal/ai/capture/recovery.rs`。无 glob 默示写权。

**Release write set:** N/A；C/D 继承 COV2-REL-01。

**Files likely touched:** `docs/development/plan/plan-20261013-coverage-schema.md`; `docs/development/plan/plan-status.md`; `docs/development/tracing/coverage-v2.md`; `docs/development/tracing/agent.md`; `COMPATIBILITY.md`; `src/internal/ai/traces.rs`; `src/internal/ai/coverage_gate.rs`; `src/internal/ai/capture/checkpoint.rs`; `src/internal/ai/capture/live_checkpoint.rs`; `src/internal/ai/capture/recovery.rs`。

**Docs and compatibility impact:** `docs/development/tracing/coverage-v2.md`; `docs/development/tracing/agent.md`; `COMPATIBILITY.md`；逐文件核内容与源码行为对应，`tests/INDEX.md` 只在本卡精确写集确有索引变化时同步。无新增网站命令行为；若实施引入公开差异，先按 ER-06a 修本卡写集再编辑。

**Rollback mode:** forward-only；恢复时签名+DCO 新 main 修复，既有 v1 claim/artifact/digest 不回写或重解释。

**Migration and rollback:** 无 DDL；Repository claim 仅用既有 versioned 键。若实现证明 DDL 必需，停止、加 GC-13 角色隔离迁移卡与旧/新读端门，再修本卡写集和 G-03/G-04。

**Security and privacy:** 继承 GC-COV2-02/03；错误或不可信版本必须 zero-success、不得向 ordinary/typed/log/error 导出候选内容；Session 例只在本机 ignored 执行。

**Performance budget:** 继承 GC-COV2-05；identity map 的 v1+v2 总量仍限 4096 条/1 MiB，任何新增解析在分配前有界；变更 hot path 时以具名门证明无无界扫描。

**Estimated scope:** M（落点/生产文件计数：8/8, 5/8, 3, 5; M）。

**Version increment:** N/A；此卡不发版。

**Release boundary:** family child of REL-COV2-01（6/20）；用户已授权逐卡 reviewed signed+DCO 直推 main，但不写 R、不 tag、不调用 release。

**C/D coverage from:** COV2-REL-01；发布点的 exact-source C/D 绿后才能 `done/complete`。

**Granularity:** `type=implementation; axis=版本化 marker 与事务围栏; recovery=forward-only; complete=yes; self-contained=yes; AC=8/8; VER=5/8; landing=3; prod-files=5; scope=M; deps=COV2-14; writeset=串行于前驱; release=family child; split-from=N/A; exception=N/A`

### Task COV2-04：有界认证 pending-v2 codec 与内存 seal

**Task type:** implementation

**Lifecycle / Acceptance:** pending / 空（新增卡；下列 AC/VER 尚未执行）

**Description:** 在 `pending_payload.rs` 与 `pending.rs` 中构造封闭 pending-v2 wire 和内存 sealed versioned artifact；从原 COV2-04 拆出 COV2-22 的真实持久化/冻结旧读端行为，由后者消费本卡 seal。该卡只证明 codec/MAC/边界；v1 历史字节仍由原始基线固定。

**Out of scope:** 不执行数据库 persist、retain、compiled frozen old reader 或 replay；无手工 seeded artifact 行。生产 v2 artifact 至少含一个 v2 claim 的策略归 COV2-22 AC3。

**Current evidence:** `src/internal/ai/capture/pending_payload.rs`、`pending.rs` 与冻结 `tests/fixtures/agent_transcripts/reasoning/pending_v1_decoder.rs`（只读）。 开工按 ER-02 在最终 signed main 刷新行号、SHA/mode；先前隔离草案未编译/验收，不作实现证据。

**Acceptance criteria:**

- [ ] AC1：每个 v2 claim 显式携带 coverage schema version。
- [ ] AC2：authenticated decode 保留完整有序 sealed tuple。
- [ ] AC3：同一 logical key 可以各出现一次 v1 和 v2。
- [ ] AC4：同版本重复 `(version,key)` 拒绝。
- [ ] AC5：无效或未知 version 拒绝。
- [ ] AC6：MAC/authentication 失败在 candidate projection 前拒绝。
- [ ] AC7：entry count 在分配前有界。
- [ ] AC8：编码 wire bytes 长度超限在分配前拒绝。

**独立谓词计数 / 兼容：** entry 与 wire-byte cap 是两项独立新行为；encoder 亦须服从 GC-COV2-05 输出上限。v1 pending canonical bytes/MAC 取变更前冻结 oracle，不由新 serializer 重生；属于硬性 ER-04 兼容门，非豁免。此卡文档必须说明 v2 artifact 尚未 durable persist。

**Verification:**

- [ ] VER1：额外 mandatory ER-04 冻结门：`source .env.test && cargo test --lib internal::ai::capture::pending::tests::cov2_v1_pending_bytes_and_mac_frozen -- --exact`（原始基线 canonical bytes/MAC，1/1）。
- [ ] VER2：AC1–3：`source .env.test && cargo test --lib internal::ai::capture::pending_payload::tests::cov2_cross_version_same_key_authenticated_codec_roundtrip -- --exact`（内存 seal/MAC，1/1）。
- [ ] VER3：AC4/5：`source .env.test && cargo test --lib internal::ai::capture::pending_payload::tests::cov2_same_version_duplicate_and_unknown_reject -- --exact`（1/1）。
- [ ] VER4：AC6–8：`source .env.test && cargo test --lib internal::ai::capture::pending_payload::tests::cov2_pending_v2_mac_and_bounds_fail_closed -- --exact`（entry、wire byte cap 分别断言；1/1）。

**旧端约束：** 冻结 `pending_v1_decoder.rs` 不编辑；本卡不从 codec roundtrip 推断 compiled old reader 可拒绝真实 artifact，该正例在 COV2-22。

**Full-suite trigger:** 按 ER-13 以最终 diff 判定；先跑本卡 4 个具名 focused selector、相关冻结 v1 回归、nightly fmt、strict Clippy 与唯一 Claude Code 最终字节 PASS；触发 shared helper/测试基建/无法归属 focused 故障时本卡双 env full。COV2-REL-01 bumped exact-tree full 无条件运行。0 matched、skip、synthetic 或未执行均非 PASS。

**Dependencies:** COV2-21；消费前驱真实最终 A/B、适用 focused/full、fmt/strict、唯一 Claude PASS 与 signed+DCO main SHA/hash/mode；不把继承 C/D 的 locally-accepted 冒称 complete。

**Deliverables:** 无独立文件；API、测试、兼容文档、逐文件 SHA/mode 和 review/验收事实写回本计划正文，随唯一发布点交付。

**Implementation write set:** I-core 精确五文件：`docs/development/plan/plan-20261013-coverage-schema.md`; `docs/development/plan/plan-status.md`; `docs/development/tracing/coverage-v2.md`; `docs/development/tracing/agent.md`; `COMPATIBILITY.md`；本卡新增精确路径：`src/internal/ai/capture/pending_payload.rs`; `src/internal/ai/capture/pending.rs`。现有测试模块可写；未列路径包括 `sql/**`、冻结 decoder、parent 计划均为只读，需另行 G-09/I 修订才能扩权。

**Release write set:** N/A；C/D 继承 COV2-REL-01。

**Files likely touched:** I-core 上述五文件；`src/internal/ai/capture/pending_payload.rs`; `src/internal/ai/capture/pending.rs`。

**Docs and compatibility impact:** I-core 的 coverage-v2、agent、COMPATIBILITY 只陈述本卡已交付的能力，不提前宣称下游 artifact/replay/Source A 已完成。无新 `--test` target；`tests/INDEX.md` 无写权。

**Rollback mode:** forward-only；失败保持旧 artifact/claim，签名+DCO 新 main 前滚并重跑下游。

**Migration and rollback:** 无 DDL；Repository claim 使用既有 versioned 键。若实际需迁移，先加角色隔离 migration 卡、旧/新 binary 门并修 G-03/G-04/I。

**Security and privacy:** 继承 GC-COV2-02/03；所有错误 fail closed、零假 ACK、零候选明文 sink；真实 native Source A 仍在父计划。

**Performance budget:** 使用已审 claim/map 上限，解析/查询前有界；不得新增无界扫描、无时限重试或私开的第二连接。

**Estimated scope:** M（AC=8/8、VER=4/8、landing=1、production files=2）。

**Version increment:** N/A；此卡不发版。

**Release boundary:** family child of REL-COV2-01（10/20）；逐卡单 Claude PASS 后 signed+DCO 直推 main，不写 R、不 tag。

**C/D coverage from:** COV2-REL-01；真实 exact-source C/D 通过后本卡才可 done/complete。

**Granularity:** `type=implementation; axis=有界认证 pending-v2 codec 与内存 seal; recovery=forward-only; complete=yes; self-contained=yes; AC=8/8; VER=4/8; landing=1; prod-files=2; scope=M; deps=COV2-21; writeset=严格串行; release=family child; split-from=N/A; exception=N/A`

### Task COV2-08：pending 同版本恢复

**Task type:** implementation

**Lifecycle / Acceptance:** pending / 空

**Description:** pending same-version recovery。本卡唯一行为轴、恢复和粒度见下方 Granularity；不得以 synthetic/native 互换验收。

**Out of scope:** 不以 pending MAC 铸造 native 来源；只消费 COV2-22 真实已持久化 v2 artifact，不以 COV2-21 SQL 探针作 durable receipt。native same-key pending 在父 OG-29 版本关联正例及真实 Source A 前保持 partial/fail closed。

**Current evidence:** src/internal/ai/capture/recovery.rs; src/internal/ai/capture/catalog.rs（基线 `dcc468b`；ER-02 开工刷新行号与实际源码）。

**Acceptance criteria:**

- [ ] AC1 v2 replay retains the original coverage version
- [ ] AC2 replay rejects stale owner
- [ ] AC3 replay rejects stale fence
- [ ] AC4 replay rejects stale revision
- [ ] AC5 failed replay cannot acknowledge a checkpoint
- [ ] AC6 interrupted replay cannot acknowledge one
- [ ] AC7 repairable pending bytes remain available after rejection
- [ ] AC8 pending authentication alone cannot mint native provenance.

**Verification:**

- [ ] VER1→AC1: `source .env.test && cargo test --lib internal::ai::capture::recovery::tests::cov2_replay_preserves_claim_version -- --exact` (1/1).
- [ ] VER2→AC2/3/4: `source .env.test && cargo test --lib internal::ai::capture::recovery::tests::cov2_replay_owner_fence_revision_fail_closed -- --exact` (1/1).
- [ ] VER3→AC5/6/7: `source .env.test && cargo test --lib internal::ai::capture::recovery::tests::cov2_failed_or_interrupted_replay_keeps_pending -- --exact` (1/1).
- [ ] VER4→AC8: `source .env.test && cargo test --lib internal::ai::capture::recovery::tests::cov2_authenticated_pending_has_no_native_authority -- --exact` (1/1). Existing v1 recovery tests are a required ER-04 A surface gate, not an unrun assumption.

**Full-suite trigger:** family child 不自行发布；按 ER-13 对最终改动重新判定 T-1/T-4/T-6。若命中 shared single-source helper、测试基建或 focused 信号不足，则在本卡 A/B 阶段运行双 env 全量 Nextest；无论是否命中，COV2-REL-01 T-2 均须在 bumped 树上重跑。

**Dependencies:** COV2-22；消费前驱实际最终 A/B、适用 focused/full、fmt/strict、单 Claude PASS 与 signed+DCO main 的 source-qualified 文件 SHA/mode；不以其尚待 C/D 的 `locally-accepted` 冒称 complete。

**Deliverables:** 无独立文件；本卡实现/测试/文档随唯一发布点交付；每卡的计数、源码 SHA、hash/mode、review/门结果写在本计划正文，不建证据文件。

**Implementation write set:** I-core = `docs/development/plan/plan-20261013-coverage-schema.md`; `docs/development/plan/plan-status.md`; `docs/development/tracing/coverage-v2.md`; `docs/development/tracing/agent.md`; `COMPATIBILITY.md`；本卡新增精确路径 = `src/internal/ai/capture/catalog.rs`; `src/internal/ai/capture/checkpoint.rs`; `src/internal/ai/capture/recovery.rs`; `src/internal/ai/capture/live_checkpoint.rs`; `src/internal/ai/coverage_gate.rs`。无 glob 默示写权。

**Release write set:** N/A；C/D 继承 COV2-REL-01。

**Files likely touched:** `docs/development/plan/plan-20261013-coverage-schema.md`; `docs/development/plan/plan-status.md`; `docs/development/tracing/coverage-v2.md`; `docs/development/tracing/agent.md`; `COMPATIBILITY.md`; `src/internal/ai/capture/catalog.rs`; `src/internal/ai/capture/checkpoint.rs`; `src/internal/ai/capture/recovery.rs`; `src/internal/ai/capture/live_checkpoint.rs`; `src/internal/ai/coverage_gate.rs`。

**Docs and compatibility impact:** `docs/development/tracing/coverage-v2.md`; `docs/development/tracing/agent.md`; `COMPATIBILITY.md`；逐文件核内容与源码行为对应，`tests/INDEX.md` 只在本卡精确写集确有索引变化时同步。无新增网站命令行为；若实施引入公开差异，先按 ER-06a 修本卡写集再编辑。

**Rollback mode:** forward-only；恢复时签名+DCO 新 main 修复，既有 v1 claim/artifact/digest 不回写或重解释。

**Migration and rollback:** 无 DDL；Repository claim 仅用既有 versioned 键。若实现证明 DDL 必需，停止、加 GC-13 角色隔离迁移卡与旧/新读端门，再修本卡写集和 G-03/G-04。

**Security and privacy:** 继承 GC-COV2-02/03；错误或不可信版本必须 zero-success、不得向 ordinary/typed/log/error 导出候选内容；Session 例只在本机 ignored 执行。

**Performance budget:** 继承 GC-COV2-05；identity map 的 v1+v2 总量仍限 4096 条/1 MiB，任何新增解析在分配前有界；变更 hot path 时以具名门证明无无界扫描。

**Estimated scope:** M（落点/生产文件计数：8/8, 4/8, 2, 5; M）。

**Version increment:** N/A；此卡不发版。

**Release boundary:** family child of REL-COV2-01（12/20）；用户已授权逐卡 reviewed signed+DCO 直推 main，但不写 R、不 tag、不调用 release。

**C/D coverage from:** COV2-REL-01；发布点的 exact-source C/D 绿后才能 `done/complete`。

**Granularity:** `type=implementation; axis=pending 同版本恢复; recovery=forward-only; complete=yes; self-contained=yes; AC=8/8; VER=4/8; landing=2; prod-files=5; scope=M; deps=COV2-22; writeset=串行于前驱; release=family child; split-from=COV2-04; exception=N/A`

### Task COV2-05：版本化 import cursor/digest/故障

**Task type:** implementation

**Lifecycle / Acceptance:** pending / 空

**Description:** version-aware import cursor/digest/fault。本卡唯一行为轴、恢复和粒度见下方 Granularity；不得以 synthetic/native 互换验收。

**Out of scope:** 不启动 OpenCode exporter/Session；若 direct import 测试后来涉及 Session，转本机 ignored 并保留纯 cursor CI 测试。

**Current evidence:** src/internal/ai/agent_import.rs; tests/agent_import_test.rs（基线 `dcc468b`；ER-02 开工刷新行号与实际源码）。

**Acceptance criteria:**

- [ ] AC1 import cursor keys by `(version,logical_turn_key)`
- [ ] AC2 v1 source digest bytes remain exact
- [ ] AC3 v1 import identity preimage remains exact
- [ ] AC4 v2 source digest has an explicit version domain
- [ ] AC5 a stale marker leaves no completed cursor
- [ ] AC6 failed claim leaves no durable imported artifact
- [ ] AC7 mixed v1/v2 import is independently idempotent.

**Verification:**

- [ ] VER1→AC1/2/3/4: `source .env.test && cargo test --lib internal::ai::agent_import::tests::cov2_import_cursor_and_digest_golden -- --exact` (1/1).
- [ ] VER2→AC7: `source .env.test && cargo test --test agent_import_test cov2_mixed_v1_v2_direct_import_idempotent -- --exact` (1/1; use only a synthetic normalized turn and local temp DB, without starting/faking an OpenCode Session. If that boundary cannot be implemented, stop and amend this card before coding; do not silently turn this exact CI gate into a Session test).
- [ ] VER3→AC5: `source .env.test && cargo test --lib internal::ai::agent_import::tests::cov2_stale_marker_keeps_cursor_unfinished -- --exact` (1/1; injected stale-marker fault, zero completed cursor).
- [ ] VER4→AC6: `source .env.test && cargo test --lib internal::ai::agent_import::tests::cov2_failed_claim_writes_no_import_artifact -- --exact` (1/1; injected claim failure). Do not map the graph test to AC5: it cannot prove either fault condition.

**Full-suite trigger:** family child 不自行发布；按 ER-13 对最终改动重新判定 T-1/T-4/T-6。若命中 shared single-source helper、测试基建或 focused 信号不足，则在本卡 A/B 阶段运行双 env 全量 Nextest；无论是否命中，COV2-REL-01 T-2 均须在 bumped 树上重跑。

**Dependencies:** COV2-08；消费前驱实际最终 A/B、适用 focused/full、fmt/strict、单 Claude PASS 与 signed+DCO main 的 source-qualified 文件 SHA/mode；不以其尚待 C/D 的 `locally-accepted` 冒称 complete。

**Deliverables:** 无独立文件；本卡实现/测试/文档随唯一发布点交付；每卡的计数、源码 SHA、hash/mode、review/门结果写在本计划正文，不建证据文件。

**Implementation write set:** I-core = `docs/development/plan/plan-20261013-coverage-schema.md`; `docs/development/plan/plan-status.md`; `docs/development/tracing/coverage-v2.md`; `docs/development/tracing/agent.md`; `COMPATIBILITY.md`；本卡新增精确路径 = `src/internal/ai/agent_import.rs`; `tests/agent_import_test.rs`; `tests/INDEX.md`。无 glob 默示写权。

**Release write set:** N/A；C/D 继承 COV2-REL-01。

**Files likely touched:** `docs/development/plan/plan-20261013-coverage-schema.md`; `docs/development/plan/plan-status.md`; `docs/development/tracing/coverage-v2.md`; `docs/development/tracing/agent.md`; `COMPATIBILITY.md`; `src/internal/ai/agent_import.rs`; `tests/agent_import_test.rs`; `tests/INDEX.md`。

**Docs and compatibility impact:** `docs/development/tracing/coverage-v2.md`; `docs/development/tracing/agent.md`; `COMPATIBILITY.md`；逐文件核内容与源码行为对应，`tests/INDEX.md` 只在本卡精确写集确有索引变化时同步。无新增网站命令行为；若实施引入公开差异，先按 ER-06a 修本卡写集再编辑。

**Rollback mode:** forward-only；恢复时签名+DCO 新 main 修复，既有 v1 claim/artifact/digest 不回写或重解释。

**Migration and rollback:** 无 DDL；Repository claim 仅用既有 versioned 键。若实现证明 DDL 必需，停止、加 GC-13 角色隔离迁移卡与旧/新读端门，再修本卡写集和 G-03/G-04。

**Security and privacy:** 继承 GC-COV2-02/03；错误或不可信版本必须 zero-success、不得向 ordinary/typed/log/error 导出候选内容；Session 例只在本机 ignored 执行。

**Performance budget:** 继承 GC-COV2-05；identity map 的 v1+v2 总量仍限 4096 条/1 MiB，任何新增解析在分配前有界；变更 hot path 时以具名门证明无无界扫描。

**Estimated scope:** M（落点/生产文件计数：7/8, 4/8, 1, 1; M）。

**Version increment:** N/A；此卡不发版。

**Release boundary:** family child of REL-COV2-01（13/20）；用户已授权逐卡 reviewed signed+DCO 直推 main，但不写 R、不 tag、不调用 release。

**C/D coverage from:** COV2-REL-01；发布点的 exact-source C/D 绿后才能 `done/complete`。

**Granularity:** `type=implementation; axis=版本化 import cursor/digest/故障; recovery=forward-only; complete=yes; self-contained=yes; AC=7/8; VER=4/8; landing=1; prod-files=1; scope=M; deps=COV2-08; writeset=串行于前驱; release=family child; split-from=N/A; exception=N/A`

### Task COV2-09：graph/doctor 双版本只读契约

**Task type:** implementation

**Lifecycle / Acceptance:** pending / 空

**Description:** graph/doctor v1+v2 read contract and unsupported-version fail-closed。本卡唯一行为轴、恢复和粒度见下方 Granularity；不得以 synthetic/native 互换验收。

**Out of scope:** 不改 doctor 生产源码，除非先按 G-10 修订本卡精确写集；不改 graph/list 公共 envelope 版本。

**Current evidence:** src/command/agent/graph.rs:329-398; src/command/agent/doctor.rs:1913-1990（基线 `dcc468b`；ER-02 开工刷新行号与实际源码）。

**Acceptance criteria:**

- [ ] AC1 graph lists distinct v1/v2 rows for one logical key
- [ ] AC2 graph does not deduplicate their revisions across versions
- [ ] AC3 graph's public envelope schema remains 1
- [ ] AC4 graph output excludes raw reasoning text
- [ ] AC5 doctor reports each versioned conflict separately
- [ ] AC6 doctor output excludes raw identity
- [ ] AC7 doctor output excludes raw content
- [ ] AC8 graph rejects unsupported coverage versions rather than displaying them as indexed success.

**Verification:**

- [ ] VER1→AC1/2: `source .env.test && cargo test --test agent_graph_test cov2_graph_keeps_versions_distinct -- --exact` (1/1).
- [ ] VER2→AC3/4: `source .env.test && cargo test --test agent_graph_test cov2_graph_envelope_and_text_privacy -- --exact` (1/1).
- [ ] VER3→AC5/6/7: `source .env.test && cargo test --test agent_doctor_repair_test doctor_surfaces_conflicted_coverage_claim_without_raw_identity -- --exact` (existing 1/1, augmented with a raw-content canary absent from both human and JSON output).
- [ ] VER4→AC8: `source .env.test && cargo test --test agent_graph_test cov2_graph_unknown_version_fails_closed -- --exact` (1/1). Implement the narrow supported-version decode in `src/command/agent/graph.rs:329-398` in this same card; preserve public envelope schema 1 and add the actionable corruption/error guidance to the EN/zh command docs and verified `cf` site. `doctor.rs:1913-1990` already groups conflicts by version and is read-only unless VER3 fails, in which case amend I/G before editing.

**Full-suite trigger:** family child 不自行发布；按 ER-13 对最终改动重新判定 T-1/T-4/T-6。若命中 shared single-source helper、测试基建或 focused 信号不足，则在本卡 A/B 阶段运行双 env 全量 Nextest；无论是否命中，COV2-REL-01 T-2 均须在 bumped 树上重跑。

**Dependencies:** COV2-05；DEP-COV2-SITE；消费前驱实际最终 A/B、适用 focused/full、fmt/strict、单 Claude PASS 与 signed+DCO main 的 source-qualified 文件 SHA/mode；不以其尚待 C/D 的 `locally-accepted` 冒称 complete。

**Deliverables:** 无独立文件；本卡实现/测试/文档随唯一发布点交付；每卡的计数、源码 SHA、hash/mode、review/门结果写在本计划正文，不建证据文件。

**Implementation write set:** I-core = `docs/development/plan/plan-20261013-coverage-schema.md`; `docs/development/plan/plan-status.md`; `docs/development/tracing/coverage-v2.md`; `docs/development/tracing/agent.md`; `COMPATIBILITY.md`；本卡新增精确路径 = `src/command/agent/graph.rs`; `tests/agent_graph_test.rs`; `tests/agent_doctor_repair_test.rs`; `tests/INDEX.md`; `docs/development/commands/agent.md`; `docs/commands/agent.md`; `docs/commands/zh-CN/agent.md`; `apps/tanstack-app/content/docs/commands/agent.en.md` (libra-backend `cf`)。无 glob 默示写权。

**Release write set:** N/A；C/D 继承 COV2-REL-01。

**Files likely touched:** `docs/development/plan/plan-20261013-coverage-schema.md`; `docs/development/plan/plan-status.md`; `docs/development/tracing/coverage-v2.md`; `docs/development/tracing/agent.md`; `COMPATIBILITY.md`; `src/command/agent/graph.rs`; `tests/agent_graph_test.rs`; `tests/agent_doctor_repair_test.rs`; `tests/INDEX.md`; `docs/development/commands/agent.md`; `docs/commands/agent.md`; `docs/commands/zh-CN/agent.md`; `apps/tanstack-app/content/docs/commands/agent.en.md` (libra-backend `cf`)。

**Docs and compatibility impact:** `docs/development/tracing/coverage-v2.md`; `docs/development/tracing/agent.md`; `COMPATIBILITY.md`; `docs/development/commands/agent.md`; `docs/commands/agent.md`; `docs/commands/zh-CN/agent.md`; `apps/tanstack-app/content/docs/commands/agent.en.md` (libra-backend `cf`)；逐文件核内容与源码行为对应，`tests/INDEX.md` 只在本卡精确写集确有索引变化时同步。网站文件仅在 DEP-COV2-SITE 的隔离 `cf` owner 中编辑、同卡 typecheck/build、签名交付；文案须写成“仅适用于首个包含 coverage-v2 的稳定发布及其后续版本”等不依赖未确定版本号的条件，不可声称当前稳定版已提供未发布能力；当前原 checkout 脏且落后，禁止覆盖。

**Rollback mode:** forward-only；恢复时签名+DCO 新 main 修复，既有 v1 claim/artifact/digest 不回写或重解释。

**Migration and rollback:** 无 DDL；Repository claim 仅用既有 versioned 键。若实现证明 DDL 必需，停止、加 GC-13 角色隔离迁移卡与旧/新读端门，再修本卡写集和 G-03/G-04。

**Security and privacy:** 继承 GC-COV2-02/03；错误或不可信版本必须 zero-success、不得向 ordinary/typed/log/error 导出候选内容；Session 例只在本机 ignored 执行。

**Performance budget:** 继承 GC-COV2-05；identity map 的 v1+v2 总量仍限 4096 条/1 MiB，任何新增解析在分配前有界；变更 hot path 时以具名门证明无无界扫描。

**Estimated scope:** M（落点/生产文件计数：8/8, 4/8, 1, 1; M）。

**Version increment:** N/A；此卡不发版。

**Release boundary:** family child of REL-COV2-01（14/20）；用户已授权逐卡 reviewed signed+DCO 直推 main，但不写 R、不 tag、不调用 release。

**C/D coverage from:** COV2-REL-01；发布点的 exact-source C/D 绿后才能 `done/complete`。

**Granularity:** `type=implementation; axis=graph/doctor 双版本只读契约; recovery=forward-only; complete=yes; self-contained=yes; AC=8/8; VER=4/8; landing=1; prod-files=1; scope=M; deps=COV2-05；DEP-COV2-SITE; writeset=串行于前驱; release=family child; split-from=COV2-05; exception=N/A`

### Task COV2-06：OpenCode part 到 typed 来源分类

**Task type:** implementation

**Lifecycle / Acceptance:** pending / 空

**Description:** OpenCode part classification into typed source。本卡唯一行为轴、恢复和粒度见下方 Granularity；不得以 synthetic/native 互换验收。

**Out of scope:** 只跑纯解析器；不发起模型调用、不创建 Session，也不认证 native 字段。

**Current evidence:** src/internal/ai/observed_agents/opencode_reasoning.rs; src/internal/ai/observed_agents/reasoning.rs（基线 `dcc468b`；ER-02 开工刷新行号与实际源码）。

**Acceptance criteria:**

- [ ] AC1 nonempty visible-only text yields one typed Reasoning record
- [ ] AC2 that record retains its source-turn coordinate
- [ ] AC3 empty visible text remains partial
- [ ] AC4 state/proof-bearing content remains partial
- [ ] AC5 unrecognized export shape remains partial
- [ ] AC6 malformed export remains partial
- [ ] AC7 duplicate-key export remains partial
- [ ] AC8 before COV2-11, the production OpenCode v2 claim entry remains unreachable.

**Verification:**

- [ ] VER1→AC1/2/3: `source .env.test && cargo test --lib internal::ai::observed_agents::opencode_reasoning::tests::cov2_visible_only_classification -- --exact` (1/1).
- [ ] VER2→AC4/5: `source .env.test && cargo test --lib internal::ai::observed_agents::opencode_reasoning::tests::cov2_candidate_and_unsupported_partial -- --exact` (1/1).
- [ ] VER3→AC5/6/7: `source .env.test && cargo test --lib internal::ai::observed_agents::opencode_reasoning::tests::cov2_malformed_duplicate_unknown_partial -- --exact` (1/1). These are pure parser tests; no Session/exporter/model runs in CI.
- [ ] VER4→AC8: `source .env.test && cargo test --test agent_reasoning_contract_test cov2_production_normalize_inert_before_activation -- --exact` (1/1; direct normalized-turn adapter, no Session; assert partial/no completed v2 claim and statically pin absence of a COV2-01 v2 API call in the live/import OpenCode claim-entry path until COV2-11 switches it).

**Full-suite trigger:** family child 不自行发布；按 ER-13 对最终改动重新判定 T-1/T-4/T-6。若命中 shared single-source helper、测试基建或 focused 信号不足，则在本卡 A/B 阶段运行双 env 全量 Nextest；无论是否命中，COV2-REL-01 T-2 均须在 bumped 树上重跑。

**Dependencies:** COV2-09；DEP-COV2-OG26；消费前驱实际最终 A/B、适用 focused/full、fmt/strict、单 Claude PASS 与 signed+DCO main 的 source-qualified 文件 SHA/mode；不以其尚待 C/D 的 `locally-accepted` 冒称 complete。

**Deliverables:** 无独立文件；本卡实现/测试/文档随唯一发布点交付；每卡的计数、源码 SHA、hash/mode、review/门结果写在本计划正文，不建证据文件。

**Implementation write set:** I-core = `docs/development/plan/plan-20261013-coverage-schema.md`; `docs/development/plan/plan-status.md`; `docs/development/tracing/coverage-v2.md`; `docs/development/tracing/agent.md`; `COMPATIBILITY.md`；本卡新增精确路径 = `src/internal/ai/observed_agents/opencode_reasoning.rs`; `src/internal/ai/observed_agents/reasoning.rs`; `src/internal/ai/observed_agents/mod.rs`; `tests/agent_reasoning_contract_test.rs`。无 glob 默示写权。

**Release write set:** N/A；C/D 继承 COV2-REL-01。

**Files likely touched:** `docs/development/plan/plan-20261013-coverage-schema.md`; `docs/development/plan/plan-status.md`; `docs/development/tracing/coverage-v2.md`; `docs/development/tracing/agent.md`; `COMPATIBILITY.md`; `src/internal/ai/observed_agents/opencode_reasoning.rs`; `src/internal/ai/observed_agents/reasoning.rs`; `src/internal/ai/observed_agents/mod.rs`; `tests/agent_reasoning_contract_test.rs`。

**Docs and compatibility impact:** `docs/development/tracing/coverage-v2.md`; `docs/development/tracing/agent.md`; `COMPATIBILITY.md`；逐文件核内容与源码行为对应，`tests/INDEX.md` 只在本卡精确写集确有索引变化时同步。无新增网站命令行为；若实施引入公开差异，先按 ER-06a 修本卡写集再编辑。

**Rollback mode:** forward-only；恢复时签名+DCO 新 main 修复，既有 v1 claim/artifact/digest 不回写或重解释。

**Migration and rollback:** 无 DDL；Repository claim 仅用既有 versioned 键。若实现证明 DDL 必需，停止、加 GC-13 角色隔离迁移卡与旧/新读端门，再修本卡写集和 G-03/G-04。

**Security and privacy:** 继承 GC-COV2-02/03；错误或不可信版本必须 zero-success、不得向 ordinary/typed/log/error 导出候选内容；Session 例只在本机 ignored 执行。

**Performance budget:** 继承 GC-COV2-05；identity map 的 v1+v2 总量仍限 4096 条/1 MiB，任何新增解析在分配前有界；变更 hot path 时以具名门证明无无界扫描。

**Estimated scope:** M（落点/生产文件计数：8/8, 4/8, 1, 3; M）。

**Version increment:** N/A；此卡不发版。

**Release boundary:** family child of REL-COV2-01（15/20）；用户已授权逐卡 reviewed signed+DCO 直推 main，但不写 R、不 tag、不调用 release。

**C/D coverage from:** COV2-REL-01；发布点的 exact-source C/D 绿后才能 `done/complete`。

**Granularity:** `type=implementation; axis=OpenCode typed 来源分类的未激活交付; recovery=forward-only; complete=yes; self-contained=yes; AC=8/8; VER=4/8; landing=1; prod-files=3; scope=M; deps=COV2-09；DEP-COV2-OG26; writeset=串行于前驱; release=family child; split-from=N/A; exception=N/A`

### Task COV2-10：typed 脱敏与零明文 sink

**Task type:** implementation

**Lifecycle / Acceptance:** pending / 空

**Description:** typed redaction and authority-zero-sink。本卡唯一行为轴、恢复和粒度见下方 Granularity；不得以 synthetic/native 互换验收。

**Out of scope:** 不持久化 proof/state-bearing 候选为普通或 typed 明文；live/import 正式接线由 COV2-11 承接。

**Current evidence:** src/internal/ai/capture/snapshot.rs; src/internal/ai/observed_agents/reasoning.rs:126-129（基线 `dcc468b`；ER-02 开工刷新行号与实际源码）。

**Acceptance criteria:**

- [ ] AC1 redaction precedes v2 digest
- [ ] AC2 redaction precedes safe projection
- [ ] AC3 redaction precedes persistence
- [ ] AC4 candidate raw bytes never reach ordinary plaintext
- [ ] AC5 candidate raw bytes never reach typed plaintext
- [ ] AC6 readable v2 cannot mint a native issuer
- [ ] AC7 readable v2 cannot create an opaque encrypted artifact
- [ ] AC8 non-OpenCode adapters retain their previous result.

**Verification:**

- [ ] VER1→AC1/2: `source .env.test && cargo test --lib internal::ai::capture::snapshot::tests::cov2_visible_text_redacted_before_digest_and_projection -- --exact` (1/1; in-memory only).
- [ ] VER2→AC3/4/5: `source .env.test && cargo test --test agent_reasoning_contract_test cov2_candidate_zero_plaintext_sinks -- --exact` (1/1; in-memory sink spy, no Session).
- [ ] VER3→AC6/7: `source .env.test && cargo test --test compat_agent_architecture_guard cov2_visible_text_cannot_mint_native_or_opaque -- --exact` (1/1; type/static guard).
- [ ] VER4→AC8: `source .env.test && cargo test --test agent_reasoning_contract_test cov2_non_opencode_adapter_unchanged -- --exact` (1/1).

**Full-suite trigger:** family child 不自行发布；按 ER-13 对最终改动重新判定 T-1/T-4/T-6。若命中 shared single-source helper、测试基建或 focused 信号不足，则在本卡 A/B 阶段运行双 env 全量 Nextest；无论是否命中，COV2-REL-01 T-2 均须在 bumped 树上重跑。

**Dependencies:** COV2-06；消费前驱实际最终 A/B、适用 focused/full、fmt/strict、单 Claude PASS 与 signed+DCO main 的 source-qualified 文件 SHA/mode；不以其尚待 C/D 的 `locally-accepted` 冒称 complete。

**Deliverables:** 无独立文件；本卡实现/测试/文档随唯一发布点交付；每卡的计数、源码 SHA、hash/mode、review/门结果写在本计划正文，不建证据文件。

**Implementation write set:** I-core = `docs/development/plan/plan-20261013-coverage-schema.md`; `docs/development/plan/plan-status.md`; `docs/development/tracing/coverage-v2.md`; `docs/development/tracing/agent.md`; `COMPATIBILITY.md`；本卡新增精确路径 = `src/internal/ai/capture/snapshot.rs`; `src/internal/ai/observed_agents/reasoning.rs`; `src/internal/ai/observed_agents/coverage.rs`; `src/internal/ai/observed_agents/opencode_reasoning.rs`; `tests/agent_reasoning_contract_test.rs`; `tests/compat/agent_architecture_guard.rs`。无 glob 默示写权。

**Release write set:** N/A；C/D 继承 COV2-REL-01。

**Files likely touched:** `docs/development/plan/plan-20261013-coverage-schema.md`; `docs/development/plan/plan-status.md`; `docs/development/tracing/coverage-v2.md`; `docs/development/tracing/agent.md`; `COMPATIBILITY.md`; `src/internal/ai/capture/snapshot.rs`; `src/internal/ai/observed_agents/reasoning.rs`; `src/internal/ai/observed_agents/coverage.rs`; `src/internal/ai/observed_agents/opencode_reasoning.rs`; `tests/agent_reasoning_contract_test.rs`; `tests/compat/agent_architecture_guard.rs`。

**Docs and compatibility impact:** `docs/development/tracing/coverage-v2.md`; `docs/development/tracing/agent.md`; `COMPATIBILITY.md`；逐文件核内容与源码行为对应，`tests/INDEX.md` 只在本卡精确写集确有索引变化时同步。无新增网站命令行为；若实施引入公开差异，先按 ER-06a 修本卡写集再编辑。

**Rollback mode:** forward-only；恢复时签名+DCO 新 main 修复，既有 v1 claim/artifact/digest 不回写或重解释。

**Migration and rollback:** 无 DDL；Repository claim 仅用既有 versioned 键。若实现证明 DDL 必需，停止、加 GC-13 角色隔离迁移卡与旧/新读端门，再修本卡写集和 G-03/G-04。

**Security and privacy:** 继承 GC-COV2-02/03；错误或不可信版本必须 zero-success、不得向 ordinary/typed/log/error 导出候选内容；Session 例只在本机 ignored 执行。

**Performance budget:** 继承 GC-COV2-05；identity map 的 v1+v2 总量仍限 4096 条/1 MiB，任何新增解析在分配前有界；变更 hot path 时以具名门证明无无界扫描。

**Estimated scope:** M（落点/生产文件计数：8/8, 4/8, 2, 4; M）。

**Version increment:** N/A；此卡不发版。

**Release boundary:** family child of REL-COV2-01（16/20）；用户已授权逐卡 reviewed signed+DCO 直推 main，但不写 R、不 tag、不调用 release。

**C/D coverage from:** COV2-REL-01；发布点的 exact-source C/D 绿后才能 `done/complete`。

**Granularity:** `type=implementation; axis=typed 脱敏与零明文 sink; recovery=forward-only; complete=yes; self-contained=yes; AC=8/8; VER=4/8; landing=2; prod-files=4; scope=M; deps=COV2-06; writeset=串行于前驱; release=family child; split-from=COV2-06; exception=N/A`

### Task COV2-11：live/import v2 生产接线与字节对齐

**Task type:** implementation

**Lifecycle / Acceptance:** pending / 空

**Description:** live/import v2 activation and byte parity。本卡唯一行为轴、恢复和粒度见下方 Granularity；不得以 synthetic/native 互换验收。

**Out of scope:** 只用直接 normalized-turn 入口验证；实际 OpenCode Session/假 exporter E2E 由 COV2-12 在 macOS 手工完成。

**Current evidence:** src/internal/ai/capture/live_checkpoint.rs; src/internal/ai/agent_import.rs（基线 `dcc468b`；ER-02 开工刷新行号与实际源码）。

**Acceptance criteria:**

- [ ] AC1 the live path writes a v2 typed claim
- [ ] AC2 historical import writes a v2 typed claim
- [ ] AC3 the two paths produce identical canonical bytes
- [ ] AC4 their digests match
- [ ] AC5 v1 turns with no typed Reasoning retain old bytes
- [ ] AC6 pending replay of that v2 claim retains its version
- [ ] AC7 candidate turns remain partial through the runtime adapter
- [ ] AC8 production live/import typed Reasoning never calls the legacy-v1 projection API.

**Verification:**

- [ ] VER1→AC1: `source .env.test && cargo test --lib internal::ai::capture::live_checkpoint::tests::cov2_direct_visible_turn_claim_version -- --exact` (1/1; direct normalized-turn function, no OpenCode Session).
- [ ] VER2→AC2/5: `source .env.test && cargo test --lib internal::ai::agent_import::tests::cov2_direct_visible_import_and_v1_parity -- --exact` (1/1; direct normalized-turn function, no Session/exporter).
- [ ] VER3→AC3/4/8: `source .env.test && cargo test --test agent_reasoning_contract_test cov2_production_typed_v2_distinct_from_rg04_legacy -- --exact` (1/1; explicit dual-path golden, no Session).
- [ ] VER4→AC6: `source .env.test && cargo test --lib internal::ai::capture::recovery::tests::cov2_runtime_claim_replay_version -- --exact` (1/1). Actual Session route remains COV2-12 manual only.
- [ ] VER5→AC7: `source .env.test && cargo test --lib internal::ai::capture::live_checkpoint::tests::cov2_direct_candidate_turn_stays_partial -- --exact` (1/1; proof/state-bearing candidate-specific direct normalized-turn assertion, no Session, no issuer/opaque mint).
- [ ] VER6→AC8: `source .env.test && cargo test --test agent_reasoning_contract_test cov2_production_typed_path_uses_v2_api -- --exact` (1/1; exercise live/import production call sites and assert typed-v2 versus RG-04 legacy-v1, plus a static source guard forbidding fallback to legacy helper for typed turns). 同卡把 COV2-06 的 `cov2_production_normalize_inert_before_activation` 守卫改为 post-activation v2 守卫并记录旧→新 selector；保留前驱签名源码的历史 PASS，不让旧“仍未激活”断言留在默认 CI 变红。

**Full-suite trigger:** family child 不自行发布；按 ER-13 对最终改动重新判定 T-1/T-4/T-6。若命中 shared single-source helper、测试基建或 focused 信号不足，则在本卡 A/B 阶段运行双 env 全量 Nextest；无论是否命中，COV2-REL-01 T-2 均须在 bumped 树上重跑。

**Dependencies:** COV2-10；DEP-COV2-SITE；消费前驱实际最终 A/B、适用 focused/full、fmt/strict、单 Claude PASS 与 signed+DCO main 的 source-qualified 文件 SHA/mode；不以其尚待 C/D 的 `locally-accepted` 冒称 complete。

**Deliverables:** 无独立文件；本卡实现/测试/文档随唯一发布点交付；每卡的计数、源码 SHA、hash/mode、review/门结果写在本计划正文，不建证据文件。

**Implementation write set:** I-core = `docs/development/plan/plan-20261013-coverage-schema.md`; `docs/development/plan/plan-status.md`; `docs/development/tracing/coverage-v2.md`; `docs/development/tracing/agent.md`; `COMPATIBILITY.md`；本卡新增精确路径 = `src/internal/ai/capture/live_checkpoint.rs`; `src/internal/ai/agent_import.rs`; `src/internal/ai/capture/snapshot.rs`; `src/internal/ai/capture/recovery.rs`; `src/internal/ai/observed_agents/mod.rs`; `tests/agent_reasoning_contract_test.rs`; `tests/INDEX.md`; `docs/development/commands/agent.md`; `docs/commands/agent.md`; `docs/commands/zh-CN/agent.md`; `apps/tanstack-app/content/docs/commands/agent.en.md` (libra-backend `cf`)。无 glob 默示写权。

**Release write set:** N/A；C/D 继承 COV2-REL-01。

**Files likely touched:** `docs/development/plan/plan-20261013-coverage-schema.md`; `docs/development/plan/plan-status.md`; `docs/development/tracing/coverage-v2.md`; `docs/development/tracing/agent.md`; `COMPATIBILITY.md`; `src/internal/ai/capture/live_checkpoint.rs`; `src/internal/ai/agent_import.rs`; `src/internal/ai/capture/snapshot.rs`; `src/internal/ai/capture/recovery.rs`; `src/internal/ai/observed_agents/mod.rs`; `tests/agent_reasoning_contract_test.rs`; `tests/INDEX.md`; `docs/development/commands/agent.md`; `docs/commands/agent.md`; `docs/commands/zh-CN/agent.md`; `apps/tanstack-app/content/docs/commands/agent.en.md` (libra-backend `cf`)。

**Docs and compatibility impact:** `docs/development/tracing/coverage-v2.md`; `docs/development/tracing/agent.md`; `COMPATIBILITY.md`; `docs/development/commands/agent.md`; `docs/commands/agent.md`; `docs/commands/zh-CN/agent.md`; `apps/tanstack-app/content/docs/commands/agent.en.md` (libra-backend `cf`)；逐文件核内容与源码行为对应，`tests/INDEX.md` 只在本卡精确写集确有索引变化时同步。网站文件仅在 DEP-COV2-SITE 的隔离 `cf` owner 中编辑、同卡 typecheck/build、签名交付；文案须写成“仅适用于首个包含 coverage-v2 的稳定发布及其后续版本”等不依赖未确定版本号的条件，不可声称当前稳定版已提供未发布能力；当前原 checkout 脏且落后，禁止覆盖。

**Rollback mode:** forward-only；恢复时签名+DCO 新 main 修复，既有 v1 claim/artifact/digest 不回写或重解释。

**Migration and rollback:** 无 DDL；Repository claim 仅用既有 versioned 键。若实现证明 DDL 必需，停止、加 GC-13 角色隔离迁移卡与旧/新读端门，再修本卡写集和 G-03/G-04。

**Security and privacy:** 继承 GC-COV2-02/03；错误或不可信版本必须 zero-success、不得向 ordinary/typed/log/error 导出候选内容；Session 例只在本机 ignored 执行。

**Performance budget:** 继承 GC-COV2-05；identity map 的 v1+v2 总量仍限 4096 条/1 MiB，任何新增解析在分配前有界；变更 hot path 时以具名门证明无无界扫描。

**Estimated scope:** M（落点/生产文件计数：8/8, 6/8, 3, 5; M）。

**Version increment:** N/A；此卡不发版。

**Release boundary:** family child of REL-COV2-01（17/20）；用户已授权逐卡 reviewed signed+DCO 直推 main，但不写 R、不 tag、不调用 release。

**C/D coverage from:** COV2-REL-01；发布点的 exact-source C/D 绿后才能 `done/complete`。

**Granularity:** `type=implementation; axis=live/import v2 生产接线与字节对齐; recovery=forward-only; complete=yes; self-contained=yes; AC=8/8; VER=6/8; landing=3; prod-files=5; scope=M; deps=COV2-10；DEP-COV2-SITE; writeset=串行于前驱; release=family child; split-from=COV2-06; exception=N/A`

### Task COV2-12：macOS 本地 Session 入口与既有 fixture 修复

**Task type:** implementation

**Lifecycle / Acceptance:** pending / 空

**Description:** macOS trusted fake exporter integration, ignored Session gates and legacy extraction repair。本卡的旧 fixture 修复只调整此 Session 入口的既有 ignored 断言/预检，不能改变生产授权或把独立行为错误塞入此卡；恢复和粒度见下方 Granularity；不得以 synthetic/native 互换验收。

**Out of scope:** 不以假 exporter 验证真实 native 来源；候选 live/import 零 sink 分别由 COV2-15/18；Linux 功能由用户后验。除本卡既有 extraction fixture 外，其余 21 个继承 Session 案例的 test bodies 只读；同一测试文件的共享 helper 仅能在本卡精确写集内按 VER1/2/3/4/5 修复（包括 `BridgeRepo::init` 的 macOS 提前返回），若 COV2-16/17 审计发现其它案例缺陷，按 ER-10 新建精确写集 FIX 卡、前滚并重新审计，不在 COV2-12 扩权。

**Current evidence:** tests/agent_opencode_bridge_test.rs:38-45; src/internal/ai/capture/live_checkpoint.rs:973-987; src/internal/ai/capture/live_pipeline_tests.rs:5553-5630（基线 `dcc468b`；ER-02 开工刷新行号与实际源码）。

**Acceptance criteria:**

- [ ] AC1 existing OpenCode Session capture E2Es are ignored in ordinary cargo/Nextest
- [ ] AC2 CI does not explicitly select those ignored cases
- [ ] AC3 macOS fake-exporter live Session capture produces a v2 claim
- [ ] AC4 macOS fake-exporter Session import/replay matches its live canonical bytes
- [ ] AC5 fail-closed Session negatives run only as ignored macOS local tests
- [ ] AC6 the existing ignored no-consumer extraction scenario writes no completed checkpoint
- [ ] AC7 a missing macOS containment prerequisite fails its manual Session test instead of returning success
- [ ] AC8 the existing ignored no-consumer extraction scenario retains its pending receipt.

**Verification:**

- [ ] VER1→AC1/2: `source .env.test && cargo test --test compat_agent_architecture_guard cov2_opencode_session_e2e_is_manual_only -- --exact` (1/1; inventory all existing/new Session cases and workflow selectors; `.github/workflows/base.yml` 的 `run_gate` 对八个 Linux sandbox unit selectors 都用 `--include-ignored`：`runner_controls_preserved`、`trusted_bwrap_preflight`、`bwrap_argv_equivalent`、`trusted_bwrap_rejects_user_writable`、`backend_missing_degrades_metadata_only`、`trusted_bwrap_exe_channel_used`、`opencode_export_sandboxed_outer_cancel_contains_setsid_writer`、`trusted_export_retry_recovers_a_transient_sandbox_failure`。逐名白名单只允许这些不产生 OpenCode Session/Checkpoint 的 Linux sandbox unit gates；其中 `trusted_bwrap_preflight` 和 `opencode_export_sandboxed_outer_cancel_contains_setsid_writer` 是 ignored，`runner_controls_preserved` 不是。任何新增或显式选择 `agent_opencode_bridge_test`、`cov2_*` 及其它 OpenCode Session/Checkpoint E2E 的 CI 路径均须使静态守卫失败，不以泛化的 `--include-ignored` 规则豁免)。
- [ ] VER2→AC3/7: **macOS manual** `source .env.test && cargo test --features test-live-agent --test agent_opencode_bridge_test cov2_macos_fake_exporter_live_session_capture -- --ignored --exact --nocapture --test-threads=1` (1/1).
- [ ] VER3→AC4/7: **macOS manual** `source .env.test && cargo test --features test-live-agent --test agent_opencode_bridge_test cov2_macos_fake_exporter_import_session_capture -- --ignored --exact --nocapture --test-threads=1` (1/1; assert the independently pinned COV2-11 digest parity too).
- [ ] VER4→AC5/7: **macOS manual** `source .env.test && cargo test --features test-live-agent --test agent_opencode_bridge_test cov2_macos_failed_export_session_zero_write -- --ignored --exact --nocapture --test-threads=1` (1/1; actual Session fail-closed negative, no CI fallback).
- [ ] VER5→AC7: `source .env.test && cargo test --test compat_agent_architecture_guard cov2_manual_session_tests_cannot_silently_skip -- --exact` (1/1; static guard on all new and pre-existing ignored Session test bodies, including the legacy lib case, and the `test-live-agent` feature gate).
- [ ] VER6→AC6/7/8: **macOS manual preflight and final gate** `source .env.test && source .env.live-test && cargo test --lib internal::ai::capture::live_pipeline::tests::export_runner_lease::end_to_end::opencode_extraction_failure_continues_checkpoint -- --ignored --exact --nocapture --test-threads=1` (final 1/1 PASS required under the current name; prior exit 101 on signed main `5cafe322` is documented in `/private/tmp/libra-opencode-ci-handoff-20261013/local-opencode-extraction-fail.log`). First rerun after COV2-11 to determine the actual sealed disposition and record it without private payloads. `live_checkpoint.rs:973-987` rejects `!has_capture_consumer()`, and `reasoning.rs:126-129` currently admits only Ordinary/NativeProven; therefore for an Unmapped no-consumer fixture, update the ignored test to assert pending receipt and zero checkpoint rather than relax production authorization. If COV2-11 establishes a ready v2 consumer for this exact fixture, stop and revise AC6/8, the selector, and G-03 before implementation; then preserve and prove the continuation assertion under the revised source-qualified card. If the test must be renamed to describe the corrected no-consumer contract, update `live_pipeline_tests.rs`, `tests/compat/live_compat_workflow.rs`, `tests/INDEX.md`, `integration-test-plan.md` and this exact VER6 selector in the same card before acceptance; the final renamed command must run 1/1, and the old failing scenario must not be deleted. COV2-12 owns only the test body in `live_pipeline_tests.rs:5553-5630`; `live_checkpoint.rs:366-369,973-987` is read-only policy. If an actual production consumer-policy defect is proven, first create a separately reviewed FIX card with exact write set before editing; no 0-test/skip counts as PASS. Rerun affected focused/full gates and Claude review on final bytes. The current positive bridge harness requires Linux `bwrap` and skips macOS; VER2/3 require a new trusted **macOS Required Seatbelt** fake-exporter path through the actual local Session hook/import, never a production trust relaxation. A real native `signature` positive remains OG-27's separate gate.

**Full-suite trigger:** family child 不自行发布；按 ER-13 对最终改动重新判定 T-1/T-4/T-6。若命中 shared single-source helper、测试基建或 focused 信号不足，则在本卡 A/B 阶段运行双 env 全量 Nextest；无论是否命中，COV2-REL-01 T-2 均须在 bumped 树上重跑。

**Dependencies:** COV2-11；消费前驱实际最终 A/B、适用 focused/full、fmt/strict、单 Claude PASS 与 signed+DCO main 的 source-qualified 文件 SHA/mode；不以其尚待 C/D 的 `locally-accepted` 冒称 complete。

**Deliverables:** 无独立文件；本卡实现/测试/文档随唯一发布点交付；每卡的计数、源码 SHA、hash/mode、review/门结果写在本计划正文，不建证据文件。

**Implementation write set:** I-core = `docs/development/plan/plan-20261013-coverage-schema.md`; `docs/development/plan/plan-status.md`; `docs/development/tracing/coverage-v2.md`; `docs/development/tracing/agent.md`; `COMPATIBILITY.md`；本卡新增精确路径 = `src/internal/ai/observed_agents/opencode_export.rs`; `src/internal/ai/capture/live_pipeline_tests.rs`; `tests/agent_opencode_bridge_test.rs`; `tests/compat/agent_architecture_guard.rs`; `tests/compat/live_compat_workflow.rs`; `tests/INDEX.md`; `docs/development/integration/integration-test-plan.md`。无 glob 默示写权。

`opencode_export.rs` 的唯一允许生产变化是让已注册本机 macOS fake-exporter Session 走现有 Required Seatbelt runner，保持原 trust/超时/取消/日志约束；VER2/3 在实际 hook/import 路由上验证该落点。若测试无需修改该文件，保持其 SHA/mode 不变；不得借此改 native 准入。

**Release write set:** N/A；C/D 继承 COV2-REL-01。

**Files likely touched:** `docs/development/plan/plan-20261013-coverage-schema.md`; `docs/development/plan/plan-status.md`; `docs/development/tracing/coverage-v2.md`; `docs/development/tracing/agent.md`; `COMPATIBILITY.md`; `src/internal/ai/observed_agents/opencode_export.rs`; `src/internal/ai/capture/live_pipeline_tests.rs`; `tests/agent_opencode_bridge_test.rs`; `tests/compat/agent_architecture_guard.rs`; `tests/compat/live_compat_workflow.rs`; `tests/INDEX.md`; `docs/development/integration/integration-test-plan.md`。

**Docs and compatibility impact:** `docs/development/tracing/coverage-v2.md`; `docs/development/tracing/agent.md`; `COMPATIBILITY.md`; `docs/development/integration/integration-test-plan.md`；逐文件核内容与源码行为对应，`tests/INDEX.md` 只在本卡精确写集确有索引变化时同步。无新增网站命令行为；若实施引入公开差异，先按 ER-06a 修本卡写集再编辑。

**Rollback mode:** forward-only；恢复时签名+DCO 新 main 修复，既有 v1 claim/artifact/digest 不回写或重解释。

**Migration and rollback:** 无 DDL；Repository claim 仅用既有 versioned 键。若实现证明 DDL 必需，停止、加 GC-13 角色隔离迁移卡与旧/新读端门，再修本卡写集和 G-03/G-04。

**Security and privacy:** 继承 GC-COV2-02/03；错误或不可信版本必须 zero-success、不得向 ordinary/typed/log/error 导出候选内容；Session 例只在本机 ignored 执行。

**Performance budget:** 继承 GC-COV2-05；identity map 的 v1+v2 总量仍限 4096 条/1 MiB，任何新增解析在分配前有界；变更 hot path 时以具名门证明无无界扫描。

**Estimated scope:** M（落点/生产文件计数：8/8, 6/8, 1, 1; M）。

**Version increment:** N/A；此卡不发版。

**Release boundary:** family child of REL-COV2-01（18/20）；用户已授权逐卡 reviewed signed+DCO 直推 main，但不写 R、不 tag、不调用 release。

**C/D coverage from:** COV2-REL-01；发布点的 exact-source C/D 绿后才能 `done/complete`。

**Granularity:** `type=implementation; axis=macOS 本地 Session 入口与既有 fixture 修复; recovery=forward-only; complete=yes; self-contained=yes; AC=8/8; VER=6/8; landing=1; prod-files=1; scope=M; deps=COV2-11; writeset=串行于前驱; release=family child; split-from=COV2-06; exception=N/A`

### Task COV2-15：候选 live Session 权限防火墙

**Task type:** implementation

**Lifecycle / Acceptance:** pending / 空

**Description:** proof/state-bearing candidate live Session firewall。本卡唯一行为轴、恢复和粒度见下方 Granularity；不得以 synthetic/native 互换验收。

**Out of scope:** 只负责 live hook 路径候选隔离；historical import 独立由 COV2-18。

**Current evidence:** src/internal/ai/capture/live_checkpoint.rs; tests/agent_opencode_bridge_test.rs（基线 `dcc468b`；ER-02 开工刷新行号与实际源码）。

**Acceptance criteria:**

- [ ] AC1 the trusted macOS fake exporter actually executes through the live hook Session path with a proof/state-bearing OpenCode part
- [ ] AC2 live classification is partial
- [ ] AC3 candidate canary is absent from ordinary plaintext
- [ ] AC4 it is absent from typed plaintext
- [ ] AC5 it is absent from persisted checkpoint objects
- [ ] AC6 it is absent from logs and public errors
- [ ] AC7 no native issuer is minted
- [ ] AC8 no opaque encrypted artifact is minted.

**Verification:**

- [ ] VER1→AC1–8: **macOS manual** `source .env.test && cargo test --features test-live-agent --test agent_opencode_bridge_test cov2_macos_fake_exporter_candidate_live_partial_no_plaintext -- --ignored --exact --nocapture --test-threads=1` (1/1; fake exporter succeeds, candidate flows through actual hook; test asserts each sink independently).
- [ ] VER2→AC1: `source .env.test && cargo test --test compat_agent_architecture_guard cov2_candidate_live_session_case_is_ignored_and_non_skippable -- --exact` (1/1; static guard; no Session in CI). Zero tests, early return or prerequisite skip is UNRUN. Mechanism-only parser/in-memory tests stay in CI. If source routing needs a different production path, amend exact I before editing; never create fake native authority.

**Full-suite trigger:** family child 不自行发布；按 ER-13 对最终改动重新判定 T-1/T-4/T-6。若命中 shared single-source helper、测试基建或 focused 信号不足，则在本卡 A/B 阶段运行双 env 全量 Nextest；无论是否命中，COV2-REL-01 T-2 均须在 bumped 树上重跑。

**Dependencies:** COV2-12；消费前驱实际最终 A/B、适用 focused/full、fmt/strict、单 Claude PASS 与 signed+DCO main 的 source-qualified 文件 SHA/mode；不以其尚待 C/D 的 `locally-accepted` 冒称 complete。

**Deliverables:** 无独立文件；本卡实现/测试/文档随唯一发布点交付；每卡的计数、源码 SHA、hash/mode、review/门结果写在本计划正文，不建证据文件。

**Implementation write set:** I-core = `docs/development/plan/plan-20261013-coverage-schema.md`; `docs/development/plan/plan-status.md`; `docs/development/tracing/coverage-v2.md`; `docs/development/tracing/agent.md`; `COMPATIBILITY.md`；本卡新增精确路径 = `src/internal/ai/capture/live_checkpoint.rs`; `src/internal/ai/observed_agents/opencode_reasoning.rs`; `tests/agent_opencode_bridge_test.rs`; `tests/compat/agent_architecture_guard.rs`; `tests/INDEX.md`。无 glob 默示写权。

**Release write set:** N/A；C/D 继承 COV2-REL-01。

**Files likely touched:** `docs/development/plan/plan-20261013-coverage-schema.md`; `docs/development/plan/plan-status.md`; `docs/development/tracing/coverage-v2.md`; `docs/development/tracing/agent.md`; `COMPATIBILITY.md`; `src/internal/ai/capture/live_checkpoint.rs`; `src/internal/ai/observed_agents/opencode_reasoning.rs`; `tests/agent_opencode_bridge_test.rs`; `tests/compat/agent_architecture_guard.rs`; `tests/INDEX.md`。

**Docs and compatibility impact:** `docs/development/tracing/coverage-v2.md`; `docs/development/tracing/agent.md`; `COMPATIBILITY.md`；逐文件核内容与源码行为对应，`tests/INDEX.md` 只在本卡精确写集确有索引变化时同步。无新增网站命令行为；若实施引入公开差异，先按 ER-06a 修本卡写集再编辑。

**Rollback mode:** forward-only；恢复时签名+DCO 新 main 修复，既有 v1 claim/artifact/digest 不回写或重解释。

**Migration and rollback:** 无 DDL；Repository claim 仅用既有 versioned 键。若实现证明 DDL 必需，停止、加 GC-13 角色隔离迁移卡与旧/新读端门，再修本卡写集和 G-03/G-04。

**Security and privacy:** 继承 GC-COV2-02/03；错误或不可信版本必须 zero-success、不得向 ordinary/typed/log/error 导出候选内容；Session 例只在本机 ignored 执行。

**Performance budget:** 继承 GC-COV2-05；identity map 的 v1+v2 总量仍限 4096 条/1 MiB，任何新增解析在分配前有界；变更 hot path 时以具名门证明无无界扫描。

**Estimated scope:** M（落点/生产文件计数：8/8, 2/8, 2, 2; M）。

**Version increment:** N/A；此卡不发版。

**Release boundary:** family child of REL-COV2-01（19/20）；用户已授权逐卡 reviewed signed+DCO 直推 main，但不写 R、不 tag、不调用 release。

**C/D coverage from:** COV2-REL-01；发布点的 exact-source C/D 绿后才能 `done/complete`。

**Granularity:** `type=implementation; axis=候选 live Session 权限防火墙; recovery=forward-only; complete=yes; self-contained=yes; AC=8/8; VER=2/8; landing=2; prod-files=2; scope=M; deps=COV2-12; writeset=串行于前驱; release=family child; split-from=COV2-12; exception=N/A`

### Task COV2-18：候选 import Session 权限防火墙

**Task type:** implementation

**Lifecycle / Acceptance:** pending / 空

**Description:** proof/state-bearing candidate import Session firewall。本卡唯一行为轴、恢复和粒度见下方 Granularity；不得以 synthetic/native 互换验收。

**Out of scope:** 只负责 import 路径候选隔离；live 由 COV2-15；不铸造假 native 权限。

**Current evidence:** src/internal/ai/agent_import.rs; tests/agent_opencode_bridge_test.rs（基线 `dcc468b`；ER-02 开工刷新行号与实际源码）。

**Acceptance criteria:**

- [ ] AC1 the trusted macOS fake exporter actually executes through historical import with a proof/state-bearing OpenCode part
- [ ] AC2 import classification is partial
- [ ] AC3 candidate canary is absent from ordinary plaintext
- [ ] AC4 it is absent from typed plaintext
- [ ] AC5 it is absent from persisted import/checkpoint objects
- [ ] AC6 it is absent from logs and public errors
- [ ] AC7 no native issuer is minted
- [ ] AC8 no opaque encrypted artifact is minted.

**Verification:**

- [ ] VER1→AC1–8: **macOS manual** `source .env.test && cargo test --features test-live-agent --test agent_opencode_bridge_test cov2_macos_fake_exporter_candidate_import_partial_no_plaintext -- --ignored --exact --nocapture --test-threads=1` (1/1; actual import route and each sink asserted).
- [ ] VER2→AC1: `source .env.test && cargo test --test compat_agent_architecture_guard cov2_candidate_import_session_case_is_ignored_and_non_skippable -- --exact` (1/1). Zero tests, early return or prerequisite skip is UNRUN. This is a separate recovery axis: import cursor and object writes can fail while live remains safe; forward-fix only the import owner, then rerun COV2-15 and final audit on the same source.

**Full-suite trigger:** family child 不自行发布；按 ER-13 对最终改动重新判定 T-1/T-4/T-6。若命中 shared single-source helper、测试基建或 focused 信号不足，则在本卡 A/B 阶段运行双 env 全量 Nextest；无论是否命中，COV2-REL-01 T-2 均须在 bumped 树上重跑。

**Dependencies:** COV2-15；消费前驱实际最终 A/B、适用 focused/full、fmt/strict、单 Claude PASS 与 signed+DCO main 的 source-qualified 文件 SHA/mode；不以其尚待 C/D 的 `locally-accepted` 冒称 complete。

**Deliverables:** 无独立文件；本卡实现/测试/文档随唯一发布点交付；每卡的计数、源码 SHA、hash/mode、review/门结果写在本计划正文，不建证据文件。

**Implementation write set:** I-core = `docs/development/plan/plan-20261013-coverage-schema.md`; `docs/development/plan/plan-status.md`; `docs/development/tracing/coverage-v2.md`; `docs/development/tracing/agent.md`; `COMPATIBILITY.md`；本卡新增精确路径 = `src/internal/ai/agent_import.rs`; `src/internal/ai/observed_agents/opencode_reasoning.rs`; `tests/agent_opencode_bridge_test.rs`; `tests/compat/agent_architecture_guard.rs`; `tests/INDEX.md`。无 glob 默示写权。

**Release write set:** N/A；C/D 继承 COV2-REL-01。

**Files likely touched:** `docs/development/plan/plan-20261013-coverage-schema.md`; `docs/development/plan/plan-status.md`; `docs/development/tracing/coverage-v2.md`; `docs/development/tracing/agent.md`; `COMPATIBILITY.md`; `src/internal/ai/agent_import.rs`; `src/internal/ai/observed_agents/opencode_reasoning.rs`; `tests/agent_opencode_bridge_test.rs`; `tests/compat/agent_architecture_guard.rs`; `tests/INDEX.md`。

**Docs and compatibility impact:** `docs/development/tracing/coverage-v2.md`; `docs/development/tracing/agent.md`; `COMPATIBILITY.md`；逐文件核内容与源码行为对应，`tests/INDEX.md` 只在本卡精确写集确有索引变化时同步。无新增网站命令行为；若实施引入公开差异，先按 ER-06a 修本卡写集再编辑。

**Rollback mode:** forward-only；恢复时签名+DCO 新 main 修复，既有 v1 claim/artifact/digest 不回写或重解释。

**Migration and rollback:** 无 DDL；Repository claim 仅用既有 versioned 键。若实现证明 DDL 必需，停止、加 GC-13 角色隔离迁移卡与旧/新读端门，再修本卡写集和 G-03/G-04。

**Security and privacy:** 继承 GC-COV2-02/03；错误或不可信版本必须 zero-success、不得向 ordinary/typed/log/error 导出候选内容；Session 例只在本机 ignored 执行。

**Performance budget:** 继承 GC-COV2-05；identity map 的 v1+v2 总量仍限 4096 条/1 MiB，任何新增解析在分配前有界；变更 hot path 时以具名门证明无无界扫描。

**Estimated scope:** M（落点/生产文件计数：8/8, 2/8, 2, 2; M）。

**Version increment:** N/A；此卡不发版。

**Release boundary:** family child of REL-COV2-01（20/20）；用户已授权逐卡 reviewed signed+DCO 直推 main，但不写 R、不 tag、不调用 release。

**C/D coverage from:** COV2-REL-01；发布点的 exact-source C/D 绿后才能 `done/complete`。

**Granularity:** `type=implementation; axis=候选 import Session 权限防火墙; recovery=forward-only; complete=yes; self-contained=yes; AC=8/8; VER=2/8; landing=2; prod-files=2; scope=M; deps=COV2-15; writeset=串行于前驱; release=family child; split-from=COV2-15; exception=N/A`

### Task COV2-19：sealed-native v2 identity 生产提交

**Task type:** implementation

**Lifecycle / Acceptance:** pending / 空

**Description:** 自 COV2-03 拆出。经既有 native proof/binding 验证的 `NativeProven` payload 借用口构造 version-keyed v2 identity plan；把该 plan、版本化 claim sidecar、私有 identity-v2 record/map 与 ref/catalog/claim 提交绑定为一次真实 HistoryManager 事务。v1 原路径和冻结字节保持。

**Out of scope:** 不开放 arbitrary bytes/普通 read context 的生产写权限；不修改 native issuer/locator/注册，也不把 synthetic proof 判为真实 Source A；不实现 pending artifact SQL/codec/replay；无 DDL。

**Current evidence:** `src/internal/ai/capture/checkpoint.rs:494,576–665,1130–1195`（既有 source binding、v1-only `ReasoningCoveragePlan`、request）；`src/internal/ai/coverage_gate.rs:167–203,373–390,2339–2520`（versioned claim sidecar 与 v1-only identity companion）；COV2-14 dual-map codec/reader，COV2-03 versioned marker/ref fence。开工按 ER-02 在最终 main 刷新行号、hash/mode。

**Acceptance criteria:**

- [ ] AC1：v2 identity production plan 仅能由 `CheckpointRedactedPayload::reasoning=NativeProven` 的封闭借用构造，Ordinary/Unmapped/Partial 不授予生产写权。
- [ ] AC2：从 proof 借用的既有 `VerifiedCaptureBinding` 与当前请求的封闭 capture context 等值；不另造可绕过它的字段级权限判断。
- [ ] AC3：source-derived versioned claim 集合与传入 `VersionedLiveClaimCommitPlan` 通过同一 canonical 集合等值判断。
- [ ] AC4：生产 v2 identity record 仅由 COV2-07/14 已审的封闭 v2 record 构造/认证 API 产生。
- [ ] AC5：生产 v2 identity map 仅由 COV2-14 已审的封闭 `identity-v2` map carrier 构造/认证 API 写入。
- [ ] AC6：同一 logical key 的 v1、v2 identity record 在同一个已认证会话中共存。
- [ ] AC7：双 map 合计越过 entry 或 encoded-byte 上限的生产提交在 catalog/claim/ref 可见变更前被拒绝，零持久化改动。
- [ ] AC8：真实 `TracesCheckpointStore.write`/HistoryManager ref-CAS 注入 identity-v2 SQL fault 后，ref、catalog、revision、两版 claim/map、object-index 同事务原子回滚且无成功 ACK。

**独立谓词计数：** 本卡 8 个新集成结果，前提是 AC2/3/4/5/7 只消费 COV2-02/07/14 已验收的封闭值对象、codec/MAC 和共同容量 API；AC7 的 guard 调用只是达成单一越界前拒绝结果的机制，不复制其内部独立判据。VER 逐字段/域变异仍须证明真实 writer 调到该 API，v1 frozen、unknown key、同版本拒绝仍继承原门。若实现需要新字段级分支、新 codec/MAC 或独立容量判据，则此 8/8 不成立，须在开工前按 G-03 拆出后继子卡并同步 G-09，不得合句或豁免。

**Verification:**

- [ ] VER1→AC1/2：`source .env.test && cargo test --lib internal::ai::capture::checkpoint::tests::cov2_v2_native_plan_rejects_unbound_source -- --exact`（1/1；native gate 与封闭 binding 的 session/provider/source 逐字段变异）。
- [ ] VER2→AC3：`source .env.test && cargo test --lib internal::ai::coverage_gate::tests::cov2_native_v2_claim_tuple_binding -- --exact`（1/1；真实 sidecar，有序集合、version/key/digest/completeness 漂移）。
- [ ] VER3→AC4/5/6：`source .env.test && cargo test --lib internal::ai::coverage_gate::tests::cov2_v2_identity_domains_and_same_key -- --exact`（1/1；经生产 companion 写/读 record/map，两个 MAC domain 各自断言；直接调用最终 map reader，分别将 v2 whole-map MAC 篡改及 v1/v2 map 存储值互换后要求 fail closed、零 claim/ref 改动，再还原并读回；保留 v1 frozen oracle。这补 COV2-14 Claude P3 读端回归，不算 Source A/native 正例）。
- [ ] VER4→AC7：`source .env.test && cargo test --lib internal::ai::coverage_gate::tests::cov2_v2_identity_aggregate_cap -- --exact`（1/1；真实 writer 必须调用已审 aggregate-cap guard，分别以 entry/encoded-byte 越界验证提交前拒绝且 ref/catalog/claim/map 均无可见变更，不以 synthetic map seed 代替）。
- [ ] VER5→AC8：`source .env.test && cargo test --lib internal::ai::coverage_gate::tests::cov2_native_v2_history_fault_atomicity -- --exact`（1/1；实际 HistoryManager/ref-CAS 注入 identity-v2 SQL fault，逐表核 ref/catalog/revision/两版 claim-map/object-index 原子回滚与零成功 ACK；后续 retry 属 ER-04 回归，不得仅直调 `TracesTxnExtra::apply`）。

**Mandatory ER-04 marker recovery（不增加本卡 G-03 AC/VER）：** 在 COV2-19 最终源码上重跑 COV2-03 新 VER5 `source .env.test && cargo test --lib internal::ai::traces::tests::cov2_completed_terminal_receipt_runtime_branch -- --exact`（1/1），并在本卡 VER5 故障场景确认预注册 marker 不属于 ref-CAS 回滚集合、仍可由恢复/重试路径读取和处理，不能出现假成功 ACK；这是前驱 marker/replay 回归，不把第二谓词藏进 AC8。两个实际结果均未运行，不计 PASS。

**Full-suite trigger:** 以最终 diff 按 ER-13 判定；先运行本卡 5 个 focused selector、冻结 v1 golden/contract oracle、nightly fmt、strict Clippy 与唯一 Claude Code PASS；若触发 shared helper/测试基建/无法归属 focused 失败则本卡双 env 全量，最终 COV2-REL-01 bumped exact-tree 全量仍无条件运行。上述新 selector 当前未实现，不能记 PASS。

**Dependencies:** COV2-03；消费最终 A/B、适用 focused/full、fmt/strict、唯一 Claude PASS 与 signed+DCO main SHA/hash/mode；不把继承 C/D 的 locally-accepted 冒称 complete。

**Deliverables:** 无独立文件；实现、测试、文档与逐文件 SHA/mode、review/门结果写回本计划正文，随唯一发布点交付。

**Implementation write set:** I-core 精确五文件：`docs/development/plan/plan-20261013-coverage-schema.md`; `docs/development/plan/plan-status.md`; `docs/development/tracing/coverage-v2.md`; `docs/development/tracing/agent.md`; `COMPATIBILITY.md`；本卡新增精确路径：`src/internal/ai/capture/checkpoint.rs`; `src/internal/ai/coverage_gate.rs`。测试写在上述两个源码文件的现有 test module，不授权 glob、`history.rs` 或 `sql/**`。

**Release write set:** N/A；C/D 继承 COV2-REL-01。

**Files likely touched:** 上述七个 I 文件。

**Docs and compatibility impact:** `docs/development/tracing/coverage-v2.md`; `docs/development/tracing/agent.md`; `COMPATIBILITY.md` 逐文件陈述 sealed-native v2 identity、v1 冻结和 Source A 尚未证明；若新增公开差异，先按 ER-06a 修写集。无新 `--test` target，`tests/INDEX.md` 无写权。

**Rollback mode:** forward-only；失败不重写 v1 或既有私有 map，签名+DCO 新 main 修复并重跑下游。

**Migration and rollback:** 无 DDL；仅使用现有 versioned claim 与 COV2-14 双 map。若需要数据库 schema/migration，先加角色隔离迁移卡及旧/新二进制门、修写集和 G-03/G-04。

**Security and privacy:** v2 writer 不接受任意来源或 read context；native field 正文不得进入 ordinary/typed/log/error/计划证据；synthetic sealed fixture 仅证明交易正确性，不证明已注册 OpenCode Source A。

**Performance budget:** 沿用 COV2-14 双 map aggregate 4096 entries/1 MiB，bounded read before allocation；变更 hot path 时验证无无界扫描。

**Estimated scope:** M（AC=8/8、VER=5/8、landing=2、production files=2）。

**Version increment:** N/A；此卡不发版。

**Release boundary:** family child of REL-COV2-01（7/20）；用户已授权逐卡 reviewed signed+DCO 直推 main，但不写 R、不 tag。

**C/D coverage from:** COV2-REL-01；其真实 exact-source C/D 通过后本卡才可 done/complete。

**Granularity:** `type=implementation; axis=sealed-native v2 identity 生产提交; recovery=forward-only; complete=yes; self-contained=yes; AC=8/8; VER=5/8; landing=2; prod-files=2; scope=M; deps=COV2-03; writeset=严格串行; release=family child; split-from=COV2-03; exception=N/A`

### Task COV2-20：版本化 live reservation 只读验领

**Task type:** implementation

**Lifecycle / Acceptance:** pending / 空（新增卡；下列 AC/VER 尚未执行）

**Description:** 自 COV2-03 拆出并经 COV2-19 接续，进一步拆出 COV2-21 的 retain 行为；本卡只提供 `verify_versioned_reserved_live_claims_with_conn(conn,scope,&VersionedLiveClaimCommitPlan)`。只读取 sealed 有序 (version,key,digest,completeness,owner,fence,previous_revision,state) 预留 tuple；使用调用方 `ConnectionTrait` 的事务视图，不更新 lease。v1 API 保持。

**Out of scope:** 不创建 pending artifact/envelope，不 retain、不改 DDL、native issuer/Source A，也不以第二连接看似成功代替调用方事务。

**Current evidence:** `src/internal/ai/coverage_gate.rs:167–203,215–287` 现有 versioned sidecar 与 v1 reservation helper；`pending.rs` 的真实 writer 本卡只读。 开工按 ER-02 在最终 signed main 刷新行号、SHA/mode；先前隔离草案未编译/验收，不作实现证据。

**Acceptance criteria:**

- [ ] AC1：用每个 claim 的实际 `(version,key)` SELECT 行。
- [ ] AC2：stored reservation 与 sealed canonical ownership tuple 精确等值才通过。
- [ ] AC3：同一 logical key 的 v1 与 v2 两版独立验领。
- [ ] AC4：租期过期而 owner/fence/revision 仍属原持有人时允许验领。
- [ ] AC5：旧 owner/fence/revision/digest/completeness/state 拒绝。
- [ ] AC6：成功与拒绝均不更改任何 claim 行。
- [ ] AC7：读取调用方同一 connection/transaction 内尚未提交的状态，不打开第二 pool transaction。

**独立谓词计数：** AC2/5 共用同一 canonical ownership tuple 的等值判断；逐字段变异是该判断的测试向量，不新造分支。现存 v1 reservation 回归是额外 mandatory ER-04 A 门：`source .env.test && cargo test --lib internal::ai::coverage_gate::tests::live_takeover_increments_fence_and_blocks_stale_commit -- --exact`（1/1；当前 UNRUN，0 matched/skip 非 PASS）；不能用本卡新 selector 代替。

**Verification:**

- [ ] VER1：AC1/2/5：`source .env.test && cargo test --lib internal::ai::coverage_gate::tests::cov2_versioned_reserved_live_claims_verify_actual_tuple -- --exact`（真实 DB，1/1）。
- [ ] VER2：AC3/4：`source .env.test && cargo test --lib internal::ai::coverage_gate::tests::cov2_versioned_reserved_live_claims_verify_same_key_and_expired_lease -- --exact`（1/1）。
- [ ] VER3：AC6/7：`source .env.test && cargo test --lib internal::ai::coverage_gate::tests::cov2_versioned_reserved_live_claims_verify_is_read_only_in_caller_transaction -- --exact`（修改调用方未提交行，helper 看到该值；拒绝后行不变；1/1）。

**Full-suite trigger:** 按 ER-13 以最终 diff 判定；先跑本卡 3 个具名 focused selector、相关冻结 v1 回归、nightly fmt、strict Clippy 与唯一 Claude Code 最终字节 PASS；触发 shared helper/测试基建/无法归属 focused 故障时本卡双 env full。COV2-REL-01 bumped exact-tree full 无条件运行。0 matched、skip、synthetic 或未执行均非 PASS。

**Dependencies:** COV2-19；消费前驱真实最终 A/B、适用 focused/full、fmt/strict、唯一 Claude PASS 与 signed+DCO main SHA/hash/mode；不把继承 C/D 的 locally-accepted 冒称 complete。

**Deliverables:** 无独立文件；API、测试、兼容文档、逐文件 SHA/mode 和 review/验收事实写回本计划正文，随唯一发布点交付。

**Implementation write set:** I-core 精确五文件：`docs/development/plan/plan-20261013-coverage-schema.md`; `docs/development/plan/plan-status.md`; `docs/development/tracing/coverage-v2.md`; `docs/development/tracing/agent.md`; `COMPATIBILITY.md`；本卡新增精确路径：`src/internal/ai/coverage_gate.rs`。现有测试模块可写；未列路径包括 `sql/**`、冻结 decoder、parent 计划均为只读，需另行 G-09/I 修订才能扩权。

**Release write set:** N/A；C/D 继承 COV2-REL-01。

**Files likely touched:** I-core 上述五文件；`src/internal/ai/coverage_gate.rs`。

**Docs and compatibility impact:** I-core 的 coverage-v2、agent、COMPATIBILITY 只陈述本卡已交付的能力，不提前宣称下游 artifact/replay/Source A 已完成。无新 `--test` target；`tests/INDEX.md` 无写权。

**Rollback mode:** forward-only；失败保持旧 artifact/claim，签名+DCO 新 main 前滚并重跑下游。

**Migration and rollback:** 无 DDL；Repository claim 使用既有 versioned 键。若实际需迁移，先加角色隔离 migration 卡、旧/新 binary 门并修 G-03/G-04/I。

**Security and privacy:** 继承 GC-COV2-02/03；所有错误 fail closed、零假 ACK、零候选明文 sink；真实 native Source A 仍在父计划。

**Performance budget:** 使用已审 claim/map 上限，解析/查询前有界；不得新增无界扫描、无时限重试或私开的第二连接。

**Estimated scope:** M（AC=7/8、VER=3/8、landing=1、production files=1）。

**Version increment:** N/A；此卡不发版。

**Release boundary:** family child of REL-COV2-01（8/20）；逐卡单 Claude PASS 后 signed+DCO 直推 main，不写 R、不 tag。

**C/D coverage from:** COV2-REL-01；真实 exact-source C/D 通过后本卡才可 done/complete。

**Granularity:** `type=implementation; axis=版本化 live reservation 只读验领; recovery=forward-only; complete=yes; self-contained=yes; AC=7/8; VER=3/8; landing=1; prod-files=1; scope=M; deps=COV2-19; writeset=严格串行; release=family child; split-from=COV2-03; exception=N/A`

### Task COV2-21：调用方事务内版本化 reservation retain

**Task type:** implementation

**Lifecycle / Acceptance:** pending / 空（新增卡；下列 AC/VER 尚未执行）

**Description:** 自 COV2-20 拆出，提供 `retain_versioned_reserved_live_claims_for_artifact_with_conn(conn,scope,&VersionedLiveClaimCommitPlan)`。使用同一 sealed canonical tuple 和调用方 SQLite transaction 更新双版本 lease；COV2-20 已独立验证只读验领。

**Out of scope:** 不产生 artifact/envelope、persist receipt、旧读端或 replay，不调用 native issuer；不能用模拟 SQL probe 声称真实 artifact 写故障已验收。

**Current evidence:** `src/internal/ai/coverage_gate.rs` 的 versioned sidecar、COV2-20 只读验领和既有 v1 retain；`pending.rs`/`pending_payload.rs` 均只读。 开工按 ER-02 在最终 signed main 刷新行号、SHA/mode；先前隔离草案未编译/验收，不作实现证据。

**Acceptance criteria:**

- [ ] AC1：UPDATE 以每个 claim 实际 `(version,key)` 寻址。
- [ ] AC2：UPDATE 要求 sealed owner/fence/revision/digest/completeness/state tuple 精确匹配。
- [ ] AC3：同一调用方事务将同 logical key 的 v1 与 v2 两行保留到 artifact lease。
- [ ] AC4：仅 lease 过期不撤销原 owner 的合法保留。
- [ ] AC5：陈旧 ownership tuple 拒绝且不改变任何行。
- [ ] AC6：第二版本 UPDATE 的真实 SQL fault 回滚第一行 lease 更新。
- [ ] AC7：调用方同一事务中的后续 SQL fault 回滚两行 lease，helper 不提前 commit。

**独立谓词计数：** AC2/5 消费同一个 sealed tuple 比较；如代码新增独立安全判断则先 G-03 重计。现存 v1 retain 行为为额外 mandatory ER-04 门：`source .env.test && cargo test --lib internal::ai::coverage_gate::tests::artifact_retained_claims_survive_lease_expiry_until_replay_commit -- --exact`（1/1；当前 UNRUN，0 matched/skip 非 PASS）；既有 v1 字节冻结门仍按 ER-04 继承。AC6/7 的事务探针不替代 COV2-22 真实 chunk/header/alias writer 故障。

**Verification:**

- [ ] VER1：AC1–4：`source .env.test && cargo test --lib internal::ai::coverage_gate::tests::cov2_versioned_reserved_live_claims_retain_both_versions -- --exact`（1/1）。
- [ ] VER2：AC5：`source .env.test && cargo test --lib internal::ai::coverage_gate::tests::cov2_versioned_reserved_live_claims_retain_stale_tuple_rejects -- --exact`（1/1）。
- [ ] VER3：AC6：`source .env.test && cargo test --lib internal::ai::coverage_gate::tests::cov2_versioned_reserved_live_claims_second_row_fault_rolls_back -- --exact`（真实 SQLite trigger，逐行核回滚；1/1）。
- [ ] VER4：AC7：`source .env.test && cargo test --lib internal::ai::coverage_gate::tests::cov2_versioned_reserved_live_claims_later_sql_fault_rolls_back -- --exact`（调用方事务后续 SQL 失败，不造假 artifact receipt；1/1）。

**Full-suite trigger:** 按 ER-13 以最终 diff 判定；先跑本卡 4 个具名 focused selector、相关冻结 v1 回归、nightly fmt、strict Clippy 与唯一 Claude Code 最终字节 PASS；触发 shared helper/测试基建/无法归属 focused 故障时本卡双 env full。COV2-REL-01 bumped exact-tree full 无条件运行。0 matched、skip、synthetic 或未执行均非 PASS。

**Dependencies:** COV2-20；消费前驱真实最终 A/B、适用 focused/full、fmt/strict、唯一 Claude PASS 与 signed+DCO main SHA/hash/mode；不把继承 C/D 的 locally-accepted 冒称 complete。

**Deliverables:** 无独立文件；API、测试、兼容文档、逐文件 SHA/mode 和 review/验收事实写回本计划正文，随唯一发布点交付。

**Implementation write set:** I-core 精确五文件：`docs/development/plan/plan-20261013-coverage-schema.md`; `docs/development/plan/plan-status.md`; `docs/development/tracing/coverage-v2.md`; `docs/development/tracing/agent.md`; `COMPATIBILITY.md`；本卡新增精确路径：`src/internal/ai/coverage_gate.rs`。现有测试模块可写；未列路径包括 `sql/**`、冻结 decoder、parent 计划均为只读，需另行 G-09/I 修订才能扩权。

**Release write set:** N/A；C/D 继承 COV2-REL-01。

**Files likely touched:** I-core 上述五文件；`src/internal/ai/coverage_gate.rs`。

**Docs and compatibility impact:** I-core 的 coverage-v2、agent、COMPATIBILITY 只陈述本卡已交付的能力，不提前宣称下游 artifact/replay/Source A 已完成。无新 `--test` target；`tests/INDEX.md` 无写权。

**Rollback mode:** forward-only；失败保持旧 artifact/claim，签名+DCO 新 main 前滚并重跑下游。

**Migration and rollback:** 无 DDL；Repository claim 使用既有 versioned 键。若实际需迁移，先加角色隔离 migration 卡、旧/新 binary 门并修 G-03/G-04/I。

**Security and privacy:** 继承 GC-COV2-02/03；所有错误 fail closed、零假 ACK、零候选明文 sink；真实 native Source A 仍在父计划。

**Performance budget:** 使用已审 claim/map 上限，解析/查询前有界；不得新增无界扫描、无时限重试或私开的第二连接。

**Estimated scope:** M（AC=7/8、VER=4/8、landing=1、production files=1）。

**Version increment:** N/A；此卡不发版。

**Release boundary:** family child of REL-COV2-01（9/20）；逐卡单 Claude PASS 后 signed+DCO 直推 main，不写 R、不 tag。

**C/D coverage from:** COV2-REL-01；真实 exact-source C/D 通过后本卡才可 done/complete。

**Granularity:** `type=implementation; axis=调用方事务内版本化 reservation retain; recovery=forward-only; complete=yes; self-contained=yes; AC=7/8; VER=4/8; landing=1; prod-files=1; scope=M; deps=COV2-20; writeset=严格串行; release=family child; split-from=COV2-20; exception=N/A`

### Task COV2-22：真实 pending artifact 事务与冻结旧读端

**Task type:** implementation

**Lifecycle / Acceptance:** pending / 空（新增卡；下列 AC/VER 尚未执行）

**Description:** 自 COV2-04 拆出，消费其 sealed codec 和 COV2-20/21 事务 API，真实执行 `SealedVersionedPendingArtifact::seal→persist(&DatabaseTransaction)→commit`；验证同事务双版本 lease/artifact、编译冻结旧读端拒绝与真实 writer 故障回滚。

**Out of scope:** 不改冻结旧 decoder、`pending_payload.rs`、`key.rs`、`coverage_gate.rs`、history/catalog/claim SQL schema，也不开放 native issuer/Source A；COV2-08 才切换 replay。若 frozen guard 必须改，先把 `tests/compat/agent_architecture_guard.rs` 精确加 I 并复审。

**Current evidence:** `src/internal/ai/capture/pending.rs:340–370` 现有 v1 writer 和 COV2-04 sealed codec；COV2-20/21 versioned verify/retain 均必须先在 signed main 接收。 开工按 ER-02 在最终 signed main 刷新行号、SHA/mode；先前隔离草案未编译/验收，不作实现证据。

**Acceptance criteria:**

- [ ] AC1：真实 sealed-v2 persist 在同一个 caller `DatabaseTransaction` 调用 COV2-20 verify 与 COV2-21 retain。
- [ ] AC2：chunk/header/alias 与 v1/v2 两版 lease 原子提交。
- [ ] AC3：全 v1 claim set 的 v2 artifact writer 在任何写前拒绝。
- [ ] AC4：编译冻结的 pre-v2 reader 认证后拒绝真实已提交的 v2 artifact。
- [ ] AC5：上述旧读端拒绝使 artifact/claims/ref/marker/receipt 字节和行均不变。
- [ ] AC6：新 versioned loader 认证同一已提交 artifact 并恢复精确 tuple。
- [ ] AC7：在 retain 之后的真实 chunk/header/alias SQL fault 回滚两版 lease、artifact/alias 行且零成功 ACK。
- [ ] AC8：OG-29 version-keyed association 存在前，native reasoning 双版本同 logical key 在 pending 写前 fail closed。

**旧读端与 Source A：** `source .env.test && cargo test --test compat_agent_architecture_guard reasoning_pending_pre_v2_reader_oracle_is_immutable -- --exact` 是本卡六条 VER 之外另一个 mandatory ER-04 命令，须额外实际 1/1；冻结 decoder 原字节不变。手插行或重签 synthetic envelope 不算 AC1/4/7。COV2-22 只做 native same-key 负例，正例由父 OG-29 在真实 Source A/issuer 约束下交付。

**Verification:**

- [ ] VER1：AC1/2/4/5：`source .env.test && cargo test --lib internal::ai::capture::pending::tests::cov2_real_v2_persist_and_frozen_old_reader_reject -- --exact`（真实 seal→persist→commit，编译冻结 decoder，全 before/after；1/1）。
- [ ] VER2：AC6：`source .env.test && cargo test --lib internal::ai::capture::pending::tests::cov2_real_v2_artifact_new_reader_roundtrip -- --exact`（1/1）。
- [ ] VER3：AC3：`source .env.test && cargo test --lib internal::ai::capture::pending::tests::cov2_real_v2_artifact_rejects_all_v1_plan -- --exact`（无写入，1/1）。
- [ ] VER4：AC7：`source .env.test && cargo test --lib internal::ai::capture::pending::tests::cov2_real_v2_artifact_later_sql_fault_rolls_back -- --exact`（两版 retain 之后真 chunk/header/alias trigger，逐行和逐字节比较；1/1）。
- [ ] VER5：AC8：`source .env.test && cargo test --lib internal::ai::capture::pending::tests::cov2_native_same_key_pending_fails_closed -- --exact`（synthetic 负例而非 native Source A 正例；1/1）。
- [ ] VER6：额外 mandatory ER-04：`source .env.test && cargo test --lib internal::ai::capture::pending::tests::cov2_v1_pending_persisted_bytes_and_mac_frozen -- --exact`（原始基线字节/MAC，1/1）。

**进一步写集门：** 若现有 compat guard 必须更新断言，先明确把 `tests/compat/agent_architecture_guard.rs` 加入本卡精确 I，并按 ER-13/单 Claude 重审；否则该文件只读。

**Full-suite trigger:** 按 ER-13 以最终 diff 判定；先跑本卡 6 个具名 focused selector及上述额外 compat guard 1/1、相关冻结 v1 回归、nightly fmt、strict Clippy 与唯一 Claude Code 最终字节 PASS；触发 shared helper/测试基建/无法归属 focused 故障时本卡双 env full。COV2-REL-01 bumped exact-tree full 无条件运行。0 matched、skip、synthetic 或未执行均非 PASS。

**Dependencies:** COV2-04；消费前驱真实最终 A/B、适用 focused/full、fmt/strict、唯一 Claude PASS 与 signed+DCO main SHA/hash/mode；不把继承 C/D 的 locally-accepted 冒称 complete。

**Deliverables:** 无独立文件；API、测试、兼容文档、逐文件 SHA/mode 和 review/验收事实写回本计划正文，随唯一发布点交付。

**Implementation write set:** I-core 精确五文件：`docs/development/plan/plan-20261013-coverage-schema.md`; `docs/development/plan/plan-status.md`; `docs/development/tracing/coverage-v2.md`; `docs/development/tracing/agent.md`; `COMPATIBILITY.md`；本卡新增精确路径：`src/internal/ai/capture/pending.rs`。现有测试模块可写；未列路径包括 `sql/**`、冻结 decoder、parent 计划均为只读，需另行 G-09/I 修订才能扩权。

**Release write set:** N/A；C/D 继承 COV2-REL-01。

**Files likely touched:** I-core 上述五文件；`src/internal/ai/capture/pending.rs`。

**Docs and compatibility impact:** I-core 的 coverage-v2、agent、COMPATIBILITY 只陈述本卡已交付的能力，不提前宣称下游 artifact/replay/Source A 已完成。无新 `--test` target；`tests/INDEX.md` 无写权。

**Rollback mode:** forward-only；失败保持旧 artifact/claim，签名+DCO 新 main 前滚并重跑下游。

**Migration and rollback:** 无 DDL；Repository claim 使用既有 versioned 键。若实际需迁移，先加角色隔离 migration 卡、旧/新 binary 门并修 G-03/G-04/I。

**Security and privacy:** 继承 GC-COV2-02/03；所有错误 fail closed、零假 ACK、零候选明文 sink；真实 native Source A 仍在父计划。

**Performance budget:** 使用已审 claim/map 上限，解析/查询前有界；不得新增无界扫描、无时限重试或私开的第二连接。

**Estimated scope:** M（AC=8/8、VER=6/8、landing=1、production files=1）。

**Version increment:** N/A；此卡不发版。

**Release boundary:** family child of REL-COV2-01（11/20）；逐卡单 Claude PASS 后 signed+DCO 直推 main，不写 R、不 tag。

**C/D coverage from:** COV2-REL-01；真实 exact-source C/D 通过后本卡才可 done/complete。

**Granularity:** `type=implementation; axis=真实 pending artifact 事务与冻结旧读端; recovery=forward-only; complete=yes; self-contained=yes; AC=8/8; VER=6/8; landing=1; prod-files=1; scope=M; deps=COV2-04; writeset=严格串行; release=family child; split-from=COV2-04; exception=N/A`

### Task COV2-16：既有 bridge/pipeline/OG-26 12 例本机审计

**Task type:** audit

**Lifecycle / Acceptance:** pending / 空

**Description:** 12-case bridge/pipeline/OG-26 macOS acceptance inventory。本卡唯一行为轴、恢复和粒度见下方 Granularity；不得以 synthetic/native 互换验收。

**Out of scope:** 只读审计，不修改生产/测试；失败回到 COV2-12/15/18 所属实现卡前滚。

**Current evidence:** tests/agent_opencode_bridge_test.rs; src/internal/ai/capture/live_pipeline_tests.rs（旧21例基线 `dcc468b`）；OG-26 signed main `734b6f03884d3aa38400acbbfd446b557e642d94` 新增 `tests/agent_checkpoint_redaction_test.rs:1326-1331` 的 `og26_candidate_has_zero_normal_sink_hits`（ER-02 开工刷新行号与实际签名源码）。

**Acceptance criteria:**

- [ ] AC1 `opencode_historical_import_e2e_uses_trusted_export_bytes`
- [ ] AC2 `opencode_export_whole_session_idempotent`
- [ ] AC3 `opencode_export_durable_snapshot_uses_repository_keyed_hmac`
- [ ] AC4 `opencode_export_rejects_wrong_native_session_without_export_claim`
- [ ] AC5 `opencode_registered_failure_releases_dirty_job_claim_and_marker`
- [ ] AC6 `opencode_export_plaintext_never_in_persist_or_logs`
- [ ] AC7 `opencode_export_binary_trust_revalidates`
- [ ] AC8 `opencode_export_oversize_session_degrades`
- [ ] AC9 `opencode_export_macos_requires_trust_before_spawn`
- [ ] AC10 `opencode_extraction_failure_continues_checkpoint` or its source-reviewed, scenario-preserving renamed successor
- [ ] AC11 `export_stage_releases_runner_lease_on_every_exit`.
- [ ] AC12 no case silently returns on absent bwrap/Seatbelt or reports 0 tests/skip as PASS.
- [ ] AC13 `og26_candidate_has_zero_normal_sink_hits`.

**Verification:**

- [ ] (new) VER1→AC1–9/12: **macOS manual** `source .env.test && source .env.live-test && cargo test --features test-live-agent --test agent_opencode_bridge_test opencode_ -- --ignored --nocapture --test-threads=1` (actual 9/9; the `opencode_` selector excludes new `cov2_` tests).
- [ ] VER2→AC10/12: **macOS manual** `source .env.test && source .env.live-test && cargo test --lib internal::ai::capture::live_pipeline::tests::export_runner_lease::end_to_end::opencode_extraction_failure_continues_checkpoint -- --ignored --exact --nocapture --test-threads=1` (1/1; if COV2-12 renamed it, use the reviewed updated exact selector and record old→new mapping).
- [ ] VER3→AC11/12: **macOS manual** `source .env.test && source .env.live-test && cargo test --lib internal::ai::capture::live_pipeline::tests::export_runner_lease::end_to_end::export_stage_releases_runner_lease_on_every_exit -- --ignored --exact --nocapture --test-threads=1` (1/1).
- [ ] VER4→AC12: compare this card's three-file source inventory and each test body's non-skip assertion path with its actual report; a target reporting PASS after `BridgeRepo::init` returns `None` is FAIL, regardless of Cargo exit code. The bridge target's current `BridgeRepo::init` bwrap early return (`tests/agent_opencode_bridge_test.rs:38-45`) must be removed for macOS by COV2-12; the OG-26 target's Required Seatbelt prerequisite must assert/fail rather than skip. Linux bwrap-specific functional acceptance remains the user's later gate, while required security CI remains. Any failure returns to the named implementation owner for reviewed forward-fix and this entire final-source audit reruns; this audit cannot edit code.
- [ ] VER5→AC12/13: **macOS manual** `source .env.test && source .env.live-test && cargo test --test agent_checkpoint_redaction_test og26_candidate_has_zero_normal_sink_hits -- --ignored --exact --nocapture --test-threads=1` (actual 1/1; confirm exporter execution and zero ordinary sink assertions on final behavior SHA; OG-26 pre-handoff VER4 historical PASS does not replace this audit).

**Full-suite trigger:** N/A：no-release audit 不自行执行 C 组；卡内本机 manual matrix 与文档 B 门必须通过，C/D 继承 COV2-REL-01。

**Dependencies:** COV2-18；消费前驱实际最终 A/B、适用 focused/full、fmt/strict、单 Claude PASS 与 signed+DCO main 的 source-qualified 文件 SHA/mode；不以其尚待 C/D 的 `locally-accepted` 冒称 complete。

**Deliverables:** `docs/development/plan/plan-20261013-coverage-schema.md`; `docs/development/plan/plan-status.md`; `docs/development/tracing/coverage-v2.md`；每卡的计数、源码 SHA、hash/mode、review/门结果写在本计划正文，不建证据文件。

**Implementation write set:** I-audit = `docs/development/plan/plan-20261013-coverage-schema.md`; `docs/development/plan/plan-status.md`; `docs/development/tracing/coverage-v2.md`；本卡新增精确路径 = 无。无 glob 默示写权。

**Release write set:** N/A；C/D 继承 COV2-REL-01。

**Files likely touched:** `docs/development/plan/plan-20261013-coverage-schema.md`; `docs/development/plan/plan-status.md`; `docs/development/tracing/coverage-v2.md`。

**Docs and compatibility impact:** `docs/development/tracing/coverage-v2.md`；逐文件核内容与源码行为对应，`tests/INDEX.md` 只在本卡精确写集确有索引变化时同步。无新增网站命令行为；若实施引入公开差异，先按 ER-06a 修本卡写集再编辑。

**Rollback mode:** forward-only；恢复时签名+DCO 新 main 修复，既有 v1 claim/artifact/digest 不回写或重解释。

**Migration and rollback:** 无 DDL；Repository claim 仅用既有 versioned 键。若实现证明 DDL 必需，停止、加 GC-13 角色隔离迁移卡与旧/新读端门，再修本卡写集和 G-03/G-04。

**Security and privacy:** 继承 GC-COV2-02/03；错误或不可信版本必须 zero-success、不得向 ordinary/typed/log/error 导出候选内容；Session 例只在本机 ignored 执行。

**Performance budget:** 继承 GC-COV2-05；identity map 的 v1+v2 总量仍限 4096 条/1 MiB，任何新增解析在分配前有界；变更 hot path 时以具名门证明无无界扫描。

**Estimated scope:** S（落点/生产文件计数：13/20, 5/20, 0, 0; S）。

**Version increment:** N/A；此卡不发版。

**Release boundary:** no-release；证据随 REL-COV2-01 的 COV2-REL-01 交付；用户已授权逐卡 reviewed signed+DCO 直推 main，但不写 R、不 tag、不调用 release。

**C/D coverage from:** COV2-REL-01；发布点的 exact-source C/D 绿后才能 `done/complete`。

**Granularity:** `type=audit; axis=既有 bridge/pipeline/OG-26 12 例本机审计; recovery=forward-only; complete=yes; self-contained=yes; AC=13/20; VER=5/20; landing=0; prod-files=0; scope=S; deps=COV2-18; writeset=串行于前驱; release=no-release; split-from=N/A; exception=N/A`

### Task COV2-17：既有 lifecycle/crash/import/oracle 10 例本机审计

**Task type:** audit

**Lifecycle / Acceptance:** pending / 空

**Description:** 10-case lifecycle/crash/import/oracle macOS acceptance inventory。本卡唯一行为轴、恢复和粒度见下方 Granularity；不得以 synthetic/native 互换验收。

**Out of scope:** 只读审计，不修改生产/测试；失败回到所属实现卡前滚并重跑 COV2-16/17。

**Current evidence:** tests/agent_lifecycle_event_test.rs; tests/agent_hook_crash_test.rs; tests/agent_import_test.rs; src/internal/ai/capture/live_oracle_tests.rs（基线 `dcc468b`；ER-02 开工刷新行号与实际源码）。

**Acceptance criteria:**

- [ ] AC1 `hook_public_contract_byte_compat_matrix`
- [ ] AC2 `opencode_status_idle_maps_turn_end`
- [ ] AC3 `opencode_status_busy_retry_fails_closed`
- [ ] AC4 `opencode_message_updated_is_single_envelope_turn_start`
- [ ] AC5 `opencode_missing_legacy_prompt_fails_before_writes`
- [ ] AC6 `opencode_legacy_alias_transition_forwarded`
- [ ] AC7 `opencode_produced_frames_reach_real_ingress_and_survive_history_rollover`
- [ ] AC8 `capture_coordinator_native_replay_matrix`
- [ ] AC9 `agent_import_confirmation_precedes_opencode_export`
- [ ] AC10 `live_checkpoint_metadata_shape_is_stable`.
- [ ] AC11 no case is counted PASS after an early return, 0 selected tests, a missing OpenCode matrix row or a printed skip.
- [ ] AC12 a genuine visible-only OpenCode model observation is captured on the same final behavior source without issuer registration or live mint; this observation grants no Source A authority.

**Verification:**

- [ ] VER1→AC1–7/11: **macOS manual** `source .env.test && source .env.live-test && cargo test --test agent_lifecycle_event_test -- --ignored --nocapture --test-threads=1` (actual 7/7).
- [ ] VER2→AC8/11: **macOS manual** `source .env.test && source .env.live-test && cargo test --test agent_hook_crash_test capture_coordinator_native_replay_matrix -- --ignored --exact --nocapture --test-threads=1` (1/1; inspect OpenCode row execution).
- [ ] VER3→AC9/11: **macOS manual** `source .env.test && source .env.live-test && cargo test --test agent_import_test agent_import_confirmation_precedes_opencode_export -- --ignored --exact --nocapture --test-threads=1` (1/1).
- [ ] VER4→AC10/11: **macOS manual** `source .env.test && source .env.live-test && cargo test --lib internal::ai::capture::live_oracle_tests::live_checkpoint_metadata_shape_is_stable -- --ignored --exact --nocapture --test-threads=1` (1/1; inspect OpenCode row execution).
- [ ] VER5→AC11: compare the exact 10-name static inventory to per-case test reports, non-skip branches and source hashes/modes; the manual report records test count, OpenCode assertion marker, elapsed time and final source SHA, never private payloads. Any failure returns to the named implementation owner for reviewed forward-fix and both COV2-16/17 final-source audits rerun; this audit cannot edit code.
- [ ] VER6→AC12: under DEP-COV2-MODEL, pre-register actual isolated OpenCode executable/build/model/plugin/transport IDs and full sanitized argv in this plan, then run one genuine visible-only local model observation on final behavior source (≤10 minutes); record actual exit, JSON field names/types/counts, version/partial outcome, keyed digest and source SHA only. Do not register an issuer, call live mint, store transcript or treat readable output as native proof. Missing/failed response is UNRUN/blocked, not synthetic PASS.

**Full-suite trigger:** N/A：no-release audit 不自行执行 C 组；卡内本机 manual matrix 与文档 B 门必须通过，C/D 继承 COV2-REL-01。

**Dependencies:** COV2-16；DEP-COV2-MODEL；消费前驱实际最终 A/B、适用 focused/full、fmt/strict、单 Claude PASS 与 signed+DCO main 的 source-qualified 文件 SHA/mode；不以其尚待 C/D 的 `locally-accepted` 冒称 complete。

**Deliverables:** `docs/development/plan/plan-20261013-coverage-schema.md`; `docs/development/plan/plan-status.md`; `docs/development/tracing/coverage-v2.md`; `docs/development/plan/plan-20260902.md`；每卡的计数、源码 SHA、hash/mode、review/门结果写在本计划正文，不建证据文件。

**Implementation write set:** I-audit = `docs/development/plan/plan-20261013-coverage-schema.md`; `docs/development/plan/plan-status.md`; `docs/development/tracing/coverage-v2.md`；本卡新增精确路径 = `docs/development/plan/plan-20260902.md`。无 glob 默示写权。

**Release write set:** N/A；C/D 继承 COV2-REL-01。

**Files likely touched:** `docs/development/plan/plan-20261013-coverage-schema.md`; `docs/development/plan/plan-status.md`; `docs/development/tracing/coverage-v2.md`; `docs/development/plan/plan-20260902.md`。

**Docs and compatibility impact:** `docs/development/tracing/coverage-v2.md`；逐文件核内容与源码行为对应，`tests/INDEX.md` 只在本卡精确写集确有索引变化时同步。无新增网站命令行为；若实施引入公开差异，先按 ER-06a 修本卡写集再编辑。

**Rollback mode:** forward-only；恢复时签名+DCO 新 main 修复，既有 v1 claim/artifact/digest 不回写或重解释。

**Migration and rollback:** 无 DDL；Repository claim 仅用既有 versioned 键。若实现证明 DDL 必需，停止、加 GC-13 角色隔离迁移卡与旧/新读端门，再修本卡写集和 G-03/G-04。

**Security and privacy:** 继承 GC-COV2-02/03；错误或不可信版本必须 zero-success、不得向 ordinary/typed/log/error 导出候选内容；Session 例只在本机 ignored 执行。

**Performance budget:** 继承 GC-COV2-05；identity map 的 v1+v2 总量仍限 4096 条/1 MiB，任何新增解析在分配前有界；变更 hot path 时以具名门证明无无界扫描。

**Estimated scope:** S（落点/生产文件计数：12/20, 6/20, 0, 0; S）。

**Version increment:** N/A；此卡不发版。

**Release boundary:** no-release；证据随 REL-COV2-01 的 COV2-REL-01 交付；用户已授权逐卡 reviewed signed+DCO 直推 main，但不写 R、不 tag、不调用 release。

**C/D coverage from:** COV2-REL-01；发布点的 exact-source C/D 绿后才能 `done/complete`。

**Granularity:** `type=audit; axis=既有 lifecycle/crash/import/oracle 10 例及真实可读响应本机审计; recovery=forward-only; complete=yes; self-contained=yes; AC=12/20; VER=6/20; landing=0; prod-files=0; scope=S; deps=COV2-16；DEP-COV2-MODEL; writeset=串行于前驱; release=no-release; split-from=N/A; exception=N/A`

### Task COV2-REL-01：coverage-v2 家族唯一 patch 发布

**Task type:** release

**Lifecycle / Acceptance:** pending / 空

**Description:** 只把经审查的 20 implementation+2 audit COV2 family 按一个稳定 patch 发布并验证；不引入新行为，冻结树不含 OG-27 source。COV2-21/22 必须是实际 signed main A/B+单 Claude PASS 交付，12 AC/12 VER 的 release 计数不因拆卡静默增删。

**Out of scope:** native Source A/issuer 注册属于父计划 DEP-OG-12/OG-27；若发布后发现缺陷，以新签名 patch 前滚，绝不改已发布 tag/asset。

**Current evidence:** 基线 `dcc468b` 时本计划尚未实施；2026-10-13 当时安装版 0.30.48 拒绝 `.2` 注册的 LBR-AGENT-005 只证明该旧版本不满足前提，非本计划正例。独立 r2 binary-to-source 有限子门 PASS（索引 SHA `b9071d9e5c22ef6eeecb2fa447137719a9a3189282a0adf496e37f07eec57bf1`；Claude raw SHA `a4f933f6875c2c41422b183d3081769c424d279949f306d444c96958140e85eb`），但 runtime/issuer/Source A NO-GO。当前 `compat_version_surface_sync` 2/2 tests 对齐三文件。

**Acceptance criteria:**

- [ ] AC1 all child behavior-file hashes/modes match the freeze
- [ ] AC2 the release tree equals the approved COV2 freeze, including every ancestor enumerated by VER2 and excluding OG-27, plus the allowed R-only bump
- [ ] AC3 COV2-12 fake-exporter live Session report is source-qualified PASS
- [ ] AC4 its fake-exporter import Session report is source-qualified PASS
- [ ] AC5 its failed-export zero-write Session report is source-qualified PASS
- [ ] AC6 the previously failing ignored extraction fixture has a source-qualified repaired 1/1 PASS with old test name or reviewed renamed successor
- [ ] AC7 COV2-15 proof/state-bearing candidate live report is partial with zero-plaintext assertions and source-qualified PASS
- [ ] AC8 COV2-18 candidate import report meets the same contract and is source-qualified PASS
- [ ] AC9 genuine local visible-only observation is PASS on final behavior source
- [ ] AC10 COV2-16 all 12 inherited bridge/pipeline/OG-26 cases have non-skipped macOS PASS on final source
- [ ] AC11 COV2-17 all 10 existing lifecycle/crash/import/oracle cases have non-skipped macOS PASS on final source
- [ ] AC12 every child's inherited C/D ledger is valid for the published SHA before `done/complete` or `DEP-OG-13=complete`.

**Verification:**

- [ ] VER1→AC1 compare each source path hash/mode handoff with frozen source SHA.
- [ ] VER2→AC2 compare the freeze diff with approved card paths and OG-27 exclusion; execute `libra log <last-stable-tag>..<freeze-main-sha>` and classify **every** carried commit by owning plan/card, signed SHA, review, required C/D state and unmet release gate. COV2 children inherit this release C/D；OG-24/25/26 须核其 A/B+Claude+signed main 及本次安全/安装门，其下游 `REL-FAST-OG-04` C/D 保持 pending；其它计划乘载提交若有应先满足的独立前置则阻断冻结。Verify `approved_build.rs` `.2` admission source is in the actual release tree; installed-binary registration remains a separate test (`libra diff --check` is normal hygiene outside this count).
- [ ] VER3→AC3 **在实际 patch bump 后、与 T-2 完全相同的 signed release-candidate commit SHA 上重新运行** macOS manual `source .env.test && cargo test --features test-live-agent --test agent_opencode_bridge_test cov2_macos_fake_exporter_live_session_capture -- --ignored --exact --nocapture --test-threads=1` (1/1)，并核对行为文件 SHA/mode。
- [ ] VER4→AC4 同树重新运行 `source .env.test && cargo test --features test-live-agent --test agent_opencode_bridge_test cov2_macos_fake_exporter_import_session_capture -- --ignored --exact --nocapture --test-threads=1` (1/1)。
- [ ] VER5→AC5 同树重新运行 `source .env.test && cargo test --features test-live-agent --test agent_opencode_bridge_test cov2_macos_failed_export_session_zero_write -- --ignored --exact --nocapture --test-threads=1` (1/1)。
- [ ] VER6→AC6 同树重新运行 `source .env.test && source .env.live-test && cargo test --lib internal::ai::capture::live_pipeline::tests::export_runner_lease::end_to_end::opencode_extraction_failure_continues_checkpoint -- --ignored --exact --nocapture --test-threads=1` (1/1；若 COV2-12 审核后改名，先把此处命令同步为其实际新 selector，并记录旧→新映射)。`5cafe322` exit-101 只保留负面基线。
- [ ] VER7→AC7 同树重新运行 `source .env.test && cargo test --features test-live-agent --test agent_opencode_bridge_test cov2_macos_fake_exporter_candidate_live_partial_no_plaintext -- --ignored --exact --nocapture --test-threads=1` (1/1)，核 exporter-executed 与各零 sink 断言。
- [ ] VER8→AC8 同树重新运行 `source .env.test && cargo test --features test-live-agent --test agent_opencode_bridge_test cov2_macos_fake_exporter_candidate_import_partial_no_plaintext -- --ignored --exact --nocapture --test-threads=1` (1/1)，核各零 sink 断言。
- [ ] VER9→AC9 由 COV2-17 所有者在 DEP-COV2-MODEL 下预登记已授权 isolated OpenCode build、模型、完整无密 argv 和 ≤10 分钟上限，再在**同一 bumped T-2 commit SHA** 上重新执行 genuine visible-only 观察；实际原始命令、exit、无敏字段类型/数量和 keyed digest 写回本计划。不得登记 issuer、调用 live mint、使用 COV2-17 的过期样本或把观察当 Source A。
- [ ] VER10→AC10 audit COV2-16's exact 9/9+1/1+1/1+1/1 results and no-skip source scan；逐行为文件 SHA/mode 必须与 bumped T-2 树完全一致，否则在该树重跑完整 12-case matrix，不能只引用旧审计 SHA。
- [ ] VER11→AC11 audit COV2-17's exact 7/7+1/1+1/1+1/1 results and OpenCode-row/non-skip source scan；逐行为文件 SHA/mode 必须与 bumped T-2 树完全一致，否则在该树重跑完整 10-case matrix，不能只引用旧审计 SHA。
- [ ] VER12→AC12 audit the source-qualified C/D ledger for completeness and exact release-SHA consistency.

**Full-suite trigger:** T-2 无条件在实际 bump 后的最终同一源码树运行 `source .env.test && source .env.live-test && cargo nextest run --all --no-fail-fast --retries 2`；即使只是版本顺延，也不得复用 bump 前全量。未运行、失败、取消或不同 release-tree SHA 均不算通过。

**Dependencies:** COV2-01/13/02/07/14/03/19/20/21/04/22/08/05/09/06/10/11/12/15/18/16/17 全部本机 source-qualified A/B+review+signed main deliverables；DEP-COV2-SITE；DEP-COV2-MODEL；本卡 VER3–9 的新增/真实观察与 COV2-16/17 全22例在最终行为源码非 skip PASS。

**Deliverables:** 稳定 patch tag/Release、签名 manifest、四平台资产、Homebrew、Cloudflare 下载与网站、指定 curl 本机安装、全部 child C/D ledger；汇总证据写入 `docs/development/plan/plan-20261013-coverage-schema.md` 与 `docs/development/plan/plan-status.md`。

**Implementation write set:** `docs/development/plan/plan-20261013-coverage-schema.md`；`docs/development/plan/plan-20260902.md`（仅 source-ready→complete/OG-27 release 边）；`docs/development/plan/plan-status.md`。无代码行为写入。

**Release write set:** `Cargo.toml`、`install.sh`、`install.ps1`、由工具链刷新的 `Cargo.lock`、签名 tag/Release/manifest/artifact；以 `compat_version_surface_sync` 报告的实时集合为权威，外部站点源由 DEP-COV2-SITE 独立 owner 写。

**Files likely touched:** 本卡 I 与 R；不修改 COV2 子卡行为源码。

**Docs and compatibility impact:** `docs/development/plan/plan-20261013-coverage-schema.md`、`docs/development/plan/plan-20260902.md`、`docs/development/plan/plan-status.md` 的实际发布结果与版本号；网站 `cf` 部署/HTTP 是 D 组证据，不能以本地 Markdown 代替。

**Rollback mode:** immutable-release；发布前失败停住修复并新冻结，发布后缺陷签名+DCO 新 patch 前滚，不复写已发布 tag/asset。

**Migration and rollback:** 无新 DDL；既有 v1/v2 用户数据不降级、不删除；必要修复保持版本域和可恢复性。

**Security and privacy:** 同一发布 SHA 的安全 CI、签名与四平台 manifest；模型 transcript、私有 issuer/key 不进入 release/计划证据。

**Performance budget:** 最终全量与性能相关 focused 门按 ER-13 执行；不得因发布版本面更改隐蔽增大 map/session 预算。

**Estimated scope:** M（唯一发布点，0 生产行为文件）。

**Version increment:** patch；发布前核最新稳定/上游版本，仅 patch +1，不覆盖已有 tag；每次实际 bump 后重跑本卡 exact-tree 全量和版本面门。

**Release boundary:** family release point of REL-COV2-01。

**C/D coverage from:** self；发布点实际 C/D 全绿后登记 20 子卡继承完成，再将父计划 `DEP-OG-13=complete`。

**Granularity:** `type=release; axis=atomic coverage-v2 family patch; recovery=immutable-release; complete=yes; self-contained=yes; AC=12/12; VER=12/12; landing=0; prod-files=0; scope=M; deps=COV2-17,DEP-COV2-SITE,DEP-COV2-MODEL; writeset=sole R publisher; release=family release point; split-from=N/A; exception=N/A`

**ER 强制门（不计 G-03）：** `source .env.test && cargo test --test compat_version_surface_sync` 必须实际 2/2；已 bump 精确树 nightly fmt、strict all-targets/all-features Clippy、双 env 全量 nextest、release build/install；同一提交签名+DCO；GitHub main 安全 CI、四平台稳定构建、manifest/签名、Cloudflare/Homebrew/站点 HTTP 与指定 `curl --proto '=https' --tlsv1.2 -sSf https://download.libra.tools/install.sh | sh` 本地安装都须实际通过。每新 commit 取消旧 commit Actions；不得把被取消的旧 run 计入当前 SHA。Linux 功能仍由用户后验。

## 测试矩阵

| 类别 | CI/默认 | macOS 本机显式 | 完成判据 |
|---|---|---|---|
| v1/v2 codec/DB/identity/pending | COV2-01/13/02/07/14/03/19/20/21/04/22/08 精确单测与静态守卫；19 真 identity writer，20 只读事务验领，21 caller retain/故障，04 有界 codec/seal，22 真 artifact persist/编译冻结旧读端/SQL fault | 无 | v1 冻结字节，v2 独立域、跨版本同键、同版本拒绝、故障零假成功 |
| import/reader/typed | COV2-05/09/06/10/11 的 direct normalized-turn、in-memory sink、graph/doctor 命令测试；不启动 OpenCode Session | 任何发现有 exporter/Session 的案例改 `#[ignore]` 并只本机跑 | v2 live/import 字节与 digest 相等，unknown version fail closed，公开 envelope=1 |
| 新增 Session cases | 不在 CI：`#[ignore]`/feature gate；静态 guard 在 CI | COV2-12 fake live/import/failure/extraction，COV2-15 candidate live，COV2-18 candidate import；COV2-17 real visible | 每例有实际 1/1 非 skip PASS、源 SHA、隐私 sink 独立断言；fake 不当 Source A |
| 继承 22 ignored cases | 不在 CI；静态清单 guard 在 CI | COV2-16 9+1+1+1；COV2-17 7+1+1+1 | 22 个名称全覆盖，OpenCode row 实际运行，缺一个即卡/发布阻塞 |
| stable release | bump 后全量 nextest（ignored Session 不入）、fmt、strict Clippy、security CI、四平台 | final-source Session 报告 + 网站/安装验证 | 真实相同 SHA/C-D 全绿；Linux 功能由用户后验 |

## 追溯表

| 来源/缺口 | 承接任务 | Libra 落点 | 文档/兼容 | 指定验收 |
|---|---|---|---|---|
| DEFER-OG25-COVERAGE-V2 / v1 四 record | COV2-01/13/11 | `observed_agents/coverage.rs`、live/import | coverage-v1/v2、COMPATIBILITY | v1 frozen 3 vectors + RG-04、production typed v2 exact |
| versioned claim/identity/pending | COV2-02/07/14/03/19/20/21/04/22/08 | `coverage_gate.rs`、metadata/key/checkpoint/pending/recovery | coverage-v2 | 03 marker/ref、19 sealed identity-v2/最终读端篡改与互换拒绝、20 read-only verify、21 同事务 retain、04 有界 authenticated codec/seal、22 真 artifact/冻结旧读端/写故障、08 replay |
| import/reader | COV2-05/09 | `agent_import.rs`、graph/doctor | EN/zh/网站 cf | cursor/digest/fault/unknown-version/privacy |
| OpenCode privacy & pipeline | COV2-06/10/11/12/15/18 | classifier/snapshot/live/import/export | agent/tracing/integration | pure typed + ignored macOS live/import/candidate |
| 22 inherited cases | COV2-16/17 | read-only tests | 本计划与 coverage-v2 summary | 12/12 + 10/10 non-skip/source-qualified |
| 交接与稳定发布 | COV2-REL-01 | 版本面/Release | parent DEP-OG-13、status/site | bumped nextest、security CI、四平台、安装、网站 |

## 里程碑验收与回滚

| 里程碑 | 完成条件 | 发布/证据 | 恢复 |
|---|---|---|---|
| M-COV2-0 | OG-26 实际 signed main source-qualified handoff；网站 cf 隔离 owner 确认 | DEP-COV2-OG26/SITE 在本计划记录 SHA/mode/UTC | 缺口保持 pending，不覆盖脏文件 |
| M-COV2-1 | COV2-01/13/02/07/14/03/19/20/21/04/22/08/05/09/06/10/11 A/B+Claude+signed main 逐卡交付 | 每卡证据写本计划，仍 locally-accepted | 出错在 owner 卡前滚、重跑下游 |
| M-COV2-2 | COV2-12/15/18 与 16/17 全 22 继承 case 完整本机 PASS | final-source manual report；`DEP-OG-13=source-ready` | 一例失败回 owner，审计全部重跑 |
| M-COV2-3 | COV2-REL-01 exact-tree stable patch 与 C/D 完成 | Release/CI/site/install 真实 SHA/run | immutable release；新 patch 修复，随后 `DEP-OG-13=complete` |

### 故障恢复矩阵

| 故障点 | 可接受残留 | 恢复动作 | 禁止结果 |
|---|---|---|---|
| v2 claim 或 marker 事务失败 | pending/未完成标记与可重试状态 | 校验 owner/fence/revision 后原子重试或 doctor | claim/ref 一半成功、假 ack、覆盖 v1 |
| v2 pending 旧读端打开 | 完整原 pending bytes | COV2-22 从真实持久化 artifact 验编译冻结读端非破坏性拒绝，升级读端后重放 | 旧端改写或降级为 v1 |
| 第二版本 pending lease retain SQL 或后续 writer 失败 | 原 v1/v2 reservation 与旧 artifact 状态 | COV2-20 只读验领、COV2-21 在 caller-owned SQLite 事务内回滚两版 lease；COV2-22 在真实 chunk/header/alias 写故障后证实两版 lease 与 artifact 全回滚；不能用 SQL probe 代替真 artifact | 第一行 lease 已变而第二行/真实 artifact 未提交 |
| identity map 未知键/超 cap/MAC 错 | 既有可见 claim 不变 | fail-closed、记录 content-free doctor 结果 | 跳过损坏条目当 absent、假成功 |
| OpenCode candidate Session | 可追溯 partial receipt | zero ordinary/typed/object/log/error sink；待真实 Source A | synthetic 冒称 native artifact |
| macOS containment 缺失 | 失败的本机测试 | 配齐 Required Seatbelt 或显式失败 | `None`/early return 伪绿 |
| release/网站/CDN 失败 | 已审核源码/可能已有 immutable tag | 分类后签名前滚/new patch，重跑受影响门 | 改写 tag/asset、用不同 SHA 的 CI/安装证据 |

## 风险登记

| 风险 | 影响 | 缓解/任务 |
|---|---|---|
| v1 fixture 被新 v2 逻辑改写 | 旧 claim/digest 不兼容 | COV2-13 三向量 + RG-04 SHA、COV2-04 codec 历史字节/MAC 与 COV2-22 编译冻结旧读端 |
| native v2 record/identity map 与密封来源、payload 或 provider 不同源 | 假 native 归档或错误身份绑定 | COV2-19 在同一事务检查来源、记录与 map，并注入真实 `identity-v2` SQL 故障 |
| pending 先写 artifact 后保留 versioned claims，或以手工插入行冒充真持久化 | 过期 lease 被接管、旧端破坏性重放、假绿 | COV2-20 提供只读 versioned verify、COV2-21 提供同事务 retain、COV2-04 提供有界 codec/seal；COV2-22 必须从真实 seal/persist/commit 运行编译冻结旧读端并注入后续 artifact SQL fault |
| 双 map 容量倍增或 unknown key 假 absent | 热路径资源超支/身份绕过 | COV2-07/14 域隔离与 aggregate cap/fail-closed |
| 本机 Session 用例因 bwrap 不存在早退 | 虚假 22/22 与 release 假绿 | COV2-12 Required Seatbelt + COV2-16/17 非 skip inventory |
| 网站 checkout 脏且落后 | 覆盖他人改动/站点源码不一致 | DEP-COV2-SITE 干净隔离 cf owner，source hash、部署 HTTP |
| v2 typed 与 native 来源混淆 | 未授权 opaque 归档/明文泄漏 | COV2-06/10/15/18、父计划 Source A 硬门 |
| 2026-10-13 旧安装版 0.30.48 不批准私有 `.2` | 当时 OG-27 注册失败，不能外推到未安装的新版本 | COV2-REL stable patch 实装后，在父计划原 issuer epoch0 流程重验；r2 binary-to-source PASS 不替代 runtime/issuer/Source A，当前仍 NO-GO |

## 性能与容量摘要

| 操作 | 单次成本/边界 | 总预算 | 验证 |
|---|---|---|---|
| 私有 v1+v2 identity maps | 解码前有界、仅两个已知 key | 合计 ≤4096 entries、≤1 MiB encoded | COV2-14 aggregate entry/byte exact tests |
| pending envelope | 继承现有上限与 MAC；v2 无无界扩展 | `key.rs` 既有 64 MiB 封闭上限，开工刷新 | COV2-20 同事务只读 verify、COV2-21 retain/rollback、COV2-04 entry 与 wire-byte 分别限额、COV2-22 真实 artifact/旧读端/SQL fault、COV2-08 replay tests |
| graph/doctor | 版本化 bounded 读取；不输出原文 | 不新增无界全库 scan | COV2-09 exact tests 与需要时性能门 |
| Session exporter | 继承有界 runner/deadline/containment | 不放宽 trust/取消/资源上限 | COV2-12 ignored E2E 与既有 oversize case |

## 兼容与文档收口

- [ ] `COMPATIBILITY.md` 与 `docs/development/tracing/coverage-v2.md` 对生产 v2、v1 冻结/旧读端拒绝、Source A 区别一致；`docs/development/tracing/agent.md` 同步。
- [ ] `docs/commands/agent.md`、`docs/commands/zh-CN/agent.md`、`docs/development/commands/agent.md`、网站 `cf` EN 页由 COV2-09/11 同卡同步；网站真正部署与 HTTP 在发布 D 门核。
- [ ] `docs/error-codes.md` 当前 N/A（不新增/改稳定 LBR 错误码）；若实际引入则先修卡 I/G/文档验收。
- [ ] `tests/INDEX.md` 在具名卡新增/调整 target 描述时同步；`Cargo.toml` `[[test]]` 和 `tests/compat/README.md` 当前 N/A（无新增 compat 文件/target），若新增须同卡修写集与 ER-13 门。
- [ ] `docs/development/plan/plan-long.md` 当前 N/A：本计划承接父计划 DEFER，不新建长期 LR；计划登记在 `docs/development/plan/plan-status.md`。

## Claude Code review log

本计划每张卡只需一名独立 Claude Code 审查者在最终源码上 `PASS`；material P0/P1/P2 修复后再审，不把旧轮 `FAIL` 覆写成 `PASS`。R4 是上游已复核规划输入；本文件实际修订/每卡仍分别需要自身最终审查。审查记录在本计划正文，不新建证据文件。

| UTC | 卡/范围 | 最终结论 | Findings/处置 | 证据 SHA 与源码 SHA |
|---|---|---|---|---|
| 2026-10-13 09:15:00 UTC | 本计划起草 | pending | 仅方案编写，未执行测试 | R4 SHA `18c34afa261f89100f2d6f0e9ce30953dbc97f7c9c1d7cf8661845d0733f7c2f` |
| 2026-10-13（暂行草案审查；未记精确 UTC） | 隔离草案 R1 | FAIL | Claude Code Sonnet 3×P1、15×P2；已在隔离草案修订，仍待复审，绝不当计划采纳 PASS | raw SHA-256 `db4f1bc5096e31060bbd67137b16f22a5e4c0a2e4a7789d49997511af0abfbd4`；被审草案 SHA `658677dad25cc9aa32eb97b808d721b3e45a26cd59ee719b46e5050f96a8c0c4`，parent patch SHA `99cb05ad88a7c8527bd3305b0bd9badf565a90a08449c0595e681a822c944a32` |
| 2026-10-13（暂行草案审查；未记精确 UTC） | 隔离草案 R2 | FAIL | 0×P0/P1、7×P2；按具名发现修订 temp 草案和父补丁；不得当最终 PASS | raw SHA-256 `998ccce51e98e71d868024d844f261bcbfc952a8e03c5726493a27aebda9093e`；被审草案 SHA `34c14dddc058bfd8ad8649aaba4d6dbbf6dec9681c25007df4a3af8cf1f45d51`，parent patch SHA `2943b0da89f231991d0c15fc4e8930fa7934f2738ec550fd6a65be9a15714d61` |
| 2026-10-13（暂行草案审查；未记精确 UTC） | 隔离草案 R3 | FAIL | 0×P0/P1、1×P2（G-03 复合 AC）；已将有关 AC 收窄为单谓词，待复审 | raw SHA-256 `f9c283c780d81bea834b64c0dbbd6d93aace7937a60d32bf3417223701467607`；被审草案 SHA `762ee903225dec3847fccd6ef98cefd57368cfcdba910dc313d30df8296b8049`，parent patch SHA `ccdb7f9823568e6da4ce4793acab9665c69026a80ce3ff0a4cf2dfe37d8e1fff` |
| 2026-10-13（暂行草案审查；未记精确 UTC） | 隔离草案 R4 | FAIL | 0×P0/P1，parent/status patch 1×P2（status table duplicate/malformed row）；temp 补丁已修订。8×P3 文案/slot一致性随本次修订；不可当计划采纳 PASS | raw SHA-256 `73f5d9b2d94283cada9b24bb659e2a5cd261acd7d1b0fadaf081625527059561`；被审草案 SHA `855f52465f92016eae29342d43fffb0a0b56960198b7a6fc5f38814c9fcb0eaf`，parent patch SHA `ccdb7f9823568e6da4ce4793acab9665c69026a80ce3ff0a4cf2dfe37d8e1fff` |
| 2026-10-13（暂行草案审查；未记精确 UTC） | 隔离草案 R5 | PASS（临时稿） | 0×P0/P1/P2；另有4×P3已在隔离稿修订，采纳到main后仍须重新审查实际字节 | raw SHA-256 `df9f77cdd9ebc5e0055a35c5c59324ecac68ce10a75c2d001159b762822cb793`；被审草案 SHA `9a042abef56de8218546b2080d10d35c38e6ab1dd8877c5682d7c575fb5f55b2`，parent patch SHA `27cbe6983691dc252283fc5f739ca53e7fa2220fa4fd78e32d6bffb41ff903cf` |
| 2026-10-13 11:33:56 UTC | 本计划最终采纳 | **PASS** | 唯一 Claude Code 最终字节复审 P0/P1/P2=0；前轮两个 P2（Linux CI 选择器与父计划 C/D 状态）已修正；只批准计划采纳，不当 COV2-01 实现评审 | raw SHA-256 `b05a5dd7761cecefe6956aa30e62e4f36a195c8f27a786bb9ab2656275a66114`；签名+DCO main `c94fd8dd60d8d9fa670b2682bf7b9979bae91548` |
| 2026-10-13 13:04:09 UTC | COV2-01 最终源码/文档 | **PASS** | 唯一 Claude Code Sonnet 最终审查 P0=0/P1=0/P2=0；保留 5 项 P3 后续注意（redaction 次序、legacy API、重复判断、Source A 未证、审查者未亲自重跑测试）；reviewer 独立核对八文件 SHA、153-byte golden digest、RG-04 快照，确认 23 skipped 未被误算验收 | raw SHA-256 `5e5ad9af5b9b917054ba5a94d6c17629d9e9b6b959edc8033b6ee72cfaa9c64c`；签名前八文件 SHA/mode 见 COV2-01 卡；后审只作本行及状态事实登记 |
| 2026-10-13 13:37:14 UTC | COV2-13 冻结 v1/RG-04 合同 | **PASS** | 唯一 Claude Code Sonnet 最终审查 P0=0/P1=0/P2=0；独立对照历史 `dcc468b` 三字节/摘要常量、fixture SHA 与四文件 post SHA/mode，确认测试非空、生产无 delta、ER-13 不冒称本卡 full；仅保留 deterministic/Source A 未证等残余 | raw SHA-256 `f900a64d58bc2b125145cec3e608d9356bb613bb8a0aca7e6d0338efb34b3afd`；起点 signed main `a3b51fd103fc2aa95f64c3395339a00937060356`；后审只补本行与状态事实 |
| 2026-10-13（审查时未记精确 UTC） | COV2-02 版本化 DB claim 仲裁 | **PASS** | 唯一完成的 Claude Code Sonnet 源码审查 P0=0/P1=0/P2=0；确认 v1 旧调用保持，v2 sidecar 暂不接生产 writer；Fable 用量上限请求无结论；审查时 full 尚在运行，后续 root 已核 exit 0/10059 passed | raw SHA-256 `02db003513b2729c9b24fc82fe5a70bcabd0eaf9de11b065f83e5078ce88b244`；五文件最终 SHA/mode 见 COV2-02 卡；后审仅补测试结果与状态事实 |
| 2026-10-13（审查时未记精确 UTC） | COV2-07 identity-v1/v2 MAC 域 | **PASS** | 唯一 Claude Code Sonnet 最终源码/文档审查 P0=0/P1=0/P2=0；v1 字节与 MAC、版本前置拒绝、私钥生命周期、跨域移植、容量和旧调用逐项检查；P3 包括 v2 record 拒绝的显式测试建议及后继 COV2-14 输入界限，均不改变本卡原语结论。审查者未运行本机测试；root 独立核验门结果 | raw SHA-256 `c275fa3872348db329daaa71591307daff4bf154483a4332687bb129b946bd9c`；六个非状态文件的最终 SHA/mode 见 COV2-07 卡；后审仅补测试/状态事实 |
| 2026-10-13（审查时未记精确 UTC） | COV2-14 双私有 map 初轮最终行为审查 | **FAIL** | 唯一 Claude Code Sonnet 发现 1×P2：公开 EN/zh 命令文档仍承诺无效 map 逐 turn partial；已先修本卡精确 I 再修文档、兼容说明 | raw SHA-256 `b1d3235da90b481c71b309592c441f303dd4cfaddb5abf029feb0448114e4285`；旧结论保留，不算 PASS |
| 2026-10-13（审查时未记精确 UTC） | COV2-14 文档守卫最终源码/文档 | **PASS** | 唯一 Claude Code Sonnet 最终审查 P0=0/P1=0/P2=0；公开文档 P2 已关闭，随后全量发现的旧文档守卫也已先扩精确 I 并更新；P3 仅为可选旧措辞负断言与历史测试名链接。审查者未运行门，root 实际核验最终全量 exit 0、10069/10069 | raw SHA-256 `2d1e5fb7425abc9aa2802db241213811dfcd6c891410e9853e554c5fcb806966`；十个非状态文件 SHA/mode 见 COV2-14 卡；后审仅记测试/状态事实 |
| 2026-10-13（审查时未记精确 UTC） | G-09 r4 三文件规划协调 | **FAIL** | 唯一 Claude Code Sonnet 最终字节审查 P0=0/P1=0/P2=2；COV2-03 新 VER5/旧回归未显式 UNRUN，COV2-19 AC7/8 各混合独立谓词；r5 修订须单独最终字节复审，不追认 r4 PASS | raw JSON SHA-256 `db4bd522390ef5eea047a8e1c051c1405115fc1bb62071ea39be80d65f9e6952`；P3 文档自足性与表格渲染同修 |
| 2026-10-13 18:08:29 UTC | G-09 r5 三文件规划协调最终字节 | **PASS** | 唯一 Claude Code Sonnet 复审 P0=0/P1=0/P2=0；r4 两处 P2 已修，最终三文件 SHA/mode 与 review prompt 的字节由 root 复核；r4 FAIL 不追认。规划已签名+DCO `main` `eaa43ec13f6c50212f0b3c2fa8a51799ec74bff5` 并 Push，COV2-03 前置成立；本次规划不计任何 COV2 AC/VER、Source A、C/D 或 release PASS | raw JSON SHA-256 `77d4820e329ade99f5fba7b43821d072d4d7068ee5ba639ada4704851686f4fd`；审查者指出两条非阻断 P3，COV2-19 marker 回归和 COV2-22 额外 compat guard 按各卡 mandatory ER-04 实测，不把未运行项视作 PASS |

## 非目标与延后项

| ID | 内容 | 原因与重启条件 | 承接 |
|---|---|---|---|
| DEFER-COV2-LINUX | Linux 功能矩阵由用户在 Linux 后验，当前标 UNRUN | 用户明确只要求 macOS 功能；安全 CI 与四平台发布仍必须通过 | 用户后验，不替代 COV2 macOS 门 |
| DEFER-COV2-NATIVE | `redactedData` / `reasoningEncryptedContent` 真实正例及 native 全字段归档 | 需要实际支持字段的 producer/模型、可信注册与 Source A；不以本计划 synthetic/visible-only 代替 | 父计划 DEP-OG-12、OG-27..29/05；原完成定义不被此表自动放宽 |

## 实施证据汇总

以下各条 `0/19`、`1/19`、`2/19`、`3/19`、`4/19`、`5/19` 是发生当时的 19 卡分母历史收据，不倒改过去的观察。COV2-14 已由 signed+DCO main `4558af64c09e63c4716fa68595776a8044f66cc7` 交接，本次 G-09 23 卡分母的当前正式进度 **0/23**、源码就绪 **6/23=26.09%**；G-09 签署时 COV2-03 新增 VER5 与旧 catalog receipt ER-04 回归均 UNRUN；现已各 1/1，最终双 env full **Summary [1618.814s] 10078 tests run: 10078 passed (14 slow), 23 skipped**/raw SHA `c49dd3c33652d55541a0e4c0269c817460b9109cf261eab5271d90760a49e8a9`、exit0；唯一最终 Claude Code PASS、P0/P1/P2=0、P3=5/raw SHA `9ef25f92615eefcf2019212a306cfa7542ef416b693286bb99111006b3ea6df7`；signed+DCO 源码/文档 main `d717e42e0d1ec75e6ae576b21242fd1519040abd` 已 Push 且 remote 同 SHA；COV2-19..22 新 AC/VER 均 UNRUN；0 matched、helper-only、skip 或重新计数都不构成 PASS。父计划正式 **55/67=82.09%**，Source A/DEP-OG-12 与家族 C/D 均未闭合。

| UTC | 卡 | main/source SHA | AC/VER 实际计数 | fmt/strict/full | Claude PASS | C/D 与剩余门 |
|---|---|---|---|---|---|---|
| 2026-10-13 09:15:00 UTC | 全计划初稿 | `dcc468b` 只读基线 | 0/19 卡完成；21 个本机旧 case 未在本计划验收 | UNRUN | UNRUN | OG-26、网站、Source A 与 release 未闭合 |
| 2026-10-13 10:59 UTC | 采纳前只读预核 | OG-26 待签工作树，实际 main SHA 尚缺 | 0/19 卡完成；22 个继承本机 case（原21+OG-26新增1）仍须最终行为源码 12/12+10/10 验收 | UNRUN | 草案修改待最终 Claude | r2 binary-to-source 有限子门 PASS，runtime/issuer/Source A/归档仍 NO-GO；OG-26 signed main 与网站 owner 未交接 |
| 2026-10-13 11:16:08 UTC | OG-26 signed main 只读交接 | `734b6f03884d3aa38400acbbfd446b557e642d94`，`HEAD=origin/main` | 0/19 卡完成；22 个继承本机 case（12+10）均未作 COV2 最终行为源码验收 | UNRUN | 采纳后最终字节单 Claude 待审 | OG-26 源码已交；网站 owner 仍待隔离确认，runtime/issuer/Source A/归档仍 NO-GO |
| 2026-10-13 11:33:56 UTC | 计划采纳 | `c94fd8dd60d8d9fa670b2682bf7b9979bae91548`，签名+DCO main | 0/19 正式完成；COV2-01 已立卡但此时未验收 | 采纳文件 docs contract 30/30、fmt、strict Clippy exit0；非 COV2 实现门 | 唯一 Claude Code **PASS**，raw SHA-256 `b05a5dd7761cecefe6956aa30e62e4f36a195c8f27a786bb9ab2656275a66114` | COV2-01 起顺序实施；Source A/native、22 个最终行为源码 Session 门及家族 C/D 仍未完成 |
| 2026-10-13 13:04:09 UTC | COV2-01 本机 A/B+单审交付 | signed+DCO main `a3b51fd103fc2aa95f64c3395339a00937060356`；八个非状态文件 SHA/mode 见卡内清单 | 8/8 AC、3/3 VER、RG-04 frozen 1/1；0/19 formal，source-ready **1/19=5.26%** | fmt exit0；strict Clippy exit0/raw SHA `ce387dd8e31968c025fdd8ba63e0b5902fd0341d3d409b50665a660039d4d334`；双 env full 10053/10053 passed、23 skipped/raw SHA `fa3327517528a07abd8ece858e56a519d70d010b7a1ac56e80890c39665cc7c0` | 唯一 Claude Code **PASS**，P0/P1/P2=0，raw SHA `5e5ad9af5b9b917054ba5a94d6c17629d9e9b6b959edc8033b6ee72cfaa9c64c` | COV2-13 下一卡；家族 C/D、22 个最终行为源码 Session 与 Source A 仍待 |
| 2026-10-13 13:37:14 UTC | COV2-13 本机 A/B+单审交付 | signed+DCO main `cd698ade182a935c4d91b0f84cb3ad13e8325e0b`；四个非状态文件 SHA/mode 见卡内清单 | 8/8 AC、4/4 VER 各1/1；0/19 formal，source-ready **2/19=10.53%** | fmt exit0；strict Clippy exit0/raw SHA `c390c26cd50bf4f19aaef41fdef60b34bd766cfd7f8a506e323a3bc38b679302`；ER-13 无独立 full 触发 | 唯一 Claude Code **PASS**，P0/P1/P2=0，raw SHA `f900a64d58bc2b125145cec3e608d9356bb613bb8a0aca7e6d0338efb34b3afd` | COV2-02 下一卡；COV2 家族 C/D 与父计划 Source A 仍待 |
| 2026-10-13 14:45:54 UTC | COV2-02 本机 A/B+单审交付 | signed+DCO main `d4f159fd17676c7b591c1f80858f2de17094260f`；五个非状态文件 SHA/mode 见卡内清单 | 8/8 AC、3/3 VER 各1/1；0/19 formal，source-ready **3/19=15.79%** | fmt/strict Clippy exit0；双 env full 10059/10059 passed、23 skipped/raw SHA `afbadf5ca0b5ce4b3265d6810ff022efd054d9d7344f7f2c3737162eed5a6d1e` | 唯一 Claude Code **PASS**，P0/P1/P2=0，raw SHA `02db003513b2729c9b24fc82fe5a70bcabd0eaf9de11b065f83e5078ce88b244` | COV2-07 下一卡；家族 C/D 与父计划 Source A 仍待 |
| 2026-10-13 15:34:12 UTC | COV2-07 本机 A/B+单审交付 | signed+DCO main `14fc516d94955fadb867c16868e7f12792c4b52e`；六个非状态文件 SHA/mode 见卡内清单 | 8/8 AC、4/4 VER 各1/1；0/19 formal，source-ready **4/19=21.05%** | fmt/strict Clippy exit0；双 env full 10063/10063 passed、23 skipped/raw SHA `553bfa450a3efb7fecdf1907fc7e5bf4d14e51c8cfef40bd2d6f2a62754d05b8` | 唯一 Claude Code **PASS**，P0/P1/P2=0，raw SHA `c275fa3872348db329daaa71591307daff4bf154483a4332687bb129b946bd9c` | COV2-14 下一卡；v2 map 封闭与共同容量、家族 C/D、父计划 Source A 仍待 |
| 2026-10-13 17:39:01 UTC | COV2-14 本机 A/B+单审交付 | signed+DCO main `4558af64c09e63c4716fa68595776a8044f66cc7` 已 Push，十个非状态文件 SHA/mode 见卡内清单 | 8/8 AC、7/7 VER 各1/1；公开文档守卫与旧行为回归各1/1；0/19 formal，本卡 signed main 后按当时19卡分母 source-ready **5/19=26.32%** | fmt/strict Clippy/diff exit0；双 env full **10069/10069 passed、23 skipped、18 slow**/raw SHA `2849dcf7e2105ed1da5e5d3b0333374e2590ac909c1f478ef8ba68966e86aec4`；两次失败全量未计通过 | 唯一 Claude Code **PASS**，P0/P1/P2=0，raw SHA `2d1e5fb7425abc9aa2802db241213811dfcd6c891410e9853e554c5fcb806966` | COV2-03 下一卡；先按 G-09 复核必要拆卡；家族 C/D、父计划 Source A 仍待 |
| 2026-10-13 20:29:37 UTC | COV2-03 本机 A/B+单审源码交付 | signed+DCO 源码/文档 main `d717e42e0d1ec75e6ae576b21242fd1519040abd` 已 Push、remote 同 SHA；八个非状态文件 SHA/mode 见卡内清单 | 8/8 AC、5/5 VER 各1/1；五项 mandatory ER-04 与两项旧库缺表回归各1/1；0/23 formal、source-ready **6/23=26.09%** | fmt/strict Clippy/diff exit0；双 env full **Summary [1618.814s] 10078 tests run: 10078 passed (14 slow), 23 skipped**/raw SHA `c49dd3c33652d55541a0e4c0269c817460b9109cf261eab5271d90760a49e8a9`；r1/r2 中止、r3 失败保留不计 PASS | 唯一最终 Claude Code **PASS**，P0/P1/P2=0、P3=5/raw SHA `9ef25f92615eefcf2019212a306cfa7542ef416b693286bb99111006b3ea6df7`；五项 P3 承接见卡内 | COV2-19 下一卡；家族 C/D、22 个最终行为源码 Session 与父计划 Source A 仍待 |

**终止链末端 D 组记账协议预登记：** 如 COV2-REL-01 最后只是纯文档登记，推送后的 main CI 观察不能写入该提交自身。按模板 v2.14 将该 SHA、run ID、结论及 UTC 留到下一次 `docs/development/plan/plan-status.md` 例行同步；次日仍无同步时单独作状态维护提交。本例外不把未观察 D 当 PASS。

## 完成判据

- [ ] 23 张卡逐一满足 G-01..11、实际 A/B/Claude PASS，20 implementation 与 2 audit 的 signed+DCO main 源码/文档 Push，唯一 release point 实际 C/D；每卡到 `done/complete`。父计划正式 67 卡分母始终独立。
- [ ] COV2-16/17 对 22 个继承 ignored OpenCode Session cases 在最终行为源码上实测 12/12+10/10，且新增 COV2-12/15/18 本机正负例均非 skip PASS；没有任何 OpenCode Session 模型/采集 E2E 进默认 CI。
- [ ] v1 原字节/digest/MAC、RG-04 fixture 与 frozen pending decoder 未变；v2 writer/reader/import/pending/replay/graph/doctor 与 privacy/failure 门全部可重复通过。
- [ ] 发布前在已 bump 的同一源码树完成 nightly fmt、strict Clippy、双 env Nextest 全绿；触发 ER-13 的各子卡局部全量均已执行并修复本计划暴露的缺陷。ignored Session 不因全量 skip 代替本机门。
- [ ] 唯一 patch 版本、版本面 2/2、Cargo.lock 工具链、签名+DCO、main 安全 CI、四平台资产/签名 manifest、Cloudflare/站点/HTTP、Homebrew 与指定 curl 安装均真实同源通过；失败或取消不得记绿。
- [ ] `docs/development/plan/plan-status.md`、`docs/development/plan/plan-20260902.md` 的 `DEP-OG-13=source-ready→complete`、`DEFER-OG25-COVERAGE-V2` 交接与发布版本/提交/UTC 事实同步；OG-27 在 stable release 完成前不 Push，完成后仍独立受 DEP-OG-12 Source A/native `signature` 门控。
- [ ] 没有在仓库创建额外证据文件或文件夹；每轮测试、review、发布和站点摘要写在本计划及状态正文；如采用终止链末端 D 组后补协议，按预登记办法补齐。
