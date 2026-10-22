# Worktree 默认目录统一计划（2026-10-19）

**模板版本:** v2.14。**更新时间:** 2026-10-21 04:22 UTC。**执行分支:** Libra 原生仓库 `main`。**来源:** 用户直接要求统一 worktree 目录，明确不处理旧数据迁移，并授权 Implement the plan；无 linked Issue。

**发布目标变更（2026-10-20）：** 用户要求发布现有 `0.30.66`，明确不要发布 `0.31.1`。`v0.30.66` 是不含本计划改动的旧标签，已单独补建 GitHub Release；本计划的实现和验证继续，但其 CLI 发布与网站上线门暂不收口，不能借用旧版发布证据。

**当前发布决议（覆盖下文旧冻结与 minor 目标）：** 用户要求基于上游最新已发布版本 patch + 1 发布。2026-10-21 17:18 UTC 上游 `99a92c1` 已打 `v0.30.84` tag 并创建 GitHub Latest Release；此前 `.83` 候选未打 tag，本计划目标随用户规则前滚为 **`v0.30.85`**。先前“不发布 `0.31.1`”继续有效；旧 Worktree tag `v0.31.0` 不复用。WD-08 须在最终源码、远端 CI、四平台资产/stable 与网站内容门完成后才能收口。公开 NAME/--path 行为变化写入升级说明；不迁移旧目录。

**并发合流补充（2026-10-21 15:27 UTC）：** 上游在 `6d50c5a` 后推进到 `81eff71`，净新增 Issue #525 的 `commit` CLI/双语命令文档/兼容测试和 MCP 文件格式化；Worktree 实现及版本面不变。上游 push 取消了 `6d50c5a` 的网络 CI 重跑，所以该 run 的七个成功作业不能组成最终全绿。按工作区常驻交付指令，网站同步范围在原八张 Worktree 相关页之外增加一张 `commit` 英文命令页；这不增加第十张 Worktree 任务卡。`0.30.83` release notes 补记 Issue #525；最终源码 SHA 的本地与远端门重新核验后才发布。

**合流审查前滚（2026-10-21 15:42 UTC）：** Claude Code 对新上游 `commit` 行为的只读审查为 `VERDICT: FAIL`：`-S` 被解析却未强制签名，可能静默生成未签名提交；位置路径/`-o` 与文档承诺不符；`-u` 被忽略；`--short` 的预览语义和旧文案互相矛盾。发布前须前滚 `commit` 源码、双语命令/开发文档、兼容表、回归测试及网站 commit 页，再取得复审 PASS；原 FAIL 与首次 focused 两项失败保留为历史证据，不计最终门。上游最新 Release 已实时复查仍为 `v0.30.82`。

**上游再次前滚（2026-10-21 17:18 UTC）：** GitHub Latest 已变为 `v0.30.84`（`99a92c1`），用户“最新上游 patch + 1”规则使本次唯一候选改为 **`v0.30.85`**，不发布未打标的 `.83`，也不发布 `.31.1`。本地 `f0ea585` 的 final-SHA CI 8/8 与 nextest 10704/10704 均已通过，但旧树 `cargo test --all` 两轮分别因 capture deadline、fetch 镜像 I/O 偶发失败而 exit101，第三轮在上游发布变化后主动中断 exit130；均不得作为 `.85` 最终门。上游新引入 `branch -f`，发布前审查发现可重置当前检出分支并绕过已有 protect/archive 策略。FIX-WD-08g 写集增加 `src/command/branch.rs`、`tests/command/branch_test.rs`、双语 `docs/commands/branch.md`、`COMPATIBILITY.md`、`docs/error-codes.md`、网站 `branch.en.md`、`CHANGELOG.md`，以及既有版本面和本计划状态文件；须用事务内同一检查拒绝危险重置、做真实 CLI 回归，并重跑最终门及 Claude Code 复审。网站同步范围因此由九页扩为十页。
**FIX-WD-08g 复审前滚（2026-10-21 17:44 UTC）：** 独立源码审查字面 `VERDICT: FAIL` 的排序帮助与覆盖缺口已核实；其 fmt 失败判断由实际 `cargo +nightly fmt --all --check` 反证。进一步手工复现证实上游 `--create-reflog` 成功返回但分支 reflog 为空，`branch -f` 对已有分支误报 `Created branch` 且 JSON action 为 `create`。先在现有命令集成用例中得到真实红灯，再把 CLI 分支指针与 reflog 的写入纳入同一事务、正确输出 reset/旧新 OID；扩展配置排序和主/linked 反向检出边界测试，双语文档与网站 branch 页说明实际行为。跟进复审提出 reflog 文案的真实兼容性缺口，写集增加 `src/internal/reflog.rs` 的 `branch` action，并以精确 JSON action/message 锁定 `Created from` / `Reset to`；`0.30.83` tag 不存在、版本四面已更新、共享 helper 无其他 force 调用均已现场核实。任何写入失败须回滚、不发成功结果；最终 branch 全组、Display-pin、严格静态门、双全量、macFUSE、final-SHA CI 与 Claude Code 源码/网站复审必须重新通过。不处理旧数据迁移。
**WD-08 网站 v0.30.85 复审前滚：** 十页网站限定同步首轮 Claude Code 字面 `VERDICT: FAIL`，提示旧版兼容边界、Worktree 示例与 `commit -S` 导入签名来源等文档风险。逐项核实：历史计划之外的线上内容无旧路径命令；backend 只有英文源页，生成器将其镜像至 zh-CN 并把相对文档链接重写为 locale 绝对路径；v0.31.0 tag 源码确实理解 registry schema 4，但缺少后续身份清理保护。写集在 WD-08 原有双语 Worktree/upgrade/branch 与网站页外增加源 `docs/commands/{commit,review,investigate,sandbox}.md` 及其 zh-CN 对应页，明确能力标记与 registry 格式的边界、外部 CLI 沙箱配置、导入签名来源，并校准默认路径、list/add/remove/move JSON 与错误表。网站只通过同步器写十页；第二轮复审发现 move 示例把目标写入另一个托管名称，已改为命令实际的仓库外目标，并消除末段能力“4”的歧义；第三轮字面 `VERDICT: PASS`。最终限定同步 drift=0、同步器 8/8、文档 Vitest 11/11、typecheck/build exit0；文档提交后仍须以最终源码 SHA 复验全量、CI 与发布。

**2026-10-22 继续验收：** `fbdf5ee` 代码树的 `cargo test --all` 完整退出 0，189 个结果组共 10736 passed / 0 failed / 26 ignored。十页最终文档复审发现 schema 4 回退说明过度限定于命名条目；已修正双语 Worktree/upgrade：任何 registry 变更都会写 schema 4，即使无命名条目，后续 worktree 操作也须使用包含身份安全修复的版本。此项 Claude Code 跟进复审字面 `VERDICT: PASS`，文档兼容测试 2/2、fmt 与严格 Clippy 通过。上游最新正式版仍为 v0.30.84，目标 v0.30.85；main 已推进至 `acebe35`，旧 `fbdf5ee` CI 被取消，不能计最终绿门。已无冲突合入 `acebe35`；保留本计划全部勘误。合流仅引入上游已有代码，本地额外差异为已复审文档。网站已保存本地页面后快进至 `b895ade`，再由同步器按最终源码更新；同步范围补入上游公开变化的 `merge` 和 `op` 页（共十二页），不增加任务卡。最终树验证待完成。

## 文档职责

统一新建 linked、FUSE 和 review/investigate 工作区的默认布局到 common storage 的 `worktree/`，保持工作文件、工作区身份与共享元数据边界正确。本文是执行契约，不是实现或验收完成声明。

### 适用范围

- `worktree` 普通/FUSE 两套 CLI、注册表、作用域发现、暂存、对象保留及存储遍历消费者。
- review/investigate 的普通副本、checkpoint 输入、恢复、清理和 sandbox 环境目录。
- 同卡双语命令文档、网站 `cf` 文档、兼容验证，以及 `0.30.85` 家族发布与线上验收；不发布 `0.31.1`。

### 非目标

- 不搬迁旧目录，不批量转换历史数据，不自动/批量补历史名称，不新增迁移命令；用户显式 add NAME --path EXISTING 的单条命名操作按 ADR-WD-02 执行。
- 不移动主工作区，不把会话记录迁出 `.libra/sessions`，不新增 FUSE move。
- 不顺带完成旧计划 CM-* 的独立缺陷、审批或发布门。

### 成功定义

新建目录遵守 ADR-WD-01；显式 `--path` 保留外部路径能力。工作文件能正确暂存、提交，缺损身份不能回落到主树，清理不能越界。WD-01..08 与 WD-02G 共九卡全部 `done/complete`、最终源码审查 PASS、全量 nextest 和真实 FUSE 门通过、CLI 发布与网站部署证据齐全后才称完成。

### 使用规则

- 执行前刷新源码锚点和 `libra status --short --branch`；只暂存命名文件，不使用 `commit -a`，不覆盖其他 Agent 改动。
- 任何卡状态推进均在同一变更同步 `plan-status.md`；新计划同步 `plan-long.md` 日期索引。协调文件由根执行者单写。
- 所有验收摘要、review 结果、SHA、run ID、时间均直接写本文「实施证据汇总」，不得新建证据文件或目录。
- 采用 v2.14 终止链协议：最终纯文档登记提交的 D 门 SHA、run ID、结论、观察 UTC 时间由下一次 `plan-status.md` 例行同步补记；至下一工作日无同步则专做计划状态例行维护，该维护不重启本计划终止链。

## 事实基线

| 类别 | 当前事实 | 证据 |
|---|---|---|
| 工作树 | 原生 `.libra`，`main`；开始时干净 | 根执行者核验 `85dcc5`；本轮 `libra status --short --branch` 为 `## main...origin/main` |
| 版本 | 源码 0.30.56，已安装 CLI 0.30.53 | 根执行者开工核验；发布前再刷新 |
| 普通入口 | add 必填路径，其他控制命令按路径选择 | `src/command/worktree.rs:98` |
| 注册表 | 当前 v3，保留历史与 epoch；拒绝未知版本 | `src/command/worktree/registry.rs:116`, `:217` |
| FUSE | 独立 CLI；旧 FUSE 注册表 `worktrees` 数组；upper 位于 `worktrees-fuse` | `src/command/worktree-fuse.rs:61`, `:248`, `:630` |
| 路径保护 | 创建拒绝 shared storage 下路径；暂存也排除整个 shared storage | `src/command/worktree-fuse.rs:720`, `src/command/add.rs:2817` |
| 自动副本 | `sessions/<thread>/<run>/workspace`；重复目标先删除 | `src/internal/ai/workspace_isolation.rs:127` |
| Checkpoint | 工作输入与运行记录目前共享 run_dir | `src/internal/ai/checkpoint_input.rs:70`, `:228` |
| 生命周期 | review/investigate stores 负责运行记录，agent clean 直接清理 run_dir | `src/internal/ai/review/store.rs:273`, `src/internal/ai/investigate/store.rs:856`, `src/command/agent/clean.rs:767` |
| 网站 | 实际 `/Volumes/Data/GitMono/libra-backend` 原生 Libra `cf`，路由 libra.tools；旧 `/Volumes/Sea/libra-backend` 不存在 | 根执行者开工核验；`../libra-backend/scripts/README-command-doc-sync.md:1` |
| 测试 | 普通/FUSE 测试都注册于 command_test；独立 review/investigate targets 存在 | `tests/command/worktree_test.rs`, `tests/command/worktree_fuse_test.rs`, `tests/INDEX.md:147` |

### 当前缺口

| ID | 缺口 | 影响 | 证据 | 计划动作 |
|---|---|---|---|---|
| GAP-WD-01 | 缺少 common storage 统一工作目录与安全边界 | 内置目录被误判为元数据或回落到主树 | 上述 add/util 锚点 | WD-01/02 |
| GAP-WD-02 | 名称、路径和两种注册表不统一 | 冲突、别名和旧二进制误操作 | 上述 CLI/registry 锚点 | WD-02/03 |
| GAP-WD-03 | 创建、恢复、清理依据不同的工作目录拼接 | 残留、覆盖、错误删除 | workspace_isolation/stores 锚点 | WD-04..07 |
| GAP-WD-04 | 网站/CLI 示例仍是旧必填路径契约 | 新版操作不可发现 | `docs/commands/worktree.md` | 各行为卡同卡同步；WD-08 验收 |

## 与其它计划的关系

| 计划/文档 | 关系 | 本计划处理 |
|---|---|---|
| `plan-long.md`, `plan-status.md` | 日期索引、状态单一视图 | 根执行者登记并逐卡同步 |
| `plan-20260927.md` CM-04/05/13 | worktree 模块结构与历史未完成门重叠 | 以当前源码为准；仅协调写集，不承接其验收或把旧状态当授权阻断 |
| `issues/476.md`, `plan-20260714.md` | 工作区身份、registry 和旧兼容契约 | 保持身份/epoch/GC 安全不变量；不宣称旧卡完成 |
| `plan-template.md`, `AGENTS.md` | 工程和交付约束 | 应用 GC-01..13、ER-01..14、G-01..11；本文更具体契约优先 |

## 评审结论与修订记录

| 维度 | 结论 | 修订动作 |
|---|---|---|
| 合理性/可行性 | 新默认目录需要行为边界同步，不能仅改路径 | 九张家族卡：原八卡加兼容能力门 WD-02G，依赖验收串行 |
| 粒度/顺序 | 每卡单一轴，统一 helper 先于消费端 | I 写集按依赖顺序串行；实现前复核计数 |
| 完整性/接口 | CLI 行为有兼容性变化；无需旧目录搬迁 | NAME/--path、旧记录只读兼容；用户最终指定基于 0.30.77 做 patch + 1，升级说明须显著提示 |
| 安全/正确性 | 内嵌目录使发现、GC、sandbox、清理都变敏感 | 无身份不向上发现，先持久化意图，限定归属清理 |
| 数据流/可靠性 | 创建/注册/清理之间存在崩溃窗口 | 单锁顺序、创建状态、失败保留记录、前滚恢复 |
| 性能/维护 | 不应新增全树扫描 | 共享布局 helper 和按运行记录查找 |

### 修订历史

