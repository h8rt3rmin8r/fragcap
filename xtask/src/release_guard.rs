// SPDX-License-Identifier: Apache-2.0

//! Release configuration verification, never deployment approval.

use serde_json::Value;
use std::fs;
use std::io::{self, Read};
use std::path::Path;

const MAX_DOCUMENT: usize = 1_048_576;

fn validate_policy(environment: &[u8], policies: &[u8]) -> Vec<String> {
    let parse = |bytes: &[u8]| {
        (bytes.len() <= MAX_DOCUMENT)
            .then(|| serde_json::from_slice::<Value>(bytes).ok())
            .flatten()
            .filter(Value::is_object)
    };
    let (Some(environment), Some(policies)) = (parse(environment), parse(policies)) else {
        return vec![
            "protection documents must be complete JSON objects no larger than 1 MiB".into(),
        ];
    };
    let mut out = Vec::new();
    if environment["id"].as_u64() != Some(21_294_112_472)
        || environment["name"].as_str() != Some("crates-io")
    {
        out.push("expected the existing crates-io environment identity".into());
    }
    if environment["can_admins_bypass"].as_bool() != Some(true) {
        out.push(
            "administrator bypass must remain explicitly enabled at the operator's direction"
                .into(),
        );
    }
    let mut reviewers = 0;
    let mut branch_rules = 0;
    match environment["protection_rules"].as_array() {
        None => out.push("missing complete protection-rule inventory".into()),
        Some(rules) => {
            for rule in rules {
                match rule["type"].as_str() {
                    Some("required_reviewers") => {
                        reviewers += 1;
                        let exact_owner = rule["reviewers"].as_array().is_some_and(|list| {
                            list.len() == 1
                                && list[0]["type"].as_str() == Some("User")
                                && list[0]["reviewer"]["id"].as_u64() == Some(46_768_484)
                                && list[0]["reviewer"]["login"].as_str() == Some("h8rt3rmin8r")
                        });
                        if !exact_owner || rule["prevent_self_review"].as_bool() != Some(false) {
                            out.push("require only the operator reviewer with owner-initiated approval possible".into());
                        }
                    }
                    Some("branch_policy") => branch_rules += 1,
                    Some("wait_timer")
                        if rule["wait_timer"].as_u64().is_some_and(|n| n <= 43_200) => {}
                    _ => out
                        .push("unknown or malformed environment protection requires review".into()),
                }
            }
        }
    }
    if reviewers != 1 || branch_rules != 1 {
        out.push(
            "exactly one required-reviewer and one deployment-policy rule are required".into(),
        );
    }
    if environment["deployment_branch_policy"]["protected_branches"].as_bool() != Some(false)
        || environment["deployment_branch_policy"]["custom_branch_policies"].as_bool() != Some(true)
    {
        out.push("custom tag-only deployment policy must be enabled".into());
    }
    let exact_tag = policies["branch_policies"].as_array().is_some_and(|list| {
        policies["total_count"].as_u64() == Some(list.len() as u64)
            && list.len() == 1
            && list[0]["id"].as_u64().is_some_and(|id| id > 0)
            && list[0]["type"].as_str() == Some("tag")
            && list[0]["name"].as_str() == Some("v*")
    });
    if !exact_tag {
        out.push(
            "require one complete tag-only v* allowance inventory (not branches or partial pages)"
                .into(),
        );
    }
    out
}

// This exact bounded step is checked within each effect-owning job. Matching
// normalized lines also rejects conditional/skippable or continue-on-error guards.
const GUARD_STEP: &str = r#"
      - name: Verify registry approval protection
        shell: bash
        env:
          GH_TOKEN: ${{ github.token }}
        run: |
          set -euo pipefail
          gh api --method GET repos/h8rt3rmin8r/fragcap/environments/crates-io > "$RUNNER_TEMP/registry-environment.json"
          gh api --method GET 'repos/h8rt3rmin8r/fragcap/environments/crates-io/deployment-branch-policies?per_page=100' > "$RUNNER_TEMP/registry-policies.json"
          cargo run --quiet --package xtask -- release-guard "$RUNNER_TEMP/registry-environment.json" "$RUNNER_TEMP/registry-policies.json"
