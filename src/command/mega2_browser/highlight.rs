//! Cached syntax spans. Only the renderer emits ANSI, never the remote file.

use std::{
    path::Path,
    sync::LazyLock,
    time::{Duration, Instant},
};

use syntect::{
    easy::HighlightLines,
    highlighting::{Color, StyleModifier, Theme, ThemeItem},
    parsing::SyntaxSet,
};

// Minified/large files remain readable without expensive syntax analysis.
const MAX_HIGHLIGHT_BYTES: usize = 64 * 1024;
const MAX_LINE_BYTES: usize = 4096;
const MAX_SPANS: usize = 32_768;
const HIGHLIGHT_BUDGET: Duration = Duration::from_secs(1);

static SYNTAXES: LazyLock<SyntaxSet> = LazyLock::new(SyntaxSet::load_defaults_newlines);
static THEME: LazyLock<Option<Theme>> = LazyLock::new(|| {
    let mut theme = Theme::default();
    theme.settings.foreground = Some(color(Tone::Plain));
    for (scope, tone) in [
        ("comment", Tone::Comment),
        ("string", Tone::String),
        ("constant", Tone::Constant),
        ("keyword, storage", Tone::Keyword),
        (
            "entity.name, entity.other.attribute-name, support.function",
            Tone::Name,
        ),
        ("markup.heading, markup.bold", Tone::Keyword),
        ("markup.inline.raw, markup.raw.block", Tone::String),
        ("invalid", Tone::Invalid),
    ] {
        theme.scopes.push(ThemeItem {
            scope: scope.parse().ok()?,
            style: StyleModifier {
                foreground: Some(color(tone)),
                ..StyleModifier::default()
            },
        });
    }
    Some(theme)
});

/// Semantic palette mapped to the user's terminal colors, not a fixed background.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum Tone {
    #[default]
    Plain = 0,
    Invalid = 31,
    String = 32,
    Constant = 33,
    Name = 34,
    Keyword = 35,
    Comment = 36,
}

impl Tone {
    pub fn ansi(self) -> &'static str {
        match self {
            Self::Plain => "\x1b[39m",
            Self::Invalid => "\x1b[31m",
            Self::String => "\x1b[32m",
            Self::Constant => "\x1b[33m",
            Self::Name => "\x1b[34m",
            Self::Keyword => "\x1b[35m",
            Self::Comment => "\x1b[36m",
        }
    }
}

// Colors are internal semantic IDs; RGB escape codes are never emitted.
fn color(tone: Tone) -> Color {
    Color {
        r: tone as u8,
        g: 0,
        b: 0,
        a: 255,
    }
}