| 时间 | 触发 | 变更内容 | 原卡 → 新卡 | 受影响的引用 |
|---|---|---|---|---|
| 2026-10-19 07:51:24 UTC | 用户授权执行、模板/源码复核 | 将八阶段会话方案落为正式卡；无迁移；纠正 full-suite 证据只认 nextest；准确区分会话设计 PASS 与正式文件待审 | WD-01..08 → 保留 | 全文、REL-WD01、状态索引 |
| 2026-10-19 08:02:56 UTC | 根执行者对照已批准会话契约 | 恢复 state/manifest schema 2 optional descriptor、既有 worktrees.lock 复用、umount 实际路径不变；登记 Windows/repair 提示消费者；双执行器均运行但 release 只认 nextest；允许文件独占准备实现并行，验收依赖串行 | WD-01..08 → 保留 | ADR、WD-02/04/05/06、并发与全量门 |
| 2026-10-19 08:09:03 UTC | Claude formal R2 FAIL | 取消中间卡提交与提前 backend 写入；固定根执行者最终提交、HEAD/origin 刷新和手工协调；补缓存/旧 binary agent 路径/opaque 相对位置/tombstone/porcelain/网站命令/Windows D 门等具体契约；R1 无 verdict、R2 FAIL 原样留史 | WD-01..08 → 保留 | ADR、REL-WD01、WD-02/04..08、DEP-WD-05、测试矩阵、完成门 |
| 2026-10-19 08:11:57 UTC | 真实旧二进制反例与 G-04 规模限制 | 旧 agent clean 删除 schema 2 运行记录、遗留新工作副本；加入只写 Repository capability 的 2026101902 receipt，forward-only 无 down；从 WD-02 拆出 WD-02G，总数九卡，用户排除的目录/数据迁移仍不做 | WD-02 → WD-02G + WD-02 | DAG、REL-WD01、GC-13、写集、审计、测试、追溯、里程碑、风险、完成门 |
| 2026-10-19 08:16:02 UTC | 旧无名 detached 的可操作性核对 | 明确 add NAME --path EXISTING 可在完整身份验证后为单条旧登记显式赋名/重挂；不改变禁止自动/批量补名与旧目录搬迁的范围 | WD-02 → 保留 | ADR-WD-02/03、非目标、兼容收口 |
| 2026-10-19 11:28:23 UTC | 实施中的真实仓库状态与 R4 复核 | 固定已写入真实 Libra DB 的 2026101902 receipt；记录 backend 尚未升级、旧 CLI 拒绝和当时网站六页准备性写入；补全 hooks 文档写集与 Windows 门触发条件 | WD-01..08、WD-02G → 保留 | GC-WD-06、REL-WD01、WD-02/08、实施证据及 review log |
| 2026-10-19 11:39:23 UTC | Claude R5 FAIL 的有效问题 | WD-02 生产文件归位、迁移 SQL hash 冻结及撞号双状态前滚、升级单向门双语/网站说明、Windows 用例与实际门命令补齐 | WD-01..08、WD-02G → 保留 | GC-WD-02/02a、WD-02/04/08、审查记录与文档范围 |
| 2026-10-19 14:30 UTC | 远端并发推进 | `origin/main` 比本地起点 `85dcc50` 前进 17 个提交到 `2e917cd`，与本计划代码和版本文件重叠；backend `cf` 前进 6 个提交到 `b77b548`，agent 文档重叠。先以精确路径的本地签名提交保存本计划快照，再 rebase/合并两库、重跑最终门；此快照不推送、不标记完成 | REL-WD01 远端合流方案前滚 | REL-WD01、最终测试与交付门 |
| 2026-10-20 04:13 UTC 前 | 发行资产名缺陷与版本前滚 | `v0.31.0` 已打签名 tag，但只上传三平台资产且 run 被取消，未进入 stable；公开文件名触发旧 status helper basename 门，重命名同字节可通过。新增 FIX-WD-03，最终交付版本改为 `0.31.1`，不复用不可变 `v0.31.0` 前缀；所有旧版失败/取消证据保留 | WD-08 C/D 发布门重新从最终源码 SHA 验证 | 发布分组、完成门、FIX-WD-03 |
| 2026-10-20 10:24 UTC 前 | 用户改为发布既有 `0.30.66` 并禁止 `0.31.1` | `v0.30.66` 已补建 GitHub Release，但其旧源码不含本计划；冻结所有新 Worktree tag/release/stable 与网站新布局部署，待用户另选版本。backend `44e9914` 已将生产七页恢复为 `0.30.66` 语义，覆盖早前 `580006b` 的未来版本文案；FIX-WD-03 只继续代码验证，不执行发行 | WD-01..08、WD-02G 保持 0/9 pending | 顶部冻结、ADR-WD-05、GC-WD-06、DEP-WD-03、REL-WD01、WD-08、FIX-WD-03、计划索引 |
| 2026-10-21 02:30 UTC | 用户要求最新上游 patch + 1 发布 | `origin/main` 已快进至 `e8d530e`，版本面与最新 tag 均为 `0.30.77`；`v0.30.78` 不存在。版本目标改为 `0.30.78`，解除 Worktree 发行与网站上线冻结，不发布 `0.31.1`；修正能力门实际始于 `0.30.67` 的文档误述 | WD-08 进入最终聚合验证和发布窗口，九卡尚未正式完成 | 顶部发布决议、ADR-WD-05、GC-WD-06、DEP-WD-03、REL-WD01、WD-08、升级/兼容文档、计划索引 |
| 2026-10-21 07:15 UTC | 最终树测试名和证据对齐 | 逐项核对九卡 Verification 与追溯表：早期草案所列 22 个 `wd_*` 占位名没有成为最终测试函数，已用实际源码和 nextest 记录中的具名用例替换；复核后卡内/追溯表的 `wd_*` 名称全部存在。只修计划中的选择器和证据映射，不放宽 AC，也不把 0 tests 记为通过 | WD-01..08、WD-02G 保持正式 pending，待 C/D 发布门完成 | 各卡 Verification、追溯表、实施证据汇总 |
| 2026-10-21 07:42 UTC | 上游 #501 合流与模板自复制前滚 | 上游 `4234095` 在 `ba7425d` 后合入 `init --template` 修复，旧 final-SHA CI 7/8 被并发 push 自动取消；Claude Code 对新 delta 字面 `VERDICT: FAIL`，指出模板路径 canonical 而目标仍为别名时会绕过自复制保护。隔离 macOS 实验复现 fresh init `ENAMETOOLONG`，reinit 还留下 `.libra/repo` 残留。增列 FIX-WD-07：按真实目标/存储路径在写前检查、创建后复制前复核，模板在 `.libra` 内直接拒绝；本地定向 7/7 通过，双语 init、网站 init 页与 changelog 同步，完整复审/全量/最终 SHA CI 待跑 | 旧 `ba7425d` 的取消 CI 和中断的 `4234095` `cargo test --all` exit130 均不计 PASS；用户旧目录不迁移范围不变 | FIX-WD-07、WD-08 写集与实施证据 |
| 2026-10-21 08:25 UTC | FIX-WD-07 数据安全复审与上游 checkout 合流 | Claude 逐轮复审发现并确认修复模板别名自复制、裸仓库布局失败误删和 Git 转换 `missing/..`/已有 bare 目录回滚误删；r4 字面 `VERDICT: PASS` 仍指出 journal、布局失败残留及并发归属边界，已前滚 journal 冲突检查、受限布局名称登记及布局失败清理，转换组 9/9。此后上游仅 checkout 六文件推进至 `fc5eafc`；七个本地脏文件逐一 SHA-256 备份，洁净 fast-forward 后逐字节还原。最终 r5 复审、双全量、macFUSE、最终 SHA CI 和发布仍待 | 九卡正式 pending；旧 run 不计最终通过 | FIX-WD-07、WD-08 发行门及本地合流证据 |
| 2026-10-21 08:38 UTC | FIX-WD-07 大小写别名与 shared 权限前滚 | Claude r5 字面 `VERDICT: FAIL/P1`：默认 APFS/NTFS 上模板 `README` 会别名指向用户已有 `readme`，字面集合误判后回滚删除；还指出已有 bare 目标 `--shared` 曾递归修改用户文件权限。以真实 `symlink_metadata` 在写前识别候选布局名、只登记真正缺失的名称，并限制 shared 扫描到新建仓库条目；新增大小写文件/目录和私有权限回归，转换组 11/11。将 Windows exact 断言扩至六个，双语/网站 init 文档同步；一轮原本启动的全量在 r5 FAIL 后主动中断 exit130，不能计 PASS。r6 复审和最终全量仍待 | 九卡正式 pending；任何已中断/旧 SHA 测试不计最终门 | FIX-WD-07、Windows CI、文档与发布证据 |
| 2026-10-21 09:07 UTC | FIX-WD-07 r6 PASS、实机 macFUSE 与网站 `--force` 勘误 | Claude r6 对最终 init 生产差异字面 `VERDICT: PASS`、P0/P1/P2=0，后续只加强两条测试断言和 changelog；转换 11/11、init 45/45、Git 导入 20/20、fmt/严格 Clippy/actionlint/兼容矩阵通过。macFUSE 使用 `LIBRA_RUN_FUSE_TESTS=1 --include-ignored` 真实挂载 16/16、无残留；先前未含 `--include-ignored` 的 15/16（1 ignored）不计实挂载门。网站 Claude 第一轮字面 FAIL：Worktree 文档误写标准构建的 `--force` 不支持；双语仓库文档、兼容表和网站页已按现行 CLI 勘误，八页限定同步 drift=0、脚本 8/8、Vitest 11/11、typecheck/build exit0；网站跟进复审字面 `VERDICT: PASS`，P0/P1=0。两轮 cargo 全量在发现未修复审查问题时主动中断 exit130，均不计 PASS；最终全量重跑中。 | 九卡正式 pending；最终树双全量、CI/发行/网站上线待完成 | FIX-WD-07、WD-08 文档写集与验收门 |
| 2026-10-21 09:24 UTC | FIX-WD-07b CLI 后置路径修复 | `7806c9135c98c770e4c77152e44fedc88fa6e900` 已签名+DCO 推送，远端同 SHA，base run `39574920827` 启动；随后独立 disposable 检查复现 Unix `init regular-file/..` exit128 却已创建父目录 `.libra`，根因初始化内部规范化目标而 `src/cli.rs` 后置开库仍用原始拼写。主动中断进行中的 cargo 全量、请求取消该旧 SHA CI，均不计通过。将实际存储路径作为初始化成功结果返回 CLI，并在 Unix 路径遍历拒绝非目录中间组件；新增两条 E2E 回归 1/1 各自通过，init 组 47/47、转换组 11/11。网站 init 页重新同步，八页限定检查 drift=0、脚本 8/8；一次并发编译时 Vitest 的单例触及 5 秒超时，须在负载下降后重跑，不能计本轮通过。 | 旧 SHA 门已作废；最终静态、双全量、复审、CI、发行与网站门待验 | FIX-WD-07b、WD-08 发布写集 |
| 2026-10-21 09:39 UTC | 上游 `v0.30.79` 发布与并发合流 | GitHub Release 列表已将 `v0.30.79` 标为 Latest，tag 指向 `9ba1369`，`main` 又合流 #523 rebase 至 `de72da7`；用户既定“最新上游 patch + 1”使目标从未发布的 `0.30.78` 前滚为 `0.30.80`。`v0.30.79` release run `39576472385` macOS build 在 `index.crates.io` DNS 解析失败，其他平台取消，CDN stable 清单仍为 `0.30.77`/revision 176；不得借 GitHub Latest 冒充完整 stable。七个本地脏文件逐字节备份，洁净快进至 `de72da7`，`libra diff 7806c91 de72da7 --` 七文件空，按 SHA-256 7/7 恢复；版本面与当前文档目标改 `0.30.80`，后续门在新树重跑 | 九卡仍 pending；v0.30.80 的源码、CI、资产/stable 与网站门均待验 | 顶部发布决议、版本面、WD-08、文档与收口门 |
| 2026-10-21 09:50 UTC | `0.30.80` 合流树静态与范围门 | 版本面 Cargo.toml/Cargo.lock/双安装脚本统一至 `0.30.80`，无公开文档遗留 `0.30.78`。最终当前树 init 47/47、Git 转换 11/11、版本面 2/2、Worktree 文档 2/2，fmt/严格全目标全特性 Clippy/actionlint/diff check exit0；网站八页限定 drift=0、同步器 8/8、Vitest 11/11、typecheck/build exit0，全站只读另有 24 页范围外漂移。Claude Code 源码最新审查字面 `VERDICT: PASS`、P0/P1/P2=0；仅三条 P3 文案精度/未发布旧标签注记具名接受，网站最终复审和双全量/实机/CI 仍待 | 九卡仍 pending；此处不把 focused 或源码 review 代替发布门 | FIX-WD-07b、WD-08 本地 A/B 与文档验收 |
| 2026-10-21 09:54 UTC | 网站最终复审 FAIL 与文档/提示前滚 | `0.30.80` 源码签名+DCO `30b11fb` 推送远端且 SHA 相同；其 base CI 尚在运行，不能当最终门。Claude Code 网站八页复审字面 `VERDICT: FAIL`：P1 英文 agent 页混入中文、macFUSE/init 版本文案的“0.30.80 或更新”错误涵盖旧 `v0.31.0` tag；P2 FUSE 构建 `--resolve-identity --yes` 不可用、`--force` 发行版本未说明、v0.30.67 macOS/Windows 缺资产、运行中转换其他命令拒绝码未说明；还指出 sequencer worktree repair 旧位置参数提示。按实际 CLI/历史资产前滚双语源文档、网站八页同步页、CHANGELOG 与 `src/internal/sequencer/mod.rs`；网站/源码复审及受影响门须重跑 | 首轮网站 FAIL 原样保留，九卡正式 pending；不发布不部署 | WD-08 网站复审补充写集、最终复审/全量/CI |
| 2026-10-21 10:02 UTC | 网站审查跟进 PASS | 双语 Worktree/init/upgrade 源文档消除跨版本误导、区分标准与 FUSE 标志、补充平台二进制可用性和中断转换拒绝码；英文 agent 源页改为纯英文；sequencer 修复提示改 `--path <worktree-path>`。网站八页重新同步并限定检查 drift=0、同步器 8/8、Vitest 11/11、typecheck/build exit0。Claude Code 对八页与这些修复跟进复审字面 `VERDICT: PASS`、P0/P1/P2=0；仅不阻断的旧英文页引用中文开发标题 P3 保留。发布顺序仍须源码最终提交/CI/tag/stable 先于网站 cf/部署 | 网站文档可进入最终交付准备，但未上线；九卡仍 pending | WD-08 网站必需门与 Claude 复审 |
| 2026-10-21 10:06 UTC | `--force` 历史版本勘误 | 源码跟进复审字面 `VERDICT: FAIL`，指出上一轮网站审查对标准构建 `add/remove --force` 首发版本的判断错误；用完整引用的 `v0.30.69:src/command/worktree.rs`、`v0.30.77:src/command/worktree.rs` 核实两处标志均存在，v0.30.68 和较早打标的 v0.31.0 不存在。更正双语 Worktree 命令文档与 0.30.80 CHANGELOG 为“自 v0.30.69 已有”，网站同步与两侧最终复审须重新通过；原 FAIL 不计 PASS | 九卡仍 pending；最终源码、全量、CI、发布、网站门待验 | WD-08 文档版本精度与复审 |
| 2026-10-21 10:16 UTC | Windows 真实发行文件名回归失败与前滚 | base run `39580327427`（`30b11fb`）的四个 Windows exact fixture、`windows_fs` 17/17、capture 2/2、workspace 10/10 与仓库锁均通过，后续 `release_binary_name_test` 在真实 `init --vault=false` 报 SQLite `unknown query parameter /C:/...`，Windows job 明确 failure；此前的定向 macOS/静态通过不能掩盖此门。修复共享 `src/internal/db.rs` 在斜杠规范化后去除 `//?/`，并覆盖普通磁盘、前向斜杠和 UNC 路径；最终 SHA Windows E2E 与双全量须复跑，旧失败保留。源码后续复审亦指出旧 `v0.31.0` 可下载而版本号更高，`libra upgrade` 不会选 v0.30.80；双语源文档、网站同步页和 CHANGELOG 明确改用签名 stable 默认安装脚本 | 九卡仍 pending，`30b11fb` 不计最终 D 门；新树须再次 review/CI | FIX-WD-07c、WD-08 Windows/双全量/升级文档门 |
| 2026-10-21 10:26 UTC | macOS 签名安装说明前滚 | 源码 Claude final3 字面 `VERDICT: FAIL`，虽确认 Windows SQLite 修复正确，仍指出 stock macOS `/usr/bin/openssl` 的 Ed25519 支持不足会使 `curl … | sh` 默认签名安装失败；`libra upgrade` 对较大旧版本号也不会选 v0.30.80。双语 upgrade 命令页加入 Homebrew OpenSSL 3 置顶 `PATH` 的实际命令、验证 `stable manifest verified`、拒绝未验证 fallback 与版本覆盖值说明，Worktree/init 双语页链接该指南，CHANGELOG 指向升级页；本机 Homebrew OpenSSL 3.5.9 的 Ed25519 keygen exit0。网站八页重新同步，双方复审仍待 | 原 FAIL 保留；九卡正式 pending，不以 FUSE 16/16 或网站已过一轮 PASS 替代新文档审查 | WD-08 升级文档与签名安装门 |
| 2026-10-21 10:50 UTC | 最终树全量发现临时端口竞态 | `92910df` 已签名+DCO 推送，远端同 SHA，base run `39586631004` 运行中；其本地 `cargo test --all` 在 lib 4581 passed / 1 failed / 9 ignored 后 exit101，唯一失败 `mega2_diag::run_classifies_connect` 的端口先释放后使用夹具在并行下返回 `transport=request`。初始轮失败原样保留，新增 FIX-WD-08a 有界重选端口并要求最终 connect 类别；在新提交/最终 SHA 上重跑 focused、全量、CI。`92910df` 的本地 diff 70/70、旧 0.30.53 九类拒绝/69 稳定文件哈希不变、Claude 合流复审 PASS、Mac 实挂载先前 16/16 均不代替红色 full suite | 九卡仍 pending，不能发布旧 SHA | FIX-WD-08a、WD-08 全量/CI 门 |
| 2026-10-21 10:54 UTC | 最终树 Windows CI 暴露测试编译门 | `39586631004` 原生 Windows 工作流四项 scoped fixture、`windows_fs`、capture、workspace、lock、新 SQLite 路径单元及重命名发行二进制均通过；在执行六项 init exact 前，`command_test` 被已有 Unix 专用测试的 `std::os::unix` / 非 Windows 使用的 import 等 32 项编译错误阻断。新增 FIX-WD-08b 仅对 Unix 专用测试与导入加平台条件，保留 Windows init 六项过滤器与严格 warnings；新 SHA 上重跑原生 CI，不能把前半段成功当作 Windows 全门 | 九卡仍 pending，Windows D 门红色 | FIX-WD-08b、WD-08 原生 Windows CI 门 |
| 2026-10-21 11:09 UTC | Windows 编译前滚复审与上游新正式版 | Claude 对 FIX-WD-08b 首轮字面 `VERDICT: FAIL/P1`，指出 Windows 接下来的 `dead_code` 门还会拦截四个只由 Unix 测试调用的辅助函数；补入四处 `#[cfg(unix)]`、相应导入及三个 Unix 锁夹具调用测试的门，二轮逐项复审字面 `VERDICT: PASS`、P0/P1/P2=0，严格 Clippy 与 fmt 已通过。与此同时上游 `f477c42` 推送并发布 `v0.30.81`，使旧 `92910df` run 自动取消，旧树本地全量也主动中断 exit130；按用户“上游最新 patch + 1”，目标从未发布的 `v0.30.80` 前滚至 `v0.30.82`，须合流、更新版本面和文档、重跑最终 SHA 全部门 | 九卡仍 pending，旧 `0.30.80` 门不可继承 | FIX-WD-08b、WD-08 版本前滚与最终门 |
| 2026-10-21 11:16 UTC | 上游合流树编译修复与端口夹具验证 | `cf3dff2` 签名+DCO 后与 `f477c42` 合流为 `c67bd0a`；版本面/双语源码文档目标前滚 `0.30.82`。首次新树 macOS lib 编译在上游 MCP `exec.rs` 普通 `extern "C"` 因 Rust 2024 必须 `unsafe extern` 而失败，补最小语法修复。FIX-WD-08a 的保留未监听 socket 方案在 macOS 实测 6/8、两项 connect 用例被分类 timeout，因此撤回新增 socket2 测试依赖；最终用 1..8 低位 loopback 端口避开并行测试 `:0` 临时端口分配，并仅接受真实 connect 错误，focused 8/8。上述新代码/版本合流仍须复审、双全量、最终 SHA 原生 CI | 九卡仍 pending，旧 .80/失败的首次 .82 夹具不计通过 | FIX-WD-08a/08c、WD-08 最终树门 |
| 2026-10-21 11:35 UTC | 上游再次发布与合流 | 上游 `24fabae` 创建 `v0.30.82` GitHub Latest，用户规则使未 tag 的本地 `.82` 前滚至 `v0.30.83`；本地 `.82` 全量因最终树变化主动中断 exit130。`f23e022` 签名+DCO 保存 `.82` 准备性修改，`8472d0f` 与 `24fabae` 冲突合流：唯一 `exec.rs` 冲突取上游同样的 `unsafe extern` 加 release peak 修复；最终 `.83` 版本面、双语/网站文档、全量、macFUSE、原生 Windows/CI 均重跑。源码审查曾指 `.81` macOS 无可用资产，已移除对 `.81` Mac 构建的文档肯定语；低端口夹具恢复 5 秒 Windows 连接超时，其他 MCP P2 边界需最终审查具名处理 | 九卡仍 pending，`.82` 本地门不计最终 C/D | WD-08 发布前滚与复审 |
| 2026-10-21 12:00 UTC | `.83` 首轮最终 SHA Windows 编译门前滚 | `a45a1c6` 签名+DCO 推送并对应 base run `39594805304`；四项 active exact、`windows_fs` 17/17、capture 2/2、workspace 10/10、lease、Windows SQLite 单元及重命名发行二进制通过。六项 init exact 开始前，`command_test` 严格 `-D warnings` 仅因 `tests/command/merge_test/message_options.rs` 的 `SIGNOFF` 常量只在 Unix 测试使用而报 `dead_code`。为此加 `#[cfg(unix)]`，最终树需重新 fmt/Clippy/Windows 原生 CI 与双全量；已运行的 `.83` 本地 cargo 全量主动中断 exit130，不将 6000+ 无失败的部分结果记为 PASS | 九卡仍 pending，首轮 `.83` Windows D 门失败 | FIX-WD-08b、WD-08 最终 SHA 门 |
| 2026-10-21 12:17 UTC | `.83` 二轮 Windows 转换回滚缺陷 | `27a72b3` 签名+DCO 推送并对应 base run `39596846832`；Windows 四项 exact、17 项 `windows_fs`、capture/workspace/lease、SQLite 路径单元、重命名发行二进制及前两项 init exact 通过，第三项 `init_bare_conversion_failure_preserves_existing_target_contents` 失败：失败回滚后新 `libra.db` 仍在。`init` 的 SeaORM 直连与共享缓存连接池在 Windows 上可能仍持有 SQLite 文件，故在转换错误回滚前显式且限时关闭两者；关闭失败则保留转换标记、返回具名人工恢复错误，不尝试部分删除。双语 init/网站页、CHANGELOG 同步；本地旧树全量主动中断 exit130，不计 C 门 | 九卡仍 pending，二轮 `.83` Windows D 门失败，新生产修复须全量/原生复验 | FIX-WD-08e、WD-08 最终 SHA 门 |
| 2026-10-21 15:50 UTC | 上游 #525 提交选项前滚复审 | 在 `81eff71` 树上修正 `-S` 的显式强制签名、`-S`/`--no-gpg-sign` 最后标志优先、`-u` 三种实际预览模式、`-o`/位置路径的稳定拒绝及 `--short` 只预览；英文/中文命令页、开发文档、兼容表、CHANGELOG 和网站第九张 commit 页同步。首次 focused 59/61，失败为新测试目录前缀误判与旧 MC7 仍预期 exit129；修正后 `commit_test` 61/61、`compat_config_history_defaults` 13/13、fmt、严格全目标全特性 Clippy 通过。Claude Code 第一轮 18 turns 耗尽不计，第二轮审查字面 `VERDICT: PASS`、P0/P1/P2=0；四项 P3（`-u foo` 错误文案、显式 `-S` 绕过坏配置、旧 porcelain 不读 untracked 配置、内部直接调 run_commit 绕过 partial 拒绝）按已知边界接受。网站限定九页 drift=0、同步器 8/8、Vitest 11/11、typecheck/build exit0；全量、最终 SHA CI、发布和线上验收仍待 | 九卡仍 pending，不以局部门代替发布 | WD-08 并发合流前滚、最终 C/D 门 |

## 已决议设计决策

### ADR-WD-01: 默认容器与显式路径

- **Status:** Accepted（用户授权的会话方案）。
- **Context:** 新建工作目录分散，用户希望统一默认目录。
- **Decision:** 根唯一为解析并验证过的 common storage + `worktree`，不依赖 cwd。普通默认 `linked/<name>`；FUSE 管理根 `fuse/<name>`，其内 `workspace` 为挂载点、`upper` 为数据层；任务为 `tasks/<run-id>/<attempt-id>/workspace` 或 `checkpoint-input`。同一 attempt 的缓存/环境目录固定为兄弟目录 `home/`、`cargo-home/`、`xdg-config/`、`xdg-cache/`、`logs/`，沿用现有 runtime 名称。外部 `--path` 是完整目标目录，相对路径以调用 cwd 解析。
- **Alternatives considered:** 强制禁止所有外部路径会破坏用户需要的显式覆盖；只改路径会破坏 add 和发现边界。
- **Consequences:** 主树不动；sessions 保存运行记录；新 CLI 裸参数是 NAME。
- **Revisit when:** 将来新增后端或需要独立容量卷。

### ADR-WD-02: NAME、选择器与生命周期

- **Status:** Accepted。
- **Decision:** `worktree add NAME [TARGET] [-b BRANCH|--detach] [--path PATH]`；NAME 不隐式建同名分支。NAME 为 `[a-z0-9][a-z0-9._-]{0,63}`，拒绝尾点及 Windows 保留设备名（含带扩展名形式）。普通/FUSE 共用唯一命名空间。PATH 保留 Unicode/空格能力。
- `lock/unlock/remove` 使用互斥 `NAME` 或 `--path PATH`；repair 同样接受选择器，均省略时保留全局 repair；doctor 的 workspace-id 选择器与 FUSE umount 的实际路径选择器保持现有语法。`move NAME --path DEST`；无名历史条目用 `move --from-path SRC --path DEST`。不新增 FUSE move。
- 已注册名称永远解析登记路径；`add NAME` 若匹配 detached 记录，只在身份匹配时重挂，不改 HEAD/index/文件，拒绝 TARGET、-b、--detach；显式 PATH 与登记路径不一致拒绝。活动名称重复创建不覆盖目标，报告已存在。唯一允许给历史无名条目赋名的入口是用户显式 `add NAME --path EXISTING`：在共享锁下确认名称未被占用、路径与旧登记匹配、工作区身份完整后赋名，并按既有状态合法重挂；保留 worktree_id/HEAD/index/文件，重挂时仍拒绝 TARGET/-b/--detach。读取和无关写入绝不自动补名。
- `remove` 默认仅 detached 并保留目录/名称。显式 `--delete-dir` 仍先做 dirty 检查；目录和注册状态清理全部成功后才释放名称；失败/tombstone 占用名称。`repair --confirm` 在验证恢复条件后可完成注册状态修复；需要释放名称的文件终结只能经显式 `remove --delete-dir`。prune 只处理登记，不删除文件，也不释放仍有文件/恢复状态的名称。
- 人读输出显示名称、实际绝对路径、状态；porcelain 仅追加可选 `name` 行，保留现有字段、相对顺序和块分隔，不重排旧字段；JSON 保持现有公开 schema v2，增加可选 name 字段，缺名省略，磁盘 registry 版本独立管理。所有 selector 语法错误保持现有 CLI 退出码约定。
- **Alternatives considered:** 把旧位置参数同时猜成路径/名称会产生歧义；历史条目补名违反用户非迁移要求。
- **Consequences:** docs 明确旧路径调用须换成 NAME/--path；move 保持稳定 worktree_id 和名称。
- **Revisit when:** 用户明确要求改名能力；当前不提供 rename-name。

### ADR-WD-03: 注册表和旧二进制安全

- **Status:** Accepted。
- **Decision:** 普通新写入 registry v4，读取 v1/v2/v3 沿用既有兼容但不自动回填名称（显式单条命名遵守 ADR-WD-02）；特别区分 v3→v4，原样保留 history/epoch/worktree_id。FUSE 新格式采用版本化 `entries`，旧 `worktrees` 只读兼容；未知/混合形状 fail-closed。新 FUSE 创建先建立普通 v4 版本门，防止旧 maintenance/GC 忽略该布局。普通构建仍解析 FUSE 名称占用，不依赖启用 mount feature。
- 公共名称/布局锁复用现有普通 registry 的 `worktrees.lock`，只在命令最外层获取一次；顺序固定为 `worktrees.lock`→进程内 FUSE state mutex，不再添加第二把同用途公共锁；被调用 helper 不重复获取外层锁。锁覆盖校验、名称预留、创建/发布及补偿边界。
- 旧二进制拒绝新格式的行为必须通过真实 0.30.53 二进制在 disposable repo 的对照验证，矩阵包含 worktree add/remove/move/prune/repair、maintenance GC、agent clean、review 运行、investigate continue。不能预设 v4 registry 足以覆盖 agent 路径；当前真实实验已确认旧 agent clean 绕过 JSON 版本门：它删除 schema 2 运行元数据却留下新工作目录。因此 WD-02G 在任何新布局写入前应用 Repository-only 2026101902 兼容 receipt，令旧二进制在 DB connect 的 future-schema 检查拒绝；仍须复验全部旧消费者，不能因加了 receipt 就预记 PASS。每个实验均比较 registry、HEAD、index、对象、工作文件与运行记录前后状态。
- **Alternatives considered:** 只给条目加字段可被旧 reader 忽略，导致名字丢失或对象误删。
- **Consequences:** 不提供降级写回；新布局投入使用后 forward-only。
- **Revisit when:** 加 receipt 后的旧二进制矩阵仍发现绕过 DB connect 的危险消费者。

### ADR-WD-04: 工作文件边界、FUSE 防递归与任务归属

- **Status:** Accepted。
- **Decision:** 统一 helper 生成并验证布局；文件是否可暂存以当前 worktree root 与本地 gitdir 判断，禁止把整个 common storage 当作子树工作文件的排除前缀。主树仍排除整个 `.libra`。任何位于内部容器的 cwd 没有可验证工作区身份，立即拒绝发现，不能向上绑定主树。
- FUSE mount 前，针对每个已验证 lower 计算 common worktree 容器相对 lower 的路径，在 upper 的该相对位置建立并验证 opaque 子容器；主树 lower 的常见位置是 `.libra/worktree`。容器不在某个 lower 内时该 lower 的此项标 N/A；包含关系无法安全证明则拒绝。必须屏蔽整个容器，不能仅隐藏当前名称子目录。挂载后再回读验证，失败卸载/保留可恢复记录，拒绝成功。外部覆盖路径不得位于 lower/upper 或其他管理目录；只有由布局 helper 生成并已做 opaque 防护的内部默认挂载点例外。
- 普通/FUSE worktree 使用可验证本地身份和作用域。任务副本/checkpoint 不是主树 VCS scope；在其中执行隐式 Libra 仓库发现必须明确拒绝，这是有意的用户行为变化，在 review/investigate EN/zh/网站文档说明，并用真实子进程命令断言拒绝且主树 HEAD/index 不变；仅命令显式提供经验证的独立作用域时例外。
- 每次执行/continue 分配新 attempt-id。任务描述包含 repo/run/attempt/backend/kind、规范路径与 provisioning/active/cleanup-pending 状态；Review/Investigate 的 state 与 manifest 升级 schema 2 并增加可选 descriptor，仍写在现有 sessions 运行记录下。schema 1 保持可读，不批量补写 descriptor；未知版本的写入和 clean fail-closed。先写创建意图，再排他创建目录，最后发布 active。取消、恢复、clean 消费同一描述；无描述或无法证明归属时 doctor 报告，不猜路径删除。
- descriptor 的归属范围包含整个 attempt 的工作目录与上述固定缓存/日志目录；只允许该清单内归属资源，不能因允许 attempt 清理而递归信任任意新增兄弟路径。工作目录及其缓存清理成功后才能清理运行元数据；失败保留意图供重试。旧记录不搬迁、不补新描述；旧清理使用既有验证边界，无法证明则保留并报告。
- **Alternatives considered:** 清理依赖 `workspace_root` 字符串或推断旧路径，不能证明归属；先清 sessions 会丢恢复信息。
- **Consequences:** 所有删除保持 no-follow、有预算和锁；重复创建不能先 remove_dir_all 任意已有目录。
- **Revisit when:** 新后端不能表达同样的身份与删除边界。

