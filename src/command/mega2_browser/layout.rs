//! Cell-width-aware, bounded terminal rendering. Remote text never emits ANSI.

use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use super::{
    BrowserState, Editor,
    explorer::Explorer,
    highlight::{Span, Tone},
};
use crate::internal::protocol::mega2_tree::ContentType;

pub fn safe_text(text: &str) -> String {
    text.chars().map(|ch| if ch.is_control() || matches!(ch, '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}' | '\u{2028}' | '\u{2029}') { '?' } else { ch }).collect()
}

/// Slice by terminal cells, never by UTF-8 bytes; pad to keep the divider fixed.
fn cells(text: &str, offset: usize, width: usize) -> String {
    styled_cells(text, offset, width, &[])
}

fn styled_cells(text: &str, offset: usize, width: usize, spans: &[Span]) -> String {
    let mut out = String::new();
    let mut column = 0;
    let mut written = 0;
    let mut span_index = 0;
    let mut active_tone = Tone::Plain;
    let mut has_text = false;
    for (byte, ch) in safe_text(text).char_indices() {
        let size = ch.width().unwrap_or(0);
        let next = column + size;
        if column < offset {
            column = next;
            continue;
        }
        if written + size > width {
            break;
        }
        if size > 0 || has_text {
            while spans.get(span_index).is_some_and(|span| byte >= span.end) {
                span_index += 1;
            }
            let tone = spans.get(span_index).map_or(Tone::Plain, |span| span.tone);
            if tone != active_tone {
                out.push_str(tone.ansi());
                active_tone = tone;
            }
            out.push(ch);
            has_text = true;
        }
        written += size;
        column = next;
    }
    if active_tone != Tone::Plain {
        out.push_str(Tone::Plain.ansi());
    }
    out.push_str(&" ".repeat(width.saturating_sub(written)));
    out
}

pub fn body_rows(height: usize) -> usize {
    height.saturating_sub(8).max(1)
}

