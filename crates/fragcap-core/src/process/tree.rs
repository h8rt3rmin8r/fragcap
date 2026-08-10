// SPDX-License-Identifier: Apache-2.0

//! The process tree of specification section 10.2.
//!
//! A fold over [`ProcessEvent`] values and [`ProcessRecord`] snapshots. It
//! opens nothing, queries nothing, and names no platform type, which is what
//! makes the whole of section 10.2 testable on any machine with no elevation
//! and no game running.
//!
//! Three properties are the reason this exists rather than a map from process
//! identifier to parent.
//!
//! **Identifiers recycle and identities do not.** A [`NodeId`] is issued once
//! per observed process and never reused within a session. A [`ProcessId`] is
//! whatever Windows had lying around. Two processes may share the second if
//! their lifetimes do not overlap, and when they do they are two nodes, neither
//! inheriting the other's children.
//!
//! **Exited nodes are retained.** Section 10.2 requires it and the reason is
//! attribution: a packet may arrive after the process that sent it has gone,
//! and a tree that forgot the process cannot name it. Nothing here removes a
//! node.
//!
//! **How a parent was learned is carried, not derived.** See [`Ancestry`].

use std::collections::HashMap;
use std::sync::Arc;

use super::{CommandLine, ProcessEvent, ProcessId, ProcessRecord};
use crate::attribution::StageId;
use crate::packet::Timestamp;

/// The session-local identity of a process.
///
/// Issued by the tree, monotonic, and never reused within a session, which is
/// exactly what a [`ProcessId`] is not. Section 10.2 turns on the distinction:
/// the synthetic identifier is the node's identity, while the pair of operating
/// system identifier and timestamp is the lookup key from the platform's
/// vocabulary into the tree.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeId(u32);

impl NodeId {
    pub const fn get(self) -> u32 {
        self.0
    }

    fn index(self) -> usize {
        self.0 as usize
    }
}

/// Where a node's parent came from.
///
/// Stored on the node rather than derived from whether a parent resolved,
/// because the two questions have different answers. Slice S06 learned this
/// about attribution fidelity, which it first derived from whether an
/// attribution existed, and which review caught claiming a live socket-table
/// hit for a resolution that had come from a text file.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ancestry {
    /// The parent was carried on a creation event, recorded at the instant of
    /// creation. Specification section 5.3: the only instant at which the
    /// relationship is unambiguous.
    Observed,
    /// The parent was read from a running process during the startup snapshot.
    /// Section 5.3 again: such a value may name an unrelated process, or a
    /// process that no longer exists, because Windows records a parent
    /// identifier and then neither maintains it nor stops reusing the values.
    Snapshot,
    /// No parent could be resolved in this tree. The observed parent identifier
    /// is still on the node as [`ProcessNode::parent_pid`]; what is missing is
    /// a node to attach it to, usually because the parent was created before
    /// fragcap started watching.
    Unresolved,
}

/// One process in the tree.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProcessNode {
    id: NodeId,
    pid: ProcessId,
    parent_pid: ProcessId,
    parent: Option<NodeId>,
    ancestry: Ancestry,
    image: Arc<str>,
    command_line: CommandLine,
    started: Option<Timestamp>,
    exited: Option<Timestamp>,
    stage: Option<StageId>,
}

impl ProcessNode {
    pub fn id(&self) -> NodeId {
        self.id
    }

    pub fn pid(&self) -> ProcessId {
        self.pid
    }

    /// The parent identifier as observed, whether or not it resolved to a node.
    ///
    /// Kept because it is an observation, and P-9 does not permit discarding
    /// one merely because nothing downstream could use it.
    pub fn parent_pid(&self) -> ProcessId {
        self.parent_pid
    }

    pub fn parent(&self) -> Option<NodeId> {
        self.parent
    }

    pub fn ancestry(&self) -> Ancestry {
        self.ancestry
    }

    /// The full image path, as observed.
    pub fn image(&self) -> &str {
        &self.image
    }

