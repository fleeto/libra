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

- 每张卡当前 Lifecycle=pending，Acceptance=空；没有 agent-ready 标签，没有代码开工授权。实施前刷新源码、计划、Issue、写集冲突和环境；先复现失败再修复。任务字段列明的 `(new)` 测试必须在同卡新增，过滤零命中不算通过。
- 每次推进同一变更同步本计划与 `plan-status.md`。证据只汇总在本计划，不创建新证据目录/文件；工具临时运行文件不作为独立交付产物。
- 全局继承模板 GC-*、ER-*、G-*；新测试复用现有 target，若需要新增 target/serial lane，先更新卡写集及 `tests/INDEX.md`/registry 等再实施，不暗自扩张。
- **用户资源硬门槛（2026-10-28）：每次启动 nextest 前先核验实际执行宿主的物理总内存及当前资源压力。总内存低于 32G 禁止启动；无法确认则不启动。不能通过降低 nextest 并发绕过。资源不达标时门记 not-run，改在获授权且达标的执行宿主运行，不得自动连接远程主机。**
- 全量门采用 `source .env.test && source .env.live-test && cargo nextest run --all --no-fail-fast --retries 2`；只在相关环境已安全核验后运行，不能打印 secrets。每次会推送的卡须 `cargo +nightly fmt --all --check`、`cargo clippy --all-targets --all-features -- -D warnings`。全量失败需前滚修复并重跑，focused 通过不能冒充全量。
- implementation 默认 independent，Version increment=patch；Release write set=权威版本面集合（`compat_version_surface_sync`）、工具链更新的 Cargo.lock、artifact。实际提交/推送/Release 需后续授权，本次建 Issue 不包含这些动作。
- implementation 默认 Rollback mode=immutable-release：未发布可回退本卡实现，已发布用修复版恢复；无 schema 迁移，不自动重写历史 index。下载旧版本无法撤销已写出的 stat，必要时安全刷新，不能保留可疑 clean 缓存。
- 默认 Docs and compatibility impact=N/A 仅适用于输出与错误完全不变的内部优化；公开行为变化卡须同卡双语与网站文档。网站路径为候选，DEP-SP-DOCS 核验后才允许开工；遵循用户 standing instruction 直接 cf，通过 Libra 提交推送，不能另建网站分支/PR/worktree。
- 无 EX 豁免，无家族合并发布；C/D coverage=self（实现卡），audit 覆盖来源为下列具体实现卡。正式 review 与 C/D 门未执行，不能把本次自查写成批准开工。

## 实施顺序与发布窗口

核心串行顺序：`SP-00 → SP-01 → SP-02 → SP-03 → SP-05 → SP-06 → SP-07 → SP-08 → SP-09`。

可选：`SP-09 + DEP-SP-OPT → SP-04`。这些顺序边既含产物依赖，也为共享源码/测试写集串行化；不是声称所有相邻卡存在算法上的前置关系。公开行为卡另依赖 DEP-SP-DOCS；SP-00 消费 DEP-SP-DATA 的可用性决策。无多 Agent 并发，发布者待实施授权后指定，任一时刻仅一个发布窗口。

Phase 0 基线：SP-00；Phase 1 正确性与 stat：SP-01..03；Phase 2 核心重复工作：SP-05..08；Phase 3 验收决策：SP-09；后续可选 SP-04。实现卡各自独立发布，禁止将 SP-09 当成替其它卡补做版本/验收的发布点。

## 任务总览

