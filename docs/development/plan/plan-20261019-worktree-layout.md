# Worktree 默认目录统一计划（2026-10-19）

**模板版本:** v2.14。**更新时间:** 2026-10-19 11:39:23 UTC。**执行分支:** Libra 原生仓库 `main`。**来源:** 用户直接要求统一 worktree 目录，明确不处理旧数据迁移，并授权 Implement the plan；无 linked Issue。

## 文档职责

统一新建 linked、FUSE 和 review/investigate 工作区的默认布局到 common storage 的 `worktree/`，保持工作文件、工作区身份与共享元数据边界正确。本文是执行契约，不是实现或验收完成声明。

### 适用范围

- `worktree` 普通/FUSE 两套 CLI、注册表、作用域发现、暂存、对象保留及存储遍历消费者。
- review/investigate 的普通副本、checkpoint 输入、恢复、清理和 sandbox 环境目录。
- 同卡双语命令文档、网站 `cf` 文档、兼容验证、0.31.0 家族发布与线上验收。

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
| 完整性/接口 | CLI 行为属于次版本变化；无需旧目录搬迁 | NAME/--path、旧记录只读兼容、minor 发布 |
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
- WD-01..07 与 WD-02G 是不可独立交付的家族子卡，WD-08 唯一发布；本家族在共享现有 main 上实施，子卡变更保持未提交，不创建中间卡提交，也不提前推 main/cf，避免其他执行者推送共享 HEAD 时带走半成品。根执行者是唯一 committer/pusher，其他 Agent 不 commit/push；WD-08 全部本地门完成后才精确暂存并创建一个完整家族提交。次版本 `0.31.0`（发布窗口已越过 0.30 时取当时版本下一 minor）；所有版本面以 compat_version_surface_sync 为权威，Cargo.lock 由 Cargo 更新。
- **Consequences:** Repository receipt 2026101902 在新二进制首次打开仓库时即应用，即使用户尚未创建新 worktree，该仓库此后所有 0.30.x 命令都会在 DB connect 处拒绝；恢复方式是安装 0.31.0 或更新的二进制。新格式上线只能前滚补丁，旧 CLI 不作回滚方案。
- **Revisit when:** 次版本窗口被其他发布占用，先记录最新版本并刷新版本决策。

## 全局工程约束

完整继承模板 GC-01..13、ER-01..14、G-01..11；特别补充：

- **GC-WD-01:** 所有路径生成/名称验证/容器边界只有一个 helper；不得以字符串 starts_with 替代规范路径和 no-follow 检查。
- **GC-WD-02:** WD-02G 仅为 Repository 增加 2026101902 兼容 receipt，并向既有 worktree_registry_capability 写 version 4；不搬旧目录、不变更旧数据或补名称。真实仓库已应用的 SQL 文件内容冻结为 SHA-256 `c1dbf1ae702e391db2adef39245fcd66b745a07bc64aa1511e7984c911333c43`；改变语义只能新增更高版本迁移。migration/bootstrap/schema manifest 的 allowlist 仅 Repository，ledger 为 schema_versions；GlobalConfig/SystemConfig 继续各自配置 ledger，不运行该 writer，Derived 拒绝通用迁移。tests 显式隔离 LIBRA_CONFIG_GLOBAL_DB/LIBRA_CONFIG_SYSTEM_DB/HOME，断言配置 receipt/表内容不变，不得落真实配置库。
- **GC-WD-02a:** 每次刷新 HEAD/origin 都核对迁移目录、README 与上述 SQL hash。若上游不同迁移占用 2026101902，立即暂停合并/提交/发布，保留真实 DB 的本地 receipt；先协调上游作者在尚未发布时改号。若上游迁移已发布，则按两种实际 DB 状态设计更高编号的条件修复迁移，使「本地已应用本计划 1902」补上外来 DDL、「上游已应用外来 1902」补上本计划 capability，并在两种克隆上逐项验证 schema、receipt 和对象数据。只有该前滚方案完成源码/测试/文档审查并取得 PASS 才恢复发布；不能仅改本地 ledger 名称或假定版本号相同意味着 DDL 已执行。
- **GC-WD-03:** 不新增生产 unwrap/expect/panic；错误说明资源、失败原因与修复方向。复用既有 LBR 错误类别；如必须新增稳定码，先扩写集并触发 T-1，Display-pin 与 docs/error-codes 同卡。
- **GC-WD-04:** 每卡 A/B focused 和 Claude Code 源码审查须真实完成；家族子卡只到 locally-accepted，继承 WD-08 C/D 后才能 done/complete。会话设计 review 不替代正式计划/实现 review。
- **GC-WD-05:** 按用户 AGENTS 同时执行 `source .env.test && cargo test --all`；模板 release/全量通过证据只认 `source .env.test && source .env.live-test && cargo nextest run --all --no-fail-fast --retries 2`。两者都必须执行并如实记录，前者不替代 nextest 发布门；取消/失败/未运行不算 PASS。任何失败修复后重跑受影响门和全量至绿。
- **GC-WD-06:** 网站直接现有 native Libra cf，保留无关改动，不建 branch/PR/worktree。各卡完成仓库双语文档并明确网站映射；网站七页和同步脚本已做可逆准备性写入但未提交/推送或计入验收；WD-08 必须按最终 CLI/仓库文档重新同步并逐文件验收，随后提交推送和验证部署。卡内 ER-06a 义务在此延迟窗口取得，未同步前不能 complete。

