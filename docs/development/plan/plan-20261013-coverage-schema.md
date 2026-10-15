# OpenCode coverage schema-v2 前置实施计划（2026-10-13）

> **2026-10-15 G09规范采纳事实（由本次签名提交采纳生效）：** 原OG24 `.2` source交接历史保留，当前`.3` profile仍须原SRC029/RUN029及原6AC8VER；本次仅采用既有任务的规划修订，父formal **59/71=83.10%**不变，source由旧62/71暂扣为 **61/71=85.92%**，71/26分母不变。COV当前source **13/26=50.00%**、formal **0/26**，COV22仍HOLD。指定dcc实际old-writer补核现已完成：实际1/1/0ignored、8 raw artifacts与旧reference同一、6 current literals与actual dcc同一，root独立复核收据SHA `31cd4e23b68633ce4f6efa348488067a9444c721d0e1423476f67b2303206315`；该有限固定fixture事实不等于whole-writer/native/source qualification。当前HOLD未闭原因精准为：旧full对04/05 dirty计划输入的适用边界补核、新soleClaude事实复核及root最终裁决尚待；不能仍把dcc采样写成UNRUN，也不提前增加COV22分子。原错误source14计入、R1/R2失败与R6准备期HOLD原因均保留为有日期历史。
>
> **实际验收范围：** 唯一Claude R4规划设计已实际 **PASS（0P0/P1/P2，3项nonblocking P3）**，REPORT SHA `d457f6770188d1fd517e42d597bcc7ac4e8a8218b8b9495210d6c59271eb0fee`、raw SHA `9b3c31fd370244cd82c46c0ac85177d6e179216f847c1b2e0baea1d0c416709b`，只覆盖exact R6三文档设计，不覆盖producer源码/构建/SDKgold/Source A。b644+exact R6独立2044输入已实际完成双env、无native/model opt-in的whole hook12/12、ARCH58/58、docs32/32及5个exact计划读者各1/1（共102 unique，5 exact为重复确认，均0ignored）、fmt和strict all-targets/all-features `-D warnings` exit0；RESULT SHA `b36e9ccce60da0369c8f7e587b867585efae9e1712a36725b51c23e794d94ef6`、strict raw SHA `bec4797042e15f172e1b60f60b88a891facee2792ad52476e77047df928cca8a`，2044前后SHA/bytes/kind/mode全同，冻结manifest SHA `5d191b8a4dcd0a5c17189375abab39546ec3e4052956de8ecad547b75f937bcf`。未跑full、模型/Session或producer；本事实delta本身的唯一Claude、相关reader QA、实际Libra diff与signed main/remote收据仍由root另行真实完成，不预填未知commitSHA/采纳UTC，不把R6 QA称作本新增事实字节的已验收结果。原Source A/三字段UNPROVEN、复杂OpenCode验收不上CI、GC03/GC08、全部任务判据及家族C/D门不变。

> **历史source计数更正（2026-10-15 R6准备期审计快照）：** signed18b旧收据曾将COV22计入source14/26；其GC-COV2-03指定dcc完整old-writer/gold同一性尚待真实补核，root已暂停该卡最终source资格，当前COV source **13/26=50.00%**、formal **0/26**。这是资格审计更正，不撤销旧signedsource事实或变更gold/GC03/AC/VER；COV08可继续focused，补核闭合前不full/sign。父formal **59/71=83.10%**、source **62/71**保持；本G09实际采用当前OG24.3修订时才source→61/71。后续仅真实dcc同一性收据和root裁决后才能事实更新恢复COV22 source，当前不预授。


> **2026-10-15 G09：OG24 incoming冻结与阶段写权当前规范。** COV incoming读取已满足的`.2` signed snapshot `6685b82a938919a8e51f46b7c51cc305b40189a2`，不再依赖OG24 live生命周期；DEP13 source-ready和COVREL自身I/R源冻结后，才可按具名sharedgrant→OG24.3signedreturn→RELcloseout写三plan事实，或选择先RELcomplete再grant的互斥route。原26卡/逐卡AC/VER/I/R不改，COV当前source13/26、formal0/26；父formal59/71，本次OG24profile规范采纳生效后source61/71。所有旧日期source计数/旧48拒绝为历史，不授SourceA/native。详见文末本G09的精确DEP/共享路径与返还规则。


> **COV2-22 采用候选条件状态（2026-10-15，未预授签推）：** 原 8 AC/6 VER 的本机机制、21 unique exact、相关质量门与双 env full 已实际执行，完整 run 为 10139/10139、2 slow/2 leaky/23 skipped/exit0，原失败与警示保留。本次采用的新增两计划事实仍须 root 最终单 Claude/裁决及本次实际 signed+DCO main 交付、remote 同一性；仅在该 source qualification 完成后 COV source 为 **14/26=53.85%**，此前仍为 **13/26=50.00%**，formal **0/26** 不变。父 formal **59/71=83.10%**、source **62/71=87.32%** 及下段最新 049 十四门／四卡 done/complete 事实全部保留；本候选不写未知提交 SHA，不把记账后的完整 2043 字节称为 full 同一输入。

> **当前049实际C/D与四卡正式收口（2026-10-14 22:28:49 UTC）：** USER-TEMP-REL-049 / 0.30.49 的实际C/D已由root按完整收据 `dad01d952f7ce629354118264a4926b048693bf851258e4cd14beb9853727f8c` 逐项裁决关闭；release source/tag peel为 `04617cd2a3ffbf3a3e68ce4ddfecf94b78bc6574`，release run `38639329379` attempt `2` 的8个job全部success，当前SHA CI/CodeQL、四平台签名manifest/资产及CDN、pinned Homebrew、网站HTTP200/visible六段、用户指定curl home安装、同正式Darwin资产原子替换root目标及installed cat-file/既有BASE1345B读回均有hash绑定的实际证据。 仅四张父卡由locally-accepted转done/complete，父formal **59/71=83.10%**（原55增加4），source仍 **62/71=87.32%**；COV source **13/26=50.00%**、formal **0/26**，71/26分母不变。COV2-25/COV2-REL-01/DEP-OG-13、OG24..26的FAST-OG-04、Source A及native三字段未因此关闭；Source A UNPROVEN，历史unsupported-state import020仍FAIL/默认拒绝/计划外deferred。真实provider idle.error正例仍未观察，本卡typed机制按原7AC/3VER验收，不把成功Session或synthetic称真实错误/native来源。既有FAIL/cancelled/skip、Mega2与cat-file延期观察保留。 下方原日期状态保留为历史；本次仅例行事实记账，不改AC/VER/I/R/DAG或扩大权限。

> **历史BASE源码就绪交接（2026-10-14 20:10:13 UTC；当时正式049/C-D仍待）：** `OG-UPGRADE-029-01`已按8AC/2VER、相关门/fmt/strict、最终单Claude R2 PASS及实际双env全量完成A/B，现`in-progress / locally-accepted`、source-qualified；signed+DCO main `1e2177e11029e28e3c81afbe6ec9f1609cdde7da`与cf `93b10519e1943ea3eea7c943e816418e92b1ca0b`均Push exit0、actual remote同SHA，12root+1site reviewed postimage/mode逐项相同。最终full实际exit0：10133/10133 passed（2slow均最终PASS，23skipped不计PASS，0retry/0flaky），raw `24febb2eba3d2aad09c6f516988e06e9ae127ad8dd96b362fbda4ea45671fdb2`；2043项前后SHA/mode同一。父source **62/71=87.32%**，formal仍 **55/71=77.46%**；COVsource **13/26=50.00%**、formal **0/26**不变。网站已实际deploy0/HTTP200、六段visible text对拍通过；正式0.30.49的最终SHA安全CI/版本guard/单审发布delta/8releasejob/四平台签名manifest/CDN/Homebrew/指定curl安装及installed smoke仍pending；Source A UNPROVEN、历史import020 FAIL/default reject/deferred不变。下方旧开工、未终态full及61/71均保留历史，不倒改失败或未验收门。

> **历史BASE例行验收事实（2026-10-14 19:08:52 UTC；最终门尚待）：** `OG-UPGRADE-029-01`的8AC/2VER已有本机实际证据，Lifecycle仍in-progress / Acceptance空、未source-qualified。新owned自然自动采集普通读回1345B、prompt1、extraction.model=`glm-5.3-flash`（hook顶层仍`unknown`）；input/output/reasoning/cache_read/cache_write为7678/29/35/0/0、total7742；files仅空集合控制。1次模型、0次manual hook/import/repair，raw=false/truncated=false，旧正式0.30.48读回同bytes。官方29 SHA `7f46ba1dad86b730903d9adf54015bb4e706fc1ab6a247affc5a635cd8a663f3`/186170000B；interactive PID50453已退出，collector是同sealed verified binary的独立Required-Seatbelt租约、PID未存档，不是同PID/endpoint证明。645源码输入实际匹配、12旧protected输入和双MANIFEST冻结前缀同一；focused/质量12项exit0及网站typecheck/build exit0，仅证明各自范围。strict已exit0，final full/最终sole-Claude、root与site签推、安全CI/049发布/C-D待；父source61/71=85.92%、formal55/71=77.46%，COV source13/26=50.00%/formal0/26均不增。历史import020仍FAIL/deferred/default reject，Source A UNPROVEN。下列旧开工、失败、UNRUN与1369B均保留为历史，不由本次对拍追认。

> **历史BASE开工与G09实际采纳（2026-10-14 18:27:15 UTC）：** G09前瞻范围修订及仅Unix测试fixture cfg前滚已由signed+DCO main `84bbde7bd6a5f12dee2477755d0a955798518413`实际Push（exit0），remote同SHA；五路径PGP header/DCO和逐blob与最终sole-Claude R2 manifest同一。R2实际PASS（0P0/P1/newP2，六P3经root裁决接受），raw `4ab4911f9253eb659bb03cf6b49fb4479e92a74145ea1ee8f982e1a12df05190`；不是本轮新BASE实现的验收。网站前驱仍为signed cf `a80d3ff9d2d194b739da6148dde3297d325f1a7e`。`OG-UPGRADE-029-01`现in-progress / Acceptance空，8AC/2VER全unchecked、未source-qualified；父source61/71=85.92%、formal55/71=77.46%，COV source13/26=50.00%/formal0/26不增。GLM历史import020计划外延期、仍FAIL/默认拒绝，不记PASS。84bb安全CI仍待实际结论，旧d842 Windows7项dead_code FAIL、首ARCH57/58 FAIL均保留；其它full/正式049/C-D/Source A门未闭。下列旧草案、旧12/26及旧closing只为其原时点历史，当前事实以本条和末尾实际收据为准。


> **历史源码交接（2026-10-14 18:03:50 UTC）：** live修复已signed main `d8422534b07724349591d53aa445ce50c38453f5` / cf `a80d3ff9d2d194b739da6148dde3297d325f1a7e` 并Push、remote同SHA；source **61/71=85.92%**、formal **55/71=77.46%**，COV source13/26/formal0/26。d842 Windows测试编译FAIL已定位仅Unix fixture cfg遗漏；本次窄前滚的本机门/Claude/签推及新SHA Windows仍待，不影响已安装macOS临时版。下述G09草案仅实际sole-Claude PASS和signed采纳后生效，不预授BASE/049通过。


> **历史G09范围修订草案（2026-10-14 17:58:11 UTC，仅实际sole-Claude PASS＋signed main采纳后生效）：** `DEFER-OG29-PARTIAL-IMPORT`把GLM unsupported-state历史import020的helper认证胶囊与direct-only持久化两轴移出本计划和USER-TEMP-REL-049；原FAIL/UNRUN保留且默认拒绝，不记import PASS。BASE仍为8AC/2VER、1生产文件的official29 namespace＋真实自然自动capture/checkpoint/普通export基线。DAG与71卡分母、COV26卡分母不变；本修订不增完成分子，source保持61/71=85.92%、formal55/71=77.46%，COV source13/26=50.00%/formal0/26；BASE及049其它实测门仍待逐项关闭。此前带时间戳的“当前”段落保留为当时历史，涉及historical import为BASE/049硬前驱的旧规范由本G09明确替代，不能倒改旧收据。


> **FIX-OG-PARTIAL-LIVE-01 本机验收与临时交付（2026-10-14 17:44:21 UTC）：** 五生产文件的 storage-only ordinary partial live 修复已完成本机 A/B：六项 new exact 各1/1；最终live15/15、architecture58/58、docs31/31、相关reasoning21（1 ignored）/OpenCode32/snapshot26/checkpoint34/pending31及14项既有ER04 exact通过。nightly fmt/diff/strict all-target/all-feature实际0；sole Claude code R1 PASS（0P0/P1、4P2/7P3，raw `16a23ab014a4304487c165f92670d416cf551870b6f4bd555b6515e856a3831e`）及R2 PASS（0P0/P1/newP2，raw `63791e1e9218f05377e0981692f6da94b9e03f6f0cb3f5014668a86b7b0acdf0`）。R1四P2通过终态/escaped-key机制与准确docs收口，旧P3及R2新P3保留，不声称历次零P2。三计划本次最终单审与signed source Push尚待；此前source仍 **60/71=84.51%**，本卡与网站owner实际签推后为 **61/71=85.92%**；formal仍 **55/71=77.46%**。COV2 source13/26=50.00%、formal0/26不变。


> **当前live partial修复 G09（2026-10-14 16:15:36 UTC，本三文档实际Claude PASS＋signed main采纳后生效）：** 用户明确要求修复刚刚的自动捕获问题后构建最新本机版本再测试。新增仅一张完整卡 `FIX-OG-PARTIAL-LIVE-01`，父计划70→71；IDLE已实际签推main `95ab8f35b3af7e8a5736c955212415ca93eebeb3` / cf `bc58acc2dbe057dc039fedfc3f3d432b9b4dd332`、remote各同SHA，源码已交接。采纳前实际source60/70=85.71%、formal55/70=78.57%；采纳后source **60/71=84.51%**、formal **55/71=77.46%**，新卡UNRUN不预计完成。COV仍26卡/source13/26=50.00%/formal0/26。原70及其较早分子均保留历史；import020必要后续目前只有未采纳private骨架，不算新增Task、不伪称BASE真实import或049门已关闭。root唯一共享源码写者并发起sole-Claude，私有prototype不是实施GO。

> **当前 FIX 源码交接与 IDLE 开工（2026-10-14 15:47:50 UTC）：** FIX-OG-NATIVE-IMPORT-01 已完成实际 A/B、fmt/diff/strict 与唯一 Claude Code PASS，signed+DCO main `6524153d97ab2f5c02f7875abd5f3ed3d343cea8` 和网站cf `ddfee87c66d35cf586ca4534c0d427a7254085d0` 已 Push、各与 remote 同 SHA。父计划当前源码就绪 **59/70=84.29%**，正式 **55/70=78.57%**；COV2 当前源码就绪 **13/26=50.00%**、正式 **0/26**。OG-IDLE-ERROR-029-01 当前仅本机三个新 exact 各1/1及完整模块31/31通过，已应用source/docs的fmt/strict与相关模块门已通过，三计划最后同步、单Claude及源码/网站signed交接仍待，**不预增60/70**。下一源码卡 OG-UPGRADE-029-01 在新独立Libra原生仓执行普通29真实采集验收，当前UNRUN；本三卡/临时049 C/D、最终新树full和Source A均未因此放行。下方旧规划/开工数值仅为当时历史快照。

> **历史普通 OpenCode 2.0.29 / G-09 三卡规划快照（2026-10-14 14:30:58 UTC，本次三文档最终单Claude和signed main采纳后生效）：** 用户要求当前工作升级2.0.29。本机global与官方隔离npm platform binary已实测同为 `opencode v2.0.29`、186170000 bytes、SHA `7f46ba1dad86b730903d9adf54015bb4e706fc1ab6a247affc5a635cd8a663f3`；官方tag/commit为 `v2.0.29`/`042d0832f5f3a73eaa039a58a7d565776f6d4290`。新增三独立卡 `FIX-OG-NATIVE-IMPORT-01`→`OG-IDLE-ERROR-029-01`→`OG-UPGRADE-029-01`；旧拟UPGRADE的idle.error轴拆出IDLE，实际内容验收只在BASE。父计划67→70卡、正式55/70=78.57%、源码就绪58/70=82.86%；本三卡均pending/空且未计新增完成。COV2仍26卡，已签旧收据保留；COV25冻结full10110的结果只证明其原冻结SHA，不是含5c及三卡新源码验收。全局27/旧26与private.1/.2仅属当时观察/冻结身份，当前普通29 source、fixture namespace和本机lane独立；native.3登记及真实Source A仍独立，不阻普通29/049；151个机械案例不构成上游OpenCode全量source/producer验收，whole-producer check/full仍UNRUN。任何旧段落保留的67卡或55/67均为本次采纳前历史口径，现行统计只采用70卡；原历史数值和失败/取消不倒改。

**状态：** 实施中；COV2-24 按25卡历史口径交付源码就绪 **12/25=48.00%**；本次 G-09 三文档采纳新增 COV2-25，现为 **26 卡（23 implementation+2 audit+1 release）**、正式 **0/26**、源码就绪 **13/26=50.00%**（COV25实际signed Push/source-qualified；原12/26是Push前历史）。COV2-24 最终 5/5 AC、3/3 VER 与七项 mandatory ER-04 精确 1/1（十 selector；focused index `39306128fdc72bbb2ae9bca583f186f2d8bfda693b68fb1a38795b815c9e31d5`），nightly fmt/strict Clippy/diff exit0（quality index `d0c676eb165802886f6f700d869d813db610a0fc7d09bf5e9759c452b71e979b`），双 env T-5 `NEXTEST_TEST_THREADS=4` 10103/10103 passed、23 skipped、exit0（raw `2908fff258247c857ae857a4759fcde9b925c212cc5912fa373ece16b5373029`）；唯一 Claude Code PASS、P0/P1/P2=0、P3=3（raw `a6d467bcdc33c42fff3c45f55681ebbaefa213d96cfd59a8b901c5d3dff942f7`）；八路径 0644 manifest `cd43b9af3980c8f3e90ce5751df7855ebaf1f604e2803130f0d32d9556b46e98`；signed+DCO main `c56de683acf71d7327436eef7ef9185d97dcff27` 与远端同 SHA。两轮失败 full 保留为失败历史；COV2-24 交接时安全 CI 仍在运行；其后 `c56de683`/前次 `5b0e7fc1` 的未完成旧 Actions 按用户新 commit 策略取消，cancelled 不记 D 组通过。 父计划当前source **61/71=85.92%**、正式 **55/71=77.46%**（BASE仍in-progress/Acceptance空，其新8AC/2VER实测不预增分子）；22 个最终行为源码 macOS Session 案例、家族 C/D 与 native Source A 未闭合。原19/23/25卡历史分母与失败收据保持原时点，不倒改。


## 文档职责

本文解决 `DEFER-OG25-COVERAGE-V2`：现有 coverage-v1 不能把 OpenCode 可见 reasoning 当第五种生产 `SemanticRecord::Reasoning`。目标是在不重写 v1 历史字节的条件下，以 schema-v2 完成 writer/reader/digest/恢复、live/import、隐私及兼容交付，并在稳定 patch 发布后把已审源码交给父计划 OG-27。本文只规划任务，不宣称已实现、已通过模型或 native 来源证明。

### 适用范围

- Libra Repository 数据库中的 versioned coverage claim/identity map、checkpoint marker、pending/replay、import、graph/doctor。
- OpenCode visible-only reasoning 的 typed v2 路由；proof/state-bearing 候选保持 partial 与零明文 sink；真实 native 来源单独由父计划 Source A 判定。
- 23 张 implementation、2 张 audit、1 张 release（26 张；原采纳 19 张、已签 G-09 的 23 张和前次已采纳 25 张均为历史口径）；macOS 本机 Session 验收，必要安全 CI 与四平台稳定发布。

### 非目标

- 不修改 `sql/**`、`src/cli.rs`、`history.rs`、native issuer/locator admission，也不修改冻结 `tests/fixtures/agent_transcripts/reasoning/contract.snap` 或 `pending_v1_decoder.rs`。
- 不把 synthetic/fake exporter/普通可读 text 当作真实 `signature`、`redactedData` 或 `reasoningEncryptedContent` 的 Source A；`reasoningEncryptedContent` 正例仍待父计划后续真实路径。
- Linux 功能验收由用户在 Linux 机器后验；本计划仍要求必要安全 CI 和四平台发布构建，不以本机功能范围取消发布质量门。
- 父计划 67 卡分母不变；本次 G-09 在 COV2-24 后拆出 COV2-25，独立计划当前 26 卡单独计数；25 卡保留为本次规划签名前的历史口径。

### 成功定义

- 带 typed Reasoning 的生产 turn 只走 coverage schema v2；不带该 record 的 v1 turn 原字节/MAC/digest 不变，旧读端对 v2 非破坏性拒绝。
- 同一逻辑 key 在 DB claim 与纯 wire codec 可跨 v1/v2 共存；在 OG-29 版本化 native alias 前，native pending 跨版本裸 key 由 COV2-23 fail closed。同版本重复、失效 owner/fence/revision、unknown version、损坏私有 map 均 fail closed，无假成功/假 ack/普通明文泄漏。
- 22 个继承 ignored OpenCode Session 用例（旧21例加 OG-26 新增1例）与本计划新增 ignored Session 用例在最终行为源码的 macOS 手工执行中非跳过通过；真实 visible-only 观测有脱敏记录，不冒称 native 来源。
- 全部 26 卡达到 `Lifecycle=done` 且 `Acceptance=complete`，唯一 patch 发布点完成精确源码全量 Nextest、单 Claude Code PASS、安全 CI、四平台、网站/安装及指定 curl 安装，随后才标父计划 `DEP-OG-13=complete`。

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
| GAP-COV2-02 | 单版本 identity map/read、marker、pending/replay/import 仍有 v1 常量/域 | 双版本同键与故障恢复不能安全成立 | COV2-02/07/14/03/19/20/21/23/04/24/25/22/08/05/09 |
| GAP-COV2-05 | 原始 outer-v2 成员尚无由仓库 key owner 颁发的精确成员完整性证明；MAC/typed DTO 不能替代原始 ordered member 证明 | 错误 loader/replay 可能接受重组后的成员或错配 tuple | COV2-25 独立封闭 verifier，COV2-22 真实 loader 消费、COV2-08 late replay 重验；仍不授 native Source A |
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
| 粒度/恢复 | 23 个单轴实现、两个不写代码的 audit、唯一 release；无 EX 豁免；COV2-25 8/8 AC、4/8 VER 已按 signed COV2-24 API 对齐，开工时仍按 ER-02 复核 | 下方 G 审计表；任一实际 AC/VER 超限先按 G-03 拆卡 |
| 依赖/写集 | 全链串行、OG-26 后排他单写者；网站另有已脏 checkout | DEP-COV2-OG26 / SITE；COV2-09/11 仅隔离 `cf` owner 写 |
| 功能/数据流 | marker、map、pending、import 都须按 version/domain 认证并原子恢复 | COV2-02..09、COV2-14、19/20/21/23/04/24/25/22；故障矩阵 |
| 安全/隐私 | fake exporter 只证明管道；候选状态/证明材料不入明文 | COV2-06/10/12/15/18；Source A 独立 |
| 测试/可靠性 | 22 个继承 ignored cases（原21+OG-26新1）全部需 final-source 本机非 skip PASS | COV2-16/17；无本机 Session CI |
| 性能/容量 | 4096 条/1 MiB 对 v1/v2 共享，不倍增预算 | COV2-14 aggregate cap 回归 |
| 发布/后验 | 单一 patch 点；本机 macOS 功能与四平台发布区分 | COV2-REL-01；Linux 用户后验 |

### 修订历史

| 时间 | 触发 | 变更内容 | 原卡 → 新卡 | 受影响引用 |
|---|---|---|---|---|
| 2026-10-15（由本次签名提交采纳生效；实际signed SHA/UTC由交付收据绑定） | G09 OG24 `.3`同轴版本维护 | 原6AC8VER仅AC2/AC6澄清；exact19rootI/1external、attest只读；OG27/OG05 current SRC029/RUN029版本与实测size绑定；COV incoming固定signed6685 `.2` | 无新增Task；本次profile采纳parent source62→61，formal59/71、71/26不变；COV13/26 hold仅为本G09采纳审计快照，dcc实际补核已闭、旧full输入边界/新事实单审与最终裁决仍待 | OG24/27/05、DEP12/13当前与历史适用范围、canonical五stage/实施顺序/DAG/共享slot及逐signed非OG24重基abort账本、COVREL协调/父REL71镜像、FAST04成员正逆序、测试矩阵/追溯/里程碑/风险/三计划状态；原判据和旧证据保留 |
| 2026-10-14 22:28:49 UTC | USER-TEMP-REL-049实际C/D例行镜像 | 父四卡正式59/71、source62/71；当前临时release门关闭 | 无新增卡；COV26/source13/formal0不变 | incoming DEP-COV2-029-BASE仅同步父状态；COV2-REL-01/DEP13/nativeSourceA不关闭 |
| 2026-10-13 09:15:00 UTC | R4 草案转模板 v2.14 | 将 16+2+1 拆分、两阶段 `DEP-OG-13`、macOS 21-case 手工审计、单一 patch 发布点写入独立计划；将 R4 非法数字后缀改为合规文件名 | 草案 COV2-01..18 → 同 ID | 依赖表、REL、矩阵、父计划修订提案 |
| 2026-10-13 10:59 UTC（采纳前预核） | OG-26 待签工作树、DEP-OG-12 独立 r2 | 继承本机 ignored Session 清单从旧21例增至22例；COV2-16 从11例/12 AC/4 VER 调整为12例/13 AC/5 VER。独立 r2 只确认 selected `.2` binary-to-source 有限子门并保留5个 P2 后续，旧 clean r4/0.30.48 仅作历史时点，不推断 Source A | COV2-16 同 ID；19卡分母不变 | 成功定义、事实/缺口、GC-08/09、COV2-12/16、REL-01、矩阵/风险/完成判据；待 OG-26 signed main SHA 与最终 Claude 审查 |
| 2026-10-13 11:16:08 UTC | OG-26 signed main handoff | 实际 `HEAD=origin/main=734b6f03884d3aa38400acbbfd446b557e642d94`；21→22 继承案例计数在签名源码上成立；仅采纳 COV2 计划，未实施任何 COV2 卡，DEP-OG-12/Source A 仍 NO-GO | COV2-01..18 与 COV2-REL-01 保持 0/19（历史） | 逐路径 signed SHA/mode 清单及三路径采纳 patch 在隔离提案中，最终采纳字节仍需单 Claude PASS |
| 2026-10-13 15:52:39 UTC（隔离候选，尚未采纳） | COV2-03 G-03 独立谓词超限及 COV2-04 真实持久化前置 | 原 COV2-03 收窄为 versioned marker/ref 围栏；新增 native identity-v2 companion 与 pending versioned verify/retain 事务 API，COV2-04 在真实持久化 artifact 上验 frozen 旧读端；原始 19 卡采纳分母改为候选当前 21 卡，历史 0/19/1/19/2/19/3/19 保持原时点事实 | COV2-03 → COV2-03/19/20 | 依赖表、REL 成员与逆序、DAG、任务审计表、04/08/REL 卡、测试矩阵、追溯表、里程碑、风险表、状态索引；候选未验收、未签名 |
| 2026-10-13 17:02:33 UTC（历史隔离草案，未采纳） | COV2-03 r7 静态差异与两处独立事务/持久化行为 | 原 COV2-03→19/20；COV2-20 验领/保留拆为只读 verify→COV2-21 retain；原 COV2-04 codec/真实 artifact 拆为 codec/seal→COV2-22 persist/旧读端/故障。新增严格边 20→21→04→22→08；20 implementation+2 audit+1 release，23 卡分母。COV2-03 AC8 的冻结 v1 bytes 移至 mandatory ER-04，保留原 golden 1/1；父计划 COV2 单写者表须显式 I 同步。此行为待 root 在 COV2-14 signed main 后重基、最终单 Claude 审查和签名采纳，所有新 AC/VER/CI 未运行。 | COV2-20→21；COV2-04→22；COV2-03→19/20 保留 | 依赖、REL、粒度、phase、测试/追溯/风险/容量、plan-status 与父计划单写者表 |
| 2026-10-13 17:43:33 UTC | COV2 G-09 23卡规范性修订及COV2-14 signed handoff | 保留原19卡历史收据；COV2-03拆出19/20/21、COV2-04拆出22，按23卡DAG/REL/G-03/写集/矩阵同步。COV2-14 signed main `4558af64c09e63c4716fa68595776a8044f66cc7`，8 AC/7 VER、双env full 10069/10069 exit0/raw `2849dcf7e2105ed1da5e5d3b0333374e2590ac909c1f478ef8ba68966e86aec4`、唯一Claude PASS/raw `2d1e5fb7425abc9aa2802db241213811dfcd6c891410e9853e554c5fcb806966`；COV2-14 交付及 G-09 签署时0/23 formal、5/23=21.74% source-ready；当时 COV2-03新增VER5、旧catalog receipt ER-04回归与COV2-19..22新AC/VER均UNRUN | COV2-03→19/20/21，COV2-04→22；父67卡不变 | 三文件精确I和父单写者表；规划协调另由最终字节单Claude与signed main交付，Source A/C-D仍待 |
| 2026-10-13 20:58:21 UTC | COV2-04 原 8/8 AC 与同源 native pending-v2 借用及内存 seal 接线独立 | 在 COV2-21→COV2-04 间新增 COV2-23：从同一 NativeProven payload 和 COV2-19 companion 生成 checked borrow；在 COV2-04→COV2-22 间新增 COV2-24：消费该 borrow 与 COV2-04 有界 codec，在 `pending_payload.rs` 构造版本化 inner projection、在 `pending.rs` 构造不持久化的 sealed-v2 artifact。COV2-04 收窄为纯 codec，COV2-22 AC8 改为已封闭 artifact 在 persist 入口的独立二次防御；裸 key 跨版本 native alias 仍在 OG-29 前 fail closed。原 19/23 卡与 COV2-03 逐卡收据不回写；采用时正式 0/25、源码就绪 6/25=24.00%，父计划仍 55/67。COV2-23/24 全 AC/VER UNRUN，Source A 未证明，家族 C/D 未闭合。 | COV2-04 → COV2-23/04/24；严格边 21→23→04→24→22 | 三文件精确 I：依赖/REL 成员与逆序/DAG/粒度/任务卡/矩阵/追溯/里程碑/风险、父计划 DEP-OG-13 与外部单写者；COV2-19 开工前核本协调最终字节单 Claude PASS raw SHA、独立 signed+DCO main SHA 与 remote 同一性，并在该卡首个状态同步登记收据 |
| 2026-10-14 00:18:49 UTC | COV2-19 exact-head Windows `-D warnings` 编译失败，COV2-20 尚未最终验收 | COV2-20 按 G-10/I 追加 `capture/checkpoint.rs`，只做八处预留 API 的平台限定 cfg 和一个 Unix-only 测试 helper guard；原 `coverage_gate.rs` 只读验领轴、7 AC/3 VER、单一 landing 与 25 卡分母均不变，生产文件从1增至2。旧失败不计网络；新提交 Windows job 才是该修复验收。 | COV2-20 同 ID；无新卡 | 本卡 I/Files likely touched、粒度表、ER-06a 文档影响判定、focused/strict/full/Claude 与 plan-status；父计划 Source A/C-D 不变 |
| 2026-10-14 11:13:55 UTC | 用户恢复原计划及真实最新上游基线 | 用户明确继续原计划；本次 G-09 preimage 为真实 signed main `35f8b15db38037b9af1fb696afc2bcad295500a8`（含 cat-file 修复），三文档原字节与 HEAD/remote 已核同。 | 恢复24→25→22→08及后续DAG；父67不变；本次规划签名采纳后26卡/source-ready12/26/formal0/26 | 最终字节单审与signed Push当前待；COV2-25 AC/VER及含上游修复的最终full未执行，Source A/C-D未闭合 |
| 2026-10-14 11:13:55 UTC | COV2-24 signed main 后 COV2-22 缺 key-owner 原始 outer-v2 成员认证 | **本次三文档签名 main 提交采纳生效：** COV2-22 保留真实持久化/旧读端/loader 主轴，新增 COV2-25 精确成员 key-owner verifier；严格边 24→25→22→08。COV2-22 的 `pending_payload.rs` 仅承接真实 v2 loader 的认证后 redacted rehydrate/typed deferred，COV2-08 同文件仅承接 late replay 的 v2 deferred/original bytes+MAC 保留；各自落入既有 AC/VER，生产文件数分别 1→2 与 5→6，不新增行为轴。COV2-24 已由 signed main `c56de683acf71d7327436eef7ef9185d97dcff27`、八路径 manifest `cd43b9af3980c8f3e90ce5751df7855ebaf1f604e2803130f0d32d9556b46e98`、T-5 full raw `2908fff258247c857ae857a4759fcde9b925c212cc5912fa373ece16b5373029` 与唯一 Claude PASS raw `a6d467bcdc33c42fff3c45f55681ebbaefa213d96cfd59a8b901c5d3dff942f7` 交接；本三文档最终字节单 Claude 与独立 signed+DCO main/remote 收据在 COV2-25 开工前核并补记；COV2-25 AC/VER 全 UNRUN，Source A/C-D 未闭合。旧25卡作为历史分母保留，本次采纳26卡。 | COV2-22→COV2-22/25；父67不变 | 三文档精确协调 I、DEP/DAG/REL/粒度/任务/矩阵/追溯/风险、父计划外部单写者；COV2-25 开工前核规划提交 receipt |
| 2026-10-14 02:55 UTC | COV2-23 首轮全量检出两个架构闭集守卫 | `prepare_pending_native_versioned_seal` 是 `from_validated_pending_context` 的合法新增调用方；G-09/I 仅给 COV2-23 精确追加既有 `tests/compat/agent_architecture_guard.rs`，在两个闭集守卫各登记该调用方，不放宽其它入口。首轮 full 10093/10095、2 failed 不计通过；窄范围 Claude 前置复审 PASS 后守卫各1/1、fmt/strict再次通过，双 env full 与最终单审仍须重跑。 | COV2-23 同 ID；25 卡分母和 8 AC/4 VER 不变 | COV2-23 I/Files likely touched、卡内失败/修复收据、plan-status；最终审查与 signed main 后才 source-ready 增计。 |
| 2026-10-14 06:22:08 UTC | COV2-24 首轮 T-5 全量检出一处架构闭集守卫遗漏 | `seal_bound_versioned_reasoning` 是本卡唯一新增的合法 `seal_pending` 生产调用点；按 G-09/I 精确追加既有 `tests/compat/agent_architecture_guard.rs`，只在 `reasoning_pending_seal_borrows_only_bound_native_witness` 的闭集增加该函数，不放宽其它入口。首轮 full 10102/10103 passed、1 failed、23 skipped、exit1，不能记 PASS；COV2-24 三项 VER、六项原继承、质量门在该守卫修订前通过，最终须追加守卫 exact 1/1，并重跑 focused、fmt/strict/full、唯一 Claude。 | COV2-24 同 ID；25 卡分母和 5 AC/3 VER、两个生产文件不变 | 本卡 I/Files likely touched 增一既有测试文件，mandatory ER-04 增一精确 selector，计划/status 记录失败及修订；父计划 Source A/C-D 不变。 |
| 2026-10-14 14:30:58 UTC | ordinary29三卡G09 R1-R3镜像及证据修复 | 父FIX→IDLE→OG-UPGRADE-029-01接入COV25→22；本COV依赖登记/DAG/发布窗口及05/06/11/15/18/22 Dependencies/Granularity/审计表同改；所有Icore后继精确重基，网站owner串行；原COV AC/VER及26卡分母不变 | 父70卡/正式55；COV26卡/正式0，source-ready不预增 | 本COV实际更新为DEP/DAG/REL/六卡依赖/审计及本日期行；新增三卡测试矩阵/追溯/里程碑/风险在父plan-20260902维护，本COV消费DEP-COV2-029-BASE镜像，不声称另改未动的COV各表；root最终单Claude/signed采纳前新门UNRUN |
| 2026-10-14 15:47:50 UTC | IDLE例行三计划同步/FIX源交接 | FIX signed6524153/cf ddfee87及raw ac889106实际交接；IDLE3exact各1/1/模块31/31机制，最终quality/Claude/signed pending；COV25生命周期旧标签与签前时态明确历史 | 父source59/70、formal55/70；COVsource13/26、formal0/26；不增卡、不预授IDLE完成 | 本COV实际只改顶部现行统计、DEP-COV2-029-BASE当前源边、COV25历史状态卫生、本日期行与最新三计划同源收据；BASE实际验收改新独立repo，原AC/VER及写集不扩张；C/D、Source A、最终49full仍待 |
| 2026-10-14 16:15:36 UTC | 单轴ordinary partial live新增G09 | 插入FIX-OG-PARTIAL-LIVE-01，parent70→71；actualIDLE95ab/cfbc58，private import未采纳 | 父source60/71/formal55/71；COV仍source13/26/formal0/26 | 本COV实际改top当前口径/DEP-COV2-029-BASE/REL窗口/此历史行与尾部实施镜像；原26卡AC/VER不变，最终049full含新live，SourceA/native不授 |

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
- **GC-COV2-03 兼容：** v1 fixture、pending envelope/MAC 与旧 struct literal 均保持；COV2-03/04/05/07/14/19/20/21/23/24/25/22 的 v1 goldens 均从冻结基线 `dcc468bd31c88a7e8d8c0d2957d6e80b65908990` 的原始字节取值，绝不由修改后代码重生成预期值。旧读端只需非破坏性拒绝 v2。受影响的命令 EN/zh、开发文档、网站 `cf`、`COMPATIBILITY.md` 同卡交付。
- **GC-COV2-04 数据库角色：** 当前无 DDL。受影响角色仅 Repository；GlobalConfig/SystemConfig/Derived 的 schema writer、receipt 和表均不变。若需 migration/bootstrap/top-up/compat 变更，按 GC-13 新增按角色隔离与旧/新 binary 门，修订写集/粒度后才能实施。
- **GC-COV2-05 资源：** v1/v2 两个 map 的条数和编码总字节共享原 4096/1 MiB；解析前限额、有界认证、事务 CAS 与 owner/fence/revision 不得削弱。无新增无界扫描/网络调用。
- **GC-COV2-06 证据：** 测试计数、SHA、run ID、评审结论和发布/站点证据写回本计划和 `docs/development/plan/plan-status.md`，不在仓库新建证据目录/文件。禁止泄露模型 transcript、密钥、token、私有路径或候选明文。终止链末端纯文档记账提交的 D 组结果按模板 v2.14 预登记：由下一次 `docs/development/plan/plan-status.md` 例行同步补记 SHA、run ID、结果、UTC；若至次日无例行同步，做一次明示状态维护提交，不建立无限自引用链。
- **GC-COV2-07 用户单评审授权：** 本对话用户原文“把所有评审改为单评审，只进行 Claude Code 评审，PASS 即可通过”；`plan-status.md:3` 亦记录现行“单 Claude Code PASS”。因此 ER-05 的 Codex 评审循环在本计划由用户较高优先级指令取代；每卡仍必须在最终源码字节上获得一名独立 Claude Code 审查者明确 PASS，处理其所有 material findings，再签名交付。草案的暂行审查不能代替 OG-26 后采纳到 main 的最终字节审查。
- **GC-COV2-08 被动随版源码：** COV2 从 OG-26 签名 main 之后开工，最终稳定 patch 会携带上次稳定 tag 至冻结 main 的**全部**祖先提交，不仅 OG-24/25/26。发布前执行 `libra log <last-stable-tag>..<freeze-main-sha>`，逐提交登记 SHA、归属计划/卡、已审与 C/D 状态；COV2 家族子卡的 C/D 由本次唯一发布点结清。OG-24/25/26 是明确登记的父计划乘载例：其 A/B、单 Claude、signed SHA/hash/mode 与本次安全 CI/安装门必须核实，但 `REL-FAST-OG-04` C/D 是本次发布的**下游**，不得反向要求其先 complete 制造循环；本发布不替它结清 ledger。任何其他计划乘载提交有应先满足的独立安全/验收门则先满足或阻断冻结，不能由 COV2 测试代替。已审的 `src/internal/ai/observed_agents/native_origin/approved_build.rs` 私有 `.2` 许可随新版二进制进入安装面，可能改变 **2026-10-13 当时已安装 0.30.48** 的 `LBR-AGENT-005` 拒绝；必须实装 COV2-REL stable patch 并重验原 issuer epoch0，不能从源码推断注册 GO。OG-27 源码不得混入冻结树。
- **GC-COV2-09 Source A 证据边界：** 旧 clean r4 与 2026-10-13 10:49 UTC 独立 r2 均只处理 selected `.2` binary-to-source；r2 的 raw Bun binary 与 selected 不同，285 个受限 nonce/hash 重建与 45,455 个 code-signature page hash 重算后才逐字节相等，不宣称 raw build determinism、notarization、独立供应链或真实 endpoint transport。独立 r2 Claude 对此有限子门 PASS，但5个 P2 后续须按 DEP-OG-12 owner 在相应强声明前处理：执行时脚本 SHA 绑定、manifest 原字节与 canonical JSON 区分、plugin/policy digest 与 selected binary 直接绑定、clean lockfile 依赖来源、tarball/member 等价复核。COV2 实施不凭此授予 issuer；真实注册、Source A、三字段归档仍受父计划原门。

## 执行检查必备需求

本计划继承 `docs/development/plan/plan-template.md` 的 ER-01..ER-14、GC-01..GC-13、G-01..G-11。逐卡具名 AC/VER 是 ER-04 A/B 的补充：所有会推送的卡仍必须在最终源码上通过 `cargo +nightly fmt --all --check`、`cargo clippy --all-targets --all-features -- -D warnings`；触发 ER-13 T-1/T-4/T-6 的卡须再跑全量；唯一发布点首次 T-2 必须在最终已 bump 的实现源码上用 `source .env.test && source .env.live-test && cargo nextest run --all --no-fail-fast --retries 2` 实际全绿，不把 `cargo test --all` 当 release 通过证据。其后仅上游纯 patch 顺延且生产源码、测试与行为文档逐路径 SHA/mode 等价时，才可按 COV2-REL-01 规则复用同实现的已绿全量/本机手工证据；新版本面、当前 SHA 安全 CI、四平台签名发布与安装仍须实证。卡内 exact 测试必须实际报告 1 passed/0 failed（审核组要求组计数），0 test、skip、早退、失败、synthetic/different-source 均不能当 PASS。子卡 `family child` 继承 COV2-REL-01 的 C/D，故 A/B + Claude PASS + signed main 后仅 `locally-accepted`；release 实际 C/D 全绿才逐卡 `complete/done`。

## 实施顺序

**OG24.3共享写权阶段协调（DEP-OG-24-029-SHARED/RETURN）：** COVsource链只消费signed6685 `.2`snapshot；原DEP13 source-ready后，COV2-REL-01先完成自身三planI及R的immutable sourcefreezeF。RouteA仅在SRC029实际ready时授出共享path/hash/mode/owner lease；grant→OG24.3landing→signed-return（或满足严格条件的abort-return）→root唯一在返回postimages写COVREL actualC/D→DEP13complete。RouteB在SRC未ready或合法abort-return后先完成RELcloseout/DEP13complete再grant；禁止混装route。R始终仅root对F，OG24不写R，不改F/tag。该coordination不改本COV各卡AC/VER/I/R或12/12发布谓词；其完整共享清单/abort条件/owner/证据见canonical DEP rows及父[shared阶段](plan-20260902.md#native029-shared-stage-handoff)。

**DEP-COV2-029-BASE（incoming，父ordinary29 source-qualified shared I–I handoff）：** artifact为FIX-OG-NATIVE-IMPORT-01/OG-IDLE-ERROR-029-01/FIX-OG-PARTIAL-LIVE-01/OG-UPGRADE-029-01最终A/B、单Claude、signed+DCO main/remote与精确源pre/postSHA/mode；owner本对话root。BASE按父当前G09只交付official29 namespace＋真实自然自动capture/checkpoint/普通export基线；DEFER-OG29-PARTIAL-IMPORT计划外延期，不授历史unsupported-state import能力，也不作为本DEP/049/COV2前驱。COV2-22仍先消费完整BASE源（不等其049 C/D），严格边25→FIX→IDLE→FIX-OG-PARTIAL-LIVE-01→BASE→22保留；COV2-05/18的agent_import、06/15/18的opencode_reasoning、11生产调用及其它后继均按实际源重基且保留原candidate/privacy/cursor oracle，不能并行写。网站FIX→IDLE→live→BASE→COV2-09→11由原cf owner串行交接。既往GLM import020 FAIL保留、默认拒绝，不由live成功替代；native3/Source A独立。本次范围修订已由signed+DCO main `84bbde7bd6a5f12dee2477755d0a955798518413`/remote同SHA实际采纳，完整五路径交接经sole-Claude R2 PASS；BASE现in-progress，8AC/2VER未验收，仍不source-qualified，COV2-22及相关后继仍须等待其全部实际source门。

### 依赖登记表

| ID | direction | 类型 | 对象 | Owner | 产物与可用性判据 | 证据 | 超时与失败策略 |
|---|---|---|---|---|---|---|---|
| DEP-COV2-OG26 | incoming | 跨计划 | `plan-20260902.md#OG-26` | OG-26 单写者；COV2-01 只读核验接收 | OG-26 最终 A/B、适用 focused/full、fmt/strict、唯一 Claude PASS；signed+DCO main SHA 与 `reasoning.rs`、`opencode_reasoning.rs`、`snapshot.rs` 及相关路径逐文件 post SHA/mode；共享写集已让出。 | 父计划 OG-26 执行记录及实际 main SHA/hash/mode；COV2-01 首笔写前刷新 | 任一缺口则 COV2-01 保持 pending，只可只读准备；当日交接无法证明即停止该卡，不覆盖 OG-26 修改。 |
| DEP-COV2-OG24-FROZEN2 | incoming | immutable source snapshot | OG24.2 signed main6685b82a938919a8e51f46b7c51cc305b40189a2及14path manifest e452796f5bce9316471ceaefa8796298826fece845927b99763b5e7c1c41ee85 | parent root交出，OG25/26及COV source owner只读消费 | 已满足的producer-independent原source资源；不指向currentOG24.3 lifecycle；native/runtime/SourceA仍不授予 | 原patch a1d530c75f359730d2f94b7d54911a16b5d54cdd8c43c49680780b6ef2522cdd；原Claude91f1759e98c542474bca5067d60e70f25afc8b78f07a523a8b6409c47c2cdd91 | 缺exactsignedsource/hashmode即停该资源复用；不得将current3→OG25/26/COV反向加环 |
| DEP-OG-12-SRC029 | incoming | current producer source/profile | 当前selected2.0.29-libra-native.3及完整source/archive/patch/modes/lockeddependencies/compiler/nativeassets/SDKindependentexpected/binaryrelation | 外部private producer source owner交出；root最终裁决；OG24.3消费 | [父计划S1–S8](plan-20260902.md#native029-source-profile-gates)全门逐项actualreceipt：全66和nativeclosure正式单Claude PASS、independentofficial29generator+revieweddelta expected实际compare、独立rebuild或受限transformed-output绑定selected；当前UNRUN | 逐门完整source/tool/dependency/assets/binary SHA/size/mode及实际raw/exit/review；candidate26d914不先approved，794c不是gold | 缺一不写approved3row/不登记；保留失败，不用旧2或synthetic代替；需要producer代码修复先细化owner写集 |
| DEP-OG-12-RUN029 | incoming | owned current runtime/issuer | SRC029合格及OG24.3 exactsigned source-qualified main，本地构建非release Libra与selected3producer双产物 | owned macOS runtime/registration owner交出；root裁决；OG27消费signature正例，OG05消费后续三字段 | source/recipe/tool/binary SHA-size-mode-version具名；独立HOME/XDG/config/registry；cold stdio/plugin/physicaltransport/freshkey/priortrust/epoch/撤权边界及至少真实signature receipt；无stable49/F包含3前置；当前UNRUN | Libra和producer分别准确身份、selectedpath、observer/policy/transport/challenge/epoch/realreceipt；仅macOS本机账号/模型，不进CI | 不改globalopencodeslot、不自动提升旧key；freshregistry才epoch0；未闭合OG27blocked；另两字段由OG05同current3 source真实逐路验，不被signature单项豁免 |
| DEP-OG-24-029-SHARED | incoming/outgoing | stage-level exclusive shared I handoff | DEP13原source-ready + COVREL自身I/R immutable sourcefreezeF + sharedpath grant | root是唯一rootsource/三plan/Rowner；cf单页原独立owner；OG24.3接收lease | RouteA须SRC029实际ready；F原门/full/单Claude/signedsource并排除3landing/OG27；grant逐path SHA-mode-lastwriter/UTC/route，覆盖三plans、tracing-agent/EN/zh/COMPAT/docscontract及完整OG24I；COV相交写者暂停，R仍只F | 原DEP13的25nonrelease/22finalmacOScases source-ready收据、F/source及具体授出manifest；当前未授出 | 缺gate不授；SRC未好选B先RELcloseout；A发grant后只能signedreturn或严格abort-return，不能时间窗豁免/双写/偷偷改route |
| DEP-OG-24-029-RETURN | incoming/outgoing | stage-level shared I return / abort | OG24.3 signed source交接或严格未落地abort-return；REL closeout事实写权 | OG24唯一root/cfowner交还，root签收并唯一写COVREL三planactualC/D | 成功return：6AC8VER/ER04/full/单Claude/signed+DCOsource、每sharedpostSHA/mode/lastwriter；或root发abort-return：未有任何3实现/站点signed或pushedlanding，且逐path为grant SHA/mode或与独立账本中准许非OG24 signed delta的完整机械重放SHA/mode相等、保护region字节不变；只此receipt可A→B | success逐源收据；abort列root/cf签推不存在的实际核查、每path grant/expected/current blob-OID/SHA/mode、唯一保护anchor/region SHA和逐commit非OG24 signed-delta ledger；未知差异拒绝。G09规范采纳commit不是3实现landing；receipt不能伪造 | 成功已签源仅forward-only，禁止以复原bytes偷偷A→B；无return REL三planfacts暂停。return后唯一root在postimages写actualC/D，不改F/tag或声称F含3 |
| DEP-COV2-029-BASE | incoming | 跨计划shared I–I source handoff | 父FIX/IDLE/新live/BASE最终source链 | 本对话root | BASE source-qualified signed main前消费新live五源；COV22/05/06/11/15/18及其它相交Icore重基，网站同parent serial | BASE已按最终A/B/full10133/soleClaudeR2与signed main `1e2177e11029e28e3c81afbe6ec9f1609cdde7da`/cf `93b10519e1943ea3eea7c943e816418e92b1ca0b`、Push0/remote同SHA完成source-qualified交接；该前置源条件现已满足，COV2后继仍须按既定shared I逐SHA/mode串行重基；父四卡formal049 C/D已实际完成；本COV2家族C-D仍pending、Source A仍UNPROVEN，020仍FAIL/deferred/defaultreject | 后继仍等BASE source-qualified；DEFER不设前驱，不并发写，不豁免Source A/COV family或import隐私门 |
| DEP-COV2-SITE | incoming | 外部仓库 | libra-backend `origin/cf` 的 `apps/tanstack-app/content/docs/commands/agent.en.md` | 经仓库 VCS 核实的独立 `cf` owner；09/11 互斥 | 记录远端 cf pre-SHA、干净隔离 checkout 根、目标页 pre-hash/mode；同卡 typecheck/build、签名+DCO 推送，发布点另核部署/HTTP。现有原 checkout 落后31且页已脏，不可覆盖。 | 隔离 owner 的 VCS/status/pre-hash 与 build/推送/部署收据，开工时刷新 | 无干净 owner 则 COV2-09 保持 pending 并停止依赖链；绝不以 locally-accepted 假过，同日无法交接即登记 blocked；发布 D 不得声称网站已部署。 |
| DEP-COV2-MODEL | incoming | 本机外部模型 | 用户已配置的 OpenCode Anthropic 可读响应路径（非 native 来源证明） | COV2-17 macOS 本机执行者 | 最终行为源码上观察 genuine visible-only 响应，必须由已审 classifier 证实该样本无 state/proof-bearing 候选；记录无敏字段名、JSON 类型/数量、版本与 keyed digest；如实际返回 `signature`/`originProof`，该样本不算 visible-only 正例。不登记 issuer、不触发 live mint、不把它当 Source A。 | 本机 sanitized 1/1 观察记录、实际 build/model/plugin/transport ID、最终源码 SHA；不得记录正文/密钥 | 每个不同最终行为源码 SHA 最多一次本机尝试、10 分钟；COV2-REL bumped SHA 的复测另计一次。无样本或模型失败则 COV2-17/REL 保持 blocked，明确 UNRUN，不用 synthetic 代替。 |
| DEP-COV2-OG27 | outgoing | 跨计划 | `plan-20260902.md#OG-27` / `DEP-OG-13` | 父计划 OG-27 接收者 | COV2-17 在全部22例本机非 skip PASS 且23张 implementation 源卡签名 main 交接后仅登记 `source-ready`，只允许隔离准备；COV2-REL-01 stable patch、四平台、安全 CI、站点/安装 C/D 全绿后才登记 `complete` 并许可 OG-27 main Push。接收者不重做 coverage-v2 家族实现，仍独立证明 `DEP-OG-12` Source A。 | 新计划 COV2-17/COV2-REL-01 报告、release run/manifest/site/install SHA/UTC；OG-27 开工时刷新 | 若接收者下一工作日未接手或门未绿，本计划保持 complete/source-ready 事实、父计划 OG-27 blocked；不替接收者做 native 注册、不以 synthetic 或旧版拒绝充当 GO。 |

**G-09 计划内新增顺序边：** `COV2-03→COV2-19→COV2-20→COV2-21→COV2-23→COV2-04→COV2-24→COV2-25→FIX-OG-NATIVE-IMPORT-01→OG-IDLE-ERROR-029-01→OG-UPGRADE-029-01→COV2-22→COV2-08`。前驱必须以最终 A/B、适用 focused/full、fmt/strict、唯一 Claude PASS 及 signed+DCO main SHA/hash/mode 交接；后继不能把其继承的家族 C/D `locally-accepted` 冒称 complete。外部 `DEP-*` 不变。

网站卡写集中的 `apps/tanstack-app/content/docs/commands/agent.en.md` 是 libra-backend 仓库的精确相对路径；执行前把实际 `origin/cf` pre-SHA、隔离 checkout 的绝对根路径和该文件的 pre-hash/mode 登记在 DEP-COV2-SITE。只在核验过的干净隔离 `cf` checkout 编辑该相对路径；`cf` 是分支名，不属于文件写集。原脏 checkout 不可写。

### 发布分组与并发窗口

| ID | 成员 | 唯一发布点 | 窗口规则 | 失败回滚顺序 | 理由 |
|---|---|---|---|---|---|
| REL-OG-TEMP-049（用户授权临时操作，不新增Task） | COV25→FIX→IDLE→FIX-OG-PARTIAL-LIVE-01→BASE，latest upstream归最终树 | 三卡same-card文档/website，COV25保留家族C-D | 唯一root R/USER-TEMP-REL-049；final-source/full含新live，BASE自动捕获基线须实际验收；排除DEFER-OG29-PARTIAL-IMPORT，原020仍FAIL；actualCI/8jobs/4平台/安装/网站全部保留 | BASE→live→IDLE→FIX→COV25逆序前滚；不complete COV family/DEP13 |
| REL-COV2-01 | `COV2-01→13→02→07→14→03→19→20→21→23→04→24→25→22→08→05→09→06→10→11→12→15→18→16→17`；23 implementation + 2 no-release audit | COV2-REL-01，家族内仅此卡写 R；USER-TEMP-REL-049 独立例外见下文 | OG-26 签名交接后开窗；每张子卡本机 A/B、Claude PASS、signed+DCO main Push，但不 bump/tag/release；除下方 USER-TEMP-REL-049 明确授权例外外，不得让其它计划在半成品 COV2 main 上发布稳定版。COV2-11 前无生产入口可激活 v2 writer，并以静态/运行时守卫证明。冻结/bump/tag 时禁止其它 COV2/OG-27 源码及 R 写入；若其它发布无法停，先证明其中不携带可触达的半成品 v2，否则 COV2 链暂停。 | `17→16→18→15→12→11→10→06→09→05→08→22→25→24→04→23→21→20→19→03→14→07→02→13→01` 逆序定位受影响 owner，签名前滚，重跑下游门并重新冻结；已发版 tag/asset 不改写，只出新 patch。 | schema/reader/持久化不能分开用户可见发布；G-08唯一版本点；DEP-OG-24-029-SHARED/RETURN显式约束sourcefreezeF→grant→signed/abort-return→rootfactscloseout（A）或closeout/complete→grant（B）；R只F，三plan事实在return后才写，原AC/VER/I/R不变。 |

### 用户明确授权的临时 0.30.49 发布窗口

> **用户临时 patch 发布 USER-TEMP-REL-049（恢复后的新增明确授权）：** 用户要求纳入上游 `cat-file` 修复，在 COV2-25 本机 A/B、最终单 Claude PASS 与 signed+DCO main 源码交付后临时发布 **0.30.49**。执行者为本对话 root 单一主仓写者/发布者；先核真实最新上游并保留双方变更，实际源码变化须在最终源码重验。此项release操作自身不新增任务卡；其本次G09兼容前驱另新增父计划FIX/IDLE/BASE三卡而COV2不增，临时操作登记REL-OG-TEMP-049，不代替 COV2-REL-01，不据此把 COV2 家族子卡或 DEP-OG-13 记 complete。原半家族禁止稳定发布的规则只对此次用户明确授权操作覆写：必须证明所有未完成 coverage-v2 生产入口仍不可触达且普通 macOS 捕获行为安全；若证明失败则先修复或保持 UNRUN，不能发布。该临时 release 须沿用用户已授权的纯版本变动例外：先把真实最新上游 `cat-file` 与本次FIX/IDLE/BASE三卡纳入最终实现树（COV25旧full不替新源码），并完成适用 focused、双 env 全量 Nextest 与唯一 Claude PASS；若后续临时 bump 仅改变三个版本面及工具链锁文件中的包版本（另可新增 `CHANGELOG.md` 的纯发布说明及本次计划记账），并以逐路径 SHA/mode 证明生产实现、测试、配置与行为文档同一，可复用该已绿 full，不再重复全量。新增 CHANGELOG 仅追加 `[0.30.49]` 节并保留全部历史节；它是发布说明的新增验收面，不从用户纯版本授权推断已测试，bump 树必须现场核 `rg -n "CHANGELOG" tests src build.rs` 的读取者，并实跑 `source .env.test && cargo test --test compat_r0_9_doc_closeout diff_docs_and_changelog_carry_the_degradation_semantics -- --exact` 具名 1/1及任何其它实际读取该文件的相关契约。bump 后仍实跑 fmt/strict、版本面契约、唯一 Claude 最终字节审查、当前 SHA 安全 CI、完整 release 流程（当前 8 个 job）全部 success、Homebrew tap 更新与 `verify-homebrew-formula` 实际运行且 success（不得 skipped 或仅缺 token warning；formula version/SHA 必须与本次 release 资产一致）、四平台签名资产/manifest、CDN 版本/manifest/资产同源及网站/安装验收、实际 macOS 安装版 `cat-file` smoke 与用户指定 curl 安装；任何实际源码/测试/配置/行为变化均须在最终实现树重新 full。网络失败仅在日志确认后用 gh 重跑。家族正式唯一发布点仍为 COV2-REL-01，Source A/native 三字段门不变。

**临时不可达扫描工具冻结（首次最终源码执行前）：** 本机只读 Python driver SHA-256 `0dc5ba2bfe09cc7f40958ffb79ccaa087fce72bc968062717ac8d699b09c6d6c`，syn scanner source `d43d3e5957780717e4b0a463cde52f29f94a3f02b085f0af13dcb77bc6e19c90`，Cargo.toml `fcd0de5173feea9d605f36b9858887f584c714b368612b79ee1a61aa1fc962d2`、工具链 Cargo.lock `1cf63de32023e1908fd95a13caab6340e4ea8e499df5df014efd80ed6b843f27`、实际 scanner binary `2d22f3b322f68f729e5efbada75b1aea1c9942799abaa7231af0aea2194fb1cc`。工具在仓库外，不是产品或验收结果；执行前核这些 hash，执行后记录全部实际 src 输入 hash/mode 和原始结果。此刻最终 COV2-25 扫描仍 UNRUN。

**临时 R 精确写集与门：** 只写 `Cargo.toml`、`install.sh`、`install.ps1`、工具链更新的 `Cargo.lock`、`CHANGELOG.md` 新增 `[0.30.49]` 纯发布说明（原历史版本节不改）及本次发布记录的三份计划文档；签名 tag/Release/manifest/四平台 artifact 与安装操作按现有稳定发布流程执行。版本面以 `compat_version_surface_sync` 实时集合为准；业务修复只来自已合入/已审上游 cat-file、COV2-25与本次FIX/IDLE/BASE各自已审精确I，不在该发布操作扩充业务轴。必须固定最终源码与未完成 v2 生产入口不可达证明的 SHA/路径/实际测试，实际执行只读生产调用 AST 扫描 `python3 /private/tmp/libra-temp49-release-preflight-r1/refresh-v2-entry-preflight.py . /private/tmp/libra-temp49-release-preflight-final-cov25`（核 helper SHA、所有 src 输入 SHA/mode 与结果；新 key verifier 对 decoder 的内部引用不等于 CLI 可达，verifier 自身必须零生产 caller），并实跑原 v1 `cov2_v1_pending_bytes_and_mac_frozen` 与 `reasoning_pending_pre_v2_reader_oracle_is_immutable` 两个 exact 1/1 门；扫描不覆盖任意外部 embedding 调用，不作全程序或 native Source A 证明。先完成含最新真实上游与三卡新源码的最终实现full（旧COV25冻结full不替新源码）；临时版本变化按上述逐路径同一性条件复用该 full，实际源码变化则重跑。完成适用 full 证据、bump fmt/strict/version-surface/macOS 安装 cat-file smoke、单审、安全 CI 与发布安装门后才记录临时 release success。未完成的 COV2 child 继续 `locally-accepted`，COV2-REL-01 继续 `pending`，父 `DEP-OG-13` 不提前解除；失败/取消/未运行不记 PASS。首次家族稳定发布仍在最终完整 bumped 实现树另行验证，不能把本临时版 full 当后续不同实现的 final-family full。

**并发声明:** 全串行；libra-backend `cf` 网站 owner 与主仓写者分仓，09/11 的同一网站页仍互斥。

**发布者:** COV2-REL-01 单一发布者；开工时登记实际执行者，不预设姓名。USER-TEMP-REL-049 临时例外由本对话 root 单一主仓写者/发布者持有当次 R 写锁，与 COV2-REL-01 互斥。

**发布窗口顺序:** 25 张非发布子卡（23 implementation+2 audit）按上表顺序 signed+DCO main 交接；COV2-17 source-ready；COV2-REL-01 首次在同一最终 bumped 实现树完成全量与版本面、单 Claude PASS、唯一 tag/release 和 D 组后，方可 OG-27 main Push。若之后仅上游纯 patch 顺延，按 COV2-REL-01 逐路径等价规则复用同实现全量；同一时刻只允许一个当前卡/临时发布操作持有 R 写锁。

**家族发布解释：** schema 与 reader 不能独立对外发布，故 G-08 例外采用唯一发布点。用户已授权每卡直推 `main`；与模板原 `family child` 本地提交流程的差异限于直接 main 源码交付，不赋予子卡 R/版本/发布权限。任何新 source 修复之后，审计和 release freeze 必须按最终 SHA 重做；不使用 `EX-OG-02` 或新 G-03 waiver。

**严格 DAG：** `OG-26 → COV2-01 → COV2-13 → COV2-02 → COV2-07 → COV2-14 → COV2-03 → COV2-19 → COV2-20 → COV2-21 → COV2-23 → COV2-04 → COV2-24 → COV2-25 → FIX-OG-NATIVE-IMPORT-01 → OG-IDLE-ERROR-029-01 → OG-UPGRADE-029-01 → COV2-22 → COV2-08 → COV2-05 → COV2-09 → COV2-06 → COV2-10 → COV2-11 → COV2-12 → COV2-15 → COV2-18 → COV2-16 → COV2-17 → COV2-REL-01 → OG-27 main Push → OG-28 → OG-29 → OG-05`。COV2-09/11 各另等 DEP-COV2-SITE；OG-27 另等父计划 Source A/DEP-OG-12。后继只消费前驱最终 A/B+Claude+signed main 的 source-qualified deliverables，不把前驱继承 C/D 假记 complete；发布点不能以 source-ready 替代 C/D。

### 字段全局默认与例外

- 所有 implementation：`Task type=implementation`、`Lifecycle=pending / Acceptance=空`、`Rollback mode=forward-only`、`Migration and rollback=无 DDL；签名前滚且历史 v1 不重写`、`Version increment=N/A`、`Release boundary=family child of REL-COV2-01`、`Release write set=N/A`、`C/D coverage from=COV2-REL-01`、`Estimated scope=M`。
- COV2-16/17：`Task type=audit`、`Lifecycle=pending / Acceptance=空`、`Rollback mode=forward-only`（审计失败返回实现 owner 重做，不撤销生产数据）、`Release boundary=no-release`、`Version increment=N/A`、`Release write set=N/A`、`C/D coverage from=COV2-REL-01`、`Estimated scope=S`。
- COV2-REL-01：`Task type=release`、`family release point of REL-COV2-01`、`Rollback mode=immutable-release`、`Version increment=patch`；版本面由 `compat_version_surface_sync` 定义，当前 2/2 tests 覆盖 `Cargo.toml`、`install.sh`、`install.ps1`，另有工具链 `Cargo.lock`。
- `I-core` 精确集合：`docs/development/plan/plan-20261013-coverage-schema.md`; `docs/development/plan/plan-status.md`; `docs/development/tracing/coverage-v2.md`; `docs/development/tracing/agent.md`; `COMPATIBILITY.md`。`I-audit` 精确集合：`docs/development/plan/plan-20261013-coverage-schema.md`; `docs/development/plan/plan-status.md`; `docs/development/tracing/coverage-v2.md`。卡内 I=相应集合加卡内显列路径；无 glob 授权。父 `docs/development/plan/plan-20260902.md` 在采纳本计划时由独立 parent/status amendment 登记依赖；此前 COV2-03 前 G-09 23卡拆卡规划协调独立于 COV2-14 和 COV2-03；本次 COV2-23/24 的 G-09 25卡协调独立于 COV2-03 和 COV2-19 两张实现卡，以**单独 signed+DCO main commit**交付，精确 I 仅为 `docs/development/plan/plan-20261013-coverage-schema.md`、`docs/development/plan/plan-status.md`、`docs/development/plan/plan-20260902.md` 三文件。额外父计划路径仅同步外部 COV2 单写者表、COV2 数量/追溯，不作 COV2-03 行为实施权。之后实施阶段仅本次 COV2-23/24 规划协调、COV2-17（source-ready）和 COV2-REL-01（complete/release）可写父计划。规划提交仍需最终字节单 Claude PASS、fmt/strict、signed+DCO main；不凭隔离补丁预记。所有重叠 I 由 DAG 串行。
- **本次 COV2-25 G-09 独立协调 I（本次签名 main 采纳）：** 仅 `docs/development/plan/plan-20261013-coverage-schema.md`、`docs/development/plan/plan-status.md`、`docs/development/plan/plan-20260902.md` 三文档；在 COV2-24 实际 signed/remote 同 SHA、最终 A/B、双 env full、唯一 Claude PASS、逐文件 hash/mode 后，独立完成三文档最终字节唯一 Claude PASS、fmt/strict、signed+DCO main Push。规划提交不实现 COV2-25，不给本卡父计划写权；没有自证式 commit SHA。
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
| COV2-20 | implementation | 版本化 live reservation 只读验领及前驱 Windows cfg 前滚修复 | forward-only | yes | yes | 7/8 | 3/8 | 1/2 | M | COV2-19 | 严格串行 | family child | COV2-03 | N/A |
| COV2-21 | implementation | 调用方事务内版本化 reservation retain | forward-only | yes | yes | 7/8 | 4/8 | 1/1 | M | COV2-20 | 严格串行 | family child | COV2-20 | N/A |
| COV2-23 | implementation | 同源 NativeProven pending-v2 借用门 | forward-only | yes | yes | 8/8 | 4/8 | 1/1 | M | COV2-21 | 严格串行 | family child | COV2-04 | N/A |
| COV2-04 | implementation | 有界认证 pending-v2 codec | forward-only | yes | yes | 8/8 | 4/8 | 1/2 | M | COV2-23 | 严格串行 | family child | N/A | N/A |
| COV2-24 | implementation | 同源借用到内存 pending-v2 seal 接线 | forward-only | yes | yes | 5/8 | 3/8 | 1/2 | M | COV2-04 | 严格串行 | family child | COV2-04 | N/A |
| COV2-25 | implementation | 原始 outer-v2 成员密钥完整性 | forward-only | yes | yes | 8/8 | 4/8 | 1/1 | M | COV2-24 | 严格串行 | family child | COV2-22 | N/A |
| COV2-22 | implementation | 真实 pending artifact 事务与冻结旧读端 | forward-only | yes | yes | 8/8 | 6/8 | 1/2 | M | COV2-25,DEP-COV2-029-BASE | 严格串行 | family child | COV2-04 | N/A |
| COV2-08 | implementation | pending 同版本恢复 | forward-only | yes | yes | 8/8 | 4/8 | 2/6 | M | COV2-22 | 严格串行 | family child | COV2-04 | N/A |
| COV2-05 | implementation | 版本化 import cursor/digest/故障 | forward-only | yes | yes | 7/8 | 4/8 | 1/1 | M | DEP-COV2-029-BASE；COV2-08 | 严格串行 | family child | N/A | N/A |
| COV2-09 | implementation | graph/doctor 双版本只读契约 | forward-only | yes | yes | 8/8 | 4/8 | 1/1 | M | COV2-05；DEP-COV2-SITE | 严格串行 | family child | COV2-05 | N/A |
| COV2-06 | implementation | OpenCode typed 来源分类的未激活交付 | forward-only | yes | yes | 8/8 | 4/8 | 1/3 | M | DEP-COV2-029-BASE；COV2-09；DEP-COV2-OG26 | 严格串行 | family child | N/A | N/A |
| COV2-10 | implementation | typed 脱敏与零明文 sink | forward-only | yes | yes | 8/8 | 4/8 | 2/4 | M | COV2-06 | 严格串行 | family child | COV2-06 | N/A |
| COV2-11 | implementation | live/import v2 生产接线与字节对齐 | forward-only | yes | yes | 8/8 | 6/8 | 3/5 | M | DEP-COV2-029-BASE；COV2-10；DEP-COV2-SITE | 严格串行 | family child | COV2-06 | N/A |
| COV2-12 | implementation | macOS 本地 Session 入口与既有 fixture 修复 | forward-only | yes | yes | 8/8 | 6/8 | 1/1 | M | COV2-11 | 严格串行 | family child | COV2-06 | N/A |
| COV2-15 | implementation | 候选 live Session 权限防火墙 | forward-only | yes | yes | 8/8 | 2/8 | 2/2 | M | DEP-COV2-029-BASE；COV2-12 | 严格串行 | family child | COV2-12 | N/A |
| COV2-18 | implementation | 候选 import Session 权限防火墙 | forward-only | yes | yes | 8/8 | 2/8 | 2/2 | M | DEP-COV2-029-BASE；COV2-15 | 严格串行 | family child | COV2-15 | N/A |
| COV2-16 | audit | 既有 bridge/pipeline/OG-26 12 例本机审计 | forward-only | yes | yes | 13/20 | 5/20 | 0/0 | S | COV2-18 | 严格串行 | no-release | N/A | N/A |
| COV2-17 | audit | 既有 lifecycle/crash/import/oracle 10 例及真实可读响应本机审计 | forward-only | yes | yes | 12/20 | 6/20 | 0/0 | S | COV2-16；DEP-COV2-MODEL | 严格串行 | no-release | N/A | N/A |
| COV2-REL-01 | release | reviewed family patch | immutable-release | yes | yes | 12/12 | 12/12 | 0/0 | M | COV2-17,DEP-COV2-SITE,DEP-COV2-MODEL | 唯一 R 写者 | family release point | N/A | N/A |

### Phase 0：OG-26 交接与 v1/v2 基础

OG-26 的最终 signed main 源码和共享路径逐字节交接完成后，依顺序执行 COV2-01/13/02/07/14/03/19/20/21/23/04/24/25/22/08。每卡先刷新事实锚点与 G-03/G-04/G-10；不在脏 shared path 上双写。

### Phase 1：import/reader/OpenCode typed 接线

严格接 COV2-05/09/06/10/11；COV2-09 与 COV2-11 的网站文档只在干净隔离 `cf` owner 中实施；同卡门是编辑、typecheck/build 与签名 `cf` 推送，部署/HTTP 由 COV2-REL-01 D 门验收，不把本地页修改当上线。

### Phase 2：macOS Session 与完整审计

COV2-12/15/18 新增 ignored 本机 Session 正负例；COV2-16/17 只读核对 22 个继承 ignored 测试实际执行。最终报告只记录所选测试、pass/fail/skip、版本/源码 SHA、必要的无敏字段类型与哈希。Linux 功能留给用户。

### Phase 3：唯一 patch 发布与交接

COV2-REL-01 冻结不含 OG-27 源码的最终 COV2 树；首次完成 bumped 实现树 full，若后来仅纯上游 patch 顺延且逐路径等价，则按 release 卡规则复用既有同实现 full；当前发布的安全 CI、四平台 stable、网站、Homebrew、manifest 与 `curl --proto '=https' --tlsv1.2 -sSf https://download.libra.tools/install.sh | sh` 本机安装。发布后 `DEP-OG-13=complete`，OG-27 还必须独立解决 Source A。2026-10-13 当时已安装的 0.30.48 拒绝私有 `.2`（LBR-AGENT-005），只读负面收据 SHA-256 `81c9d1ec7142785357c9b121bc3edf5c0700ed27f974874d1a7ff86e1a7bacb1`；这是**版本及时点限定的历史事实**，未来稳定 patch 安装后必须重测，不能把新版本注册结果从该负面事实或独立 r2 binary-to-source 子门推断出来。

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

**Full-suite trigger:** `coverage.rs` 是 live/import/export 共用规范 turn/digest 源，本卡客观命中 ER-13 T-1；在 COV2-01 最终源码的 A/B 阶段必须运行 `source .env.test && source .env.live-test && cargo nextest run --all --no-fail-fast --retries 2`。COV2-REL-01 首次 T-2 仍须在最终 bumped 实现树上独立重跑，不能复用本卡全量；后续仅上游纯 patch 顺延按该卡逐路径同一性规则复用。

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

**Release boundary:** family child of REL-COV2-01（1/23）；用户已授权逐卡 reviewed signed+DCO 直推 main，但不写 R、不 tag、不调用 release。

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

**Full-suite trigger:** 本卡只在现有文件新增 `#[cfg(test)]` oracle、既有 compat 测试和文档说明，不改 production helper、测试基建、公开 CLI/错误码、SQL 或 CI；按 ER-13 复核未命中 T-1/T-4/T-6，四项 exact VER 已覆盖 8 AC，因此本卡不独立重跑全量。COV2-01 的 10053/10053 仅是前驱源码证据，不冒称本卡全量；COV2-REL-01 首次 T-2 仍须在最终 bumped 实现树上重跑；后续仅上游纯 patch 顺延按该卡逐路径同一性规则复用。

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

**Release boundary:** family child of REL-COV2-01（2/23）；用户已授权逐卡 reviewed signed+DCO 直推 main，但不写 R、不 tag、不调用 release。

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

**Full-suite trigger:** 本卡改动 shared `coverage_gate.rs`，命中 ER-13 T-1；已在最终源码的 A/B 阶段执行双 env 全量 Nextest 10059/10059。COV2-REL-01 首次 T-2 仍须在最终 bumped 实现树上独立重跑，不能复用本卡结果；后续仅上游纯 patch 顺延按该卡逐路径同一性规则复用。

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

**Release boundary:** family child of REL-COV2-01（3/23）；用户已授权逐卡 reviewed signed+DCO 直推 main，但不写 R、不 tag、不调用 release。

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

**Full-suite trigger:** 本卡修改共享 `src/internal/metadata.rs` 身份 codec，命中 ER-13 T-1；已在最终源码 A/B 阶段实际运行双 env Nextest 10063/10063。COV2-REL-01 首次 T-2 仍须在最终 bumped 实现树上独立重跑；后续仅上游纯 patch 顺延按该卡逐路径同一性规则复用。

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

**Release boundary:** family child of REL-COV2-01（4/23）；用户已授权逐卡 reviewed signed+DCO 直推 main，但不写 R、不 tag、不调用 release。

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

**Full-suite trigger:** family child 不自行发布；按 ER-13 对最终改动重新判定 T-1/T-4/T-6。若命中 shared single-source helper、测试基建或 focused 信号不足，则在本卡 A/B 阶段运行双 env 全量 Nextest；无论是否命中，COV2-REL-01 首次 T-2 均须在最终 bumped 实现树上重跑；后续仅上游纯 patch 顺延按该卡逐路径同一性规则复用。

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

**Release boundary:** family child of REL-COV2-01（5/23）；用户已授权逐卡 reviewed signed+DCO 直推 main，但不写 R、不 tag、不调用 release。

**C/D coverage from:** COV2-REL-01；发布点的 exact-source C/D 绿后才能 `done/complete`。

**Granularity:** `type=implementation; axis=双版本私有 identity map 与共同容量; recovery=forward-only; complete=yes; self-contained=yes; AC=8/8; VER=7/8; landing=2; prod-files=2; scope=M; deps=COV2-07; writeset=串行于前驱; release=family child; split-from=COV2-07; exception=N/A`

### Task COV2-03：版本化 marker 与事务围栏

**Task type:** implementation

**Lifecycle / Acceptance:** in-progress / locally-accepted（AC1–8 的五项 VER、五项旧 v1 mandatory ER-04 与两项缺表回归均逐项 1/1；最终双 env full **Summary [1618.814s] 10078 tests run: 10078 passed (14 slow), 23 skipped**/raw SHA-256 `c49dd3c33652d55541a0e4c0269c817460b9109cf261eab5271d90760a49e8a9`、exit0；唯一最终 Claude Code PASS，P0/P1/P2=0、P3=5/raw SHA-256 `9ef25f92615eefcf2019212a306cfa7542ef416b693286bb99111006b3ea6df7`；signed+DCO 源码/文档 main `d717e42e0d1ec75e6ae576b21242fd1519040abd` 已 Push 且 remote 同 SHA；本卡 2026-10-13 20:29:37 UTC 交接时按已签 23 卡口径 source-ready 6/23=26.09%（历史收据；本次 25 卡修订的当前口径为 6/25=24.00%）。家族 C/D 仍待；0 matched、只有 helper 或 skip 均不算 PASS）

**Description:** versioned marker and transactional fence。本卡唯一行为轴、恢复和粒度见下方 Granularity；不得以 synthetic/native 互换验收。

**Out of scope:** 不改 SQL schema；本卡不签发 native 来源、不写 identity-v2、不负责 pending verify/retain 或 wire/version。由本卡拆出 COV2-19（封闭 native identity-v2 companion）、COV2-20（只读版本化 reservation verify）与 COV2-21（同事务 retain）；COV2-04 只做有界 authenticated codec，COV2-24 做同源 native 内存 seal，COV2-22 负责真实 pending persist/冻结旧读端，COV2-08 接恢复。`src/internal/ai/capture/catalog.rs` 对本卡仅只读；completed receipt 的精确 revision-set 比较必须落在本卡 I 内的 `traces.rs` 或 `capture/checkpoint.rs`。若最终行为确需修改 catalog，先按 ER-06a 修订精确 I、粒度与单写者表并重审，不隐式扩权。

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

**Full-suite trigger:** family child 不自行发布；按 ER-13 对最终改动重新判定 T-1/T-4/T-6。若命中 shared single-source helper、测试基建或 focused 信号不足，则在本卡 A/B 阶段运行双 env 全量 Nextest；无论是否命中，COV2-REL-01 首次 T-2 均须在最终 bumped 实现树上重跑；后续仅上游纯 patch 顺延按该卡逐路径同一性规则复用。旧隔离草案不替代本卡已验收的最终源码；`verify_checkpoint_claim_set_with_conn` 不得以空 expected fence 提前返回而漏掉非空 committed set，`TerminalReceiptAlreadyComplete` 不得未经精确集合比较给成功 ACK；最终源码须按 VER1/VER5 分别真实实测或 fail closed，另跑旧 v1 回执 ER-04 回归，保留 v1 冻结合约。

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

**Release boundary:** family child of REL-COV2-01（6/23）；用户已授权逐卡 reviewed signed+DCO 直推 main，但不写 R、不 tag、不调用 release。

**C/D coverage from:** COV2-REL-01；发布点的 exact-source C/D 绿后才能 `done/complete`。

**Granularity:** `type=implementation; axis=版本化 marker 与事务围栏; recovery=forward-only; complete=yes; self-contained=yes; AC=8/8; VER=5/8; landing=3; prod-files=5; scope=M; deps=COV2-14; writeset=串行于前驱; release=family child; split-from=N/A; exception=N/A`

### Task COV2-04：有界认证 pending-v2 codec

**Task type:** implementation

**Lifecycle / Acceptance:** in-progress / locally-accepted：COV2-04 纯 codec 的 8/8 AC、4/4 VER 与额外 producer-count 回归已在最终格式化源码本机验证，nightly fmt/strict Clippy/diff exit0；唯一 Claude Code PASS、signed+DCO main `03b10f887368a3db5206589f5dd6947432ffe26c` 已 Push 且远端同 SHA；家族 C/D 未结。

**Description:** 本卡的一条纯 codec 落点跨两个生产文件：`pending_payload.rs` 实现 versioned claim/identity 的有界内层表示与规范化编码，`pending.rs` 实现独立 outer-v2 envelope 的有界规范化 encoder，以及只接受既有 exact-byte MAC 验证所得 `AuthenticatedPendingEnvelope` witness 的 authenticated decoder。该 decoder 校验版本、binding 和有序 sealed tuple 后只返回内存 projection/claim plan，不签发 native 来源或可持久化 artifact 权限。从原 COV2-04 拆出 COV2-23 的同源 checked native borrow、COV2-24 的内存 sealed artifact 接线，以及 COV2-22 的真实持久化/冻结旧读端行为。纯 codec 可表达跨版本同 logical key；COV2-24 才消费本卡 codec 和 COV2-23 borrow。v1 历史字节仍由原始基线固定。

**Out of scope:** 不消费 `PreparedPendingNativeSeal`、不构造 `SealedVersionedPendingArtifact`、不执行数据库 persist/retain、compiled frozen old reader 或 replay；无手工 seeded artifact 行。生产 v2 artifact 至少含一个 v2 claim 的策略归 COV2-22 AC3。

**Current evidence / ER-02 source anchors（2026-10-14）：** 前驱 COV2-23 signed+DCO main `d977d13c5e23111b93d78477b9e1a70015a111b8` 已与 remote 核为同 SHA。开工前 `pending_payload.rs` SHA-256 `7170af6c3319825aad2aea63523405a949b0b546f2f532f6994f5d298f093d2e`、102537 bytes、mode `0644`；`pending.rs` SHA-256 `53e03c011b13722d74e57d6fd50d31246ff10524c1219037df2923f90185b182`、227245 bytes、mode `0644`；冻结只读 `tests/fixtures/agent_transcripts/reasoning/pending_v1_decoder.rs` SHA-256 `96d71660e8b75d61cff2dccaa59a410b90b499afd63e10c9dcc2cc678312e32c`、mode `0644`。本卡隔离生产补丁 SHA-256 `1f97200d4dcc92e2f5eced1b846119554c38920147f6669bba15de7d6ec38979` 已在 signed main 上 zero-fuzz/no-offset 重放；隔离生产补丁 post SHA-256 为 `719135942c0b277a3ff2f0bd3c2433c3ad8931269b3930973f6baa1e4c85b1cb`、`fff43b515233349bd4d14d7daf09b361b80a3b0ebcc7eb6e75387b1d0486003b`；新增 producer-count 回归并执行 nightly rustfmt 后，本卡最终候选两生产文件 SHA-256 为 `8b5b0eb27e1365780c9e92cbc40fab32183b22a1e9e375d4d527b54f83cc5520`（135496 bytes）、`7d10a864f4c2a80f13ed96317731b36236e5a91db9819396da2647db4438f3bf`（242101 bytes），mode 均 `0644`。生产落点 `pending_payload.rs:220,388,396`（versioned inner/projection 与分配前 claim 数 guard）、`pending.rs:213,328,369`（outer-v2 envelope、encoder、认证 decoder）。这些是源码镜像定位，不构成本卡测试、Claude 或 Source A 验收。

**Acceptance criteria:**

- [x] AC1：每个 v2 claim 显式携带 coverage schema version。
- [x] AC2：authenticated decode 保留完整有序 sealed tuple。
- [x] AC3：同一 logical key 可以各出现一次 v1 和 v2。
- [x] AC4：同版本重复 `(version,key)` 拒绝。
- [x] AC5：无效或未知 version 拒绝。
- [x] AC6：MAC/authentication 失败在 candidate projection 前拒绝。
- [x] AC7：entry count 在分配前有界。
- [x] AC8：编码 wire bytes 长度超限在分配前拒绝。

**独立谓词计数 / 兼容：** entry 与 wire-byte cap 是两项独立新行为；encoder 亦须服从 GC-COV2-05 输出上限。v1 pending canonical bytes/MAC 取变更前冻结 oracle，不由新 serializer 重生；属于硬性 ER-04 兼容门，非豁免。此卡文档必须说明 v2 artifact 尚未 durable persist。

**Verification:**

- [x] VER1：额外 mandatory ER-04 冻结门：`source .env.test && cargo test --lib internal::ai::capture::pending::tests::cov2_v1_pending_bytes_and_mac_frozen -- --exact`（原始基线 canonical bytes/MAC，1/1）。
- [x] VER2：AC1–3：`source .env.test && cargo test --lib internal::ai::capture::pending_payload::tests::cov2_cross_version_same_key_authenticated_codec_roundtrip -- --exact`（直接调用生产 `pending.rs` outer-v2 encoder 与 authenticated decoder，并核 `pending_payload.rs` 的 versioned 内层有序 tuple；不可由测试自造 serializer 代替；不授 native 权限；1/1）。
- [x] VER3：AC4/5：`source .env.test && cargo test --lib internal::ai::capture::pending_payload::tests::cov2_same_version_duplicate_and_unknown_reject -- --exact`（1/1）。
- [x] VER4：AC6–8：`source .env.test && cargo test --lib internal::ai::capture::pending_payload::tests::cov2_pending_v2_mac_and_bounds_fail_closed -- --exact`（生产 `pending.rs` outer-v2 bytes 的 MAC/witness 篡改在 typed decoder 前拒绝；生产 encoder/内层 codec 的 entry、wire-byte cap 分别断言；1/1）。

**额外 producer-count 回归（不增加 G-03 VER）：** `source .env.test && cargo test --lib internal::ai::capture::pending_payload::tests::cov2_versioned_producer_claim_count_guard_precedes_projection -- --exact` 以 4096/4097 边界验证 `from_live` 接受上限、拒绝超限且不写 envelope；源码核查确认入口 count guard 位于 `collect` 前。测试本身不区分入口 guard 与后续 `validate` 的同一拒绝错误；最终收据须含该 selector 的实际 1/1 与 raw log SHA，不以 VER4 的已分配 DTO 断言代替。

**COV2-04 本机验收收据（2026-10-14 04:41:27 UTC）：** VER1..4 加额外 4096/4097 producer-count 回归共五个具名 focused selector，在最终格式化源码上各 1 passed/0 failed/0 ignored，私有日志索引 SHA-256 `292bef85b72f6fe8bd8a5a4513c749d34c3d3dc4275bce9e7802a3c75395f45f`。nightly fmt、`libra diff --check` 与 strict all-target/all-feature Clippy 均 exit0，私有质量日志索引 SHA-256 `064fadb470b87350d6885ea6064ae1994c12bab17ee322352136f4c618847911`（文档收尾后再次运行 diff check exit0）；七文件最终 SHA/mode/diff manifest、唯一 Claude Code 单评审、signed+DCO main/remote 同源仍待；因此当前 source-ready 仍 10/25。本卡 ER-13 `none`，COV2-REL-01 的 T-2 exact-tree full 仍强制。这些 synthetic codec 测试不计 OpenCode Session、真实 native Source A 或家族 C/D。

**COV2-04 最终源码交接收据（2026-10-14 04:52 UTC 复核）：** 上述签名前候选历史不倒改。七文件完整 diff/SHA/mode manifest SHA-256 `b2206b7579e0cd0b1a05b858d6968e238ec56ce0ad9b95210509484d4b2535b2` 经评审前后和提交树复核一致；唯一 Claude Code Sonnet 对最终七文件给出 **PASS**，P0/P1/P2=0、P3=4，原始 JSON SHA-256 `3a9749ad4679e8e038eb7f3d5d09e36e0e367de4411e4f5848371c55d6ad5aac`。非阻断 P3 为 status 嵌套反引号、COV2-23 AC3 引用表述、decoder full-tree 峰值/数值测试建议、AC7/8 容量守卫的 instrumentation 建议；不把建议冒称修复。`libra commit -s` 的 `03b10f887368a3db5206589f5dd6947432ffe26c` 已验证 PGP `gpgsig` 和 DCO，Push 后 `libra ls-remote` 与 GitHub main 均同 SHA；源码就绪增为 **11/25=44.00%**，正式仍 0/25。COV2-04 ER-13 `none`，未在本卡跑 full；当前 COV2-24 的 A/B/T-5 与家族 C/D、Source A 仍待。

**旧端约束：** 冻结 `pending_v1_decoder.rs` 不编辑；本卡不从 codec roundtrip 推断 native authority、内存 seal 或 compiled old reader 可拒绝真实 artifact；内存 seal 在 COV2-24，真实 artifact/旧端正例在 COV2-22。

**Full-suite trigger:** `none`（本卡最终 diff 仅新增私有纯 codec 与现有测试模块，不触发 ER-13 T-1–T-6 的全局单一事实源/共享 helper、公开 surface、测试基建或不可归因 focused 失败）。本卡运行 VER1..4、额外 `cov2_versioned_producer_claim_count_guard_precedes_projection` 精确回归、nightly fmt、strict Clippy 和唯一 Claude Code 最终字节 PASS；发布点 COV2-REL-01 首次按 T-2 对 bumped 实现树运行双 env full；后续仅上游纯 patch 顺延按该卡逐路径同一性规则复用。COV2-23 的历史 full 不替代本卡或发布点 full；0 matched、skip、synthetic 或未执行均非 PASS。

**Dependencies:** COV2-23（传递依赖 COV2-21/20/19）；消费前驱真实最终 A/B、适用 focused/full、fmt/strict、唯一 Claude PASS 与 signed+DCO main SHA/hash/mode；不把继承 C/D 的 locally-accepted 冒称 complete。

**Deliverables:** 无独立文件；API、测试、兼容文档、逐文件 SHA/mode 和 review/验收事实写回本计划正文，随唯一发布点交付。

**Implementation write set:** I-core 精确五文件：`docs/development/plan/plan-20261013-coverage-schema.md`; `docs/development/plan/plan-status.md`; `docs/development/tracing/coverage-v2.md`; `docs/development/tracing/agent.md`; `COMPATIBILITY.md`；本卡新增精确路径：`src/internal/ai/capture/pending_payload.rs`; `src/internal/ai/capture/pending.rs`。现有测试模块可写；未列路径包括 `sql/**`、冻结 decoder、parent 计划均为只读，需另行 G-09/I 修订才能扩权。

**Release write set:** N/A；C/D 继承 COV2-REL-01。

**Files likely touched:** I-core 上述五文件；`src/internal/ai/capture/pending_payload.rs`（versioned 内层 claim/identity 有界 codec）；`src/internal/ai/capture/pending.rs`（outer-v2 规范化 encoder、authenticated decoder 与 v1 冻结字节回归）。

**Docs and compatibility impact:** I-core 的 coverage-v2、agent、COMPATIBILITY 只陈述本卡纯 codec 已交付，不提前宣称内存 seal、下游 durable artifact/replay/Source A 已完成。无新 `--test` target；`tests/INDEX.md` 无写权。

**Rollback mode:** forward-only；失败保持旧 artifact/claim，签名+DCO 新 main 前滚并重跑下游。

**Migration and rollback:** 无 DDL；Repository claim 使用既有 versioned 键。若实际需迁移，先加角色隔离 migration 卡、旧/新 binary 门并修 G-03/G-04/I。

**Security and privacy:** 继承 GC-COV2-02/03；所有错误 fail closed、零假 ACK、零候选明文 sink；真实 native Source A 仍在父计划。

**Performance budget:** 使用已审 claim/map 上限，解析/查询前有界；不得新增无界扫描、无时限重试或私开的第二连接。

**Estimated scope:** M（AC=8/8、VER=4/8、landing=1、production files=2；一条纯 codec 行为轴横跨 `pending_payload.rs` 内层与 `pending.rs` 外层生产 API，后者不只是测试落点）。

**Version increment:** N/A；此卡不发版。

**Release boundary:** family child of REL-COV2-01（11/23）；逐卡单 Claude PASS 后 signed+DCO 直推 main，不写 R、不 tag。

**C/D coverage from:** COV2-REL-01；真实 exact-source C/D 通过后本卡才可 done/complete。

**Granularity:** `type=implementation; axis=有界认证 pending-v2 codec; recovery=forward-only; complete=yes; self-contained=yes; AC=8/8; VER=4/8; landing=1; prod-files=2; scope=M; deps=COV2-23; writeset=严格串行; release=family child; split-from=N/A; exception=N/A`

### Task COV2-08：pending 同版本恢复

**Task type:** implementation

**Lifecycle / Acceptance:** pending / 空

**Description:** pending same-version recovery。本卡唯一行为轴、恢复和粒度见下方 Granularity；不得以 synthetic/native 互换验收。

**Out of scope:** 不以 pending MAC 铸造 native 来源；每次 late replay 必须复用 COV2-25 原始成员 verifier 并保留可修复原始 bytes/MAC。`pending_payload.rs` 的本卡写权仅用于 AC1/5/7、VER1/3 的 outer-v2 deferred reasoning 关联、original bytes/MAC 留存与 late reauthentication，不改 COV2-22 的持久化 loader 主轴；只消费 COV2-22 真实已持久化 v2 artifact，不以 COV2-21 SQL 探针作 durable receipt。native same-key pending 在父 OG-29 版本关联正例及真实 Source A 前保持 partial/fail closed。

**Current evidence:** `recovery.rs` 与 `catalog.rs` 持有旧重放/目录；`pending_payload.rs` 当前仅有 v1 deferred replay/rehydrate 入口（基线 `dcc468b`；ER-02 开工刷新行号与最终源码）。本卡为已持久化 outer-v2 的 late replay 补 versioned deferred 关联及可修复原始 bytes/MAC 留存；VER1/3 以真实 COV2-22 artifact 消费它，不另造第二恢复轴。

**Acceptance criteria:**

- [ ] AC1 v2 replay retains the exact ordered `(coverage_schema_version, logical_turn_key)` tuple from the same committed artifact
- [ ] AC2 replay rejects stale owner
- [ ] AC3 stale fence yields zero checkpoint ACK
- [ ] AC4 replay rejects stale revision
- [ ] AC5 late COV2-25 original-member reauthentication failure yields zero checkpoint ACK
- [ ] AC6 interrupted replay cannot acknowledge one
- [ ] AC7 repairable pending bytes remain available after rejection
- [ ] AC8 pending authentication alone cannot mint native provenance.

**Verification:**

- [ ] VER1→AC1: `source .env.test && cargo test --lib internal::ai::capture::recovery::tests::cov2_replay_preserves_claim_version -- --exact` (1/1；由真实 COV2-22 artifact 重放，断言原始有序 version/key tuple 不变).
- [ ] VER2→AC2/3/4: `source .env.test && cargo test --lib internal::ai::capture::recovery::tests::cov2_replay_owner_fence_revision_fail_closed -- --exact` (1/1).
- [ ] VER3→AC5/6/7: `source .env.test && cargo test --lib internal::ai::capture::recovery::tests::cov2_failed_or_interrupted_replay_keeps_pending -- --exact` (1/1；含 late COV2-25 原始成员复验失败零 ACK、原始 bytes/MAC 留存；fence 零 ACK 归 VER2/AC3).
- [ ] VER4→AC8: `source .env.test && cargo test --lib internal::ai::capture::recovery::tests::cov2_authenticated_pending_has_no_native_authority -- --exact` (1/1). Existing v1 recovery tests are a required ER-04 A surface gate, not an unrun assumption.

**Full-suite trigger:** family child 不自行发布；按 ER-13 对最终改动重新判定 T-1/T-4/T-6。若命中 shared single-source helper、测试基建或 focused 信号不足，则在本卡 A/B 阶段运行双 env 全量 Nextest；无论是否命中，COV2-REL-01 首次 T-2 均须在最终 bumped 实现树上重跑；后续仅上游纯 patch 顺延按该卡逐路径同一性规则复用。

**Dependencies:** DEP-COV2-029-BASE（必须包含FIX-OG-PARTIAL-LIVE-01实际五源交接，不等正式049 C/D）； COV2-22；另显式依赖其前驱 COV2-25 的 key-owner integrity API；消费前驱实际最终 A/B、适用 focused/full、fmt/strict、单 Claude PASS 与 signed+DCO main 的 source-qualified 文件 SHA/mode；不以其尚待 C/D 的 `locally-accepted` 冒称 complete。

**Deliverables:** 无独立文件；本卡实现/测试/文档随唯一发布点交付；每卡的计数、源码 SHA、hash/mode、review/门结果写在本计划正文，不建证据文件。

**Implementation write set:** I-core = `docs/development/plan/plan-20261013-coverage-schema.md`; `docs/development/plan/plan-status.md`; `docs/development/tracing/coverage-v2.md`; `docs/development/tracing/agent.md`; `COMPATIBILITY.md`；本卡新增精确路径 = `src/internal/ai/capture/catalog.rs`; `src/internal/ai/capture/checkpoint.rs`; `src/internal/ai/capture/recovery.rs`; `src/internal/ai/capture/pending_payload.rs`; `src/internal/ai/capture/live_checkpoint.rs`; `src/internal/ai/coverage_gate.rs`。无 glob 默示写权。

**Release write set:** N/A；C/D 继承 COV2-REL-01。

**Files likely touched:** `docs/development/plan/plan-20261013-coverage-schema.md`; `docs/development/plan/plan-status.md`; `docs/development/tracing/coverage-v2.md`; `docs/development/tracing/agent.md`; `COMPATIBILITY.md`; `src/internal/ai/capture/catalog.rs`; `src/internal/ai/capture/checkpoint.rs`; `src/internal/ai/capture/recovery.rs`; `src/internal/ai/capture/pending_payload.rs`; `src/internal/ai/capture/live_checkpoint.rs`; `src/internal/ai/coverage_gate.rs`。

**Docs and compatibility impact:** `docs/development/tracing/coverage-v2.md`; `docs/development/tracing/agent.md`; `COMPATIBILITY.md`；逐文件核内容与源码行为对应，`tests/INDEX.md` 只在本卡精确写集确有索引变化时同步。无新增网站命令行为；若实施引入公开差异，先按 ER-06a 修本卡写集再编辑。

**Rollback mode:** forward-only；恢复时签名+DCO 新 main 修复，既有 v1 claim/artifact/digest 不回写或重解释。

**Migration and rollback:** 无 DDL；Repository claim 仅用既有 versioned 键。若实现证明 DDL 必需，停止、加 GC-13 角色隔离迁移卡与旧/新读端门，再修本卡写集和 G-03/G-04。

**Security and privacy:** 继承 GC-COV2-02/03；错误或不可信版本必须 zero-success、不得向 ordinary/typed/log/error 导出候选内容；Session 例只在本机 ignored 执行。

**Performance budget:** 继承 GC-COV2-05；identity map 的 v1+v2 总量仍限 4096 条/1 MiB，任何新增解析在分配前有界；变更 hot path 时以具名门证明无无界扫描。

**Estimated scope:** M（落点/生产文件计数：8/8, 4/8, 2, 6; M）。

**Version increment:** N/A；此卡不发版。

**Release boundary:** family child of REL-COV2-01（15/23）；用户已授权逐卡 reviewed signed+DCO 直推 main，但不写 R、不 tag、不调用 release。

**C/D coverage from:** COV2-REL-01；发布点的 exact-source C/D 绿后才能 `done/complete`。

**Granularity:** `type=implementation; axis=pending 同版本恢复; recovery=forward-only; complete=yes; self-contained=yes; AC=8/8; VER=4/8; landing=2; prod-files=6; scope=M; deps=COV2-22,COV2-25; writeset=串行于前驱; release=family child; split-from=COV2-04; exception=N/A`

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

**Full-suite trigger:** family child 不自行发布；按 ER-13 对最终改动重新判定 T-1/T-4/T-6。若命中 shared single-source helper、测试基建或 focused 信号不足，则在本卡 A/B 阶段运行双 env 全量 Nextest；无论是否命中，COV2-REL-01 首次 T-2 均须在最终 bumped 实现树上重跑；后续仅上游纯 patch 顺延按该卡逐路径同一性规则复用。

**Dependencies:** DEP-COV2-029-BASE（父三卡final source重基，shared I–I串行，非native能力）； COV2-08；消费前驱实际最终 A/B、适用 focused/full、fmt/strict、单 Claude PASS 与 signed+DCO main 的 source-qualified 文件 SHA/mode；不以其尚待 C/D 的 `locally-accepted` 冒称 complete。

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

**Release boundary:** family child of REL-COV2-01（16/23）；用户已授权逐卡 reviewed signed+DCO 直推 main，但不写 R、不 tag、不调用 release。

**C/D coverage from:** COV2-REL-01；发布点的 exact-source C/D 绿后才能 `done/complete`。

**Granularity:** `type=implementation; axis=版本化 import cursor/digest/故障; recovery=forward-only; complete=yes; self-contained=yes; AC=7/8; VER=4/8; landing=1; prod-files=1; scope=M; deps=DEP-COV2-029-BASE,COV2-08; writeset=串行于前驱; release=family child; split-from=N/A; exception=N/A`

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

**Full-suite trigger:** family child 不自行发布；按 ER-13 对最终改动重新判定 T-1/T-4/T-6。若命中 shared single-source helper、测试基建或 focused 信号不足，则在本卡 A/B 阶段运行双 env 全量 Nextest；无论是否命中，COV2-REL-01 首次 T-2 均须在最终 bumped 实现树上重跑；后续仅上游纯 patch 顺延按该卡逐路径同一性规则复用。

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

**Release boundary:** family child of REL-COV2-01（17/23）；用户已授权逐卡 reviewed signed+DCO 直推 main，但不写 R、不 tag、不调用 release。

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

**Full-suite trigger:** family child 不自行发布；按 ER-13 对最终改动重新判定 T-1/T-4/T-6。若命中 shared single-source helper、测试基建或 focused 信号不足，则在本卡 A/B 阶段运行双 env 全量 Nextest；无论是否命中，COV2-REL-01 首次 T-2 均须在最终 bumped 实现树上重跑；后续仅上游纯 patch 顺延按该卡逐路径同一性规则复用。

**Dependencies:** DEP-COV2-029-BASE（必须包含FIX-OG-PARTIAL-LIVE-01实际五源交接，不等正式049 C/D）； DEP-COV2-029-BASE（父三卡final source重基，shared I–I串行，非native能力）； COV2-09；DEP-COV2-OG26；消费前驱实际最终 A/B、适用 focused/full、fmt/strict、单 Claude PASS 与 signed+DCO main 的 source-qualified 文件 SHA/mode；不以其尚待 C/D 的 `locally-accepted` 冒称 complete。

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

**Release boundary:** family child of REL-COV2-01（18/23）；用户已授权逐卡 reviewed signed+DCO 直推 main，但不写 R、不 tag、不调用 release。

**C/D coverage from:** COV2-REL-01；发布点的 exact-source C/D 绿后才能 `done/complete`。

**Granularity:** `type=implementation; axis=OpenCode typed 来源分类的未激活交付; recovery=forward-only; complete=yes; self-contained=yes; AC=8/8; VER=4/8; landing=1; prod-files=3; scope=M; deps=DEP-COV2-029-BASE,COV2-09；DEP-COV2-OG26; writeset=串行于前驱; release=family child; split-from=N/A; exception=N/A`

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

**Full-suite trigger:** family child 不自行发布；按 ER-13 对最终改动重新判定 T-1/T-4/T-6。若命中 shared single-source helper、测试基建或 focused 信号不足，则在本卡 A/B 阶段运行双 env 全量 Nextest；无论是否命中，COV2-REL-01 首次 T-2 均须在最终 bumped 实现树上重跑；后续仅上游纯 patch 顺延按该卡逐路径同一性规则复用。

**Dependencies:** DEP-COV2-029-BASE（必须包含FIX-OG-PARTIAL-LIVE-01实际五源交接，不等正式049 C/D）； COV2-06；消费前驱实际最终 A/B、适用 focused/full、fmt/strict、单 Claude PASS 与 signed+DCO main 的 source-qualified 文件 SHA/mode；不以其尚待 C/D 的 `locally-accepted` 冒称 complete。

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

**Release boundary:** family child of REL-COV2-01（19/23）；用户已授权逐卡 reviewed signed+DCO 直推 main，但不写 R、不 tag、不调用 release。

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

**Full-suite trigger:** family child 不自行发布；按 ER-13 对最终改动重新判定 T-1/T-4/T-6。若命中 shared single-source helper、测试基建或 focused 信号不足，则在本卡 A/B 阶段运行双 env 全量 Nextest；无论是否命中，COV2-REL-01 首次 T-2 均须在最终 bumped 实现树上重跑；后续仅上游纯 patch 顺延按该卡逐路径同一性规则复用。

**Dependencies:** DEP-COV2-029-BASE（必须包含FIX-OG-PARTIAL-LIVE-01实际五源交接，不等正式049 C/D）； DEP-COV2-029-BASE（父三卡final source重基，shared I–I串行，非native能力）； COV2-10；DEP-COV2-SITE；消费前驱实际最终 A/B、适用 focused/full、fmt/strict、单 Claude PASS 与 signed+DCO main 的 source-qualified 文件 SHA/mode；不以其尚待 C/D 的 `locally-accepted` 冒称 complete。

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

**Release boundary:** family child of REL-COV2-01（20/23）；用户已授权逐卡 reviewed signed+DCO 直推 main，但不写 R、不 tag、不调用 release。

**C/D coverage from:** COV2-REL-01；发布点的 exact-source C/D 绿后才能 `done/complete`。

**Granularity:** `type=implementation; axis=live/import v2 生产接线与字节对齐; recovery=forward-only; complete=yes; self-contained=yes; AC=8/8; VER=6/8; landing=3; prod-files=5; scope=M; deps=DEP-COV2-029-BASE,COV2-10；DEP-COV2-SITE; writeset=串行于前驱; release=family child; split-from=COV2-06; exception=N/A`

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

**Full-suite trigger:** family child 不自行发布；按 ER-13 对最终改动重新判定 T-1/T-4/T-6。若命中 shared single-source helper、测试基建或 focused 信号不足，则在本卡 A/B 阶段运行双 env 全量 Nextest；无论是否命中，COV2-REL-01 首次 T-2 均须在最终 bumped 实现树上重跑；后续仅上游纯 patch 顺延按该卡逐路径同一性规则复用。

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

**Release boundary:** family child of REL-COV2-01（21/23）；用户已授权逐卡 reviewed signed+DCO 直推 main，但不写 R、不 tag、不调用 release。

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

**Full-suite trigger:** family child 不自行发布；按 ER-13 对最终改动重新判定 T-1/T-4/T-6。若命中 shared single-source helper、测试基建或 focused 信号不足，则在本卡 A/B 阶段运行双 env 全量 Nextest；无论是否命中，COV2-REL-01 首次 T-2 均须在最终 bumped 实现树上重跑；后续仅上游纯 patch 顺延按该卡逐路径同一性规则复用。

**Dependencies:** DEP-COV2-029-BASE（必须包含FIX-OG-PARTIAL-LIVE-01实际五源交接，不等正式049 C/D）； DEP-COV2-029-BASE（父三卡final source重基，shared I–I串行，非native能力）； COV2-12；消费前驱实际最终 A/B、适用 focused/full、fmt/strict、单 Claude PASS 与 signed+DCO main 的 source-qualified 文件 SHA/mode；不以其尚待 C/D 的 `locally-accepted` 冒称 complete。

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

**Release boundary:** family child of REL-COV2-01（22/23）；用户已授权逐卡 reviewed signed+DCO 直推 main，但不写 R、不 tag、不调用 release。

**C/D coverage from:** COV2-REL-01；发布点的 exact-source C/D 绿后才能 `done/complete`。

**Granularity:** `type=implementation; axis=候选 live Session 权限防火墙; recovery=forward-only; complete=yes; self-contained=yes; AC=8/8; VER=2/8; landing=2; prod-files=2; scope=M; deps=DEP-COV2-029-BASE,COV2-12; writeset=串行于前驱; release=family child; split-from=COV2-12; exception=N/A`

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

**Full-suite trigger:** family child 不自行发布；按 ER-13 对最终改动重新判定 T-1/T-4/T-6。若命中 shared single-source helper、测试基建或 focused 信号不足，则在本卡 A/B 阶段运行双 env 全量 Nextest；无论是否命中，COV2-REL-01 首次 T-2 均须在最终 bumped 实现树上重跑；后续仅上游纯 patch 顺延按该卡逐路径同一性规则复用。

**Dependencies:** DEP-COV2-029-BASE（父三卡final source重基，shared I–I串行，非native能力）； COV2-15；消费前驱实际最终 A/B、适用 focused/full、fmt/strict、单 Claude PASS 与 signed+DCO main 的 source-qualified 文件 SHA/mode；不以其尚待 C/D 的 `locally-accepted` 冒称 complete。

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

**Release boundary:** family child of REL-COV2-01（23/23）；用户已授权逐卡 reviewed signed+DCO 直推 main，但不写 R、不 tag、不调用 release。

**C/D coverage from:** COV2-REL-01；发布点的 exact-source C/D 绿后才能 `done/complete`。

**Granularity:** `type=implementation; axis=候选 import Session 权限防火墙; recovery=forward-only; complete=yes; self-contained=yes; AC=8/8; VER=2/8; landing=2; prod-files=2; scope=M; deps=DEP-COV2-029-BASE,COV2-15; writeset=串行于前驱; release=family child; split-from=COV2-15; exception=N/A`

### Task COV2-19：sealed-native v2 identity 生产提交

**Task type:** implementation

**Lifecycle / Acceptance:** in-progress / locally-accepted（8/8 AC、5/5 VER、16 项继承回归、独立 replay 与架构守卫均已具名通过；nightly fmt、strict Clippy、diff check 与双 env full exit0；唯一 Claude Code PASS，P0/P1/P2=0、P3=3；signed+DCO main `354ff768c406bdad74607da62002bff810147c24` 已 Push、remote 同 SHA；家族 C/D、真实 Source A 未完成）

**Description:** 自 COV2-03 拆出。经既有 native proof/binding 验证的 `NativeProven` payload 借用口构造 version-keyed v2 identity plan；把该 plan、版本化 claim sidecar、私有 identity-v2 record/map 与 ref/catalog/claim 提交绑定为一次真实 HistoryManager 事务。本卡交付封闭 `NativeVersionedReasoningCoveragePlan::{matches_payload, bind_claims, turn}` 只读消费面、`NativeVersionedReasoningClaimCommitPlan::new` 封闭构造器，以及该 commit plan 的 `reasoning()`/`versioned()` 只读访问器；COV2-23 只能在本卡最终 signed main 的 API 已审且具名 VER 通过后消费，隔离候选不算交付。`matches_payload` 比较 native proof/opaque artifact 元数据和 session/provider 的 source tag，不证明整个 redacted payload 对象逐字节相同，也不签发 Source A。v1 原路径和冻结字节保持。

**Out of scope:** 不开放 arbitrary bytes/普通 read context 的生产写权限；不修改 native issuer/locator/注册，也不把 synthetic proof 判为真实 Source A；不实现 pending artifact SQL/codec/replay；无 DDL。

**Prestart evidence（历史锚点）：** `src/internal/ai/capture/checkpoint.rs:494,576–665,1130–1195`（既有 source binding、v1-only `ReasoningCoveragePlan`、request）；`src/internal/ai/coverage_gate.rs:167–203,373–390,2339–2520`（versioned claim sidecar 与 v1-only identity companion）；COV2-14 dual-map codec/reader，COV2-03 versioned marker/ref fence。最终源码锚点和 hash/mode 见下。

**ER-02 refreshed source anchors（2026-10-13 22:43 UTC，待最终评审固定）：** `checkpoint.rs:1123,1284,1995,2665,3314,3396,3707,3844,3951,4239` 对应封闭 source、request、replay expectation、marker 前认证、注册竞态与 VER1；`coverage_gate.rs:2586,2611,2703,4236,4512,4705,4931,5067` 对应 companion 构造、重放认证、真实 ref-CAS 适配及额外安全回归/VER2–5。当前两生产文件均 mode `0644`；SHA-256 分别 `0a0da9757c04595817c4d083c1739a5323855b2f197b8509e814e569b012c0f2` 与 `34332a09e5fe9dcd95cc587c239308b2557fafc66ef92fc8837d57a7914df1e0`。三份行为文档 mode 均 `0644`，`coverage-v2.md` / `agent.md` / `COMPATIBILITY.md` SHA-256 分别 `2b4cdc0b91b912720c75e1f24b9a66848af5e637386b9be1c635000678ba55b4` / `f591267ac2b521a389057f0cbd3edbb3e847bf2b51f5e2f04e721729708ead58` / `4c344205a4c364a1b284d3c7d7015d24ff78a926d9232daf3cadef703805a227`；新增架构守卫 `tests/compat/agent_architecture_guard.rs:9485,9573` 固定四个同模块借用点，修订前 SHA-256 `46d67ec907e09bd8982ef5713d03cdf0cf4419ddc1d0346511fe6d85c73bbae3`、修订后 SHA-256 `5b91331db4abb2358f1d03975ea0ec7790e5ec3e560e670d96cdc6b15cd9b753`、mode `0644`；计划/状态文件的自指哈希不写入自身，最终评审 manifest 对八文件逐一固定 SHA/mode。

**Prestart G-09 receipt:** 25 卡三文件协调的最终字节唯一 Claude Code PASS，P0/P1/P2=0，raw JSON SHA-256 `0575dd35ddb3503021d34181df6651362b0525c2c0113020d344ba8c4044781b`；独立 signed+DCO main `3e37e55dd781e44532c6463a1a0cec484c0bb759` 已 Push，开工时本机 `HEAD=origin/main` 均为该 SHA。此收据仅解除规划前置；COV2-19 实现、验收和家族 C/D 独立记账。

**Acceptance criteria:**

- [x] AC1：v2 identity production plan 仅能由已通过既有 `VerifiedCaptureBinding` 的 `CheckpointRedactedPayload::reasoning=NativeProven` 封闭借用构造；Ordinary/Unmapped/Partial 不授予生产写权。`VerifiedCaptureBinding` 复验 capture session/provider；当前 scope 围栏由既有 checkpoint/claim 门处理，均不在本卡重复计为新谓词。
- [x] AC2：即使另一合法 `NativeProven` payload 具有相同 capture session/provider，`NativeVersionedReasoningCoveragePlan::matches_payload` 的 source tag 不匹配时，生产 writer 也必须在写前拒绝该 companion；本 AC 仅计这一新同源比较结果。
- [x] AC3：同一 `NativeProven` batch 导出的 source→versioned identity→claim 是完整、无孤儿的 canonical 关联；`NativeVersionedReasoningCoveragePlan::bind_claims` 拒绝与该关联不等的 `VersionedLiveClaimCommitPlan`，封闭 companion 构造和事务应用消费同一结果。
- [x] AC4：生产 v2 identity record 仅由 COV2-07/14 已审的封闭 v2 record 构造/认证 API 产生。
- [x] AC5：生产 v2 identity map 仅由 COV2-14 已审的封闭 `identity-v2` map carrier 构造/认证 API 写入。
- [x] AC6：封闭 source plan 的 `turn(version,key)` 对同名 logical key 分别返回与所请求 v1/v2 版本匹配的 identity，不能退化为裸 key 查询；双 map 同 key 共存已由 COV2-14 AC3/VER2 验收，作为本卡继承回归而非新增谓词。
- [x] AC7：真实生产 writer 必须消费已审双 map 共用的 4096-entry/1-MiB encoded 容量 guard，并在可达的越界提交前拒绝，catalog/claim/ref/map 均无可见变更；entry-only 算法边界由同一 guard 的 COV2-14 已审精确测试在本卡最终源码复跑，不伪造无法到达的合法生产输入。
- [x] AC8：真实 `TracesCheckpointStore.write`/HistoryManager ref-CAS 注入 identity-v2 SQL fault 后，ref、catalog、revision、两版 claim/map、object-index 同事务原子回滚且无成功 ACK。

**独立谓词计数：** 本卡 8 个新集成结果：AC1 只计已有 native 权限门下的封闭生产构造，`VerifiedCaptureBinding` 的 session/provider 校验和当前 scope 围栏为继承前置；AC2 只计 source-tag 异源重组拒绝；AC3 计 canonical claim 集等值；AC4/5 各计 record/map 生产；AC6 只计版本化 identity 查询；AC7 计真实 writer 的越界前拒绝；AC8 计同事务故障回滚。COV2-14 的同名 key 双 map 共存、双 map entry-only 容量边界及既有 capture binding 由具名继承回归覆盖，不把它们塞进 AC2/6 或增加本卡 VER 计数。AC3 的同一 canonical source→identity→claim 集合由 source factory 的 `inputs⊆known_keys`（每个 native proof key 都有 normalized turn）、每个 known turn 的 version-keyed identity 和 `bind_claims` 完整集合等值共同证明；因而同源 batch 的 `pending_logical_turns()` 不能留下无 identity/claim 的孤儿，这是继承 v1 安全约束的验证，不是第九个新行为。AC3/4/5/7 消费前驱已验收的封闭值对象、codec/MAC 和共同容量 API；若最终实现需要新字段级权限分支、新 codec/MAC、独立容量判据，或不能用上述同源关联证明 no-orphan，先按 G-03/G-09 拆卡并重审，不合句或豁免。

**Verification:**

- [x] VER1→AC1/2：`source .env.test && cargo test --lib internal::ai::capture::checkpoint::tests::cov2_v2_native_plan_rejects_unbound_source -- --exact`（1/1；AC1 重验既有 NativeProven/`VerifiedCaptureBinding` 门的非 native 状态与 session/provider 漂移，不作新的 AC2 谓词；AC2 直接断言 `source.matches_payload(&payload,session,provider)` 同源 true、另一 opaque proof 不同但仍合法的 native payload 为 false，并由真实 checkpoint writer 的 request binding 拒绝该异源 companion、零可见写。须在最终源码实际具名 1/1，隔离候选不计 PASS）。
- [x] VER2→AC3：`source .env.test && cargo test --lib internal::ai::coverage_gate::tests::cov2_native_v2_claim_tuple_binding -- --exact`（1/1；真实 sidecar 与 `NativeVersionedReasoningClaimCommitPlan::new` 调用 `source.bind_claims(&versioned)`，证明 canonical 集合等值；version/key/digest/completeness 的缺失、额外、漂移各自变异后，逐项断言有名失败且无 companion。其同一具名测试还须直接遍历源 batch 的 `pending_logical_turns()`，断言每个 key 在 source 的 version-keyed identity 与 bound claim 中均有对应，并用 proof key 不在 normalized turns 中的反例直接调用 source factory、断言拒绝。额外项及 no-orphan 断言在最终源码补齐后才可记 1/1 PASS，不能凭隔离候选 selector 名称放行）。
- [x] VER3→AC4/5/6：`source .env.test && cargo test --lib internal::ai::coverage_gate::tests::cov2_v2_identity_domains_and_same_key -- --exact`（1/1；经生产 companion 写/读 record/map 并分别断言两个 MAC domain；AC6 直接以 `source.turn(V1,key)`/`source.turn(V2,key)` 的 slot/kind/descriptor MAC 对照相应已认证 record，验证版本查询不退化为裸 key。双 map 同 key 共存是 COV2-14 继承回归，不在 AC6 重计；最终 reader 对 v2 whole-map MAC 篡改及 v1/v2 存储值互换须 fail closed、零 claim/ref 改动，复原后读回；保留 v1 frozen oracle。synthetic 不是 Source A）。
- [x] VER4→AC7：`source .env.test && cargo test --lib internal::ai::coverage_gate::tests::cov2_v2_identity_aggregate_cap -- --exact`（1/1；真实 `TracesCheckpointStore.write` 调用已审双 map aggregate guard，以 encoded-byte 越界证明提交前拒绝且 ref/catalog/claim/map 均无可见变更；synthetic signed map 仅构造合法留存基线，不能替代真实 crossing writer）。
- [x] VER5→AC8：`source .env.test && cargo test --lib internal::ai::coverage_gate::tests::cov2_native_v2_history_fault_atomicity -- --exact`（1/1；实际 HistoryManager/ref-CAS 注入 identity-v2 SQL fault，逐表核 ref/catalog/revision/两版 claim-map/object-index 原子回滚与零成功 ACK；后续 retry 属 ER-04 回归，不得仅直调 `TracesTxnExtra::apply`）。

**Mandatory ER-04 inherited regressions（不增加本卡 G-03 AC/VER）：** 在 COV2-19 最终源码重跑 COV2-14 VER2 `source .env.test && cargo test --lib internal::ai::coverage_gate::tests::cov2_dual_private_map_same_key_coexists -- --exact`（同名 logical key 双 map 1/1），以及 COV2-14 VER3 `source .env.test && cargo test --lib internal::ai::coverage_gate::tests::cov2_private_map_aggregate_entry_and_byte_caps -- --exact`（同一 guard 的 entry-only/encoded-byte 边界 1/1）。合法 record/map 的 slot、context logical key、descriptor MAC 各至少 91 ASCII bytes，record MAC 至少 89 bytes，仅四段每 entry 已 ≥362 bytes；4097 条 JSON 在 entry cap 前必越过 1 MiB byte cap，故不伪造真实 entry-only writer 路径或放宽限额。另重跑 COV2-03 VER5 `source .env.test && cargo test --lib internal::ai::traces::tests::cov2_completed_terminal_receipt_runtime_branch -- --exact`（1/1）；在本卡 VER5 故障场景确认预注册 marker 不属 ref-CAS 回滚集合，仍可由恢复/重试读取处理且无假成功 ACK。上述继承门已在最终生产源码的 16 项 inherited/v1/RG-04 清单中逐项 1/1，不计本卡第六 VER或新的 AC。

**Mandatory ER-04 architecture regression（不增加本卡 G-03 AC/VER）：** `source .env.test && cargo test --test compat_agent_architecture_guard reasoning_bound_identity_accessor_has_closed_adapter_caller -- --exact` 须 1/1。原封闭调用者白名单没有本卡的 source-tag 与 native-v2 planner，导致首次 `umask 0022` 双环境全量在 10084 项中 10083 PASS、该守卫 1 FAIL；原始失败日志 SHA-256 `bac1f752929588e69fc1aed1e7c00d7e00204f807cead8a1ad2ae4f71152e402`。本卡在已修订的精确写集内固定四个 `checkpoint.rs` 借用点，拒绝其它 owner、路径及重复借用；旧全量是 FAIL 历史，修复后该 exact 已 1/1（raw SHA-256 `74fb7cdf69876d0778241f58b46e3413a479c509b2ee844f6d16bba869c9694e`）；nightly fmt、strict Clippy、diff check 与双环境 full 均已重跑通过，唯一 Claude 最终评审仍待。

**Replay safety regression（不增加 G-03 AC/VER）：** `source .env.test && cargo test --lib internal::ai::coverage_gate::tests::cov2_native_v2_replay_requires_authenticated_identity -- --exact` 必须 1/1：先由普通 versioned writer 以同 ID/claim 完成，native companion 重放与注册竞态均不得假 `AlreadyExists`；普通完成但清理失败所留下的 marker 在拒绝 native 假重放后必须逐字节保留；同源 native 已认证记录的精确重放仍可 ACK；相同 version/key slot 被后续 native checkpoint 覆盖后，旧 checkpoint 的重放保守冲突但已提交数据不变；私有 whole-map MAC 篡改须返回 store failure 和无私有正文的 doctor 诊断。此回归针对预审发现的额外重放路径，不把它计为 Source A 或第六 VER；最终生产源码具名 1/1，原始日志 SHA-256 `d813fc03edd3e8e22a47a780664cc53f10e2ca4321c65ba975073567ce1518fb`。

**Interim local evidence（2026-10-13 22:05 UTC，非最终源码）：** 五项本卡 VER 与 16 项 inherited/v1/RG-04 selector 各 1 passed/0 failed/0 ignored，21 份原始日志清单 SHA-256 `f4a8c3dcf1904abda53a155a2f0e9451f9bee906e457e23f727f57c363332a77`；追加 replay 回归另 1/1，日志 SHA-256 `6538c105ade4645c70b1da7a219d55b98cc2b74360535649587e94ae06c615c3`。nightly fmt exit0。随后将 native 重放认证提前至旧 marker 清理之前，故上述通过结果须在最终源码重跑，不倒记为最终 PASS。预审指出普通 v2 同 ID/claim 可能被误 ACK，已在精确两生产文件写集内修复；旧候选两次编译签名错误、VER1 空 batch fixture 失败与发现缺陷后中断的非最终 full 均保留为失败/未完成历史。当前 strict Clippy、最终双 env full、唯一 Claude 和 signed main 尚待；synthetic 不证明 Source A，C/D 仍待家族 release。

**Pre-Clippy focused evidence（2026-10-13 22:17 UTC，非最终源码）：** 五项 VER 与上述 16 项 inherited/v1/RG-04 精确 selector 再次各 1 passed/0 failed/0 ignored，21 份原始日志清单 SHA-256 `05f621c9b737c7c52d7d78868c9a3f474c7380201bae799e0beaf10f13605891`；新增 replay/marker 保留回归 1/1，原始日志 SHA-256 `657e2a16df7c2ede6c98b7bd0827dfc0371caa1434fd1a91c0ff3fd415d786af`。nightly fmt 与 `libra diff --check` exit0。strict Clippy 报新增 replay 函数 8 参数超过阈值，已将两个关联预期收束为一个结构体；该源码变更要求重新跑 focused 与 Clippy，先前通过的日志不倒记最终 PASS。双 env full、唯一 Claude 与签名 main 亦尚待。该确定性 synthetic 门不等于真实 OpenCode/native Source A，Linux 功能由用户后验。

**Pre-review focused evidence（2026-10-13 22:20 UTC，非最终源码）：** 结构体修订后的五项 VER 和 16 项 inherited/v1/RG-04 在当时源码逐项 1 passed/0 failed/0 ignored，21 份日志清单 SHA-256 `40c7d4685732e2b8fecb76244f9b086d8e22c4202f2320472b671edfe2416434`；额外 replay/marker 回归 1/1，日志 SHA-256 `702cb05cc04f6d6324db9b38b1104537d53cd354e64f08d7bb6edd203b8f4c0f`。nightly fmt exit0；此时 strict Clippy、双 env full、唯一 Claude、signed main 待运行，不把旧失败/中断日志计绿。

**Strict gate（2026-10-13 22:24 UTC，非最终源码）：** `cargo clippy --all-targets --all-features -- -D warnings` 在当时源码 exit0（已修此前八参数告警）；`cargo +nightly fmt --all --check` exit0。双 env full 后因 Claude 预审意见中断，不能计 PASS；其后又修改测试及错误分类，strict 门须重跑。

**Preliminary Claude Code feedback（2026-10-13 22:27 UTC，非正式 PASS）：** 唯一 Claude Code 全读七文件 diff、`AGENTS.md` 和 COV2-19 卡，给出 P0=0/P1=0/P2=1/P3=3，原始 JSON SHA-256 `1eb510eb50d01e182cb810afd860ab9bbebc87bbf36362340d2150ca39ac159a`。P2 是同槽后续 checkpoint 覆盖当前 map 后旧重放保守冲突缺测试与精确说明；P3 指出损坏 map 诊断与本卡旧文案。已补同槽后续 checkpoint、MAC 篡改精确回归，并收窄三份文档及旧 full-trigger 语句；因修改了测试及重放错误分类，旧 focused/Clippy 通过和已中断 full 不计最终 PASS，最终字节还须重测与 Claude 复审。

**Post-review focused evidence（2026-10-13 22:39 UTC）：** 同槽覆盖场景先前用两个 complete claim 不能取得第二次预留，已改为生产允许的 incomplete→complete 升级并直接断言第二 revision。当前源码五项 VER 与 16 项 inherited/v1/RG-04 再次各 1 passed/0 failed/0 ignored，21 份日志清单 SHA-256 `1b961dc1b93e1bbe4cd1c41e4f4bbd46d8a5375759947355573dc717a5c5d973`；额外 replay/marker/同槽覆盖/损坏 map 回归 1/1，日志 SHA-256 `d813fc03edd3e8e22a47a780664cc53f10e2ca4321c65ba975073567ce1518fb`。中途 helper 返回 `anyhow::Error` 的类型断言编译失败及两个 complete 的预留断言失败保留原始失败历史，修复后才计当前源码 1/1。nightly fmt 与 `libra diff --check` exit0；strict Clippy、双 env full、最终 Claude、signed main 仍待。

**Clippy 与预审复核（2026-10-13 22:43 UTC）：** 当前源码 `cargo clippy --all-targets --all-features -- -D warnings` exit0，nightly fmt 与 diff check exit0。唯一 Claude Code 对先前 P2/P3 作只读预审复核，**PRELIM_CLEAN**、P0/P1/P2=0、P3=1，原始 JSON SHA-256 `4407a17a73a3ca0bef86cabc0e38753ea8722b5425dea52cf9f2079f3ac72268`；确认同槽保守冲突是具名测试和文档固定的安全契约、历史 ACK 要求新增持久证据及独立卡，剩余 P3 仅建议在测试中另加直接 helper 断言。此为预审而非正式 PASS；该段是预审时点；随后架构守卫补入第八文件并重跑双 env full，最终八文件仍须正式唯一 Claude 评审和 signed main。

**Final pre-review A/B evidence（2026-10-13 23:57:33 UTC，签名前快照）：** 五项 COV2-19 VER 与 16 项 inherited/v1/RG-04 各 1 passed/0 failed/0 ignored（21 份日志索引 SHA-256 `1b961dc1b93e1bbe4cd1c41e4f4bbd46d8a5375759947355573dc717a5c5d973`）；独立 native replay/marker/同槽覆盖/损坏 map 回归 1/1（raw SHA-256 `d813fc03edd3e8e22a47a780664cc53f10e2ca4321c65ba975073567ce1518fb`）；架构守卫精确 1/1（raw SHA-256 `74fb7cdf69876d0778241f58b46e3413a479c509b2ee844f6d16bba869c9694e`）。`cargo +nightly fmt --all --check`、`cargo clippy --all-targets --all-features -- -D warnings`、`libra diff --check` 均 exit0；strict Clippy 原始日志 SHA-256 `cc93332ff41e2ad92f29da9ea767d0939586bcfc89b6d79f6f25317c74191748`。`umask 0022` 下双 env full **Summary [1242.578s] 10084 tests run: 10084 passed (15 slow, 1 leaky), 23 skipped**，exit0、raw SHA-256 `20378e04b5c585c954421780520a6a62ebbf1f6c7ba0336db40dd76541058e43`。唯一 leaky 项是与本卡写集无关的 `command::cat_file_test::test_cat_file_ai_type_nonexistent`，Nextest 仍计入通过；先前 umask077 引发的三项 mode 假失败及修订前守卫 1 FAIL 保留为失败历史，不倒记本轮结果。八文件最终 diff 的唯一 Claude Code 正式评审、signed+DCO main Push 与 remote 核验仍待；其实际收据在下一卡开工记录。本机功能只验 macOS；synthetic 不证明真实 Source A，家族 C/D 未运行。

**Final COV2-19 source handoff（2026-10-14 00:07:12 UTC）：** 本卡最终八文件 manifest SHA-256 `f400931268c37875cd7131ab3c3bbf4e6178a6b4fa4d55129ee905456330aa62`；逐文件 SHA/mode 由 root 对实际字节、diff 与 manifest 核验为 8/8 match。唯一 Claude Code 在完整八文件 diff 上正式 **PASS**，P0/P1/P2=0、P3=3，raw JSON SHA-256 `64068902908f4c521e380dc7e5cf593134eaa99697de208344b65f75b3c3caa8`；三项非阻断 P3 为容量拒绝测试的错误类型断言、deadline 分类诊断、plan 预计算 O(turns×inputs)，均在未开放生产接线的下游卡继续观察，不将其冒称已修。`libra commit -s` 提交 `354ff768c406bdad74607da62002bff810147c24` 内含 PGP `gpgsig` 与 DCO trailer，已 Push 到 `main`，本机 `HEAD=origin/main` 与 GitHub main 均同 SHA；仅保留两个不属本卡的原脏计划文件。COV2-19 源码就绪为 COV2 第七张（7/25=28.00%，正式0/25），下一卡 COV2-20；本卡家族 C/D 与 Source A 仍待，不以 synthetic、leaky 或 skip 替代真实 native 归档证明。

**Full-suite trigger:** 本卡改动共享 checkpoint/replay helper，命中 ER-13 T-1；本卡五项 focused selector、冻结 v1 golden/contract oracle、nightly fmt 与 strict Clippy 须在最终源码实测，双 env 全量 Nextest 与唯一 Claude Code 最终字节 PASS 亦为本卡 A/B 门。COV2-REL-01 首次 bumped 实现树全量仍须独立运行；后续仅上游纯 patch 顺延按该卡逐路径同一性规则复用。新增 selector 已实现；预审后为覆盖同槽后续 checkpoint 的保守重放行为而补测试，先前 focused/full 日志不能倒记最终 PASS。

**Dependencies:** COV2-03；消费最终 A/B、适用 focused/full、fmt/strict、唯一 Claude PASS 与 signed+DCO main SHA/hash/mode；不把继承 C/D 的 locally-accepted 冒称 complete。

**Deliverables:** 无独立文件；实现、测试、文档与逐文件 SHA/mode、review/门结果写回本计划正文，随唯一发布点交付。

**Implementation write set:** I-core 精确五文件：`docs/development/plan/plan-20261013-coverage-schema.md`; `docs/development/plan/plan-status.md`; `docs/development/tracing/coverage-v2.md`; `docs/development/tracing/agent.md`; `COMPATIBILITY.md`；本卡新增精确路径：`src/internal/ai/capture/checkpoint.rs`; `src/internal/ai/coverage_gate.rs`; `tests/compat/agent_architecture_guard.rs`（仅更新封闭 disposition identity 借用口的架构守卫）。其余测试写在上述两个源码文件的现有 test module，不授权 glob、`history.rs` 或 `sql/**`。

**Release write set:** N/A；C/D 继承 COV2-REL-01。

**Files likely touched:** 上述八个 I 文件。

**Docs and compatibility impact:** `docs/development/tracing/coverage-v2.md`; `docs/development/tracing/agent.md`; `COMPATIBILITY.md` 逐文件陈述 sealed-native v2 identity、v1 冻结和 Source A 尚未证明；若新增公开差异，先按 ER-06a 修写集。无新 `--test` target，`tests/INDEX.md` 无写权。

**Rollback mode:** forward-only；失败不重写 v1 或既有私有 map，签名+DCO 新 main 修复并重跑下游。

**Migration and rollback:** 无 DDL；仅使用现有 versioned claim 与 COV2-14 双 map。若需要数据库 schema/migration，先加角色隔离迁移卡及旧/新二进制门、修写集和 G-03/G-04。

**Security and privacy:** v2 writer 不接受任意来源或 read context；native field 正文不得进入 ordinary/typed/log/error/计划证据；synthetic sealed fixture 仅证明交易正确性，不证明已注册 OpenCode Source A。

**Performance budget:** 沿用 COV2-14 双 map aggregate 4096 entries/1 MiB，bounded read before allocation；变更 hot path 时验证无无界扫描。

**Estimated scope:** M（AC=8/8、VER=5/8、landing=2、production files=2）。

**Version increment:** N/A；此卡不发版。

**Release boundary:** family child of REL-COV2-01（7/23）；用户已授权逐卡 reviewed signed+DCO 直推 main，但不写 R、不 tag。

**C/D coverage from:** COV2-REL-01；其真实 exact-source C/D 通过后本卡才可 done/complete。

**Granularity:** `type=implementation; axis=sealed-native v2 identity 生产提交; recovery=forward-only; complete=yes; self-contained=yes; AC=8/8; VER=5/8; landing=2; prod-files=2; scope=M; deps=COV2-03; writeset=严格串行; release=family child; split-from=COV2-03; exception=N/A`

### Task COV2-20：版本化 live reservation 只读验领

**Task type:** implementation

**Lifecycle / Acceptance:** locally-accepted / 本机 7/7 AC、3/3 VER 与冻结 v1 回归各 1/1；nightly fmt、diff check、strict Clippy、双 env full、唯一 Claude PASS 与 signed+DCO main 已通过；当前精确提交的 Windows job `115625055658` 已 success，完整 CI 与家族 C/D 未闭合

**Description:** 自 COV2-03 拆出并经 COV2-19 接续，进一步拆出 COV2-21 的 retain 行为；本卡只提供 `verify_versioned_reserved_live_claims_with_conn(conn,scope,&VersionedLiveClaimCommitPlan)`。只读取 sealed 有序 (version,key,digest,completeness,owner,fence,previous_revision,state) 预留 tuple；使用调用方 `ConnectionTrait` 的事务视图，不更新 lease。v1 API 保持。

**Out of scope:** 不创建 pending artifact/envelope，不 retain、不改 DDL、native issuer/Source A，也不以第二连接看似成功代替调用方事务。

**Current evidence / ER-02 source anchors（2026-10-14 00:07:12 UTC 的修订前快照）：** COV2-19 最终 signed+DCO `main` `354ff768c406bdad74607da62002bff810147c24`、remote 同 SHA；其 `src/internal/ai/coverage_gate.rs` 前驱 SHA-256 `34332a09e5fe9dcd95cc587c239308b2557fafc66ef92fc8837d57a7914df1e0`、mode `0644`，已有 versioned sidecar 与 v1 reservation helper。本卡初始候选在 `coverage_gate.rs:252` 加只读验领入口、`:3430,3507,3554` 加三项具名测试；修订前候选 SHA-256 `4b3a7444b417a4a19bced2b342e356671e1a10591f53ed01685ae6a2306ab952`、mode `0644`，与隔离补丁 r3 manifest 一致。`pending.rs` 的真实 writer 本卡只读；隔离构建/重放不是 Cargo、Claude 或本机验收 PASS，以下门须在本工作区最终字节实测。

**Acceptance criteria:**

- [x] AC1：用每个 claim 的实际 `(version,key)` SELECT 行。
- [x] AC2：stored reservation 与 sealed canonical ownership tuple 精确等值才通过。
- [x] AC3：同一 logical key 的 v1 与 v2 两版独立验领。
- [x] AC4：租期过期而 owner/fence/revision 仍属原持有人时允许验领。
- [x] AC5：旧 owner/fence/revision/digest/completeness/state 拒绝。
- [x] AC6：成功与拒绝均不更改任何 claim 行。
- [x] AC7：读取调用方同一 connection/transaction 内尚未提交的状态，不打开第二 pool transaction。

**独立谓词计数：** AC2/5 共用同一 canonical ownership tuple 的等值判断；逐字段变异是该判断的测试向量，不新造分支。现存 v1 reservation 回归是额外 mandatory ER-04 A 门：`source .env.test && cargo test --lib internal::ai::coverage_gate::tests::live_takeover_increments_fence_and_blocks_stale_commit -- --exact`（1/1；当前 UNRUN，0 matched/skip 非 PASS）；不能用本卡新 selector 代替。

**Verification:**

- [x] VER1：AC1/2/5：`source .env.test && cargo test --lib internal::ai::coverage_gate::tests::cov2_versioned_reserved_live_claims_verify_actual_tuple -- --exact`（真实 DB，1/1）。
- [x] VER2：AC3/4：`source .env.test && cargo test --lib internal::ai::coverage_gate::tests::cov2_versioned_reserved_live_claims_verify_same_key_and_expired_lease -- --exact`（1/1）。
- [x] VER3：AC6/7：`source .env.test && cargo test --lib internal::ai::coverage_gate::tests::cov2_versioned_reserved_live_claims_verify_is_read_only_in_caller_transaction -- --exact`（修改调用方未提交行，helper 看到该值；拒绝后行不变；1/1）。

**Focused evidence（2026-10-14 00:11:45 UTC，Windows 修订前快照）：** 当时六文件写集中的唯一生产文件 `src/internal/ai/coverage_gate.rs` SHA-256 `4b3a7444b417a4a19bced2b342e356671e1a10591f53ed01685ae6a2306ab952`、mode `0644`；VER1/2/3 与 inherited v1 精确 selector 各 1 passed/0 failed/0 ignored，原始日志索引 SHA-256 `09bf9dff38d8cc5a23b16692a977d9c1eae0100c8bbbc22793fda72c9ce8603f`。`cargo +nightly fmt --all --check` 与 `libra diff --check` exit0；当时 strict Clippy、双 env full、唯一 Claude 与 signed+DCO main 仍待。此处是 deterministic DB/SQLite fixture，不是 OpenCode Session、真实模型或 native Source A。

**G-10/I 精确写集修订（2026-10-14 00:18:49 UTC）：** 前驱 COV2-19 的 signed main `354ff768c406bdad74607da62002bff810147c24` 在 Windows `compat-scoped-input-windows` job `115606533849` 的 `cargo test --lib -- --list` 下，由 `RUSTFLAGS=-D warnings` 报出八处 `dead_code` 编译错误；日志证实并非临时网络失败，不能用 `gh` 原样重跑冒充修复。原因是 native-v2 预留 API 的现有消费者位于 Unix-only 测试，旧 `#[cfg_attr(not(test), allow(dead_code))]` 在 Windows 的 `lib test` 构建失效。本卡在尚未最终验收和评审前追加精确生产路径 `src/internal/ai/capture/checkpoint.rs`，只允许平台限定的 cfg 修复以及同一类 `coverage_gate.rs` 的 cfg 修复；AC/VER 功能判据不变，生产文件数从 1 改为 2。最终字节重跑 focused、fmt、strict Clippy、双 env full 和唯一 Claude，并以新提交的 Windows scoped job 实际结果验收；旧失败保留，不计 PASS。

**最终候选 A/B 中间收据（2026-10-14 00:27:29 UTC，签名前）：** Windows 前滚补丁仅把八处预留 API 的 `cfg_attr` 限定为非 Unix 测试构建时允许暂未消费，并把一个仅 Unix 测试使用的合成 turn helper 标记 `cfg(unix)`；补丁 SHA-256 `04514909988f8892cf74750f03cf24a448bba0061cdf217a28b51904258f32f8`。当前 `coverage_gate.rs` SHA-256 `1104ba09ef8b638b34a59c6844eb74a975cc05fd1343d66d74f4e00341c84ac9`、`checkpoint.rs` SHA-256 `cae575f0d15662ffcc72c2890ace5134aee1d657831fc750e758d362011acb69`，均 mode `0644`。修订后 VER1/2/3 与 inherited v1 精确 selector 各 1 passed/0 failed/0 ignored，原始日志索引 SHA-256 `5dd3034d489ae5139e8ce1ce6af4f1703e6b01d08a780e19019ed0a4271255ef`；nightly fmt、`libra diff --check`、strict all-targets/all-features Clippy `-D warnings` 均 exit0（Clippy raw SHA-256 `1998ed3e2de86c9e7f782002681ed6b58c48bf3527697c5bfa1486a2fa7e3ba3`）。双 env full 正在最终源码运行；新提交 Windows CI、唯一 Claude 与 signed main 尚未发生。

**最终源码双 env full 收据（2026-10-14 01:00:52 UTC，签名前）：** `umask 0022`、`source .env.test`、`source .env.live-test` 后执行 `cargo nextest run --all --no-fail-fast --retries 2`，exit0；**Summary [1069.256s] 10087 tests run: 10087 passed (13 slow, 4 leaky), 23 skipped**，原始私有日志 SHA-256 `6efa8a7d8461a9f4a5740661c5e99cb276804fe6d2b2a80d9264a9513da2124f`。四个 leaky 名称保留在原始日志，不视为失败或额外 OpenCode Session 验收；23 skipped 不计本机 Session 通过。此收据与修订后四项 focused、fmt/strict、源码 SHA 绑定；唯一 Claude、signed main 与新提交 Windows CI 仍待。

**最终源码/单审交接收据（2026-10-14，COV2-21 开工前复核）：** 最终七文件 manifest SHA-256 `cb769d9a65e5b6db868d2e62fd6df1cc28ff33eb9e071ebab196827785a90a41`，root 已在评审前后独立复核实际 SHA/mode/diff。唯一 Claude Code Sonnet 对最终七文件差异给出 **PASS**，P0/P1/P2=0、P3=4，原始 JSON SHA-256 `6f3381d9061e72afebfe32f9714331ada567665d33eb0b045782fd06646667a5`；四项 P3 为 scope/fence 负例、fail-closed 分支、Windows 待验与 4096 条上限内逐行查询。signed+DCO main `aa0ea018ddd044f09d62a88428e812a359cef8ed` 已 Push，`libra ls-remote` 与 GitHub main 均同 SHA；后续 COV2-21 消费此前驱。旧 COV2-19 Windows job `115606533849` 是确定性 `dead_code` 失败，不计网络故障。当前提交 CI run `38493804953` 的 Windows job `115625055658` 后续已 success；完整 workflow 仍运行，不得提前记 D 组成功。本卡为 COV2 第八张源码就绪卡（8/25=32.00%），正式仍 0/25。

**Full-suite trigger:** ER-13 T-1：本卡把 versioned reservation 的 canonical ownership 验领新增为共享单一 helper，后续 pending writer 消费，故本卡最终源码必须跑双 env full。先跑本卡 3 个具名 focused selector、相关冻结 v1 回归、nightly fmt 与 strict all-targets/all-features Clippy；再核双 env `cargo nextest run --all --no-fail-fast --retries 2` exit0，最后对完整结果作唯一 Claude Code 最终字节评审。COV2-REL-01 首次 bumped 实现树 full 仍须独立运行；后续仅上游纯 patch 顺延按该卡逐路径同一性规则复用。0 matched、skip、synthetic 或未执行均非 PASS；本机 OpenCode Session/模型复杂验收仍不进入 CI。

**Dependencies:** COV2-19；消费前驱真实最终 A/B、适用 focused/full、fmt/strict、唯一 Claude PASS 与 signed+DCO main SHA/hash/mode；不把继承 C/D 的 locally-accepted 冒称 complete。

**Deliverables:** 无独立文件；API、测试、兼容文档、逐文件 SHA/mode 和 review/验收事实写回本计划正文，随唯一发布点交付。

**Implementation write set:** I-core 精确五文件：`docs/development/plan/plan-20261013-coverage-schema.md`; `docs/development/plan/plan-status.md`; `docs/development/tracing/coverage-v2.md`; `docs/development/tracing/agent.md`; `COMPATIBILITY.md`；本卡新增精确路径：`src/internal/ai/coverage_gate.rs`; `src/internal/ai/capture/checkpoint.rs`（仅上述 Windows `-D warnings` cfg 前滚修复）。现有测试模块可写；未列路径包括 `sql/**`、冻结 decoder、parent 计划均为只读，后续新增路径仍需先修 G-10/I。

**Release write set:** N/A；C/D 继承 COV2-REL-01。

**Files likely touched:** I-core 上述五文件；`src/internal/ai/coverage_gate.rs`; `src/internal/ai/capture/checkpoint.rs`。

**Docs and compatibility impact:** I-core 的 coverage-v2、agent、COMPATIBILITY 只陈述本卡已交付的能力，不提前宣称下游 artifact/replay/Source A 已完成。ER-06a 判定：仅内部 reservation helper 与测试平台 cfg 前滚，不改变 `libra agent` 参数、输出、错误、配置或示例，`docs/commands/` EN/zh-CN 与相邻 `libra-backend` 网站命令文档均 N/A。无新 `--test` target；`tests/INDEX.md` 无写权。

**Rollback mode:** forward-only；失败保持旧 artifact/claim，签名+DCO 新 main 前滚并重跑下游。

**Migration and rollback:** 无 DDL；Repository claim 使用既有 versioned 键。若实际需迁移，先加角色隔离 migration 卡、旧/新 binary 门并修 G-03/G-04/I。

**Security and privacy:** 继承 GC-COV2-02/03；所有错误 fail closed、零假 ACK、零候选明文 sink；真实 native Source A 仍在父计划。

**Performance budget:** 使用已审 claim/map 上限，解析/查询前有界；不得新增无界扫描、无时限重试或私开的第二连接。

**Estimated scope:** M（AC=7/8、VER=3/8、landing=1、production files=2；第二生产文件仅 Windows cfg 前滚修复）。

**Version increment:** N/A；此卡不发版。

**Release boundary:** family child of REL-COV2-01（8/23）；逐卡单 Claude PASS 后 signed+DCO 直推 main，不写 R、不 tag。

**C/D coverage from:** COV2-REL-01；真实 exact-source C/D 通过后本卡才可 done/complete。

**Granularity:** `type=implementation; axis=版本化 live reservation 只读验领及前驱 Windows cfg 前滚修复; recovery=forward-only; complete=yes; self-contained=yes; AC=7/8; VER=3/8; landing=1; prod-files=2; scope=M; deps=COV2-19; writeset=严格串行; release=family child; split-from=COV2-03; exception=N/A`

### Task COV2-21：调用方事务内版本化 reservation retain

**Task type:** implementation

**Lifecycle / Acceptance:** locally-accepted / 7/7 AC、4/4 VER 与五项继承精确测试各 1/1；fmt/strict/full、唯一 Claude PASS 与 signed+DCO main 已交接；家族 C/D 待 COV2-REL-01

**Description:** 自 COV2-20 拆出，提供 `retain_versioned_reserved_live_claims_for_artifact_with_conn(conn,scope,&VersionedLiveClaimCommitPlan)`。使用同一 sealed canonical tuple 和调用方 SQLite transaction 更新双版本 lease；COV2-20 已独立验证只读验领。

**Out of scope:** 不产生 artifact/envelope、persist receipt、旧读端或 replay，不调用 native issuer；不能用模拟 SQL probe 声称真实 artifact 写故障已验收。

**Current evidence / ER-02 source anchors（2026-10-14，开工快照）：** 前驱 COV2-20 signed+DCO main `aa0ea018ddd044f09d62a88428e812a359cef8ed`，远端同 SHA；本卡只改该提交之上的 `src/internal/ai/coverage_gate.rs` 和精确 I-core 文档，`pending.rs`/`pending_payload.rs` 只读。候选 retain 入口在 `coverage_gate.rs:335`，文件候选 SHA-256 `d05c286627e117fddc2f4e311c5fe53946b93d9ee2098ab78962cf718ed0be3f`、mode `0644`；四项新测试在同文件测试模块。此处是候选源码定位，不是验收通过证据。

**Acceptance criteria:**

- [x] AC1：UPDATE 以每个 claim 实际 `(version,key)` 寻址。
- [x] AC2：UPDATE 要求 sealed owner/fence/revision/digest/completeness/state tuple 精确匹配。
- [x] AC3：同一调用方事务将同 logical key 的 v1 与 v2 两行保留到 artifact lease。
- [x] AC4：仅 lease 过期不撤销原 owner 的合法保留。
- [x] AC5：陈旧 ownership tuple 拒绝；调用方回滚后不改变任何持久化行。
- [x] AC6：第二版本 UPDATE 的真实 SQL fault 回滚第一行 lease 更新。
- [x] AC7：调用方同一事务中的后续 SQL fault 回滚两行 lease，helper 不提前 commit。

**独立谓词计数：** AC2/5 消费同一个 sealed tuple 比较；如代码新增独立安全判断则先 G-03 重计。现存 v1 retain 行为为额外 mandatory ER-04 门：`source .env.test && cargo test --lib internal::ai::coverage_gate::tests::artifact_retained_claims_survive_lease_expiry_until_replay_commit -- --exact`（1/1；当前 UNRUN，0 matched/skip 非 PASS）；既有 v1 字节冻结门仍按 ER-04 继承。AC6/7 的事务探针不替代 COV2-22 真实 chunk/header/alias writer 故障。

**Verification:**

- [x] VER1：AC1–4：`source .env.test && cargo test --lib internal::ai::coverage_gate::tests::cov2_versioned_reserved_live_claims_retain_both_versions -- --exact`（1/1）。
- [x] VER2：AC5：`source .env.test && cargo test --lib internal::ai::coverage_gate::tests::cov2_versioned_reserved_live_claims_retain_stale_tuple_rejects -- --exact`（1/1）。
- [x] VER3：AC6：`source .env.test && cargo test --lib internal::ai::coverage_gate::tests::cov2_versioned_reserved_live_claims_second_row_fault_rolls_back -- --exact`（真实 SQLite trigger，逐行核回滚；1/1）。
- [x] VER4：AC7：`source .env.test && cargo test --lib internal::ai::coverage_gate::tests::cov2_versioned_reserved_live_claims_later_sql_fault_rolls_back -- --exact`（调用方事务后续 SQL 失败，不造假 artifact receipt；1/1）。

**Focused evidence（2026-10-14 01:12 UTC）：** 本卡四项 VER、inherited v1 retain、冻结 v1 bytes/digests 与 RG-04 SHA/projection 共九个精确 selector，各 **1 passed/0 failed/0 ignored**；原始私有日志索引 SHA-256 `38c8589ec7654c0e7952494df0dc33a424643002ec028de777bbb6d1a8689679`。此时生产文件 `src/internal/ai/coverage_gate.rs` SHA-256 `d05c286627e117fddc2f4e311c5fe53946b93d9ee2098ab78962cf718ed0be3f`、mode `0644`；`cargo +nightly fmt --all --check` 与 `libra diff --check` exit0。严格 Clippy、双 env full、唯一 Claude 与 signed main 仍待；SQLite 故障注入不是实际 pending artifact writer 验收。

**Strict-quality 收据（2026-10-14）：** 同一生产源码上 `cargo +nightly fmt --all --check`、`libra diff --check`、`cargo clippy --all-targets --all-features -- -D warnings` 均 exit0，三项原始日志索引 SHA-256 `9654e45f1416b6fd205fc9e07a0814a667b49c5db247779367255f02a83db8ee`；本卡仍待双 env full、唯一 Claude 与 signed main，不把这些检查当成真实 OpenCode Session 采集。

**最终源码双 env full 收据（2026-10-14，单审前）：** `umask 0022`，依次载入 `.env.test` 与 `.env.live-test` 后执行 `cargo nextest run --all --no-fail-fast --retries 2`，exit0；**Summary [932.144s] 10091 tests run: 10091 passed (14 slow), 23 skipped**，原始私有日志 SHA-256 `eb4623ffedb916ba1dd86583bd1d65d2f4f5ff838eb7b6db33458e2028f9e8f7`。未发生 FAIL/RETRY/LEAK；23 skipped 不计本机 OpenCode Session 通过。本卡 7/7 AC、4/4 VER、五项继承精确测试、fmt/strict/full 已就绪，唯一 Claude 和 signed main 仍待；真实 pending artifact writer、Source A 与家族 C/D 仍属于后继门。

**最终源码/单审交接收据（2026-10-14，COV2-23 开工前复核）：** 六文件 manifest SHA-256 `c3a53df17c2b3db500e6a4829a2c17ddc903fe5f5826e0de08f150c97df10533`，root 在评审前后独立逐文件核旧/新 SHA、0644 mode 与完整 diff SHA，六项均 match。唯一 Claude Code Sonnet 对全部六个差异正式 **PASS**，P0/P1/P2=0、P3=3，原始 JSON SHA-256 `844fb61282a7e12ebb08221bd3cad687fed4e286bf7a41642256541cbfe7aedd`；非阻断 P3 为部分 fail-closed 负例、单版子计划 retain 独立性与成功例仅弱比较非 lease 字段，后续共享 writer 接线继续留意。`libra commit -s` 的 `6b24d797583afe1c0265f6e9fcbd364566671167` 含 raw commit `gpgsig` 与 DCO，已直推 `main`；`libra ls-remote` 和 GitHub main 同 SHA。旧 COV2-20 run `38493804953` 在本新提交后依用户要求取消（Windows job `115625055658` 与 network job 先前 success，offline-core 未完成，不能把旧 run 记全绿），当前新提交 run `38498688764` 仍在运行。本卡是 COV2 第九张源码就绪卡（9/25=36.00%，正式0/25）；下一卡 COV2-23，家族 C/D、22 个最终行为源码 Session 与父 Source A 仍待。

**Full-suite trigger:** ER-13 T-1：本卡 retain 是 COV2-22 真实 artifact writer 将消费的共享单一 helper，最终源码必须跑双 env full。先跑本卡 4 个具名 focused selector、相关冻结 v1 回归、nightly fmt 与 strict Clippy，再跑 `source .env.test && source .env.live-test && cargo nextest run --all --no-fail-fast --retries 2`（`umask 0022`），最后唯一 Claude Code 审查最终字节。COV2-REL-01 首次 bumped 实现树 full 仍须独立运行；后续仅上游纯 patch 顺延按该卡逐路径同一性规则复用。0 matched、skip、synthetic 或未执行均非 PASS；OpenCode Session/模型复杂验收只在 macOS 本机手工执行，不进 CI。

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

**Release boundary:** family child of REL-COV2-01（9/23）；逐卡单 Claude PASS 后 signed+DCO 直推 main，不写 R、不 tag。

**C/D coverage from:** COV2-REL-01；真实 exact-source C/D 通过后本卡才可 done/complete。

**Granularity:** `type=implementation; axis=调用方事务内版本化 reservation retain; recovery=forward-only; complete=yes; self-contained=yes; AC=7/8; VER=4/8; landing=1; prod-files=1; scope=M; deps=COV2-20; writeset=严格串行; release=family child; split-from=COV2-20; exception=N/A`

### Task COV2-22：真实 pending artifact 事务与冻结旧读端

**当前source资格暂停（事实覆写，原判据不变）：** root已因GC-COV2-03指定dcc完整old-writer/gold等价补核未闭合，暂停COV22最终source资格；原逐门运行与signed18b源收据保留其历史范围，不由旧7ef reference冒充dcc。当前source不计本卡，COV13/26；真实dcc采样/比较尚在进行，未运行或未通过不得记PASS，COV08可focused但不full/sign。后续仅实际收据+root裁决才恢复资格；不改本卡8AC/6VER、GC03或gold值、不回退隐私/SourceA。

**Task type:** implementation

**Lifecycle / Acceptance:** in-progress / locally-accepted（COV2-22 采用候选：原 8 AC/6 VER 与继承本机机制门已实际验收；最终新增事实文档的单 Claude 裁决、本次 signed+DCO main 交付及 remote 同一性仍由 root 完成，不在本私有候选中自授 source-qualified 或 formal complete）

**Description:** 自 COV2-04 拆出，且原始 outer-v2 成员完整性轴进一步拆出 COV2-25；消费 COV2-24 已审的内存 `SealedVersionedPendingArtifact`、COV2-25 key-owner verifier 与 COV2-20/21 事务 API，真实执行 `seal→persist(&DatabaseTransaction)→commit→load`。`pending_payload.rs` 只为 AC6/VER2 补同一持久化 artifact 的认证后 redacted rehydrate/typed deferred loader，不能新增来源铸造或 replay ACK；同时验证同事务双版本 lease/artifact、编译冻结旧读端拒绝、真实 writer 故障回滚，以及原 v1 生产 seal→persist→commit→load 后持久化 chunk/header 原字节/MAC 与历史冻结常量兼容。本卡 AC8 是 persist 入口对已封闭 artifact 的二次防御，不重复 COV2-23 的 checked-borrow 准入门。

**Out of scope:** 不改冻结旧 decoder、`key.rs`、`coverage_gate.rs`、history/catalog/claim SQL schema，也不开放 native issuer/Source A；COV2-08 才切换 replay。若 frozen guard 必须改，先把 `tests/compat/agent_architecture_guard.rs` 精确加 I 并复审。

**Current evidence:** `pending.rs` 现有 v1 writer；既有 `cov2_v1_pending_bytes_and_mac_frozen` 只钉住内存级 v1 bytes/MAC，不能证明经过生产 v1 persist/commit/load 后 chunk/header 仍与历史常量逐字节相同；本卡 VER6 须补真持久化回归。`pending_payload.rs` 的 `VersionedPendingPayloadProjection` 当前只给 pure codec/内存 seal，尚无来自真实持久化 outer-v2 的受认证 redacted rehydrate/typed deferred loader；COV2-22 AC6/VER2 要求在该原 owner 文件补此 loader，和 `pending.rs` 的真 persist/loader 同一行为轴。COV2-04 codec、COV2-24 sealed artifact 与 COV2-20/21 versioned verify/retain 均须先在 signed main 接收。开工按 ER-02 刷新最终源码行号、SHA/mode；隔离候选不作实现证据。

**Acceptance criteria:**

- [x] AC1：真实 sealed-v2 persist 在同一个 caller `DatabaseTransaction` 调用 COV2-20 verify 与 COV2-21 retain。
- [x] AC2：chunk/header/alias 与 v1/v2 两版 lease 原子提交。
- [x] AC3：全 v1 claim set 的 v2 artifact writer 在任何写前拒绝。
- [x] AC4：编译冻结的 pre-v2 reader 认证后拒绝真实已提交的 v2 artifact。
- [x] AC5：上述旧读端拒绝使 artifact/claims/ref/marker/receipt 字节和行均不变。
- [x] AC6：新 versioned loader 仅在 COV2-25 从同一已提交 artifact 的原始 bytes/MAC 验证成员后，重建并返回精确有序 `(coverage_schema_version, logical_turn_key)` tuple；认证失败无 verified payload。
- [x] AC7：在 retain 之后的真实 chunk/header/alias SQL fault 回滚两版 lease、artifact/alias 行且零成功 ACK。
- [x] AC8：OG-29 version-keyed association 存在前，`persist(&DatabaseTransaction)` 对已经封闭但含 native 双版本同裸 logical key 的 artifact 在首次 SQLite 写前再次 fail closed；这是与 COV2-23 borrow 拒绝不同的持久化入口防御。

**旧读端与 Source A：** `source .env.test && cargo test --test compat_agent_architecture_guard reasoning_pending_pre_v2_reader_oracle_is_immutable -- --exact` 与现有 `source .env.test && cargo test --lib internal::ai::capture::pending::tests::cov2_v1_pending_bytes_and_mac_frozen -- --exact` 均为本卡六条 VER 之外的 mandatory ER-04 冻结门，须各额外实际 1/1；冻结 decoder、v1 canonical bytes 与 MAC 不变。手插行或重签 synthetic envelope 不算 AC1/4/7。COV2-22 的 same-key 负例须用 test-only adversarial 构造绕过 COV2-23，仅为触达生产 `persist` 的防御门；不得把该 synthetic 构造或 MAC 当作 native Source A 正例。正常生产 seal 路径先由 COV2-23 拒绝，版本化 native 同 key 正例留父 OG-29 与真实 Source A。

**Verification:**

- [x] VER1：AC1/2/4/5：`source .env.test && cargo test --lib internal::ai::capture::pending::tests::cov2_real_v2_persist_and_frozen_old_reader_reject -- --exact`（真实 seal→persist→commit，编译冻结 decoder，全 before/after；1/1）。
- [x] VER2：AC6：`source .env.test && cargo test --lib internal::ai::capture::pending::tests::cov2_real_v2_artifact_new_reader_roundtrip -- --exact`（同一真实 seal→persist→commit→load artifact 验精确 tuple，并对原始成员篡改/错误 MAC 无 verified payload；1/1）。
- [x] VER3：AC3：`source .env.test && cargo test --lib internal::ai::capture::pending::tests::cov2_real_v2_artifact_rejects_all_v1_plan -- --exact`（无写入，1/1）。
- [x] VER4：AC7：`source .env.test && cargo test --lib internal::ai::capture::pending::tests::cov2_real_v2_artifact_later_sql_fault_rolls_back -- --exact`（两版 retain 之后真 chunk/header/alias trigger，逐行和逐字节比较；1/1）。
- [x] VER5：AC8：`source .env.test && cargo test --lib internal::ai::capture::pending::tests::cov2_sealed_native_same_key_persist_defense_fails_closed -- --exact`（test-only 构造同一 exact sealed bytes/coverage 的 native same-key artifact，绕过 COV2-23 仅为触达生产 persist 入口；断言 guard 在首次 SQLite 写/verify/retain 前失败、零 artifact/alias/lease/ACK 变化，不以普通 seal 提前拒绝冒充本卡通过；synthetic 非 Source A，1/1）。
- [x] VER6→v1 writer compatibility：`source .env.test && cargo test --lib internal::ai::capture::pending::tests::cov2_v1_pending_persisted_bytes_and_mac_frozen -- --exact`（在固定旧版 scope/IDs/clock/key 的受控 fixture 中真实调用生产 v1 `seal→persist(&DatabaseTransaction)→commit→load`，再从已提交存储读取 chunk/header 原字节与 MAC，逐项比较独立保留的历史冻结常量；期望值不得由当前 serializer、当前 loader 的 roundtrip 或测试重签生成；1/1）。

**进一步写集门：** 若现有 compat guard 必须更新断言，先明确把 `tests/compat/agent_architecture_guard.rs` 加入本卡精确 I，并按 ER-13/单 Claude 重审；否则该文件只读。

**Mandatory inherited COV2-25 ER-04（不计本卡六项 VER）：** 最终源码额外实跑下列四个 COV2-25 selector 各 1/1，另跑该卡列出的八个 v1/key/codec 继承命令各 1/1；0 matched/skip/旧源码结果非 PASS，synthetic 不构成 Source A。

- `source .env.test && cargo test --lib internal::ai::capture::pending::tests::cov2_pending_v2_original_members_authenticate_exact_tuple -- --exact`。
- `source .env.test && cargo test --lib internal::ai::capture::pending::tests::cov2_pending_v2_original_members_reject_substitution_or_bad_mac -- --exact`。
- `source .env.test && cargo test --lib internal::ai::capture::pending::tests::cov2_pending_v2_member_auth_preserves_v1 -- --exact`。
- `source .env.test && cargo test --test compat_agent_architecture_guard cov2_v2_pending_original_member_issuer_is_integrity_only -- --exact`。

**Full-suite trigger:** ER-13 T-1：本卡修改共享 `pending_payload.rs`、真实 reader 与 loader；最终源码须运行本卡 6 个具名 focused selector、上述两个 mandatory ER-04 冻结门各 1/1、相关 COV2-25 继承回归、双 env 全量 Nextest、nightly fmt、strict Clippy 与唯一 Claude Code 最终字节 PASS。COV2-REL-01 首次 bumped 实现树还须独立运行 full；后续仅上游纯 patch 顺延按该卡逐路径同一性规则复用。0 matched、skip、synthetic 或未执行均非 PASS。

**Dependencies:** DEP-COV2-029-BASE（父三卡final source重基，shared I–I串行，非native能力）； COV2-25；消费前驱真实最终 A/B、适用 focused/full、fmt/strict、唯一 Claude PASS 与 signed+DCO main SHA/hash/mode；不把继承 C/D 的 locally-accepted 冒称 complete。

**Deliverables:** 无独立文件；API、测试、兼容文档、逐文件 SHA/mode 和 review/验收事实写回本计划正文，随唯一发布点交付。

**Implementation write set:** I-core 精确五文件：`docs/development/plan/plan-20261013-coverage-schema.md`; `docs/development/plan/plan-status.md`; `docs/development/tracing/coverage-v2.md`; `docs/development/tracing/agent.md`; `COMPATIBILITY.md`；本卡新增精确路径：`src/internal/ai/capture/pending.rs`; `src/internal/ai/capture/pending_payload.rs`。现有测试模块可写；未列路径包括 `sql/**`、冻结 decoder、parent 计划均为只读，需另行 G-09/I 修订才能扩权。

**Release write set:** N/A；C/D 继承 COV2-REL-01。

**Files likely touched:** I-core 上述五文件；`src/internal/ai/capture/pending.rs`; `src/internal/ai/capture/pending_payload.rs`。

**Docs and compatibility impact:** I-core 的 coverage-v2、agent、COMPATIBILITY 只陈述本卡已交付的能力，不提前宣称下游 artifact/replay/Source A 已完成。无新 `--test` target；`tests/INDEX.md` 无写权。

**Rollback mode:** forward-only；失败保持旧 artifact/claim，签名+DCO 新 main 前滚并重跑下游。

**Migration and rollback:** 无 DDL；Repository claim 使用既有 versioned 键。若实际需迁移，先加角色隔离 migration 卡、旧/新 binary 门并修 G-03/G-04/I。

**Security and privacy:** 继承 GC-COV2-02/03；所有错误 fail closed、零假 ACK、零候选明文 sink；真实 native Source A 仍在父计划。

**Performance budget:** 使用已审 claim/map 上限，解析/查询前有界；不得新增无界扫描、无时限重试或私开的第二连接。

**Estimated scope:** M（AC=8/8、VER=6/8、landing=1、production files=2；VER6 是真实持久化 v1 writer 兼容性新门，既有内存级 v1 bytes/MAC 与 frozen old-reader oracle 两个 ER-04 门另计）。

**Version increment:** N/A；此卡不发版。

**Release boundary:** family child of REL-COV2-01（14/23）；逐卡单 Claude PASS 后 signed+DCO 直推 main，不写 R、不 tag。

**C/D coverage from:** COV2-REL-01；真实 exact-source C/D 通过后本卡才可 done/complete。

**Granularity:** `type=implementation; axis=真实 pending artifact 事务与冻结旧读端; recovery=forward-only; complete=yes; self-contained=yes; AC=8/8; VER=6/8; landing=1; prod-files=2; scope=M; deps=DEP-COV2-029-BASE,COV2-25; writeset=严格串行; release=family child; split-from=COV2-04; exception=N/A`

**COV2-22 采用候选的实际本机事实（2026-10-15；交付资格由 root 最终裁决）：** 本候选把下述两生产 owner 与三行为文档的精确受测后像交给本次实际 signed+DCO main 提交，未知提交 SHA 不预写为事实；root 尚须完成最新两计划事实增量的唯一 Claude Code 审查、父级裁决及签推同一性。原 AC/VER、命令、I/R、DAG、Granularity 与能力边界不变。上方 Current evidence 是原开工态，下面“历史隔离准备快照”保留其原时点状态，不用旧 UNRUN 文字否认本段实际运行，也不倒改旧失败。

实现仍是单一 caller transaction 的 v2 verify→retain→chunk/header/alias 与原始成员认证后的 redacted/deferred loader；封闭 artifact 在首次 SQLite 前拒绝 all-v1 和 native 跨版本同裸 key。非 Clone／非 Serde carrier 保留原 bytes/MAC/source/有序 version-key tuple 与本地 integrity stamp，不接生产 Session、native restore 或 replay ACK。`pending.rs` 受测 SHA `f8f1160cd1832de2f76ccedde9e67fecd36c8e47628619e635ee8c8d7c0abde9`、`pending_payload.rs` 为 `cadba46d0960cde91650ea4129a03113333303683a4b891b1a82a24c774b667c`，两者 mode 0644；三行为文档逐字使用同一 full/Claude 后像。原 v1 seal/persist/reader、原 memory gold、冻结 decoder `96d71660e8b75d61cff2dccaa59a410b90b499afd63e10c9dcc2cc678312e32c` 不变。Source A、父计划 native 三字段、历史 import020、Linux 功能及家族 C/D 均不由本卡扩权或放行。

原六项 VER、两个冻结门、COV2-25 继承与三闭集守卫合计 21 unique exact 各实际 1/1、无所选用例 ignored/零匹配；pending owner 70/70、payload 31/31、完整 architecture 58/58、docs 32/32，nightly fmt 与 strict all-targets/all-features Clippy `-D warnings` 实际 exit0，strict raw `8a0830ace293d01c005fcffde899cc5196399034c7539c65cdee82d177238f68`。原 8 AC 的机制断言到六项 VER 的索引及 36 个测试动作组已逐项列明；这不新增判据、落点或豁免，目录落点 1／生产 owner 2／scope M 不变。

双 env 完整 Nextest r4 曾实际 exit100：10139 run、10133 passed、6 failed、2 slow、23 skipped，raw `3b6c0c7ea30bd73c4f61460aff7204d3499e1be86113f5553499c2cc38a954fd`；六项各三次 FAIL，共 18 failed attempts／12 additional retries，不能改称 PASS。它们是纯源码私有目录缺少真实仓库上下文引出的既有 macOS exporter `storage_path()` 一项和五个 serial classifier root-precondition 失败。root 仅授权以正常 a70 helper 执行私有 `libra init --vault false --initial-branch main`，真实 DB/HEAD 有效、未复制根元数据／凭据、未 commit/push/remote 或初始化全局 vault；原 2043 文件 SHA/mode 不变，未生成额外默认 untracked 文件。六项随后各 Nextest exact 实际 1 run/1 passed，并在下一完整 run 内均实际 PASS；没有改 registry、阈值或源码来绕过断言。

双 env r5 原命令 `cargo nextest run --all --no-fail-fast --retries 2`、jobs2／test_threads4，2026-10-14 21:57:47.085756→22:26:35.513934 UTC 实际 exit0：**10139/10139 passed（2 slow、2 leaky），23 skipped，0 failed/flaky/retry/timeout/abort/execfail**；raw `59e34e861d296be52474835a559ebb259f58e4f8501836f8311d6c1136382cec`，完整收据 `87594346f55f6c724e639415e64324cc6ded20581b1b52132204606da50832fa`。runner 的 10139 passed 含 10137 条普通 PASS 与两条 LEAK，不能写成零警示。两条分别为 `internal::ai::authorized_read::tests::helper_report_accepts_evolving_offsets_and_cumulative_lengths` 和 `command::diff_plumbing_test::diff_plumbing_outside_repository_is_an_error`；同环境各一次原 Nextest exact 后分别 1 run/1 passed、0 LEAK/重试，raw `faf7f5af66d55e6662f9723b409896d99244f88e26e3087d2638f253067db95a`／`6adced633bc8ecdb279907251f1d6206efba540999ef403311e82ca99c421a1c`。单独诊断不擦除 full 两条警示、不证明其原因，也不新增零 LEAK 门禁。

VER6 的唯一 expected 来源是独立原 `7efcf868d49a5183e7f0851a89abebee91357155`／0.30.46 完整旧源码 writer 实际 `seal→persist→commit→load` 后先行封存的原始 chunk/header/alias/sidecar/MAC；历史生产、依赖和其余 2029 文件不变，仅追加采样 test。历史 whole-source writer receipt `1bb278f923d0f5e3ab83a1deba498b056218553cd12878eac676f0f39a18ed11`，原 envelope 2104B/SHA `bf87589384983e2ad5ae74f3773f8459c9a20c56e4736775eb3129fe52a005ef`，root 独立原 tag/源核验 `c734b52441f51901f6f8c9a3752b1d95b3558025a44e37874515df50b131ad43`。原 Python-only KAT、VER3/VER6 失败、历史 sampler r1 编译失败均保留；expected 不由当前 serializer/loader/重签产生，不冒称历史发布 binary 或真实 provider Source A。

root 发起的唯一 Claude Code R1 实际 SOURCE PASS、P0/P1/P2=0，raw `9cd81f9801f5313ac59813f796d7008736a8c5436d160a4b712b6b93b27ea020`；root 已核七 diff 的 Read 工具完整覆盖。该 R1 审查的是受测原后像；本次两计划新记账字节仍待 root 的最终文档增量审查，不借旧 PASS 预授新字节。full 前后及后续诊断的 2043 文件字节/mode 同一，受测 manifest `8592fc2bcc13c4a2eac6cac205f49eaba553b988697ccacfe8eabda75227afe9`、完整源 ZIP `fb700edeeb8927c794b023befe2ea005be3c363de0b834cc52706d1ca7b016ea`；**本次例行记账改变两计划，且 root 的父计划已有 049 纯事实增量，因此不宣称采用后的完整 2043 字节与 full 相同**。采用时以逐路径程序核对证明受测实现、测试、配置和原行为契约同一，三行为文档也为受测精确后像。

**条件进度与后继：** 仅在 root 将本候选实际完成最终审查/裁决并以本次 signed+DCO main 交付、核对 remote 后，COV source 从 13/26 变为 **14/26=53.85%**；formal 仍 **0/26**。本候选本身不是已经发生的 source-qualified 提交，不伪造未知 SHA；父计划保持 formal **59/71=83.10%**、source **62/71=87.32%**，71/26 分母不变。最新 USER-TEMP-REL-049 的十四门及四张父卡实际 done/complete 直接继承 root 22:28:49 UTC 的完整收据 `dad01d952f7ce629354118264a4926b048693bf851258e4cd14beb9853727f8c`，不被旧隔离冻结文字回退；它不关闭 COV2-REL-01、DEP-OG-13 或 native Source A。COV2-22 实际 source-qualified 后才按原 DAG 转 COV2-08，本次没有实施该后继。

**历史隔离准备快照（原文按当时状态保留；实际运行与当前采用条件见上段）：**

**COV2-22 隔离候选准备快照（2026-10-15，非最终验收）：** 已消费 BASE 的 signed+DCO main `1e2177e11029e28e3c81afbe6ec9f1609cdde7da` 精确源交接；该前驱最终双 env full 10133/10133、23 skipped、exit0/raw `24febb2eba3d2aad09c6f516988e06e9ae127ad8dd96b362fbda4ea45671fdb2`，sole-Claude R2 PASS/raw `1da1fc13e2798ab51918f09f83de5499e87f204deea9614612b6ea6c01ca9c56` 只证明前驱最终源码，不替代本卡新行为。隔离候选仅写本卡两生产 owner 与五 I-core 文档，不改变任何 AC/VER、冻结 oracle、原 v1 seal/persist/reader 或其它卡范围。候选新增 caller-txn v2 verify→retain→chunk/header/alias、原始成员认证后的 redacted/deferred loader、非 Clone/非 Serde 的原 bytes/MAC/source/有序 tuple/integrity carrier；在首次 SQLite 动作前再次拒绝 all-v1 与 native 跨版本同裸 key。没有接入生产 Session/replay/native issuer/ACK，没有模型或 OpenCode 集成 CI。

六个新具名测试已准备，包括真实 reserve→seal→persist→commit、冻结旧读端拒绝后全部相关行（含 SQLite 类型、ID、完整 agent_session receipt metadata）与 ref 原字节不变；原 bytes/MAC loader 的坏 MAC、同 MAC 字节变异、独立 root/storage/scope 拒绝，及 None-source codec sentinel 不能进入 native loader；all-v1/same-key 以 test-only 已封闭对抗 artifact 到达 persist 且用事务内缺表探针辨别首次 SQL 前拒绝；三处 retain 后真实 trigger、调用方 rollback 与零 ACK。对抗构造仅在 cfg(test)，不放宽正常 source factory，也不是 native 来源正例。私有 preflight 的首轮 VER3 fixture 未到 persist，已保留失败并仅修 test 构造；其后 VER6 的原 Python-only KAT 在真实 alias 验证失败，原常量与 FAIL 留档，不能把该错误 KAT 当作历史 gold。

VER6 现仅消费独立历史源码 `7efcf868d49a5183e7f0851a89abebee91357155`（0.30.46）的真实完整 v1 writer `seal→persist→commit→load` 先行产出并封存的原始 chunk/header/alias/sidecar/MAC 常量。旧 2030 源文件经 archive blob/mode 核验；仅追加 cfg(test) 采样、2029 其它文件及旧生产前缀和 Cargo.lock 不变。历史 sampler r2 已实际 exact 1/1、exit0，raw `3fdb5f22a7b75518c80b0bac304eff1b960db69f9e0851d2709ba2e31f3b67ea`；封存 receipt `1bb278f923d0f5e3ab83a1deba498b056218553cd12878eac676f0f39a18ed11`，原 envelope 2104 bytes/SHA `bf87589384983e2ad5ae74f3773f8459c9a20c56e4736775eb3129fe52a005ef`。当前测试不生成期望值、不重签 gold，再调用不可变 frozen old inner producer/coverage、当前生产 v1 writer 与新旧读端逐项对拍。此证据是受控 synthetic 输入下的历史完整源码 writer 执行，不冒称历史发布 binary 或真实 OpenCode Source A；历史 sampler r1 编译失败亦保留。

本快照不勾选 8 AC/6 VER，不改 lifecycle/acceptance/source/formal 分子。私有候选 focused 运行的逐命令结果、失败与最终重跑另附私有交接，均不替代 root 最终整合源码的全部 exact/mandatory、fmt/strict、双 env full、唯一 Claude、signed main/remote。本卡继承的 C/D 仍归 COV2-REL-01。049 发布冻结期间此候选不得推入 main。Source A/native 三字段、历史 import020、Linux 功能与家族发布状态均不从本准备推断通过。


### Task COV2-23：同源 NativeProven pending-v2 借用门

**Task type:** implementation

**Lifecycle / Acceptance:** in-progress / locally-accepted（8/8 AC、4/4 VER、四项继承与两项修订后架构守卫各 1/1；fmt/strict/full 与唯一 Claude PASS；signed+DCO main `d977d13c5e23111b93d78477b9e1a70015a111b8` 已交接；家族 C/D 待 COV2-REL-01）

**Description:** 自原 COV2-04 的内存 seal 前置拆出，且消费 COV2-19 的最终已签名 `NativeVersionedReasoningClaimCommitPlan`。只在 `src/internal/ai/capture/checkpoint.rs` 增加 `CheckpointRedactedPayload::prepare_pending_native_versioned_seal(scope, &companion, provider) -> Result<PreparedPendingNativeSeal<'_>, CheckpointStoreError>`。该入口从同一个已封闭 payload 借用 NativeProven batch，验证 companion 的 payload/source tag、当前 capture binding、完整 versioned claim 集，按 `(coverage_schema_version, logical_turn_key)` 取每个 identity，并生成既有 `PreparedPendingNativeSeal`。在下游别名/投影仍按裸 logical key 关联期间，native 跨版本同 key 在此 fail closed；COV2-04 的纯 wire codec 同 key roundtrip 仍合法，不是 native 授权正例。

**Out of scope:** 不修改 pending.rs/pending_payload.rs、coverage_gate.rs、数据库/schema、issuer/locator 或冻结旧读端；不创建 artifact/MAC、不持久化、不保留 lease、不升级 OpenCode 来源证明。COV2-24 才消费本卡 checked borrow 和 COV2-04 codec 完成内存 seal，COV2-22 才做真实事务；native 同 key 正例留给 OG-29 且受真实 Source A 硬门。

**Current evidence / ER-02 source anchors（2026-10-14 开工快照）：** COV2-19/20/21 均已本机 A/B、单 Claude PASS 与 signed+DCO main 交接，最新前驱 `6b24d797583afe1c0265f6e9fcbd364566671167`、远端同 SHA。前驱 `checkpoint.rs` SHA-256 `cae575f0d15662ffcc72c2890ace5134aee1d657831fc750e758d362011acb69`、mode `0644`，保留 COV2-20 Windows cfg 修复；`coverage_gate.rs` SHA-256 `d05c286627e117fddc2f4e311c5fe53946b93d9ee2098ab78962cf718ed0be3f`。本卡隔离候选 patch SHA-256 `2ef602e39504b80a16f7b3a2c433ae8b4f372bf50b0997de18ab48783d26f6c3`，已在实际 main 零 fuzz 重放且候选 `checkpoint.rs` SHA-256 `97bbf56b4a424cf84989e99358d901af3a29dc879fc915538d486654ee92549b`、mode `0644`；新入口在 `checkpoint.rs:1827`，现有 v1 factory 的 no-orphan 检查和 COV2-19 的 source-bound companion API 已在最终前驱源码核实。本段是源码/镜像定位，不构成本卡 Cargo、Claude 或 native Source A 验收。

**Acceptance criteria:**

- [x] AC1：只有 payload 的 `ReasoningSourceDisposition::NativeProven` 可返回 checked borrow；Ordinary/Unmapped/Partial 全部拒绝。
- [x] AC2：调用 COV2-19 最终已审 `companion.reasoning().matches_payload(payload,session,provider)`，native proof/opaque metadata 不同的另一合法 payload 必须 source-tag 不等并拒绝；此 tag 不是整个 redacted payload 对象 identity 或 Source A 凭据。
- [x] AC3：从该 payload 当前 capture proof 借用的 session/provider 与 companion 当前 capture context 精确匹配；传入 scope 与 companion scope、source plan scope 精确匹配；不能由 caller supplied DTO、label 或旧绑定重组权限。
- [x] AC4：调用 COV2-19 最终已审 `companion.reasoning().bind_claims(companion.versioned())`，完整 `(version,key,digest,completeness)` canonical 集合必须相等；缺失、额外或变异均拒绝。
- [x] AC5：按 `companion.versioned().claims()` 的原始顺序，用 `(version,key)` 逐个取得 `companion.reasoning().turn(version,key)`，返回 associations 与该有序 identity 向量完全相等；不退回裸 key 查找。
- [x] AC6：空 claim 集在产生 prepared witness 前拒绝，不能把 native payload 伪装成无 opaque claim 的 pending。
- [x] AC7：即使同模块测试注入绕过 COV2-19 factory 的内部 companion，claim 数达到 `REASONING_ARTIFACT_MAX_ENTRIES+1` 时本入口也必须在 associations 分配/克隆前拒绝且不返回 witness。这是本卡可直接证伪的防御性 entry guard；opaque/descriptor byte 与双 map encoded-byte 上限仍归 COV2-19/14 已审前驱及其继承回归，本入口不声称另做 byte 校验。
- [x] AC8：因既有 `PreparedPendingNativeSeal` 与投影 alias 仍使用裸 key，跨版本重复裸 logical key 在该 native 准入门拒绝；同版本重复由封闭 companion 原门拒绝。仅纯 codec 可保留跨版本同 key 值，不向 native writer 授权。

**独立谓词计数:** AC1–8 为八个分别可失败的检查。`VerifiedPendingBinding::from_validated_pending_context` 必须在 AC2/3 的同一 payload borrow 内构造，是实现机制，不另声称新 Source A 或额外成功谓词。错误仅标资源/修复建议，不包含候选字节、原始 key 或来源秘密。所有反例须确认该纯准备入口未作存储写入。

**Inherited v1 no-orphan safety（ER-04 回归，不新增本卡 AC/VER）：** v1 factory 的 `batch.pending_logical_turns().all(|key| identity.turn(key).is_some())` 不得在版本化入口丢失。COV2-19 source factory 从同一 `NativeProven` batch 借出全部 proof inputs、验证 `inputs⊆known_keys`，并为 known turns 建 version-keyed identities；其 `bind_claims` 将完整 source-derived turn/identity 集与传入 claims 等值绑定。本卡 AC2 的 `matches_payload` 保证消费的是原 batch，AC8 排除一个裸 key 对应两个 native 版本，因此 AC4/5 形成的 associations 必须覆盖该 batch 的每个 pending key。COV2-19 VER2 直接证明前两步，本卡 VER2 再直接断言返回 associations 无 orphan。若最终前驱的 API 或测试不能证明这条链，不能只凭计数文字放行，须先修 G-09 与验收。

**Verification:**

- [x] VER1→AC1–3：`source .env.test && cargo test --lib internal::ai::capture::checkpoint::tests::cov2_pending_native_versioned_source_borrow_rejects_recombination -- --exact`（新 deterministic synthetic sealed fixture；状态变异、第二 payload、scope/session/provider 变异及同源正例；实际 1/1）。
- [x] VER2→AC4/5：`source .env.test && cargo test --lib internal::ai::capture::checkpoint::tests::cov2_pending_native_versioned_claims_bind_by_version_and_order -- --exact`（新 deterministic test；缺失/额外/version/digest/completeness/顺序变异与 identity 精确对照；同一源 batch 的每个 `pending_logical_turns()` key 都须在返回 associations 中找到唯一 version-keyed identity/claim，直接断言无 orphan；实际 1/1）。
- [x] VER3→AC6/7：`source .env.test && cargo test --lib internal::ai::capture::checkpoint::tests::cov2_pending_native_versioned_claim_count_is_bounded -- --exact`（新 deterministic test；空集合与同源正常 companion；在 `checkpoint.rs` 同模块测试中克隆已封闭同源 source，仅为反例补成 513 个互异且内部一致的 versioned turns/identities/claims，再经 `NativeVersionedReasoningClaimCommitPlan::new` 构造测试专用 companion 并调用本入口。若 `new` 先拒绝 513，则仅在同文件 `#[cfg(test)]` 下设置可受审的内部构造 seam，使该反例确实到达本入口，不放宽生产构造器。必须由本入口自身在 associations 分配/identity clone 前返回错误、零 prepared witness；该测试不以 COV2-19 factory 的 512-entry 拒绝代替本入口断言，也不要求前驱卡拥有 513 反例；实际 1/1）。
- [x] VER4→AC8：`source .env.test && cargo test --lib internal::ai::capture::checkpoint::tests::cov2_pending_native_same_bare_key_fails_closed -- --exact`（新 deterministic test；跨版本同 key/native 拒绝、不同 key 正例，零 pending output；实际 1/1）。

**Focused evidence（2026-10-14，最终生产源码）：** VER1/2/3/4 与四项 mandatory ER-04（v1 retained-claim/no-orphan、v1 pending association、COV2-19 source/claim binding）共八个精确 selector，各 **1 passed/0 failed/0 ignored**；原始私有日志索引 SHA-256 `b254267dcbe8e9fcdbc5021af3fa566c4f81ab1321e4b48b701e8f109f357bc8`。`checkpoint.rs` SHA-256 `97bbf56b4a424cf84989e99358d901af3a29dc879fc915538d486654ee92549b`、mode `0644`；候选仍待双 env full、唯一 Claude 与 signed main。合成 sealed fixture 只证明内部借用门，不证明真实 OpenCode Source A。

**Strict-quality 收据（2026-10-14）：** `cargo +nightly fmt --all --check`、`libra diff --check` 与 `cargo clippy --all-targets --all-features -- -D warnings` 均 exit0，三项原始日志索引 SHA-256 `ff9906c0ed73e1c39feab9469b719abb4371c11c4f21ad5c33e2d775469d44cb`。COV2-23 显式 ER-13 T-5 双 env full、唯一 Claude 与 signed main 仍待。

**首轮双 env full 失败与 G-09/I 修订收据（2026-10-14 02:55 UTC）：** `cargo nextest run --all --no-fail-fast --retries 2` 实际 exit1，**Summary [1056.301s] 10095 tests run: 10093 passed (15 slow, 1 leaky), 2 failed, 23 skipped**，原始私有日志 SHA-256 `b55e5ad4986f89773c21d2beea2f28bb029403cf79ae05b7ba6c7bf8a95c1246`。失败分别为 `reasoning_pending_restore_has_closed_callers` 与 `reasoning_pending_signers_and_native_record_fillers_are_closed`，两者都精确报告 `from_validated_pending_context` 多出本卡 `prepare_pending_native_versioned_seal` 调用方。此失败是确定性守卫差异，不作网络重跑或 PASS。按下述精确 I 扩写集后仅登记这一可信新调用方，保留旧闭集；两项精确测试与双 env full 都必须重新实际通过。

**G-09/I 前置复审与修复收据（2026-10-14）：** 唯一 Claude Code 对本卡新增 `tests/compat/agent_architecture_guard.rs` 的窄范围计划修订给出 **PASS**（P0/P1/P2=0，P3=2；raw SHA-256 `2d1027ce9380c0c64e199a80556524af2a6d6e0eab67713012530daaa176b571`），明确这不是最终卡验收。随后两个原失败守卫在仅增旧 v1 factory 加本卡 versioned factory 的精确闭集下各 **1/1 PASS**，原始日志 SHA-256 分别 `f8fb417307b0750dca08a1627df8351f98a6d99ebc58b6bd73b03a743a879d96`、`b1ce6ece72c0942f83f1298f1d8ca7d0c1f3e3127b817094446bb725b7dfa765`；守卫文件 SHA-256 `a71a065dc43cf23612974acce9b18d35e4473482f10eca2a25e56f71db441547`、mode `0644`。修复后 nightly fmt、diff check 与 strict Clippy 再次 exit0，日志索引 SHA-256 `a9fed70899111705529d15c890ccc2e0a8091f3a769db4a2cd960ca461abb6c1`。生产 `checkpoint.rs` 字节未变；本收据时点双 env full 重跑与最终七文件单 Claude 仍待。

**修订后最终源码双 env full 收据（2026-10-14 03:42 UTC，最终单审前）：** 在 `umask 0022`、`.env.test` 与 `.env.live-test` 下 `cargo nextest run --all --no-fail-fast --retries 2` **实际 exit0**，原始日志 SHA-256 `5b20f0bfcf4ab2664b89dcb806c14ba812aa5597c7c7b48ca2cac621f5ca6599`，**Summary [1122.120s] 10095 tests run: 10095 passed (17 slow, 1 flaky, 2 leaky), 23 skipped**。原失败的两个架构守卫在本轮各 PASS；flaky 是 `test_fetch_into_mirror_updates_and_prunes_all_refs` 2/3 次尝试转绿，两个 leaky 名称可由原始日志核对。验收 wrapper 原先错误要求 `Summary` 顶格，因摘要前有空格而在 Cargo exit0 后自行 exit1；已修正为允许前导空格，并在**同一原始日志**上验证摘要匹配 exit0，未把 wrapper 误报写成 Cargo 失败，也未重跑/改写测试日志。修订后本卡八项 focused 精确测试再各 1/1，日志索引 SHA-256 `b127dc532ddcc1c3c463966eb4f02695e11aac0522090d8be11f73d24ecf38c9`；nightly fmt、diff check、strict Clippy 最后复核均 exit0，日志索引 SHA-256 `fad493accfc52ba89181c8f250b74cf6eebb70d367fe6f0938ad6e3a6fdf3e5e`。23 skipped 不计 OpenCode Session/Source A；最终七文件 Claude 与 signed main 仍待。

**Signed source handoff（2026-10-14）：** 七文件 manifest SHA-256 `39a3bb167df0279a0dbc67acb3c8575a791ec34c5851423c460b365575f78f63` 已由 root 在唯一 Claude 单审前后逐文件核 old/new SHA、mode 与完整 diff；唯一 Claude Code **PASS**（P0/P1/P2=0，P3=3；raw SHA-256 `024c7011547dbe0cbb466e2df8d1fe3c5ed555b9b56de6eb09377bfc64a83cd1`）。非阻断 P3：AC3 的 scope 来源措辞、test-only 零写断言强度、两个闭集守卫的重复分支；AC3 文案随 COV2-04 前滚，其余保留为后续维护建议。七文件签名+DCO commit `d977d13c5e23111b93d78477b9e1a70015a111b8` 已直接 Push main，本机、`libra ls-remote` 与 GitHub main 同 SHA；只剩未暂存的外部 `plan-20260904/05`。新 commit 后按用户要求已请求取消旧 run `38498688764`，不能将其未完成 offline-core 冒称通过。当前 COV2 源码就绪 **10/25=40.00%**，正式仍 **0/25**；下一卡 COV2-04，家族 C/D、真实 Source A 与 Linux 用户后验仍待。

**Mandatory ER-04 inherited regressions（不计 G-03 VER）:** 本卡最终源码上重跑 `source .env.test && cargo test --lib internal::ai::capture::checkpoint::tests::pending_reasoning_registration_requires_exact_retained_claims -- --exact`、`source .env.test && cargo test --lib internal::ai::capture::pending::tests::reasoning_authenticated_pending_rejects_inconsistent_identity_claim_associations -- --exact`，以及 COV2-19 的 source-bound native companion VER1/VER2；每个真实 1/1。若 selector 因最终前驱改变，先修计划和守卫而非报 0 matched PASS。测试仅 deterministic synthetic sealed fixture，不启动 OpenCode Session、不调模型，因此可留 CI；真实 Source A 仍未证明。

**Full-suite trigger:** T-5：本卡显式加严要求全量。新增 native pending 权限入口横跨已审 companion、checkpoint borrow 与旧 pending 调用者，focused 门可能遗漏回归；这是一项卡内保守全量要求，不冒称命中 ER-13 T-1 的列举表面。最终源码上除上述 focused、nightly fmt、strict Clippy 之外，执行双 env `cargo nextest run --all --no-fail-fast --retries 2` 全量；COV2-REL-01 首次 bumped 实现树全量仍须另行运行；后续仅上游纯 patch 顺延按该卡逐路径同一性规则复用。未运行/失败/skip/synthetic Source A 不计 PASS。

**Dependencies:** COV2-21（传递依赖 COV2-20/19）；必须消费三张卡实际最终 A/B、适用 focused/full、fmt/strict、唯一 Claude PASS 与 signed+DCO main SHA/hash/mode，不把家族 C/D pending 冒称 complete。

**Deliverables:** 无独立仓库文件；封闭借用 API、现有 test module 内具名测试、逐文件 SHA/mode、评审/验收事实写回本计划和状态，随唯一发布点交付。

**Implementation write set（G-09/I 修订，2026-10-14）：** I-core 精确五文件：`docs/development/plan/plan-20261013-coverage-schema.md`; `docs/development/plan/plan-status.md`; `docs/development/tracing/coverage-v2.md`; `docs/development/tracing/agent.md`; `COMPATIBILITY.md`；本卡生产/测试落点 `src/internal/ai/capture/checkpoint.rs` 的现有模块；另加既有 `tests/compat/agent_architecture_guard.rs`，仅把 `from_validated_pending_context` 的封闭调用方集合从旧 v1 factory 扩为旧 v1 factory 加本卡 versioned factory。新增调用方必须在两个既有闭集守卫中精确登记，不放宽其它调用方或 native 来源检查；本次全量中两项守卫因该新增调用方按预期失败，修订后须各精确 1/1 并重跑双 env full。`pending.rs`/`pending_payload.rs`、`coverage_gate.rs`、冻结 decoder、`sql/**`、parent plan 在本卡均只读。若最终实现还需要它们，先修 G-09/I 并复审。

**Release write set:** N/A；C/D 继承 COV2-REL-01。

**Files likely touched:** 上述七个精确 I 文件（五个 I-core 文档、`checkpoint.rs`、既有架构闭集守卫）。

**Docs and compatibility impact:** `docs/development/tracing/coverage-v2.md`、`docs/development/tracing/agent.md`、`COMPATIBILITY.md` 逐文件陈述本卡仅准备 checked native borrow，尚无 pending-v2 seal/persist/Source A；`docs/commands/agent.md`、`docs/commands/zh-CN/agent.md` 和 libra-backend 网站命令页 N/A，因为本卡不改用户可见命令、配置、输出或错误码。若实际 diff 改了公开行为，按 ER-06a 先扩精确写集并同卡同步三处。无新 --test target，`tests/INDEX.md` N/A。

**Rollback mode:** forward-only；本卡尚无新持久化路径，失败维持旧 v1 流程与已有数据，已签 main 后以新签名修复并重跑下游；后续 COV2-24/22 不得使用被撤销的借用 API。

**Migration and rollback:** 无 DDL；若需 migration，先另立 GC-13 角色隔离卡与旧/新 binary 门，不隐式扩权。

**Security and privacy:** `matches_payload` 仅为已验证 NativeProven 的同源相等标签，不是 issuer 凭据；不得从 MAC/DTO/synthetic 创建 native 权限。无候选明文写 ordinary/typed/log/error/计划；AC8 fail closed 直到 OG-29 与 Source A。

**Performance budget:** 本入口在 associations 分配/clone 前先按 COV2-19 `REASONING_ARTIFACT_MAX_ENTRIES` 拒绝超限 claim 数；opaque/descriptor bytes 与 COV2-14 双 map 4096 entries/1 MiB encoded 上限在前驱 factory/writer 守卫，不把它们算作本入口 byte 检查；不得复制整个 payload/opaque 正文。

**Estimated scope:** M（AC=8/8、VER=4/8、landing=1、production files=1；单一内部权限入口）。

**Version increment:** N/A；family child 不发版。

**Release boundary:** family child of REL-COV2-01（10/23）；单 Claude PASS 后 signed+DCO 直推 main，无 R/tag。

**C/D coverage from:** COV2-REL-01；真实 exact-source C/D 绿后才 `done/complete`。

**Granularity:** `type=implementation; axis=同源 NativeProven pending-v2 借用门; recovery=forward-only; complete=yes; self-contained=yes; AC=8/8; VER=4/8; landing=1; prod-files=1; scope=M; deps=COV2-21; writeset=严格串行; release=family child; split-from=COV2-04; exception=N/A`

### Task COV2-24：同源借用到内存 pending-v2 seal 接线

**Task type:** implementation

**Lifecycle / Acceptance:** in-progress / locally-accepted；COV2-24 最终 5/5 AC、3/3 VER 与七项 mandatory ER-04 精确 1/1（十 selector；focused index `39306128fdc72bbb2ae9bca583f186f2d8bfda693b68fb1a38795b815c9e31d5`），nightly fmt/strict Clippy/diff exit0（quality index `d0c676eb165802886f6f700d869d813db610a0fc7d09bf5e9759c452b71e979b`），双 env T-5 `NEXTEST_TEST_THREADS=4` 10103/10103 passed、23 skipped、exit0（raw `2908fff258247c857ae857a4759fcde9b925c212cc5912fa373ece16b5373029`）；唯一 Claude Code PASS、P0/P1/P2=0、P3=3（raw `a6d467bcdc33c42fff3c45f55681ebbaefa213d96cfd59a8b901c5d3dff942f7`）；八路径 0644 manifest `cd43b9af3980c8f3e90ce5751df7855ebaf1f604e2803130f0d32d9556b46e98`；signed+DCO main `c56de683acf71d7327436eef7ef9185d97dcff27` 与远端同 SHA。两轮失败 full 保留为失败历史；COV2-24 交接时安全 CI 仍在运行；其后 `c56de683`/前次 `5b0e7fc1` 的未完成旧 Actions 按用户新 commit 策略取消，cancelled 不记 D 组通过。 本卡 C/D 仍由 COV2-REL-01 继承，尚未 formal done/complete。

**Description:** 自 COV2-04 的内存 seal 行为轴拆出。在 `src/internal/ai/capture/pending.rs` 实现 `SealedVersionedPendingArtifact::seal` 与封闭 `VersionedPendingSealRequest`：从请求中同一个 `CheckpointRedactedPayload` 和 COV2-19 已封闭 `NativeVersionedReasoningClaimCommitPlan`，在本入口调用 COV2-23 `prepare_pending_native_versioned_seal`；在 `src/internal/ai/capture/pending_payload.rs` 实现 `PendingPayloadProjection::seal_bound_versioned_reasoning`，将该 checked borrow 的有序 identity 与同一 companion 的有序 `(coverage_schema_version,logical_turn_key)` claim 精确配对，形成版本化 inner projection；最后消费 COV2-04 有界 codec，生成未持久化的 exact-byte authenticated outer-v2 artifact。COV2-23 既有 `PreparedPendingNativeSeal` 的 association 值仍不带 version，故不得单凭该裸 key 向内层投影补猜 version。`provider` 只从已验证 alias/capture context 导出，不额外接受可伪造标签。现有 v1 `SealedPendingArtifact::seal`、`PendingPayloadProjection::seal_bound_reasoning` 与其字节保持原路径。

**Out of scope:** 不修改 `checkpoint.rs`/`coverage_gate.rs`、issuer/Source A、DB schema、`persist`/retain、冻结旧读端、replay/ACK；不调用 OpenCode Session/模型。COV2-22 才把本卡内存 artifact 放进 caller-owned SQLite 事务；OG-29 才评估真实 Source A 的 native 跨版本同裸 key 正例。

**Current evidence（开工基线）：** signed COV2-04 main `03b10f887368a3db5206589f5dd6947432ffe26c` 的 `pending.rs` SHA-256 `7d10a864f4c2a80f13ed96317731b36236e5a91db9819396da2647db4438f3bf`、`pending_payload.rs` SHA-256 `8b5b0eb27e1365780c9e92cbc40fab32183b22a1e9e375d4d527b54f83cc5520`，均为 0644；COV2-23 checked borrow 在只读 `checkpoint.rs`，COV2-04 已有 bounded outer codec，而生产 v2 seal 与 versioned inner projection 尚未接通。v1 `SealedPendingArtifact::seal` 与 `PendingPayloadProjection::seal_bound_reasoning` 仍走原路径。隔离补丁/未运行测试不作 PASS；开工时拟仅修改两个生产文件与五份文档；首轮 T-5 后按 G-09/I 精确追加一个既有架构守卫测试文件，最终 I 为八文件。

**Acceptance criteria:**

- [x] AC1：唯一生产 v2 native seal 入口直接从同一个 payload 调用 COV2-23 已审 `prepare_pending_native_versioned_seal`，只接收 COV2-19 sealed companion；不能从 caller DTO、独立 `PreparedPendingNativeSeal` 或可替换标签造 native 权限。不同 payload、Ordinary/Partial、跨版本同裸 key 在此真实 seal 路径按前驱门拒绝。
- [x] AC2：header `PendingBinding`、prepared alias、payload source commitment 与 companion 的 session/scope/checkpoint/parent/versioned claim plan 必须在内存 seal 前一致；任一变异无 sealed artifact。
- [x] AC3：`pending_payload.rs` 的生产 `PendingPayloadProjection::seal_bound_versioned_reasoning` 只消费 AC1 的同源 checked `batch/binding/associations` 和该已封闭 companion 的有序 versioned claims，按原顺序逐项核 `(version,key)` 与 checked identity 对应后写入版本化 inner association/已审 descriptor MAC；任一错配拒绝，不调用 v1-only `seal_bound_reasoning`、不把裸 key 当 version、也不从普通 projection/DTO 重组 native 权限。
- [x] AC4：outer-v2 bytes 仅由 COV2-04 有界 codec 编码，原 header-v1/现存 MAC 域签认证的**精确** outer bytes；authenticated v2 decode 恢复同一 versioned tuple 与 redacted projection，未认证输入拒绝。
- [x] AC5：`seal` 只生成内存 artifact；成功或任何错误均不写 pending chunk/header/alias、claim lease、marker/receipt/ref，不产生成功 ACK；超界输入只消费 COV2-04 已审容量门而不得另开无界分配路径。

**独立谓词计数 / 继承：** AC1 是真实 seal 入口调用已审 COV2-23 gate 的单个集成结果；AC2 是 artifact binding；AC3 是同一 checked borrow 到 versioned inner projection 的身份配对结果（含生产 `pending_payload.rs` 接线）；AC4 是 exact-byte outer wire/MAC；AC5 是 seal-only side-effect boundary。COV2-23 负责同源/裸 key 判据本身，COV2-04 负责纯 codec 各边界，本卡必须证明生产 seal 实际经过二者，不重复定义其内部谓词。所有 negative fixture 为 deterministic synthetic，不证明 OpenCode Source A。

**Verification:**

- [x] VER1→AC1/3：`source .env.test && cargo test --lib internal::ai::capture::pending::tests::cov2_versioned_native_seal_consumes_checked_borrow -- --exact`（同一 sealed payload 成功；换 payload、Ordinary/Partial、native 双版本同裸 key 经**实际 seal**入口拒绝；断言生产 `seal_bound_versioned_reasoning` 已执行、inner association 的 version/key/slot/已审 descriptor MAC 与同一 companion 的有序 claim/identity 逐项一致，v1/v2 descriptor MAC 域或 `(version,key)` 身份互换必须拒绝，错配不落普通或 v1-only projection；1/1）。
- [x] VER2→AC2/5：`source .env.test && cargo test --lib internal::ai::capture::pending::tests::cov2_versioned_native_seal_binding_and_zero_write -- --exact`（source commitment/session/provider/scope/checkpoint/parent/alias/claim 逐项变异，成功与失败前后 DB artifact/lease/marker/receipt/ref 相同；1/1）。
- [x] VER3→AC4/5：`source .env.test && cargo test --lib internal::ai::capture::pending::tests::cov2_versioned_native_seal_authenticated_outer_roundtrip -- --exact`（header1+outer2，MAC exact bytes、typed decode 还原同一 inner versioned association 与 outer `(version,key)` claim tuple/order、COV2-04 bounds 的真实 seal 消费和未认证拒绝；复核 v1 `seal_bound_reasoning`/原字节未改变；1/1）。

**Focused evidence（2026-10-14 05:00 UTC，owner-only material 边界修复前候选）：** 本卡三项 VER 与六项继承均由 `.env.test` 下的精确 selector 实测 1 passed/0 failed/0 ignored，九份私有日志索引 SHA-256 `fc888d8e69afbfa8bcbed9dcfb12ed78cccf42033c0b38287035ae74410e466d`。成功 seal 前后 DB artifact/lease/marker/receipt/ref 保持一致；claim 变异是在 sealed companion 构造阶段拒绝，未冒称进入 seal。初轮 strict Clippy 的 `too_many_arguments` 确定性失败保留为失败历史；收拢 checked material 后上述九项已全部重跑。T-5 full、最终质量门、唯一 Claude 与 signed main 尚未完成；synthetic case 不计 OpenCode Source A。

**最终同源边界修复后 focused 收据（2026-10-14 05:12 UTC）：** `VersionedPendingSealMaterial` 移至外层 `pending.rs`，其 payload/checked borrow/companion 字段私有；仅 `SealedVersionedPendingArtifact::seal` 在直接调用 COV2-23 gate 后能构造，内层投影只按值消费这一组材料，不能独立替换 payload。无需第二次 source 扫描或复制 opaque 正文。三项 VER 与六项 mandatory ER-04 在该最终生产源码上各精确 1 passed/0 failed/0 ignored，私有日志索引 SHA-256 `86b69f1d8ba3be516971a0da3e02aa5ddeac74cbeada43774f5cb83b1c7fe1f6`；此前 `fc888d8e69afbfa8bcbed9dcfb12ed78cccf42033c0b38287035ae74410e466d` 是边界修复前候选，不替代本次结果。T-5 双 env full、最终质量门、唯一 Claude 与 signed main 仍待。

**最终源码质量门（2026-10-14 05:13 UTC）：** `cargo +nightly fmt --all --check`、`libra diff --check` 与 `cargo clippy --all-targets --all-features -- -D warnings` 在 owner-only material 修复后均 exit0，私有质量日志索引 SHA-256 `7b0e008e680ef503de27807451e70658fd66e969970f5e36dcdabc717e758bee`。初轮 `too_many_arguments` Clippy exit1 已修且保留失败日志，不计 PASS；T-5 full、最终单 Claude 与签名 main 尚待。

**最终 focused/质量收据（2026-10-14 08:19:08 UTC）：** 三项 VER 与七项 mandatory ER-04 各精确 1 passed/0 failed/0 ignored，包含追加的 `reasoning_pending_seal_borrows_only_bound_native_witness`；十份私有 focused 日志索引 SHA-256 `39306128fdc72bbb2ae9bca583f186f2d8bfda693b68fb1a38795b815c9e31d5`。同一最终生产源码的 nightly fmt、`libra diff --check`、strict all-targets/all-features Clippy `-D warnings` 均 exit0，质量日志索引 SHA-256 `d0c676eb165802886f6f700d869d813db610a0fc7d09bf5e9759c452b71e979b`；当前生产/守卫 SHA-256 依次为 `fa486918ea80a0f818663116cee3293f8255ad839f34d5a25c6c4968c30cba33`、`3e74e26c7fb70f024bb7ef7d68289472f57b00cf0b02dc9f431fd51fcfce158e`、`89540b4675f4f2d1fe0ff112e4004f52b066a4c206ff8c238f726c4fa154ef3b`。

**T-5 双 env 完整测试收据（2026-10-14 08:19:08 UTC）：** 首轮 10102/10103、1 failed、23 skipped、exit1 是缺少新增合法架构调用点；失败日志 SHA-256 `1d70392229a6fdb29c995a0690679aa86b173184316f634161cf02f169d90d1e`。修复守卫后第二轮同样 10102/10103、1 failed、23 skipped、exit1，唯一失败为旧 `opencode_installer_rejects_non_regular_config_paths_without_blocking` 在高并发负载下超过每组 5 秒墙钟断言；失败日志 SHA-256 `129610d12c36494ba8769e166b079cd2f4e7e20bf10f789c23863771a4fc45e5`。该用例随后单独运行 1/1，通过日志 SHA-256 `3219d447ff82032ef792dfa2ccf068391cd9f575177e5d6d1ae3c17c4fef26c6`。第三轮沿用同一最终生产源码、同一 `.env.test`+`.env.live-test` 与未过滤的 `cargo nextest run --all --no-fail-fast --retries 2`，仅以 runner 环境 `NEXTEST_TEST_THREADS=4` 降低机器负载，**Summary [2058.924s] 10103 tests run: 10103 passed (2 slow), 23 skipped**、exit0；原始日志 SHA-256 `2908fff258247c857ae857a4759fcde9b925c212cc5912fa373ece16b5373029`。旧计时用例及新增架构守卫在第三轮分别 PASS [10.097s]、PASS [0.082s]。两个失败全量均不计通过；没有 OpenCode Session/模型实时集成进入 CI。唯一 Claude 与 signed main 尚待。

**COV2-24 签名源码交接（2026-10-14）：** COV2-24 最终 5/5 AC、3/3 VER 与七项 mandatory ER-04 精确 1/1（十 selector；focused index `39306128fdc72bbb2ae9bca583f186f2d8bfda693b68fb1a38795b815c9e31d5`），nightly fmt/strict Clippy/diff exit0（quality index `d0c676eb165802886f6f700d869d813db610a0fc7d09bf5e9759c452b71e979b`），双 env T-5 `NEXTEST_TEST_THREADS=4` 10103/10103 passed、23 skipped、exit0（raw `2908fff258247c857ae857a4759fcde9b925c212cc5912fa373ece16b5373029`）；唯一 Claude Code PASS、P0/P1/P2=0、P3=3（raw `a6d467bcdc33c42fff3c45f55681ebbaefa213d96cfd59a8b901c5d3dff942f7`）；八路径 0644 manifest `cd43b9af3980c8f3e90ce5751df7855ebaf1f604e2803130f0d32d9556b46e98`；signed+DCO main `c56de683acf71d7327436eef7ef9185d97dcff27` 与远端同 SHA。两轮失败 full 保留为失败历史；COV2-24 交接时安全 CI 仍在运行；其后 `c56de683`/前次 `5b0e7fc1` 的未完成旧 Actions 按用户新 commit 策略取消，cancelled 不记 D 组通过。 此前候选时点「单审/签名待」的段落只说明当时状态，不覆盖本交接。

**Mandatory ER-04 inherited regressions（不计 G-03 VER）：** 因本卡修改 `pending_payload.rs` 的 inner projection，同一最终源码须重跑既有 `source .env.test && cargo test --lib internal::ai::capture::pending::tests::reasoning_authenticated_pending_rejects_inconsistent_identity_claim_associations -- --exact`（v1-only，1/1），并重跑 COV2-23 的 `cov2_pending_native_versioned_source_borrow_rejects_recombination`、`cov2_pending_native_same_bare_key_fails_closed`，以及 COV2-04 的 `cov2_cross_version_same_key_authenticated_codec_roundtrip`、`cov2_pending_v2_mac_and_bounds_fail_closed` 和 `cov2_v1_pending_bytes_and_mac_frozen`，各须 exact 1/1。新增第七项既有闭集守卫：`source .env.test && cargo test --test compat_agent_architecture_guard reasoning_pending_seal_borrows_only_bound_native_witness -- --exact`，必须 1/1 且调用点集合仅为既有 `with_bound_reasoning` 与本卡 `seal_bound_versioned_reasoning`。首轮 T-5 此项三次同一断言失败的完整日志保留，不作为通过证据。COV2-04 纯 codec 同 key 正例与 COV2-23/24 native 同 key 负例不得互换；0 matched/skip 不算 PASS。

**Full-suite trigger:** T-5：本卡显式加严要求全量，因为在两个生产文件接通 native pending 权限、versioned inner projection 与可签 outer-v2 artifact；最终源码上先跑三项 focused（VER1/3 对 inner/versioned tuple 与 authenticated outer 各自具名断言）、mandatory ER-04、nightly fmt、strict all-targets/all-features Clippy，再跑双 env `cargo nextest run --all --no-fail-fast --retries 2`；唯一 Claude Code 在最终字节明确 PASS 后才 signed+DCO main Push。OpenCode Session/模型复杂验收仍只本机，不放 CI。

**Dependencies:** COV2-04（传递依赖 COV2-23/21/20/19）；须消费前驱真实最终 A/B、适用 focused/full、fmt/strict、唯一 Claude PASS、signed+DCO main SHA/hash/mode；不把继承 C/D 的 locally-accepted 冒称 complete。

**Deliverables:** 无独立文件；封闭 in-memory seal API 与 versioned inner projection、`pending.rs` 现有 test module 的三个具名集成测试及 inner projection 的 v1 继承回归、逐文件 SHA/mode 与评审/验收事实写回本计划/status，随唯一发布点交付。

**Implementation write set:** I-core 精确五文件：`docs/development/plan/plan-20261013-coverage-schema.md`; `docs/development/plan/plan-status.md`; `docs/development/tracing/coverage-v2.md`; `docs/development/tracing/agent.md`; `COMPATIBILITY.md`；本卡生产/现有测试模块落点 `src/internal/ai/capture/pending.rs`; `src/internal/ai/capture/pending_payload.rs`；首轮 T-5 失败后仅追加既有 `tests/compat/agent_architecture_guard.rs` 的 `reasoning_pending_seal_borrows_only_bound_native_witness` 精确闭集一处。`checkpoint.rs`、`coverage_gate.rs`、冻结 decoder、`sql/**` 与 parent 计划在本卡只读；若实际需要新增写入，先修 G-09/I 与粒度再复审。

**Release write set:** N/A；C/D 继承 COV2-REL-01。

**Files likely touched:** I-core 上述五文件；`src/internal/ai/capture/pending.rs`; `src/internal/ai/capture/pending_payload.rs`; `tests/compat/agent_architecture_guard.rs`（仅上述闭集）。

**Docs and compatibility impact:** `docs/development/tracing/coverage-v2.md`、`docs/development/tracing/agent.md`、`COMPATIBILITY.md` 逐文件说明 checked native borrow 经 versioned inner projection 后仅生成 authenticated 内存 artifact，尚无 durable persist/replay/Source A；`docs/commands/agent.md`、`docs/commands/zh-CN/agent.md` 与 libra-backend 命令页 N/A，因不改用户可见命令/配置/输出/错误码。若最终 diff 出现公开行为，先扩精确 I 并同卡同步。无新 `--test` target，`tests/INDEX.md` N/A。

**Rollback mode:** forward-only；无持久化路径，旧 v1 seal 不变。签名 main 后若集成缺陷，以新签名修复并重跑 COV2-22 及下游；不可让已撤销 seal API 进入 release freeze。

**Migration and rollback:** 无 DDL；若需迁移，先另立 GC-13 角色隔离卡/旧新 binary 门并重审 G-03/G-04/I。

**Security and privacy:** 只有同一个 NativeProven payload 通过 COV2-23 gate 才可进入 typed sealer；MAC 是本地 artifact 认证，不是 issuer/Source A 证明。错误、日志、计划证据不得包含候选字节或原始 key；native 跨版本同裸 key 继续 fail closed。

**Performance budget:** versioned inner association 仅由 COV2-23 已审的 `REASONING_ARTIFACT_MAX_ENTRIES`（当前 512）内 checked borrow 按有序 claim 等长构造；另消费前驱双 map 4096-entry/1 MiB guard 与 COV2-04 entry/wire-byte bounded codec，不在配对前无界分配。不得复制未上界 opaque buffer、私开第二连接或追加无期限重试。

**Estimated scope:** M（AC=5/8、VER=3/8、landing=1、production files=2；`pending_payload.rs` 的 inner projection 与 `pending.rs` 的 outer in-memory seal 均在 `src/internal/ai/capture/` 这一个行为落点，不独立发布）。

**Version increment:** N/A；family child 不发版。

**Release boundary:** family child of REL-COV2-01（12/23）；单 Claude PASS 后 signed+DCO 直推 main，无 R/tag。

**C/D coverage from:** COV2-REL-01；实际 exact-source C/D 绿后才 `done/complete`。

**Granularity:** `type=implementation; axis=同源借用到内存 pending-v2 seal 接线; recovery=forward-only; complete=yes; self-contained=yes; AC=5/8; VER=3/8; landing=1; prod-files=2; scope=M; deps=COV2-04; writeset=严格串行; release=family child; split-from=COV2-04; exception=N/A`

### Task COV2-25：原始 outer-v2 成员密钥完整性

**Task type:** implementation

**Lifecycle / Acceptance:** locally-accepted / 空；实际代码Claude R2 PASS、signed+DCO783041fe已随4c628d4 Push/source-qualified；当前源码13/26，正式0/26。原18exact、fmt/strict及冻结full10110/10110按下方收据保留，实际49含5c/ordinary29新树full仍待；Source A/C-D不授予。原签前历史状态：in-progress / 空；实现已落入批准写集；最终 r4 四项 VER、八项 mandatory 与三闭集/三上游 cat-file 补充共18个 exact selector 各1/1，八项 AC 的本机机制断言通过；r4 fmt/diff/strict Clippy exit0，冻结2b实现树的双env full 10110/10110 passed、23 skipped、exit0。最终单审与signed交付仍待，不预增源码就绪；新上游5c及29适配不在本轮full内，临时49须在其真实最终实现树另验。G-09 三文档已由唯一 Claude Code R2 实际 PASS（P0/P1/P2=0，P3=3；raw SHA-256 `078014fb07a626ba2bf80de0afdae79fbcafce4404a9f8aa3ea67df4221d65d9`）、nightly fmt/diff/strict Clippy exit0、hook contract 11/11（quality index `bea125fa600546496462c208f1d268de9b46b5b66d2adbfa38cbb4f1ef71d085`），signed+DCO main `2b80a56242c89518e706eaca18ceb4b0d66798d6` 已 Push 且 Libra/GitHub remote 同 SHA；三路径最终 manifest `c0a907fed96d481125bdeed1a931daf070b497bd98e41086c22cb3cf9f874eab` 逐字节对拍。协调本身不算本卡验收。

**Description:** 自 COV2-22 拆出。密钥所有者读取既有 repository key、预期 capture scope/storage/root/deadline 与原始 canonical outer-v2 bytes/MAC；在 COV2-04 有界 authenticated decoder/preflight 后精确认证原始 binding、嵌套 `payload.redacted`、reasoning source、ordered versioned associations、coverage 和 ordered `(version,key)` tuple，只颁发本地 `PendingIntegrityToken` 或等价 integrity-only 结果。原 v1 成员入口、字节和 MAC 域不变。本卡不生成 native provenance，也不做 persist/ACK。

**Out of scope:** COV2-22 durable writer/loader/事务，COV2-08 replay/late ACK，SQL/schema，issuer/locator/Source A，OpenCode Session/模型测试。测试用 adversarial envelope 仅为机制证据。

**Current evidence（历史开工快照，COV2-24 signed main 基线，2026-10-14）：** 当前 `pending.rs` 既有 `versioned_native_seal_fixture()` 及所返回结构位于 private `#[cfg(test)]` 模块；COV2-25 将在 `pending.rs` 既有 `#[cfg(test)]` 模块内新增 Unix-only VER 测试：`Some/Some` native 正例必须借现有 fixture 真实调用生产 `SealedVersionedPendingArtifact::seal` 并保留其原始 canonical bytes/MAC，不能手写或测试重签；唯 `None/None` 本地完整性正例可由受控 COV2-04 bounded codec 构造省略 source 的 canonical outer-v2，再走现有 `CaptureScope::sign_pending_envelope_until` 仓库 key 正常签认路径，供 key-owner 校验。后者仅为 synthetic integrity-only fixture，不是 production native seal、durable persist 或 Source A。当前 `src/internal/ai/capture/key.rs:282–537` 包含 whole-envelope MAC 与只接受 outer-v1 的 `verify_pending_reasoning_members`，`observed_agents/reasoning.rs:628–684` 是不可序列化 token/stamp；`tests/compat/agent_architecture_guard.rs:10538–10546` 闭集限定 `from_authenticated_members` 的 key owner。COV2-24 signed main `c56de683acf71d7327436eef7ef9185d97dcff27`/remote 同 SHA；八路径 manifest `cd43b9af3980c8f3e90ce5751df7855ebaf1f604e2803130f0d32d9556b46e98`。当前 `pending.rs` SHA-256 `fa486918ea80a0f818663116cee3293f8255ad839f34d5a25c6c4968c30cba33`、`pending_payload.rs` `3e74e26c7fb70f024bb7ef7d68289472f57b00cf0b02dc9f431fd51fcfce158e`、`key.rs` `cfb505f061883e6331a80de350774658b541dad6158fb146d518d304c4667e7a`、兼容守卫 `89540b4675f4f2d1fe0ff112e4004f52b066a4c206ff8c238f726c4fa154ef3b`，均 mode 0644；`pending.rs:373` authenticated decoder、`:482–493` sealed artifact/request、`:2452` test fixture、`pending_payload.rs:650` versioned inner seal 已在签名源码定位。COV2-25 的生产 verifier/测试仍 UNRUN，隔离候选不作本卡已验收 API。

**历史实际开工源码（当时未完成验收，不能作为当前生命周期）：** 最终基线已含上游 `35f8b15` 的 cat-file 修复；本卡只在 `key.rs` 新增原始 v2 成员完整性入口，在 `pending.rs` 的既有 Unix test 模块补真实 sealer/codec/key 机制正负例，在已有 architecture guard 登记两个精确 key-owner。`pending.rs` 生产前缀 80056 bytes 与 signed G-09 基线完全一致；`pending_payload.rs`、`reasoning.rs`、冻结 v1 decoder、SQL、公开 CLI、CI 与版本面均只读未改。None-source sentinel 已增加实际 Some 原始 source digest 不可匹配断言；五项有效 MAC 的 unknown/duplicate/malformed 输入也独立拒绝。Rust 编译、测试、full 与最终单审仍待实际结果，不能从 private rustfmt/静态检查预记 PASS。

**首次 focused 失败及前滚 fixture 修正：** r1 的闭集 signers guard 和 VER1 各具名 1/1，VER2 实际 FAIL（raw SHA-256 `467fdb6501037ae2dacdea8f0abcd8c890cd54771be40f49a2b6481e29790b09`）。原因是将同仓库的子路径当 foreign root：既有 `CaptureScope::assert_capture_key_storage_binding_until` 按 scope 和 canonical storage 选择仓库，子路径仍解析到同一个仓库，不应被当成跨根。仅把 VER2 fixture 改为第二个独立初始化仓库的 root；生产鉴权、原 v1 API 和 pending 生产前缀未改。r1 失败保留，不作网络重试或 PASS；r2 整组 exact、最终质量/full/单审仍待实际结果。

**Acceptance criteria（8/8 独立谓词，locally-mechanism-passed；r4 新增 storage 负例已具名通过；Source A 未证明）：**

- [x] AC1：v2 integrity result 的颁发以同一原始 whole envelope 的既有认证为前置。
- [x] AC2：binding 成员与原始 authenticated binding slice 字节完全相同。
- [x] AC3：redacted projection 与原始 nested `payload.redacted` slice 字节完全相同。
- [x] AC4：caller 的可选 reasoning source 与原始 nested source slice 按 `Option<raw bytes>` 精确相等。VER1 必须分别证明 COV2-24 生产 `seal` 的 `Some/Some`、以及既有 COV2-04 codec 加仓库 key 正规认证路径的 synthetic `None/None`，两者都经原始 canonical bytes/MAC 到 key-owner integrity-only token；`None` caller 对 `Some` 原始成员由 VER2 拒绝。`None/None` 纯 helper 对照只补充语义断言，不算 MAC/token 正例，更不证明 native Source A。若现有 codec/仓库 key 合法路径无法构造完整 `None` 原始成员，则 AC4 保持 UNRUN 并在实施前按 G-03 修订，不能把 helper-only 或重签 DTO 计作通过。
- [x] AC5：versioned associations 与原始 nested association slice 字节完全相同。
- [x] AC6：versioned coverage 与原始 coverage slice 字节完全相同。
- [x] AC7：caller 的有序 `(coverage_schema_version, logical_turn_key)` tuple 与同一已认证 envelope 的有界 decoder tuple 精确相等。
- [x] AC8：v2 key-owner 入口唯一返回现有本地 `PendingIntegrityToken` 权限类型。

**Verification（4/8 卡特有门；r2 下述四个具名 selector 各1/1）：**

- [x] VER1→AC1–7：`source .env.test && cargo test --lib internal::ai::capture::pending::tests::cov2_pending_v2_original_members_authenticate_exact_tuple -- --exact`（同一具名测试须含两个完整认证正例：① COV2-24 生产 `seal` 的 `Some/Some` source；② COV2-04 既有 bounded codec 生成原始 outer-v2 `None/None` source，再由仓库 key 的正常签认路径得到真实 MAC，并经 v2 key-owner 从这些原始 bytes/MAC 颁发本地 integrity-only token。两例均核全部原始成员与 ordered tuple；②是 deterministic synthetic 机制正例，不是 native Source A、也不冒称 COV2-24 native seal 能产出 `None`。helper-only `None/None` 只能作为额外断言；不能替代②。若②合法路径不可构造，本 VER 与 AC4 UNRUN，先 G-03 修订；1/1）。
- [x] VER2→AC1–7：`source .env.test && cargo test --lib internal::ai::capture::pending::tests::cov2_pending_v2_original_members_reject_substitution_or_bad_mac -- --exact`（原始 source `Some` 而 caller `None`、原始省略 source 而 caller `Some`、把省略 source 非规范地改成 `null`、错误 MAC/scope/root/各成员/tuple/畸形超界输入逐项负例；token 不颁发、key 不改变；1/1）。
- [x] VER3→AC1/AC8 与 v1 隔离：`source .env.test && cargo test --lib internal::ai::capture::pending::tests::cov2_pending_v2_member_auth_preserves_v1 -- --exact`（v1 bytes/MAC 相同，两 API 对相反 outer version 均拒绝；1/1）。
- [x] VER4→AC8：`source .env.test && cargo test --test compat_agent_architecture_guard cov2_v2_pending_original_member_issuer_is_integrity_only -- --exact`（精确 key-owner token 构造闭集与唯一返回类型；另以既有 proof/ACK 闭集 ER-04 回归，不合并计数；1/1）。

**实现接线与 Optional-source 约束：** 未接生产 caller 的 v2 verifier 及其私有类型使用范围限定的 `allow(dead_code)`，COV2-22 接线时复核并删除不再需要的限定；Unix-only test helper/VER 各自 cfg 匹配，Windows strict 编译不因 Unix-only seam 放宽。`None/None` 的 source digest 固定为 `SHA256(b"libra/pending-v2/no-reasoning-source\0")`，只作域分隔的本地完整性 sentinel；VER1 必须额外断言该 token 不匹配实际 `Some` 原始 source 的 digest stamp，不能据此恢复 native proof。此为 AC4/AC8 的断言细化，不新增行为轴或计数。

**AC4 验收边界：** 当前生产 native seal 要求完整 source commitment，只能提供 `Some/Some` 的 production-sealed 正例；COV2-04 的 `VersionedPendingPayloadProjection::new_codec` 接受 `None`，但这只是编码可行的源码证据，尚未证明仓库 key 完整认证路径。VER1 的第二个 `None/None` 正例必须经该 codec 的真实 canonical bytes、正常仓库 key MAC 和 v2 key-owner token，且仅授本地完整性；纯 helper 通过不计。不能构造时为具体阻断，先 G-03 改卡而不验收 AC4。开工时所有新 VER 均 UNRUN；现在 r2 四项均具名1/1，含 None-source sentinel 对真实 Some source digest 的负例，仅为本地完整性机制证据。

**Mandatory ER-04 inherited regressions（不计本卡四项 VER）：** 以下八个 selector 均已在 COV2-24 候选时点的现有源码中按精确名称定位；在 COV2-25 最终源码上逐条运行，每条须实际 1 passed/0 failed/0 ignored。0 matched、skip、失败或旧源码结果不是 PASS。

- `source .env.test && cargo test --lib internal::ai::capture::key::tests::reasoning_pending_members_require_original_mac_and_exact_claim_membership -- --exact`（原始 v1 成员 MAC/claim 关联）。
- `source .env.test && cargo test --test compat_agent_architecture_guard reasoning_pending_restore_has_closed_issuer -- --exact`（本地 issuer 精确闭集由原 v1 owner 扩为 v1 owner 加本卡 v2 owner，其余 native/proof 构造闭集不放宽）。
- `source .env.test && cargo test --test compat_agent_architecture_guard reasoning_pending_pre_v2_reader_oracle_is_immutable -- --exact`（冻结旧读端）。
- `source .env.test && cargo test --lib internal::ai::capture::pending::tests::cov2_v1_pending_bytes_and_mac_frozen -- --exact`（v1 canonical bytes/MAC）。
- `source .env.test && cargo test --lib internal::ai::capture::pending_payload::tests::cov2_cross_version_same_key_authenticated_codec_roundtrip -- --exact`（跨版本同 key、认证 codec 与有序 tuple）。
- `source .env.test && cargo test --lib internal::ai::capture::pending_payload::tests::cov2_same_version_duplicate_and_unknown_reject -- --exact`（同版本重复与未知版本）。
- `source .env.test && cargo test --lib internal::ai::capture::pending_payload::tests::cov2_pending_v2_mac_and_bounds_fail_closed -- --exact`（MAC、entry/wire 上限）。
- `source .env.test && cargo test --lib internal::ai::capture::pending_payload::tests::cov2_versioned_producer_claim_count_guard_precedes_projection -- --exact`（producer 4096/4097 分配前边界）。

复杂 OpenCode Session/模型验收仅 macOS 本机，不进 CI。

**Full-suite trigger:** ER-13 T-1：`key.rs` 是共享单一认证来源，且架构守卫修改；最终源码运行双 env `cargo nextest run --all --no-fail-fast --retries 2`、nightly fmt、strict all-target/all-feature Clippy、精确 focused。家族首次 release bumped exact-tree 仍须全量；后续仅上游纯 patch 顺延可按 COV2-REL-01 的逐路径同一性规则复用已绿全量。

**r2 focused 实际收据：** 日志索引 SHA-256 `6be14aba178bf940dcf9012ee2b1cc05e8c5c0334fddf54b9cd70df78e6b861b`；四项 VER、八项上述 mandatory、额外三个旧 key/proof 调用闭集，以及随临时49乘载的上游 `commit_message_body`/tree mode/raw commit 三项 exact，共18 selector 各报告1 passed/0 failed/0 ignored。r2 不含独立 foreign-storage 参数负例，其原收据把跨 root/storage 的组合拒绝列为单独 storage 门属于过度表述，已被下述审查指出；本段仅保留历史，不代表修正后最终源码通过。其中完整 MAC 的 malformed JSON、unknown outer/payload、duplicate outer/payload、None/Some/null、scope/storage/root/deadline、tuple与同key跨envelope拼接拒绝均在 VER2，第二独立repo root替代错误的同仓子目录fixture。Some/Some正例走真实生产sealer，None/None走真实codec+existing-key MAC，均synthetic机制，不是模型native Source A。最终质量/full/单审、signed+DCO main/remote及家族C/D尚未据此通过。

**r2 strict 检查中断：** nightly fmt/diff check 已 exit0；strict all-target/all-feature Clippy 子进程初始化后约10分钟无CPU/日志进展，root仅停止本次拥有的20个 cargo/clippy进程。此轮 strict 实际 exit1，不作PASS，也不判源码编译缺陷或网络错误。原始日志与中断receipt保留；改用 runner-only `CARGO_BUILD_JOBS=4` 重跑同一 `cargo clippy --all-targets --all-features -- -D warnings`，无项目/CI配置或检查面缩减。最终full、单审与signed交付仍待。

**源码审查 R1 FAIL 与测试补齐：** 唯一 Claude Code R1 实际 FAIL（P0/P1=0、P2=1、P3=4；raw SHA-256 `5c621a92df654579df39509008ced7fa62c6848ac6afff570b3c7c9557f0c1b4`），未发现生产 verifier 缺陷，P2 是 r2 收据声称独立 storage 拒绝而测试只覆盖 foreign root。仅在既有 VER2 测试添加 original scope/root/bytes/MAC + 第二 repo storage 的独立负例；生产文件 key.rs 和 pending 生产前缀未改，新 exact/full/strict/最终单审待。P3 边界如实登记：post-parse deadline 的位置由结构 guard 验证，runtime expired case 仅早期deadline；VER3的v1历史bytes/MAC同一性依赖独立mandatory frozen用例，不由VER3单独证明；member多次stream比较是整-envelope byte gate后的防御；AC/VER checkbox仅 `locally-mechanism-passed`，不等于最终 Acceptance。

**r4 最终 focused 补齐：** 新增 independent foreign-storage 负例后，r4 全部18 exact selector 再次各1/1，日志索引 SHA-256 `066a7684766b957208e3774c91dd97b3d0a59fff045670c89fb423468bc5d45f`。VER2 此轮实际分别拒绝 original storage + foreign root，以及 original root + foreign storage；r2原过度收据不倒写。r3四jobs质量 fmt/diff/strict exit0（index `208360fb18549e1ff7867e976442352e61ae72f2d549e2bfbbc3c9814a5ca5fb`）属于新增storage断言前的旧快照，最终r4仍独立跑质量/full/单审，不能从旧质量预记通过。

**r4 最终质量与临时接线预检：** nightly fmt/diff/四jobs strict all-target/all-feature `-D warnings` 实际exit0，quality index SHA-256 `8cdf4695a85138e2e7c2ac1cdfc9c4f5214e10385a8d81ee8a0367e3a4a5b7d2`。固定driver/scanner hash后只读AST实扫640个src文件，输入manifest `8a1d7b82ee1f02b4716b52b78ec3e86a235fe8501d53b28b2ed583dd43e122ed`；唯一生产表达式新增为本卡authenticator→boundeddecoder内部引用，authenticator自身仍零生产caller。只证明当前CLI/provider接线未启用未完v2，不证明任意embedding不可调用，不证明native Source A。下步在该最终实现/测试/行为文档快照运行双env全量；最终单Claude、签名Push及家族C/D仍待。

**r4 冻结源码 full 实际收据（2026-10-14 13:36:45 UTC）：** 同时加载 `.env.test` 与 `.env.live-test`，runner-only `CARGO_BUILD_JOBS=4`、`NEXTEST_TEST_THREADS=4`，实际 `cargo nextest run --all --no-fail-fast --retries 2` exit0：`Summary [2645.131s] 10110 tests run: 10110 passed (2 slow), 23 skipped`；raw SHA-256 `01dafbcbf51f87db809fa1c3b57ad44333913286f0cc72daf181e6735e34b789`。full期间八个本卡候选文件的SHA/mode与冻结快照同一；生产 key SHA `97575b6a46fe05a86e25b011d9d33292ce07ba3bfb39bf96f58a433c24cca4f6`、pending test后像 `ec9fa86821d21148c66b839021950951802e074dcdffb71f5b885366b1b31ed3`、architecture guard `be9bdd53b8cba8f8071fc2c9019f003122f858829e3715265058a3c55c58ab7a`。23 skipped不作验收通过，synthetic不证明Source A。该full基于HEAD `2b80a56242c89518e706eaca18ceb4b0d66798d6` 加本卡精确I，包含35f8 cat-file修复；期间新上游 `5c0eb71c57f5cf73e8f711e459826758086a4c11` 与官方OpenCode29/新普通适配均未写入此冻结树，本收据不能冒称它们的最终源码验收。最终单Claude/signed交付、最新上游合流与临时49最终源码full/C-D仍待；source-ready12/26、formal0/26不预增。

**Dependencies:** COV2-24 实际最终 A/B、适用 focused/full、fmt/strict、唯一 Claude Code PASS、signed+DCO main 与远端同 SHA、逐路径 SHA/mode；另先完成本次三文档独立 G-09 协调单审及 signed main。只消费 source-qualified 前驱，不冒称继承 C/D complete。

**Deliverables:** 无独立仓库文件；key-owner verifier、现有 test module/compat 守卫、逐文件 SHA/mode 与 review/测试收据写入本计划及 status，随唯一 release 点交付。

**Implementation write set:** I-core 精确五文件：`docs/development/plan/plan-20261013-coverage-schema.md`; `docs/development/plan/plan-status.md`; `docs/development/tracing/coverage-v2.md`; `docs/development/tracing/agent.md`; `COMPATIBILITY.md`；另仅 `src/internal/ai/capture/key.rs`; `tests/compat/agent_architecture_guard.rs`; `src/internal/ai/capture/pending.rs`（仅既有 `#[cfg(test)]` 模块内以 `#[cfg(unix)]` 限定的 fixture/helper 与 VER1–3 测试体；生产实现不可改）。`pending_payload.rs`、`observed_agents/reasoning.rs`、SQL、冻结 decoder、父计划、Cargo.toml、tests/INDEX.md 只读；如确需写入，先修 G-03/G-09/I 并复审。

**Release write set:** N/A；C/D 继承 COV2-REL-01。

**Files likely touched:** 上述 I-core 五文件；`src/internal/ai/capture/key.rs`; `tests/compat/agent_architecture_guard.rs`; `src/internal/ai/capture/pending.rs`（仅 Unix-only test fixture/helper 与 VER1–3 测试体）。

**Docs and compatibility impact:** I-core 仅说明 integrity-only proof、v1 不变、durable loader/replay 仍待；不改公开 CLI/config/error-code/网站。若实际公开行为变化先按 ER-06a 修精确 I；既有 compat test target，无新增 tests/INDEX.md/Cargo.toml/compat README 写权。

**Rollback mode:** forward-only；无 DDL/数据重写；失败以新签名 main 前滚、重跑 COV2-22/08 下游门，原始 artifact bytes 保持可修复。

**Migration and rollback:** 无 DDL；若需要 schema/migration，先按 GC-13 独立角色隔离卡及旧/新 binary 门重审。

**Security and privacy:** 复用历史 v1 pending Envelope MAC domain，不另造新 key/domain；MAC 是本地完整性而非 native 来源。原始成员不入日志；在 tuple allocation 前运行 COV2-04 count/wire preflight，outer envelope 继承64 MiB上限。VER2的错误MAC、scope/root、无key、替换/重排/跨envelope拼接均为 AC1–7 的具名负例；既有 key/scope/cap/version/canonical 的独立强制门归 ER-04。VER4 精确守卫 key.rs 对 `PendingIntegrityToken::from_authenticated_members` 的生产构造闭集；返回类型不能铸 native proof/issuer 或 recovery ACK，这些权限仍由各自 owner 验证，synthetic 不构成 Source A。

**Performance budget:** 借用有界 raw member slices，避免为 64 MiB envelope 再造全量拷贝；不得新增无界 Vec/HashSet 或无 deadline 重试。

**Estimated scope:** M（AC1 whole-envelope prerequisite、AC2 binding、AC3 redacted、AC4 optional source、AC5 associations、AC6 coverage、AC7 ordered tuple、AC8 integrity-only return：八个独立谓词；VER=4/8、landing=1、production files=1）。实际实现若新增第九独立谓词或另一行为轴，先按 G-03 再拆。

**Version increment:** N/A；本卡不发版。

**Release boundary:** family child of REL-COV2-01（13/23）；逐卡唯一 Claude Code PASS 后 signed+DCO 直推 main，无 R/tag。

**C/D coverage from:** COV2-REL-01；最终 exact-tree C/D 通过后才 done/complete。

**Granularity:** `type=implementation; axis=authenticated original outer-v2 member integrity; recovery=forward-only; complete=yes; self-contained=yes; AC=8/8; VER=4/8; landing=1; prod-files=1; scope=M; deps=COV2-24; writeset=严格串行; release=family child; split-from=COV2-22; exception=N/A`

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

**Description:** 只把经审查的 23 implementation+2 audit COV2 family 按一个稳定 patch 发布并验证；不引入新行为，冻结树不含 OG-27 source。COV2-21/23/04/24/25/22 必须是实际 signed main A/B+单 Claude PASS 交付，12 AC/12 VER 的 release 计数不因拆卡静默增删。

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

**Verification:** 以下 VER3–11 首次发布在同一最终行为源码验收；若上游仅推高 patch 版本且上述逐路径等价成立，可沿同实现复用已签 full/本机原始证据，但必须把旧证据映射到新 release SHA 并重新核版本面、安装、安全 CI 与四平台发布，不把不同源码或不同模型/配置的旧门冒称通过。

- [ ] VER1→AC1 compare each source path hash/mode handoff with frozen source SHA.
- [ ] VER2→AC2 compare the freeze diff with approved card paths and OG-27 exclusion; execute `libra log <last-stable-tag>..<freeze-main-sha>` and classify **every** carried commit by owning plan/card, signed SHA, review, required C/D state and unmet release gate. COV2 children inherit this release C/D；OG-24/25/26 须核其 A/B+Claude+signed main 及本次安全/安装门，其下游 `REL-FAST-OG-04` C/D 保持 pending；其它计划乘载提交若有应先满足的独立前置则阻断冻结。Verify `approved_build.rs` `.2` admission source is in the actual release tree; installed-binary registration remains a separate test (`libra diff --check` is normal hygiene outside this count).
- [ ] VER3→AC3 **在首次实际 patch bump 后的最终行为源码树上重新运行** macOS manual `source .env.test && cargo test --features test-live-agent --test agent_opencode_bridge_test cov2_macos_fake_exporter_live_session_capture -- --ignored --exact --nocapture --test-threads=1` (1/1)，并核对行为文件 SHA/mode。
- [ ] VER4→AC4 同一最终行为源码树重新运行 `source .env.test && cargo test --features test-live-agent --test agent_opencode_bridge_test cov2_macos_fake_exporter_import_session_capture -- --ignored --exact --nocapture --test-threads=1` (1/1)。
- [ ] VER5→AC5 同一最终行为源码树重新运行 `source .env.test && cargo test --features test-live-agent --test agent_opencode_bridge_test cov2_macos_failed_export_session_zero_write -- --ignored --exact --nocapture --test-threads=1` (1/1)。
- [ ] VER6→AC6 同一最终行为源码树重新运行 `source .env.test && source .env.live-test && cargo test --lib internal::ai::capture::live_pipeline::tests::export_runner_lease::end_to_end::opencode_extraction_failure_continues_checkpoint -- --ignored --exact --nocapture --test-threads=1` (1/1；若 COV2-12 审核后改名，先把此处命令同步为其实际新 selector，并记录旧→新映射)。`5cafe322` exit-101 只保留负面基线。
- [ ] VER7→AC7 同一最终行为源码树重新运行 `source .env.test && cargo test --features test-live-agent --test agent_opencode_bridge_test cov2_macos_fake_exporter_candidate_live_partial_no_plaintext -- --ignored --exact --nocapture --test-threads=1` (1/1)，核 exporter-executed 与各零 sink 断言。
- [ ] VER8→AC8 同一最终行为源码树重新运行 `source .env.test && cargo test --features test-live-agent --test agent_opencode_bridge_test cov2_macos_fake_exporter_candidate_import_partial_no_plaintext -- --ignored --exact --nocapture --test-threads=1` (1/1)，核各零 sink 断言。
- [ ] VER9→AC9 由 COV2-17 所有者在 DEP-COV2-MODEL 下预登记已授权 isolated OpenCode build、模型、完整无密 argv 和 ≤10 分钟上限，再在**首次 bumped 最终行为源码树**上重新执行 genuine visible-only 观察；实际原始命令、exit、无敏字段类型/数量和 keyed digest 写回本计划。不得登记 issuer、调用 live mint、使用 COV2-17 的过期样本或把观察当 Source A。
- [ ] VER10→AC10 audit COV2-16's exact 9/9+1/1+1/1+1/1 results and no-skip source scan；逐行为文件 SHA/mode 必须与 bumped T-2 树完全一致，否则在该树重跑完整 12-case matrix，不能只引用旧审计 SHA。
- [ ] VER11→AC11 audit COV2-17's exact 7/7+1/1+1/1+1/1 results and OpenCode-row/non-skip source scan；逐行为文件 SHA/mode 必须与 bumped T-2 树完全一致，否则在该树重跑完整 10-case matrix，不能只引用旧审计 SHA。
- [ ] VER12→AC12 audit the source-qualified C/D ledger for completeness and exact release-SHA consistency.

**Full-suite trigger:** T-2：首次 COV2 stable release 必须在最终已 bump 实现树运行双 env `cargo nextest run --all --no-fail-fast --retries 2` 并实际全绿。若其后仅因上游稳定版号前进而顺延本计划 patch 版本，且生产源码、测试及行为文档逐路径 SHA/mode 与已全量通过的实现树完全相同，只有版本面与工具链生成的 root package lock 版本改变，可依用户授权复用同实现的既有全量/手工证据；记录两树逐路径等价、原全量 raw SHA/exit0、最终版本面 parity 与新 release SHA，不把旧 CI 当新 SHA 通过。任何非纯版本源码/测试/行为文档变化均在最终 bumped 树重新跑全量及相关本机门；失败、取消或未运行不能复用。

**Shared-path coordination（非AC/VER/I/R变更）:** DEP-OG-24-029-SHARED/RETURN约束本卡source-freeze→事实closeout的唯一writer阶段：RouteA必须OG24.3 signed-return或严格abort-return后才可由root在返回postimage写actualC/D；RouteB先本卡closeout再grant。F及R已冻结且后续只构建/发布F，当前3不被该F携带；新main不改变tag/source归属。原GC-COV2-08、VER2与其installed-F原`.2` issuer兼容登记门继续有效，仅在独立owned legacy HOME/XDG的fresh registry中按epoch0执行，不用global，不作current29/3验收或SourceA。其余12AC12VER/I/R/D依赖全保留。

**Dependencies:** COV2-01/13/02/07/14/03/19/20/21/23/04/24/25/22/08/05/09/06/10/11/12/15/18/16/17 全部本机 source-qualified A/B+review+signed main deliverables；DEP-COV2-SITE；DEP-COV2-MODEL；本卡 VER3–9 的新增/真实观察与 COV2-16/17 全22例在最终行为源码非 skip PASS。

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

**Version increment:** patch；发布前核最新稳定/上游版本，仅 patch +1，不覆盖已有 tag。首次 release 在 bumped 实现树重跑 full；后续仅上游 patch 顺延、实现逐路径等价时按本卡 Full-suite trigger 复用同实现 full，只更新版本面、工具链 lock、签名构建/安装与发布验收。

**Release boundary:** family release point of REL-COV2-01。

**C/D coverage from:** self；发布点实际 C/D 全绿后登记 25 张非发布子卡继承完成，再将父计划 `DEP-OG-13=complete`。

**Granularity:** `type=release; axis=atomic coverage-v2 family patch; recovery=immutable-release; complete=yes; self-contained=yes; AC=12/12; VER=12/12; landing=0; prod-files=0; scope=M; deps=COV2-17,DEP-COV2-SITE,DEP-COV2-MODEL; writeset=sole R publisher; release=family release point; split-from=N/A; exception=N/A`

**ER 强制门（不计 G-03）：** `source .env.test && cargo test --test compat_version_surface_sync` 必须实际 2/2；首次 bumped 实现树 nightly fmt、strict all-targets/all-features Clippy、双 env 全量 nextest、release build/install；仅上游纯 patch 版本顺延可按上述逐路径等价收据复用同实现 full，本次版本面/构建/安装与发布后门仍须实证。同一提交签名+DCO；GitHub main 安全 CI、四平台稳定构建、manifest/签名、Cloudflare/Homebrew/站点 HTTP 与指定 `curl --proto '=https' --tlsv1.2 -sSf https://download.libra.tools/install.sh | sh` 本地安装都须实际通过。每新 commit 取消旧 commit Actions；不得把被取消的旧 run 计入当前 SHA。Linux 功能仍由用户后验。

## 测试矩阵

| 类别 | CI/默认 | macOS 本机显式 | 完成判据 |
|---|---|---|---|
| v1/v2 codec/DB/identity/pending | COV2-01/13/02/07/14/03/19/20/21/23/04/24/25/22/08 精确单测与静态守卫；19 真 identity writer，20 只读事务验领，21 caller retain/故障，23 同源 NativeProven checked borrow 与 native 同 key 拒绝，04 有界 codec，24 在 `pending_payload.rs` 配对 versioned inner identity 后由 `pending.rs` 消费 borrow 完成内存 seal，25 在 key owner 复用既有 Envelope MAC 域精确认证原始 outer-v2 成员，22 真 artifact persist/编译冻结旧读端/SQL fault | 无 | v1 冻结字节，v2 独立域、跨版本同键、同版本拒绝、故障零假成功 |
| import/reader/typed | COV2-05/09/06/10/11 的 direct normalized-turn、in-memory sink、graph/doctor 命令测试；不启动 OpenCode Session | 任何发现有 exporter/Session 的案例改 `#[ignore]` 并只本机跑 | v2 live/import 字节与 digest 相等，unknown version fail closed，公开 envelope=1 |
| 新增 Session cases | 不在 CI：`#[ignore]`/feature gate；静态 guard 在 CI | COV2-12 fake live/import/failure/extraction，COV2-15 candidate live，COV2-18 candidate import；COV2-17 real visible | 每例有实际 1/1 非 skip PASS、源 SHA、隐私 sink 独立断言；fake 不当 Source A |
| 继承 22 ignored cases | 不在 CI；静态清单 guard 在 CI | COV2-16 9+1+1+1；COV2-17 7+1+1+1 | 22 个名称全覆盖，OpenCode row 实际运行，缺一个即卡/发布阻塞 |
| stable release | 首次最终已 bump 实现树全量 nextest（ignored Session 不入）、fmt、strict Clippy、security CI、四平台；后续纯上游 patch 顺延按 COV2-REL-01 逐路径同一性规则复用全量 | final-source Session 报告 + 网站/安装验证 | 版本/源码证据映射清楚、当前 SHA 的安全 CI 与 C/D 全绿；Linux 功能由用户后验 |

## 追溯表

| 来源/缺口 | 承接任务 | Libra 落点 | 文档/兼容 | 指定验收 |
|---|---|---|---|---|
| DEFER-OG25-COVERAGE-V2 / v1 四 record | COV2-01/13/11 | `observed_agents/coverage.rs`、live/import | coverage-v1/v2、COMPATIBILITY | v1 frozen 3 vectors + RG-04、production typed v2 exact |
| versioned claim/identity/pending | COV2-02/07/14/03/19/20/21/23/04/24/25/22/08 | `coverage_gate.rs`、metadata/key/checkpoint/pending_payload/pending/recovery | coverage-v2 | 03 marker/ref、19 sealed identity-v2/最终读端篡改与互换拒绝、20 read-only verify、21 同事务 retain、23 同源 checked native borrow/裸 key collision 拒绝、04 有界 authenticated codec、24 versioned inner association + 内存 source-bound seal、25 原始成员 key-owner 完整性、22 真 artifact/冻结旧读端/写故障、08 replay |
| import/reader | COV2-05/09 | `agent_import.rs`、graph/doctor | EN/zh/网站 cf | cursor/digest/fault/unknown-version/privacy |
| OpenCode privacy & pipeline | COV2-06/10/11/12/15/18 | classifier/snapshot/live/import/export | agent/tracing/integration | pure typed + ignored macOS live/import/candidate |
| 22 inherited cases | COV2-16/17 | read-only tests | 本计划与 coverage-v2 summary | 12/12 + 10/10 non-skip/source-qualified |
| 交接与稳定发布 | COV2-REL-01 | 版本面/Release | parent DEP-OG-13、status/site | 首次 bumped 实现树 nextest；后续纯上游 patch 顺延按逐路径同一性复用，当前 SHA 的 security CI、四平台、安装、网站仍实际验证 |

## 里程碑验收与回滚

| 里程碑 | 完成条件 | 发布/证据 | 恢复 |
|---|---|---|---|
| M-COV2-0 | OG-26 实际 signed main source-qualified handoff；网站 cf 隔离 owner 确认 | DEP-COV2-OG26/SITE 在本计划记录 SHA/mode/UTC | 缺口保持 pending，不覆盖脏文件 |
| M-COV2-1 | COV2-01/13/02/07/14/03/19/20/21/23/04/24/25/22/08/05/09/06/10/11 A/B+Claude+signed main 逐卡交付 | 每卡证据写本计划，仍 locally-accepted | 出错在 owner 卡前滚、重跑下游 |
| M-COV2-2 | COV2-12/15/18 与 16/17 全 22 继承 case 完整本机 PASS | final-source manual report；`DEP-OG-13=source-ready` | 一例失败回 owner，审计全部重跑 |
| M-COV2-3 | COV2-REL-01 exact-tree stable patch 与 C/D 完成 | Release/CI/site/install 真实 SHA/run | immutable release；新 patch 修复，随后 `DEP-OG-13=complete` |

### 故障恢复矩阵

| 故障点 | 可接受残留 | 恢复动作 | 禁止结果 |
|---|---|---|---|
| v2 claim 或 marker 事务失败 | pending/未完成标记与可重试状态 | 校验 owner/fence/revision 后原子重试或 doctor | claim/ref 一半成功、假 ack、覆盖 v1 |
| v2 pending 旧读端打开 | 完整原 pending bytes | COV2-22 从真实持久化 artifact 验编译冻结读端非破坏性拒绝，升级读端后重放 | 旧端改写或降级为 v1 |
| 第二版本 pending lease retain SQL 或后续 writer 失败 | 原 v1/v2 reservation 与旧 artifact 状态 | COV2-20 只读验领、COV2-21 在 caller-owned SQLite 事务内回滚两版 lease；COV2-23 只做同源 native borrow 和写前跨版本裸 key 拒绝；COV2-22 在真实 chunk/header/alias 写故障后证实两版 lease 与 artifact 全回滚；不能用 SQL probe 代替真 artifact | 第一行 lease 已变而第二行/真实 artifact 未提交 |
| identity map 未知键/超 cap/MAC 错 | 既有可见 claim 不变 | fail-closed、记录 content-free doctor 结果 | 跳过损坏条目当 absent、假成功 |
| OpenCode candidate Session | 可追溯 partial receipt | zero ordinary/typed/object/log/error sink；待真实 Source A | synthetic 冒称 native artifact |
| macOS containment 缺失 | 失败的本机测试 | 配齐 Required Seatbelt 或显式失败 | `None`/early return 伪绿 |
| release/网站/CDN 失败 | 已审核源码/可能已有 immutable tag | 分类后签名前滚/new patch，重跑受影响门 | 改写 tag/asset、用不同 SHA 的 CI/安装证据 |

## 风险登记

| 风险 | 影响 | 缓解/任务 |
|---|---|---|
| v1 fixture 被新 v2 逻辑改写 | 旧 claim/digest 不兼容 | COV2-13 三向量 + RG-04 SHA、COV2-04 codec 历史字节/MAC 与 COV2-22 编译冻结旧读端 |
| native v2 record/identity map 与密封来源、payload 或 provider 不同源 | 假 native 归档或错误身份绑定 | COV2-19 在同一事务检查来源、记录与 map，并注入真实 `identity-v2` SQL 故障 |
| native pending 的同名裸 key 从其它 payload 或跨版本 claim 重组 | 假冒 native 权限、identity alias 指向错误 turn | COV2-23 只从同一 NativeProven payload 与 COV2-19 companion 借用，并拒绝跨版本裸 key；COV2-24 在 `pending_payload.rs` 对 checked borrow 与 versioned claims 逐项配对，不从裸 key 猜 version；COV2-22 只消费其 sealed artifact 并在 persist 入口二次拒绝 native same-key；OG-29 另验版本化正例和 Source A。synthetic 或 DTO/MAC 不得当 issuer；无真实 Source A 仍不归档 |
| outer-v2 原始成员被替换、重排或从另一有效 envelope 拼接 | loader/replay 把错误 tuple 当已认证，形成假成功或错误 ACK | COV2-25 key owner 对原始 MACed bytes 的 binding/redacted/source/association/coverage/ordered tuple 逐项认证；COV2-22 真 loader 必须消费其 integrity-only 结果，COV2-08 每次 late replay 重验并保留可修复原始 bytes/MAC；任一失败零 verified payload/ACK，不冒称 native Source A |
| pending 先写 artifact 后保留 versioned claims，或以手工插入行冒充真持久化 | 过期 lease 被接管、旧端破坏性重放、假绿 | COV2-20 提供只读 versioned verify、COV2-21 提供同事务 retain、COV2-23 提供同源 NativeProven checked borrow 且跨版本裸 key fail closed，COV2-04 提供有界 codec、COV2-24 提供同源内存 seal；COV2-22 必须从真实 seal/persist/commit 运行编译冻结旧读端并注入后续 artifact SQL fault |
| 双 map 容量倍增或 unknown key 假 absent | 热路径资源超支/身份绕过 | COV2-07/14 域隔离与 aggregate cap/fail-closed |
| 本机 Session 用例因 bwrap 不存在早退 | 虚假 22/22 与 release 假绿 | COV2-12 Required Seatbelt + COV2-16/17 非 skip inventory |
| 网站 checkout 脏且落后 | 覆盖他人改动/站点源码不一致 | DEP-COV2-SITE 干净隔离 cf owner，source hash、部署 HTTP |
| v2 typed 与 native 来源混淆 | 未授权 opaque 归档/明文泄漏 | COV2-06/10/15/18、父计划 Source A 硬门 |
| 2026-10-13 旧安装版 0.30.48 不批准私有 `.2` | 当时 OG-27 注册失败，不能外推到未安装的新版本 | COV2-REL stable patch 实装后，在父计划原 issuer epoch0 流程重验；r2 binary-to-source PASS 不替代 runtime/issuer/Source A，当前仍 NO-GO |

## 性能与容量摘要

| 操作 | 单次成本/边界 | 总预算 | 验证 |
|---|---|---|---|
| 私有 v1+v2 identity maps | 解码前有界、仅两个已知 key | 合计 ≤4096 entries、≤1 MiB encoded | COV2-14 aggregate entry/byte exact tests |
| pending envelope | 继承现有上限与 MAC；v2 无无界扩展 | `key.rs` 既有 64 MiB 封闭上限，开工刷新 | COV2-20 同事务只读 verify、COV2-21 retain/rollback、COV2-23 同源 native borrow/裸 key 负例、COV2-04 entry 与 wire-byte 分别限额、COV2-24 内存 seal 消费上限、COV2-25 复用 COV2-04 preflight 后认证原始成员、COV2-22 真实 artifact/旧读端/SQL fault、COV2-08 replay tests |
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

以下各条 `0/19`、`1/19`、`2/19`、`3/19`、`4/19`、`5/19` 及已签 G-09 23 卡时点的 `0/23`、`5/23`、`6/23` 都是历史收据，不倒改过去的观察。COV2-14 已由 signed+DCO main `4558af64c09e63c4716fa68595776a8044f66cc7` 交接，COV2-24 签名前的25卡口径正式进度 **0/25**、源码就绪 **11/25=44.00%**；G-09 签署时 COV2-03 新增 VER5 与旧 catalog receipt ER-04 回归均 UNRUN；现已各 1/1，最终双 env full **Summary [1618.814s] 10078 tests run: 10078 passed (14 slow), 23 skipped**/raw SHA `c49dd3c33652d55541a0e4c0269c817460b9109cf261eab5271d90760a49e8a9`、exit0；唯一最终 Claude Code PASS、P0/P1/P2=0、P3=5/raw SHA `9ef25f92615eefcf2019212a306cfa7542ef416b693286bb99111006b3ea6df7`；signed+DCO 源码/文档 main `d717e42e0d1ec75e6ae576b21242fd1519040abd` 已 Push 且 remote 同 SHA；COV2-19 本机 8/8 AC、5/5 VER、16 项继承与 replay/架构守卫通过，最终唯一 Claude PASS（P0/P1/P2=0、P3=3；raw SHA `64068902908f4c521e380dc7e5cf593134eaa99697de208344b65f75b3c3caa8`），signed+DCO main `354ff768c406bdad74607da62002bff810147c24` 已 Push、remote 同 SHA；COV2-20 已在本机 7/7 AC、3/3 VER、全量 10087/10087 与单 Claude PASS 后由 signed main `aa0ea018ddd044f09d62a88428e812a359cef8ed` 交接；COV2-21 7/7 AC、4/4 VER 与五项继承精确测试各1/1，strict/full 与唯一 Claude PASS 后由 signed main `6b24d797583afe1c0265f6e9fcbd364566671167` 交接；COV2-23 的 8/8 AC、4/4 VER 与四项继承、两项架构闭集守卫各1/1，fmt/strict 与第二轮双 env full 10095/10095 exit0/raw 5b20f0bfcf4ab2664b89dcb806c14ba812aa5597c7c7b48ca2cac621f5ca6599，唯一 Claude PASS/raw 024c7011547dbe0cbb466e2df8d1fe3c5ed555b9b56de6eb09377bfc64a83cd1，signed+DCO main `d977d13c5e23111b93d78477b9e1a70015a111b8` 已与远端同源；首轮 full 10093/10095、2 failed 保留为失败历史；COV2-04 已由唯一 Claude PASS 与 signed+DCO main `03b10f887368a3db5206589f5dd6947432ffe26c` 交接，ER-13 `none` 未独立跑 full；COV2-24 最终 5/5 AC、3/3 VER 与七项 mandatory ER-04 精确 1/1（十 selector；focused index `39306128fdc72bbb2ae9bca583f186f2d8bfda693b68fb1a38795b815c9e31d5`），nightly fmt/strict Clippy/diff exit0（quality index `d0c676eb165802886f6f700d869d813db610a0fc7d09bf5e9759c452b71e979b`），双 env T-5 `NEXTEST_TEST_THREADS=4` 10103/10103 passed、23 skipped、exit0（raw `2908fff258247c857ae857a4759fcde9b925c212cc5912fa373ece16b5373029`）；唯一 Claude Code PASS、P0/P1/P2=0、P3=3（raw `a6d467bcdc33c42fff3c45f55681ebbaefa213d96cfd59a8b901c5d3dff942f7`）；八路径 0644 manifest `cd43b9af3980c8f3e90ce5751df7855ebaf1f604e2803130f0d32d9556b46e98`；signed+DCO main `c56de683acf71d7327436eef7ef9185d97dcff27` 与远端同 SHA。两轮失败 full 保留为失败历史；COV2-24 交接时安全 CI 仍在运行；其后 `c56de683`/前次 `5b0e7fc1` 的未完成旧 Actions 按用户新 commit 策略取消，cancelled 不记 D 组通过。；COV2-22 AC/VER 未运行；COV2-20 旧 CI 因新提交取消，不能记完整通过；0 matched、helper-only、skip 或重新计数都不构成 PASS。父计划正式 **55/67=82.09%**，Source A/DEP-OG-12 与家族 C/D 均未闭合。
**历史COV2-25签前开工快照（原文含12/26，当前source13/26按后续signed交接，不倒改原数字）：**

G-09 本次独立三文档修订在前驱 COV2-24 签名后增加 COV2-25；签名前25卡历史口径正式0/25、source-ready 12/25=48.00%，本次采纳后当前正式0/26、source-ready 12/26=46.15%。COV2-25 8 AC/4 VER、COV2-22 8 AC/6 VER 与 COV2-08 后续门均 UNRUN；家族 C/D、22例最终行为源码 macOS Session 和父 Source A/DEP-OG-12 未闭合。

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
| 2026-10-13 23:57:33 UTC | COV2-19 签名前 A/B 快照 | main 仍为前置 G-09 `3e37e55dd781e44532c6463a1a0cec484c0bb759`；本卡八文件最终 diff/mode 待唯一正式审查 | 8/8 AC、5/5 VER、16 inherited/v1/RG-04、native replay 与架构守卫各 1/1；formal 0/25、source-ready 6/25=24.00%（签名 Push 前不预增） | fmt/strict Clippy/diff exit0；双 env full **Summary [1242.578s] 10084 tests run: 10084 passed (15 slow, 1 leaky), 23 skipped**、exit0/raw SHA `20378e04b5c585c954421780520a6a62ebbf1f6c7ba0336db40dd76541058e43`；旧失败两轮不计绿 | 唯一正式 Claude Code 待 | signed+DCO main/remote 与家族 C/D、22个最终行为源码 Session、父 Source A 仍待；完成后下一卡 COV2-20 |
| 2026-10-14 00:07:12 UTC | COV2-19 本机 A/B+单审源码交付 | signed+DCO main `354ff768c406bdad74607da62002bff810147c24` 已 Push、remote 同 SHA；八文件 manifest SHA `f400931268c37875cd7131ab3c3bbf4e6178a6b4fa4d55129ee905456330aa62` | 8/8 AC、5/5 VER、16 inherited/v1/RG-04、native replay 与架构守卫各1/1；formal 0/25、source-ready **7/25=28.00%** | fmt/strict Clippy/diff exit0；双 env full **10084/10084 passed（15 slow、1 leaky）、23 skipped**、exit0/raw SHA `20378e04b5c585c954421780520a6a62ebbf1f6c7ba0336db40dd76541058e43` | 唯一最终 Claude Code **PASS**，P0/P1/P2=0、P3=3/raw SHA `64068902908f4c521e380dc7e5cf593134eaa99697de208344b65f75b3c3caa8` | 下一卡 COV2-20；家族 C/D、22 个最终行为源码 Session 与父 Source A 未闭合 |
| 2026-10-14 01:10 UTC | COV2-20 本机 A/B+单审源码交付复核 | signed+DCO main `aa0ea018ddd044f09d62a88428e812a359cef8ed` 已 Push、remote 同 SHA；七文件 manifest SHA `cb769d9a65e5b6db868d2e62fd6df1cc28ff33eb9e071ebab196827785a90a41` | 7/7 AC、3/3 VER 与冻结 v1 精确回归各1/1；formal 0/25、source-ready **8/25=32.00%** | fmt/strict Clippy/diff exit0；双 env full **10087/10087 passed（13 slow、4 leaky）、23 skipped**、exit0/raw SHA `6efa8a7d8461a9f4a5740661c5e99cb276804fe6d2b2a80d9264a9513da2124f` | 唯一最终 Claude Code **PASS**，P0/P1/P2=0、P3=4/raw SHA `6f3381d9061e72afebfe32f9714331ada567665d33eb0b045782fd06646667a5` | 下一卡 COV2-21；当前提交 Windows job `115625055658` success、完整 CI 待，家族 C/D、22 个最终行为源码 Session 与父 Source A 未闭合 |
| 2026-10-14 02:05 UTC | COV2-21 本机 A/B+单审源码交付复核 | signed+DCO main `6b24d797583afe1c0265f6e9fcbd364566671167` 已 Push、remote 同 SHA；六文件 manifest SHA `c3a53df17c2b3db500e6a4829a2c17ddc903fe5f5826e0de08f150c97df10533` | 7/7 AC、4/4 VER 与五项继承精确测试各1/1；formal 0/25、source-ready **9/25=36.00%** | fmt/strict Clippy/diff exit0；双 env full **10091/10091 passed（14 slow）、23 skipped**、exit0/raw SHA `eb4623ffedb916ba1dd86583bd1d65d2f4f5ff838eb7b6db33458e2028f9e8f7` | 唯一最终 Claude Code **PASS**，P0/P1/P2=0、P3=3/raw SHA `844fb61282a7e12ebb08221bd3cad687fed4e286bf7a41642256541cbfe7aedd` | 下一卡 COV2-23；新 run `38498688764` 在运行，旧 run 按用户要求取消；家族 C/D、22 个最终行为源码 Session 与父 Source A 未闭合 |
| 2026-10-14 03:51:32 UTC | COV2-23 本机 A/B+单审源码交付 | signed+DCO main `d977d13c5e23111b93d78477b9e1a70015a111b8`、remote 同 SHA；七文件 manifest SHA `39a3bb167df0279a0dbc67acb3c8575a791ec34c5851423c460b365575f78f63` | 8/8 AC、4/4 VER、四项继承与两项架构守卫各1/1；formal 0/25、source-ready **10/25=40.00%** | fmt/strict Clippy/diff exit0；修订后双 env full **10095/10095 passed（17 slow、1 flaky、2 leaky）、23 skipped**、exit0/raw SHA `5b20f0bfcf4ab2664b89dcb806c14ba812aa5597c7c7b48ca2cac621f5ca6599`；首轮 2 failed 保留不计绿 | 唯一 Claude Code **PASS**，P0/P1/P2=0、P3=3/raw SHA `024c7011547dbe0cbb466e2df8d1fe3c5ed555b9b56de6eb09377bfc64a83cd1` | 下一卡 COV2-04；旧 run `38498688764` 因新 commit 取消，未完成的 offline-core 不计通过；家族 C/D、Session 与 Source A 未闭合 |
| 2026-10-14 04:41:27 UTC | COV2-04 纯 codec 验收进行中 | 当前候选未提交；signed main 待 | 8/8 AC、4/4 VER、额外 producer-count 1/1；formal **0/25**、source-ready **10/25=40.00%**（签名 Push 前不预增） | ER-13 `none`；五项 focused 各1/1/raw index `292bef85b72f6fe8bd8a5a4513c749d34c3d3dc4275bce9e7802a3c75395f45f`；nightly fmt/strict Clippy/diff exit0，quality index `064fadb470b87350d6885ea6064ae1994c12bab17ee322352136f4c618847911`；发布 T-2 full 待 | 唯一 Claude Code 最终字节评审待 | 最终单审与 signed main 后进入 COV2-24；家族 C/D、Session 与 Source A 未闭合 |

| 2026-10-14 08:19:08 UTC | COV2-24 本机 A/B 与 T-5 最终源码候选 | 当前八文件候选尚未评审/签名；正式 0/25、source-ready 11/25=44.00% 不预增 | 5/5 AC、3/3 VER、七项 mandatory ER-04 exact 各1/1；focused index SHA `39306128fdc72bbb2ae9bca583f186f2d8bfda693b68fb1a38795b815c9e31d5` | fmt/strict Clippy/diff exit0，quality index SHA `d0c676eb165802886f6f700d869d813db610a0fc7d09bf5e9759c452b71e979b`；第三轮双 env full **10103/10103 passed（2 slow）、23 skipped**、exit0/raw SHA `2908fff258247c857ae857a4759fcde9b925c212cc5912fa373ece16b5373029`；首轮守卫失败、第二轮旧测试高负载计时失败均保留为 exit1 | 唯一 Claude Code 最终八文件评审待 | signed+DCO main/remote 后才把 source-ready 增至 12/25；COV2-22 与家族 C/D、Session 和 Source A 仍待 |
| 2026-10-14 08:51:00 UTC（签名后复核） | COV2-24 最终源码交接 | signed+DCO main `c56de683acf71d7327436eef7ef9185d97dcff27`/remote 同 SHA；八路径 0644 manifest `cd43b9af3980c8f3e90ce5751df7855ebaf1f604e2803130f0d32d9556b46e98` | 5/5 AC、3/3 VER、七项 mandatory ER-04 exact 各1/1；25卡口径 formal0/25、source-ready12/25=48.00%；G-09 采纳后26卡口径 formal0/26、source-ready12/26=46.15% | fmt/strict Clippy/diff exit0；focused index `39306128fdc72bbb2ae9bca583f186f2d8bfda693b68fb1a38795b815c9e31d5`、quality index `d0c676eb165802886f6f700d869d813db610a0fc7d09bf5e9759c452b71e979b`；T-5 第三轮 10103/10103 passed（2 slow）、23 skipped、exit0/raw `2908fff258247c857ae857a4759fcde9b925c212cc5912fa373ece16b5373029` | 唯一 Claude Code PASS，P0/P1/P2=0、P3=3/raw `a6d467bcdc33c42fff3c45f55681ebbaefa213d96cfd59a8b901c5d3dff942f7` | 下一卡 COV2-25 仅在本次 G-09 三文档最终字节单审及签名采纳后开工；安全 CI 当时未完成，其后旧 Actions 取消未通过；家族 C/D、Session、Source A 未闭合 |

**终止链末端 D 组记账协议预登记：** 如 COV2-REL-01 最后只是纯文档登记，推送后的 main CI 观察不能写入该提交自身。按模板 v2.14 将该 SHA、run ID、结论及 UTC 留到下一次 `docs/development/plan/plan-status.md` 例行同步；次日仍无同步时单独作状态维护提交。本例外不把未观察 D 当 PASS。

## 完成判据

- [ ] 拟 26 张卡逐一满足 G-01..11、实际 A/B/Claude PASS，23 implementation 与 2 audit 的 signed+DCO main 源码/文档 Push，唯一 release point 实际 C/D；每卡到 `done/complete`。父计划当前70卡分母始终独立（67为本次兼容前驱采纳前历史）。
- [ ] COV2-16/17 对 22 个继承 ignored OpenCode Session cases 在最终行为源码上实测 12/12+10/10，且新增 COV2-12/15/18 本机正负例均非 skip PASS；没有任何 OpenCode Session 模型/采集 E2E 进默认 CI。
- [ ] v1 原字节/digest/MAC、RG-04 fixture 与 frozen pending decoder 未变；v2 writer/reader/import/pending/replay/graph/doctor 与 privacy/failure 门全部可重复通过。
- [ ] 首次发布前在已 bump 的同一实现源码树完成 nightly fmt、strict Clippy、双 env Nextest 全绿；若之后仅上游纯 patch 顺延且逐路径 SHA/mode 等价，按 COV2-REL-01 规则复用同实现已绿 full/本机证据，并在当前发布 SHA 实证版本面、安全 CI、四平台及安装。触发 ER-13 的各子卡局部全量均已执行并修复本计划暴露的缺陷；ignored Session 不因全量 skip 代替本机门。
- [ ] 唯一 patch 版本、版本面 2/2、Cargo.lock 工具链、签名+DCO、main 安全 CI、四平台资产/签名 manifest、Cloudflare/站点/HTTP、Homebrew 与指定 curl 安装均真实同源通过；失败或取消不得记绿。
- [ ] `docs/development/plan/plan-status.md`、`docs/development/plan/plan-20260902.md` 的 `DEP-OG-13=source-ready→complete`、`DEFER-OG25-COVERAGE-V2` 交接与发布版本/提交/UTC 事实同步；OG-27 在 stable release 完成前不 Push，完成后仍独立受 DEP-OG-12 Source A/native `signature` 门控。
- [ ] 没有在仓库创建额外证据文件或文件夹；每轮测试、review、发布和站点摘要写在本计划及状态正文；如采用终止链末端 D 组后补协议，按预登记办法补齐。


**REL-OG-TEMP-049 当前窗口覆写（2026-10-14 14:30:58 UTC，同步USER-TEMP-REL-049）：** 临时release操作自身不新增任务卡；本次兼容前驱G-09另新增父计划三卡 `FIX-OG-NATIVE-IMPORT-01`、`OG-IDLE-ERROR-029-01`、`OG-UPGRADE-029-01`，COV2不增卡。0.30.49精确成员为最新上游cat-file已审变化＋COV2-25源码＋按`COV2-25→FIX-OG-NATIVE-IMPORT-01→OG-IDLE-ERROR-029-01→OG-UPGRADE-029-01→COV2-22`串行交付的三卡最终实现/测试/文档；反向定位`OG-UPGRADE-029-01→OG-IDLE-ERROR-029-01→FIX-OG-NATIVE-IMPORT-01→COV2-25`，已持久化/已发布部分只前滚，不改tag/asset。当前R拥有者为本对话root，唯一发布操作ID是USER-TEMP-REL-049，三卡`C/D coverage from`均明确指向该ID（其实际门由REL-OG-TEMP-049登记）；不是新增release Task，COV2-REL-01及DEP-OG-13不因此complete。最终full必须含latest5c cat-file及本三卡新源码；旧COV25 full不能替代。最终含源码变化的实现树完成适用focused、fmt/strict、单Claude及双env full之后，若bump仅既有版本faces/self-lock/纯release说明和记账且逐文件实现/测试/配置/行为文档SHA/mode同一，才按用户既授权例外复用该final full，不再重复；其它源码变化重跑。安全CI、8个releasejob全部success、四平台/manifest/CDN/Homebrew/网站/HTTP、指定curl本机安装和installed cat-file smoke、未完成v2入口不可达证明全部保留，复杂OpenCode Session/account/model仅macOS本机，绝不进入CI。三卡实际release前locally-accepted，不记formalcomplete；签后source允许按noWait继续COV2，不以成员后置C/D制造依赖循环。

**本次修订镜像（2026-10-14 14:30:58 UTC）：** 父拟UPGRADE拆idle单轴并另加location FIX，父70卡；COV26原AC/VER不变。本COV实际更新DEP/DAG/审计/窗口/源重基/六卡依赖与修订历史；新增三卡追溯/矩阵/里程碑/风险具体表在父plan-20260902，同一修订由本COV的DEP-COV2-029-BASE消费；原67卡及COV25旧签名/full历史不被冒称新源码PASS。

**历史G09 R3签前preimage边界（已由后续实际交接收据覆盖）：** signed+DCO merge `dc1aab0905c770b3badd80054734c81862cc6a0e`，parents `783041fe9e0db6daa360edc41222c0d3f48415ec`（COV25）/`5c0eb71c57f5cf73e8f711e459826758086a4c11`（upstream cat-file）；merge尚未Push。COV25 actual Claude R2 PASS raw `4c6487fcfa0eea755457819506984a0c79ec3fd47ff7e9f89973f7445b12fefe`，其冻结full10110/10110只证明旧树；不得把它记为含5c及三卡的最终49full。父source-ready分子和COV source-ready不预增，三卡所有新门UNRUN。本三文档修订须最终singleClaude、signed+DCO main/remote同一后采纳。

**DEP-OG-029-SITE已核开工事实（不代表文档交付）：** root复用干净Libra原生隔离仓 `/private/tmp/og20-cf.iE5Kb7/libra-backend`（.libra，无.git），fetch/ff后cf为 `597b86cf254c212cfc142ebc070d54b112d2af89`，status空；唯一页面 `apps/tanstack-app/content/docs/commands/agent.en.md` 开工SHA256 `871a541dddd965b38d4d7953c5bca63f57cd0e9963aaeeacdac167872849d0c3`/0644，由root后续唯一页面writer。原backend四dirty页面未写；新页面编辑/typecheck/build/Claude/signed源/部署HTTP均UNRUN，不能记交付。


### 2026-10-14 14:56:42 UTC — FIX-OG-NATIVE-IMPORT-01 本机源码开工/具名门

G09 三文档修订实际 Claude R4 PASS（0P0/0P1/0P2/3P3），raw `cf0da86d05ae1044a504f0d839d8fc9b85089ae88ca473a250e172c02b1abe5e`；三文件最终manifest `44548edb2893d22d452174a5fa4218933efdff92ec617fbaaf8dc9504f6a2b58`，独立signed+DCO main `4c628d4d3d5a3990eccbaee6c502bc64c73c52cd`/实际GitHub main同SHA，fmt/diff/strict exit0、docs-contract30/30。R1/R2/R3 FAIL历史和R4三个非阻断P3保留；这是规划采纳，不是新源码或Source A验收。

COV2-25 最终代码此前已独立 Claude R2 PASS（0P0/0P1/0P2/3P3），raw `4c6487fcfa0eea755457819506984a0c79ec3fd47ff7e9f89973f7445b12fefe`；signed+DCO `783041fe9e0db6daa360edc41222c0d3f48415ec` 通过signed merge `dc1aab0905c770b3badd80054734c81862cc6a0e` 随4c628d4实际Push。原八owned最终postimages/PGP header/DCO及18个exact各1/1、冻结2b实现树full10110/10110均已核；5c cat-file本机lib20/20与command33/33、组合strict exit0。COV2-25 现在source-qualified locally-accepted，COV当前源码13/26=50.00%、正式0/26；旧12/26仅为Push前历史。上述冻结full不含5c及三张ordinary29新实现，实际49最终full仍待，不据此称家族complete或Source A。

本FIX从signed4c628d4开工，唯一production修改collect_facts，独立收集directory/cwd/location.directory字符串；既有canonical/same-storage门未改。源码postSHA `7dad06a4d5829abdbd172db9e53db87040078ff1f9554719f6e2a2e0dcb117f5`、mode0644；两个具名新门各1/1，分别raw `e46e7b0547ff860cadc369975d9410ce3cd91c2581a66d66fd8322b33d357c2e` / `bf3eac3e3e6ee5817872339e9db9d38086c4dcb234a55f732e8b1dd02b060726`。完整agent_import模块38/38/raw `e7afa18878f0911bc3a96838342978d0b1ac67f43c325ecb56ade4abea91e4f1`、docs-contract30/30/raw `c50b16ae555c172a0f8c96669c186ec6dea3ce85928ef52397c40b9f2541d150`。VER2包含三种目录冲突、缺失、五类非string、classic有效string控制、跨repo与词法/Unix symlink别名。两unit不直接覆盖descriptor helper；真实existing Session非零内容仅在后续OG-UPGRADE-029-01 VER2实际验收，当前UNRUN，原空checkpoint不可据此称已补全。

精确旧JSON/MANIFEST、catalog、settings/helper等开工/收口SHA及mode比较未变化；不改native来源/issuer/CI/model权限。隔离网站cf597b86唯一页面追加与EN同段，typecheck/build实际exit0，签名源交接及49部署HTTP待。最终本FIX fmt/strict/Claude/source主与网站Push待；父正式55/70=78.57%、源码58/70=82.86%暂不预增。下一源码卡OG-IDLE-ERROR-029-01，真实内容与普通29基线后置；新实现树full/049 C-D所有门保留。


### 2026-10-14 15:47:50 UTC — FIX 已交接；IDLE 本机机械门与后续独立仓验收

FIX-OG-NATIVE-IMPORT-01 唯一 Claude Code 实际 PASS（0P0/0P1/0P2/5P3），raw SHA256 `ac889106b8764b2d4ecfba26bfef88b650200c692d4cb6c199bfa762eaf210a8`。root交接收据 `/private/tmp/libra-fix-og-native-import-review-r1/signed-local-source-receipts.json` 的SHA256为 `86249934f689efeff06f3cefe99fbd99292a4154517dd550dde9b36e8a898f26`：主仓signed+DCO `6524153d97ab2f5c02f7875abd5f3ed3d343cea8`、网站cf signed+DCO `ddfee87c66d35cf586ca4534c0d427a7254085d0` 均Push exit0、remote同SHA、reviewed owned blobs一致；两exact各1/1、import模块38/38、docs30/30、fmt/diff/strict与网站typecheck/build exit0。网站已签页面postimage SHA256 `ca7673546b434551f25ffbec2d07653a10f7d000574a023e9508197466698042`、mode0644；部署HTTP和实际049 C/D仍待。父formal55/70=78.57%、source-ready59/70=84.29%，COV formal0/26、source-ready13/26=50.00%；FIX源可由IDLE消费，未称release complete。

FIX五个P3保留为正式审查历史：非string directory配有效cwd的组合未单独pin、descriptor helper独有scope gate未由两unit直接覆盖、cross-repo错误码路径差异、旧计划Lifecycle标签/merge时态卫生及段落空行排版。本次例行同步仅清理旧生命周期重复标签并标注过时签前状态；不冒称其它P3已修复，descriptor/真实内容门仍由后续独立仓BASE验收且当前UNRUN。

OG-IDLE-ERROR-029-01 从上述已交接FIX源码开工。唯一production路径 `src/internal/ai/observed_agents/opencode_reasoning.rs` 当前SHA256 `7da0e57485c5dc02179240b928a0f67f9f134efe7c7b76029c21ae80bfea4cc5`、0644；三新exact分别实际1/1（0failed/0ignored），VER1/2/3 raw SHA256为 `b91253f55c5e45bd7b902100098a9832d78f18b4e427a4d753d9a205ce5f1d0d` / `deea4de5ff5f21f17ad14b2fffdc506b5836a13dc9cbb046d47d3c50ffa99c29` / `ef3b05a90632170268e55f7a5c532622f6d9cef3e6cbf5221fac75aa733e1bda`；完整opencode_reasoning模块实际31/31、0failed/0ignored，raw SHA256 `e084224c7a96df8437b80b2e93998abd49218bd461b5268d18ad933d69ea2133`。这些只证明idle.error typed ordinary投影的离线机制；真实失败事件未观察，不能称Source A或native三字段来源通过。EN/zh/tracing/COMPAT与三计划为本卡既定同卡文档，根仓最终I为8路径、production仍1路径；网站1页由root唯一owner串行承接FIX cf。网站typecheck/build root实际exit0，对应raw SHA256 `38e40aea10a235b64913aaad108f33fce8ae6d81da82886313a8fd72d4ee0f27` / `a3901b05e2fd699f3bc116161d95b1fe986a2edb7792c3126e75e43aafbfd1d8`；候选页SHA256 `287edb4afb51c5849a0671e34f8c8c7fdc0d898b812413e8c938999d6c675306`/0644，signed源交接和部署HTTP待。

**IDLE最终收据待root据实补齐：** 当前已应用五source/docs路径的fmt/diff/strict实际exit0（strict raw SHA256 `27ed297e64983359a3e7128122f595b7a5eec8625ba986d58d73ec37eba30aed`）；module31/31、docs30/30、architecture57/57、hook11/11均非零通过，后三raw SHA256依次为 `b9c4ccdad36fd39a7d73b855bf05fa0cda54214d7113184c105cb3ced95c8376` / `d202a33dc9321a243ae80fbf2a20ba1f13cdd60178983d1ee6d85e4285908f3f` / `194ded6e69b84e1dbcd9e43282f7f496aff3ceb176f8680bfd9c49680e2c09f6`。本次例行三计划正式应用后最后diff/docs同步=实际exit0：最终diff raw SHA256 e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855，最终docs-contract30/30 raw SHA256 ab172a47fe0401c7f348628472d83eed8b4a91f02794fd764996968550e97e0c，不把追加前结果自动记为新增计划字节验收；唯一Claude=IDLE-029-CLAUDE-PENDING；主仓signed+DCO/remote=IDLE-029-SOURCE-PENDING；网站signed+DCO/remote=IDLE-029-SITE-PENDING。这些文字是明确pending字段，不是SHA/成功占位证据。上述最终门关闭且Push后才登记IDLE source-qualified并把父源码分子59→60；当前Lifecycle=in-progress/Acceptance空、formal55不变。临时049最终含latest cat-file/FIX/IDLE/BASE新树full和全部C/D继续待，不复用COV25旧冻结full冒充。

**后续BASE实际场地：** OG-UPGRADE-029-01 VER2在本轮新建的干净隔离Libra原生仓（.libra、无.git）执行；root记录该独立repo的canonical路径/storage、selected官方29二进制SHA、同29 server/PID、ordinary trust与Required Seatbelt。Session必须绑定该独立repo；仅由root按既有授权显式选择/建立真实Session，验收recipe不自动新增模型调用。export→capture/import→checkpoint/export非零并逐项对拍prompt/model/token及既有files兼容门；SessionEnd显式恢复若使用仍逐实记。既往UNRUN/空内容历史保留，不能借新场地抹去或算通过；当前本新lane也UNRUN。不改变生产写集、AC/VER计数、native登记或Source A判据，不加入CI。

**当前临时本机安装收据：** `/private/tmp/libra-temp-install-20261014-r1/temporary-install-receipt.json` SHA256 `1bca5c4f834ffb1e632ada8ef6bed0b207bd2201683e808266cab33ae6f91e1d`。两个目标 `/Users/eli/.libra/bin/libra` 与 `/Volumes/Sea/libra/libra` 为同一193866248bytes/0755/SHA256 `1747731f8df72dcc6eb52e541056640440e1cdbf89eb762516d47f1edf830e00` 的本机debug构建，版本面仍libra0.30.48，基于signedFIX及当时尚未提交的IDLE代码；不是官方新release或049安装通过。原正式binary/marker已备份，临时official marker已移除。正式049实际验收后按用户指定curl安装并验证release digest/size/marker，再原子替换repo内第二目标，使两路径同release SHA/版本；若之后用户改动目标，先保留该改动再替换。本临时receipt不增加任务卡、版本提交或C/D PASS。


### 2026-10-14 16:15:36 UTC — 新增storage-only ordinary partial live修复；本机临时构建先行

IDLE实际唯一Claude PASS（0P0/P1/P2/5P3），raw SHA256 `829c06700313d1740ccad0c5ad1e3d9acf9816e2df8f2c0d2708a85dedebf1ee`；signed+DCO main `95ab8f35b3af7e8a5736c955212415ca93eebeb3` / cf `bc58acc2dbe057dc039fedfc3f3d432b9b4dd332` 两Push exit0、GitHub remote各同SHA，PGP header/DCO预检及trailer门实际0。收据为 `/private/tmp/libra-opencode29-idle-error-review-r1/signed-local-source-receipts.json`。已知源码60/70、formal55/70为本G09采纳前事实，IDLE5P3保留；当前新live源码/本机成功/full/正式C-D均UNRUN。旧Actions取消请求列表为空，不推断任何旧run已通过。

本G09只新增 `FIX-OG-PARTIAL-LIVE-01` 一条live direct checkpoint轴，完整sealed producer→snapshot current sourceHMAC→专用payload factory→emptyclaim直接store为五生产文件/四落点，8AC/8VER/M；不写agent_import、pending[_payload]、coverage_gate、native registry/issuer/CI。原始树任何originProof（含错型/未知嵌套）或typedcandidate禁止fallback；generic Unproven不是permit。默认ordinary/capture入口不放开，普通来源保持Partial/Unproven，nativeartifact/identity/ACK/Complete coverage0。只有已确认durable direct commit可给明确partial ordinary lifecycle收据；失败保持dirty/content-free pending/no成功ACK，不能重命名为Complete或seal partial replay。

**实施/DEP/G10新串行边：** `COV2-25→FIX-OG-NATIVE-IMPORT-01→OG-IDLE-ERROR-029-01→FIX-OG-PARTIAL-LIVE-01→OG-UPGRADE-029-01→COV2-22→COV2-08`。新卡先消费actualIDLE postimages，后BASE及所有相交COV Icore按 `DEP-OG-029-COV-REBASING` / `DEP-COV2-029-BASE` 从该五源保留修复、逐路径hash/mode重新交接。新卡不需要BASE或native3作为前驱。网站同owner `FIX→IDLE→FIX-OG-PARTIAL-LIVE-01→BASE→COV2-09→11`；root从actual cfbc58单页preimage串行写，不覆盖原dirty checkout。

> **历史规范：** 下段保留原结论；其historical import作为BASE/049前驱的部分，由2026-10-14 17:58:11 UTC G09显式替代，原020失败未改为PASS。

**REL-OG-TEMP-049新增精确成员与反序：** USER-TEMP-REL-049仍是唯一临时正式release操作，不新增release Task；当前精确成员含最新upstream cat-file、COV25及FIX/IDLE/本live卡/BASE，按上述串行链交接。失败定位逆序 `BASE→FIX-OG-PARTIAL-LIVE-01→IDLE→FIX→COV25`，已持久化/发布只前滚，新卡C/D明确继承USER-TEMP-REL-049。正式0.30.49最终实现树full必须含本live修复及实际后来源变化；用户既有纯版本同一实现复用例外保留。BASE实际GLM import020已失败，不能手工剥原provider形状造clean输入充PASS；新live成功不代替该门，普通import修复须另行精确规划/Claude采纳，故BASE/049 import门仍未闭合。原native OG27/28、SourceA与COV2家族C/D不由此次解除。

**用户当前本机临时交付授权：** 修复后先构建本机debug版并在独立owned仓复测，不等待BASE或正式049 release。本卡必要focused、fmt/strict、实际源码sole-Claude及源身份核对须真通过；根代码只由root应用/签推，私有prototype不等于实施或PASS。临时安装两已有目标及旧正式marker/backup按前条实际receipt保留，再记录新构建源码manifest/binSHA/size/version，不bump、不冒称官方release/full/C-D，正式049后按指定curl安装同源正式版。


**本G09 R2实际审查与规范修订：** 核心Claude实际PASS（0P0/P1、6P2/3P3），raw `69f387501d3a170f1382f31e85bd13783074177345f392b92c75ddfbb4ee0e0a`；仅设计/core，不是源码/模型/full/三镜像PASS。三镜像R1实际FAIL（0P0/P1、1P2/5P3），raw `132725fee31f79e4a8e5a93cc70d7fce6bf8a668f14eaad6e0c3aea0cd664a0c`；本R2补父修订历史row，core六P2具体wording与OG26窄例外/实施UNRUN。不同idle允许bounded whole-source重复；HMAC plainString不是snapshot自行验证scope的凭证，carrier绑定repo/libraSID/providerSID/kind后与actualrequest核对；info.id精确映射。已存在auth/scope/length及closed native/pending门作为ER04继承安全契约，不添第九AC。默认v2import Partial zeroBytes仍拒，existingexitvariants/ENTRY_RECOUNT不扩。本R2最终三镜像单Claude/signed采纳待；源码及所有运行新门UNRUN，不伪造零P2或actualimport通过。


### 2026-10-14 17:44:21 UTC — FIX-OG-PARTIAL-LIVE-01 实际 A/B 与用户临时安装

G09已由signed+DCO main `dea12cf6d1f41569e51851b8e8537fe0765be151`实际Push采纳。生产只改既定五源；共享live私有接口使用provider-neutral `VerifiedOrdinaryPartialSnapshot` / `VerifiedOrdinaryPartialBinding`及对应consumer，唯一mint仍在strict OpenCode producer，不扩generic Unproven/native/pending/V2import权限。ARCH精确增加同一match不同arm的两个sealed ExportStage handoff，单settle/单checkpoint消费与alias拒绝仍闭合；CheckpointStageExit仍只由原class helper构造。未改SQL、CI、CLI、native issuer、pending或agent_import。公开wire enum/serde五定义及旧pending decoder逐字节不变；ER04 validator/readback机械清单不替运行。

实际六VER new exact各1/1，最终live模块15/15（包含terminal成功/durable replay/过期content-free pending）、ARCH58/58、docs31/31及ER04旧selectors14项各1/1；reasoning21通过/1 ignored、OpenCode32、snapshot26、checkpoint34、pending31通过。`cargo +nightly fmt --all --check`、`libra diff --check`与`cargo clippy --all-targets --all-features -- -D warnings`实际exit0，strict3m49s。网站同owner单页typecheck/build0。首compile两错误、早期中断focused/strict、ARCH undeclared quote与55/3、57/1失败、store fixture路径与CAS竞争ref误断言失败均保留；只最终实际运行记通过，不将取消或synthetic称成功。单元测试是纯parser/key/store/SQL lease/AST seam，不调用OpenCode Session/exporter/CLI/账号/模型，不新增CI采集。

macOS真实一次 `volcengine-coding-plan/glm-5.3-flash`，隔离repo `/Volumes/Sea/libra-partial-live-ver8-ar5mj9gh`；官方29二进制SHA `7f46ba1dad86b730903d9adf54015bb4e706fc1ab6a247affc5a635cd8a663f3`，ownedstdio同PID/endpoint、cwd和PWD先核。provider `ses_ec483a846ffeAQghpE0mS079I7`；自然自动生成checkpoint（初观`3e6ef7e6-f77b-5f04-b552-5b3f49fe9f0f`及后续idle `e08d6f2c-c04a-5b5c-9bc4-b79bc567f386`），后者普通export **1369 bytes**、SHA `16d44f3f9e527a49b27bc509d03c5239691ce4ea79fdd260934b77214c2a1191`，raw=false/truncated=false、partial/source_authorization_mismatch、trusted_export/current HMAC。真实unsupported reasoning state及关联text/time已剔除，普通user/assistant与message/part坐标读回对拍；artifact/identity/Complete coverage/pending/inflight均0，job clean/processed与实际generation一致。无manual hook/import/repair、仅一次模型。旧官方48 SHA `26878d8b52ad8ba87789a07e8823cbcc2c4fa554f19fafb8197e0a431e13686e`正常reader实际读回相同1369bytes/SHA。Source A仍UNPROVEN。

原private verify最后一步误把旧show `data.checkpoint.checkpoint_id`当平铺字段，实际CLI与两份payload均已成功，原FAIL/VERIFY.once/STATE不抹去。只读R2复核既有证据与exact-owned DB（不再CLI/模型/hook）实际PASS；receipt `/private/tmp/libra-partial-live-ver8-verification-resume-r2-vhzhorvj/VER8-RECEIPT-r2.json` SHA `9575ffd297f68b60e25a1500dbc133c6d06690dd80dd6034135713689029de60`。这不将失败脚本自身改称通过。

用户要求的新临时debug artifact已实际安装到 `/Users/eli/.libra/bin/libra` 与 `/Volumes/Sea/libra/libra`：均193956968bytes/0755/SHA `a70adfbbf474a741f16497384b7c833214979507dfba4367f37bcd2e566dfd1f`、`libra 0.30.48`；不是正式049或版本bump。源manifest SHA `052dda0aeedccec8def356cff792638d556f1a021c3aea52dbab6463f07c57c6`、codesign验证0，旧临时和原正式备份保留、无official marker。安装receipt `/private/tmp/libra-partial-live-temp-install-yxw0xwia/temporary-install-receipt.json`。正式049通过后仍按用户指定HTTPS curl安装，验证正式manifest/digest/marker后替换这两个路径。未改用户Demo数据。

**历史live签前收尾（对应2026-10-14 17:44:21 UTC收据，原文保留，已由当前实际signed交接及G09范围采纳覆盖）：**

本卡保持in-progress/Acceptance空（仅A/B locally accepted，当前三计划单审与signed root/site Push待）。实际签推后父source61/71，formal55/71；不将temp/fixture/本机A/B充C/D。BASE真实import仍020拒绝、相关必要后续尚未另行采纳，不因本live成功或旧reader读回放行BASE/049 import门；最终049新实现树双envfull、四平台/安全CI/稳定release/Homebrew/CDN/指定curl及native Source A都未完成。下一步先完成本卡source交接与三计划收据，继续普通29基线及import边界的必要后续；不重触用户Demo。


### 2026-10-14 17:58:11 UTC — G09：BASE自动捕获基线与DEFER-OG29-PARTIAL-IMPORT范围修订

**依据与效力：** 按用户既有“同类复杂问题延后、基础采集优先”和无需再问默认推荐授权，本次提出明确范围修订；只在最终三文档实际sole-Claude PASS、signed+DCO main采纳后生效。它不是认定旧门已通过，也不使BASE/049自动通过。本修订前日期记录、raw失败/UNRUN、旧完整门原文作为历史保留；旧关于“BASE/049 import门仍未闭合”的规范在本采纳后由本段前瞻性替代，而非修改其当时结论。

**范围与计数：** 原OG-UPGRADE-029-01编号保留namespace＋真实自然自动capture/checkpoint/普通export主轴，原8AC/2VER及1 landing/1生产文件、原I/R与逐文件文档/网站义务不变；仅AC5与VER2移除historical import正例。AC1/2 pin/namespace、AC3/4实际同29 binary/runtime、AC6/7/8真实prompt/model/token投影仍逐项实际验收，files原兼容控制保留。未完成项保持unchecked/UNRUN，不因计数8/8文字把8项写成已通过。DEFER-OG29-PARTIAL-IMPORT登记为计划外未实施两轴，不新增Task或完成分子；父71卡/source61/71/formal55/71，COV26卡/source13/26/formal0/26保持，BASE自身待门。

**延期与负例：** 原GLM unsupported-state历史import020仍未修复，默认Unproven拒绝；live自然自动partial checkpoint非零成功仅证明live，不是import PASS，也不是native/Complete claim/Source A。不得修改原provider字段、清洗源、补假证据或用synthetic对象伪造正例；无state ordinary若另有真实通过只按实际源单独记。两轴helper认证partial胶囊与direct-only导入持久化由计划外遗留承接，未来另做具体设计/写集/单Claude/原样实际正例，当前不实现，不授权新增模型调用。

**DAG / DEP / G10：** 严格顺序仍为COV2-25→FIX-OG-NATIVE-IMPORT-01→OG-IDLE-ERROR-029-01→FIX-OG-PARTIAL-LIVE-01→OG-UPGRADE-029-01→COV2-22→COV2-08；无新边或卡。DEP-OG-029-PLAN及SITE、DEP-OG-029-COV-REBASING与incoming DEP-COV2-029-BASE只按修订后BASE最终A/B/fmt/strict/单Claude/signed源码/hash/mode交接，不把计划外导入修复作为incoming前驱。相交COV2-05/06/11/15/18及Icore从最终源精确重基，原cursor/candidate/隐私/字节对齐oracle不减；若执行这些卡需要新consumer，应在其各自范围实现或明确复议，不能引用本延期跳验。root共享单写者、干净cf owner及FIX→IDLE→live→BASE→COV2-09→11站点顺序不变。

**REL-OG-TEMP-049 / USER-TEMP-REL-049：** 唯一临时patch0.30.49操作，成员保持latest upstream cat-file、COV2-25和FIX/IDLE/live/BASE最终实现；显式排除DEFER-OG29-PARTIAL-IMPORT能力，发布说明和网站必须披露unsupported-state历史import仍拒绝/未修复。此排除只改变发布范围，不证明release可执行：最终实现树focused/fmt/strict/sole-Claude及双env full、实际最新SHA安全CI、8releasejob/四平台签名manifest与资产、CDN/Homebrew、网站typecheck/build/签源/部署HTTP、指定HTTPS curl安装及installed cat-file smoke仍全部必需。纯版本同一实现复用例外不变；未完成v2入口不可达证明、COV2家族C/D和Source A硬门不变，不以本临时发布complete COV family/DEP-OG-13。

**安全与证据边界：** 没有生产代码/默认ordinary或native权限/旧V2 Partial零bytes/pending replay变化；G09规范修订自身仅三计划，同次收口另含原FIX范围内的Unix测试helper cfg前滚，后续BASE仍只写其原exact I，不能顺带实现延期6文件。真实OpenCode Session/account/model/hook/CLI只macOS本机、绝不加入CI，不新增自动模型重试。既有复杂失败不被延期为PASS；最终全量若暴露实际回归仍须forward-fix。未实施能力在最终说明中明确列为限制，已发布只新patch前滚。

**历史草案当前门（原草案时点，已由84bb实际采纳收据覆盖）：** 本段草案未做正式Claude、没有VCS/模型/cargo/真实验收；root采纳时须补实际最终三文件manifest、sole-Claude raw/结论、signed+DCO main与remote同SHA。该规划采纳不改变BASE unchecked及049 C/D状态。下一实施仍为BASE current namespace和自然自动capture内容门，随后COV2-22；延期不占当前写slot。

**计划外遗留镜像：** DEFER-OG29-PARTIAL-IMPORT由父plan-20260902登记，未来独立承接未指定；本COV不接管两轴、不新增任务，也不变更本COV任一AC/VER/原import oracle。


### 2026-10-14 18:03:50 UTC — 已签 live 源码交接及 Unix 测试 fixture 前滚

FIX-OG-PARTIAL-LIVE-01 实际 signed+DCO main `d8422534b07724349591d53aa445ce50c38453f5`（精确14路径）与网站 cf `a80d3ff9d2d194b739da6148dde3297d325f1a7e`（精确1页）均已 Push exit0、remote同SHA、PGP header/DCO/逐blob同一性核验通过。三计划例行收据实际sole-Claude PASS，raw SHA `e30b717ec6754916b7fd87a171346357227abfa526081d797e7f54ec6cd35557`，0P0/P1/P2、5P3保留。本卡source-qualified，父source61/71=85.92%、formal55/71=77.46%；COV source13/26=50.00%、formal0/26。此前“签推待”是当时事实，不回改旧收据。

当前 d842 的 Windows 必要安全CI job `116010393408` / run `38609801760` 实际失败：四个仅Unix测试消费的常量、`VerifiedPartialStoreFixture`及其impl、`verified_partial_fixture_snapshot`未同步Unix cfg，`-D dead_code`导致lib-test编译失败。原始日志SHA `d22b0a896e123b351caf01e0437290a88a1c09aba618539491b1bf513533d481`；属于确定性代码问题，不作为network重试。只在原FIX写集 `capture/live_checkpoint.rs` 和 `capture/snapshot.rs` 的测试helper补7处cfg属性，无生产行为、旧测试删除/ignored、CI或新卡/写集扩张。当前 macOS fmt/strict/focused、单Claude及本次signed Push仍待；Windows只有后续实际新SHA成功才可关闭，不能以本机编译代替。

G09规范修订自身仅三计划；同次收口另包含上述原FIX范围内测试cfg修正。BASE全部8AC/2VER仍unchecked，本次不预增完成。已有真实自然自动采集的prompt/model/token/files只读核对输入可作为后续BASE候选：prompt1、typed extraction.model同原native assistant、五token input7681/output32/reasoning14/cache0/0与持久化7727总数一致；files仅零工具/空路径对照，不是修改文件正例。顶层hook model与native model不同，不能拿它冒称实际模型；只用已核typed extraction.model。该核查未新增CLI/服务/模型，未重跑live或import，不独自构成BASE PASS。GLM历史import020依G09明确延期，Source A仍UNPROVEN；正式049最终full/安全CI/四平台/发布/安装及网站门全部保留。


### 2026-10-14 18:10:19 UTC — G09 R1 P2 规范澄清（最终字节复审待）

R1实际sole-Claude为PASS（0P0/P1、4P2/4P3），raw SHA `6921a0cb37742edc903c5f106104f80bdf507a1440e7270031936ec864aed479`；保留该结论与全部意见，不把其写成零P2。最终复审另核完整长行覆盖，不用“读到EOF”替代完整字节覆盖。

1. **source-qualified与CI分层：** source-qualified是本计划内部的最终A/B、适用focused/fmt/strict、唯一Claude、signed+DCO source Push及hash/mode交接；Windows/其它远端CI属于ER-04 D组，按用户no-wait授权不阻下一张源码卡开工，但失败必须保留且阻正式release/C-D。因此d842的内部source61/71成立，Windows失败不记通过，formal仍55/71；本次前滚新SHA必须实际Windows绿后才可关闭该远端门。DEP-OG-029-PLAN/DEP-COV2-029-BASE按内部source交接，不声称D组已通过，不把source-ready充formal complete。

2. **真实证据复用范围：** 1369-byte VER8摘要本身只支撑对应的AC5/6/7/8普通内容、prompt/model/token及files空对照，不能单独支撑AC1–4。AC1/2必须由BASE新增常量/namespace及VER1实际核验；AC3/4须另核原官方29实物version/SHA/size、同一server PID/endpoint/runtime与对应exporter的独立precheck原收据。BASE最终Libra源码到原实际capture运行时的逐路径同一性、仅测试cfg/无运行consumer常量变化及新候选普通读回必须由root机械核实并经最终Claude接受；任何未证实项维持UNRUN，不声称不同binary SHA相同。不得用新无state源替代原GLM失败。

3. **强制范围披露：** BASE的ER-06 Deliverables必须同步EN、zh、tracing、COMPAT与唯一cf agent页，明确“含不支持推理状态的历史OpenCode Session导入仍以LBR-AGENT-020拒绝，修复延期”，并保留自动捕获可保存安全ordinary Partial、native Source A未证明的边界。新增既有docs-contract函数`opencode29_ordinary_extension_is_distinct_from_frozen_native_source`须逐四root文档断言该披露；网站逐页验收由root核与EN新增段逐字相同及实际typecheck/build/签源/部署HTTP，不把外部网站文件引入CI。USER-TEMP-REL-049的实际CHANGELOG0.30.49说明与最终网站须实核同一限制；不新增本卡特有AC/VER或版本写权。

4. **旧规范显式历史化：** 原“REL-OG-TEMP-049新增精确成员与反序”每份现存未引用正文前插入本G09替代标记，保留原文字与旧FAIL结论。日期快照仍保留；本次仅前瞻性调整发布范围，不修改曾经失败的import门结果。

**历史G09签前closing（原文保留，末句待审/待签状态已由84bb实际采纳收据覆盖）：**

AC5的对应性明确只指validator安全普通投影；snapshot保持Partial/Unproven，unsupported state及关联reasoning text/time被擦除，不承诺原始完整性、typed/native归档或Complete coverage。文件维度仍仅empty control，不记修改文件正例。最终G09仍须实际单Claude复审PASS及signed采纳，BASE仍全unchecked，不预授049或COV完成。


### 2026-10-14 18:10:59 UTC — 测试cfg写法与既有AST守卫对齐

窄前滚首轮fmt/diff实际0、live15/15、snapshot26/26通过；ARCH实际57/58、1失败，raw SHA `50a6986e1eaec4ca69423502645a6304c915d80f31aedde7b21b1d815abc6a03`，保持FAIL。原因是既有provider-neutral AST守卫只识别独立`cfg(test)`，把新`cfg(all(test, unix))` helper误算为生产AgentKind入口。改为保留原独立`#[cfg(test)]`并另加`#[cfg(unix)]`，Rust含义同为test AND unix；四常量/fixture struct/impl仍各Unix限定，共七个新增Unix属性。不修改或放宽AST守卫/旧计数、不新增生产provider分支。最终ARCH/docs/fmt/strict及完整长行单Claude复审仍待，Windows须新SHA实际验证。


### 2026-10-14 18:27:15 UTC — G09实际采纳、测试fixture前滚交接与BASE开工（例行事实同步）

**实际signed交接：** root已核signed+DCO main `84bbde7bd6a5f12dee2477755d0a955798518413`，parent为`d8422534b07724349591d53aa445ce50c38453f5`；实际main Push exit0，remote精确同SHA。五路径PGP header/DCO及blob/mode等于最终sole-Claude R2 manifest：R2 raw SHA `4ab4911f9253eb659bb03cf6b49fb4479e92a74145ea1ee8f982e1a12df05190`，actual PASS/0P0/0P1/0newP2；R1 PASS的四P2已由R2逐项关闭，六个P3经父级裁决接受，不伪称历次零P2。此门只覆盖84bb已签的原五路径，不借其PASS覆盖后续BASE实现或本例行记账新字节。实际交接收据时间2026-10-14 18:23:51 UTC；网站source前驱保持signed cf `a80d3ff9d2d194b739da6148dde3297d325f1a7e`，BASE网站新段、发布部署/HTTP仍须自身实际门。

**前滚与QA：** 旧d842 Windows job `116010393408`真实FAIL（七项仅Unix测试fixture在Windows引出dead_code）；root只给四常量、fixture struct/impl、snapshot test helper共七项增独立Unix cfg，保留原cfg(test)，未改生产运行函数、CI或原测试计数。首轮ARCH真实57/58 FAIL/raw `50a6986e1eaec4ca69423502645a6304c915d80f31aedde7b21b1d815abc6a03`保留；最终QA七门实际exit0：fmt、diff、live15/15/raw `1e1be5ea5b783a1f18807d95a7675170fd804f672a9cf9afe57706f7376823de`，snapshot26/26/raw `731a884a4651dc09e68170e1b3c4195d19fbbc4330ac77c3ee52befc0ed9c9cc`，ARCH58/58/raw `8025733a319e6249dbdb444be67eb0d09686e8e109a0cc5c87b59e712c914246`，docs31/31/raw `9066fed57587926c671d3af7e626a40835d53e6c2e4f7fead428233e4ce87d23`，strict all-target/all-feature -Dwarnings 18:12:46→18:22:01 UTC实际0/raw `3a9e6d6a12396ff911c7b47353de076c7d06ce158532029d0b33dc6ff7eae7a4`。这是84bb本机适用门，不是新SHA Windows或BASE/049 full验收。

**CI仅开工观察：** root在BASE开始时只读看到84bb CodeQL `38614865589`为running、base `38614865565`为queued；Quality `38614866097`和cancel workflow `38614865604`实际success。它们不证明base/Windows/完整安全CI通过，后续须按精确SHA和job真实结论另记。本例行同步未重新查询CI。旧d842 base `38609801760`的取消请求实际成功，terminal结论当时未查；不能写已cancelled或PASS。所有failed/cancelled/queued/running和未运行门不得充PASS，用户新commit取消旧Actions策略保留。

**当前范围与下一卡：** G09已前瞻生效，DEFER-OG29-PARTIAL-IMPORT仍计划外deferred/未修复；GLM unsupported-state历史import020仍FAIL/默认拒绝，不能由live成功、无state源或洗源替代。BASE现in-progress/Acceptance空，只实现official29 namespace＋自然自动capture/checkpoint/ordinary export的安全prompt/model/token与files空控制；全部8AC/2VER仍unchecked，AC1/2/VER1、本机VER2逐项证据同一性与内容对拍、同卡四root文档/网站披露及最终QA/Claude/sourcepush另录。父source61/71、formal55/71，COVsource13/26、formal0/26不增；BASE签推与source-qualified门闭合后才转COV2-22。本次不新增任务、不改其它卡的生命周期/AC/VER，不记source62/71。

**历史收据与发布门：** 页首较早“草案待采纳”、live签前closing和12/26段仅为其原时点历史，原正文/失败不回写。84bb当前13/26和本次实际采纳为现行事实；旧full只覆盖旧冻结树。最终049仍要求最终实现树适用focused/fmt/strict/sole-Claude/双envfull、实际安全CI/8releasejob/四平台签名manifest/CDN/Homebrew、BASE网站typecheck/build/签源/部署HTTP、指定HTTPS curl安装及installed cat-file smoke；Source A/native GO、V2 Partial零bytes、pending、未完v2入口不可达与COV家族C/D门均不变。复杂OpenCode Session/account/model/CLI只macOS本机，不进入CI；Linux功能仍由用户后验。

**同文件文档guard必要对齐（本卡范围，尚非实现/通过）：** `tests/compat/agent_docs_contract.rs`仍在BASE原I，允许新增`opencode29_ordinary_extension_is_distinct_from_frozen_native_source`，并仅在既有`opencode_verified_partial_live_capture_keeps_native_and_replay_closed`（84bb现1334行起）中把现1389行`BASE's real import gate`和1459行`BASE 真实 import 门`这两项过时文档披露term，改为与G09一致的“历史unsupported-state import仍LBR-AGENT-020拒绝/修复延期”准确断言。最终词面/两行postimage须由文档owner与真实四root文档对齐，root精确diff和最终sole-Claude裁决后才应用旧fn；不为守旧字符串把内部BASE实施门写进产品文档。该fn其它privacy/native/Partial/V2zero等全部原断言保持，其余30个既有fn逐字保持；旧31fn数量保持、新增BASE后完整target预期32（不是已PASS，须实际运行）。先前私有“整旧31fn字节prefix冻结”只是未采纳草案，本次明确以这两term必要修正替代该过宽主张，不涉及原ER04冻结JSON/MANIFEST/native fixture/decoder字节。此为ER06同一文档轴，不新增文件/生产/卡/AC/VER/landing或放宽安全守卫。


### 2026-10-14 19:08:52 UTC — 消费BASE例行实测，COV交接与分子仍待

父`OG-UPGRADE-029-01`8AC/2VER已有新owned自然自动capture/checkpoint/普通export1345B、prompt1、GLM extraction model（hook model仍unknown）、五token7678/29/35/0/0(total7742)及空files控制对拍。1次模型、0次manual hook/import/repair，raw=false/truncated=false，旧48读回同bytes。official29 SHA/version同一；interactive PID50453已退出，collector为同sealed verified binary的独立Required-Seatbelt租约且PID未存档，不声称同PID/endpoint。645实际构建输入、12旧protected和双MANIFEST frozen prefix均同一。VER2收据SHA256 `c02902d46324ec28ec2968e8d13132fbc58479b54baccaf42dd7acbdd273caae`、独立投影SHA256 `0a1de7f096516737d9ecc2545592ff516baf73716714a2aab5df2a977919d924`，完整边界和QA原始收据见父卡同日记录。

本轮根已完成12项exit0（snapshot49/49）及网站typecheck/build exit0只证明各自范围；strict已exit0，final full/最终sole-Claude、signed root/site及049安全CI/C-D尚待。故BASE仍in-progress/Acceptance空、未source-qualified，COV2-22只可独立准备，实际消费前仍须既有DEP-COV2-029-BASE的精确signed交接，依赖/写集/卡数不改。父source61/71/formal55/71与COV source13/26=50.00%/formal0/26保持；历史1369B/UNRUN/失败记录和GLM historical import020 FAIL/deferred/default reject保留，Source A UNPROVEN，COV家族不complete。

### 2026-10-14 20:10:13 UTC — BASE source-qualified交接与临时049例行事实（不改规范）

**实际源码与裁决：** `OG-UPGRADE-029-01`的8AC/2VER和既定mandatory门已按本卡实际证据关闭A/B，Lifecycle为`in-progress / locally-accepted`、source-qualified，不是formalcomplete。sole Claude Code R2 raw SHA256 `1da1fc13e2798ab51918f09f83de5499e87f204deea9614612b6ea6c01ca9c56`实际PASS（0P0/0P1/0P2，8P3由root接受）；此前BASE R1为PASS并含1P2（已修复），更早失败及其余历史观察保留。本轮12root+1site的postimage/mode均与实际R2 manifest同一；该PASS只覆盖已签原字节，本次三计划新增事实随发布delta再交soleClaude，不借旧审查预授新字节。root实际交接收据`/private/tmp/libra-base29-final-source-claude-r2/SOURCE-QUALIFIED-HANDOFF.json`，SHA256 `80a5018684f5f61a63381e999e38ed3b3f88e48f990502dac15d3b7f1a038cc9`。

**最终实现full真实终态：** 同时加载两份本地测试环境的`cargo nextest run --all --no-fail-fast --retries 2`，2026-10-14 19:13:00→20:03:39 UTC实际exit0：`Summary [2009.557s] 10133 tests run: 10133 passed (2 slow), 23 skipped`；2slow均最终PASS，实际0retry/0flaky，23skipped不计通过，不代替本机OpenCode/账号/模型或22例家族Session验收。raw SHA256 `24febb2eba3d2aad09c6f516988e06e9ae127ad8dd96b362fbda4ea45671fdb2`；`RESULT.json` SHA256 `da15c86a546db964843cc9ccb0ab64554e248ec471a03b53b6ef23be6f8a1ae3`。全程2043项源码/测试/配置/文档输入的前后SHA/mode相同；该full包含最新5c cat-file、FIX/IDLE/live/BASE当前最终实现，取代旧2b+COV25冻结full作为本次实现验收，旧记录仍按原边界保留。仅后三计划例行事实及纯版本faces/self-lock/发布说明变动，可依用户原授权逐文件证明实现同一后复用此full；实际实现变化仍须重新验证，不解除计划最终全量要求。

**signed源与计数：** main `1e2177e11029e28e3c81afbe6ec9f1609cdde7da`（parent84bb）和网站cf `93b10519e1943ea3eea7c943e816418e92b1ca0b`（parenta80d）均signed+DCO、Push实际0；root实际Libra ls-remote各terminal0核同SHA。父source从61/71变为 **62/71=87.32%**，formal仍 **55/71=77.46%**；COVsource **13/26=50.00%**、formal **0/26**均不变，不新增Task/分母。后继COV2-22可消费该精确source链按原DAG/共享I逐hash/mode串行重基；本次049冻结发布期间不并发Push新main源码。网站cf `93b10519e1943ea3eea7c943e816418e92b1ca0b`已实际`pnpm deploy:cf` exit0，765 tracked source前后SHA/mode同一；Worker `f446c1f2-117d-4095-939f-b4db87037512`，生产`https://libra.tools/en/docs/commands/agent`实际HTTP200，BASE整节六个完整段落在去script/style的visible HTML中规范空白逐段匹配全true（不是整页pixel比较）。部署RESULT SHA256 `057564431f1382edcfe37c59b17cfde40f94721c9b81bf9efe5fe8a20803e603`、HTTP-RESULT SHA256 `018d8899dc7450104205f0683c98d067716e06ea167a98352f7326870912118c`；旧同目录RESULT的HTTP=PENDING只是部署完成/HTTP测量前历史，后续HTTP-RESULT已实际覆盖。

**继承遗留明确延期，不计卡/不记修复：** Mega2 interactive `--ref`显示所选ref但变更写trunk的既有风险，actual passive Claude raw `ca9acecec67478006ce9cab2e8acf2cb2a5a0200fb6b16111b8a705d67efe99c`为literalPASS/0P0/P1、1P2/5P3，按用户外围难点延期政策保留UNFIXED/deferred。最新upstream5c cat-file actual Claude raw `8ff6548ff34d745d407239b724c9e919f510f714bb5db66b4915058f5c39a292`为literalPASS/0P0/P1、3P2/2P3，root按用户单Claude PASS规则接受；buffer队列/损坏对象及EPIPE专项测试缺口、长期blocking read、既有BrokenPipe exit0及更大buffer丢失窗口、trim文案精度、无界输入queue/line观察全部未实现修复/延期。最终full中现存cat-file用例实际通过，不宣称补齐缺失场景。默认CLI的main→cli::parse静态路径使用Tokio multi_thread；这只澄清默认runtime事实，不证明任意embedding或取消/阻塞边界已修复。

**版本操作事实（root执行，不在本三计划候选写集）：** root已对Cargo.toml/install.sh/install.ps1三个版本faces、Cargo.lock自包及CHANGELOG作最小0.30.49 bump；`cargo update --workspace --offline`实际exit0，lock仅libra自包48→49、无依赖漂移。该动作不等于版本guard/发布审查、签推/安全CI、tag/稳定发布/8job或安装通过；本候选没有改任何版本文件。

**发布与权限边界：** USER-TEMP-REL-049仍是唯一临时patch0.30.49操作。最新SHA必要安全CI、版本面/fmt/strict/发布delta单Claude、8releasejob全部success、四平台签名manifest/资产、CDN/Homebrew、用户指定HTTPS curl安装home正式目标、以同一正式资产原子替换仓库临时目标及installed cat-file smoke均尚待实际结果；不能因本次full或source-qualified记发布/C-D/全计划complete。历史unsupported-state OpenCode import仍LBR-AGENT-020 FAIL/默认拒绝且计划外延期；Source A/native GO仍UNPROVEN，旧V2 Partial零bytes、pending/native权限与COV2家族C/D/DEP-OG-13均不放宽。复杂OpenCode Session/account/model仅macOS本机，Linux功能用户后验；不将未运行/失败/skip/synthetic记PASS。

### 2026-10-14 22:28:49 UTC — USER-TEMP-REL-049 四卡实际C/D收口

USER-TEMP-REL-049 / 0.30.49 的实际C/D已由root按完整收据 `dad01d952f7ce629354118264a4926b048693bf851258e4cd14beb9853727f8c` 逐项裁决关闭；release source/tag peel为 `04617cd2a3ffbf3a3e68ce4ddfecf94b78bc6574`，release run `38639329379` attempt `2` 的8个job全部success，当前SHA CI/CodeQL、四平台签名manifest/资产及CDN、pinned Homebrew、网站HTTP200/visible六段、用户指定curl home安装、同正式Darwin资产原子替换root目标及installed cat-file/既有BASE1345B读回均有hash绑定的实际证据。

**精确覆盖四卡：** `FIX-OG-NATIVE-IMPORT-01`、`OG-IDLE-ERROR-029-01`、`OG-UPGRADE-029-01`、`FIX-OG-PARTIAL-LIVE-01`。原逐卡A/B、单Claude、signed source/网站交接及fresh BASE证据依现行四卡映射继承；不新增模型/Session或历史import验收。仅四张父卡由locally-accepted转done/complete，父formal **59/71=83.10%**（原55增加4），source仍 **62/71=87.32%**；COV source **13/26=50.00%**、formal **0/26**，71/26分母不变。COV2-25/COV2-REL-01/DEP-OG-13、OG24..26的FAST-OG-04、Source A及native三字段未因此关闭；Source A UNPROVEN，历史unsupported-state import020仍FAIL/默认拒绝/计划外deferred。真实provider idle.error正例仍未观察，本卡typed机制按原7AC/3VER验收，不把成功Session或synthetic称真实错误/native来源。既有FAIL/cancelled/skip、Mega2与cat-file延期观察保留。

**实际身份：** signed manifest核验后的Darwin binary SHA256 `fffd723d0997d2cc0a2fccad7e31c0ae0d19f55d648ed635b3636f1ac051409e`、`79866176` bytes；home与root两个安装目标为同一正式资产。BASE原saved checkpoint `7d465f02-26f6-5fb8-a1d6-72164ac425f2`普通读回1345B/SHA `96583a9c4f4db01efffee8a6a31fe9f4a791ed866cda9c2a6893ebf8fd422d4a`、raw=false/truncated=false/native artifacts0。完整本机C/D收据 `/private/tmp/libra-temp49-parent-complete-r1-tdqm0h5x/USER-TEMP-REL-049-COMPLETE.json` / SHA256 `dad01d952f7ce629354118264a4926b048693bf851258e4cd14beb9853727f8c`；它只证明所列原门，不追认任何旧失败。

### 2026-10-15 — COV2-22 采用候选的例行事实（不改规范）

仅合并本卡原实现说明、原 8 AC/6 VER 的实际机制 checkbox、R4 环境失败/合法私有 init/R5 完整 10139 passed（2 slow/2 leaky/23 skipped）及两条原测试一次诊断、真实旧源码 writer 原样本、原 Claude R1 与最终新事实审查待门。root 最新 049 完成收据与父 59/71/source62、COV family formal0 原样保留；source14/26 仅为本次采用实际 source-qualified 后的条件计数。原判据/命令/I/R/DAG/Granularity 未改，本次不实施 COV2-08，不预授未知签名提交或 Source A。


## 2026-10-15 G09：OG24 incoming snapshot / DEP13共享写权阶段（当前覆写）

本节只修订跨计划前置资源与共享I顺序；COV各卡原AC/VER/I/R/Granularity、26卡分母和COVREL12AC12VER不变。当前13/26 source、formal0/26，source-ready仍须全部25非发布卡及22例最终行为源码macOS非skip门，不因当前计数或本G09满足。

本COV canonical「依赖登记表」已逐项登记对应五个stage resource及八列owner/产物/判据/证据/失败策略；SRC029完整来源门消费[父计划S1–S8](plan-20260902.md#native029-source-profile-gates)。

## Stage ordering and current/count mirrors

RouteA: frozenOG24.2@6685 → COV25-nonrelease-chain → DEP13.source-ready → COVREL.source-freezeF → SHAREDgrant → OG24.3landing → RETURN → COVREL.closeout → DEP13.complete → OG27.
RouteB ifSRC029notready beforegrant: frozenOG24.2@6685 → COVsourcechain → DEP13.source-ready → COVREL.source-freezeF → COVREL.closeout → DEP13.complete → SHAREDgrant → OG24.3landing → RETURN → OG27.
Both: SRC029 → OG24.3landing → RUN029 → OG27. Routes mutually exclusive and receipt-selected beforegrant; never unite conflictingedges. These are existingTask stages/resources, not newTasks.

DEP13 source-ready retains exact25nonrelease/22cases signedsource gates; source-ready.sharedpaths additionally requires release-own I/R sourcefreeze and explicit lease. complete retains COVREL actualstable/signatures/safetyCI/4platform/site/Homebrew/curl/install/allC-D. Originalold48reject and 'installCOVstable then retry2' remain historical .2 only; current3runtime uses local exactsigned3Libra, not stable49. OG27 mainPush stillaftercomplete+realRUNsignature.


### COVREL I/R交出与事实写权归还

DEP13.source-ready.sharedpaths须同时具有原source-ready收据、COVREL自己的三planI及R最终源冻结F（full/唯一Claude/signed源、逐pathSHA/mode和原排除OG27门）、具名owner/UTC/route与排他lease。sharedroot路径明确三plan、tracing/agent.md、EN/zh agent.md、COMPATIBILITY.md、tests/compat/agent_docs_contract.rs；OG24其它5src/2tests/2fixtures/2MANIFEST/内部文档也仅其owner可写。cf独立owner单页agent.en.md另给signed SHA/mode。COVREL R不交出，始终仅root对immutableF构建/tag/release/manifest；grant期间COV/REL相交source/docs/三plan事实写者暂停；OG24唯一writer按grant范围持有共享写权，cf仍由登记的唯一owner按其grant写入；其它计划非保护region只可按父计划abort equality的逐signed非OG24账本由root串行接入，不全局冻结无关计划。此为G06/G10有向stage约束，不是时间窗例外。

RouteA在SRC029已合格时：F→grant→OG24.3landing→signedreturn→COVRELcloseout→DEP13complete。return提供每共享postimageSHA/mode、signedsource和最后writer；root在返回postimage上**唯一**写实际C/D三planfacts，保留3记录，不改F/tag，不称F含3。COV站点既有内容保留；若cf页源被OG24改，原cfowner须绑定新signed页源重跑既定site/HTTP而不继承旧页面hash。RouteB在SRC未就绪时：F→RELcloseout/complete→grant→OG24.3landing→return。两route在grant前互斥选定收据，不能混成cycle。需要源修复则中止generation/新F和grant，immutable发布不改写。

COV incoming OG24只有signed6685 `.2`snapshot，不消费liveOG24current3生命周期；旧0.30.48拒绝仅历史；GC-COV2-08/COVREL VER2的installed-F legacy原`.2` issuer在fresh owned registry中按epoch0兼容登记仍绑定F，独立owned HOME/XDG执行，不改global、不作current29/`.3`或SourceA验收，当前RUN029用exactsigned3main本地非releaseLibra/ownedHOME/XDG，不要求本COVrelease含3；OG27 mainPush仍等DEP13complete及RUN真实signature。SourceA/oldimport020FAIL/deferred/native三字段和复杂Session不上CI边界全保留。

父计划本日G09 S1–S8的全66、official1705nativeclosure/dependency/nativeasset、9506/8595mode与membership、READMEreader、independentofficial29SDKexpected/repro硬门是SRC029来源条件；COV不替它审source/授issuer。FAST04 C/D仍独立，下游不能反作本COV source前置。发布被动ledger按F实际祖先核全量；后续main3source不是F祖先，不隐瞒、不倒称其由COVstable携带。

**无live OG24回边：** OG25/OG26原producer-independent先序在父Dependencies/Granularity/主图与审计表均绑定signed6685 `.2` snapshot；本COV incoming同一版本化资源，current3只在COVrelease阶段grant之后落地。原历史日期图不建立新current3→OG25/26/COV边。

**A阶段abort-return（唯一post-grant A→B条件）：** root先核没有任何OG24.3实现/站点landing提交已signed或pushed（G09规范采纳不是实现landing）。每个shared path必须等于grant SHA/mode，或严格等于下述逐项准许的非OG24 signed delta重放结果；不能仅以“无关改动”标签放行。未提交OG24草案只留私有目录，不夹入COV事实/R。root出具具名abort-return receipt，绑定grant/F/route、root/cf签推核查、逐path grant/expected/current blob-OID/SHA/mode、全部准许commit及独立ledger SHA、保护region校验、UTC和唯一返还owner后，才返还共享写权并显式A→B。任何3实现/站点landing一旦signed或pushed，即使字节复原也必须原forward-only源交接/修复和成功return，不得abort换B；F、真实gate和原route选择条件不变。

**非OG24 signed-delta重基账本（abort equality的唯一扩展）：** grant时对每个shared path记录完整blob-OID/SHA/mode、唯一边界anchor及该边界内原始bytes/SHA。保护区逐项覆盖OG24卡片与其I/criteria、当前G09/DEP12/13/stage/DAG/统计归属、COVREL与其F/版本source/排除条件、所有lease/grant/return字段；无法用唯一anchor完整界定的path默认整文件受保护。生产源码/fixture/manifest的OG24归属全文件受保护；cf同样由唯一owner登记页面保护段。grant后其它计划仍可推进独立路径和已登记的非保护region，不能并发写共享文件：其signed变更只由root按唯一写者顺序接入，记录实际commit/parent-chain、每path前后blob-OID/完整SHA/mode、精确diff和owner/卡归属，不全局冻结无关计划。abort前root在独立只读ledger逐项核签名/DCO及实际对象链，确认准许delta未触任何保护区/anchor、OG24或COVREL归属、lease、版本source与F条件；模式变化、跨区hunk、未知owner/路径或无法还原的rebase一律拒绝。再从grant完整preimage按实际有序commit delta机械重放（重基也列原/新parent与blob链、精确delta），逐path证明生成的完整expected SHA/mode等于当前actual SHA/mode，同时原保护region bytes/anchors/SHA仍相同。没有delta则expected就是grant；任何差异未被枚举或重放不等即不出abort receipt。此ledger不移动F、不改变source-ready/原gate、不授权OG24已签实现回退。

**R6准备期状态快照（2026-10-15；下段命令/判据继续有效，原UNRUN状态由页首实际QA事实覆写）：**

**本次G09规范采纳提交的读者门（mandatory ER04，不新增Task/AC/VER）：** 三plan本身也是编译/测试输入；在本次exact拟采纳字节上实际运行 `source .env.test && cargo test --test opencode_hook_contract_test`（整target，含原VER8读者）、`source .env.test && cargo test --test compat_agent_docs_contract`、`source .env.test && cargo test --test compat_agent_architecture_guard`、`cargo +nightly fmt --all --check`、`cargo clippy --all-targets --all-features -- -D warnings`、`libra diff --check`，并用rg核实际其它plan读者再跑相关target。全部非零实际通过、soleClaude本exact规范PASS后root才能signed+DCO采纳；此时均UNRUN，不以probe字节相同代替运行。原OG24.3源码landing另跑原8VER/full等门不减。

**F独立legacy兼容lane（原门保留，非current profile替代）：** GC-COV2-08及COVREL VER2对immutableF的`.2`source/admission兼容核验和installed-F原`.2` issuer在fresh owned registry中按epoch0登记重验仍按原文执行，属于COVREL发布/安装验收owner；只在新owned HOME/XDG/registry与固定旧`.2`产物运行，freshregistry才0，不改用户globalOpenCode/exporterslot。它读取F/历史6685`.2`许可，不依赖current3landing/RUN029/OG27或真实native字段归档，故不向源链回加边。该lane通过也只证F legacy兼容，不是official29/`.3`验收或SourceA；currentSRC/RUN3及OG05三字段门仍独立。旧0.30.48拒绝事实保留。

**RouteA分支边明确登记：** `SHARED.grant_A→OG24.3.unsigned_attempt→RETURN.abort_A→REL.closeout→DEP13.complete→SHARED.grant_B→OG24.3.landing→RETURN.signed_B` 仅在abort严格谓词全部满足后成立；否则只可原成功A边`grant_A→OG24.3.landing→RETURN.signed_A→REL.closeout`。abort不是signed实现回退，不把两分支同时当活动边；所有C/D事实仍由root在实际return后写。
