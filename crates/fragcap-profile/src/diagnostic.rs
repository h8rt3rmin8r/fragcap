// SPDX-License-Identifier: Apache-2.0

//! What a refusal looks like.
//!
//! Specification section 15.4 requires that validation report every problem
//! found rather than stopping at the first, so the unit of being wrong is a
//! collection rather than a single error. [`Diagnostics`] is that collection,
//! and it is the shape every check in this crate pushes into.
//!
//! # What is stable and what is not
//!
//! [`DiagnosticCode`] is stable surface. It is what tests, the command line's
//! `profile validate` output, and any future documentation key on, so adding a
//! variant is a change a reviewer sees.
//!
//! A diagnostic's location string and message are not. They exist to point an
//! author at a line in their own file, and committing to their shape would
//! freeze a formatting choice for the benefit of a consumer that does not exist.
//! Tests in this repository may assert on them because they change together.
//!
//! # Ordering
//!
//! The set is sorted by byte offset and then by code. A stable order is a
//! correctness property rather than a nicety: an operator compares two runs, and
//! the parser this crate uses iterates tables in key order rather than document
//! order, so relying on traversal order would tie the output to a container
//! choice inside a dependency.

use std::fmt;

/// A one-based line and column in the profile text.
///
/// One-based because that is what an editor shows the author who has to fix the
/// file. Derived from a byte offset in [`Position::from_offset`], which is the
/// only place the conversion happens.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Position {
    /// One-based line number.
    pub line: usize,
    /// One-based column, counted in characters rather than bytes, so a path
    /// containing non-ASCII does not report a column the author cannot find.
    pub column: usize,
}

