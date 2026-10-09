// SPDX-License-Identifier: Apache-2.0

//! Shared terminal display-cell accounting for CLI tables and reports.

use unicode_width::UnicodeWidthStr;

/// Return the visible terminal-cell width of `value`.
pub(crate) fn display_width(value: &str) -> usize {
    let mut chars = value.chars().peekable();
    let mut plain = String::with_capacity(value.len());
    while let Some(c) = chars.next() {
        if c == '\u{1b}' && chars.peek() == Some(&'[') {
            chars.next();
            for escaped in chars.by_ref() {
                if ('@'..='~').contains(&escaped) {
                    break;
                }
            }
        } else if c == '\u{1b}' && chars.peek() == Some(&']') {
            chars.next();
            while let Some(escaped) = chars.next() {
                if escaped == '\u{7}' || (escaped == '\u{1b}' && chars.peek() == Some(&'\\')) {
                    if escaped == '\u{1b}' {
                        chars.next();
                    }
                    break;
                }
            }
        } else if !c.is_control() {
            plain.push(c);
        }
    }
    plain.width()
}

/// One measured report block. Every adjacent pair has exactly four cells of gap.
pub(crate) struct ColumnLayout {
    indent: usize,
    widths: Vec<usize>,
}

impl ColumnLayout {
    pub(crate) fn new(indent: usize, rows: &[Vec<String>]) -> Self {
        let count = rows.iter().map(Vec::len).max().unwrap_or(0);
        let mut widths = vec![0; count];
        for row in rows {
            for (column, cell) in row.iter().enumerate() {
                widths[column] = widths[column].max(display_width(cell));
            }
        }
        Self { indent, widths }
    }

    pub(crate) fn anchor(&self, column: usize) -> usize {
        self.indent + self.widths.iter().take(column).sum::<usize>() + column * 4
    }

    pub(crate) fn render_row(&self, cells: &[String]) -> String {
        self.render_wrapped_row(cells, usize::MAX)
    }

    pub(crate) fn render_wrapped_row(&self, cells: &[String], width: usize) -> String {
        let end = cells
            .iter()
            .rposition(|cell| !cell.is_empty())
            .map_or(0, |index| index + 1);
        let cells = &cells[..end];
        let mut out = " ".repeat(self.indent);
        for (column, cell) in cells.iter().enumerate() {
            if column > 0 {
                out.push_str("    ");
            }
            if column + 1 == cells.len() {
                out.push_str(&wrap_hanging(cell, self.anchor(column), width));
            } else {
                out.push_str(&pad_display(cell, self.widths[column]));
            }
        }
        out
    }
}

/// Render exact human key/value rows using the widest emitted key.
pub(crate) fn render_fields(indent: usize, fields: &[(String, String)], width: usize) -> String {
    let rows: Vec<Vec<String>> = fields
        .iter()
        .map(|(key, value)| vec![human_display_value(key), human_display_value(value)])
        .collect();
    let layout = ColumnLayout::new(indent, &rows);
    rows.iter()
        .map(|row| format!("{}\n", layout.render_wrapped_row(row, width)))
        .collect()
}

/// Pad `value` with ASCII spaces to the requested visible width.
pub(crate) fn pad_display(value: &str, width: usize) -> String {
    let mut padded = String::from(value);
    for _ in display_width(value)..width {
        padded.push(' ');
    }
    padded
}

/// Represent controls that would otherwise become terminal layout syntax.
pub(crate) fn human_display_value(value: &str) -> String {
    let mut displayed = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '\t' => displayed.push_str("\\t"),
            '\r' => displayed.push_str("\\r"),
            '\n' => displayed.push_str("\\n"),
            c if c.is_control() => displayed.push_str(&format!("\\u{{{:x}}}", c as u32)),
            _ => displayed.push(character),
        }
    }
    displayed
}

/// Select the bounded width used by human stdout reports.
pub(crate) fn selected_stdout_width(stdout_terminal: bool) -> usize {
    let reported = stdout_terminal.then(reported_stdout_width).flatten();
    human_width(stdout_terminal, reported)
}

/// Select the bounded width of human diagnostics independently of stdout.
pub(crate) fn selected_stderr_width(stderr_terminal: bool) -> usize {
    #[cfg(windows)]
    let reported = stderr_terminal
        .then(|| terminal_size::terminal_size_of(std::io::stderr()))
        .flatten()
        .map(|(width, _)| usize::from(width.0));
    #[cfg(not(windows))]
    let reported = None;
    human_width(stderr_terminal, reported)
}

#[cfg(windows)]
fn reported_stdout_width() -> Option<usize> {
    terminal_size::terminal_size_of(std::io::stdout()).map(|(width, _)| usize::from(width.0))
}

#[cfg(not(windows))]
fn reported_stdout_width() -> Option<usize> {
    // fragcap is a Windows product. Keep unsupported-host builds deterministic
    // without broadening the Windows-only terminal_size dependency.
    None
}

