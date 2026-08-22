// SPDX-License-Identifier: Apache-2.0

//! Appendix A handle test vectors (slice S051, specification section 15.8).
//!
//! Every row of the issue #138 Appendix A table is a case here. Special
//! characters are written as explicit `\u{...}` escapes so the vector is
//! unambiguous regardless of editor or source encoding.

use fragcap_targets::handle::{derive_handle, disambiguate, normalize};

/// The table's positive vectors: name -> expected handle.
#[test]
fn appendix_a_positive_vectors() {
    let cases: &[(&str, &str)] = &[
        // Tom Clancy's(TM) The Division 2  -- TM (U+2122) stripped, apostrophe deleted.
        (
            "Tom Clancy's\u{2122} The Division 2",
            "tom_clancys_the_division_2",
        ),
        // The Elder Scrolls(R) Online -- registered sign (U+00AE) stripped.
        (
            "The Elder Scrolls\u{00AE} Online",
            "the_elder_scrolls_online",
        ),
        // Pokemon with acute -- NFKD splits the accent to a combining mark, stripped.
        ("Pok\u{00E9}mon", "pokemon"),
        // Ratchet & Clank -- & expands to "and" (slice S066, issue #173), rather
        // than disappearing as it did before.
        ("Ratchet & Clank", "ratchet_and_clank"),
        // Final Fantasy + Roman numeral four (U+2163) -- NFKD -> "IV" -> "iv".
        ("Final Fantasy \u{2163}", "final_fantasy_iv"),
        // Half-Life 2: Episode One.
        ("Half-Life 2: Episode One", "half_life_2_episode_one"),
        // S.T.A.L.K.E.R. -- each dot is its own run.
        ("S.T.A.L.K.E.R.", "s_t_a_l_k_e_r"),
        // Rock Band 360 + degree sign (U+00B0, So) stripped.
        ("Rock Band 360\u{00B0}", "rock_band_360"),
        // Vulgar half (U+00BD, No, not stripped) -- NFKD -> 1, fraction slash, 2.
        ("\u{00BD} Life", "1_2_life"),
        ("Portal 2", "portal_2"),
    ];
    for (name, expected) in cases {
        assert_eq!(normalize(name).as_deref(), Some(*expected), "name {name:?}");
    }
}

/// Purely numeric names are declined (a bare integer is a row-index selector).
#[test]
fn purely_numeric_name_is_declined() {
    assert_eq!(normalize("2048"), None);
}

/// Whitespace-only falls back to the exe stem, then to `target_<n>`.
#[test]
fn whitespace_only_falls_back() {
    assert_eq!(normalize("   "), None);
    assert_eq!(derive_handle("   ", Some("Game"), 3), "game");
    assert_eq!(derive_handle("   ", None, 3), "target_3");
}

/// A 90-character title truncates to 64 characters with no trailing underscore.
#[test]
fn overlong_title_truncates_and_trims() {
    // 63 'a', then a separator, then more: truncation lands on the separator,
    // which the final trim removes, so the result is 63 characters, no trailing _.
    let name = format!("{}{}{}", "a".repeat(63), " ", "b".repeat(30));
    let handle = normalize(&name).expect("non-empty");
    assert_eq!(handle, "a".repeat(63));
    assert!(handle.len() <= 64);
    assert!(!handle.ends_with('_'));

    // A plain 90-char run truncates to exactly 64.
    let plain = normalize(&"a".repeat(90)).expect("non-empty");
    assert_eq!(plain, "a".repeat(64));
}

/// `Portal 2` registered twice: the second gets `_2` on the new item.
#[test]
fn collision_suffixes_the_new_item() {
    let first = normalize("Portal 2").expect("handle");
    assert_eq!(first, "portal_2");
    let taken = [first.clone()];
    let second = disambiguate(&normalize("Portal 2").expect("handle"), |h| {
        Ok::<_, ()>(taken.iter().any(|t| t == h))
    })
    .unwrap();
    assert_eq!(second, "portal_2_2");
}

/// The & expansion (slice S066, issue #173): the conjunction survives as a word
/// rather than disappearing, and a name with no other special characters is
/// unaffected by the new step.
#[test]
fn ampersand_expands_to_and() {
    assert_eq!(
        normalize("Trapped with Ivy & Piper").as_deref(),
        Some("trapped_with_ivy_and_piper")
    );
    assert_eq!(
        derive_handle("Trapped with Ivy & Piper", None, 1),
        "trapped_with_ivy_and_piper"
    );
    // No surrounding whitespace still splits into separate words.
    assert_eq!(normalize("Ivy&Piper").as_deref(), Some("ivy_and_piper"));
    // A comma-and-colon-bearing name with no ampersand is unaffected by this step.
    assert_eq!(
        normalize("Warhammer 40,000: Dawn of War").as_deref(),
        Some("warhammer_40_000_dawn_of_war")
    );
}

/// A nonspacing mark with canonical combining class 0 is still stripped, proving
/// the strip is by general category (`Mn`), not by combining class.
#[test]
fn mn_is_stripped_by_category_not_combining_class() {
    // U+2DE0 COMBINING CYRILLIC LETTER BE is category Mn with combining class 0.
    let handle = normalize("a\u{2DE0}b").expect("non-empty");
    assert_eq!(handle, "ab");
}