"#;

fn job<'a>(workflow: &'a str, name: &str) -> Option<&'a str> {
    let marker = format!("\n  {name}:\n");
    let start = workflow.find(&marker)? + marker.len();
    let rest = &workflow[start..];
    let end = rest
        .match_indices("\n  ")
        .find(|(offset, _)| {
            let line = rest[*offset + 1..].lines().next().unwrap_or_default();
            !line.starts_with("   ") && line.ends_with(':')
        })
        .map_or(rest.len(), |(offset, _)| offset);
    Some(&rest[..end])
}

fn normalized(text: &str) -> Vec<&str> {
    text.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect()
}

fn has_guard(job: &str) -> bool {
    let marker = "      - name: Verify registry approval protection\n";
    if job.matches(marker).count() != 1 {
        return false;
    }
    let start = job.find(marker).unwrap();
    let rest = &job[start..];
    let end = rest.find("\n      - ").unwrap_or(rest.len());
    normalized(&rest[..end]) == normalized(GUARD_STEP)
}

fn validate_workflow(workflow: &str) -> Vec<String> {
    let mut out = Vec::new();
    for name in ["identity", "publish"] {
        if !job(workflow, name).is_some_and(has_guard) {
            out.push(format!(
                "{name} job must contain the exact fresh fail-closed protection step"
            ));
        }
    }
    for (name, requirement) in [
        ("package-certification", "needs: identity"),
        ("release", "needs: package-certification"),
        ("publish", "needs: release"),
        ("publish", "environment: crates-io"),
    ] {
        if !job(workflow, name).is_some_and(|block| normalized(block).contains(&requirement)) {
            out.push(format!("{name} job requires {requirement}"));
        }
    }
    if !job(workflow, "identity").is_some_and(|block| {
        let tag = block.find("test \"$tagged\" = \"$declared\"");
        let guard = block.find("- name: Verify registry approval protection");
        matches!((tag, guard), (Some(tag), Some(guard)) if tag < guard)
    }) {
        out.push("identity must check exact tag/version agreement before protection".into());
    }
    if !job(workflow, "publish").is_some_and(|block| {
        let guard = block.find("- name: Verify registry approval protection");
        let publish = block.find("run: cargo run --package xtask -- publish --execute");
        matches!((guard, publish), (Some(guard), Some(publish)) if guard < publish)
    }) {
        out.push("publication must follow fresh protection verification".into());
    }
    out
}

fn read_bounded(path: &Path) -> io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    fs::File::open(path)?
        .take((MAX_DOCUMENT + 1) as u64)
        .read_to_end(&mut bytes)?;
    Ok(bytes)
}

