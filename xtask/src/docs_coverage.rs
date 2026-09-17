// SPDX-License-Identifier: Apache-2.0

//! Closed native documentation traceability, not independent review acceptance.

use serde_json::Value;
use std::{collections::BTreeSet, fs, io, path::Path};

const REGISTRY: &str = "docs/audits/native-documentation-coverage.v2.json";
const TOPICS: [&str; 13] = [
    "architecture",
    "setup",
    "capture-modes",
    "cli",
    "protocols-routing",
    "refusals",
    "artifacts-correlation",
    "diagnostics-recovery",
    "security-privacy",
    "bounds",
    "packaging-migration",
    "public-api",
    "completion-status",
];
const EXAMPLE_AUTHORITIES: [&str; 5] = [
    "current-command-corpus",
    "manifest-specimens",
    "packet-goldens",
    "native-bundle-contract",
    "stable-api-contract",
];
const CURRENT_SOURCE_BOUNDARY: [&str; 4] = ["S154", "S155", "S156", "S157"];
const COMPLETION_BOUNDARY: [u64; 5] = [333, 413, 331, 334, 278];
const OBSOLETE_CURRENT_TEXT: [&str; 3] = [
    "S154's coverage inventory and regression improvements are candidate",
    "mitmdump",
    "external proxy backend",
];

pub fn run(root: &Path) -> io::Result<Vec<String>> {
    let inventory: Value = serde_json::from_str(&fs::read_to_string(root.join(REGISTRY))?)
        .map_err(io::Error::other)?;
    let mut problems = validate(root, &inventory);
    if problems.is_empty() {
        let mut areas = inventory["topics"].as_array().cloned().unwrap_or_default();
        areas.extend(
            inventory["example_authorities"]
                .as_array()
                .into_iter()
                .flatten()
                .map(|authority| serde_json::json!({"tests": authority["tests"]})),
        );
        problems.extend(crate::review_handoff::discover_references(
            root,
            &serde_json::json!({"areas": areas}),
        )?);
    }
    Ok(problems)
}

fn exact_keys(value: &Value, keys: &[&str]) -> bool {
    value.as_object().is_some_and(|object| {
        object.len() == keys.len() && keys.iter().all(|key| object.contains_key(*key))
    })
}

fn confined_path(root: &Path, path: &str) -> Result<std::path::PathBuf, String> {
    if path.contains('\\')
        || path.contains(':')
        || path.split('/').any(|part| matches!(part, "" | "." | ".."))
    {
        return Err("not a confined repository-relative path".into());
    }
    let boundary = root.canonicalize().map_err(|error| error.to_string())?;
    let candidate = root
        .join(path)
        .canonicalize()
        .map_err(|error| error.to_string())?;
    if !candidate.starts_with(boundary) {
        return Err("resolved outside repository".into());
    }
    Ok(candidate)
}

fn confined_text(root: &Path, path: &str) -> Result<String, String> {
    let candidate = confined_path(root, path)?;
    let metadata = candidate.metadata().map_err(|error| error.to_string())?;
    if !metadata.is_file() || metadata.len() > 1_048_576 {
        return Err("not a bounded regular file".into());
    }
    fs::read_to_string(candidate).map_err(|error| error.to_string())
}

