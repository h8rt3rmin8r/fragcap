// SPDX-License-Identifier: Apache-2.0

//! The master JSON Schema and its structural validation surface.
//!
//! This module owns the single versioned schema that governs every
//! machine-readable targeting and attribution artifact (profile, package, hint,
//! export), discriminated by a top-level `kind`. The schema document is
//! [`document::schema_document`]; validation is [`validate_json`].
//!
//! # Structural, not semantic
//!
//! This surface enforces what a schema can enforce: types, required keys, enum
//! ranges, string shapes, unknown-key refusal, and the `kind`/`schema`
//! discriminators. It deliberately does not enforce the semantic invariants of
//! section 15.4 (acyclic `descends_from`, at most one terminal stage, role
//! reachability, no ambiguous image match); those remain the profile-load
//! path's responsibility and are rewired onto JSON by #76. A document that
//! passes here is asserting structural conformance only.
//!
//! # Every problem at once
//!
//! Like the profile validator, this accumulates every violation rather than
//! stopping at the first. A JSON syntax error is reported distinctly from a
//! schema violation, because the two ask the author to do different things.

pub mod diagnostic;
pub mod document;
mod variants;

pub use diagnostic::{SchemaCode, SchemaDiagnostic, SchemaDiagnostics};
pub use document::schema_document;

/// The outcome of validating one candidate document.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Validation {
    /// The input was not syntactically valid JSON. Carries the parser's message.
    /// This is distinct from a schema violation: the author must fix the JSON
    /// before structure can even be checked.
    Malformed(String),
    /// The input parsed. Carries every structural violation found (empty when
    /// the document is valid).
    Checked(SchemaDiagnostics),
}

impl Validation {
    /// Whether the document parsed and has no structural violations.
    pub fn is_valid(&self) -> bool {
        matches!(self, Validation::Checked(d) if d.is_empty())
    }
}

/// Validate JSON text against the master schema.
///
/// Parses the text and, if it parses, runs the structural checks. A parse
/// failure returns [`Validation::Malformed`]; a successful parse returns
/// [`Validation::Checked`] with every violation (possibly none).
pub fn validate_json(text: &str) -> Validation {
    match serde_json::from_str::<serde_json::Value>(text) {
        Err(e) => Validation::Malformed(e.to_string()),
        Ok(value) => Validation::Checked(variants::check(&value)),
    }
}

/// Validate an already-parsed value against the master schema.
///
/// The structural half of [`validate_json`], for a caller that has already parsed
/// the text to a [`serde_json::Value`] (the profile-load path does, and reuses
/// this so there is one structural implementation rather than two).
pub fn validate_value(value: &serde_json::Value) -> SchemaDiagnostics {
    variants::check(value)
}
