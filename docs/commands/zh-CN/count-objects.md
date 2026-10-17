# libra count-objects

清点松散对象及其磁盘占用，对齐 `git count-objects`。

## 用法

```
libra count-objects [-H | --human-readable] [-v | --verbose]
```

无参数时输出 `"<数量> objects, <大小> kilobytes"`。`-H` 以人类可读单位渲染
（bytes / KiB / MiB / GiB，阈值与 Git 完全一致）。`-v` 输出 Git 的详细字段集：

```
count: <松散对象数>
size: <松散 KiB>
in-pack: <pack 内对象数>
packs: <pack 文件数>
size-pack: <pack KiB>
prune-packable: <同时存在于 pack 的松散对象数>
garbage: <垃圾条目数>
size-garbage: <垃圾 KiB>
alternate: <alternate 对象目录>      （0..n 行）
```

`--json` / `--machine` 输出结构化清点（原始字节与 KiB 字段，`-v` 追加
`garbage_entries` 的 `path`/`reason`，上限 100 条并带
`garbage_entries_truncated`）。结构化模式不输出人读 warning，垃圾明细进入
`data.garbage_entries`。`data.alternates` 是没有 C 显示引号的原始 UTF-8
路径字符串；非 UTF-8 alternate 在结构化模式报 `LBR-IO-001`，可用文本模式
查看逐字节转义。人类 `alternate:` 行采用 Git 默认的 C 风格转义，普通空格
不会触发引号。

## 示例

```
libra count-objects                   清点松散对象与总大小
libra count-objects -H                人类可读大小
libra count-objects -v                详细 Git 字段集
libra count-objects -v --json         Agent 用结构化清点
```

## 与 Git 的有意差异

- garbage 与 `alternate:` 路径打印为绝对路径（Libra 可从任意子目录运行，
  且存储目录名不同）。
- 分片或 pack 目录不可读时 fail-closed 报 `LBR-IO-001`；Git 只告警后继续
  （会少计数，可能让前后比对假通过）。
- 真实变更命令写入的 Operation Log 快照对象计入 `count`，因此变更后 Libra
  计数通常大于 Git。
- 垃圾明细和人类 warning 最多列出前 100 条；总数仍完整，仅超过 100 条时
  truncation flag 才为 true。
- 只统计本地存储层，不统计 tiered 远端对象，不发网络请求。

## 保证

- 只读：不重建 `.idx`、不迁移 schema、不取维护锁、不记录 operation。
  `count-objects -v` 输出逐字节稳定。
- 裸仓库、子目录、linked worktree 均可用，成功时 exit 0。

## 错误

- 仓库外 `LBR-REPO-001`（128）。
- 未知 flag 或位置参数 `LBR-CLI-002`（129）。
- 对象库不可读 `LBR-IO-001`（128）。
