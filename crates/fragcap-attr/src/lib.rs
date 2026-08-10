// SPDX-License-Identifier: Apache-2.0

//! Flow attribution and process tree watching.
//!
//! Kept separate from acquisition by constitution principle P-3.
//!
//! Two attributors live here and both are load-bearing.
//!
//! [`ScriptedAttributor`], from slice S04, answers from a declared script
//! rather than a socket table. That is what makes port reuse and retained
//! attribution testable without a live machine and a stopwatch, and it is what
//! specification section 25.1 means when it claims the pipeline is a
//! deterministic function from fixture input to output. It stays: it is the
//! tier 1 attributor for the corpus and the pipeline tests.
//!
//! [`SocketTableAttributor`], from slice S10, is specification section 11: a
//! socket table snapshot joined against captured flows by 5-tuple, with the
//! cadence of section 11.2, the retention window of section 11.4, and the
//! publication contract of section 11.6. It is the first attributor in the
//! project that can be wrong, which is why the matching rules live on an
//! immutable value in [`index`] and why every one of them is a pure function of
//! a table a test wrote down.
//!
//! # Process watching
//!
//! Slice S11 added the other half, and it answers a different question.
//! Attribution says which process holds a socket; a watcher says which
//! processes exist and which created which. Only the second can be wrong in a
//! way that looks like success, because binding to the wrong member of a
//! launcher chain produces a session that exits zero and contains no gameplay.
//!
//! [`proc_script::ScriptedWatcher`] is to a live watcher what
//! [`ScriptedAttributor`] is to the socket table, and for the same reason: the
//! launcher chains reconnaissance observed replay on any machine, so ancestry
//! is testable without elevation and without a game. The ETW watcher itself is
//! in [`etw`], behind the `etw` feature and off by default.
//!
//! The process tree those events fold into lives in `fragcap-core`, not here.
//! It is a decision over values with no platform surface, and putting it beside
//! the watcher would have gated every test of specification section 10.2 behind
//! an elevated Windows session.
//!
//! Profile stage matching arrives in S12. Until then, an attribution from here
//! carries a process identifier and an image name and no role, and a process
//! node carries a place for a stage and nothing in it.

pub mod etw;
pub mod index;
pub mod proc_script;
pub mod resolver;
pub mod schedule;
pub mod script;
pub mod scripted;
pub mod seam;
pub mod socket;
pub mod table;

#[cfg(all(feature = "socket-table", windows))]
pub mod platform;

pub use etw::WatcherError;
pub use index::{AttributionIndex, MatchRank, PublishedIndex, RetainedEntry, RetentionMap};
pub use proc_script::{ProcessScript, ScriptedWatcher};
pub use resolver::PublishedResolver;
pub use schedule::RefreshSchedule;
pub use script::{AttributionScript, ScriptEntry, ScriptError, Window};
pub use scripted::ScriptedAttributor;
pub use seam::{
    Clock, DeclaredNames, DeclaredTable, ProcessNamer, SocketTableSource, SystemClock, TestClock,
};
pub use socket::{AttributorConfig, SocketTableAttributor};
pub use table::{SocketTable, SocketTableEntry};

#[cfg(all(feature = "socket-table", windows))]
pub use platform::{IpHelperTable, ToolhelpNamer};

#[cfg(all(windows, feature = "etw"))]
pub use etw::EtwWatcher;
