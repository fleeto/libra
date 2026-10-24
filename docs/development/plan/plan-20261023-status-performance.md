# status 性能修复计划（2026-10-23）

## 本轮执行裁决（2026-10-23 15:16:46 UTC）

用户已要求实施 SP-03～SP-08。原准备阶段的 pending、Nextest、发布及等待依赖描述保留为历史，不作为本轮本地执行门。最新用户要求优先：仅本地修改，只做相关单元测试与 Libra/Git status 性能比较，不运行 nextest、不做 commit/push/安装/发布、不改真实 benchmark 仓库 index。无关联的新 Issue，不创建远端 Issue。当前分支 status-speedup，保留已有本地修改。

目标：2114 tracked 文件、stat 命中、干净工作树的默认 status，每轮新版/Git 暖跑中位数比 ≤5。历史测量：安装版 1713 ms；项目 release 原仓库 1470 ms；同一隔离副本缓存刷新前 1466 ms、刷新后 950 ms；Git 22 ms。历史数值不能替代最终验收。原仓库 2114 条 stat 为零，不自动刷新。

### 统一字段与约束

- Release boundary：local-only；Version increment / C/D coverage from / Release write set：N/A。本轮不宣称正式发布门通过。
- Rollback mode：revert，只撤销本卡差异、保留用户已有工作；无 schema/index 格式迁移。
- 公共 CLI、输出、错误码不变；保留 no-follow、beneath、root 检查、deadline、partial/fail-closed 与预算。不新增依赖、daemon、监控或跨调用缓存。
- 各卡随附写集：本计划、plan-status.md、docs/development/commands/status.md；新增串行单元测试同步 tests/SRC_SERIAL_REGISTRY.tsv。
- 证据写入本计划，临时 fixture/原始 JSON 位于 /tmp，不新增库内证据目录。实施中每卡记录 UTC 时间、真实命令和非零测试数；零匹配不能算通过。
- 单元测试使用现有 LIBRA_TEST_HOME 隔离用户状态；运行 fmt --check 与 libra diff --check。不把相关单测描述为全量通过。

### 新增依赖与顺序

| ID | 产物/约束 | 可用条件与失败路径 |
|---|---|---|
| DEP-SP-LOCAL | 已有 SP-01/02 与 restore 本地差异 | 已存在；先核对、不得覆盖 |
| DEP-SP-BENCH | 三个二进制、真实仓库与隔离 fixture | 记录版本/HEAD/内容/stat；失败不作性能结论，独立单测可继续 |
| DEP-SP-WRITER | 写集没有并发 writer | 当前串行执行；发生冲突保留他人修改并协调 |

顺序：SP-03 → SP-04 → SP-05 → SP-06 → SP-07 → SP-08。无并行 Agent、无发布窗口。

### 本轮任务卡

### Task SP-03：完成恢复流程 stat 缓存修复

**Task type:** `implementation`

**Lifecycle / Acceptance:** `done` / `locally-accepted`

**Description:** 沿用当前 restore_index_tracked 的 refresh_candidates 修复；只为经过内容验证的工作树缓存 stat。

**Current evidence:** src/command/restore.rs::restore_index_tracked 已实现基础刷新；隔离副本 2114 条全部命中。 开工时刷新行号。

**Dependencies:** DEP-SP-LOCAL、DEP-SP-WRITER

**Out of scope:** 统一约束所排除的行为；不自动执行发布或真实仓库迁移。

**Acceptance criteria:**

- [x] AC-1 同时恢复工作树和索引后匹配的普通文件获得有效 stat。
- [x] AC-2 仅恢复索引不为对象新条目采信工作树 stat。
- [x] AC-3 同大小内容修改不能刷新成干净。
- [x] AC-4 缺失文件不获得有效 stat。
- [x] AC-5 可执行位不匹配保留 dirty 判定。
- [x] AC-6 符号链接按链接内容校验。
- [x] AC-7 skip-worktree 不被错误刷新。

**Verification:** 新过滤名为实施约定，必须非零匹配；环境通过 `LIBRA_TEST_HOME` 隔离。

1. `rtk proxy cargo test --lib command::restore::tests`。
2. `rtk proxy cargo test --lib utils::stat_diff::tests`。

**Implementation write set:** src/command/restore.rs；src/command/refresh.rs（仅边界测试证明有缺陷时）；加统一随附写集。

