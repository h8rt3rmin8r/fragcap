// SPDX-License-Identifier: Apache-2.0

//! Contract checks between the public CLI reference and the real clap tree.
//!
//! These tests parse arguments but never dispatch a command. They therefore
//! need no driver, elevation, game, store, network, proxy, or trust change.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use clap::{ArgAction, Command};
use regex::Regex;

const TABLE_HEADER: &str = "| Option | Values | Default | Availability | Meaning |";
const OPEN_VALUE: &str = "open";
const NO_DEFAULT: &str = "none";

#[derive(Clone, Debug, Eq, PartialEq)]
struct OptionContract {
    short: Option<char>,
    values: BTreeSet<String>,
    defaults: Vec<String>,
}

#[derive(Clone, Debug)]
struct ReferenceOption {
    contract: OptionContract,
    availability: Availability,
    line: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Availability {
    All,
    Net,
}

impl Availability {
    fn active(self) -> bool {
        self == Self::All || cfg!(feature = "net")
    }
}

#[derive(Clone, Debug)]
struct Reference {
    commands: BTreeMap<String, BTreeMap<String, ReferenceOption>>,
    globals: BTreeMap<String, ReferenceOption>,
}

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn reference_path() -> PathBuf {
    repository_root().join("site/content/docs/reference/cli.mdx")
}

fn read_reference() -> String {
    std::fs::read_to_string(reference_path()).expect("the CLI reference must be readable")
}

fn generated_control(action: &ArgAction) -> bool {
    matches!(
        action,
        ArgAction::Help | ArgAction::HelpShort | ArgAction::HelpLong | ArgAction::Version
    )
}

fn command_contract() -> BTreeMap<String, BTreeMap<String, OptionContract>> {
    fn options(command: &Command, include_globals: bool) -> BTreeMap<String, OptionContract> {
        command
            .get_arguments()
            .filter(|arg| {
                !arg.is_hide_set()
                    && (include_globals || !arg.is_global_set())
                    && !generated_control(arg.get_action())
            })
            .filter_map(|arg| {
                let long = arg.get_long()?.to_string();
                let values = arg
                    .get_possible_values()
                    .into_iter()
                    .filter(|value| !value.is_hide_set())
                    .map(|value| value.get_name().to_string())
                    .collect();
                let defaults = arg
                    .get_default_values()
                    .iter()
                    .map(|value| value.to_string_lossy().into_owned())
                    .collect();
                Some((
                    long,
                    OptionContract {
                        short: arg.get_short(),
                        values,
                        defaults,
                    },
                ))
            })
            .collect()
    }

    fn walk(
        command: &Command,
        parent: &str,
        result: &mut BTreeMap<String, BTreeMap<String, OptionContract>>,
    ) {
        // clap's generated `help` tree is a parser control rather than an
        // authored public command, just as Help/Version ArgActions are parser
        // controls rather than authored options.
        for subcommand in command
            .get_subcommands()
            .filter(|sub| !sub.is_hide_set() && sub.get_name() != "help")
        {
            let path = if parent.is_empty() {
                subcommand.get_name().to_string()
            } else {
                format!("{parent} {}", subcommand.get_name())
            };
            result.insert(path.clone(), options(subcommand, false));
            walk(subcommand, &path, result);
        }
    }

    let mut root = fragcap_cli::command();
    root.build();
    let mut result = BTreeMap::new();
    result.insert("<global>".to_string(), options(&root, true));
    walk(&root, "", &mut result);
    result
}

fn clean_cell(cell: &str) -> String {
    cell.trim().replace('`', "")
}

fn parse_list(cell: &str, empty: &str) -> Result<Vec<String>, String> {
    let cell = clean_cell(cell);
    if cell == empty {
        return Ok(Vec::new());
    }
    let values: Vec<String> = cell
        .split(',')
        .map(str::trim)
        .map(str::to_string)
        .filter(|value| !value.is_empty())
        .collect();
    if values.is_empty() {
        Err(format!(
            "expected `{empty}` or a comma-separated value list"
        ))
    } else {
        Ok(values)
    }
}

fn parse_option_row(line: &str, line_number: usize) -> Result<(String, ReferenceOption), String> {
    let cells: Vec<&str> = line.split('|').collect();
    if cells.len() != 7 || !cells.first().is_some_and(|cell| cell.is_empty()) {
        return Err(format!(
            "invalid reference contract at cli.mdx:{line_number}: option row has the wrong column count"
        ));
    }

    let flags = clean_cell(cells[1]);
    let mut long = None;
    let mut short = None;
    for flag in flags.split(',').map(str::trim) {
        if let Some(name) = flag.strip_prefix("--") {
            if name.is_empty() || long.replace(name.to_string()).is_some() {
                return Err(format!(
                    "invalid reference contract at cli.mdx:{line_number}: `{flags}` has an invalid long option"
                ));
            }
        } else if let Some(name) = flag.strip_prefix('-') {
            let mut chars = name.chars();
            let value = chars.next();
            if value.is_none() || chars.next().is_some() || short.replace(value.unwrap()).is_some()
            {
                return Err(format!(
                    "invalid reference contract at cli.mdx:{line_number}: `{flags}` has an invalid short option"
                ));
            }
        } else {
            return Err(format!(
                "invalid reference contract at cli.mdx:{line_number}: `{flag}` is not an option"
            ));
        }
    }
    let long = long.ok_or_else(|| {
        format!("invalid reference contract at cli.mdx:{line_number}: `{flags}` has no long option")
    })?;

    let values = parse_list(cells[2], OPEN_VALUE)
        .map_err(|error| format!("invalid reference contract at cli.mdx:{line_number}: {error}"))?
        .into_iter()
        .collect();
    let defaults = parse_list(cells[3], NO_DEFAULT)
        .map_err(|error| format!("invalid reference contract at cli.mdx:{line_number}: {error}"))?;
    let availability = match clean_cell(cells[4]).as_str() {
        "all" => Availability::All,
        "net" => Availability::Net,
        other => {
            return Err(format!(
                "invalid reference contract at cli.mdx:{line_number}: availability `{other}` is not `all` or `net`"
            ));
        }
    };

    Ok((
        long,
        ReferenceOption {
            contract: OptionContract {
                short,
                values,
                defaults,
            },
            availability,
            line: line_number,
        },
    ))
}

fn parse_reference(source: &str) -> Result<Reference, Vec<String>> {
    let heading = Regex::new(r"^### `([^`]+)`$").unwrap();
    let lines: Vec<&str> = source.lines().collect();
    let mut commands = BTreeMap::new();
    let mut globals = BTreeMap::new();
    let mut errors = Vec::new();
    let mut current: Option<String> = None;
    let mut saw_globals = false;
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];
        if line == "## Global options" {
            current = Some("<global>".to_string());
            saw_globals = true;
        } else if let Some(captures) = heading.captures(line) {
            let path = captures[1].to_string();
            if commands.insert(path.clone(), BTreeMap::new()).is_some() {
                errors.push(format!(
                    "invalid reference contract at cli.mdx:{}: duplicate command section `{path}`",
                    i + 1
                ));
            }
            current = Some(path);
        } else if line == TABLE_HEADER {
            let owner = current.clone().unwrap_or_default();
            if owner.is_empty() {
                errors.push(format!(
                    "invalid reference contract at cli.mdx:{}: option table has no command section",
                    i + 1
                ));
            }
            i += 2;
            while i < lines.len() && lines[i].starts_with('|') {
                match parse_option_row(lines[i], i + 1) {
                    Ok((long, option)) => {
                        let table = if owner == "<global>" {
                            &mut globals
                        } else {
                            commands.entry(owner.clone()).or_default()
                        };
                        if table.insert(long.clone(), option).is_some() {
                            errors.push(format!(
                                "invalid reference contract at cli.mdx:{}: duplicate `--{long}` row for `{owner}`",
                                i + 1
                            ));
                        }
                    }
                    Err(error) => errors.push(error),
                }
                i += 1;
            }
            continue;
        }
        i += 1;
    }

    if !saw_globals {
        errors.push("invalid reference contract: missing `Global options` section".to_string());
    }
    if errors.is_empty() {
        Ok(Reference { commands, globals })
    } else {
        Err(errors)
    }
}

