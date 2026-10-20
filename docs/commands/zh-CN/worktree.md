# `libra worktree`

管理附加到此仓库的多个工作树。

**别名：** `wt`

## 概要

```
libra worktree add <NAME> [TARGET] [--path <PATH>]
libra worktree list [--porcelain]
libra worktree lock <NAME> [--reason <text>]
libra worktree lock --path <PATH> [--reason <text>]
libra worktree unlock (<NAME> | --path <PATH>)
libra worktree move <NAME> --path <DEST>
libra worktree move --from-path <SRC> --path <DEST>
libra worktree prune
libra worktree remove (<NAME> | --path <PATH>) [--delete-dir]
libra worktree umount <path> [--cleanup]
libra worktree repair [<NAME> | --path <PATH>] --confirm
libra worktree repair [<NAME> | --path <PATH>] --migrate-layout [--dry-run] [--confirm]
libra worktree repair (<NAME> | --path <PATH>) --resolve-identity --yes
libra worktree doctor [<workspace-id>] [--limit <n>] [--cursor <cursor>]
```

## 说明

`libra worktree` 管理共享同一个仓库数据库和对象存储的多个工作树。这允许你同时拥有同一仓库的多个 checkout，适用于同时处理多个分支、编辑代码时运行构建，或隔离测试更改。

新 worktree 使用稳定的**名称**。普通 linked worktree 默认位于 `<common-storage>/worktree/linked/<NAME>`，通常就是主仓库的 `.libra/worktree/linked/<NAME>`。所有 linked worktree 共用同一个共享存储，因此从 linked worktree 或任意子目录调用 `add` 都采用同一默认根目录。主工作树仍在原目录。`--path <PATH>` 可指定完整的外部目标路径；相对路径以调用者的当前目录为基准。

名称为 1–64 个小写 ASCII 字符，满足 `[a-z0-9][a-z0-9._-]{0,63}`；不能以点结尾，也不能使用 Windows 保留设备名（包括带扩展名的设备名）。路径可以包含空格和 Unicode。普通与 FUSE worktree 共用仓库级名称空间，已分离和等待清理的条目也占用名称。已有名称始终解析为 registry 保存的实际路径，移动后也一样；再指定不同的 `--path` 会被拒绝。

每个 linked worktree 包含自己的真实 `.libra` gitdir——本地目录而非符号链接，保存各自的 `HEAD`、index 和 `HEAD` reflog，以及指向共享存储的 `commondir` 和稳定的 `worktree_id`。所有 worktree 共享数据库、对象库、branch/tag/remote refs 和配置，但检出与暂存状态隔离。主工作树的文件操作跳过其 `.libra` 元数据；托管 linked worktree 内的命令正常处理该工作树的文件。托管容器内若缺失或损坏本地身份，命令会拒绝，不会向上误解析成主工作树。

磁盘 `worktrees.json` 使用 `schema_version: 4`，条目带可选 `name`。现有 v2/v3 registry 仍可读取；正常变更时写入当前格式，保留原身份与 v3 注册代次（`epoch_counter` 和各条目的 `epoch`）。旧无名条目和主工作树不自动分配名称。本次变化不移动旧目录、不批量补名称，也不新增目录迁移命令；已有无名 worktree 可用 `--path` 选择。registry 通过原子替换写入。公开的 `worktree list` JSON 使用版本 2，与磁盘 registry 版本独立。

旧共享 `.libra` 符号链接布局仍保留原有诊断和显式 `repair --migrate-layout` 流程；该操作修复 gitdir 隔离，不把 worktree 搬到新的默认目录。旧二进制不得重写新 registry 格式；创建命名 worktree 后应继续使用兼容的 Libra 版本。

**消解 registry 身份冲突。** 旧 registry 可能存在两个条目占用同一个由路径推导的身份。变更类 worktree 操作会拒绝该歧义。`libra worktree doctor` 指出冲突条目；`libra worktree repair --path <PATH> --resolve-identity --yes` 分离指定条目，同时保留文件和 scoped 状态。随后可用 `worktree remove --path <PATH> --delete-dir` 完成删除，或用 `worktree add <NAME>` 重新挂接已有命名条目。

新建 worktree 从所选提交（默认 HEAD）填充，不复制已暂存的索引更改。重新挂接分离的 worktree 不执行 checkout，也不替换文件、HEAD 或 index。

## 选项

### 子命令：`add`

在托管目录或显式指定的目标目录创建命名 linked worktree。

| 参数 / 标志 | 说明 |
|-------------|------|
| `<NAME>` | 稳定的 worktree 名称。默认在共享存储下创建 `.libra/worktree/linked/<NAME>`。 |
| `[TARGET]` | 可选 branch 或 commit-ish。已有分支附着检出；任一 worktree 已检出该分支时拒绝。其他 commit-ish 创建分离 HEAD。未知目标在创建前拒绝；不支持远程分支猜测和 `--track`/`--no-track`。 |
| `--path <PATH>` | 完整外部目标路径，可绝对或相对调用者当前目录。必须不存在或为空目录，不能是符号链接；共享存储内只允许该名称自身的托管默认路径。它不是需要再拼接名称的父目录。 |
| `--detach` | 即使 `TARGET` 是分支，也使用分离 HEAD。 |
| `-b, --new-branch <NEW_BRANCH>` | 在 `TARGET`（省略时为源 HEAD）创建并检出新分支。已有分支被拒绝；后续创建失败会回滚新分支。 |

