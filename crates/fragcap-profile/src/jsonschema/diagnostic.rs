// SPDX-License-Identifier: Apache-2.0

//! What a structural refusal looks like.
//!
//! This is the JSON analogue of the profile crate's TOML-oriented
//! [`crate::diagnostic`] set, kept separate on purpose: locations here are JSON
//! pointers, not source line and column, and the codes describe structural
//! conformance to the master schema rather than the semantic checks of section
//! 15.4. The two may be unified when the profile parser moves onto JSON (#76);
//! until then this surface stands alone.
//!
//! As with the profile diagnostics, every problem is collected rather than the
//! first one thrown: a file with four mistakes reports four.

use std::fmt;

/// Why one part of a document failed structural validation.
///
/// A closed enumeration so a caller matches on a variant rather than on message
/// prose. Kebab identifiers are what `schema validate` prints.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SchemaCode {
    /// The document root is not a JSON object.
    NotAnObject,
    /// `schema` is absent, or not the supported version.
    UnsupportedSchema,
    /// `kind` is absent.
    MissingKind,
    /// `kind` is present but not one of the known variants.
    UnknownKind,
    /// `fidelity` is absent where it is required.
    MissingFidelity,
    /// `fidelity` is present but outside the closed tier set.
    InvalidFidelity,
    /// A required key is absent.
    MissingField,
    /// `provenance` is absent where the variant requires it.
    MissingProvenance,
    /// A value has a type the schema does not allow there.
    WrongType,
    /// A key that is not in the accepted set for its object.
    UnknownKey,
    /// `game.id` is present but outside the slug character set.
    InvalidSlug,
    /// `lifecycle` is not `transient`, `session`, or `service`.
    InvalidLifecycle,
    /// `capture.mode` is not `file`, `stream`, or `ring`.
    InvalidMode,
    /// A `match` object carries no predicate, so it would match every process.
    EmptyMatch,
    /// A strict variant declares an empty `stage` array.
    EmptyStages,
    /// A string that must be non-empty is empty (for example `provenance.source`).
    EmptyString,
}

impl SchemaCode {
    /// A short stable identifier for output a person reads.
    pub fn as_str(self) -> &'static str {
        match self {
            SchemaCode::NotAnObject => "not-an-object",
            SchemaCode::UnsupportedSchema => "unsupported-schema",
            SchemaCode::MissingKind => "missing-kind",
            SchemaCode::UnknownKind => "unknown-kind",
            SchemaCode::MissingFidelity => "missing-fidelity",
            SchemaCode::InvalidFidelity => "invalid-fidelity",
            SchemaCode::MissingField => "missing-field",
            SchemaCode::MissingProvenance => "missing-provenance",
            SchemaCode::WrongType => "wrong-type",
            SchemaCode::UnknownKey => "unknown-key",
            SchemaCode::InvalidSlug => "invalid-slug",
            SchemaCode::InvalidLifecycle => "invalid-lifecycle",
            SchemaCode::InvalidMode => "invalid-mode",
            SchemaCode::EmptyMatch => "empty-match",
            SchemaCode::EmptyStages => "empty-stages",
            SchemaCode::EmptyString => "empty-string",
        }
    }
}

impl fmt::Display for SchemaCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One structural problem, located by JSON pointer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SchemaDiagnostic {
    /// The stable identity of the problem.
    pub code: SchemaCode,
    /// A JSON pointer to the value this concerns, for example `/stage/1/match`.
    /// The empty string points at the document root.
    pub pointer: String,
    /// What went wrong, in words. May be reworded without notice.
    pub message: String,
}

impl SchemaDiagnostic {
    /// Build a diagnostic.
    pub fn new(
        code: SchemaCode,
        pointer: impl Into<String>,
        message: impl Into<String>,
    ) -> SchemaDiagnostic {
        SchemaDiagnostic {
            code,
            pointer: pointer.into(),
            message: message.into(),
        }
    }
}

impl fmt::Display for SchemaDiagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let at = if self.pointer.is_empty() {
            "<root>"
        } else {
            self.pointer.as_str()
        };
        write!(f, "{}: {}: {}", at, self.code, self.message)
    }
}

/// Every structural problem in one document, deterministically ordered.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct SchemaDiagnostics {
    items: Vec<SchemaDiagnostic>,
}

impl SchemaDiagnostics {
    /// An empty set.
    pub fn new() -> SchemaDiagnostics {
        SchemaDiagnostics { items: Vec::new() }
    }

    /// Add one diagnostic. Insertion order does not matter; [`Self::finish`] sorts.
    pub fn push(&mut self, d: SchemaDiagnostic) {
        self.items.push(d);
    }

    /// Shorthand: build and push in one call.
    pub fn report(
        &mut self,
        code: SchemaCode,
        pointer: impl Into<String>,
        message: impl Into<String>,
    ) {
        self.items
            .push(SchemaDiagnostic::new(code, pointer, message));
    }

    /// Whether anything was found.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// How many problems were found.
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// The diagnostics, in reporting order.
    pub fn iter(&self) -> std::slice::Iter<'_, SchemaDiagnostic> {
        self.items.iter()
    }

    /// Whether any diagnostic carries the given code.
    pub fn has(&self, code: SchemaCode) -> bool {
        self.items.iter().any(|d| d.code == code)
    }

    /// Sort into reporting order (by pointer, then code) and freeze.
    ///
    /// A stable order is a correctness property: an operator compares two runs,
    /// and serde_json preserves object order but a walk should not depend on it.
    /// Array-index segments compare numerically, so `/stage/2` precedes
    /// `/stage/10` rather than sorting lexicographically after it.
    pub fn finish(mut self) -> SchemaDiagnostics {
        self.items
            .sort_by(|a, b| cmp_pointer(&a.pointer, &b.pointer).then(a.code.cmp(&b.code)));
        self
    }
}

/// Compare two JSON pointers segment by segment, comparing all-digit segments
/// numerically so array indices order as `2` before `10`. A shorter pointer
/// that is a prefix of a longer one sorts first.
fn cmp_pointer(a: &str, b: &str) -> std::cmp::Ordering {
    use std::cmp::Ordering;
    let mut sa = a.split('/');
    let mut sb = b.split('/');
    loop {
        match (sa.next(), sb.next()) {
            (None, None) => return Ordering::Equal,
            (None, Some(_)) => return Ordering::Less,
            (Some(_), None) => return Ordering::Greater,
            (Some(x), Some(y)) => {
                let ord = match (x.parse::<u64>(), y.parse::<u64>()) {
                    (Ok(nx), Ok(ny)) => nx.cmp(&ny),
                    _ => x.cmp(y),
                };
                if ord != Ordering::Equal {
                    return ord;
                }
            }
        }
    }
}

impl fmt::Display for SchemaDiagnostics {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn array_index_segments_order_numerically() {
        let mut set = SchemaDiagnostics::new();
        for i in [10usize, 2, 1] {
            set.report(SchemaCode::MissingField, format!("/stage/{i}/match"), "x");
        }
        let set = set.finish();
        let pointers: Vec<&str> = set.iter().map(|d| d.pointer.as_str()).collect();
        assert_eq!(
            pointers,
            vec!["/stage/1/match", "/stage/2/match", "/stage/10/match"],
            "array indices must order numerically, not lexicographically"
        );
    }
}