pub fn render(
    view: &Explorer,
    state: &BrowserState,
    server: &str,
    width: usize,
    height: usize,
) -> String {
    // Leave the last column/row unused to avoid autowrap and terminal scrolling.
    let width = width.min(512).saturating_sub(1);
    let height = height.min(256).saturating_sub(1);
    let mut lines = Vec::new();
    if width < 48 || height < 9 {
        lines.push(cells(
            "Window too small: resize to at least 49 x 10.",
            0,
            width,
        ));
        lines.push(cells("q quit", 0, width));
    } else if let Some(panel) = &state.panel {
        let rendered = panel.render_lines();
        let all: Vec<_> = rendered.lines().collect();
        let footer = all.len().saturating_sub(4);
        let selected = all
            .iter()
            .position(|line| line.starts_with("> "))
            .unwrap_or(0);
        let visible = height.saturating_sub(5);
        let top = selected
            .saturating_sub(visible.saturating_sub(1))
            .min(footer);
        lines.push(cells("Tags — t/Esc returns to directory tree", 0, width));
        lines.extend(
            all[top..footer]
                .iter()
                .take(visible)
                .map(|line| cells(line.trim_end_matches('\r'), 0, width)),
        );
        lines.extend(
            all[footer..]
                .iter()
                .map(|line| cells(line.trim_end_matches('\r'), 0, width)),
        );
    } else {
        let left = (width / 3).clamp(18, 48);
        let right = width - left - 3;
        let pair = |a: &str, b: &str| format!("{} │ {}", cells(a, 0, left), cells(b, 0, right));
        lines.push(cells(
            &format!(
                "mega2 browser  {server}  ref: {}",
                state.git_ref.as_deref().unwrap_or("(default)")
            ),
            0,
            width,
        ));
        let content_title = format!(
            "{}{}",
            if view.content_focus {
                "File content [active]"
            } else {
                "File content"
            },
            if view.preview.highlighted.language.is_empty() {
                String::new()
            } else {
                format!(" · {}", view.preview.highlighted.language)
            }
        );
        lines.push(pair(
            if view.content_focus {
                "Directory tree"
            } else {
                "Directory tree [active]"
            },
            &content_title,
        ));
        lines.push(pair(
            &view.tree.root,
            view.preview.path.as_deref().unwrap_or("Select a file"),
        ));
        lines.push(format!("{}─┼─{}", "─".repeat(left), "─".repeat(right)));
        let visible = body_rows(height + 1);
        let top = view.tree.selection.saturating_sub(visible - 1);
        for row in 0..visible {
            let index = top + row;
            let tree_text = if let Some(item) = view.tree.rows.get(index) {
                let last = view.tree.rows[index + 1..]
                    .iter()
                    .find(|next| next.depth <= item.depth)
                    .is_none_or(|next| next.depth < item.depth);
                let branch = if last { "└─" } else { "├─" };
                let icon = match item.kind {
                    ContentType::Directory if item.expanded => "[-]",
                    ContentType::Directory => "[+]",
                    ContentType::File => "   ",
                };
                format!(
                    "{} {}{branch} {icon} {}{}",
                    if index == view.tree.selection {
                        ">"
                    } else {
                        " "
                    },
                    "  ".repeat(item.depth.min(left.saturating_sub(20) / 2)),
                    item.name,
                    if item.kind == ContentType::Directory {
                        "/"
                    } else {
                        ""
                    }
                )
            } else if row == 0 && view.tree.rows.is_empty() {
                "(empty directory)".into()
            } else {
                String::new()
            };
            let content = if let Some(line) = view.preview.lines.get(view.preview.top + row) {
                let number = format!("{:>5} ", view.preview.top + row + 1);
                format!(
                    "{number}{}",
                    styled_cells(
                        line,
                        view.preview.left,
                        right.saturating_sub(number.width()),
                        if view.color_enabled {
                            view.preview
                                .highlighted
                                .lines
                                .get(view.preview.top + row)
                                .map_or(&[], Vec::as_slice)
                        } else {
                            &[]
                        },
                    )
                )
            } else if view.preview.lines.is_empty() {
                cells(
                    if view.preview.message.is_empty() {
                        "Select a file to preview."
                    } else {
                        &view.preview.message
                    },
                    row * right,
                    right,
                )
            } else {
                cells("", 0, right)
            };
            // Both halves are already clipped/sanitized. Never re-sanitize the
            // renderer's own color codes or include their bytes in cell widths.
            lines.push(format!("{} │ {content}", cells(&tree_text, 0, left)));
        }
        lines.push("─".repeat(width));
        let status = if let Some(editor) = &state.editor {
            match editor {
                Editor::CreateDir(input) => {
                    format!("new directory: {input}  | Enter create, Esc cancel")
                }
                Editor::ConfirmDelete(name) => {
                    format!("delete '{name}'? Enter/y confirm, Esc cancel")
                }
                Editor::MoveDir { name, input } => {
                    format!("move '{name}' to parent: {input} | Enter confirm, Esc cancel")
                }
                Editor::RenameDir { name, input } => {
                    format!("rename '{name}' to: {input} | Enter confirm, Esc cancel")
                }
            }
        } else if let Some(status) = &state.status {
            status.clone()
        } else if view.content_focus {
            format!(
                "Content: line {}/{}  column {} | arrows/PgUp/PgDn scroll",
                (view.preview.top + 1).min(view.preview.lines.len()),
                view.preview.lines.len(),
                view.preview.left + 1
            )
        } else {
            "+ new dir | d delete | m move | R rename | r reload | t tags".into()
        };
        lines.push(cells(&status, 0, width));
        let help = if state.editor.is_some() {
            state
                .status
                .as_deref()
                .unwrap_or("Enter confirm | Esc cancel")
        } else {
            "Tab pane | arrows move | Enter open/fold | q quit"
        };
        lines.push(cells(help, 0, width));
    }
    lines.truncate(height);
    format!("\x1b[2J\x1b[H{}\x1b[0m", lines.join("\r\n"))
}

