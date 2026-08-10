// SPDX-License-Identifier: Apache-2.0

//! Game profile schema, parsing, validation, and matching.
//!
//! A profile is a declarative description of one game's process topology and
//! capture defaults, per specification section 15. It is the mechanism by which
//! fragcap supports a specific title without containing knowledge of it: adding
//! support for a game means writing a TOML file, and never means modifying
//! Rust.
//!
//! Slice S05 built the schema (section 15.2), the resolution order (section
//! 15.3), and the validation set (section 15.4). Slice S12 added [`matching`],
//! which evaluates a profile's stage predicates against an observed process tree
//! and binds each process to its stage.
//!
//! # Being wrong well
//!
//! Most of this crate is about refusing a bad profile precisely. Two properties
//! carry it.
//!
//! **Every problem is reported at once.** Section 15.4 requires it, and the
//! reason is the authoring loop: a validator that stops at the first fault turns
//! a profile with four mistakes into four edit-run cycles. Nothing on a
//! diagnostic path uses `?`; faults accumulate into a [`Diagnostics`] set.
//!
//! **A [`Profile`] cannot exist in an invalid state.** [`Profile::parse`]
//! returns either a validated profile or the complete diagnostic set, and there
//! is no public constructor past it. Section 15.4's requirement that validation
//! run before every capture then costs nothing to honor and cannot be forgotten
//! by a later caller.
//!
//! # Two dependencies, and why these two
//!
//! S04, S06, and S07 hand-rolled pcap, pcapng, and JSON Lines, so a reader
//! arriving here is right to ask why TOML is not hand-rolled too. The
//! difference is the direction the bytes travel. Those three formats were the
//! deliverable, produced by fragcap or by a tool, and hand-rolling gave
//! verification something independent to judge against. A profile is a file a
//! contributor typed, and a hand-rolled subset would refuse legal TOML that an
//! author's editor produced, which is exactly the failure section 15.1's promise
//! cannot survive.
//!
//! `regex` is here because section 15.4 requires compiling `path_regex`, and it
//! must be the engine that later evaluates it: validating with one engine and
//! matching with another would let a pattern pass validation and fail during a
//! capture.
//!
//! The `exe` glob is hand-rolled despite that, because section 15.4 needs glob
//! intersection rather than glob matching and no crate offers it. See [`glob`]
//! and slice S05 research R-2.
//!
//! # What this crate does not do
//!
//! It observes nothing. A profile describes processes; reading the description
//! opens no handle, enumerates no process, and reads no socket table.
//! Constitution P-1 is not engaged here and must stay that way: a convenience
//! check on whether a declared image name exists on this machine would engage
//! it.

pub mod diagnostic;
pub mod glob;
pub mod matching;
pub mod parse;
pub mod resolve;
pub mod schema;
pub mod validate;

pub use diagnostic::{Diagnostic, DiagnosticCode, Diagnostics, Position};
pub use glob::{ImagePattern, PatternError, MAX_PATTERN_CHARS};
pub use parse::{load, LoadError, MAX_PROFILE_BYTES, MAX_STAGES};
pub use resolve::{
    resolve, BundledSet, DuplicateGameId, ProfileSource, ResolveError, Resolved, SearchPath,
};
pub use schema::{
    CaptureDefaults, CaptureMode, Game, GameId, Lifecycle, MatchPredicates, PathRegex, Profile,
    Stage, SCHEMA_VERSION,
};