impl Position {
    /// Convert a byte offset into the text into a line and column.
    ///
    /// An offset past the end of the text is clamped to the end rather than
    /// panicking. A diagnostic about a truncated document is still worth
    /// delivering, and a panic while reporting an error is the worst available
    /// outcome.
    pub fn from_offset(text: &str, offset: usize) -> Position {
        let end = offset.min(text.len());
        let before = &text[..end];
        let line = before.matches('\n').count() + 1;
        let line_start = before.rfind('\n').map(|i| i + 1).unwrap_or(0);
        let column = text[line_start..end].chars().count() + 1;
        Position { line, column }
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

/// Why a profile was refused.
///
/// A closed enumeration so that a caller matches on a variant rather than on
/// message prose. Every variant is produced by at least one test; a code that
/// cannot be produced is indistinguishable from one that is wired up wrong.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum DiagnosticCode {
    /// The document is not valid JSON.
    Syntax,
    /// The declared schema version is not one this build supports.
    UnsupportedSchema,
    /// A required key is absent.
    MissingField,
    /// A key's value has a type the schema does not allow there.
    WrongType,
    /// A key that is not in the accepted set for its table.
    UnknownKey,
    /// The candidate file is larger than [`crate::MAX_PROFILE_BYTES`].
    FileTooLarge,
    /// `game.id` is empty or outside the slug character set.
    InvalidSlug,
    /// `lifecycle` is not `transient`, `session`, or `service`.
    InvalidLifecycle,
    /// `capture.mode` is not `file`, `stream`, or `ring`.
    InvalidMode,
    /// A duration literal does not parse.
    InvalidDuration,
    /// An `exe` pattern is not a well-formed image name pattern.
    InvalidGlob,
    /// A `path_regex` does not compile, including because it exceeds the
    /// engine's own compiled size limit.
    InvalidRegex,
    /// A `match` table carries no predicate, so it would match every process.
    EmptyMatch,
    /// `capture.roles` is present and empty, so it names nothing to capture.
    EmptyRoles,
    /// The profile declares no stage.
    NoStages,
    /// The profile declares more stages than [`crate::MAX_STAGES`].
    TooManyStages,
    /// Two stages declare the same role.
    DuplicateRole,
    /// More than one stage is marked terminal.
    MultipleTerminal,
    /// A terminal stage's lifecycle is not `session`.
    TerminalLifecycle,
    /// `descends_from` names a role no stage declares.
    UnknownDescendsFrom,
    /// The `descends_from` relation contains a cycle.
    DescendsFromCycle,
    /// `capture.roles` names a role no stage declares.
    UndeclaredCaptureRole,
    /// Every stage is a service, so nothing can ever trigger acquisition.
    AllServices,
    /// Two stages can match one image name and at least one has nothing else to
    /// distinguish it.
    AmbiguousImageMatch,
    /// A capture profile declares `fidelity: observed`. The `observed` tier is a
    /// runtime result the observation provider stamps, not a trust level an
    /// author can claim, and allowing it would let the top-precedence provider
    /// answer below the fidelity of a lower one (section 15.7).
    ObservedProfileFidelity,
}

impl DiagnosticCode {
    /// A short stable identifier, for output that a person reads.
    ///
    /// Kebab-case rather than the variant name, because this is what appears in
    /// `fragcap profile validate` output and reads better there.
    pub fn as_str(self) -> &'static str {
        match self {
            DiagnosticCode::Syntax => "syntax",
            DiagnosticCode::UnsupportedSchema => "unsupported-schema",
            DiagnosticCode::MissingField => "missing-field",
            DiagnosticCode::WrongType => "wrong-type",
            DiagnosticCode::UnknownKey => "unknown-key",
            DiagnosticCode::FileTooLarge => "file-too-large",
            DiagnosticCode::InvalidSlug => "invalid-slug",
            DiagnosticCode::InvalidLifecycle => "invalid-lifecycle",
            DiagnosticCode::InvalidMode => "invalid-mode",
            DiagnosticCode::InvalidDuration => "invalid-duration",
            DiagnosticCode::InvalidGlob => "invalid-glob",
            DiagnosticCode::InvalidRegex => "invalid-regex",
            DiagnosticCode::EmptyMatch => "empty-match",
            DiagnosticCode::EmptyRoles => "empty-roles",
            DiagnosticCode::NoStages => "no-stages",
            DiagnosticCode::TooManyStages => "too-many-stages",
            DiagnosticCode::DuplicateRole => "duplicate-role",
            DiagnosticCode::MultipleTerminal => "multiple-terminal",
            DiagnosticCode::TerminalLifecycle => "terminal-lifecycle",
            DiagnosticCode::UnknownDescendsFrom => "unknown-descends-from",
            DiagnosticCode::DescendsFromCycle => "descends-from-cycle",
            DiagnosticCode::UndeclaredCaptureRole => "undeclared-capture-role",
            DiagnosticCode::AllServices => "all-services",
            DiagnosticCode::AmbiguousImageMatch => "ambiguous-image-match",
            DiagnosticCode::ObservedProfileFidelity => "observed-profile-fidelity",
        }
    }
}

impl fmt::Display for DiagnosticCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One problem found in one profile.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diagnostic {
    /// The stable identity of the problem.
    pub code: DiagnosticCode,
    /// A dotted key path, for example `stage[1].match.descends_from`. For a
    /// person to read; not a grammar to parse.
    pub location: String,
    /// Byte offset of the value this concerns, when the parser reported one.
    pub offset: Option<usize>,
    /// Line and column derived from `offset`.
    pub position: Option<Position>,
    /// What went wrong, in words. May be reworded without notice.
    pub message: String,
}

impl Diagnostic {
    /// Build a diagnostic that carries a location in the source text.
    pub fn at(
        code: DiagnosticCode,
        location: impl Into<String>,
        text: &str,
        offset: usize,
        message: impl Into<String>,
    ) -> Diagnostic {
        Diagnostic {
            code,
            location: location.into(),
            offset: Some(offset),
            position: Some(Position::from_offset(text, offset)),
            message: message.into(),
        }
    }

