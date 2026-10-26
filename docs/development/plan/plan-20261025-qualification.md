# Libra 2026-10-25 来源资格前滚修复

## 2026-10-26 跨机器接续入口（当前要求）

用户本轮明确要求：「请把 Libra 需要的前置任务写入 docs/development/plan/plan-20261025-qualification.md 这个文件中，把要求的任务写明细，然后单独提交，我会在其他的机器上启动一个 agent 完成」；随后指定「libra 的交接任务卡，请在 main 分支提交，我也会在 main 分支开发」。本次交接只提交本文件到 `main`，不携带程序候选、版本修改或原 scoped-traces 三份治理 WIP。接收 Agent 在本机定位真实目录，用原生 Libra 拉取并核验最新 main 后接续；不得复制旧机器绝对路径。

本文的任务定义和交接审查不等于实现准入、验收通过或新增发布豁免。用户已改为仅调用本地 `claude` 命令审查，不调用 API，不使用隔离 Codex。后文带旧日期的双审、feature branch/PR、旧版本、预算和 pending 字样按原时点保留；当前开发分支遵守本次用户 `main` 指令，质量门、签名、无 force 和实际 D 不变。远端保护若要求 PR/他人批准，记录真实阻塞并按保护规则处理，不能 admin bypass。

### 当前事实、范围与任务表

文档准备时已只读核验远端 `main=b73e98a77ecc7c3db1ea905ab55478785d76c041`，版本 `0.30.87`。这是核对时点，不是要求接收方退回的固定基线；本交接提交及其后新增提交均须保留。FIX01 的 PR635 合并为 `f33be958dcc3266df3a6ab796ed90d1766f3cd93`，v0.30.87 完整发布已核；PR636 合并为 b73，104 项项目 Rustdoc warnings 已修。`proc-macro-error2 2.0.1` 的 future-incompatibility 提示按用户要求排除，不修改第三方库。

| 前置事项 | 当前状态 | 接收 Agent 必须交付的结果 |
|---|---|---|
| FIX-LTR-01 | done/complete | 按当前来源限域消费已有 schema doctor 修复，保留原签名、C/D 与失败历史；不重做已完成卡 |
| DEP-QUAL-F2-ATTRIB / DEP-QUAL-F2-CLOSE | 归属待审 / 关闭未解决，暂定 P1 | 先作可为unknown的只读归属决定，再另核关闭证据；查明 `command::reset_test::reset_pathspec_from_file_missing_path_errors` 的真实 stderr/退出；不能根据别的用例推断为 SQLite 锁定 |
| FIX-LTR-05P | family child，已在 main `8a115ba` | 存在性探测第一切片已推送；实现审查 dc579753 PASS；不消耗 v0.30.88 |
| FIX-LTR-05Q | family child，已在 main `c0373cb` | show、maintenance、command restore、Blob::save 已推送；准入 3de3e11b，实现审查 cf7454d4 PASS；不消耗 v0.30.88 |
| FIX-LTR-05R | family child，已在 main `80ef760` | snapshot、operation store、operation restore、repack 已推送；准入 54f3e8cf，实现审查 146781d3 PASS；不消耗 v0.30.88 |
| FIX-LTR-05S | family child，已在 main `6177646` | client_storage 的索引补登记与修订后缀类型过滤已推送；准入 71cf457d，实现审查 d6849287 PASS；不消耗 v0.30.88 |
| FIX-LTR-05T | family child，已在 main `e728736` | am 的本地对象搜索已推送；准入 986c352a，实现审查 02aa713c PASS；不消耗 v0.30.88 |
| FIX-LTR-05U | family child，已在 main `1d3c8d2` | tiered 与 remote 的 search_checked 已推送；准入 c1d476b0，实现审查 bc600c5b PASS；不消耗 v0.30.88 |
| FIX-LTR-05V | family child，已在 main `835cfc5` | fetch AutoFollow 批量探测已推送；准入 95af8652，实现审查 72196c7f PASS；不消耗 v0.30.88 |
| FIX-LTR-05 | 新登记，blocked/空 | 修复配置解析失败被错误当作 LocalStorage 成功的单一行为轴；具体卡见下；发布例外已由用户原话批准为仅 v0.30.88，设计 ER03 要求先拆卡，本卡仍不得先实施 |
| FIX-LTR-04 | blocked/locally-accepted | 只保留两行 cfg 候选与旧 A/源审的有限效力；F1/F2 关闭后重新准入、完整新 C/D，消除两方法 release warning |
| FIX-LTR-02 | blocked/locally-accepted | 在 FIX04 发布后精确恢复索引结清 helper 候选；保留原断言并完成新来源完整 C/D |
| FIX-LTR-03 | pending/空 | 修复 MCP 测试拥有的暂停 worker 的清理，完成自己的 C/D，随后执行计划最终资格与受控整合 |
| F3 / RISK-QUAL-STDIN | 具名 P2，未修 | `subagent_content` fixture 的 BrokenPipe/时序风险单独处理；不能混入 FIX05；若升级为 P1 或阻断当前门，先另立 FIX 卡和依赖 |
| DOC-LTR-01 | locally-accepted | PR636 的文档修复与实际 main D 已发生，仍按原声明由 FIX03 覆盖；不因本交接晋升 |
| DOC-LTR-02 | 本次任务定义交接 | 仅本文件、main、signed/DCO、实际本地 Claude；无产品版本发布，覆盖由 FIX03 承载 |

新增 FIX05 后，本外部计划实现卡为 **1/5**；DOC01、DOC02 单独记账。原 Libra scoped-traces 计划仍 **0/26**，不增加其分母；Mega2 已完成 MTR103，**23/118**，联合 **23/144**。卡片数不表示工时比例。两仓原计划、plan-status 的本轮单文件交接后置同步由 root 在既有协调写集中完成；用户本次明确只提交此文件，不能为满足索引同步而混入原工作区整份 plan-status。

### 实际失败与可移植证据

FIX04 C1 的 11 个命令机械退出 0、10761 个最终通过、21 个原 ignored，不能替代 review。该轮存在 4 个 flaky 身份、5 次失败尝试；Claude session `c7f9fbae-734f-416c-9d9d-3650ede3f824` 的四项结论均 FAIL，root 已记录 STOP。原失败版本四面已精确退出 0.30.87；未提交该失败树、未发布 0.30.88。旧 A1 的两项 flake、37 项 runtime 覆盖缺口、C1 的 F1/F2、旧 FIX3 的诊断 3/3、A/B 8/8 与 B8 失败全部保留，迁机不重置。

| 证据 | 可核事实 / SHA256 |
|---|---|
| F1 源文件 | b73 的 `src/utils/client_storage.rs` SHA256 `c5a60476681ab325cd70b41634b1fc8df29536a8f94bcc13e2b311a34a576b4d`；`create_storage_backend` 将 typed resolver 的 `Other` 错误送入 `storage_config_resolution_fallback`；后者无条件 stderr Warning 并返回 LocalStorage；repository config open 的锁超时为 200ms |
| F1 原失败观察 | fetch 的 machine-error 用例实际收到 `SqliteError code 5: database is locked` 的额外 Warning，破坏 JSON 错误输出；尚未证明历史锁 owner/SQL，也不能称所有 SQLite 错误均来自此处 |
| F2 源文件与缺口 | b73 `tests/command/reset_test.rs` SHA256 `72dbdbc3ff385198b21814cf26512fd757ddc256bc5fc99fa8213dc02f026cd3`；原 TRY1 JSON parse 失败、TRY2 PASS，原失败未输出 subprocess stderr；历史根因未知，不能靠新一次 PASS 关闭 |
| 原 C1 review | output SHA256 `b110df54dbe47f75636ef98fbbabc7a1f4e4e74c1585ae207844a4930b13cafd`；结论 FAIL，不追写成 PASS |
| 发布次序审查 | 本地 Claude `2891a30b-0b77-417a-b18e-c18591bc9236`：`SOURCE_BASELINE_VERDICT: PASS`、`RELEASE_ORDER_VERDICT: FAIL`、`VERDICT: FAIL`；manifest `724b5d4f137838722582d2689122b9fad98e6c340475ce7a411c865ced19a1bd`，output `72719067e04e448dcf9bd5c3ad47281e27f44e1cd6fb9cf1c9c59f215cbe96fe` |
| FIX04 保存候选 | ingress 基线 SHA256 `6ef6f6aa2d4dbf2374f9764039f83a981c8140838d4e4903eaa220500e48cd2c`；仅在 `from_payload` 与 `from_payload_with_hint_policy` 定义前加 `#[cfg(any(test, debug_assertions))]`，候选 SHA256 `2b7aa33698c2072d160cc6bea7801ffb8fb180a13bb0ebe78901ff888b7bb778`；不能将其提前放入 FIX05 |
| FIX04 迁机前保存像 | 原native stash `c763911e0c73699ad5f739309c203da3945a7405`包含两cfg候选、本计划旧稿SHA256 `5316f6478095374fdb33150504386105bae85de209fa2e3955bc9ea5439e8fa0`及status旧稿 `8d2d1193cea451230e84be04971ad5ea321f083eebdd0c603e730acabb3621cc`；只在当前交付机器，未推送且接收方不能假定可用。恢复只能精确取当前卡路径，不能stash-pop整份治理。 |
| FIX02 保存候选 | 原 native stash `988ded20adb6be7ebbc43ad195fd55768ed69a26` 仅在旧机器；不能假定远端含该对象。按 FIX02 卡的函数/断言重建时先核当前来源并重新审查，不把新文件当旧已测候选 |

这些摘要可以随仓库迁移，但不是原始输出本身。接收方如需 carry-forward，必须取得并核验原件/manifest/脱敏衍生件关系；只有摘要或旧 `/tmp` 路径时记录「原件不可得」，不能重造历史 PASS。原 SHA256 `217cea13519e3b9e8e23e818e669f8f73c5b5391921018f460a68a32a186484b` 的更早交接包已丢失，不是本次可获取产物。本机旧绝对路径只作历史定位线索；当前执行合同、工具链、数据库、服务、凭据与预算须在接收机器重新核验。

### 发布顺序与待决事项

合法恢复的**候选**顺序是：

`FIX01(done) → FIX05 → FIX04 → FIX02 → FIX03 → 计划最终资格 → 终态登记及受控整合 → 原 LTR10 的资格前置闭合`。

FIX05 前需要只读的 `DEP-QUAL-F2-ATTRIB` 决定与当前本地Claude ER03；ATTRIB可以明确为unknown，不代表P1关闭。真正的 `DEP-QUAL-F2-CLOSE` 在FIX05或另一个具名FIX实际C/D及关闭审查之后，且在FIX04之前；FIX05不依赖CLOSE。若F2不同根因，先登记新的FIX卡、写集与独立发布顺序；若unknown仍保留CLOSE阻塞FIX04和最终资格，不能靠FIX05全量一次PASS关闭。FIX05自己的实际新失败/flake仍按ER10 STOP，不能以F2另记为由放行。新F2卡不受FIX05唯一warning例外覆盖，必须另取得其具体发布决定。不得将不同缺陷合并成 G08 家族、移植 Mega G13，或省略各卡 patch/C/D。

**DEC-QUAL-05-RELEASE-WARNING（pending，非已批准 EX）：** 既定 release 零项目 warning 门使两种顺序均不合规：FIX05 先发布会留下 ingress 两方法的既存 dead_code；FIX04 先发布又留下未关闭的 F1/F2 P1。最小待决事项仅是：是否允许 **FIX05 这一次独立版本发布**在 default/keyring release 构建保留 b73 的 `from_payload`、`from_payload_with_hint_policy` 这一个具名 dead_code 诊断，紧接着由 FIX04 的下一 patch 消除。它不是模板 G-* waiver、不是第三方 proc-macro 提示，也不允许其它 warning、全量失败、Clippy warning 或未关闭 P1。用户本次要求写卡、main 提交和跨机开发，**未等同批准此例外**。接收 Agent 先完成可独立的只读核验及候选设计，再向用户取得具体决定、记录原话/UTC与范围，并获得真实本地 Claude 的发布次序 PASS；在此之前 FIX05 不进入实现/A/C/发布。若决定维持零警告，保持阻塞并提出新可审方案，不擅自扩大例外。

**现行 DEC-QUAL-05-RELEASE-WARNING（用户原话，2026-10-26T05:40:46Z）：** DEC-QUAL-05-RELEASE-WARNING：我批准仅 v0.30.88 这一次 FIX05 发布可以保留 CaptureIngressCommand::from_payload 与 from_payload_with_hint_policy 的 dead_code 诊断，保留到下一个 patch 标签为止，之后的版本不得保留。范围只有这两个方法。UTC 2026-10-26T05:40:46Z。

- UTC：`2026-10-26T05:40:46Z`。
- 范围：只有 `CaptureIngressCommand::from_payload` 与 `CaptureIngressCommand::from_payload_with_hint_policy`。
- 期限：保留到下一个 patch 标签为止；之后的版本不得保留这两条诊断。
- v0.30.88 这一次 FIX05 发布可否带上这两条 dead_code 诊断：可以。只此一次，只这两个方法。
- 本地 Claude session `4042079a-78eb-4af9-b86d-34e0d05ffe9c`：`RELEASE_ORDER_VERDICT: PASS`，`VERDICT: PASS`，`SCHEME: GRANT-ONE-VERSION`，`MAY_SHIP_DEAD_CODE: yes`，`DURATION: until-next-patch-tag`，`FIX05_VERSION: v0.30.88`。
- 该决定不关闭 F2，不把 cfg 放进 FIX05 提交，也不代替 FIX-LTR-05 配置失败后端的 ER03。2891a30b 的发布次序 FAIL 仍是当时的 FAIL。上文 pending 段保留为决定之前的入口。

### 接收 Agent 的第一轮动作

1. 定位原生 Libra 仓库，核 main、工作区、index、远端、提交祖先与实际写者；保留 COV/OG、SCAP 等上游成果以及未知改动。原工作区 `7541e9ba...` ahead 10 的 scoped-traces 历史与本计划修复来源分开，禁止将整条旧本地 main 推到远端。独立 clone 的 `.libra` 必须是实目录，不能采用共享 HEAD/index 的 worktree。
2. 读取当前 AGENTS、CLAUDE、plan-template 和本入口，核原生 CLI 版本、vault 签名、DCO、gh 权限及 workflow。只把 `gh auth status` 的有效 repo/workflow 权限作为能力证据，不能再误判为不能使用 `gh release`；实际动作仍核返回值与远端 ref。
3. 完成 F2-ATTRIB 的只读归属决定（允许明确unknown但不关闭P1），审计 F1 的 `ClientStorage::init` 调用者、所有 `Storage` 方法和错误映射，再冻结 FIX05 实际 I/AC/VER/规模。下面的窄实现路线不是对未实现方案的 PASS；若超出卡范围，先 ER03 拆分。
4. 取得 DEC-QUAL-05-RELEASE-WARNING 决定和本地 Claude 准入，再实现 FIX05。两份 env 文件缺失、所需 live 凭据缺失或新机器预算无依据时，先完成无依赖的准备，再报告具体缺项，不能 skip 对应必需门。
5. 每卡把源码、验收、Claude、提交、main push、版本 tag/Release、CI/D分别记账。每次 main push 前重新核远端；有新增提交先原生前滚整合并重验受影响门，不 force。用户在另一机器 main 开发不代表可以并发发布；接收方是 Libra 当前唯一实现/发布者，跨仓 root 继续 Mega2，共享协议修改须先协调。

### F2：先归属、后关闭，诊断由已准入FIX05承载

`DEP-QUAL-F2-ATTRIB`只读核原case `command::reset_test::reset_pathspec_from_file_missing_path_errors`、原TRY1退出断言已过而JSON解析失败/TRY2通过的原件缺口及当前控制流；本地Claude须给出「同一机制 / 独立缺陷 / unknown」具名决定。unknown允许完成归属登记，但不是根因证明或P1关闭。该只读阶段不得改跟踪文件或启动Cargo；不新增独立1800秒窗口。

只有FIX05本身获准后，才由FIX05在已登记 `tests/command/reset_test.rs` 中补失败时的合成stdout/stderr/exit诊断，保留原命令、参数、JSON/code/exit断言，禁止剥离Warning、过滤前缀或弱化断言。诊断属于FIX05自己的VER-4完整reset模块，单次 `source .env.test && source .env.live-test && cargo nextest run --test command_test -E 'test(command::reset_test::)' --no-fail-fast --retries 0`，编译与执行计入其已准入A预算；其全量A/C继续保留真实并发，不以单case通过替代。诊断输入、资源归属及执行器须在该卡ER03中审定，无前置跟踪写集空档。

`DEP-QUAL-F2-CLOSE`须真实证据及本地Claude明确关闭：同机制时依赖FIX05完整C/D；独立缺陷依赖另立FIX的完整C/D；未知时保持P1/阻塞FIX04。新的受控实验只能证明当前机制，不能补造已经缺失的历史stderr，也不能把未复现/原native retry成功写成旧C1接受。归属与关闭是两个不同节点，故不存在FIX05等待自己的环。

### Task FIX-LTR-05: 存储配置读取失败时停止错误的本地降级

**Task type:** `implementation`

**Lifecycle / Acceptance:** `blocked` / 空（DEC-QUAL-05 用户原话与 session 4042079a 发布次序 PASS 已记录；F2-ATTRIB 仍是 unknown；配置失败后端的 ER03 与源码仍待。无本卡源码、A/C、提交或版本）

**Description:** 单一行为轴是对象存储配置解析失败的 fail-closed 语义。普通解析/数据库/解密错误不能自动创建可继续写入的 LocalStorage，也不能用额外 stderr Warning 破坏 machine/JSON 输出。保留明确 local-only、合法 remote 配置、配置角色隔离及 COV/OG 已有 GlobalSchemaFuture 处理策略。优先审计保持 `ClientStorage::init -> Self` 签名的私有失败后端方案，避免把现有约99处调用点改成新的 fallible API；不是只移除 eprintln。候选后端必须审计并覆写所有可能返回成功/执行副作用的方法，尤其 `exist_checked`、`object_size`、`heal`、`evict_local`、`delete_payload` 的默认实现及其batch路径；legacy bool/search 方法不能将未知持久化状态当成功或删除依据。实际调用链若无法在本卡写集内保证正确性，先拆卡，不以返回 false 或空集合掩盖错误。

**Out of scope:** ingress cfg 属 FIX04；reasoning fixture 属 FIX02；MCP worker teardown 属 FIX03；F3/BrokenPipe 不混入。F2 不同根因时另卡。无 schema 迁移、无全局 `ClientStorage::init` 签名重构、无新增依赖、无延长 SQLite busy timeout、无额外重试，也不把配置读取错误假装成配置不存在。

**Current evidence:** 上表F1及实际fetch失败可核；历史锁owner/SQL仍未知。R1本地Claude已识别以下真实源码限制，故“只加一个失败后端”的设计未获准：①env未设置时resolver仍读repository DB，锁住时无法区分local-only与remote，直接fail-closed可能把普通local操作变成另一错误码；②`src/utils/util.rs::cached_objects_storage`按路径永久缓存ClientStorage，一次busy不能永久毒化长进程；③`src/command/fetch.rs`先write_pack_and_index，再objects_storage，单后端不能保证pack/index零副作用；④Storage的exist_checked默认折叠bool，object_size默认Ok(None)，delete_payload默认Ok(())且obliteration driver可能先unlink。`GlobalSchemaFuture`另有既定策略，不能顺带改变；create_storage_backend中的builder panic也须在同一配置失败轴内移除，不能在被改函数保留生产panic。

**设计准入待办（不是已通过的粒度结论）：** 先完成VER-6，只读确定实际缓存恢复、配置busy政策与写入前检查的落点。下列I是3生产文件候选范围，不表示实现已自洽。原local-only正常读取必须继续成功；配置busy时若需要改变既有local命令错误码/可用性，必须先提交明确兼容方案与具名用户决定，更新本卡AC/双语文档/测试，不能直接改旧预期让新错误通过。所有普通配置/构造失败应有安全错误路径；不能把未知状态写成“没有remote”。若无法在本轴下完成、需要改Storage trait/其他删除调用者或突破3文件候选，先更新I/G-*并真实ER03，必要时拆卡。

**Acceptance criteria:**

- [ ] AC-1: 普通配置解析错误不产生可操作的 LocalStorage 后端。
- [ ] AC-2: 经ClientStorage/Storage的写入和状态变更拒绝失败配置；fetch在pack/index安装前完成同一配置检查，无错误成功及本卡调用路径的对象/index/ref副作用。全局绕过Storage的删除路径先VER-6核清，不把此条伪称覆盖所有命令。
- [ ] AC-3: machine/JSON 失败输出保持原结构与退出约定，不夹入未结构化 Warning。
- [ ] AC-4: 错误诊断不泄漏配置值、凭据或解密原文。
- [ ] AC-5: local-only配置可正常读取时保持原可用行为；DB busy无法判定local/remote时不得静默降级，行为/错误码兼容方案先审定，未经具名决定不改变既有local命令契约。
- [ ] AC-6: 合法 remote 配置保持原存储选择与持久化行为。
- [ ] AC-7: COV/OG 的全局 future-schema/配置角色策略保持原契约。
- [ ] AC-8: 配置瞬时失败不被路径缓存永久固定；配置恢复后同一长进程按已审定策略恢复，无自动无限重试。

**Verification:**

- [ ] VER-1: `source .env.test && source .env.live-test && cargo nextest run --lib -E 'test(utils::client_storage::)' --no-fail-fast --retries 0`；完整模块非零，新增 `storage_config_failure_` 用例覆盖受控真实 SQLite 锁、失败后端全部方法、无落盘副作用、canary不泄漏、local/remote有效路径。使用同步屏障证明锁持有，保持原200ms，不用延长timeout掩盖故障。
- [ ] VER-2: `source .env.test && source .env.live-test && cargo nextest run --test command_test -E 'test(command::fetch_test::)' --no-fail-fast --retries 0`；完整fetch模块非零，原machine/JSON、code、exit断言保持。
- [ ] VER-3: `source .env.test && source .env.live-test && cargo nextest run --test command_test -E 'test(command::output_flags_test::)' --no-fail-fast --retries 0`；完整输出模式模块，不能靠过滤stderr使其通过。
- [ ] VER-4: `source .env.test && source .env.live-test && cargo nextest run --test command_test -E 'test(command::reset_test::)' --no-fail-fast --retries 0`；完整reset模块；F2诊断、归属和已有断言分别记录，未证明根因不记修复。
- [ ] VER-5: `source .env.test && source .env.live-test && cargo nextest run --test compat_global_config_schema_future --no-fail-fast --retries 0`；完整COV/OG兼容目标，保留权限、只读与future-schema断言。
- [ ] VER-6: 准入前逐方法/调用者只读审计 `Storage`、`ClientStorage::init`、util缓存、fetch写入顺序、exist_checked/batch、object_size、先unlink后delete_payload、bool/search与错误输出映射；以当前源码列出可能吞错的方法、对应真实测试及残余限制。发现写集外消费者违反fail-closed即ER03，不仅审计get/put。

**Full-suite trigger:** T-1：跨命令共享存储核心；A、C均完整默认套件，沿本文双env/nextest原命令；不以focused PASS替代。

**Dependencies:** `FIX-LTR-01`、`FIX-LTR-05P`、`FIX-LTR-05Q`、`FIX-LTR-05R`、`FIX-LTR-05S`、`FIX-LTR-05T`、`FIX-LTR-05U`、`FIX-LTR-05V`、`DEP-QUAL-SOURCE`、`DEP-QUAL-PUBLISH`、`DEP-QUAL-F2-ATTRIB`、`DEC-QUAL-05-RELEASE-WARNING`。本卡拆出 `FIX-LTR-05P` 至 `FIX-LTR-05V`。05V 只改 AutoFollow 的对象探测。配置失败后端、缓存和 pack 写入顺序仍未做，所以本卡仍 blocked。F2-CLOSE不是本卡前驱；独立缺陷必须另外登记其FIX-ID和边；不以未来FIX04的C/D为前驱，避免环。

**Deliverables:** 单轴生产修复、完整测试、下列同卡文档与证据摘要；不是仅输出诊断报告。

**Implementation write set:** 候选 `src/utils/client_storage.rs`（配置失败路径、消除该路径panic及模块测试）、`src/utils/util.rs`（缓存失败恢复与相邻单测）、`src/command/fetch.rs`（必要的写入前同轴检查及相邻单测）；VER-6审定前不得实施；`tests/command/fetch_test.rs`、`tests/command/output_flags_test.rs`、`tests/command/reset_test.rs`（诊断/同轴回归）；`docs/commands/config.md`、`docs/commands/zh-CN/config.md`、`docs/commands/fetch.md`、`docs/commands/zh-CN/fetch.md`、`docs/development/commands/config.md`、`docs/development/commands/fetch.md`、`COMPATIBILITY.md`、`tests/INDEX.md`、本计划、`docs/development/plan/plan-status.md`仅本计划行。网站同卡范围为已定位libra-backend的 `apps/tanstack-app/content/docs/commands/config.en.md`、`fetch.en.md`（本机当前仅有英文对应页，不虚构不存在的zh文件），在真实cf分支按ER06a交付；接收机器先核实际文件/命令/部署。若受影响公开命令或错误码超出上述清单，先逐文件补I并ER03，不遗漏同卡文档。

**Release write set:** Inherited；当前 `Cargo.toml`、`install.sh`、`install.ps1` 三版本面及工具链刷新 `Cargo.lock`、artifact；以执行时 `compat_version_surface_sync` 为权威。

**Files likely touched:** 与I/R相同；不修改ingress、reasoning helper、MCP worker或模板。锁测试须防env优先级遮蔽真实DB路径：用受审隔离guard清除合成LIBRA_STORAGE控制变量、保存并恢复原环境，匹配env/cwd串行lane；如新增serial影响 `tests/SERIAL_REGISTRY.tsv` / `.config/nextest.toml`，先逐文件补I并按仓库生成脚本重生成，ER03后才能修改，不手改并发组来掩盖故障。

**Docs and compatibility impact:** 这是用户可见的错误处理变化；先中文后英文，同卡同步config/fetch、开发说明、COMPATIBILITY与网站。明确「未配置remote」与「读取remote配置失败」的区别、错误后的恢复指引及machine输出；保留GlobalSchemaFuture已有特殊策略。不预造新的错误码；若需要新增，先更新I、错误码文档及对应Display/CLI测试。网站命令/typecheck/build、Claude、签名cf提交/push和部署HTTP均实际核验，缺外部条件不能宣称文档完成。

**Rollback mode:** `immutable-release`

**Migration and rollback:** 无schema迁移；已发布提交/版本不可撤回，若回归则新patch前滚。失败配置下不得新写数据；用户修正配置后重新发起操作。不得自动降级、重试、移除数据或重置已有ref。