### ADR-WD-05: Sandbox、Checkpoint 与发布家族

- **Status:** Accepted。
- **Decision:** 向 sandbox 显式传入工作根和本 attempt 的固定缓存目录清单，禁止从 sessions_root 推断。HOME→home、CARGO_HOME→cargo-home、XDG_CONFIG_HOME→xdg-config、XDG_CACHE_HOME→xdg-cache、LIBRA_LOG_FILE→logs/libra.log；不改变现有 Unix RUSTUP_HOME 保留语义。受限 policy 仅放行工作根和这些缓存目录，不能通过父目录或 symlink 写 common storage/兄弟工作区。现有显式 danger-full-access 不伪称隔离有效，也不得因布局变化静默扩大默认权限。
- Checkpoint 生产 API 分离运行元数据句柄和物化目录句柄；继续在清理/写入前验证 catalog/object、路径、budget/deadline/cancel；只读输入不降低权限防护。运行记录保持 sessions，输入使用 tasks/attempt。
- WD-01..07 与 WD-02G 是不可独立交付的家族子卡，WD-08 唯一发布。原拟次版本 `0.31.0`，失败资产后候选曾前滚至 `0.31.1`；用户禁止发布后者，并在 2026-10-21 指定上游最新已发布版本 patch + 1。上游已发布 `v0.30.84`，当前目标据此为 `0.30.85`。公开 CLI 兼容变化须在该补丁版的 release notes 和升级文档显著说明；版本面、Cargo.lock 与安装脚本在最终树核对。
- **Consequences:** Repository receipt 2026101902 在支持新布局的二进制首次打开仓库时即应用，即使用户尚未创建新 worktree。该能力已进入 `v0.30.67` 源码；`0.30.66` 及更早版本会在 DB connect 拒绝已升级仓库，`0.30.67` 及更新版本支持该 receipt。新格式只能前滚补丁，旧 CLI 不作回滚方案。
- **Revisit when:** 上游版本再次推进或 `v0.30.85` 被占用，先刷新源码版本与 tag，再按用户“上游最新 patch + 1”重新计算目标。

## 全局工程约束

完整继承模板 GC-01..13、ER-01..14、G-01..11；特别补充：

- **GC-WD-01:** 所有路径生成/名称验证/容器边界只有一个 helper；不得以字符串 starts_with 替代规范路径和 no-follow 检查。
- **GC-WD-02:** WD-02G 仅为 Repository 增加 2026101902 兼容 receipt，并向既有 worktree_registry_capability 写 version 4；不搬旧目录、不变更旧数据或补名称。真实仓库已应用的 SQL 文件内容冻结为 SHA-256 `c1dbf1ae702e391db2adef39245fcd66b745a07bc64aa1511e7984c911333c43`；改变语义只能新增更高版本迁移。migration/bootstrap/schema manifest 的 allowlist 仅 Repository，ledger 为 schema_versions；GlobalConfig/SystemConfig 继续各自配置 ledger，不运行该 writer，Derived 拒绝通用迁移。tests 显式隔离 LIBRA_CONFIG_GLOBAL_DB/LIBRA_CONFIG_SYSTEM_DB/HOME，断言配置 receipt/表内容不变，不得落真实配置库。
- **GC-WD-02a:** 每次刷新 HEAD/origin 都核对迁移目录、README 与上述 SQL hash。若上游不同迁移占用 2026101902，立即暂停合并/提交/发布，保留真实 DB 的本地 receipt；先协调上游作者在尚未发布时改号。若上游迁移已发布，则按两种实际 DB 状态设计更高编号的条件修复迁移，使「本地已应用本计划 1902」补上外来 DDL、「上游已应用外来 1902」补上本计划 capability，并在两种克隆上逐项验证 schema、receipt 和对象数据。只有该前滚方案完成源码/测试/文档审查并取得 PASS 才恢复发布；不能仅改本地 ledger 名称或假定版本号相同意味着 DDL 已执行。
- **GC-WD-03:** 不新增生产 unwrap/expect/panic；错误说明资源、失败原因与修复方向。复用既有 LBR 错误类别；如必须新增稳定码，先扩写集并触发 T-1，Display-pin 与 docs/error-codes 同卡。
- **GC-WD-04:** 每卡 A/B focused 和 Claude Code 源码审查须真实完成；家族子卡只到 locally-accepted，继承 WD-08 C/D 后才能 done/complete。会话设计 review 不替代正式计划/实现 review。
- **GC-WD-05:** 按用户 AGENTS 同时执行 `source .env.test && cargo test --all`；模板 release/全量通过证据只认 `source .env.test && source .env.live-test && cargo nextest run --all --no-fail-fast --retries 2`。两者都必须执行并如实记录，前者不替代 nextest 发布门；取消/失败/未运行不算 PASS。任何失败修复后重跑受影响门和全量至绿。
- **GC-WD-06:** 网站直接使用现有 native Libra `cf`，保留无关改动，不建 branch/PR/worktree。未来布局文档曾部署，已由 backend `44e9914` 恢复为当时 `0.30.66` 语义；该恢复不计 WD-08 新功能验收。当前 `0.30.85` 发布窗口内按最终 CLI/仓库文档重新同步原七页、FIX-WD-05 的 init、上游 commit 和 branch 共十页，逐文件验收、提交推送并核验线上新契约；此前不能 complete。

## 执行检查必备需求

1. 每卡 ER-01..03 安全检查、锚点和粒度复核；扩写集先更新本文。
2. A 组按写集运行单元/集成/feature gates；B 组按任务类型；ER-05 Codex 和 Claude Code reviewer 必须实际读最终差异，最终 `VERDICT: PASS`。
3. 每个会推送的树运行 fmt/clippy。子卡相关 T-1/T-3 风险由家族发布点包含最终变更的全量承担；WD-08 T-2 恒定全量。
4. `0.30.85` 的 C/D 使用模板完整发布协议：版本面 parity、工具链 lockfile、bump 树 nextest、release build/install/版本检查、签名+DCO、精确暂存、native push、tag/release.yml、远端产物与 stable 清单证据。任何外部门失败保留 remote-pending。
5. 新布局网站必需门在最终 CLI/仓库文档就绪后，于现有 native cf 的 backend cwd 执行：先只读全库检查并记录基线漂移，再以 `--only worktree agent review investigate sandbox hooks upgrade init commit branch` 分别运行同步器 `--write` 与检查；`python3 scripts/test_sync_libra_command_docs.py`；`pnpm exec vitest run tests/unit/docs-manifest.test.ts tests/unit/docs-search.test.ts`；`pnpm typecheck`；`pnpm build`。原七张 Worktree 命令页加 FIX-WD-05 的 init、上游 Issue #525 的 commit、上游 #526 的 branch 页共十张；网站只存 EN 命令页，zh-CN 在渲染时镜像，不新增不存在的中文源文件。
6. 最终提交前重新读取本地 HEAD、origin/main 和 backend cf/origin/cf，核对远端没有部分 WD 变更，重新检查精确写集和当前 dirty diff。若上游发生变化，先保存当前工作字节并人工合并到最新源码，重新审查/运行受影响门；禁止自动 rebase/reset 或推送无关变更。该核验失败则保持 dirty 未提交，不尝试绕过。
7. 原生 Windows D 门是 `.github/workflows/base.yml` 的 `compat-scoped-input-windows`：最终 main SHA 对应的四个 active exact scoped-input fixture 与 windows_fs 单元组均需通过，cross-check/Windows artifact 编译均不能替代。旧 `cc8f437` 的门已通过，但合流后的最终 SHA 必须重新通过才可触发 tag/artifact release；现有并发 relocation DEFER 仍不算 PASS。
8. COMPATIBILITY 仅增加本行为所需记录，不无关重排现有 ledger 锚点；实际触及的守卫测试通过才作兼容证据，不把未测试文字当限制。
9. 测试文件/serial 新增必须在同卡维护 `tests/INDEX.md`、`tests/SERIAL_REGISTRY.tsv`、`tests/SRC_SERIAL_REGISTRY.tsv`、`.config/nextest.toml` 的实际影响；只有新增 target 才改 Cargo.toml 注册。本文计划使用现有 targets，不凭空新增 target。

## 实施顺序

`WD-01 -> WD-02G -> WD-02 -> WD-03 -> WD-04 -> WD-05 -> WD-06 -> WD-07 -> WD-08`。外部依赖在下表登记；无卡外隐藏先决条件。

### 依赖登记表

| ID | direction | 类型 | 对象 | Owner | 产物与可用性判据 | 证据 | 超时与失败策略 |
|---|---|---|---|---|---|---|---|
| DEP-WD-CODEX | incoming | 审查 | 当前 Codex 执行者 | 根执行者 | 最终源码给出具名 `VERDICT: PASS`、P0/P1 清零及真实验证清单；不可用则阻断，不自填 | 实现仍在进行 | 在最终 Claude 审查之前修复发现并重跑受影响门 |
| DEP-WD-01 | incoming | 审查 | Claude Code 本地 reviewer | 当前根执行者协调 | 正式本文及最终源码分别取得字面 VERDICT: PASS | 正式计划 R6 PASS；`f0ea585` 的 #525 前滚源码复审 PASS，`.85` 上游 #526/FIX-WD-08g 的最终差异须跟进复审 | review 失败先修；工具不可用保留明确阻断，不伪称 PASS |
| DEP-WD-02 | incoming | 执行环境 | 受支持 Unix FUSE 环境 | 根执行者 | feature 编译、真实 mount/opaque/unmount 可执行 | macFUSE 5.4.0 与 OrbStack Linux `/dev/fuse` 完整组各 16/16；`0.30.85` 最终树待复验 | 缺驱动或最终树未运行不得发布 |
| DEP-WD-03 | incoming | 发布服务与用户版本决策 | GitHub release、Cloudflare artifact/site | 用户已选“上游最新 patch + 1”；根执行者单一发布 | 认证可用且对应 SHA release 与部署可观察 | 2026-10-21 17:18 UTC GitHub Latest `v0.30.84` tag 指向 `99a92c1`，目标 `v0.30.85` 发布前须再次查占用；资产/stable 另验 | 最终 SHA、资产/stable 与网站内容未实证前保持 remote-pending |
| DEP-WD-05 | incoming | 原生平台验收 | final main SHA 的 compat-scoped-input-windows | 根执行者 | windows-latest 原生 exact scoped-input 与 windows_fs 组全绿 | `.github/workflows/base.yml:2-4` 的 main push；`:229-246` 以 `utils::windows_fs::tests::` 模块过滤且断言至少一例通过，须在 run log 核对新三例名称 | 失败或取消保留 remote-pending，修复重推后重跑；不得发 tag 或用交叉编译替代 |
| DEP-WD-04 | incoming | 兼容工具 | 既有 0.30.53 CLI | 当前根执行者 | disposable repo 旧二进制拒绝矩阵可运行 | 本机 installed 0.30.53；必须在安装新版前固定可执行副本或可复现下载 | 无可用旧二进制则 WD-02 不能本地验收 |

### 发布分组与并发窗口

| ID | 成员 | 唯一发布点 | 窗口规则 | 失败回滚顺序 | 理由 |
|---|---|---|---|---|---|
| REL-WD01 | WD-01..08, WD-02G | WD-08 | 当前授权 `0.30.85`；先刷新 HEAD/origin、精确写集与最终源码门，随后按 main CI→tag/release→stable→网站内容顺序发布；不复用 `v0.31.0`、不发布 `v0.31.1` | 发行失败按当时最新上游版本前滚，不重指不可变 tag 或降级 writer | CLI/schema/路径/cleaner 必须在同一新版本中上线；旧 `v0.30.66` 不计此门 |

**并发声明:** WD-01..08 与 WD-02G 的依赖 API 接入与验收推进串行；可并行只读研究、审查及已明确文件独占、I 写集不相交的准备性实现/测试。准备性代码不得在上游门通过前被当作下游卡已验收；共享文件只能单写者编辑，根执行者集成后按 DAG 逐卡验收。共享仓库其它执行者须知 installed 0.30.53 已被 future-schema 门拒绝，统一使用本计划验证过的候选二进制。**发布者:** 当前根执行者（单一写入者，不虚构外部负责人）。**真实工作库边界:** 验收实验一律 disposable repo；Libra 真实仓库已应用 2026101902，此后 VCS 使用 `target/debug/libra`；安装旧 0.30.53 的只读对照副本固定在独立路径，不随安装覆盖。**当前发布窗口顺序：** 刷新版本面和最终源码→重跑受影响本地/远端门→按最终 CLI 同步网站文档→核对 HEAD/origin 与精确写集→签名+DCO 推送→final-SHA 原生 Windows D 门→新 tag/release→网站 cf/deploy→远端收口登记。`cc8f437` CI 全绿不能直接替代最终 SHA 的门。

### Phase 0: 边界与手动 worktree

进入：正式计划 review PASS，工作区安全检查通过。退出：WD-01、WD-02G、WD-02、WD-03 各自 A/B 与实现 review PASS，均 locally-accepted，未提前发布。

### Phase 1: 自动工作区与发布

进入：WD-03 已本地验收。退出：WD-04..07 本地验收，WD-08 完成全量与发布门，所有卡继承 C/D 成为 complete。

## 任务卡

### 字段全局默认与例外

`Task type=implementation`；`Release boundary=family child of REL-WD01`；`Release write set=N/A`；`Version increment=N/A`；`C/D coverage from=WD-08`；`Rollback mode=forward-only`；`Migration and rollback=不搬迁旧数据；在当前新布局前滚修复，验证 doctor/list/工作文件 hash 保持`；`Security and privacy=GC-WD-01..03/ADR-WD-04`；`Performance budget=按注册表/指定 run 查找，不递归扫描工作区内容`。这些全局默认即逐卡字段值；卡内未重复书写时仍由本行提供，不视为遗漏。所有新增 test fn 下文标 (new)，实现时名称必须一致或同批修计划；过滤运行匹配 0 tests 不算通过，必须核对实际匹配用例数。

默认覆盖：WD-01 在未对外写新布局前可 revert；WD-02G 为 metadata-only migration/forward-only；WD-08 为 release/immutable-release/self/patch（2026-10-21 用户明确覆盖原 minor 目标）。**规则 waiver:** 仅版本增量由用户改为 patch + 1；其它未批准豁免，实现超过 M 或 AC/VER 上限必须先拆卡修订，不能为保留原卡数而压缩独立判据。

### Task WD-01: 公共布局与仓库发现边界

**Task type:** implementation
**Lifecycle / Acceptance:** pending / 空
**Description:** 建立 common storage 驱动的布局与发现边界，使内部工作区不会误绑定主树。
**Out of scope:** 创建/名称 CLI 由 WD-02/03 承接；任务物化由 WD-04 承接；旧目录搬迁永久非目标。
**Current evidence:** `src/utils/util.rs:205` 已识别 linked 本地元数据，但新容器不存在；`src/internal/worktree_scope.rs:75` 要求固定作用域；`src/command/maintenance.rs:2572` 消费普通 registry。
**Acceptance criteria:**
- [ ] common storage 的统一布局结果不随主树、子目录或 linked 调用位置改变。
- [ ] 内部容器 cwd 的缺失/损坏身份产生明确拒绝，不回落主树。
- [ ] 规范路径和符号链接别名不能绕过容器范围检查。
- [ ] 存储遍历消费者有明确工作副本排除边界，保留对象/index 根枚举。
- [ ] ER-06/06a：本卡文档逐文件验收（不计特有 AC）。
**Verification:**
- [ ] `source .env.test && cargo test --lib worktree_layout`（new 模块单元集；若含 WD-02G 的 `worktree_layout_capability`，计数分别列出，不挪作 WD-01 证据）。
- [ ] `source .env.test && cargo test --test command_test named_worktree_defaults_use_common_storage_from_every_scope`（最终用例名；含主树和 linked 调用位置）。
- [ ] `source .env.test && cargo test --test command_test worktree_isolation_test`（含身份拒绝与 GC 根边界）。
- [ ] `source .env.test && cargo test --test command_test named_worktree_nested_files_stage_and_survive_gc_while_metadata_stays_hidden`（最终用例名；工作文件与元数据边界）。
**Full-suite trigger:** T-1：共享路径/发现 helper；C 由 WD-08 执行。
**Dependencies:** DEP-WD-01。
**Implementation write set:** `src/utils/worktree_layout.rs` (new), `src/utils/mod.rs`, `src/utils/util.rs`, `src/internal/worktree_scope.rs`, `src/command/maintenance.rs`, `src/command/cloud.rs`, `src/command/clone.rs`, `tests/command/worktree_isolation_test.rs`, `docs/development/commands/worktree.md`。
**Docs and compatibility impact:** `docs/development/commands/worktree.md` 登记内部边界；EN/zh/网站 N/A：本卡未启用新 CLI 或新布局写入，后续卡同卡说明公开行为。
**Rollback mode:** revert
**Estimated scope:** M
**Version increment:** N/A
**C/D coverage from:** WD-08
**Granularity:** `type=implementation; axis=布局发现边界; recovery=撤销尚未启用的helper; complete=yes; self-contained=yes; AC=4/8; VER=3/8; landing=4; prod-files=7; scope=M; deps=DEP-WD-01; writeset=序列化于WD-02G; release=family child; split-from=N/A; exception=N/A`

### Task WD-02G: Repository 工作区布局能力门

**Task type:** migration（仅兼容元数据；不迁移用户数据）
**Lifecycle / Acceptance:** pending / 空
**Description:** 用 Repository schema receipt 阻止旧二进制在不了解新任务布局时删除运行元数据。
**Out of scope:** 不迁移任何目录、内容、历史名称；不改 GlobalConfig/SystemConfig；普通 CLI 由 WD-02 接续。本卡自 WD-02 的旧二进制保护轴拆出，避免其超过十二个生产文件。
**Current evidence:** 根执行者真实 0.30.53 disposable repo 实验：schema 2 review state/manifest、终态时间 2000、新 tasks workspace sentinel；`agent clean --gc` exit 0，运行元数据被删除，workspace 留下。现有 `src/internal/db/schema.rs:392` 对 future receipt 拒绝；`sql/migrations/README.md:194` 的 2026101901 已被 agent_kind_grok 占用，已确认下一号 2026101902。
**Acceptance criteria:**
- [ ] 新布局写入前 Repository 存在 2026101902 receipt。
- [ ] writer 只对既有 capability 表幂等写入 version 4，不改用户行。
- [ ] GlobalConfig/SystemConfig 的配置 ledger 与配置内容保持不变。
- [ ] 新建与已有 Repository 得到相同能力门。
- [ ] 普通 rollback 在删除该 receipt 前返回 IrreversibleMigration。
- [ ] 真实旧二进制 agent clean 在 Repository DB connect 的 future-schema 检查拒绝且所有哨兵保持。
- [ ] ER-06/06a：下列文档逐文件验收（不计特有 AC）。
**Verification:**
- [ ] `source .env.test && cargo test --lib worktree_layout_capability`（new，Repository scope、future-version 拒绝及幂等）。
- [ ] `source .env.test && cargo test --test db_migration_test worktree_layout_capability`（new 三例：幂等元数据升级、新库重开、拒绝 rollback 并保留状态）。
- [ ] `source .env.test && cargo test --test db_migration_test`（新 forward-only tip 之前的历史 rollback 测试显式限定基线，不删除其断言）。
- [ ] 真实 0.30.53 disposable repo 重跑已失败的 `agent clean --gc` fixture；断言非零拒绝且 schema 2 state/manifest、tasks sentinel 与 DB 用户行保持。
**Full-suite trigger:** T-1：SQL/schema manifest；C 由 WD-08 包含本卡最终变更的全量承担。
**Dependencies:** WD-01（布局边界）、DEP-WD-04（旧二进制）。
**Implementation write set:** `src/internal/db/migration.rs`, `src/internal/db/schema.rs`, `sql/migrations/2026101902_worktree_layout_capability.sql` (new), `sql/migrations/README.md`, `tests/db_migration_test.rs`, `docs/commands/worktree.md`, `docs/commands/zh-CN/worktree.md`, `docs/commands/agent.md`, `docs/commands/zh-CN/agent.md`, `COMPATIBILITY.md`, `../libra-backend/apps/tanstack-app/content/docs/commands/worktree.en.md`, `../libra-backend/apps/tanstack-app/content/docs/commands/agent.en.md`。
**Release write set:** N/A
**Docs and compatibility impact:** 上列 worktree/agent EN/zh、网站两个映射页与 COMPATIBILITY 说明新布局能力 receipt、旧二进制拒绝和只可前滚；SQL README 注册新号。网站七页已有准备性 dirty 写入；按 GC-WD-06 在最终窗口重新同步验收。
**Rollback mode:** forward-only
**Migration and rollback:** allowlist=Repository，ledger=schema_versions；up 使用既有 worktree_registry_capability 表 INSERT OR IGNORE version 4，schema shape 不变。注册 down=None，不新增 down SQL，由现有 IrreversibleMigration 在任何撤销前拒绝。恢复方法为升级兼容二进制前滚修复，禁止手删 receipt 或降级 DB；配置角色测试使用隔离配置库。
**Estimated scope:** M
**Version increment:** N/A
**C/D coverage from:** WD-08
**Granularity:** `type=migration; axis=旧二进制布局能力门; recovery=保留receipt前滚升级; complete=yes; self-contained=yes; AC=6/8; VER=4/8; landing=2; prod-files=3; scope=M; deps=WD-01,DEP-WD-04; writeset=序列化于WD-01/02; release=family child; split-from=WD-02; exception=N/A`

### Task WD-02: 普通 worktree 名称与内置目录闭环

