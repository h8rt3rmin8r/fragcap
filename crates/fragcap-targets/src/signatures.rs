// SPDX-License-Identifier: Apache-2.0

//! The detection signature seed (slice S053).
//!
//! The `signature` table in `catalog.db` is data, refreshed through the same
//! catalog-seed family that refreshes the title catalog. This module parses the
//! bundled Appendix B seed document into [`Signature`]s and seeds the store from it,
//! the same offline-from-a-bundled-asset shape as [`crate::seed::seed_catalog`]. A
//! live signature feed would be a later `net`-gated addition behind a source seam,
//! exactly as the catalog seeder is.

use fragcap_profile::{Signature, SignatureCategory, SignatureConfidence, SignatureKind};

use crate::store::Store;
use crate::TargetsError;

/// The bundled Appendix B signature set, embedded at build time. Seeding from this
/// is the offline default; `targets seed-signatures` writes it into a catalog.
pub const BUNDLED_SIGNATURES: &str = include_str!("../assets/signatures.json");

/// Parse a signature seed document (a JSON array of signature objects) into
/// [`Signature`]s. Every object must carry a known `category`, `kind`, `pattern`,
/// `product`, and `confidence`; an unknown enum value or a missing field is an
/// error naming the offending entry, never a silently dropped row (P-4, P-9).
pub fn parse_seed_document(json: &str) -> Result<Vec<Signature>, TargetsError> {
    let value: serde_json::Value = serde_json::from_str(json)
        .map_err(|e| TargetsError::Model(format!("signature seed is not valid JSON: {e}")))?;
    let array = value
        .as_array()
        .ok_or_else(|| TargetsError::Model("signature seed must be a JSON array".to_string()))?;

    let mut out = Vec::with_capacity(array.len());
    for (i, entry) in array.iter().enumerate() {
        out.push(parse_entry(entry, i)?);
    }
    Ok(out)
}

/// Seed the store's signature table from the bundled Appendix B document. Idempotent
/// through [`Store::seed_signatures`]: re-running reloads the same table.
pub fn seed_bundled(store: &mut Store) -> Result<usize, TargetsError> {
    let signatures = parse_seed_document(BUNDLED_SIGNATURES)?;
    store.seed_signatures(&signatures)?;
    Ok(signatures.len())
}

fn parse_entry(entry: &serde_json::Value, index: usize) -> Result<Signature, TargetsError> {
    let field = |name: &str| -> Result<String, TargetsError> {
        entry
            .get(name)
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| {
                TargetsError::Model(format!(
                    "signature entry {index} missing string field {name:?}"
                ))
            })
    };

    let category_text = field("category")?;
    let kind_text = field("kind")?;
    let pattern = field("pattern")?;
    let product = field("product")?;
    let confidence_text = field("confidence")?;

    let category = SignatureCategory::parse(&category_text).ok_or_else(|| {
        TargetsError::Model(format!(
            "signature entry {index} has unknown category {category_text:?}"
        ))
    })?;
    let kind = SignatureKind::parse(&kind_text).ok_or_else(|| {
        TargetsError::Model(format!(
            "signature entry {index} has unknown kind {kind_text:?}"
        ))
    })?;
    let confidence = SignatureConfidence::parse(&confidence_text).ok_or_else(|| {
        TargetsError::Model(format!(
            "signature entry {index} has unknown confidence {confidence_text:?}"
        ))
    })?;
    if pattern.is_empty() {
        return Err(TargetsError::Model(format!(
            "signature entry {index} has an empty pattern"
        )));
    }
    if product.is_empty() {
        return Err(TargetsError::Model(format!(
            "signature entry {index} has an empty product"
        )));
    }

    Ok(Signature {
        category,
        kind,
        pattern,
        product,
        confidence,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_bundled_document_parses() {
        let sigs = parse_seed_document(BUNDLED_SIGNATURES).expect("bundled seed parses");
        assert!(!sigs.is_empty(), "the bundled seed has signatures");
    }

    #[test]
    fn the_bundled_document_covers_every_appendix_b_product() {
        let sigs = parse_seed_document(BUNDLED_SIGNATURES).expect("bundled seed parses");
        // SC-001: every Appendix B product is represented by at least one row.
        const PRODUCTS: &[&str] = &[
            "Unity",
            "Unreal",
            "Source",
            "Godot",
            "CryEngine",
            "RE Engine",
            "Easy Anti-Cheat",
            "BattlEye",
            "Vanguard",
            "mhyprot",
            "nProtect GameGuard",
            "Xigncode3",
            "Denuvo",
            "Steam DRM",
            "Arxan",
            "VMProtect",
        ];
        for product in PRODUCTS {
            assert!(
                sigs.iter().any(|s| s.product == *product),
                "Appendix B product {product:?} is seeded"
            );
        }
    }

    #[test]
    fn an_unknown_category_is_rejected_not_dropped() {
        let json = r#"[{"category":"nonsense","kind":"filename","pattern":"x.dll","product":"X","confidence":"definitive"}]"#;
        assert!(parse_seed_document(json).is_err());
    }

    #[test]
    fn a_missing_field_is_rejected() {
        let json =
            r#"[{"category":"engine","kind":"filename","product":"X","confidence":"definitive"}]"#;
        assert!(parse_seed_document(json).is_err());
    }
}