**Deliverables:** 缓存修复、单元覆盖、历史零 stat 适用边界。

**Docs and compatibility impact:** 内部缓存行为；status 仍不回写 index。

**Rollback mode:** `revert`，仅本卡差异。

**Version increment / C/D coverage from / Release write set:** N/A / N/A / N/A。

**Release boundary:** local-only。

**Estimated scope:** S。

**Granularity:** `AC=7/8; VER=2/8; prod-files≤2; scope=S; writeset=serial`。

### Task SP-04：建立 status 性能归因与比较工具

**Task type:** `implementation`

**Lifecycle / Acceptance:** `done` / `locally-accepted`

**Description:** 新增 benchmark/status.sh 三方 benchmark；默认关闭的 status-perf feature 提供阶段计数，写显式临时诊断文件，不写正常 stdout/stderr。普通 release 排除插桩；阶段覆盖启动/preflight、HEAD/index、tracked、untracked、rename、其它元数据与输出，计数 worker 提交与内容哈希。

**Current evidence:** benchmark/run.sh 当前未提供本会话所需三方资源比较。 开工时刷新行号。

**Dependencies:** SP-03、DEP-SP-BENCH、DEP-SP-WRITER

**Out of scope:** 统一约束所排除的行为；不自动执行发布或真实仓库迁移。

**Acceptance criteria:**

- [x] AC-1 记录二进制校验和、版本、HEAD、文件数和 stat 命中率。
- [x] AC-2 报告中位数和 P95。
- [x] AC-3 报告 user/system CPU。
- [x] AC-4 报告进程树内存采样及短峰值遗漏限制。
- [x] AC-5 报告 voluntary/involuntary context switches。
- [x] AC-6 子命令失败不得成为成功性能样本。
- [x] AC-7 普通 release 不包含诊断路径。
- [x] AC-8 计数可区分 worker 提交和文件哈希。

**Verification:** 新过滤名为实施约定，必须非零匹配；环境通过 `LIBRA_TEST_HOME` 隔离。

1. `rtk proxy cargo test --lib --features status-perf status_perf_`。
2. `rtk proxy cargo build --release --bin libra`。
3. `rtk proxy bash benchmark/status.sh --installed-libra /home/dusti/.libra/bin/libra --built-libra /home/dusti/projects/libra/target/release/libra --git /usr/bin/git --libra-repo /home/dusti/projects/libra-lbr --git-repo /home/dusti/projects/libra-git --warmup 3 --runs 20 --rounds 3 --output /tmp/libra-status-performance.json`。

**Implementation write set:** benchmark/status.sh；benchmark/README.md；Cargo.toml；src/main.rs；src/cli.rs；src/utils/mod.rs；src/utils/status_perf.rs（新增）；src/command/status.rs（输出阶段计时）；src/command/status/scan.rs；src/command/status_untracked.rs；src/internal/worktree_io/executor.rs；加统一随附写集。

**Deliverables:** benchmark、诊断计数、分阶段基线。

**Docs and compatibility impact:** 默认关闭的开发 feature；公共 CLI 不变。

**Rollback mode:** `revert`，仅本卡差异。

**Version increment / C/D coverage from / Release write set:** N/A / N/A / N/A。

**Release boundary:** local-only。

**Estimated scope:** M。

**Granularity:** `AC=8/8; VER=3/8; prod-files≤9; scope=M; writeset=serial`。

### Task SP-05：批量 stat 与有界调度

**Task type:** `implementation`

**Lifecycle / Acceptance:** `done` / `locally-accepted`

**Description:** 每批最多 64 路径、序列化请求预算 256 KiB，最多 4 批并发，继续受现有 frame cap 限制。超长单项走原单项限制。逐路径开始/终态携带序号，按输入顺序归并。deadline 按路径，不由其它进度延期；超时保留完成项，blocked 当前项，重调度尚未开始项。取消不再提交。非法/截断协议 fail-closed。保持原 root 打开方式，不新增描述符缓存。

**Current evidence:** status_untracked.rs::collect_tracked_worktree_changes 逐条 deadline_stat；executor.rs::submit_with_token 同步等待。 开工时刷新行号。

**Dependencies:** SP-04、DEP-SP-WRITER

**Out of scope:** 统一约束所排除的行为；不自动执行发布或真实仓库迁移。

**Acceptance criteria:**

