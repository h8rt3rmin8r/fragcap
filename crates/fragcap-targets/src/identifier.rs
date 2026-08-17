// SPDX-License-Identifier: Apache-2.0

//! The stable target identifier (slice S051), specification section 15.8.
//!
//! An anchored target's identifier is a deterministic BLAKE3 truncation over its
//! canonical anchor string, so two independent registrations of one title compute
//! the same value and merge on identity instead of duplicating. An unanchored
//! target's identifier is drawn from OS entropy with a reserved locality bit set,
//! so it is distinguishable from an anchored value and can be superseded (and kept
//! as an alias) when the target later gains an anchor.
//!
//! The value occupies 63 bits so it is always non-negative in SQLite's signed
//! 64-bit integer column. Bit 62 is the locality bit: an anchored identifier
//! clears it (using the low 62 bits of the hash), and an unanchored identifier
//! sets it, so [`is_unanchored`] partitions the space exactly. The identifier
//! derives only from the anchor, never from the name, handle, or install path.

/// Bit position of the locality marker within the 63-bit value.
const LOCALITY_BIT: u32 = 62;

/// Mask of the low 62 bits (the significant payload of any identifier).
const PAYLOAD_MASK: u64 = (1u64 << LOCALITY_BIT) - 1;

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

/// The deterministic 63-bit identifier for a canonical anchor string.
///
/// The low 62 bits of BLAKE3 over the anchor bytes; the locality bit (62) and the
/// sign bit (63) are cleared, so the value is non-negative and distinguishable
/// from an unanchored identifier. Deterministic: the same anchor always yields the
/// same value, which is what makes independent registrations merge.
pub fn anchored_id(anchor: &str) -> i64 {
    let hash = blake3::hash(anchor.as_bytes());
    let first8: [u8; 8] = hash.as_bytes()[..8].try_into().expect("32-byte hash");
    let value = u64::from_le_bytes(first8) & PAYLOAD_MASK;
    value as i64
}

/// A fresh unanchored 63-bit identifier from OS entropy, with the locality bit set.
///
/// Two calls almost never collide (62 bits of entropy), and the locality bit marks
/// the value as unanchored so it is never mistaken for a hash of some anchor.
pub fn unanchored_id() -> i64 {
    let mut buf = [0u8; 8];
    getrandom::fill(&mut buf).expect("OS entropy is available");
    let value = (u64::from_le_bytes(buf) & PAYLOAD_MASK) | (1u64 << LOCALITY_BIT);
    value as i64
}

/// Whether an identifier was drawn as unanchored (locality bit set).
pub fn is_unanchored(id: i64) -> bool {
    (id as u64) & (1u64 << LOCALITY_BIT) != 0
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
    fn anchored_ids_are_non_negative_and_not_marked_unanchored() {
        let id = anchored_id(&steam_anchor(2221490));
        assert!(id >= 0);
        assert!(!is_unanchored(id));
    }

    #[test]
    fn unanchored_ids_are_marked_non_negative_and_distinct() {
        let a = unanchored_id();
        let b = unanchored_id();
        assert!(a >= 0 && b >= 0);
        assert!(is_unanchored(a) && is_unanchored(b));
        assert_ne!(a, b);
    }
}