`--create-branch` 是 `--new-branch` 的别名。省略目标时，新 worktree 以**源提交的分离 HEAD** 创建；名称不会隐式创建同名分支。不支持 `--lock`、`--orphan`、`--no-checkout`、远程猜测和 `-B`/`--force`；创建后可用 `worktree lock` 锁定。

对于已分离的 registry 条目，`add <NAME>` 校验已保存身份和共享存储指针，然后重新挂接现有文件与暂存状态。重新挂接时拒绝 `TARGET`、`-b` 和 `--detach`，可选的 `--path` 必须等于注册路径。tombstone 条目清理成功前继续占用名称。

```bash
# 在 .libra/worktree/linked/my-feature 创建，源提交处分离 HEAD
libra worktree add my-feature
libra --json worktree add my-feature

# 检出已有分支
libra worktree add fix-1 hotfix

# 在 tag 或 commit 处分离
libra worktree add --detach probe v1.2.0

# 显式创建分支
libra worktree add -b topic topic main

# 使用这个完整外部路径，允许空格与 Unicode
libra worktree add experiment --path "../实验 workspace"
```

#### FUSE worktree

启用 `worktree-fuse` 的 Unix 构建中，`add <NAME> --fuse` 以 `.libra/worktree/fuse/<NAME>/workspace` 为挂载点，同级 `upper/` 为可写 overlay。显式 `--path` 可选择外部挂载点；位于 lower tree、upper tree、共享元数据或其他托管工作区内的目标会在创建前拒绝。内部默认目录通过 overlay 中 opaque 的 `.libra/worktree` 目录遮蔽，避免递归暴露自身挂载点、upper 或兄弟工作区。文件系统无法建立该遮蔽时拒绝挂载。

FUSE 与普通 worktree 不能占用同一名称。FUSE registry 使用 `{ "schema_version": 1, "entries": [...] }`；已知旧格式仍可读取，不移动原目录。FUSE worktree 不支持 `move`，也不支持按名称或路径修复身份（`repair <NAME>` / `repair --path <PATH>`）。注册表修复请用全局 `worktree repair --confirm`；FUSE 不采用普通 linked worktree 的私有身份布局。其 `remove` 继续使用卸载并删除 upper 的既有语义，不采用普通 worktree 的保留目录分离语义。对于新的托管默认布局，还会删除空挂载目录和容器；目录非空或清理失败时保留注册及名称供重试。旧布局和外部挂载点的目录仍保留。`umount` 继续接收实际路径，不接收 worktree 名称。

```bash
libra worktree add overlay --fuse
libra worktree umount /repo/.libra/worktree/fuse/overlay/workspace
```

### 子命令：`list`

列出所有已注册 worktrees 及其状态。`--porcelain` 输出稳定的机器可读格式：每个 worktree 输出 `worktree <path>`、它自己的 `HEAD <sha>` 行，以及 `branch <ref>` 或 `detached` 行（每个 worktree 各自拥有 HEAD），`worktree` 行后按需输出 `name <name>`，被锁定时再加 `locked [<reason>]` 行，条目间空行分隔。若某 worktree 的 HEAD 无法解析（旧的共享 `.libra` 布局，或缺失/损坏的 scope），则省略 HEAD 相关行，而不是错误地标上其它 worktree 的提交。

```bash
libra worktree list
libra worktree list --porcelain
libra --json worktree list
libra --machine worktree list
```

结构化输出使用 `worktree.list` 命令信封，`data.schema_version: 2`。每个条目报告可选 `name`、实际绝对 `path`、`kind`、`is_main`、`locked`、`lock_reason`、路径是否存在、持久化 `worktree_id`、注册代次 `epoch` 和生命周期 `state`（`active`、`detached_from_registry`、`tombstone`）。磁盘 `layout` 为 `main`、`linked-v2`、`legacy-symlink`、`missing` 或 `corrupt`；porcelain 也输出相应 `layout` 行。旧无名条目和主工作树的 JSON 省略 `name`，porcelain 也不输出 `name` 行。命名条目的人类可读形式为 `worktree <name> <actual-path>`。

### 子命令：`repair`

修复 worktree 元数据。每个会变更状态的 repair 都必须带 `--confirm`；否则命令会在取得 registry 锁或写入数据库/文件系统前拒绝。不带名称或 `--path` 时，已确认的操作会移除重复条目（相同规范路径）、确保恰好存在一个主 worktree 条目，并运行 W3 lifecycle 恢复引擎——确定性回放中断的 add/move/remove/prune intent journal（恢复过程绝不删除目录）、重试 tombstone 条目的 scoped 清理、按 registry 重建 detached 标记与 SQL lifecycle 镜像；只有实际做出更改时才写入状态文件。只有 `--migrate-layout --dry-run` 是无需确认的只读预览。