- [x] AC-1 2114 普通路径正常 stat 批次数为 ceil(2114/64)=34。
- [x] AC-2 活跃批次≤4。
- [x] AC-3 批量与单项结果一致。
- [x] AC-4 完成顺序不影响输出顺序。
- [x] AC-5 单路径超时保留完成结果。
- [x] AC-6 取消停止剩余提交。
- [x] AC-7 非法/截断协议不产生伪 clean。
- [x] AC-8 路径逃逸防护有效。

**Verification:** 新过滤名为实施约定，必须非零匹配；环境通过 `LIBRA_TEST_HOME` 隔离。

1. `rtk proxy cargo test --lib status_stat_batch_`。
2. `rtk proxy cargo test --lib internal::worktree_io::executor::tests`。
3. `rtk proxy cargo test --lib command::status_io_worker`。

**Implementation write set:** src/internal/worktree_io/protocol.rs；src/internal/worktree_io/handler.rs；src/internal/worktree_io/executor.rs；src/command/status_io_worker.rs；src/command/status_untracked.rs；加统一随附写集。

**Deliverables:** 批量协议、调度、partial 处理、真实 helper 单元用例及资源对比。

**Docs and compatibility impact:** 内部协议变化，父子同一二进制，不支持跨版本 helper。

**Rollback mode:** `revert`，仅本卡差异。

**Version increment / C/D coverage from / Release write set:** N/A / N/A / N/A。

**Release boundary:** local-only。

**Estimated scope:** M。

**Granularity:** `AC=8/8; VER=3/8; prod-files≤5; scope=M; writeset=serial`。

### Task SP-06：合并 stat 与条件哈希

**Task type:** `implementation`

**Lifecycle / Acceptance:** `done` / `locally-accepted`

**Description:** 请求携带必要 index 字段、index mtime、fileMode、hash kind；worker 返回 stat/可选哈希/错误，父进程保留分类。复用 stat_diff 与既有 LFS/符号链接哈希，不复制算法。

**Current evidence:** tracked stat 失效及 mode-only 修复候选当前另行 deadline_file_blob_hash。 开工时刷新行号。

**Dependencies:** SP-05、DEP-SP-WRITER

**Out of scope:** 统一约束所排除的行为；不自动执行发布或真实仓库迁移。

**Acceptance criteria:**

- [x] AC-1 可信 stat 命中普通文件哈希次数为零。
- [x] AC-2 stat 失效无需第二次父进程提交。
- [x] AC-3 racy-clean 仍确认内容。
- [x] AC-4 同大小修改可检出。
- [x] AC-5 mode-only 与修复候选语义一致。
- [x] AC-6 LFS 比较语义一致。
- [x] AC-7 符号链接比较语义一致。
- [x] AC-8 特殊索引跳过/占位规则一致。

**Verification:** 新过滤名为实施约定，必须非零匹配；环境通过 `LIBRA_TEST_HOME` 隔离。

1. `rtk proxy cargo test --lib status_tracked_probe_`。
2. `rtk proxy cargo test --lib utils::stat_diff::tests`。
3. `rtk proxy cargo test --lib command::status_io_worker`。

**Implementation write set:** src/internal/worktree_io/protocol.rs；src/internal/worktree_io/handler.rs；src/command/status_io_worker.rs；src/command/status_untracked.rs；src/utils/stat_diff.rs；加统一随附写集。

**Deliverables:** 合并 probe、单元覆盖、stat 命中/失效资源数据。

**Docs and compatibility impact:** 不改变公共行为或 index 写回。

**Rollback mode:** `revert`，仅本卡差异。

**Version increment / C/D coverage from / Release write set:** N/A / N/A / N/A。

**Release boundary:** local-only。

**Estimated scope:** M。

**Granularity:** `AC=8/8; VER=3/8; prod-files≤5; scope=M; writeset=serial`。

### Task SP-07：调用内共享 index 快照

**Task type:** `implementation`

**Lifecycle / Acceptance:** `done` / `locally-accepted`

**Description:** 调用上下文持有 index 与快照时间戳，通过引用供 staged/worktree/unmerged/rename/porcelain 消费；其它调用者的独立入口保留包装函数，不跨调用缓存。

**Current evidence:** status.rs staged 与 status_untracked.rs 分别 Index::load；scan 另有加载入口。 开工时刷新行号。

**Dependencies:** SP-06、DEP-SP-WRITER

**Out of scope:** 统一约束所排除的行为；不自动执行发布或真实仓库迁移。