**Security and privacy:** 不降低远端持久化保证，不输出secret/配置值/真实token；测试用合成canary与owned隔离库，失败清理须证明所有权。`exist=false`不能授权危险删除或把未知远端状态当已确认不存在。

**Performance budget:** 维持现有200ms配置DB锁上限，不增加重试，不新增无界扫描。F2诊断归入VER-4，无额外窗口；FIX05 A/C各候选一次10800秒，包含编译、listing、全部门、release构建、安装与资源后检。这些基于旧机器暖缓存耗时，仅为待审上限；新机必须先核工具链/资源/缓存与具体命令成本，由本地Claude准入。旧claim不能复用；未预审、冷缓存超预算或失败均STOP/ER03，不在claim外预编译或到时续窗。

**Estimated scope:** `pending VER-6`；候选3生产文件（client_storage/util/fetch），随附测试/双语文档完整登记。尚未通过G-04/ER03；冻结实际落点、文件数和恢复自洽性后填正式S/M，禁止以本次文档PASS代替实施准入。

**Version increment:** `patch`；当前0.30.87之后的0.30.88只是未被占用时的候选，必须现场核main/tag/Release；不得把本交接文档提交当版本占用。

**Release boundary:** `independent`；独立完整C/D及下一patch，不合并FIX04。

**C/D coverage from:** `self`；实际C/D后置证据由FIX04重新开工时在本计划与plan-status本计划行补记，不新建未登记补记提交。

**Granularity（未准入候选，禁止作为G-* PASS消费）:** `type=implementation; axis=配置读取失败时对象存储fail-closed; recovery=immutable-release前滚同轴修复; complete=pending VER-6; self-contained=pending VER-6; AC=8/8; VER=6/8; landing=pending VER-6; prod-files=3候选; scope=pending VER-6; deps=FIX-LTR-01,DEP-QUAL-SOURCE,DEP-QUAL-PUBLISH,DEP-QUAL-F2-ATTRIB,DEC-QUAL-05-RELEASE-WARNING; writeset=串行于FIX04之前; release=independent; split-from=FIX04-C1-F1; exception=N/A`。接收方必须把pending项变成真实满足模板的正式字段并获得本地Claude ER03 PASS，才可实施。本次交接审查只评价缺口/要求登记是否准确，不准入不自洽设计。

**明确的release profile门：** A/C均运行 `source .env.test && source .env.live-test && cargo build --release` 及 `source .env.test && source .env.live-test && cargo build --release --features keyring`。项目warning默认零；只有DEC实际获批时才按其唯一FIX05版本/两方法诊断范围核对。除此之外任何项目warning均STOP，不能把候选例外写入通用模板。

### Task FIX-LTR-05P: 存在性探测改为可失败结果

**Task type:** `migration`

**Lifecycle / Acceptance:** `in-progress` / 空（session `7ed2380b-d9f1-4b5c-917f-203734f1f478` 已准入本写集；源码实施中）

**Description:** 单一行为轴是：存储读取失败不能再被布尔 `exist` 的 `false` 说成“对象不存在”。本卡增加 `ClientStorage::is_object_type_checked`，并把下列已点名的 `exist()==false` 缺失路径改成 `exist_checked`。`classify_presence` 改为返回 `Result`，存储错误由 `src/command/file.rs` 原样失败，不得映射成 `Absent`、`LooseOnly` 或 `PackedOnly`。`Storage::exist -> bool`、`ClientStorage::exist`、`ClientStorage::search`、`ClientStorage::init` 和 `objects_storage()` 的签名与现有成功路径保持不变。

**Out of scope:** 不实现配置失败后端，不改 `create_storage_backend` 的本地降级，不改 fetch 的 pack 写入顺序，不改 ingress cfg，不迁移本卡五个生产文件以外的 `exist` / `search` / `is_object_type` 调用方，不 bump、不打 tag。v0.30.88 仍只留给 FIX-LTR-05。本卡完成也不准入 FIX-LTR-05。五个生产文件的落点仍是 4 个：`is_object_type_checked`、fsck 的 13 处、cat-file 的 2 处、以及 `classify_presence` 的 `Result`（`file.rs` 只消费这个结果，不另算落点）。

**Current evidence:** `src/utils/client_storage.rs:2469` 的 `exist` 返回 `bool`；`:2546` 的 `is_object_type` 在读取错误时返回 `false`；`:2476` 的 `exist_checked` 已能返回 `Result`。本卡纳入的 `exist` 调用点是 `src/command/fsck.rs` 的 467、471、481、933、1769、1779、2388、2562、2656、2671、2750、2864、3006，以及 `src/command/cat_file.rs` 的 517、1219。`src/internal/obliteration/mod.rs:266` 的 `classify_presence` 在 `false` 时返回 `Absent`；唯一调用方是 `src/command/file.rs:182`，`emit_preview` 在 `:270`。这三个文件当前都不调用 `is_object_type`，所以新方法本卡只由 VER-1 覆盖，不假装已有生产调用方。ER03 session `e52e4025-3bab-423b-ae01-a666afcb458a` 要求先拆卡。相关调用分布在 66 个生产文件，本卡只收下面 5 个。

**Acceptance criteria:**

- [ ] AC-1: `is_object_type_checked` 在对象类型读取失败时返回 `Err`，不返回 `false`。
- [ ] AC-2: fsck 上列 13 处 `exist` 缺失判断改用 `exist_checked`，存储错误不是 “missing”。
- [ ] AC-3: cat-file 的 517 与 1219 两处改用 `exist_checked`，存储错误不是缺失退出。
- [ ] AC-4: `classify_presence` 返回 `Result`；`file.rs` 对 `Err` 失败返回，稳定错误码是 `StableErrorCode::IoReadFailed`，并且不把它显示成 absent、loose 或 packed。
- [ ] AC-5: 未迁移调用方的 `exist`、`search`、`init` 行为不变，且本卡不引入配置失败后端。
- [ ] AC-6: 三个版本面保持 `0.30.87`，不创建 `v0.30.88` tag。

**Verification:**

- [ ] VER-1: `source .env.test && source .env.live-test && cargo nextest run --lib -E 'test(is_object_type_checked_returns_read_error)' --no-fail-fast --retries 0`
- [ ] VER-2: `source .env.test && source .env.live-test && cargo nextest run --test command_test -E 'test(fsck_storage_read_error_is_not_missing)' --no-fail-fast --retries 0`
- [ ] VER-3: `source .env.test && source .env.live-test && cargo nextest run --test command_test -E 'test(cat_file_storage_read_error_is_not_absent)' --no-fail-fast --retries 0`
- [ ] VER-4: `source .env.test && source .env.live-test && cargo nextest run --lib -E 'test(classify_presence_storage_read_error_is_not_absent)' --no-fail-fast --retries 0`
- [ ] VER-5: `source .env.test && source .env.live-test && cargo nextest run --test command_test -E 'test(file_obliterate_storage_read_error_is_not_absent)' --no-fail-fast --retries 0`。该测试必须断言稳定错误码 `StableErrorCode::IoReadFailed`，并且输出不是 absent、loose 或 packed。

**Full-suite trigger:** `N/A`（本卡不改共享存储失败语义，也不碰 `sql/**`、`src/cli.rs`、错误码或版本面；收口全量仍由计划最终资格执行）。

**Dependencies:** `FIX-LTR-01`、`DEP-QUAL-SOURCE`。

**Deliverables:** 上述五个生产文件中的可失败探测，以及对应测试。`file.rs` 对 `classify_presence` 的 `Err` 使用现有 `StableErrorCode::IoReadFailed`，不新增错误码。

**Implementation write set:** `src/utils/client_storage.rs`、`src/command/fsck.rs`、`src/command/cat_file.rs`、`src/internal/obliteration/mod.rs`、`src/command/file.rs`、`tests/command/fsck_test.rs`、`tests/command/cat_file_test.rs`、`tests/command/file_obliterate_test.rs`、本计划、`docs/development/plan/plan-status.md` 仅本计划行。

**Release write set:** `N/A`

**Files likely touched:** 与 I 相同。

**Docs and compatibility impact:** `N/A`。本卡不新增命令、参数、配置或稳定错误码；存储读取失败改为可见错误是内部探测的失败传播，既有缺失对象的成功路径保持原退出与文案。没有对应的网站命令页需要改写。

**Rollback mode:** `immutable-release`

**Migration and rollback:** 无 schema 迁移。已推送提交只以前滚修正。回退行为是调用方重新使用布尔 `exist`，因此不得在本卡发布配置失败后端。

**Security and privacy:** 错误文本不包含配置值、凭据或数据库路径。`exist_checked` 的 `Err` 不得被当成删除许可。

**Performance budget:** 不增加重试，不放宽现有 200ms 配置锁。本卡本地验证一次 3600 秒，含编译。

**Estimated scope:** `M`

**Version increment:** `N/A`

**Release boundary:** `family child`。不 bump、不 tag。已批准的 `v0.30.88` 仍只属于 FIX-LTR-05 那一次发布；本卡提交可以进入 main，但不得占用该版本号。

**C/D coverage from:** `FIX-LTR-05`（本卡无独立 release artifact；main 上的 fmt/clippy 与本卡 VER 另记）。

**Granularity:** `type=migration; axis=存储读取失败不再被布尔存在性探测说成对象不存在; recovery=immutable-release前滚，且不携带配置失败后端; complete=yes; self-contained=yes; AC=6/8; VER=5/8; landing=4; prod-files=5; scope=M; deps=FIX-LTR-01,DEP-QUAL-SOURCE; writeset=序列化于FIX-LTR-05之前; release=family child; split-from=FIX-LTR-05; exception=N/A`

### Task FIX-LTR-05Q: 下一批存在性探测迁移

**Task type:** `migration`

**Lifecycle / Acceptance:** `in-progress` / 空（session `3de3e11b-98e4-46ee-807c-e903b42bcf0e` 已准入这四个生产文件）

**Description:** 把 `show`、`maintenance`、`restore` 和 `Blob::save` 里把存储读取失败当成对象不存在的探测改成可失败结果。`Ok(false)` 保持原来的缺失或非提交语义。读失败分别使用 `IoReadFailed`、`RestoreError::ReadObject` 或 `io::Error`，不得调用 `put`。

**Out of scope:** 不包含 `am.rs` 的 `search`，因为 `Storage::search` 仍返回 `Vec`。不包含 `fetch.rs` 和配置失败后端。不 bump、不打 tag。

**Current evidence:** `src/command/show.rs` 的 `resolve_existing_object_hash`、`src/command/maintenance.rs` 的 `collect_agent_run_manifest_roots`、`src/command/restore.rs` 的两处 `is_object_type`、`src/utils/object_ext.rs` 的 `BlobExt::save`。准入 session `3de3e11b-98e4-46ee-807c-e903b42bcf0e`。

**Acceptance criteria:**

- [ ] AC-1: `show` 的存储读取失败是 `LBR-IO-001`，不是 bad revision。
- [ ] AC-2: maintenance 清单里的存储读取失败是 `IoReadFailed`，不是 “not in the store”。
- [ ] AC-3: `restore` 的类型读取失败是读错误，不是 “not a commit”。
- [ ] AC-4: `Blob::save` 在读取失败时返回错误并且不写入 loose 对象。

**Verification:**

- [ ] VER-1: `source .env.test && source .env.live-test && cargo nextest run --test command_test -E 'test(show_storage_read_error_is_not_bad_revision)' --no-fail-fast --retries 0`
- [ ] VER-2: `source .env.test && source .env.live-test && cargo nextest run --lib -E 'test(maintenance_storage_read_error_is_not_repo_corrupt)' --no-fail-fast --retries 0`
- [ ] VER-3: `source .env.test && source .env.live-test && cargo nextest run --lib -E 'test(restore_storage_read_error_is_not_non_commit)' --no-fail-fast --retries 0`
- [ ] VER-4: `source .env.test && source .env.live-test && cargo nextest run --lib -E 'test(blob_save_storage_read_error_does_not_write)' --no-fail-fast --retries 0`

**Dependencies:** `FIX-LTR-05P`。

**Deliverables:** 上述四个生产文件的可失败探测和对应测试。

**Implementation write set:** `src/command/show.rs`、`src/command/maintenance.rs`、`src/command/restore.rs`、`src/utils/object_ext.rs`、`tests/command/show_test.rs`、`tests/SRC_SERIAL_REGISTRY.tsv`、`tests/SRC_SERIAL_CENSUS.tsv`、本计划、`docs/development/plan/plan-status.md` 仅本计划行。

**Release write set:** `N/A`

**Files likely touched:** 与 I 相同。

**Docs and compatibility impact:** `N/A`。本卡不新增命令、参数、配置或稳定错误码。对象缺失和非提交引用的原错误保持不变；只有存储读取失败改为已有的 `LBR-IO-001` 或读错误。没有对应的网站命令页需要改写。

**Rollback mode:** `immutable-release`

**Migration and rollback:** 未实施，无迁移。

**Security and privacy:** 未实施。

**Performance budget:** 不增加重试。本卡本地验证一次 3600 秒，含编译。

**Estimated scope:** `M`

**Version increment:** `N/A`

**Release boundary:** `family child`

**C/D coverage from:** `FIX-LTR-05`

**Granularity:** `type=migration; axis=show、maintenance、restore 和 Blob::save 不再把存储读取失败当成对象不存在; recovery=immutable-release前滚; complete=yes; self-contained=yes; AC=4/8; VER=4/8; landing=4; prod-files=4; scope=M; deps=FIX-LTR-05P; writeset=序列化于FIX-LTR-05之前; release=family child; split-from=FIX-LTR-05; exception=N/A`

### Task FIX-LTR-05R: 操作日志与 repack 的存在性探测

**Task type:** `migration`

**Lifecycle / Acceptance:** `in-progress` / 空（session `54f3e8cf-2bd8-426e-8a64-193e4d2f77f3` 已准入这四个生产文件；实现审查 `146781d3-4c39-4e90-b1f2-c34678d523e4` VERDICT PASS）

**Description:** 把操作日志快照、存储、恢复和 `repack --local` 里把存储读取失败当成对象不存在的探测改成可失败结果。`Ok(false)` 保持原来的缺失、回退哈希或丢弃 alternate-only 对象的语义。读失败分别是 `ScanError::Io`、`SnapshotError::Object`、`StoreError::Object`、`RestoreError::Object` 或 `StableErrorCode::IoReadFailed`，不得写成缺失、非提交，也不得调用 `put`。

**Out of scope:** 不包含 `am.rs` 的 `search`，因为 `Storage::search` 仍返回 `Vec`。不包含 `fetch.rs`、`client_storage` 的 `ensure_existing_object_index` / `search_result`、`cloud/agent_capture.rs`、tiered/remote 的布尔 `exist`，以及配置失败后端。不删除公开的 `OperationStoreV2::is_object_type`。不 bump、不打 tag。本卡完成也不准入 FIX-LTR-05。

**Current evidence:** `try_reuse_index_oid` 与 `put_content_addressed_object` 在 `src/internal/operation/snapshot.rs`；`OperationStoreV2::is_object_type` 在 `src/internal/operation/store.rs`；两处提交类型探测在 `src/internal/operation/restore.rs`；`retain_local_objects` 在 `src/command/repack.rs`。准入 session `54f3e8cf-2bd8-426e-8a64-193e4d2f77f3`。

**Acceptance criteria:**

- [ ] AC-1: `try_reuse_index_oid` 的存储读取失败是 `ScanError::Io`，不是 `Ok(None)`。
- [ ] AC-2: `put_content_addressed_object` 的存储读取失败是 `SnapshotError::Object`，并且不调用 `put`。`Ok(false)` 仍写入。
- [ ] AC-3: operation restore 的类型读取失败是 `RestoreError::Object`，正文保留读取错误，并且不是 “does not reference a commit object”。`Ok(false)` 仍使用这句缺失文案。
- [ ] AC-4: `retain_local_objects` 的存储读取失败是 `StableErrorCode::IoReadFailed`，正文含无完整索引，并且不是 missing 或 absent。`Ok(false)` 仍丢弃该对象。

**Verification:**

- [ ] VER-1: `source .env.test && source .env.live-test && cargo nextest run --lib -E 'test(snapshot_index_reuse_read_error_does_not_fall_through) or test(snapshot_storage_read_error_does_not_write)' --no-fail-fast --retries 0`
- [ ] VER-2: `source .env.test && source .env.live-test && cargo nextest run --lib -E 'test(store_type_read_error_stays_an_object_error)' --no-fail-fast --retries 0`
- [ ] VER-3: `source .env.test && source .env.live-test && cargo nextest run --lib -E 'test(internal::operation::restore::tests::restore_storage_read_error_is_not_non_commit)' --no-fail-fast --retries 0`
- [ ] VER-4: `source .env.test && source .env.live-test && cargo nextest run --lib -E 'test(repack_storage_read_error_is_not_absent)' --no-fail-fast --retries 0`

**Full-suite trigger:** `N/A`（本卡不改 `sql/**`、`src/cli.rs`、稳定错误码或版本面；公开 `is_object_type` 保留，避免 T-3）。

**Dependencies:** `FIX-LTR-05Q`。

**Deliverables:** 上述四个生产文件的可失败探测和对应测试。

**Implementation write set:** `src/internal/operation/snapshot.rs`、`src/internal/operation/store.rs`、`src/internal/operation/restore.rs`、`src/command/repack.rs`、`tests/SRC_SERIAL_REGISTRY.tsv`、`tests/SRC_SERIAL_CENSUS.tsv`、本计划、`docs/development/plan/plan-status.md` 仅本计划行。

**Release write set:** `N/A`

**Files likely touched:** 与 I 相同。

**Docs and compatibility impact:** `N/A`。本卡不新增命令、参数、配置或稳定错误码。对象缺失仍沿用原缺失或非提交文案；只有存储读取失败改为已有读错误。没有对应的网站命令页需要改写。

**Rollback mode:** `immutable-release`

**Migration and rollback:** 无 schema 迁移。已推送提交只以前滚修正。

**Security and privacy:** 错误文本不包含配置值、凭据或数据库路径。读取失败不得被当成对象不存在而跳过或覆盖。

**Performance budget:** 不增加重试。本卡本地验证一次 3600 秒，含编译。

**Estimated scope:** `M`

**Version increment:** `N/A`

**Release boundary:** `family child`。不 bump、不 tag。已批准的 `v0.30.88` 仍只属于 FIX-LTR-05 那一次发布。

**C/D coverage from:** `FIX-LTR-05`

**Granularity:** `type=migration; axis=操作日志快照、恢复和 repack 不再把存储读取失败当成对象不存在; recovery=immutable-release前滚; complete=yes; self-contained=yes; AC=4/8; VER=4/8; landing=4; prod-files=4; scope=M; deps=FIX-LTR-05Q; writeset=序列化于FIX-LTR-05之前; release=family child; split-from=FIX-LTR-05; exception=N/A`

### Task FIX-LTR-05S: 索引补登记与修订后缀的类型过滤

**Task type:** `migration`

**Lifecycle / Acceptance:** `in-progress` / 空（session `71cf457d-5724-48c9-b2e4-68c4ac1a8bc4` 已准入 `client_storage.rs`；实现审查 `d6849287-9d4e-46c7-8148-a41e8e5e412d` VERDICT PASS）

**Description:** `ensure_existing_object_index` 在存储读取失败时不再说载荷已经不存在。`search_result` 在 `~` / `^` 导览里过滤提交类型时，类型读取失败返回 `Err`；对象缺失和类型不符仍丢弃该哈希。`storage.search` 仍返回 `Vec`。

**Out of scope:** 不包含 `am.rs`、`fetch.rs` 的 AutoFollow `exist`、local/tiered/remote 的布尔 `exist`、trait 的 `exist_checked` 默认实现，以及配置失败后端。不 bump、不打 tag。本卡完成也不准入 FIX-LTR-05。

**Current evidence:** `ensure_existing_object_index` 与 `search_result` 都在 `src/utils/client_storage.rs`。准入 session `71cf457d-5724-48c9-b2e4-68c4ac1a8bc4`。

**Acceptance criteria:**

- [ ] AC-1: `ensure_existing_object_index` 的存储读取失败不是 `NotFound`，正文不含 “no longer present”。`Ok(false)` 仍是这句缺失文案。
- [ ] AC-2: `search_result` 的 `~` / `^` 类型读取失败返回 `Err`，不是空列表。`ObjectNotFound` 仍得到空列表。

**Verification:**

- [ ] VER-1: `source .env.test && source .env.live-test && cargo nextest run --lib -E 'test(ensure_existing_object_index_read_error_is_not_missing)' --no-fail-fast --retries 0`
- [ ] VER-2: `source .env.test && source .env.live-test && cargo nextest run --lib -E 'test(search_result_type_read_error_is_not_empty)' --no-fail-fast --retries 0`

**Full-suite trigger:** `N/A`（本卡不改 `sql/**`、`src/cli.rs`、稳定错误码或版本面）。

**Dependencies:** `FIX-LTR-05R`。

**Deliverables:** 上述一个生产文件的两处可失败探测和同文件测试。

**Implementation write set:** `src/utils/client_storage.rs`、`tests/SRC_SERIAL_REGISTRY.tsv`、`tests/SRC_SERIAL_CENSUS.tsv`、本计划、`docs/development/plan/plan-status.md` 仅本计划行。

**Release write set:** `N/A`

**Files likely touched:** 与 I 相同。

**Docs and compatibility impact:** `N/A`。本卡不新增命令、参数、配置或稳定错误码。载荷缺失仍是原来的 NotFound 文案。没有对应的网站命令页需要改写。

**Rollback mode:** `immutable-release`

**Migration and rollback:** 无 schema 迁移。已推送提交只以前滚修正。

**Security and privacy:** 错误文本不包含配置值、凭据或数据库路径。读取失败不得被当成载荷缺失。

**Performance budget:** 不增加重试。本卡本地验证一次 3600 秒，含编译。

**Estimated scope:** `S`

**Version increment:** `N/A`

**Release boundary:** `family child`。不 bump、不 tag。已批准的 `v0.30.88` 仍只属于 FIX-LTR-05 那一次发布。

**C/D coverage from:** `FIX-LTR-05`

**Granularity:** `type=migration; axis=索引补登记和修订后缀不再把存储读取失败当成对象不存在; recovery=immutable-release前滚; complete=yes; self-contained=yes; AC=2/8; VER=2/8; landing=2; prod-files=1; scope=S; deps=FIX-LTR-05R; writeset=序列化于FIX-LTR-05之前; release=family child; split-from=FIX-LTR-05; exception=N/A`

### Task FIX-LTR-05T: am 的本地对象搜索不再把不可读目录当成缺失

**Task type:** `migration`

**Lifecycle / Acceptance:** `in-progress` / 空（session `986c352a-927e-4f3f-855e-f9879764e9e7` 已准入这四个生产文件；实现审查 `02aa713c-60c8-4b6a-b5c6-7e09cf2096af` VERDICT PASS。前一轮 `de5171c1` 的 Windows 测试模块 P1 已用 `cfg(all(test, unix))` 关闭）

**Description:** `Storage::search_checked` 在本地对象目录或 pack 索引读失败时返回 `Err`。对象目录不存在仍是空的 loose 结果。`am` 的三方基线解析使用这条路径。没有唯一匹配时仍留下该 id。读失败是 `StableErrorCode::IoReadFailed`，不说成缺失。`Storage::search` 的 `Vec` 行为保持不变。

**Out of scope:** 不改 `tiered.rs` 和 `remote.rs`。它们继续使用 trait 默认实现，因此 tiered 仓库的本地目录错误仍被吞掉。该项登记为 `DEFER-QUAL-05-TIERED-SEARCH`，重启条件是后续切片为 tiered 覆盖 `search_checked`。不改 `fetch.rs`，不改 `search_result` 的修订解析，不 bump、不打 tag。本卡完成也不准入 FIX-LTR-05。

**Current evidence:** `LocalStorage::search` 在 `src/utils/storage/local.rs` 跳过不可读目录项。`resolve_three_way_bases` 在 `src/command/am.rs`。准入 session `986c352a-927e-4f3f-855e-f9879764e9e7`。

**Acceptance criteria:**

- [ ] AC-1: 本地对象目录不可读时，`search_checked` 返回 `Err`。目录不存在仍是空结果。
- [ ] AC-2: `am` 把该错误显示为 `IoReadFailed`，正文不是 missing、absent 或 not found。没有唯一匹配时仍不解析该 id。

**Verification:**

- [ ] VER-1: `source .env.test && source .env.live-test && cargo nextest run --lib -E 'test(am_unreadable_object_dir_is_not_a_missing_base)' --no-fail-fast --retries 0`

**Full-suite trigger:** `N/A`（本卡不改 `sql/**`、`src/cli.rs`、稳定错误码表或版本面）。

**Dependencies:** `FIX-LTR-05S`。

**Deliverables:** 上述四个生产文件的可失败前缀搜索，以及 `am` 的同文件测试。

**Implementation write set:** `src/utils/storage/mod.rs`、`src/utils/storage/local.rs`、`src/utils/client_storage.rs`、`src/command/am.rs`、`tests/SRC_SERIAL_REGISTRY.tsv`、`tests/SRC_SERIAL_CENSUS.tsv`、本计划、`docs/development/plan/plan-status.md` 仅本计划行。

**Release write set:** `N/A`

**Files likely touched:** 与 I 相同。

**Docs and compatibility impact:** `N/A`。本卡不新增命令、参数、配置或稳定错误码。没有唯一对象匹配时的原行为保持。没有对应的网站命令页需要改写。`ClientStorage::search_checked` 只做前缀搜索，不解析 `HEAD`、分支或 `~` / `^`。

**Rollback mode:** `immutable-release`

**Migration and rollback:** 无 schema 迁移。已推送提交只以前滚修正。

**Security and privacy:** 错误文本不包含配置值、凭据或数据库路径。目录读失败不得被当成补丁基线不存在。

**Performance budget:** 不增加重试。本卡本地验证一次 3600 秒，含编译。

**Estimated scope:** `M`

**Version increment:** `N/A`

**Release boundary:** `family child`。不 bump、不 tag。已批准的 `v0.30.88` 仍只属于 FIX-LTR-05 那一次发布。

**C/D coverage from:** `FIX-LTR-05`

**Granularity:** `type=migration; axis=am 的本地对象搜索不再把不可读目录当成缺失; recovery=immutable-release前滚; complete=yes; self-contained=yes; AC=2/8; VER=1/8; landing=4; prod-files=4; scope=M; deps=FIX-LTR-05S; writeset=序列化于FIX-LTR-05之前; release=family child; split-from=FIX-LTR-05; exception=N/A`

