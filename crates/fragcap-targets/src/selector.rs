// SPDX-License-Identifier: Apache-2.0

//! Selector resolution (slice S051), specification section 15.8.
//!
//! A selector resolves to at most one target. A bare integer is a 1-based row
//! index over the most recent listing snapshot (slice S055), so a number names the
//! row the user last saw even after an intervening add or remove; any other token
//! is an exact handle first, then a case-insensitive exact name; `--id <n>` is the
//! explicit, durable, machine-facing form that resolves by stable identifier (or a
//! superseded alias).
//!
//! A name that matches more than one target is a [`Selection::Ambiguous`]: the
//! caller must print the matches and refuse to guess (P-9), never pick one. A name
//! that matches nothing is a clean [`Selection::NoMatch`], distinct from ambiguity.

use crate::entry::TargetEntry;
use crate::store::Store;
use crate::TargetsError;

/// The outcome of resolving a selector.
#[derive(Debug)]
pub enum Selection {
    /// Exactly one target resolved.
    Resolved(Box<TargetEntry>),
    /// No target matched (distinct from ambiguity).
    NoMatch,
    /// More than one target matched a name; the caller must not choose one.
    Ambiguous(Vec<TargetEntry>),
}

/// Resolve a positional selector: a bare integer is a 1-based row index over the
/// most recent listing snapshot; any other token is an exact handle, then a
/// case-insensitive exact name.
///
/// The row-index branch resolves through the snapshot (position -> stable_id ->
/// entry), so `n` names the row the user last saw. A position past the snapshot,
/// one whose target was since removed, or a bare integer with no snapshot written
/// yet, is a [`Selection::NoMatch`] on the row-index path (callers map that to a
/// usage error for a bare integer).
pub fn resolve_positional(store: &Store, token: &str) -> Result<Selection, TargetsError> {
    // A bare integer is always the row-index path, never a handle or name: a numeric
    // token that is out of range (or `"0"`, which is not a valid 1-based position) is
    // a clean row-index no-match, which the caller maps to a usage error. It must not
    // fall through to a name lookup, or `0` would read as a name miss (exit 0)
    // instead of an invalid position (exit 2).
    if is_row_index(token) {
        let resolved = match parse_position(token) {
            Some(position) => match store.listing_snapshot_nth(position)? {
                Some(stable_id) => store.target_by_stable_id(stable_id)?,
                None => None,
            },
            None => None,
        };
        return Ok(match resolved {
            Some(t) => Selection::Resolved(Box::new(t)),
            None => Selection::NoMatch,
        });
    }
    if let Some(t) = store.target_by_handle(token)? {
        return Ok(Selection::Resolved(Box::new(t)));
    }
    let by_name = store.targets_by_name(token)?;
    Ok(match by_name.len() {
        0 => Selection::NoMatch,
        1 => Selection::Resolved(Box::new(by_name.into_iter().next().expect("len 1"))),
        _ => Selection::Ambiguous(by_name),
    })
}

/// Resolve an explicit `--id` selector by stable identifier, consulting superseded
/// aliases so a former id still lands on the merged entry.
pub fn resolve_id(store: &Store, id: i64) -> Result<Selection, TargetsError> {
    Ok(match store.target_by_stable_id(id)? {
        Some(t) => Selection::Resolved(Box::new(t)),
        None => Selection::NoMatch,
    })
}

/// Whether a token is a bare integer row index: purely numeric and non-empty,
/// including `"0"`. Handles are never purely numeric (enforced at derivation and by
/// the schema CHECK), so a numeric token never shadows a handle. This is the single
/// source for the row-index predicate the CLI dispatch also consults, so the
/// dispatch's miss-exit choice and this resolution agree that every numeric token is
/// the row-index path (an invalid position, `"0"` included, is a usage error, not a
/// clean name miss).
pub fn is_row_index(token: &str) -> bool {
    !token.is_empty() && token.bytes().all(|b| b.is_ascii_digit())
}

/// The valid 1-based position a numeric token names, or `None` when it is out of
/// range for a position (`"0"`, or a value that does not parse). A non-numeric token
/// is not a position either; callers gate on [`is_row_index`] first.
fn parse_position(token: &str) -> Option<usize> {
    if !is_row_index(token) {
        return None;
    }
    match token.parse::<usize>().ok() {
        Some(n) if n >= 1 => Some(n),
        _ => None,
    }
}