pub fn terminal_size() -> (usize, usize) {
    #[cfg(unix)]
    {
        // SAFETY: winsize is initialized and ioctl writes only that structure.
        let mut size: libc::winsize = unsafe { std::mem::zeroed() };
        if unsafe { libc::ioctl(libc::STDOUT_FILENO, libc::TIOCGWINSZ, &mut size) } == 0
            && size.ws_col > 0
            && size.ws_row > 0
        {
            return (size.ws_col as usize, size.ws_row as usize);
        }
    }
    #[cfg(windows)]
    {
        use windows_sys::Win32::System::Console::{
            CONSOLE_SCREEN_BUFFER_INFO, GetConsoleScreenBufferInfo, GetStdHandle, STD_OUTPUT_HANDLE,
        };
        // SAFETY: the console API validates the handle and writes to initialized info.
        let mut info: CONSOLE_SCREEN_BUFFER_INFO = unsafe { std::mem::zeroed() };
        if unsafe { GetConsoleScreenBufferInfo(GetStdHandle(STD_OUTPUT_HANDLE), &mut info) } != 0 {
            return (
                (info.srWindow.Right - info.srWindow.Left + 1) as usize,
                (info.srWindow.Bottom - info.srWindow.Top + 1) as usize,
            );
        }
    }
    (100, 30)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::internal::protocol::mega2_tree::Mega2TreeSession;

    #[test]
    fn cells_handle_unicode_and_never_emit_terminal_controls() {
        assert_eq!(cells("中文x", 0, 3), "中 ");
        assert_eq!(cells("中文x", 2, 3), "文x");
        assert_eq!(cells("a\x1b[2J\u{202e}", 0, 9), "a?[2J?   ");
    }

    #[test]
    fn rust_preview_highlights_syntax_without_executing_file_control_codes() {
        let mut state = BrowserState::new(
            Mega2TreeSession::new("http://localhost:9000").unwrap(),
            "/",
            None,
        )
        .unwrap();
        state.entries = vec![("main.rs".into(), ContentType::File)];
        let mut view = Explorer::new(&state).unwrap();
        view.apply_preview(Ok(
            "fn main() { let s = \"hello\"; }\n// \x1b[2J danger".into()
        ));
        let frame = render(&view, &state, "http://localhost:9000", 120, 30);
        assert!(frame.contains("\x1b[35mfn"), "Rust keyword must be colored");
        assert!(
            frame.contains("\x1b[32m"),
            "string must have a distinct color"
        );
        assert!(frame.contains("?[2J danger"));
        assert_eq!(
            frame.matches("\x1b[2J").count(),
            1,
            "only renderer clears screen"
        );
    }

    fn without_colors(text: &str) -> String {
        (31..=39).fold(text.to_string(), |text, code| {
            text.replace(&format!("\x1b[{code}m"), "")
        })
    }

    #[test]
    fn colored_cells_preserve_unicode_width_and_reset_before_padding() {
        let text = "中文let = \"hello\"";
        let spans = [
            Span {
                end: 9,
                tone: Tone::Keyword,
            },
            Span {
                end: text.len(),
                tone: Tone::String,
            },
        ];
        for offset in [0, 1, 2, 4, 7, 99] {
            for width in 0..10 {
                let rendered = styled_cells(text, offset, width, &spans);
                assert_eq!(without_colors(&rendered), cells(text, offset, width));
                assert_eq!(without_colors(&rendered).width(), width);
                if rendered.contains('\x1b') {
                    assert!(rendered.trim_end().ends_with("\x1b[39m"));
                }
            }
        }
    }

    #[test]
    fn color_never_preserves_layout_and_content_without_syntax_escapes() {
        let mut state = BrowserState::new(
            Mega2TreeSession::new("http://localhost:9000").unwrap(),
            "/",
            None,
        )
        .unwrap();
        state.entries = vec![("main.rs".into(), ContentType::File)];
        let mut view = Explorer::new(&state).unwrap();
        view.apply_preview(Ok("fn main() { let s = \"中文\"; }".into()));
        for (width, height) in [(60, 12), (100, 30)] {
            view.color_enabled = true;
            let colored = render(&view, &state, "http://localhost:9000", width, height);
            view.color_enabled = false;
            let plain = render(&view, &state, "http://localhost:9000", width, height);
            assert_eq!(without_colors(&colored), plain);
            assert!(!plain.contains("\x1b[3"));
            let body = plain
                .strip_prefix("\x1b[2J\x1b[H")
                .unwrap()
                .strip_suffix("\x1b[0m")
                .unwrap();
            assert!(
                body.lines()
                    .all(|line| line.trim_end_matches('\r').width() < width)
            );
        }
    }

    #[test]
    fn split_layout_fits_terminal_and_shows_file_lines() {
        let mut state = BrowserState::new(
            Mega2TreeSession::new("http://localhost:9000").unwrap(),
            "/",
            None,
        )
        .unwrap();
        state.entries = vec![("文件.txt".into(), ContentType::File)];
        let mut view = Explorer::new(&state).unwrap();
        view.apply_preview(Ok("first line\nsecond\x1b[2J".into()));
        for (width, height) in [(100, 30), (60, 12), (20, 4), (1, 1)] {
            let frame = render(&view, &state, "http://localhost:9000", width, height);
            let body = frame
                .strip_prefix("\x1b[2J\x1b[H")
                .unwrap()
                .strip_suffix("\x1b[0m")
                .unwrap();
            assert!(!body.contains('\x1b'));
            assert!(body.lines().count() <= height.saturating_sub(1));
            assert!(
                body.lines()
                    .all(|line| line.trim_end_matches('\r').width() < width)
            );
            if width >= 60 {
                assert!(body.contains("File content"));
                assert!(body.contains("first line"));
            }
        }
    }
}
