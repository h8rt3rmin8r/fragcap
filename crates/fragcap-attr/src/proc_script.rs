// SPDX-License-Identifier: Apache-2.0

//! A [`ProcessWatcher`] that publishes a declared sequence of events.
//!
//! The offline half of slice S11, and the reason specification section 25.1's
//! testability claim survives contact with process observation. It mirrors
//! [`crate::scripted::ScriptedAttributor`] from S04 exactly: a declared script
//! stands in for a live source, so the consumer is exercised without the
//! machinery the consumer does not own.
//!
//! What makes this worth having rather than merely convenient is that both
//! watchers feed the same [`ProcessTree`](fragcap_core::ProcessTree). A tree
//! built from a script and a tree built from ETW go through one `apply`, so a
//! test that passes here states something the real watcher must also satisfy.
//!
//! Available on every target and behind no feature. The whole of specification
//! section 10.2 is exercised through this on a machine with no elevation, no
//! capture driver, and no game.
//!
//! The script is built in code rather than parsed from a file. S04's
//! attribution script has a text format because a committed fixture corpus
//! needed one; a process script has two users so far, both of them the
//! launcher chains in this crate's own tests, and a file format for two
//! in-code callers would be speculative until S12 shows what a stage matcher
//! needs to be tested against.

use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Mutex;

use fragcap_core::packet::Timestamp;
use fragcap_core::process::{CommandLine, ProcessEvent, ProcessRecord};
use fragcap_core::traits::ProcessWatcher;

/// A declared sequence of process events, plus the startup snapshot that would
/// have accompanied them.
#[derive(Clone, Debug, Default)]
pub struct ProcessScript {
    events: Vec<ProcessEvent>,
    snapshot: Vec<ProcessRecord>,
}

impl ProcessScript {
    pub fn new() -> Self {
        Self::default()
    }

    /// The processes that were already running when the watcher started.
    pub fn with_snapshot(mut self, records: Vec<ProcessRecord>) -> Self {
        self.snapshot = records;
        self
    }

    /// A process creation, with its command line observed.
    pub fn started(
        mut self,
        pid: u32,
        parent: u32,
        image: &str,
        command_line: &str,
        at: i64,
    ) -> Self {
        self.events.push(ProcessEvent::started(
            pid,
            parent,
            image,
            command_line,
            Timestamp::from_nanos(at),
        ));
        self
    }

    /// A process creation whose command line the platform did not supply.
    ///
    /// Rare from a start event and ordinary from a snapshot, but expressible
    /// here so that a consumer's handling of the case can be tested rather than
    /// assumed.
    pub fn started_without_cmdline(mut self, pid: u32, parent: u32, image: &str, at: i64) -> Self {
        self.events.push(ProcessEvent::Started {
            pid,
            parent,
            image: image.into(),
            command_line: CommandLine::Unavailable,
            at: Timestamp::from_nanos(at),
        });
        self
    }

    pub fn exited(mut self, pid: u32, at: i64) -> Self {
        self.events.push(ProcessEvent::Exited {
            pid,
            at: Timestamp::from_nanos(at),
        });
        self
    }

    pub fn events(&self) -> &[ProcessEvent] {
        &self.events
    }

    pub fn snapshot(&self) -> &[ProcessRecord] {
        &self.snapshot
    }
}

/// Publishes a [`ProcessScript`] to any number of subscribers.
#[derive(Debug)]
pub struct ScriptedWatcher {
    script: ProcessScript,
    subscribers: Mutex<Vec<Sender<ProcessEvent>>>,
}

impl ScriptedWatcher {
    pub fn new(script: ProcessScript) -> Self {
        ScriptedWatcher {
            script,
            subscribers: Mutex::new(Vec::new()),
        }
    }

