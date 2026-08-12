// SPDX-License-Identifier: Apache-2.0

//! The semantic checks of specification section 15.4.
//!
//! Structural faults (types, required keys, enum ranges, unknown keys, an empty
//! `match`) are the schema's job and are found by the structural layer in
//! [`crate::parse`]. These are the faults that need the whole profile in view.
//! Every check runs, and every check that cannot run for want of an input is
//! skipped rather than aborting, so a profile with several problems reports all
//! of them.
//!
//! Locations are JSON pointers, matching the structural layer; there is no byte
//! position, because serde_json exposes no per-value span.
//!
//! # Three checks beyond the section 15.4 list
//!
//! Section 15.4's list is a floor. Three checks are added here, and each is in
//! the same failure class as the two unusual checks section 15.4 already names,
//! which is a run that succeeds and captures nothing:
//!
//! - A `capture.roles` entry naming an undeclared role captures nothing under
//!   that role.
//! - A `terminal` stage whose lifecycle is not `session` ends the capture when a
//!   process that was expected to exit exits. For a `transient` launcher that is
//!   immediately, producing a short well-formed file.
//! - A `descends_from` cycle is unsatisfiable, so every stage in it binds
//!   nothing.

use std::collections::{BTreeMap, BTreeSet};

use crate::diagnostic::{Diagnostic, DiagnosticCode, Diagnostics};
use crate::parse::{Draft, DraftStage};
use crate::schema::Lifecycle;

/// Run every semantic check over the draft.
pub(crate) fn check(draft: &Draft, d: &mut Diagnostics) {
    let declared = declared_roles(&draft.stages);

    unique_roles(&draft.stages, d);
    terminal_stage(&draft.stages, d);
    at_least_one_non_service(draft, d);
    descends_from_resolves(&draft.stages, &declared, d);
    descends_from_is_acyclic(&draft.stages, &declared, d);
    capture_roles_are_declared(draft, &declared, d);
    ambiguous_image_match(&draft.stages, d);
}

/// Every role the profile declares, mapped to the first stage that declared it.
fn declared_roles(stages: &[DraftStage]) -> BTreeMap<&str, usize> {
    let mut out = BTreeMap::new();
    for s in stages {
        if let Some(role) = s.role.as_deref() {
            out.entry(role).or_insert(s.index);
        }
    }
    out
}

/// Role names are unique within a profile, and a collision names both.
fn unique_roles(stages: &[DraftStage], d: &mut Diagnostics) {
    let mut first: BTreeMap<&str, usize> = BTreeMap::new();
    for s in stages {
        let Some(role) = s.role.as_deref() else {
            continue;
        };
        match first.get(role) {
            None => {
                first.insert(role, s.index);
            }
            Some(earlier) => d.push(Diagnostic::located(
                DiagnosticCode::DuplicateRole,
                s.loc("role"),
                format!(
                    "role `{role}` is already declared by stage[{earlier}]; \
                     role names are unique within a profile"
                ),
            )),
        }
    }
}

/// At most one terminal stage, and its lifecycle is `session`.
fn terminal_stage(stages: &[DraftStage], d: &mut Diagnostics) {
    let terminal: Vec<&DraftStage> = stages.iter().filter(|s| s.terminal).collect();

    if let Some(first) = terminal.first() {
        for extra in terminal.iter().skip(1) {
            d.push(Diagnostic::located(
                DiagnosticCode::MultipleTerminal,
                extra.loc("terminal"),
                format!(
                    "stage[{}] is already terminal; at most one stage per profile \
                     may be",
                    first.index
                ),
            ));
        }
    }

    for s in &terminal {
        match s.lifecycle {
            None => {}
            Some(Lifecycle::Session) => {}
            Some(other) => {
                let name = match other {
                    Lifecycle::Transient => "transient",
                    Lifecycle::Service => "service",
                    Lifecycle::Session => unreachable!("handled above"),
                };
                d.push(Diagnostic::located(
                    DiagnosticCode::TerminalLifecycle,
                    s.loc("terminal"),
                    format!(
                        "a terminal stage must have lifecycle `session`, not `{name}`; \
                         section 10.4 defines a {name} exit as expected, so this would \
                         end the capture as soon as that process exits"
                    ),
                ));
            }
        }
    }
}

/// At least one stage is not a service.
///
/// Section 10.4: a service is never awaited during acquisition, because waiting
/// for something already running deadlocks. A profile consisting entirely of
/// services can therefore never trigger acquisition.
fn at_least_one_non_service(draft: &Draft, d: &mut Diagnostics) {
    let stages = &draft.stages;
    if stages.is_empty() {
        // Already reported structurally as a missing or empty stage array.
        return;
    }
    // Skip if any lifecycle failed to parse: the profile is refused anyway, and
    // reporting this too would be reporting a consequence of the other fault.
    if stages.iter().any(|s| s.lifecycle.is_none()) {
        return;
    }
    if stages
        .iter()
        .all(|s| s.lifecycle == Some(Lifecycle::Service))
    {
        d.push(Diagnostic::located(
            DiagnosticCode::AllServices,
            "/stage",
            "every stage is a service, so nothing can trigger acquisition; \
             at least one stage must be transient or session",
        ));
    }
}