### Task FIX-LTR-05U: tiered 与 remote 的前缀搜索不再把列举失败当成缺失

**Task type:** `migration`

**Lifecycle / Acceptance:** `in-progress` / 空（session `c1d476b0-876f-46a7-bb94-ce56405a85ee` 已准入 `remote.rs` 与 `tiered.rs`；实现审查 `bc600c5b-b019-4988-a347-7f25472ba8b7` VERDICT PASS）

**Description:** `RemoteStorage::search_checked` 在对象列举失败时返回 `Err`。`NotFound` 和空列举仍是没有匹配。`TieredStorage::search_checked` 同时采用本地和远端的可失败结果。离线或 local 读策略不访问远端。两端的 `search -> Vec` 保持原样。

**Out of scope:** 不改 `fetch.rs`、`client_storage.rs`、`local.rs` 和配置失败后端。不 bump、不打 tag。本卡完成也不准入 FIX-LTR-05。`DEFER-QUAL-05-TIERED-SEARCH` 的搜索路径由本卡关闭；fetch 的 AutoFollow 存在性判断随后由 FIX-LTR-05V 承接。

**Current evidence:** `remote.rs` 的 `search` 跳过列举错误。`tiered.rs` 的 `search` 合并两个 `Vec`。准入 session `c1d476b0-876f-46a7-bb94-ce56405a85ee` 要求离线策略不把远端不可达变成硬失败。

**Acceptance criteria:**

- [ ] AC-1: 远端列举失败是 `Err`，空列举和 `NotFound` 仍是空结果。
- [ ] AC-2: tiered 在自动策略下传播任一端的读失败。离线策略不访问远端。本地目录不可读仍是 `Err`。

**Verification:**

- [ ] VER-1: `source .env.test && source .env.live-test && cargo nextest run --lib -E 'test(remote_search_checked_distinguishes_list_failure_from_absence) or test(tiered_search_checked_local_only_skips_unreachable_remote) or test(tiered_search_checked_unreadable_local_dir_is_not_empty)' --no-fail-fast --retries 0`

**Full-suite trigger:** `N/A`（本卡不改 `sql/**`、`src/cli.rs`、稳定错误码或版本面）。

**Dependencies:** `FIX-LTR-05T`。

**Deliverables:** 上述两个生产文件的 `search_checked` 和同文件测试。

**Implementation write set:** `src/utils/storage/remote.rs`、`src/utils/storage/tiered.rs`、`tests/SRC_SERIAL_CENSUS.tsv`、本计划、`docs/development/plan/plan-status.md` 仅本计划行。

**Release write set:** `N/A`

**Files likely touched:** 与 I 相同。

**Docs and compatibility impact:** `N/A`。本卡不新增命令、参数、配置或稳定错误码。没有匹配对象时的空结果保持。没有对应的网站命令页需要改写。

**Rollback mode:** `immutable-release`

**Migration and rollback:** 无 schema 迁移。已推送提交只以前滚修正。

**Security and privacy:** 错误文本不包含配置值、凭据或数据库路径。列举失败不得被当成没有对象。

**Performance budget:** 不增加重试。本卡本地验证一次 3600 秒，含编译。

**Estimated scope:** `S`

**Version increment:** `N/A`

**Release boundary:** `family child`。不 bump、不 tag。已批准的 `v0.30.88` 仍只属于 FIX-LTR-05 那一次发布。

**C/D coverage from:** `FIX-LTR-05`

**Granularity:** `type=migration; axis=tiered 与 remote 的前缀搜索不再把列举失败当成缺失; recovery=immutable-release前滚; complete=yes; self-contained=yes; AC=2/8; VER=1/8; landing=2; prod-files=2; scope=S; deps=FIX-LTR-05T; writeset=序列化于FIX-LTR-05之前; release=family child; split-from=FIX-LTR-05; exception=N/A`

### Task FIX-LTR-05V: fetch AutoFollow 不再把存储读取失败当成缺失标签

**Task type:** `migration`

**Lifecycle / Acceptance:** `in-progress` / 空（session `95af8652-b4f3-4116-8eba-16ae3237a780` 已准入 `fetch.rs`、`pull.rs`、`clone.rs`；实现审查 `72196c7f-6172-41c8-ae03-c5d1ccb2dccf` VERDICT PASS）

**Description:** AutoFollow 用一次 `exist_checked_many` 判断标签对象是否已在本地。`Ok(true)` 保留标签。`Ok(false)` 仍丢弃。无法解析的哈希仍丢弃。读取失败或结果缺键是 `FetchError::TagObjectProbe`，稳定错误码仍是 `StableErrorCode::IoReadFailed`。不新增错误码。

**Out of scope:** 不改配置失败后端、对象缓存、`write_pack_and_index` 的顺序，也不改 `Storage::exist`。不 bump、不打 tag。本卡完成也不准入 FIX-LTR-05。

**Current evidence:** `fetch.rs` 的 AutoFollow 原先对 `storage.exist` 使用 `unwrap_or(false)`。准入 session `95af8652-b4f3-4116-8eba-16ae3237a780` 要求用批量探测，因为逐个 `exist_checked` 会把正在安装的无索引 pack 误报为失败。

**Acceptance criteria:**

- [ ] AC-1: 存储读取失败是 `TagObjectProbe` / `IoReadFailed`，Display 模板不含 missing、absent 或 not found。`Ok(false)` 仍丢弃标签。
- [ ] AC-2: `pull` 与 `clone` 把该变体映射为 `IoReadFailed`，不落到网络不可用。

**Verification:**

- [ ] VER-1: `source .env.test && source .env.live-test && cargo nextest run --lib -E 'test(autofollow_storage_read_error_is_not_a_missing_tag)' --no-fail-fast --retries 0`

**Full-suite trigger:** `N/A`（本卡不改 `sql/**`、`src/cli.rs`、稳定错误码表或版本面）。

**Dependencies:** `FIX-LTR-05U`。

**Deliverables:** AutoFollow 批量探测、`FetchError::TagObjectProbe`，以及 `pull` / `clone` 各一处映射。

**Implementation write set:** `src/command/fetch.rs`、`src/command/pull.rs`、`src/command/clone.rs`、本计划、`docs/development/plan/plan-status.md` 仅本计划行。

**Release write set:** `N/A`

**Files likely touched:** 与 I 相同。

**Docs and compatibility impact:** `N/A`。本卡不新增命令、参数、配置或稳定错误码。标签对象确实不存在时仍不写入该标签。没有对应的网站命令页需要改写。

**Rollback mode:** `immutable-release`

**Migration and rollback:** 无 schema 迁移。已推送提交只以前滚修正。探测发生在引用更新之后，与 `persist_fetched_tags` 相同，重跑 fetch 是幂等的。

**Security and privacy:** 错误文本不包含配置值或凭据。读取失败不得被当成标签对象不存在而静默丢弃。tiered 远端列举若返回非 NotFound（例如对象存储 403），AutoFollow 会失败而不是丢掉标签。

**Performance budget:** 不增加重试。一次批量探测，不按标签重复扫描 pack 目录。本卡本地验证一次 3600 秒，含编译。

**Estimated scope:** `M`

**Version increment:** `N/A`

**Release boundary:** `family child`。不 bump、不 tag。已批准的 `v0.30.88` 仍只属于 FIX-LTR-05 那一次发布。

**C/D coverage from:** `FIX-LTR-05`

**Granularity:** `type=migration; axis=fetch AutoFollow 不再把存储读取失败当成缺失标签; recovery=immutable-release前滚; complete=yes; self-contained=yes; AC=2/8; VER=1/8; landing=4; prod-files=3; scope=M; deps=FIX-LTR-05U; writeset=序列化于FIX-LTR-05之前; release=family child; split-from=FIX-LTR-05; exception=N/A`

### FIX04、FIX02、FIX03 的接续细则（覆盖旧开工快照）

- **FIX04：** Dependencies现为FIX05完整C/D、DEP-QUAL-F2-CLOSE已审定关闭、SOURCE/PUBLISH。从实际已发布main重新建源；仅两cfg与本计划/status行，不携带helper5025。原A1、源B、自己ER05仅在源与覆盖未变且原件可核时限域carry；失败C1的bump/lock、全量、release构建、安装不能继承。先登记当前恢复及新预算，Claude PASS后才claim；default与keyring release必须零项目warning，之后每卡继续零warning。版本只能在FIX05实际版本上patch+1。
- **FIX02：** Dependencies仍为FIX01、FIX04。只恢复 `tests/helpers/reasoning_artifact.rs` 的索引结清方案，不能stash-pop旧两份治理/旧bump；无stash原件时按卡片重建并新审新测。它的两个真实消费者、shared fixture全量触发、原timeout/断言均保留。A2/ownR2不是新源C/D；新版本是FIX04实际版本+1。
- **FIX03：** Dependencies仍为FIX02。先按现有卡解决worker PID/身份/PGID/父进程的完整teardown，不能按名字kill或把父wait当worker退出。完整MCP、full、doc、network原3例及ownClaude/C/D不省略；新版本是FIX02实际版本+1。F3等独立风险若阻断门，先另卡，不把手工清理当产品修复。
- **计划最终资格：** 五张实现卡分别完整C/D后，在FIX03最终发布merge/main树完成独立最终fmt/clippy/full/doc/network与实际Claude资格审查，再做原登记的终态文档提交及其真实D。用户在main开发时，原文「merge树=被测树」解释为实际main提交树=最终被测树；遇上游变化先整合重验，不用文档提交掩盖代码漂移。终态不得只停在本机安装成功。
- **受控整合：** 最后将这些实际发布提交与原scoped-traces未发布历史精确整合，逐文件保留COV/OG/SCAP等上游成果、签名/DCO和无关WIP；原7541e9ba/ed566b0相关合并签名政策与当前写者交接仍须具名处置。此步骤不等于直接推送原main的整条ahead历史。

### 原 scoped-traces 主线仍需的跨仓前置

本资格链闭合只满足 `DEP-TR-QUAL-REPAIRS`（目前登记在原工作区未发布治理中，远端main原计划尚无此边；受控整合前不得假定远端已登记）。LTR10另外需要 `DEP-TR-H0`、`DEP-TR-H1`、`DEP-TR-WRITERS` 和独立的 `DEP-TR-LIVE-SERVICE`；LIVE含持续服务owner、fixture寿命和可用性，并不由一次H1 PASS替代。Mega2剩余链为 `MTR11 → MTR40 → MTR13 → MTR39 → MTR92 → MTR12/H1`；此前MTR103已真实发布v0.42.75，但不等于H1。H1要记录真实服务SHA/配置摘要/端口/认证身份和owner，提供HTTP/SSH scoped Git、两身份/target权限及持续fixture；不得用内部模块PASS或普通服务已启动代替。之后按原计划 `LTR10 → LTR11 → LTR12 → LTR01/H2`，前3卡为原REL-LTR-PUSH子卡，LTR01才是唯一发布点；本main文档交接不扩大子卡push/bump/tag权限。

### Task DOC-LTR-02: 跨机器前置任务定义交接

**Task type:** `docs`

**Lifecycle / Acceptance:** `in-progress` / 空（本次用户明确要求，仅任务定义与迁机交接；不宣称FIX05准入）

**Description:** 将当前失败、FIX05及后继卡细则和未决授权完整写入本文件，单独signed/DCO提交并推送main供另一机器接收；保留所有旧失败/STOP及已消费预算。

**Out of scope:** 不实现产品、不改变版本/lock、不批准警告例外、不修改原scoped-traces或网站文件、不取消任何原卡的C/D。

**Current evidence:** 用户2026-10-26明确单文件/main交接指令；当前来源、原件hash和真实失败见本入口。本节是提交前任务定义快照；先冻结最终正文并执行门，随后真实本地Claude ER05，实际终态/session/输入hash写入同一signed提交的message及仓外原件，保持被测/受审/提交正文逐字一致。以下pending不冒充未来验收，最终状态由后继既定同步补记。

**Acceptance criteria:**

- [ ] AC-1: 前置任务、依赖、未决授权和迁机证据缺口均明确登记。
- [ ] AC-2: 原失败、STOP及预算历史保留，不写未来PASS。
- [ ] AC-3: 提交仅本文件，main父提交是现场核验的远端，签名和DCO真实核验。
- [ ] AC-4: 接收方可从远端main取得同一文件与提交。
- [ ] AC-5: 该main SHA实际base全部8个job及CodeQL两个job的run/job/attempt终态逐项核验；失败STOP，pending不计完成，现场workflow变化重核映射。

**Verification:**

- [ ] VER-1: 原生diff/逐文件hash确认唯一文档变更，结构、任务字段、依赖无环及相对链接检查通过。
- [ ] VER-2: 本地Claude审查当前全文与差异，明确 `VERDICT: PASS`；P0/P1关闭，P2具名处置。
- [ ] VER-3: `source .env.test && source .env.live-test && cargo nextest run --test compat_agent_architecture_guard --no-fail-fast --retries 0`，完整现有目标非零；文档消费者扫描若发现其它受影响target，先补门及预算。

**强制本地门：** `cargo +nightly fmt --all --check`、`cargo clippy --all-targets --all-features -- -D warnings`，加VER-3完整target；编译均计入本卡一次窗口。

**Full-suite trigger:** `N/A`（no-release、仅计划文本；自己的nightly fmt/strict Clippy及实际文档消费者不可免，产品完整C由FIX03覆盖）。

**Dependencies:** `DEP-QUAL-SOURCE`；本次用户明确单文件/main交接，不依赖未来FIX05完成。

**Deliverables:** 本文件的任务定义与跨机交接。

**Implementation write set:** 仅 `docs/development/plan/plan-20261025-qualification.md`。本次单文件要求优先于通常同批status行同步；后置状态由root既有协调写集或下一张实施卡补记，不带原status全文。

**Release write set:** `N/A`

**Files likely touched:** 仅上述一文件。

**Docs and compatibility impact:** 无用户命令/配置/协议行为变化，命令双语与网站N/A。

**Rollback mode:** `immutable-release`

**Migration and rollback:** main推送后仅前滚更正文档，保留原失败，不回退远端或改tag。

**Security and privacy:** 正文仅源码锚点/摘要/hash，不含env/token/私钥；仓库不新增证据目录。

**Performance budget:** 本地一次3600秒，包含nightly fmt、strict Clippy、文档消费者target和实际结果核对；编译计入，失败STOP。缓存/工具链在claim前确认；本次是新文档门，不重置旧FIX04窗口。具体执行器及输入先审查。

**Estimated scope:** `S`（1文档，无生产落点）

**Version increment:** `N/A`

**Release boundary:** `no-release`；按用户指令main独立文档交接，无bump/tag/Release；实际main base/CodeQL仍观察，未触发工作流记N/A。

**C/D coverage from:** `FIX-LTR-03`（自身文档门/main D另记；此前最多locally-accepted，文件已提交不等于产品或计划完成）。

**Granularity:** `type=docs; axis=跨机器前置任务定义交接; recovery=immutable-release前滚文档; complete=yes; self-contained=yes; AC=5/20; VER=3/20; landing=0; prod-files=0; scope=S; deps=DEP-QUAL-SOURCE; writeset=本文件独占; release=no-release; split-from=用户20261026迁机交接; exception=N/A`

### 2026-10-26 修订登记

本轮正式登记FIX05（仍blocked）与DOC02，新增DEP-QUAL-F2-ATTRIB/CLOSE及pending决定，明确FIX05→FIX04→FIX02→FIX03条件顺序、原计划H1等独立前置、main分工与单文件提交范围。原四实现卡统计1/4保留在历史段，当前是1/5；文档卡不计入实现卡分母。旧正文的feature PR、旧来源和已消费预算为历史执行记录；本入口覆盖当前接续要求。文档自身Claude/本地门/提交/main D必须待实际取得后记账，不能预填。


## 既有任务与历史执行记录

以下保留已有卡片及全部失败历史；旧来源、feature PR、已消费窗口及旧统计按其日期理解，当前接续要求见文首2026-10-26入口。FIX04 Dependencies/Granularity已补FIX05/F2边，原字段在末尾原样保留。

> **FIX04 C1实际退出终态（2026-10-25 18:59:44 UTC）：** FIX04 C1的精确退出已实际执行：本地Claude退出R1 session ff79d09e-3485-4fdb-9316-258cd5d0b614四项PASS、50输入重核；root采纳后，原生restore及外层工具18851于2026-10-25 18:57:47 UTC实际exit0。四个版本/lock逐字等于published b73/.87，四个0664模式保持，原生stash list确认988ded20及完整列表不变；七路径失败树已在持久私有位置按SHA256/MANIFEST及模式补充保存。两cfg及其余全部tracked blob/mode、原Mega9/publisher9/原Libra4路径WIP保持，index空，仅余本计划/本计划status行/ingress三个脏路径。这是退出执行事实，当前等待退出终态增量单审；不授任何新A/C、commit/push/tag/Release。FIX04原C1机械11阶段0但native审查四FAIL、root STOP保持，Lifecycle仍blocked/locally-accepted，仅旧A1/源审限域有效；storage错误回退P1、reset实际stderr不足暂定P1与其它具名风险尚未解决，资格1/4、原scoped-traces0/26不变。失败C1已无active bump；后续独立Mega102须在本次终态增量单审及自己的具体C2合同通过后才claim。
> 下方候选的“待审/未执行”属于原时点历史，不覆盖本段实际结果；本次增量审查尚未发生。

> **FIX04 C1真实FAIL/STOP与精确退出候选（2026-10-25 18:47:47 UTC，未执行）：** FIX04原C1于2026-10-25 18:10:38 UTC结束，11阶段exit0、私有安装.88及资源后检exit0，5348.666525/10800秒；外层工具96753实际exit0。完整10761身份/21原ignored保留，4个flake/5个失败尝试后native重试成功；实际本地Claude session c7f9fbae-734f-416c-9d9d-3650ede3f824的四项判定全部FAIL，root于18:17:28 UTC记录STOP，C不接受。F1 P1为storage配置读取错误后无条件LocalStorage回退并污染machine stderr；F2暂定P1为reset missing-path失败未保留实际stderr，根因未知；F3 stdin fixture race与F4未执行accept脚本列P2。所有旧FAIL/STOP、A1两flake、37运行时覆盖缺口及21 ignored保持，历史SQLite锁owner/SQL与C2数据库根因未证。本次仅候选RB-FIX04-EXIT-C1：先保存七路径失败树及SHA清单，再用原生Libra精确恢复本卡四个未发布版本/lock到原before-bump（逐字等于b73/.87），保持两cfg和其它全部源码、原stash988ded20、两仓原WIP。经本机Claude EXIT/ER12/H0/VERDICT PASS和root采纳才执行；不reset/force/stash-pop/commit/push，不新开A/C，不授.88再次发布。退出成功仅释放本次失败C1发布占用，FIX04仍blocked/locally-accepted，资格1/4、原scoped-traces0/26不变。后续F1/F2/F3须先审定合法独立修复与依赖；不得把无G13的本仓改成聚合，也不借G08归并独立缺陷。Mega102自己的ER05已PASS，独立C2仍需退出终态与自身具体执行合同PASS。
> 下方日期段均保留原时点事实；本段只提出退出，不把未执行动作写成完成。

> **FIX04自身审查实际终态（2026-10-25T14:24:41.918607+00:00 root采纳）：** 本地Claude `e76e3d02-d544-4259-9f2b-42a937cc49e9` 于14:22:16.368832Z给出ER05/H0/VERDICT三PASS，P0/P1=0；root实算73输入并承接五项P2。源码仍仅两cfg，A1原2个native flake、37个runtime覆盖缺口和1个被断言负例原样保留，未证明baseline/历史根因。最终C执行审查、bump与全部发布C/D仍待，资格1/4、scoped-traces0/26不变。下方日期横幅和A1时点pending判断保留为历史，不覆盖本段实际终态。

> **FIX04实际A1/B与资源终态（2026-10-25 14:16:54 UTC）：** fmt/strictClippy、capture436、architecture65、default/keyring release、full10761/21原ignored已核；2个原生flake和当前runtime覆盖缺口保留，自己的ER05及最终C/D待。资格1/4，未bump/提交/推送；下文旧时点“未claim/待执行”均保留为历史。

> **FIX04实际恢复与实现入口（2026-10-25T12:28:29.801459+00:00）：** 本地Claude恢复R2 session 15ccf47d-7c29-42d1-be62-3515b59119ed ER03/RECOVERY/H0/VERDICT全部PASS，manifest 9fcb72036655861f6cf82c2300ab292ba507d8e878973b6fcca9b845e67ecc96、output e32e43f55b23e038ffc712ba84b00442151f5c58a3988015452c22fc72780121；P0/P1=0，P2 owner=root：审后实际收据入本段、D1终态/资源/自身审查采纳逐hash绑定、旧横幅保留为历史、keyring无release缓存须具体执行器审预算、18个远端job为root实际gh只读观察。按实际native命令完成自有.88四面精确退出为.87；FIX02三个WIP保存在stash 988ded20adb6be7ebbc43ad195fd55768ed69a26，逐blob核对且保留未pop/drop。新branch fix/capture-ingress-profile-20261025来自b73，仅ingress两cfg及两治理；helper恢复published b73字节。21份物理H0文件按受审候选同步。两次只读原生参数前检失败和stash后只读ref检查失败保留，未触发A/C；不重复stash/退出。当前源码已实现，A/B尚待具体执行，自己的ER05、C/D、提交/推送/Release均未完成；版本0.30.87。

> **恢复R2候选（2026-10-25T12:23:22.622751+00:00）：** 保留R1 H0 FAIL；补完整D1终态/自身审查采纳条件、精确源hash、前驱job级收据与四卡warning门。R2待实际Claude；不授新A/C。

> **FIX02 C1 STOP后新恢复候选（2026-10-25T12:03:44.446299+00:00，未采纳）：** Source审ce6fb162已证release profile根因，RECOVERY FAIL的两P1由本候选“FIX04唯一生产I特许+实际A release警告门”修订，完整ER03/H0待。当前顺序FIX01(done)→FIX04(pending)→FIX02(blocked/locally-accepted)→FIX03；四实现卡1/4，DOC-LTR01仍仅locally-accepted等FIX03覆盖，原scope-traces0/26/H1未解锁。以下旧时点PASS/FAIL/STOP与统计全保留，不倒改。


### FIX01与DOC已发生的发布证据（R2补记）

FIX01原D terminal SHA256 `4e2009fab779281e2cb2efbb09aa94ab28d0b619288c34341bca0522757c6cf9`，观察2026-10-25T05:20:51.187280Z；merge f33be958/tree90ca1d20、签名/DCO、v0.30.87和渠道二进制manifest校验均已在该原件实核。本轮重新只读取得以下job原件，仓外FIX01-job-receipts-R2.json SHA256 `c66879aa74f29da560d3fe438a4b62adb473fb3156487955e1417da94a0f8fe8`。

| workflow/run | job/id | 实际结论 | 完成UTC |
|---|---|---|---|
| main-base/40156941519 | compat-offline-core/121060210750 | success | 2026-10-25T05:05:07Z |
| main-base/40156941519 | compat-rustfmt/121060210778 | success | 2026-10-25T03:07:03Z |
| main-base/40156941519 | compat-scoped-input-windows/121060210810 | success | 2026-10-25T03:17:27Z |
| main-base/40156941519 | compat-owner-liveness-macos/121060210816 | success | 2026-10-25T03:43:09Z |
| main-base/40156941519 | compat-redundancy/121060210836 | success | 2026-10-25T03:06:11Z |
| main-base/40156941519 | opencode-export-sandbox-linux/121060210851 | success | 2026-10-25T03:08:43Z |
| main-base/40156941519 | compat-clippy/121060210918 | success | 2026-10-25T03:10:44Z |
| main-base/40156941519 | compat-network-remotes/121060210942 | success | 2026-10-25T03:30:46Z |
| main-CodeQL/40156941508 | security-codeql-actions/121060210278 | success | 2026-10-25T03:06:41Z |
| main-CodeQL/40156941508 | security-codeql-rust/121060210465 | success | 2026-10-25T03:38:17Z |
| release/40157032761 | build-and-upload (libra, x86_64-pc-windows-msvc, windows, amd64, windows-latest)/121060495046 | success | 2026-10-25T03:30:45Z |
| release/40157032761 | build-and-upload (libra, aarch64-apple-darwin, darwin, arm64, macos-latest)/121060495166 | success | 2026-10-25T03:28:24Z |
| release/40157032761 | build-and-upload (libra, aarch64-unknown-linux-gnu, linux, arm64, ubuntu-24.04-arm)/121060495174 | success | 2026-10-25T03:19:00Z |
| release/40157032761 | build-and-upload (libra, x86_64-unknown-linux-gnu, linux, amd64, ubuntu-latest)/121060495177 | success | 2026-10-25T03:22:18Z |
| release/40157032761 | update-homebrew-tap/121064822971 | success | 2026-10-25T03:30:55Z |
| release/40157032761 | upload-install-scripts/121064823276 | success | 2026-10-25T03:31:01Z |
| release/40157032761 | request-stable-manifest/121064823359 | success | 2026-10-25T03:31:02Z |
| release/40157032761 | verify-homebrew-formula/121064852609 | success | 2026-10-25T03:31:22Z |

DOC PR636原merge收据05:42:10Z为b73e98a/tree73eba0d7；main-terminal-audit于06:48:23Z核base八job与CodeQL、network原3例及10761/21原ignored/0retry，项目warning0。其自身A/B/Claude及signed/DCO已发生，仍locally-accepted等FIX03覆盖；旧未提交/待main横幅只属历史，不回改。

### 新FIX04预算依据与R1处置

FIX01真实C release-build于2026-10-24T23:09:42.190321Z至23:31:27.838239Z，耗时1305.648秒；FIX02 C1从10:09:46.616474Z claim到11:24:46.666098Z进入release已4500.050秒，release只运行约551秒即因实际项目warning被停止，不能把该部分当完整build耗时。新FIX04沿用已有target、jobs=1：估算前序门4500秒、default release保守1800秒、keyring增量保守1800秒、后检180秒，约8280秒低于候选10800秒；这些是claim前可否准入的估算，不是实测保证。具体执行器审查必须核同toolchain/target/profile缓存及两release重编译范围；依据不足先ER03，不在claim外预编译、不延长已消费窗口。

R1真实ER03/RECOVERY PASS、H0/VERDICT FAIL（sessionbb82411c-1e59-4e52-a7f1-930ab10005b8，output c9b9e57128d488b10393ff21be13ccbd1da5107231e486965780f0533c1413b1）保留。P1-H0由上面完整终态+采纳收据条件修订；D1现已实际GO/资源closed，ownClaude06c202a4 PASS，root于12:22:25Z采纳并独立重核stdout/编译诊断。G13仍pending。P2 owner=root：以candidate.json及before-R2保存像为权威、生成差异本脚本留存，不重跑旧生成器；ingress锁定精确SHA；FIX01 job收据与DOC当前状态已补；四卡措辞及全局warning门已补；预算按上述claim前实审；原Libra状态旧资格行由下一次原scope治理承载同步，其带日期旧快照不倒改。R2尚待实审，未恢复版本/stash/branch/实现/claim。

