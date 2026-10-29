# status 正确性与性能优化任务计划（2026-10-28）

> 模板：`plan-template.md` v2.14。创建时间：2026-10-28 01:56:39 UTC。源码基线：`380c262257c5b4eae0725898444bb092bb86919e`；分支 `remove-status-speedup-proposal`。任务与 Issue 已建立；用户随后授权提交并推送计划文档，本次不实施代码、不发布版本。总览 Issue：[#638](https://github.com/libra-tools/libra/issues/638)。

## 文档职责与范围

以建卡时位于根目录的 `LIBRA_STATUS_PERFORMANCE_REVIEW.md`、`LIBRA_STATUS_PERFORMANCE_REVIEW_AUDIT.md` 和用户后续决策为输入，规划可独立验收的修复与优化。两份报告已由用户移出工作区，不纳入本次提交；此次不重新访问外部副本。本计划正文与 Issue 自包含，不依赖未发布报告链接才可理解。

**非目标：** CE_VALID 默认值及历史标志修复由他人负责，本计划不修改策略、不添加前置依赖。无机会性 status index 写回，无新后台服务，无 SQLite migration，无直接修改全局配置。pack 会话缓存、marker 合并、stat/hash IPC 批处理暂缓。

**成功定义：** 两项正确性缺陷修复；核心路径减少重复工作且输出/错误/安全语义不退化；收益有前后对照，未设置未经测量的 Git 性能追平目标。可选项可以保持 deferred，不阻塞核心轮次完成。

## 事实基线与审计校正

- restore 批量 size 缓存将加载预算 `peak_bytes` 当作 decoded size；缓存只预取未 tracked 路径，不能声称所有更新条目必然受影响。
- status 安全加载根 tree 后，递归子树仍进入会 panic 的 `Tree::load`。
- 默认 rename 和 porcelain v2 可重复展开 HEAD；原报告 `--no-renames` 基准不代表默认命令全部成本。
- `path::objects()` 仍走 ambient storage discovery；缓存 storage 前计算路径也有成本。
- `ignore_sources_for_target` 从 target 的父目录开始，因此检查目录并未预读目录自身的 ignore 文件；仅根预热不能证明 tracked 短路等价。
- 出队 marker probe 明确承担防目录替换的重新验证，不可直接删除；线程池提交次数不能证明固定线程切换下界。
- 审计表的 9,647 次 readlink 属于 242 次总发现的归因，不能全部算给 206 次对象读取。
- 约 733→491 ms、关闭 untracked 约省 200 ms、约 12 万次 idx read 均是历史报告引用。本轮未读取工作区外的原始归档，未独立复算，不承诺各项收益可相加。

## 设计决策与优先级

| 决策 | 约束 |
|---|---|
| ADR-SP-01 | 正确性先行：panic、size 契约为 P1 必选；CE_VALID 完全排除。 |
| ADR-SP-02 | 本轮核心：stat、HEAD 复用、request storage scope、pack 基础查询、ignore 上下文。 |
| ADR-SP-03 | SP-04 为建议可选项，默认本轮不启用；延后缓存/marker/IPC 需复测和新实施决定。 |
| ADR-SP-04 | 不移除 deadline、beneath/no-follow、racy-clean、逐路径错误及部分结果保护以换取速度。 |
| ADR-SP-05 | 现有 load-cost 接口保持保守预算语义，新增 decoded-size 能力；避免误伤 preview 限额。 |

## 与现有工作关系及依赖登记

2026-10-28 查询上游 open Issues 和标题含 status 的历史 Issues，未发现同范围开放任务；#470/#476 等为已关闭的 mode/worktree 兼容工作，不重复开启。#611 为 add/fsck 性能工作，与本计划不等价。`plan-status.md` 未检出同名 status 性能计划。`plan-20261025-qualification.md` 涉及 client_storage/util，实施前必须刷新冲突窗口。

| ID | 产物/负责人 | 可用标准 | 不可用处理 |
|---|---|---|---|
| DEP-SP-STORAGE | 现有资格计划的 storage/util 行为与写集；负责人由该计划登记，不在此虚构 | 开工核验其当前源码及在跑写集，无 I–I 冲突；配置失败不降级的回归可运行 | 暂停 SP-06，先协调顺序，不复用旧 PASS 代替当前验证 |
| DEP-SP-DOCS | 网站仓库 cf 文档交付；负责人待实施指定 | 在用户授权路径内核实 VCS、cf、逐文件路径及无冲突，完成 sync/check、相关前端验证、Claude Code PASS、部署验证 | 公开行为卡不得标 complete；当前仅列候选文件，未访问外仓 |
| DEP-SP-OPT | 用户/维护者对 SP-04 的启用决定 | SP-09 后有明确 adopt 决策及写集重新审查 | 保持 optional pending，不阻塞核心完成 |
| DEP-SP-DATA | 历史 Linux 原始归档或等价隔离 fixture；负责人待指定 | 访问范围获授权、数据/命令可核验；否则选择本工作区临时 fixture 建立新基线 | 不访问相邻目录；历史数字维持引用，不能声称复现历史样本 |

Nowledge 定向检索已尝试，服务连接被环境阻止；本计划依据当前仓库，不依赖未取回的历史结论。

## 全局执行约束与字段默认

- 建卡时各卡 Lifecycle=pending，Acceptance=空且没有 agent-ready 标签；当前状态以任务卡为准。2026-10-29 用户已选定并授权 SP-01 实施；未授权的后续卡不因此获得开工权限。实施前刷新源码、计划、Issue、写集冲突和环境；先复现失败再修复。任务字段列明的 `(new)` 测试必须在同卡新增，过滤零命中不算通过。
- 每次推进同一变更同步本计划与 `plan-status.md`。证据只汇总在本计划，不创建新证据目录/文件；工具临时运行文件不作为独立交付产物。
- 全局继承模板 GC-*、ER-*、G-*；新测试复用现有 target，若需要新增 target/serial lane，先更新卡写集及 `tests/INDEX.md`/registry 等再实施，不暗自扩张。
- **用户资源硬门槛（2026-10-30 澄清）：32G 内存门槛仅约束本机 nextest 执行；本机启动前核验物理总内存及当前资源压力，低于 32G 或无法确认则不启动，不通过降低并发绕过。GitHub Actions CI 不在此限制内，按现有工作流触发全量测试，不要求修改 CI 或增加资源预检。该澄清不授权自动 SSH 到其它主机；本机 not-run 与 CI 实际运行结果分别记账。**
- 全量门采用 `source .env.test && source .env.live-test && cargo nextest run --all --no-fail-fast --retries 2`；只在相关环境已安全核验后运行，不能打印 secrets。每次会推送的卡须 `cargo +nightly fmt --all --check`、`cargo clippy --all-targets --all-features -- -D warnings`。全量失败需前滚修复并重跑，focused 通过不能冒充全量。
- implementation 默认 independent，Version increment=patch；Release write set=权威版本面集合（`compat_version_surface_sync`）、工具链更新的 Cargo.lock、artifact。实际提交/推送/Release 需后续授权，本次建 Issue 不包含这些动作。
- implementation 默认 Rollback mode=immutable-release：未发布可回退本卡实现，已发布用修复版恢复；无 schema 迁移，不自动重写历史 index。下载旧版本无法撤销已写出的 stat，必要时安全刷新，不能保留可疑 clean 缓存。
- 默认 Docs and compatibility impact=N/A 仅适用于输出与错误完全不变的内部优化；公开行为变化卡须同卡双语与网站文档。网站路径为候选，DEP-SP-DOCS 核验后才允许开工；遵循用户 standing instruction 直接 cf，通过 Libra 提交推送，不能另建网站分支/PR/worktree。
- 无家族合并发布；C/D coverage=self（实现卡），SP-09 继承下列具体实现卡。SP-00 按用户批准的 EX-SP-00-AUDIT-01 独立文档验收（见下）；该例外不免除评审、fmt、Clippy 或本卡 PR CI，也不授权其它卡开工。

### SP-01 修复卡收口决策（EX-SP-01-CLOSE-01）

2026-10-30，用户在 PR #655 的全部检查通过后要求“设置卡片完成，推送更新 PR”。本次以修复实现、本仓库文档及 PR CI 验收作为 SP-01 的完成口径，登记 `done / complete`；这是对模板 ER-04/ER-08 完整发布切片要求的本卡具名例外，不代表版本发布或网站部署已完成，不扩展至其它实现卡。网站交付按用户此前决定转入 DEFER-SP-DOCS-01，独立 patch 发布转入 DEFER-SP-REL-01，均保留为未执行的后续交付项，不作为本次修复卡关闭条件。

验收证据为 PR head `bb88db39feb584792821cfd2589a50ff7651123d`：base run [40910569542](https://github.com/libra-tools/libra/actions/runs/40910569542) 与 CodeQL run [40910569525](https://github.com/libra-tools/libra/actions/runs/40910569525) 均 success；全量 nextest 10841/10841 passed（577.819s、13 slow、23 skipped），其它适用 PR 检查均 SUCCESS。本次后续状态更新只改文档；新 head 的 CI 仍须单独报告，不把旧 head 的运行说成新运行。原本机 16 GiB 的 not-run 记录继续保留，CI 成功不改写本机历史。

### SP-00 独立文档验收决策（EX-SP-00-AUDIT-01）

2026-10-29，用户接受“取消 SP-00 对 SP-01 发布验收的继承，保留 SP-01 对基线的执行依赖”的建议，并明确授权修改、提交和推送。本决策仅针对 SP-00，作为模板 ER-04 对 no-release 卡必须继承发布点 C/D 的具名例外；不修改模板，不扩展至 SP-09 或任何实现卡。

- SP-00 的产物仅为本计划中的基线协议、测量记录和审计修正，以及状态索引；独立验证这些产物不需要 SP-01 的生产修复或二进制发布。继续保留 no-code / no-config、no-release、版本递增 N/A，不 bump、不安装、不创建 tag/Release。
- 本卡自行承担文档验收：四项 AC、两项 Verification、文档结构/内部链接与变更范围检查、Codex review PASS、最终文档状态下的 `cargo +nightly fmt --all --check` 和 `cargo clippy --all-targets --all-features -- -D warnings`；不以本决策免除任何现有文档判据或推送前静态门。
- 本卡 D 门为 PR #652 最终 head 上 `.github/workflows/base.yml`（Check, Build and Test）及 `.github/workflows/codeql.yml`（CodeQL Advanced）的全部适用检查成功；`neutral` 的汇总检查不替代底层 CodeQL 成功。随后合并 PR，核验文档进入上游 main，才可登记 `done / complete` 并关闭 #639。检查未完成时保留 `in-progress / locally-accepted`；失败则修复后重验，不以 mergeable 代替 CI 通过。
- 本卡不执行 C 组的版本 bump、release build、安装及发布步骤，不继承 SP-01 的全量测试或发布后置门；基准采样不因归属调整而重跑，历史实测与本轮文档核对继续区分。
- SP-01 的 `Dependencies: SP-00, DEP-SP-DOCS`、实现范围、回归测试、独立 patch 发布及完整 C/D 门保持原样。它消费的是本卡已冻结并通过文档验收的基线；不得再以 SP-01 的未来发布反向阻止本卡收口。
- 恢复：如撤销本决策，恢复 SP-00 的 `C/D coverage from: SP-01` 及原状态条件；不能回写历史 CI 或发布结果。

## 实施顺序与发布窗口

核心串行顺序：`SP-00 → SP-01 → SP-02 → SP-03 → SP-05 → SP-06 → SP-07 → SP-08 → SP-09`。

可选：`SP-09 + DEP-SP-OPT → SP-04`。这些顺序边既含产物依赖，也为共享源码/测试写集串行化；不是声称所有相邻卡存在算法上的前置关系。公开行为卡另依赖 DEP-SP-DOCS；SP-00 消费 DEP-SP-DATA 的可用性决策。无多 Agent 并发，发布者待实施授权后指定，任一时刻仅一个发布窗口。

Phase 0 基线：SP-00；Phase 1 正确性与 stat：SP-01..03；Phase 2 核心重复工作：SP-05..08；Phase 3 验收决策：SP-09；后续可选 SP-04。实现卡各自独立发布，禁止将 SP-09 当成替其它卡补做版本/验收的发布点。

## 任务总览

| 卡 | 优先级 | 选择 | Issue | Lifecycle / Acceptance |
|---|---|---|---|---|
| SP-00 冻结 status 性能基线与修正审计口径 | P1 | 必选验收前置 | [#639](https://github.com/libra-tools/libra/issues/639) | done / complete |
| SP-01 status 递归子树加载返回错误而非 panic | P1 | 必选正确性 | [#640](https://github.com/libra-tools/libra/issues/640) | done / complete |
| SP-02 restore 区分 decoded size 与对象加载内存成本 | P1 | 必选正确性 | [#641](https://github.com/libra-tools/libra/issues/641) | pending / 空 |
| SP-03 materialization 后安全写入真实 index stat | P1 | 本轮核心提速 | [#642](https://github.com/libra-tools/libra/issues/642) | pending / 空 |
| SP-04 单次 status 复用同一 index 快照 | P2 | 可选建议项，默认未启用 | [#643](https://github.com/libra-tools/libra/issues/643) | pending / 空 |
| SP-05 status 共享一次 HEAD tree 展开结果 | P2 | 本轮核心提速：第一梯队 | [#644](https://github.com/libra-tools/libra/issues/644) | pending / 空 |
| SP-06 对象读取复用请求级 storage scope | P2 | 本轮核心提速：第二梯队 | [#645](https://github.com/libra-tools/libra/issues/645) | pending / 空 |
| SP-07 pack index 整块 fanout 与二分查找 | P2 | 本轮核心提速：第一梯队 | [#646](https://github.com/libra-tools/libra/issues/646) | pending / 空 |
| SP-08 walk 级 ignore 上下文与 tracked 短路 | P2 | 本轮核心提速：第二梯队 | [#648](https://github.com/libra-tools/libra/issues/648) | pending / 空 |
| SP-09 核心优化复测、全量收口与可选项决策 | P2 | 必选验收收口 | [#647](https://github.com/libra-tools/libra/issues/647) | pending / 空 |

## 任务卡

### Task SP-00: 冻结 status 性能基线与修正审计口径

**Task type:** audit

**Lifecycle / Acceptance:** done / complete

**Priority / Selection:** P1；必选验收前置

**Issue:** [#639](https://github.com/libra-tools/libra/issues/639)

**Description:** 建立可重放的性能与语义对照协议；纠正旧审计的过强结论，不把文档中的历史统计当作本轮实测。

**Out of scope:** CE_VALID 永久不在本计划范围；其它行为轴按任务总览分别承接；pack 会话缓存/marker 合并/IPC 批处理登记为 DEFER-SP-01..03，未启用。

**Current evidence:** LIBRA_STATUS_PERFORMANCE_REVIEW.md:72；LIBRA_STATUS_PERFORMANCE_REVIEW_AUDIT.md:168,221,228；benchmark/run.sh；benchmark/README.md（基线 380c262，开工刷新锚点）。

**Acceptance criteria:**

- [x] 在本计划记录实际基线源码修订、Git/Libra 二进制标识、操作系统、测试数据形状与样本协议。

- [x] 记录默认/刷新索引 × 显示/不显示未跟踪文件 × 默认重命名/关闭重命名 × 机器可读 v1/v2 的 Git/Libra 对照命令矩阵。

- [x] 逐项修正审计中的 242/206 次发现混用、线程切换下界与零风险预热结论。

- [x] 记录每个已采样格子的输出、退出状态与索引不变量；缺失格子明确标未测。


**Verification:**

- [x] 人工核对：本计划基线表的每格均有准确命令、源码修订、样本数及 measured/not-run 状态。

- [x] 人工核对：两份报告的历史统计与本轮实测分栏，未访问的外部归档不标独立复算。


**Full-suite trigger:** N/A：纯文档 audit；按 EX-SP-00-AUDIT-01 独立验收，不执行版本/二进制发布 C 组

**Dependencies:** DEP-SP-DATA

**Deliverables:** 本计划正文与 plan-status.md；不产生新证据文件

**Implementation write set:** `docs/development/plan/plan-20261028-status-performance.md`, `docs/development/plan/plan-status.md`

**Release write set:** N/A

**Docs and compatibility impact:** N/A：仅规划/审计记录，不改变产品行为。

**Rollback mode:** revert

**Migration and rollback:** 无 schema 迁移；撤销本卡文档变更即可；远端 Issue 保留讨论历史，不自动删除。

**Security and privacy:** 保留既有路径边界、错误隔离与超时；只操作隔离 fixture，禁止读取/修改真实全局配置或外传 secrets。

**Performance budget:** 无未经实测的绝对毫秒承诺；按 SP-00 的同环境前后配对协议报告中位数、尾延迟及工作量；语义退化即失败，噪声内变化记不确定。

**Estimated scope:** S

**Version increment:** N/A

**Release boundary:** no-release；本卡文档随 PR #652 独立合入上游 main，不等待 SP-01 发版

**C/D coverage from:** self（EX-SP-00-AUDIT-01）；自行通过文档、评审、fmt/Clippy 与 PR #652 最终 head 的 base/CodeQL 门，不继承 SP-01 发布。PR #652 最终 head `5e395e0` 的全部检查成功；2026-10-29T15:47:49Z 合并为 `d0578dbe6dd3d484521db4cf60daff736256dc59`，文档已进入上游 main。Issue #639 仍 open，状态关闭未同步。

**Granularity:** type=audit; axis=冻结 status 性能基线与修正审计口径; recovery=回退文档; complete=yes; self-contained=yes; AC=4/20; VER=2/20; landing=0; prod-files=0; scope=S; deps=DEP-SP-DATA; writeset=按实施顺序串行; release=no-release; split-from=N/A; exception=EX-SP-00-AUDIT-01

### Task SP-01: status 递归子树加载返回错误而非 panic

**Task type:** implementation

**Lifecycle / Acceptance:** done / complete

**Priority / Selection:** P1；必选正确性

**Issue:** [#640](https://github.com/libra-tools/libra/issues/640)

**Description:** 为 status 的 HEAD 递归展开提供可失败的读取路径，缺失或损坏子树返回上下文错误，禁止退化为空树或 clean。

**Out of scope:** CE_VALID 永久不在本计划范围；其它行为轴按任务总览分别承接；pack 会话缓存/marker 合并/IPC 批处理登记为 DEFER-SP-01..03，未启用。

**Current evidence:** src/utils/object_ext.rs:54,111；src/command/status.rs:1030-1039；src/command/status/scan.rs:1327；src/command/status/output.rs:1361（基线 380c262，开工刷新锚点）。

**Acceptance criteria:**

- [x] 缺失子树返回 LBR-REPO-002 而非 panic。

- [x] 损坏子树返回包含受影响 OID 的可操作错误。

- [x] human/JSON/porcelain 错误输出遵守现有格式与退出契约。

- [x] 合法含 gitlink 的 tree 不被当成缺失子树。

- [x] ER-06a：本仓库 EN/zh-CN status、COMPATIBILITY 与错误码文档同步并验收；网站按用户决定登记 DEFER-SP-DOCS-01，仍未执行，依 EX-SP-01-CLOSE-01 不计本次修复卡完成门。


**Verification:**

- [x] `cargo test --test command_test status_error_test::`

- [x] `cargo test --test command_test status_error_test::status_nested_tree_ -- --nocapture` — `(new)` 测试前缀；须非零命中。


**Full-suite trigger:** T-1：共享 tree 读取辅助逻辑

**Dependencies:** SP-00（已验收）；DEP-SP-DOCS 的网站交付按 EX-SP-01-CLOSE-01 转 DEFER-SP-DOCS-01，仍未执行、不阻塞本次修复卡收口

**Deliverables:** 实现、同卡回归及列明文档

**Implementation write set:** `src/utils/object_ext.rs`, `src/command/status.rs`, `src/command/status/scan.rs`, `src/command/status/output.rs`, `tests/command/status_error_test.rs`, `docs/commands/status.md`, `docs/commands/zh-CN/status.md`, `COMPATIBILITY.md`, `docs/error-codes.md`, `/Volumes/Sea/libra-backend/apps/tanstack-app/content/docs/commands/status.en.md`

**Release write set:** Inherited

**Docs and compatibility impact:** `docs/commands/status.md`；`docs/commands/zh-CN/status.md`；`COMPATIBILITY.md`；`docs/error-codes.md`；`/Volumes/Sea/libra-backend/apps/tanstack-app/content/docs/commands/status.en.md`。网站路径仍待核实，后续交付由 DEFER-SP-DOCS-01 跟踪；本仓库四份文档已验收。

**Rollback mode:** immutable-release

**Migration and rollback:** 无 schema 迁移；发布前回退本卡，发布后前滚修复；不把版本回退视为历史 index 修复。

**Security and privacy:** 保留既有路径边界、错误隔离与超时；只操作隔离 fixture，禁止读取/修改真实全局配置或外传 secrets。

**Performance budget:** 无未经实测的绝对毫秒承诺；按 SP-00 的同环境前后配对协议报告中位数、尾延迟及工作量；语义退化即失败，噪声内变化记不确定。

**Estimated scope:** M

**Version increment:** patch（DEFER-SP-REL-01：后续独立发布时执行；本次修复卡收口不 bump）

**Release boundary:** independent（发布交付在 DEFER-SP-REL-01 单独跟踪；本次修复卡按 EX-SP-01-CLOSE-01 收口）

**C/D coverage from:** self（EX-SP-01-CLOSE-01 的修复/PR CI 完成口径）；本地 18 个不同用例、fmt/严格 Clippy 与本仓库文档检查通过；bb88db3 的 PR base/CodeQL 全绿，全量 nextest 10841 passed、23 skipped。发布构建/安装/tag/Release/CDN 等仍未执行，登记 DEFER-SP-REL-01，不称完整发布 C/D 通过。

**Granularity:** type=implementation; axis=status 递归子树加载返回错误而非 panic; recovery=前滚修复版; complete=yes; self-contained=yes; AC=4/8; VER=2/8; landing=3; prod-files=4; scope=M; deps=SP-00,DEP-SP-DOCS; writeset=按实施顺序串行; release=independent; split-from=N/A; exception=EX-SP-01-CLOSE-01

### Task SP-02: restore 区分 decoded size 与对象加载内存成本

**Task type:** implementation

**Lifecycle / Acceptance:** pending / 空

**Priority / Selection:** P1；必选正确性

**Issue:** [#641](https://github.com/libra-tools/libra/issues/641)

**Description:** 补充语义明确的 decoded-size 查询用于 restore index size；保留原 object_size(s) 的保守加载预算语义，避免预览安全限额被削弱。

**Out of scope:** CE_VALID 永久不在本计划范围；其它行为轴按任务总览分别承接；pack 会话缓存/marker 合并/IPC 批处理登记为 DEFER-SP-01..03，未启用。

**Current evidence:** src/command/restore.rs:730-759,1831；src/utils/storage/mod.rs:241；src/utils/storage/load_cost/pack.rs:376（基线 380c262，开工刷新锚点）。

**Acceptance criteria:**

- [ ] 普通与 OFS/REF delta 对象产生真实 decoded size。

- [ ] 多层 delta 查询不把 peak_bytes 写入 index size。

- [ ] 查询失败或缓存未命中的 fallback 保留原错误语义。

- [ ] 原有 preview load-cost 限额仍能拒绝超预算重建。

- [ ] staged-only 不读取工作区 stat 来伪造 clean。

- [ ] ER-06a：文档清单逐文件同步并验收（全局必需门，不计入特有 AC）。


**Verification:**

- [ ] `cargo test --lib utils::storage::`

- [ ] `cargo test --test command_test restore_test::`

- [ ] `cargo test --test command_test restore_test::restore_delta_index_size_ -- --nocapture` — `(new)` 测试前缀；须非零命中。


**Full-suite trigger:** T-1：共享存储接口及预算逻辑

**Dependencies:** SP-01, DEP-SP-DOCS

**Deliverables:** 实现、同卡回归及列明文档

**Implementation write set:** `src/command/restore.rs`, `src/utils/client_storage.rs`, `src/utils/storage/mod.rs`, `src/utils/storage/local.rs`, `src/utils/storage/load_cost/pack.rs`, `tests/command/restore_test.rs`, `docs/commands/restore.md`, `docs/commands/zh-CN/restore.md`, `/Volumes/Sea/libra-backend/apps/tanstack-app/content/docs/commands/restore.en.md`

**Release write set:** Inherited

**Docs and compatibility impact:** `docs/commands/restore.md`；`docs/commands/zh-CN/restore.md`；`/Volumes/Sea/libra-backend/apps/tanstack-app/content/docs/commands/restore.en.md`。网站候选路径待 DEP-SP-DOCS 核实。

**Rollback mode:** immutable-release

**Migration and rollback:** 无 schema 迁移；发布前回退本卡，发布后前滚修复；不把版本回退视为历史 index 修复。

**Security and privacy:** 保留既有路径边界、错误隔离与超时；只操作隔离 fixture，禁止读取/修改真实全局配置或外传 secrets。

**Performance budget:** 无未经实测的绝对毫秒承诺；按 SP-00 的同环境前后配对协议报告中位数、尾延迟及工作量；语义退化即失败，噪声内变化记不确定。

**Estimated scope:** M

**Version increment:** patch

**Release boundary:** independent

**C/D coverage from:** self；尚未运行。

**Granularity:** type=implementation; axis=restore 区分 decoded size 与对象加载内存成本; recovery=前滚修复版; complete=yes; self-contained=yes; AC=5/8; VER=3/8; landing=4; prod-files=5; scope=M; deps=SP-01,DEP-SP-DOCS; writeset=按实施顺序串行; release=independent; split-from=N/A; exception=N/A

### Task SP-03: materialization 后安全写入真实 index stat

**Task type:** implementation

**Lifecycle / Acceptance:** pending / 空

**Priority / Selection:** P1；本轮核心提速

**Issue:** [#642](https://github.com/libra-tools/libra/issues/642)

**Description:** 仅在实际写入 worktree 并更新 index 的受控路径建立可信 stat 缓存，减少 fresh clone 的无效 hash；不把 staged-only index 与工作区内容混为一谈。

**Out of scope:** CE_VALID 永久不在本计划范围；其它行为轴按任务总览分别承接；pack 会话缓存/marker 合并/IPC 批处理登记为 DEFER-SP-01..03，未启用。

**Current evidence:** src/command/restore.rs:1291,1654,1831；src/command/clone.rs:2041；src/command/status_untracked.rs:194；src/command/refresh.rs:218（基线 380c262，开工刷新锚点）。

**Acceptance criteria:**

- [ ] worktree+staged 落盘后普通文件 size/stat 与可靠文件快照一致。

- [ ] staged-only 后的未暂存修改仍能被 status 检出。

- [ ] hash/写入期间的并发修改不能形成错误 clean 缓存。

- [ ] racy-clean 文件仍触发必要的内容校验。

- [ ] LFS smudge 后 index size 使用实际工作区长度。

- [ ] symlink 使用 no-follow 元数据语义。

- [ ] 既有 mode 与 flags 的合法语义保持不变；不处理 CE_VALID 策略。

- [ ] ER-06a：文档清单逐文件同步并验收（全局必需门，不计入特有 AC）。


**Verification:**

- [ ] `cargo test --test command_test restore_test::`

- [ ] `cargo test --test command_test clone_test::`

- [ ] `cargo test --test command_test restore_test::restore_stat_snapshot_ -- --nocapture` — `(new)` 测试前缀；须非零命中。


**Full-suite trigger:** none

**Dependencies:** SP-02, DEP-SP-DOCS

**Deliverables:** 实现、同卡回归及列明文档

**Implementation write set:** `src/command/restore.rs`, `src/command/refresh.rs`, `tests/command/restore_test.rs`, `tests/command/clone_test.rs`, `docs/commands/clone.md`, `docs/commands/zh-CN/clone.md`, `docs/commands/restore.md`, `docs/commands/zh-CN/restore.md`, `/Volumes/Sea/libra-backend/apps/tanstack-app/content/docs/commands/clone.en.md`, `/Volumes/Sea/libra-backend/apps/tanstack-app/content/docs/commands/restore.en.md`

**Release write set:** Inherited

**Docs and compatibility impact:** `docs/commands/clone.md`；`docs/commands/zh-CN/clone.md`；`docs/commands/restore.md`；`docs/commands/zh-CN/restore.md`；`/Volumes/Sea/libra-backend/apps/tanstack-app/content/docs/commands/clone.en.md`；`/Volumes/Sea/libra-backend/apps/tanstack-app/content/docs/commands/restore.en.md`。网站候选路径待 DEP-SP-DOCS 核实。

**Rollback mode:** immutable-release

**Migration and rollback:** 无 schema 迁移；发布前回退本卡，发布后前滚修复；不把版本回退视为历史 index 修复。

**Security and privacy:** 保留既有路径边界、错误隔离与超时；只操作隔离 fixture，禁止读取/修改真实全局配置或外传 secrets。

**Performance budget:** 无未经实测的绝对毫秒承诺；按 SP-00 的同环境前后配对协议报告中位数、尾延迟及工作量；语义退化即失败，噪声内变化记不确定。

**Estimated scope:** M

**Version increment:** patch

**Release boundary:** independent

**C/D coverage from:** self；尚未运行。

**Granularity:** type=implementation; axis=materialization 后安全写入真实 index stat; recovery=前滚修复版; complete=yes; self-contained=yes; AC=7/8; VER=3/8; landing=1; prod-files=2; scope=M; deps=SP-02,DEP-SP-DOCS; writeset=按实施顺序串行; release=independent; split-from=N/A; exception=N/A

### Task SP-04: 单次 status 复用同一 index 快照

**Task type:** implementation

**Lifecycle / Acceptance:** pending / 空

**Priority / Selection:** P2；可选建议项，默认未启用

**Issue:** [#643](https://github.com/libra-tools/libra/issues/643)

**Description:** 在 status 入口获取一致的 index 内容与文件版本信息，供 staged/unstaged 和输出共同消费；循环复用 entry 引用。

**Out of scope:** CE_VALID 永久不在本计划范围；其它行为轴按任务总览分别承接；pack 会话缓存/marker 合并/IPC 批处理登记为 DEFER-SP-01..03，未启用。

**Current evidence:** src/command/status.rs:999；src/command/status_untracked.rs:84,93,235；src/command/status/scan.rs:101（基线 380c262，开工刷新锚点）。

**Acceptance criteria:**

- [ ] 单次 status 的 staged 与 unstaged 使用同一 index 快照。

- [ ] racy 判定使用与该 index 版本对应的 mtime，不在读盘后另取可能已替换路径的 mtime。

- [ ] 并发原子替换 index 不产生两份 index 混合的 staged/unstaged 结果。

- [ ] 损坏 index 仍保留既有错误行为。


**Verification:**

- [ ] `cargo test --test command_test status_test::`

- [ ] `cargo test --test command_test status_test::status_index_snapshot_ -- --nocapture` — `(new)` 测试前缀；须非零命中。


**Full-suite trigger:** none

**Dependencies:** SP-09, DEP-SP-OPT

**Deliverables:** 实现、同卡回归及列明文档

**Implementation write set:** `src/command/status.rs`, `src/command/status_untracked.rs`, `src/command/status/scan.rs`, `src/command/status/output.rs`, `tests/command/status_test.rs`

**Release write set:** Inherited

**Docs and compatibility impact:** N/A：保持现有 CLI 参数、输出、错误、配置及存储格式，仅消除内部重复工作；若实现改变契约必须先更新本字段和逐文件写集。

**Rollback mode:** immutable-release

**Migration and rollback:** 无 schema 迁移；发布前回退本卡，发布后前滚修复；不把版本回退视为历史 index 修复。

**Security and privacy:** 保留既有路径边界、错误隔离与超时；只操作隔离 fixture，禁止读取/修改真实全局配置或外传 secrets。

**Performance budget:** 无未经实测的绝对毫秒承诺；按 SP-00 的同环境前后配对协议报告中位数、尾延迟及工作量；语义退化即失败，噪声内变化记不确定。

**Estimated scope:** M

**Version increment:** patch

**Release boundary:** independent

**C/D coverage from:** self；尚未运行。

**Granularity:** type=implementation; axis=单次 status 复用同一 index 快照; recovery=前滚修复版; complete=yes; self-contained=yes; AC=4/8; VER=2/8; landing=2; prod-files=4; scope=M; deps=SP-09,DEP-SP-OPT; writeset=按实施顺序串行; release=independent; split-from=N/A; exception=N/A

### Task SP-05: status 共享一次 HEAD tree 展开结果

**Task type:** implementation

**Lifecycle / Acceptance:** pending / 空

**Priority / Selection:** P2；本轮核心提速：第一梯队

**Issue:** [#644](https://github.com/libra-tools/libra/issues/644)

**Description:** 在单次 status 中将 HEAD tree 展开结果共享给 staged 比较、rename 及 porcelain v2 构造，避免干净仓库默认多次展开。

**Out of scope:** CE_VALID 永久不在本计划范围；其它行为轴按任务总览分别承接；pack 会话缓存/marker 合并/IPC 批处理登记为 DEFER-SP-01..03，未启用。

**Current evidence:** src/command/status/scan.rs:101,177；src/command/status.rs:1039；src/command/status/output.rs:1361（基线 380c262，开工刷新锚点）。

**Acceptance criteria:**

- [ ] 有 HEAD 时单次 status 最多展开一次 HEAD tree。

- [ ] 默认 rename 与 --no-renames 的原分类行为保持一致。

- [ ] porcelain v1/v2 与 JSON 输出保持既有语义。

- [ ] unborn HEAD 和子树读取失败遵循既有错误处理。

- [ ] 在相同 packed fixture 上记录前后对象读取次数和无插桩墙钟。


**Verification:**

- [ ] `cargo test --test command_test status_test::`

- [ ] `cargo test --test command_test status_wave0_test::`

- [ ] `cargo test --test command_test status_test::status_head_expansion_ -- --nocapture` — `(new)` 测试前缀；须非零命中。


**Full-suite trigger:** none

**Dependencies:** SP-03

**Deliverables:** 实现、同卡回归及列明文档

**Implementation write set:** `src/command/status.rs`, `src/command/status/scan.rs`, `src/command/status/output.rs`, `tests/command/status_test.rs`, `tests/command/status_wave0_test.rs`

**Release write set:** Inherited

**Docs and compatibility impact:** N/A：保持现有 CLI 参数、输出、错误、配置及存储格式，仅消除内部重复工作；若实现改变契约必须先更新本字段和逐文件写集。

**Rollback mode:** immutable-release

**Migration and rollback:** 无 schema 迁移；发布前回退本卡，发布后前滚修复；不把版本回退视为历史 index 修复。

**Security and privacy:** 保留既有路径边界、错误隔离与超时；只操作隔离 fixture，禁止读取/修改真实全局配置或外传 secrets。

**Performance budget:** 无未经实测的绝对毫秒承诺；按 SP-00 的同环境前后配对协议报告中位数、尾延迟及工作量；语义退化即失败，噪声内变化记不确定。

**Estimated scope:** M

**Version increment:** patch

**Release boundary:** independent

**C/D coverage from:** self；尚未运行。

**Granularity:** type=implementation; axis=status 共享一次 HEAD tree 展开结果; recovery=前滚修复版; complete=yes; self-contained=yes; AC=5/8; VER=3/8; landing=2; prod-files=3; scope=M; deps=SP-03; writeset=按实施顺序串行; release=independent; split-from=N/A; exception=N/A

### Task SP-06: 对象读取复用请求级 storage scope

**Task type:** implementation

**Lifecycle / Acceptance:** pending / 空

**Priority / Selection:** P2；本轮核心提速：第二梯队

**Issue:** [#645](https://github.com/libra-tools/libra/issues/645)

**Description:** 让 status 对象读取消费已 pin 的仓库身份，避免每个子树读取前重新从 cwd 发现仓库；优先显式传递 storage，不顺带重写 database/hooks 全局路径。

**Out of scope:** CE_VALID 永久不在本计划范围；其它行为轴按任务总览分别承接；pack 会话缓存/marker 合并/IPC 批处理登记为 DEFER-SP-01..03，未启用。

**Current evidence:** src/utils/path.rs:46；src/utils/util.rs:1136,1364；src/utils/object_ext.rs:54（基线 380c262，开工刷新锚点）。

**Acceptance criteria:**

- [ ] 同一请求子树数量增长不再线性增加仓库发现次数。

- [ ] linked worktree 对象读取仍指向正确 common storage。

- [ ] 并发不同仓库请求不串用 storage。

- [ ] 无 pinned scope 的合法调用保留明确 fallback 行为。

- [ ] 配置/后端读取失败不被缓存成成功或静默回退。


**Verification:**

- [ ] `cargo test --lib utils::`

- [ ] `cargo test --test command_test status_test::status_storage_scope_ -- --nocapture` — `(new)` 测试前缀；须非零命中。


**Full-suite trigger:** T-1：共享路径与对象读取上下文

**Dependencies:** SP-05, DEP-SP-STORAGE

**Deliverables:** 实现、同卡回归及列明文档

**Implementation write set:** `src/utils/path.rs`, `src/utils/util.rs`, `src/utils/object_ext.rs`, `src/command/status/scan.rs`, `tests/command/status_test.rs`

**Release write set:** Inherited

**Docs and compatibility impact:** N/A：保持现有 CLI 参数、输出、错误、配置及存储格式，仅消除内部重复工作；若实现改变契约必须先更新本字段和逐文件写集。

**Rollback mode:** immutable-release

**Migration and rollback:** 无 schema 迁移；发布前回退本卡，发布后前滚修复；不把版本回退视为历史 index 修复。

**Security and privacy:** 保留既有路径边界、错误隔离与超时；只操作隔离 fixture，禁止读取/修改真实全局配置或外传 secrets。

**Performance budget:** 无未经实测的绝对毫秒承诺；按 SP-00 的同环境前后配对协议报告中位数、尾延迟及工作量；语义退化即失败，噪声内变化记不确定。

**Estimated scope:** M

**Version increment:** patch

**Release boundary:** independent

**C/D coverage from:** self；尚未运行。

**Granularity:** type=implementation; axis=对象读取复用请求级 storage scope; recovery=前滚修复版; complete=yes; self-contained=yes; AC=5/8; VER=2/8; landing=2; prod-files=4; scope=M; deps=SP-05,DEP-SP-STORAGE; writeset=按实施顺序串行; release=independent; split-from=N/A; exception=N/A

### Task SP-07: pack index 整块 fanout 与二分查找

**Task type:** implementation

**Lifecycle / Acceptance:** pending / 空

**Priority / Selection:** P2；本轮核心提速：第一梯队

**Issue:** [#646](https://github.com/libra-tools/libra/issues/646)

**Description:** 将普通单对象 pack index 查询改为整块 fanout 读取与有界二分查找，先减少小 read 与线性扫描，不在本卡增加跨请求缓存。

**Out of scope:** CE_VALID 永久不在本计划范围；其它行为轴按任务总览分别承接；pack 会话缓存/marker 合并/IPC 批处理登记为 DEFER-SP-01..03，未启用。

**Current evidence:** src/utils/storage/local.rs:1440,1465,1535,1827（基线 380c262，开工刷新锚点）。

**Acceptance criteria:**

- [ ] v1/v2 查询返回与原实现相同的对象 offset。

- [ ] SHA-1/SHA-256 的对象名与 offset 解码正确。

- [ ] 截断或损坏索引返回错误而非 panic 或错误对象。

- [ ] repack/索引替换不读取已失效的复用状态。

- [ ] 固定 packed fixture 的 read 次数显著低于旧逐项/线性读取且报告真实墙钟。


**Verification:**

- [ ] `cargo test --lib utils::storage::local::`

- [ ] `cargo test --lib utils::storage::local::status_pack_lookup_ -- --nocapture` — `(new)` 测试前缀；须非零命中。


**Full-suite trigger:** T-1：共享对象读取路径

**Dependencies:** SP-06

**Deliverables:** 实现、同卡回归及列明文档

**Implementation write set:** `src/utils/storage/local.rs`

**Release write set:** Inherited

**Docs and compatibility impact:** N/A：保持现有 CLI 参数、输出、错误、配置及存储格式，仅消除内部重复工作；若实现改变契约必须先更新本字段和逐文件写集。

**Rollback mode:** immutable-release

**Migration and rollback:** 无 schema 迁移；发布前回退本卡，发布后前滚修复；不把版本回退视为历史 index 修复。

**Security and privacy:** 保留既有路径边界、错误隔离与超时；只操作隔离 fixture，禁止读取/修改真实全局配置或外传 secrets。

**Performance budget:** 无未经实测的绝对毫秒承诺；按 SP-00 的同环境前后配对协议报告中位数、尾延迟及工作量；语义退化即失败，噪声内变化记不确定。

**Estimated scope:** S

**Version increment:** patch

**Release boundary:** independent

**C/D coverage from:** self；尚未运行。

**Granularity:** type=implementation; axis=pack index 整块 fanout 与二分查找; recovery=前滚修复版; complete=yes; self-contained=yes; AC=5/8; VER=2/8; landing=1; prod-files=1; scope=S; deps=SP-06; writeset=按实施顺序串行; release=independent; split-from=N/A; exception=N/A

### Task SP-08: walk 级 ignore 上下文与 tracked 短路

**Task type:** implementation

**Lifecycle / Acceptance:** pending / 空

**Priority / Selection:** P2；本轮核心提速：第二梯队

**Issue:** [#648](https://github.com/libra-tools/libra/issues/648)

**Description:** 在实际进入的每个目录建立完整 ignore 上下文后跳过 tracked 文件的无效匹配，消除重复发现与不必要调度；保持 unreadable/timeout 和路径安全契约。

**Out of scope:** CE_VALID 永久不在本计划范围；其它行为轴按任务总览分别承接；pack 会话缓存/marker 合并/IPC 批处理登记为 DEFER-SP-01..03，未启用。

**Current evidence:** src/utils/util.rs:3181,3365,3504；src/command/status_untracked.rs:368,473,588（基线 380c262，开工刷新锚点）。

**Acceptance criteria:**

- [ ] 每个实际进入目录的自身 ignore 文件被纳入检查，不能仅预热根目录。

- [ ] 根/深层 ignore 读取失败仍产生约定 io_blocked。

- [ ] 逐目录、info/exclude、全局 excludes 的优先级不变。

- [ ] tracked 文件短路不遗漏所需错误报告。

- [ ] 真正 untracked 文件的输出与原规则一致。

- [ ] 目录替换及 symlink 不能把规则读取引向工作区外。

- [ ] 同一 walk 的重复上下文发现与文件级调度减少并有测量证据。


**Verification:**

- [ ] `cargo test --test command_test status_test::`

- [ ] `cargo test --test command_test status_wave0_test::`

- [ ] `cargo test --test command_test status_test::status_ignore_context_ -- --nocapture` — `(new)` 测试前缀；须非零命中。


**Full-suite trigger:** T-1：共享 ignore 解析辅助逻辑

**Dependencies:** SP-07

**Deliverables:** 实现、同卡回归及列明文档

**Implementation write set:** `src/utils/util.rs`, `src/command/status_untracked.rs`, `tests/command/status_test.rs`, `tests/command/status_wave0_test.rs`

**Release write set:** Inherited

**Docs and compatibility impact:** N/A：保持现有 CLI 参数、输出、错误、配置及存储格式，仅消除内部重复工作；若实现改变契约必须先更新本字段和逐文件写集。

**Rollback mode:** immutable-release

**Migration and rollback:** 无 schema 迁移；发布前回退本卡，发布后前滚修复；不把版本回退视为历史 index 修复。

**Security and privacy:** 保留既有路径边界、错误隔离与超时；只操作隔离 fixture，禁止读取/修改真实全局配置或外传 secrets。

**Performance budget:** 无未经实测的绝对毫秒承诺；按 SP-00 的同环境前后配对协议报告中位数、尾延迟及工作量；语义退化即失败，噪声内变化记不确定。

**Estimated scope:** M

**Version increment:** patch

**Release boundary:** independent

**C/D coverage from:** self；尚未运行。

**Granularity:** type=implementation; axis=walk 级 ignore 上下文与 tracked 短路; recovery=前滚修复版; complete=yes; self-contained=yes; AC=7/8; VER=3/8; landing=2; prod-files=2; scope=M; deps=SP-07; writeset=按实施顺序串行; release=independent; split-from=N/A; exception=N/A

### Task SP-09: 核心优化复测、全量收口与可选项决策

**Task type:** audit

**Lifecycle / Acceptance:** pending / 空

**Priority / Selection:** P2；必选验收收口

**Issue:** [#647](https://github.com/libra-tools/libra/issues/647)

**Description:** 在核心修复完成后按冻结矩阵验证实际收益和语义，执行最终全量门，决定是否值得启用后续一致性或 IPC/cache 工作。

**Out of scope:** CE_VALID 永久不在本计划范围；其它行为轴按任务总览分别承接；pack 会话缓存/marker 合并/IPC 批处理登记为 DEFER-SP-01..03，未启用。

**Current evidence:** 本计划 SP-00 的基线矩阵；LIBRA_STATUS_PERFORMANCE_REVIEW.md:200-214（基线 380c262，开工刷新锚点）。

**Acceptance criteria:**

- [ ] 已冻结矩阵逐格有前后可比无插桩计时，缺失数据不包装成收益。

- [ ] 语义验证覆盖 clean/dirty、rename、ignore、错误与 index 不变量。

- [ ] 报告新增风险和残余热点，不将 syscall 占比等同墙钟占比。

- [ ] SP-04 与三个 DEFER 项分别记录 adopt/defer 决策及依据。

- [ ] 最终全量门失败全部处置并重跑，未通过时不宣称本轮完成。


**Verification:**

- [ ] 人工核对：基线矩阵每个性能结论能追溯到相同环境/fixture/选项的前后计时与样本数。

- [ ] 人工核对：SP-01/02/03/05/06/07/08 的 focused、fmt/clippy、适用全量和 C/D 证据逐项存在。


**Full-suite trigger:** T-2：核心轮次聚合与最终全量门

**Dependencies:** SP-08

**Deliverables:** 本计划正文与 plan-status.md；不产生新证据文件

**Implementation write set:** `docs/development/plan/plan-20261028-status-performance.md`, `docs/development/plan/plan-status.md`

**Release write set:** N/A

**Docs and compatibility impact:** N/A：仅规划/审计记录，不改变产品行为。

**Rollback mode:** revert

**Migration and rollback:** 无 schema 迁移；撤销本卡文档变更即可；远端 Issue 保留讨论历史，不自动删除。

**Security and privacy:** 保留既有路径边界、错误隔离与超时；只操作隔离 fixture，禁止读取/修改真实全局配置或外传 secrets。

**Performance budget:** 无未经实测的绝对毫秒承诺；按 SP-00 的同环境前后配对协议报告中位数、尾延迟及工作量；语义退化即失败，噪声内变化记不确定。

**Estimated scope:** S

**Version increment:** N/A

**Release boundary:** no-release；规划记录随实现交付进入仓库

**C/D coverage from:** SP-08；尚未运行。

**Granularity:** type=audit; axis=核心优化复测、全量收口与可选项决策; recovery=回退文档; complete=yes; self-contained=yes; AC=5/20; VER=2/20; landing=0; prod-files=0; scope=S; deps=SP-08; writeset=按实施顺序串行; release=no-release; split-from=N/A; exception=N/A

## 延后项（不创建实施任务、不阻塞本轮）

| ID | 事项 | 启用条件 | 不做的理由 |
|---|---|---|---|
| DEFER-SP-DOCS-01 | SP-01 网站 status 文档同步、验证和部署；未执行 | 网站仓库可用且用户恢复交付时，按原 DEP-SP-DOCS 验收 | 用户先前明确暂搁；不伪称与本仓库文档一并完成 |
| DEFER-SP-REL-01 | SP-01 独立 patch bump、发布构建/安装、tag/Release 及发布后置门；未执行 | 用户授权发布并协调资格修复占用的 v0.30.92 后，确定后续版本并执行原发布门 | 本次指令仅关闭修复卡、提交推送更新 PR；无发布成功证据 |
| DEFER-SP-01 | 会话级 pack 列表/句柄/fanout 缓存 | SP-09 仍显示 pack 打开/解析占显著成本，且缓存失效设计获接受，再建卡/Issue | 先验证整块读与二分收益；避免过早引入 repack/并发失效复杂性 |
| DEFER-SP-02 | marker probe 与 read-dir 合并 | SP-09 证实 IPC 残余成本，且替换安全/嵌套仓库语义有可复现回归，再建卡/Issue | 两次 probe 时间边界不同，不能直接删出队检查 |
| DEFER-SP-03 | tracked stat/hash 有界批处理 | SP-09 证实主要残余为该串行路径，并验证逐路径 deadline/取消/部分结果设计，再建卡/Issue | 上游重复工作减少后才有可靠收益判断 |

## 任务卡粒度审计表

| 卡 | type | AC | VER | landing / prod-files | scope | writeset | release | exception |
|---|---|---|---|---|---|---|---|---|
| SP-00 | audit | 4/20 | 2/20 | 0/0 | S | 按依赖串行 | no-release | EX-SP-00-AUDIT-01 |
| SP-01 | implementation | 4/8 | 2/8 | 3/4 | M | 按依赖串行 | independent | EX-SP-01-CLOSE-01 |
| SP-02 | implementation | 5/8 | 3/8 | 4/5 | M | 按依赖串行 | independent | none |
| SP-03 | implementation | 7/8 | 3/8 | 1/2 | M | 按依赖串行 | independent | none |
| SP-04 | implementation | 4/8 | 2/8 | 2/4 | M | 按依赖串行 | independent | none |
| SP-05 | implementation | 5/8 | 3/8 | 2/3 | M | 按依赖串行 | independent | none |
| SP-06 | implementation | 5/8 | 2/8 | 2/4 | M | 按依赖串行 | independent | none |
| SP-07 | implementation | 5/8 | 2/8 | 1/1 | S | 按依赖串行 | independent | none |
| SP-08 | implementation | 7/8 | 3/8 | 2/2 | M | 按依赖串行 | independent | none |
| SP-09 | audit | 5/20 | 2/20 | 0/0 | S | 按依赖串行 | no-release | none |

## 测试矩阵与追溯

| 卡 | 报告来源 | 回归落点 | 性能/正确性证据 |
|---|---|---|---|
| SP-00/09 | 两报告统计与本轮用户决策 | 本计划 | 无插桩计时与插桩工作量分开；原样本未复算 |
| SP-01 | Audit Finding 3 | command_test::status_error_test | 缺失/损坏子树错误传播 |
| SP-02 | Audit Finding 1 | storage lib + command_test::restore_test | decoded size 与 peak_bytes 分离 |
| SP-03 | Review §5 | command_test::restore_test/clone_test | stat/racy/LFS/symlink/staged-only |
| SP-04 | Audit Finding 6 | command_test::status_test | 同一 index 快照及版本 mtime |
| SP-05 | Audit Finding 3 | command_test::status_test/status_wave0_test | 默认 rename 与 v2 的单次 HEAD 展开 |
| SP-06 | Audit Finding 4（修正归因口径） | utils lib + command_test::status_test | scope 隔离与发现次数 |
| SP-07 | Review §4 | storage::local lib | v1/v2、hash 类型、损坏索引、read 次数 |
| SP-08 | Review §6 + Audit Finding 5（修正预热前提） | command_test::status_test/status_wave0_test | 深层 ignore 错误、安全路径与调度次数 |

### 基线与实施证据汇总

实现卡仍全部 not-run；没有实施 PASS、独立审查 PASS、网站/发布/CI 完成证据。SP-00 已执行 release 构建与下述基线采样，`cargo +nightly fmt --all --check` 通过；该历史采样轮未运行 Clippy 和全量测试。Clippy 仍是推送前必跑门；全量测试按 audit 卡的 N/A 触发条件不在本卡执行，当前验收归属按 EX-SP-00-AUDIT-01 调整为本卡独立文档验收；历史登记 SP-01 已被该决策取代。

#### SP-00 冻结协议与环境

DEP-SP-DATA 按用户指定采用公开标准仓库 [`libra-tools/libra@d0a5c28`](https://github.com/libra-tools/libra/commit/d0a5c28a6b236421c14a38339b592fe37137585e)：Git 与 Libra 各自完整克隆并检出该提交，不再使用临时生成 5,000 个文件的测试仓库。未访问历史 Linux 原始归档或工作区外报告，历史数字只保留为引用。基线定义为**同一系统内，指定版本 Git 与当前构建 Libra 在标准仓库的等价状态下直接性能对照**，不是某个操作系统的绝对毫秒数。下表是 macOS 样本；其他系统使用相同协议、指定 Git 版本和当前 Libra 重跑即可验证，不要求持有 Mac，也不跨系统直接比较绝对耗时或倍率。

| 项目 | 冻结值 |
|---|---|
| 标准仓库与提交 | `https://github.com/libra-tools/libra.git`；`d0a5c28a6b236421c14a38339b592fe37137585e` |
| 标准仓库形状 | 2,125 个跟踪文件；两边初始状态干净；另构造 10 个暂存重命名、10 个未暂存修改、100 个未跟踪文件 |
| Libra 源码修订 | `8abdcf7633f8fef8f6d0994d9f0621bb074de292` |
| Libra 构建 | `cargo +nightly build --release --locked`；`rustc 1.100.0-nightly (574ff7d98 2026-09-14)` |
| Libra 发布构建 | `libra 0.30.91`；SHA-256 `72967e40bba1b2d0e8c85d337771509b7ad873a653f5a3d7c89114f8b9b0dc9e` |
| Git 对照 | `git version 2.56.0`；`/opt/homebrew/bin/git`；SHA-256 `40163333061f7223ecb604ed0f12988cfc8096d57ca8b1db006d410d4aeb700e` |
| 本次样本系统 | macOS 27.0.1 (26A434)，Darwin 27.0.0 arm64，Apple M1 |
| 缓存 | 新 CLI 进程、热页面缓存；清系统缓存 not-run（未获额外操作授权） |
| 插桩工作量 | not-run；本表只含无插桩实际经过时间，不能与历史插桩计数混算 |

仓库 runner 仍只直接覆盖 `status --short`。本卡使用一次性 Python 标准库采样器（非交付产物），实际入口为 `python3 /tmp/libra-sp00-standard-baseline.8HXV64/run-baseline.py`，结果为同目录的 `result.json`；以下命令、路径选择规则与断言是重放依据，临时脚本和原始结果不是交付物。

所有子进程使用临时 `HOME`、`USERPROFILE`、`XDG_CONFIG_HOME`、`LIBRA_HOME`、`LIBRA_CONFIG_GLOBAL_DB`、`LIBRA_CONFIG_SYSTEM_DB` 和空 `GIT_CONFIG_GLOBAL`，并设置 `GIT_CONFIG_NOSYSTEM=1`、`GIT_OPTIONAL_LOCKS=0`、`NO_COLOR=1`、`LC_ALL=C`，不读取真实全局配置。`GIT_OPTIONAL_LOCKS=0` 防止 Git status 的可选索引写回，使两边遵守相同的只读索引不变量。克隆、检出、测试状态构造与刷新均不计时，Git/Libra 分别执行以下等价步骤：

1. Git 执行 `git clone --quiet https://github.com/libra-tools/libra.git <repo>` 和 `git checkout --quiet --detach d0a5c28a6b236421c14a38339b592fe37137585e`；Libra 执行 `libra clone --quiet --no-progress https://github.com/libra-tools/libra.git <repo>` 和 `libra checkout --quiet --detach <同一提交>`。两边 `rev-parse HEAD` 均等于标准提交，初始 `status --porcelain=v1 --untracked-files=normal` 均为空，跟踪路径集合一致。
2. 逐文件核对克隆结果：2,123 个文件内容及符号链接目标一致。`install.ps1` 与 `template/pre-commit.ps1` 是仅有的两个例外，Git 按 `eol=crlf` 检出 CRLF，Libra 检出 LF；去除 CRLF 中的 CR 后内容一致，且两边初始状态均干净。该实际检出差异予以保留，不改写文件制造逐字节相同。
3. 按路径字典序取 `src/**/*.rs` 中前 20 个普通非符号链接文件。前 10 个在原目录改名为 `sp00-renamed-<原文件名>`，各自执行 `add -A -- <新旧路径>`，形成 10 个内容不变的暂存重命名；后 10 个各追加 `SP-00 status baseline change\n`，形成 10 个未暂存修改。HEAD 始终保持标准提交。
4. 在 `sp00-untracked/dir-01/` 至 `sp00-untracked/dir-10/` 各写 10 个文件，共 100 个未跟踪文件。复制整个测试仓库得到默认/已刷新两种状态；仅已刷新副本执行一次对应工具的 `update-index --refresh`。两边均对 10 个修改文件报告 `needs update` 并返回 1，这是预期构造结果，不是 status 样本失败。

矩阵是索引 `{默认, 已刷新}` × 未跟踪文件 `{normal, no}` × 重命名 `{默认, --no-renames}` × 机器可读输出 `{v1, v2}`，共 16 个场景；每个场景分别测量 Git 和 Libra。两边使用相同参数：`--no-pager status --porcelain=v1|v2 --untracked-files=normal|no`，重命名默认格不追加标志，关闭格追加 `--no-renames`。

每个场景和索引状态执行 25 轮，Git/Libra 按 AB/BA 交替顺序运行；每边前 5 次为预热，后 20 次为正式样本。以 `time.perf_counter_ns()` 测量独立进程实际经过时间；中位数为排序后第 10/11 个值的均值，p95 为 nearest-rank（20 个排序样本的第 19 个），另计算 Libra/Git 中位数倍率。每次调用均断言退出码 0、标准错误为空、各工具自身原始标准输出稳定，并比较 `.git/index` 或 `.libra/index` 调用前后 SHA-256。

跨工具语义断言使用排序后的机器可读记录集合：每行作为一条记录，排序后计算 SHA-256，Git/Libra 与两种索引状态必须一致；原始输出哈希仍分别记录，不假定逐字节相同。实测发现 v2 中 Git 将未暂存修改排在重命名前，Libra 顺序相反，记录内容一致，因此采用该规范化规则而不是掩盖排序差异。另断言 normal/no 的未跟踪标记有无，以及默认重命名的重命名记录与 `--no-renames` 的删除/新增记录。下表记录双方原始输出 SHA-256、字节/行数、退出码和索引不变量；预热也计入后三项，但不计入耗时统计。

#### SP-00 基线结果

16/16 个场景均为 measured，Git/Libra 每边各 5 次预热 + 20 次正式样本，无缺失格子；最终完整矩阵共 800 次调用、640 次正式计时，先前采样器断言调试轮不属于样本。Git 与 Libra 测试仓库 HEAD 均为标准提交 `d0a5c28a6b236421c14a38339b592fe37137585e`。耗时单位是 ms；命令列省略两边共同的 `--no-pager status` 前缀。原始输出列依次记录 Git/Libra SHA-256、字节数和行数；退出与索引列均覆盖每边全部 25 次调用。

| 索引 | 未跟踪 | 重命名 | 输出 | 共同命令参数 | 状态 | 每边 n | Git 中位数 / p95 | Libra 中位数 / p95 | L/G 中位数 | 原始输出 SHA-256 G/L / bytes / lines | 退出 G/L | 索引不变 G/L |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 默认 | no | 默认 | v1 | `--porcelain=v1 --untracked-files=no` | measured | 5+20 | 9.970 / 12.893 | 686.544 / 903.225 | 68.863× | G `d59c81cf692430c79ceba8b5834af4d758cee38089788c678c604f2d13652c57` / L `d59c81cf692430c79ceba8b5834af4d758cee38089788c678c604f2d13652c57` / 1008 / 20 | 0/0（各 25/25） | 25/25（每边各 25） |
| 默认 | no | 默认 | v2 | `--porcelain=v2 --untracked-files=no` | measured | 5+20 | 9.630 / 11.281 | 761.951 / 807.037 | 79.119× | G `88aed3eb0ca7b1a07f37995b2c5d0c7608d92222480dd44b144ec095546f02e6` / L `d84f73f2d21971c2507c6503dcbb989df6ee58f64cde345edb6e3102f6a7583f` / 3228 / 20 | 0/0（各 25/25） | 25/25（每边各 25） |
| 默认 | no | no-renames | v1 | `--porcelain=v1 --untracked-files=no --no-renames` | measured | 5+20 | 10.299 / 11.489 | 581.572 / 667.757 | 56.466× | G `d9a0482a655e669a54646e5c88e3f33d3bc238ec4afde579050c5ee5a3b2dd79` / L `d9a0482a655e669a54646e5c88e3f33d3bc238ec4afde579050c5ee5a3b2dd79` / 1008 / 30 | 0/0（各 25/25） | 25/25（每边各 25） |
| 默认 | no | no-renames | v2 | `--porcelain=v2 --untracked-files=no --no-renames` | measured | 5+20 | 9.712 / 11.677 | 680.091 / 833.644 | 70.029× | G `9fdd114d73b798732dba9cdc02acdab8f0a1589060d3d6231a8d431f5812e7de` / L `9fdd114d73b798732dba9cdc02acdab8f0a1589060d3d6231a8d431f5812e7de` / 4308 / 30 | 0/0（各 25/25） | 25/25（每边各 25） |
| 默认 | normal | 默认 | v1 | `--porcelain=v1 --untracked-files=normal` | measured | 5+20 | 15.777 / 20.037 | 915.928 / 1029.299 | 58.055× | G `168ee3f173d5a9da0501cae47e23edc35272e25eee6fe0b2815926e338da8990` / L `168ee3f173d5a9da0501cae47e23edc35272e25eee6fe0b2815926e338da8990` / 1027 / 21 | 0/0（各 25/25） | 25/25（每边各 25） |
| 默认 | normal | 默认 | v2 | `--porcelain=v2 --untracked-files=normal` | measured | 5+20 | 15.896 / 20.614 | 1022.059 / 1205.166 | 64.296× | G `223d1ec03f76660de9413da87f81ef0968cded7430251896e4a1f0356583a078` / L `d3132dbaa1f635c41561c34c8ce97e689e8582d35600df329e95460d61193f6d` / 3246 / 21 | 0/0（各 25/25） | 25/25（每边各 25） |
| 默认 | normal | no-renames | v1 | `--porcelain=v1 --untracked-files=normal --no-renames` | measured | 5+20 | 15.699 / 22.133 | 818.139 / 1027.019 | 52.113× | G `74beebb3d6cc22253e1cf47d5a4379bc25aab5bd0e99227732365302dde19435` / L `74beebb3d6cc22253e1cf47d5a4379bc25aab5bd0e99227732365302dde19435` / 1027 / 31 | 0/0（各 25/25） | 25/25（每边各 25） |
| 默认 | normal | no-renames | v2 | `--porcelain=v2 --untracked-files=normal --no-renames` | measured | 5+20 | 20.144 / 23.943 | 1205.840 / 1649.260 | 59.862× | G `c810acd9dfed468e286f4c1a91fcfd35c7f44eb7d4cc92a8dd5532bccc2e2a6b` / L `c810acd9dfed468e286f4c1a91fcfd35c7f44eb7d4cc92a8dd5532bccc2e2a6b` / 4326 / 31 | 0/0（各 25/25） | 25/25（每边各 25） |
| 已刷新 | no | 默认 | v1 | `--porcelain=v1 --untracked-files=no` | measured | 5+20 | 11.773 / 16.355 | 500.876 / 754.638 | 42.543× | G `d59c81cf692430c79ceba8b5834af4d758cee38089788c678c604f2d13652c57` / L `d59c81cf692430c79ceba8b5834af4d758cee38089788c678c604f2d13652c57` / 1008 / 20 | 0/0（各 25/25） | 25/25（每边各 25） |
| 已刷新 | no | 默认 | v2 | `--porcelain=v2 --untracked-files=no` | measured | 5+20 | 11.442 / 20.081 | 608.665 / 809.822 | 53.196× | G `88aed3eb0ca7b1a07f37995b2c5d0c7608d92222480dd44b144ec095546f02e6` / L `d84f73f2d21971c2507c6503dcbb989df6ee58f64cde345edb6e3102f6a7583f` / 3228 / 20 | 0/0（各 25/25） | 25/25（每边各 25） |
| 已刷新 | no | no-renames | v1 | `--porcelain=v1 --untracked-files=no --no-renames` | measured | 5+20 | 10.848 / 13.265 | 332.737 / 416.497 | 30.672× | G `d9a0482a655e669a54646e5c88e3f33d3bc238ec4afde579050c5ee5a3b2dd79` / L `d9a0482a655e669a54646e5c88e3f33d3bc238ec4afde579050c5ee5a3b2dd79` / 1008 / 30 | 0/0（各 25/25） | 25/25（每边各 25） |
| 已刷新 | no | no-renames | v2 | `--porcelain=v2 --untracked-files=no --no-renames` | measured | 5+20 | 11.325 / 14.416 | 467.609 / 554.719 | 41.291× | G `9fdd114d73b798732dba9cdc02acdab8f0a1589060d3d6231a8d431f5812e7de` / L `9fdd114d73b798732dba9cdc02acdab8f0a1589060d3d6231a8d431f5812e7de` / 4308 / 30 | 0/0（各 25/25） | 25/25（每边各 25） |
| 已刷新 | normal | 默认 | v1 | `--porcelain=v1 --untracked-files=normal` | measured | 5+20 | 16.611 / 22.278 | 694.357 / 1073.463 | 41.802× | G `168ee3f173d5a9da0501cae47e23edc35272e25eee6fe0b2815926e338da8990` / L `168ee3f173d5a9da0501cae47e23edc35272e25eee6fe0b2815926e338da8990` / 1027 / 21 | 0/0（各 25/25） | 25/25（每边各 25） |
| 已刷新 | normal | 默认 | v2 | `--porcelain=v2 --untracked-files=normal` | measured | 5+20 | 18.886 / 26.372 | 897.033 / 1294.295 | 47.498× | G `223d1ec03f76660de9413da87f81ef0968cded7430251896e4a1f0356583a078` / L `d3132dbaa1f635c41561c34c8ce97e689e8582d35600df329e95460d61193f6d` / 3246 / 21 | 0/0（各 25/25） | 25/25（每边各 25） |
| 已刷新 | normal | no-renames | v1 | `--porcelain=v1 --untracked-files=normal --no-renames` | measured | 5+20 | 18.447 / 28.521 | 647.583 / 975.519 | 35.105× | G `74beebb3d6cc22253e1cf47d5a4379bc25aab5bd0e99227732365302dde19435` / L `74beebb3d6cc22253e1cf47d5a4379bc25aab5bd0e99227732365302dde19435` / 1027 / 31 | 0/0（各 25/25） | 25/25（每边各 25） |
| 已刷新 | normal | no-renames | v2 | `--porcelain=v2 --untracked-files=normal --no-renames` | measured | 5+20 | 19.897 / 32.019 | 799.181 / 1038.466 | 40.166× | G `c810acd9dfed468e286f4c1a91fcfd35c7f44eb7d4cc92a8dd5532bccc2e2a6b` / L `c810acd9dfed468e286f4c1a91fcfd35c7f44eb7d4cc92a8dd5532bccc2e2a6b` / 4326 / 31 | 0/0（各 25/25） | 25/25（每边各 25） |

最终完整矩阵全部 800 次 status 调用的标准错误均为空；16 组 Git/Libra/索引状态的规范化记录集合一致，各工具原始输出稳定，未跟踪与重命名语义断言通过。采样结束后的索引 SHA-256 为：Git 默认 `64ddde456b6812b4c6442fc4980bbce28434b6df54b443531ca2b25bf5e13da9`、Git 已刷新 `eb75558abcd6e37822a75a38e0d1c77402ba14d673bfd7e8b494df07300bdc6f`、Libra 默认 `4c0d8ce6c8c3c2e694a84f0c033c4b9e8795b8208e2059228eea588496d44634`、Libra 已刷新 `1c31c085e4c09ff31ba902c174eda230d00d664f1ee7cef1f38898f5fd45a624`。两边预备刷新均返回预期 1，status 样本自身全部返回 0。当前样本的 Libra/Git 中位数倍率为 30.672–79.119×；不显示未跟踪文件时为 30.672–79.119×，显示时为 35.105–64.296×。该表以 Git 2.56.0 为外部参照，不设未经决策的追平目标，也不把单轮倍率或两种索引状态的差值解释为后续优化收益。

审计口径复核：9,647 次 readlink 属于 242 次总发现，206 次对象读取只是其中的另一归因子集，不能把全部 readlink 算到对象读取；线程池提交次数不能推出固定线程切换下界；`ignore_sources_for_target` 从 target 父目录开始，根预热既未覆盖目录自身 ignore 文件，也不能单独证明 tracked 短路等价，因此“零风险预热”结论不成立。出队 marker probe 仍承担防目录替换的重新验证，本卡不建议删除。

历史报告中的约 733→491 ms、关闭 untracked 约省 200 ms、约 12 万次 idx read 及 242/206 次计数均为未访问原始归档的历史引用，本轮未独立复算；本轮实测仅以上述同机 Git/Libra 16 场景无插桩对照为准。冷缓存、其他 OS 和插桩工作量仍为 not-run；其他系统可按同一协议产生自己的 Git/Libra 对照，不需要复现这份 macOS 绝对数据。

### 里程碑、风险与恢复

| 里程碑 | 完成条件 | 恢复 |
|---|---|---|
| M0 | SP-00 基线与边界确认 | 文档回退，不改代码 |
| M1 | SP-01..03 正确性门通过 | 前滚修复；staged-only 不伪造 stat |
| M2 | SP-05..08 核心重复工作消除 | 各独立切片前滚，保留安全检查 |
| M3 | SP-09 全矩阵与最终全量通过 | 失败留 open，修复后重跑；可选项可保持延后 |

风险：共享 storage/util 与资格计划冲突（DEP-SP-STORAGE）；外部归档不可用（DEP-SP-DATA）；网站落点未核实（DEP-SP-DOCS）；缓存/批处理扩大安全面（DEFER）；多个因素交互导致收益不可加和（SP-00/09 配对复测）。

## 完成判据

- [ ] 必选与本轮核心任务达到 done/complete，所有适用门证据写入本计划；未启用 SP-04 明确保留 optional pending。
- [ ] SP-09 的最终全量、fmt/clippy 通过；发现的缺陷前滚修复并重跑。
- [ ] 默认命令与原 no-renames 基准都被覆盖；未测平台和冷缓存限制明确披露。
- [ ] 适用网站文档、部署及远端 C/D 门完成，或明确保持未完成，不能以局部绿代替。
- [ ] plan-status 同步；Issue 关闭依据为实际验收而非创建任务或合并 PR。

## 自审与修订历史

本轮自审：正确性与性能分卡；无 CE_VALID 重复工作；实现卡均不超过 M/8AC/8VER；重叠写集按顺序串行；网站与外部数据未核实处有显式依赖。正式外部 review 未执行，不宣称 agent-ready。最终纯文档登记提交采用模板 v2.14 终止链协议：推送后 D 组 SHA/run ID/结论/UTC 由下一次 plan-status 例行同步补记，不制造无限自引用提交。

| UTC 时间 | 变更 | 状态 |
|---|---|---|
| 2026-10-28 01:56:39 UTC | 依据两报告及用户决策建立 SP-00..09；CE_VALID 排除，SP-04 可选，缓存/marker/IPC 延后 | pending；无实施 |

| 2026-10-28 02:18:21 UTC | 用户授权提交并推送两份计划文档；原报告由用户移出工作区。文档路径/任务依赖 DAG 自查通过，`cargo +nightly fmt --all --check` 通过；Clippy 在依赖获取前被 Cargo 缓存写权限阻塞，提升权限两次自动审核超时，未完成代码检查；nextest 未启动 | 仅文档交付，不表示实现卡或 C/D 验收完成 |

| 2026-10-28 09:42:48 UTC | SP-00 在 `codex/SP-00` 冻结 macOS 新基线；16/16 格、400 次调用与 320 次正式样本完成，输出/退出/index 断言全过，审计口径与历史引用边界已复核 | 当时曾标本地 done / complete；后续被 Git 直接对照方案取代 |

| 2026-10-28 10:19:45 UTC | 用户要求以指定 Git 与当前 Libra 直接对照；SP-00 重开并完成 16 场景、800 次调用与 640 次正式计时，双方语义、退出与索引断言全过，v2 排序差异以规范化记录集合处理 | SP-00 本地 done / complete；未提交、未推送，Issue #639 保持 open |

| 2026-10-28 12:30:24 UTC | 用户指定 `libra-tools/libra@d0a5c28a6b236421c14a38339b592fe37137585e` 为标准仓库；Git/Libra 分别完整克隆并检出该提交，在 2,125 个跟踪文件的真实工作区上重跑 16 场景，最终完整矩阵 800 次调用、640 次正式计时及语义/退出/索引断言全过；两个 PowerShell 文件的 CRLF/LF 检出差异已披露，旧合成仓库数据仅保留为历史 | SP-00 本地 done / complete；未提交、未推送，Issue #639 保持 open |

| 2026-10-29 | #639 收口复核：在 `codex/SP-00` 以未提交合并方式合入 main `1e5310c24ef4eaa07575256482490705181c360a`（含 PR #654），保留其全部生产/测试修复；`plan-status.md` 冲突保留本卡基线行及 main 其它最新条目。16 格唯一组合、800/640 样本数及展示精度下倍率一致性检查通过；历史采样原件未读取、基准未重跑，原采样结果按 PR #652 记录引用，非本轮独立实测。修正“audit 可免 Clippy”表述及过早 complete：本卡为 in-progress / locally-accepted，SP-01 的 C/D 覆盖仍待；本轮 fmt 与合并后 `cargo clippy --all-targets --all-features --offline -- -D warnings` 均 exit 0（Clippy 58.85s）；依赖 `block 0.1.6` 的 future-incompat 提示保留，不称零警告。首轮在合并完成前读到旧源码的 Clippy exit 101，仅作失败历史，不用于判断 #654；本轮未启动 nextest。 | PR #652 已提交/推送、仍 open；本轮修复尚未提交/推送，Issue 未关闭；不发布版本、不扩大到 SP-01 |

| 2026-10-29 | 用户批准 EX-SP-00-AUDIT-01：SP-00 取消继承 SP-01 发布验收，改为自行完成文档/评审/静态门及 PR #652 最终 head 的 CI；保留 SP-01 对 SP-00 的执行依赖及其原有 C/D。历史反向继承记录保留为当时事实，以本决策为现行口径。 | Codex 本轮文档复核 PASS（范围、依赖方向、现行/历史分离及验收条件一致）；SP-01 卡逐字未改、4 AC/2 VER 保留、workflow 与变更内链接目标及冲突标记检查通过。fmt 与严格 Clippy（offline，1.88s）均 exit 0，既有 block 0.1.6 future-incompat 提示保留；未重跑基准或 nextest。旧索引中的跨仓链接不在本次变更检查范围。待提交/推送；最终 head CI、合并与 Issue 收口仍未完成，保持 in-progress / locally-accepted |

| 2026-10-29 SP-01 开工 | 实核 #652 已合并 d0578db，最终 head 全部 CI SUCCESS；据 EX-SP-00-AUDIT-01 将 SP-00 登记 done/complete，#639 仍 open 另计。用户授权 SP-01，从 upstream/main d0578db 创建 codex/SP-01；发现根 tree 安全加载后仍有三处使用递归 panic 展开路径。测试边界沿用卡内指定 status CLI 集成测试。 | 第一条缺失第二层子树回归已写，等待 RED；网站 /Volumes/Sea/libra-backend 不存在，用户明确暂且搁置网站同步，公开文档交付门仍未完成；本机 macOS arm64、物理内存 17179869184 bytes（16 GiB）且资源压力已查，禁止 nextest。两份 .env 测试文件缺失；本轮仅运行使用隔离 HOME/config 的既有 deterministic CLI 测试，不宣称全量通过 |

| SP-01 本仓库 RED → GREEN | 首次短过滤器搭配 --exact 命中 0，不计验收；改为完整 `command::status_error_test::status_nested_tree_missing_returns_repo_error` 后实际 1/1 RED：Tree::load 缺失第二层子树 panic、CLI exit 1 而非 128。加入携带 OID/source 的 TreeExpansionError 和独立 fallible 展开函数，status 三处展开调用改为错误传播；保留 TreeExt 原公开契约与其它命令原路径。首次 missing 1/1 GREEN；扩展四例 4/4 GREEN；独立函数调整后整个 status_error_test 14/14 GREEN（9.43s），包含四例新回归。 | 三类错误各覆盖 human/no-renames/porcelain v1/v2/JSON/machine 六种调用，均 exit 128、LBR-REPO-002、包含 OID 与 fsck 提示、stdout 空、结构化模式单 JSON、index 字节不变；合法嵌套 gitlink 三种输出成功。EN/zh-CN status、错误码表和兼容表已同步；网站按用户决定搁置。严格 Clippy（all-targets/all-features，offline，2m12s）exit 0，既有 block 0.1.6 future-incompat 提示保留；Display 单元 1/1、现有缺失 HEAD 1/1、现有 gitlink 2/2 通过；新增前缀在最终实现再次 4/4 通过（4.26s）。含 status_error_test 14 项共 18 个不同用例，重跑不重复计数。补充回归初次误用文件名 status_wave0_test 作模块过滤命中 0，不计通过；查询测试清单后以实际模块 command::status_wave0 执行以上 1/1 和 2/2。fmt、diff check、双语文档及 TreeExt 契约/三调用点检查通过。代码差异自查未新增生产 panic/unwrap/expect；正式完整验收与 C/D 未取得，仍 in-progress / 验收空 |

| SP-01 当前交付边界 | 本仓库四个生产文件、status_error_test 与四份现有命令/错误/兼容文档已修改；原 TreeExt trait 逐字保持，status 的三处展开均改为 fallible 调用。测试和静态门实证见上一行。 | 网站依用户决定暂搁；16 GiB nextest 硬门槛与缺失双 env 使 T-1 全量 not-run；版本 bump、release build/安装、提交/推送、PR/发布及远端 C/D 均未执行，不将局部绿标为 complete |

| 2026-10-30 用户澄清与 CI 交付 | 用户明确“在本机资源不足时候不触发，CI 不在此列”；32G 门槛仅适用于本机，CI 按现有 base.yml 运行，不修改工作流、不增加资源检查。SP-01 实现已 signed+DCO 提交 aaa2a986be5a6d613749aa78d27f000105a57700 并推送 origin/codex/SP-01；用户已授权创建 PR 和触发全量。 | 本轮同步澄清后推送并创建草稿 PR，触发现有 compat-offline-core 全量；CI 结果待实证，本机 nextest 仍 not-run，网站仍依用户决定搁置；版本发布与完整 C/D 未完成 |

| 2026-10-30 SP-01 修复卡完成 | 用户明确要求设置卡片完成、推送更新 PR。依据 EX-SP-01-CLOSE-01，本仓库修复、文档、18 个不同定向用例、fmt/Clippy，以及 bb88db3 的全量 CI/CodeQL 均通过；nextest 10841/10841 passed、13 slow、23 skipped，577.819s。 | SP-01 done/complete；网站和独立版本发布仍未执行，分别登记 DEFER-SP-DOCS-01 / DEFER-SP-REL-01。PR #655 仍 open，未合并；本次仅文档更新，不回写旧 head 运行结果、不声称新 head CI 已成功 |
