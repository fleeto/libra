# CI 基线稳定性修复任务（2026-09-20）

## 背景

PR #505 的 Rustdoc 修复已通过 `compat-clippy` 与 Rustfmt，但
`compat-offline-core` 在 `main` 基线暴露了 22 个测试失败。失败不属于
Rustdoc 改动本身，主要由 CI runner 的文件权限环境和两个测试对仓库上下文
的隐式依赖造成。

## 执行顺序

1. `fix/ci-test-umask` / PR：修复 CI 测试环境的 `umask`，并让配置修复
   fixture 显式创建安全文件权限；目标是消除可执行位与配置修复测试的权限
   假设差异。
2. `fix/archive-tests-repository-context` / PR：让两个 archive 测试使用
   临时 Libra 仓库，不依赖 GitHub checkout 根目录已存在 `.libra`。

两个 PR 均从最新 `origin/main` 创建，分别验证后再合并。PR #505 保持独立，
不混入上述测试/CI 基线修复。

## 当前任务

- 当前分支：`fix/ci-test-umask`
- 当前阶段：`umask` 与配置 fixture 权限修复已实施并通过 focused 验证，待提交 PR
- 后续分支：`fix/archive-tests-repository-context`

## 验收

- 相关 focused nextest 用例通过；
- `cargo +nightly fmt --all --check` 通过；
- `cargo clippy --all-targets --all-features -- -D warnings` 通过；
- `compat-offline-core` 全量测试无新增失败。