pub fn run(root: &Path, args: &[String]) -> io::Result<usize> {
    if !matches!(args.len(), 0 | 2) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "use release-guard [environment.json policies.json]",
        ));
    }
    let mut problems = validate_workflow(&fs::read_to_string(
        root.join(".github/workflows/release.yml"),
    )?);
    if args.len() == 2 {
        problems.extend(validate_policy(
            &read_bounded(Path::new(&args[0]))?,
            &read_bounded(Path::new(&args[1]))?,
        ));
    }
    for problem in &problems {
        eprintln!("release-guard: {problem}");
    }
    if problems.is_empty() {
        println!("release-guard: workflow wiring verified{}; no deployment approved or release published", if args.is_empty() { " (offline)" } else { " and supplied protection configuration verified" });
    }
    Ok(problems.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn policy() -> (Value, Value) {
        (
            json!({"id":21294112472_u64,"name":"crates-io","can_admins_bypass":true,
                "protection_rules":[{"type":"required_reviewers","prevent_self_review":false,
                    "reviewers":[{"type":"User","reviewer":{"id":46768484,"login":"h8rt3rmin8r"}}]},
                    {"type":"branch_policy"}],
                "deployment_branch_policy":{"protected_branches":false,"custom_branch_policies":true}}),
            json!({"total_count":1,"branch_policies":[{"id":123,"type":"tag","name":"v*"}]}),
        )
    }

    fn problems(environment: &Value, policies: &Value) -> Vec<String> {
        validate_policy(
            &serde_json::to_vec(environment).unwrap(),
            &serde_json::to_vec(policies).unwrap(),
        )
    }

    #[test]
    fn exact_owner_approval_policy_is_configuration_not_approval() {
        let (environment, policies) = policy();
        assert!(problems(&environment, &policies).is_empty());
    }

    #[test]
    fn missing_or_unapprovable_reviewer_and_admin_policy_drift_are_refused() {
        let (environment, policies) = policy();
        for mutation in [
            "missing-review",
            "self-review",
            "bypass",
            "other-owner",
            "other-id",
            "extra-reviewer",
            "missing-bypass",
            "missing-self-review",
            "duplicate-review",
        ] {
            let mut changed = environment.clone();
            match mutation {
                "missing-review" => changed["protection_rules"] = json!([]),
                "self-review" => {
                    changed["protection_rules"][0]["prevent_self_review"] = json!(true)
                }
                "bypass" => changed["can_admins_bypass"] = json!(false),
                "other-owner" => {
                    changed["protection_rules"][0]["reviewers"][0]["reviewer"]["login"] =
                        json!("other")
                }
                "other-id" => {
                    changed["protection_rules"][0]["reviewers"][0]["reviewer"]["id"] = json!(1)
                }
                "extra-reviewer" => changed["protection_rules"][0]["reviewers"]
                    .as_array_mut()
                    .unwrap()
                    .push(json!({"type":"Team"})),
                "missing-bypass" => {
                    changed.as_object_mut().unwrap().remove("can_admins_bypass");
                }
                "missing-self-review" => {
                    changed["protection_rules"][0]
                        .as_object_mut()
                        .unwrap()
                        .remove("prevent_self_review");
                }
                "duplicate-review" => {
                    let duplicate = changed["protection_rules"][0].clone();
                    changed["protection_rules"]
                        .as_array_mut()
                        .unwrap()
                        .push(duplicate);
                }
                _ => unreachable!(),
            }
            assert!(
                !problems(&changed, &policies).is_empty(),
                "accepted {mutation}"
            );
        }
    }

    #[test]
    fn wrong_environment_unknown_protection_and_unrestricted_policy_are_refused() {
        let (environment, policies) = policy();
        for mutation in [
            "name",
            "id",
            "unknown",
            "null-policy",
            "protected",
            "missing-rules",
        ] {
            let mut changed = environment.clone();
            match mutation {
                "name" => changed["name"] = json!("other"),
                "id" => changed["id"] = json!(1),
                "unknown" => changed["protection_rules"]
                    .as_array_mut()
                    .unwrap()
                    .push(json!({"type":"future-rule"})),
                "null-policy" => changed["deployment_branch_policy"] = Value::Null,
                "protected" => {
                    changed["deployment_branch_policy"]["protected_branches"] = json!(true)
                }
                "missing-rules" => {
                    changed.as_object_mut().unwrap().remove("protection_rules");
                }
                _ => unreachable!(),
            }
            assert!(
                !problems(&changed, &policies).is_empty(),
                "accepted {mutation}"
            );
        }
    }

    #[test]
    fn wider_branch_duplicate_and_partial_allowances_are_refused() {
        let (environment, policies) = policy();
        for mutation in [
            "branch",
            "wildcard",
            "missing-type",
            "partial",
            "duplicate",
            "missing-count",
            "empty",
            "invalid-id",
        ] {
            let mut changed = policies.clone();
            match mutation {
                "branch" => changed["branch_policies"][0]["type"] = json!("branch"),
                "wildcard" => changed["branch_policies"][0]["name"] = json!("*"),
                "missing-type" => {
                    changed["branch_policies"][0]
                        .as_object_mut()
                        .unwrap()
                        .remove("type");
                }
                "partial" => changed["total_count"] = json!(101),
                "duplicate" => {
                    changed["total_count"] = json!(2);
                    let duplicate = changed["branch_policies"][0].clone();
                    changed["branch_policies"]
                        .as_array_mut()
                        .unwrap()
                        .push(duplicate);
                }
                "missing-count" => {
                    changed.as_object_mut().unwrap().remove("total_count");
                }
                "empty" => {
                    changed["branch_policies"] = json!([]);
                    changed["total_count"] = json!(0);
                }
                "invalid-id" => changed["branch_policies"][0]["id"] = json!(0),
                _ => unreachable!(),
            }
            assert!(
                !problems(&environment, &changed).is_empty(),
                "accepted {mutation}"
            );
        }
    }

    #[test]
    fn malformed_missing_and_oversized_documents_are_refused() {
        let (environment, policies) = policy();
        let environment = serde_json::to_vec(&environment).unwrap();
        let policies = serde_json::to_vec(&policies).unwrap();
        for bad in [
            b"".to_vec(),
            b"null".to_vec(),
            b"{".to_vec(),
            vec![b' '; 1_048_577],
        ] {
            assert!(!validate_policy(&bad, &policies).is_empty());
            assert!(!validate_policy(&environment, &bad).is_empty());
        }
    }

    #[test]
    fn service_metadata_and_valid_wait_timer_do_not_remove_other_protection() {
        let (mut environment, mut policies) = policy();
        environment["updated_at"] = json!("service metadata");
        environment["protection_rules"]
            .as_array_mut()
            .unwrap()
            .push(json!({"type":"wait_timer","wait_timer":30}));
        policies["branch_policies"][0]["node_id"] = json!("service id");
        assert!(problems(&environment, &policies).is_empty());
    }

    #[test]
    fn actual_release_workflow_has_both_fresh_guards_and_approval_binding() {
        assert!(validate_workflow(include_str!("../../.github/workflows/release.yml")).is_empty());
    }

    #[test]
    fn removing_approval_or_dependency_wiring_is_refused() {
        let actual = include_str!("../../.github/workflows/release.yml");
        for (from, to) in [
            ("environment: crates-io", "environment: unprotected"),
            ("needs: identity", "needs: missing"),
            ("needs: package-certification", "needs: identity"),
            ("needs: release", "needs: identity"),
        ] {
            assert!(
                !validate_workflow(&actual.replace(from, to)).is_empty(),
                "accepted {from}"
            );
        }
    }

    #[test]
    fn missing_stale_conditional_or_nonblocking_guards_are_refused() {
        let actual = include_str!("../../.github/workflows/release.yml");
        for (from, to) in [
            ("      - name: Verify registry approval protection\n", "      - name: Skip registry approval protection\n"),
            ("gh api --method GET repos/h8rt3rmin8r/fragcap/environments/crates-io", "echo stale-environment"),
            ("per_page=100", "per_page=1"),
            ("set -euo pipefail", "set +e"),
            ("        shell: bash\n        env:\n          GH_TOKEN:", "        if: false\n        shell: bash\n        env:\n          GH_TOKEN:"),
            ("        shell: bash\n        env:\n          GH_TOKEN:", "        continue-on-error: true\n        shell: bash\n        env:\n          GH_TOKEN:"),
            ("-- release-guard", "-- notes"),
        ] {
            assert!(!validate_workflow(&actual.replace(from, to)).is_empty(), "accepted guard mutation {from}");
        }
    }

    #[test]
    fn bounded_file_reads_and_bad_invocation_do_not_claim_verification() {
        let root = crate::repo_root();
        assert!(run(&root, &["only-one-input".into()]).is_err());
        assert!(read_bounded(&root.join("target/s152-missing-protection-input.json")).is_err());
    }
}
