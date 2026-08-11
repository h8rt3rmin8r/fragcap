// SPDX-License-Identifier: Apache-2.0

//! A small hand-rolled parser for the Valve key-value text format (VDF).
//!
//! Steam records library locations in `libraryfolders.vdf` and per-title
//! metadata in `appmanifest_<app_id>.acf`, both in this format. Specification
//! section 16.2 settles that the format is small and stable and not worth a
//! dependency; this module is that decision (S17 D3). It covers the subset those
//! two manifest kinds use: quoted or bare keys, quoted-or-bare string values,
//! nested `{ ... }` blocks, `//` line comments, and `\\`/`\"` escapes inside
//! quoted strings.
//!
//! Malformed input returns a positioned [`VdfError`] rather than a panic or a
//! silent mis-parse, so a discovery caller can report the offending file and
//! skip it while the well-formed files survive (FR-004).

use std::fmt;

/// A parsed VDF value: either a string leaf or an ordered set of members.
///
/// Members keep declaration order and permit duplicate keys, because the format
/// does; [`VdfValue::get`] returns the first match.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VdfValue {
    /// A string leaf.
    Str(String),
    /// A key-value block. Order preserved; keys may repeat.
    Obj(Vec<(String, VdfValue)>),
}

impl VdfValue {
    /// This value as a string, if it is a leaf.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            VdfValue::Str(s) => Some(s),
            VdfValue::Obj(_) => None,
        }
    }

    /// The members of this value, if it is a block.
    pub fn entries(&self) -> Option<&[(String, VdfValue)]> {
        match self {
            VdfValue::Obj(v) => Some(v),
            VdfValue::Str(_) => None,
        }
    }

    /// The first member whose key matches `key`, case-insensitively.
    ///
    /// Case-insensitive because Valve treats these keys that way, and a manifest
    /// written `AppState` and one written `appstate` name the same thing.
    pub fn get(&self, key: &str) -> Option<&VdfValue> {
        match self {
            VdfValue::Obj(members) => members
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case(key))
                .map(|(_, v)| v),
            VdfValue::Str(_) => None,
        }
    }
}

/// A parse failure, carrying the byte position it was found at.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VdfError {
    /// Byte offset into the input where the fault was found.
    pub position: usize,
    /// What was wrong.
    pub detail: String,
}

impl fmt::Display for VdfError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "VDF parse error at byte {}: {}",
            self.position, self.detail
        )
    }
}

impl std::error::Error for VdfError {}

/// Parse a VDF document into its top-level block.
///
/// The two manifest kinds each have a single top-level `"key" { ... }` pair, but
/// the parser accepts any number of top-level pairs and returns them as one
/// [`VdfValue::Obj`].
pub fn parse(input: &str) -> Result<VdfValue, VdfError> {
    let mut p = Parser {
        chars: input.char_indices().collect(),
        i: 0,
        end: input.len(),
    };
    let members = p.members(true)?;
    p.skip_trivia();
    if let Some(c) = p.peek() {
        return Err(p.err(format!("unexpected `{c}` at top level")));
    }
    Ok(VdfValue::Obj(members))
}

struct Parser {
    chars: Vec<(usize, char)>,
    i: usize,
    end: usize,
}

impl Parser {
    fn peek(&self) -> Option<char> {
        self.chars.get(self.i).map(|(_, c)| *c)
    }

    /// The byte offset of the cursor, or the input length at end of input.
    fn pos(&self) -> usize {
        self.chars.get(self.i).map(|(b, _)| *b).unwrap_or(self.end)
    }

    fn bump(&mut self) -> Option<char> {
        let c = self.peek();
        if c.is_some() {
            self.i += 1;
        }
        c
    }

    fn err(&self, detail: String) -> VdfError {
        VdfError {
            position: self.pos(),
            detail,
        }
    }

    /// Skip whitespace and `//` line comments.
    fn skip_trivia(&mut self) {
        loop {
            match self.peek() {
                Some(c) if c.is_whitespace() => {
                    self.bump();
                }
                Some('/') if self.chars.get(self.i + 1).map(|(_, c)| *c) == Some('/') => {
                    while let Some(c) = self.peek() {
                        self.bump();
                        if c == '\n' {
                            break;
                        }
                    }
                }
                _ => break,
            }
        }
    }

    /// Read the members of a block: pairs until `}` (not consumed) or, at the top
    /// level, end of input.
    fn members(&mut self, top: bool) -> Result<Vec<(String, VdfValue)>, VdfError> {
        let mut out = Vec::new();
        loop {
            self.skip_trivia();
            match self.peek() {
                None => {
                    if top {
                        return Ok(out);
                    }
                    return Err(self.err("unterminated block: expected `}`".to_string()));
                }
                Some('}') => return Ok(out),
                Some(_) => {
                    let key = self.string()?;
                    let value = self.value()?;
                    out.push((key, value));
                }
            }
        }
    }