    /// The image file name, derived from the path.
    ///
    /// Specification section 10.3 matches the file name with its `exe`
    /// predicate and the full path with `path_contains` and `path_regex`, so
    /// one recorded value has to serve both. Deriving rather than storing means
    /// the two can never disagree.
    pub fn image_name(&self) -> &str {
        self.image.rsplit(['\\', '/']).next().unwrap_or(&self.image)
    }

    pub fn command_line(&self) -> &CommandLine {
        &self.command_line
    }

    /// When the process started, or `None` when the platform did not report it.
    ///
    /// `None` is not "at the start of the session". See
    /// [`ProcessTree::resolve`] for what it means in a lookup.
    pub fn started(&self) -> Option<Timestamp> {
        self.started
    }

    pub fn exited(&self) -> Option<Timestamp> {
        self.exited
    }

    pub fn is_live(&self) -> bool {
        self.exited.is_none()
    }

    /// The profile stage this node is bound to.
    ///
    /// Always `None` in slice S11. Specification sections 10.3 and 10.4 are
    /// S12's, and this is the place they write to. The field is reserved here
    /// rather than added there so that the node's shape does not change under a
    /// consumer that has already been written against it.
    pub fn stage(&self) -> Option<&StageId> {
        self.stage.as_ref()
    }

    /// Whether this node's lifetime contains `at`.
    ///
    /// A node with an unknown start time is treated as having started before
    /// anything observed, which is the only assumption that is safe: it was
    /// found already running.
    fn contains(&self, at: Timestamp) -> bool {
        let after_start = match self.started {
            Some(s) => at >= s,
            None => true,
        };
        let before_end = match self.exited {
            Some(e) => at <= e,
            None => true,
        };
        after_start && before_end
    }
}

/// The nodes, the ancestry relation over them, and what is known to be missing.
#[derive(Clone, Debug, Default)]
pub struct ProcessTree {
    nodes: Vec<ProcessNode>,
    by_pid: HashMap<ProcessId, Vec<NodeId>>,
    /// Exits whose start event has not arrived. A trace consumer delivers from
    /// several buffers and does not order events by timestamp across them, so
    /// an exit before its start is ordinary rather than pathological.
    pending_exits: HashMap<ProcessId, Vec<Timestamp>>,
    events_lost: u64,
}

impl ProcessTree {
    pub fn new() -> Self {
        Self::default()
    }

    // -- folding ----------------------------------------------------------

    /// Fold one observed event.
    pub fn apply(&mut self, event: ProcessEvent) {
        match event {
            ProcessEvent::Started {
                pid,
                parent,
                image,
                command_line,
                at,
            } => self.apply_started(ProcessId(pid), ProcessId(parent), image, command_line, at),
            ProcessEvent::Exited { pid, at } => self.apply_exited(ProcessId(pid), at),
        }
    }

    /// Fold the startup snapshot.
    ///
    /// Reconciles against nodes already present rather than adding a second
    /// node for a process the event stream has already reported. The watcher
    /// subscribes before it snapshots, so a process created during startup
    /// appears in both, and one node is the required outcome.
    pub fn apply_snapshot(&mut self, records: &[ProcessRecord]) {
        for r in records {
            let pid = ProcessId(r.pid);
            if self.live_node_for(pid).is_some() {
                // Already known from the event stream, which carries
                // creation-time ancestry. Keep the stronger of the two.
                continue;
            }
            let parent_pid = ProcessId(r.parent);
            let parent = r.started.and_then(|s| self.resolve(parent_pid, s)).or_else(
                // A snapshot record often has no start time, and the parent it
                // names is usually older than anything observed. Resolve
                // against the parent's own live node in that case.
                || self.live_node_for(parent_pid),
            );
            let ancestry = if parent.is_some() {
                Ancestry::Snapshot
            } else {
                Ancestry::Unresolved
            };
            self.push(ProcessNode {
                id: NodeId(self.nodes.len() as u32),
                pid,
                parent_pid,
                parent,
                ancestry,
                image: Arc::clone(&r.image),
                command_line: r.command_line.clone(),
                started: r.started,
                exited: None,
                stage: None,
            });
            self.join_pending_exit(pid);
        }
    }