带 `--migrate-layout` 时(自 v0.19.60,W3-s3 §C.6):从**主 worktree** 把 legacy 共享 `.libra` 符号链接布局迁移到隔离布局(`--dry-run` 只报告;不带 path 迁移全部 legacy 条目)。迁移以原子 rename 安装带 journal 标识的新 gitdir(legacy 链接保留为 backup 直至校验通过),以共享 HEAD 快照播种分离 HEAD 并据其重建 private index:工作区文件绝不改动(迁移后按新 index 显示为 dirty/untracked),共享 index 的 staged 状态绝不复制——请先在主 worktree commit/stash。共享 index 存在冲突或主 scope 有进行中 sequencer 时在任何 rename 前拒绝;中断的迁移由下一次 `worktree repair --confirm` 按身份恢复。面向目标的生命周期命令(两种模式的 `worktree remove --path <PATH>` 与 `worktree repair --path <PATH>`)对 legacy-symlink 目标同样拒绝——共享符号链接会把写入导向**主**存储——须先完成迁移。

带名称或 `--path` 时（身份从 registry 读取）：依据 registry 中持久化的 stable id 恢复该 **linked** worktree 的 gitdir 身份——重写缺失或损坏的 `.libra/worktree_id`，并恢复缺失或损坏（空/不可读）的 `commondir` 指针（指向本仓库共享存储）。身份永远来自 registry，绝不猜测；`commondir` 有效地指向**另一个**存储时会被拒绝（绝不悄悄改挂仓库），且拒绝时不产生任何写入。未注册路径与主 worktree 会被拒绝；registry 仍为 legacy v1 格式时（无持久化身份）同样拒绝——先运行一次不带参数的 `libra worktree repair --confirm` 完成升级后重试。

```bash
libra worktree repair --confirm
libra --json worktree repair --confirm
libra worktree repair experiment --confirm
libra --json worktree repair --path ../experiment --confirm
```


### 子命令：`doctor`

诊断 Agent **workspace** scope。普通调用**严格只读**（plan-20260714 Part C W0
§C.11）：只报告、绝不修复——不会写入任何行、registry 条目、lease 或文件，
调用前后 registry、数据库、lease 状态与文件系统逐字节不变。诊断命令必须能
安全地在一个你尚未理解的仓库上运行——那正是你会用到它的时刻——所以裸
`doctor` 绝不自行 adopt、reclaim 或修复，任何错误提示都不会承诺裸 `doctor`
会修好什么；唯一的例外是下表中单独命名、需 `--confirm` 的变更操作。
（Libra 专有；Git 无对应命令。）

*workspace* 是 Agent runtime 接管某个 worktree 时建立的关联记录（见 `libra agent workspace list`）。人类使用 linked worktree 从不需要它，因此没有 Agent 活动的仓库不会报告任何内容。

| 参数 / 选项 | 说明 |
|-------------|------|
| `<workspace-id>` | 只诊断一个 workspace，而不是分页遍历全部。不能与 `--limit`/`--cursor` 同时使用（单 scope 不是一页）——组合使用是用法错误 `LBR-CLI-002`。 |
| `--limit <n>` | 每页最多返回的诊断条数。默认 50，上限 500。 |
| `--cursor <cursor>` | 从上一页继续：原样回传 `next_cursor` 的值。cursor 是 opaque 的；非本命令签发的 cursor 会以 `LBR-WORKTREE-001` 拒绝，而不是悄悄从第一页重来。 |
| `--adopt-capture-session <session-id>` | 明确归属一个 `legacy_unknown` 的历史 capture session。必须同时给出 `<workspace-id>` 和 `--confirm`，不能和 `--limit`/`--cursor` 组合。它是单独的变更操作；不带该选项的 doctor 始终只读。 |
| `--adopt-info-to <worktree-path>` | W0 §C.4.1.1：把仓库共享存储（main 的 `.libra/info/*`）中的 `info/exclude`/`info/attributes` 显式复制进**一个** linked worktree 自己的 gitdir——绝不自动、绝不覆盖已存在的目标文件。需要 `--confirm`。 |
| `--clear-common-info` | 删除共享存储中的 `.libra/info/exclude` 与 `info/attributes`（这些规则自 W0 起只作用于 main worktree；显式清除表示它们不应再作用于任何地方）。需要 `--confirm`。 |
| `--adopt-approved-project <legacy-project-id>` | 把 opaque `project_id` 与当前 `libra.repoid` 不一致的 Always 审批行归并到仓库规范身份（plan-20260715 W4-07）。迁移不会改写 `project_id`。需要 `--confirm`。只读 doctor 会列出可发现的 legacy ID。 |
| `--clear-approved-project <legacy-project-id>` | 删除某个 legacy（非规范）`project_id` 下的 Always 审批，不归并。拒绝作用于当前 `libra.repoid`。需要 `--confirm`。 |
| `--confirm` | 确认一项 doctor 变更操作（capture 归属、info 文件 adopt/clear、approved_permission adopt/clear）。 |