**Acceptance criteria:**

- [x] AC-1 默认 status 实际 index 加载一次。
- [x] AC-2 不保留第二份完整 index 副本。
- [x] AC-3 缺失/损坏保持错误语义。
- [x] AC-4 unmerged 分类一致。
- [x] AC-5 rename 分类一致。
- [x] AC-6 porcelain 与同次收集使用相同 index。
- [x] AC-7 下一次调用可见 index 变化。

**Verification:** 新过滤名为实施约定，必须非零匹配；环境通过 `LIBRA_TEST_HOME` 隔离。

1. `rtk proxy cargo test --lib status_index_snapshot_`。
2. `rtk proxy cargo test --lib status_head_snapshot_expands_once_for_staged_and_rename`。

**Implementation write set:** src/command/status.rs；src/command/status/scan.rs；src/command/status_untracked.rs；加统一随附写集。

**Deliverables:** 调用级快照、兼容包装、实际 loader 计数与错误测试。

**Docs and compatibility impact:** 内部接口调整，CLI 不变。

**Rollback mode:** `revert`，仅本卡差异。

**Version increment / C/D coverage from / Release write set:** N/A / N/A / N/A。

**Release boundary:** local-only。

**Estimated scope:** M。

**Granularity:** `AC=7/8; VER=2/8; prod-files≤3; scope=M; writeset=serial`。

### Task SP-08：Git 五倍目标与本地证据收口

**Task type:** `audit`

**Lifecycle / Acceptance:** `in-progress` / 空

**Description:** 冻结二进制/fixture，使用默认 status（不禁用 rename/untracked），每轮预热3次、交错20次、共3轮。比较安装 Libra/新 release/Git。主门使用同内容同状态隔离 fixture；保留真实仓库对照。仅审计，不临时改生产实现。

**Current evidence:** 历史缓存命中950ms、Git22ms；仍约1.9万自愿上下文切换，五倍目标未验证。 开工时刷新行号。

**Dependencies:** SP-07、DEP-SP-BENCH、DEP-SP-WRITER

**Out of scope:** 统一约束所排除的行为；不自动执行发布或真实仓库迁移。

**Acceptance criteria:**

- [ ] AC-1 三轮主场景新版/Git中位比均≤5。
- [x] AC-2 CPU 相对同轮冻结基线不恶化超过10%。
- [ ] AC-3 同口径进程树内存不恶化超过10%。
- [ ] AC-4 零stat/少量dirty/较多untracked/rename无超过10%的未解决回退。
- [x] AC-5 1k/10k/50k规模不突破并发/队列上限。
- [x] AC-6 真实仓库 index 校验和不变。
- [x] AC-7 默认输出与退出状态符合预期。
- [x] AC-8 区分本地验证、目标达成、未执行发布门。

**Verification:** 新过滤名为实施约定，必须非零匹配；环境通过 `LIBRA_TEST_HOME` 隔离。

1. `SP-04 benchmark/status.sh 在真实仓库三方复测`。
2. `相同参数在同内容同状态隔离 fixture 主门复测`。
3. `相同参数扩展场景复测`。
4. `测量前后真实仓库 index 校验和比较`。
5. `计划记录人工核对`。

**Implementation write set:** 本计划；docs/development/plan/plan-status.md；docs/development/commands/status.md；加统一随附写集。

**Deliverables:** 三方表、分阶段收益、目标判定、剩余热点与复测命令。

**Docs and compatibility impact:** 仅实测结论，不新增性能保证。

**Rollback mode:** `revert`，仅本卡差异。

**Version increment / C/D coverage from / Release write set:** N/A / N/A / N/A。

**Release boundary:** local-only。

**Estimated scope:** S。

**Granularity:** `AC=8/20; VER=5/20; prod-files≤0; scope=S; writeset=serial`。

### 目标失败与执行证据

SP-08 未达标不得标验收通过。保留有效优化，记录差额和剩余热点；不关闭功能、不放宽安全边界、不改变测量口径达标。超出已声明任务的热点另立针对性卡。

- SP-01/02：已有本地实现和历史 focused 单元测试证据；无正式发布/全量/评审 PASS。
- SP-03：基础实现及 restore 8 项、stat_diff 1 项单测通过；扩展边界验收待执行。

## 历史准备正文（以下门被本轮执行裁决覆盖）

## 文档职责

