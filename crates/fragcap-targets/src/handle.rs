// SPDX-License-Identifier: Apache-2.0

//! Handle normalization (slice S051), specification section 15.8.
//!
//! A handle is the unique, human-readable selector for a target, derived
//! deterministically from a name. [`normalize`] applies the specified steps in
//! exactly this order: strip Unicode `So`/`Sk`/`Cf`; NFKD; strip `Mn`; lowercase;
//! expand `&` to ` and ` (slice S066, issue #173); delete apostrophes and quotes
//! outright; replace each run outside `[a-z0-9]` with a single `_`; trim leading
//! and trailing `_`; truncate to 64 then trim any trailing `_`. [`derive_handle`]
//! wraps it with the fallback chain (exe stem, then `target_<n>`) so a name that
//! normalizes to nothing still yields a usable handle, and [`disambiguate`]
//! appends `_2`, `_3`, ... on a collision.
//!
//! The `&` expansion is forward-looking only: it changes what a *new* name
//! normalizes to, never an already-derived, already-stored handle. `Trapped with
//! Ivy & Piper` now derives `trapped_with_ivy_and_piper` rather than silently
//! dropping the conjunction (`trapped_with_ivy_piper`), which read as if a word
//! were missing.
//!
//! Two rules make the handle namespace safe for the selector (section 15.8): a
//! handle is never purely numeric (a bare integer is a row-index selector), and a
//! handle is never empty. Both are enforced here and, as defense in depth, by a
//! CHECK in the schema.

use unicode_normalization::UnicodeNormalization;
use unicode_properties::{GeneralCategory, UnicodeGeneralCategory};

/// The maximum handle length, in characters, before the final trailing-`_` trim.
const MAX_LEN: usize = 64;

/// Apply the section 15.8 normalization to a name, returning the handle or `None`
/// when the result is empty or purely numeric (the caller falls back).
///
/// `None` means "unusable, fall back", never an error: normalization does not
/// fail, it declines. A purely numeric result is declined because a bare integer
/// is a row-index selector and a numeric handle would collide with it.
pub fn normalize(name: &str) -> Option<String> {
    // 1. Strip symbol-other (So), symbol-modifier (Sk), and format (Cf). Done
    //    before NFKD so a decorative glyph cannot decompose into digits or
    //    letters that survive (a trademark or registered sign, a degree sign).
    let stripped: String = name
        .chars()
        .filter(|c| {
            !matches!(
                c.general_category(),
                GeneralCategory::OtherSymbol
                    | GeneralCategory::ModifierSymbol
                    | GeneralCategory::Format
            )
        })
        .collect();

    // 2. Compatibility decomposition (NFKD): a Roman numeral becomes ASCII
    //    letters, a vulgar fraction becomes digits and a fraction slash, an
    //    accented letter splits into a base and a combining mark.
    let decomposed: String = stripped.nfkd().collect();

    // 3. Strip nonspacing marks (Mn): the combining marks NFKD just produced (an
    //    accent), by category rather than by combining class so a class-0 Mn is
    //    still removed.
    let no_marks: String = decomposed
        .chars()
        .filter(|c| c.general_category() != GeneralCategory::NonspacingMark)
        .collect();

    // 4. Lowercase.
    let lowered = no_marks.to_lowercase();

    // 5. Expand & to " and " (slice S066, issue #173), so the conjunction survives
    //    as a word rather than either disappearing (the pre-S066 behavior, since
    //    step 7 would otherwise collapse it to a single separator) or gluing two
    //    words together. Padded with spaces on both sides so "Ivy&Piper" (no
    //    surrounding whitespace) still splits into separate words.
    let expanded = lowered.replace('&', " and ");

    // 6. Delete apostrophes and quotation marks outright, so a possessive joins
    //    rather than splitting ("clancys", not "clancy_s").
    let deapostrophed: String = expanded
        .chars()
        .filter(|c| !is_apostrophe_or_quote(*c))
        .collect();

    // 7. Replace each maximal run of characters outside [a-z0-9] with a single _.
    let mut collapsed = String::with_capacity(deapostrophed.len());
    let mut in_gap = false;
    for c in deapostrophed.chars() {
        if c.is_ascii_lowercase() || c.is_ascii_digit() {
            collapsed.push(c);
            in_gap = false;
        } else if !in_gap {
            collapsed.push('_');
            in_gap = true;
        }
    }

    // 8. Trim leading and trailing underscores.
    let trimmed = collapsed.trim_matches('_');

    // 9. Truncate to MAX_LEN characters, then trim any trailing underscore the cut
    //    left behind, so no handle ends in '_'.
    let truncated: String = trimmed.chars().take(MAX_LEN).collect();
    let handle = truncated.trim_end_matches('_').to_string();

    if handle.is_empty() || is_purely_numeric(&handle) {
        None
    } else {
        Some(handle)
    }
}