**Task type:** implementation
**Lifecycle / Acceptance:** pending / 空
**Description:** 让普通 worktree 的命名创建与控制完整遵守新布局和身份契约。
**Out of scope:** 旧二进制 Repository 能力门已拆出 WD-02G；FUSE mount 由 WD-03；任务目录由 WD-04；自动/批量历史补名及搬迁永久非目标。
**Current evidence:** `src/command/worktree.rs:98` 路径 CLI；`src/command/worktree/registry.rs:217` v3 reader；`src/command/add.rs:2817` 排除 common storage；`src/command/worktree/operations.rs:61` 创建入口。
**Acceptance criteria:**
- [ ] NAME/--path 和全部普通控制选择器满足 ADR-WD-02。
- [ ] v4 reader/writer 满足 ADR-WD-03 的旧记录身份保留规则。
- [ ] detached 重挂/显式删除遵守名称释放状态机。
- [ ] 内置 linked 中 add 普通/-N/-p/冲突路径能正确写其本地 index。
- [ ] 主树 add/clean 与子树 commit/checkout 保持作用域隔离。
- [ ] GC 保留只在子树 index 引用的对象。
- [ ] 旧二进制 destructive 矩阵拒绝新格式且不改变数据。
- [ ] ER-06/06a：下列文档逐文件验收（不计特有 AC）。
**Verification:**
- [ ] `source .env.test && cargo test --test command_test worktree_test`（含最终用例 `named_worktree_defaults_use_common_storage_from_every_scope`、`named_worktree_explicit_path_and_move_keep_logical_identity`、`named_worktree_detach_reattach_preserves_head_index_and_files`）。
- [ ] `source .env.test && cargo test --test command_test worktree_isolation_test`（含最终用例 `corrupt_linked_identity_refuses_mutations_and_points_at_repair`、`gc_and_repack_run_in_multi_worktree_repo_keeping_private_roots`）。
- [ ] `source .env.test && cargo test --test command_test worktree_doctor_test`。
- [ ] `source .env.test && cargo test --test compat_worktree_docs_v2_layout`。
- [ ] `source .env.test && cargo test --test compat_worktree_delete_dir`。
- [ ] 真实 0.30.53 对 disposable 新布局 repo 执行 worktree add/remove/move/prune/repair、maintenance GC，并用手工构造的 schema 2 task/state fixture 执行 agent clean、review、investigate continue；WD-08 再以最终真实格式重跑；逐项记录拒绝退出码与 registry/HEAD/index/对象/工作文件/运行记录前后 hash 一致性（前置门，不等发布后）。
**Full-suite trigger:** T-1/T-3：共享 registry 与公开 CLI 重定义；C 由 WD-08 执行。
**Dependencies:** WD-02G（旧二进制仓库能力门；已依赖 WD-01 的布局 helper），DEP-WD-04。
**Implementation write set:** `src/command/worktree.rs`, `src/command/worktree/registry.rs`, `src/command/worktree/operations.rs`, `src/command/worktree/lock.rs`, `src/command/worktree/doctor.rs`, `src/command/add.rs`, `src/utils/worktree_layout.rs`, `src/cli.rs`, `src/command/control_lock.rs`, `src/command/hooks.rs`, `src/command/agent/hooks.rs`, `src/internal/worktree_scope.rs`, `tests/command/worktree_test.rs`, `tests/command/worktree_isolation_test.rs`, `tests/command/worktree_doctor_test.rs`, `tests/compat/worktree_docs_v2_layout.rs`, `tests/compat/worktree_delete_dir.rs`, `docs/commands/worktree.md`, `docs/commands/zh-CN/worktree.md`, `docs/commands/hooks.md`, `docs/commands/zh-CN/hooks.md`, `docs/development/commands/worktree.md`, `COMPATIBILITY.md`, `../libra-backend/apps/tanstack-app/content/docs/commands/worktree.en.md`, `../libra-backend/apps/tanstack-app/content/docs/commands/hooks.en.md`。
**Docs and compatibility impact:** 上列 worktree/hooks 双语文档及网站两页、COMPATIBILITY.md 同卡同步；解释 NAME 与旧 PATH 行为变化、JSON 可选 name、detached 和 --path，并统一 repair 提示的 `--confirm --path` 语法。cli/control_lock/hooks/agent hooks/worktree_scope 的写入仅限同型 repair 提示更新；sandbox/mod.rs 的同型提示更新归 WD-04 已有写集，不新增命令语义。
**Rollback mode:** forward-only
**Estimated scope:** M
**Version increment:** N/A
**C/D coverage from:** WD-08
**Granularity:** `type=implementation; axis=普通worktree命名生命周期; recovery=按v4身份前滚修复; complete=yes; self-contained=yes; AC=7/8; VER=6/8; landing=4; prod-files=12; scope=M; deps=WD-02G,DEP-WD-04; writeset=序列化于WD-02G/03; release=family child; split-from=N/A; exception=N/A`

### Task WD-03: FUSE 名称、管理目录与递归防护

**Task type:** implementation
**Lifecycle / Acceptance:** pending / 空
**Description:** 让 FUSE 后端在统一布局内安全挂载并共享名称约束。
**Out of scope:** 不新增 FUSE move；自动任务仍保持 copy backend。
**Current evidence:** `src/command/worktree-fuse.rs:61` 重复 CLI；`:709` add；`:630` 旧 upper root；`:248` 未版本化注册表。
**Acceptance criteria:**
- [ ] 两套 CLI 的名称/路径选择器一致，FUSE move 明确拒绝。
- [ ] 普通/FUSE 并发同名创建只有一个胜者。
- [ ] FUSE 新登记使用版本化 entries 与普通 v4 旧二进制门。
- [ ] 默认管理根/挂载点/upper 符合 ADR-WD-01。
- [ ] opaque 子容器阻止挂载路径递归暴露。
- [ ] 外部路径覆盖遵守 lower/upper/管理目录禁入。
- [ ] 挂载/健康/保存失败留下可验证恢复状态，不泄漏挂载或任意删除用户目录。
- [ ] ER-06/06a：文档逐文件验收（不计特有 AC）。
**Verification:**
- [ ] `source .env.test && cargo test --features worktree-fuse --lib worktree`。
- [ ] `source .env.test && cargo test --features worktree-fuse --test command_test worktree_fuse_test`（含最终用例 `wd_fuse_live_mount_boundary`、`test_fuse_multiple_worktrees_mount_and_access`、`fuse_named_controls_preserve_registration_when_cleanup_is_incomplete`）。
- [ ] `source .env.test && cargo test --test command_test named_worktree_namespace_includes_fuse_entries_without_mounting`（最终无 FUSE feature 用例名）。
- [ ] 受支持 Unix 实机执行新 `wd_fuse_live_mount_boundary` ignored case：`source .env.test && LIBRA_RUN_FUSE_TESTS=1 cargo test --features worktree-fuse --test command_test command::worktree_fuse_test::wd_fuse_live_mount_boundary -- --ignored --exact`；证明 mount→读 opaque→兄弟不可见→unmount，缺驱动是未运行。macOS/macFUSE 另运行 `source .env.test && LIBRA_RUN_FUSE_TESTS=1 RUST_TEST_THREADS=1 cargo test --features worktree-fuse --test command_test worktree_fuse_test -- --include-ignored`，覆盖完整真实挂载组。
**Full-suite trigger:** T-1/T-3：两套公开入口/共享名称锁及 serial/nextest 配置；C 由 WD-08 执行。
**Dependencies:** WD-02（注册表和共享锁），DEP-WD-02，DEP-WD-04。
**Implementation write set:** `src/command/worktree-fuse.rs`, `src/command/restore.rs` (显式指定 FUSE 填充目标，保持调用者索引与对象来源，并提供无挂载 L1 回归), `src/command/worktree/registry.rs`, `src/command/worktree/lock.rs`, `src/utils/worktree_layout.rs`, `src/utils/fuse.rs`, `src/utils/util.rs`, `tests/command/worktree_fuse_test.rs`, `tests/command/worktree_test.rs`, `tests/SERIAL_REGISTRY.tsv`, `tests/SRC_SERIAL_REGISTRY.tsv`, `.config/nextest.toml`, `docs/commands/worktree.md`, `docs/commands/zh-CN/worktree.md`, `docs/development/commands/worktree.md`, `COMPATIBILITY.md`, `../libra-backend/apps/tanstack-app/content/docs/commands/worktree.en.md`。
**Docs and compatibility impact:** 上列三个 worktree 文档、网站 worktree.en.md、COMPATIBILITY.md 同卡写清 FUSE 默认布局、平台要求、umount 与 move 边界。
**Rollback mode:** forward-only
**Estimated scope:** M
**Version increment:** N/A
**C/D coverage from:** WD-08
**Granularity:** `type=implementation; axis=FUSE命名挂载生命周期; recovery=卸载后按归属前滚清理; complete=yes; self-contained=yes; AC=7/8; VER=4/8; landing=4; prod-files=6; scope=M; deps=WD-02,DEP-WD-02,DEP-WD-04; writeset=序列化于WD-02/04; release=family child; split-from=N/A; exception=N/A`

### Task WD-04: 任务工作区描述与受限物化基础设施

**Task type:** implementation
**Lifecycle / Acceptance:** pending / 空
**Description:** 建立以 run/attempt 为归属的任务创建、清理和 sandbox 工作根接口；安全句柄删除在 Unix/Windows 保持同一边界。
**Out of scope:** review/investigate 默认切换分别由 WD-05/06；checkpoint 输入由 WD-07。
**Current evidence:** `src/internal/ai/workspace_isolation.rs:127` 先删除旧目标再 copy；`src/internal/ai/sandbox/runtime.rs:184` 环境目录派生；`src/internal/ai/sandbox/policy.rs:620` workspace rebase。
**Acceptance criteria:**
- [ ] 创建意图持久化早于目录创建，并以排他创建防止覆盖。
- [ ] 任务描述保存可验证的 repo/run/attempt/kind 和生命周期。
- [ ] 清理仅接受合法归属，未证明归属目录保留并报告。
- [ ] sandbox 显式接收本 attempt 的工作根/缓存根。
- [ ] 受限 policy 拒绝共享元数据和兄弟目录写入。
- [ ] 共享 descriptor 类型提供 schema 2 所需的版本和归属校验接口；WD-05/06 在各自 store 接线 schema 2、旧 schema 1 reader 与未知版本拒绝。
- [ ] ER-06/06a：文档逐文件验收（不计特有 AC）。
**Verification:**
- [ ] `source .env.test && cargo test --lib workspace_isolation`（补 new `wd_attempt_intent`、`wd_cleanup_ownership`）。
- [ ] `source .env.test && cargo test --lib sandbox::runtime`（补 new `wd_explicit_workspace_environment`）。
- [ ] `source .env.test && cargo test --lib sandbox::policy`（含最终用例 `task_workspace_policy_allows_only_owned_workspace_and_cache_directories`、`task_workspace_policy_refuses_tampered_or_unowned_descriptors`、`task_workspace_read_denials_cover_managed_and_external_fuse_aliases`）。
- [ ] `source .env.test && cargo test --test command_test sandbox_status_test`。
- [ ] final-SHA Windows D 门包含 `utils::windows_fs::tests::workspace_creation_is_exclusive_and_pins_its_identity`、`managed_container_creation_is_idempotent_and_refuses_files`、`workspace_cleanup_refuses_a_replacement_identity` 及 `workspace_isolation::tests::wd_full_copy_refuses_junction_to_outside_directory`；这些新用例验证句柄钉住、排他创建、容器幂等、替换目标拒绝及复制时不跟随 junction。
**Full-suite trigger:** T-1：共享任务物化/sandbox helper；C 由 WD-08 执行。
**Dependencies:** WD-03（全部目录容器边界就绪）。
**Implementation write set:** `src/internal/ai/workspace_isolation.rs`, `src/internal/ai/sandbox/runtime.rs`, `src/internal/ai/sandbox/policy.rs`, `src/internal/ai/sandbox/mod.rs`, `src/utils/worktree_layout.rs`, `src/utils/windows_fs.rs`, `src/command/worktree/doctor.rs`, `src/command/agent/doctor.rs`, `src/command/maintenance.rs`, `tests/command/sandbox_status_test.rs`, `.github/workflows/base.yml`, `docs/commands/sandbox.md`, `docs/commands/zh-CN/sandbox.md`, `docs/development/commands/sandbox.md`, `../libra-backend/apps/tanstack-app/content/docs/commands/sandbox.en.md`。
**Docs and compatibility impact:** sandbox EN/zh、开发文档和网站 sandbox.en.md 同卡同步受限工作根/缓存语义；sandbox/mod.rs 的损坏身份诊断复用 `worktree repair --confirm --path`，同卡核对并纳入 sandbox 状态测试。review/investigate 尚未切换，下一卡同卡更新。agent doctor 与 maintenance 只补共享 schema 2/未知归属的消费者检查，不新增无关 doctor 修复或 GC 策略。
**Rollback mode:** forward-only
**Estimated scope:** M
**Version increment:** N/A
**C/D coverage from:** WD-08
**Granularity:** `type=implementation; axis=任务工作区受限物化; recovery=按持久化意图前滚恢复; complete=yes; self-contained=yes; AC=6/8; VER=4/8; landing=4; prod-files=9; scope=M; deps=WD-03; writeset=序列化于WD-03/05; release=family child; split-from=N/A; exception=N/A`

### Task WD-05: Review 运行与清理接入

**Task type:** implementation
**Lifecycle / Acceptance:** pending / 空
**Description:** 将 review 普通工作副本完整切换到统一任务目录生命周期。
**Out of scope:** investigate 由 WD-06；checkpoint 分支由 WD-07；不搬旧 review 目录。
**Current evidence:** `src/internal/ai/review/runner.rs:470` sessions_root 构造；`src/internal/ai/review/store.rs:273` workspace_root；`src/command/agent/clean.rs:767` run_dir 清理。
**Acceptance criteria:**
- [ ] 新 review 普通工作副本位于 tasks/run/attempt/workspace。
- [ ] 成功、失败、取消和崩溃恢复消费同一任务描述。
- [ ] 清理失败保留运行记录和可重试状态。
- [ ] agent clean 先清归属工作目录，再清运行元数据。
- [ ] 并行 review 不能覆盖或清除兄弟工作目录。
- [ ] review state/manifest 写 schema 2 optional descriptor，兼容读 schema 1 且未知版本拒绝写/clean。
- [ ] ER-06/06a：文档逐文件验收（不计特有 AC）。
**Verification:**
- [ ] `source .env.test && cargo test --test agent_review_workflow_test`（含最终用例 `review_checkpoint_scoped`、`fix_rg_scoped_03_completed_review_clean`、`fix_rg_scoped_03_infrastructure_error_clean`）。
- [ ] `source .env.test && cargo test --lib review::store`（含最终用例 `managed_cleanup_preserves_metadata_until_ownership_is_proven`、`fix_rg_scoped_03_cleanup_boundary`）。
- [ ] `source .env.test && cargo test --test command_test agent_clean_test`（清理消费者组；运行记录归属断言见上述 workflow/store 用例）。
- [ ] `source .env.test && cargo test --test agent_review_span_test`。
**Full-suite trigger:** none：消费已定义生命周期接口；C 继承 WD-08。
**Dependencies:** WD-04（任务描述/清理接口和 Windows handle adapter）。
**Implementation write set:** `src/internal/ai/review/runner.rs`, `src/internal/ai/review/store.rs`, `src/command/agent/review.rs`, `src/command/agent/clean.rs`, `tests/agent_review_workflow_test.rs`, `tests/agent_review_span_test.rs`, `tests/command/agent_clean_test.rs`, `docs/commands/review.md`, `docs/commands/zh-CN/review.md`, `docs/development/commands/review.md`, `docs/commands/agent.md`, `docs/commands/zh-CN/agent.md`, `docs/development/commands/agent.md`, `../libra-backend/apps/tanstack-app/content/docs/commands/review.en.md`, `../libra-backend/apps/tanstack-app/content/docs/commands/agent.en.md`。
**Docs and compatibility impact:** 上列 review/agent EN/zh、开发文档、网站 review.en.md/agent.en.md 同卡说明工作文件与 sessions 分离、clean 失败保留语义、任务副本内隐式 Libra 仓库发现有意拒绝；网站产物按 GC-WD-06 在最终窗口重新同步验收。
**Rollback mode:** forward-only
**Estimated scope:** M
**Version increment:** N/A
**C/D coverage from:** WD-08
**Granularity:** `type=implementation; axis=review任务工作区生命周期; recovery=按run描述前滚清理; complete=yes; self-contained=yes; AC=6/8; VER=4/8; landing=2; prod-files=4; scope=M; deps=WD-04; writeset=序列化于WD-04/06; release=family child; split-from=N/A; exception=N/A`

### Task WD-06: Investigate 与 continue 接入

**Task type:** implementation
**Lifecycle / Acceptance:** pending / 空
**Description:** 让 investigate 每次执行/continue 使用独立 attempt 工作副本并安全结束。
**Out of scope:** checkpoint 分支由 WD-07；不搬旧 investigate 目录。
**Current evidence:** `src/internal/ai/investigate/runner.rs:727` copy 分支；`:1395` guard；`src/internal/ai/investigate/store.rs:856` run cleanup。
**Acceptance criteria:**
- [ ] 新 investigate 使用 tasks/run/attempt/workspace。
- [ ] continue 分配新 attempt，不覆盖上一轮目录。
- [ ] 失败、取消、resume 和 clean 使用同一描述归属边界。
- [ ] 运行锁冲突拒绝第二次物化。
- [ ] investigate state/manifest 写 schema 2 optional descriptor，兼容读 schema 1 且未知版本拒绝写/clean。
- [ ] 清理失败保留恢复记录。
- [ ] ER-06/06a：文档逐文件验收（不计特有 AC）。
**Verification:**
- [ ] `source .env.test && cargo test --test agent_investigate_workflow_test`（含最终用例 `stalled_cancelled_paused_and_continue_resume_are_pinned`、`investigate_checkpoint_scoped`、`fix_rg_scoped_03_completed_investigate_clean`）。
- [ ] `source .env.test && cargo test --lib investigate::store`（含最终用例 `replaced_managed_attempt_keeps_retry_metadata_and_outside_files`、`fix_rg_scoped_03_cleanup_boundary`）。
- [ ] `source .env.test && cargo test --test command_test agent_clean_test`（清理消费者组；运行记录归属断言见上述 workflow/store 用例）。
- [ ] `source .env.test && cargo test --test agent_investigate_span_test`。
**Full-suite trigger:** none：消费生命周期接口；C 继承 WD-08。
**Dependencies:** WD-05（共享 clean 接线完成）。
**Implementation write set:** `src/internal/ai/investigate/runner.rs`, `src/internal/ai/investigate/store.rs`, `src/command/agent/investigate.rs`, `src/command/agent/clean.rs`, `tests/agent_investigate_workflow_test.rs`, `tests/agent_investigate_span_test.rs`, `tests/command/agent_clean_test.rs`, `docs/commands/investigate.md`, `docs/commands/zh-CN/investigate.md`, `docs/development/commands/investigate.md`, `docs/commands/agent.md`, `docs/commands/zh-CN/agent.md`, `docs/development/commands/agent.md`, `../libra-backend/apps/tanstack-app/content/docs/commands/investigate.en.md`, `../libra-backend/apps/tanstack-app/content/docs/commands/agent.en.md`。
**Docs and compatibility impact:** 上列 investigate/agent EN/zh、开发文档与网站两页同卡说明 attempt 和 resume/clean 行为，以及任务副本内隐式 Libra 仓库发现有意拒绝；网站产物按 GC-WD-06 在最终窗口重新同步验收。
**Rollback mode:** forward-only
**Estimated scope:** M
**Version increment:** N/A
**C/D coverage from:** WD-08
**Granularity:** `type=implementation; axis=investigate尝试生命周期; recovery=保留attempt描述后前滚恢复; complete=yes; self-contained=yes; AC=6/8; VER=4/8; landing=2; prod-files=4; scope=M; deps=WD-05; writeset=序列化于WD-05/07; release=family child; split-from=N/A; exception=N/A`

### Task WD-07: Checkpoint 输入目录和句柄分离

**Task type:** implementation
**Lifecycle / Acceptance:** pending / 空
**Description:** 将 checkpoint 输入移到任务 attempt，保留独立元数据验证和安全物化/删除边界。
**Out of scope:** 不改 checkpoint 内容格式和来源证明；不搬现存输入。
**Current evidence:** `src/internal/ai/checkpoint_input.rs:228` 生产物化 API、`:1605` RunRootHandle；review/investigate 生产分支目前传 run_dir。
**Acceptance criteria:**
- [ ] 生产物化使用不同的运行元数据句柄和输入目录句柄。
- [ ] 新输入位于 tasks/run/attempt/checkpoint-input。
- [ ] 目录迁出不绕过 catalog/object 内容验证。
- [ ] cancel/deadline/no-follow 防护保持有效。
- [ ] 输入保持只读并能在合法归属下安全回收。
- [ ] continue 不能覆写前一 attempt 输入。
- [ ] ER-06/06a：文档逐文件验收（不计特有 AC）。
**Verification:**
- [ ] `source .env.test && cargo test --lib checkpoint_input`（含最终用例 `fix_rg_scoped_04_metadata_boundary`、`fix_rg_scoped_04_readonly_cleanup`、`managed_checkpoint_rejects_replaced_metadata_run`）。
- [ ] `source .env.test && cargo test --lib review::store`。
- [ ] `source .env.test && cargo test --lib investigate::store`。
- [ ] `source .env.test && cargo test --test agent_review_workflow_test review_checkpoint_scoped`（最终用例名）。
- [ ] `source .env.test && cargo test --test agent_investigate_workflow_test investigate_checkpoint_scoped`（最终用例名）。
**Full-suite trigger:** T-1：共享安全物化 helper；C 由 WD-08 执行。
**Dependencies:** WD-06（普通任务完整接入）。
**Implementation write set:** `src/internal/ai/checkpoint_input.rs`, `src/internal/ai/review/runner.rs`, `src/internal/ai/review/store.rs`, `src/internal/ai/investigate/runner.rs`, `src/internal/ai/investigate/store.rs`, `src/internal/ai/workspace_isolation.rs`, `tests/agent_review_workflow_test.rs`, `tests/agent_investigate_workflow_test.rs`, `docs/commands/review.md`, `docs/commands/zh-CN/review.md`, `docs/development/commands/review.md`, `docs/commands/investigate.md`, `docs/commands/zh-CN/investigate.md`, `docs/development/commands/investigate.md`, `../libra-backend/apps/tanstack-app/content/docs/commands/review.en.md`, `../libra-backend/apps/tanstack-app/content/docs/commands/investigate.en.md`。
**Docs and compatibility impact:** 上列 review/investigate EN/zh、开发文档和网站两页同卡写 checkpoint 输入位置、只读和不迁旧数据。
**Rollback mode:** forward-only
**Estimated scope:** M
**Version increment:** N/A
**C/D coverage from:** WD-08
**Granularity:** `type=implementation; axis=checkpoint输入安全物化生命周期; recovery=按独立句柄前滚清理; complete=yes; self-contained=yes; AC=6/8; VER=5/8; landing=4; prod-files=6; scope=M; deps=WD-06; writeset=序列化于WD-06/08; release=family child; split-from=N/A; exception=N/A`

### Task WD-08: 家族聚合验证与补丁版本交付

