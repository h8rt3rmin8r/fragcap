// SPDX-License-Identifier: Apache-2.0

//! Registry publication in dependency order.
//!
//! crates.io rejects a crate whose dependencies are not already in the
//! registry, so the ten crates go up in a fixed order. That order is
//! written here and asserted against the dependency graph in `deps`, so it
//! cannot drift away from the architecture it is derived from. A reordering
//! that breaks publication fails a unit test rather than a release.
//!
//! Publication is never the default. Without `--execute` this prints the plan
//! and changes nothing. Publishing is irreversible: a version can be yanked
//! but never replaced, so it is not something a caller should be able to do
//! by accident.
//!
//! A run is resumable. Uploading ten crates is ten separate network
//! operations, and any of them can be interrupted by a failure, a cancelled
//! job, or a dropped connection. A crate whose version is already on the
//! registry is skipped rather than treated as an error, so rerunning after a
//! partial release continues from where it stopped instead of failing on the
//! first crate and leaving the release permanently half published.

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
    "fragcap-targets",
    "fragcap-proxy",
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

/// What happened to one crate.
#[derive(Debug, PartialEq, Eq)]
pub enum Outcome {
    Published,
    /// The version is already on the registry. The end state the caller
    /// wanted, so not an error.
    AlreadyPresent,
    Failed,
}

/// Classify one `cargo publish` invocation from its exit status and output.
///
/// Cargo refuses a duplicate version locally after refreshing the index, with
/// "already exists on crates.io index". A duplicate that reaches the server
/// instead comes back as "already uploaded". Both mean the version is present,
/// which is what the caller was trying to achieve, so both are a skip.
///
/// Matching on text is unpleasant, and it is what is available: cargo offers
/// no skip-existing flag and no distinct exit code for this case. The
/// alternative, querying the registry over HTTP, would put a network client
/// into a crate that deliberately has no external dependencies.
pub fn classify(success: bool, output: &str) -> Outcome {
    if success {
        return Outcome::Published;
    }
    let lower = output.to_lowercase();
    if lower.contains("already exists on crates.io index")
        || lower.contains("already exists on the registry")
        || lower.contains("already uploaded")
    {
        return Outcome::AlreadyPresent;
    }
    Outcome::Failed
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

    let mut uploaded = 0usize;
    let mut skipped = 0usize;

    for name in ORDER {
        println!("publish: {name}");
        let (ok, output) = crate::cargo_captured(&["publish", "-p", name]);

        match classify(ok, &output) {
            Outcome::Published => uploaded += 1,
            Outcome::AlreadyPresent => {
                println!("publish: {name} is already on the registry at this version, skipping");
                skipped += 1;
            }
            Outcome::Failed => {
                // Stop rather than continue. Every later crate depends on
                // something in the part that did not go up, so continuing
                // would turn one failure into a run of confusing ones.
                eprintln!("publish: {name} failed. Later crates were not attempted.");
                eprintln!("publish: rerunning is safe. Crates already uploaded are skipped.");
                return 1;
            }
        }
    }

    println!("publish: {uploaded} uploaded, {skipped} already present");
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

    // The exact wording cargo produced when asked to publish a version that
    // was already on the registry, captured from a real run on 2026-08-08.
    const ALREADY: &str = "error: crate fragcap-core@0.1.0 already exists on crates.io index";

    #[test]
    fn a_successful_publish_is_published() {
        assert_eq!(classify(true, ""), Outcome::Published);
    }

    #[test]
    fn an_existing_version_is_a_skip_not_a_failure() {
        assert_eq!(classify(false, ALREADY), Outcome::AlreadyPresent);
    }

    #[test]
    fn a_server_side_duplicate_is_also_a_skip() {
        let body = "the remote server responded with an error (status 400 Bad Request): \
                    crate version `0.1.0` is already uploaded";
        assert_eq!(classify(false, body), Outcome::AlreadyPresent);
    }

    #[test]
    fn a_real_failure_is_not_swallowed() {
        assert_eq!(
            classify(false, "error: failed to get a 200 OK response"),
            Outcome::Failed
        );
        assert_eq!(
            classify(false, "error: 429 Too Many Requests"),
            Outcome::Failed
        );
    }

    #[test]
    fn a_crate_absent_from_the_order_is_rejected() {
        let short = ["fragcap-core"];
        let problems = order_violations(&short, &[("fragcap-profile", "fragcap-core")]);
        assert_eq!(problems.len(), 1);
        assert!(problems[0].contains("absent from the publication order"));
    }
}