## 本轮受审恢复动作与来源边界（未授执行）

本候选以实际published b73的两个blob为底重建，不复制stash中旧WIP全文。仅选择可核的FIX01实际C/D、FIX02 A2/own/C1 STOP事实入文；FIX04是新的独立代码轴。FIX01完整收口原件root-card-closeout于05:20:51Z记done/complete，源审/C/签名/PR635/merge f33be958/v0.30.87与全部D实核；旧release warning P2作为历史保留。DOC PR636/main已实际成功，但本no-release卡完整C/D仍继承FIX03，不提前done。

计划PASS后才可执行以下精确恢复。入场先读新远端，main若不是b73或v0.30.88的tag/Release已被占用则STOP/ER09重审，不覆写。实核qualification为原生独立clone .libra实目录/no.git、HEAD/origin b73、branch fix/qualification-index-drain、index空、恰七dirty/无新untracked、源/工具身份与C1保存像一致；无他人写者。已有七路径failed-C-tree和C1 final-inputs/tree7388378均保存。

1. 逐一核C1的四个before-bump文件hash等于b73的Cargo.toml/Cargo.lock/install.sh/install.ps1 blob、当前四文件等于失败C1最终hash/mode；保存原件后只write这些文件的原始字节回各自前像并保持既有mode。Cargo.lock是完整已核原件恢复，不手改其版本行；不reset、改远端或他人source。核四面.87 parity与helper5025/两WIP文档字节不变。
2. 用原生 `libra stash push -m "FIX-LTR-02 preserved before profile-warning repair" -- tests/helpers/reasoning_artifact.rs docs/development/plan/plan-20261025-qualification.md docs/development/plan/plan-status.md` 只保这三个own WIP；先核暂存空，后核stash实际object/ref/tree恰含三路径delta且helper5025可回取、七保存像完整、工作区clean。保留stash，不pop/drop。目录、新原件与对象不视为窗口重置。
3. `libra switch -c fix/capture-ingress-profile-20261025 b73e98a77ecc7c3db1ea905ab55478785d76c041`；只应用本受审候选两个治理文件与ingress两cfg；ingress最终字节必须等于source审candidate-ingress.rs SHA256 `2b7aa33698c2072d160cc6bea7801ffb8fb180a13bb0ebe78901ff888b7bb778`。完整其它tracked blob/mode必须等于b73，尤其reasoning_artifact.rs等于published blob，不能含5025 WIP；无ed566b0或原未发布Traces治理祖先/源。无force/reset。新branch与唯一root publisher用既有独立clone/target，不创建共享Libra worktree。
4. A/C分别只能在本机现行Mega D1实际终态/资源后检后、具体执行器单审PASS/实核toolchain/env/暖cache后唯一claim。本恢复不授FIX02 C2，不把已耗5051秒视为可用余额。FIX04实际C/D后，先精确前滚整合到新上游，再只恢复helper保存像，两个重叠治理以新published字节为底三方重建；禁止pop旧文档/旧bump。之后FIX02新的ER03决定原A2/own的有限carry及新版本/新C，不预填PASS。

两原日期计划/SSOT/索引只做本地H0同步，仍由既定MTR104候选合法承载点/原LTR01覆盖，不能随FIX04推送原未发布Traces治理。当前Mega D1已于11:39:55Z真实claim，冻结的原三Mega与原Libra三治理不改；本轮九逻辑治理H0候选只能在D1协调器实际退出、资源closed，且D1自身review（原工具句柄24409）已有实际终态并完成root采纳/收据之后，按当时before hash逐项匹配采纳；任何不匹配先重建overlay并重审。A/C Cargo与发布仍root全局串行，stage/commit精确，不用commit-a/无admin bypass。新的恢复方案不放宽任何版本、测试、Review或远端D门。

> **2026-10-25 04:25:21 UTC 当前：** DOC-LTR-01源码已采纳、A/B实际通过（48命令、22组help相同、874.796秒），当前`in-progress/locally-accepted`；自己的本地Claude ER05、提交与PR/main D待，完整覆盖仍由FIX03承担。FIX01已合并f33be958并发布v0.30.87，release/CodeQL实际成功，main base仍待；三实现卡0/3。以下旧快照保留。

> **2026-10-25T03:14:21.976945+00:00 PR635实际合并/发布触发与文档修复候选：** FIX-LTR-01本地C所需门/完整默认10761、release build/install已核；签名/DCO head2852497、PR635全部13 checks成功后gh squash merge `f33be958dcc3266df3a6ab796ed90d1766f3cd93`，树90ca1d20与最终C相同、GitHub签名/DCO已核；人工Release/tag v0.30.87已触发release run40157032761，main base40156941519及CodeQL40156941508仍待，未done。原attempt1 eli-MI fontconfig失败与本地retry/资源缺口保留；用户移除该runner，attempt2在Mechaevo成功，非证明eli-MI已修。用户要求Libra下一提交只修104条项目Rustdoc warning，明确排除第三方proc-macro-error2。DOC-LTR-01候选见文末；三张实现修复仍0/3，本新增文档卡独立记账、由FIX03覆盖。以下旧时点记录均保留。

> **2026-10-24 21:42:59 UTC FIX-LTR-01本地A实际终态：** 独立clone的唯一结构oracle修复已通过nightly fmt、strict all-target/all-feature Clippy、exact1/1、完整target54/54及默认全量10761/10761的框架核验，exit均0；21原ignored不变，无失败/flake，3962.768/7200秒、2126输入无漂移。输出38个runtime skip标记已逐身份/逐文字归属，其中1个是缺interpreter的负向断言，37个是既有feature/live/工具/平台覆盖缺口，不能据此宣称所有live行为已测。唯一遗留worker归属当前status-pause无helper用例，已核PID/start identity并通过pidfd人工SIGTERM，确认exit/reap；原资源缺口保留，FIX03未修。A仅本卡locally-accepted，自己的真实Claude ER05、patch/C/D均待；版本仍0.30.86、无提交/push。三卡正式0/3，FIX02/03 pending。以下旧时点事实保留。**

> **2026-10-24 20:23:39 UTC FIX-LTR-01实施入口：** 本计划工作副本位于独立原生clone `/media/eli/data/libra-qualification-20261025`，branch `fix/qualification-schema-doctor`，基线0ca47cc。FIX11实际done/complete且两个告警fixed；恢复R5方案/治理审查PASS。现仅改既有结构字串加入Ui排除，生产CLI、runtime断言和相邻startup guard不变。A未启动、自己的代码ER05未取得；没有bump/提交/push/Release/CI新结果。外部修复仍0/3。下方原时点快照保留。

> **FIX-MTR-11实际收口（2026-10-24 20:22:25 UTC）：Dependabot23/24已由GitHub实际标fixed，fixed_at分别2026-10-24T20:16:51Z/20:16:52Z，均非dismissed；main与v0.42.70解引用均932731d9b7f274638d2fb1bfa10d571207a54a39，人工Release已发布，main Harbor40112708799及tag Docker40112738161三job全部success。自己的Claude R3 ER05/H0 PASS、21 targets/3085 PASS/0 FAIL/3原ignored的五门C与signed/DCO均已核，现done/complete；RED1 FAIL/STOP与原P2不改。原件D-alerts-fixed-terminal-20261025 SHA256 2585fecfeacde5f37a6bae1eed701898e20f8254e11d704cc08e017129a0e7f9。Mega15/112（活动15/111）、Libra0/26（活动0/25）、联合15/138（活动15/136）；外部恢复0/3。恢复方案R4及治理自身R5真实本地Claude均PASS。安全前置已解除；下一步优先在独立Libra clone核入场并实施FIX-LTR-01，MTR101亦仅解除本安全前置，不跳过各卡独立门。原Libra merge签名政策、H1/LTR10及LTR01仍未完成。**

> **2026-10-24 20:15:32 UTC 治理采纳与自身审查实际收据：** 九份文档已按受审hash采纳；真实本地Claude R5 `DOCS_ER05_VERDICT / ER03_VERDICT / H0_SYNC_VERDICT / VERDICT: PASS`，P0/P1=0，session `e72fd9de-c479-457d-9eff-71ea07a7b49d`，审查输出SHA256 `cd632543ed9722003ebdda14f6e6f32512eb84f7b8e63e0940ce13a96291d4b7`，manifest SHA256 `096eae5326d3714271eaa2c4c3ba59dfa1f7c1156b6aa3d48d8bdf5c7f0fa78a`。本行是审查之后的事实收据，不声称审查器读过其自身后产出的输出。root接受非阻断P2：例行维护收据/预算约束以下方具名表为准，执行前并入对应条款并实审；历史横幅保留，本文首收据为当前时点；终态执行器仍须生成/冻结/实审。缓存复制实际exit0，31147文件（deps11178/build4563/fingerprint15406）、19035122524字节，root已核排除第一方/测试可执行文件及硬链接；这只完成源准备，不是验收。两仓精确signed本地提交待，尚未push；原治理由既定MTR01/LTR01承载。三张修复仍pending、0/3；原14/138不变；FIX11两告警实际fixed及逐卡入场条件未满足前不实施，不复用任何旧FAIL/STOP窗口。

> **2026-10-24 20:09:49 UTC 当前审查状态：** 恢复方案R4真实本地Claude ER03/H0/VERDICT均PASS，P0/P1=0；原R1～R3 FAIL保留。本文现为P2事实修订后最终治理候选，自己的DOCS-ER05/H0审查待取得；三卡仍pending/未实施、未测试、未提交、未发布。

# Libra 2026-10-25 来源资格前滚修复

> ER-03 候选，尚未采纳、实施或开启测试窗口。依据 [plan-template](plan-template.md) 与仓库 AGENTS.md；按用户当前指令只使用真实本地 Claude CLI 审查，不运行隔离 Codex。本文当前承接下列四个修复行为轴（新增FIX04为本轮待审候选），不替换 `plan-20261015.md` scoped-traces 计划（原工作区现行未发布治理另行承载）。

## 目标与来源

恢复选定上游 `0ca47cc147b2099ffb835ade91272c4cfe1efd21` 的真实来源资格，保留新 UI 行为和全部原测试断言。原工作区 HEAD `ed566b0229dfe2805a2317f889355e449f7d4e77` 是外部合并，包含未发布治理且没有签名/DCO；本计划不改写它、不替它补造签名，也不从该 HEAD 推送修复。

原首次资格已 FAIL/STOP：coordinator exit1、native full exit100，2026-10-24T19:12:05Z；4982.973/7200秒，非超时。10761 eligible 身份全部到达，10760最终通过、1最终失败、21 source ignored、1 flaky success；实际失败尝试4次（3+1），最终报告重复的失败标题不重复计数。fmt/strict Clippy/build 已 exit0；doc/network 未开始。2125输入未漂移。原件在仓外 `libra-upstream-qualification-20261025/r2`；terminal SHA256 `7f1a5258024676525210f3cd6a48c834d8450dd209a6251d3b25348481fc02a6`，full JUnit `e9f08229091d55e028307cbe1d98f71b477b63299be7110532f564ee51a8c5fa`。现存 proc-macro-error2 future-incompatibility 提示仍是具名风险，不能称零诊断。

实际本地 Claude 失败处置审查：session `a87e189c-332a-4dd0-9db9-c412cc81e314`，2026-10-24T19:20:05.975264Z，exit0、FAILURE_VERDICT/VERDICT PASS、无漂移，输出 SHA256 `7e9531e8736c8d67b9e83ea17b954885f3694dc918ae9047b9bdd5c357ba4edf`。它只认可失败记账与三个独立修复方向，不是产品/资格 PASS，也不是本计划的 ER-03 准入。

### 独立发布与原工作区保护

- root 是唯一写者和串行发布者。使用真正独立的原生 Libra clone（不是共享 HEAD/index 的 worktree），从现场核实的 origin/main 建立短期 feature branch。先检查类型、HEAD、远端 ancestry、工作区/索引、并发写者、身份和 vault 签名；若上游变化，先审差异并重核本卡，不覆盖新增提交。
- 每卡独立 patch、A/B、实际 Claude ER-05、C、PR、merge、tag/人工 Release 和真实 D；不建立 G-08 家族或 G-13 聚合，不把测试修复塞进 LTR-01。未来卡的实现不得混入前一卡提交。
- 修复 checkout 的公开写集仅本计划、基于实际已发布上游版本的 `docs/development/plan/plan-status.md` 中本计划登记行、当前卡已登记测试文件、该卡实际版本面/工具链 lock；FIX-LTR-05的生产/测试/文档候选写集只按文首VER-6后冻结的正式I准入；FIX-LTR-04额外允许 `src/internal/ai/capture/ingress.rs` 两个方法定义前各加一行 `#[cfg(any(test, debug_assertions))]`，不改其方法体或生产入口。与原工作区未发布 scoped-traces 治理提交做 ancestry 和逐文件 diff 守卫，禁止携带 `ed566b0`、其本地治理父链或原三份治理 WIP；plan-status 只在上游版本中新增/更新本计划行，禁止复制原工作区整份文件。本文是新的独立修复计划，不复制未发布的 scoped-traces 文本到发布分支。
- 原工作区两仓既有八份治理文件只登记事实/外部依赖，仍由原 MTR-01/LTR-01 与 Mega 后续既定 C/D 承载；其中原 scoped-traces 治理增量不属于五张实现卡的发布写集；plan-status 的新修复计划行是下述独立同步写集。新修复发布后，原工作区只通过原生前滚整合保留双方历史；若本文本地未跟踪副本与远端已发布版本碰撞，先保留快照、核同源与内容，再只处理 root 自建副本，不清理无关文件、不 reset/force。
- `ed566b0` 的签名/DCO 政策仍须在原工作区 LTR-10/LTR-01/任何 push 前明确处置。新修复不以其为祖先，不借独立发布解决或掩盖旧合并政策。

### 使用规则与共享状态保护

依模板使用规则，新建本计划必须在 `plan-status.md` 的「计划一览」登记；推进每张卡时同一变更更新该行的卡状态、实际版本/commit及UTC秒级时间。每卡 I 都包含本文件和该行。原工作区先增加候选行；独立发布 clone 从实际上游 plan-status 开始，只新增/修改该行。逐行 diff 守卫必须证明原 scoped-traces 未发布段没有被带入。后续原生整合时对同一状态行逐项合并，保留双方历史，不以整文件覆盖解决冲突。原「LTR01唯一承载」指 Traces 治理的发布，不涵盖此独立恢复计划。

### 执行预算与停止规则

用户已明确授权为完成两个计划进行必要操作；本候选把必要的前滚修复具体化，必须先获得实际本地 Claude ER-03/H0 PASS，才按各卡新来源开始。原7200秒 claim/STOP封存，不使用剩余约2217秒，不重开旧FIX3/LTR00的3/3诊断及8/8窗口，不以换目录重置旧预算。失败处置 review 自身不授执行权。

候选独立 A 窗口：FIX-LTR-01一次7200秒，FIX-LTR-02/03/04/05各一次10800秒（共享夹具真实云消费者或最终doc/network，均包含完整默认套件）；每卡最终 bump 后本地 C 各一次10800秒；FIX-LTR-03 的 C 包含完整全量、doc及原3例network。五张实现卡实际D全绿后另有一次计划最终资格窗口10800秒，执行最终发布树fmt/clippy/full/doc/network，不以C代替模板计划完成门。上限是新独立来源的命令编排预算，不修改任何测试内部 deadline、并发组、断言或原 nextest retries。依据本机已测 build+listing+full4983秒及 strict Clippy约876秒，额外预留最终 release build/网络 feature 编译；不把冷构建慢当代码原因。本地窗口从第一条Cargo命令到本地门/构建/安装结束，包含编译和listing；PR CI、merge及Release D远端等待不计入该本地执行预算，单独登记run/job/ref，观察超时不重启作业、不记PASS。任一意外失败/超时/漂移立即保留终态并STOP，不自动再开窗口；后续只读归属与另行ER-03，不因剩余时间继续命令。

所有 Cargo 门 source `.env.test` 和 `.env.live-test`，核非秘密 presence/控制项、原工具链与 Cargo 配置；缺文件或必要凭据停止相应门。产品/集成测试延续已审隔离环境、已核 CARGO_BUILD_JOBS=1、umask0022，不复制用户凭据到仓库或日志。本次DOC02仅运行源码结构守卫及fmt/Clippy，具体合同保持登录HOME原值、只设CARGO_BUILD_JOBS=1并加载双env；target主要读源码；唯一runtime守卫 `agent_kind_enum_sql_check_and_doc_roster_stay_in_sync` 自建TempDir repo/home，用CARGO_BIN_EXE_libra init后只读SQLite并显式close，实际隔离依靠测试自身HOME重定向及执行器在调用环境、双env加载后均拒绝XDG_CONFIG_HOME/LIBRA_CONFIG_GLOBAL_DB/LIBRA_HOME三个override；LIBRA_TEST_HOME在产品二进制不生效，不能充当隔离证据。通过该检查后不访问用户库/live provider。环境差异及该fixture资源终态须在准入复审确认。listing与JUnit按(target,case)精确核数、保留原ignored/重试及成功输出中的runtime skip；缺环境、缺 feature、early return不冒充所需行为通过。代码完成、自身review、提交、push、D逐项分别登记。

构建缓存：独立 clone 使用自己的 `target/`，不与原工作区共享可写 target。不携带旧日志作为新证据；确认原 target 无 Cargo/rustc 写者后可复制缓存到新 target（禁止硬链接），记录工具链/配置/来源，Cargo仍在新来源重新核fingerprint。不能完成安全复制则使用冷 target，并在claim前重新审预算；不在claim外预编译来隐藏成本。当前7200/10800秒是暖缓存依据下的候选上限，不保证冷构建能完成，不得到时自动续窗。

完整默认套件的唯一命令为 `source .env.test && source .env.live-test && cargo nextest run --all --no-fail-fast --retries 2`；nightly fmt为 `cargo +nightly fmt --all --check`，严格Clippy为 `cargo clippy --all-targets --all-features -- -D warnings`。所有门读取并绑定同一受控输入，不改变nextest并发/配置。

每卡 A 包含其 Verification、nightly fmt及strict all-target/all-feature Clippy；触发全量的卡另跑原双env默认全量。自己的 ER-05 PASS 后，C严格按模板ER-04/08顺序：版本面parity→实际patch→工具链刷新lock→最终fmt/clippy和本卡完整全量/所需focused→`cargo build --release`（五张实现卡全局C⑤要求项目warning=0，唯一排除为用户具名proc-macro-error2 future提示；FIX04/FIX05另加keyring release门）→将本卡release二进制安装到仓外、每卡独立的 `install/bin/libra` 并以绝对路径核version/hash（复制+chmod0755，hash须等于本次target/release/libra）；不覆盖原工作区使用的libra或改变全局PATH→精确`libra add`及signed+DCO提交→feature branch/PR→该head的base/CodeQL全部适用job通过→squash merge并核merge SHA/tree/DCO/签名；merge tree必须等于已测bump后tree，任何上游变化或树不同即STOP并ER-03，不tag、不借旧PASS→人工Release及版本tag→release工作流与渠道artifact真实核实。遇分支保护、必需他人review或环境approval，保留实际pending状态；不使用admin bypass、不自行批准。未完成D不得done。编译诊断按实际记录；若既有future-incompat提示违反当次发布门，先独立归属，不静默豁免。

### 被测树、逐卡证据与终止链（执行前预登记，模板v2.14）

1. **每卡C前冻结。** 先把已真实取得的A/B、自己的Claude ER-05、前一卡已发生的C/D写入本计划与plan-status本计划行；当前卡只写实际已知状态，C/D未知项明确pending，不预填成功、未来commit或run。完成patch与工具链lock更新后冻结全部输入；本次C就在这棵树上执行。C到提交/PR/merge之间不再修改任何跟踪文件，包括状态和证据文档，因此最终提交树必须逐字等于C被测树，merge tree也必须相等。任何漂移STOP/ER03；不以文档白名单免除这三个树相等的约束。
2. **C/D后置证据归属。** FIX01的真实C/D、提交/tag/version/job/UTC收据由本次FIX04恢复准入在两份既定文档内补记；FIX05后置真实C/D由FIX04重新开工时在本计划与plan-status本计划行这两个既定文档补记；FIX04后置真实C/D由FIX02重新开工时补记；FIX02由FIX03同样补记。前一卡只有实际C/D全绿后才启动后继，不以文档中尚待补记的旧快照当证据。状态推进与证据更新在同一文档变更中完成，并在后继自己的C前冻结，后继C实际覆盖这些新增字节。若后继尚未开工，真实结果保持待补记，不提前执行后继、不发布单独的未登记补记提交；若后继STOP，随该次ER-03前滚修订在本计划正文同步补记前一卡已发布的C/D事实，仍用两文档既定写集与自身文档验收，不允许事实永久只留仓外。
3. **FIX03 done与计划完成分开。** FIX03自己的AC/VER、A/B、自身ER-05和独立C/D实际全部通过后才可done/complete；计划最终资格不是其AC的递归前置。随后在FIX03实际merge SHA的冻结树运行已登记的一次10800秒计划最终资格（fmt、strict Clippy、双env全量、doc、network原3例），再取得真实本地Claude来源资格PASS。该窗口不能包含之后才写出的终态文档，也不能把已发布产品树换成新的上游树。
4. **唯一终态登记提交。** 计划最终资格通过后，在独立clone创建一笔纯文档登记提交，I精确为 `docs/development/plan/plan-20261025-qualification.md` 与 `docs/development/plan/plan-status.md` 的本计划行；R=N/A、version=N/A、无tag/Release、不改变任一产品/测试/配置/lock/版本面。该提交承接FIX03真实C/D、最终资格和五张实现卡状态，以及它自己的真实Claude文档ER-05收据；核对实际证据，登记本终态文档自身D仍pending。它是本计划预登记终止链末端，不是第六个产品修复或新版本发布点，不替代五张实现卡各自C/D。没有独立发布原工作区Traces治理的权限。
5. **终态登记验收。** 终态文档相对FIX03 merge树只允许上两路径；逐行证明plan-status只改本计划行，其它来源逐文件相同。该文档差异不能宣称被前面的全量覆盖：先在候选文档完成结构/链接/状态一致性验收，再获得真实Claude文档ER-05 PASS；仅追加该审查的实际session/输入hash/结论/P2处置/UTC收据，不能预填或修改其它内容。随后在含已登记ER-05收据的最终字节上重新做结构/链接/状态一致性、`cargo +nightly fmt --all --check`、`cargo clippy --all-targets --all-features -- -D warnings`，及 `source .env.test && source .env.live-test && cargo nextest run --test compat_agent_architecture_guard --no-fail-fast --retries 0`（实际非零完整target，包含读取plan-status/long的守卫）；另只读扫描全部文档消费者，若还影响其它target则执行其完整target，输入超出此两路径或超出已登记预算先ER-03。预登记本地一次3600秒，仅这些文档相关门，无新的产品全量窗口。追加审查收据以外的任何改动都必须重新审查并重跑相关本地门；不得因此自动追加3600秒窗口，超出预算先ER-03。冻结最终文档字节，精确signed+DCO提交并核签名；走feature PR，base八job/CodeQL两job全绿后squash，核提交树=merge树=最终本地门被测树（含已登记的ER-05收据）；审查原输入与唯一后置收据差异分别记录，不声称Claude读过自己后来产生的输出，再观察merge SHA的全部适用base/CodeQL。无tag则release.yml不触发，具名N/A而非success。此文档提交的产品C覆盖仅继承FIX03已发布产品树；新增文档字节由自己的本地门与真实PR/main D覆盖。
6. **消除自引用，不静默完成。** 最终登记提交自身SHA、run ID、结论和观察UTC，只能推送后取得。依模板v2.14不再为它新建本计划提交：在下一次 `plan-status.md` 例行同步中补记，搭车既有维护提交；若到下一工作日尚无例行同步，以一笔明确的计划状态例行同步补记，属于plan-status维护，不属本计划终止链、不重启终止链。例行维护提交仍须精确signed+DCO、结构/链接/nightlyfmt/strictClippy与完整compat_agent_architecture_guard target（双env、retries0），真实Claude文档审查及PR/main base/CodeQL；不因属于维护而免门，后续若暴露其它读者则按实际影响补齐。这是库内证据落点，须实际执行；末端D未绿时不报告计划完成。原两计划接收本资格时同时登记该末端ref/D以及例行补记归属，不改原卡独立门。

任何最终登记门/PR/main D失败都保留实际FAIL、STOP并前滚处置，不为截断终止链豁免失败。计划最终资格与末端文档review都必须是实际结果，不预写未来PASS。仓库内不新增证据文件/目录；摘要、SHA、计数、run/UTC与修订历史落在上述计划正文，唯一例外是第6项模板指定的plan-status例行补记。

**终态B的只读执行器：** 使用仓外 `/tmp/libra-qualification-recovery-plan-20261025/terminal-document-check.py`（在该末端执行前生成、hash冻结并纳入真实Claude输入，不属于产品源码）。执行命令为 `python3 /tmp/libra-qualification-recovery-plan-20261025/terminal-document-check.py`，以已核FIX03 merge SHA、候选树及两路径allowlist为显式冻结输入；缺输入即失败。必须以原生Libra diff逐路径证明只有本计划与plan-status本计划行变化；全其它来源hash一致；卡片/索引状态与已发生原件逐项对应；本计划全部必需章节/五张实现卡字段存在；相对链接解析到当前clone；已登记审查收据只作后置事实，不改审查结论或其它字节；保留所有旧FAIL/STOP。自动结构结果不替代人审语义和真实门结果。

## 依赖登记表