**Task type:** release
**Lifecycle / Acceptance:** pending / 空
**Description:** 在同一最终 bump 树上验证全部布局行为，发布 CLI 和网站文档。
**当前版本决议:** 用户要求基于最新上游已发布版本 patch + 1；GitHub Latest `v0.30.84` 后当前目标是 `v0.30.85`，旧 `.83` 候选未发布；仍禁止发布 `v0.31.1`。最终 tag 前重新核对远端版本、源码 SHA、所有门与资产名；如 `v0.30.85` 被并发占用，再按上游最新 patch + 1 前滚，不覆盖已有 tag。
**Out of scope:** 不引入新行为；发现缺陷在原行为卡前滚或新增 FIX 卡后重新验收。
**Current evidence:** 上游 GitHub 最新已发布 `v0.30.84`，当前目标 `v0.30.85`；本地已洁净快进到 `99a92c1`，版本四面一致，`branch -f` 的五个聚焦用例由修复前 2/5 前滚至 5/5，最终 branch 全组 52/52、Display-pin 1/1、版本面 2/2。严格 Clippy/fmt/diff --check exit0，FIX-WD-08g 跟进 Claude Code 源码复审字面 `VERDICT: PASS`；真实旧版 0.30.53 对九种消费者全为 exit128，69 个稳定文件哈希未变。网站十页限定同步 drift=0、脚本 8/8、Vitest 11/11、typecheck/build exit0，第三轮独立文档复审字面 `VERDICT: PASS`。`fbdf5ee` 上的 `cargo test --all` 因文档复审前滚主动中断 exit130，不计最终门；最终文档提交后的双全量、macFUSE、final-SHA CI、release 资产/stable 与网站上线仍须复验。旧 `v0.30.66` 与候选 `v0.30.80/82/83` 不计本卡 C/D。
**Acceptance criteria:**
- [ ] WD-01..07 与 WD-02G 的最终变更均被发布树包含。
- [ ] 旧二进制拒绝矩阵在最终格式复验通过。
- [ ] 最终 FUSE 真实挂载门通过。
- [ ] 仓库/网站命令文档同步通过。
- [ ] 最终实现 Claude Code `VERDICT: PASS`。
- [ ] 版本面一致并采用用户指定的上游最新 patch + 1（当前 `0.30.85`）；升级说明明确公开兼容变化。
- [ ] 签名+DCO main 推送与远端 SHA 对应。
- [ ] release.yml 和 Cloudflare artifact/stable 指向此次发布。
- [ ] 网站 cf 推送部署后线上命令页呈现新契约。
- [ ] 全部卡继承 C/D 后才推进 done/complete。
**Verification:**
- [ ] `source .env.test && cargo test --test compat_version_surface_sync`。
- [ ] `source .env.test && cargo test --test compat_serial_registry`。
- [ ] `source .env.test && cargo test --features worktree-fuse --test command_test worktree_fuse_test`。
- [ ] 真实 FUSE `wd_fuse_live_mount_boundary` 与旧 0.30.53 二进制拒绝矩阵在最终树各留独立结果（2 门）。
- [ ] ER-04 静态必需门（不计特有 VER）：`cargo +nightly fmt --all --check`。
- [ ] ER-04 静态必需门（不计特有 VER）：`cargo clippy --all-targets --all-features -- -D warnings`。
- [ ] 用户要求的全套检查（不计特有 VER）：`source .env.test && cargo test --all`。
- [ ] ER-13/14 发布权威全量门（不计特有 VER）：`source .env.test && source .env.live-test && cargo nextest run --all --no-fail-fast --retries 2`。
- [ ] ER-06a 网站同步（backend cwd；不计特有 VER）：`python3 scripts/sync-libra-command-docs.py --libra-root ../libra --write --only worktree agent review investigate sandbox hooks upgrade init commit branch`，随后同命令去掉 `--write` 检查十页；另记录全库只读检查的范围外漂移。
- [ ] ER-06a 网站脚本测试（backend cwd；不计特有 VER）：`python3 scripts/test_sync_libra_command_docs.py`。
- [ ] ER-06a 网站单元测试（backend cwd；不计特有 VER）：`pnpm exec vitest run tests/unit/docs-manifest.test.ts tests/unit/docs-search.test.ts`。
- [ ] ER-06a 网站类型检查（backend cwd；不计特有 VER）：`pnpm typecheck`。
- [ ] ER-06a 网站构建（backend cwd；不计特有 VER）：`pnpm build`。
- [ ] ER-04 D 必需门（不计特有 VER）：final main SHA 的 `compat-scoped-input-windows` 原生四 exact fixture 与 windows_fs 组全绿，记录 SHA/run ID；tag 发布前必须取得。
- [ ] 实读最终源码/文档的 Claude Code 输出与全部 P0/P1 关闭记录。
- [ ] 已发布 CLI 在 disposable repo 的 NAME 默认路径、--path、list、remove 冒烟。
- [ ] 生产 `https://libra.tools/en/docs/commands/worktree` 与中文对应页及 review/investigate 页的实际页面内容核验（不以部署成功状态替代内容）。
**Full-suite trigger:** T-2（家族发布点）/T-1（安装版本面）；bump 树执行全量 nextest，fmt/clippy 独立必需。
**Dependencies:** WD-07、DEP-WD-01、DEP-WD-02、DEP-WD-03、DEP-WD-04、DEP-WD-05。
**Deliverables:** 本文「实施证据汇总」与版本面/release notes；全部结果写本文，不新建证据文件。
**Implementation write set:** 本计划文件、`docs/development/plan/plan-status.md`、`docs/development/plan/plan-long.md`；网站同步范围控制的 `../libra-backend/scripts/sync-libra-command-docs.py`、`../libra-backend/scripts/test_sync_libra_command_docs.py`、`../libra-backend/scripts/README-command-doc-sync.md`、`docs/commands/upgrade.md`、`docs/commands/zh-CN/upgrade.md` 与原七张受影响 EN 命令页（含 `../libra-backend/apps/tanstack-app/content/docs/commands/upgrade.en.md`）；FIX-WD-05 前滚补入 `src/command/control_lock.rs`、`src/command/init_conversion.rs`、`src/command/service.rs`、`src/cli.rs`、`tests/command/init_conversion_test.rs`、`.github/workflows/base.yml`、`docs/commands/init.md`、`docs/commands/zh-CN/init.md` 与 `../libra-backend/apps/tanstack-app/content/docs/commands/init.en.md`，FIX-WD-04 补入 `tests/agent_mcp_exec_test.rs`，所有新增写入均已在本计划登记。如只有文档收口修正，可编辑 `COMPATIBILITY.md` 及各行为卡已列文档，须登记回原卡并重审。
**Release write set:** 以 `compat_version_surface_sync` 当次报告集合为准（当前 `Cargo.toml`, `install.sh`, `install.ps1`），工具链更新的 `Cargo.lock` 与 release artifacts。
**FIX-WD-06 write-set addendum:** 最终全量暴露 macOS 工作流兼容性守卫不能解析新增第二个 `run` 步骤；补入 `tests/compat/matrix_alignment.rs` 的按步骤提取及第二步私有 Cargo stub 回归。仅修测试守卫，不扩大运行时行为。
**FIX-WD-07 write-set addendum:** 上游 #501 `init --template` 修复在最终 release 树引入别名路径自复制回归，补入 `src/command/init.rs`、`tests/command/init_test.rs`、`tests/command/init_conversion_test.rs`、`.github/workflows/base.yml` 的原生 Windows 六个 exact 断言、`docs/commands/init.md`、`docs/commands/zh-CN/init.md`、`CHANGELOG.md`；网站 `init.en.md` 已属 FIX-WD-05 八页范围。转换回滚对同类别名路径也须保全已有目录。只修发布树的 init 安全边界，不改变 Worktree 默认布局或增加旧数据迁移。
**FIX-WD-07b write-set addendum:** 发布前独立目标路径检查发现 Unix 的 `init regular-file/..` 已创建父目录仓库却因 CLI 用原始拼写再开库而 exit128；`missing/..` 也存在相同后置开库失败。补入已列发布写集的 `src/cli.rs`，并继续修改 `src/command/init.rs`、`tests/command/init_test.rs`、`docs/commands/init.md`、`docs/commands/zh-CN/init.md`、`CHANGELOG.md`、网站八页范围内的 `init.en.md`。在 Unix 目标解析时以现有 `LBR-REPO-002` 拒绝已有非目录中间组件，从初始化的实际规范化存储路径进行 CLI 后置哈希格式读取，回归覆盖“失败零写”与“规范化成功”；最终树须重跑受影响 focused、fmt/严格 Clippy、双全量、macFUSE 与 final-SHA CI/源码复审。不改变 Worktree 默认布局或增加旧数据迁移。
**FIX-WD-07c write-set addendum:** `30b11fb` 的原生 Windows CI `compat-scoped-input-windows` 在发行文件名回归的真实 `init --vault=false` 失败：数据库 URI 将 canonical `//?/C:/...` 中的 `?` 当成查询起点（`unknown query parameter /C:/...`）。补入 `src/internal/db.rs` 的 Windows SQLite 路径规范化及专门单元断言、`.github/workflows/base.yml` 的该精确 Windows 测试过滤器，继续使用既有 `tests/release_binary_name_test.rs` 端到端命令验证，并更新 `CHANGELOG.md` 与本文。修复必须在最终 SHA 的原生 Windows CI 中复跑成功；此共享 helper 触发 WD-08 双全量，不能以 macOS 本地通过代替。
**FIX-WD-08a full-suite write-set addendum:** `92910df` 树的 `source .env.test && cargo test --all` 于 `mega2_diag::run_classifies_connect` 失败（4581 passed / 1 failed / 9 ignored，exit101）：测试先释放 `:0` 临时端口，再异步请求，其他并行测试或进程占用同端口时得到 `transport=request`。初版有界重新选端口夹具独立复审虽字面 PASS，但仍可能抢占其他测试的单次监听端口。保留未监听 socket 的实验在 macOS 返回 timeout（focused 6/8，故撤回 socket2 依赖），最终 `src/internal/protocol/mega2_diag.rs` 选用 1..8 低位 loopback 端口（OS `:0` 临时端口分配不选此区间），有界尝试宿主已有服务占用并严格要求真实 connect 错误；新树 focused 8/8。最终树重新跑 fmt/严格 Clippy、独立审查、整套 cargo 与 nextest；旧红色全量不计完成。
**FIX-WD-08c macOS upstream build write-set addendum:** `v0.30.81` 上游合流后的新树首个 Mac lib 编译被 `src/internal/ai/mcp_server/exec.rs` 的 `extern "C"` Rust 2024 语法拒绝；将该唯一外部函数声明改为 `unsafe extern "C"`，既有具体调用仍只在原有 `unsafe` 区块。另更新 `CHANGELOG.md`，在最终 Mac/Linux/Windows CI 与双全量、独立审查中验证，不将此前 macFUSE 成绩继承到合流树。
**FIX-WD-08d 上游 MCP 采样器边界 addendum:** `.82` 合流树的独立源码复审指出上游 MCP 内存采样线程可能在 `try_wait` 回收 worker 之后、观察 stop 之前对重用的 PID/进程组发 SIGKILL。`src/internal/ai/mcp_server/exec.rs` 为采样与回收共用短时互斥门，在回收时于门内标记停止，deadline/error 分支于异步回收前停止采样；原有 512 MiB 预算/30 秒 deadline 不变。`docs/commands/agent.md`、网站 agent 页与 `CHANGELOG.md` 同时将“所有超限必定报 LBR-AGENT-028”的过强断言收紧为“已检测超限报 028、Windows OS memory-limit 退出可能报 037”。受影响 MCP focused、fmt/严格 Clippy、双全量、独立审查与最终 SHA CI 须重跑；这属于上游合流的发布前缺陷修复，不扩展旧 Worktree 数据迁移范围。
**FIX-WD-08e Windows bare conversion rollback addendum:** `27a72b3` 的原生 Windows CI 第三项 init exact 在失败回滚后仍见 `libra.db`，暴露 `src/command/init.rs` 转换错误路径中 SeaORM SQLite pool 在 Windows 删除新文件前未显式关闭。仅在 Windows Git 转换期间保留新 pool 克隆，任何转换错误回滚前先对 `src/internal/db.rs` 的共享缓存 pool 与新 pool 执行总计 10 秒有界关闭；若无法关闭则保留 marker 并返回 `LBR-REPO-003` 人工恢复指引，避免部分删除。缓存关闭按规范路径匹配同一 SQLite 文件的多个路径别名，并尝试关闭全部匹配 pool 后返回首个错误。测试扩展既有 bare 转换回滚用例以注入 `after-setup`（会打开缓存 pool），再以独立 Windows 用例注入关闭失败并验证 marker、数据库与用户文件全部保留；数据库单元测试覆盖 `..` 路径别名。写集增加 `src/command/init.rs`、`src/internal/db.rs`、`tests/command/init_conversion_test.rs`、`.github/workflows/base.yml` 的第七项 Windows init exact 和数据库别名单元过滤项、`docs/commands/{init,zh-CN/init}.md`、网站既有 init 页、`CHANGELOG.md` 与本文；最终 SHA 原生 CI 必须逐项通过。共享 DB helper/命令/CI 变更触发双全量、fmt/严格 Clippy、源码/网站独立复审，不将前两项 init 成绩视为整组通过。
**FIX-WD-08b Windows command-test write-set addendum:** `39586631004` Windows CI 在原生捕获、工作区、锁、新 DB 单元和发行二进制回归均通过后，六项 init path-safety exact 的第一个因 `command_test` 全目标编译出现 32 个既有 Unix 专用测试错误而失败。补入 `tests/command/mod.rs` 及相关测试文件的最小 `#[cfg(unix)]` 和平台专用 import 条件；首轮 Claude FAIL/P1 的四个 Windows `dead_code` 辅助函数也加门，三项 Unix 锁调用测试同步加门；二轮字面 `VERDICT: PASS`。随后 `a45a1c6` 的原生 Windows 同样在六项 init exact 前只剩 `tests/command/merge_test/message_options.rs` 的 Unix 专用 `SIGNOFF` 常量触发 `dead_code`；该常量亦加 `#[cfg(unix)]`。保留非 Unix 可编译/可运行测试，六项 init 仍须在新 SHA 原生 Windows 严格警告门以非零匹配逐项通过。此为测试编译门前滚，不改变运行时行为；全量本地 cargo/nextest 和最终 SHA CI 必须重跑。
**WD-08 网站复审补充写集：** `0.30.80` 八页网站最终 Claude 复审首轮字面 `VERDICT: FAIL`：英文 agent 源页混入中文、macFUSE/init 以“0.30.80 或更新”误涵盖早于修复的 `v0.31.0` tag。补入 `docs/commands/agent.md`、`docs/commands/worktree.md`、`docs/commands/zh-CN/worktree.md`、`docs/commands/init.md`、`docs/commands/zh-CN/init.md`、`docs/commands/upgrade.md`、`docs/commands/zh-CN/upgrade.md`、`CHANGELOG.md` 与既有网站八页范围，按实际源码/平台资产区分支持版本；同时修复 `src/internal/sequencer/mod.rs` 沿用旧位置参数的 worktree repair 提示。网站复审的四项 P2：FUSE 构建缺 `--resolve-identity --yes`、`--force` 可用版本、0.30.67 macOS/Windows 缺资产、运行中转换 owner 的拒绝码，一并在双语源文档和网站同步页前滚。用户可见提示修正低影响，不为字面字符串单独写实现镜像测试；按 WD-08 全量门验证。最终源码/网站均须重新审查 PASS，CI 与双全量重跑。
**WD-08 文档核对补充写集：** 网站最终复审发现 `worktree` 页面把标准构建现有 `--force` 误写为不支持；补入 `docs/commands/worktree.md`、`docs/commands/zh-CN/worktree.md`、`COMPATIBILITY.md` 和网站八页范围内的 `worktree.en.md`，明确 `add --force` 可复用非空目标、`remove --delete-dir --force` 会删除未提交修改、FUSE 构建的 `-f` 是 `--fuse` 且官方二进制未启用 FUSE。此为现行 CLI 文档勘误，不改命令行为；须复审和重跑最终文档/全量门。
**WD-08 网站复审后文档勘误：** 新一轮网站复审指出旧 registry 的无名 worktree 不能仅凭 `add <NAME>` 重新挂接，必须使用 `add <NAME> --path <注册路径>`；同时 init 的 SIGINT/SIGTERM 自动回滚仅限 Unix，`v0.30.79` 只有源码无可下载发行二进制。同步修改 `docs/commands/{worktree,init}.md` 及对应 zh-CN 页、网站 EN 同步页，明确无名条目的命名/重新挂接、Windows 中断后标记恢复和 v0.30.83 源码构建版本。该修正不改变运行时行为，须重新同步、网站复审和验收。
**FIX-WD-08f 身份冲突恢复提示与日志边界：** 网站复审进一步指出 `repair --resolve-identity` 后仍有另一活跃条目占用相同身份，旧提示建议的立刻重新挂接必然失败；且旧 `reattach_worktree` 在拒绝该身份前已写入 Add intent，留下无需的未决日志。进一步审查发现旧提示建议的 `remove --delete-dir` 会按共享 worktree_id GC 幸存条目的 HEAD/reflog 等 scoped 行，构成数据丢失风险；仅检查活跃条目又会漏掉两条目均已分离的路径。将重新挂接的身份检查移至 intent 写入前；删除前若任何其他注册条目仍共享身份则拒绝，不动目录/数据。`src/command/worktree/operations.rs` 的所有 scoped GC 调用均传入 registry 快照和目标路径，拒绝对仍被其他注册条目占用的身份执行清理；`src/command/worktree/doctor.rs` 的旧未决 journal/tombstone 恢复同样 fail-closed。扩展既有 `resolve_identity_detaches_one_claimant_and_mirrors_the_lifecycle` 集成用例，让 registry 与工作目录真实共享身份，断言拒绝挂接不新增 journal、拒绝删除保持目录与幸存 scoped 行、两条目均分离后仍拒绝删除，并注入旧 Add intent 验证 repair 不清理共享数据。修正 `src/command/worktree.rs`/`src/utils/util.rs`/`src/command/worktree/registry.rs` 相关提示及双语/网站 Worktree 文档，明确保留文件并人工恢复，不许诺当前无安全路径的自动重新挂接/删除；不实施用户排除的旧数据迁移。写集增加 `src/command/worktree/{operations.rs,doctor.rs,registry.rs}`、`src/command/worktree.rs`、`src/utils/util.rs`、`tests/command/worktree_isolation_test.rs`、双语 Worktree 页、网站 Worktree EN 页、`CHANGELOG.md` 与本文；此处变更运行时写入顺序和拒绝条件，最终树需重跑 focused、fmt/严格 Clippy、双全量、最终 SHA CI 与独立复审。
**FIX-WD-08f lifecycle mirror follow-up:** `worktree repair --confirm` 的生命周期镜像以 worktree_id 为键，若先访问已分离条目再访问幸存活跃条目，原 Active 分支会删除共享键下的 detached 镜像行，令 down-migration 看不到仍在磁盘的分离目录。现只在该身份没有其他注册条目时删除镜像；集成夹具把分离条目放在前面并执行 repair，验证镜像持续可见。旧身份冲突没有自动迁移或自动清理出口，本版明确保留目录、继续使用幸存条目并待人工恢复。
**FIX-WD-08f 末端提示/镜像补充：** 除 `repair --confirm` 的 Active 分支外，普通重新挂接及旧 Add journal 前滚也不得删除仍描述另一已分离条目的共享 lifecycle 镜像。同步修订 `--resolve-identity` help/确认提示、工作目录 detached 提示、doctor 查验/旧意图提示、双语 Worktree 文档的 journal 说明和 CHANGELOG：身份仍共享时未决日志或 tombstone 可持续保留，不能承诺自动清理；保留旧目录并人工恢复，符合用户不迁移旧数据的边界。
**Docs and compatibility impact:** 原卡所有用户/网站文档逐文件结果登记本文；`0.30.85` release notes 和 upgrade 双语/网站页明确 NAME/--path breaking change、首次以支持新布局的二进制打开仓库即写入不可回退的 2026101902 receipt、`0.30.66` 及更早版本此后所有命令拒绝且不迁移旧数据。不把 `0.31.1` 写成已发布稳定版。
**Rollback mode:** immutable-release
**Migration and rollback:** 已发布版本不撤销或覆盖；失败发布新修复版本。旧二进制禁止写新布局；网站问题在 cf 前滚。
**Estimated scope:** M
**Version increment:** patch（用户明确指定覆盖原 minor 目标）
**Release boundary:** family release point of REL-WD01
**C/D coverage from:** self
**Granularity:** `type=release; axis=家族发布交付; recovery=不可变版本前滚发布; complete=yes; self-contained=yes; AC=10/12; VER=8/12; landing=0; prod-files=0（仅Libra生产代码，backend同步工具另列）; scope=M; deps=WD-07,DEP-WD-01,DEP-WD-02,DEP-WD-03,DEP-WD-04,DEP-WD-05; writeset=序列化于WD-07; release=family release point; split-from=N/A; exception=用户明确选择patch+1覆盖minor`

### 任务卡粒度审计表

| 任务 | type | axis | recovery | complete | self-contained | AC | VER | landing / prod-files | scope | deps | writeset | release | split-from | exception |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| WD-01 | implementation | 布局发现边界 | revert | yes | yes | 4/8 | 3/8 | 4/7 | M | DEP-WD-01 | 串行 WD-02G | family child | N/A | N/A |
| WD-02G | migration | 旧二进制能力门 | forward-only | yes | yes | 6/8 | 4/8 | 2/3 | M | WD-01,DEP-WD-04 | 串行 WD-01/02 | family child | WD-02 | N/A |
| WD-02 | implementation | 普通命名生命周期 | forward-only | yes | yes | 7/8 | 6/8 | 4/12 | M | WD-02G,DEP-WD-04 | 串行 WD-02G/03 | family child | N/A | N/A |
| WD-03 | implementation | FUSE 命名挂载 | forward-only | yes | yes | 7/8 | 4/8 | 4/6 | M | WD-02,DEP-WD-02/04 | 串行 WD-02/04 | family child | N/A | N/A |
| WD-04 | implementation | 任务物化 | forward-only | yes | yes | 6/8 | 4/8 | 4/9 | M | WD-03 | 串行 WD-03/05 | family child | N/A | N/A |
| WD-05 | implementation | review 生命周期 | forward-only | yes | yes | 6/8 | 4/8 | 2/4 | M | WD-04 | 串行 WD-04/06 | family child | N/A | N/A |
| WD-06 | implementation | investigate 生命周期 | forward-only | yes | yes | 6/8 | 4/8 | 2/4 | M | WD-05 | 串行 WD-05/07 | family child | N/A | N/A |
| WD-07 | implementation | checkpoint 物化 | forward-only | yes | yes | 6/8 | 5/8 | 4/6 | M | WD-06 | 串行 WD-06/08 | family child | N/A | N/A |
| WD-08 | release | 家族交付 | immutable-release | yes | yes | 10/12 | 8/12 | 0/0 | M | WD-07,DEP-WD-01..05 | 串行 WD-07 | family release point | N/A | N/A |

## 测试矩阵

| 类别 | 必须覆盖 | Target / command |
|---|---|---|
| 布局/发现 | main、subdir、linked、缺损元数据、symlink | `--lib worktree_layout`, `command_test::worktree_isolation_test` |
| 普通生命周期 | NAME/--path、并发/别名、重挂、失败不释放名字 | `command_test::worktree_test` |
| 工作文件 | add/-N/-p/冲突、HEAD/index 隔离、主树 clean、GC index-only 对象 | `command_test::worktree_isolation_test` |
| FUSE | feature/no-feature 名称占用、opaque、mount 失败、真实递归边界 | `command_test::worktree_fuse_test`, feature `worktree-fuse` |
| 任务 | 成功/失败/取消/崩溃、resume、clean、并行互不删除 | `agent_review_workflow_test`, `agent_investigate_workflow_test`, `command_test::agent_clean_test` |
| Checkpoint | 独立句柄、对象校验、readonly、cancel/deadline、no-follow | `--lib checkpoint_input` 与 review/investigate 集成 new filters |
| Sandbox | 显式根/缓存、metadata/兄弟路径拒绝、环境变量 | `--lib sandbox::runtime`, `--lib sandbox::policy` |
| 兼容 | v3 历史不重置、v4 旧二进制拒绝、JSON 可选 name | WD-02G/02/08 disposable repo 矩阵，worktree tests |
| 兼容 receipt | Repository-only、配置库不变、forward-only、旧 agent clean 拒绝 | worktree_layout_capability、db_migration_test |
| 原生 Windows | 分离句柄安全边界与 Windows filesystem 单元 | final-SHA compat-scoped-input-windows；crosscompile 不替代 |
| 文档/发布 | 双语/网站同步、版本/serial guards、线上页面 | WD-08 和全局网站必需门 |

## 追溯表