fn active_options(options: &BTreeMap<String, ReferenceOption>) -> BTreeMap<String, OptionContract> {
    options
        .iter()
        .filter(|(_, option)| option.availability.active())
        .map(|(name, option)| (name.clone(), option.contract.clone()))
        .collect()
}

fn compare_contracts(
    runtime: &BTreeMap<String, BTreeMap<String, OptionContract>>,
    reference: &Reference,
) -> Vec<String> {
    let mut documented = reference.commands.clone();
    documented.insert("<global>".to_string(), reference.globals.clone());
    let runtime_paths: BTreeSet<_> = runtime.keys().cloned().collect();
    let documented_paths: BTreeSet<_> = documented.keys().cloned().collect();
    let mut failures = Vec::new();

    if runtime_paths != documented_paths {
        failures.push(format!(
            "command-tree drift: runtime-only {:?}, reference-only {:?}",
            runtime_paths
                .difference(&documented_paths)
                .collect::<Vec<_>>(),
            documented_paths
                .difference(&runtime_paths)
                .collect::<Vec<_>>()
        ));
    }

    for path in runtime_paths.intersection(&documented_paths) {
        let expected = &runtime[path];
        let actual = active_options(&documented[path]);
        let expected_names: BTreeSet<_> = expected.keys().cloned().collect();
        let actual_names: BTreeSet<_> = actual.keys().cloned().collect();
        if expected_names != actual_names {
            failures.push(format!(
                "command-tree drift for `{path}` options: runtime-only {:?}, reference-only {:?}",
                expected_names.difference(&actual_names).collect::<Vec<_>>(),
                actual_names.difference(&expected_names).collect::<Vec<_>>()
            ));
        }
        for name in expected_names.intersection(&actual_names) {
            if expected[name] != actual[name] {
                let line = documented[path][name].line;
                failures.push(format!(
                    "command-tree drift for `{path} --{name}` at cli.mdx:{line}: runtime {:?}, reference {:?}",
                    expected[name], actual[name]
                ));
            }
        }
    }
    failures
}