    /// Record that the platform reported losing events.
    ///
    /// Once called with a non-zero count the tree is permanently incomplete,
    /// because a lost start event removes a node and silently orphans
    /// everything beneath it. That is not recoverable by observing more.
    pub fn note_lost(&mut self, events: u64) {
        self.events_lost = self.events_lost.saturating_add(events);
    }

    fn apply_started(
        &mut self,
        pid: ProcessId,
        parent_pid: ProcessId,
        image: Arc<str>,
        command_line: CommandLine,
        at: Timestamp,
    ) {
        // Reconciliation in the other arrival order: the snapshot got here
        // first and its node is weaker in every field. Upgrade it in place
        // rather than adding a second node for one process.
        if let Some(existing) = self.live_node_for(pid) {
            if self.nodes[existing.index()].ancestry == Ancestry::Snapshot
                || self.nodes[existing.index()].started.is_none()
            {
                let parent = self.resolve_parent_for(existing, parent_pid, at);
                let n = &mut self.nodes[existing.index()];
                n.parent_pid = parent_pid;
                n.parent = parent;
                n.ancestry = if parent.is_some() {
                    Ancestry::Observed
                } else {
                    Ancestry::Unresolved
                };
                n.image = image;
                n.command_line = command_line;
                n.started = Some(at);
                return;
            }
        }

        let id = NodeId(self.nodes.len() as u32);
        let parent = self.resolve(parent_pid, at);
        let ancestry = if parent.is_some() {
            Ancestry::Observed
        } else {
            Ancestry::Unresolved
        };
        self.push(ProcessNode {
            id,
            pid,
            parent_pid,
            parent,
            ancestry,
            image,
            command_line,
            started: Some(at),
            exited: None,
            stage: None,
        });
        self.join_pending_exit(pid);
    }

    fn apply_exited(&mut self, pid: ProcessId, at: Timestamp) {
        match self.live_node_for(pid) {
            Some(id) => self.nodes[id.index()].exited = Some(at),
            // Held, not counted. The start may still arrive.
            None => self.pending_exits.entry(pid).or_default().push(at),
        }
    }

    fn push(&mut self, node: ProcessNode) {
        let id = node.id;
        let pid = node.pid;
        self.nodes.push(node);
        self.by_pid.entry(pid).or_default().push(id);
    }

    /// Attach the earliest held exit that is not before this process's start.
    fn join_pending_exit(&mut self, pid: ProcessId) {
        let Some(id) = self.live_node_for(pid) else {
            return;
        };
        let started = self.nodes[id.index()].started;
        let Some(held) = self.pending_exits.get_mut(&pid) else {
            return;
        };
        let pick = held
            .iter()
            .enumerate()
            .filter(|(_, t)| started.is_none_or(|s| **t >= s))
            .min_by_key(|(_, t)| **t)
            .map(|(i, _)| i);
        if let Some(i) = pick {
            let at = held.remove(i);
            if held.is_empty() {
                self.pending_exits.remove(&pid);
            }
            self.nodes[id.index()].exited = Some(at);
        }
    }

    /// Resolve a parent for a node that already exists, refusing a candidate
    /// that would close a cycle.
    ///
    /// Only the snapshot-upgrade path can reach a candidate with a higher
    /// identifier than the node being given a parent, so only that path can
    /// build a cycle. An ordinary new node always resolves against nodes that
    /// already existed, whose identifiers are lower, so its chain terminates by
    /// construction.
    fn resolve_parent_for(
        &self,
        node: NodeId,
        parent_pid: ProcessId,
        at: Timestamp,
    ) -> Option<NodeId> {
        let candidate = self.resolve(parent_pid, at)?;
        if candidate == node {
            return None;
        }
        let mut walk = self.nodes[candidate.index()].parent;
        while let Some(p) = walk {
            if p == node {
                return None;
            }
            walk = self.nodes[p.index()].parent;
        }
        Some(candidate)
    }

    // -- reading ----------------------------------------------------------

