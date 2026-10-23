# status 性能修复计划（2026-10-23）

## 文档职责

修复已由本机测量和源码交叉定位的 pack 索引读取放大、HEAD 树重复展开。本文是修复准备草案，依据 plan-template v2.14；不代表实现、评审或发布完成。本文仅描述两张待实施任务卡，不代表代码修改、评审或发布已完成。

### 适用范围

普通 packed-object 查询、CLI status 的 staged/rename HEAD 快照，以及同卡测试和性能说明。

### 非目标

不改变 status 默认 rename、索引持久化策略、文件访问安全协议或输出格式；不承诺追平 Git，不新增跨调用索引缓存。

### 成功定义

普通索引查询消除逐槽 fanout 读取及桶内线性搜索；一次 status 的 staged/rename 共享一次 HEAD 展开；正确性回归、确定性工作量测试和同环境复测完成。两卡的适用验收及发布门全部满足才可标记计划完成。

## 事实基线

行号是准备时锚点，开工时必须刷新。以下数值来自本次会话此前实际测量，不是本轮新跑的测试。

| 类别 | 当前事实 | 证据 |
|---|---|---|
| 比较对象 | Libra 0.30.84、Git 2.53.0；HEAD `2b67073abbbd1acafbb03664c967dfc1f08769f9`，2114 tracked 文件 | 本仓库与 `../libra-git`；两个安装脚本内容不同，不能称逐字节相同工作树 |
| 原始耗时 | 默认 status 五次暖跑中位数：Libra 1.90584 s / CPU 1.47068 s；Git 0.009454 s / CPU 0.007352 s | 实际进程测量；CPU 为 user + system |
| stat 缺口 | Libra 2114 条 index ctime/mtime 为零；隔离副本 refresh 后命中 2114 条 | `src/command/status_untracked.rs:214`、`:317`；`src/utils/stat_diff.rs` |
| refresh 后 | 隔离副本默认 status 1.381567 s / CPU 1.201183 s；Git 同样 refresh 后 0.008679 s / CPU 0.00683 s | 均五次中位数；原仓库 index 未修改；不能和不同 fixture 的原始数值混算改进比例 |
| fanout | 每次查询读取 256 次四字节；普通查询随后线性搜索桶 | `src/utils/storage/local.rs:1281`、`:1299`、`:1305` |
| 已有复用点 | 批量查询已有二分查找；普通入口尚未使用 | `src/utils/storage/local.rs:1375` |
| 缓存位置 | pack 对象缓存命中之前仍先查 idx | `src/utils/storage/local.rs:1441`、`:1667` |
| 重复展开 | staged 比较及默认 rename 各自读取 HEAD 树 | `src/command/status.rs:996`；`src/command/status/scan.rs:178`、`:1327` |
| 调用计数 | `status -uno --no-renames`：idx 123323 次读取 / 1058396 bytes；默认 rename：246646 次 / 2116792 bytes | LD_PRELOAD read/pread64 计数；pack 内容读取两者均为 237 次 / 1941504 bytes |
| 模型核对 | 86449 对象的 v1 idx；HEAD commit + 204 trees 共 205 次查询，按当前算法预测恰为 123323 次读取 | 上述实测与模型精确一致；不据此断言它占总耗时的某个百分比 |
| 现有测试 | 二分边界、status 格式与 rename 回归已存在 | `utils::storage::local::tests::checked_batch_binary_index_finds_bucket_edges_and_misses`；`tests/command/status_test.rs` |
| 文档 | EN、zh-CN、开发说明 | `docs/commands/status.md`、`docs/commands/zh-CN/status.md`、`docs/development/commands/status.md` |

### 当前缺口

| ID | 缺口 | 动作 |
|---|---|---|
| GAP-SP-01 | 无缓存普通对象查询的小读取与线性搜索放大 | SP-01 |
| GAP-SP-02 | 一次 status 重复展开同一 HEAD | SP-02 |
| GAP-SP-03 | stat 失效、逐文件 IPC、路径重复解析仍有成本 | 延后；首轮后重新归因，不混入两卡 |

