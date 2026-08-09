// SPDX-License-Identifier: Apache-2.0

//! Image name patterns, and whether two of them can name one process.
//!
//! The `exe` match predicate of specification section 10.3 is a glob over an
//! executable file name, compared case-insensitively. Specification section
//! 15.4 needs a second question answered about the same syntax: whether two
//! patterns "can match the same image name". That is glob intersection rather
//! than glob matching, and no glob crate answers it, which is why this module
//! exists rather than a dependency. See slice S05 research R-2.
//!
//! Both questions are one walk. Matching a name is the intersection of a
//! pattern with a literal, so [`ImagePattern::matches`] and
//! [`ImagePattern::intersects`] share a decision procedure and cannot drift
//! apart into two readings of the same syntax.
//!
//! # The syntax, completely
//!
//! - `*` matches any run of characters, including none.
//! - `?` matches exactly one character.
//! - Every other character is a literal.
//! - There is no escape sequence, because Windows forbids `*` and `?` in a file
//!   name, so an occurrence of either always means the wildcard.
//!
//! Every non-empty string is therefore a well-formed pattern. The empty pattern
//! is refused, because it matches only the empty image name.
//!
//! # A note on case folding and constitution P-9
//!
//! Comparison folds case, per section 10.3. The fold is applied to copies
//! during comparison; [`ImagePattern::as_str`] returns what the profile author
//! wrote, unaltered. A reviewer finding a lowercase conversion here should read
//! it against that distinction rather than as a P-9 violation: the stored
//! observation is verbatim, and only the comparison is insensitive.

use std::fmt;

/// One element of a pattern.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Tok {
    /// A literal character, compared with case folding.
    Lit(char),
    /// `?`, which consumes exactly one character.
    Any,
    /// `*`, which consumes any run including none.
    Star,
}

impl Tok {
    /// Whether this token consumes exactly one character.
    ///
    /// True for a literal and for `?`, false for `*`. The walk needs this to
    /// decide whether a `*` on the other side has a character to absorb.
    fn is_single(self) -> bool {
        !matches!(self, Tok::Star)
    }
}

/// Two characters are equal under simple case folding.
///
/// Compares the lowercase expansions rather than a single mapped character, so
/// that a character whose lowercase form is more than one character is handled
/// rather than silently truncated. Deterministic and locale-independent, which
/// matters because a profile must validate the same way on every machine.
fn folded_eq(a: char, b: char) -> bool {
    a == b || a.to_lowercase().eq(b.to_lowercase())
}

/// Whether two tokens can both consume one same character.
fn compatible(a: Tok, b: Tok) -> bool {
    match (a, b) {
        (Tok::Lit(x), Tok::Lit(y)) => folded_eq(x, y),
        (Tok::Any, Tok::Lit(_)) | (Tok::Lit(_), Tok::Any) | (Tok::Any, Tok::Any) => true,
        _ => false,
    }
}

/// An `exe` glob, holding the author's text verbatim.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImagePattern {
    source: String,
    toks: Vec<Tok>,
}

/// Why a pattern was refused.
///
/// One variant, because every non-empty string is a well-formed pattern in this
/// syntax. It is an enumeration rather than a unit type so that adding a rule
/// later does not change the shape of every call site.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PatternError {
    /// The pattern was empty, which matches only the empty image name.
    Empty,
}

impl fmt::Display for PatternError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PatternError::Empty => write!(f, "empty image name pattern"),
        }
    }
}

impl std::error::Error for PatternError {}

impl ImagePattern {
    /// Build a pattern from what the profile declared.
    ///
    /// # Errors
    ///
    /// [`PatternError::Empty`] if the pattern is empty.
    pub fn new(source: &str) -> Result<Self, PatternError> {
        if source.is_empty() {
            return Err(PatternError::Empty);
        }
        let toks = source
            .chars()
            .map(|c| match c {
                '*' => Tok::Star,
                '?' => Tok::Any,
                other => Tok::Lit(other),
            })
            .collect();
        Ok(ImagePattern {
            source: source.to_string(),
            toks,
        })
    }

    /// The pattern as the author wrote it, unaltered.
    pub fn as_str(&self) -> &str {
        &self.source
    }

