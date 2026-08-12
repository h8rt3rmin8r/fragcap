// SPDX-License-Identifier: Apache-2.0

//! The embedded master schema document.
//!
//! The schema the binary enforces and the schema it publishes are the same
//! bytes, because they are one file included at compile time. `schema print`
//! emits exactly this, and the drift check compares it to the repository copy.

/// The master JSON Schema (Draft 2020-12) as embedded at build time.
///
/// This is the single source of truth. It is a standard schema document for
/// external tooling (editors, agents, the submission pipeline); fragcap's own
/// validation is hand-rolled in [`super::variants`] and bound to this document
/// by the conformance corpus test.
pub const SCHEMA_JSON: &str = include_str!("../../assets/target-schema.v1.json");

/// The embedded schema document, byte for byte.
pub fn schema_document() -> &'static str {
    SCHEMA_JSON
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_embedded_schema_is_valid_json() {
        let parsed: serde_json::Value =
            serde_json::from_str(SCHEMA_JSON).expect("embedded schema must be valid JSON");
        assert!(parsed.is_object(), "the schema document is a JSON object");
        assert_eq!(
            parsed.get("$schema").and_then(|v| v.as_str()),
            Some("https://json-schema.org/draft/2020-12/schema"),
            "the dialect is Draft 2020-12"
        );
    }
}