## 执行检查必备需求

1. 每卡 ER-01..03 安全检查、锚点和粒度复核；扩写集先更新本文。
2. A 组按写集运行单元/集成/feature gates；B 组按任务类型；ER-05 Codex 和 Claude Code reviewer 必须实际读最终差异，最终 `VERDICT: PASS`。
3. 每个会推送的树运行 fmt/clippy。子卡相关 T-1/T-3 风险由家族发布点包含最终变更的全量承担；WD-08 T-2 恒定全量。
4. C/D 使用模板完整发布协议：版本面 parity、工具链 lockfile、bump 树 nextest、release build/install/版本检查、签名+DCO、精确暂存、native push、tag/release.yml、远端产物与 stable 清单证据。任何外部门失败保留 remote-pending。
5. 网站必需门（只在最终 CLI/仓库文档就绪后，在现有 native cf 的 backend cwd 执行）：先只读全库检查并记录基线漂移，再以 `--only worktree agent review investigate sandbox hooks upgrade` 分别运行同步器 `--write` 与检查；`python3 scripts/test_sync_libra_command_docs.py`；`pnpm exec vitest run tests/unit/docs-manifest.test.ts tests/unit/docs-search.test.ts`；`pnpm typecheck`；`pnpm build`。本轮只同步七张受影响命令页，不把先前 12 张无关页面的漂移混入。网站只存 EN 命令页，zh-CN 由该内容在渲染时镜像并保留语言内链接，不新增不存在的中文源文件。
6. 最终提交前重新读取本地 HEAD、origin/main 和 backend cf/origin/cf，核对远端没有部分 WD 变更，重新检查精确写集和当前 dirty diff。若上游发生变化，先保存当前工作字节并人工合并到最新源码，重新审查/运行受影响门；禁止自动 rebase/reset 或推送无关变更。该核验失败则保持 dirty 未提交，不尝试绕过。
7. 原生 Windows D 门是 `.github/workflows/base.yml` 的 `compat-scoped-input-windows`：最终 main SHA 对应的四个 active exact scoped-input fixture 与 windows_fs 单元组均需通过，cross-check/Windows artifact 编译均不能替代。main 推送后观察此 job 全绿，才触发 tag/artifact release；现有并发 relocation DEFER 仍不算 PASS。
8. COMPATIBILITY 仅增加本行为所需记录，不无关重排现有 ledger 锚点；实际触及的守卫测试通过才作兼容证据，不把未测试文字当限制。
9. 测试文件/serial 新增必须在同卡维护 `tests/INDEX.md`、`tests/SERIAL_REGISTRY.tsv`、`tests/SRC_SERIAL_REGISTRY.tsv`、`.config/nextest.toml` 的实际影响；只有新增 target 才改 Cargo.toml 注册。本文计划使用现有 targets，不凭空新增 target。

## 实施顺序

`WD-01 -> WD-02G -> WD-02 -> WD-03 -> WD-04 -> WD-05 -> WD-06 -> WD-07 -> WD-08`。外部依赖在下表登记；无卡外隐藏先决条件。

