//! Cell-width-aware, bounded terminal rendering. Remote text never emits ANSI.

use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use super::{
    BrowserState,
    dialog::Dialog,
    explorer::Explorer,
    highlight::{Span, Tone},
    move_picker::MovePicker,
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
    let dialog = Dialog::from_state(state);
    let modal_open = dialog.is_some() || state.move_picker.is_some();
    let mut lines = Vec::new();
    if width < 48 || height < 9 {
        lines.push(cells(
            "Window too small: resize to at least 49 x 10.",
            0,
            width,
        ));
        lines.push(cells(
            if modal_open { "Esc cancel" } else { "q quit" },
            0,
            width,
        ));
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
                        if view.color_enabled && !modal_open {
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
        let status = if modal_open {
            "Dialog open — complete or cancel to return to the browser".into()
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
            "a new dir | d delete | m move | e rename | r reload | t tags".into()
        };
        lines.push(cells(&status, 0, width));
        let help = if modal_open {
            "Enter confirm | Esc cancel"
        } else {
            "Tab pane | arrows move | Enter open/fold | q quit"
        };
        lines.push(cells(help, 0, width));
    }
    lines.truncate(height);
    if width >= 48 && height >= 9 {
        if let Some(picker) = state.move_picker.as_ref() {
            overlay_move_picker(&mut lines, picker, width, height);
        } else if let Some(dialog) = dialog {
            overlay_dialog(&mut lines, &dialog, width, height);
        }
    }
    format!("\x1b[2J\x1b[H{}\x1b[0m", lines.join("\r\n"))
}

fn overlay_dialog(lines: &mut Vec<String>, dialog: &Dialog<'_>, width: usize, height: usize) {
    let box_width = width.saturating_sub(4).min(72);
    let inner = box_width - 4;
    let row = |text: &str| format!("│ {} │", cells(text, 0, inner));
    let input = match dialog.input {
        Some(input) => {
            let text = format!("{}▏", safe_text(input));
            let available = inner - 2;
            let offset = text.width().saturating_sub(available);
            format!("> {}", cells(&text, offset, available))
        }
        None => "This action cannot be undone.".into(),
    };
    let error = dialog
        .error
        .map_or(String::new(), |error| format!("Error: {error}"));
    let title = format!(" {} ", dialog.title);
    let popup = [
        format!("┌{}{}┐", title, "─".repeat(box_width - 2 - title.width())),
        row(&dialog.context),
        row(""),
        row(dialog.label),
        row(&input),
        row(&cells(&error, 0, inner)),
        row(&cells(&error, inner, inner)),
        row(dialog.help),
        format!("└{}┘", "─".repeat(box_width - 2)),
    ];
    let left = (width - box_width) / 2;
    let top = (height - popup.len()) / 2;
    lines.resize(height, " ".repeat(width));
    for (index, popup_line) in popup.iter().enumerate() {
        let background = &lines[top + index];
        // Background syntax colors are disabled while a dialog is open, so
        // slicing uses only terminal cells, never ANSI bytes or UTF-8 offsets.
        lines[top + index] = format!(
            "{}{popup_line}{}",
            cells(background, 0, left),
            cells(background, left + box_width, width - left - box_width),
        );
    }
}

/// Renders the move picker separately from text editors.  `Right` is the only
/// key that opens a destination directory; `Enter` selects it and then needs a
/// second explicit confirmation before a write can be planned.
fn overlay_move_picker(lines: &mut Vec<String>, picker: &MovePicker, width: usize, height: usize) {
    let box_width = width.saturating_sub(4).min(72);
    let inner = box_width - 4;
    let row = |text: &str| format!("│ {} │", cells(text, 0, inner));
    let title = match picker.kind {
        ContentType::Directory => " Move directory ",
        ContentType::File => " Move file ",
    };
    let title_row = format!("┌{}{}┐", title, "─".repeat(box_width - 2 - title.width()));
    let error = picker
        .error
        .as_deref()
        .map_or_else(String::new, |error| format!("Error: {error}"));
    let mut popup = vec![title_row, row(&format!("Source: {}", picker.source_path()))];
    if let Some(destination) = picker.confirmed_parent.as_deref() {
        popup.push(row(&format!("Destination: {destination}")));
        popup.push(row(""));
        popup.push(row("Enter move | Esc back"));
    } else {
        popup.push(row(&format!(
            "Destination within {}: {}",
            picker.destination_root, picker.folder
        )));
        // Six structural rows leave the remaining height for the visible
        // directory choices.  The selected row is always retained.
        let visible = height.saturating_sub(6).clamp(1, 7);
        let choices = picker.directories.len() + 1;
        let start = picker
            .selection
            .saturating_sub(visible.saturating_sub(1))
            .min(choices.saturating_sub(visible));
        for index in start..(start + visible).min(choices) {
            let choice = if index == 0 {
                format!("[.] {}", picker.folder)
            } else {
                format!("[+] {}/", picker.directories[index - 1])
            };
            popup.push(row(&format!(
                "{} {choice}",
                if index == picker.selection { ">" } else { " " }
            )));
        }
        popup.push(row(
            "Enter choose | Right open | r reload | Left parent | Esc cancel",
        ));
    }
    popup.push(row(&error));
    popup.push(format!("└{}┘", "─".repeat(box_width - 2)));

    let left = (width - box_width) / 2;
    let top = (height - popup.len()) / 2;
    lines.resize(height, " ".repeat(width));
    for (index, popup_line) in popup.iter().enumerate() {
        let background = &lines[top + index];
        lines[top + index] = format!(
            "{}{popup_line}{}",
            cells(background, 0, left),
            cells(background, left + box_width, width - left - box_width),
        );
    }
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
    use crate::{command::mega2_browser::Editor, internal::protocol::mega2_tree::Mega2TreeSession};

    #[test]
    fn cells_handle_unicode_and_never_emit_terminal_controls() {
        assert_eq!(cells("中文x", 0, 3), "中 ");
        assert_eq!(cells("中文x", 2, 3), "文x");
        assert_eq!(cells("a\x1b[2J\u{202e}", 0, 9), "a?[2J?   ");
    }

    #[test]
    fn editors_render_centered_dialogs_instead_of_footer_inputs() {
        use super::super::tag_panel::{TagEditor, TagPanel};

        let mut state = BrowserState::new(
            Mega2TreeSession::new("http://localhost:9000").unwrap(),
            "/project",
            None,
        )
        .unwrap();
        let view = Explorer::new(&state).unwrap();
        for (editor, title, label) in [
            (
                Editor::CreateDir("new-name".into()),
                "New directory",
                "Directory name",
            ),
            (
                Editor::RenameDir {
                    name: "src".into(),
                    input: "new-name".into(),
                },
                "Rename directory",
                "New name",
            ),
            (
                Editor::ConfirmDelete("src".into()),
                "Delete directory",
                "Confirm deletion",
            ),
        ] {
            state.editor = Some(editor);
            assert_centered_dialog(&render(&view, &state, "server", 100, 30), title, label);
        }
        state.editor = None;
        for (editor, title, label) in [
            (TagEditor::CreateName("v1.0".into()), "New tag", "Tag name"),
            (
                TagEditor::CreateMessage {
                    name: "v1.0".into(),
                    message: "release".into(),
                },
                "New tag",
                "Message (empty = lightweight)",
            ),
            (
                TagEditor::ConfirmDelete("v1.0".into()),
                "Delete tag",
                "Confirm deletion",
            ),
        ] {
            let mut panel = TagPanel::new();
            panel.editor = Some(editor);
            state.panel = Some(panel);
            assert_centered_dialog(&render(&view, &state, "server", 100, 30), title, label);
        }
    }

    #[test]
    fn move_picker_renders_explicit_choose_then_confirm_controls() {
        use super::super::move_picker::MovePicker;

        let mut state = BrowserState::new(
            Mega2TreeSession::new("http://localhost:9000").unwrap(),
            "/artifact",
            None,
        )
        .unwrap();
        let mut picker =
            MovePicker::new("/artifact".into(), "readme.txt".into(), ContentType::File);
        picker.directories = vec!["destination".into()];
        picker.selection = 1;
        picker.apply_listing(
            "/artifact".into(),
            crate::internal::protocol::mega2_tree::Listing {
                entries: vec![crate::internal::protocol::mega2_tree::ListingEntry {
                    name: "destination".into(),
                    content_type: ContentType::Directory,
                }],
            },
        );
        picker.selection = 1;
        state.move_picker = Some(picker);
        let view = Explorer::new(&state).unwrap();
        let choose = render(&view, &state, "server", 100, 30);
        assert!(choose.contains("Move file"), "{choose}");
        assert!(choose.contains("Source: /artifact/readme.txt"), "{choose}");
        assert!(choose.contains("Enter choose | Right open"), "{choose}");

        state.move_picker.as_mut().expect("picker").confirmed_parent =
            Some("/artifact/destination".into());
        let confirm = render(&view, &state, "server", 100, 30);
        assert!(
            confirm.contains("Destination: /artifact/destination"),
            "{confirm}"
        );
        assert!(confirm.contains("Enter move | Esc back"), "{confirm}");
    }

    fn assert_centered_dialog(frame: &str, title: &str, label: &str) {
        let lines: Vec<_> = frame.lines().collect();
        let top = lines
            .iter()
            .position(|line| line.contains('┌'))
            .expect("popup border");
        let bottom = lines
            .iter()
            .position(|line| line.contains('└') && line.contains('┘'))
            .unwrap();
        assert_eq!(top, (29 - (bottom - top + 1)) / 2);
        let line = lines[top].trim_end_matches('\r');
        let start = line.find('┌').unwrap();
        let end = line.find('┐').unwrap() + '┐'.len_utf8();
        assert_eq!(line[..start].width(), (99 - line[start..end].width()) / 2);
        let popup = lines[top..=bottom].join("\n");
        assert!(popup.contains(title));
        assert!(popup.contains(label));
        assert!(popup.contains("Esc cancel"));
        assert!(!lines[bottom + 1..].join("\n").contains("new-name"));
    }

    #[test]
    fn dialog_keeps_input_tail_and_validation_visible_within_terminal_bounds() {
        let mut state = BrowserState::new(
            Mega2TreeSession::new("http://localhost:9000").unwrap(),
            "/",
            None,
        )
        .unwrap();
        state.editor = Some(Editor::CreateDir(format!("{}TAIL", "中".repeat(100))));
        state.status = Some("Invalid name: remove the slash\x1b[2J".into());
        let mut view = Explorer::new(&state).unwrap();
        view.color_enabled = false;
        for (width, height) in [(100, 30), (60, 12), (49, 10), (20, 4), (1, 1)] {
            let frame = render(&view, &state, "server", width, height);
            let body = frame
                .strip_prefix("\x1b[2J\x1b[H")
                .unwrap()
                .strip_suffix("\x1b[0m")
                .unwrap();
            assert!(!body.contains('\x1b'));
            assert!(body.lines().count() < height);
            assert!(
                body.lines()
                    .all(|line| line.trim_end_matches('\r').width() < width)
            );
            if width >= 49 {
                assert!(body.contains("TAIL▏"));
                assert!(body.contains("Invalid name"));
                assert!(body.contains("Esc cancel"));
            }
        }
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