修复已由本机测量和源码交叉定位的 pack 索引读取放大、HEAD 树重复展开。本文依据 plan-template v2.14，实施范围以最新用户裁决为准。本文已扩展为 SP-01～SP-08 的本地性能优化计划；本地验证与正式发布验收分开登记。

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

## 执行记录 2026-10-23 22:37:54 UTC

- 未运行 nextest。SP-03：`LIBRA_TEST_HOME=/tmp/libra-status-unit-home LIBRA_TEST=1 cargo test --lib command::restore::tests`，9 passed，包括新增 missing/exec/symlink/skip-worktree；既有 stat_diff 1 passed 证据保留。
- SP-04：诊断 feature 单测首次编译被 SIGKILL，串行重跑 1 passed；诊断 release 构建成功。普通 release 已构建成功。既有 ingress dead_code 与 proc-macro-error2 future-incompat warning 保留。
- SP-04 诊断样本（仅归因）：head/index 184.406 ms，tracked 320.019 ms，untracked 370.512 ms，metadata 6.963 ms，rename 0.699 ms，parse/preflight 19.598 ms；worker_requests=2724，stat 命中无内容哈希。阶段存在嵌套，不相加作为最终性能门。
- benchmark/status.sh 工具短跑成功（1 warmup/3 runs/1 round）；与编译存在并行，**不作为性能证据**。资源脚本区分 wait4 主进程+已回收子进程 CPU 与 /proc 采样进程树 CPU/RSS；后者有 tick/采样误差、共享页重复计数。
- SP-05～07：本地实现已写入；完整 focused 验证和正常 release 性能门待完成，未标验收通过。

### 2026-10-24 02:39:46 UTC 本地验证与并发调优

- focused 单元测试：status_io_worker 23、executor 16、status 9、stat_diff 1、restore 9、status_perf 1，共59 passed。真实 helper FIFO 超时在150ms窗口返回，完成前缀保留。初次新增测试中类型/作用域编译错误已修正，最终上述命令通过；未使用 nextest。
- 四 helper 的隔离副本暖跑（3×20）：新版中位 613/616/618 ms，进程树RSS采样峰值约94MiB，相比安装版约56MiB违反内存回退门。该批数据另发现两个 PowerShell 文件的 LF/CRLF 差异，故只作诊断，不计同内容主验收。
- 在“最多4批并发”的上限内，将默认并发收敛为1；单批直接提交，不创建额外提交线程。保留 bounded 调度及显式 hash kind。普通 release 已重建；新增加2114路径实际34次提交计数测试及隐藏 index 后仍能完成 staged/worktree/v2 的快照测试，正在复验。
- 隔离 Git 副本两文件改为与 Libra 相同的LF，并通过 git add 更新stat；两文件blob OID不变，Git工作树重新干净。未修改真实两个benchmark仓库。基准脚本新增内容摘要与HEAD匹配判定，不匹配不能判性能目标通过。

### SP-05 规模回退前滚修复

SP-08 的50k短跑发现 RSS 采样 77996→140096 KiB，不能标资源验收通过。审计暂停，回到已有 SP-05 写集修复：提交批次有界之外，准备的路径/check及待消费结果也须逐批持有。属于既有队列/资源预算修复，不新增热点优化范围。随后重新构建、相关单测及受影响性能验证。

## 最终本地验收记录 2026-10-24 08:24:25 UTC

当前普通 release 位于 `target/release/libra`，不带 status-perf。未提交、安装或发布。SP-03～07 已完成本地实现与相关验证；SP-08 审计已执行但**五倍目标与内存门未通过**，计划不标完成。

### 验证范围

- 相关单元测试累计60项通过：restore 9、stat_diff 1、status_perf 1、status 9、status_io_worker 24、executor 16。最后分批消费修复后重跑 status 9 和 status_io_worker 24，33项通过。
- 使用 `LIBRA_TEST_HOME=/tmp/libra-status-unit-home LIBRA_TEST=1 cargo test --lib --features status-perf <filter>`；restore/stat_diff早期命令和日志见前文。实际2114路径34批次测试、真实helper超时、隐藏index后的单快照测试均通过。
- 最终 `cargo build --release --bin libra` 成功；fmt检查及最终 `libra diff --check` 通过。未运行 nextest、全量测试、集成测试或 clippy；不宣称这些门通过。

### 三轮主对比

