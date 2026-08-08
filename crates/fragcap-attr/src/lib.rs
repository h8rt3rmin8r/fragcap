// SPDX-License-Identifier: Apache-2.0

//! Flow attribution and process tree watching.
//!
//! Kept separate from acquisition by constitution principle P-3.
//!
//! Slice S04 filled the testing half. [`scripted::ScriptedAttributor`] answers
//! from a declared script rather than a socket table, which is what makes port
//! reuse and retained attribution testable without a live machine and a
//! stopwatch. It matches through the same key derivation and wildcard bind rule
//! specification section 8.4 defines, so a test that passes against a script is
//! one the real attributor has to satisfy.
//!
//! The socket table attributor arrives in S10 and the process watcher in S11.

pub mod script;
pub mod scripted;

pub use script::{AttributionScript, ScriptEntry, ScriptError, Window};
pub use scripted::ScriptedAttributor;