每条诊断报告该 workspace 的身份（`workspace_id`、`repo_id`、`path`、`worktree_id`）、`lease_state`（`none`/`held`/`expired`），以及 `scope_diagnostics` 发现列表。每项发现带稳定的 `code` 与 `severity`（`warning` 或 `error`）：

| Code | Severity | 含义 |
|------|----------|------|
| `foreign_repository_identity` | error | 该记录写于此前的仓库身份下：常规列表看不到它，且它会阻塞新 workspace 注册。 |
| `registry_path_mismatch` | error | worktree registry 与 workspace 记录对 scope 所在路径的说法不一致。 |
| `scope_layout_corrupt` | error | 该路径的 gitdir 布局无法识别。 |
| `workspace_orphaned` | warning | 拆除失败或 owner 消失；该 workspace 仍持有恢复状态。 |
| `lease_expired` | warning | lease 期限已过。在被显式回收之前，lease 仍属于其 owner。 |
| `workspace_path_missing` | warning | 声称的路径上没有目录。 |
| `registry_entry_missing` | warning | 没有 worktree registry 条目拥有该 scope。 |
| `registry_entry_detached` | warning | 该 worktree 已被 `worktree remove`（保留目录）注销。 |
| `registry_entry_tombstoned` | warning | 目录已删除但其 scoped 行仍待清理；`libra worktree repair --confirm` 会重试。 |
| `scope_layout_legacy_symlink` | warning | 该 worktree 仍是隔离布局之前的共享 `.libra` 符号链接布局；用 `libra worktree repair --migrate-layout --confirm` 迁移。 |

完全无法读取的 scope——无法解析的 registry、不可读的记录、缺失的仓库身份——一律以 `LBR-WORKTREE-002` fail-closed，而不是给出残缺诊断。

```bash
libra worktree doctor
libra --json worktree doctor
libra --json worktree doctor --limit 20
libra --json worktree doctor --cursor "$cursor"
libra worktree doctor 9f1c2f1e-4a0e-4c0e-9a71-6a2f4a2e0b13
```

迁移 `2026080401` 会把旧版未保存 scope 的 capture 行标成
`legacy_unknown`，而不是猜测它们属于主 worktree；因此 hook 和历史导入会
fail-closed。确认原始 session 与目标 workspace 后，唯一的归属路径是：

```bash
libra worktree doctor <workspace-id> \
  --adopt-capture-session <session-id> --confirm
```

目标 workspace 必须持有当前 live lease fence。该命令会把同一 provider
session 的全部 legacy capture 行（session、export job、import identity）归入
该仓库/worktree/workspace fence；若该 provider session 已有任何 scoped 行则
拒绝，随后写入一条不可变 audit 记录。操作不可撤销。其 JSON 使用独立的
`worktree.doctor.adopt_capture` envelope；只读 `worktree.doctor` 的分页 schema
不变。通常传入 catalog session id；若 export/import 行在 catalog session 已
清理后仍保留，则传入 provider session id 以恢复该 orphan 行。

**Legacy 共享 info 文件**（W0 §C.4.1.1）：`info/exclude` 与 `info/attributes`
改为 worktree 本地后，共享存储（main 的 `.libra/info/*`）中的文件只作用于
main worktree。存在 linked worktree 时，只读报告会在 main 条目上说明这一点，
并给出两个显式、需确认的操作（绝不自动复制）：

```bash
# 把 main 的 info/exclude + info/attributes 复制进一个 linked worktree
# 自己的 gitdir（已存在的目标文件保留、绝不覆盖）：
libra worktree doctor --adopt-info-to <worktree-path> --confirm

# 从共享存储删除它们（这些规则不再作用于任何地方）：
libra worktree doctor --clear-common-info --confirm
```

两者在任何副作用之前都要求 `--confirm`，并与其它 mutating repair 操作一样
运行在同一条 operation-log audit 边界内。`--json`/`--machine` 模式下各自使用
独立 envelope——`worktree.doctor.adopt_info`（`data.target` + `data.report`）
与 `worktree.doctor.clear_common_info`（`data.report`）——只读
`worktree.doctor` 分页 schema 保持不变。只读报告的文本条目还会列出每个
worktree 实际生效的本地 info 来源（如 `.libra/info/exclude`，双布局树还包括
`.git/info/exclude`）。

**Legacy Always 审批**（plan-20260715 W4-07）：`approved_permission` 中
`project_id` 与当前 `libra.repoid` 不一致的行对 runtime 不可见。迁移只回填
空 provenance，绝不改写 `project_id`。只读 doctor 会列出可发现的 legacy ID；
恢复必须显式且带确认：