| ID | direction | 类型 | 对象 | Owner | 产物与可用性判据 | 证据 | 超时与失败策略 |
|---|---|---|---|---|---|---|---|
| DEP-QUAL-SOURCE | incoming | 上游revision | libra-tools/libra@b73e98a77ecc7c3db1ea905ab55478785d76c041（初始0ca47cc保留为历史） | root | 独立clone的实际远端来源、允许差异及签名工具链已核；变化先重审 | 本文来源与精确比较记录 | 不追写原工作区，不用旧资格冒充新树 |
| DEP-QUAL-F2-ATTRIB | incoming | 只读归属 | reset真实stderr缺口与当前源码 | Libra接收Agent | Claude审定same/separate/unknown决定；unknown不代表关闭；不写源码不跑Cargo | 当前待审 | FIX05前置，无新诊断窗口 |
| DEP-QUAL-F2-CLOSE | incoming | P1关闭 | reset失败真实机制 | Libra接收Agent | 同轴则FIX05实际C/D+关闭审查，另轴则新FIX实际C/D；unknown不关闭 | 当前未解决 | FIX04前置；不作为FIX05前驱；新FIX不自动继承warning例外 |
| DEC-QUAL-05-RELEASE-WARNING | incoming | 用户决定 | FIX05唯一版本的两方法dead_code例外 | 用户；Libra接收Agent记录 | 用户原话已记录：UTC 2026-10-26T05:40:46Z；范围只有 from_payload 与 from_payload_with_hint_policy；期限到下一个 patch 标签；v0.30.88 可以带上这两条诊断。session 4042079a RELEASE_ORDER_VERDICT PASS 且 VERDICT PASS | 2891a30b 仍是当时的 FAIL；现行决定是 GRANT-ONE-VERSION | 不扩大例外；不把 cfg 放进 FIX05；F2 unknown 仍不关闭 |
| DEP-QUAL-SECURITY | incoming | 跨仓前置 | Mega2 plan-20261015 FIX-MTR-11 | root | 既定安全卡完整C/D及两个告警实际fixed；不改变其不自行延期条款 | FIX11已done/complete，932731d/v0.42.70；23/24实际fixed于2026-10-24 20:16:51/52 UTC，见文首真实D收据（此前open保留为历史） | 未满足不实施本计划；只做恢复治理与审查 |
| DEP-QUAL-PUBLISH | incoming | 发布前置 | 独立clone与实际GitHub工作流 | root | 发布branch不包含原未发布治理祖先/文件；vault签名与DCO、版本面、PR/Release权限与artifact门实际可用 | 开工逐项原生核对，不能用gh浏览器登录互代 | 分叉/缺凭据/来源变化先停止归属，不force |
| DEP-QUAL-HANDOFF | outgoing | 来源资格 | plan-20261015 LTR-10 / LTR-01 | root | 五张实现卡独立C/D完成、FIX03 bump后C及发布后最终资格窗口全量/doc/network与本地Claude明确PASS（绑定实际merge树）、终态登记提交实际PR/main D通过并具名例行补记，再核受控整合树、原工作区受控整合/受影响门通过 | 后续真实提交/版本/远端ref/原件，当前无 | 来源资格未闭合不得解除LTR-10前置；不更改H1或替代LTR各卡门 |

## 实施顺序与发布窗口

`DEP-QUAL-SECURITY + DEP-QUAL-SOURCE + DEP-QUAL-PUBLISH -> FIX-LTR-01 -> FIX-LTR-05P -> FIX-LTR-05Q -> FIX-LTR-05R -> FIX-LTR-05S -> FIX-LTR-05T -> FIX-LTR-05U -> FIX-LTR-05V -> FIX-LTR-05 -> FIX-LTR-04 -> FIX-LTR-02 -> FIX-LTR-03 -> DEP-QUAL-HANDOFF`；另有 `DEP-QUAL-F2-ATTRIB + DEC-QUAL-05-RELEASE-WARNING -> FIX-LTR-05`，另有 `FIX05/具名新FIX -> DEP-QUAL-F2-CLOSE -> FIX04`。`FIX-LTR-05V` 当前切片只含 AutoFollow 的批量对象探测。配置失败后端仍未做，所以 FIX-LTR-05 仍 blocked。顺序用于共享本计划/版本面的串行发布，不表示合并发布。先修确定性全量阻断，再修共享fixture，最后完成MCP测试清理与整份修复计划的最终资格。每卡真正 C/D 闭合后才发布下一卡。`FIX-LTR-05P` 到 `FIX-LTR-05V` 都是 family child，不占用 v0.30.88。

### 实际 D 映射与现场复核

