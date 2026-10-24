# Libra CLI benchmark

This directory provides a reproducible, end-to-end performance benchmark for
the Libra CLI. It adopts the useful experimental boundaries from
[version-control-bench](https://github.com/gitbutlerapp/version-control-bench):
fixtures are built before timing begins, each scenario begins from a known
state, runs are repeated, and the result is machine-readable. It is **not** an
agent benchmark and it does not reuse or report the upstream project's claims.

## Run

From the repository root:

```bash
benchmark/run.sh
```

The default run resolves the current `HEAD` with `libra rev-parse`, exports that
exact source tree through `libra archive --format zip`, builds it in a temporary directory
with `LIBRA_SKIP_WEB_BUILD=1 cargo build --release --locked`, creates fixtures using that binary, and
writes `benchmark/results/<revision>-<timestamp>.json`. The build and fixture
setup times are recorded or excluded; only a fresh CLI process for the requested
operation is timed.

The runner requires `bash`, `cargo`, `libra`, `perl` (for a monotonic-enough
wall-clock sample), `unzip`, and `/usr/bin/time`. It never switches the current
worktree or writes to the repository's Cargo target directory.

The `libra` used to resolve and archive a commit must understand the current
repository schema. The runner first tries `libra` from `PATH`; if it cannot
resolve the revision, it builds a temporary control binary from the current
worktree and records that cost separately as `control_build_elapsed_ms`. Set
`--control-binary /path/to/libra` (or `LIBRA_BENCHMARK_CONTROL_BINARY`) when a
compatible binary is already available.

Benchmark an older local commit or tag without switching the current worktree:

```bash
benchmark/run.sh --revision a0567ce --runs 20 --warmup 5
benchmark/run.sh --revision v0.18.84 --scenario log_history
```

`--revision` must resolve in the local Libra object store. The selected revision
must support the fixture commands and the scenarios it is asked to run; this is
intentional, because a missing command is a compatibility failure rather than a
performance result.

For a quick smoke run or script development, lower the fixture scale and sample
count. `--binary` intentionally bypasses the isolated source build and is only
for this use case.

```bash
benchmark/run.sh --binary "$(command -v libra)" --scenario status_clean \
  --file-count 100 --runs 1 --warmup 0
```

## Scenarios and measurement contract

| Scenario | Fixture | Timed command |
| --- | --- | --- |
| `status_clean` | committed tree with `--file-count` files | `libra status --short` |
| `status_dirty` | same shape, with ten tracked files changed | `libra status --short` |
| `log_history` | linear history with `--history-count` commits | `libra log --oneline` |
| `rev_list_refs` | one commit with `--ref-count` branches | `libra rev-list --all --count` |
| `fsck_history` | the linear history fixture | `libra fsck` |

The defaults deliberately line up with the existing repository performance
budgets: 1,000 commits for history operations and 10,000 refs for `rev-list`.
Every recorded sample starts a new CLI process and runs in the same prepared
fixture. The runner reports min/median/mean/max wall time plus maximum RSS for
each scenario. It uses `/usr/bin/time -l` on macOS and `/usr/bin/time -v` on
Linux; memory is normalized to bytes.

Results contain the exact resolved revision, build mode, fixture scale, platform,
sample counts, and per-scenario aggregates. Generated JSON files are ignored so
that comparisons can be attached to a PR or stored by CI without accidentally
committing local measurements.

## Verification

```bash
benchmark/test/test-runner.sh
```

This is an end-to-end contract test: it verifies CLI options and confirms that a
pair of real scenarios writes the documented JSON shape. By default it follows
the revision-build path, so it is intentionally as expensive as a release build.
After building a compatible local binary, use
`LIBRA_BENCHMARK_TEST_BINARY=/path/to/libra benchmark/test/test-runner.sh` for a
fast fixture-and-result check.

## 本机 status 三方对比

`bash benchmark/status.sh --installed-libra /path/to/installed/libra --built-libra /path/to/release/libra --git /usr/bin/git --libra-repo /path/to/libra-repo --git-repo /path/to/git-repo --output /tmp/status.json`

Linux 上默认预热 3 次、交错测量 20 次、3 轮；脚本不创建 fixture、不刷新 index。输入仓库需使用可校验的 SHA-1 v2/v3 index。记录二进制、HEAD、tracked 内容摘要和 stat 命中率。命令失败立即拒绝该次测量。子进程警告以字节数和摘要记录，不掩盖执行环境差异。wait4 CPU/context switches 仅覆盖主进程和它已回收的子进程，不能当作完整 worker CPU；独立资源运行同时采样进程树 CPU（内核 tick 精度、可能遗漏退出前尾部）与 RSS；内存在独立非计时运行中每 5 ms 采样进程树 RSS，可能漏掉短峰值，且共享页会重复计数。不得与此前单进程 maxrss 混算。

诊断构建：`cargo build --release --features status-perf --bin libra`；设置 `LIBRA_STATUS_PERF_FILE=/tmp/status-perf.json` 后运行一次 status。文件必须不存在，且仅包含阶段耗时与计数，不记录内容。正常验收须重新使用不带该 feature 的 release；诊断阶段存在嵌套，不可相加当作总耗时。