```bash
# 把某个 legacy project_id 下的 Always 审批归并到 libra.repoid：
libra worktree doctor --adopt-approved-project <legacy-project-id> --confirm

# 或删除该 legacy bucket（不归并）：
libra worktree doctor --clear-approved-project <legacy-project-id> --confirm
```

两者在任何副作用之前都要求 `--confirm`，并运行在同一条 operation-log
audit 边界内。`--json` envelope 为 `worktree.doctor.adopt_approved_project`
与 `worktree.doctor.clear_approved_project`（`data.action`、
`data.legacy_project_id`、`data.rows_affected`）。

### 子命令：`lock`

锁定 worktree，防止 prune、move 或 remove。使用名称或 `--path` 二选一选择目标。`--reason <text>` 可记录原因，供 `list` 显示。

```bash
libra worktree lock my-feature
libra worktree lock my-feature --reason "long-running experiment"
libra --json worktree lock --path ../existing-tree --reason "long-running experiment"
```

### 子命令：`unlock`

按名称或 `--path` 解锁 worktree。对未锁定 worktree 解锁是无操作。

```bash
libra worktree unlock my-feature
libra --machine worktree unlock --path ../existing-tree
```

### 子命令：`move`

把普通 linked worktree 移到同一文件系统上的显式目标目录。名称和稳定身份不变；之后按名称调用的命令会解析为新保存路径。跨文件系统移动会在复制任何文件前拒绝，源目录及其注册记录保持原样。请选择同一文件系统内的目标，或在另一文件系统新建独立 worktree 后显式转移修改。不能移动主工作树、已锁定或 FUSE worktree。

| 参数 / 标志 | 说明 |
|-------------|------|
| `<NAME>` | 注册的源名称；与 `--from-path` 互斥。 |
| `--from-path <SRC>` | 按实际路径选择源，支持已有无名 worktree。 |
| `--path <DEST>` | 完整目标路径。不得已存在或已注册。共享存储内只允许该名称自身的托管默认路径。 |

```bash
libra worktree move my-feature --path ../my-feature-v2
libra --json worktree move --from-path ../existing-tree --path ../existing-tree-v2
```

### 子命令：`prune`

从注册表移除磁盘目录已不存在的 worktree。只有 stat 返回 NotFound 的路径才算缺失——权限错误或未挂载卷绝不会把 worktree 判为缺失。主 worktree、已锁定 worktree、tombstone 条目（由 repair 处理）以及存在进行中 rebase/cherry-pick/bisect 的 scope 绝不会被 prune。被 prune 条目的 scoped 状态清理失败时,条目保留为 `tombstone`(输出 `tombstoned` 字段)由 `libra worktree repair --confirm` 重试。

```bash
libra worktree prune
libra --machine worktree prune
```

### 子命令：`remove`

按名称或 `--path` 移除普通 worktree。默认保留磁盘目录，registry 条目转入 `detached_from_registry`。**名称继续保留**，私有 HEAD、index、工作文件和 scoped 数据库状态均不丢失。分离目录中的命令会拒绝，直到重新挂接或完成删除。

用 `libra worktree add <NAME>` 重新挂接命名条目。命令会验证目录身份和共享存储指针，不执行 checkout，也不覆盖暂存或未暂存工作。此时不能提供 `TARGET`、`-b`、`--detach`；可选 `--path` 必须等于原保存路径。

用 `--delete-dir` 完成删除。删除前检查未提交更改，并验证目录仍属于注册的 worktree；替换目录、符号链接或无效身份均拒绝。只有确定缺失的路径才算不存在，检查错误不授权删除。删除目录后再清理 scoped 状态；清理失败会保留 `tombstone` 条目和名称，由 `libra worktree repair --confirm` 重试。仅当清理完成才释放名称。主工作树、已锁定或有进行中 sequencer 的 worktree 不能移除。

| 参数 / 标志 | 说明 |
|-------------|------|
| `<NAME>` | 注册名称；与 `--path` 互斥。 |
| `--path <PATH>` | 按实际路径选择已有 worktree。 |
| `--delete-dir` | 删除干净且通过身份验证的目录和 scoped 状态，再释放注册及名称。 |

```bash
# 保留文件、暂存状态和名称
libra worktree remove my-feature
libra worktree add my-feature

# 删除干净 worktree 并释放名称
libra worktree remove my-feature --delete-dir
libra --machine worktree remove --path ../existing-tree --delete-dir
```

行为有意不同于 Git：Git 默认删除目录。Libra 默认保留目录以防意外数据丢失；`--delete-dir` 以显式 opt-in 恢复类 Git 语义。动机见 [`COMPATIBILITY.md`](../../../COMPATIBILITY.md) 和 [`compatibility/worktree-surface.md`](../../development/commands/worktree.md)。

### 子命令：`umount`