fn tone(color: Color) -> Tone {
    match color.r {
        31 => Tone::Invalid,
        32 => Tone::String,
        33 => Tone::Constant,
        34 => Tone::Name,
        35 => Tone::Keyword,
        36 => Tone::Comment,
        _ => Tone::Plain,
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Span {
    pub end: usize,
    pub tone: Tone,
}

#[derive(Default)]
pub struct Highlighted {
    pub language: String,
    pub lines: Vec<Vec<Span>>,
}

impl Highlighted {
    fn fallback(reason: &str) -> Self {
        Self {
            language: reason.into(),
            ..Self::default()
        }
    }

    /// Input is already sanitized and tab-expanded. Highlight once per load so
    /// scrolling preserves multiline syntax without re-parsing the file.
    pub fn for_file(path: &str, lines: &[String]) -> Self {
        if lines.iter().any(|line| line.len() > MAX_LINE_BYTES)
            || lines.iter().map(|line| line.len() + 1).sum::<usize>() > MAX_HIGHLIGHT_BYTES
        {
            return Self::fallback("Text: highlighting limit");
        }
        let name = Path::new(path)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("");
        let extension = Path::new(name)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        let alias = match name {
            ".buckconfig" | ".gitconfig" | ".editorconfig" => "ini",
            "BUCK" | "TARGETS" | "WORKSPACE" | "BUILD" | "BUILD.bazel" => "py",
            _ => extension.as_str(),
        };
        if matches!(alias, "toml" | "ini" | "cfg" | "conf" | "properties") {
            let mut scanner = super::config_highlight::ConfigScanner::default();
            let mut highlighted = Self {
                language: "TOML / INI".into(),
                lines: Vec::new(),
            };
            let mut count = 0;
            for line in lines {
                let spans = scanner.line(line);
                count += spans.len();
                if count > MAX_SPANS {
                    return Self::fallback("Text: highlighting limit");
                }
                highlighted.lines.push(spans);
            }
            return highlighted;
        }
        let syntaxes = &*SYNTAXES;
        let Some(theme) = THEME.as_ref() else {
            return Self::fallback("Text: highlighting unavailable");
        };
        // Do not use find_syntax_for_file: this path is remote, not a local file.
        let syntax = syntaxes
            .find_syntax_by_extension(name)
            .or_else(|| syntaxes.find_syntax_by_extension(alias))
            .or_else(|| {
                lines
                    .first()
                    .filter(|line| line.starts_with("#!"))
                    .and_then(|line| syntaxes.find_syntax_by_first_line(line))
            });
        let Some(syntax) = syntax else {
            return Self::fallback("Text");
        };
        let mut parser = HighlightLines::new(syntax, theme);
        let mut highlighted = Self {
            language: syntax.name.clone(),
            lines: Vec::with_capacity(lines.len()),
        };
        let started = Instant::now();
        let mut count = 0;
        for line in lines {
            // Check between lines; each regex invocation also has a backtrack
            // limit and receives at most MAX_LINE_BYTES bytes.
            if started.elapsed() > HIGHLIGHT_BUDGET {
                return Self::fallback("Text: highlighting limit");
            }
            let with_newline = format!("{line}\n");
            let Ok(parts) = parser.highlight_line(&with_newline, syntaxes) else {
                return Self::fallback("Text: highlighting unavailable");
            };
            let mut end = 0;
            let mut spans: Vec<Span> = Vec::new();
            for (style, text) in parts {
                end += text.len();
                let tone = tone(style.foreground);
                if let Some(last) = spans.last_mut().filter(|last| last.tone == tone) {
                    last.end = end.min(line.len());
                } else {
                    spans.push(Span {
                        end: end.min(line.len()),
                        tone,
                    });
                }
            }
            count += spans.len();
            if count > MAX_SPANS {
                return Self::fallback("Text: highlighting limit");
            }
            highlighted.lines.push(spans);
        }
        highlighted
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn highlight(path: &str, text: &str) -> Highlighted {
        Highlighted::for_file(path, &text.lines().map(str::to_string).collect::<Vec<_>>())
    }

    #[test]
    fn common_file_types_and_special_names_have_syntax_colors() {
        for (path, text) in [
            ("/main.rs", "fn main() {}"),
            ("/main.py", "def main(): return True"),
            ("/main.js", "const answer = 42;"),
            ("/data.JSON", "{\"answer\": 42}"),
            ("/config.yaml", "answer: true"),
            ("/README.md", "# Heading"),
            ("/run.sh", "echo \"hello\""),
            ("/main.cpp", "int main() { return 0; }"),
            ("/page.html", "<h1>Hello</h1>"),
            ("/style.css", "body { color: red; }"),
            ("/Cargo.toml", "[package]\nname = \"demo\""),
            ("/.buckconfig", "[cells]\nroot = ."),
            ("/BUCK", "rust_binary(name = \"demo\")"),
            ("/script", "#!/usr/bin/env python\nprint(42)"),
        ] {
            let highlighted = highlight(path, text);
            assert!(
                highlighted
                    .lines
                    .iter()
                    .flatten()
                    .any(|span| span.tone != Tone::Plain),
                "no colors for {path}: {}",
                highlighted.language
            );
        }
    }

    #[test]
    fn multiline_comment_state_is_kept_across_lines() {
        let highlighted = highlight("/main.rs", "/* comment\nfn not_code() {}\n*/\nfn code() {}");
        assert!(
            highlighted.lines[1]
                .iter()
                .all(|span| span.tone == Tone::Comment)
        );
        assert_eq!(highlighted.lines[3][0].tone, Tone::Keyword);
    }

    #[test]
    fn unknown_and_over_budget_files_fall_back_without_losing_text() {
        assert!(
            highlight("/unknown.custom", "fn fake() {}")
                .lines
                .is_empty()
        );
        assert!(
            highlight("/main.rs", &"x".repeat(MAX_LINE_BYTES + 1))
                .lines
                .is_empty()
        );
        assert!(
            highlight(
                "/main.rs",
                &"fn x() {}\n".repeat(MAX_HIGHLIGHT_BYTES / 10 + 1)
            )
            .lines
            .is_empty()
        );
    }
}