    /// Whether this pattern matches one concrete image name.
    ///
    /// The name is treated as literal throughout: a `*` inside a name is a
    /// literal asterisk, not a wildcard. Windows cannot produce such a name,
    /// but reading the haystack as a pattern would be a way for a crafted name
    /// to widen its own match, so the distinction is made explicitly.
    pub fn matches(&self, name: &str) -> bool {
        let literal: Vec<Tok> = name.chars().map(Tok::Lit).collect();
        reachable(&self.toks, &literal)
    }

    /// Whether some image name exists that both patterns match.
    ///
    /// Exact rather than conservative. Specification section 15.4 turns this
    /// answer into a validation error, and both directions of approximation are
    /// harmful: a false negative admits the silent empty capture the check
    /// exists to prevent, and a false positive refuses a legal profile with
    /// advice its author cannot act on.
    pub fn intersects(&self, other: &ImagePattern) -> bool {
        reachable(&self.toks, &other.toks)
    }
}

/// Decide whether two token sequences can generate a common string.
///
/// A breadth-first walk over a table of positions. Cell `(i, j)` is reachable
/// when the prefixes `a[..i]` and `b[..j]` can be produced by one common
/// string, and the answer is whether `(a.len(), b.len())` is reachable.
///
/// Cost is `O(a.len() * b.len())`: each cell is enqueued at most once. Slice
/// S05 research R-2 records why that bound is left stated rather than capped.
fn reachable(a: &[Tok], b: &[Tok]) -> bool {
    let (n, m) = (a.len(), b.len());
    let mut seen = vec![false; (n + 1) * (m + 1)];
    let idx = |i: usize, j: usize| i * (m + 1) + j;
    let mut stack = vec![(0usize, 0usize)];
    seen[idx(0, 0)] = true;

    while let Some((i, j)) = stack.pop() {
        if i == n && j == m {
            return true;
        }
        let push = |i: usize, j: usize, stack: &mut Vec<(usize, usize)>, seen: &mut Vec<bool>| {
            let k = idx(i, j);
            if !seen[k] {
                seen[k] = true;
                stack.push((i, j));
            }
        };

        // A star on either side may consume nothing and be done with.
        if i < n && a[i] == Tok::Star {
            push(i + 1, j, &mut stack, &mut seen);
        }
        if j < m && b[j] == Tok::Star {
            push(i, j + 1, &mut stack, &mut seen);
        }
        // A star on one side absorbs one character the other side produces.
        if i < n && a[i] == Tok::Star && j < m && b[j].is_single() {
            push(i, j + 1, &mut stack, &mut seen);
        }
        if j < m && b[j] == Tok::Star && i < n && a[i].is_single() {
            push(i + 1, j, &mut stack, &mut seen);
        }
        // Both sides consume one character, and it can be the same one.
        if i < n && j < m && compatible(a[i], b[j]) {
            push(i + 1, j + 1, &mut stack, &mut seen);
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(s: &str) -> ImagePattern {
        ImagePattern::new(s).expect("pattern")
    }

    #[test]
    fn the_empty_pattern_is_refused() {
        assert_eq!(ImagePattern::new(""), Err(PatternError::Empty));
    }

    #[test]
    fn the_pattern_is_stored_verbatim() {
        let pat = p("*Launcher.EXE");
        assert_eq!(
            pat.as_str(),
            "*Launcher.EXE",
            "constitution P-9: the declared value is not normalized"
        );
    }

    #[test]
    fn matches_literals_and_wildcards() {
        let cases: &[(&str, &str, bool)] = &[
            ("eso64.exe", "eso64.exe", true),
            ("eso64.exe", "eso32.exe", false),
            ("eso64.exe", "eso64.exe.bak", false),
            ("*", "anything", true),
            ("*", "", true),
            ("*.exe", "eso64.exe", true),
            ("*.exe", "eso64.dll", false),
            ("*Launcher.exe", "ESOLauncher.exe", true),
            ("*Launcher.exe", "Launcher.exe", true),
            ("*Launcher.exe", "LauncherX.exe", false),
            ("eso*", "eso64.exe", true),
            ("e*o*.exe", "eso64.exe", true),
            ("?so64.exe", "eso64.exe", true),
            ("?so64.exe", "so64.exe", false),
            ("?", "a", true),
            ("?", "ab", false),
            ("a?c", "abc", true),
            ("a?c", "ac", false),
            ("**", "abc", true),
            ("a**b", "ab", true),
            ("a**b", "axxb", true),
        ];
        for (pattern, name, expected) in cases {
            assert_eq!(
                p(pattern).matches(name),
                *expected,
                "pattern {pattern:?} against name {name:?}"
            );
        }
    }

    #[test]
    fn matching_folds_case_per_section_10_3() {
        assert!(p("ESO64.EXE").matches("eso64.exe"));
        assert!(p("eso64.exe").matches("ESO64.EXE"));
        assert!(p("*launcher.exe").matches("ESOLauncher.exe"));
    }

    #[test]
    fn a_wildcard_in_the_name_is_a_literal_not_a_wildcard() {
        // Windows cannot produce such a name. Asserted so that the haystack is
        // never read as a pattern, which would let a name widen its own match.
        assert!(!p("eso64.exe").matches("*"));
        assert!(p("*").matches("*"));
    }

    #[test]
    fn intersecting_pairs_are_reported_as_intersecting() {
        let cases: &[(&str, &str)] = &[
            ("eso64.exe", "eso64.exe"),
            ("ESO64.EXE", "eso64.exe"),
            ("*Launcher.exe", "ESOLauncher.exe"),
            ("*Launcher.exe", "*.exe"),
            ("*", "eso64.exe"),
            ("*", "*"),
            ("?so64.exe", "eso64.exe"),
            ("?so64.exe", "*so64.exe"),
            ("a*", "*b"),
            ("a*z", "a*z"),
            ("TheDivision2.exe", "TheDivision2.exe"),
            ("*.exe", "*.exe"),
            ("a?c", "abc"),
            ("a*c", "abbbc"),
        ];
        for (x, y) in cases {
            assert!(
                p(x).intersects(&p(y)),
                "expected {x:?} and {y:?} to intersect"
            );
            assert!(
                p(y).intersects(&p(x)),
                "intersection must be symmetric: {y:?} and {x:?}"
            );
        }
    }

    #[test]
    fn disjoint_pairs_are_reported_as_disjoint() {
        let cases: &[(&str, &str)] = &[
            ("eso64.exe", "eso32.exe"),
            ("a*.exe", "b*.exe"),
            // A shared prefix is not a shared whole name.
            ("eso*.exe", "esoteric*.dll"),
            // A shared suffix is not a shared whole name.
            ("launcher.exe", "xlauncher.exe"),
            ("?.exe", "ab.exe"),
            ("?", "ab"),
            ("a?c", "ac"),
            ("*.exe", "*.dll"),
            ("eso64.exe", ""),
        ];
        for (x, y) in cases {
            // The empty pattern cannot be constructed, so the last case is
            // expressed through the token walk directly.
            if y.is_empty() {
                assert!(!reachable(&p(x).toks, &[]));
                continue;
            }
            assert!(
                !p(x).intersects(&p(y)),
                "expected {x:?} and {y:?} to be disjoint"
            );
            assert!(
                !p(y).intersects(&p(x)),
                "disjointness must be symmetric: {y:?} and {x:?}"
            );
        }
    }

    #[test]
    fn the_section_5_4_case_is_reported_as_intersecting() {
        // Three processes under one image name, only the last of which holds
        // sockets. Two stages matching on `exe` alone would both bind here,
        // which is what specification section 15.4 makes a validation error.
        let a = p("TheDivision2.exe");
        let b = p("TheDivision2.exe");
        assert!(a.intersects(&b));
    }

    #[test]
    fn matching_and_intersection_agree() {
        // The property that would catch the two walks drifting apart if they
        // are ever separated: a name matched by both patterns is a witness that
        // the pair intersects.
        let patterns = [
            "eso64.exe",
            "ESO64.EXE",
            "*",
            "*.exe",
            "*Launcher.exe",
            "?so64.exe",
            "a*z",
            "a?c",
            "TheDivision2.exe",
        ];
        let names = [
            "eso64.exe",
            "ESO64.EXE",
            "ESOLauncher.exe",
            "abc",
            "az",
            "a123z",
            "TheDivision2.exe",
            "unrelated.dll",
            "",
        ];
        for x in patterns {
            for y in patterns {
                let witness = names.iter().any(|n| p(x).matches(n) && p(y).matches(n));
                if witness {
                    assert!(
                        p(x).intersects(&p(y)),
                        "{x:?} and {y:?} share a matching name but were reported disjoint"
                    );
                }
            }
        }
    }
}