    /// Read a value: a nested block or a string.
    fn value(&mut self) -> Result<VdfValue, VdfError> {
        self.skip_trivia();
        match self.peek() {
            Some('{') => {
                self.bump();
                let members = self.members(false)?;
                match self.peek() {
                    Some('}') => {
                        self.bump();
                        Ok(VdfValue::Obj(members))
                    }
                    _ => Err(self.err("expected `}` to close block".to_string())),
                }
            }
            None => Err(self.err("expected a value, found end of input".to_string())),
            Some('}') => Err(self.err("expected a value, found `}`".to_string())),
            _ => Ok(VdfValue::Str(self.string()?)),
        }
    }

    /// Read a string token: quoted (with `\\` and `\"` escapes) or bare.
    fn string(&mut self) -> Result<String, VdfError> {
        self.skip_trivia();
        match self.peek() {
            Some('"') => {
                self.bump();
                let mut s = String::new();
                loop {
                    match self.bump() {
                        None => {
                            return Err(self.err("unterminated quoted string".to_string()));
                        }
                        Some('"') => return Ok(s),
                        Some('\\') => match self.bump() {
                            None => {
                                return Err(
                                    self.err("unterminated escape in quoted string".to_string())
                                );
                            }
                            Some('n') => s.push('\n'),
                            Some('t') => s.push('\t'),
                            Some(other) => s.push(other),
                        },
                        Some(c) => s.push(c),
                    }
                }
            }
            Some('{') | Some('}') | None => Err(self.err("expected a key or value".to_string())),
            _ => {
                // A bare token: up to the next whitespace, brace, or quote.
                let mut s = String::new();
                while let Some(c) = self.peek() {
                    if c.is_whitespace() || c == '{' || c == '}' || c == '"' {
                        break;
                    }
                    s.push(c);
                    self.bump();
                }
                Ok(s)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_flat_block() {
        let v = parse("\"AppState\" { \"appid\" \"900883\" \"name\" \"ESO\" }").unwrap();
        let app = v.get("appstate").unwrap();
        assert_eq!(app.get("appid").and_then(|x| x.as_str()), Some("900883"));
        assert_eq!(app.get("name").and_then(|x| x.as_str()), Some("ESO"));
    }

    #[test]
    fn parses_nested_blocks() {
        let text = r#"
            "libraryfolders"
            {
                "0"  { "path" "C:\\Steam" }
                "1"  { "path" "D:\\Games\\Steam" }
            }
        "#;
        let v = parse(text).unwrap();
        let libs = v.get("libraryfolders").unwrap();
        assert_eq!(
            libs.get("0").unwrap().get("path").and_then(|x| x.as_str()),
            Some(r"C:\Steam")
        );
        assert_eq!(
            libs.get("1").unwrap().get("path").and_then(|x| x.as_str()),
            Some(r"D:\Games\Steam")
        );
    }

    #[test]
    fn handles_line_comments() {
        let text = "\"root\" {\n // a comment\n \"k\" \"v\" // trailing\n }";
        let v = parse(text).unwrap();
        assert_eq!(
            v.get("root").unwrap().get("k").and_then(|x| x.as_str()),
            Some("v")
        );
    }

    #[test]
    fn handles_escapes() {
        let v = parse(r#""root" { "p" "a\\b\"c" }"#).unwrap();
        assert_eq!(
            v.get("root").unwrap().get("p").and_then(|x| x.as_str()),
            Some("a\\b\"c")
        );
    }

    #[test]
    fn preserves_order_and_duplicates() {
        let v = parse(r#""r" { "a" "1" "a" "2" }"#).unwrap();
        let r = v.get("r").unwrap();
        // get() returns the first; entries() sees both.
        assert_eq!(r.get("a").and_then(|x| x.as_str()), Some("1"));
        assert_eq!(r.entries().unwrap().len(), 2);
    }

    #[test]
    fn unterminated_block_is_a_positioned_error() {
        let err = parse("\"r\" { \"k\" \"v\"").unwrap_err();
        assert!(
            err.detail.contains("unterminated"),
            "detail: {}",
            err.detail
        );
        assert!(err.position > 0);
    }

    #[test]
    fn unterminated_string_is_a_positioned_error() {
        let err = parse("\"r\" { \"k\" \"vvv").unwrap_err();
        assert!(
            err.detail.contains("unterminated"),
            "detail: {}",
            err.detail
        );
    }

    #[test]
    fn a_stray_close_brace_at_top_level_is_reported_not_ignored() {
        let err = parse("\"r\" \"v\" }").unwrap_err();
        assert!(err.detail.contains("top level"), "detail: {}", err.detail);
    }
}
