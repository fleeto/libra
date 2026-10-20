# `libra repack`

把仓库对象合并到单个 pack。

## 概要

```
libra repack [-a|--all] [-d|--delete] [-l|--local] [-q|--quiet]
             [--window <N>] [--depth <N>]
```

## 说明

`repack` 将仓库对象编码进一个新的 `pack-<checksum>.pack`（以及匹配的 `pack-<checksum>.idx`），位于 `.libra/objects/pack/` 下。它使用与 `maintenance` tasks 相同的共享 pack writer，因此 Libra 在磁盘上写出的每个 pack 都经过一个格式良好的编码器 — 生成的 pack 可通过 `libra index-pack` 和 `libra verify-pack` 往返验证。

默认只打包当前为 **loose** 的可达对象；已经位于现有 pack 中的对象保持原位。`--all` 会扩大集合到所有可达对象，生成一个合并后的单一 pack。

可达性从 refs、reflogs 和 index 计算 — 与 `libra maintenance run --task gc` 完全相同 — 因此仅被 reflog 引用的对象永远不会被丢弃。

未给 `--window`/`--depth` 时，取值来自 `pack.window` / `pack.depth` 配置，默认为 10 / 50（与 Git 一致）。window 作为滑动 delta 搜索窗口转发给共享 pack 编码器：`0` 禁用 delta 压缩，更大的值会花更多时间寻找更小的 delta。`--depth` 被接受并校验以对齐 Git CLI；上游编码器应用固定的链长上限 50（已登记的上游依赖 ER-MX-03），因此任何被接受的值都会产出合法 pack。

## 选项

| 选项 | 说明 |
|------|------|
| `-a`, `--all` | 把所有可达对象（包括已存储在 pack 中的对象）打包进一个新的单一 pack。 |
| `-d`, `--delete` | 打包后删除现在已位于新 pack 中的 loose objects。只删除 object id 位于新 pack 中的文件。与 `-a` 组合时，对象全部被新 pack 覆盖的旧 pack 也会被删除 — 但只在新 pack 与索引写入并同步落盘之后，且删除前会逐个 pack 重新确认覆盖（任一对象未被覆盖则保留该 pack）。有其他仓库通过 alternates 借用本对象库时拒绝删除（`LBR-CONFLICT-002`）。 |
| `-l`, `--local` | 跳过仅存在于 alternate 对象目录中的对象（`pack-objects --local` 语义）；alternate 专属对象保持原位。 |
| `-q`, `--quiet` | 抑制信息摘要。 |
| `--window <N>` | delta 压缩窗口大小（默认：`pack.window`，再默认 10；`0` 禁用 delta）。 |
| `--depth <N>` | 最大 delta 链深度（默认：`pack.depth`，再默认 50）。 |

带 `--json` / `--machine` 时，命令输出一个信封，其 `data` 对象包含 `pack`（新 pack 名称）、`objects_packed`、`loose_removed`、`packs_removed`，以及解析后的 `window` / `depth`。

## 退出状态

- `0` — repack 完成（包括无需打包的 no-op 情况）。
- 非零 — 命令在仓库外运行，或 pack 无法写入。

## 兼容性

这是 Git `git repack` 的聚焦子集。Libra 每次运行通过共享 writer 写出一个新 pack，支持 `--window`/`--depth` delta 调优（以及 `-a -d` 的冗余 *pack* 删除）；geometric repacking（`--geometric`）、bitmap 生成与 `.keep` 处理未实现。见 [`COMPATIBILITY.md`](../../../COMPATIBILITY.md)。

## 示例

```
# 把 loose reachable objects 打包成一个 pack。
libra repack

# 把每个可达对象合并进一个 pack。
libra repack -a

# 重新打包所有内容，删除已打包的 loose copies 和被完全覆盖的旧 packs。
libra repack -a -d

# 同上，但跳过仅存在于 alternate 中的对象。
libra repack -a -d -l

# 不做 delta 压缩地合并（CPU 开销更小，pack 更大）。
libra repack -a -d --window=0

# 花更多时间做 delta 搜索，得到更小的 pack。
libra repack -a -d --window=50 --depth=100

# 机器可读摘要。
libra --json repack -a -d
```

## 另见

- [`libra maintenance`](maintenance.md) — 计划优化任务（其 `loose-objects` 和 `incremental-repack` tasks 共享此 writer）。
- [`libra index-pack`](index-pack.md) — 为现有 pack 构建 index。
- [`libra verify-pack`](verify-pack.md) — 验证 pack 及其 index。