卸载 FUSE worktree 挂载点。主要用于在操作系统报告路径 busy 时清理陈旧的 Agent task worktree。该命令也接受旧 Libra task FUSE 根目录，并自动解析其 `workspace` 挂载点。`--cleanup` 只支持旧 `libra-task-worktree-fuse-*` 根目录，不删除新的命名 FUSE 容器或 task attempt；清理目标会在卸载前检查，即使命名 FUSE worktree 的名称以旧前缀开头也会拒绝。命名 FUSE worktree 请用 `worktree remove <NAME>`，review/investigate attempt 请用各自的 run 清理命令。

别名：`unmount`

| 参数 / 标志 | 说明 |
|-------------|------|
| `<path>` | FUSE 挂载点路径，或包含 `workspace` 挂载点的 Libra task worktree 根目录。 |
| `--cleanup` | 卸载后，移除 Libra task worktree 根目录。只接受 task FUSE worktree 路径。 |

```bash
libra worktree umount /repo/.libra/worktrees/tasks/libra-task-worktree-fuse-29353-id/workspace --cleanup
libra --json worktree umount /repo/.libra/worktrees/tasks/libra-task-worktree-fuse-29353-id --cleanup
```

JSON / machine 输出信封：

```json
{
  "ok": true,
  "command": "worktree.umount",
  "data": {
    "mountpoint": "/repo/.libra/worktrees/tasks/libra-task-worktree-fuse-29353-id/workspace",
    "unmounted": true,
    "cleanup_requested": true,
    "cleanup_root": "/repo/.libra/worktrees/tasks/libra-task-worktree-fuse-29353-id",
    "cleanup_root_removed": true
  }
}
```


## 常用命令

```bash
# 创建新 worktree
libra worktree add experiment

# 列出所有 worktrees
libra wt list

# 锁定 worktree 以保护它
libra wt lock experiment --reason "production hotfix in progress"

# 完成后解锁
libra wt unlock experiment

# 将 worktree 移到新位置
libra wt move experiment --path ../experiment-v2

# 清理目录已删除的 worktrees
libra wt prune

# 注销 worktree（保留磁盘文件）
libra wt remove experiment

# 修复不一致的 worktree 元数据
libra wt repair --confirm

# 只读诊断全部 Agent workspace scope
libra wt doctor
```

## 人工输出

**`worktree add`**：

```text
/Users/alice/projects/my-repo/.libra/worktree/linked/my-feature
```

**`worktree list`**：

```text
main /Users/alice/projects/my-repo
worktree my-feature /Users/alice/projects/my-repo/.libra/worktree/linked/my-feature
worktree hotfix /Users/alice/projects/my-repo/.libra/worktree/linked/hotfix [locked: production hotfix in progress]
```

**`worktree remove`**：

```text
Removed worktree '/Users/alice/projects/my-feature' from registry. Directory kept on disk.
Removed worktree '/Users/alice/projects/my-feature' from registry and deleted directory.
```

**`worktree prune`**（有陈旧条目）：

```text
Will prune 2 worktrees:
  /Users/alice/projects/old-experiment
  /Users/alice/projects/deleted-branch
Pruned 2 worktrees
```

**`worktree prune`**（没有需要 prune 的条目）：

```text
No worktrees to prune
```

## JSON 输出

`worktree add`、`lock`、`unlock`、`move`、`prune`、`remove` 和 `repair` 使用命令专用信封。`--machine` 以紧凑单行 JSON 输出相同 schema。

**`worktree.add`**：

```json
{
  "ok": true,
  "command": "worktree.add",
  "data": {
    "path": "/Users/alice/projects/my-feature",
    "already_exists": false
  }
}
```

**`worktree.list`**：

```json
{
  "ok": true,
  "command": "worktree.list",
  "data": {
    "schema_version": 2,
    "worktrees": [
      {
        "kind": "main",
        "path": "/Users/alice/projects/my-repo",
        "is_main": true,
        "locked": false,
        "lock_reason": null,
        "exists": true
      }
    ]
  }
}
```

**`worktree.lock`**：

```json
{
  "ok": true,
  "command": "worktree.lock",
  "data": {
    "path": "/Users/alice/projects/my-feature",
    "locked": true,
    "lock_reason": "long-running experiment",
    "changed": true
  }
}
```

**`worktree.unlock`**：

```json
{
  "ok": true,
  "command": "worktree.unlock",
  "data": {
    "path": "/Users/alice/projects/my-feature",
    "locked": false,
    "changed": true
  }
}
```

**`worktree.move`**：

```json
{
  "ok": true,
  "command": "worktree.move",
  "data": {
    "source": "/Users/alice/projects/my-feature",
    "destination": "/Users/alice/projects/my-feature-v2",
    "registry_updated": true,
    "disk_directory_moved": true
  }
}
```

**`worktree.prune`**：

```json
{
  "ok": true,
  "command": "worktree.prune",
  "data": {
    "pruned": ["/Users/alice/projects/old-experiment"],
    "pruned_count": 1
  }
}
```

**`worktree.remove`**：