| 任务 | 来源/证据 | Libra 落点 | 文档/兼容动作 | 指定测试 |
|---|---|---|---|---|
| WD-01 | util:205/worktree_scope:75 | 布局、发现、遍历边界 | 开发 worktree 契约 | worktree_layout、named_worktree_defaults_use_common_storage_from_every_scope |
| WD-02G | 旧 0.30.53 agent clean 实验 | Repository receipt / role manifest | forward-only 能力门 | worktree_layout_capability、db_migration_test |
| WD-02 | worktree:98/add:2817 | 普通 CLI/registry/add | NAME/--path、v4/旧 binary | named_worktree_explicit_path_and_move_keep_logical_identity、named_worktree_nested_files_stage_and_survive_gc_while_metadata_stays_hidden |
| WD-03 | worktree-fuse:709 | FUSE/opaque/name lock | FUSE 布局与平台 | wd_fuse_live_mount_boundary、test_fuse_multiple_worktrees_mount_and_access |
| WD-04 | workspace_isolation:127 | 描述/意图/sandbox | sandbox 根语义 | wd_attempt_intent |
| WD-05 | review/runner:470 | review/clean | review/agent 命令页 | review_checkpoint_scoped、fix_rg_scoped_03_completed_review_clean |
| WD-06 | investigate/runner:727 | investigate/continue | investigate/agent 命令页 | investigate_checkpoint_scoped、fix_rg_scoped_03_completed_investigate_clean |
| WD-07 | checkpoint_input:228 | 安全物化句柄 | checkpoint 输入位置 | fix_rg_scoped_04_metadata_boundary、fix_rg_scoped_04_readonly_cleanup |
| WD-08 | 模板 ER-08/13/14 | 发布/部署/记录 | patch release notes，显著兼容变化 | 全量 nextest、FUSE 实机、线上内容 |

## 里程碑验收与回滚

| 里程碑 | 完成条件 | 发布/证据 | 回滚或前滚 |
|---|---|---|---|
| M0 | 正式计划通过审查、基线/工具固定 | 仅计划记录 | 无布局写入 |
| M1 | WD-01、WD-02G、WD-02、WD-03 本地验收 | 未提交不发布、无 complete 声明 | 当前轴前滚修复 |
| M2 | WD-04..07 本地验收 | 未提交不发布、无 complete 声明 | 保留意图和已有文件，前滚 |
| M3 | WD-08 全量/发布/部署/远端验收 | 唯一 REL-WD01 发布 | 不可变版本前滚 |

### 故障恢复矩阵

| 故障点 | 可接受残留 | 恢复动作 | 禁止结果 |
|---|---|---|---|
| 名称预留后创建失败 | 有归属的失败记录 | 锁下补偿仅本次创建资源 | 删除原有目录或释放仍占用名称 |
| FUSE health/opaque 失败 | 可恢复失败记录；若卸载失败保留 upper | bounded unmount，doctor 报告 | 装成成功或递归删除活动挂载 |
| 任务意图后进程崩溃 | provisioning 描述和局部目录 | 验证归属后恢复/清理 | 猜路径删除 |
| 工作目录清理失败 | sessions 与 cleanup-pending 描述 | 下次 clean 重试 | 先删元数据失去归属 |
| registry/身份损坏 | 原文件不变 | 明确诊断，显式合法 repair | 回落主树修改 HEAD/index |
| 发布/网站外部门失败 | immutable artifact/提交 | 记录 remote-pending，前滚修复 | 把本地通过写成计划完成 |

## 风险登记

| 风险 | 影响 | 缓解 | 任务 |
|---|---|---|---|
| common storage 排除误吞子树文件 | 高：静默漏暂存 | 本地 metadata 边界、多 add 模式集成 | WD-01/02 |
| 身份缺损向上发现 | 高：主树状态破坏 | 容器硬边界、fail-closed 测试 | WD-01/03/04 |
| FUSE lower 包含自身挂载路径 | 高：递归/资源耗尽 | opaque 全容器、实机验证 | WD-03 |
| 旧 GC/agent clean 忽略新布局 | 高：对象或运行归属丢失 | Repository receipt + v4 门、真实旧 binary 矩阵 | WD-02G/02/03/08 |
| clean 失去任务归属 | 高：错误删除或孤儿 | 持久化意图、cleanup 顺序 | WD-04..07 |
| 接口变更说明不足 | 高：调用方中断 | 用户指定 patch 发布，NAME/--path 兼容变化须在 release notes、升级说明与示例同卡同步 | WD-02/08 |
| 本地缺 FUSE/发布环境 | 验收阻断 | DEP-WD-02/03 不降级为 PASS | WD-08 |

## 性能与容量摘要

| 操作 | 单次成本 | 累积成本 | 预算/上限 | 验证 |
|---|---|---|---|---|
| NAME 查找 | O(登记条目数) | 不读取工作副本内容 | 沿用现有 registry 规模，禁止新全树 walk | 同名/并发测试 |
| 任务恢复/clean | O(所选 run/attempt) + 原有文件删除成本 | 已结束 run 可回收 | 沿用现有 deadline/取消预算 | 失败/并行测试 |
| copy | 原有文件树复制成本 | 每 attempt 独立 | 不复制 `.libra`/工作区容器，不因路径内嵌递归 | 物化边界测试 |
| FUSE | mount/health/unmount 原有 bounded timeout | 仅合法活动 mount | 保留 15s/5s/15s 现有预算 | timeout/opaque 门 |

## 兼容与文档收口

- [ ] COMPATIBILITY、worktree EN/zh/开发文档及网站同步。
- [ ] review/investigate/agent/sandbox EN/zh/开发文档及网站同步。
- [ ] docs/error-codes.md：当前 N/A，计划复用已有类别；新增码必须同卡扩写并验证。
- [ ] tests/INDEX、serial registry、nextest 配置按实际改动同步；无新 target 时 Cargo.toml [[test]] N/A。
- [ ] plan-status 与 plan-long 日期索引同步。
- [ ] 不新增迁移命令、不自动/批量补名、不搬旧数据的范围经实际 diff 复核；显式单条命名有身份校验。

## Codex review log

| Round | Scope | Result | P0/P1 | P2 处置 | Evidence |
|---|---|---|---|---|---|
| R2 (Claude formal) | 正式计划全文，no-tool review | FAIL | P1：共享 main 上中间本地提交可能被其他推送带出，release hold 不可靠；本轮改为全家族 dirty 未提交、root 唯一提交者、最终刷新/核对远端与写集 | P2：缓存布局、任务发现拒绝、旧 binary agent 路径、opaque 相对路径、tombstone、porcelain、网站命令、Windows 原生门、doctor focused、兼容表守卫；本轮逐项修订，待复审 | 根执行者读取的实际 R2 输出摘要；本行不是修复后 PASS |
| R4 (Claude formal) | R3 后的修订计划文字，no-tool review | FAIL | 所报旧版 CLI 可继续读写真实仓库的 P0 与实测相反：installed 0.30.53 的 `status` 和 `worktree list` 均在 DB open 以 future-schema exit128 拒绝；不能作为真实缺陷关闭记录 | 网站准备性写入、hooks 写集、真实 DB receipt 与 Windows 触发条件需要计划文本同步；已修订，待复审 | R4 无真实工具调用；旧版 CLI 的上述两条实测输出另见实施证据，不采信其自称执行命令的文字 |
| R5 (Claude formal) | 11:28 修订计划全文，明确无工具文字评审 | FAIL | P1：WD-02 增补 sandbox/mod.rs 后超出 12 个生产文件，已归回 WD-04 原写集；2026101902 上游撞号时可能静默跳过外来 DDL，已冻结 SQL hash 并加入 GC-WD-02a 的停止/协调/双 DB 前滚方案 | 补齐 upgrade 双语/网站页、Windows 新测试名称、旧 binary fixture 阶段、旧版拒绝点和完整测试命令；最新版本待复审 | `/tmp/libra-wd-claude-plan-r5.txt`；字面 `VERDICT: FAIL`，不能作为本轮 PASS |
| R6 (Claude formal) | R5 修订计划全文，明确无工具文字复审 | PASS | P0/P1 均为 0；确认 WD-02 的 12 个生产文件与撞号停止/双状态前滚方案闭合 | P2：核实 Windows filter、sandbox 提示验收、七页时间线、默认字段说明、候选 binary hash 与 backend 工具计数；实际 workflow filter 与七页限定同步已核验，其余实施收口时继续登记 | `/tmp/libra-wd-claude-plan-r6.txt`；字面 `VERDICT: PASS`，仅代表该版计划可执行，不代表源码或发布门通过 |

R1（Claude formal）使用 Read/Grep，max-turns 12 用尽，没有 verdict，不能记 PASS/FAIL 或当通过证据。R3 对修订版给出字面 `VERDICT: PASS`，P0/P1=0；其七项 P2 已记录为本轮收口修订要求，实施/发布仍各自需要独立审查。R2 的 shared-main 风险有实际行动项；输出中未获源码证据支持的行号/自动 rebase 危险假设不作为事实，本计划直接禁止自动 rebase/reset。会话设计已由 Claude Code 五轮审查最终字面 `VERDICT: PASS`（根执行者确认），仅作为设计决策来历；正式文件与实现必须分别再审；R3 修订版已完成正式计划审查 PASS；此后文字协调不替代最终源码评审。用户已授权完整计划，范围内常规实现选择由执行者直接决定，不新增用户审批关口。review 表只写实际 PASS/FAIL，不填虚构占位 PASS。

## 非目标与延后项

无 DEFER：用户明确排除旧数据迁移，不作为等待完成的计划债务；旧独立计划风险保持原归属，不虚构本计划接手。

在最终签提交之前逐卡把精确写集做受限权限的仓库外字节备份，作为恢复副本而非验收证据；最终暂存时比对写集，防并行提交/restore 覆盖。2026101902 已写入真实 Libra Repository DB，编号不得修改、删除或复用；任何后续修复都必须新增更高编号 receipt 并前滚。若 final SHA 的 main push 没有触发 Windows job，暂停 tag/release，使用仓库实际支持的同 SHA CI 路径补跑并记录 run ID，不能以无运行代替 PASS。

## 实施证据汇总

