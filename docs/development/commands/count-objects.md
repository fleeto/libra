# count-objects

## 状态

`partial` — Git 的 `count-objects` 语义已实现（无参数、`-v`、`-H`、JSON），
用于「命令未写入对象」断言与 gc/repack 后校验。issue #483。

## 契约

- `command_scope`: `ReadOnly`；预检 `repo_hash_kind_without_schema_guard`
  （不迁移 schema）；分发 `execute_safe`。
- 扫描器在 `src/command/count_objects.rs`，严格只读：
  - 松散分片按 Git 语义分类（2 位十六进制分片、名称长度 = hexsz-2、普通
    文件），垃圾条目按 `garbage_found` / `no_corresponding_*` 报告。
  - pack 目录按基名分组，`packs`/`in-pack`/`size-pack` 仅本地 pack；idx 头
    或 fanout 损坏则跳过该 pack，不失败。
  - `prune-packable` 对每个松散对象在本地与 alternate 的 idx 上做一次
    fanout 桶内二分查找。
  - alternates 走 `alternates::resolve_chain`（传递、防环、深度上限），
    仅报告 `alternate:` 行与其 pack 目录垃圾。
- 只读 pack idx helper 在 `src/utils/storage/local.rs`：
  `LocalStorage::pack_index_object_count`（fanout 尾）与
  `LocalStorage::pack_index_contains`（复用 `read_idx_from_open_binary`）。
  禁止复用会写盘的 `list_all_idx` 与 `list_loose_objects`。
- hexsz 取 `get_hash_kind().size() * 2`，不硬编码。
- 人类可读 `humanise_bytes` 逐字复刻 Git `strbuf_humanise_bytes`。
- 路径按 `quote_c_style` 转义。

## 有意差异（四处同步：EN/zh 命令页、COMPATIBILITY、本文件）

1. garbage/alternate 路径为绝对路径。
2. 分片/pack 目录不可读 fail-closed `LBR-IO-001`（Git 告警后继续）。
3. Operation Log 快照对象计入 `count`。
4. 只统计本地 tier。

## 错误码

复用 `LBR-REPO-001`（128）、`LBR-CLI-002`（129）、`LBR-IO-001`（128），
无新增。

## 测试

- `--lib command::count_objects`（humanise、quote 边界）
- `--test command_test count_objects_test`（M-FMT/M-LOOSE/M-PACK/M-GARBAGE/
  M-ALT/M-REPO/M-ERR、零写入、JSON）
- `--test compat_matrix_alignment`、`compat_help_examples_banner`、
  `compat_command_docs_examples_section`、`compat_ledger_schema`

## 延后

- `gc` / `prune-packed`（#385）。
- `read_idx` 热路径改二分（DEFER-05）。
- gc/evict 松散枚举迁移到本扫描器（DEFER-06）。
- linked checkout garbage（DEFER-04）。