### 依赖登记表

| ID | direction | 类型 | 对象 | Owner | 产物与可用性判据 | 证据 | 超时与失败策略 |
|---|---|---|---|---|---|---|---|
| DEP-WD-CODEX | incoming | 审查 | 当前 Codex 执行者 | 根执行者 | 最终源码给出具名 `VERDICT: PASS`、P0/P1 清零及真实验证清单；不可用则阻断，不自填 | 实现仍在进行 | 在最终 Claude 审查之前修复发现并重跑受影响门 |
| DEP-WD-01 | incoming | 审查 | Claude Code 本地 reviewer | 当前根执行者协调 | 正式本文及最终源码分别取得字面 VERDICT: PASS | 会话设计五轮 PASS；正式文件 R3 PASS、R4/R5 FAIL，最新修订待复审；最终源码未审 | review 失败先修；工具不可用保留明确阻断，不伪称 PASS |
| DEP-WD-02 | incoming | 执行环境 | 受支持 Unix FUSE 环境 | 根执行者 | feature 编译、真实 mount/opaque/unmount 可执行 | 开工探测待记 | 本地缺少驱动时寻找已授权测试主机；仍不可用则发布阻断 |
| DEP-WD-03 | incoming | 发布服务 | GitHub release、Cloudflare artifact/site | 当前根执行者单一发布 | 认证可用、对应 SHA release 与部署可观察 | 开工及发布窗口核验待记 | 只保留 remote-pending，禁止用本地绿替代 |
| DEP-WD-05 | incoming | 原生平台验收 | final main SHA 的 compat-scoped-input-windows | 根执行者 | windows-latest 原生 exact scoped-input 与 windows_fs 组全绿 | `.github/workflows/base.yml:2-4` 的 main push；`:229-246` 以 `utils::windows_fs::tests::` 模块过滤且断言至少一例通过，须在 run log 核对新三例名称 | 失败或取消保留 remote-pending，修复重推后重跑；不得发 tag 或用交叉编译替代 |
| DEP-WD-04 | incoming | 兼容工具 | 既有 0.30.53 CLI | 当前根执行者 | disposable repo 旧二进制拒绝矩阵可运行 | 本机 installed 0.30.53；必须在安装新版前固定可执行副本或可复现下载 | 无可用旧二进制则 WD-02 不能本地验收 |

### 发布分组与并发窗口

| ID | 成员 | 唯一发布点 | 窗口规则 | 失败回滚顺序 | 理由 |
|---|---|---|---|---|---|
| REL-WD01 | WD-01..08, WD-02G | WD-08 | root 唯一 committer/pusher；因远端并发推进，允许精确路径的本地签名快照提交以安全 rebase，不得提前推送或视为交付；backend 准备性 dirty 写入不视为交付；合流后按最终源码重做同步、测试与审查；WD-08 独占 R 写集，刷新 HEAD/origin 并核对无远端半成品后推送完整家族 | 发布前在当前轴内修复；新格式/已发布后从失败消费者向 helper 前滚，绝不降级 writer | CLI/schema/路径/cleaner 必须同步上线 |

**并发声明:** WD-01..08 与 WD-02G 的依赖 API 接入与验收推进串行；可并行只读研究、审查及已明确文件独占、I 写集不相交的准备性实现/测试。准备性代码不得在上游门通过前被当作下游卡已验收；共享文件只能单写者编辑，根执行者集成后按 DAG 逐卡验收。共享仓库其它执行者须知 installed 0.30.53 已被 future-schema 门拒绝，统一使用本计划验证过的候选二进制。**发布者:** 当前根执行者（单一写入者，不虚构外部负责人）。 **真实工作库边界:** 验收实验一律 disposable repo；Libra 真实仓库已于 10:53:05 UTC 应用 2026101902，此后 VCS 使用 `target/debug/libra`，每次暂存/提交前记录该文件 SHA-256、`--version`、构建树与已通过的测试，最终使用验证后重建且未再修改源码的候选二进制；backend 真实仓库目前最大 receipt 2026101901、capability 2/3，最终 VCS 操作才允许由同一候选二进制写入 2026101902；安装旧 0.30.53 的只读对照副本固定在独立路径，不随安装覆盖。**发布窗口顺序:** 合流远端并重跑最终 bump 树→最终 cf 文档同步/验证→刷新 HEAD/origin 与精确写集→完整家族/main 推送→final-SHA 原生 Windows D 门→tag/release→网站 cf/deploy→远端收口登记。

