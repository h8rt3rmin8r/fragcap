// SPDX-License-Identifier: Apache-2.0

//! Stage matching, specification section 10.3.
//!
//! A profile declares stages; a process tree records processes. This module is
//! the decision that joins them: for each process, the first stage all of whose
//! predicates hold binds that process and gives it the stage's role.
//!
//! It lives here rather than in `fragcap-core` because it reads the profile
//! schema, and rather than in `fragcap-attr` because that would put a capture
//! sibling in the profile crate's dependencies. `fragcap-profile` already
//! depends on `fragcap-core`, so it is the one crate that can read both the
//! stages and the tree without a new edge. The mutation it performs, binding a
//! node to a stage, is `fragcap-core`'s [`ProcessTree::bind_stage`]; the decision
//! is here and the recording is there.
//!
//! Everything here is a decision over values. It opens nothing and touches no
//! platform interface, so the whole of section 10.3 is tested against a tree
//! built from a scripted event stream, with no capture driver, no elevation, and
//! no game.
//!
//! # `descends_from` and constitution P-9
//!
//! `cmdline_contains` never matches a command line that was not observed. A
//! [`CommandLine::Unavailable`](fragcap_core::process::CommandLine::Unavailable)
//! is not an empty string, and treating it as one would be the substitution P-9
//! forbids: it would report a match against a value fragcap never saw.

use fragcap_core::attribution::StageId;
use fragcap_core::process::tree::NodeId;
use fragcap_core::process::{ProcessNode, ProcessTree};

use crate::schema::{MatchPredicates, Profile, Stage};

/// The first stage, in declaration order, all of whose specified predicates hold
/// for the node, or `None` when no stage matches.
///
/// Pure: it reads the tree, including the bindings already applied to ancestors,
/// which is what `descends_from` resolves against. It mutates nothing. The
/// returned reference borrows the profile, not the tree, so a caller may bind
/// through `&mut ProcessTree` immediately after.
///
/// When more than one stage matches, the first in declaration order wins
/// (decision D-3). Section 15.4 validation has already made an ambiguous image
/// match within a chain an error, so this only makes the residual case total
/// rather than dependent on iteration order.
pub fn stage_for<'p>(profile: &'p Profile, tree: &ProcessTree, node: NodeId) -> Option<&'p Stage> {
    let n = tree.node(node)?;
    profile
        .stages()
        .iter()
        .find(|stage| predicates_hold(stage.predicates(), tree, node, n))
}

/// Bind every node to its matching stage, in creation order.
///
/// `NodeId` is assigned as events are folded, so iterating in identifier order
/// replays the causal creation order S11 guarantees. A stage that matches an
/// ancestor binds on the ancestor's event, so by the time a descendant is
/// evaluated its ancestor is already bound and `descends_from` resolves
/// (decision D-2). A node already bound is left alone.
pub fn bind_stages(profile: &Profile, tree: &mut ProcessTree) {
    let mut ids: Vec<NodeId> = tree.nodes().map(|n| n.id()).collect();
    ids.sort_by_key(|id| id.get());

    for id in ids {
        if tree.node(id).and_then(|n| n.stage()).is_some() {
            continue;
        }
        // The decision borrows the profile only; the returned owned StageId ends
        // every borrow of the tree before the mutable bind below.
        let decision = stage_for(profile, tree, id).map(|s| StageId::new(s.role()));
        if let Some(stage) = decision {
            tree.bind_stage(id, stage);
        }
    }
}

/// Whether every specified predicate holds for one node.
///
/// All specified predicates must hold (section 10.3). An empty predicate set
/// cannot occur: [`MatchPredicates::is_empty`] is a validation error, so a stage
/// on a validated profile always constrains at least one field.
fn predicates_hold(
    pred: &MatchPredicates,
    tree: &ProcessTree,
    id: NodeId,
    node: &ProcessNode,
) -> bool {
    if let Some(exe) = pred.exe() {
        if !exe.matches(node.image_name()) {
            return false;
        }
    }
    if let Some(sub) = pred.path_contains() {
        if !contains_ignore_case(node.image(), sub) {
            return false;
        }
    }
    if let Some(re) = pred.path_regex() {
        if !re.regex().is_match(node.image()) {
            return false;
        }
    }
    if let Some(sub) = pred.cmdline_contains() {
        match node.command_line().as_str() {
            Some(cl) if cl.contains(sub) => {}
            // Unavailable, or observed but not containing the substring. An
            // unavailable command line never matches (P-9): it was not observed,
            // so it cannot be reported as containing anything.
            _ => return false,
        }
    }
    if let Some(role) = pred.descends_from() {
        if !ancestor_bound_to(tree, id, role) {
            return false;
        }
    }
    true
}