每轮预热3次、交错测量20次。下表按每轮中位数报告，单位ms；所有输入在测量前后校验不变。clean/control位于/tmp，真实仓库位于项目磁盘，不能跨文件系统拼接比值。

| 场景/轮 | 安装版或冻结基线 | 新版 | Git | 新版/Git |
|---|---:|---:|---:|---:|
| clean/1 | 1129.61 | 637.09 | 6.98 | 91.32 |
| clean/2 | 1142.60 | 639.66 | 7.35 | 87.04 |
| clean/3 | 1150.84 | 655.00 | 7.53 | 87.03 |
| control/1 | 925.37 | 642.78 | 7.12 | 90.29 |
| control/2 | 914.47 | 645.39 | 7.23 | 89.24 |
| control/3 | 900.57 | 637.26 | 7.11 | 89.64 |
| original/1 | 1620.36 | 791.64 | 21.42 | 36.96 |
| original/2 | 1612.09 | 785.78 | 20.34 | 38.62 |
| original/3 | 1623.98 | 783.69 | 21.02 | 37.28 |

clean/original 的 installed 为本机安装版；control 的 installed 字段实际是冻结的 `/tmp/libra-sp03-baseline`，已含SP-01/02/03，用于隔离本轮收益。

| 主场景二进制 | user中位数范围 ms | system中位数范围 ms | 自愿切换范围 | 进程树RSS采样峰值范围 KiB |
|---|---:|---:|---:|---:|
| clean/installed | 214.9–221.9 | 775.8–798.1 | 20634.0–20678.0 | 55500–56172 |
| clean/built | 141.2–171.0 | 433.1–439.9 | 12431.5–12508.0 | 56828–57796 |
| clean/git | 0.0–0.0 | 3.5–5.3 | 4.0–7.0 | 5632–5632 |
| control/installed | 187.4–196.4 | 575.3–596.3 | 19366.0–19404.5 | 33776–34744 |
| control/built | 155.5–168.8 | 416.5–440.4 | 12433.5–12463.5 | 58564–59448 |
| control/git | 0.0–0.6 | 3.2–4.8 | 5.0–6.0 | 5632–5632 |
| original/installed | 269.2–280.2 | 973.4–992.1 | 29017.0–29078.0 | 61896–62124 |
| original/built | 163.8–174.1 | 436.6–451.4 | 13492.0–13521.5 | 61040–62156 |
| original/git | 8.7–10.3 | 8.6–10.1 | 4.0–7.5 | 8840–9100 |

CPU来自wait4（父进程及已回收子进程）；进程树CPU另在原始JSON采样。RSS为独立运行每5ms采样的树RSS之和，重复计算共享页、可能遗漏短峰值，不能视为精确物理内存。Git短进程尤其易漏采。P95、非自愿切换、逐样本数据及stderr摘要均见原始JSON。

### 冻结输入

- installed: `/home/dusti/.libra/bin/libra`；libra 0.30.86；SHA256 `a384ddeec47c15cb8a13c8b51a6eb2a8d9f25c0e97fcaf58830e4b8208a49df5`。
- built: `/home/dusti/projects/libra/target/release/libra`；libra 0.30.86；SHA256 `cc06deac5c009a1ed27370c3d213cf6b1d668d5d80798d870c28dd23f7b45660`。
- git: `/usr/bin/git`；git version 2.53.0；SHA256 `ed8b2df973b28c0fb773b9c0dc0816d3ee2b3835bb932d9153c21417bb51b362`。
- clean/built: `/tmp/libra-stat-benchmark`，HEAD `83dfafd801025bb9cae3efb2af8b5bbc5f291f2c`，entries=2114，matching_stat=2114，zero_stat=0，index SHA256 `b00d76b0d4d2da64f45c9a1bf8f15947b674ebbea8207189f6b520168c47b483`。
- clean/git: `/tmp/libra-status-git-main`，HEAD `83dfafd801025bb9cae3efb2af8b5bbc5f291f2c`，entries=2114，matching_stat=2114，zero_stat=0，index SHA256 `3f646c640f71524e3cde2c458f9f0e8a3afbcefaa10e9bc8285d7536c0667787`。
- original/built: `/home/dusti/projects/libra-lbr`，HEAD `83dfafd801025bb9cae3efb2af8b5bbc5f291f2c`，entries=2114，matching_stat=0，zero_stat=2114，index SHA256 `69eaadd51ce9f758c84ec161d1c33ef4cead176e9d55ba373d78af3a0652b3f6`。
- original/git: `/home/dusti/projects/libra-git`，HEAD `83dfafd801025bb9cae3efb2af8b5bbc5f291f2c`，entries=2114，matching_stat=2114，zero_stat=0，index SHA256 `a23895dad7ac9c5f8559ac882b3442497774034965d072f92a728b7e4bd18b3d`。