当前 `base.yml` 为PR及main/release/** push，`codeql.yml`为main目标PR及main/release/** push，均无paths过滤；实际job/if必须在每卡开工重新读取完整workflow，绑定最终head与merge SHA，不能只看workflow标题。`release.yml`由v* tag触发，当前build-and-upload有linux amd64/arm64、darwin arm64、windows amd64四个平台并通过OIDC release API上传；完整发布job/渠道探针一并核真实成功。失败前滚，不回写版本tag，不伪造未触发workflow通过。


### 五张实现卡共同适用的逐项D清单

每卡逐项登记run/job/attempt/ref/实际终态；不得用一条workflow success代替漏项。无顶层job if的项均适用；步骤条件和后续workflow变化在开工重新核。PR被测merge-ref、PR head和main merge SHA分开记录。

| Workflow | Job | 触发事件与ref | 判据 |
|---|---|---|---|
| `base.yml` | `format / compat-rustfmt` | PR目标main（refs/pull/<n>/merge及head）和merge后push refs/heads/main；base与CodeQL均另适用release/** push | 格式零diff |
| `base.yml` | `clippy / compat-clippy` | PR目标main（refs/pull/<n>/merge及head）和merge后push refs/heads/main；base与CodeQL均另适用release/** push | all-target/all-feature lint零warning且rustdoc links通过 |
| `base.yml` | `owner-liveness-macos / compat-owner-liveness-macos` | PR目标main（refs/pull/<n>/merge及head）和merge后push refs/heads/main；base与CodeQL均另适用release/** push | 真实macOS具名owner safety用例非零且全通过 |
| `base.yml` | `scoped-input-windows / compat-scoped-input-windows` | PR目标main（refs/pull/<n>/merge及head）和merge后push refs/heads/main；base与CodeQL均另适用release/** push | 真实Windows scoped input全部具名门通过 |
| `base.yml` | `redundancy / compat-redundancy` | PR目标main（refs/pull/<n>/merge及head）和merge后push refs/heads/main；base与CodeQL均另适用release/** push | 现行冗余/生成表检查全部通过 |
| `base.yml` | `test / compat-offline-core` | PR目标main（refs/pull/<n>/merge及head）和merge后push refs/heads/main；base与CodeQL均另适用release/** push | 实际L1+已注入L2/L3与独立feature步骤全通过，记录skip/flake |
| `base.yml` | `network-remotes / compat-network-remotes` | PR目标main（refs/pull/<n>/merge及head）和merge后push refs/heads/main；base与CodeQL均另适用release/** push | 真实network-remotes具名目标通过 |
| `base.yml` | `opencode-export-sandbox-linux` | PR目标main（refs/pull/<n>/merge及head）和merge后push refs/heads/main；base与CodeQL均另适用release/** push | Ubuntu隔离工具及具名export sandbox门全通过 |
| `codeql.yml` | `analyze / security-codeql-actions、security-codeql-rust` | PR目标main（refs/pull/<n>/merge及head）和merge后push refs/heads/main；base与CodeQL均另适用release/** push | 两个matrix job分别success |
| `release.yml` | `build-and-upload × linux-amd64/linux-arm64/darwin-arm64/windows-amd64` | 人工创建版本tag/Release触发push refs/tags/v<本卡版本>，解引用为已核merge SHA | 四平台编译上传全部success，digest/size与下载artifact匹配 |
| `release.yml` | `upload-install-scripts` | 人工创建版本tag/Release触发push refs/tags/v<本卡版本>，解引用为已核merge SHA | 脚本上传成功、发布版本面与CDN脚本实际一致 |
| `release.yml` | `update-homebrew-tap` | 人工创建版本tag/Release触发push refs/tags/v<本卡版本>，解引用为已核merge SHA | 实际tap更新成功并绑定本卡版本/digest |
| `release.yml` | `verify-homebrew-formula` | 人工创建版本tag/Release触发push refs/tags/v<本卡版本>，解引用为已核merge SHA | 更新后formula验证job实际success |
| `release.yml` | `request-stable-manifest` | 人工创建版本tag/Release触发push refs/tags/v<本卡版本>，解引用为已核merge SHA | 四平台完整签名manifest发布success，stable/CDN实际version/digest对应本卡版本 |

## 任务卡

### Task FIX-LTR-01: 同步全局schema doctor的完整auto-upgrade结构守卫

**Task type:** `implementation`

**Lifecycle / Acceptance:** `done` / `complete`（实际f33be958/v0.30.87与完整D见本轮已核后置事实；旧A/C/CI待快照保留）

**Description:** 只更新现有完整结构字符串，使它同时保留schema_doctor、Upgrade和当前Ui排除，恢复确定性兼容测试；不改CLI行为。

**Out of scope:** fixture时序由FIX-LTR-02承接，MCP teardown由FIX-LTR-03承接；UI产品行为测试/解释由上游UI所有者承接，若需要改变生产逻辑则本卡停止并重新ER-03。

**Current evidence:** `tests/compat/global_config_schema_future_test.rs:727`在三次原生尝试均失败。该文件在2755eb7和0ca47cc的SHA均为e16a5fa6e3307b2dfe6eeb0827b3337faae4a4de18903363112a10829585e2ab；`src/cli.rs:3636`新增Ui排除，旧结构字符串未同步。前面的真实scope/role/path/canary断言已到达并通过。

**Acceptance criteria:**

- [x] 完整字符串包含原schema_doctor和Upgrade排除，并加入当前Ui排除；不得删除结构守卫或改为松散子串。
- [x] 该用例全部原runtime角色/路径/损坏canary/legacy断言与相邻startup recovery守卫逐字保留。
- [x] 完整compat_global_config_schema_future目标通过，生产CLI与其它测试字节不变。

**Verification:**

- [x] `source .env.test && source .env.live-test && cargo nextest run --test compat_global_config_schema_future -E 'test(=config_doctor_uses_global_role)' --retries 0`；必须精确1例、0失败、不可skip。
- [x] `source .env.test && source .env.live-test && cargo nextest run --test compat_global_config_schema_future --no-fail-fast --retries 0`；与运行前实际完整listing逐名一致。
- [x] 只读diff守卫：唯一测试语义改动为既有完整字串增加`|Commands::Ui(_)`；原runtime断言/CLI源码均不变。

**Full-suite trigger:** T-5：本卡修复新来源全量的确定性阻断；A与最终C均明确要求完整默认套件，不以单例PASS替代。

**Dependencies:** DEP-QUAL-SOURCE, DEP-QUAL-SECURITY, DEP-QUAL-PUBLISH；仅消费具名发布后来源与当前卡实际准入，前驱未complete不开始本卡发布。

**Deliverables:** N/A；测试修复及本计划同卡记录完整交付。

**Implementation write set:** `tests/compat/global_config_schema_future_test.rs`；本计划文件；`docs/development/plan/plan-status.md`仅本计划行（以实际发布上游文件为基线）。原scoped-traces三治理文件的未发布增量仅由协调者另行本地同步，不进入本卡发布分支。

**Release write set:** Inherited；以当前compat_version_surface_sync为权威的Cargo.toml/install.sh/install.ps1等实际版本面、工具链Cargo.lock及artifact；开工重核，不手改lock。

**Files likely touched:** 与I/R完全一致，不新增target或公共helper注册。

**Docs and compatibility impact:** 本计划及plan-status本计划行同卡更新真实状态；用户CLI/参数/输出/schema没有变化，双语命令指南、COMPATIBILITY、错误码与网站N/A。tests/INDEX既有目标名称/用途不变；若实际发生变化先扩写I并ER-03，不能漏文档。

**Rollback mode:** `immutable-release`

**Migration and rollback:** 无schema/数据迁移；未发布源码保留精确差异，已发布问题只用新签名提交/新patch前滚，不回退ref或版本tag。

**Security and privacy:** 只使用合成fixture与具名测试资源；不读取真实provider正文，不在证据中保存env值/token。进程清理必须证明归属，云资源按fixture唯一namespace，不清扫既有资源。

**Performance budget:** A一次7200秒；C一次10800秒；单测timeout/retry不变。一次claim，非预期失败STOP；具体执行器与输入manifest在执行前实审。

**Estimated scope:** `S`

**Version increment:** `patch`

**Release boundary:** `independent`；本卡自己的完整C/D，不免bump、不合并其它修复。

**C/D coverage from:** `self`

**Granularity:** `type=implementation; axis=global-schema doctor的auto-upgrade结构oracle对齐; recovery=immutable-release前滚本卡单轴测试修复; complete=yes; self-contained=yes; AC=3/8; VER=3/8; landing=1; prod-files=0; scope=S; deps=DEP-QUAL-SOURCE, DEP-QUAL-SECURITY, DEP-QUAL-PUBLISH; writeset=五张实现卡本计划与R串行; release=independent; split-from=新上游资格20261025实际FAIL/STOP; exception=N/A`

### Task FIX-LTR-04: 对齐同步 ingress 测试构造器的编译配置

**Task type:** `implementation`

**Lifecycle / Acceptance:** `blocked` / `locally-accepted`（旧A1/源审限域保留；C1审查FAIL/STOP，四版本面精确退出已执行/待终态增量审查，F1/F2及新恢复仍待，未发布）

**历史退出前Lifecycle字段：** **Lifecycle / Acceptance:** `blocked` / `locally-accepted`（仅原A1/源审限域保留；C1 native审查FAIL/root STOP，具体退出待审/待执行，未发布）

**历史Lifecycle / Acceptance（C1前）：** **Lifecycle / Acceptance:** `in-progress` / `locally-accepted`（A1/源B、root资源后检及自身本地Claude R1三PASS已核；两原生flake按具名P2限域承接，最终C/D未完成）

**Description:** `CaptureIngressCommand::from_payload` 与 `from_payload_with_hint_policy` 只供现有 debug/test lowerer 和测试使用，两个定义却留在默认 release。仅在两定义前添加与消费者相同的 `#[cfg(any(test, debug_assertions))]`，消除真实发布构建的项目 dead_code 诊断。

**Out of scope:** 不改两个方法体、可见性、生产 `from_stdin`/parse/finish、CLI、策略、测试断言或依赖；不修 FIX02 fixture/FIX03 MCP，不处理用户明确排除的第三方 proc-macro-error2 未来警告；不扩大为 release-profile integration 测试支持。

**Current evidence:** FIX02 C1 的 release-build 实际输出一项项目 dead_code、涉及 ingress.rs:453/472 两方法。ROOT在警告已违背严格门时停止已核协调进程，原terminal于2026-10-25T11:33:57.665096Z记RuntimeError/STOP、5051.049/10800秒；native build 被中断，数值退出未记录，不记成功或编造0/101。此前9阶段exit0，完整10761/21原ignored/0重试/0flake，模块15、真实云1、fmt/strictClippy保留；安装/提交/推送/Release未执行。原件在仓外fix-ltr02-C1-20261025与failure目录。source审ce6fb162 SOURCE_AUDIT PASS证明唯一非test消费者在cfg(debug/test)而方法未cfg；RECOVERY_GUIDANCE FAIL的P1必须由本次全局I特许及A release gate关闭。独立源为published b73/.87，不含FIX02 helper5025候选或原scoped-traces未发布祖先。

**Acceptance criteria:**

- [x] 默认 release 构建不再产生项目 warning；排除项仅为用户具名第三方 future-incompatibility 提示。
- [x] release 的 keyring 配置不产生项目 warning，保持实际发布 feature 的可编译性。
- [x] 原 debug/test 调用者在其既有配置中仍可使用同步构造器；具体测试证据见Verification。
- [x] 源码行为差异严格只有两行 cfg；方法体、from_stdin 生产入口及所有断言不变。

**Verification:**

- [x] `source .env.test && source .env.live-test && cargo build --release`；实际exit0，逐条诊断核项目warning0。
- [x] `source .env.test && source .env.live-test && cargo build --release --features keyring`；实际exit0，逐条诊断核项目warning0。
- [x] `source .env.test && source .env.live-test && cargo nextest run --lib -E 'test(internal::ai::capture::)' --no-fail-fast --retries 0`；完整capture模块listing与实际身份一致，0 tests/early return不得充当所需行为。
- [x] `source .env.test && source .env.live-test && cargo nextest run --test compat_agent_architecture_guard --no-fail-fast --retries 0`；完整target非零、逐身份核对。
- [x] 相对b73的源diff恰为ingress.rs两行cfg；既有lowerer消费者cfg与候选条件相等/为其子集，helper reasoning_artifact.rs等于b73 blob；精确I/R守卫。

**Full-suite trigger:** `T-4`：同步构造器承接共享lowerer，超过6个integration target复用；A和最终bump后的C均保留双env完整默认10761/原ignored身份核验。唯一命令仍为 `source .env.test && source .env.live-test && cargo nextest run --all --no-fail-fast --retries 2`，不改timeout/retry/并发。VER为附加门，不替代全量。

**Dependencies:** `FIX-LTR-01`、`FIX-LTR-05`、`DEP-QUAL-F2-CLOSE`、`DEP-QUAL-SOURCE`、`DEP-QUAL-PUBLISH`；FIX05实际C/D与F2已审定关闭后才新准入；当前字段按2026-10-26跨机入口，旧字段见历史记录。

**Deliverables:** 本卡两行代码及本计划/plan-status本计划行的实际来源、审查和验收记录；原C1 STOP、旧P2接受和unknown数值exit保留。

**Implementation write set:** 仅本卡额外允许生产文件 `src/internal/ai/capture/ingress.rs` 的两个cfg属性；`docs/development/plan/plan-20261025-qualification.md`；`docs/development/plan/plan-status.md`仅本计划行（以published b73为底）。这是一项卡级精确特许，不扩大FIX01/02/03原测试写集。

**Release write set:** 权威版本面 `Cargo.toml(version)`、`install.sh`、`install.ps1`，及工具链刷新 `Cargo.lock`；禁止手改lock或加入其它依赖变化。

**Files likely touched:** 与I/R一致，最终精确7路径；FIX02 helper候选/原scoped-traces三治理不得入本卡。

**Docs and compatibility impact:** 两方法均pub(crate)，已有公开debug/test lowerer可用配置不变，无用户CLI/参数/输出/schema/协议改变；用户指南双语、COMPATIBILITY、error-codes、网站N/A。现有内部代码span注释不改，不新建仓库证据目录。

**Rollback mode:** `immutable-release`

**Migration and rollback:** 无数据迁移；未发布只撤精确自有cfg与本卡治理，保护保存像/stash。已发布只新签名patch前滚，不reset/force/rewrite/tag覆盖。原FIX02失败.88四面退出、三个WIP精确stash与从b73建本卡branch须先通过恢复ER03，不由本卡偷偷执行。FIX04实际D全绿后才三方重建后继治理并恢复helper5025，不能直接pop旧文档/版本面。

**Security and privacy:** 原始provider frame边界不变，生产入口仍为from_stdin；不加allow、不打印body/token、不复制env到提交。只清理精确own资源，旧FIX03 worker尚未修，按既有审定形状/有界pidfd收尾与未知STOP规则。

**Performance budget:** A1候选一次10800秒、命令10620秒，含fresh编译/listing、fmt/strictClippy、VER focused、完整默认、default/keyring两release profile及资源后检；C1候选一次10800秒同样含实际bump/工具链lock与最终完整门/default+keyring两release构建/私有安装。编译暖缓存来源在claim前实核，禁止claim外预编译；若实际预算无法成立先ER03，不自动续窗。两窗口必须各自具体执行器真实ClaudePASS后唯一claim，零自动重跑；不借FIX02 C1余量，不复位任何旧FIX3/LTR00窗口。

**A1/B实际结果（2026-10-25 14:16:54 UTC，自身ER05待）：** 唯一A1原件`/tmp/fix-ltr04-A1-20261025`，具体执行审查session c8ad29bf-1b9a-46ed-8f8c-d4a5c3827560四PASS后才claim。十阶段native exit0：nightlyfmt、all-target/all-feature strict Clippy、完整capture 436、architecture 65、default/keyring两release构建、完整默认10761/21原ignored及其实际listing；终态执行耗时5795.491/10800秒，资源后检及root核验在同一原窗口内。终态SHA256 `2e5cfb669bba2dfe1ebe0a04c61d63d30a9f6af9c8cd6f6248a288689e6439e0`、full JUnit `0480f5861306f2682d9014561b0072d908180e73dfe7dabbb825701ccc222db7`、root-A `74b187277a94a396df429417d0e35edec2510d4fb9135b50c154a919112c790e`。source B原件`/tmp/fix-ltr04-source-B-20261025`：相对published b73，ingress source `2b7aa33698c2072d160cc6bea7801ffb8fb180a13bb0ebe78901ff888b7bb778`仅增加两cfg，方法体/断言/from_stdin不变、所有调用者条件为同条件或子集、FIX02 helper等于published基线。源B初次误猜out-of-line模块owner的检查失败已保留，纠正为实际live_pipeline.rs后核验通过，没有产品重跑。

**历史A1限定与具名风险（2026-10-25 14:16:54 UTC，owner=root；当时须自身Claude裁定，后续真实裁定见下段）：** 原定`--retries 2`产生2个首试FAIL、次试PASS：`libra::command::agent::checkpoint::tests::fix_rg_scoped_01_fresh_error_mapping`；`libra::command_test::command::stash_test::test_stash_no_change_untracked_only_and_quiet_guards`；保留原始输出和JUnit，不记零失败尝试。fresh-error-mapping只观察到CatalogUnavailable，该错误合并多个连接/查询/关闭/期限原因；stash quiet只观察到SQLite code5及storage-config fallback的runtime Warning，具体锁持有者及原因未证。两组相关源码均与published b73和A1冻结输入字节相同，这不等于历史复现或无回归证明，不以机器负载解释根因。分别登记RISK-QUAL-FRESH-MAPPING、RISK-QUAL-STASH-QUIET，root在后续资格维护按ER03评估；自身审查若判P0/P1必须先解决，不预设可接受。runtime输出按本次日志身份/文字与当前未变源码逐项归属：{"optional Mega LFS endpoint absent": 1, "case-insensitive filesystem prerequisite": 2, "test-live-cloud feature disabled in default suite": 9, "Bun interpreter runtime coverage unavailable": 1, "asserted missing-interpreter negative scenario, not skipped test": 1, "explicit GitHub live fixture prerequisites absent": 10, "explicit Mega2 live server/write-root prerequisites absent": 12, "current build rejects staging non-UTF8 filename; status read path unproved": 1, "PowerShell interpreter runtime coverage unavailable": 1}；不能把默认suite PASS等同未启用live/platform分支覆盖。capture与architecture无runtime opt-out或flake。资源关闭只处理本次精确观测的known worker，实际owned/new-orphan/target均0；FIX03清理源码尚未修，不倒推旧历史进程归属。仅用户具名proc-macro-error2 2.0.1 future warning排除，项目编译warning0；上述runtime Warning作为失败原件保留，不称已修。

**自身ER05实际结果与限域采纳：** 本地Claude session `e76e3d02-d544-4259-9f2b-42a937cc49e9`于2026-10-25T14:22:16.368832Z真实exit0、ER05/H0/VERDICT三PASS；manifest `cafd99c544a1d718d68b50ed170a7eac0899ad2eef217506efa84a9337757902`、output `ebd11737a87762761b0da72f71684b963fbfc86242348e73c2b001468a1b73b5`。root于14:24:41.918607Z实算全部73输入后采纳，P0/P1=0；原件`/tmp/fix-ltr04-own-review-R1-20261025/`及`/tmp/fix-ltr04-A1-root-audit-20261025/root-own-review-adoption.json`。这是审查后的实际收据；不能声称Claude事先读过本段，本两治理差异须由最终C的具体执行审查绑定并由最终被测树覆盖。两cfg源码和原A1输出不变，不借本记录重跑A或取得C资格。

**五项P2处置（owner=root）：** （1）`RISK-QUAL-STASH-QUIET`：本卡C原样保留retries2并保存实际FLAKY/FAIL，最终失败STOP；资格计划完成前另按受审ER03为SQLite BUSY导致storage-config退回本地登记FIX/DEFER并取得真实baseline证据，当前锁持有者/原因未证。（2）`RISK-QUAL-FRESH-MAPPING`：同样保留native尝试和STOP规则，资格完成前为CatalogUnavailable合并原因及InternalInvariant映射登记FIX/DEFER和真实baseline，不沿用FIX02旧subagent flake结论。（3）最终C必须实际default/keyring双release、项目warning0；永久release-profile warning防回归守卫作为root后续维护，未在本卡实现。（4）本地lock/源码冻结，实际版本D四平台必须覆盖--locked/target/strip及全部渠道artifact，不能以本地两个build替代。（5）最终C包含这两份治理最新字节，C后不改tracked文件，后置事实由FIX02既有治理承载。仅用户具名proc-macro-error2 v2.0.1 future提示排除。审查所称cfg不改变debug/test语义不等于baseline复现，也不证明历史DB根因；自身审查未读的focused JUnit/listing/执行器/进程原件已由root核验，具体C审查须补充原件输入，不冒称自身审查已读。

**发布状态：** 原C1不接受且STOP；实际四版本面已退出到b73/.87，保留两cfg及7路径失败像，无active bump。退出终态增量审查待；本卡仍blocked，未授新A/C，未commit/push/PR/tag/Release。

**历史退出前发布字段：** **发布状态：** C1已执行全部11阶段及私有安装，但native新失败审查FAIL，root最终不接受C并STOP；候选.88未commit/push/PR/tag/Release，不能发布。精确退出四版本面的方案待本地Claude及root采纳；退出后仍blocked，未授任何新A/C。

**历史发布状态（C1前）：** **发布状态：** A1与源B限域验收、自身ER05/H0已取得；独立C1具体执行器审查、最终bump后完整门、私有安装、精确signed/DCO、PR/merge/tag/人工Release和全部适用D均未完成。0.30.87未bump，FIX02旧C1 STOP及stash988ded20保留；正式资格1/4、原scoped-traces0/26/H1不变。原scoped-traces三治理不进入本分支，本卡C/D后由FIX02两治理承接，不在C后修改已测提交树。

**Estimated scope:** `S`

**Version increment:** `patch`（当前base0.30.87，.30.88仅未占用时的候选；actual claim前核remote/tag/Release，变化则ER09/ER03）

**Release boundary:** `independent`；本卡自己的A/B、实际ClaudeER05、版本bump、完整C、精确signed/DCO、feature PR全部base/CodeQL、gh squash且merge tree=被测tree、人工版本Release/tag、全部渠道D。无G08/G13合并。

**C/D coverage from:** `self`；本卡后置真实C/D由FIX02重新开工时在已登记两治理中承接并由其自身最终C覆盖，不改终止链末端FIX03的全量资格/终态登记义务。

**Granularity:** `type=implementation; axis=同步ingress测试构造器的profile可用性; recovery=immutable-release; complete=yes; self-contained=yes; AC=4/8; VER=5/8; landing=1; prod-files=1; scope=S; deps=FIX-LTR-01,FIX-LTR-05,DEP-QUAL-F2-CLOSE,DEP-QUAL-SOURCE,DEP-QUAL-PUBLISH; writeset=两cfg及既定两治理+版本面; release=independent; split-from=FIX02-C1实际release警告STOP; exception=N/A`

**RB-FIX04-EXIT-C1（候选）：** FIX04原C1于2026-10-25 18:10:38 UTC结束，11阶段exit0、私有安装.88及资源后检exit0，5348.666525/10800秒；外层工具96753实际exit0。完整10761身份/21原ignored保留，4个flake/5个失败尝试后native重试成功；实际本地Claude session c7f9fbae-734f-416c-9d9d-3650ede3f824的四项判定全部FAIL，root于18:17:28 UTC记录STOP，C不接受。F1 P1为storage配置读取错误后无条件LocalStorage回退并污染machine stderr；F2暂定P1为reset missing-path失败未保留实际stderr，根因未知；F3 stdin fixture race与F4未执行accept脚本列P2。所有旧FAIL/STOP、A1两flake、37运行时覆盖缺口及21 ignored保持，历史SQLite锁owner/SQL与C2数据库根因未证。本次仅候选RB-FIX04-EXIT-C1：先保存七路径失败树及SHA清单，再用原生Libra精确恢复本卡四个未发布版本/lock到原before-bump（逐字等于b73/.87），保持两cfg和其它全部源码、原stash988ded20、两仓原WIP。经本机Claude EXIT/ER12/H0/VERDICT PASS和root采纳才执行；不reset/force/stash-pop/commit/push，不新开A/C，不授.88再次发布。退出成功仅释放本次失败C1发布占用，FIX04仍blocked/locally-accepted，资格1/4、原scoped-traces0/26不变。后续F1/F2/F3须先审定合法独立修复与依赖；不得把无G13的本仓改成聚合，也不借G08归并独立缺陷。Mega102自己的ER05已PASS，独立C2仍需退出终态与自身具体执行合同PASS。

**退出执行守卫：** 唯一root；旧C outer/driver均终态、owned/new-orphan/target=0且无新Cargo/Claude；HEAD/origin/main/远端main均b73，当前分支fix/capture-ingress-profile-20261025，index为空且精确7脏路径。当前全tracked blob/mode、七路径前像、两版本前像/安装器及published对象逐字绑定；已发布main若变先STOP/ER03。无v0.30.88 tag/Release的实际HTTP404必须再核，禁止把权限错误当不存在。先创建一次退出claim，再仅执行libra restore --source b73e98a77ecc7c3db1ea905ab55478785d76c041 --worktree Cargo.toml Cargo.lock install.sh install.ps1；Cargo.lock是恢复原工具链生成前像，不手工修改依赖。随后写入受审两治理候选，核其余全部tracked文件、原两仓9+4路径及stash不变，index空、版本.87；任一步失败保留实际部分状态和STOP，无自动重入。原private .88二进制仅隔离证据，不加入PATH/不作为发布或后续测试工具。实际退出收据在同一两治理候选上追加真实UTC，并另经增量本地Claude核对后用于下个发布准入；不得宣称本次审查已读未来收据。

**实际退出证据（2026-10-25 18:59:44 UTC）：** FIX04 C1的精确退出已实际执行：本地Claude退出R1 session ff79d09e-3485-4fdb-9316-258cd5d0b614四项PASS、50输入重核；root采纳后，原生restore及外层工具18851于2026-10-25 18:57:47 UTC实际exit0。四个版本/lock逐字等于published b73/.87，四个0664模式保持，原生stash list确认988ded20及完整列表不变；七路径失败树已在持久私有位置按SHA256/MANIFEST及模式补充保存。两cfg及其余全部tracked blob/mode、原Mega9/publisher9/原Libra4路径WIP保持，index空，仅余本计划/本计划status行/ingress三个脏路径。这是退出执行事实，当前等待退出终态增量单审；不授任何新A/C、commit/push/tag/Release。FIX04原C1机械11阶段0但native审查四FAIL、root STOP保持，Lifecycle仍blocked/locally-accepted，仅旧A1/源审限域有效；storage错误回退P1、reset实际stderr不足暂定P1与其它具名风险尚未解决，资格1/4、原scoped-traces0/26不变。失败C1已无active bump；后续独立Mega102须在本次终态增量单审及自己的具体C2合同通过后才claim。 root实际核50输入/原生全stash列表/四mode/9+9+4保护；不声称曾存在Git refs/stash：首次只读probe实际exit129已保留，改用本仓原生JSON stash list。REST单tag/Release404之外，已登录gh完整分页release list也无v0.30.88（含draft）。恢复命令可写Libra内部租约等metadata；“六路径”只指tracked写集，不把metadata写成零变更。

### Task FIX-LTR-02: 让合成reasoning fixture在索引结清后进入erase

**Task type:** `implementation`

**Lifecycle / Acceptance:** `blocked` / `locally-accepted`（保存helper5025候选A2/ownR2；C1项目release警告STOP，不视完成；待FIX04实际C/D后重新ER03 source/version/new C）

**Description:** 在 `seed_checkpoint` 中既有writer marker退休成功后、`conn.close()`之前建立有界、可失败的索引结清前置（在调用方直接INSERT索引及CLI erase之前），保留原删除保护和所有reasoning隐私/生命周期断言；不是修改生产erase为等待或重试。

**Out of scope:** 不改HistoryManager/ClientStorage生产API，不删除/重放marker以造绿，不归因历史C2或机器负载。结构oracle由FIX-LTR-01、MCP teardown由FIX-LTR-03承接。

**Current evidence:** 现有QUAL-LTR-FLAKY在新旧来源各出现一次；本轮metadata OID689bb4bc1a6da590532651d8db132eef0b0037b3仍有durable marker，删除栅栏正确返回WouldBlock。`seed_checkpoint`使用capture_scope=None，未等待后台索引；同仓`agent_checkpoint_export_test.rs:741-747`已有有界等待先例。失败当时为何未drain尚未证明。

**Acceptance criteria:**

- [ ] 使用现有有界后台任务等待，界限不超过已有同类fixture的10秒；超界明确失败，不sleep固定延迟或重试erase。
- [ ] 等待返回后只读枚举本fixture `.libra/object-index-repair/`，目录不存在或为空才通过，存在任何marker或枚举错误均失败；不扩大私有常量/失败计数生产API。消费者失败留下marker时必须失败，安全诊断仅OID/类型/文件名，不打印body/secret。
- [ ] 原erase及GC、reasoning私密refs/export和坏digest断言保持，生产删除栅栏与队列实现不改。
- [ ] 两个直接消费者均验收：完整agent_checkpoint_export_test模块及真实云reasoning生命周期用例；真实云缺凭据不可skip通过。

**Verification:**

- [ ] `source .env.test && source .env.live-test && cargo nextest run --test command_test -E 'test(command::agent_checkpoint_export_test::)' --no-fail-fast --retries 0`；按实际完整模块listing非零逐名核对。
- [ ] `source .env.test && source .env.live-test && cargo nextest run --features test-live-cloud --test agent_cloud_tombstone_test -E 'test(=reasoning_artifact_mirrors_and_erases_with_checkpoint)' --retries 0`；精确1例真实D1/R2，保留fixture唯一repo UUID及cleanup证据，禁止early-return冒PASS。
- [ ] 检查失败路径仍fail-closed：保留marker不擦除、不放宽erase断言；只读核该helper两个直接消费者及无新增生产API。新增确定性回归如必需，须在同一行为轴/I中先登记selector并审查，不以重复跑同一flake代替因果。

**Full-suite trigger:** T-4/T-5：修改tests/helpers共享测试基建，A及最终C完整默认套件；真实云selector另跑，不被默认feature的PASS替代。

**Dependencies:** FIX-LTR-01、FIX-LTR-04；仅消费具名发布后来源与当前卡实际准入，前驱未complete不开始本卡发布。

**Deliverables:** N/A；测试修复及本计划同卡记录完整交付。

**Implementation write set:** `tests/helpers/reasoning_artifact.rs`；本计划文件；`docs/development/plan/plan-status.md`仅本计划行（以实际发布上游文件为基线）。原scoped-traces三治理文件的未发布增量仅由协调者另行本地同步，不进入本卡发布分支。

**Release write set:** Inherited；以当前compat_version_surface_sync为权威的Cargo.toml/install.sh/install.ps1等实际版本面、工具链Cargo.lock及artifact；开工重核，不手改lock。

**Files likely touched:** 与I/R完全一致，不新增target或公共helper注册。

**Docs and compatibility impact:** 本计划及plan-status本计划行同卡更新真实状态；用户CLI/参数/输出/schema没有变化，双语命令指南、COMPATIBILITY、错误码与网站N/A。tests/INDEX既有目标名称/用途不变；若实际发生变化先扩写I并ER-03，不能漏文档。

**Rollback mode:** `immutable-release`

**Migration and rollback:** 无schema/数据迁移；未发布源码保留精确差异，已发布问题只用新签名提交/新patch前滚，不回退ref或版本tag。

**Security and privacy:** 只使用合成fixture与具名测试资源；不读取真实provider正文，不在证据中保存env值/token。进程清理必须证明归属，云资源按fixture唯一namespace，不清扫既有资源。

**Performance budget:** 原A1 FAIL/STOP、恢复A2实际2483.782秒与C1实际5051.049秒STOP全部保留；原C1剩余不再可用，本候选不授A3/C2。只有FIX04实际C/D与原生受控整合后，新的ER03按来源/版本/行为轴审定有限carry与新具名C预算；原10秒fixture、nextest原retries2/timeout不改。

**已保存候选验收 / root（本轮事实登记）：** helper SHA5025cbd4004e47fd508f4b28efe1bd6788444ff923d638b0b9fc008aec48f5b7仍在七文件保存像；A1于2427.745秒FAIL/STOP保留。A2由实际执行R2审查2130b342准入，于08:53:46.754733Z唯一claim、09:35:10.536517Z终态，2483.782/10800秒；fmt/strictClippy、模块15、真实云1、完整10761/21原ignored实际通过。A2有1个原生retry-success：bounded_discovery_outer_cancellation_kills_pipe_holding_descendant，原因未证；38标记中37既有覆盖缺口、1缺interpreter负控，不冒称全live。自己的本地Claude R2 b5c96cc9于09:48:52Z ER05/H0/VERDICT PASS，raw output e8485ca7f464c9fd4e9dfbacc45c17b824fd68fb54dfc2e3c0c144f331c2c255。它们只保留原b73候选来源，FIX04之后不能自动当新树PASS。C1于10:09:46.616474Z唯一claim，11:33:57.665096Z STOP，5051.049/10800秒；完整10761无失败/重试/flake、21原ignored，模块15/真实云1和fmt/clippy通过；release出现一项dead_code涉及两个同步构造器。root据严格项目warning门停止正在构建的原协调器，terminal只记RuntimeError/coordinator interrupted15；native build数值exit未记录、不伪填0/101/不称已完成，安装未执行。terminal SHAe389574db8bccda8e8467763f4017612ba0c3896996ab4c5b03026e579973311。root在原窗口5350.267秒内核七失败树文件、精确pidfd清理4031071并确认gone/reaped，owned/new-orphan/target0，实际云16D1表零/R2空。未提交/推送/Release；旧余量不复用，C2未授权。

**Estimated scope:** `S`

**Version increment:** `patch`

**Release boundary:** `independent`；本卡自己的完整C/D，不免bump、不合并其它修复。

**C/D coverage from:** `self`

**Granularity:** `type=implementation; axis=合成checkpoint fixture的索引结清前置; recovery=immutable-release前滚本卡单轴测试修复; complete=yes; self-contained=yes; AC=4/8; VER=3/8; landing=1; prod-files=0; scope=S; deps=FIX-LTR-01,FIX-LTR-04; writeset=五张实现卡本计划与R串行; release=independent; split-from=新上游资格20261025实际FAIL/STOP; exception=N/A`

### Task FIX-LTR-03: 回收MCP无helper断言用例拥有的暂停worker

**Task type:** `implementation`

**Lifecycle / Acceptance:** `pending` / 空（ER-03候选；无源码/测试/提交/发布）

**Description:** 仅修当前MCP测试的owned worker/父进程清理，成功、断言panic及可归属的失败路径均回收；不改变产品worker生命周期。

**Out of scope:** 不加生产PDEATHSIG、不改worker deadline/内存限制/子进程禁止语义，不扩大到其它MCP功能；不能声称重建本轮未保存的PID文件。其余两个独立缺陷由FIX-LTR-01/02承接。stdout-holder fixture的KillHold早期失败缺口尚未排期，具名RISK-QUAL-HOLDER；若本轮资源审计命中则STOP并ER-03登记修复，不吞掉该失败。

**Current evidence:** 本轮full退出后遗留原session3874874的worker PID4064241，PPid1、独立PGID、deleted fixture cwd；root经pidfd核身份后SIGTERM，进程实际消失。精确原case未采集，不能以时间重合证明。静态`read_worker_status_spawns_no_helpers`在703-704行仅kill/wait父进程，而exec.rs:141让worker拥有独立进程组，pause hook永久等待，构成明确清理缺口。原遗留进程也不能排除stdout-holder fixture；其KillHold在191-204行前的失败路径无guard、kill -9不核身份是另一处已知静态风险，不声称本卡能证明历史PID来自哪个case。

**Acceptance criteria:**

- [ ] 保留descendant_pids(worker).is_empty原断言；取得worker PID后先建立具有身份核验的cleanup guard，再执行可能panic的断言。
- [ ] 成功/panic以及PID取得前后的失败路径均处理当前测试拥有的父进程与可证明归属的worker；不信任复用PID、不杀其它用例或系统进程。
- [ ] 保留pause文件发现上限10秒，cleanup退出观察上限1秒；先向核身份的owned worker/其独立组发退出信号，再kill+wait MCP父进程，避免先杀父丢失归属；不把父进程wait成功当worker清理证明；Linux核 `/proc/<pid>/stat` 的starttime/state，区分已退出僵尸与活进程并另记是否已回收，不以kill(pid,0)独证；其它平台用实际进程身份/退出能力，不能照搬procfs或跳过原测试；现有descendant_pids在非Linux上恒为空，不能当归属证据，若当前I/现有平台API不足以建立安全身份，必须在claim前ER-03。PID文件取得前先枚举MCP父进程的真实子关系并核身份（Linux覆盖所有 `/proc/<mcp>/task/*/children`），再执行清理；不能先杀父进程再猜孤儿归属。不提高测试超时。
- [ ] 完整agent_mcp_exec_test通过，原deadline、内存、slot、禁止helper断言全部保持；本卡bump后C全量/doc/network通过，实际本卡Claude ER-05通过；五张实现卡D之后的计划最终资格属于计划完成门，不倒挂本卡done条件。

**Verification:**

- [ ] `source .env.test && source .env.live-test && cargo nextest run --test agent_mcp_exec_test --no-fail-fast --retries 0`；完整目标按listing逐名核验。
- [ ] `source .env.test && source .env.live-test && cargo nextest run --test agent_mcp_exec_test -E 'test(=read_worker_status_spawns_no_helpers)' --retries 0`；精确1例。成功路径核owned PID/start identity/PGID与exit；失败路径逐分支静态审查spawn/write/try_wait/读PID/parse/panic及Drop顺序，不宣称已做错误注入运行；不运行无界循环或额外重试。若当前I不足以确定安全归属，先ER-03停止，不能退化为按进程名批量kill。
- [ ] `source .env.test && source .env.live-test && cargo test --doc`；原doc目录/计数/ignored与实际退出核实。
- [ ] `source .env.test && source .env.live-test && cargo nextest run --features test-network --test network_remotes_test --no-fail-fast --retries 0`；精确原3例，无skip/过滤。其oracle分别是refs、有限PACK标头、LFS batch JSON，不冒称完整对象下载。

**Full-suite trigger:** T-5：本计划最后一个独立修复，A及最终C完整默认套件并完成doc/network；不是把前三卡聚合为一次发布。

**Dependencies:** FIX-LTR-02；仅消费具名发布后来源与当前卡实际准入，前驱未complete不开始本卡发布。

**Deliverables:** N/A；测试修复及本计划同卡记录完整交付。

**Implementation write set:** `tests/agent_mcp_exec_test.rs`；本计划文件；`docs/development/plan/plan-status.md`仅本计划行（以实际发布上游文件为基线）。原scoped-traces三治理文件的未发布增量仅由协调者另行本地同步，不进入本卡发布分支。

**Release write set:** Inherited；以当前compat_version_surface_sync为权威的Cargo.toml/install.sh/install.ps1等实际版本面、工具链Cargo.lock及artifact；开工重核，不手改lock。

**Files likely touched:** 与I/R完全一致，不新增target或公共helper注册。

**Docs and compatibility impact:** 本计划及plan-status本计划行同卡更新真实状态；用户CLI/参数/输出/schema没有变化，双语命令指南、COMPATIBILITY、错误码与网站N/A。tests/INDEX既有目标名称/用途不变；若实际发生变化先扩写I并ER-03，不能漏文档。

**Rollback mode:** `immutable-release`

**Migration and rollback:** 无schema/数据迁移；未发布源码保留精确差异，已发布问题只用新签名提交/新patch前滚，不回退ref或版本tag。

**Security and privacy:** 只使用合成fixture与具名测试资源；不读取真实provider正文，不在证据中保存env值/token。进程清理必须证明归属，云资源按fixture唯一namespace，不清扫既有资源。

**Performance budget:** A一次10800秒（含最终全量/doc/network）；C一次10800秒（含bump后完整全量/doc/network）；不提高原测试timeout或retries。一次claim，非预期失败STOP；具体执行器与输入manifest在执行前实审。

**Estimated scope:** `S`

**Version increment:** `patch`

**Release boundary:** `independent`；本卡自己的完整C/D，不免bump、不合并其它修复。

**C/D coverage from:** `self`

**Granularity:** `type=implementation; axis=MCP read-worker测试夹具完整teardown; recovery=immutable-release前滚本卡单轴测试修复; complete=yes; self-contained=yes; AC=4/8; VER=4/8; landing=1; prod-files=0; scope=S; deps=FIX-LTR-02; writeset=五张实现卡本计划与R串行; release=independent; split-from=新上游资格20261025实际FAIL/STOP; exception=N/A`

## 测试入口、追溯与里程碑（本轮恢复）

| 任务/里程碑 | 当前来源与入口 | 覆盖与失败边界 |
|---|---|---|
| FIX05 | F1源缺陷、R1指出缓存/写入顺序/配置busy设计缺口；先VER-6与ATTRIB、用户决定和本地ER03 | 粒度/方案未准入；独立default/keyring release及完整A/C/D，不豁免未知F2关闭 |
| FIX04 | FIX02 C1 release警告→源审ce6fb162→ingress两cfg；capture完整lib模块、compat_agent_architecture_guard、default/keyring release及完整默认 | A/C各具名新窗口、原timeout/retries不改；warning仍有即STOP |
| FIX02恢复 | 已保存A2/ownR2和C1 STOP→FIX04实际C/D→新ER03 source/version/C | 不借C1余量，不盲carry b73资格；helper/两消费者保持原断言 |
| 修复里程碑 | FIX01(done)→FIX05→FIX04→FIX02→FIX03分别独立完整C/D | 发布串行，不能合并、不能以手工worker清理当FIX03修复 |
| 最终资格/HANDOFF | FIX03真实merge树10800秒fmt/clippy/full/doc/network，再终态文档及PR/main D与例行补记 | 未全部满足不解除原LTR10 incoming；H1/旧merge签名政策不豁免 |

## 完成判据与交接

五张实现卡分别实际A/B、Claude ER-05 PASS、精确signed/DCO提交、独立patch/PR/merge/tag/人工Release、实际CI与发布artifact D完成；全部P0/P1关闭，P2具名处置。FIX-LTR-03 的C④在bump后树执行完整默认套件、`cargo test --doc`及原3例network；merge树与被测树不等即STOP。五张实现卡各自实际D完成后，在与FIX03发布merge树相同的冻结来源执行上述一次10800秒计划最终资格窗口（fmt、strict Clippy、双env完整默认套件、doc、原3例network），核全部身份/退出/ignored/flake/runtime skip与owned资源；根因未知的缺口不得改成已证。最终实际本地Claude来源资格PASS后，才把五个具体发布SHA/版本及适用覆盖交给原scoped-traces计划；其H1、原merge签名政策、LTR自身A/B/C/D仍独立。

当前（2026-10-26）两原计划正式23/118、0/26、23/144；本外部五实现卡1/5，DOC01/02另记，不增加原26卡。联合依赖按文首ATTRIB→FIX05、FIX05/新FIX→CLOSE→FIX04→FIX02→FIX03与H0/H1/LIVE/LTR10核无环；任一卡/末端证据不足均未完成。旧17/118、1/4统计保留于下方2026-10-25快照。

> 历史初稿统计：原Mega14/112、Libra0/26、联合14/138；外部0/3，保留其原时点。

## 修订记录

| 日期 | 触发 | 变更 | 实际状态 |
|---|---|---|---|
| 2026-10-26 10:50:00 UTC | 05V 准入 session 95af8652 与实现审查 72196c7f | 冻结 fetch AutoFollow 批量探测及 pull/clone 的 IoReadFailed 映射；focused 测试已通过；不占用 v0.30.88；FIX-LTR-05 仍 blocked | in-progress/空；实现审查 PASS |
| 2026-10-26 10:30:00 UTC | 05U 准入 session c1d476b0 与实现审查 bc600c5b | 冻结 remote.rs 与 tiered.rs 的 search_checked；离线策略不访问远端；focused 测试已通过；不占用 v0.30.88；FIX-LTR-05 仍 blocked | in-progress/空；实现审查 PASS |
| 2026-10-26 10:05:00 UTC | 05T 准入 session 986c352a；实现审查 de5171c1 FAIL 后 02aa713c PASS | 冻结 storage trait、local、client_storage、am；Windows 测试模块改为 cfg(all(test, unix))；不占用 v0.30.88；FIX-LTR-05 仍 blocked；tiered 搜索记为 DEFER-QUAL-05-TIERED-SEARCH | in-progress/空；实现审查 PASS |
| 2026-10-26 09:40:00 UTC | 05S 准入 session 71cf457d 与实现审查 d6849287 | 冻结 client_storage.rs 的索引补登记与修订后缀类型过滤；focused 测试已通过；不占用 v0.30.88；FIX-LTR-05 仍 blocked | in-progress/空；实现审查 PASS |
| 2026-10-26 09:20:00 UTC | 05R 准入 session 54f3e8cf 与实现审查 146781d3 | 冻结 snapshot.rs、store.rs、operation restore.rs、repack.rs；focused 测试已通过；不占用 v0.30.88；FIX-LTR-05 仍 blocked | in-progress/空；实现审查 PASS |
| 2026-10-26 08:10:00 UTC | 05Q 准入 session 3de3e11b 与实现审查 cf7454d4 | 冻结 show.rs、maintenance.rs、restore.rs、object_ext.rs；四个 focused 测试通过；不占用 v0.30.88；FIX-LTR-05 仍 blocked | in-progress/空；实现审查 PASS |
| 2026-10-26 06:40:00 UTC | 准入复审 f5697874 要求补齐计数、依赖边与 file.rs 验证 | FIX-LTR-05 写明拆出 05P 与 05Q；05P 写集含 file.rs，VER-5 覆盖 obliterate 错误传播；05Q 仍未冻结不准入；现行顺序见「实施顺序与发布窗口」 | blocked/空；源码未改 |
| 2026-10-26 06:20:00 UTC | ER03 session e52e4025 要求先拆卡 | 新增 FIX-LTR-05P，只含 client_storage 可失败类型探测及 fsck/cat-file/obliteration 的 exist_checked 迁移；66 文件全迁不进本卡；v0.30.88 仍只属于 FIX-LTR-05 | blocked/空；源码未改，待本卡 Claude 准入 |
| 2026-10-25 18:59:44 UTC | 退出R1实际四PASS与精确native退出exit0 | 四版本回到.87、0664/stash/源树及9+9+4保护已核，真实C失败/STOP保留；实际终态待增量单审 | blocked/locally-accepted；无active bump或新窗口，资格1/4 |
| 2026-10-25 18:47:47 UTC | FIX04 C1全命令0但真实native审查FAIL/root STOP | 新增仅四个未发布版本面的精确退出候选，保存7路径失败树；两P1与独立恢复待，不授新A/C或聚合 | blocked/locally-accepted；退出待Claude，资格1/4 |
| 2026-10-25T14:24:41.918607+00:00 | FIX04自身实际Claude R1终态及root采纳 | ER05/H0/VERDICT三PASS、73输入、五P2具名处置；新增事实只限既有两治理，原FAIL/STOP及A1 flakes不变 | locally-accepted；最终C/D未取得，资格1/4 |
| 2026-10-25 14:16:54 UTC | FIX04唯一A1/B实际终态及root资源后检 | 两cfg源B、十阶段及10761身份核验；保留2个原生flake与runtime缺口、旧STOP | locally-accepted，自身ER05/C/D待；资格1/4 |
| 2026-10-25T12:03:44.446299+00:00 | FIX02 C1实际release项目警告STOP；source审SOURCE PASS/RECOVERY FAIL两P1 | 新增独立FIX04、精确生产I特许、A/C release profile门、原C1 STOP/保存像及新顺序/终止链 | ER03/H0候选，未授权新A/C或正式源码 |
| 2026-10-24 19:47:13 UTC | 新上游资格full实际FAIL、既有flake复现与手工清理的worker | 三个独立测试修复、隔离发布与原计划incoming交接 | 本地Claude ER-03/H0待审；没有执行或发布 |

## 任务卡粒度审计表

| 卡 | 行为轴 | I（测试+同步文档） | AC/VER | landing/prod | 依赖 | release/C-D |
|---|---|---|---|---|---|---|
| FIX-LTR-01 | doctor完整结构oracle | global_config_schema_future_test.rs、本计划、plan-status本计划行 | 3/8、3/8 | 1/0 S | DEP-QUAL-SOURCE/SECURITY/PUBLISH | independent/self |
| FIX-LTR-05P | 存储读取失败不再被布尔存在性探测说成对象不存在 | client_storage.rs、fsck.rs、cat_file.rs、obliteration/mod.rs、file.rs 及本卡测试、本计划、plan-status 本计划行 | 6/8、5/8 | 4/5 M；file.rs 只消费 classify_presence 的 Result | FIX-LTR-01 + DEP-QUAL-SOURCE | family child；不占用 v0.30.88；实现审查 dc579753 PASS，尚未推送 |
| FIX-LTR-05Q | show、maintenance、restore 和 Blob::save 的读取失败不再当成对象不存在 | show.rs、maintenance.rs、restore.rs、object_ext.rs 及本卡测试、本计划、plan-status 本计划行 | 4/8、4/8 | 4/4 M | FIX-LTR-05P | family child；不占用 v0.30.88；已推送 c0373cb |
| FIX-LTR-05R | 操作日志快照、恢复和 repack 的读取失败不再当成对象不存在 | snapshot.rs、store.rs、operation restore.rs、repack.rs 及本卡测试、本计划、plan-status 本计划行 | 4/8、4/8 | 4/4 M | FIX-LTR-05Q | family child；不占用 v0.30.88；已推送 80ef760 |
| FIX-LTR-05S | 索引补登记和修订后缀不再把存储读取失败当成对象不存在 | client_storage.rs 及本卡测试、本计划、plan-status 本计划行 | 2/8、2/8 | 2/1 S | FIX-LTR-05R | family child；不占用 v0.30.88；已推送 6177646 |
| FIX-LTR-05T | am 的本地对象搜索不再把不可读目录当成缺失 | storage/mod.rs、local.rs、client_storage.rs、am.rs 及本卡测试、本计划、plan-status 本计划行 | 2/8、1/8 | 4/4 M | FIX-LTR-05S | family child；不占用 v0.30.88；已推送 e728736 |
| FIX-LTR-05U | tiered 与 remote 的前缀搜索不再把列举失败当成缺失 | remote.rs、tiered.rs 及本卡测试、本计划、plan-status 本计划行 | 2/8、1/8 | 2/2 S | FIX-LTR-05T | family child；不占用 v0.30.88；已推送 1d3c8d2 |
| FIX-LTR-05V | fetch AutoFollow 不再把存储读取失败当成缺失标签 | fetch.rs、pull.rs、clone.rs 及本卡测试、本计划、plan-status 本计划行 | 2/8、1/8 | 4/3 M | FIX-LTR-05U | family child；不占用 v0.30.88；session 95af8652 已准入 |
| FIX-LTR-05 | 配置失败与对象存储fail-closed（未准入候选） | client_storage/util/fetch候选及卡内逐文件同步集 | 8/8、6/8 | 3文件候选；落点/规模pending VER-6 | FIX01 + 05P 至 05V + SOURCE/PUBLISH + F2-ATTRIB + v0.30.88决定；05V不足以解除阻塞 | independent/self；不得先实施 |
| FIX-LTR-04 | 同步ingress测试构造器profile可用性 | ingress.rs仅两cfg、本计划、plan-status本计划行 | 4/8、5/8 | 1/1 S | FIX-LTR-05 + F2-CLOSE + SOURCE/PUBLISH | independent/self |
| FIX-LTR-02 | reasoning fixture索引结清 | reasoning_artifact.rs、本计划、plan-status本计划行 | 4/8、3/8 | 1/0 S | FIX-LTR-01/FIX-LTR-04 | independent/self |
| FIX-LTR-03 | MCP owned worker teardown | agent_mcp_exec_test.rs、本计划、plan-status本计划行 | 4/8、4/8 | 1/0 S | FIX-LTR-02 | independent/self |

## 风险登记

**现行profile警告 / root：** P2-QUAL-RELEASE-DEADCODE的旧接受仅解释v0.30.87历史，本轮用户严格项目warning指令取代该放行，由FIX04独立关闭；只排除proc-macro-error2第三方future提示。DOC-LTR01旧句“默认debug build如出现capture-ingress警告”保留为历史措辞，现已证明该警告只在默认release出现，DOC的debug build无该警告不是修复证据。新FIX04 A及C必须实际跑default/keyring release并对项目warning失败；不得用dev Clippy/测试PASS代替。


| ID | 事实/边界 | root处置/重启条件 | 状态 |
|---|---|---|---|
| RISK-QUAL-FUTURE | proc-macro-error2现有future-incompat提示；旧Clippy exit0不等于无诊断 | 原件保留；任何适用发布门拒绝则STOP独立归属，不allow压制 | open P2 |
| RISK-QUAL-FLAKE | marker为什么未drain尚未证；fixture边界缺等待有源码证据 | FIX02严格有界等待并独立核marker，真实云消费者；不能宣称证明历史C2 | open，FIX02验收待 |
| RISK-QUAL-HOLDER | 历史PID不能排除stdout-holder；KillHold早期失败缺口及裸kill -9已知 | root接受为FIX03当前范围外残余风险，尚未排期；若实际资源审计命中或当前路径失败，立即STOP并ER03新增修复，不以手工kill当自动PASS | open P2，非历史PID确定归因 |
| RISK-QUAL-CACHE | 独立路径可能使Cargo重新冷编译，暖缓存时间不保证 | 独占缓存、记录配置；安全复制不可用或需超候选预算则claim前ER03；运行超时STOP不自动续时 | open P2 |
| RISK-QUAL-MERGE | PR期间upstream/tree/版本可能变化；原unsigned merge未处理 | 比较被测tree与merge tree，不等STOP；原merge仅原工作区后续解决，不进入修复祖先 | open，硬门 |

## Claude review log

按用户指令单用本地Claude，替代模板reviewer名称，不降低实审或PASS要求。

| Round | Scope | Result | P0/P1 | P2处置 | Evidence |
|---|---|---|---|---|---|
| R1 | ER03/H0候选 | FAIL | P1-1顺序表漏门、P1-2状态登记缺失、P1-3发布树资格未绑定 | R2逐项修订；残余风险由root具名登记 | session04ce4421-a050-4bf3-a003-7662306ee05f；output SHA256 52d6588527ba5c8a16b96072e6857cf0aa60576ba7bcb28d7bcab270f962f96f |
| R2 | ER03/H0恢复候选 | FAIL | 原三P1已闭；新终态证据/被测树P1 | R3前滚修订，未执行产品窗口 | session e9f0e17d-07b4-40d8-9098-3f2a6e68c857；output 92b3745fc0f02790221e06d6db9f80ce56acab39d945378cfeeb23cab6e6c5ba；H0 PASS |
| R3 | ER03/H0终止链候选 | FAIL | 终态ER05收据无落点 | R4闭合，原FAIL不改 | session 82ca0f53-d8f6-4148-af66-13721b381b95；output 8f15fea4fd997f6157f3a82d323b286a9838b91fe8ef5e960a05be1baaa9876b；H0 PASS |
| R4 | ER03/H0最终恢复方案 | PASS | 无 | root具名逐项处置如下；最终事实字节待DOCS-ER05/H0实审 | session 67756716-0e46-4d90-bf70-fd23dff6b918；output 48f93079f925f4129fa9523b78a5cf2bae123e9aa4fab890be11b9b15dd793a2；manifest 925901de4eac79a2d4a1e12540bfff0bd24417c42158a9606875041ff7b222ed |

R2实际ER03 FAIL/H0 PASS，session e9f0e17d-07b4-40d8-9098-3f2a6e68c857，output SHA256 92b3745fc0f02790221e06d6db9f80ce56acab39d945378cfeeb23cab6e6c5ba；原三P1已闭，新的终态证据/被测树矛盾由本轮预登记终止链修订。**R3时点快照：** 当时R3尚待实际审查；后来R3 FAIL与R4 PASS见实际日志，不倒改原结论。P2-7已实际核GitHub上游存在旧plan-20261015（blob bfcf4e20aff8289dce9c7a0889e5ebb9ddb926f9）；不能因此认为含当前本地治理。新计划改用纯文本说明原计划；发布clone仍须自身链接核验。

### 本轮前滚修订历史

| 时间 | 触发 | 变更内容 | 原卡→新卡 | 受影响引用 |
|---|---|---|---|---|
| 2026-10-24 19:47:13 UTC | 本地Claude R1 FAIL | 同步五门、状态登记窄写集、bump后C及发布后资格树、逐项D与安装缓存界限、风险/审查/粒度表 | 三卡编号/轴不变 | 两仓8份治理、外部新计划、原LTR10顺序/卡片/审计表；R2待审 |

### R3审查修订

| 时间 | 触发 | 变更 | 卡/依赖影响 | 当前事实 |
|---|---|---|---|---|
| 2026-10-24 19:56:18 UTC | R2 ER03 FAIL/H0 PASS | C前冻结pending事实、C到merge零文档漂移，后继补记C/D，v2.14终态纯文档提交/例行补记，FIX03自身与计划完成门分离 | 三卡编号/轴/依赖不变；新增末端文档两路径门与一次3600秒预算，非新产品发布卡 | R3候选；未实施/测试/提交/发布 |

**R2残余P2处置（root）：** 非Linux与PID取得前的身份能力在claim前核，不足则ER03；D的release/**措辞修正。来源存在性现以独立clone HEAD=origin/main=`0ca47cc147b2099ffb835ade91272c4cfe1efd21`为冻结锚点，不能把未绑定ref的旧GitHub查询当精确来源证据。独立clone已核干净、独立SQLite、4702个祖先不含ed566b0/2755eb7；身份已从继承的测试默认值改为Eli Ma <eli@patch.sh>，本地签名开启、公钥1067ADBC49C4E7D18BA154AC21594333739ACBAF，尚无新提交签名验证。旧target927GB、reflink不支持，缓存复制/实际冷编译预算尚待claim前核定，不以方案PASS跳过该门。

**上游发布状态行边界：** 独立clone的plan-status本计划行只写本计划的三卡状态、实际来源/验收/版本/提交/UTC；不复制原工作区的scoped-traces0/26、两仓治理R轮或其未发布状态。本文自身R1/R2/R3实审记录是本独立计划的历史，可以随本计划发布。原工作区本地索引中的交叉前沿只由原Traces承载。

**B证据能力：** 自动检查证明字段/链接/依赖表一致、历史字节保持与写集边界；`published_tree_qualification_bound`等布尔项只是合同存在检查，不能证明未来运行或语义正确。终止链的可执行性由本轮实际Claude审查，未来树/hash与退出由真实执行核验。

### R4前滚与历史纠正

| 时间 | 触发 | 变更 | 结果 |
|---|---|---|---|
| 2026-10-24 20:03:41 UTC | R3 ER03 FAIL/H0 PASS | 终态ER05先审、仅补记收据、最终字节本地门、三树相等；STOP时前一卡C/D落点；例行维护自身门；只读B脚本合同 | R4待真实审查，零产品执行 |

R3真实session `82ca0f53-d8f6-4148-af66-13721b381b95`，output SHA256 `8f15fea4fd997f6157f3a82d323b286a9838b91fe8ef5e960a05be1baaa9876b`，ER03 FAIL/H0 PASS，原件保留。R3的候选生成器把R2已写的两条历史行时间从2026-10-24 19:47:13 UTC改成19:56:18 UTC；本轮恢复原始行时间，仅本轮新行使用新时钟。不倒改R3原件或把其失败写成成功。

**RISK-QUAL-TERMINAL-BASE（root，P2）：** FIX03 merge之后main可能前进，使终态树相对冻结资格树不再只有两路径。必须STOP并ER03审新差异/所需门；不能换基线后继承旧PASS、force或忽略新增来源。该风险可能延长交付，但不降低资格范围。

**签名证据边界（root，P2）：** 新clone仅完成本地身份、公钥与signing设置。首次实际提交后必须在同一个SHA核DCO、gpgsig并以导出公钥实际验签，失败不得push；GitHub签名verification单独按实际回执记录，不把本地开关或新公钥当成GitHub已信任。若实际仓库策略要求额外信任/注册，先解决后发布，不绕过规则。

### R4 PASS后的P2具名处置与治理自身审查

| 时间 | 事项 | root处置 | 边界 |
|---|---|---|---|
| 2026-10-24 20:09:49 UTC | P2-1 ISO历史横幅 | 逐字去时钟比较R1/R2后，将两仓8份文档共同横幅恢复至首次相同文本的原时间；R4未覆盖的ISO时间问题在本轮纠正，UTC两行恢复事实保持。原R1/R2/R3/R4候选及审查输出不改 | 本轮真实修改范围由最终diff/hash绑定，不把R4对旧字节PASS扩大到尚未审的修订 |
| 2026-10-24 20:09:49 UTC | P2-2 review log与旧状态 | 表内补R2/R3/R4真实结论、session/hash，R3待审句标历史快照；当前索引明确R4方案PASS、治理自身ER05待 | 不填未来审查/验收PASS |
| 2026-10-24 20:09:49 UTC | P2-3 后续例行维护 | 维护提交照终态第5项顺序：先文档核验及真实Claude，收据仅写plan-status本计划行，再对最终字节跑相关本地门，核提交/merge/被测树一致。执行前必须在当次维护写集中登记其独立的一次窗口及预算并实审，不挪用本计划终态3600秒、不自动再开；目前维护尚未执行 | 属v2.14例行维护，未获自己的实际执行准入，不影响本次方案审查；缺准入不得执行 |
| 2026-10-24 20:09:49 UTC | P2-4 缓存复制 | 原cache-selection只是开始前的依赖文件子集11178；实际复制31147还包括第三方build/.fingerprint，排除libra-*目录、第一方库/测试二进制、incremental、硬链接。当前复制仍在运行；必须取实际终态、逐项核数/排除与独占来源后才能claim | 缓存不是源码/测试资格，也未消耗或获得新产品验收PASS |
| 2026-10-24 20:09:49 UTC | P2-5 执行期残余 | 后继STOP补记、终态基线变化、同SHA验签/远端信任与终态只读执行器冻结均保留硬门 | 三卡和末端执行均未开工 |

本次R5仅对以上事实修订及两仓治理自身做真实本地Claude DOCS-ER05/H0审查，并确认R4方案准入没有规范回退；结果未取得前不宣称治理自身review完成。两仓原治理写集及MTR01/LTR01承载关系不变，新独立计划未来随自身三卡发布，原工作区仅本地保存。


### FIX-LTR-01 A终态及具名限制（2026-10-24 21:42:59 UTC）

- 单一A claim `2026-10-24T20:33:43.564369Z`，终态 `2026-10-24T21:39:46.332180Z`，3962.768秒/7200秒，执行器与原nextest配置/timeout/retries未改；原资格FAIL/STOP及耗尽的旧窗口保留，没有自动重跑。输入manifest2126项 SHA256 `eb26c80a4cc4b05274a1349b20e8ee59ec5dc56e337bd2aaa0da096b5e363dde`；终态SHA256 `c393a8baaf2cf741e19c1b4cac91fc1ad0bcaf9fd865b865f881670bf284a0cf`；root限定A收据SHA256 `e92bbed501956b8019ecadfae14cfbd2e2c7c6d3d652fdf391fe6baeda64692d`。
- 自身准入来自真实本地Claude session `f8b78ffe-b7d4-4459-a633-333bf24c55b0`，EXECUTION/VERDICT PASS，仅A准入、不是本卡代码ER05。自己的最终代码ER05仍待；未使用隔离Codex。
- 三个执行集合按listing/JUnit逐(target,case)核实：exact1、完整compat_global_config_schema_future54、默认全量10761；各自0失败、0framework skip、0flake，未丢失/额外case。全量10782个列举身份中原21 ignored保持逐名相同。full stderr SHA256 `c22ec3f2f66d6e9e7c3939a03cebffdcaee1e9b8a4fc764e5d19c7c080d4b1de`；JUnit SHA256 `80f85859f67f676cb7632bde4d81cc5a2019b866bfdca7a7d3e7089a3be44953`。本卡唯一测试差异仍为完整字串增加Ui排除，生产cli和所有runtime/startup guard不变；测试文件SHA256 `535f31b6aa50c500ad9fe3876e893e8fb02581caef83a6aef2e23ed13ae000bd`。
- **P2-QUAL-RUNTIME（owner root）：** 当前38个标记与原来源逐身份/文字相同，来源hash逐项核实；1个是`opencode_plugin_skip_visible_when_interpreter_missing`的真实负向assert，不是跳过。其余37个包含：9个default-feature下未启用test-live-cloud、10个显式GitHub live未配置、12个Mega2 browser live未配置、1个Mega LFS endpoint未配置、2个大小写不敏感文件系统前提、1个当前build拒绝stage非UTF8文件、Bun与PowerShell各1个。保持已登记默认全量命令并披露缺口，不称已证明这些行为；本卡exact/完整target没有runtime skip。FIX02必须另跑具名真实云用例；H1/LTR的live门不被本卡覆盖。原件 `/tmp/fix-ltr01-A-20261025/root-terminal-audit.json` 及 `runtime-skip-source-reference.json`；自己的Claude必须独立判断此限定是否充分。
- **P2-QUAL-WORKER（owner root）：** 本次唯一新PPid1进程为PID2074789/start_ticks10678129、session1941756，exe属于本clone且fixture cwd已deleted；命令为read-worker，STATUS_PAUSE=1及本fixture pause.pid、无READ_DEADLINE_MS，匹配当前`read_worker_status_spawns_no_helpers`的唯一环境形状，区别于带2000ms deadline的另一调用点。没有原PID文件，不倒推旧历史PID4064241身份。终态后pidfd绑定身份、单次SIGTERM，`2026-10-24T21:40:25.108156Z`确认退出并已回收，全量baseline差集无其它新孤儿/owned残留。人工清理SHA256 `456b49ad48fc97b83ceb29025f3b465703f5ffb6b609742659dea36cf5dfe70b`；原资源失败事实保留，不宣称自动cleanup通过、FIX03修复或最终来源资格完成。此处仅处置已登记FIX03风险；自己的Claude若判P0/P1则STOP，不以root的P2归属代替PASS。
- **P2-QUAL-COMPILER（owner root）：** strict Clippy exit0；既有`proc-macro-error2 2.0.1` future-incompatibility提示仍存在，不把“零lint失败”写成“零任意诊断文本”。若后续C门对此有新的禁止条件，先独立归属，不静默免门。
- C入口只读预检：远端main仍0ca47cc，三版本面0.30.86一致；账户有push/admin权限但不使用admin bypass，release environments无人工review要求。两clone均没有release target，不能把debug cache冒充release暖缓存；C一次10800秒仍未claim，须以实际冷release条件和本次66分钟A重新进行真实Claude执行准入/预算判断，不在窗口外预编译、不自动延长。草稿 `/tmp/fix-ltr01-C-20261025/run-C.py` 尚未获准或执行。
- FIX02外部条件只读预检在 `2026-10-24T21:30:04.500055Z` 完成：D1仅SELECT常数、R2仅HEAD既有bucket，均HTTP200；无fixture/用户数据读写，无secret值日志。它不是FIX02的真实写入/erase/cleanup验收，更不是本卡full的live覆盖。
- 当前代码完成、A通过；自己的ER05待、提交无、push无、CI/Release无；本卡仍in-progress/locally-accepted，正式0/3。B按implementation无额外类型门，但两文档结构/链接、状态行only与原生产字节保持须在自己的审查前再核。C前只允许追加实际自己的review收据、更新本计划状态行，然后按已登记版本/bump/最终树门执行；没有放行后继实施或借原工作区治理推送。


### FIX-LTR-01 自身ER-05实际收据及C准入条件（2026-10-24 21:54:29 UTC）

- 真实本地Claude于2026-10-24 21:48:29 UTC返回`ER05_VERDICT: PASS`及`VERDICT: PASS`，P0/P1无；session `c7fb3f6b-3738-4d33-830a-be70172cd39f`，输入manifest SHA256 `504664b7f68d4b13f6f02703543fee559db322f45de7580d978f1d40397467e7`，输出SHA256 `8b43a841bf8566e6a3692601e6c0a13643dc49bfe5534005c4ec31a6177d83a1`。原件 `/tmp/fix-ltr01-own-review-20261025`；root实际重算manifest/output及逐输入hash，未漂移。Claude明确披露未自行计算hash，不能把root核验写成Claude独立计算。该结论只覆盖既定源码/文档和已发生A终态，不代表C准入或C/D完成。
- P2-1由root承接：37项runtime early-return缺口只是打印标记识别的下限，不穷举静默early-return或未启用feature；FIX02及最终资格必须保持此限定和具名live门。1项负向assert与37项缺口分开。
- P2-2由root承接：C执行期间如发现与本次A同样形状的已登记FIX03孤儿，仅允许在终态后、核本窗口观察到的PID/start identity、nextest session、独立PGID、read-worker argv、已删除fixture cwd及STATUS_PAUSE/pause.pid/无READ_DEADLINE_MS后，以pidfd单次SIGTERM人工回收并记录exit/reap；不发组信号、不按进程名清扫、不自动重试。任何其它形状、多个未能唯一归属的进程或stdout-holder（RISK-QUAL-HOLDER）立即STOP/ER03；不以手工清理冒充自动cleanup PASS。FIX03后的最终来源资格必须零孤儿。C执行器只记录并拒绝未知残留，不自行执行人工清理。
- P2-3由root承接：保留proc-macro-error2的future-incompat提示，strict Clippy实际exit0不扩称零诊断文本。P2-4的旧候选句/横幅格式在FIX02开工的既定文档写集中标为历史，本次不改已审正文。
- P2-5由root承接：cold release、实际A耗时及C一次10800秒上限必须另获真实本地Claude执行准入；尚未claim、bump或预编译，超时/失败STOP，不追加或转移旧窗口。只冻结本次实际后置收据和本计划status行后，再由C审查绑定最终字节/执行器/环境控制。
- 审查边界披露：Claude为核计数使用Grep读取full.stderr的Summary、两条MCP PASS和FLAKY/TRY计数，并读Clippy提示两行，超出原prompt禁止原始测试输出的约束；未读.env/credentials/JUnit正文。保留原输出及披露，不伪造为完全遵守；后续准入明确允许这些无secret的受限摘要。
- 当前：源码及A通过、自身ER05 PASS，Lifecycle/Acceptance仍in-progress/locally-accepted；提交、push、CI/Release与C/D均未完成，正式0/3。本文收据为审查后的事实追加，不声称Claude事先审过自己的输出；未来C/D由FIX02按既定终止链补记。


### Task DOC-LTR-01: 清除PR635暴露的项目Rustdoc警告

**Task type:** `docs`

**Lifecycle / Acceptance:** `in-progress` / `locally-accepted`（A/B及自己的Claude ER-05已实际通过；signed/DCO 5c449bba已推送并经PR636合并b73e98a，PR/main全部适用D实际通过；完整C/D仍待FIX-LTR-03覆盖，不提前done）

**Description:** 用户要求“在 Libra 的下一个提交修复这些 warning”，随后明确排除第三方 `proc-macro-error2 2.0.1` future-incompatibility。只修PR635实际job 121059262116的104条项目Rustdoc诊断：72私有目标链接、28非法HTML占位符、4冗余显式链接。不改依赖、lock、可见性、运行逻辑、CLI输出或lint等级。

**Out of scope:** 第三方未来相容性警告按用户指令保留；FIX02/03源码、原scoped-traces治理及功能未完成状态不借此关闭。本卡没有版本bump、tag、Release或release workflow。

**Current evidence:** PR635 head2852497、被测tree90ca1d20、squash merge f33be958dcc3266df3a6ab796ed90d1766f3cd93；base40143598901 attempt2在Mechaevo的strict Clippy与Rustdoc均exit0但有104条文档warning。原attempt1在eli-MI缺fontconfig.pc失败保留；用户已移除该不一致runner，不能称已修复该机器。Cargo第三方警告出现两次，不计入104条项目warning。日志 https://github.com/libra-tools/libra/actions/runs/40143598901/job/121059262116 。精确源锚点：`src/command/remote.rs:139`（Add.master，不是set-head）、`src/utils/worktree_blob.rs:15`（私有与冗余诊断同处）、`src/command/auth.rs:66`（相对时间占位符）。

**Acceptance criteria:**

- [x] 私有目标的说明保留为代码字面量，不扩大生产可见性。
- [x] HTML占位符在Rustdoc显示为原意字面量。
- [x] 公开目标的3条冗余显式链接（sandbox/mod.rs:88、d1_client.rs:8/361）改为等价短链接；worktree_blob.rs:15同处私有目标已由AC1覆盖，保留代码字面量。
- [x] 非文档源保持逐字节相同。
- [x] 普通编译的Clap help文档输入与输出保持相同。
- [x] 第三方依赖及lock保持原字节，不抑制lint。

**Verification:**

- [x] `source .env.test && source .env.live-test && RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features`；全部项目Rustdoc警告为0，唯一允许的Cargo级提示是用户排除的proc-macro-error2 future incompatibility，分开记录不称零诊断。
- [x] 精确diff守卫：仅下列53文件的文档注释/文档属性变化；还原普通编译的cfg_attr(not(doc),doc=原文)后，去文档源码与基线逐字节相同；版本、Cargo.lock、依赖、配置、所有测试不变。
- [x] 真实新二进制与已验收0.30.87二进制逐字节比较受影响命令的`--help`/`-h` stdout、stderr、exit：auth login、cat-file、cherry-pick、clone、commit、format-patch、rebase、remote add、show-ref、switch、symbolic-ref。help子进程不加载.env文件，以白名单环境运行在仓外空临时目录：stdout/stderr管道（非TTY），COLUMNS=100、LANG=LC_ALL=C.UTF-8、TZ=UTC、TERM=dumb、NO_COLOR=1、CLICOLOR=0、CLICOLOR_FORCE=0，隔离HOME/XDG/全局及系统config路径。旧binary是已核C树90ca1d20的`/tmp/fix-ltr01-C-20261025/install/bin/libra`，SHA256 `acc1e55fad788f8c809beeb2025b1b839842d51bb8151018571d5742d1ae985c`；新binary来自本卡实际debug build。所有调用必须exit0；debug/release或元数据差异也STOP，不能忽略。

**Full-suite trigger:** `none`（本卡no-release，不自行执行C；完整覆盖仍继承FIX-LTR-03；本卡只有文档注释及cfg(doc)文档属性，没有CLI注册、参数/错误/配置/执行逻辑改动。若普通编译行为/元数据不再等价，则STOP、ER03重分类及所需全量，不能沿用本判定）。

**Dependencies:** `DEP-DOC-PR635`（incoming：精确已合并f33be958/tree90ca1d20可读取；本卡只消费已保存源与本地A/C，不宣称FIX01远端D或整体来源资格已完成）。

**Deliverables:** 下列53源文件内的Rustdoc注释/属性、本计划与plan-status本计划行；不新增仓库证据文件。

**Implementation write set:** `src/cli.rs`；`src/command/add.rs`；`src/command/agent/investigate.rs`；`src/command/agent/review.rs`；`src/command/auth.rs`；`src/command/bisect.rs`；`src/command/blame.rs`；`src/command/branch.rs`；`src/command/cat_file.rs`；`src/command/cherry_pick.rs`；`src/command/clone.rs`；`src/command/cloud.rs`；`src/command/commit.rs`；`src/command/format_patch.rs`；`src/command/gc.rs`；`src/command/init.rs`；`src/command/rebase.rs`；`src/command/rebase/state.rs`；`src/command/remote.rs`；`src/command/rename_detect.rs`；`src/command/rerere.rs`；`src/command/shortlog.rs`；`src/command/show_ref.rs`；`src/command/status.rs`；`src/command/status/input.rs`；`src/command/switch.rs`；`src/command/symbolic_ref.rs`；`src/internal/ai/agent_bridge/vcs.rs`；`src/internal/ai/agent_run/event.rs`；`src/internal/ai/agent_run/workspace_strategy.rs`；`src/internal/ai/capture/ingress.rs`；`src/internal/ai/capture/live.rs`；`src/internal/ai/capture_scope.rs`；`src/internal/ai/history.rs`；`src/internal/ai/hooks/runner.rs`；`src/internal/ai/hooks/runtime.rs`；`src/internal/ai/observed_agents/builtin/gemini.rs`；`src/internal/ai/observed_agents/reasoning.rs`；`src/internal/ai/observed_agents/redaction.rs`；`src/internal/ai/sandbox/mod.rs`；`src/internal/ai/sandbox/proxy.rs`；`src/internal/ai/sandbox/runtime.rs`；`src/internal/alternates/mod.rs`；`src/internal/auth.rs`；`src/internal/branch.rs`；`src/internal/db.rs`；`src/internal/sequencer/mod.rs`；`src/internal/upgrade/txn.rs`；`src/internal/workspace.rs`；`src/utils/client_storage.rs`；`src/utils/d1_client.rs`；`src/utils/redact.rs`；`src/utils/worktree_blob.rs`；`docs/development/plan/plan-20261025-qualification.md`；`docs/development/plan/plan-status.md`仅本计划行。

**Release write set:** `N/A`。

**Files likely touched:** 与I相同。53源文件中只改Rustdoc；Clap消费的HTML占位符文档使用成对cfg_attr(doc)/cfg_attr(not(doc))保留普通编译原始doc字符串，不能改arg属性或产品分支。

**Docs and compatibility impact:** 正式Rust API文档由源注释生成；CLI普通help逐字保持，不增加/删除命令、参数、配置或用户语义，因此双语命令指南、COMPATIBILITY、错误码、网站内容N/A。任何实际help变化先STOP，不能宣称文档-only继续。

**Rollback mode:** `immutable-release`

**Migration and rollback:** 无数据迁移；已推送只以精确后续提交前滚，不重置或force。保留原warning日志与两次CI事实。

**Security and privacy:** 不动权限/可见性；只读隔离help，不传真实凭据；审查不读env值，原始证据位于仓外受控目录，仓内仅本计划摘要。

**Performance budget:** 一次新的本卡A窗口3600秒（含nightlyfmt、strict all-target/all-feature Clippy、上述Rustdoc、debug build、help比对、B及资源检查）；不运行全量、不占用旧FIX01或FIX3预算，不自动重跑。与Mega C的Cargo/构建/提交/push/发布严格串行。失败STOP并保留真实终态后ER03。

**Estimated scope:** `M`；按G-11文档-only的≤2人日口径，53源文件文档+2治理文件，预计≤1人日；不以生产代码卡的5文件上限伪装53文件实现。

**Version increment:** `N/A`

**Release boundary:** `no-release`；先精确signed/DCO本地提交（满足用户下一提交约束），可推短期docs分支并经本地Claude PASS、实际PR base/CodeQL后gh squash merge；不自行触发版本发布。作为现行资格计划文档，进入FIX-LTR-03唯一指定覆盖树，不把FIX01的旧C覆盖本次修改。

**C/D coverage from:** `FIX-LTR-03`；本卡自己的PR/main D还须实际核验；FIX03最终C/完整D覆盖本卡后才done/complete。在此前只可locally-accepted，不新增资格实现卡计数（原三卡仍分开计0/3）。

**Granularity:** `type=docs; axis=项目Rustdoc诊断收敛并保留CLI help; recovery=文档精确前滚; complete=yes; self-contained=yes; AC=6/20; VER=3/20; landing=1同主题Rust API文档集; prod-files=0; scope=M(≤2人日); deps=DEP-DOC-PR635; writeset=与FIX02/03共享计划行串行; release=no-release; split-from=用户指定PR635 warning后续提交; exception=N/A`

**ER01/02/03与消费规则：** 独立原生clone，当前branch docs/pr635-rustdoc-warnings来自f33be958，不含原工作区unsigned merge或未发布治理。准入R2与执行器R2实际PASS后已采纳55路径，A/B实际通过；自己的Claude ER05仍须绑定当前最终源码及下述实际终态，通过后才精确提交。source候选的diff自动守卫不等于真实Rustdoc/二进制验收。模板G-03文档清单与G-11时间口径适用；没有G08/G12/G13聚合。root唯一写者与发布者。FIX02/03依赖不改，不制造DOC与覆盖点的执行依赖环；最终资格收口同时核本文档卡覆盖。

**后置事实落点：** 本卡提交前冻结最终文档字节；自己的审查/A/commit/PR/main收据由FIX02的既有本计划/状态行写集补记，FIX03 C/D后补记覆盖。当前未知项不得预填PASS，后置事实不声称已被先前审查读取；不新建证据目录。若FIX02开始时本卡PR未完成，保持pending并补同步，不机械重跑未变源码。

| 修订UTC | 来源 | 变化 | 状态 |
|---|---|---|---|
| 2026-10-25T03:14:21.976945+00:00 | 用户指定下一提交修PR635项目warning并明确排除第三方 | 新增DOC-LTR-01，53文件文档-only；无依赖升级/版本bump；FIX03承载C/D | 候选待Claude，未实施 |


**DOC-LTR-01开工R1实际FAIL与R2前滚（2026-10-25）：** 本地Claude session `17ca5be0-55a2-4f0f-acc4-3821a4d22635`，ER03 FAIL / DOC_ONLY PASS / 总FAIL；输出SHA `ab500ad0f082f89210ab4e98f2a3cd2f8d661721000bde845c38d4eac07655ca`。两P1已在此候选改正：VER3必须测remote add，AC3区分3个公开短链接与1个私有字面量。R1精确126输入已仓外封存，源码候选53文件字节不变；仍未采纳、未运行A、未提交，R2待审。

- root/P2-1：guard还原成对普通doc后，逐文件行数和每个原诊断行号/位置逐字比较，非文档源码仍逐字相同；最终help证据只能来自22次真实二进制调用。
- root/P2-2：55个前置文件实际用`libra show f33be958:<path>`核blob SHA，HEAD/index/worktree清洁；采纳前再核前像，采纳后允许diff必须恰好55路径，Cargo/lock/配置及测试hash仍全量守卫。
- root/P2-3：VER3具名旧二进制与固定隔离环境；Cargo门仍source双env，help自身用独立白名单env，不继承凭据或真实config。任何输出差异STOP，不删断言。
- root/P2-4/5：B明确包括本计划结构/相对链接、I精确55路径及原历史记录保持；Rustdoc -D warnings承担源内部链接门。A比CI更严格；新lint不扩修，失败STOP/ER03。本次none仅文档注释，不改CLI注册/全局flag。
- root/P2-6：cat_file.rs两处同句`<input>`虽不在104诊断里，Rustdoc会把它当HTML input元素；仅cfg(doc)侧同样按字面量呈现，普通help原字节保持，不作为删减旧诊断计数。
- root/P2-7：首次claim前将执行器/控制项/源树真实Claude审查；A后若只追加收据，限此两治理文件并记录精确diff，最终ownER05仍须绑定实际终态及最终字节。不得在Mega C期间执行本卡Cargo/提交/push/发布。
- root/P2-8：FIX02/03开工来源必须包含本卡实际merge SHA（待发生后填入），FIX03最终C树须含本卡最终55路径；不符STOP/ER03，不将未覆盖的DOC记done。此为来源入场门，不改变原两卡依赖边或制造覆盖循环。
- root/P2-9：上方Current evidence补精确源行。默认debug build如出现原FIX01已审的capture-ingress两associated functions dead_code提示，需绑定未变源码、单列原诊断与具名P2审查，不称零诊断，不用allow或夹带生产修复。本卡目标仍为所给job的104项目Rustdoc警告；第三方警告按用户排除。


| 2026-10-25T03:30:53.098949+00:00 | DOC-LTR-01准入R2 | 两P1已关闭，ER03/DOC_ONLY/总PASS，源码仍未采纳；执行器实审与A/ownER05待 | 准入通过，未验收 |

**DOC-LTR-01准入R2实际收据（2026-10-25T03:30:53.098949+00:00）：** 本地Claude session `b2377d23-968e-469f-ad00-521220bdbdd1`，`ER03_VERDICT / DOC_ONLY_VERDICT / VERDICT: PASS`，输出SHA `47dd9c0d4d3be46a6b3b0d523655bfa1882d378543539f3f7c99be48353f633d`；132输入实核无漂移并仓外封存，原R1 FAIL保留。root具名承接R2六项P2：采纳前程序逐项比较55原生blob前像、采纳后精确55路径；22次真实help比对使用每次全新空HOME且相同固定路径/环境，不能忽略debug/release差异；本收据及状态行补记时点；旧binary先核SHA，缺失STOP；Cargo/构建/提交/push/发布仅在`/tmp/mtr99-C1-20261025/root-local-C-acceptance.json`实际成立后启动；R1 P2处置继续有效。上述仅准入PASS，不是实施、A、ER05、C或D结果。

### DOC-LTR-01 A/B实际终态与最终审查入口（2026-10-25 04:25:21 UTC）

一次A1从2026-10-25 04:08:57 UTC至04:23:30 UTC，实际874.796/3600秒，coordinator exit0、STOP=false。48命令全部exit0：nightly fmt无diff、strict all-target/all-feature Clippy、`RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features`、debug build，以及11命令×2help选项×旧/新二进制44个进程。22组stdout/stderr/退出码逐字节相同，实际help与CLI语义未变化。四Cargo门加载双env；第三方proc-macro-error2提示按用户指令保留，Clippy/Rustdoc/build的项目warning均0（本次build没有出现先前已登记的capture-ingress dead_code提示）。不称所有诊断为零，不升级依赖、不抑制lint。

53源文件仅文档变化、16组成对cfg_attr保留普通Clap输入；全部2126输入在A期间保持冻结，版本/lock/测试/配置不变。B实际55路径、190相对链接、6AC/3VER通过；剩余owned进程0、新孤儿0、无需人工清理。本卡按已审ER13不运行全量，不能替代FIX03完整C/D。终态SHA256 `72f9bcd758491d27bcbd27280d553897865d16ef4231ce5595281cd45b865847`；命令结果SHA256 `7d471fb2d5ace8d669828784c5eda1f1d2f88c90e4764c0dacbcb6248c8282ce`；help比对SHA256 `ad7afb84a0eb26ae65b37e0625876a5751f113dc832cc5fdfef24f762c5da680`；B SHA256 `34bf4a1954cfd0e936a11c874eb97bac67d1f7115d13a5fe25374000250146b0`；输入manifest SHA256 `b02b8ceadd197d69df767d20c9e95fc716f96fdcf6a705f8034eeb8a217e5ece`。原件仓外`/tmp/doc-ltr01-A1-20261025`。

实际执行器R1 FAIL（status领先空格被strip）保留；该问题于claim之前修复，R2本地Claude session `8dda363f-cecb-48ea-9d67-6b5b01caefc1`，EXECUTION/DOC_ONLY/总PASS，输出SHA256 `8f9dd9147b5d3e921255c31e8dcf2aa36ac622809a4bb728ae0c6aab6bfea292`。源码准入R1 FAIL/R2 PASS同样保留。执行器审查不是自己的ER05；本节之后只变更本计划状态/实际证据与plan-status本计划行，源码/测试/配置/版本/lock与A相同，最终文档差异另做B并纳入真实Claude审查，不机械重跑未变编译/二进制。自己的审查、signed/DCO提交、push/PR/main D目前均待；审查以后事实按既定后置协议由FIX02补记，不预填结果。

root继续承担准入与执行器P2：任何新源码/元数据/help变化先STOP重评，既有FAIL/预算不重置；环境/进程异常不得自动续窗或杀非owned进程；53文件文档范围不可扩大。DOC自己的PR/main必须实际核验，FIX03最终覆盖树必须含本卡最终55路径。FIX01的main base40156941519尚未终态前不合并DOC至main，以免取消前一卡适用D；CodeQL40156941508及release40157032761已实际success，FIX01仍未done。原计划三实现卡0/3不变。

| 修订UTC | 来源 | 变化 | 状态 |
|---|---|---|---|
| 2026-10-25 04:25:21 UTC | DOC-LTR-01 A1实际终态 | 48命令/22help/B/资源通过，保留历史FAIL；仅两治理文件补事实 | locally-accepted，自身ER05及提交/PR/覆盖待 |

### FIX04旧依赖字段原件（2026-10-26前）

```text
**Dependencies:** `FIX-LTR-01`、`DEP-QUAL-SOURCE`、`DEP-QUAL-PUBLISH`；FIX02的已保存C1警告是来源证据，不以其done为前置。`FIX-LTR-04 -> FIX-LTR-02`，无反向覆盖依赖。
**Granularity:** `type=implementation; axis=同步ingress测试构造器的profile可用性; recovery=immutable-release; complete=yes; self-contained=yes; AC=4/8; VER=5/8; landing=1; prod-files=1; scope=S; deps=FIX-LTR-01,DEP-QUAL-SOURCE,DEP-QUAL-PUBLISH; writeset=两cfg及既定两治理+版本面; release=independent; split-from=FIX02-C1实际release警告STOP; exception=N/A`
```


### 2026-10-26 DOC02 R1真实FAIL与前滚修订

本地Claude session `df82ac52-8f07-4963-9319-786726e02a92`，五项FAIL，manifest `4788f76846dea304dd9293b0149c9682a2fee41e0504496ca5b2c4857964de2a`、output `e424f6da315e011a52e390d2267163583c6004ee5bbd2e550e47f04151751600`；root保留原结论。P1-1五卡汇总/完成门已前滚；P1-2拆ATTRIB与CLOSE，诊断只在已准入FIX05内；P1-3/4登记真实源码限制及未准入粒度，不称一文件设计可行。P2 owner=root：补FIX05 keyring门、env/serial条件与配置panic范围、LIVE/未发布incoming边、新stash；DOC02改docs并登记fmt/Clippy/main D。历史行号差一行保留原文。以上是修订候选，复审未取得前不记PASS，不授FIX05实现或warning例外。

以下原规范行按2026-10-25快照保留；现行要求见前滚后的对应段，禁止把旧四卡门当成当前完成条件。

```text
候选独立 A 窗口：FIX-LTR-01一次7200秒，FIX-LTR-02/03/04各一次10800秒（共享夹具真实云消费者或最终doc/network，均包含完整默认套件）；每卡最终 bump 后本地 C 各一次10800秒；FIX-LTR-03 的 C 包含完整全量、doc及原3例network。四卡实际D全绿后另有一次计划最终资格窗口10800秒，执行最终发布树fmt/clippy/full/doc/network，不以C代替模板计划完成门。上限是新独立来源的命令编排预算，不修改任何测试内部 deadline、并发组、断言或原 nextest retries。依据本机已测 build+listing+full4983秒及 strict Clippy约876秒，额外预留最终 release build/网络 feature 编译；不把冷构建慢当代码原因。本地窗口从第一条Cargo命令到本地门/构建/安装结束，包含编译和listing；PR CI、merge及Release D远端等待不计入该本地执行预算，单独登记run/job/ref，观察超时不重启作业、不记PASS。任一意外失败/超时/漂移立即保留终态并STOP，不自动再开窗口；后续只读归属与另行ER-03，不因剩余时间继续命令。
每卡 A 包含其 Verification、nightly fmt及strict all-target/all-feature Clippy；触发全量的卡另跑原双env默认全量。自己的 ER-05 PASS 后，C严格按模板ER-04/08顺序：版本面parity→实际patch→工具链刷新lock→最终fmt/clippy和本卡完整全量/所需focused→`cargo build --release`（四卡全局C⑤要求项目warning=0，唯一排除为用户具名proc-macro-error2 future提示；FIX04另加keyring release门）→将本卡release二进制安装到仓外、每卡独立的 `install/bin/libra` 并以绝对路径核version/hash（复制+chmod0755，hash须等于本次target/release/libra）；不覆盖原工作区使用的libra或改变全局PATH→精确`libra add`及signed+DCO提交→feature branch/PR→该head的base/CodeQL全部适用job通过→squash merge并核merge SHA/tree/DCO/签名；merge tree必须等于已测bump后tree，任何上游变化或树不同即STOP并ER-03，不tag、不借旧PASS→人工Release及版本tag→release工作流与渠道artifact真实核实。遇分支保护、必需他人review或环境approval，保留实际pending状态；不使用admin bypass、不自行批准。未完成D不得done。编译诊断按实际记录；若既有future-incompat提示违反当次发布门，先独立归属，不静默豁免。
4. **唯一终态登记提交。** 计划最终资格通过后，在独立clone创建一笔纯文档登记提交，I精确为 `docs/development/plan/plan-20261025-qualification.md` 与 `docs/development/plan/plan-status.md` 的本计划行；R=N/A、version=N/A、无tag/Release、不改变任一产品/测试/配置/lock/版本面。该提交承接FIX03真实C/D、最终资格和四卡状态，以及它自己的真实Claude文档ER-05收据；核对实际证据，登记本终态文档自身D仍pending。它是本计划预登记终止链末端，不是第五个产品修复或新版本发布点，不替代四卡各自C/D。没有独立发布原工作区Traces治理的权限。
**终态B的只读执行器：** 使用仓外 `/tmp/libra-qualification-recovery-plan-20261025/terminal-document-check.py`（在该末端执行前生成、hash冻结并纳入真实Claude输入，不属于产品源码）。执行命令为 `python3 /tmp/libra-qualification-recovery-plan-20261025/terminal-document-check.py`，以已核FIX03 merge SHA、候选树及两路径allowlist为显式冻结输入；缺输入即失败。必须以原生Libra diff逐路径证明只有本计划与plan-status本计划行变化；全其它来源hash一致；卡片/索引状态与已发生原件逐项对应；本计划全部必需章节/四卡字段存在；相对链接解析到当前clone；已登记审查收据只作后置事实，不改审查结论或其它字节；保留所有旧FAIL/STOP。自动结构结果不替代人审语义和真实门结果。
| DEP-QUAL-F2 | incoming | 当前失败归属 | reset_pathspec_from_file_missing_path_errors真实stderr与根因 | Libra接收Agent | 按2026-10-26入口完成受审诊断及P1处置；未知不能关闭，独立根因另卡 | 旧stderr缺失，当前未完成 | 不借native retry PASS或未复现关闭；诊断预算另审 |
| DEP-QUAL-HANDOFF | outgoing | 来源资格 | plan-20261015 LTR-10 / LTR-01 | root | 四卡独立C/D完成、FIX03 bump后C及发布后最终资格窗口全量/doc/network与本地Claude明确PASS（绑定实际merge树）、终态登记提交实际PR/main D通过并具名例行补记，再核受控整合树、原工作区受控整合/受影响门通过 | 后续真实提交/版本/远端ref/原件，当前无 | 来源资格未闭合不得解除LTR-10前置；不更改H1或替代LTR各卡门 |
`DEP-QUAL-SECURITY + DEP-QUAL-SOURCE + DEP-QUAL-PUBLISH -> FIX-LTR-01 -> FIX-LTR-05 -> FIX-LTR-04 -> FIX-LTR-02 -> FIX-LTR-03 -> DEP-QUAL-HANDOFF`；另有 `DEP-QUAL-F2 + DEC-QUAL-05-RELEASE-WARNING -> FIX-LTR-05`，当前授权与诊断未闭合，候选顺序不等于准入。顺序用于共享本计划/版本面的串行发布，不表示合并发布。先修确定性全量阻断，再修共享fixture，最后完成MCP测试清理与整份修复计划的最终资格。每卡真正 C/D 闭合后才发布下一卡。