```json
{
  "ok": true,
  "command": "worktree.remove",
  "data": {
    "path": "/Users/alice/projects/my-feature",
    "registry_removed": true,
    "disk_directory_deleted": false
  }
}
```

**`worktree.repair`**：

```json
{
  "ok": true,
  "command": "worktree.repair",
  "data": {
    "changed": true,
    "journal_recovered": 1,
    "tombstones_cleaned": 1,
    "tombstones_pending": 0,
    "notes": ["completed interrupted remove of '/abs/path/wt'"]
  }
}
```

**带路径的 `worktree.repair`**：

```json
{
  "ok": true,
  "command": "worktree.repair",
  "data": {
    "path": "/abs/path/to/experiment",
    "worktree_id": "1f0c…",
    "worktree_id_restored": true,
    "commondir_restored": true
  }
}
```

**`worktree.doctor`**（分页视图——`data.diagnostics[]` 加 `data.next_cursor`，
最后一页为 `null`）：

```json
{
  "ok": true,
  "command": "worktree.doctor",
  "data": {
    "schema_version": 1,
    "diagnostics": [
      {
        "workspace_id": "9f1c2f1e-4a0e-4c0e-9a71-6a2f4a2e0b13",
        "repo_id": "8f5b…",
        "kind": "linked",
        "state": "active",
        "path": "/Users/alice/projects/my-feature",
        "worktree_id": "1f0c…",
        "lease_state": "expired",
        "lease_owner": "agent-review-3",
        "lease_fence": 2,
        "lease_expires_at": 1785304730095,
        "scope_diagnostics": [
          {
            "code": "lease_expired",
            "severity": "warning",
            "detail": "the lease deadline passed (expires_at 1785304730095, now 1785305330095); the lease still belongs to its owner until it is explicitly reclaimed"
          }
        ]
      }
    ],
    "next_cursor": null
  }
}
```

**`worktree.doctor <workspace-id>`**（单 scope 视图——单数 `data.diagnostic`，
无分页字段）：

```json
{
  "ok": true,
  "command": "worktree.doctor",
  "data": {
    "schema_version": 1,
    "diagnostic": {
      "workspace_id": "9f1c2f1e-4a0e-4c0e-9a71-6a2f4a2e0b13",
      "repo_id": "8f5b…",
      "kind": "linked",
      "state": "active",
      "path": "/Users/alice/projects/my-feature",
      "worktree_id": "1f0c…",
      "lease_state": "held",
      "lease_owner": "agent-review-3",
      "lease_fence": 2,
      "lease_expires_at": 1785305930095,
      "scope_diagnostics": []
    }
  }
}
```

## 设计动机

### 为什么使用 JSON 文件持久化，而不是像 Git 那样使用文件系统链接？

Git 通过一组文件系统结构跟踪 worktree：主 `.git/worktrees/` 目录包含每个 worktree 的目录，里面有 `gitdir`、`HEAD` 和 `commondir` 文件，每个 linked worktree 又有一个指回去的 `.git` 文件（不是目录）。这种方式与 Git 基于文件的架构强耦合，并要求在多个位置之间仔细交叉引用。

Libra 在共享存储目录中使用单个 `worktrees.json` 文件。这有几个优势：所有 worktree 元数据位于一个可查询位置；状态通过临时文件重命名原子写入；格式也便于人类和 AI agent 检查。每个 linked worktree 拥有自己真实的 `.libra` gitdir（内含指回共享存储的 `commondir` 指针与稳定 `worktree_id`，不是符号链接），比 Git 的双向指针系统更简单。代价是 JSON 文件成为单一事实来源，必须保持一致，因此存在 `repair`。

### 为什么 lock 上有 `--reason`？

Git 的 `git worktree lock` 也支持 `--reason`，Libra 保留了这一点。锁定原因在团队环境和 AI agent 管理 worktree 时很有价值：它提供了为什么不应 prune 或 remove 该 worktree 的上下文。没有原因时，锁定 worktree 是不透明的，其他用户（或 agent）无法判断锁是否仍然相关。该原因会显示在 `list` 输出中，使锁状态自解释。

### 为什么 `remove` 不删除磁盘目录？

删除文件是不可撤销的破坏性操作。Libra 默认的 `remove` 改为**分离**：目录、其 scoped 数据库状态与身份全部保留,条目转入 `detached_from_registry` 状态,目录被冻结(其中一切命令 fail-closed 并提示 re-add/删除)。这是有意的安全选择:用户可用 `worktree add` 重新挂接,或确认后用 `--delete-dir` 完成删除。如果 worktree 包含未提交工作,这也能防止意外数据丢失。Git 的 `git worktree remove` 默认会删除目录,这曾导致工作丢失。

### 为什么 `move` 拒绝已锁定 worktree？

已锁定 worktree 表示它不应被修改。移动它会改变其文件系统路径，可能破坏其他工具、脚本或 agent 配置中对该路径的引用。用户必须先显式解锁 worktree 再移动，确保该操作是有意的。

