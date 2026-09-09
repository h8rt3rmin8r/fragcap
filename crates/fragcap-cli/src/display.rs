// SPDX-License-Identifier: Apache-2.0

//! Shared terminal display-cell accounting for CLI tables and reports.

/// Return the visible terminal-cell width of `value`.
pub(crate) fn display_width(value: &str) -> usize {
    value.chars().map(display_cell_width).sum()
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

fn display_cell_width(c: char) -> usize {
    let u = c as u32;
    if c.is_control()
        || matches!(
            u,
            0x0300..=0x036F
                | 0x1AB0..=0x1AFF
                | 0x1DC0..=0x1DFF
                | 0x200C..=0x200D
                | 0x20D0..=0x20FF
                | 0xFE00..=0xFE0F
                | 0xFE20..=0xFE2F
        )
    {
        0
    } else if matches!(
        u,
        0x1100..=0x115F
            | 0x231A..=0x231B
            | 0x2329..=0x232A
            | 0x23E9..=0x23EC
            | 0x23F0
            | 0x23F3
            | 0x25FD..=0x25FE
            | 0x2614..=0x2615
            | 0x2648..=0x2653
            | 0x267F
            | 0x2693
            | 0x26A1
            | 0x26AA..=0x26AB
            | 0x26BD..=0x26BE
            | 0x26C4..=0x26C5
            | 0x26CE
            | 0x26D4
            | 0x26EA
            | 0x26F2..=0x26F3
            | 0x26F5
            | 0x26FA
            | 0x26FD
            | 0x2705
            | 0x270A..=0x270B
            | 0x2728
            | 0x274C
            | 0x274E
            | 0x2753..=0x2755
            | 0x2757
            | 0x2764
            | 0x2795..=0x2797
            | 0x27B0
            | 0x27BF
            | 0x2B1B..=0x2B1C
            | 0x2B50
            | 0x2B55
            | 0x2E80..=0xA4CF
            | 0xAC00..=0xD7A3
            | 0xF900..=0xFAFF
            | 0xFE10..=0xFE19
            | 0xFE30..=0xFE6F
            | 0xFF00..=0xFF60
            | 0xFFE0..=0xFFE6
            | 0x1F300..=0x1FAFF
            | 0x20000..=0x3FFFD
    ) {
        2
    } else {
        1
    }
}

#[cfg(test)]
mod tests {
    use super::{display_width, human_display_value, human_width, pad_display};

    #[test]
    fn counts_terminal_cells_for_supported_character_classes() {
        assert_eq!(display_width("ascii"), 5);
        assert_eq!(display_width("e\u{301}"), 1);
        assert_eq!(display_width("\u{2764}\u{fe0f}"), 2);
        assert_eq!(display_width("界"), 2);
        assert_eq!(display_width("Ａ"), 2);
        assert_eq!(display_width("🎮"), 2);
        assert_eq!(pad_display("界", 4), "界  ");
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