fn human_width(stdout_terminal: bool, reported: Option<usize>) -> usize {
    if stdout_terminal {
        reported.unwrap_or(80).clamp(40, 80)
    } else {
        80
    }
}

/// Wrap on display-cell word boundaries, preserving indivisible exact tokens.
pub(crate) fn wrap_hanging(text: &str, indent: usize, width: usize) -> String {
    let avail = width.saturating_sub(indent).max(1);
    let mut lines: Vec<String> = Vec::new();
    let mut cur = String::new();
    for word in text.split_inclusive(' ') {
        if cur.is_empty()
            || display_width(&cur) + display_width(word) <= avail
            || word.trim().is_empty()
        {
            cur.push_str(word);
        } else {
            let trailing = cur.len() - cur.trim_end_matches(' ').len();
            let line = cur.trim_end_matches(' ').to_string();
            lines.push(line);
            cur.clear();
            cur.push_str(&" ".repeat(trailing.saturating_sub(1)));
            cur.push_str(word);
        }
    }
    if !cur.is_empty() {
        lines.push(cur);
    }
    let indent_str = " ".repeat(indent);
    lines.join(&format!("\n{indent_str}"))
}

#[cfg(test)]
mod tests {
    use super::{display_width, human_display_value, human_width, pad_display};

    #[test]
    fn counts_terminal_cells_for_supported_character_classes() {
        assert_eq!(display_width("\u{1b}[31m界\u{1b}[0m"), 2);
        assert_eq!(
            display_width("\u{1b}]8;;https://example.test\u{7}界\u{1b}]8;;\u{7}"),
            2
        );
        assert_eq!(display_width("ascii"), 5);
        assert_eq!(display_width("e\u{301}"), 1);
        assert_eq!(display_width("\u{2764}\u{fe0f}"), 2);
        assert_eq!(display_width("界"), 2);
        assert_eq!(display_width("Ａ"), 2);
        assert_eq!(display_width("🎮"), 2);
        assert_eq!(display_width("א\u{5b0}"), 1);
        assert_eq!(display_width("👩\u{200d}💻"), 2);
        assert_eq!(display_width("🇺🇸"), 2);
        assert_eq!(pad_display("界", 4), "界  ");
    }

    #[test]
    fn wrapping_preserves_repeated_spaces() {
        let value = "exact  words   here";
        assert_eq!(super::wrap_hanging(value, 0, 80), value);
    }

    #[test]
    fn empty_values_and_wrapping_emit_no_padding_at_line_ends() {
        let fields = vec![
            ("short:".into(), "".into()),
            (
                "optional [100]:".into(),
                "several separate words stay complete".into(),
            ),
        ];
        let rendered = super::render_fields(2, &fields, 32);
        assert!(rendered.starts_with("  short:\n"));
        assert!(rendered.lines().all(|line| line == line.trim_end()));
        for word in ["several", "separate", "words", "stay", "complete"] {
            assert!(rendered.contains(word));
        }
        for line in rendered.lines().skip(2) {
            assert!(line.starts_with(&" ".repeat(2 + 15 + 4)));
        }
    }

    #[test]
    fn measures_optional_keys_and_every_column_in_styled_unicode_rows() {
        let rows = vec![
            vec!["key:".into(), "界".into(), "last".into()],
            vec![
                "optional [100]:".into(),
                "\u{1b}[32me\u{301}\u{1b}[0m".into(),
                "value".into(),
            ],
        ];
        let layout = super::ColumnLayout::new(8, &rows);
        assert_eq!(layout.anchor(1), 8 + 15 + 4);
        assert_eq!(layout.anchor(2), 8 + 15 + 4 + 2 + 4);
        for row in &rows {
            let line = layout.render_row(row);
            let last = line.find(row[2].as_str()).unwrap();
            assert_eq!(display_width(&line[..last]), layout.anchor(2));
        }
        let wrapped = layout.render_wrapped_row(
            &["key:".into(), "界".into(), "several words here".into()],
            40,
        );
        for line in wrapped.lines().skip(1) {
            assert!(line.starts_with(&" ".repeat(layout.anchor(2))));
        }
    }

    #[test]
    fn represents_layout_controls_without_changing_other_characters() {
        assert_eq!(
            human_display_value("界\tline\r\ne\u{301}"),
            "界\\tline\\r\\ne\u{301}"
        );
    }

    #[test]
    fn bounds_interactive_width_and_defaults_other_output() {
        assert_eq!(human_width(true, Some(20)), 40);
        assert_eq!(human_width(true, Some(63)), 63);
        assert_eq!(human_width(true, Some(120)), 80);
        assert_eq!(human_width(true, None), 80);
        assert_eq!(human_width(false, Some(50)), 80);
        assert_eq!(human_width(false, None), 80);
    }
}