fn current_documentation_pages(root: &Path) -> Result<Vec<String>, String> {
    fn walk(root: &Path, directory: &Path, pages: &mut Vec<String>) -> Result<(), String> {
        let mut entries = fs::read_dir(directory)
            .map_err(|error| format!("cannot read {}: {error}", directory.display()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| format!("cannot enumerate {}: {error}", directory.display()))?;
        entries.sort_by_key(fs::DirEntry::path);
        for entry in entries {
            let path = entry.path();
            let file_type = entry
                .file_type()
                .map_err(|error| format!("cannot inspect {}: {error}", path.display()))?;
            if file_type.is_symlink() {
                return Err(format!(
                    "current documentation path is a symbolic link: {}",
                    path.display()
                ));
            }
            if file_type.is_dir() {
                let name = path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("");
                if !matches!(name, "changelog" | "glossary") {
                    walk(root, &path, pages)?;
                }
            } else if file_type.is_file()
                && path.extension().is_some_and(|extension| extension == "mdx")
            {
                let relative = path
                    .strip_prefix(root)
                    .map_err(|error| error.to_string())?
                    .to_string_lossy()
                    .replace('\\', "/");
                pages.push(relative);
            }
        }
        Ok(())
    }

    let mut pages = Vec::new();
    walk(root, &root.join("site/content/docs"), &mut pages)?;
    pages.sort();
    Ok(pages)
}

fn obsolete_text_problems(page: &str, source: &str) -> Vec<String> {
    let folded = source.to_ascii_lowercase();
    OBSOLETE_CURRENT_TEXT
        .iter()
        .filter(|obsolete| folded.contains(&obsolete.to_ascii_lowercase()))
        .map(|obsolete| format!("current page {page}: obsolete text `{obsolete}`"))
        .collect()
}

fn markdown_fence(line: &str) -> Option<(u8, usize, &str)> {
    let indentation = line
        .bytes()
        .take_while(|candidate| *candidate == b' ')
        .count();
    if indentation > 3 {
        return None;
    }
    let trimmed = &line[indentation..];
    let marker = *trimmed.as_bytes().first()?;
    if !matches!(marker, b'`' | b'~') {
        return None;
    }
    let width = trimmed
        .as_bytes()
        .iter()
        .take_while(|candidate| **candidate == marker)
        .count();
    (width >= 3).then(|| (marker, width, &trimmed[width..]))
}

fn authored_headings(source: &str) -> Vec<String> {
    let mut headings = Vec::new();
    let mut fence = None;
    for line in source.lines() {
        if let Some((marker, width, suffix)) = markdown_fence(line) {
            match fence {
                None => {
                    fence = Some((marker, width));
                    continue;
                }
                Some((opening_marker, opening_width))
                    if marker == opening_marker
                        && width >= opening_width
                        && suffix.trim().is_empty() =>
                {
                    fence = None;
                    continue;
                }
                Some(_) => {}
            }
        }
        if fence.is_none() {
            if let Some(heading) = line
                .strip_prefix("## ")
                .or_else(|| line.strip_prefix("### "))
            {
                headings.push(heading.to_string());
            }
        }
    }
    headings
}

fn valid_string_array(value: &Value, nonempty: bool) -> bool {
    value.as_array().is_some_and(|values| {
        (!nonempty || !values.is_empty())
            && values
                .iter()
                .all(|value| value.as_str().is_some_and(|value| !value.is_empty()))
            && values
                .iter()
                .map(Value::to_string)
                .collect::<BTreeSet<_>>()
                .len()
                == values.len()
    })
}

fn validate_tests(root: &Path, owner: &str, tests: &Value, problems: &mut Vec<String>) {
    if tests.as_array().is_none_or(Vec::is_empty) {
        problems.push(format!("{owner}: no executable test authorities"));
        return;
    }
    let mut seen = BTreeSet::new();
    for test in tests.as_array().into_iter().flatten() {
        let path = test["path"].as_str().unwrap_or("");
        let function = test["function"].as_str().unwrap_or("");
        if !exact_keys(test, &["path", "function", "features"]) || !seen.insert((path, function)) {
            problems.push(format!("{owner}: malformed or duplicate test authority"));
        }
        if !(path.starts_with("crates/") || path.starts_with("xtask/src/"))
            || !path.ends_with(".rs")
            || function.is_empty()
            || !function
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
        {
            problems.push(format!(
                "{owner}: invalid test reference {path}::{function}"
            ));
            continue;
        }
        match confined_text(root, path) {
            Ok(source) => {
                if let Err(reason) = crate::threat_model::validate_test_source(&source, function) {
                    problems.push(format!("{owner} test {path}::{function}: {reason}"));
                }
            }
            Err(reason) => problems.push(format!("{owner} test {path}: {reason}")),
        }
        if !valid_string_array(&test["features"], false) {
            problems.push(format!(
                "{owner}: invalid or duplicate feature declarations"
            ));
        }
    }
}

fn validate(root: &Path, inventory: &Value) -> Vec<String> {
    let mut problems = Vec::new();
    if !exact_keys(
        inventory,
        &[
            "schema_version",
            "published_baseline",
            "current_source_boundary",
            "topics",
            "example_authorities",
            "completion_boundary",
        ],
    ) || inventory["schema_version"].as_u64() != Some(2)
    {
        problems.push(
            "inventory must declare the exact documentation contract version 2 fields".into(),
        );
    }

    let published = &inventory["published_baseline"];
    if !exact_keys(published, &["version", "source_revision"]) {
        problems.push("published baseline must declare exactly version and source_revision".into());
    }
    match confined_text(root, "docs/published-release.json").and_then(|source| {
        serde_json::from_str::<Value>(&source).map_err(|error| error.to_string())
    }) {
        Ok(release)
            if published["version"] == release["version"]
                && published["source_revision"] == release["source_revision"] => {}
        Ok(_) => {
            problems.push("published baseline does not match docs/published-release.json".into())
        }
        Err(reason) => problems.push(format!("published release authority: {reason}")),
    }

    let source_boundary: Vec<_> = inventory["current_source_boundary"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .collect();
    if source_boundary != CURRENT_SOURCE_BOUNDARY {
        problems.push("current source boundary must be exactly S154 through S157".into());
    }

    let mut topic_ids = BTreeSet::new();
    let mut authority_references = Vec::new();
    for (index, topic) in inventory["topics"]
        .as_array()
        .into_iter()
        .flatten()
        .enumerate()
    {
        let id = topic["id"].as_str().unwrap_or("");
        if !TOPICS.contains(&id) || !topic_ids.insert(id) {
            problems.push(format!("unknown or duplicate topic {id}"));
        }
        if TOPICS.get(index).copied() != Some(id) {
            problems.push(format!("topic {id}: not in canonical order"));
        }
        if !exact_keys(
            topic,
            &["id", "page", "sections", "tests", "example_authorities"],
        ) {
            problems.push(format!(
                "topic {id}: fields are not the closed version 2 schema"
            ));
        }
        let page = topic["page"].as_str().unwrap_or("");
        let current = page.starts_with("site/content/docs/")
            && page.ends_with(".mdx")
            && !page
                .split('/')
                .any(|part| matches!(part, "changelog" | "glossary"));
        if !current {
            problems.push(format!("topic {id}: not a current authored page: {page}"));
        } else {
            match confined_text(root, page) {
                Ok(source) => {
                    let headings = authored_headings(&source);
                    if !valid_string_array(&topic["sections"], true) {
                        problems.push(format!(
                            "topic {id}: invalid or duplicate required sections"
                        ));
                    }
                    let mut previous = None;
                    for section in topic["sections"].as_array().into_iter().flatten() {
                        let Some(section) = section.as_str() else {
                            continue;
                        };
                        let matches: Vec<_> = headings
                            .iter()
                            .enumerate()
                            .filter_map(|(position, heading)| {
                                (heading == section).then_some(position)
                            })
                            .collect();
                        if matches.len() != 1 {
                            problems.push(format!(
                                "topic {id}: required section `{section}` occurs {} times in {page}",
                                matches.len()
                            ));
                        } else if previous.is_some_and(|position| position >= matches[0]) {
                            problems.push(format!(
                                "topic {id}: required section `{section}` is out of order in {page}"
                            ));
                        } else {
                            previous = Some(matches[0]);
                        }
                    }
                }
                Err(reason) => problems.push(format!("topic {id} page {page}: {reason}")),
            }
        }
        validate_tests(root, &format!("topic {id}"), &topic["tests"], &mut problems);
        if !valid_string_array(&topic["example_authorities"], true) {
            problems.push(format!(
                "topic {id}: invalid or duplicate example authorities"
            ));
        }
        authority_references.extend(
            topic["example_authorities"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
                .map(|authority| (id.to_string(), authority.to_string())),
        );
    }
    for id in TOPICS {
        if !topic_ids.contains(id) {
            problems.push(format!("missing topic {id}"));
        }
    }

    let mut example_ids = BTreeSet::new();
    for (index, authority) in inventory["example_authorities"]
        .as_array()
        .into_iter()
        .flatten()
        .enumerate()
    {
        let id = authority["id"].as_str().unwrap_or("");
        if !EXAMPLE_AUTHORITIES.contains(&id) || !example_ids.insert(id) {
            problems.push(format!("unknown or duplicate example authority {id}"));
        }
        if EXAMPLE_AUTHORITIES.get(index).copied() != Some(id) {
            problems.push(format!("example authority {id}: not in canonical order"));
        }
        if !exact_keys(authority, &["id", "kind", "paths", "tests"]) {
            problems.push(format!("example authority {id}: fields are not closed"));
        }
        let kind = authority["kind"].as_str().unwrap_or("");
        if !matches!(
            kind,
            "command-corpus" | "committed-artifact" | "controlled-contract"
        ) {
            problems.push(format!("example authority {id}: unknown kind {kind}"));
        }
        if !valid_string_array(&authority["paths"], kind == "committed-artifact") {
            problems.push(format!(
                "example authority {id}: invalid or duplicate paths"
            ));
        }
        for path in authority["paths"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
        {
            if let Err(reason) = confined_text(root, path) {
                problems.push(format!("example authority {id} path {path}: {reason}"));
            }
        }
        validate_tests(
            root,
            &format!("example authority {id}"),
            &authority["tests"],
            &mut problems,
        );
    }
    for id in EXAMPLE_AUTHORITIES {
        if !example_ids.contains(id) {
            problems.push(format!("missing example authority {id}"));
        }
    }
    for (topic, authority) in authority_references {
        if !example_ids.contains(authority.as_str()) {
            problems.push(format!(
                "topic {topic}: unknown example authority {authority}"
            ));
        }
    }

    let completion: Vec<_> = inventory["completion_boundary"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_u64)
        .collect();
    if completion != COMPLETION_BOUNDARY {
        problems.push("completion boundary must be exactly #333, #413, #331, #334 and #278".into());
    }

    match current_documentation_pages(root) {
        Ok(pages) => {
            for page in pages {
                match confined_text(root, &page) {
                    Ok(source) => problems.extend(obsolete_text_problems(&page, &source)),
                    Err(reason) => problems.push(format!("current page {page}: {reason}")),
                }
            }
        }
        Err(reason) => problems.push(format!("current documentation corpus: {reason}")),
    }
    problems
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn root() -> &'static Path {
        Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap()
    }

    fn test_authority() -> Value {
        json!({"path":"xtask/src/lint.rs","function":"clean_source_produces_no_findings","features":[]})
    }

    fn fixture() -> Value {
        let release: Value = serde_json::from_str(
            &fs::read_to_string(root().join("docs/published-release.json")).unwrap(),
        )
        .unwrap();
        json!({
            "schema_version":2,
            "published_baseline":{"version":release["version"],"source_revision":release["source_revision"]},
            "current_source_boundary":CURRENT_SOURCE_BOUNDARY,
            "topics":TOPICS.map(|id|json!({
                "id":id,
                "page":"site/content/docs/architecture.mdx",
                "sections":["Capture: passive packet truth"],
                "tests":[test_authority()],
                "example_authorities":["current-command-corpus"]
            })),
            "example_authorities":EXAMPLE_AUTHORITIES.map(|id|json!({
                "id":id,
                "kind":if id == "manifest-specimens" {"committed-artifact"} else {"controlled-contract"},
                "paths":if id == "manifest-specimens" {json!(["docs/schema/examples/deep-capture-manifest-v1.json"])} else {json!([])},
                "tests":[test_authority()]
            })),
            "completion_boundary":COMPLETION_BOUNDARY
        })
    }

    #[test]
    fn complete_version_two_inventory_passes() {
        assert_eq!(validate(root(), &fixture()), Vec::<String>::new());
    }

    #[test]
    fn committed_inventory_has_complete_current_source_authorities() {
        let inventory =
            serde_json::from_str(&fs::read_to_string(root().join(REGISTRY)).unwrap()).unwrap();
        assert_eq!(validate(root(), &inventory), Vec::<String>::new());
    }

    #[test]
    fn schema_keys_version_and_boundaries_are_closed() {
        for mutate in ["version", "key", "release", "source", "completion"] {
            let mut changed = fixture();
            match mutate {
                "version" => changed["schema_version"] = json!(1),
                "key" => changed["invented"] = json!(true),
                "release" => changed["published_baseline"]["version"] = json!("0.0.0"),
                "source" => changed["current_source_boundary"] = json!(["S154"]),
                "completion" => changed["completion_boundary"] = json!([333, 334]),
                _ => unreachable!(),
            }
            assert!(!validate(root(), &changed).is_empty(), "{mutate}");
        }
    }

    #[test]
    fn missing_duplicate_unknown_and_reordered_topics_fail() {
        let mut missing = fixture();
        missing["topics"].as_array_mut().unwrap().pop();
        assert!(validate(root(), &missing)
            .iter()
            .any(|problem| problem.contains("missing topic completion-status")));
        for id in ["architecture", "invented"] {
            let mut changed = fixture();
            changed["topics"][1]["id"] = json!(id);
            assert!(validate(root(), &changed)
                .iter()
                .any(|problem| problem.contains("unknown or duplicate")));
        }
        let mut reordered = fixture();
        reordered["topics"].as_array_mut().unwrap().swap(0, 1);
        assert!(validate(root(), &reordered)
            .iter()
            .any(|problem| problem.contains("canonical order")));
    }

    #[test]
    fn unsafe_absent_historical_and_section_mutations_fail() {
        for page in [
            "site/content/docs/../../outside.mdx",
            "site/content/docs/absent.mdx",
            "site/content/docs/changelog/v0.10.0.mdx",
            "C:/outside.mdx",
        ] {
            let mut changed = fixture();
            changed["topics"][0]["page"] = json!(page);
            assert!(!validate(root(), &changed).is_empty(), "{page}");
        }
        for sections in [
            json!([]),
            json!(["Absent heading"]),
            json!([
                "Capture: passive packet truth",
                "Capture: passive packet truth"
            ]),
            json!([
                "Deep Capture: explicit proxy observations beside Capture",
                "Capture: passive packet truth"
            ]),
        ] {
            let mut changed = fixture();
            changed["topics"][0]["sections"] = sections;
            assert!(!validate(root(), &changed).is_empty());
        }
    }

    #[test]
    fn absent_non_test_and_malformed_feature_authorities_fail() {
        for function in ["invented_test", "run"] {
            let mut changed = fixture();
            changed["topics"][0]["tests"][0]["function"] = json!(function);
            assert!(!validate(root(), &changed).is_empty());
        }
        for features in [
            json!(null),
            json!([1]),
            json!(["deep-capture", "deep-capture"]),
        ] {
            let mut changed = fixture();
            changed["topics"][0]["tests"][0]["features"] = features;
            assert!(!validate(root(), &changed).is_empty());
        }
    }

    #[test]
    fn example_authority_schema_reference_path_and_order_mutations_fail() {
        let mut unknown = fixture();
        unknown["topics"][0]["example_authorities"] = json!(["invented"]);
        assert!(validate(root(), &unknown)
            .iter()
            .any(|problem| problem.contains("unknown example authority")));
        for mutation in ["kind", "path", "duplicate", "order"] {
            let mut changed = fixture();
            match mutation {
                "kind" => changed["example_authorities"][0]["kind"] = json!("approximate"),
                "path" => changed["example_authorities"][1]["paths"] = json!(["../outside"]),
                "duplicate" => {
                    changed["example_authorities"][1]["id"] = json!("current-command-corpus")
                }
                "order" => changed["example_authorities"]
                    .as_array_mut()
                    .unwrap()
                    .swap(0, 1),
                _ => unreachable!(),
            }
            assert!(!validate(root(), &changed).is_empty(), "{mutation}");
        }
    }

    #[test]
    fn heading_discovery_ignores_exact_backtick_and_tilde_fences() {
        assert_eq!(
            authored_headings(
                "## Kept\n~~~~markdown\n## Ignored tilde\n~~~\n## Still ignored tilde\n```\n## Also ignored tilde\n~~~~~\n### Also kept\n```markdown\n## Ignored backtick\n~~~~\n## Still ignored backtick\n````\n### Final\n"
            ),
            ["Kept", "Also kept", "Final"]
        );
    }

    #[test]
    fn obsolete_language_scan_owns_every_nonhistorical_site_page() {
        let pages = current_documentation_pages(root()).unwrap();
        for required in [
            "site/content/docs/index.mdx",
            "site/content/docs/contributing.mdx",
            "site/content/docs/reference/target-schema.mdx",
        ] {
            assert!(pages.iter().any(|page| page == required), "{required}");
        }
        assert!(pages
            .iter()
            .all(|page| !page.contains("/changelog/") && !page.contains("/glossary/")));
        assert_eq!(
            obsolete_text_problems(
                "site/content/docs/reference/unowned.mdx",
                "Configure the external proxy backend here."
            ),
            ["current page site/content/docs/reference/unowned.mdx: obsolete text `external proxy backend`"]
        );
    }
}
