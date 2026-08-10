// SPDX-License-Identifier: Apache-2.0

//! Process lifecycle observations, and the tree built from them.
//!
//! The vocabulary here is split by what can be wrong about it. A
//! [`ProcessEvent`] is something the platform reported happening; a
//! [`ProcessRecord`] is something the platform said was already true when
//! fragcap started. Specification section 5.3 is why they are not the same
//! kind of thing: a parent identifier is unambiguous at the instant of
//! creation and unreliable at every later instant, because Windows records it
//! and then neither maintains it nor stops reusing the values.
//!
//! [`tree::ProcessTree`] folds both into the ancestry relation of section
//! 10.2. It lives here rather than beside the ETW watcher in `fragcap-attr`
//! because it is a decision over values: it opens nothing, queries nothing, and
//! names no platform type, so the whole of section 10.2 is testable on any
//! machine with no elevation and no game. `interface::select` has the same
//! shape for the same reason.
//!
//! Nothing here requires opening a process handle. Constitution P-1 forbids a
//! handle carrying memory-read rights against a target, and the parent
//! identifier on a [`ProcessEvent`] comes from creation-time ancestry reported
//! by an ETW kernel provider rather than from inspecting a running process.

pub mod tree;

use std::sync::Arc;

use crate::packet::Timestamp;

pub use tree::{Ancestry, ProcessNode, ProcessTree};

/// An operating system process identifier.
///
/// A newtype rather than a bare `u32` because this value recycles and
/// [`tree::NodeId`] does not, and the two are otherwise indistinguishable
/// small integers that the compiler would happily let a caller confuse.
/// Specification section 10.2 turns on that distinction.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProcessId(pub u32);

impl ProcessId {
    pub const fn get(self) -> u32 {
        self.0
    }
}

impl From<u32> for ProcessId {
    fn from(v: u32) -> Self {
        ProcessId(v)
    }
}

/// A process command line, or a record that none could be obtained.
///
/// Deliberately not `Option<Arc<str>>`. An `Option` invites
/// `unwrap_or_default`, which converts "we could not see it" into "it was
/// empty" at one call site and loses the distinction everywhere downstream.
/// Constitution P-9 forbids exactly that substitution, so the type carries the
/// reason for the absence rather than only the absence.
///
/// [`Unavailable`](CommandLine::Unavailable) is not a failure. A process the
/// startup snapshot found cannot yield a command line without a handle
/// carrying memory-read rights, which P-1 forbids, so its absence is the
/// expected and correct outcome for those processes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CommandLine {
    /// Exactly what the platform reported, unaltered.
    Observed(Arc<str>),
    /// No command line could be obtained without a denylisted technique.
    Unavailable,
}

impl CommandLine {
    pub fn observed(s: impl AsRef<str>) -> Self {
        CommandLine::Observed(Arc::from(s.as_ref()))
    }

    /// The command line, or `None` when none was obtained.
    ///
    /// Callers that want a string should match instead. This exists for the
    /// cases that genuinely have nothing to say about an unavailable one, and
    /// it deliberately does not offer a default.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            CommandLine::Observed(s) => Some(s),
            CommandLine::Unavailable => None,
        }
    }

    pub fn is_available(&self) -> bool {
        matches!(self, CommandLine::Observed(_))
    }
}

/// A change in the set of running processes.
///
/// Non-exhaustive because a later platform backend may report a kind of change
/// these two variants do not cover, and adding one must not break every
/// caller.
///
/// **`Started` gained `command_line` in slice S11, which is a breaking change
/// to the variant rather than an additive one.** `#[non_exhaustive]` on an enum
/// permits new variants; it does not permit new fields on an existing variant
/// without breaking every pattern that names the fields. The change is required
/// by specification sections 10.1 and 10.2, which respectively state that the
/// start event carries a command line and that the tree records one, and it is
/// recorded as a deviation in the S11 slice rather than made quietly.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ProcessEvent {
    /// A process was created. `parent` is the creating process at creation
    /// time, which is what makes launcher chain reconstruction possible even
    /// after the parent exits.
    ///
    /// `image` is the full image path, not the file name. Specification
    /// section 10.3 matches the file name with one predicate and the path with
    /// two others, so the path is what gets recorded and
    /// [`ProcessNode::image_name`] derives the rest.
    Started {
        pid: u32,
        parent: u32,
        image: Arc<str>,
        command_line: CommandLine,
        at: Timestamp,
    },
    /// A process exited.
    Exited { pid: u32, at: Timestamp },
}

impl ProcessEvent {
    /// Convenience constructor for a start event with an observed command line.
    pub fn started(
        pid: u32,
        parent: u32,
        image: impl AsRef<str>,
        command_line: impl AsRef<str>,
        at: Timestamp,
    ) -> Self {
        ProcessEvent::Started {
            pid,
            parent,
            image: Arc::from(image.as_ref()),
            command_line: CommandLine::observed(command_line),
            at,
        }
    }

    /// The process this event concerns.
    pub fn pid(&self) -> u32 {
        match self {
            ProcessEvent::Started { pid, .. } => *pid,
            ProcessEvent::Exited { pid, .. } => *pid,
        }
    }

    /// When it happened.
    pub fn at(&self) -> Timestamp {
        match self {
            ProcessEvent::Started { at, .. } => *at,
            ProcessEvent::Exited { at, .. } => *at,
        }
    }
}