## 与其它计划的关系

- [#486](issues/486.md) 已处理 ahead/behind 正确性，其旧新 Libra 测量不能替代 Git 对照；不重做该卡或其 DEFER 项。
- [MCP 计划](plan-20261019-agent-mcp-server.md) MC-33 是有界收集，不能替代本计划；与 status 写集可能相交，登记 DEP-SP-02。MC-32 的状态仍为 `in-progress/locally-accepted`，不能把安装版本号当成其完整验收证明。
- [结构拆分计划](plan-20260927.md) 涉及 status 模块；开工时核对最终源码与写集交接，纳入 DEP-SP-02。
- `plan-long.md`：本次是具体热点修复，未分配 LR 编号；不虚构长周期排期。状态唯一登记在 `plan-status.md`。

## 评审结论与修订记录

草案待维护者评审；无正式 PASS，无已分配负责人或承诺日期。

### 修订历史

| UTC | 修订 | 状态 |
|---|---|---|
| 2026-10-23 06:58:52 UTC | 初稿：两卡串行；冻结既有计数证据、兼容边界和延后范围 | pending / 空 |

## 已决议设计决策

尚无维护者 Accepted 决策。以下均为 Proposed：

- ADR-SP-01：以固定 1024-byte 缓冲解析 fanout，普通入口复用二分查询；不建立全局 idx 缓存，避免新增 pack 替换/GC/跨仓库失效协议。
- ADR-SP-02：status 在单次收集内复用同一 HEAD OID 的树映射；保留 `changes_to_be_committed_safe` 对其它调用者的签名及语义。不得引入第二份完整映射来假装共享。
- ADR-SP-03：不自动写回 index。refresh 仅在隔离 benchmark fixture 中用于区分 stat 命中与失效。

## 全局工程约束

完整继承 [模板](plan-template.md) GC-01..GC-13。补充：

- 只改当前分支 `status-speedup` 的授权文件；不改真实工作区 index、refs、配置库或 sibling Git 仓库。
- 不削弱 no-follow、root 身份检查、deadline、rename 预算或错误传播；不新增生产 unwrap/expect/panic。
- 性能计数器只用于测试；不得向 CLI stdout/stderr 增加诊断。测量须隔离配置库，不能隐式回落真实全局配置。
- 基准冻结二进制、HEAD、对象布局、文件内容、stat 命中率和文件系统；时间比较不使用 LD_PRELOAD 插桩结果。所有数值必须注明新进程、暖跑、样本数。
- 证据直接写入本计划，不新增独立日志/报告/证据目录；临时测试 fixture 按既有测试惯例清理。

## 执行检查必备需求（强制）

继承模板 ER-01..ER-14。当前只是准备，以下是实施门而非已执行事实：

1. 开工前刷新源码、测试名、依赖与写集；用户授权实现后才实施。
2. 每卡先运行 focused 回归，零匹配不能算通过；两卡均因共享 helper 命中 ER-13 T-1，需完整 Nextest。
3. `cargo +nightly fmt --all --check`、`cargo clippy --all-targets --all-features -- -D warnings`；全量按模板双环境准备运行 `source .env.test && source .env.live-test && cargo nextest run --all --no-fail-fast --retries 2`。缺环境文件必须报告，不读取或输出秘密。
4. 本地验收后正式代码 review，修复再验；不把本文件结构核对当作代码 review PASS。
5. 如后续授权发布，每卡独立 patch + 1，版本面以 `compat_version_surface_sync` 为准，lockfile 只由工具链刷新；ER-04 C/D 与 ER-08/14 均须履行。本准备不授予 commit/push/PR/tag/release 权限。
6. 网站按用户 standing instruction 在 cf 交付，须同步检查、相关前端验证及 Claude Code PASS；不可因本机路径缺失而宣称完成。

## 实施顺序

`DEP-SP-01 -> SP-01 -> SP-02`；`DEP-SP-02 -> SP-01`、`DEP-SP-02 -> SP-02`。DEP-WEB-DOC 是网站文档交付条件（不是任务卡），也是发布前置，不阻塞本地代码验证。顺序边用于共享计划/文档写集串行化，不表示 SP-02 算法依赖 SP-01。

### 依赖登记表

| ID | 产物 / owner | 当前证据与可用条件 | 失败或超时处理 |
|---|---|---|---|
| DEP-SP-01 | 范围采纳与实现授权；维护者，尚未指定具名负责人 | 用户当前要求修复准备；两张卡仍 pending，待明确接受并授权实现 | 保留草案，不自行实现或发布；无虚构到期日 |
| DEP-SP-02 | storage/status 写集交接；相关计划执行者，待核对 | MCP MC-32 未完整收口、MC-33 pending；结构拆分计划需刷新。开工确认对应文件无活动 writer，记录具体来源 SHA/交接；不要求等待整个 MCP 计划 | 有重叠则串行等待或修订依赖；不得默认并发，无超时抢占 |
| DEP-WEB-DOC（外部条件，不是任务卡） | 网站 status 页具体路径与验证命令；网站维护者待指定 | 本机 `/Volumes/Sea/libra-backend` 不存在；交付前须读取 cf 仓库并将逐文件写集及实际命令补入本计划 | 本地验证可继续；网站及发布不得标 complete，不擅自新建替代仓库 |

### 发布分组与并发窗口

SP-01、SP-02 各为独立发布切片，顺序 SP-01 -> SP-02，无 REL 合并发布例外。实现写集含共同文档，禁止并发；发布写集由届时指定的单一发布者串行维护。当前无发布窗口或发布者授权。

### Phase 0：准备

进入条件：热点证据已获得。退出条件：范围采纳、源码锚点刷新、DEP-SP-01/02 满足；当前尚未退出。

### Phase 1：pack 查询

进入条件：Phase 0 退出。退出条件：SP-01 本地门与 review 通过，独立发布按授权完成。

### Phase 2：HEAD 复用

进入条件：SP-01 完成且 DEP-SP-02 针对 status 重新核对。退出条件：SP-02 及最终全量、复测、文档、发布门完成。

## 任务卡

### 字段全局默认与例外

Task type=implementation；Release boundary=independent；Release write set=`Cargo.toml`、`install.sh`、`install.ps1`、工具链生成的 `Cargo.lock`、release artifact（发布前按权威 target 复核）；Migration=N/A；Security 继承全局约束；Performance 按各卡计数门。无 waiver。

### Task SP-01：普通 pack 索引查询消除小读取放大

**Task type:** `implementation`

**Lifecycle / Acceptance:** `pending` / 空

**Description:** 普通对象读取使用固定大小 fanout 解码及既有桶内二分算法。通过私有 `Read + Seek` 抽象或等价测试接缝计数，不扩展公开 API。

**Out of scope:** HEAD 重复遍历由 SP-02 承接；跨调用 idx 缓存、pack 目录缓存尚未排期。

**Current evidence:** `src/utils/storage/local.rs:1281` 逐槽 fanout、`:1305` 线性查询、`:1375` 既有二分；同文件 `checked_batch_binary_index_finds_bucket_edges_and_misses` 可复用 fixture。

**Acceptance criteria:**

- [ ] AC-1 普通入口在 v1/SHA1、v2/SHA1/SHA256/BLAKE3 fixture 中返回预期 offset 或 missing（含空桶、桶边界及大 offset）。
- [ ] AC-2 fanout 正常文件由一次 1024-byte `read_exact` 完成；短读 reader 仍遵守 Read 契约，不把逻辑调用数冒充所有平台的 syscall 数。
- [ ] AC-3 容量 B 的桶最多比较 `ceil(log2(B + 1))` 个名称，B=0 时零比较。
- [ ] AC-4 截断/非法 fanout fixture 返回受控错误，不能静默视为对象缺失。
- [ ] AC-5 packed-object 既有读取回归通过。

**Verification:** 下列新增测试名是实施约定，当前不存在；每条须实际匹配测试后才可验收。

1. `cargo nextest run --lib ordinary_pack_index_lookup_matrix`（new，AC-1）。
2. `cargo nextest run --lib pack_index_fanout_bulk_read_contract`（new，AC-2）。
3. `cargo nextest run --lib ordinary_pack_index_logarithmic_comparisons`（new，AC-3）。
4. `cargo nextest run --lib ordinary_pack_index_invalid_input`（new，AC-4）。
5. `cargo nextest run --lib utils::storage::local::tests`（existing，AC-5）。

**Full-suite trigger:** T-1，共享对象存储读取 helper；最终收口再次跑全量。

**Dependencies:** DEP-SP-01、DEP-SP-02；交付前 DEP-WEB-DOC。

**Deliverables:** 普通路径查询修复、同模块测试、性能边界说明及复测摘要。

**Implementation write set:** `src/utils/storage/local.rs`；`docs/commands/status.md`；`docs/commands/zh-CN/status.md`；`docs/development/commands/status.md`；本计划及 `docs/development/plan/plan-status.md`。网站文件待 DEP-WEB-DOC 具名后才能进入交付，不能使用目录通配符替代登记。

**Release write set:** Inherited。

**Files likely touched:** 与 Implementation write set 相同；新增同模块测试不创建新 target。

**Docs and compatibility impact:** EN/zh-CN status 文档说明已验证的性能边界，开发文档描述查询算法；网站同步对应页。无参数、输出、稳定错误码或兼容功能增删，`COMPATIBILITY.md`、`docs/error-codes.md`、`tests/INDEX.md` 无需改动；若实现改变该判断须先修订写集。

**Rollback mode:** `immutable-release`；发布后发修复版本，用户可降级到该卡之前的已发布二进制；对象/index 格式不变，兼容窗口持续至另行宣布格式迁移。

**Migration and rollback:** 无迁移，不改已有对象库内容。

**Security and privacy:** 对 fanout 单调性/长度及 offset 运算做错误检查，禁止依赖未验证的输入触发 panic 或无界分配。

**Performance budget:** 固定 1024-byte fanout 缓冲；名称比较 O(log B)，不按对象总数分配新缓存。

**Estimated scope:** S（一个行为落点、一个生产文件）。

**Version increment:** patch；不预占具体版本号。

**Release boundary:** independent。

**C/D coverage from:** self。

**Granularity:** `AC=5/8; VER=5/8; landing=1; prod-files=1; scope=S; recovery=immutable-release; writeset=serial; exception=N/A`。

### Task SP-02：status 共享一次 HEAD 树展开

**Task type:** `implementation`

**Lifecycle / Acceptance:** `pending` / 空

**Description:** 在 status 收集内部复用按同一 HEAD OID 解析的树映射，供 staged 比较和 rename 使用；其它调用者保留原入口。保持快照生命周期局限于一次调用。

**Out of scope:** 不顺带改 index 读取次数、ahead/behind、rename 开关、stat IPC 或缓存写回；这些仍未排期。

**Current evidence:** `src/command/status.rs:996` staged 展开；`src/command/status/scan.rs:178`、`:1327` rename 再展开；旧版开启 rename 的 idx 读取数恰好翻倍。

**Acceptance criteria:**

- [ ] AC-1 clean 仓库启用 rename 时，staged 与 rename 对同一 HEAD 的完整树展开总计一次。
- [ ] AC-2 staged rename fixture 与既有预期输出相同。
- [ ] AC-3 unborn HEAD 的 status 结果与既有预期相同。
- [ ] AC-4 冲突状态仍保持 unmerged 分类。
- [ ] AC-5 status 既有格式/开关回归通过。
- [ ] AC-6 缺失 HEAD tree 对象不会被共享快照转换为空的 clean 结果。

**Verification:** 新测试须计数实际 loader 调用，不能只断言缓存字段存在；命令零匹配视为未验证。

1. `cargo nextest run --lib status_head_snapshot_expands_once`（new，AC-1；私有接缝测试）。
2. `cargo nextest run --test command_test test_status_find_renames_detects_content_rename`（existing，AC-2）。
3. `cargo nextest run --test command_test test_status_empty_repository`（existing，AC-3）。
4. `cargo nextest run --test compat_conflict_status_diff`（existing，AC-4）。
5. `cargo nextest run --test command_test status_test`（existing，AC-5）。
6. `cargo nextest run --test command_test test_status_missing_head_tree_is_error`（new，AC-6）。

**Full-suite trigger:** T-1，共享 staged 比较 helper；最终收口再次跑全量。

**Dependencies:** SP-01、DEP-SP-01、DEP-SP-02；交付前 DEP-WEB-DOC。

**Deliverables:** 单次调用 HEAD 快照复用、确定性计数与错误回归、同环境基准摘要。

**Implementation write set:** `src/command/status.rs`；`src/command/status/scan.rs`；`tests/command/status_test.rs`；`tests/SERIAL_REGISTRY.tsv`；`.config/nextest.toml`；`docs/commands/status.md`；`docs/commands/zh-CN/status.md`；`docs/development/commands/status.md`；本计划及 `docs/development/plan/plan-status.md`。新增串行测试按既有 lane 登记并运行 `sh tests/NEXTEST_GROUPS.sh`；网站文件待 DEP-WEB-DOC 补齐。

**Release write set:** Inherited。

**Files likely touched:** 与 Implementation write set 相同，不新增 test target。

**Docs and compatibility impact:** EN/zh-CN 命令页及网站说明一次调用内的复用范围，开发页记录 HEAD 快照生命周期。无新选项、输出字段或错误码，`COMPATIBILITY.md`、`docs/error-codes.md`、`tests/INDEX.md` 不变；公开行为若改变必须先修订计划。

**Rollback mode:** `immutable-release`；发布后以前滚版本修复，可降级至 SP-02 前的已发布版本；无数据格式变化，兼容窗口持续至另行宣布格式迁移。

**Migration and rollback:** 无迁移，无 index 自动刷新。

**Security and privacy:** 不吞掉对象读取错误；不放宽 conflict 排除、rename 比较预算或 helper 安全边界。

**Performance budget:** 同次 staged/rename 最多一次完整 HEAD 展开；共享已有映射，不保留第二份全量拷贝。

**Estimated scope:** S（一个行为落点、两个生产文件）。

**Version increment:** patch。

**Release boundary:** independent。

**C/D coverage from:** self。

**Granularity:** `AC=6/8; VER=6/8; landing=1; prod-files=2; scope=S; recovery=immutable-release; writeset=serial-after-SP-01; exception=N/A`。

## 测试矩阵

| 面 | 归属 | 验证 |
|---|---|---|
| v1/v2、hash kinds、missing/大 offset、损坏输入 | SP-01 | 新普通入口矩阵 + 既有 local storage 单测 |
| fanout 读取、桶比较上界 | SP-01 | 注入计数 reader；逻辑工作量硬门 |
| clean/rename/unborn/conflict/缺失对象 | SP-02 | 私有 loader 计数 + command/compat 回归 |
| 公共 helper 的跨命令影响 | 两卡 | 各卡 T-1 全量；最终树再全量 |
| 端到端 CPU/elapsed | 两卡 | 下节固定协议；不把单次耗时作为 CI 硬门 |

## 追溯表

| 缺口 | 卡 | 实现 | 测试 |
|---|---|---|---|
| GAP-SP-01 | SP-01 | local.rs 普通 idx lookup | SP-01 VER-1..5 |
| GAP-SP-02 | SP-02 | status staged/rename snapshot | SP-02 VER-1..6 |

## 里程碑验收与回滚

M0：范围和交接确认；M1：SP-01 完整独立验收；M2：SP-02 完整独立验收及最终基准。当前均未完成。

### 故障恢复矩阵

| 情况 | 动作 | 验证 |
|---|---|---|
| 查询 offset 错误或损坏索引被吞错 | 停止该卡发布，修复输入检查/定位算法 | SP-01 矩阵与全量重新通过 |
| status 误报 clean 或 rename 丢失 | 停止 SP-02 发布，修复快照传递 | SP-02 错误、rename、conflict 回归重跑 |
| 已发布版本回归 | 新 patch 前滚，必要时用户安装前一版本 | 原始复现 + 最终全量；不改写 tag |
| 文档环境不可用 | 保留本地验收状态，等待 DEP-WEB-DOC | 网站同步与部署门不能标完成 |

## 风险登记

| 风险 | 处理 |
|---|---|
| 只测 v2 而实际热点为 v1 | 普通入口测试显式覆盖两种布局 |
| 新增缓存引入 GC/repack 失效问题 | 首轮不新增跨调用缓存 |
| snapshot 重构改变错误/冲突处理 | 缺失对象错误与 unmerged 回归作为硬门 |
| 混用 stale/refreshed index 夸大收益 | 每组记录 stat 命中数，同 fixture 比较 |
| 工作集或工具链版本漂移 | 记录两个二进制来源、版本和 fixture 摘要；结果只适用于记录环境 |

## 性能与容量摘要

硬验收使用工作量计数，不预设缺乏依据的“耗时下降 80%”。实现前保存可复跑的旧 release 二进制，后续同机、同临时 fixture 用新 release 对照：每组 3 次预热 + 10 次计时，交替旧/新顺序，记录 wall、user、sys、RSS 中位数；进程失败不计作快速样本。

矩阵为 `status`、`status -uno`、`status -uno --no-renames`，分别测试 stat 失效与 refresh 命中；Git fixture 也 refresh，固定同一文件集/HEAD，单独披露仍有内容差异的文件。插桩仅核验 idx 调用次数，不参与时间比较。结果内联本节，发生稳定退化时重新定位，不能用计数改进掩盖整体回归。

现有 `benchmark/run.sh` 有 status_clean/status_dirty，但其 fixture 及结果聚合没有覆盖本次 v1 packed-tree 热点、Git 对照与 CPU 归因，不能单独充当本计划验收。本计划不顺带重写整个 benchmark runner；其 README 中过时的 `LIBRA_SKIP_WEB_BUILD` 示例另行处理。

## 兼容与文档收口

两卡不改 CLI 语义或磁盘格式。具体 EN、zh-CN、开发文档已逐文件登记；网站精确文件和命令必须通过 DEP-WEB-DOC 补齐后交付。新测试置于现有 target；若实际新增/重命名 target，先更新写集并同步 INDEX/Cargo 注册。准备阶段仅创建本计划并登记状态，不更新命令文档为“已修复”。

## Codex review log

尚未进行正式评审，无 PASS/FAIL 行；准备时结构核对不替代实现 review。

## 非目标与延后项

- DEFER-SP-01：status 自动刷新 stat/index 的只读语义、锁和并发写回设计，尚未排期。
- DEFER-SP-02：tracked stat 批量 IPC，需单独定义超时、取消、安全路径和队列预算，尚未排期。
- DEFER-SP-03：存储路径解析、executor 跳转、pack 目录/索引缓存，首轮复测后按剩余成本排序。
- DEFER-SP-04：永久 Git 对照/CPU benchmark 扩展与 README 陈旧示例清理，尚未排期。

## 完成判据

- [ ] 范围采纳、写集交接和网站依赖闭合。
- [ ] 两卡 AC/VER 全部有实际证据，最终树全量通过。
- [ ] 各卡 review 明确 PASS，未遗留 P0/P1。
- [ ] 前后 CPU/elapsed 与工作量数据内联，说明剩余瓶颈。
- [ ] 文档、版本与全部适用发布/远端门完成，才标记两卡 `done/complete`。
