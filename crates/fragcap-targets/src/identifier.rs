// SPDX-License-Identifier: Apache-2.0

//! The stable target identifier (slice S051), specification section 15.8.
//!
//! An anchored target's identifier is a deterministic BLAKE3 truncation over its
//! canonical anchor string, so two independent registrations of one title compute
//! the same value and merge on identity instead of duplicating. An unanchored
//! target's identifier is drawn from OS entropy, and is superseded (and kept as an
//! alias) when the target later gains an anchor.
//!
//! The value occupies the low 63 bits so it is always non-negative in SQLite's
//! signed 64-bit integer column. An anchored identifier is exactly the low 63 bits
//! of BLAKE3 over the canonical anchor: this is the durable, cross-implementation
//! contract, so anything computing "the low 63 bits of BLAKE3 of the anchor"
//! arrives at the same value. The identifier derives only from the anchor, never
//! from the name, handle, or install path. Whether an entry is anchored is read
//! from its `anchor` column, not from a bit of the identifier, so no bit is
//! reserved and the full 63-bit truncation is preserved.

/// Mask of the low 63 bits: an identifier is always non-negative (the sign bit is
/// clear), so it displays and compares cleanly as a SQLite integer.
const MASK_63: u64 = (1u64 << 63) - 1;

/// The canonical anchor string for a Steam title.
pub fn steam_anchor(app_id: u32) -> String {
    format!("steam:{app_id}")
}

/// The canonical anchor string for an Epic title.
pub fn epic_anchor(catalog_item_id: &str) -> String {
    format!("epic:{catalog_item_id}")
}

/// The canonical anchor string for a GOG title.
pub fn gog_anchor(product_id: &str) -> String {
    format!("gog:{product_id}")
}

/// Canonicalize an anchor so logically identical anchors hash identically: trim
/// surrounding whitespace and lowercase the platform prefix (the text before the
/// first `:`). The platform-specific id after the colon is left as written,
/// because some platforms' ids are case-sensitive (an Epic `catalogItemId` is a
/// hex string). A string with no colon is lowercased whole.
pub fn canonicalize_anchor(anchor: &str) -> String {
    let trimmed = anchor.trim();
    match trimmed.split_once(':') {
        Some((platform, id)) => format!("{}:{}", platform.to_lowercase(), id),
        None => trimmed.to_lowercase(),
    }
}

/// The deterministic 63-bit identifier for an anchor: the low 63 bits of BLAKE3
/// over the canonical anchor string.
///
/// The anchor is canonicalized first (so `STEAM:620` and `steam:620` agree), then
/// hashed; only the sign bit is cleared, so the value is the documented 63-bit
/// truncation. Deterministic: the same anchor always yields the same value, which
/// is what makes independent registrations merge.
pub fn anchored_id(anchor: &str) -> i64 {
    let canonical = canonicalize_anchor(anchor);
    let hash = blake3::hash(canonical.as_bytes());
    let first8: [u8; 8] = hash.as_bytes()[..8].try_into().expect("32-byte hash");
    (u64::from_le_bytes(first8) & MASK_63) as i64
}

/// A fresh unanchored 63-bit identifier from OS entropy.
///
/// Two calls almost never collide (63 bits of entropy). An unanchored entry is
/// recognized by its null `anchor`, not by any bit of this value.
pub fn unanchored_id() -> i64 {
    let mut buf = [0u8; 8];
    getrandom::fill(&mut buf).expect("OS entropy is available");
    (u64::from_le_bytes(buf) & MASK_63) as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_anchor_yields_same_id() {
        let a = anchored_id(&steam_anchor(2221490));
        let b = anchored_id(&steam_anchor(2221490));
        assert_eq!(a, b);
    }

    #[test]
    fn different_anchors_differ() {
        assert_ne!(
            anchored_id(&steam_anchor(2221490)),
            anchored_id(&steam_anchor(620))
        );
    }

    #[test]
    fn ids_are_non_negative() {
        assert!(anchored_id(&steam_anchor(2221490)) >= 0);
        assert!(unanchored_id() >= 0);
    }

    #[test]
    fn unanchored_ids_are_distinct() {
        assert_ne!(unanchored_id(), unanchored_id());
    }

    #[test]
    fn anchored_id_is_the_low_63_bits_of_blake3_of_the_canonical_anchor() {
        // The durable contract: the low 63 bits of BLAKE3 over the canonical
        // anchor bytes, sign bit cleared. Anything computing that arrives here.
        let anchor = "steam:620";
        let hash = blake3::hash(anchor.as_bytes());
        let first8: [u8; 8] = hash.as_bytes()[..8].try_into().unwrap();
        let expected = (u64::from_le_bytes(first8) & MASK_63) as i64;
        assert_eq!(anchored_id(anchor), expected);
    }

    #[test]
    fn anchor_prefix_is_canonicalized_before_hashing() {
        // A non-canonical prefix or surrounding whitespace produces the same id as
        // the canonical form, so a CLI-supplied `STEAM:620` resolves like `steam:620`.
        let canonical = anchored_id("steam:620");
        assert_eq!(anchored_id("STEAM:620"), canonical);
        assert_eq!(anchored_id("  Steam:620  "), canonical);
        // The id after the colon is case-sensitive (some platforms' ids are).
        assert_ne!(anchored_id("epic:AbC"), anchored_id("epic:abc"));
    }
}