### Phase 0: 边界与手动 worktree

进入：正式计划 review PASS，工作区安全检查通过。退出：WD-01、WD-02G、WD-02、WD-03 各自 A/B 与实现 review PASS，均 locally-accepted，未提前发布。

### Phase 1: 自动工作区与发布

进入：WD-03 已本地验收。退出：WD-04..07 本地验收，WD-08 完成全量与发布门，所有卡继承 C/D 成为 complete。

## 任务卡

### 字段全局默认与例外

`Task type=implementation`；`Release boundary=family child of REL-WD01`；`Release write set=N/A`；`Version increment=N/A`；`C/D coverage from=WD-08`；`Rollback mode=forward-only`；`Migration and rollback=不搬迁旧数据；在当前新布局前滚修复，验证 doctor/list/工作文件 hash 保持`；`Security and privacy=GC-WD-01..03/ADR-WD-04`；`Performance budget=按注册表/指定 run 查找，不递归扫描工作区内容`。这些全局默认即逐卡字段值；卡内未重复书写时仍由本行提供，不视为遗漏。所有新增 test fn 下文标 (new)，实现时名称必须一致或同批修计划；过滤运行匹配 0 tests 不算通过，必须核对实际匹配用例数。

默认覆盖：WD-01 在未对外写新布局前可 revert；WD-02G 为 metadata-only migration/forward-only；WD-08 为 release/immutable-release/self/minor。**规则 waiver:** N/A，无已批准豁免；实现超过 M 或 AC/VER 上限必须先拆卡修订，不能为保留原卡数而压缩独立判据。

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
- [ ] `source .env.test && cargo test --test command_test worktree_isolation_test::wd_nested_discovery`（new）。
- [ ] `source .env.test && cargo test --test command_test worktree_isolation_test::wd_storage_consumers`（new，含 cloud/clone/maintenance 边界）。
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
- [ ] `source .env.test && cargo test --test command_test worktree_test`（补 new `wd_name_path_lifecycle`、`wd_concurrent_name_reservation`）。
- [ ] `source .env.test && cargo test --test command_test worktree_isolation_test`（补 new `wd_internal_add_modes`、`wd_gc_index_roots`）。
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
- [ ] `source .env.test && cargo test --features worktree-fuse --test command_test worktree_fuse_test`（补 new `wd_fuse_named_layout`、`wd_fuse_opaque_boundary`、`wd_fuse_creation_failure`）。
- [ ] `source .env.test && cargo test --test command_test worktree_test::wd_fuse_name_without_feature`（new）。
- [ ] 受支持 Unix 实机执行新 `wd_fuse_live_mount_boundary` ignored case：`source .env.test && cargo test --features worktree-fuse --test command_test command::worktree_fuse_test::wd_fuse_live_mount_boundary -- --ignored --exact`；证明 mount→读 opaque→兄弟不可见→unmount，缺驱动是未运行。
**Full-suite trigger:** T-1/T-3：两套公开入口/共享名称锁及 serial/nextest 配置；C 由 WD-08 执行。
**Dependencies:** WD-02（注册表和共享锁），DEP-WD-02，DEP-WD-04。
**Implementation write set:** `src/command/worktree-fuse.rs`, `src/command/worktree/registry.rs`, `src/command/worktree/lock.rs`, `src/utils/worktree_layout.rs`, `src/utils/fuse.rs`, `src/utils/util.rs`, `tests/command/worktree_fuse_test.rs`, `tests/command/worktree_test.rs`, `tests/SERIAL_REGISTRY.tsv`, `.config/nextest.toml`, `docs/commands/worktree.md`, `docs/commands/zh-CN/worktree.md`, `docs/development/commands/worktree.md`, `COMPATIBILITY.md`, `../libra-backend/apps/tanstack-app/content/docs/commands/worktree.en.md`。
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
- [ ] `source .env.test && cargo test --lib sandbox::policy`（补 new `wd_sibling_storage_denied`）。
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
- [ ] `source .env.test && cargo test --test agent_review_workflow_test`（补 new `wd_review_attempt_cleanup`、`wd_review_libra_discovery_refused`）。
- [ ] `source .env.test && cargo test --lib review::store`（补 new `wd_review_cleanup_failure_keeps_metadata`）。
- [ ] `source .env.test && cargo test --test command_test agent_clean_test`（补 new `wd_review_clean_owned_workspace`）。
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
- [ ] `source .env.test && cargo test --test agent_investigate_workflow_test`（补 new `wd_investigate_resume_attempt`、`wd_investigate_libra_discovery_refused`）。
- [ ] `source .env.test && cargo test --lib investigate::store`（补 new `wd_investigate_cleanup_failure_keeps_metadata`）。
- [ ] `source .env.test && cargo test --test command_test agent_clean_test`（补 new `wd_investigate_clean_owned_workspace`）。
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
- [ ] `source .env.test && cargo test --lib checkpoint_input`（补 new `wd_split_metadata_and_input_handles`）。
- [ ] `source .env.test && cargo test --lib review::store`。
- [ ] `source .env.test && cargo test --lib investigate::store`。
- [ ] `source .env.test && cargo test --test agent_review_workflow_test wd_checkpoint`（new）。
- [ ] `source .env.test && cargo test --test agent_investigate_workflow_test wd_checkpoint`（new）。
**Full-suite trigger:** T-1：共享安全物化 helper；C 由 WD-08 执行。
**Dependencies:** WD-06（普通任务完整接入）。
**Implementation write set:** `src/internal/ai/checkpoint_input.rs`, `src/internal/ai/review/runner.rs`, `src/internal/ai/review/store.rs`, `src/internal/ai/investigate/runner.rs`, `src/internal/ai/investigate/store.rs`, `src/internal/ai/workspace_isolation.rs`, `tests/agent_review_workflow_test.rs`, `tests/agent_investigate_workflow_test.rs`, `docs/commands/review.md`, `docs/commands/zh-CN/review.md`, `docs/development/commands/review.md`, `docs/commands/investigate.md`, `docs/commands/zh-CN/investigate.md`, `docs/development/commands/investigate.md`, `../libra-backend/apps/tanstack-app/content/docs/commands/review.en.md`, `../libra-backend/apps/tanstack-app/content/docs/commands/investigate.en.md`。
**Docs and compatibility impact:** 上列 review/investigate EN/zh、开发文档和网站两页同卡写 checkpoint 输入位置、只读和不迁旧数据。
**Rollback mode:** forward-only
**Estimated scope:** M
**Version increment:** N/A
**C/D coverage from:** WD-08
**Granularity:** `type=implementation; axis=checkpoint输入安全物化生命周期; recovery=按独立句柄前滚清理; complete=yes; self-contained=yes; AC=6/8; VER=5/8; landing=4; prod-files=6; scope=M; deps=WD-06; writeset=序列化于WD-06/08; release=family child; split-from=N/A; exception=N/A`