/// Every `descends_from` names a role declared in the same profile.
fn descends_from_resolves(
    stages: &[DraftStage],
    declared: &BTreeMap<&str, usize>,
    d: &mut Diagnostics,
) {
    for s in stages {
        let Some(target) = s.predicates.descends_from() else {
            continue;
        };
        if !declared.contains_key(target) {
            let known: Vec<&str> = declared.keys().copied().collect();
            d.push(Diagnostic::located(
                DiagnosticCode::UnknownDescendsFrom,
                s.loc("match.descends_from"),
                format!(
                    "`descends_from` names role `{target}`, which no stage declares; \
                     declared roles: {}",
                    if known.is_empty() {
                        "none".to_string()
                    } else {
                        known.join(", ")
                    }
                ),
            ));
        }
    }
}

/// The `descends_from` relation is acyclic, and a cycle names every role in it.
///
/// No process assignment can satisfy a cycle, so every stage in one binds
/// nothing. Includes the self-reference case, which is a cycle of length one.
fn descends_from_is_acyclic(
    stages: &[DraftStage],
    declared: &BTreeMap<&str, usize>,
    d: &mut Diagnostics,
) {
    // Edges only between declared roles. An edge to an undeclared role is
    // already reported and cannot be part of a cycle.
    let mut edge: BTreeMap<&str, &str> = BTreeMap::new();
    let mut at: BTreeMap<&str, usize> = BTreeMap::new();
    for s in stages {
        let Some(role) = s.role.as_deref() else {
            continue;
        };
        let Some(target) = s.predicates.descends_from() else {
            continue;
        };
        if declared.contains_key(target) {
            edge.insert(role, target);
            at.insert(role, s.index);
        }
    }

    let mut reported: BTreeSet<&str> = BTreeSet::new();
    for start in edge.keys().copied() {
        if reported.contains(start) {
            continue;
        }
        // Walk forward. The relation is a function, so a cycle is found by
        // revisiting a role within this walk.
        let mut path: Vec<&str> = Vec::new();
        let mut seen: BTreeSet<&str> = BTreeSet::new();
        let mut cur = start;
        loop {
            if seen.contains(cur) {
                let from = path.iter().position(|r| *r == cur).unwrap_or(0);
                let cycle: Vec<&str> = path[from..].to_vec();
                for role in &cycle {
                    reported.insert(role);
                }
                let index = at.get(cur).copied().unwrap_or(0);
                let mut named = cycle.clone();
                named.push(cur);
                d.push(Diagnostic::located(
                    DiagnosticCode::DescendsFromCycle,
                    format!("/stage/{index}/match/descends_from"),
                    format!(
                        "`descends_from` forms a cycle: {}. No process can satisfy it, \
                         so every stage in the cycle would bind nothing",
                        named.join(" -> ")
                    ),
                ));
                break;
            }
            seen.insert(cur);
            path.push(cur);
            match edge.get(cur) {
                Some(next) => cur = next,
                None => break,
            }
        }
    }
}

/// `capture.roles` is non-empty when present and every entry is declared.
fn capture_roles_are_declared(
    draft: &Draft,
    declared: &BTreeMap<&str, usize>,
    d: &mut Diagnostics,
) {
    let Some(roles) = draft.capture.roles() else {
        return;
    };

    // Emptiness is judged on what the author declared, not on what survived
    // parsing. A list of `["ghost", 1]` has one bad element and is not empty.
    if draft.roles_declared == Some(0) {
        d.push(Diagnostic::located(
            DiagnosticCode::EmptyRoles,
            "/capture/roles",
            "`capture.roles` is empty, so it names nothing to capture; omit the key \
             to capture every declared role",
        ));
        return;
    }

    for role in roles {
        if !declared.contains_key(role.as_str()) {
            let known: Vec<&str> = declared.keys().copied().collect();
            d.push(Diagnostic::located(
                DiagnosticCode::UndeclaredCaptureRole,
                "/capture/roles",
                format!(
                    "`capture.roles` names role `{role}`, which no stage declares, so \
                     nothing would be captured under it; declared roles: {}",
                    if known.is_empty() {
                        "none".to_string()
                    } else {
                        known.join(", ")
                    }
                ),
            ));
        }
    }
}

/// The ambiguous image match check of section 15.4.
///
/// For every unordered pair of stages whose `exe` patterns can match a common
/// image name, the pair is refused unless both stages carry at least one
/// predicate other than `exe`. Two stages that are both pinned are permitted to
/// share an image name, which is exactly the section 15.2 profile for the second
/// focal title.
fn ambiguous_image_match(stages: &[DraftStage], d: &mut Diagnostics) {
    for (i, a) in stages.iter().enumerate() {
        let Some(pa) = a.predicates.exe() else {
            continue;
        };
        for b in stages.iter().skip(i + 1) {
            let Some(pb) = b.predicates.exe() else {
                continue;
            };
            if !pa.intersects(pb) {
                continue;
            }
            if a.predicates.is_pinned() && b.predicates.is_pinned() {
                continue;
            }
            let unpinned = if a.predicates.is_pinned() { b } else { a };
            d.push(Diagnostic::located(
                DiagnosticCode::AmbiguousImageMatch,
                unpinned.loc("match.exe"),
                format!(
                    "stage[{}] (`{}`) and stage[{}] (`{}`) can match one image name, and \
                     stage[{}] has no other predicate to distinguish it. A stage bound to \
                     the wrong process produces a complete, well formed, empty capture; \
                     add `descends_from`, `path_contains`, `path_regex`, or \
                     `cmdline_contains`",
                    a.index,
                    pa.as_str(),
                    b.index,
                    pb.as_str(),
                    unpinned.index,
                ),
            ));
        }
    }
}