### 为什么 `add` 从 HEAD 而不是索引填充？

创建 linked worktree 时，Libra 从 HEAD 提交恢复内容，而不是当前索引状态。这确保新 worktree 反映最后一次提交的状态，而不是只存在于原始 worktree 上下文中的已暂存未提交更改。这符合用户预期：新 worktree 从已知良好状态开始。

## 参数对比：Libra vs Git vs jj

| 操作 | Libra | Git | jj |
|------|-------|-----|----|
| 创建 worktree | `worktree add <NAME> [TARGET] [--path <PATH>]`（无目标：源 commit 分离） | `worktree add <path> [<branch>]` | `workspace add <path>` |
| 在分支上创建 | `worktree add <NAME> <branch>`（附着；任一处已检出即拒；无 DWIM） | `worktree add <path> <branch>` | `workspace add <path>`（然后 `jj edit`） |
| 创建 detached | `worktree add --detach <NAME> [<commit>]`（`add <NAME> <commit>` 亦可） | `worktree add --detach <path> <commit>` | N/A |
| 列出 worktrees | `worktree list` | `worktree list [--porcelain]` | `workspace list` |
| 锁定 | `worktree lock <NAME> [--reason]` | `worktree lock [--reason] <worktree>` | N/A |
| 解锁 | `worktree unlock <NAME>` | `worktree unlock <worktree>` | N/A |
| 移动 | `worktree move <NAME> --path <DEST>` | `worktree move <worktree> <new-path>` | N/A |
| Prune | `worktree prune` | `worktree prune [--dry-run]` | N/A（自动） |
| Remove | `worktree remove <NAME>`（分离——目录与状态保留、冻结） | `worktree remove [--force] <worktree>`（删除目录） | `workspace forget <name>` |
| Repair | `worktree repair [<NAME>] --confirm` | `worktree repair [<path>...]` | N/A |
| 别名 | `wt` | N/A | N/A |
| 每个 worktree 一个分支 | `-b <new> [<start>]` 显式创建（全回滚；无 basename 默认） | 自动（新分支或已有分支） | 自动（新 working copy commit） |
| 存储 | JSON 文件（`worktrees.json`） | 文件系统结构（`.git/worktrees/`） | Operation log |
| Worktree 链接 | 真实本地 `.libra` gitdir，内含指向共享存储的 `commondir` 指针（legacy 布局为符号链接） | 指向 `gitdir` 的 `.git` 文件 | 指向共享 `.jj` 的符号链接 |

注意：jj 使用术语 "workspace" 而不是 "worktree"。每个 workspace 会自动获得自己的 working copy commit，并且 workspaces 记录在 operation log 中。jj workspaces 比 Git worktrees 更简单，因为 jj 基于变更的模型不需要为每个 workspace 单独管理分支。

## 错误处理

| 代码 | 条件 |
|------|------|
| `LBR-REPO-001` | 不是 libra 仓库 |
| `LBR-REPO-002` | `worktrees.json` 损坏 |
| `LBR-CLI-003` | 名称无效，或显式目标进入受保护元数据或其他托管工作区 |
| `LBR-CLI-003` | 目标已存在且不是目录 |
| `LBR-CLI-003` | 没有该 worktree（lock、unlock、move、remove） |
| `LBR-CLI-003` | 不能移动或移除主 worktree |
| `LBR-CLI-003` | 不能移动或移除已锁定 worktree |
| `LBR-CLI-003` | 对非 task FUSE worktree 路径请求了 `worktree umount --cleanup` |
| `LBR-CONFLICT-002` | 目标目录已存在且非空 |
| `LBR-CONFLICT-002` | 目标已包含 `.libra` 条目 |
| `LBR-CONFLICT-002` | 目标已存在（move） |
| `LBR-CONFLICT-002` | 目标已注册为 worktree（move） |
| `LBR-CONFLICT-002` | 因 worktree 脏而拒绝 `--delete-dir` |
| `LBR-IO-001` | 读取或检查 worktree 路径/状态/status 失败 |
| `LBR-IO-002` | 写入 worktrees.json 失败 |
| `LBR-IO-002` | 从 HEAD 填充 worktree 失败 |
| `LBR-CLI-002` | `worktree doctor <workspace-id>` 与 `--limit`/`--cursor` 同时使用 |
| `LBR-CLI-003` | `worktree doctor <workspace-id>` 指向不存在的 workspace |
| `LBR-WORKTREE-001` | `worktree doctor --cursor` 收到非本命令签发（或已过期）的 cursor |
| `LBR-WORKTREE-002` | `worktree doctor` 无法读取某个 scope（registry 无法解析、记录不可读、仓库身份缺失），拒绝给出残缺诊断 |

支持统一 worktree 目录的版本首次打开仓库时，会写入仓库能力标记 4。此后 Libra 0.30.x 会拒绝该仓库中的所有命令，包括与 worktree 无关的命令；请升级到 Libra 0.31.1 或更新版本。现有 worktree 目录不会移动。