### Task WD-08: 家族聚合验证与次版本交付

**Task type:** release
**Lifecycle / Acceptance:** pending / 空
**Description:** 在同一最终 bump 树上验证全部布局行为，发布 CLI 和网站文档。
**Out of scope:** 不引入新行为；发现缺陷在原行为卡前滚或新增 FIX 卡后重新验收。
**Current evidence:** 三个版本面由 `tests/compat/version_surface_sync.rs` 权威定义；网站同步脚本和 package scripts 已核实；无本计划实现/全量/发布证据。
**Acceptance criteria:**
- [ ] WD-01..07 与 WD-02G 的最终变更均被发布树包含。
- [ ] 旧二进制拒绝矩阵在最终格式复验通过。
- [ ] 最终 FUSE 真实挂载门通过。
- [ ] 仓库/网站命令文档同步通过。
- [ ] 最终实现 Claude Code `VERDICT: PASS`。
- [ ] 版本面一致并采用 ADR-WD-05 minor。
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
- [ ] ER-06a 网站同步（backend cwd；不计特有 VER）：`python3 scripts/sync-libra-command-docs.py --libra-root ../libra --write --only worktree agent review investigate sandbox hooks upgrade`，随后同命令去掉 `--write` 检查七页；另记录全库只读检查的既有漂移。
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
**Implementation write set:** 本计划文件、`docs/development/plan/plan-status.md`、`docs/development/plan/plan-long.md`；网站同步范围控制的 `../libra-backend/scripts/sync-libra-command-docs.py`、`../libra-backend/scripts/test_sync_libra_command_docs.py`、`../libra-backend/scripts/README-command-doc-sync.md`、`docs/commands/upgrade.md`、`docs/commands/zh-CN/upgrade.md` 与七张受影响 EN 命令页（含 `../libra-backend/apps/tanstack-app/content/docs/commands/upgrade.en.md`）；如只有文档收口修正，可编辑 `COMPATIBILITY.md` 及各行为卡已列文档，须登记回原卡并重审。不得在 release 卡引入新生产行为。
**Release write set:** 以 `compat_version_surface_sync` 当次报告集合为准（当前 `Cargo.toml`, `install.sh`, `install.ps1`），工具链更新的 `Cargo.lock` 与 release artifacts。
**Docs and compatibility impact:** 原卡所有用户/网站文档逐文件结果登记本文；release notes 和 upgrade 双语/网站页明确 NAME/--path breaking change、首次以 0.31.0 打开任何仓库即写入不可回退的 2026101902 receipt、0.30.x 此后所有命令拒绝且不迁移旧数据。
**Rollback mode:** immutable-release
**Migration and rollback:** 已发布版本不撤销或覆盖；失败发布新修复版本。旧二进制禁止写新布局；网站问题在 cf 前滚。
**Estimated scope:** M
**Version increment:** minor
**Release boundary:** family release point of REL-WD01
**C/D coverage from:** self
**Granularity:** `type=release; axis=家族发布交付; recovery=不可变版本前滚发布; complete=yes; self-contained=yes; AC=10/12; VER=8/12; landing=0; prod-files=0（仅Libra生产代码，backend同步工具另列）; scope=M; deps=WD-07,DEP-WD-01,DEP-WD-02,DEP-WD-03,DEP-WD-04,DEP-WD-05; writeset=序列化于WD-07; release=family release point; split-from=N/A; exception=N/A`

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
| WD-01 | util:205/worktree_scope:75 | 布局、发现、遍历边界 | 开发 worktree 契约 | worktree_layout、wd_nested_discovery |
| WD-02G | 旧 0.30.53 agent clean 实验 | Repository receipt / role manifest | forward-only 能力门 | worktree_layout_capability、db_migration_test |
| WD-02 | worktree:98/add:2817 | 普通 CLI/registry/add | NAME/--path、v4/旧 binary | wd_name_path_lifecycle、wd_internal_add_modes |
| WD-03 | worktree-fuse:709 | FUSE/opaque/name lock | FUSE 布局与平台 | wd_fuse_opaque_boundary |
| WD-04 | workspace_isolation:127 | 描述/意图/sandbox | sandbox 根语义 | wd_attempt_intent |
| WD-05 | review/runner:470 | review/clean | review/agent 命令页 | wd_review_attempt_cleanup |
| WD-06 | investigate/runner:727 | investigate/continue | investigate/agent 命令页 | wd_investigate_resume_attempt |
| WD-07 | checkpoint_input:228 | 安全物化句柄 | checkpoint 输入位置 | wd_split_metadata_and_input_handles |
| WD-08 | 模板 ER-08/13/14 | 发布/部署/记录 | minor release notes | 全量 nextest、FUSE 实机、线上内容 |

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
| 接口变更说明不足 | 高：调用方中断 | minor、NAME/--path 示例同卡同步 | WD-02/08 |
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

