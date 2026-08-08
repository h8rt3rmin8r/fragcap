// SPDX-License-Identifier: Apache-2.0

//! Registry publication in dependency order.
//!
//! crates.io rejects a crate whose dependencies are not already in the
//! registry, so the eight crates go up in a fixed order. That order is
//! written here and asserted against the dependency graph in `deps`, so it
//! cannot drift away from the architecture it is derived from. A reordering
//! that breaks publication fails a unit test rather than a release.
//!
//! Publication is never the default. Without `--execute` this prints the plan
//! and changes nothing. Publishing is irreversible: a version can be yanked
//! but never replaced, so it is not something a caller should be able to do
//! by accident.

use std::path::Path;

use crate::deps::EXPECTED;

/// Publication order. Every crate appears after everything it depends on,
/// which `order_matches_the_graph` asserts against `deps::EXPECTED`.
pub const ORDER: &[&str] = &[
    "fragcap-core",
    "fragcap-profile",
    "fragcap-capture",
    "fragcap-attr",
    "fragcap-sink",
    "fragcap-steam",
    "fragcap",
    "fragcap-cli",
];

/// Every edge is `(dependent, dependency)`, and a dependency must be
/// published first. Returns one message per ordering that would fail.
///
/// Pure, so the rule is tested against a deliberately broken order rather
/// than only against the one that happens to be correct.
pub fn order_violations(order: &[&str], edges: &[(&str, &str)]) -> Vec<String> {
    let position = |name: &str| order.iter().position(|c| *c == name);
    let mut out = Vec::new();

    for (dependent, dependency) in edges {
        match (position(dependent), position(dependency)) {
            (Some(later), Some(earlier)) if earlier < later => {}
            (Some(_), Some(_)) => {
                out.push(format!("{dependency} must be published before {dependent}"));
            }
            _ => {
                out.push(format!(
                    "{dependent} or {dependency} is absent from the publication order"
                ));
            }
        }
    }

    out.sort();
    out.dedup();
    out
}

pub fn run(_root: &Path, execute: bool) -> usize {
    let problems = order_violations(ORDER, EXPECTED);
    if !problems.is_empty() {
        for p in &problems {
            eprintln!("publish: {p}");
        }
        return problems.len();
    }

    if !execute {
        println!("publish: order is consistent with the dependency graph");
        for (i, name) in ORDER.iter().enumerate() {
            println!("publish:   {}. {name}", i + 1);
        }
        println!("publish: nothing was published. Pass --execute to publish.");
        return 0;
    }

    for name in ORDER {
        println!("publish: publishing {name}");
        if !crate::cargo(&["publish", "-p", name]) {
            // Stop rather than continue. Every later crate depends on
            // something in the part that did not go up, so continuing would
            // turn one failure into a run of confusing ones.
            eprintln!("publish: {name} failed. Later crates were not attempted.");
            return 1;
        }
    }

    println!("publish: all {} crates published", ORDER.len());
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn order_matches_the_graph() {
        assert_eq!(order_violations(ORDER, EXPECTED), Vec::<String>::new());
    }

    #[test]
    fn every_workspace_crate_is_in_the_order() {
        for (dependent, dependency) in EXPECTED {
            assert!(ORDER.contains(dependent), "{dependent} missing from ORDER");
            assert!(
                ORDER.contains(dependency),
                "{dependency} missing from ORDER"
            );
        }
    }

    #[test]
    fn a_reversed_order_is_rejected() {
        let reversed: Vec<&str> = ORDER.iter().rev().copied().collect();
        let problems = order_violations(&reversed, EXPECTED);
        assert!(!problems.is_empty());
        assert!(problems
            .iter()
            .any(|p| p.contains("must be published before")));
    }

    #[test]
    fn a_crate_absent_from_the_order_is_rejected() {
        let short = ["fragcap-core"];
        let problems = order_violations(&short, &[("fragcap-profile", "fragcap-core")]);
        assert_eq!(problems.len(), 1);
        assert!(problems[0].contains("absent from the publication order"));
    }
}
