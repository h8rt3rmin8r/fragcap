// SPDX-License-Identifier: Apache-2.0

//! JSON string escaping and hex encoding.
//!
//! The other place this format goes wrong invisibly. An unescaped control
//! character produces a line that no parser accepts, and an unescaped quote
//! produces a line that parses into the wrong thing, which is worse.
//!
//! Escaping is the strongest argument for using a serialization library, and
//! the answer here is not that it was judged unnecessary: every string this
//! module escapes is fed back through `serde_json` in the tests below. The
//! hand-rolled writer is verified by a real parser rather than by its own
//! reader, which is the arrangement S06 wanted for pcapng and could not have.

/// Append `value` to `out` as a quoted, escaped JSON string.
///
/// Escapes the two structural characters and every C0 control, using the short
/// forms JSON defines where they exist and `\uXXXX` where they do not.
/// Characters above 0x7F are emitted as UTF-8: JSON permits them, every parser
/// handles them, and escaping them would mean encoding surrogate pairs by hand
/// for no benefit to any consumer.
pub(crate) fn write_json_string(value: &str, out: &mut String) {
    out.push('"');
    for c in value.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{08}' => out.push_str("\\b"),
            '\u{0C}' => out.push_str("\\f"),
            c if (c as u32) < 0x20 => {
                out.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

/// Append `bytes` to `out` as a quoted lowercase hex string.
///
/// Lowercase per the section 13.5 example. The pcapng annotation
/// percent-encodes in uppercase, following that encoding's convention; the two
/// differ because each follows its own format, and both are fixed so goldens
/// are stable.
pub(crate) fn write_hex_string(bytes: &[u8], out: &mut String) {
    out.push('"');
    for b in bytes {
        out.push_str(&format!("{b:02x}"));
    }
    out.push('"');
}

#[cfg(test)]
mod tests {
    use super::*;

    fn esc(s: &str) -> String {
        let mut out = String::new();
        write_json_string(s, &mut out);
        out
    }

    fn hex(b: &[u8]) -> String {
        let mut out = String::new();
        write_hex_string(b, &mut out);
        out
    }

    #[test]
    fn ordinary_text_is_quoted_and_otherwise_untouched() {
        assert_eq!(esc("eso64.exe"), "\"eso64.exe\"");
    }

    #[test]
    fn structural_characters_are_escaped() {
        assert_eq!(esc("say \"hi\""), r#""say \"hi\"""#);
        assert_eq!(esc(r"C:\Games"), r#""C:\\Games""#);
    }

    #[test]
    fn controls_with_short_forms_use_them() {
        assert_eq!(esc("a\nb"), r#""a\nb""#);
        assert_eq!(esc("a\rb"), r#""a\rb""#);
        assert_eq!(esc("a\tb"), r#""a\tb""#);
        assert_eq!(esc("a\u{08}b"), r#""a\bb""#);
        assert_eq!(esc("a\u{0C}b"), r#""a\fb""#);
    }

    #[test]
    fn controls_without_short_forms_use_the_u_form() {
        assert_eq!(esc("a\u{0}b"), r#""a\u0000b""#);
        assert_eq!(esc("a\u{1}b"), r#""a\u0001b""#);
        assert_eq!(esc("a\u{1f}b"), r#""a\u001fb""#);
    }

    #[test]
    fn delete_is_not_escaped() {
        // 0x7F is not a C0 control and JSON does not require escaping it. The
        // pcapng annotation does escape it, because it breaks that grammar's
        // containing format rather than this one's.
        assert_eq!(esc("a\u{7f}b"), "\"a\u{7f}b\"");
    }

    #[test]
    fn characters_above_ascii_are_emitted_as_utf8() {
        assert_eq!(
            esc("\u{30b2}\u{30fc}\u{30e0}"),
            "\"\u{30b2}\u{30fc}\u{30e0}\""
        );
    }

    #[test]
    fn an_empty_string_is_a_pair_of_quotes() {
        assert_eq!(esc(""), "\"\"");
    }

    #[test]
    fn hex_is_lowercase_and_two_digits_per_byte() {
        assert_eq!(hex(&[0x3f, 0x8a, 0x01]), "\"3f8a01\"");
        assert_eq!(hex(&[0x00, 0xff]), "\"00ff\"");
    }

    #[test]
    fn an_empty_payload_is_an_empty_string() {
        // Distinct from payload-free mode, which omits the key entirely.
        assert_eq!(hex(&[]), "\"\"");
    }

    /// The external oracle. A hand-rolled escaper that only satisfies
    /// hand-rolled expectations has proven that two functions agree.
    #[test]
    fn every_escaped_string_round_trips_through_a_real_parser() {
        let cases = [
            "eso64.exe",
            "say \"hi\"",
            r"C:\Games\eso.exe",
            "a\nb\r\nc\td",
            "\u{0}\u{1}\u{1f}",
            "\u{08}\u{0C}",
            "\u{30b2}\u{30fc}\u{30e0}.exe",
            "\u{1F600}",
            "",
            "\u{7f}",
            "mixed \"quotes\" and \\backslashes\\ and \u{0} nulls",
        ];
        for case in cases {
            let escaped = esc(case);
            let parsed: String = serde_json::from_str(&escaped)
                .unwrap_or_else(|e| panic!("{escaped} must parse as a JSON string: {e}"));
            assert_eq!(parsed, case, "round trip changed {case:?}");
        }
    }

    #[test]
    fn every_control_character_round_trips() {
        for cp in 0u32..0x20 {
            let s = format!("a{}b", char::from_u32(cp).unwrap());
            let escaped = esc(&s);
            let parsed: String = serde_json::from_str(&escaped)
                .unwrap_or_else(|e| panic!("U+{cp:04X} produced unparseable output: {e}"));
            assert_eq!(parsed, s);
        }
    }
}