    /// Publish every event in the script, in order, to every subscriber.
    ///
    /// A subscriber whose receiver has been dropped is removed rather than
    /// treated as a failure. Nothing is discarded by that: an event nobody is
    /// listening for was never received, which is a different thing from an
    /// event received and thrown away, and only the second would need a counter
    /// under P-4.
    pub fn play(&self) {
        let mut subs = self.subscribers.lock().expect("subscriber lock");
        for event in &self.script.events {
            subs.retain(|s| s.send(event.clone()).is_ok());
        }
    }

    pub fn script(&self) -> &ProcessScript {
        &self.script
    }
}

impl ProcessWatcher for ScriptedWatcher {
    fn subscribe(&self) -> Receiver<ProcessEvent> {
        let (tx, rx) = channel();
        self.subscribers.lock().expect("subscriber lock").push(tx);
        rx
    }

    fn snapshot(&self) -> Vec<ProcessRecord> {
        self.script.snapshot.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fragcap_core::ProcessTree;

    fn two_events() -> ProcessScript {
        ProcessScript::new()
            .started(10, 4, "C:\\Windows\\explorer.exe", "explorer.exe", 1)
            .started(20, 10, "C:\\Steam\\steam.exe", "steam.exe -silent", 2)
    }

    #[test]
    fn a_script_publishes_its_events_in_order() {
        let w = ScriptedWatcher::new(two_events());
        let rx = w.subscribe();
        w.play();

        let got: Vec<_> = rx.try_iter().collect();
        assert_eq!(got.len(), 2);
        assert_eq!(got[0].pid(), 10);
        assert_eq!(got[1].pid(), 20);
    }

    #[test]
    fn every_subscriber_gets_an_independent_stream() {
        let w = ScriptedWatcher::new(two_events());
        let a = w.subscribe();
        let b = w.subscribe();
        w.play();

        assert_eq!(a.try_iter().count(), 2);
        assert_eq!(b.try_iter().count(), 2);
    }

    #[test]
    fn a_subscriber_sees_only_what_was_published_after_it_subscribed() {
        let w = ScriptedWatcher::new(two_events());
        let early = w.subscribe();
        w.play();
        let late = w.subscribe();

        assert_eq!(early.try_iter().count(), 2);
        assert_eq!(late.try_iter().count(), 0);
    }

    #[test]
    fn a_dropped_receiver_does_not_stop_the_others() {
        let w = ScriptedWatcher::new(two_events());
        let gone = w.subscribe();
        let kept = w.subscribe();
        drop(gone);
        w.play();

        assert_eq!(kept.try_iter().count(), 2);
    }

    #[test]
    fn the_snapshot_is_what_the_script_declared() {
        let w = ScriptedWatcher::new(
            ProcessScript::new().with_snapshot(vec![ProcessRecord::new(4, 0, "System")]),
        );
        let snap = w.snapshot();
        assert_eq!(snap.len(), 1);
        assert_eq!(snap[0].pid, 4);
        assert_eq!(snap[0].command_line, CommandLine::Unavailable);
    }

    #[test]
    fn a_start_without_a_command_line_says_so_rather_than_saying_empty() {
        let w = ScriptedWatcher::new(ProcessScript::new().started_without_cmdline(
            10,
            4,
            "C:\\a.exe",
            1,
        ));
        let rx = w.subscribe();
        w.play();

        let mut t = ProcessTree::new();
        for e in rx.try_iter() {
            t.apply(e);
        }
        let n = t.nodes().next().unwrap();
        assert_eq!(n.command_line(), &CommandLine::Unavailable);
        assert_eq!(n.command_line().as_str(), None);
    }

    #[test]
    fn a_watcher_is_usable_behind_the_trait_object_the_pipeline_will_hold() {
        let w: Box<dyn ProcessWatcher> = Box::new(ScriptedWatcher::new(two_events()));
        let rx = w.subscribe();
        // `play` is not on the trait, which is correct: publishing is the
        // scripted watcher's own affair and a live one publishes by itself.
        assert!(rx.try_recv().is_err());
        assert!(w.snapshot().is_empty());
    }
}
