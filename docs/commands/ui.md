# `libra ui`

Open the Libra desktop window. This is a Libra extension. Git has no equivalent.

The window is compiled in only when Libra is built with the `gui` feature.
A default `cargo build` still produces the CLI, and `libra ui` on that
binary explains how to rebuild.

## Synopsis

```
libra ui
```

## Description

`libra ui` with no further arguments opens the Sessions window on the
process main thread. The window uses [Ely GPUI Components](https://github.com/ELY-Open-Source/Ely-GPUI-Components)
and the bundled sessions prototype: a session list, Overview / Conversation /
Changes, search, and a capability map. The rows are demonstration data, not
a live capture index. Close the window to exit.

The command does not read or write a repository.

`libra ui --help` prints this command's usage and does not open a window.
Any other argument is a usage error, also without opening a window.

A build without `--features gui` exits 128 with `LBR-UNSUPPORTED-001`.
The message names `cargo build --features gui`.

Calling the command through the library dispatcher (`libra::exec`) does not
open a window. The window has to start from the `libra` process entry point,
because the UI toolkit owns the main thread.

## Options

`ui` takes no flags of its own. Global flags that appear before the
subcommand (`libra --no-pager ui`) are ignored by the window.

## Exit codes

| Code | Meaning |
|------|---------|
| `0` | The window was opened and then closed. Requires `--features gui`. |
| `128` | This binary has no desktop UI (`LBR-UNSUPPORTED-001`), or the window failed to open. |
| `129` | Extra arguments. `libra ui --help` exits 0. |

## Examples

```bash
# Default CLI build: explains the missing feature and exits 128.
libra ui

# Build the window into the same libra binary, then open it.
cargo build --features gui
./target/debug/libra ui

# Usage text. Does not open a window.
libra ui --help
```

## Comparison with Git

Git has no desktop subcommand. `ui` is `intentionally-different` in
[`COMPATIBILITY.md`](../../COMPATIBILITY.md).