    /// The node live at `at` for this operating system identifier.
    ///
    /// This is the lookup from the platform's vocabulary into the tree, and it
    /// is where identifier recycling is answered. Two nodes may share a
    /// [`ProcessId`], but not at the same instant, so at most one can contain
    /// `at`.
    ///
    /// A node whose start time is unknown orders before every observed event
    /// and is selected only when no node with a known start time contains `at`.
    /// That rule exists because such a node was found already running, so
    /// preferring it over a process whose creation was actually observed would
    /// answer with the weaker evidence.
    pub fn resolve(&self, pid: ProcessId, at: Timestamp) -> Option<NodeId> {
        let ids = self.by_pid.get(&pid)?;

        let known = ids
            .iter()
            .copied()
            .filter(|id| {
                let n = &self.nodes[id.index()];
                n.started.is_some() && n.contains(at)
            })
            .max_by_key(|id| self.nodes[id.index()].started);
        if known.is_some() {
            return known;
        }

        ids.iter().copied().find(|id| {
            let n = &self.nodes[id.index()];
            n.started.is_none() && n.contains(at)
        })
    }

    pub fn node(&self, id: NodeId) -> Option<&ProcessNode> {
        self.nodes.get(id.index())
    }

    pub fn nodes(&self) -> impl Iterator<Item = &ProcessNode> {
        self.nodes.iter()
    }

    /// The path from the root to this node, in creation order, ending with the
    /// node itself.
    ///
    /// Exited ancestors are included, which is the whole point: specification
    /// section 5.4's observed chains contain transient launchers that have
    /// already terminated by the time the client is worth asking about.
    pub fn ancestry(&self, id: NodeId) -> Vec<NodeId> {
        let mut path = Vec::new();
        let mut cursor = Some(id);
        while let Some(c) = cursor {
            path.push(c);
            cursor = self.nodes.get(c.index()).and_then(|n| n.parent);
        }
        path.reverse();
        path
    }

    /// Whether `id` has `ancestor` somewhere above it. Strict: a node does not
    /// descend from itself.
    ///
    /// This is the relation specification section 10.3's `descends_from`
    /// predicate is built on. The predicate itself resolves a stage name to a
    /// node first, and that half belongs to S12, which is what keeps the
    /// profile schema out of `fragcap-core`.
    pub fn descends_from(&self, id: NodeId, ancestor: NodeId) -> bool {
        let mut cursor = self.nodes.get(id.index()).and_then(|n| n.parent);
        while let Some(c) = cursor {
            if c == ancestor {
                return true;
            }
            cursor = self.nodes[c.index()].parent;
        }
        false
    }

    /// How many nodes are retained.
    ///
    /// Exposed because section 10.2 retains every node for the session and
    /// estimates the cost as "a few kilobytes". An operator should be able to
    /// see the real number during a long session rather than trust the
    /// estimate.
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Whether every event this tree was built from was observed.
    ///
    /// False once the platform has reported losing anything. A consumer holding
    /// only the tree can therefore tell an incomplete tree from a complete one
    /// without also holding the watcher's report.
    pub fn is_complete(&self) -> bool {
        self.events_lost == 0
    }

    pub fn events_lost(&self) -> u64 {
        self.events_lost
    }

    /// Exits that never found a start.
    ///
    /// Only meaningful at the end of a session. Before then an exit may simply
    /// be waiting for a start event delivered out of order.
    pub fn unmatched_exits(&self) -> u64 {
        self.pending_exits.values().map(|v| v.len() as u64).sum()
    }