    /// Build a diagnostic located by a JSON pointer, with no byte position.
    ///
    /// The profile-load path locates faults by JSON pointer (for example
    /// `/stage/1/match/exe`) because serde_json exposes no per-value byte span;
    /// the pointer names the exact value, and the line and column are not
    /// available. `offset` and `position` are therefore `None`.
    pub fn located(
        code: DiagnosticCode,
        pointer: impl Into<String>,
        message: impl Into<String>,
    ) -> Diagnostic {
        Diagnostic {
            code,
            location: pointer.into(),
            offset: None,
            position: None,
            message: message.into(),
        }
    }

    /// Build a diagnostic with no position, for a problem about the file as a
    /// whole rather than about a value inside it.
    pub fn whole_file(
        code: DiagnosticCode,
        location: impl Into<String>,
        message: impl Into<String>,
    ) -> Diagnostic {
        Diagnostic {
            code,
            location: location.into(),
            offset: None,
            position: None,
            message: message.into(),
        }
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.position {
            Some(p) => write!(
                f,
                "{}: {}: {}: {}",
                p, self.code, self.location, self.message
            ),
            None => write!(f, "{}: {}: {}", self.code, self.location, self.message),
        }
    }
}

/// Every problem found in one profile, deterministically ordered.
///
/// Non-empty whenever it is returned as an error: a failure that reports nothing
/// is a failure an author cannot act on.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Diagnostics {
    items: Vec<Diagnostic>,
}

impl Diagnostics {
    /// An empty set.
    pub fn new() -> Diagnostics {
        Diagnostics { items: Vec::new() }
    }

    /// Add one diagnostic.
    ///
    /// Order of insertion does not matter; [`Diagnostics::finish`] sorts.
    pub fn push(&mut self, d: Diagnostic) {
        self.items.push(d);
    }

    /// Whether anything was found.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// How many problems were found.
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// The diagnostics, in the order they will be reported.
    pub fn iter(&self) -> std::slice::Iter<'_, Diagnostic> {
        self.items.iter()
    }

    /// Whether any diagnostic carries the given code.
    ///
    /// Present because it is what a test wants to assert, and a test that
    /// searches the message text instead would be asserting on the unstable
    /// half of a diagnostic.
    pub fn has(&self, code: DiagnosticCode) -> bool {
        self.items.iter().any(|d| d.code == code)
    }

    /// Every code present, sorted and deduplicated.
    pub fn codes(&self) -> Vec<DiagnosticCode> {
        let mut out: Vec<DiagnosticCode> = self.items.iter().map(|d| d.code).collect();
        out.sort_unstable();
        out.dedup();
        out
    }

    /// Sort into reporting order and freeze.
    ///
    /// By offset, then code, then location. A diagnostic with no offset sorts
    /// first, because it concerns the file as a whole and is the context for
    /// everything after it.
    pub fn finish(mut self) -> Diagnostics {
        self.items.sort_by(|a, b| {
            a.offset
                .cmp(&b.offset)
                .then(a.code.cmp(&b.code))
                .then_with(|| a.location.cmp(&b.location))
        });
        self
    }
}

impl fmt::Display for Diagnostics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, d) in self.items.iter().enumerate() {
            if i > 0 {
                writeln!(f)?;
            }
            write!(f, "{d}")?;
        }
        Ok(())
    }
}

impl std::error::Error for Diagnostics {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn position_is_one_based_from_the_first_character() {
        let text = "schema = 1\n";
        assert_eq!(
            Position::from_offset(text, 0),
            Position { line: 1, column: 1 }
        );
    }

    #[test]
    fn position_finds_a_value_mid_line() {
        let text = "schema = 1\n";
        assert_eq!(
            Position::from_offset(text, 9),
            Position {
                line: 1,
                column: 10
            }
        );
    }

    #[test]
    fn position_counts_lines_and_restarts_columns() {
        let text = "a = 1\nb = 2\nc = 3\n";
        assert_eq!(
            Position::from_offset(text, 6),
            Position { line: 2, column: 1 }
        );
        assert_eq!(
            Position::from_offset(text, 10),
            Position { line: 2, column: 5 }
        );
        assert_eq!(
            Position::from_offset(text, 12),
            Position { line: 3, column: 1 }
        );
    }