| 卡 | 优先级 | 选择 | Issue | Lifecycle / Acceptance |
|---|---|---|---|---|
| SP-00 冻结 status 性能基线与修正审计口径 | P1 | 必选验收前置 | [#639](https://github.com/libra-tools/libra/issues/639) | pending / 空 |
| SP-01 status 递归子树加载返回错误而非 panic | P1 | 必选正确性 | [#640](https://github.com/libra-tools/libra/issues/640) | pending / 空 |
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

**Lifecycle / Acceptance:** pending / 空

**Priority / Selection:** P1；必选验收前置

**Issue:** [#639](https://github.com/libra-tools/libra/issues/639)

**Description:** 建立可重放的性能与语义对照协议；纠正旧审计的过强结论，不把文档中的历史统计当作本轮实测。

**Out of scope:** CE_VALID 永久不在本计划范围；其它行为轴按任务总览分别承接；pack 会话缓存/marker 合并/IPC 批处理登记为 DEFER-SP-01..03，未启用。

**Current evidence:** LIBRA_STATUS_PERFORMANCE_REVIEW.md:72；LIBRA_STATUS_PERFORMANCE_REVIEW_AUDIT.md:168,221,228；benchmark/run.sh；benchmark/README.md（基线 380c262，开工刷新锚点）。

**Acceptance criteria:**

- [ ] 在本计划记录实际基线 revision、release binary 标识、OS、fixture 形状与样本协议。

- [ ] 记录默认/刷新 index × normal/no-untracked × 默认 rename/no-renames × porcelain v1/v2 的命令矩阵。

- [ ] 逐项修正审计中的 242/206 次发现混用、线程切换下界与零风险预热结论。

- [ ] 记录每个已采样格子的输出、退出状态与 index 不变量；缺失格子明确标未测。


**Verification:**

- [ ] 人工核对：本计划基线表的每格均有准确命令、revision、样本数及 measured/not-run 状态。

- [ ] 人工核对：两份报告的历史统计与本轮实测分栏，未访问的外部归档不标独立复算。


**Full-suite trigger:** N/A：audit，不执行发布 C 组

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

**Release boundary:** no-release；规划记录随实现交付进入仓库

**C/D coverage from:** SP-01；尚未运行。

**Granularity:** type=audit; axis=冻结 status 性能基线与修正审计口径; recovery=回退文档; complete=yes; self-contained=yes; AC=4/20; VER=2/20; landing=0; prod-files=0; scope=S; deps=DEP-SP-DATA; writeset=按实施顺序串行; release=no-release; split-from=N/A; exception=N/A

### Task SP-01: status 递归子树加载返回错误而非 panic

**Task type:** implementation

**Lifecycle / Acceptance:** pending / 空

**Priority / Selection:** P1；必选正确性

**Issue:** [#640](https://github.com/libra-tools/libra/issues/640)

**Description:** 为 status 的 HEAD 递归展开提供可失败的读取路径，缺失或损坏子树返回上下文错误，禁止退化为空树或 clean。

**Out of scope:** CE_VALID 永久不在本计划范围；其它行为轴按任务总览分别承接；pack 会话缓存/marker 合并/IPC 批处理登记为 DEFER-SP-01..03，未启用。

**Current evidence:** src/utils/object_ext.rs:54,111；src/command/status.rs:1030-1039；src/command/status/scan.rs:1327；src/command/status/output.rs:1361（基线 380c262，开工刷新锚点）。

**Acceptance criteria:**

- [ ] 缺失子树返回 LBR-REPO-002 而非 panic。

- [ ] 损坏子树返回包含受影响 OID 的可操作错误。

- [ ] human/JSON/porcelain 错误输出遵守现有格式与退出契约。

- [ ] 合法含 gitlink 的 tree 不被当成缺失子树。

- [ ] ER-06a：文档清单逐文件同步并验收（全局必需门，不计入特有 AC）。


**Verification:**

- [ ] `cargo test --test command_test status_error_test::`

- [ ] `cargo test --test command_test status_error_test::status_nested_tree_ -- --nocapture` — `(new)` 测试前缀；须非零命中。


**Full-suite trigger:** T-1：共享 tree 读取辅助逻辑

**Dependencies:** SP-00, DEP-SP-DOCS

**Deliverables:** 实现、同卡回归及列明文档

**Implementation write set:** `src/utils/object_ext.rs`, `src/command/status.rs`, `src/command/status/scan.rs`, `src/command/status/output.rs`, `tests/command/status_error_test.rs`, `docs/commands/status.md`, `docs/commands/zh-CN/status.md`, `COMPATIBILITY.md`, `docs/error-codes.md`, `/Volumes/Sea/libra-backend/apps/tanstack-app/content/docs/commands/status.en.md`

**Release write set:** Inherited

**Docs and compatibility impact:** `docs/commands/status.md`；`docs/commands/zh-CN/status.md`；`COMPATIBILITY.md`；`docs/error-codes.md`；`/Volumes/Sea/libra-backend/apps/tanstack-app/content/docs/commands/status.en.md`。网站候选路径待 DEP-SP-DOCS 核实。

**Rollback mode:** immutable-release

**Migration and rollback:** 无 schema 迁移；发布前回退本卡，发布后前滚修复；不把版本回退视为历史 index 修复。

**Security and privacy:** 保留既有路径边界、错误隔离与超时；只操作隔离 fixture，禁止读取/修改真实全局配置或外传 secrets。

**Performance budget:** 无未经实测的绝对毫秒承诺；按 SP-00 的同环境前后配对协议报告中位数、尾延迟及工作量；语义退化即失败，噪声内变化记不确定。

**Estimated scope:** M

**Version increment:** patch

**Release boundary:** independent

**C/D coverage from:** self；尚未运行。

**Granularity:** type=implementation; axis=status 递归子树加载返回错误而非 panic; recovery=前滚修复版; complete=yes; self-contained=yes; AC=4/8; VER=2/8; landing=3; prod-files=4; scope=M; deps=SP-00,DEP-SP-DOCS; writeset=按实施顺序串行; release=independent; split-from=N/A; exception=N/A

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
| DEFER-SP-01 | 会话级 pack 列表/句柄/fanout 缓存 | SP-09 仍显示 pack 打开/解析占显著成本，且缓存失效设计获接受，再建卡/Issue | 先验证整块读与二分收益；避免过早引入 repack/并发失效复杂性 |
| DEFER-SP-02 | marker probe 与 read-dir 合并 | SP-09 证实 IPC 残余成本，且替换安全/嵌套仓库语义有可复现回归，再建卡/Issue | 两次 probe 时间边界不同，不能直接删出队检查 |
| DEFER-SP-03 | tracked stat/hash 有界批处理 | SP-09 证实主要残余为该串行路径，并验证逐路径 deadline/取消/部分结果设计，再建卡/Issue | 上游重复工作减少后才有可靠收益判断 |

## 任务卡粒度审计表

| 卡 | type | AC | VER | landing / prod-files | scope | writeset | release | exception |
|---|---|---|---|---|---|---|---|---|
| SP-00 | audit | 4/20 | 2/20 | 0/0 | S | 按依赖串行 | no-release | none |
| SP-01 | implementation | 4/8 | 2/8 | 3/4 | M | 按依赖串行 | independent | none |
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

当前全部 not-run。未运行 cargo、benchmark、fmt/clippy 或全量测试；没有实施 PASS、独立审查 PASS、网站/发布/CI 完成证据。

SP-00 必须在此冻结具体命令和 fixture 构造过程，再开始生产修改。当前仓库 runner 仅直接覆盖 status --short，不能冒称支持全部矩阵；先复用其隔离 fixture 方法，补充精确的一次性临时采样命令写入本节。每格 warmup=5、测量至少20次，前后交错执行，记录 median/p95、退出码及语义断言。热缓存与清缓存分开；清系统缓存需要额外操作授权时标 not-run，不能擅自修改宿主。Linux 历史归档不可用时用当前平台新基线，不能跨平台比较绝对毫秒。

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
