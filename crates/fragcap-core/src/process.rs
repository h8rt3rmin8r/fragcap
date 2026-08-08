// SPDX-License-Identifier: Apache-2.0

//! Process lifecycle observations.
//!
//! Both types here are the minimal shape the [`crate::traits::ProcessWatcher`]
//! signature requires. Slice S11 owns the ETW process watcher and the process
//! tree, and will extend them.
//!
//! Neither type carries anything that would require opening a process handle.
//! Constitution P-1 forbids a handle carrying memory-read rights against a
//! target, and the parent identifier here comes from creation-time ancestry
//! reported by an ETW kernel provider rather than from inspecting a running
//! process.

use std::sync::Arc;

use crate::packet::Timestamp;

/// A change in the set of running processes.
///
/// Non-exhaustive because slice S11 will add at least image path and command
/// line availability events, and adding a variant must not break every caller.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ProcessEvent {
    /// A process was created. `parent` is the creating process at creation
    /// time, which is what makes launcher chain reconstruction possible even
    /// after the parent exits.
    Started {
        pid: u32,
        parent: u32,
        image: Arc<str>,
        at: Timestamp,
    },
    /// A process exited.
    Exited { pid: u32, at: Timestamp },
}

impl ProcessEvent {
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
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProcessRecord {
    pub pid: u32,
    pub parent: u32,
    pub image: Arc<str>,
    /// When the process started, when the platform reports it.
    pub started: Option<Timestamp>,
}

impl ProcessRecord {
    pub fn new(pid: u32, parent: u32, image: impl AsRef<str>) -> Self {
        ProcessRecord {
            pid,
            parent,
            image: Arc::from(image.as_ref()),
            started: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_event_reports_its_pid_and_time() {
        let started = ProcessEvent::Started {
            pid: 42,
            parent: 1,
            image: Arc::from("eso64.exe"),
            at: Timestamp::from_nanos(100),
        };
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
        let r = ProcessRecord::new(42, 1, "eso64.exe");
        let c = r.clone();
        assert!(Arc::ptr_eq(&r.image, &c.image));
    }

    #[test]
    fn start_time_is_optional_because_not_every_platform_reports_it() {
        assert!(ProcessRecord::new(1, 0, "x.exe").started.is_none());
    }
}