    fn live_node_for(&self, pid: ProcessId) -> Option<NodeId> {
        self.by_pid
            .get(&pid)?
            .iter()
            .rev()
            .copied()
            .find(|id| self.nodes[id.index()].is_live())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(n: i64) -> Timestamp {
        Timestamp::from_nanos(n)
    }

    fn start(t: &mut ProcessTree, pid: u32, parent: u32, image: &str, when: i64) {
        t.apply(ProcessEvent::started(pid, parent, image, "cmd", at(when)));
    }

    fn exit(t: &mut ProcessTree, pid: u32, when: i64) {
        t.apply(ProcessEvent::Exited { pid, at: at(when) });
    }

    #[test]
    fn a_new_tree_is_empty_and_complete() {
        let t = ProcessTree::new();
        assert!(t.is_empty());
        assert!(t.is_complete());
        assert_eq!(t.unmatched_exits(), 0);
    }

    #[test]
    fn synthetic_identifiers_are_never_reused() {
        let mut t = ProcessTree::new();
        start(&mut t, 100, 1, "a.exe", 10);
        exit(&mut t, 100, 20);
        start(&mut t, 100, 1, "b.exe", 30);

        let ids: Vec<_> = t.nodes().map(|n| n.id()).collect();
        assert_eq!(ids.len(), 2);
        assert_ne!(ids[0], ids[1]);
    }

    #[test]
    fn the_image_name_is_derived_from_the_path() {
        let mut t = ProcessTree::new();
        start(&mut t, 1, 0, "C:\\Program Files\\Zenimax\\eso64.exe", 1);
        let n = t.node(NodeId(0)).unwrap();
        assert_eq!(n.image_name(), "eso64.exe");
        assert_eq!(n.image(), "C:\\Program Files\\Zenimax\\eso64.exe");
    }

    #[test]
    fn a_root_with_no_parent_in_the_tree_is_unresolved_and_keeps_its_pid() {
        let mut t = ProcessTree::new();
        start(&mut t, 100, 4, "explorer.exe", 10);
        let n = t.node(NodeId(0)).unwrap();
        assert_eq!(n.ancestry(), Ancestry::Unresolved);
        assert_eq!(n.parent(), None);
        // The observation survives even though nothing could be done with it.
        assert_eq!(n.parent_pid(), ProcessId(4));
    }

    #[test]
    fn a_chain_reports_its_ancestry_in_creation_order() {
        let mut t = ProcessTree::new();
        start(&mut t, 10, 1, "explorer.exe", 1);
        start(&mut t, 20, 10, "steam.exe", 2);
        start(&mut t, 30, 20, "launcher.exe", 3);

        let path = t.ancestry(NodeId(2));
        assert_eq!(path, vec![NodeId(0), NodeId(1), NodeId(2)]);
        assert!(t.descends_from(NodeId(2), NodeId(0)));
        assert!(!t.descends_from(NodeId(0), NodeId(2)));
        // Strict: a node does not descend from itself.
        assert!(!t.descends_from(NodeId(0), NodeId(0)));
    }

    #[test]
    fn ancestry_survives_the_whole_parent_chain_exiting() {
        let mut t = ProcessTree::new();
        start(&mut t, 10, 1, "explorer.exe", 1);
        start(&mut t, 20, 10, "steam.exe", 2);
        start(&mut t, 30, 20, "client.exe", 3);
        exit(&mut t, 20, 4);
        exit(&mut t, 10, 5);

        assert_eq!(t.ancestry(NodeId(2)).len(), 3);
        assert_eq!(t.len(), 3, "exited nodes are retained for the session");
        assert!(!t.node(NodeId(1)).unwrap().is_live());
    }

    #[test]
    fn a_recycled_identifier_produces_two_nodes_and_no_inherited_children() {
        let mut t = ProcessTree::new();
        start(&mut t, 100, 1, "first.exe", 10);
        start(&mut t, 200, 100, "child-of-first.exe", 11);
        exit(&mut t, 100, 20);
        // The operating system hands 100 out again to something unrelated.
        start(&mut t, 100, 1, "second.exe", 30);
        start(&mut t, 300, 100, "child-of-second.exe", 31);

        let first = NodeId(0);
        let second = NodeId(2);
        assert_ne!(first, second);
        assert_eq!(t.node(NodeId(1)).unwrap().parent(), Some(first));
        assert_eq!(t.node(NodeId(3)).unwrap().parent(), Some(second));
        assert!(!t.descends_from(NodeId(3), first));
        assert!(!t.descends_from(NodeId(1), second));
    }

    #[test]
    fn resolution_selects_the_node_live_at_the_instant_asked_about() {
        let mut t = ProcessTree::new();
        start(&mut t, 100, 1, "first.exe", 10);
        exit(&mut t, 100, 20);
        start(&mut t, 100, 1, "second.exe", 30);

        assert_eq!(t.resolve(ProcessId(100), at(15)), Some(NodeId(0)));
        assert_eq!(t.resolve(ProcessId(100), at(35)), Some(NodeId(1)));
        // Between the two lifetimes nothing held the identifier.
        assert_eq!(t.resolve(ProcessId(100), at(25)), None);
        assert_eq!(t.resolve(ProcessId(999), at(1)), None);
    }

    #[test]
    fn an_unknown_start_time_never_wins_against_a_known_one() {
        let mut t = ProcessTree::new();
        // Found already running, start time not reported by the platform.
        t.apply_snapshot(&[ProcessRecord::new(100, 4, "old.exe")]);
        // The identifier is later reused by a process whose creation is seen.
        exit(&mut t, 100, 20);
        start(&mut t, 100, 1, "new.exe", 30);

        assert_eq!(t.len(), 2);
        assert_eq!(t.resolve(ProcessId(100), at(35)), Some(NodeId(1)));
        assert_eq!(t.resolve(ProcessId(100), at(5)), Some(NodeId(0)));
    }

    #[test]
    fn an_exit_arriving_before_its_start_is_held_and_then_joined() {
        let mut t = ProcessTree::new();
        exit(&mut t, 100, 20);
        assert_eq!(t.unmatched_exits(), 1);
        assert_eq!(t.len(), 0, "no node is fabricated for an unmatched exit");

        start(&mut t, 100, 1, "a.exe", 10);
        assert_eq!(t.unmatched_exits(), 0);
        assert_eq!(t.node(NodeId(0)).unwrap().exited(), Some(at(20)));
    }

    #[test]
    fn an_exit_that_never_finds_a_start_stays_unmatched() {
        let mut t = ProcessTree::new();
        exit(&mut t, 4242, 20);
        assert_eq!(t.unmatched_exits(), 1);
        assert_eq!(t.len(), 0);
    }

    #[test]
    fn a_held_exit_before_the_start_it_might_join_is_not_taken() {
        let mut t = ProcessTree::new();
        // An exit for a process whose lifetime ended before this start.
        exit(&mut t, 100, 5);
        start(&mut t, 100, 1, "a.exe", 10);
        assert!(t.node(NodeId(0)).unwrap().is_live());
        assert_eq!(t.unmatched_exits(), 1);
    }

    #[test]
    fn the_snapshot_does_not_duplicate_a_process_the_events_already_reported() {
        let mut t = ProcessTree::new();
        start(&mut t, 10, 4, "parent.exe", 5);
        start(&mut t, 100, 10, "a.exe", 10);
        t.apply_snapshot(&[
            ProcessRecord::new(10, 4, "parent.exe"),
            ProcessRecord::new(100, 10, "a.exe"),
        ]);

        assert_eq!(t.len(), 2, "the snapshot adds nothing already observed");
        // Creation-time ancestry survives, rather than being overwritten by the
        // weaker kind the snapshot carries.
        assert_eq!(t.node(NodeId(1)).unwrap().ancestry(), Ancestry::Observed);
    }

    #[test]
    fn the_events_do_not_duplicate_a_process_the_snapshot_already_reported() {
        let mut t = ProcessTree::new();
        t.apply_snapshot(&[ProcessRecord::new(100, 10, "a.exe")]);
        assert_eq!(t.node(NodeId(0)).unwrap().ancestry(), Ancestry::Unresolved);

        start(&mut t, 100, 10, "C:\\a.exe", 10);

        assert_eq!(t.len(), 1, "one process, one node, in either order");
        let n = t.node(NodeId(0)).unwrap();
        assert_eq!(n.started(), Some(at(10)));
        assert_eq!(n.image(), "C:\\a.exe");
        assert!(n.command_line().is_available());
    }

    #[test]
    fn snapshot_ancestry_is_distinguishable_from_observed_ancestry() {
        let mut t = ProcessTree::new();
        t.apply_snapshot(&[
            ProcessRecord::new(10, 4, "explorer.exe"),
            ProcessRecord::new(20, 10, "steam.exe"),
        ]);
        start(&mut t, 30, 20, "client.exe", 100);

        assert_eq!(t.node(NodeId(0)).unwrap().ancestry(), Ancestry::Unresolved);
        assert_eq!(t.node(NodeId(1)).unwrap().ancestry(), Ancestry::Snapshot);
        assert_eq!(t.node(NodeId(2)).unwrap().ancestry(), Ancestry::Observed);
        // And the relation still works across the two kinds.
        assert!(t.descends_from(NodeId(2), NodeId(0)));
    }

    #[test]
    fn a_snapshot_process_has_no_command_line_and_the_tree_says_so() {
        let mut t = ProcessTree::new();
        t.apply_snapshot(&[ProcessRecord::new(10, 4, "explorer.exe")]);
        assert_eq!(
            t.node(NodeId(0)).unwrap().command_line(),
            &CommandLine::Unavailable
        );
    }

    #[test]
    fn a_command_line_reaches_the_tree_byte_for_byte() {
        // Non-ASCII through a localized user directory, and a value longer than
        // any buffer an implementation would plausibly have picked.
        let odd = "app.exe --path \"C:\\Users\\Ünïcødé Ω\\Games\" --tag=日本語";
        let long: String = format!("app.exe {}", "-x ".repeat(20_000));

        let mut t = ProcessTree::new();
        t.apply(ProcessEvent::started(1, 0, "C:\\app.exe", odd, at(1)));
        t.apply(ProcessEvent::started(2, 0, "C:\\app.exe", &long, at(2)));

        assert_eq!(
            t.node(NodeId(0))
                .unwrap()
                .command_line()
                .as_str()
                .unwrap()
                .as_bytes(),
            odd.as_bytes()
        );
        assert_eq!(
            t.node(NodeId(1))
                .unwrap()
                .command_line()
                .as_str()
                .unwrap()
                .as_bytes(),
            long.as_bytes()
        );
    }

    #[test]
    fn a_lost_event_makes_the_tree_permanently_incomplete() {
        let mut t = ProcessTree::new();
        assert!(t.is_complete());
        t.note_lost(3);
        assert!(!t.is_complete());
        assert_eq!(t.events_lost(), 3);

        start(&mut t, 1, 0, "a.exe", 1);
        assert!(!t.is_complete(), "observing more does not repair a hole");
    }

    #[test]
    fn noting_zero_losses_leaves_the_tree_complete() {
        let mut t = ProcessTree::new();
        t.note_lost(0);
        assert!(t.is_complete());
    }

    #[test]
    fn the_stage_is_reserved_and_empty_until_s12() {
        let mut t = ProcessTree::new();
        start(&mut t, 1, 0, "a.exe", 1);
        assert!(t.node(NodeId(0)).unwrap().stage().is_none());
    }

    #[test]
    fn nothing_is_discarded_at_session_scale() {
        // Ten times the larger reconnaissance session, which scanned 2,526
        // command lines over roughly twenty minutes.
        const N: u32 = 25_000;
        let mut t = ProcessTree::new();
        for i in 0..N {
            // Every process is a child of the previous one, and the identifier
            // space is deliberately small so that recycling happens constantly.
            let pid = 1000 + (i % 64);
            let parent = 1000 + ((i + 63) % 64);
            start(&mut t, pid, parent, "a.exe", i as i64 * 2);
            exit(&mut t, pid, i as i64 * 2 + 1);
        }
        assert_eq!(t.len(), N as usize, "every observed process is retained");
        assert_eq!(t.unmatched_exits(), 0);
        assert!(t.is_complete());
    }

    #[test]
    fn the_snapshot_upgrade_path_cannot_build_a_cycle() {
        let mut t = ProcessTree::new();
        // A node from the snapshot, and a later node claiming it as parent.
        t.apply_snapshot(&[ProcessRecord::new(100, 4, "a.exe")]);
        start(&mut t, 200, 100, "b.exe", 10);
        // Now the snapshot node's own creation event arrives, naming its own
        // descendant as its parent. Absurd, and it must not hang.
        start(&mut t, 100, 200, "a.exe", 11);

        assert_eq!(t.node(NodeId(0)).unwrap().parent(), None);
        assert_eq!(t.node(NodeId(0)).unwrap().ancestry(), Ancestry::Unresolved);
        assert_eq!(t.ancestry(NodeId(1)).len(), 2);
    }
}