/// Whether some strict ancestor of `id` is bound to `role`.
///
/// `descends_from` resolves over the synthetic tree, not the operating system
/// parent chain (section 10.3). [`ProcessTree::ancestry`] returns the path from
/// the root to the node inclusive; dropping the last element leaves the strict
/// ancestors, so a node never satisfies `descends_from` on itself.
fn ancestor_bound_to(tree: &ProcessTree, id: NodeId, role: &str) -> bool {
    let path = tree.ancestry(id);
    let strict = &path[..path.len().saturating_sub(1)];
    strict.iter().any(|anc| {
        tree.node(*anc)
            .and_then(|n| n.stage())
            .is_some_and(|s| s.as_str() == role)
    })
}

/// Case-insensitive substring, per section 10.3.
///
/// Compared on lowercased copies, so the stored image path is untouched (P-9);
/// only the comparison folds case. An image path is one short string and this is
/// evaluated per process start rather than per packet, so the allocation is not
/// on any hot path.
fn contains_ignore_case(haystack: &str, needle: &str) -> bool {
    haystack.to_lowercase().contains(&needle.to_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;
    use fragcap_core::packet::Timestamp;
    use fragcap_core::process::{CommandLine, ProcessEvent, ProcessTree};

    use crate::schema::Profile;

    fn at(n: i64) -> Timestamp {
        Timestamp::from_nanos(n)
    }

    /// Build a test profile from a JSON stage-array body (the objects inside
    /// `"stage": [ ... ]`).
    fn profile(stages: &str) -> Profile {
        let text = format!(
            r#"{{"schema":1,"kind":"profile","fidelity":"verified","game":{{"id":"t","name":"T"}},"stage":[{stages}]}}"#
        );
        Profile::parse(&text).unwrap_or_else(|d| {
            panic!(
                "test profile did not validate: {:?}",
                d.iter().map(|x| x.message.clone()).collect::<Vec<_>>()
            )
        })
    }

    /// Bind against a tree and read the role a node ended up with.
    fn role_of(tree: &ProcessTree, node: NodeId) -> Option<String> {
        tree.node(node)
            .and_then(|n| n.stage())
            .map(|s| s.as_str().to_string())
    }

    /// Node identifiers in creation order. `NodeId` has no public constructor,
    /// so a consumer obtains one only from the tree; this is that path.
    fn ids(tree: &ProcessTree) -> Vec<NodeId> {
        let mut v: Vec<NodeId> = tree.nodes().map(|n| n.id()).collect();
        v.sort_by_key(|id| id.get());
        v
    }

    #[test]
    fn exe_matches_the_file_name_case_insensitively() {
        let p = profile(r#"{"role":"client","lifecycle":"session","match":{"exe":"eso64.exe"}}"#);
        let mut t = ProcessTree::new();
        t.apply(ProcessEvent::started(
            1,
            0,
            "C:\\Games\\ESO\\ESO64.EXE",
            "ESO64.EXE",
            at(1),
        ));
        bind_stages(&p, &mut t);
        assert_eq!(role_of(&t, ids(&t)[0]).as_deref(), Some("client"));
    }

    #[test]
    fn path_contains_is_a_case_insensitive_substring_of_the_full_path() {
        let p = profile(
            r#"{"role":"client","lifecycle":"session","match":{"path_contains":"zenimax"}}"#,
        );
        let mut t = ProcessTree::new();
        t.apply(ProcessEvent::started(
            1,
            0,
            "C:\\Program Files\\ZeniMax\\eso64.exe",
            "eso64.exe",
            at(1),
        ));
        bind_stages(&p, &mut t);
        assert_eq!(role_of(&t, ids(&t)[0]).as_deref(), Some("client"));
    }

    #[test]
    fn path_regex_matches_the_full_path() {
        let p = profile(
            r#"{"role":"client","lifecycle":"session","match":{"path_regex":"(?i)eso\\d+\\.exe$"}}"#,
        );
        let mut t = ProcessTree::new();
        t.apply(ProcessEvent::started(
            1,
            0,
            "C:\\Games\\eso64.exe",
            "eso64.exe",
            at(1),
        ));
        bind_stages(&p, &mut t);
        assert_eq!(role_of(&t, ids(&t)[0]).as_deref(), Some("client"));
    }

    #[test]
    fn cmdline_contains_matches_an_observed_command_line() {
        let p = profile(
            r#"{"role":"client","lifecycle":"session","match":{"cmdline_contains":"-sessionid"}}"#,
        );
        let mut t = ProcessTree::new();
        t.apply(ProcessEvent::started(
            1,
            0,
            "C:\\Games\\eso64.exe",
            "eso64.exe -sessionid 7",
            at(1),
        ));
        bind_stages(&p, &mut t);
        assert_eq!(role_of(&t, ids(&t)[0]).as_deref(), Some("client"));
    }

    #[test]
    fn cmdline_contains_never_matches_an_unavailable_command_line() {
        // P-9: an unavailable command line was not observed, so it cannot be
        // reported as containing anything. A snapshot process has no command
        // line, and the stage must not bind it.
        let p = profile(
            r#"{"role":"client","lifecycle":"session","match":{"cmdline_contains":"-sessionid"}}"#,
        );
        let mut t = ProcessTree::new();
        t.apply(ProcessEvent::Started {
            pid: 1,
            parent: 0,
            image: "C:\\Games\\eso64.exe".into(),
            command_line: CommandLine::Unavailable,
            at: at(1),
        });
        bind_stages(&p, &mut t);
        assert_eq!(role_of(&t, ids(&t)[0]), None);
    }

    #[test]
    fn all_specified_predicates_must_hold() {
        // exe matches but the command line does not, so the stage does not bind.
        let p = profile(
            r#"{"role":"client","lifecycle":"session","match":{"exe":"eso64.exe","cmdline_contains":"-sessionid"}}"#,
        );
        let mut t = ProcessTree::new();
        t.apply(ProcessEvent::started(
            1,
            0,
            "C:\\Games\\eso64.exe",
            "eso64.exe --windowed",
            at(1),
        ));
        bind_stages(&p, &mut t);
        assert_eq!(
            role_of(&t, ids(&t)[0]),
            None,
            "one failing predicate is enough"
        );
    }

    #[test]
    fn descends_from_binds_the_descendant_where_exe_alone_would_bind_the_shim() {
        // The section 5.4 case: several processes share one image name and only
        // the descendant of the launcher is the client. Ordered so the first
        // process carrying the shared name is NOT the descendant, so a match on
        // exe alone would bind the wrong one and descends_from is what fixes it.
        let p = profile(
            r#"{"role":"launcher","lifecycle":"transient","match":{"exe":"launcher.exe"}},{"role":"client","lifecycle":"session","terminal":true,"match":{"exe":"game.exe","descends_from":"launcher"}}"#,
        );
        let mut t = ProcessTree::new();
        // ids(&t)[0]: the launcher.
        t.apply(ProcessEvent::started(
            100,
            0,
            "C:\\L\\launcher.exe",
            "l",
            at(1),
        ));
        // ids(&t)[1]: a game.exe NOT under the launcher, created first.
        t.apply(ProcessEvent::started(150, 9, "C:\\G\\game.exe", "g", at(2)));
        // ids(&t)[2]: a game.exe under the launcher, created second.
        t.apply(ProcessEvent::started(
            200,
            100,
            "C:\\G\\game.exe",
            "g",
            at(3),
        ));

        bind_stages(&p, &mut t);

        assert_eq!(role_of(&t, ids(&t)[0]).as_deref(), Some("launcher"));
        assert_eq!(
            role_of(&t, ids(&t)[1]),
            None,
            "the game.exe not descended from the launcher does not bind the client"
        );
        assert_eq!(
            role_of(&t, ids(&t)[2]).as_deref(),
            Some("client"),
            "descends_from binds the descendant of the launcher, not the first name match"
        );
    }

    #[test]
    fn descends_from_does_not_match_when_no_ancestor_is_bound() {
        let p = profile(
            r#"{"role":"launcher","lifecycle":"transient","match":{"exe":"launcher.exe"}},{"role":"client","lifecycle":"session","terminal":true,"match":{"exe":"game.exe","descends_from":"launcher"}}"#,
        );
        let mut t = ProcessTree::new();
        // A game.exe with no launcher ancestor at all.
        t.apply(ProcessEvent::started(200, 9, "C:\\G\\game.exe", "g", at(1)));
        bind_stages(&p, &mut t);
        assert_eq!(role_of(&t, ids(&t)[0]), None);
    }

    #[test]
    fn a_process_matching_two_stages_binds_the_first_declared() {
        // Two stages both match a single process without an exe-glob ambiguity:
        // one keys on exe, the other on the command line. Declaration order
        // decides (D-3).
        let p = profile(
            r#"{"role":"first","lifecycle":"session","match":{"exe":"dup.exe"}},{"role":"second","lifecycle":"transient","match":{"cmdline_contains":"dup"}}"#,
        );
        let mut t = ProcessTree::new();
        t.apply(ProcessEvent::started(
            1,
            0,
            "C:\\A\\dup.exe",
            "dup --run",
            at(1),
        ));
        bind_stages(&p, &mut t);
        assert_eq!(
            role_of(&t, ids(&t)[0]).as_deref(),
            Some("first"),
            "the first stage in declaration order binds"
        );
    }
}