现行顺序（2026-10-26，不以本快照为准）：`FIX-LTR-01 -> FIX-LTR-05P -> FIX-LTR-05Q -> FIX-LTR-05R -> FIX-LTR-05S -> FIX-LTR-05T -> FIX-LTR-05U -> FIX-LTR-05V -> FIX-LTR-05`。05V 当前切片已准入，但配置失败后端仍未做，FIX-LTR-05 仍 blocked。见上文「实施顺序与发布窗口」。
### 四卡共同适用的逐项D清单
**Granularity:** `type=implementation; axis=global-schema doctor的auto-upgrade结构oracle对齐; recovery=immutable-release前滚本卡单轴测试修复; complete=yes; self-contained=yes; AC=3/8; VER=3/8; landing=1; prod-files=0; scope=S; deps=DEP-QUAL-SOURCE, DEP-QUAL-SECURITY, DEP-QUAL-PUBLISH; writeset=四卡本计划与R串行; release=independent; split-from=新上游资格20261025实际FAIL/STOP; exception=N/A`
**Granularity:** `type=implementation; axis=合成checkpoint fixture的索引结清前置; recovery=immutable-release前滚本卡单轴测试修复; complete=yes; self-contained=yes; AC=4/8; VER=3/8; landing=1; prod-files=0; scope=S; deps=FIX-LTR-01,FIX-LTR-04; writeset=四卡本计划与R串行; release=independent; split-from=新上游资格20261025实际FAIL/STOP; exception=N/A`
- [ ] 完整agent_mcp_exec_test通过，原deadline、内存、slot、禁止helper断言全部保持；本卡bump后C全量/doc/network通过，实际本卡Claude ER-05通过；四卡D之后的计划最终资格属于计划完成门，不倒挂本卡done条件。
**Granularity:** `type=implementation; axis=MCP read-worker测试夹具完整teardown; recovery=immutable-release前滚本卡单轴测试修复; complete=yes; self-contained=yes; AC=4/8; VER=4/8; landing=1; prod-files=0; scope=S; deps=FIX-LTR-02; writeset=四卡本计划与R串行; release=independent; split-from=新上游资格20261025实际FAIL/STOP; exception=N/A`
| 修复里程碑 | FIX01(done)→FIX04→FIX02→FIX03分别独立完整C/D | 发布串行，不能合并、不能以手工worker清理当FIX03修复 |
四卡分别实际A/B、Claude ER-05 PASS、精确signed/DCO提交、独立patch/PR/merge/tag/人工Release、实际CI与发布artifact D完成；全部P0/P1关闭，P2具名处置。FIX-LTR-03 的C④在bump后树执行完整默认套件、`cargo test --doc`及原3例network；merge树与被测树不等即STOP。四卡各自实际D完成后，在与FIX03发布merge树相同的冻结来源执行上述一次10800秒计划最终资格窗口（fmt、strict Clippy、双env完整默认套件、doc、原3例network），核全部身份/退出/ignored/flake/runtime skip与owned资源；根因未知的缺口不得改成已证。最终实际本地Claude来源资格PASS后，才把四个具体发布SHA/版本及适用覆盖交给原scoped-traces计划；其H1、原merge签名政策、LTR自身A/B/C/D仍独立。
当前两原计划正式17/118、0/26、17/144（活动17/142）；本外部候选四个实现节点独立单列1/4，不增加原26卡或改写初始统计。联合依赖审计展开FIX01→FIX04→FIX02→FIX03并与H0/H1/LTR10核无环；任一卡/末端交接证据不足均未完成。
```


### 2026-10-26 DOC02 R2真实终态与修订

本地Claude `6efd0e29-4c53-4b59-beda-261df8c0bab2`：HANDOFF/SOURCE_AND_DEPENDENCY/EXECUTION_BOUNDARY/VERDICT四FAIL、GRANULARITY PASS（仅候选登记，不准入FIX05）；manifest `1a72abb34ce985319ee0f39ae444a3d0b5ca59df6571913e194d4dd8ead30315`，output `4fcc1356ac3c1b62860c70a9e9bef82f7b87afa7106c5071a421dd78b55583a4`。R1原问题已关闭，R2三个P1由本次修订候选处理：FIX04粒度边改F2-CLOSE，并规定FIX05后置C/D由FIX04的两治理补记；执行器在claim前及每个命令加载双env后均拒绝三个全局目录override；提交器核own-review包含最终正文与实际gate终态，signed message补manifest/terminal/gate-input hash。复审未取得前不写PASS。

P2 owner=root：真实DAG仍须Claude逐字段人审，机器图仅辅助；环境异常移到claim前，不消费新窗口；deadline后的未知/遗留进程保持STOP并核PID/start身份，不靠进程名清理；当前root为Codex进程，Claude CLI是原生可执行文件，竞争检查适用本机，新机器重新核；两env同hash仅说明本机字节相同，不证明live凭据有效；签名失败只留下未推送本地提交，后续必须审定恢复；缓存容量/现存build产物在准入前只读登记，冷编译不自动延长。原“四修复轴”候选横幅及历史行号不改。

补存R1未逐行抄入快照的三行原文（2026-10-25旧规范，当前已被前滚段替代）：

```text
- 修复 checkout 的公开写集仅本计划、基于实际已发布上游版本的 `docs/development/plan/plan-status.md` 中本计划登记行、当前卡已登记测试文件、该卡实际版本面/工具链 lock；仅FIX-LTR-04额外允许 `src/internal/ai/capture/ingress.rs` 两个方法定义前各加一行 `#[cfg(any(test, debug_assertions))]`，不改其方法体或生产入口。与原工作区未发布 scoped-traces 治理提交做 ancestry 和逐文件 diff 守卫，禁止携带 `ed566b0`、其本地治理父链或原三份治理 WIP；plan-status 只在上游版本中新增/更新本计划行，禁止复制原工作区整份文件。本文是新的独立修复计划，不复制未发布的 scoped-traces 文本到发布分支。
- 原工作区两仓既有八份治理文件只登记事实/外部依赖，仍由原 MTR-01/LTR-01 与 Mega 后续既定 C/D 承载；其中原 scoped-traces 治理增量不属于四个修复分支的发布写集；plan-status 的新修复计划行是下述独立同步写集。新修复发布后，原工作区只通过原生前滚整合保留双方历史；若本文本地未跟踪副本与远端已发布版本碰撞，先保留快照、核同源与内容，再只处理 root 自建副本，不清理无关文件、不 reset/force。
所有 Cargo 门 source `.env.test` 和 `.env.live-test`，核非秘密 presence/控制项、原工具链与 Cargo 配置；缺文件或必要凭据停止相应门。延续隔离 HOME、已核 CARGO_BUILD_JOBS=1、umask0022，不复制用户凭据到仓库或日志。listing与JUnit按(target,case)精确核数、保留原ignored/重试及成功输出中的runtime skip；缺环境、缺 feature、early return不冒充所需行为通过。代码完成、自身review、提交、push、D逐项分别登记。
```