## 全量门暴露的基线修复

**FIX-WD-01（capture 架构与 oracle 基线，执行中）：** 首轮 `cargo test --all` 在 `compat_agent_architecture_guard` 失败 4/64。未修改的 HEAD `85dcc50`（0.30.56）在隔离 baseline worktree 运行 `cargo test --test agent_lifecycle_event_test hook_public_contract_claude_codex_ci -- --exact` 同样失败，实测 Claude `PermissionRequest` receipt 与两条 subagent boundary 均已经产生；`6554e13`（CC-02）将这些事件接入 Observe ingress，而冻结 golden 未同步。此问题不由 worktree 迁移引起。按 Claude Code 对实际 diff 首轮 `VERDICT: FAIL` 的要求先复现/归因，再取得基线证据复审 `VERDICT: PASS`（仅限 oracle 修复）。修复写集限 `tests/agent_lifecycle_event_test.rs` 的原有 oracle 行与可逆 fixture、`tests/compat/agent_architecture_guard.rs` 的两枚指纹及历史迁移链检查、`docs/development/tracing/agent.md` 的 ADR-ACF-10 修订；同一门的原有 provider 反向依赖将 MessageDisplay 计数门从 `hooks/providers/claude/events.rs` 原样移到 `capture/ingress.rs`，`capture/extraction.rs` 的非首批 provider 改成 partial-empty fallback，避免提高 AgentKind path ratchet。该修复不改变 worktree 合约。`compat_agent_architecture_guard` 已 focused 64/64、Claude/Codex oracle 1/1；仍需最终全量、独立源码复审与发布树验证。基线日志 `/tmp/libra-wd-head-oracle-baseline.log`，首次门日志 `/tmp/libra-wd-cargo-all-final.log`，Claude 复审 `/tmp/libra-wd-acf-baseline-evidence-review.txt`。

