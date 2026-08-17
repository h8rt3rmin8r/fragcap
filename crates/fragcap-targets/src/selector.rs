// SPDX-License-Identifier: Apache-2.0

//! Selector resolution (slice S051), specification section 15.8.
//!
//! A selector resolves to at most one target. A bare integer is an ephemeral,
//! 1-based row index over the current listing; any other token is an exact handle
//! first, then a case-insensitive exact name; `--id <n>` is the explicit, durable,
//! machine-facing form that resolves by stable identifier (or a superseded alias).
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

/// Resolve a positional selector: a bare integer is a 1-based ephemeral row index
/// over the current listing; any other token is an exact handle, then a
/// case-insensitive exact name.
pub fn resolve_positional(store: &Store, token: &str) -> Result<Selection, TargetsError> {
    if let Some(index) = parse_row_index(token) {
        let all = store.targets()?;
        return Ok(match all.into_iter().nth(index) {
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

/// A purely numeric token is a 1-based row index, returned zero-based. Handles are
/// never purely numeric (enforced at derivation and by the schema CHECK), so a
/// numeric token never shadows a handle. `"0"` and non-numeric tokens return
/// `None`.
fn parse_row_index(token: &str) -> Option<usize> {
    if !token.is_empty() && token.bytes().all(|b| b.is_ascii_digit()) {
        token.parse::<usize>().ok().and_then(|n| n.checked_sub(1))
    } else {
        None
    }
}