clean/control 的HEAD与tracked内容摘要一致；真实仓库HEAD一致，但install.ps1与template/pre-commit.ps1存在CRLF/LF差异，故original仅作实际环境对照。只在隔离Git副本归一LF并git add刷新stat，blob OID未变化。真实Libra的2114条零stat保持原样。

### 扩展场景（探索性复测）

dirty/untracked/rename/zero/1k/10k：预热1次、5次测量、1轮；50k内存修复复测：预热1次、3次测量、1轮。不是三轮主验收的等强度证据。基线列均为冻结SP-03构建。

| 场景 | 基线 ms | 新版 ms | Git ms | 基线树RSS KiB | 新版树RSS KiB | 新版进程树峰值 |
|---|---:|---:|---:|---:|---:|---:|
| dirty | 935.74 | 653.50 | 7.45 | 36036 | 61820 | 2 |
| untracked | 926.53 | 707.03 | 7.48 | 34424 | 58808 | 2 |
| rename | 911.74 | 639.65 | 6.98 | 34128 | 59264 | 2 |
| zero | 1366.88 | 787.76 | 6.71 | 35960 | 62076 | 2 |
| 1000 | 408.74 | 278.38 | 4.20 | 32888 | 57472 | 2 |
| 10000 | 2744.20 | 1434.87 | 11.04 | 34172 | 58296 | 2 |
| 50000 | 13303.03 | 6797.97 | 36.20 | 57432 | 80888 | 2 |

- dirty：修改10个tracked Markdown文件；untracked：新增一个目录、1000个文件；rename：README.md改名并暂存；zero：隔离Libra复制真实零stat index，Git仍命中缓存，不能当缓存等价对比。
- 规模fixture：每500文件一个目录，内容统一为 `status scale fixture\n`；Git创建临时提交，隔离Libra复制对象并reset到相同HEAD，add --refresh后测量。只在/tmp准备fixture，真实仓库不写入。
- 七种场景新版与冻结基线的默认stdout逐字节一致，退出码均0。所有测量输入校验不变。源码每批64路径/256KiB，默认单批；当前生产调用按64条准备并消费，不持有全仓库probe结果。
- 50k修复前新版树RSS采样140096 KiB，修复后80888 KiB；修复后同轮基线57432 KiB，仍超过10%资源门。不同轮基线也有波动（早前77996 KiB），不能据此宣称回退消除。

### 判定与剩余工作

最终control主场景的冻结基线树RSS采样为33776–34744 KiB，新版58564–59448 KiB，主场景资源门也未通过；不只50k有差距。此前轮次冻结基线约55MiB，最终轮次约33MiB，变化原因尚未确认，需复核进程树采样覆盖与内存组成。保留原始数据，不能选择较有利轮次宣布通过。

五倍目标失败，SP-08不验收通过。批量IPC、条件hash、调用内index共享带来真实收益，但仍有大量系统时间/上下文切换。此前诊断显示HEAD/index和untracked扫描成本高；这是下一轮定位方向，不是当前精确阶段占比。规模内存残差也需继续归因，不能靠放宽门、关闭默认功能或弱化安全检查解决。新增热点实现须另立针对性卡，本次审计不擅自扩范围。

复测入口：`bash benchmark/status.sh --installed-libra <installed-or-frozen> --built-libra /home/dusti/projects/libra/target/release/libra --git /usr/bin/git --libra-repo <libra-fixture> --git-repo <git-fixture> --warmup 3 --runs 20 --rounds 3 --output /tmp/result.json`。

原始数据：`/tmp/libra-sp08-final-{clean,control,original}.json`、`/tmp/libra-sp08-aux-{dirty,untracked,rename,zero,1000,10000}.json`、`/tmp/libra-sp08-memory-fix-50000.json`、`/tmp/libra-sp08-output-checks.json`。修复前数据保存在 `/tmp/libra-sp08-pre-memory-*.json`；/tmp文件非持久交付，关键结果已内联。
