# cache 命令开发设计

## 命令实现目标

`libra cache info` 报告本进程解析出的分层存储 / LRU 缓存可调参数（storage 类型、
小/大对象阈值、LRU 磁盘预算），把已有的 `LIBRA_STORAGE_*` 能力暴露出来供检视
（`lore.md` §0.10）。纯配置检视（env + 全局 config DB），不需要仓库。

## 对比 Git 与兼容性

- 兼容级别：`intentionally-different`。Git 无对应；Libra 分层对象存储的诊断扩展。
- 已支持：`cache info`（human + `--json`/`--machine`：`{ storage_type, tiered, threshold_bytes, cache_size_bytes }`）。
- 退出码：0；存储配置值无法解析（如全局 config DB 不可读）时非 0（`resolve_cache_config()?` 上抛，不静默回落）。

## 设计方案

- 入口与分发：`src/cli.rs::Commands::Cache` → `command::cache::execute_safe`；
  列入 `CommandPreflight::none()`（无需仓库/hash-kind preflight）。
- 配置收口：`src/utils/client_storage.rs::resolve_cache_config` 用
  `resolve_env_sync`（先 env 再全局 config DB）做静态报告，
  并共享默认值常量 `DEFAULT_STORAGE_THRESHOLD_BYTES`（1 MiB）/`DEFAULT_CACHE_SIZE_BYTES`
  （200 MiB）。非法数值回落默认。`tiered=false` 只表示静态检查没过，不是 `init`
  造出了可写本地后端。
- 源码分层：`src/command/cache.rs`：`CacheArgs`（子命令 `CacheCommand::Info`）、
  `CacheInfo`（serde）、`execute_safe`/`info`。`CacheConfig` 结构与 `resolve_cache_config`
  在 `client_storage.rs`。
- 底层操作对象：只读存储/缓存 env（或全局 config DB）。始终读 `LIBRA_STORAGE_TYPE`/
  `LIBRA_STORAGE_THRESHOLD`/`LIBRA_STORAGE_CACHE_SIZE`（后两者报告值）；当类型为 `s3`/`r2`
  时，`tiered_static_checks_pass` 还按后端相同顺序解析 `LIBRA_STORAGE_BUCKET`/
  `LIBRA_STORAGE_ENDPOINT`/`LIBRA_STORAGE_REGION`/`LIBRA_STORAGE_ACCESS_KEY`/
  `LIBRA_STORAGE_SECRET_KEY`/`LIBRA_STORAGE_ALLOW_HTTP`，在首个静态回退点（空 bucket/
  非法 endpoint/空 access/secret key）短路 `tiered=false`，从不过报 tiered。无对象库/refs/网络写。

## 实现历史

- 2026-07-02（`lore.md` Phase 0 / 0.10）：`cache info` 暴露分层存储/LRU 可调参数；
  抽出 `resolve_cache_config` + 默认常量。确认失败的对象存储不再由 `init` 降级到本地。

## 当前状态

- 仓库库打不开，或解析没有返回时，`ClientStorage::init` 失败，不构造可写本地后端，只报告设置名，也不进入进程内对象缓存。全局库文件存在但打不开时，对象存储仍用历史本地后端，已有对象仍可读取。future global schema、正好 `local` 和可读的未设置类型仍是本地结果，并且仍可缓存。
- 公开状态：已公开（`Commands::Cache`）。
- 测试：`tests/command/cache_test.rs`（local 默认、tiered env 覆盖、非法数值回落默认、
  human 输出标签）。
- 用户文档：`docs/commands/cache.md`。

## 还未实现的功能

| 类别 | 未完成项 | 当前处理 |
|---|---|---|
| 配置写入 | `cache set` / reserved config 持久化写入 | 仅 `info` 检视；调参仍经 `LIBRA_STORAGE_*` env（或全局 config DB）。写入子命令为后续项。 |
| 缓存用量 | 报告当前 LRU 实时占用 | LRU 为进程内、跨进程不可查；`cache info` 只报配置。用量观测为后续项。 |

## 维护要求

- 改进本命令前先阅读 [docs/development/commands/_general.md](_general.md)。
- 任何对存储/缓存 env 语义或默认值的改动必须同时改 `client_storage.rs`（
  `resolve_cache_config` 与对象存储打开路径共享默认常量），不得各自解析。
