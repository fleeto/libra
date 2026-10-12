//! Linear, bounded TOML/INI lexical coloring; this never parses configuration.

use super::highlight::{Span, Tone};

#[derive(Default)]
pub struct ConfigScanner {
    // Only triple-quoted TOML strings continue across lines.
    quote: Option<(u8, usize)>,
}

fn push(spans: &mut Vec<Span>, end: usize, tone: Tone) {
    if let Some(last) = spans.last_mut().filter(|last| last.tone == tone) {
        last.end = end;
    } else {
        spans.push(Span { end, tone });
    }
}

impl ConfigScanner {
    pub fn line(&mut self, text: &str) -> Vec<Span> {
        let bytes = text.as_bytes();
        let mut spans = Vec::new();
        let mut at = 0;
        if self.quote.is_none() {
            let start = text.len() - text.trim_start().len();
            if bytes.get(start) == Some(&b'[') {
                if let Some(end) = text[start..].find(']') {
                    at = start + end + 1;
                    while bytes.get(at) == Some(&b']') {
                        at += 1;
                    }
                    push(&mut spans, at, Tone::Name);
                }
            } else if !matches!(bytes.get(start), Some(b'#' | b';')) {
                // Ignore equals signs in quoted keys and in comments.
                let mut quote = None;
                let mut escaped = false;
                for (index, ch) in text.char_indices() {
                    if escaped {
                        escaped = false;
                        continue;
                    }
                    if ch == '\\' && quote == Some('"') {
                        escaped = true;
                        continue;
                    }
                    if let Some(delimiter) = quote {
                        if ch == delimiter {
                            quote = None;
                        }
                    } else if matches!(ch, '\'' | '"') {
                        quote = Some(ch);
                    } else if matches!(ch, '#' | ';') {
                        break;
                    } else if ch == '=' {
                        at = index;
                        if at > 0 {
                            push(&mut spans, at, Tone::Name);
                        }
                        break;
                    }
                }
            }
        }
        while at < bytes.len() {
            if let Some((delimiter, count)) = self.quote {
                if bytes[at] == b'\\' && delimiter == b'"' {
                    at += 1;
                    if let Some(ch) = text[at..].chars().next() {
                        at += ch.len_utf8();
                    }
                } else if bytes[at..].starts_with(&[delimiter; 3][..count]) {
                    at += count;
                    self.quote = None;
                } else if let Some(ch) = text[at..].chars().next() {
                    at += ch.len_utf8();
                }
                push(&mut spans, at, Tone::String);
                continue;
            }
            match bytes[at] {
                b'#' | b';' => {
                    push(&mut spans, bytes.len(), Tone::Comment);
                    break;
                }
                delimiter @ (b'\'' | b'"') => {
                    let count = if bytes[at..].starts_with(&[delimiter; 3]) {
                        3
                    } else {
                        1
                    };
                    self.quote = Some((delimiter, count));
                    at += count;
                    push(&mut spans, at, Tone::String);
                }
                b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' => {
                    let start = at;
                    while bytes.get(at).is_some_and(|byte| {
                        byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'.' | b'+' | b'-')
                    }) {
                        at += 1;
                    }
                    let word = &text[start..at];
                    let tone = if bytes[start].is_ascii_digit()
                        || matches!(word, "true" | "false" | "null" | "inf" | "nan")
                    {
                        Tone::Constant
                    } else {
                        Tone::Plain
                    };
                    push(&mut spans, at, tone);
                }
                _ => {
                    if let Some(ch) = text[at..].chars().next() {
                        at += ch.len_utf8();
                    }
                    push(&mut spans, at, Tone::Plain);
                }
            }
        }
        if self.quote.is_some_and(|(_, count)| count == 1) {
            self.quote = None;
        }
        spans
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tone_at(line: &str, spans: &[Span], needle: &str) -> Tone {
        let at = line.find(needle).unwrap();
        spans.iter().find(|span| span.end > at).unwrap().tone
    }

    #[test]
    fn configuration_keys_strings_comments_and_numbers_are_distinct() {
        let mut scanner = ConfigScanner::default();
        for (line, needle, tone) in [
            ("[cells] # heading", "cells", Tone::Name),
            ("\"key=中\" = \"#string\" # comment", "key", Tone::Name),
            ("name = \"#中=文\" # comment", "#中", Tone::String),
            ("name = \"x\" # comment", "# comment", Tone::Comment),
            ("count = 42", "42", Tone::Constant),
            ("enabled = true", "true", Tone::Constant),
            ("# comment = none", "comment", Tone::Comment),
        ] {
            let spans = scanner.line(line);
            assert_eq!(tone_at(line, &spans, needle), tone, "{line}");
        }
    }

    #[test]
    fn triple_quoted_strings_keep_state_and_unclosed_single_line_strings_do_not() {
        let mut scanner = ConfigScanner::default();
        scanner.line("text = \"\"\"first");
        assert!(
            scanner
                .line("# second = 中文")
                .iter()
                .all(|span| span.tone == Tone::String)
        );
        let line = "last\"\"\" # comment";
        let spans = scanner.line(line);
        assert_eq!(tone_at(line, &spans, "# comment"), Tone::Comment);
        scanner.line("bad = \"unclosed");
        assert_eq!(scanner.line("# next")[0].tone, Tone::Comment);
    }
}
