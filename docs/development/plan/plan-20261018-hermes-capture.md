# Hermes Agent 本地会话采集计划（2026-10-18）

> **起草与审计时间：2026-10-18 02:00:57 UTC；模板v2.14。** 全仓`rg -i hermes`仅找到旧SCAP删除说明与无关发布文档，无独立provider计划、AgentKind或adapter。本计划补齐缺失方案；当前未安装`hermes`，不宣称已完成本机/真实模型验收。

> **基础依赖镜像 DEP-ACF-MIRROR：** 前置已由 ACF-09（`Lifecycle=done`、`Acceptance=complete`）交接；terminal work 必须注入共用 `CaptureFinalizePolicy`，不得私建 ledger/finalizer。本镜像只消费已完成机制，开工仍须逐文件复验当前接口。

## 文档职责、适用范围与成功定义

为NousResearch Hermes Agent建立独立source审计、Repository identity、本地普通session导入与observe-only采集，复用Libra已有ACF/SCAP/COV2机制。先准备来源/非控制协议，再公开supported capture；cloud mirror/restore未获支持时总入口拒绝。仅本地CLI coding session/cwd绑定的内容在范围内；gateway、Telegram/Discord/多租户消息、远程账号或跨用户历史不采集。

非目标：provider decryptor、扫描凭据、替Hermes执行任务、接管其tool/context/output、review/investigate launcher、从热SQLite分别复制db/WAL、修改Hermes数据库或跑其会自动迁移的PythonSessionDB构造器。

## 事实基线与当前缺口