    #[test]
    fn position_at_a_line_end_is_the_newline_column() {
        let text = "ab\ncd\n";
        assert_eq!(
            Position::from_offset(text, 2),
            Position { line: 1, column: 3 }
        );
    }

    #[test]
    fn position_counts_characters_not_bytes() {
        // A path with a non-ASCII directory name is ordinary on Windows, and a
        // byte column would point the author at the wrong place.
        let text = "a = 'José'\n";
        let offset = text.find("'\n").expect("closing quote");
        let p = Position::from_offset(text, offset);
        assert_eq!(p.line, 1);
        assert_eq!(
            p.column, 10,
            "nine characters precede the closing quote, not ten bytes"
        );
    }

    #[test]
    fn position_past_the_end_is_clamped_rather_than_panicking() {
        let text = "a = 1\n";
        let p = Position::from_offset(text, 9_999);
        assert_eq!(
            p.line, 2,
            "the text ends with a newline, so offset end is line 2"
        );
    }

    fn d(code: DiagnosticCode, loc: &str, offset: usize) -> Diagnostic {
        Diagnostic {
            code,
            location: loc.to_string(),
            offset: Some(offset),
            position: Some(Position { line: 1, column: 1 }),
            message: "m".to_string(),
        }
    }

    #[test]
    fn insertion_order_does_not_affect_reporting_order() {
        let mut a = Diagnostics::new();
        a.push(d(DiagnosticCode::WrongType, "game.name", 30));
        a.push(d(DiagnosticCode::MissingField, "game.id", 10));
        a.push(d(DiagnosticCode::UnknownKey, "capture.payloads", 20));

        let mut b = Diagnostics::new();
        b.push(d(DiagnosticCode::UnknownKey, "capture.payloads", 20));
        b.push(d(DiagnosticCode::WrongType, "game.name", 30));
        b.push(d(DiagnosticCode::MissingField, "game.id", 10));

        assert_eq!(a.finish(), b.finish());
    }

    #[test]
    fn reporting_order_is_by_offset() {
        let mut set = Diagnostics::new();
        set.push(d(DiagnosticCode::WrongType, "z", 30));
        set.push(d(DiagnosticCode::MissingField, "a", 10));
        let set = set.finish();
        let offsets: Vec<_> = set.iter().map(|x| x.offset).collect();
        assert_eq!(offsets, vec![Some(10), Some(30)]);
    }

    #[test]
    fn a_whole_file_diagnostic_sorts_before_positioned_ones() {
        let mut set = Diagnostics::new();
        set.push(d(DiagnosticCode::MissingField, "game.id", 10));
        set.push(Diagnostic::whole_file(
            DiagnosticCode::FileTooLarge,
            "<file>",
            "too large",
        ));
        let set = set.finish();
        assert_eq!(
            set.iter().next().map(|x| x.code),
            Some(DiagnosticCode::FileTooLarge)
        );
    }

    #[test]
    fn display_is_stable_across_runs() {
        let build = || {
            let mut set = Diagnostics::new();
            set.push(d(DiagnosticCode::WrongType, "game.name", 30));
            set.push(d(DiagnosticCode::MissingField, "game.id", 10));
            set.finish().to_string()
        };
        assert_eq!(build(), build());
    }

    #[test]
    fn codes_are_reported_deduplicated() {
        let mut set = Diagnostics::new();
        set.push(d(DiagnosticCode::MissingField, "a", 1));
        set.push(d(DiagnosticCode::MissingField, "b", 2));
        set.push(d(DiagnosticCode::WrongType, "c", 3));
        assert_eq!(
            set.finish().codes(),
            vec![DiagnosticCode::MissingField, DiagnosticCode::WrongType]
        );
    }
}