#[test]
fn public_reference_matches_command_tree() {
    let reference = parse_reference(&read_reference()).unwrap_or_else(|failures| {
        panic!(
            "{} invalid reference contract finding(s):\n{}",
            failures.len(),
            failures.join("\n")
        )
    });
    let failures = compare_contracts(&command_contract(), &reference);
    assert!(
        failures.is_empty(),
        "{} command-tree drift finding(s):\n{}",
        failures.len(),
        failures.join("\n")
    );
}

fn function_body<'a>(source: &'a str, function: &str) -> &'a str {
    let function_start = source
        .find(&format!("fn {function}"))
        .unwrap_or_else(|| panic!("{function} must exist"));
    let after = &source[function_start..];
    let next_function = after[1..]
        .find("\nfn ")
        .map_or(after.len(), |offset| offset + 1);
    &after[..next_function]
}

fn match_arm_keys(source: &str, function: &str) -> BTreeSet<String> {
    let body = function_body(source, function);
    // Both parser functions put their accepted top-level keys one indent
    // inside the outer match. Nested value matches are indented farther and
    // must not become scheme or modifier names.
    let arm =
        Regex::new(r#"^        "([a-z][a-z0-9-]*)"(?:\s*\|\s*"([a-z][a-z0-9-]*)")*\s*=>"#).unwrap();
    let quoted = Regex::new(r#""([a-z][a-z0-9-]*)""#).unwrap();
    let mut result = BTreeSet::new();
    for line in body.lines() {
        if arm.is_match(line) {
            for capture in quoted.captures_iter(line) {
                result.insert(capture[1].to_string());
            }
        }
    }
    result
}

fn authority_prefix_schemes(source: &str, function: &str) -> BTreeSet<String> {
    let prefix = Regex::new(r#"strip_prefix\("([a-z][a-z0-9-]*)://"\)"#).unwrap();
    prefix
        .captures_iter(function_body(source, function))
        .map(|capture| capture[1].to_string())
        .collect()
}

fn labeled_tokens(source: &str, label: &str) -> Result<BTreeSet<String>, String> {
    let prefix = format!("**{label}**:");
    let line = source
        .lines()
        .find(|line| line.starts_with(&prefix))
        .ok_or_else(|| format!("invalid reference contract: missing `{label}` token line"))?;
    let token = Regex::new(r"`([^`]+)`").unwrap();
    let values: BTreeSet<_> = token
        .captures_iter(line)
        .map(|capture| capture[1].to_string())
        .collect();
    if values.is_empty() {
        Err(format!(
            "invalid reference contract: `{label}` names no tokens"
        ))
    } else {
        Ok(values)
    }
}

#[test]
fn sink_reference_matches_parser() {
    let source = std::fs::read_to_string(repository_root().join("crates/fragcap-cli/src/args.rs"))
        .expect("args.rs must be readable");
    let mut schemes = match_arm_keys(&source, "parse_destination");
    schemes.extend(authority_prefix_schemes(&source, "parse_destination"));
    let modifiers = match_arm_keys(&source, "apply_option");
    let reference = read_reference();
    let documented_schemes = labeled_tokens(&reference, "Accepted sink schemes").unwrap();
    let documented_modifiers = labeled_tokens(&reference, "Accepted sink modifiers").unwrap();

    assert_eq!(
        schemes, documented_schemes,
        "sink-grammar drift: parser schemes and reference schemes differ"
    );
    assert_eq!(
        modifiers, documented_modifiers,
        "sink-grammar drift: parser modifiers and reference modifiers differ"
    );
}

#[derive(Debug, Eq, PartialEq)]
struct Invocation {
    line: usize,
    text: String,
}

fn strip_comment(line: &str) -> String {
    let mut quote = None;
    for (index, character) in line.char_indices() {
        match character {
            '\'' | '"' if quote == Some(character) => quote = None,
            '\'' | '"' if quote.is_none() => quote = Some(character),
            '#' if quote.is_none() => return line[..index].trim_end().to_string(),
            _ => {}
        }
    }
    line.trim_end().to_string()
}

fn invocations(source: &str) -> Vec<Invocation> {
    let mut result = Vec::new();
    let mut in_fence = false;
    let mut pending: Option<Invocation> = None;

    for (index, raw) in source.lines().enumerate() {
        let line_number = index + 1;
        if raw.trim_start().starts_with("```") {
            in_fence = !in_fence;
            continue;
        }
        if !in_fence {
            continue;
        }
        let mut line = strip_comment(raw).trim().to_string();
        let continued = line.ends_with('`') || line.ends_with('\\');
        if continued {
            line.pop();
            line = line.trim_end().to_string();
        }
        if let Some(invocation) = &mut pending {
            if !line.is_empty() {
                invocation.text.push(' ');
                invocation.text.push_str(&line);
            }
            if !continued {
                result.push(pending.take().unwrap());
            }
        } else if line == "fragcap" || line.starts_with("fragcap ") {
            let invocation = Invocation {
                line: line_number,
                text: line,
            };
            if continued {
                pending = Some(invocation);
            } else {
                result.push(invocation);
            }
        }
    }
    if let Some(invocation) = pending {
        result.push(invocation);
    }
    result
}

fn tokenize(command: &str) -> Result<Vec<String>, String> {
    let mut result = Vec::new();
    let mut current = String::new();
    let mut quote = None;
    let mut started = false;
    let mut chars = command.chars().peekable();
    while let Some(character) = chars.next() {
        match character {
            '\'' | '"' if quote == Some(character) => quote = None,
            '\'' | '"' if quote.is_none() => {
                quote = Some(character);
                started = true;
            }
            '\\' if quote == Some('"') && chars.peek() == Some(&'"') => {
                current.push(chars.next().unwrap());
                started = true;
            }
            value if value.is_whitespace() && quote.is_none() => {
                if started {
                    result.push(std::mem::take(&mut current));
                    started = false;
                }
            }
            value => {
                current.push(value);
                started = true;
            }
        }
    }
    if let Some(unclosed) = quote {
        return Err(format!("unclosed `{unclosed}` quote"));
    }
    if started {
        result.push(current);
    }
    Ok(result)
}

#[test]
fn worked_invocations_parse_without_dispatch() {
    let mut failures = Vec::new();
    let invocations = invocations(&read_reference());
    assert!(
        !invocations.is_empty(),
        "invalid reference contract: no executable `fragcap` examples were discovered"
    );
    for invocation in invocations {
        match tokenize(&invocation.text) {
            Ok(argv) => {
                if let Err(error) = fragcap_cli::command().try_get_matches_from(argv) {
                    failures.push(format!(
                        "invalid worked invocation at cli.mdx:{}: {}\n{}",
                        invocation.line,
                        invocation.text,
                        error.render()
                    ));
                }
            }
            Err(error) => failures.push(format!(
                "invalid worked invocation at cli.mdx:{}: {}: {error}",
                invocation.line, invocation.text
            )),
        }
    }
    assert!(
        failures.is_empty(),
        "{} invalid worked invocation(s):\n{}",
        failures.len(),
        failures.join("\n")
    );
}

#[test]
fn reference_parser_rejects_duplicate_sections_and_bad_availability() {
    let source = "## Global options\n\n### `doctor`\n\n### `doctor`\n\n\
                  | Option | Values | Default | Availability | Meaning |\n\
                  | --- | --- | --- | --- | --- |\n\
                  | `--fix` | open | none | sometimes | Bad row. |\n";
    let failures = parse_reference(source).unwrap_err();
    assert!(failures
        .iter()
        .any(|failure| failure.contains("duplicate command section")));
    assert!(failures
        .iter()
        .any(|failure| failure.contains("availability `sometimes`")));
}

#[test]
fn comparison_reports_command_option_alias_value_and_default_drift() {
    fn option(short: Option<char>, values: &[&str], defaults: &[&str]) -> OptionContract {
        OptionContract {
            short,
            values: values.iter().map(|value| (*value).to_string()).collect(),
            defaults: defaults.iter().map(|value| (*value).to_string()).collect(),
        }
    }

    let expected = option(Some('f'), &["one", "two"], &["one"]);
    let runtime = BTreeMap::from([
        ("<global>".to_string(), BTreeMap::new()),
        (
            "sample".to_string(),
            BTreeMap::from([("flag".to_string(), expected.clone())]),
        ),
    ]);
    let base = Reference {
        commands: BTreeMap::from([(
            "sample".to_string(),
            BTreeMap::from([(
                "flag".to_string(),
                ReferenceOption {
                    contract: expected,
                    availability: Availability::All,
                    line: 7,
                },
            )]),
        )]),
        globals: BTreeMap::new(),
    };

    let mut missing_command = base.clone();
    missing_command.commands.clear();
    assert!(compare_contracts(&runtime, &missing_command)[0].contains("runtime-only"));

    let mut missing_option = base.clone();
    missing_option.commands.get_mut("sample").unwrap().clear();
    assert!(compare_contracts(&runtime, &missing_option)[0].contains("options"));

    for changed in [
        option(None, &["one", "two"], &["one"]),
        option(Some('f'), &["one"], &["one"]),
        option(Some('f'), &["one", "two"], &["two"]),
    ] {
        let mut drifted = base.clone();
        drifted
            .commands
            .get_mut("sample")
            .unwrap()
            .get_mut("flag")
            .unwrap()
            .contract = changed;
        let failures = compare_contracts(&runtime, &drifted);
        assert_eq!(failures.len(), 1);
        assert!(failures[0].contains("sample --flag"));
        assert!(failures[0].contains("runtime"));
        assert!(failures[0].contains("reference"));
    }
}

#[test]
fn example_parser_handles_quotes_comments_and_continuations() {
    let source = "```powershell\n\
                  fragcap technologies --path `\n\
                    \"C:\\Games\\Sample Game\" # local fixture\n\
                  ```\n\
                  ```bash\n\
                  fragcap schema validate \\\n                    'fixtures/sample file.json'\n\
                  ```\n";
    assert_eq!(
        invocations(source),
        vec![
            Invocation {
                line: 2,
                text: "fragcap technologies --path \"C:\\Games\\Sample Game\"".to_string(),
            },
            Invocation {
                line: 6,
                text: "fragcap schema validate 'fixtures/sample file.json'".to_string(),
            },
        ]
    );
    assert_eq!(
        tokenize(&invocations(source)[0].text).unwrap(),
        [
            "fragcap",
            "technologies",
            "--path",
            "C:\\Games\\Sample Game"
        ]
    );
}