| UTC 时间 | 范围 | 事实与命令 | 结论 |
|---|---|---|---|
| 2026-10-19 07:51:24 UTC | 正式计划基线 | native main `85dcc5`、源码0.30.56/installed0.30.53；本轮 status clean；模板与路径只读核验 | 规划事实；无实现/测试/发布通过声明 |
| 2026-10-19 08:11:57 UTC | 旧二进制实验（根执行者） | installed 0.30.53 在 disposable repo：schema 2 review state/manifest，terminal 时间 2000，tasks workspace sentinel；agent clean --gc exit 0 | FAIL：删除运行元数据、遗留 workspace；触发 WD-02G。后续在隔离副本加入 2026101902 receipt 后，旧版 exit128 拒绝，manifest/state 字节与 workspace 哨兵均保留；当时真实工作库未升级 |
| 2026-10-19 08:16:02 UTC 后 | Claude formal R3 | 修订九卡全文、R2 逐项复核，无工具文字评审 | 字面 `VERDICT: PASS`；P0/P1=0，七项 P2 收口文字要求继续登记，不视为实现验收 |
| 2026-10-19 10:53:05 UTC | 真实工作库 Repository receipt | 从真实仓库 cwd 运行新版 `init --bare` 复现命令时，命令前置 Repository DB open 应用了 2026101902；只读 SQLite 查询 `schema_versions` 确认时间 | 旧 installed 0.30.53 此后拒绝真实仓库；已明确改用当前构建 CLI，未回滚或改写 DB；本次副作用不是测试断言通过证据 |
| 2026-10-19 11:05 UTC 后 | 网站基线/范围 | backend `cf` 起始 clean；全库同步检查发现 18 页漂移，当时五页属本计划、13 页既有无关；给同步器加入 `--only` 与合成夹具测试 8/8 | 后续 hooks 错误修复使当时六页属本计划、12 页仍为既有无关漂移；后来 upgrade 成第七页，须重新计算全库漂移；当时按六页写入、限定检查通过，不宣称全库检查绿 |
| 2026-10-19 11:28 UTC 后 | R4 事实复核与前滚边界 | installed 0.30.53 在真实 Libra 仓库执行 `status --short --branch`、`worktree list` 均报 schema 2026101902 newer than supported 2026101901，exit128；backend DB 只读查询 `MAX(schema_versions.version)=2026101901`、capability 2/3；`.github/workflows/base.yml` 对 main push 触发且包含 `scoped-input-windows` | R4 的旧 CLI 可读写真实仓库判断被实测否定；Libra 2026101902 不可改号，backend 待最终交付窗口；Windows D 仍须记录 final SHA/run ID/结论 |
| 2026-10-19 11:50 UTC 前 | 旧 0.30.53 完整消费者矩阵 | 隔离 HOME/config 的 disposable 新库由当前 CLI 创建提交并 `worktree add alpha` 于 `.libra/worktree/linked/alpha`；记录 71 个非瞬态文件 SHA-256 基线。旧 CLI 的 add/remove/move/prune、`repair --confirm`、maintenance GC、agent clean --gc、review --agent codex、investigate continue 共 9 项均在 DB open 以 future-schema exit128 拒绝，逐项文件 hash 集合一致。另对未带 `--confirm` 的 repair 预期先行拒绝也观察到零变更 | 旧版写保护矩阵通过；真实 task schema 2 state/manifest 的旧 clean 对照另见 08:11 实验；最终格式在 WD-08 复核，不能由本行替代发布验收 |
| 2026-10-19 11:50 UTC 前 | 计划复审与网站范围 | Claude R6 文字复审字面 `VERDICT: PASS`、P0/P1=0；网站 `--only worktree agent review investigate sandbox hooks upgrade` 七页检查 drift=0，同步器夹具 8/8；全库只读检查尚有 13 张其它页漂移 | 仅计划可执行和本轮七页映射门通过；网站 typecheck/build 与最终同步、源码审查/发布门仍待完成；其它页未写入 |
| 2026-10-19 15:35 UTC 前 | 最新上游与安全合流 | 源码原始 HEAD `85dcc5` 后，上游推进至 `2e917cdb`；精确暂存任务文件形成签名+DCO 本地快照 `7649eaa`，手工处理 11 处冲突并原生 rebase 成 `33ad715e`，未推送；backend `cf` 同理从 `78b7942` 合流到 `b77b548`，本地签名+DCO `5e7047ba`，未推送。源码上游又推进到 `262c700e`，尚待本轮测试结束后合流 | 这两个提交都是可审计的本地安全快照，不是最终发布 SHA；MCP 并发计划行须在最终提交前与本任务写集分离 |
| 2026-10-19 15:35 UTC 前 | A/B 与专项门 | postmerge 架构守卫 65/65、版本面守卫 2/2、serial 守卫 26/26、macOS `worktree-fuse` 命令组 15 passed/1 ignored；OrbStack Linux 容器以 `/dev/fuse` 真挂载运行 `wd_fuse_live_mount_boundary` 1/1，通过 `wd_move_cross_device_refuses_without_copy` 1/1；`cargo +nightly fmt --all --check` 与 `cargo clippy --all-targets --all-features -- -D warnings` 通过 | 真实 FUSE、跨设备拒绝和静态门已验证；后续源码合流/改动须重跑受影响门。先前一次全量在 config ownership 缓存表与并行 cwd/env 测试失败，分别前滚修复并 focused 通过；本轮 `cargo test --all` 正在重跑，不能记绿 |
| 2026-10-19 15:35 UTC 前 | 网站与独立评审 | backend 七页限定检查和全库只读同步检查均 drift=0，同步器夹具 8/8、Vitest 11/11、`pnpm typecheck`、`pnpm build` 通过；源码 Claude 的跨设备/FUSE、registry/Windows/DB、task/sandbox/doctor、Windows/docs 与 postmerge capture/DB 分区复审各有字面 `VERDICT: PASS` | 这些 PASS 只覆盖对应版本与写集；最终 `main` SHA 与 Windows CI、release、网站部署仍未完成；最终修订需再审 |
| 2026-10-19 15:54 UTC 前 | postmerge `cargo test --all` 第二轮 | lib 4528/4528、`command_test` 4420/4420 通过，之后 `compat_agent_docs_contract` 33/34、1 失败：中文 agent 文档原句“稳定错误码不变”未保留测试锁定的字面“不新增稳定错误码”；该轮 exit 101，绝不记全量通过。改为明确“不新增稳定错误码”后，`source .env.test && cargo test --test compat_agent_docs_contract` 34/34 通过 | 文档契约的 focused 修复已通过；最终合流后仍须完整全量重跑 |
| 2026-10-19 16:40 UTC 前 | 最新源码合流、独立差异复审 | 原生 rebase `380abd3` 后本地签名+DCO 快照 `4d8cc20`，新上游 Unix 测试 focused 1/1；Claude 补审实际 132 行 patch 首轮 `VERDICT: FAIL`：`plan-status.md` 三个与本任务无关的历史数值被旧快照带回。逐项按 `origin/main` 和原计划恢复后，补审 159 行完整大文件差异与更正的状态差异，字面 `VERDICT: PASS`（`/tmp/libra-wd-claude-final-delta-followup.txt`）。上游随后仅推进计划文档到 `cbee959c`，尚待最终合流 | 三项历史值修复保留在未提交差异，最终提交须精确暂存；不将初轮 FAIL 记为 PASS，不把上游计划文档变动当 worktree 功能修改 |
| 2026-10-19 16:40 UTC 前 | 最终树 Linux 专项与网站 | OrbStack privileged Linux `/dev/fuse` 真挂载 `wd_fuse_live_mount_boundary` 1/1、跨设备移动拒绝 `wd_move_cross_device_refuses_without_copy` 1/1（`/tmp/libra-wd-linux-final-rebased.log`）；backend 合流至 `214f1e1` 后本地签名+DCO `23533e9`，七页 `--only` 检查 drift=0，全库只读漂移 13 页均为并行 #478 页。同步器 8/8、Vitest 11/11、typecheck/build 通过；网站 Claude 最终 scoped 复审字面 `VERDICT: PASS` | `agent` 页中并行提交提前宣称 Grok 生产 Thinking 采集的段落被当前源码权威文本纠正；其余 13 页未改。网站尚未推送/部署 |
| 2026-10-19 16:40 UTC 前 | final-rebased `cargo test --all` 第三轮 | lib 4528/4528，`command_test` 4419/4420；失败 `get_best_effort_reads_value_inside_repository` 在 setup 阶段误读另一测试的 global config 未来收据 2026101903，exit101；隔离单测重复 20 次均通过。根因是本用例只持有 `cwd` lane，而 `global_config_create_uses_global_role` 通过进程环境设置 future-schema 配置库。用 `ConfigDbFixture` 显式隔离配置路径并把 lane 扩为 `cwd+env`，分类器判定一致、serial registry 26/26、focused 1/1、config 测试组 184/184 通过 | 第三轮全量仍为 FAIL，须在最新树上重跑全量至绿；串行 registry 与源测试改动纳入本轮写集和最终复审 |
| 2026-10-19 16:50 UTC 前 | FIX-WD-02 生产者侧复核 | Claude 首轮仅审消费者修复，字面 `VERDICT: PASS` 但 P1 指出 future-schema 配置测试仍只持有 `env` lane。将 `assert_configuration_role_writer` 的整个注入窗口放入 `ChangeDirGuard`，并使 global/system 两个调用者均持有 `env+cwd`；分类器三例 lane 与注册表一致，生成的 nextest 配置无内容变化。修改后 config 组 184/184、serial registry 26/26、fmt 与严格 clippy 通过；Claude 跟进复审及最终双全量仍待 | 保护读者与写者两侧；没有修改生产配置代码；原第三轮失败仍保留 |
| 2026-10-19 22:28 UTC 前 | 最终 macOS 与 Linux 双全量及安全矩阵 | `e37e693` 直接基于上游 `17a6766`，签名+DCO；`cargo +nightly fmt --all --check`、全目标全特性严格 clippy 通过；`source .env.test && RUST_TEST_THREADS=4 cargo test --all` exit0，182 个目标合计 10485 passed/0 failed/26 ignored；`source .env.test && source .env.live-test && cargo nextest run --all --no-fail-fast --retries 2` exit0，10465/10465 passed、46 slow、23 skipped，无重试。旧 0.30.53 对九项消费者均以 schema 2026101902 exit128 拒绝，69 个稳定文件哈希不变。最终树 OrbStack `/dev/fuse` 真挂载 1/1，跨设备移动拒绝 1/1；初次筛选为零项或 ignored 的命令未计入通过 | 本地双全量、旧版拒绝和真实 Linux FUSE 门通过；后续 Windows 专有修复不在 macOS 编译路径，仍需最终 SHA Windows 原生门 |
| 2026-10-19 22:31 UTC 后 | 源码/网站发布前远端与线上页面 | 功能提交 `e37e693` 推送 main；backend `cf` 签名+DCO `f143ebd` 推送后，Wrangler 部署版本 `c8678629-fc28-4d17-a9c6-45a901c51acf` 占 100%。网站限定七页同步 drift=0，脚本 8/8、Vitest 11/11、typecheck/build 通过；线上 EN/zh 的 worktree/review/investigate 六页 HTTP 200，分别检出 linked/fuse/tasks 与 checkpoint 新路径；网站 Claude scoped 字面 `VERDICT: PASS` | 网站部署与线上内容门通过；CLI 标签/稳定 manifest 仍待最终 Windows 门和发布流程 |
| 2026-10-19 22:59 UTC 前 | 原生 Windows 失败与前滚修复 | CI run `39335102748` 在 `e37e693` 的 Windows test 编译因测试局部 `identity` 遮蔽函数而失败；测试专用签名+DCO修复 `a84f972` 后 run `39335967323` 的四个 exact fixture 通过，但 17 个 `windows_fs` 测试为 13 passed/4 failed，四例 Windows sharing violation 32。新增可变目录句柄只开放 READ/WRITE 共享，保留 DELETE 拒绝与身份核对；收紧的 `open_mutable_tree/open_mutable_at/exclusive_create_mutable_directory_at` 接入 task workspace，并将 Windows CI 扩为全部 11 个 workspace isolation 测试，签名+DCO `ec77730` 已推送。macOS 同组 11/11、fmt/严格 clippy 通过；Claude 首轮审查 `VERDICT: FAIL` 的调用方/宽共享 P1 经逐项核对和改动后跟进字面 `VERDICT: PASS` | 两次失败 CI 不计 PASS；run `39338632413` 的最终 SHA Windows 门进行中，未打 release tag |
| 2026-10-20 06:39 UTC 前 | #479/#480 上游合流后复验 | 远端 `main` 推进到 `e24c2d47fd37af6311b2e847aeeaba8f1c2d7c11`（#479 命令实现与测试、#480 计划收口）；本轮 15 路径补丁本地签名+DCO `52874fe` 经原生 rebase 成 `6003a0c96990f59b2df5e52ec92f14d59a864571`，父提交确为 `e24c2d4`，未夹带 MCP 未提交计划。合流树 fmt/严格 Clippy、Worktree 隔离 120/120、版本面 2/2；`source .env.test && RUST_TEST_THREADS=4 cargo test --all` exit0：182 targets、10538 passed、0 failed、26 ignored。OrbStack privileged `/dev/fuse` 真挂载 `wd_fuse_live_mount_boundary` 1/1、跨设备移动拒绝 1/1；初试缺 `LIBRA_RUN_FUSE_TESTS=1` 与 `fusermount3` 的两次失败保留且不计通过。网站七页限定同步 drift=0。合流树 nextest 正在运行 | 远端 #479 代码不沿用旧全量；新树的 cargo 全量、Linux 真门已通过，仍须 nextest/最终原生 Windows CI 与发布 |
| 2026-10-20 07:11 UTC 前 | 合流树发布模式与旧版拒绝复验 | `6003a0c96990f59b2df5e52ec92f14d59a864571` 已推远端 `main` 且远端 SHA 相同，base run `39384504690` 已启动。合流树 `cargo build --locked --release --features keyring` exit0；优化二进制以平台式原文件名运行 version/init/add/commit、默认 `.libra/worktree/linked/alpha`、显式外部 `--path`、list/remove 冒烟 PASS。隔离 disposable 库上的实际 0.30.53 对 worktree add/remove/move/prune/repair、maintenance GC、agent clean、review、investigate continue 共九项均在 DB open 以 future-schema exit128 拒绝，69 个稳定文件 SHA-256 每步不变，日志 `/tmp/libra-wd-v0311-merged-old-binary-matrix.log`。backend `cf` 曾以 `580006b408a22cba4841cd131f271ccaa3a2630e` 将三页英文命令文档版本下限更新至 0.31.1；它随 `cf` 推送后，最终网站内容已由 `44e9914` 恢复成 0.30.66 稳定版语义，不得单独重推/部署 `580006b` 的未来文档 | 旧版兼容拒绝与本地 release 模式通过；新布局正式资产、stable 和网站验收仍未完成 |
| 2026-10-20 07:18 UTC 前 | 最终 SHA Windows 集成编译阻断 | base run `39384504690` 的 `compat-scoped-input-windows` 在四个 exact fixture、`windows_fs` 17/17、capture key 2/2、workspace isolation 10/10 后，在新发行文件名集成测试的非测试库编译遇到 `-D dead-code`：`src/utils/windows_fs.rs::read_directory_names` 只由同模块 `#[cfg(test)]` 测试使用。该 job 明确 `failure`，不计 D 门通过。将该测试专用包装函数标记 `#[cfg(test)]`，生产调用的 `read_directory_names_bounded` 保留；需新 SHA 原生 Windows 作业实证 | Windows D 门未通过，修复提交与整套新 SHA CI 待验 |
| 2026-10-20 07:23 UTC 前 | Windows 集成编译前滚 | 仅 `src/utils/windows_fs.rs` 增加一行 `#[cfg(test)]`，fmt 与严格全目标全特性 Clippy exit0，签名+DCO `426f446722724d2a2741c7ecf1f9be7ffea1c0d5` 已推远端 `main`，base run `39386086979` 绑定此 SHA。旧 run `39384504690` 因已知 Windows 失败请求取消，不把其余进行中的作业当作通过。合流树先前的 nextest 10518/10518 passed、56 slow、1 flaky（第 2/3 次成功）、4 leaky、23 skipped；补丁只改变 Windows 测试专用函数编译，最终 SHA 本地双全量另行重跑 | 等最终 SHA 本地双全量、新 Windows 及整套 CI，发布仍阻断 |
| 2026-10-20 07:36 UTC 前 | Windows 聚合测试编译范围前滚 | run `39386086979` 的四个 exact fixture 成功；`windows_fs` 17/17、capture key 2/2、workspace isolation 10/10 通过后，整块 `command_test` 的 Windows 编译出现 33 个与本功能无关的测试源码错误（含 `status_wave0_test.rs` Unix-only `Permissions::from_mode`、`add_test.rs` unused variable、`config_test.rs` unused mut），新发行名测试未执行到，Windows job 为 failure。为给发布回归独立的跨平台编译边界，将该测试移至 `tests/release_binary_name_test.rs`、在 `tests/INDEX.md` 注册并使 Windows CI 只编译此 top-level target；本机该独立测试 1/1、actionlint 与严格 Clippy 通过。run `39386086979` 已请求取消；该轮最终树 `cargo test --all` 因测试目标移动而主动中断（exit130），不可计绿，新最终树须重新全量 | Windows D 门仍待独立 target 实际 Windows 验证，双全量重跑待做 |
| 2026-10-20 07:44 UTC 前 | 独立发行名集成目标落地 | 四路径精确暂存并签名+DCO `a63bb185af0dde891aef85fc931028eefa2315d9` 推远端 `main`；base run `39388454368` 绑定此 SHA。`tests/release_binary_name_test.rs` 独立 1/1，原 Worktree 隔离组移除该用例后 119/119；fmt、严格 Clippy、actionlint 通过。Claude Code 首轮对未展示完整暂存状态返回 `VERDICT: FAIL`，补入 staged blob、Cargo 自动发现、旧引用零命中与生产 fail-closed 证据后跟进字面 `VERDICT: PASS`；新目标 Linux/macOS/Windows 文件名跟发行资产匹配。最终 SHA `cargo test --all` 重跑已启动，nextest、Windows/整套 CI 仍待 | 不沿用两轮失败 CI 或被中断的本地全量；发布仍阻断 |
| 2026-10-20 07:51 UTC 前 | 最终树 Linux 实设备门 | `a63bb185af0dde891aef85fc931028eefa2315d9` 的 OrbStack privileged Linux 容器挂载 `/dev/fuse`，带 `LIBRA_RUN_FUSE_TESTS=1` 与 `--ignored --exact` 实跑 `command::worktree_fuse_test::wd_fuse_live_mount_boundary` 1/1，打印实际 `.libra/worktree/fuse/wd-live/workspace`；跨设备拒绝 `command::worktree_test::wd_move_cross_device_refuses_without_copy` 1/1，日志 `/tmp/libra-wd-v0311-a63-fuse-linux.log`、`/tmp/libra-wd-v0311-a63-cross-device-linux.log` | 最终源码 SHA 的真实 FUSE 与跨设备安全门通过；Windows CI、双全量与发布仍待 |
| 2026-10-20 07:55 UTC 前 | 原生 Windows 运行时操作锁阻断 | run `39388454368` 四个 exact fixture、`windows_fs` 17/17、capture key 2/2、workspace isolation 10/10 均通过；独立发行名目标实际运行 `add` 时失败，`operation-v2-repository.lock` 的 `try_lock` 返回 Windows Access denied (os error 5)，故该 job 明确 failure。源码 `src/internal/operation/middleware/lease.rs::open_repository_files` 的非 Unix 分支只用 `.append(true)` 打开文件；Rust `File::try_lock` 文档明确 Windows 的仅 append 句柄不可锁定。前滚改为复用已有 `open_leaf` 的 read/write、禁止 reparse-point 跟随路径，新增 Windows 直接锁定单元用例与 CI filter；本轮本地全量因生产代码改变主动中断 exit130，不能计通过。run `39388454368` 已请求取消 | 这是原生 Windows CLI 写入的真实生产缺陷；新 SHA Windows 单元+发行名 E2E 与双全量仍待，发布阻断 |
| 2026-10-20 08:13 UTC 前 | Windows 仓库锁前滚与最终树实设备复验 | `src/internal/operation/middleware/lease.rs` 的 Windows/non-Unix 仓库锁打开方式复用已有安全 `open_leaf`，增加 Windows `repository_lease_file_can_be_locked` 单元测试，CI 新增其原生 filter。签名+DCO `141de9ab49e10d18c03d765c9ee1728bf97c4afa` 已推远端 `main`，base run `39391000275` 绑定该 SHA；本地 lease 21/21、发行名 1/1、fmt/严格 Clippy/actionlint exit0。最终 SHA OrbStack 实际 FUSE 挂载 1/1、跨设备移动拒绝 1/1，日志 `/tmp/libra-wd-v0311-141-fuse-linux.log` 与 `/tmp/libra-wd-v0311-141-cross-device-linux.log`。本地全量重跑中；首次 Claude scoped 审查无输出超过 8 分钟被终止，不计 PASS，限时复审待结果 | Windows 原生运行时与双全量/整套 CI 尚待；不提前打 tag |
| 2026-10-20 08:17 UTC 前 | 最终 SHA 原生 Windows D 门完成 | base run `39391000275`、SHA `141de9ab49e10d18c03d765c9ee1728bf97c4afa` 的 `compat-scoped-input-windows` job `success`；四个 active exact fixture 各 1/1，`windows_fs` 17/17、capture key 2/2、workspace isolation 10/10、仓库锁 `repository_lease_file_can_be_locked` 1/1、独立发行文件名 `renamed_release_binary_can_start_status_io_worker` 1/1。真实 `add` 不再报 operation lease Access denied，日志 `/tmp/libra-wd-v0311-141-windows-success.log`。Claude Code 第一次 scoped 审查指出未证实的 append/强制锁假设并字面 FAIL；代码检索确认 lock 文件仅作 private `ScopeLease` 句柄、不写内容、无第二句柄读取或清理后，跟进审查字面 `VERDICT: PASS`，仍以原生 CI 为决定性运行证据 | Windows D 门已通过；其余 CI、双全量、release/stable/线上仍待 |
| 2026-10-20 08:30 UTC 前 | 网络远端测试读取范围前滚 | run `39391000275` 的 `compat-network-remotes` 中公开 GitHub refs 与 LFS 用例通过，`https_upload_pack_returns_pack_data` 在读取整份持续增长的公开 Libra pack 时，约 65 秒后 reqwest body `TimedOut`，该 job 为 failure。测试本来只断言 `PACK` 标记；`tests/network_remotes_test.rs` 改为最多读取 1 MiB 响应前缀并在见到标记时结束，避免完整 clone 规模/网络耗时干扰该线协议冒烟。本机该用例实网 1/1、4.60 秒通过；全网络目标继续验证。最终 SHA 当轮本地 `cargo test --all` 中 `add_test::test_add_lock_timeout_names_foreign_holder` 的子 helper `flock` 首次失败，而同一父测试隔离重跑 1/1 约 12 秒；该全量在已知失败且新增网络测试改动后主动中断 exit130，绝不计绿，需独立归因/重跑。整套旧 run 后续取消，网络测试前滚需新 SHA CI | 双全量与网络远端作业均待新 SHA 验收；不因隔离重跑而宣称本地全量通过 |
| 2026-10-20 08:38 UTC 前 | 有界 Git pack 头检查交付 | 在 1 MiB 上限内匹配 Git pack 规范的完整 `PACK` + network-order v2/v3 头，避免裸 `PACK` 文本假阳性；完整 `--features test-network --test network_remotes_test` 本机实网 3/3、upload-pack 约 5 秒，fmt 与严格 Clippy 通过。Claude Code 初轮 `VERDICT: PASS` 但提出裸签名 P2 建议；加固后跟进字面 `VERDICT: PASS`。测试单文件签名+DCO `cc8f43769fdae215357e52c3509cbb71126dad56` 已推远端 `main`，base run `39394787003` 绑定该 SHA；已知网络失败的旧 run `39391000275` 请求取消。本地最终树用 `RUST_TEST_THREADS=1 cargo test --all` 重新跑以定位并发锁测试，nextest 后续单独运行 | 最新 SHA 网络/全量/整套 CI 仍待；前轮 Windows 成功仅作补充，不冒充新 SHA 门 |
| 2026-10-20 09:19 UTC 前 | 当前 Worktree 源码 SHA 整套 CI | `cc8f43769fdae215357e52c3509cbb71126dad56` 与远端 `main` 相同；base run `39394787003` 八作业全部 `success`，包括原生 Windows、macOS owner、严格 Clippy、fmt、离线全量及网络远端。网络作业实跑 3/3，曾超时的 `https_upload_pack_returns_pack_data` 本轮约 7 秒通过；本地单线程 `cargo test --all` 与后续 nextest 仍独立验收 | CI 门通过；本地双全量、发布资产、stable 与网站上线尚不可记完成 |
| 2026-10-20 10:53 UTC 前 | 当前 Worktree 源码本地双全量 | 源码 SHA `cc8f43769fdae215357e52c3509cbb71126dad56` 上 `source .env.test && RUST_TEST_THREADS=1 cargo test --all` exit0：183 个目标 10538 passed、0 failed、26 ignored，`command_test` 4470/4470。`source .env.test && source .env.live-test && cargo nextest run --all --no-fail-fast --retries 2` exit0：10518/10518 passed、27 slow、1 flaky、1 leaky、23 skipped。flaky 为 `fetch_test::test_fetch_into_mirror_updates_and_prunes_all_refs` 首次 status I/O worker 错误、第二次通过；leaky 为 `subagent_content::tests::unregistered_host_disables_unbounded_discovery`。此前并发 `add` 锁 helper 失败的中断轮次仍保留，不借隔离复跑充数 | 当前代码树双全量通过并如实记录重试/泄漏；用户的发布冻结仍阻断 WD-08 C/D |
| 2026-10-20 15:30 UTC 前 | macOS/macFUSE 实机补验与前滚修复 | 本机 macFUSE 5.4.0 实际挂载揭示旧 opaque xattr 对 macOS 的 OCI whiteout 不生效，初次 live gate 以递归遮蔽拒绝失败。`src/utils/fuse.rs` 现用 pinned 目录 fd 排他建立 `.wh..wh..opq`，Linux 保持 xattr；真实 mount→I/O→兄弟与自身目录遮蔽→unmount 通过。macOS 真实整组初轮 15/16，剩余分支创建用例揭示 restore 使用主工作区路径；`src/command/restore.rs` 只把写入根定向到新挂载点，索引、对象和分支来源仍锚定调用者。加强回归使目标分支来自较早提交，主工作区有较新提交及未提交改动，证明 FUSE 内容、源文件与源索引隔离。最新 `LIBRA_RUN_FUSE_TESTS=1 RUST_TEST_THREADS=1 cargo test --features worktree-fuse --test command_test worktree_fuse_test -- --include-ignored` 16/16、普通 restore 47/47、Worktree 单元 157/157；无残留 macFUSE mount。失败轮日志保留，不计通过 | WD-03 平台实机门本地通过；该树的严格 Clippy、全量、独立审查与冻结解除仍待，九卡保持 0/9 |
| 2026-10-20 18:05 UTC 前 | WD-03 macFUSE/Linux 双平台最终本地门 | 本地未提交树上 macFUSE 5.4.0 与 OrbStack privileged `/dev/fuse` 均以 `LIBRA_RUN_FUSE_TESTS=1` 实跑完整 `worktree_fuse_test --include-ignored`，各 16/16、无残留挂载；后者 Linux 容器源码只读挂载、独立 Cargo target，分支恢复与 live opacity 亦通过。新无挂载默认单元 3/3、普通 restore 47/47、Worktree 单元 157/157、serial registry 26/26；`cargo +nightly fmt --all --check` 与全目标全特性严格 Clippy exit0。`source .env.test && RUST_TEST_THREADS=1 cargo test --all` exit0：175 个结果组（含文档测试）、10533 passed、0 failed、26 ignored，`command_test` 4470/4470。`source .env.test && source .env.live-test && cargo nextest run --all --no-fail-fast --retries 2` exit0：10521/10521 passed、18 slow、4 leaky、23 skipped、无失败/重试。Claude Code 差异审查先以未进默认 CI 的回归给出 FAIL，补上无挂载单元、确认现存 root 提前拒绝重复准备；额外核对发现旧空目录清理可越过显式目标边界，改为根内清理并拒绝相对 pathspec/索引写入，最终只读复审字面 `VERDICT: PASS`。原始失败日志保留，不借隔离通过覆盖 | WD-03 本地代码、测试与复审门通过；该未提交树的远端 CI、WD-08 发行、生产网站新布局均未执行，九卡正式 0/9，用户发布冻结继续 |
| 2026-10-21 02:30 UTC | 上游版本前滚与旧改动合流 | `origin/main` 从本地 `5f0e808` 前进 26 提交到 `e8d530e`，版本面 `0.30.77` 且有 `v0.30.77` tag，`v0.30.78` 不存在；GitHub Releases 列表当前最新仍为旧 `v0.30.66`。已将九个未提交文件逐字节备份于 `/tmp/libra-wd-preserve-20261021-ic23xzky`，stash 保存后快进 main；stash apply 对 `worktree-fuse.rs`/`plan-status.md` 安全拒绝且未留下部分改动，随后手工将七个无上游变更文件与两个重叠文件合流。后者保留上游 `force`/remove 接口参数及其它计划状态行。当前候选版本面前滚为 `0.30.78`，升级/兼容文档纠正能力门实际从 `v0.30.67` 开始；`v0.30.67` tag 含 2026101902 SQL，旧 `17a6766` 不含。`libra diff --check` exit0 | 合流/版本与文档准备中；最终树测试、Claude Code 复审、main CI、发行与网站验收仍待，九卡正式 0/9 |
| 2026-10-20 09:53 UTC | 用户更改发布目标为 `0.30.66` | 用户明确要求发布 `0.30.66`、不要 `0.31.1`。远端既有 `v0.30.66` tag 指向旧源码 `17a6766cf14547c696afdd9269a13091720899f3`，既有 release run `39347223159` 八作业成功，签名 stable manifest 当前版本 `0.30.66` 且四平台资产均列出。以该不可变旧 tag 补建并公开 GitHub Release，`https://github.com/libra-tools/libra/releases/tag/v0.30.66` 为 Latest、非 draft/prerelease，公网 HTTP 200；没有重指标签或变更 stable。该旧源码不含本计划的 Worktree 新布局，故此发布**不计** WD-08 C/D 门，`v0.31.1` tag 与网站新增 0.31.1 说明均不发布 | Worktree 实现仍在 `cc8f437`，本地验证继续；WD-08 与九卡收口等待另行确定可发布版本，不借用 `v0.30.66` 证据 |
| 2026-10-20 10:24 UTC 前 | 生产网站文档恢复到稳定版 | 已部署的网站七页命令文档原先展示未发布的 Worktree 新布局；在 backend `cf` 只改 worktree/agent/review/investigate/sandbox/hooks/upgrade 七页，以源码 `17a6766` 的只读归档运行全站命令文档同步检查 drift=0。同步脚本单测 8/8、文档 Vitest 11/11、typecheck/build 通过；广义 `pnpm test` 仅 259/265，6 例失败在未改动的 credits、email、validators 测试，保留红色结果。Claude Code 只读复审字面 `VERDICT: PASS`。签名+DCO `44e9914e40f63cef9789b53fc6fb94ec826e8d6a` 已推 backend `cf`，Cloudflare 版本 `8d16d31f-32b9-4c8e-a53b-ea8ccfae67d0` 已部署；EN/zh-CN 七页共 14 个线上 URL 均可访问，旧 `worktree add <path>` 语法恢复且无 `.libra/worktree/linked/`、`.libra/worktree/tasks/` 或 `0.31.1` 文案。远端 `cf` 等于本地 HEAD、工作区干净 | 网站只说明现行 `0.30.66`，本计划新增布局仍未上线；这不计 WD-08 网站新功能验收 |
| 2026-10-20 05:40 UTC 前 | `0.31.1` 本地最终门 | `cargo +nightly fmt --all --check`、严格全目标全特性 Clippy exit0；`cargo build --locked --release --features keyring` exit0，优化构建以 `libra-wd-v0.31.1-darwin-arm64` 原名运行 Worktree 默认/外部路径/list/remove 冒烟 PASS；`worktree_io` 27/27、Worktree 隔离 120/120、版本面 2/2、重命名资产回归 1/1。`source .env.test && RUST_TEST_THREADS=4 cargo test --all` exit0：182 targets、10486 passed、0 failed、26 ignored。`source .env.test && source .env.live-test && cargo nextest run --all --no-fail-fast --retries 2` exit0：10466/10466 passed、52 slow、2 leaky、23 skipped、无 flaky/重试；leaky 为 agent import 进程内状态及 GPG 子密钥绑定两例。网站七页限定同步 drift=0、脚本 8/8、Vitest 11/11、typecheck/build exit0。Claude 修复源码审查与最终 Windows CI/文档差异审查分别字面 `VERDICT: PASS`，另一次宽范围审查耗尽 18 turns 无 verdict，不计通过 | 本地双全量和发布模式修复验收通过；Windows final-SHA CI、全平台发布与 stable/线上验收仍待 |
| 2026-10-20 04:02 UTC 前 | 发布资产冒烟阻断与前滚 | `v0.31.0` release run `39364820008` 的 macOS 资产 hash 与 sidecar 一致，但以公开资产文件名 `libra-darwin-arm64` 运行时 `init` 可成功、`add` 报 `status helper is not available in this executable`。同一字节重命名为 `libra` 后默认 linked、显式 `--path`、list/remove 全通过。根因 `src/internal/worktree_io/executor.rs` 以 basename 判断是否为 CLI，忽略 main 入口事实；Windows 尚在构建时已请求取消该 release，最终 run cancelled，stable manifest 保持 `0.30.66`，未发布该版本的 Homebrew/manifest/install 脚本。前滚修复改为 main 显式注册 CLI helper 能力，新增重命名二进制真实 `add` 回归；版本和安装/升级文档前滚至 `0.31.1`，网站准备性同步三页，发布须使用全新 tag | `v0.31.0` 只保留签名 tag 和已上传不可变资产，绝不计作稳定发布或 WD-08 完成；FIX-WD-03 本地 focused 及完整发布门待验 |
| 2026-10-20 03:32 UTC 前 | 最终 SHA 整套 CI 与标签 | `ec77730a43efab02ed47462f31d1d20226796b9e` 的 base run `39338632413` 八作业全部 success；offline-core 的 L1+L2+L3 nextest 10494/10494 passed、28 slow、21 skipped，`init_from_git_rejects_sha256_object_format` 首次失败后第 2 次重试成功，明确 1 flaky；OTLP 2/2、keyring 1/1、upgrade 17/17+12/12。远端 `main` 后续只新增两笔其它计划文档提交，已验证源码 SHA 不变。已创建本地签名验证通过的 annotated tag `v0.31.0`，远端 tag object `3368f5b46a8247f56edac2f263696bb7cdbd0819` 剥离后指向 `ec77730`；release run `39364820008` 已触发 | 最终 SHA CI 全绿，flaky 原样记录；release 产物、stable 和已发布 CLI 尚待验收，不把 tag 推送当发布完成 |
| 2026-10-19 23:07 UTC 前 | 最终 SHA 原生 Windows 门 | CI run `39338632413` 的 `compat-scoped-input-windows` 绑定 `ec77730a43efab02ed47462f31d1d20226796b9e`，四个 exact fixture 各 1/1，`utils::windows_fs::tests::` 17/17、Windows capture key 2/2、`workspace_isolation::tests::` 10/10，作业 `success`；工作流的 rustfmt、clippy、redundancy 与 Linux OpenCode sandbox 作业亦已通过，offline-core/network-remotes/owner-liveness 当时仍在运行 | Windows D 门通过；仍需整套 CI 与 release/tag/artifact/stable 证据，不沿用前两轮失败结果 |