/// A process as it stood at the moment of a snapshot.
///
/// Produced by query-only enumeration, which is on the permitted set in
/// specification section 19.2.
///
/// Distinct from [`ProcessEvent::Started`] because its `parent` was read from a
/// running process rather than observed at creation, and section 5.3 says such
/// a value may name an unrelated process or nothing at all. The tree keeps that
/// difference on the node as [`Ancestry`] rather than losing it here.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProcessRecord {
    pub pid: u32,
    pub parent: u32,
    /// The full image path where the platform supplies one. Toolhelp
    /// enumeration supplies only the file name, which is recorded as observed
    /// rather than padded into a path that was never seen.
    pub image: Arc<str>,
    /// Always [`CommandLine::Unavailable`] from the Windows snapshot: reading a
    /// running process's command line means reading its process environment
    /// block, which requires a memory-read right that P-1 forbids. The field
    /// exists so that a platform able to supply one without a denylisted
    /// technique is not blocked by the type.
    pub command_line: CommandLine,
    /// When the process started, when the platform reports it. `None` has a
    /// defined meaning in [`ProcessTree::resolve`]: such a node orders before
    /// every observed event and is selected only when nothing with a known
    /// start time covers the instant asked about.
    pub started: Option<Timestamp>,
}

impl ProcessRecord {
    pub fn new(pid: u32, parent: u32, image: impl AsRef<str>) -> Self {
        ProcessRecord {
            pid,
            parent,
            image: Arc::from(image.as_ref()),
            command_line: CommandLine::Unavailable,
            started: None,
        }
    }

    /// The same record with a start time the platform did supply.
    pub fn started_at(mut self, at: Timestamp) -> Self {
        self.started = Some(at);
        self
    }
}

/// What a [`crate::traits::ProcessWatcher`] observed about its own operation.
///
/// Deliberately not part of [`crate::stats::CaptureStats`]. That structure
/// carries the capture's accounting, and specification section 12.4's
/// conservation identity is asserted over it in every pipeline test. A quantity
/// that is not a packet must not enter that identity, or the one assertion that
/// catches an uncounted discard path stops meaning what it says.
///
/// The shape mirrors [`crate::stats::SourceStats`]: a value a backend produces
/// about itself, which the run's report assembles alongside the capture's own.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WatcherReport {
    /// Events the kernel itself reported dropping. Relayed, never accumulated
    /// into anything else.
    pub events_lost: u64,
    /// Trace buffers the kernel itself reported dropping.
    pub buffers_lost: u64,
    /// Whether the session is still consuming.
    pub running: bool,
}

impl WatcherReport {
    /// Whether anything is known to have been missed.
    ///
    /// A tree built while this was true may have a hole in it, which is why
    /// [`ProcessTree::is_complete`] exists separately: a consumer holding only
    /// the tree must still be able to tell.
    pub fn lost_anything(&self) -> bool {
        self.events_lost > 0 || self.buffers_lost > 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_event_reports_its_pid_and_time() {
        let started = ProcessEvent::started(
            42,
            1,
            "C:\\Program Files\\Zenimax\\eso64.exe",
            "eso64.exe -sessionid 7",
            Timestamp::from_nanos(100),
        );
        assert_eq!(started.pid(), 42);
        assert_eq!(started.at().as_nanos(), 100);

        let exited = ProcessEvent::Exited {
            pid: 42,
            at: Timestamp::from_nanos(200),
        };
        assert_eq!(exited.pid(), 42);
        assert_eq!(exited.at().as_nanos(), 200);
    }

    #[test]
    fn a_record_shares_its_image_name_on_clone() {
        let r = ProcessRecord::new(42, 1, "C:\\Program Files\\Zenimax\\eso64.exe");
        let c = r.clone();
        assert!(Arc::ptr_eq(&r.image, &c.image));
    }

    #[test]
    fn start_time_is_optional_because_not_every_platform_reports_it() {
        assert!(ProcessRecord::new(1, 0, "C:\\Windows\\explorer.exe")
            .started
            .is_none());
    }

    #[test]
    fn a_snapshot_record_has_no_command_line_and_says_so() {
        // Not an empty string. Obtaining one for a running process needs a
        // memory-read right that P-1 forbids, so the absence is the correct
        // outcome and is recorded as an absence.
        let r = ProcessRecord::new(1, 0, "C:\\Windows\\explorer.exe");
        assert_eq!(r.command_line, CommandLine::Unavailable);
        assert_eq!(r.command_line.as_str(), None);
        assert!(!r.command_line.is_available());
    }

    #[test]
    fn an_observed_command_line_is_carried_verbatim() {
        let odd = "app.exe --path \"C:\\Users\\Ünïcødé\\Games\" --flag=\"a b\"";
        let cl = CommandLine::observed(odd);
        assert_eq!(cl.as_str(), Some(odd));
    }

    #[test]
    fn a_watcher_report_defaults_to_having_lost_nothing() {
        let r = WatcherReport::default();
        assert!(!r.lost_anything());
        assert!(!r.running);
    }

    #[test]
    fn a_watcher_report_that_lost_anything_says_so() {
        let r = WatcherReport {
            events_lost: 1,
            ..Default::default()
        };
        assert!(r.lost_anything());
    }
}