**FIX-WD-02（全量文档契约与 config 测试环境隔离，执行中）：** 第二轮全量锁定的中文 agent 文档字面承诺已在既有 WD-04 文档写集恢复为“不新增稳定错误码”，`compat_agent_docs_contract` 34/34。第三轮全量中 `config_test::get_best_effort_reads_value_inside_repository` 初始化误用其他测试的 future-schema global config：`global_config_create_uses_global_role` 正在修改进程环境，本用例只持有 `cwd` lane。修复写集增列 `tests/command/config_test.rs`、`tests/SERIAL_REGISTRY.tsv`：读者使用独立 `ConfigDbFixture` 并持有 `cwd+env`，global/system 两个 future-schema 写者在注入期间持有 `ChangeDirGuard` 且标记 `env+cwd`。`sh tests/SERIAL_CLASSIFY.sh` 对三例的判定均与 registry 一致，`sh tests/NEXTEST_GROUPS.sh` 重生配置但 `.config/nextest.toml` 内容未变；focused 1/1、config 组 184/184、serial registry 26/26、fmt 和严格 clippy 通过。此修复仅隔离既有测试，不改变生产配置语义；仍需独立审查和最终 `cargo test --all`、nextest 两门通过，失败轮次不计完成。

## 完成判据

- [ ] 粒度审计满足 G-*，无未登记超限或写集冲突。
- [ ] WD-01..08 与 WD-02G 共九卡全部 Lifecycle=done / Acceptance=complete；A/B、review、继承 C/D 的证据均实际存在。
- [ ] 最终 bump 树 `cargo +nightly fmt --all --check`、`cargo clippy --all-targets --all-features -- -D warnings`、用户要求的 `source .env.test && cargo test --all` 与发布权威门 `source .env.test && source .env.live-test && cargo nextest run --all --no-fail-fast --retries 2` 均执行并通过；cargo test --all 不替代 nextest 发布证据；FLAKY/skipped/失败准确记录。
- [ ] 全量暴露问题已修复且全量重跑至绿；既有失败有基线复现与正式 FIX/DEFER 记录，红色全量不能收口。
- [ ] 指定 focused、feature、真实 FUSE、旧二进制 worktree/agent 消费者数据不变、故障恢复以及 final-SHA 原生 Windows scoped-input 门全部通过。
- [ ] 双语命令/网站同步、网站测试/typecheck/build、实际部署页面验收通过。
- [ ] Codex 与 Claude Code 最终审查 PASS，P0/P1 清零；P2 只有具名接受才保留。
- [ ] minor 版本面 parity、Cargo 工具链 lockfile、签名+DCO、main/cf 推送及远端 SHA、release.yml、artifact/stable 证据全部齐全。
- [ ] 所有证据写本文，无新证据文件/目录；修订历史、plan-status/plan-long 与终止链协议已履行。