| 2026-10-21 03:18 UTC | `0.30.78` 合流树本地定向门与独立审查 | 上游 `main=e8d530e`、`v0.30.78` 仍未占用。最终树 `cargo +nightly fmt --all --check` 与全目标全特性严格 Clippy exit0；macFUSE 5.4.0 真机及 OrbStack Linux `/dev/fuse` 分别实跑完整 FUSE 组各 16/16，无残留挂载。显式 restore 目标单元 4/4、MCP 执行目标 9/9；初轮 `cargo test --all` 在新增上游 MCP 用例因 macOS 无 `/proc/<pid>` 误判而失败，前滚为 Unix `kill(pid, 0)` / Windows `OpenProcess` 存活探测并仅在 Unix 执行 stdout-holder 用例，最终全量仍在运行，不能借定向复跑计绿。官方 CDN `0.30.53` macOS binary 的 SHA256 与 sidecar 一致；隔离新布局仓库 2026101902 receipt 存在，旧版对 worktree add/remove/move/prune/repair、maintenance GC、agent clean、review、investigate continue 九项均 exit128 future-schema，73 个稳定文件逐项 hash 不变（`/tmp/libra-wd-0.30.78-old-binary-matrix.json`）。网站七页限定同步 drift=0、脚本 8/8、Vitest 11/11、typecheck/build exit0；全站只读检查另有 20 个未纳入本次写集的既有漂移。当前差异 Claude Code 只读独立审查字面 `VERDICT: PASS`，无 P0/P1/阻断 P2 | 本地 FUSE/旧版/静态/网站与审查门通过；全量、release build、final-SHA CI、tag/资产/stable 与网站部署尚待，九卡仍 0/9 |
| 2026-10-21 05:47 UTC 前 | 最终全量兼容守卫前滚 | 基于 `b03fb93` 的 `cargo test --all` 中库测试 4582 passed/9 ignored、`command_test` 4521/4521 passed，随后 `compat_matrix_alignment` 9/11、2 failed，进程 exit101：守卫把新增的第二个 macOS `run` 步骤当成第一个步骤的 shell 行。修复 `tests/compat/matrix_alignment.rs` 为按具名步骤截取且接受 YAML 空行，并用私有 Cargo stub 验证新增五 fixture 的 pass/zero/wrong/ignored/nonzero；定向目标 12/12 通过，`actionlint` exit0。随后上游 `main` 快进至 `3471175`，仅新增 add 文档、计划与 clone 测试，28 个本任务文件仓外备份后原生快进并逐文件 SHA-256 28/28 恢复 | 失败全量绝不计绿；最终 `3471175` 树仍须重跑 fmt、Clippy、cargo 全量与 nextest，补测新增 clone 用例 |
| 2026-10-21 06:10 UTC | `3471175` 最终树发布权威门 | `cargo +nightly fmt --all --check`、全目标全特性严格 Clippy、`actionlint`、`libra diff --check` exit0；上游深历史 clone 新例 1/1，macOS 工作流兼容守卫 12/12。`source .env.test && source .env.live-test && cargo nextest run --all --no-fail-fast --retries 2` exit0：10664/10664 passed、20 slow、3 leaky、23 skipped、无 FAIL/RETRY；三条 LEAK 分别为 cov2_native_v2_history_fault_atomicity、checked_batch_combines_local_and_remote_presence、bisect_help_lists_full_subcommand_surface。网站八页限定检查 drift=0；全站只读检查 22 页漂移均在任务范围外。Claude Code 对新增工作流/守卫差异字面 `VERDICT: PASS`，无 P0/P1，三条非阻断 P2：stub 未钉死 target/参数（由最终原生 macOS CI 真执行兜底）、Python3 依赖（该 workflow 本就调用 Python3，托管 runner 提供）、极低概率 ETXTBSY（沿用既有 stub 模式，nextest 本轮通过）；本轮具名接受，不作为发布阻断。另一轮宽范围 Claude 达 max turns、无 verdict，不计通过 | nextest 权威全量通过；`cargo test --all` 最终重跑、发布模式/远端 CI/资产/stable/网站部署仍待，不提前关闭九卡 |
| 2026-10-21 07:15 UTC | `ba7425d` 最终本地门与签名源码提交 | 最终 `source .env.test && RUST_TEST_THREADS=4 cargo test --all` exit0：181 个零筛选主目标摘要、10676 passed、0 failed、26 ignored；测试内部另打印 8 个有筛选的子进程摘要，未混入主目标计数。`source .env.test && source .env.live-test && cargo nextest run --all --no-fail-fast --retries 2` 仍为同一源码树的 10664/10664 passed，3 leaky/23 skipped。`cargo build --locked --release --features keyring` exit0，以真实发行文件名 `libra-darwin-arm64` 在 disposable repo 完成 version、NAME 默认路径、`--path`、list、remove 冒烟。macFUSE 5.4.0 在已提交 SHA 上重新跑完整实挂载组 16/16，卸载后无残留挂载；OrbStack Linux `/dev/fuse` 在相同 Worktree 生产源码上 16/16 和 foreign macOS v2 owner 1/1。签名+DCO 源码提交 `ba7425d9f8ad525b3be06ab1e43ee0ef4b6e874d` 已原生推送 `main`，远端 HEAD 相同；CodeQL 与代码质量 success，base run `39554165644` 当时 7/8 success，离线核心仍在运行。网站八页限定同步 drift=0、脚本 8/8、Vitest 11/11、typecheck/build exit0，Claude Code 网站范围字面 `VERDICT: PASS`；八页尚未推送部署 | 本地、macFUSE、旧版保护与源码提交已验；最终 SHA 的 base CI 8/8、tag/资产/stable/GitHub Release、网站部署和计划终止链仍待，不提前关闭九卡 |
| 2026-10-21 07:42 UTC | `4234095` 并发合流、FIX-WD-07 复现与修复 | 新上游 merge `4234095454ab8e2d02ad5ceb0aafb66d69ac1dc1` 只净增 `init.rs`、`init_test.rs`、双语 init 文档与 #473 记录，版本仍 `0.30.78`；`ba7425d` base run `39554165644` 被自动取消，不能记全绿。计划文档备份 SHA-256 `5790aa56...` 后单文件还原，原生 `pull --ff-only` 到 `4234095`，再按同 SHA-256 恢复；新旧 #501 定向 4/4 与网站八页限定 drift=0、脚本 8/8、Vitest 11/11、typecheck/build 通过。Claude delta review 字面 FAIL/P2 指出 `--template` 与目标路径 canonical/raw 比较不一致；隔离 fresh init 和 reinit 同因 `ENAMETOOLONG` 失败，后者留嵌套文件。FIX-WD-07 采用已有 `canonicalize_deepest_existing` 做写前比较，在目标创建后再 `fs::canonicalize` 判同根/祖先及 `.libra` 内模板，实际复制跳过 canonical target；符号链接/`..`/reinit/内嵌模板七个定向用例 7/7，双语文档、网站 init 页和 changelog 同步。原 `4234095` cargo 全量在修复前主动中止 exit130，不计通过 | 修复仍未推送；严格 Clippy、双全量、Claude 跟进 PASS、最终 SHA CI 与发布门待重跑 |
| 2026-10-21 09:24 UTC | `7806c91` 提交后发现的 init 路径尾检缺陷 | 签名+DCO `7806c9135c98c770e4c77152e44fedc88fa6e900` 已推远端 main，远端 SHA 相同；该轮 `cargo test --all` 正在运行、无失败摘要时，隔离 CLI 冒烟复现 `regular-file/..` 于 Unix 先创建父目录 `.libra`、后以原始拼写开库失败 exit128。全量主动 SIGINT、中途结果不计通过，base run `39574920827` 请求取消。FIX-WD-07b 新增存储真实路径传回 CLI、Unix 非目录组件前置拒绝，两条端到端用例分别 1/1、init 47/47、转换 11/11，fmt check exit0；源码 Claude 复审与严格 Clippy 进行中。网站八页限定同步 drift=0、脚本 8/8；Vitest 10/11，唯一失败为与 Cargo 编译并行时超过 5 秒，不能计通过，待独立重跑。 | 保留 `7806c91` 的提交和失败/取消证据，不把它当 final SHA；最终树双全量、CI、发布及网站部署仍待 |
| 2026-10-21 09:39 UTC | 上游发布失败与 Worktree 目标前滚 | `v0.30.79` tag 指向 `9ba1369`、GitHub Release 为 Latest，但 release run `39576472385` 的 macOS Build Binary (Unix) 因 `Could not resolve host: index.crates.io` exit101，其余构建取消，Homebrew、安装脚本、stable manifest 步骤跳过；现场 CDN stable 仍 `0.30.77`/176。远端 `main=de72da7` 合入 #633 和 #523，七个本地未提交文件备份于 `/tmp/libra-wd-0.30.80-pre-9ba` 后原生洁净快进；这些文件相对 `7806c91` 的上游差异为空，按清单 SHA-256 7/7 恢复。用户规则令新目标为 `0.30.80`，Cargo.toml/Cargo.lock/双安装脚本已改版本，Worktree/init 文档支持版本改为 `0.30.80`；最终树静态、双全量、发布与网站检查待重跑 | 不把 `v0.30.79` 的 GitHub Release 当作 stable 发布，也不把 `7806c91` CI 当作最终门；后续若上游再推进须重新计算版本 |
| 2026-10-21 09:50 UTC | `de72da7` + FIX-WD-07b + `0.30.80` 定向验证 | `source .env.test && cargo test --test command_test init_test` 47/47（含 Unix 3 路径 x bare/普通的零写与 `LBR-REPO-002`，及 SHA-256 `missing/..` 成功）、`init_conversion_test` 11/11、`compat_version_surface_sync` 2/2、`compat_worktree_docs_v2_layout` 2/2；`cargo +nightly fmt --all --check`、`cargo clippy --all-targets --all-features -- -D warnings`、`actionlint .github/workflows/base.yml`、`libra diff --check` 均 exit0。网站八页限定同步 drift=0、脚本 8/8、Vitest 11/11、typecheck/build exit0；全站只读 24 页范围外漂移未改。Claude Code 对最新源码差异字面 `VERDICT: PASS`、P0/P1/P2=0；P3 的广义 NotADirectory 提示、0.30.79 无资产注记和 Changelog 文案精度具名接受，均不影响数据安全或本轮门。网站复审、最终双全量、macFUSE 实挂载、final-SHA CI 与发行待验 | 仅定向和静态门通过；九卡仍 0/9，不提前发布 |
| 2026-10-21 09:54 UTC | `30b11fb` 推送与网站审查阻断 | 签名+DCO `30b11fbde2a0a8bcdd0ed624bc76bf47ef9ccaa2` 已推 `main`，远端 HEAD 相同，CodeQL Code Quality success，base run `39580327427` 启动但尚未完成。网站限定八页同步/测试/typecheck/build 曾全绿，随后 Claude Code 只读复审字面 `VERDICT: FAIL`，具体 P1/P2 如上；这使先前网站门失效，不可提交/部署 cf。双语源文档、网站页和 sequencer 的旧位置参数提示已前滚修正；修改后的限定同步/测试、Claude 跟进 PASS、静态/双全量与 final-SHA CI 待验 | 保留失败 review，旧 SHA CI/网站检查不计最终门；计划仍 0/9 |
| 2026-10-21 10:02 UTC | 网站八页审查阻断闭合 | 双语 Worktree/init/upgrade 与英文 agent 源文档已前滚，`src/internal/sequencer/mod.rs` 旧位置参数提示改为 `--path <worktree-path>`。网站 `--only worktree agent review investigate sandbox hooks upgrade init` 检查 drift=0，同步器单测 8/8、文档 Vitest 11/11、`pnpm typecheck` 和 `pnpm build` exit0；Claude Code 跟进审查 `/tmp/libra-wd-0.30.80-claude-site-followup-review.log` 字面 `VERDICT: PASS`、P0/P1/P2=0。它指出 0.30.80 未 tag、无 stable 前不得部署，以及英文页引用中文开发文档原标题 P3；按既定发布顺序待验，不将网站 PASS 当线上门 | 网站文档范围合格，final-SHA 源码/CI、release、cf 推送/部署与 live 页面仍待 |

## 全量门暴露的基线修复

**FIX-WD-01（capture 架构与 oracle 基线，已验证）：** 首轮 `cargo test --all` 在 `compat_agent_architecture_guard` 失败 4/64。未修改的 HEAD `85dcc50`（0.30.56）在隔离 baseline worktree 运行 `cargo test --test agent_lifecycle_event_test hook_public_contract_claude_codex_ci -- --exact` 同样失败，实测 Claude `PermissionRequest` receipt 与两条 subagent boundary 均已经产生；`6554e13`（CC-02）将这些事件接入 Observe ingress，而冻结 golden 未同步。此问题不由 worktree 迁移引起。按 Claude Code 对实际 diff 首轮 `VERDICT: FAIL` 的要求先复现/归因，再取得基线证据复审 `VERDICT: PASS`（仅限 oracle 修复）。修复写集限 `tests/agent_lifecycle_event_test.rs` 的原有 oracle 行与可逆 fixture、`tests/compat/agent_architecture_guard.rs` 的两枚指纹及历史迁移链检查、`docs/development/tracing/agent.md` 的 ADR-ACF-10 修订；同一门的原有 provider 反向依赖将 MessageDisplay 计数门从 `hooks/providers/claude/events.rs` 原样移到 `capture/ingress.rs`，`capture/extraction.rs` 的非首批 provider 改成 partial-empty fallback，避免提高 AgentKind path ratchet。该修复不改变 worktree 合约。`compat_agent_architecture_guard` focused 64/64、Claude/Codex oracle 1/1；独立源码复审及最终 `cargo test --all`、nextest 均已通过，发布树 CI 的离线全量亦通过。基线日志 `/tmp/libra-wd-head-oracle-baseline.log`，首次门日志 `/tmp/libra-wd-cargo-all-final.log`，Claude 复审 `/tmp/libra-wd-acf-baseline-evidence-review.txt`。

**FIX-WD-02（全量文档契约与 config 测试环境隔离，已验证）：** 第二轮全量锁定的中文 agent 文档字面承诺已在既有 WD-04 文档写集恢复为“不新增稳定错误码”，`compat_agent_docs_contract` 34/34。第三轮全量中 `config_test::get_best_effort_reads_value_inside_repository` 初始化误用其他测试的 future-schema global config：`global_config_create_uses_global_role` 正在修改进程环境，本用例只持有 `cwd` lane。修复写集增列 `tests/command/config_test.rs`、`tests/SERIAL_REGISTRY.tsv`：读者使用独立 `ConfigDbFixture` 并持有 `cwd+env`，global/system 两个 future-schema 写者在注入期间持有 `ChangeDirGuard` 且标记 `env+cwd`。`sh tests/SERIAL_CLASSIFY.sh` 对三例的判定均与 registry 一致，`sh tests/NEXTEST_GROUPS.sh` 重生配置但 `.config/nextest.toml` 内容未变；focused 1/1、config 组 184/184、serial registry 26/26、fmt 和严格 clippy 通过。此修复仅隔离既有测试，不改变生产配置语义；独立审查和最终 `cargo test --all`、nextest 两门均通过，失败轮次仍按历史 FAIL 保留。

**FIX-WD-03（发布资产名与 CLI helper 能力：代码修复已验证、`0.30.83` 发行待验）：** `v0.31.0` 的已上传 macOS 资产在原始平台文件名下复现 `add` 失败，重命名后通过，说明二进制本身可工作而 helper 入口的文件名推断错误。`src/main.rs` 在 CLI 入口登记进程内能力，`src/internal/worktree_io/executor.rs` 据此调度子进程；独立发行文件名回归已在本地和原生 Windows 通过。源码 Cargo/安装版本面曾前滚至 `0.31.1`，但用户明确禁止发布此版本；现在按其新指令以上游最新 `v0.30.82` 的 patch + 1，即 `0.30.83` 前滚。旧 focused、严格 Clippy、release 模式、分区 Claude 复审、`cc8f437` CI 8/8、本地双全量证据不能替代当前合流树；须重核版本面、四平台资产与 stable/安装验收，不能将取消的 `v0.31.0` run 或旧 `v0.30.66` 发布当作通过。

**FIX-WD-04（全量 MCP 测试跨平台存活判定，定向已验证）：** `0.30.78` 首轮 `cargo test --all` 在上游新增的 `agent_mcp_exec_test::deadline_reply_does_not_wait_for_stdout_holder` 失败，因为其 `/proc/<pid>` 探测在 macOS 恒为 false，`timed_out_worker_is_killed` 也因此曾假阳性。修复仅限 `tests/agent_mcp_exec_test.rs`：Unix `kill(pid, 0)`、Windows query-only `OpenProcess`/`GetExitCodeProcess`，且只在实际生成 stdout holder 的 Unix 上运行该用例。定向 9/9 已通过；完整 cargo/nextest 仍须在最终树重新通过。

**FIX-WD-05（macOS Git 转换硬中断恢复，定向已验证）：** 第二轮 `cargo test --all` 在 `command::init_conversion_test::init_conversion_hard_kill_is_detected_and_recovered` 失败；同一用例隔离重跑也失败。macOS 的 `read_pid_starttime` 原先恒为 `None`，`ConversionOwner::is_live` 对没有启动时间的标记保守视为永远存活，导致已退出的转换 owner 被报 `LBR-CONFLICT-002`，而现有 `docs/commands/init.md` 已承诺可安全重试。`src/command/control_lock.rs` 现在用 macOS `proc_pidinfo(PROC_PIDTBSDINFO)` 读取微秒级启动时间并核对 PID；读取失败仍返回 `None`、保持保守行为。新增 macOS 单元 1/1，端到端硬中断恢复 1/1。第一次端到端筛选因误加不完整 `--exact` 匹配 0 tests，不计通过；随后使用完整名称重跑 1/1。旧全量在记录失败后以 exit130 中断，非完整结果；最终树必须重跑全部门。`CHANGELOG.md`、`docs/commands/init.md` 与网站 `init.en.md` 同步修复后的恢复说明，网站发布范围因此由原七页扩至八页；同步/前端验证及 Claude Code 复审在最终树重新执行。

**FIX-WD-05 安全/兼容补充：** 独立复审指出两个边界：Mac 上进程仍存活但 `proc_pidinfo` 暂时不可读时，不可把身份未知当已死亡；以及给 service sidecar 新增启动时间会令旧 Mac 客户端误报服务不存在。`src/command/init_conversion.rs` 对已有启动时间但读取失败的活 PID 保守拒绝恢复，仍保留旧 `start_time=None` 标记必须人工恢复的跨机器保护；`src/command/service.rs` 在 macOS 保留旧版 PID-only sidecar 格式。两条新单元分别 1/1，端到端恢复 1/1。`.github/workflows/base.yml` 增加五个非零 exact macOS fixture，并顺带修正该工作流既有的两处 shell 变量未引用问题使 `actionlint` exit0；远端原生 macOS job 尚待最终 SHA 验证。本补充新增写集：`src/command/init_conversion.rs`、`src/command/service.rs`、`.github/workflows/base.yml`；旧标记的手动恢复约束在双端 init 文档中明确。

**FIX-WD-05 混合版本与跨主机收口（最终门待跑）：** 后续独立审查指出：若新 Mac 标记直接填 `owner.start_time`，旧 `0.30.77` Mac 客户端会误把运行中的转换判死；现改为旧客户端忽略的 `owner_start_time_v2` 字段，旧 `owner.start_time` 保持空。实际下载 `v0.30.77` macOS arm64 发行资产，SHA256 与 CDN sidecar 同为 `494242b685bb0dad7301757c645123c9ddc526b931fdcba409c799510c022b39`，模拟运行中新版标记时旧二进制返回 `LBR-CONFLICT-002`，标记保留、无 aside。新版同时以 macOS boot session UUID 锁定身份：只有同一启动会话可用 PID/启动时间证明退出；其他主机、重启后或缺身份的标记均须确认无转换后人工恢复。`src/cli.rs` 普通命令与 `init` 重试共用此判断，避免旧标记在 status 中提示无限等待。新单元、旧标记命令回归及硬中断回归已定向通过；这批新增源码必须随最终树重新跑 fmt、严格 Clippy、cargo 全量、nextest、原生 macOS CI 和 Claude Code 复审。一次 macFUSE 组因并行 Cargo 构建覆盖 `target/debug/libra` 出现失败且有挂载清理卡顿；停止并行构建后独立重跑真实挂载 16/16 通过、无残留挂载，失败轮次不计通过。

**FIX-WD-05 对称跨平台保护：** 当前树第一次最终独立审查字面 `VERDICT: FAIL`：非 Mac 读者曾把 Mac 专属 `owner_start_time_v2` 当作本地进程时间，可能在共享目录中误移走仍运行的转换。非 Mac 分支现仅使用本平台写入的旧 `owner.start_time`，Linux 容器 `foreign_macos_v2_owner_requires_manual_recovery` 1/1 通过；真实 Linux FUSE 16/16 另验。第二次只读复审确认该方向关闭，但字面 `VERDICT: FAIL` 指出反方向：Mac 曾把 Linux/Windows 的旧字段当本机微秒时间。Mac 分支现忽略旧字段，仅在本机 boot session 匹配时接受 Mac v2 字段；新增 `foreign_legacy_start_time_requires_manual_recovery` 与生产标记写入器驱动的 live/source-mismatch 测试。Mac 转换单元 13/13、命令组 5/5 通过，严格 Clippy 发现的嵌套条件已前滚修正；最终源码的再次静态门、双平台 FUSE、双全量、Claude Code 复审及远端 CI 均须重新通过。`CHANGELOG.md` 将已纳入本发行的 Unreleased 项归入 `0.30.78`，`docs/commands/zh-CN/init.md` 补齐中断转换恢复，网站仍仅同步 EN 由渲染时镜像。

**FIX-WD-06（macOS 工作流兼容守卫）：** 最终全量在 `compat_matrix_alignment` 暴露守卫只支持单个 `run` 步骤，对新增转换恢复步骤的合法 YAML 空行报缩进错误。按具名步骤独立截取脚本，并用私有 Cargo stub 证明新增步骤的五个 exact fixture 全部执行且拒绝零用例、错误用例、ignored、非零退出；定向 12/12 与 `actionlint` 通过。首轮全量 exit101、9/11 的事实保留，最终树双全量仍待。

**FIX-WD-07（上游 #501 模板和 Git 转换安全前滚，发行门待跑）：** 首版目标路径检查只用 `canonicalize_deepest_existing`，Claude 复审与隔离实测相继发现 `missing/..` 可使模板自复制、裸仓库布局错误回滚误删模板，以及转换回滚误删已有目标。最终实现改为逐组件解析现有 symlink 与缺失组件、写前统一规范目标/存储路径、以独占创建证明存储目录归属，并在实际布局写入前后保持同一安全边界。已有 bare 目标转换仅登记本次可能生成的布局条目和数据库文件，不登记用户目录；默认 macOS/Windows 的大小写别名用真实 `symlink_metadata` 在预检及归属快照中判断。r4 Claude 对前述数据丢失 P1 给出字面 `VERDICT: PASS`，其指出的 SQLite journal 与布局失败残留已继续前滚；r5 又给出字面 `VERDICT: FAIL/P1`，指出模板大小写变体可能让回滚误删已有 bare 用户文件，已修并新增文件/目录、shared 私有权限回归，转换组 11/11。上游 `fc5eafc` 合流后的 fmt/Clippy、r6 Claude 源码复审字面 `VERDICT: PASS`（P0/P1/P2=0）及原生 macFUSE 真挂载 16/16 均已通过；最终双全量和 final-SHA CI 待验。源码/双语 init/网站 init/changelog 均在 FIX-WD-07 写集内；旧数据迁移仍不在范围。

## 完成判据

- [ ] 粒度审计满足 G-*，无未登记超限或写集冲突。
- [ ] WD-01..08 与 WD-02G 共九卡全部 Lifecycle=done / Acceptance=complete；A/B、review、继承 C/D 的证据均实际存在。
- [ ] 最终 bump 树 `cargo +nightly fmt --all --check`、`cargo clippy --all-targets --all-features -- -D warnings`、用户要求的 `source .env.test && cargo test --all` 与发布权威门 `source .env.test && source .env.live-test && cargo nextest run --all --no-fail-fast --retries 2` 均执行并通过；cargo test --all 不替代 nextest 发布证据；FLAKY/skipped/失败准确记录。
- [ ] 全量暴露问题已修复且全量重跑至绿；既有失败有基线复现与正式 FIX/DEFER 记录，红色全量不能收口。
- [ ] 指定 focused、feature、真实 FUSE、旧二进制 worktree/agent 消费者数据不变、故障恢复以及 final-SHA 原生 Windows scoped-input 门全部通过。
- [ ] 双语命令/网站同步、网站测试/typecheck/build、实际部署页面验收通过。
- [ ] Codex 与 Claude Code 最终审查 PASS，P0/P1 清零；P2 只有具名接受才保留。
- [ ] 用户指定的 patch + 1 版本面 parity、Cargo 工具链 lockfile、签名+DCO、main/cf 推送及远端 SHA、release.yml、artifact/stable 证据全部齐全。
- [ ] 所有证据写本文，无新证据文件/目录；修订历史、plan-status/plan-long 与终止链协议已履行。