- Libra `observed_agents/adapter.rs:26`与`registry.rs:125`无Hermes；需配对Repository CHECK/enum，不能冒用OpenCode/RPC或其它db tag。
- 本轮通过官方GitHub API冻结[NousResearch/hermes-agent@ccc6956](https://github.com/NousResearch/hermes-agent/tree/ccc695635094d7615373db267b77f894d978b1a0)。`hermes_state.py` SHA256 `4104bfd048be745035528915e56dfa69c0b0c495551ab65d639038e091459ffd`；`website/docs/user-guide/features/hooks.md` SHA256 `be1240ff77a535a69b4d9e54638da938cd75f01630c3040d8d10c75686f1cf01`。
- 官方hook文档区分observer/control/transform；`on_session_end`通常每turn触发，不能按名字映射SessionEnd；`on_session_finalize`是teardown但surface/shape不同。HM-00必须核具体callsite，不能把post_llm_call的成功触发当所有失败/中断都有事件。
- 主源码含`hermes_state_schema.py`、`hermes_state_messages.py`、`hermes_cli/plugins_dispatch.py`等，SQLite schema、profile/consent/plugin安装不由文档名称猜测。source分支HEAD不是稳定支持版本，HM-00须冻结可安装release/package与完整commit对应。
- 本机无Hermes binary；静态审计和合成机制可先行，实际supported提升被DEP-HM-03阻塞。

## 与其它计划的关系、依赖登记与实施顺序

强制采用 [四 provider 执行契约](plan-20260904.md#capture-review-20261018)。DEP-CAP-BASE/RG/OG/WRITE在本计划镜像登记：具体已交付采集owner、ordinary来源策略与四provider shared I串行；当前卡执行者为核验owner，dirty/in-progress或缺机制停止对应实现。

| DEP | direction / 产物与可用性 | Owner / 失败路径 |
|---|---|---|
| DEP-HM-01 | incoming upstream：完整commit、可安装版本、hook callsite/observer与实际source export形状由HM-00冻结 | HM-00审计者；pin/schema/ID无证则相关parser/installerHOLD，不换main猜测 |
| DEP-HM-02 | incoming cross-plan：plan-20260911 PI-05/09提供白名单/并发snapshot边界、PI-07提供安全CHECK runner，PI-01提供当时新增kind的role迁移经验；HM不复制fence owner | PI/HM交接者；PI-05未source-qualified前HM-02不写；HM-01保留当时所有kinds，不等整份Pi计划 |
| DEP-HM-03 | incoming local：匹配冻结release的Hermes binary和隔离项目/profile，明确本机模型smoke与trust授权 | HM-06执行者；当前缺binary，只有supported提升阻塞，不让计划audit假PASS运行 |
| DEP-HM-DB | incoming：plan-20260910 MIG-02/03已done/complete；当前role manifest与旧binary oracle | schema owner；role/旧reader无法复验，HM-01 blocked |
| DEP-HM-LIVE | incoming：仅HM-00的export字段/argv/只读callsite语义GO；HM-08的trust/sandbox/lease/deadline是其实施验收，真实binary行为由HM-06另验，不构成HM-08自依赖 | HM-00审计者；语义缺证HOLD，HM-08不开工，HM-06不提升supported |
| DEP-HM-DOC | incoming website：原生Libra cf及agent/cloud页面clean，部署渠道可验 | 文档writer；按四provider契约交付，不能只写仓库文档 |
| DEP-HM-RELEASE | incoming release：签名工具链与逐卡固定source C/D | 单一发布者；缺项locally-accepted/remote-pending |

**严格串行：HM-00 → HM-02 → HM-01 → HM-03 → HM-04 → HM-07 → HM-08 → HM-09 → HM-05 → HM-06。** HM-02先于任何Hermes row writer。HM-01前还须PI-07及PI-01 source交接以避免同一enum/schema/files冲突；HM-02只审计PI-05/09已交付的通用白名单与snapshot边界，先行cloud fence无需schema允许该tag。无REL家族/批量发布例外。

## 已决议设计决策与全局工程约束

- ADR-HM-01：known observer与control/transform彻底区分，默认不注册pre_tool_call/pre_llm_call/transform；注册observer不返回控制值，不以异常/timeout改变provider行为。不抓整份conversation_history作为无界hook envelope。
- ADR-HM-02：`on_session_end`按已证turn outcome映射TurnEnd/失败事实，不能过早关闭session；teardown/reset/finalize与new/resume分开。失败/中断缺终态时partial并由显式导入/既有owner recovery对账，不伪造complete。
- ADR-HM-03：本轮仅接受HM-00证实的官方JSON export，不接受SQLite/db-only或“已关闭备份”，因为没有WAL时quick_check也不能证明历史完整。已核source出现`hermes sessions export`线索，但JSON选项、profile/cwd、只读性与预算仍须HM-00核callsite，不编造参数。历史import只用用户明确提供的已证官方export。实时内容须由LiveCaptureProvider::transcript_exporter经HM-08 Hermes专用受信任/沙箱导出桥及现有lease owner执行同一官方exporter；HM-00只判定fixed argv、只读callsite、不触发热库迁移、session/cwd/profile可绑定的语义GO/HOLD；trust/sandbox/lease/2s机制由HM-08实现验收，真实binary的2s实测由HM-06完成，不能反向作为HM-08开工前置。HOLD只允许无内容metadata观察，HM-06不能supported，不把静态import当live成功。

- ADR-HM-04：profile/host/session/turn/message/tool ID联合界定scope；本计划只绑定当前本地repo/worktree，缺cwd/identity则拒绝或partial，不合并不同profile、gateway或跨用户会话。稳定ID与去重由HM-00字段矩阵证明，不按时间或display title猜测。
- ADR-HM-05：普通可读reasoning先typed redaction；unknown/encrypted-like字段剥离partial，无Hermes opaque或native issuer授权。新provider envelope不得绕过共享snapshot的原始读取限制。
- ADR-HM-06：本地only支持先于任何writer发布cloud fence，复用PI-05的冻结generation/对象闭包并发边界及当前权限/tombstone/撤权复验；不能只在后置mirror拒绝，也不能因旧快照复活已擦除内容。Repository CHECK仅走ADR-CAP-CHECK/PI-07专用连接；新增唯一ID大于当前tip，CHECK保留Pi及所有已有值，Global/System/Derived零写，未来旧reader零写拒绝；只前滚，不drop用户数据。
- 上限继承四provider契约：16MiB source、1MiB行/envelope、2s总budget、8in-flight、3次尝试共享deadline；单导入最多10000messages、1000sessions枚举上限，截断/超限有明确partial/拒绝，不悄悄完成。只写Libra-owned plugin asset/配置项，绝不替用户修改Hermes consent allowlist。

## 当前缺口与12维自审

| GAP | 当前事实/风险 | 卡 |
|---|---|---|
| GAP-HM-01 | 无Hermes identity/schema/adapter | HM-01/03，PI-07前置 |
| GAP-HM-02 | 版本/callsite/live来源未GO | HM-00、DEP-HM-LIVE |
| GAP-HM-03 | D1不支持新增kind | PI-05白名单及PI-09快照，HM-02审计 |
| GAP-HM-04 | 无普通import/worker/导出桥/scope/observer/安装 | HM-04/07/08/09/05/06 |

| 维度 | 规划判断 |
|---|---|
| 合理性 | 当前无实现，以本地ordinary范围补齐 |
| 可行性 | HM-00分别给静态与live GO/HOLD，不猜参数 |
| 粒度 | 10卡各单轴，实际≤4落点/12生产文件 |
| 依赖 | 具体PI-05/07/01与HM卡产物，无整计划空等待 |
| 完整性 | 实现/测试/文档同卡，audit只写计划 |
| 安全 | 仅已证CLI/scope/profile，gateway等零转发 |
| 正确性 | turn/session终态分开，缺ID/未知shape partial |
| 兼容 | D1旧七值不变，PreviewAgent先行、支持提升最后 |
| 数据控制流 | 共享snapshot/coordinator，intent先于对象/ref |
| 性能容量 | 分页spool与有界source/child，禁止history无界序列化 |
| 可靠性 | 幂等/崩溃/撤权及迁移child摘要全门 |
| 可维护性 | 复用CAP/PreviewAgent/runner/D1 SSOT |

## 全局工程约束与执行检查必备需求

完整采用plan-template.md的GC-01..GC-13、ER-01..ER-14及四provider契约，不删减强制门。ER-01记录基线/精确I/R，ER-02刷新source，ER-04双状态，ER-05逐卡Claude PASS，ER-06/06a逐文件文档/网站，ER-07同提交签名/DCO，ER-08每实现卡patch，ER-13/14双env nextest。ER-12唯一发布者为所选卡执行Agent，版本/tag/Release与网站cf窗口串行；未指派个人，不捏造姓名。D-CAP-STD固定workflow/job/event/ref/source取证。无REL/EX例外。

## Phase与发布分组窗口

| Phase | 卡 | 进入/退出 |
|---|---|---|
| 0 source | HM-00 | 本轮规划PASS且选卡授权；退出各能力GO/HOLD |
| 1 identity | HM-02→HM-01 | PI-05/09/07/01实际交接；退出白名单回归/无损CHECK/preview |
| 2 ordinary | HM-03→HM-04→HM-07 | source GO；退出export/import/有界snapshot/worker |
| 3 live/promotion | HM-08→HM-09→HM-05→HM-06 | LIVE_SOURCE GO/匹配binary；退出observer/本机smoke/支持提升 |

每实现卡independent release；审计no-release承载卡在卡内具名。版本面单写者，不把后继delta混入已冻结source；source-qualified后可开发下卡，formal complete仍等该卡实际C/D。失败前滚，不改已发布artifact。

## 依赖证据与超时登记补充

| DEP | 证据 | 超时/失败 |
|---|---|---|
| DEP-HM-01 | 事实基线完整commit/blob，HM-00 inline字段/callsite矩阵 | ≤2人日，无证HOLD |
| DEP-HM-02 | PI-05白名单/PI-09 spool、PI-07 runner、PI-01 signed source/逐文件handoff | 未交接/dirty/in-progress即blocked |
| DEP-HM-03 | HM-06 actual binary/version/场景/结果inline | 无binary/授权不运行，skip不通过 |
| DEP-HM-DB | MIG-02/03实际完成与PI-07门 | role/oracle/child门红则blocked |
| DEP-HM-LIVE | HM-00语义GO、HM-08 fake-exporter机制门与HM-06真实binary门 | 2s总deadline；无证metadata-only，HM-06 blocked |
| DEP-HM-DOC | cf文档postimage/sync/frontend/Claude/部署收据 | 任一失败不发布网站 |
| DEP-HM-RELEASE | D-CAP-STD SHA/ref/run/job/安装inline | 未完remote-pending |

## 任务卡粒度审计表

按实际I集逐文件/目录计，SQL只含开工分配ID的两文件；README、测试、强制文档不计生产。无EX豁免。

| ID | type | axis | AC | VER | landing | prod-files | scope | deps | release |
|---|---|---|---|---|---|---|---|---|---|
| HM-00 | audit | Hermes版本、source与观察协议审计 | 5 | 2 | 0 | 0 | M | DEP-HM-01 | no-release |
| HM-02 | audit | 现有白名单的Hermes拒绝审计 | 3 | 1 | 0 | 0 | M | HM-00、DEP-HM-02 | no-release |
| HM-01 | migration | Repository Hermes identity与role迁移 | 5 | 1 | 4 | 10 | M | HM-02、DEP-HM-DB、DEP-HM-02（PI-01 source交接） | independent |
| HM-03 | implementation | 安全Hermes source adapter | 4 | 1 | 2 | 5 | M | HM-01 | independent |
| HM-04 | implementation | Hermes普通内容归一化与显式import | 4 | 1 | 4 | 5 | M | HM-03 | independent |
| HM-07 | implementation | Hermes有界普通采集能力 | 4 | 1 | 3 | 7 | M | HM-04、DEP-CAP-BASE、DEP-CAP-RG | independent |
| HM-08 | implementation | Hermes受信任且沙箱化的导出桥 | 8 | 2 | 3 | 8 | M | HM-07、DEP-HM-LIVE | independent |
| HM-09 | implementation | Hermes本地CLI与scope准入过滤 | 6 | 1 | 2 | 5 | M | HM-08 | independent |
| HM-05 | implementation | Hermes observer plugin与现有ingress | 8 | 1 | 3 | 6 | M | HM-09、DEP-HM-LIVE | independent |
| HM-06 | implementation | Hermes采集支持提升与交付 | 5 | 1 | 2 | 4 | M | HM-05、DEP-HM-03、DEP-HM-DOC、DEP-HM-LIVE | independent |

## 测试矩阵与追溯表

| 卡 | 来源/行为 | 验证族 | 文档与恢复 |
|---|---|---|---|
| HM-00 | GAP02 source | 版本/callsite/LIVE_SOURCE矩阵 | 本计划，缺证HOLD |
| HM-02 | GAP03 PI-05 | 已有unsupported_capture_kind_族 | 本计划，无代码 |
| HM-01 | GAP01 schema | hermes_role_及PI-07 child摘要 | agent/compat，前滚 |
| HM-03 | GAP01 export | hermes_source_ | agent source限制，前滚 |
| HM-04 | GAP04 import | hermes_normalize_ | agent import/local-only，前滚 |
| HM-07 | GAP04 worker | hermes_bounded_capture_ | agent readable/partial，前滚 |
| HM-08 | GAP04受信导出 | hermes_exporter_及hermes_export_bridge_ | trust/doctor/agent/网站，前滚 |
| HM-09 | GAP04 scope | hermes_scope_admission_ | 内部策略，前滚 |
| HM-05 | GAP04 observer | hermes_observe_含gateway/unknown负向 | agent协议/恢复，前滚 |
| HM-06 | GAP04 supported | roster与实际本机binary smoke | agent/cloud/网站/安装，前滚 |

## 故障恢复矩阵

| 故障 | 可接受状态 | 恢复/禁止结果 |
|---|---|---|
| source/live无证 | 对应HOLD | 回HM-00，不猜export参数 |
| gateway/unknown scope | 零转发 | 固定诊断，不把私有消息归仓库 |
| export timeout/malformed | partial/pending | 显式重导，不读热DB/不false complete |
| migration失败 | rollback且child摘要不变 | discard专用连接，不drop用户事实 |
| child/ref/DB中断 | durable intent/已有pending | 幂等replay，未知trace不上云 |
| erase/撤权变化 | 拒绝发布冻结snapshot | 重新快照，不复活旧body |
| release/网站失败 | locally-accepted/remote-pending | 固定source前滚，不伪PASS |

## 性能与容量表

| 路径 | 上限 | 退化 |
|---|---|---|
| source | 16MiB、每行1MiB、10000messages | 超限partial/拒绝 |
| 枚举 | 1000sessions | 显式续页/拒绝无界遍历 |
| exporter/child | 2s、8in-flight、3尝试共享deadline | kill/reap与安全pending |
| cloud snapshot | 同PI-05按页spool，内存按页 | 磁盘不足remote前失败 |

## 当前任务卡

### Task HM-00: Hermes版本、source与观察协议审计

**Task type:** audit

**Lifecycle / Acceptance:** pending / 空

**Description:** 冻结官方source与可安装release对应，给每项能力GO/HOLD；不安装Hermes、不读真实会话。

**Current evidence:** `src/internal/ai/observed_agents/adapter.rs:26`、`preview.rs:31-65`、`capture/extraction.rs:426`、`src/internal/db/migration.rs:615-653`；官方hermes_state.py:123和hooks.md:457/471-472由HM-00补callsite证明。

**Out of scope:** 未登记的公共命令、provider 私有执行器、解密、凭据探测和其它计划的行为重构。审计卡不修改生产代码。

**Acceptance criteria:**

- [ ] 完整source/release/package映射或HOLD记录，当前未安装事实保留。
- [ ] 逐callsite区分turn结束/真实session teardown/失败/中断/重置/子agent，核对identity和observer返回语义。
- [ ] 逐callsite判定export的argv/JSON/只读性/无热库迁移/identity绑定语义GO-HOLD；HM-08负责沙箱/lease/2s机制，HM-06负责真实binary实测，不要求本审计证明尚未实现机制。
- [ ] plugin发现/配置/consent/owner-only路径有精确source证据，observer注册不会更改tool/context/message。
- [ ] source字段分ordinary/unknown opaque/unsupported并给合成fixture矩阵；缺证能力HOLD，后继不得凭名字实现。

**Verification:**

- [ ] 人工核读固定commit的`hermes_state*.py`、`hermes_cli/plugins*.py`及官方hooks文档；版本、callsite、GO矩阵写本计划。
- [ ] `libra status --short --branch`与`libra diff --name-only`逐项对照开工基线，证明仅审计增量且未覆盖既有改动

**Full-suite trigger:** N/A（只读审计，无代码变更）

**Dependencies:** DEP-HM-01

**Blocking condition:** 依赖产物缺失、source pin 不可复验、目标文件有未交接写者或本卡任一安全门失败即 blocked；计划 PASS 只放行选定卡的工作，不替代后继的实际依赖与验收。

**Deliverables:** source/GO-HOLD审计矩阵和结论仅写本计划，不创建独立证据文件或fixture。

**Implementation write set:** `docs/development/plan/plan-20261018-hermes-capture.md`；`docs/development/plan/plan-status.md`仅本卡状态；测试仅运行，不授权修改代码、fixture或Cargo配置。

**Release write set:** N/A

**Docs and compatibility impact:** 仅本计划审计/状态，不修改fixture/代码/配置或公开行为。

**Rollback mode:** revert

**Migration and rollback:** 撤回本卡文档/合成 fixture，不删除历史事实或用户数据。

**Security and privacy / Performance budget:** 强制继承 [四 provider 执行契约](plan-20260904.md#capture-review-20261018) 的来源、脱敏、权限、预算与恢复边界；本卡 AC 收紧时取更严格值。

**Estimated scope:** M（审计时间箱≤2人日，超时记录具名HOLD，不扩大为实现）

**Version increment:** N/A

**Release boundary:** no-release

**C/D coverage from:** HM-01（具体后继release承载文档渠道）；本audit/spike自身signed main/CI仍实际登记，不bump/tag/artifact。

**Granularity:** `type=audit; axis=Hermes版本、source与观察协议审计; recovery=revert; complete=yes; self-contained=yes; AC=5; VER=2; landing=0; prod-files=0; scope=M; deps=DEP-HM-01; writeset=本计划链及DEP-CAP-WRITE逐文件串行; release=no-release; split-from=本轮G-09修订表; exception=N/A`

### Task HM-02: 现有白名单的Hermes拒绝审计

**Task type:** audit

**Lifecycle / Acceptance:** pending / 空

**Description:** 只审计回归PI-05白名单同样拒绝Hermes/future tag；不新增fence或改生产代码。

**Current evidence:** `src/internal/ai/observed_agents/adapter.rs:26`、`preview.rs:31-65`、`capture/extraction.rs:426`、`src/internal/db/migration.rs:615-653`；官方hermes_state.py:123和hooks.md:457/471-472由HM-00补callsite证明。

**Out of scope:** 未登记的公共命令、provider 私有执行器、解密、凭据探测和其它计划的行为重构。审计卡不修改生产代码。

**Acceptance criteria:**

- [ ] 任意Hermes row时所有D1/R2/ensure/sync mutation零副作用，稳定conflict/128。
- [ ] preflight后并发新writer不能进入冻结对象闭包，已有Pi与七provider回归绿。
- [ ] force/JSON/progress不绕过；不改D1 CHECK或restore图。

**Verification:**

- [ ] `source .env.test && cargo test --test command_test cloud_test::unsupported_capture_kind_ -- --test-threads=1`（只运行既有或前驱实现卡已交付测试，匹配必须非零）

**Full-suite trigger:** N/A（只读审计，无代码变更）

**Dependencies:** HM-00、DEP-HM-02

**Blocking condition:** 依赖产物缺失、source pin 不可复验、目标文件有未交接写者或本卡任一安全门失败即 blocked；计划 PASS 只放行选定卡的工作，不替代后继的实际依赖与验收。

**Deliverables:** source/GO-HOLD审计矩阵和结论仅写本计划，不创建独立证据文件或fixture。

**Implementation write set:** `docs/development/plan/plan-20261018-hermes-capture.md`；`docs/development/plan/plan-status.md`仅本卡状态；测试仅运行，不授权修改代码、fixture或Cargo配置。

**Release write set:** N/A

**Docs and compatibility impact:** 仅本计划审计/状态，不修改fixture/代码/配置或公开行为。

**Rollback mode:** revert

**Migration and rollback:** 保留已写 session、checkpoint、schema receipt 和旧格式读能力，停用新增入口后前滚修复；不得用 down/drop/重写 traces 消除用户事实。迁移卡还须执行本计划数据库角色合同。

**Security and privacy / Performance budget:** 强制继承 [四 provider 执行契约](plan-20260904.md#capture-review-20261018) 的来源、脱敏、权限、预算与恢复边界；本卡 AC 收紧时取更严格值。

**Estimated scope:** M

**Version increment:** N/A

**Release boundary:** no-release

**C/D coverage from:** HM-01（具体后继release承载文档渠道）；本audit/spike自身signed main/CI仍实际登记，不bump/tag/artifact。

**Granularity:** `type=audit; axis=现有白名单的Hermes拒绝审计; recovery=revert; complete=yes; self-contained=yes; AC=3; VER=1; landing=0; prod-files=0; scope=M; deps=HM-00、DEP-HM-02; writeset=本计划链及DEP-CAP-WRITE逐文件串行; release=no-release; split-from=本轮G-09修订表; exception=N/A`

### Task HM-01: Repository Hermes identity与role迁移

**Task type:** migration

**Lifecycle / Acceptance:** pending / 空

**Description:** 自洽增加Hermes enum/registry stub及Repository CHECK，仍unsupported。

**Current evidence:** `src/internal/ai/observed_agents/adapter.rs:26`、`preview.rs:31-65`、`capture/extraction.rs:426`、`src/internal/db/migration.rs:615-653`；官方hermes_state.py:123和hooks.md:457/471-472由HM-00补callsite证明。

**Out of scope:** 未登记的公共命令、provider 私有执行器、解密、凭据探测和其它计划的行为重构。本卡不重构无关owner或扩张其它能力。

**Acceptance criteria:**

- [ ] 迁移ID高于当前tip且唯一，CHECK只扩展当时完整集合，保留Pi。
- [ ] enum/db/CLI/adapter穷尽映射与CHECK配对，stub禁用capture/安装/launch。
- [ ] 只走PI-07专用连接；全部级联child预置数据的正常/每步失败/崩溃摘要保留，FK/index/trigger完整，重复up幂等。
- [ ] Global/System/Derived零schema/receipt写；旧binary future receipt零写拒绝。
- [ ] 有Hermes数据禁止down/drop，未知kind继续拒绝。

**Verification:**

- [ ] `source .env.test && cargo test --test agent_capture_migration_test hermes_role_ -- --test-threads=1`（拟新增过滤名；须列出实际匹配并确认非零）

**Full-suite trigger:** T-1/T-4（共享采集、存储、CLI 或测试契约）；执行四 provider 契约的完整双环境 nextest 门

**Dependencies:** HM-02、DEP-HM-DB、DEP-HM-02（PI-01 source交接）

**Blocking condition:** 依赖产物缺失、source pin 不可复验、目标文件有未交接写者或本卡任一安全门失败即 blocked；计划 PASS 只放行选定卡的工作，不替代后继的实际依赖与验收。

**Deliverables:** 下列路径中的本卡单一行为、回归测试及同卡文档；同名拟新增测试在实现时落地。

**Implementation write set:** `sql/migrations/`（仅开工分配并登记的up/down两文件）、`sql/migrations/README.md`、`src/internal/db/migration.rs`、`src/internal/db/schema.rs`、`src/internal/ai/observed_agents/adapter.rs`、`src/internal/ai/observed_agents/registry.rs`、`src/internal/ai/observed_agents/mod.rs`、`src/internal/ai/observed_agents/preview.rs`、`tests/agent_capture_migration_test.rs`、`docs/commands/agent.md`、`docs/commands/zh-CN/agent.md`、`docs/development/commands/agent.md`、`docs/development/tracing/agent.md`、`COMPATIBILITY.md`、`/Volumes/Sea/libra-backend/apps/tanstack-app/content/docs/commands/agent.en.md`、`tests/INDEX.md`、`Cargo.toml`、`src/internal/ai/capture/extraction.rs`、`src/internal/ai/observed_agents/extract.rs`、`tests/compat/agent_docs_contract.rs`、`tests/compat/agent_capability_matrix_pin.rs`、`tests/compat/agent_architecture_guard.rs`；本计划与 `docs/development/plan/plan-status.md` 仅更新本卡状态。

**Release write set:** Inherited：`compat_version_surface_sync` 权威集合 + 工具链生成的 `Cargo.lock` + release artifacts

**Docs and compatibility impact:** 上述 EN/zh-CN/development/COMPATIBILITY 与网站页面逐文件同步并执行网站交付门。 新稳定错误码才同步 `docs/error-codes.md`；拟新增 target 须同卡在 `Cargo.toml`、`tests/INDEX.md` 注册，compat target 另更新 `tests/compat/README.md`。

**Rollback mode:** forward-only

**Migration and rollback:** 保留已写 session、checkpoint、schema receipt 和旧格式读能力，停用新增入口后前滚修复；不得用 down/drop/重写 traces 消除用户事实。迁移卡还须执行本计划数据库角色合同。

**Security and privacy / Performance budget:** 强制继承 [四 provider 执行契约](plan-20260904.md#capture-review-20261018) 的来源、脱敏、权限、预算与恢复边界；本卡 AC 收紧时取更严格值。

**Estimated scope:** M

**Version increment:** patch

**Release boundary:** independent

**C/D coverage from:** self；四 provider 执行契约的逐卡 C/D

**D 组:** `plan-20260904.md` D-CAP-STD；workflow/event/ref/source SHA与全部实际job逐卡取证，未完remote-pending。

**Granularity:** `type=migration; axis=Repository Hermes identity与role迁移; recovery=forward-only; complete=yes; self-contained=yes; AC=5; VER=1; landing=4; prod-files=10; scope=M; deps=HM-02、DEP-HM-DB、DEP-HM-02（PI-01 source交接）; writeset=本计划链及DEP-CAP-WRITE逐文件串行; release=independent; split-from=本轮G-09修订表; exception=N/A`

### Task HM-03: 安全Hermes source adapter

**Task type:** implementation

**Lifecycle / Acceptance:** pending / 空

**Description:** 注册builtin/hermes.rs替换preview stub，只读取已证官方export，内部adapter不开放安装。

**Current evidence:** `src/internal/ai/observed_agents/adapter.rs:26`、`preview.rs:31-65`、`capture/extraction.rs:426`、`src/internal/db/migration.rs:615-653`；官方hermes_state.py:123和hooks.md:457/471-472由HM-00补callsite证明。

**Out of scope:** 未登记的公共命令、provider 私有执行器、解密、凭据探测和其它计划的行为重构。本卡不重构无关owner或扩张其它能力。

**Acceptance criteria:**

- [ ] 只接受HM-00 GO的格式与版本；无安全官方export则blocked，不尝试热db复制。
- [ ] consent/root/no-follow/regular先于bytes，SQLite/db-only输入一律拒绝；export shape/identity/来源声明不合矩阵即拒绝。
- [ ] profile/session/cwd当前repo/worktree匹配，gateway/remote/cross-user拒绝。
- [ ] source/row/枚举预算受限，busy/oversize/漂移返回partial或拒绝，不显示原始SQL/body/path。

**Verification:**

- [ ] `source .env.test && cargo test --test agent_import_test hermes_source_ -- --test-threads=1`（拟新增过滤名；须列出实际匹配并确认非零）

**Full-suite trigger:** T-1/T-4（共享采集、存储、CLI 或测试契约）；执行四 provider 契约的完整双环境 nextest 门

**Dependencies:** HM-01

**Blocking condition:** 依赖产物缺失、source pin 不可复验、目标文件有未交接写者或本卡任一安全门失败即 blocked；计划 PASS 只放行选定卡的工作，不替代后继的实际依赖与验收。

**Deliverables:** 下列路径中的本卡单一行为、回归测试及同卡文档；同名拟新增测试在实现时落地。

**Implementation write set:** `src/internal/ai/observed_agents/builtin/hermes.rs`、`src/internal/ai/observed_agents/transcript_source.rs`、`src/internal/ai/observed_agents/mod.rs`、`tests/agent_import_test.rs`、`docs/commands/agent.md`、`docs/commands/zh-CN/agent.md`、`docs/development/commands/agent.md`、`docs/development/tracing/agent.md`、`COMPATIBILITY.md`、`/Volumes/Sea/libra-backend/apps/tanstack-app/content/docs/commands/agent.en.md`、`tests/INDEX.md`、`Cargo.toml`、`tests/compat/agent_docs_contract.rs`、`src/internal/ai/observed_agents/builtin/mod.rs`、`src/internal/ai/observed_agents/preview.rs`；本计划与 `docs/development/plan/plan-status.md` 仅更新本卡状态。

**Release write set:** Inherited：`compat_version_surface_sync` 权威集合 + 工具链生成的 `Cargo.lock` + release artifacts

**Docs and compatibility impact:** 上述 EN/zh-CN/development/COMPATIBILITY 与网站页面逐文件同步并执行网站交付门。 新稳定错误码才同步 `docs/error-codes.md`；拟新增 target 须同卡在 `Cargo.toml`、`tests/INDEX.md` 注册，compat target 另更新 `tests/compat/README.md`。

**Rollback mode:** forward-only

**Migration and rollback:** 保留已写 session、checkpoint、schema receipt 和旧格式读能力，停用新增入口后前滚修复；不得用 down/drop/重写 traces 消除用户事实。迁移卡还须执行本计划数据库角色合同。

**Security and privacy / Performance budget:** 强制继承 [四 provider 执行契约](plan-20260904.md#capture-review-20261018) 的来源、脱敏、权限、预算与恢复边界；本卡 AC 收紧时取更严格值。

**Estimated scope:** M

**Version increment:** patch

**Release boundary:** independent

**C/D coverage from:** self；四 provider 执行契约的逐卡 C/D

**D 组:** `plan-20260904.md` D-CAP-STD；workflow/event/ref/source SHA与全部实际job逐卡取证，未完remote-pending。

**Granularity:** `type=implementation; axis=安全Hermes source adapter; recovery=forward-only; complete=yes; self-contained=yes; AC=4; VER=1; landing=2; prod-files=5; scope=M; deps=HM-01; writeset=本计划链及DEP-CAP-WRITE逐文件串行; release=independent; split-from=本轮G-09修订表; exception=N/A`

### Task HM-04: Hermes普通内容归一化与显式import

**Task type:** implementation

**Lifecycle / Acceptance:** pending / 空

**Description:** 将授权source写入既有snapshot/coverage/claim store，保持local-only。

**Current evidence:** `src/internal/ai/observed_agents/adapter.rs:26`、`preview.rs:31-65`、`capture/extraction.rs:426`、`src/internal/db/migration.rs:615-653`；官方hermes_state.py:123和hooks.md:457/471-472由HM-00补callsite证明。

**Out of scope:** 未登记的公共命令、provider 私有执行器、解密、凭据探测和其它计划的行为重构。本卡不重构无关owner或扩张其它能力。

**Acceptance criteria:**

- [ ] 消息按已证稳定ordinal/turn排序；镜像/重试/失败usage不重复，未知usage不是0。
- [ ] prompt/assistant/tool先typed redaction；thinking在HM-07前显式unsupported/partial，unknown opaque剥离partial。
- [ ] 同session/turn/branch重放与crash恢复不重复checkpoint，失败不false complete。
- [ ] 跨profile/cwd/无证parent不关联，session/intent先提交再写对象/ref，导入不提升supported或launcher。

**Verification:**

- [ ] `source .env.test && cargo test --test agent_import_test hermes_normalize_ -- --test-threads=1`（拟新增过滤名；须列出实际匹配并确认非零）

**Full-suite trigger:** T-1/T-4（共享采集、存储、CLI 或测试契约）；执行四 provider 契约的完整双环境 nextest 门

**Dependencies:** HM-03

**Blocking condition:** 依赖产物缺失、source pin 不可复验、目标文件有未交接写者或本卡任一安全门失败即 blocked；计划 PASS 只放行选定卡的工作，不替代后继的实际依赖与验收。

**Deliverables:** 下列路径中的本卡单一行为、回归测试及同卡文档；同名拟新增测试在实现时落地。

**Implementation write set:** `src/internal/ai/observed_agents/builtin/hermes.rs`、`src/internal/ai/observed_agents/coverage.rs`、`src/internal/ai/observed_agents/extract.rs`、`src/internal/ai/agent_import.rs`、`src/command/agent/import.rs`、`tests/agent_import_test.rs`、`docs/commands/agent.md`、`docs/commands/zh-CN/agent.md`、`docs/development/commands/agent.md`、`docs/development/tracing/agent.md`、`COMPATIBILITY.md`、`/Volumes/Sea/libra-backend/apps/tanstack-app/content/docs/commands/agent.en.md`、`tests/INDEX.md`、`Cargo.toml`、`tests/compat/agent_docs_contract.rs`；本计划与 `docs/development/plan/plan-status.md` 仅更新本卡状态。

**Release write set:** Inherited：`compat_version_surface_sync` 权威集合 + 工具链生成的 `Cargo.lock` + release artifacts

**Docs and compatibility impact:** 上述 EN/zh-CN/development/COMPATIBILITY 与网站页面逐文件同步并执行网站交付门。 新稳定错误码才同步 `docs/error-codes.md`；拟新增 target 须同卡在 `Cargo.toml`、`tests/INDEX.md` 注册，compat target 另更新 `tests/compat/README.md`。

**Rollback mode:** forward-only

**Migration and rollback:** 保留已写 session、checkpoint、schema receipt 和旧格式读能力，停用新增入口后前滚修复；不得用 down/drop/重写 traces 消除用户事实。迁移卡还须执行本计划数据库角色合同。

**Security and privacy / Performance budget:** 强制继承 [四 provider 执行契约](plan-20260904.md#capture-review-20261018) 的来源、脱敏、权限、预算与恢复边界；本卡 AC 收紧时取更严格值。

**Estimated scope:** M

**Version increment:** patch

**Release boundary:** independent

**C/D coverage from:** self；四 provider 执行契约的逐卡 C/D

**D 组:** `plan-20260904.md` D-CAP-STD；workflow/event/ref/source SHA与全部实际job逐卡取证，未完remote-pending。

**Granularity:** `type=implementation; axis=Hermes普通内容归一化与显式import; recovery=forward-only; complete=yes; self-contained=yes; AC=4; VER=1; landing=4; prod-files=5; scope=M; deps=HM-03; writeset=本计划链及DEP-CAP-WRITE逐文件串行; release=independent; split-from=本轮G-09修订表; exception=N/A`

### Task HM-07: Hermes有界普通采集能力

**Task type:** implementation

**Lifecycle / Acceptance:** pending / 空

**Description:** 自HM-04/05拆出snapshot/worker与readable接线；实际live source仍受DEP-HM-LIVE约束。

**Current evidence:** `src/internal/ai/observed_agents/adapter.rs:26`、`preview.rs:31-65`、`capture/extraction.rs:426`、`src/internal/db/migration.rs:615-653`；官方hermes_state.py:123和hooks.md:457/471-472由HM-00补callsite证明。

**Out of scope:** 未登记的公共命令、provider 私有执行器、解密、凭据探测和其它计划的行为重构。本卡不重构无关owner或扩张其它能力。

**Acceptance criteria:**

- [ ] 注册Hermes readable tag/normalizer/worker封闭映射，旧provider wire不变，无opaque/decrypt能力。
- [ ] snapshot只接已证export，先剥unknown opaque标partial再typed redaction，thinking不冒充assistant answer。
- [ ] worker只接redacted bytes，预算/取消/坏frame安全，ingress/snapshot/coordinator/store保持单owner。
- [ ] 同export的import/live-capability合成路径一致；无真实export GO则live HOLD，不开放supported。

**Verification:**

- [ ] `source .env.test && cargo test --test agent_transcript_intelligence_test hermes_bounded_capture_ -- --test-threads=1`（拟新增过滤名；须列出实际匹配并确认非零）

**Full-suite trigger:** T-1/T-4（共享采集、存储、CLI 或测试契约）；执行四 provider 契约的完整双环境 nextest 门

**Dependencies:** HM-04、DEP-CAP-BASE、DEP-CAP-RG

**Blocking condition:** 依赖产物缺失、source pin 不可复验、目标文件有未交接写者或本卡任一安全门失败即 blocked；计划 PASS 只放行选定卡的工作，不替代后继的实际依赖与验收。

**Deliverables:** 下列路径中的本卡单一行为、回归测试及同卡文档；同名拟新增测试在实现时落地。

**Implementation write set:** `src/internal/ai/observed_agents/reasoning.rs`、`src/internal/ai/observed_agents/coverage.rs`、`src/internal/ai/observed_agents/extract.rs`、`src/internal/ai/observed_agents/live_capture.rs`、`src/internal/ai/observed_agents/builtin/hermes.rs`、`src/internal/ai/capture/extraction.rs`、`src/internal/ai/capture/snapshot.rs`、`tests/agent_transcript_intelligence_test.rs`、`docs/commands/agent.md`、`docs/commands/zh-CN/agent.md`、`docs/development/commands/agent.md`、`docs/development/tracing/agent.md`、`COMPATIBILITY.md`、`/Volumes/Sea/libra-backend/apps/tanstack-app/content/docs/commands/agent.en.md`、`tests/INDEX.md`、`Cargo.toml`、`tests/compat/agent_docs_contract.rs`；本计划与 `docs/development/plan/plan-status.md` 仅更新本卡状态。

**Release write set:** Inherited：`compat_version_surface_sync` 权威集合 + 工具链生成的 `Cargo.lock` + release artifacts

**Docs and compatibility impact:** 上述 EN/zh-CN/development/COMPATIBILITY 与网站页面逐文件同步并执行网站交付门。 新稳定错误码才同步 `docs/error-codes.md`；拟新增 target 须同卡在 `Cargo.toml`、`tests/INDEX.md` 注册，compat target 另更新 `tests/compat/README.md`。

**Rollback mode:** forward-only

**Migration and rollback:** 保留已写 session、checkpoint、schema receipt 和旧格式读能力，停用新增入口后前滚修复；不得用 down/drop/重写 traces 消除用户事实。迁移卡还须执行本计划数据库角色合同。

**Security and privacy / Performance budget:** 强制继承 [四 provider 执行契约](plan-20260904.md#capture-review-20261018) 的来源、脱敏、权限、预算与恢复边界；本卡 AC 收紧时取更严格值。

**Estimated scope:** M

**Version increment:** patch

**Release boundary:** independent

**C/D coverage from:** self；四 provider 执行契约的逐卡 C/D

**D 组:** `plan-20260904.md` D-CAP-STD；workflow/event/ref/source SHA与全部实际job逐卡取证，未完remote-pending。

**Granularity:** `type=implementation; axis=Hermes有界普通采集能力; recovery=forward-only; complete=yes; self-contained=yes; AC=4; VER=1; landing=3; prod-files=7; scope=M; deps=HM-04、DEP-CAP-BASE、DEP-CAP-RG; writeset=本计划链及DEP-CAP-WRITE逐文件串行; release=independent; split-from=本轮G-09修订表; exception=N/A`

### Task HM-08: Hermes受信任且沙箱化的导出桥

**Task type:** implementation

**Lifecycle / Acceptance:** pending / 空

**Description:** 承接ADR-HM-03的真实执行边界；现有OpenCode专用导出桥不复用为Hermes授权。

**Current evidence:** `src/internal/ai/observed_agents/adapter.rs:26`、`preview.rs:31-65`、`capture/extraction.rs:426`、`src/internal/db/migration.rs:615-653`；官方hermes_state.py:123和hooks.md:457/471-472由HM-00补callsite证明。

**Out of scope:** 未登记的公共命令、provider 私有执行器、解密、凭据探测和其它计划的行为重构。本卡不重构无关owner或扩张其它能力。

**Acceptance criteria:**

- [ ] 显式Hermes exporter trust绑定闭合启动descriptor：canonical launcher、解释器/运行时、package/profile根与被测摘要；漂移拒绝，不借OpenCode信任。
- [ ] Python启动环境按封闭allowlist重建，运行时加载覆盖不得继承。
- [ ] 独立Hermes沙箱profile禁止网络和非scratch写入，仅允许已审package/profile数据根；不能放宽OpenCode profile。
- [ ] 官方export只用HM-00核定的fixed argv与session，2s总deadline。
- [ ] 通过既有LiveTranscriptExporter/lease owner运行，未持lease不启动。
- [ ] 输出仅经授权source关联进入snapshot，stdout/stderr/log不泄露raw内容。
- [ ] 公开trust/doctor的Hermes分支、Display、EN/zh/网站与compat同卡验证。
- [ ] 真实live验收前只算机制source-qualified，受信任本地用户边界不等于native来源证明或same-UID攻击保证。

**Verification:**

- [ ] `source .env.test && cargo test --test command_test agent_rpc_trust_test::hermes_exporter_ -- --test-threads=1`（拟新增过滤名；须列出实际匹配并确认非零）
- [ ] `source .env.test && cargo test --test agent_hook_crash_test hermes_export_bridge_ -- --test-threads=1`（拟新增过滤名；须列出实际匹配并确认非零）

**Full-suite trigger:** T-1/T-4（共享采集、存储、CLI 或测试契约）；执行四 provider 契约的完整双环境 nextest 门

**Dependencies:** HM-07、DEP-HM-LIVE

**Blocking condition:** 依赖产物缺失、source pin 不可复验、目标文件有未交接写者或本卡任一安全门失败即 blocked；计划 PASS 只放行选定卡的工作，不替代后继的实际依赖与验收。

**Deliverables:** 下列路径中的本卡单一行为、回归测试及同卡文档；同名拟新增测试在实现时落地。

**Implementation write set:** `src/internal/ai/observed_agents/hermes_export.rs`、`src/internal/ai/observed_agents/trust.rs`、`src/internal/ai/observed_agents/mod.rs`、`src/internal/ai/observed_agents/live_capture.rs`、`src/internal/ai/sandbox/hermes_export.rs`、`src/internal/ai/sandbox/mod.rs`、`src/command/agent/rpc.rs`、`src/command/agent/doctor.rs`、`tests/command/agent_rpc_trust_test.rs`、`tests/agent_hook_crash_test.rs`、`docs/commands/agent.md`、`docs/commands/zh-CN/agent.md`、`docs/development/commands/agent.md`、`docs/development/tracing/agent.md`、`COMPATIBILITY.md`、`/Volumes/Sea/libra-backend/apps/tanstack-app/content/docs/commands/agent.en.md`、`tests/INDEX.md`、`Cargo.toml`、`tests/compat/agent_docs_contract.rs`；本计划与 `docs/development/plan/plan-status.md` 仅更新本卡状态。

**Release write set:** Inherited：`compat_version_surface_sync` 权威集合 + 工具链生成的 `Cargo.lock` + release artifacts

**Docs and compatibility impact:** 上述 EN/zh-CN/development/COMPATIBILITY 与网站页面逐文件同步并执行网站交付门。 新稳定错误码才同步 `docs/error-codes.md`；拟新增 target 须同卡在 `Cargo.toml`、`tests/INDEX.md` 注册，compat target 另更新 `tests/compat/README.md`。

**Rollback mode:** forward-only

**Migration and rollback:** 保留已写 session、checkpoint、schema receipt 和旧格式读能力，停用新增入口后前滚修复；不得用 down/drop/重写 traces 消除用户事实。迁移卡还须执行本计划数据库角色合同。

**Security and privacy / Performance budget:** 强制继承 [四 provider 执行契约](plan-20260904.md#capture-review-20261018) 的来源、脱敏、权限、预算与恢复边界；本卡 AC 收紧时取更严格值。

**Estimated scope:** M

**Version increment:** patch

**Release boundary:** independent

**C/D coverage from:** self；四 provider 执行契约的逐卡 C/D

**D 组:** `plan-20260904.md` D-CAP-STD；workflow/event/ref/source SHA与全部实际job逐卡取证，未完remote-pending。

**Granularity:** `type=implementation; axis=Hermes受信任且沙箱化的导出桥; recovery=forward-only; complete=yes; self-contained=yes; AC=8; VER=2; landing=3; prod-files=8; scope=M; deps=HM-07、DEP-HM-LIVE; writeset=本计划链及DEP-CAP-WRITE逐文件串行; release=independent; split-from=本轮G-09修订表; exception=N/A`

### Task HM-09: Hermes本地CLI与scope准入过滤

**Task type:** implementation

**Lifecycle / Acceptance:** pending / 空

**Description:** 自HM-05拆出的纯平台/profile/cwd准入轴，生产者与receiver使用同一冻结策略，拒绝发生在正文序列化/读取前。本卡仅在providers/mod.rs声明pub mod hermes，提供可调用纯parser/scope API，不注册HookProvider；HM-05之后才接线注册。

**Current evidence:** `src/internal/ai/observed_agents/adapter.rs:26`、`preview.rs:31-65`、`capture/extraction.rs:426`、`src/internal/db/migration.rs:615-653`；官方hermes_state.py:123和hooks.md:457/471-472由HM-00补callsite证明。

**Out of scope:** 未登记的公共命令、provider 私有执行器、解密、凭据探测和其它计划的行为重构。本卡不重构无关owner或扩张其它能力。

**Acceptance criteria:**

- [ ] 仅HM-00已证的本地CLI platform可进入。
- [ ] 缺cwd拒绝。
- [ ] cwd必须等于现有CAP已验证的仓库/worktree事实。
- [ ] profile必须等于HM-08受信descriptor所绑定profile，不从全局默认或event猜根。
- [ ] 拒绝路径不建立capture row。
- [ ] producer与receiver的准入策略由同一fixture矩阵生成并一致。

**Verification:**

- [ ] `source .env.test && cargo test --lib internal::ai::hooks::providers::hermes::scope::tests::hermes_scope_admission_ -- --test-threads=1`（拟新增过滤名；须列出实际匹配并确认非零）

**Full-suite trigger:** T-1/T-4（共享采集、存储、CLI 或测试契约）；执行四 provider 契约的完整双环境 nextest 门

**Dependencies:** HM-08

**Blocking condition:** 依赖产物缺失、source pin 不可复验、目标文件有未交接写者或本卡任一安全门失败即 blocked；计划 PASS 只放行选定卡的工作，不替代后继的实际依赖与验收。

**Deliverables:** 下列路径中的本卡单一行为、回归测试及同卡文档；同名拟新增测试在实现时落地。

**Implementation write set:** `src/internal/ai/hooks/providers/hermes/scope.rs`、`src/internal/ai/hooks/providers/hermes/parser.rs`、`src/internal/ai/hooks/providers/hermes/mod.rs`、`src/internal/ai/hooks/providers/hermes/libra_observer.py`、`docs/development/tracing/agent.md`、`tests/INDEX.md`、`Cargo.toml`、`src/internal/ai/hooks/providers/mod.rs`；本计划与 `docs/development/plan/plan-status.md` 仅更新本卡状态。

**Release write set:** Inherited：`compat_version_surface_sync` 权威集合 + 工具链生成的 `Cargo.lock` + release artifacts

**Docs and compatibility impact:** 只更新内部契约；public roster/help/JSON 不变并有负向断言。 新稳定错误码才同步 `docs/error-codes.md`；拟新增 target 须同卡在 `Cargo.toml`、`tests/INDEX.md` 注册，compat target 另更新 `tests/compat/README.md`。

**Rollback mode:** forward-only

**Migration and rollback:** 保留已写 session、checkpoint、schema receipt 和旧格式读能力，停用新增入口后前滚修复；不得用 down/drop/重写 traces 消除用户事实。迁移卡还须执行本计划数据库角色合同。

**Security and privacy / Performance budget:** 强制继承 [四 provider 执行契约](plan-20260904.md#capture-review-20261018) 的来源、脱敏、权限、预算与恢复边界；本卡 AC 收紧时取更严格值。

**Estimated scope:** M

**Version increment:** patch

**Release boundary:** independent

**C/D coverage from:** self；四 provider 执行契约的逐卡 C/D

**D 组:** `plan-20260904.md` D-CAP-STD；workflow/event/ref/source SHA与全部实际job逐卡取证，未完remote-pending。

**Granularity:** `type=implementation; axis=Hermes本地CLI与scope准入过滤; recovery=forward-only; complete=yes; self-contained=yes; AC=6; VER=1; landing=2; prod-files=5; scope=M; deps=HM-08; writeset=本计划链及DEP-CAP-WRITE逐文件串行; release=independent; split-from=本轮G-09修订表; exception=N/A`

### Task HM-05: Hermes observer plugin与现有ingress

**Task type:** implementation

**Lifecycle / Acceptance:** pending / 空

**Description:** 用已证明observer注册fixed-argv/stdin协议，接现有ACF；安装曝光留HM-06。

**Current evidence:** `src/internal/ai/observed_agents/adapter.rs:26`、`preview.rs:31-65`、`capture/extraction.rs:426`、`src/internal/db/migration.rs:615-653`；官方hermes_state.py:123和hooks.md:457/471-472由HM-00补callsite证明。

**Out of scope:** 未登记的公共命令、provider 私有执行器、解密、凭据探测和其它计划的行为重构。本卡不重构无关owner或扩张其它能力。

**Acceptance criteria:**

- [ ] 只注册HM-00已证observer，返回值无控制含义。
- [ ] turn outcome不伪造成SessionEnd。
- [ ] 稳定session/turn/message/phase键重放幂等。
- [ ] stdin只含冻结的allowlist字段。
- [ ] child生命周期满足共享契约第5条资源合同。
- [ ] source只经HM-08桥交给snapshot，plugin不读热DB。
- [ ] terminal只注入既有CaptureFinalizePolicy。
- [ ] 只有HM-09已准入的context才能摄取。

**Verification:**

- [ ] `source .env.test && cargo test --test agent_lifecycle_event_test hermes_observe_ -- --test-threads=1`（拟新增过滤名；须列出实际匹配并确认非零）

**Full-suite trigger:** T-1/T-4（共享采集、存储、CLI 或测试契约）；执行四 provider 契约的完整双环境 nextest 门

**Dependencies:** HM-09、DEP-HM-LIVE

**Blocking condition:** 依赖产物缺失、source pin 不可复验、目标文件有未交接写者或本卡任一安全门失败即 blocked；计划 PASS 只放行选定卡的工作，不替代后继的实际依赖与验收。

**Deliverables:** 下列路径中的本卡单一行为、回归测试及同卡文档；同名拟新增测试在实现时落地。

**Implementation write set:** `src/internal/ai/hooks/providers/hermes/mod.rs`、`src/internal/ai/hooks/providers/hermes/parser.rs`、`src/internal/ai/hooks/providers/hermes/settings.rs`、`src/internal/ai/hooks/providers/hermes/libra_observer.py`、`src/internal/ai/hooks/providers/mod.rs`、`src/command/agent/hooks.rs`、`tests/agent_lifecycle_event_test.rs`、`docs/commands/agent.md`、`docs/commands/zh-CN/agent.md`、`docs/development/commands/agent.md`、`docs/development/tracing/agent.md`、`COMPATIBILITY.md`、`/Volumes/Sea/libra-backend/apps/tanstack-app/content/docs/commands/agent.en.md`、`tests/INDEX.md`、`Cargo.toml`、`tests/compat/agent_docs_contract.rs`；本计划与 `docs/development/plan/plan-status.md` 仅更新本卡状态。

**Release write set:** Inherited：`compat_version_surface_sync` 权威集合 + 工具链生成的 `Cargo.lock` + release artifacts

**Docs and compatibility impact:** 上述 EN/zh-CN/development/COMPATIBILITY 与网站页面逐文件同步并执行网站交付门。 新稳定错误码才同步 `docs/error-codes.md`；拟新增 target 须同卡在 `Cargo.toml`、`tests/INDEX.md` 注册，compat target 另更新 `tests/compat/README.md`。

**Rollback mode:** forward-only

**Migration and rollback:** 保留已写 session、checkpoint、schema receipt 和旧格式读能力，停用新增入口后前滚修复；不得用 down/drop/重写 traces 消除用户事实。迁移卡还须执行本计划数据库角色合同。

**Security and privacy / Performance budget:** 强制继承 [四 provider 执行契约](plan-20260904.md#capture-review-20261018) 的来源、脱敏、权限、预算与恢复边界；本卡 AC 收紧时取更严格值。

**Estimated scope:** M

**Version increment:** patch

**Release boundary:** independent

**C/D coverage from:** self；四 provider 执行契约的逐卡 C/D

**D 组:** `plan-20260904.md` D-CAP-STD；workflow/event/ref/source SHA与全部实际job逐卡取证，未完remote-pending。

**Granularity:** `type=implementation; axis=Hermes observer plugin与现有ingress; recovery=forward-only; complete=yes; self-contained=yes; AC=8; VER=1; landing=3; prod-files=6; scope=M; deps=HM-09、DEP-HM-LIVE; writeset=本计划链及DEP-CAP-WRITE逐文件串行; release=independent; split-from=本轮G-09修订表; exception=N/A`

### Task HM-06: Hermes采集支持提升与交付

**Task type:** implementation

**Lifecycle / Acceptance:** pending / 空

**Description:** 匹配版本真实smoke后才公开capture enable/disable/list。

**Current evidence:** `src/internal/ai/observed_agents/adapter.rs:26`、`preview.rs:31-65`、`capture/extraction.rs:426`、`src/internal/db/migration.rs:615-653`；官方hermes_state.py:123和hooks.md:457/471-472由HM-00补callsite证明。

**Out of scope:** 未登记的公共命令、provider 私有执行器、解密、凭据探测和其它计划的行为重构。本卡不重构无关owner或扩张其它能力。

**Acceptance criteria:**

- [ ] install/update/remove仅marker+hash确认为Libra-owned asset，unmanaged/漂移文件不覆盖。
- [ ] 缺binary/config时bare enable安全skip；unsupported时explicit enable无副作用。
- [ ] 隔离本机真实普通session通过install/capture/checkpoint/doctor/disable，失败/中断恢复分别验证。
- [ ] supported/capability/roster/help与docs同批，review/investigate launchable保持false。
- [ ] cloud-local-only、supported pin、覆盖缺口与恢复写入agent/cloud文档及网站；CI只用fake。

**Verification:**

- [ ] `source .env.test && cargo test --test command_test agent_roster_test::hermes_ -- --test-threads=1`（拟新增过滤名；须列出实际匹配并确认非零）

**Full-suite trigger:** T-1/T-4（共享采集、存储、CLI 或测试契约）；执行四 provider 契约的完整双环境 nextest 门

**Dependencies:** HM-05、DEP-HM-03、DEP-HM-DOC、DEP-HM-LIVE

**Blocking condition:** 依赖产物缺失、source pin 不可复验、目标文件有未交接写者或本卡任一安全门失败即 blocked；计划 PASS 只放行选定卡的工作，不替代后继的实际依赖与验收。

**Deliverables:** 下列路径中的本卡单一行为、回归测试及同卡文档；同名拟新增测试在实现时落地。

**Implementation write set:** `src/internal/ai/observed_agents/registry.rs`、`src/command/agent/mod.rs`、`src/command/agent/list.rs`、`src/command/agent/doctor.rs`、`tests/command/agent_roster_test.rs`、`docs/commands/agent.md`、`docs/commands/zh-CN/agent.md`、`docs/development/commands/agent.md`、`docs/development/tracing/agent.md`、`COMPATIBILITY.md`、`/Volumes/Sea/libra-backend/apps/tanstack-app/content/docs/commands/agent.en.md`、`docs/commands/cloud.md`、`docs/commands/zh-CN/cloud.md`、`docs/development/commands/cloud.md`、`/Volumes/Sea/libra-backend/apps/tanstack-app/content/docs/commands/cloud.en.md`、`tests/INDEX.md`、`Cargo.toml`、`tests/compat/agent_docs_contract.rs`、`tests/compat/agent_capability_matrix_pin.rs`、`tests/compat/agent_architecture_guard.rs`；本计划与 `docs/development/plan/plan-status.md` 仅更新本卡状态。

**Release write set:** Inherited：`compat_version_surface_sync` 权威集合 + 工具链生成的 `Cargo.lock` + release artifacts

**Docs and compatibility impact:** 上述 EN/zh-CN/development/COMPATIBILITY 与网站页面逐文件同步并执行网站交付门。 新稳定错误码才同步 `docs/error-codes.md`；拟新增 target 须同卡在 `Cargo.toml`、`tests/INDEX.md` 注册，compat target 另更新 `tests/compat/README.md`。

**Rollback mode:** forward-only

**Migration and rollback:** 保留已写 session、checkpoint、schema receipt 和旧格式读能力，停用新增入口后前滚修复；不得用 down/drop/重写 traces 消除用户事实。迁移卡还须执行本计划数据库角色合同。

**Security and privacy / Performance budget:** 强制继承 [四 provider 执行契约](plan-20260904.md#capture-review-20261018) 的来源、脱敏、权限、预算与恢复边界；本卡 AC 收紧时取更严格值。

**Estimated scope:** M

**Version increment:** patch

**Release boundary:** independent

**C/D coverage from:** self；四 provider 执行契约的逐卡 C/D

**D 组:** `plan-20260904.md` D-CAP-STD；workflow/event/ref/source SHA与全部实际job逐卡取证，未完remote-pending。

**Granularity:** `type=implementation; axis=Hermes采集支持提升与交付; recovery=forward-only; complete=yes; self-contained=yes; AC=5; VER=1; landing=2; prod-files=4; scope=M; deps=HM-05、DEP-HM-03、DEP-HM-DOC、DEP-HM-LIVE; writeset=本计划链及DEP-CAP-WRITE逐文件串行; release=independent; split-from=本轮G-09修订表; exception=N/A`

## 测试矩阵与追溯表

每张卡的 Current evidence → AC → Verification 是本计划追溯链。审计核 source/version/字段矩阵；配置卡核 no-follow、ownership、安装/卸载；parser 卡核合成 source、重放、截断；落库卡核 scope/redaction、事务/claim/CAS、崩溃恢复；公开卡核 CLI/JSON/docs/网站；迁移卡核 role/旧 reader/数据保留。每个新过滤名均是待实现测试，不能把当前零匹配视为成功。`--exact` 只用于已确认完整测试名，模块过滤不加 `--exact`。

## 里程碑验收与回滚

审计卡给逐能力 GO/HOLD/NO-GO，不把未知事项交给实现者猜测；后继只消费明确 GO 的范围。配置或 parser 失败保持旧入口并报告 partial；pending/replay 失败保持既有事实和恢复键；发布失败前滚，不能把 locally-accepted 记为 complete。恢复动作与每卡 Rollback mode 一致。

## 风险登记与性能容量摘要

上游漂移、输入伪造、缺失稳定 identity、半行/坏尾、并发 source 变化、重复事件、写入中断、预算耗尽和普通 sink 泄露分别由 source pin、scope/授权、拒绝无证关联、partial、同 snapshot、幂等键、故障注入、有界资源及 canary 负向测试覆盖。预算见 [四 provider 执行契约](plan-20260904.md#capture-review-20261018)；不能承诺任意宿主 I/O 的绝对实时上限。

## 兼容与文档收口

每卡 I 集中的文档与测试索引是交付内容；计划修订自身没有新的产品行为，不提前修改 agent/cloud 公共文案或网站。未来公开行为卡须按四 provider 契约同步并验证网站。默认收集同意与 raw-export 授权继续沿用；不宣称支持尚未验证的 provider、ciphertext 或 launcher。

## 非目标与延后项

provider decryptor、寻找密钥、跨 provider 仿造 native issuer、来源不明的 opaque bytes 准入均不在本计划。缺来源的能力保留 HOLD 与 partial，不能据此撤销来源门或宣称完成。

## 完成判据

- [ ] HM-00, HM-02, HM-01, HM-03, HM-04, HM-07, HM-08, HM-09, HM-05, HM-06 均按本修订逐卡实际验收，Lifecycle=done / Acceptance=complete；本次计划审查不增加实现完成数。
- [ ] 当前最终源码执行 `cargo +nightly fmt --all --check`、`cargo clippy --all-targets --all-features -- -D warnings`、`source .env.test && source .env.live-test && cargo nextest run --all --no-fail-fast --retries 2`；前滚修复发现的全部失败后重跑至绿。
- [ ] 同一最终实现经 Claude Code PASS，逐卡签名/DCO main 交付、四平台 release、安装、网站及安全 CI 的实际 C/D 已记录；Source/本机合格与远端完成分开。
- [ ] 所有审核/验收内容写入本计划；末端纯记账提交自己的 CI 在下一次 plan-status 例行同步补记 SHA/run/conclusion/UTC，不为自引用制造无限收口提交（模板 v2.14 预登记）。

## Codex review log

2026-10-18 02:00:57 UTC，本轮根 Agent 依据 OpenCode ordinary 最终范围、当前实现和一手 source 自审；已修订上游 pin、shared capture owner、来源门、执行顺序、可恢复性与文档交付。本轮对当前规范正文自审 `VERDICT: PASS`（规划可执行性）；内部DAG/AC-VER/原始已完成块保留已核验。来源GO、真实provider smoke、逐卡实现及C/D仍须后续实际验收；Claude本轮结论另列，不以本条代替。

## Claude review log

实际Claude Code 2.1.300 / claude-opus-5-5；同一只读session `095e26d7-0019-4a37-9c97-527341b76e98`。仅Read/Glob/Grep，无文件修改/执行/委派/真实home读取。R1/R2/R3原始FAIL保留，绝不由后续PASS倒改。

| 轮次 | 实际意见与处置 | raw SHA256 | report SHA256 |
|---|---|---|---|
| R1 | 四份FAIL；迁移级联P1及规划P2，后续修订关闭 | `ca3d16af131d971f0128de4c350dc4777b4e4c87db70fea1f472d2d8a851aeb0` | `a822c2d39a41b22b7551c23ea9e909711e09226d5c1b1d3ca5b6df1fe7d37ad5` |
| R2 | 四份FAIL；P1已关闭，剩P2，后续修订关闭 | `36ead2d99595b1a67eac5d5952ce8c0af5ca38921b9a545aaa0c21439e8cbf15` | `9ae155fc7142f00c7552e253378cf03a574da1e0cb061be9bde8092fe4a56ce5` |
| R3 | CX/CC/PI PASS，HM/总FAIL；两P2，R4修订 | `a798e21ab58af05a16443f3b0a3c42ba616921cc4637d8435b4942cfa5ac4f25` | `fdd6e50e6ec59957aaec22b0fb31594202522b3e7d5878055298facf2c8de52b` |
| R4 | 四份及总VERDICT: PASS；非阻断建议作窄补充，R5复核 | `9233b369618dff2c975b408e7d5d29b63f83a2667640bbefe20871ccd39e7d77` | `586a8b35a0afb9a575907485116425f8659090659f9e23772c1b27668f989f04` |
| R5（最终） | 四份及总VERDICT: PASS；R4范围复用＋五项窄核通过，root采纳 | `06e9bc29ae44c1dcc9d4db00c8a90ca1799cf9bb652980e181b7e3fad57772f4` | `e2f40302bada484aaf0963f620205f213d635d94b18d7ece91374d081462b805` |

R4实际四份及总PASS，0阻断。随后补CLI互斥镜像（含issues/483）、Hermes专用桥措辞、journal erase失效/清理语义并去重写集；R5已逐项确认无新增阻断。2026-10-18 03:20:47 UTC：root独立复核后判定本修订规划PASS并采纳Claude R5；四入口审计卡可在选卡授权后按依赖执行，59卡实现完成数仍为0。


R5输入manifest SHA256 `1fdd95bebb3fea118db887ed81c9c85f3742ff4fb8e39fd4b839371d99b6f326`（七文件）；本计划被审输入SHA256 `566a3bea22df22b50dad498ba5e7adfd9a4a9d2379e74ed1f3d4c2b05e25e9cd`。终审后只登记实际意见、验证摘要、索引状态与修订时间格式，不改变被审规范。Claude PASS不代表Source A/真实live/provider smoke或未来卡C/D已经通过；来源及外部HOLD继续生效。

### 本轮计划改动的实际验证（2026-10-18 03:11:44 UTC登记）

- `cargo +nightly fmt --all --check`：exit0。
- `cargo clippy --all-targets --all-features -- -D warnings`：exit0。
- 双env文档守卫：architecture64/64、docs33/33，共97/97，exit0。
- 双env完整nextest：`Summary [1360.925s] 10262 tests run: 10262 passed (12 slow, 1 leaky), 23 skipped`，exit0；运行包含编译/枚举共2476.26s。LEAK对应既有`command::branch::tests::branch_error_display_pins_static_message_variants`的输出管道标记；skip按原输出保留，不说0LEAK，不替代未来真实provider/source验收。
- 上述测试是当前未修改Rust源码的现有机制/计划读者验证；不是59张未来卡已实现或来源GO。


最终窄补后双env架构64/64＋文档33/33再次实际PASS（97/97，exit0，3.30s），raw SHA256 `6af0a8b4fa48b778ced0eb363fd65c8d1a8752efbd44c9a64ce4099b687187ea`。既有QA raw SHA256：strict clippy `9e45f0588de5e23828fb929df622c6146e99415aabafebbde927e33d9d0b9420`；完整nextest `f848cc543bddd50af116face20b03fb6840cda1e43a4ef49220b31cc7751634a`。本次只修改计划/索引文档，没有修改Rust/fixture/test源码；完整默认套件的LEAK/skip观察如上保留。

## 修订历史

| 时间 | 触发 | 拆分 | 协调范围 |
|---|---|---|---|
| 2026-10-18 03:02:15 UTC | R2剩余P2与G-09 | PI-05→PI-05/PI-09；PI-03→PI-03/PI-10；HM-05→HM-08/HM-09/HM-05 | DAG、依赖、I集、粒度/测试/追溯、里程碑与索引同步；只改计划，未增加实现完成数 |
| 2026-10-18 02:42:09 UTC（本轮R2修订） | Claude R1 FAIL后按G-09拆分 | PI-01→PI-01/PI-07；PI-03/04→PI-03/PI-08/PI-04；HM-04/05→HM-04/HM-07/HM-05；HM-02改audit | DAG/依赖/写集/测试矩阵/粒度表/里程碑与两个索引同步；无实现完成数增加 |

| 时间 | 修订 | 范围 |
|---|---|---|
| 2026-10-18 02:00:57 UTC | 按当前 OpenCode 方案 C 与 ACF/COV2 实现复核；按模板 v2.14 增量修订未完成卡，不倒改历史实测 | HM 当前卡、依赖、索引与验收 |