/// Derive a handle from a name, falling back to the executable stem and then to
/// `target_<n>` when normalization declines. Always terminates and always yields a
/// valid handle.
///
/// `exe_stem` is the launch executable's file stem when one is known (used when
/// the name normalizes to nothing, as for a whitespace-only or purely-symbolic
/// name). `index` seeds the final `target_<n>` fallback so it is stable for a
/// given registration position.
pub fn derive_handle(name: &str, exe_stem: Option<&str>, index: u64) -> String {
    if let Some(h) = normalize(name) {
        return h;
    }
    if let Some(stem) = exe_stem {
        if let Some(h) = normalize(stem) {
            return h;
        }
    }
    // Last resort: never empty, never numeric (the prefix guarantees both).
    format!("target_{index}")
}

/// Resolve a base handle against existing handles, appending `_2`, `_3`, ... until
/// it is unique. The suffix lands on the new item; an existing entry is never
/// touched, and the result never exceeds the 64-character limit.
///
/// `exists` reports whether a candidate handle is already taken, and may fail: a
/// store error propagates rather than being swallowed into a false "free", which
/// would risk a duplicate insert or hide a real fault. The base is returned
/// unchanged when free.
pub fn disambiguate<E>(
    base: &str,
    mut exists: impl FnMut(&str) -> Result<bool, E>,
) -> Result<String, E> {
    if !exists(base)? {
        return Ok(base.to_string());
    }
    let mut n: u64 = 2;
    loop {
        let candidate = with_suffix(base, &format!("_{n}"));
        if !exists(&candidate)? {
            return Ok(candidate);
        }
        n += 1;
    }
}

/// Append a collision suffix while keeping the whole handle within [`MAX_LEN`]
/// characters: truncate the base on a character boundary to leave room, then trim
/// any trailing `_` the cut leaves, so a suffixed handle is never longer than one
/// normalization itself could produce.
fn with_suffix(base: &str, suffix: &str) -> String {
    let room = MAX_LEN.saturating_sub(suffix.chars().count());
    let mut kept: String = base.chars().take(room).collect();
    while kept.ends_with('_') {
        kept.pop();
    }
    format!("{kept}{suffix}")
}

/// Validate a user-supplied handle override under the same rules as a derived
/// handle: non-empty, not purely numeric, and already in normalized shape (equal
/// to its own normalization). Returns the accepted handle or an explanation.
pub fn validate_override(candidate: &str) -> Result<String, String> {
    if candidate.is_empty() {
        return Err("handle may not be empty".to_string());
    }
    if is_purely_numeric(candidate) {
        return Err("handle may not be purely numeric".to_string());
    }
    match normalize(candidate) {
        Some(n) if n == candidate => Ok(n),
        Some(n) => Err(format!(
            "handle {candidate:?} is not in normalized form (would normalize to {n:?})"
        )),
        None => Err(format!("handle {candidate:?} is not a valid handle")),
    }
}

/// Whether a string is a non-empty run of ASCII digits.
fn is_purely_numeric(s: &str) -> bool {
    !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit())
}

/// Whether a character is an apostrophe or quotation mark to be deleted outright.
/// Covers the ASCII forms and the common typographic single and double quotes.
fn is_apostrophe_or_quote(c: char) -> bool {
    matches!(
        c,
        '\'' | '"'
            | '\u{2018}'
            | '\u{2019}'
            | '\u{201C}'
            | '\u{201D}'
            | '\u{2032}'
            | '\u{2033}'
            | '`'
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn purely_numeric_is_declined() {
        assert_eq!(normalize("2048"), None);
    }

    #[test]
    fn whitespace_only_is_declined_then_falls_back() {
        assert_eq!(normalize("   "), None);
        // exe_stem is the stem (no extension); the CLI strips it via Path::file_stem.
        assert_eq!(derive_handle("   ", Some("Game"), 7), "game");
        assert_eq!(derive_handle("   ", None, 7), "target_7");
    }

    #[test]
    fn disambiguate_suffixes_the_new_item() {
        let taken = ["portal_2"];
        let out = disambiguate("portal_2", |h| Ok::<_, ()>(taken.contains(&h))).unwrap();
        assert_eq!(out, "portal_2_2");
    }

    #[test]
    fn disambiguate_propagates_a_lookup_error() {
        let out = disambiguate("portal_2", |_h| Err::<bool, _>("store down"));
        assert_eq!(out, Err("store down"));
    }

    #[test]
    fn collision_suffix_stays_within_the_limit() {
        // A base already at the 64-character limit must not grow past it when a
        // suffix is appended: the base is truncated to make room.
        let base = "a".repeat(64);
        let taken = [base.clone()];
        let out = disambiguate(&base, |h| Ok::<_, ()>(taken.iter().any(|t| t == h))).unwrap();
        assert!(
            out.chars().count() <= 64,
            "got {} chars",
            out.chars().count()
        );
        assert!(out.ends_with("_2"));
    }

    #[test]
    fn override_must_be_normalized_and_non_numeric() {
        assert!(validate_override("my_game").is_ok());
        assert!(validate_override("123").is_err());
        assert!(validate_override("Not Normalized").is_err());
        assert!(validate_override("").is_err());
    }
}
