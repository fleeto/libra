### `cli.gc-smoke`

目的：覆盖已公开的 `gc` 命令的基本冒烟路径（新仓库 exit 0、有历史仓库的打包与删除、`--auto` 阈值以下零写入、`--json` envelope），并确保同族维护命令 `prune` 当前仍未公开、不会被误当作已发布 CLI；runner 同时验证已发布的 `maintenance run --dry-run --task gc` 入口仍返回成功 JSON envelope，并在场景末尾验证仓库健康。

最小步骤：

```bash
SCENARIO="cli.gc-smoke"
RUN_DIR="$RUN_ROOT/repos/$SCENARIO"
mkdir -p "$RUN_DIR"
cd "$RUN_DIR"
# (prelude libra - short converged)

libra init repo
cd repo
libra config set user.name "Libra GC Test"
libra config set user.email "gc@example.invalid"
printf 'tracked\n' > tracked.txt
libra add tracked.txt
libra commit -m "test: gc base" --no-verify

# 公开的 gc：默认策略跑通，可达对象保留
libra --json gc >gc.json
python3 -c "import json; d=json.load(open('gc.json')); assert d['ok'] is True and len(d['data']['steps']) == 5"
libra rev-parse HEAD
libra show -s --format=%s HEAD | grep 'test: gc base'

printf 'gc unreachable blob\n' > unreachable.txt
OID="$(libra hash-object -w unreachable.txt)"
libra cat-file -t "$OID" | grep '^blob$'

# --prune=now 删除不可达对象；--auto 低于阈值零写入
libra gc --prune=now
! libra cat-file -t "$OID" >/dev/null 2>&1
libra gc --auto | grep 'Auto gc skipped'

# prune 仍未公开为顶层命令
! libra --json prune --dry-run
libra --json maintenance run --dry-run --task gc >maintenance-gc.json
python3 -c "import json; d=json.load(open('maintenance-gc.json')); assert d['ok'] is True"
libra fsck --connectivity-only
```

断言：`libra gc` 与 `libra gc --prune=now` / `--auto` / `--json` 成功；`gc` 后可达对象仍可读且 HEAD 不变；`--auto` 在默认阈值下零写入并提示跳过；`prune` 当前未注册为顶层命令，`libra --json prune --dry-run` 必须非 0 退出，JSON 错误码为 `LBR-CLI-001`；`maintenance run --dry-run --task gc` 必须成功；操作后 `libra fsck --connectivity-only` 通过。

补充可执行断言：
- `libra --json gc` 必须 `ok:true` 且 `data.steps` 含 5 个步骤。
- `libra gc --prune=now` 必须删除刚写入的不可达 loose object。
- `libra gc --auto` 必须输出 `Auto gc skipped` 且不产生 pack。
- `libra --json prune --dry-run` 必须失败，错误码为 `LBR-CLI-001`。
- `libra --json maintenance run --dry-run --task gc` 必须 `ok:true`。
- 操作后 `libra fsck --connectivity-only` 通过。
