// SPDX-License-Identifier: Apache-2.0

//! One authority for the embedded Deep Capture journey and its pasteable commands.

use std::path::{Path, PathBuf};

use clap::ValueEnum;

use crate::{
    cli::{
        CalibrateArgs, DeepCaptureArgs, DeepCaptureCalibrationProtocolArg,
        DeepCaptureProxyFamilyArg,
    },
    exit::CliError,
};

pub(crate) const ROOT_LONG_HELP: &str = r#"First Deep Capture session:
  1. Check the environment:      fragcap doctor
  2. Find the installed game:    fragcap targets discover
  3. Inspect or register it:     fragcap targets show "<installed game>"
                                  fragcap targets add --steam 620
  4. Measure exact support:      fragcap calibrate "<installed game>"
  5. Run after calibration:      fragcap deep-capture "<installed game>" --launch

Deep Capture is explicit, target-scoped proxy inspection. Environment readiness
does not establish target compatibility; calibration records what this exact
target and launch actually demonstrated."#;

pub(crate) const DEEP_CAPTURE_LONG_HELP: &str = r#"First successful session:
  1. fragcap doctor
  2. fragcap targets discover
  3. fragcap targets show "<installed game>"
  4. fragcap calibrate "<installed game>"
  5. fragcap deep-capture "<installed game>" --launch

Steam example:
  fragcap targets add --steam 620
  fragcap calibrate "Portal 2"
  fragcap deep-capture "Portal 2" --launch

Direct or publisher example:
  fragcap targets scan "C:\Games\My Game"
  fragcap calibrate "My Game"
  fragcap deep-capture "My Game" --launch

Troubleshooting examples:
  fragcap calibrate "My Game" --restart-warm
  fragcap deep-capture "My Game" --launch --bundle "<new empty directory>"
  fragcap doctor --fix

Ordinary Deep Capture requires a stored target, a managed cold launch, and
current observed compatibility for that exact case. Inspection is not universal
and does not bypass certificate pinning. Key logs, HAR, and mutual-TLS identity
inputs can expose plaintext, credentials, or private key material. Recover an
interrupted prior session with `fragcap doctor --fix`."#;

pub(crate) const CALIBRATE_LONG_HELP: &str = r#"Guided calibration records only directly observed facts for one stored target.
It begins with reachability, then measures requested or observed protocols under
a fresh visible plan and separate confirmation for every attempt.

Safe path:
  fragcap calibrate "<installed game>"

Resume a paused workflow:
  fragcap calibrate --resume 1

Wait for an operator-managed warm shutdown:
  fragcap calibrate "My Game" --restart-warm

After current reachability and requested protocol facts are established:
  fragcap deep-capture "<installed game>" --launch

Use --candidate only with an exact identity printed by the current refusal.
Launch-case, routing-strategy, proxy-family, and protocol are advanced exact-case
controls. A request is not compatibility evidence until the target is observed."#;

pub(crate) const DOCTOR_LONG_HELP: &str = r#"Doctor checks Capture and Deep Capture environment readiness independently.
A ready environment does not register a game or establish compatibility.

Next commands:
  fragcap targets discover
  fragcap calibrate "<installed game>"

Use `fragcap doctor --fix` only for remediations Doctor reports, including exact
recovery of interrupted Deep Capture sessions."#;

pub(crate) const TARGETS_LONG_HELP: &str = r#"Deep Capture target path:
  fragcap targets discover
  fragcap targets add --steam 620
  fragcap targets show "<installed game>"
  fragcap calibrate "<installed game>"

Registration identifies one installed target. Calibration separately measures
whether its exact managed launch and protocols are currently compatible."#;

pub(crate) const TARGETS_DISCOVER_LONG_HELP: &str = r#"Discovery is read-only and does not claim that a candidate is a game or compatible.

Next commands:
  fragcap targets add --steam 620
  fragcap targets show "<installed game>"
  fragcap calibrate "<installed game>""#;

pub(crate) const TARGETS_ADD_LONG_HELP: &str = r#"Registering creates or updates one stored target. It does not measure Deep Capture compatibility.

Steam example:
  fragcap targets add --steam 620

Next commands:
  fragcap targets show "Portal 2"
  fragcap calibrate "Portal 2""#;

pub(crate) const TARGETS_SHOW_LONG_HELP: &str = r#"Target detail reports stored launch authority and exact local compatibility facts without aggregating a guessed verdict.

Next command when setup is incomplete:
  fragcap calibrate "<installed game>"

Next command after current compatibility is established:
  fragcap deep-capture "<installed game>" --launch"#;

pub(crate) const BUNDLE_CLEANUP_LONG_HELP: &str = r#"This destructive operation removes only sensitive artifacts declared by a completed bundle.
Review the bundle before passing --yes. If cleanup fails, retain manifest.json,
cleanup.jsonl, resources.jsonl, and the session-owner record so
`fragcap doctor --fix` can retry the exact recovery authority.

Confirmed cleanup:
  fragcap bundle cleanup <bundle> --yes"#;

pub(crate) const BUNDLE_EXPORT_LONG_HELP: &str = r#"Export creates a separate share copy and never modifies the source bundle.
The transformation manifest lists every omission. Keep the source protected: it
may contain plaintext application traffic, credentials, client identities, or a
TLS key log.

Share copy:
  fragcap bundle export <bundle> --out <share-copy>"#;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FirstRunRefusal {
    Environment,
    Target,
    Launch,
    Process,
    Compatibility,
    Recovery,
    Bundle,
    Authorization,
    Calibration,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TargetReference<'a> {
    Selector(&'a str),
    Id(i64),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorkflowVerb {
    Calibrate,
    DeepCapture,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct CalibrationCommandContext {
    catalog_db: Option<PathBuf>,
    local_db: Option<PathBuf>,
    launch_case: Option<String>,
    routing_strategy: Option<String>,
    proxy_family: Option<String>,
    protocols: Vec<String>,
}

impl CalibrationCommandContext {
    pub(crate) fn for_calibrate(args: &CalibrateArgs, effective_local_db: Option<PathBuf>) -> Self {
        Self {
            catalog_db: args.catalog_db.clone(),
            local_db: effective_local_db.or_else(|| args.local_db.clone()),
            launch_case: args.launch_case.as_ref().map(value_name),
            routing_strategy: args.routing_strategy.as_ref().map(value_name),
            proxy_family: Some(value_name(
                args.proxy_family
                    .as_ref()
                    .unwrap_or(&DeepCaptureProxyFamilyArg::Ipv4),
            )),
            protocols: args.protocol.iter().map(value_name).collect(),
        }
    }

    pub(crate) fn for_deep_capture(
        args: &DeepCaptureArgs,
        effective_local_db: Option<PathBuf>,
    ) -> Self {
        Self {
            catalog_db: args.catalog_db.clone(),
            local_db: effective_local_db.or_else(|| args.local_db.clone()),
            launch_case: args.launch_case.as_ref().map(value_name),
            routing_strategy: None,
            proxy_family: Some(value_name(&args.proxy_family)),
            protocols: args
                .calibration_protocol
                .iter()
                .filter(|protocol| **protocol != DeepCaptureCalibrationProtocolArg::Routing)
                .map(value_name)
                .collect(),
        }
    }

    fn append_to(&self, command: &mut String) {
        append_path_option(command, "--catalog-db", self.catalog_db.as_deref());
        append_path_option(command, "--local-db", self.local_db.as_deref());
        append_value_option(command, "--launch-case", self.launch_case.as_deref());
        append_value_option(
            command,
            "--routing-strategy",
            self.routing_strategy.as_deref(),
        );
        append_value_option(command, "--proxy-family", self.proxy_family.as_deref());
        for protocol in &self.protocols {
            append_value_option(command, "--protocol", Some(protocol));
        }
    }
}

fn value_name<T: ValueEnum>(value: &T) -> String {
    value
        .to_possible_value()
        .expect("every CLI value enum variant has a possible value")
        .get_name()
        .to_string()
}

fn append_path_option(command: &mut String, flag: &str, path: Option<&Path>) {
    let Some(path) = path.and_then(Path::to_str) else {
        return;
    };
    command.push(' ');
    command.push_str(flag);
    command.push(' ');
    command.push_str(&quote_powershell_argument(path));
}

fn append_value_option(command: &mut String, flag: &str, value: Option<&str>) {
    let Some(value) = value else {
        return;
    };
    command.push(' ');
    command.push_str(flag);
    command.push(' ');
    command.push_str(value);
}

#[cfg(test)]
const FIRST_RUN_REFUSALS: &[FirstRunRefusal] = &[
    FirstRunRefusal::Environment,
    FirstRunRefusal::Target,
    FirstRunRefusal::Launch,
    FirstRunRefusal::Process,
    FirstRunRefusal::Compatibility,
    FirstRunRefusal::Recovery,
    FirstRunRefusal::Bundle,
    FirstRunRefusal::Authorization,
    FirstRunRefusal::Calibration,
];

#[cfg(test)]
struct CommandExample {
    pub(crate) id: &'static str,
    pub(crate) display: &'static str,
    pub(crate) argv: &'static [&'static str],
}

#[cfg(test)]
const COMMAND_EXAMPLES: &[CommandExample] = &[
    CommandExample {
        id: "doctor",
        display: "fragcap doctor",
        argv: &["fragcap", "doctor"],
    },
    CommandExample {
        id: "discover",
        display: "fragcap targets discover",
        argv: &["fragcap", "targets", "discover"],
    },
    CommandExample {
        id: "steam-register",
        display: "fragcap targets add --steam 620",
        argv: &["fragcap", "targets", "add", "--steam", "620"],
    },
    CommandExample {
        id: "direct-scan",
        display: "fragcap targets scan \"C:\\Games\\My Game\"",
        argv: &["fragcap", "targets", "scan", "C:\\Games\\My Game"],
    },
    CommandExample {
        id: "show",
        display: "fragcap targets show \"<installed game>\"",
        argv: &["fragcap", "targets", "show", "My Game"],
    },
    CommandExample {
        id: "calibrate",
        display: "fragcap calibrate \"<installed game>\"",
        argv: &["fragcap", "calibrate", "My Game"],
    },
    CommandExample {
        id: "resume",
        display: "fragcap calibrate --resume 1",
        argv: &["fragcap", "calibrate", "--resume", "1"],
    },
    CommandExample {
        id: "deep-capture",
        display: "fragcap deep-capture \"<installed game>\" --launch",
        argv: &["fragcap", "deep-capture", "My Game", "--launch"],
    },
    CommandExample {
        id: "show-portal",
        display: "fragcap targets show \"Portal 2\"",
        argv: &["fragcap", "targets", "show", "Portal 2"],
    },
    CommandExample {
        id: "calibrate-portal",
        display: "fragcap calibrate \"Portal 2\"",
        argv: &["fragcap", "calibrate", "Portal 2"],
    },
    CommandExample {
        id: "deep-capture-portal",
        display: "fragcap deep-capture \"Portal 2\" --launch",
        argv: &["fragcap", "deep-capture", "Portal 2", "--launch"],
    },
    CommandExample {
        id: "calibrate-direct",
        display: "fragcap calibrate \"My Game\"",
        argv: &["fragcap", "calibrate", "My Game"],
    },
    CommandExample {
        id: "deep-capture-direct",
        display: "fragcap deep-capture \"My Game\" --launch",
        argv: &["fragcap", "deep-capture", "My Game", "--launch"],
    },
    CommandExample {
        id: "calibrate-warm",
        display: "fragcap calibrate \"My Game\" --restart-warm",
        argv: &["fragcap", "calibrate", "My Game", "--restart-warm"],
    },
    CommandExample {
        id: "deep-capture-new-bundle",
        display: "fragcap deep-capture \"My Game\" --launch --bundle \"<new empty directory>\"",
        argv: &[
            "fragcap",
            "deep-capture",
            "My Game",
            "--launch",
            "--bundle",
            "new-bundle",
        ],
    },
    CommandExample {
        id: "doctor-fix",
        display: "fragcap doctor --fix",
        argv: &["fragcap", "doctor", "--fix"],
    },
    CommandExample {
        id: "bundle-cleanup",
        display: "fragcap bundle cleanup <bundle> --yes",
        argv: &["fragcap", "bundle", "cleanup", "bundle", "--yes"],
    },
    CommandExample {
        id: "bundle-export",
        display: "fragcap bundle export <bundle> --out <share-copy>",
        argv: &[
            "fragcap",
            "bundle",
            "export",
            "bundle",
            "--out",
            "share-copy",
        ],
    },
];

pub(crate) fn quote_powershell_argument(value: &str) -> String {
    let mut quoted = String::with_capacity(value.len() + 2);
    quoted.push('"');
    for character in value.chars() {
        match character {
            '`' => quoted.push_str("``"),
            '"' => quoted.push_str("`\""),
            '$' => quoted.push_str("`$"),
            '\r' => quoted.push_str("`r"),
            '\n' => quoted.push_str("`n"),
            '\t' => quoted.push_str("`t"),
            _ => quoted.push(character),
        }
    }
    quoted.push('"');
    quoted
}

pub(crate) fn target_reference<'a>(
    positional: Option<&'a str>,
    explicit: Option<&'a str>,
    id: Option<i64>,
) -> Option<TargetReference<'a>> {
    positional
        .or(explicit)
        .map(TargetReference::Selector)
        .or_else(|| id.map(TargetReference::Id))
}

pub(crate) fn calibration_command(
    target: TargetReference<'_>,
    context: Option<&CalibrationCommandContext>,
) -> String {
    let mut command = match target {
        TargetReference::Selector(selector) => {
            format!("fragcap calibrate {}", quote_powershell_argument(selector))
        }
        TargetReference::Id(id) => format!("fragcap calibrate --id {id}"),
    };
    if let Some(context) = context {
        context.append_to(&mut command);
    }
    command
}

pub(crate) fn deep_capture_command(target: TargetReference<'_>) -> String {
    match target {
        TargetReference::Selector(selector) => format!(
            "fragcap deep-capture {} --launch",
            quote_powershell_argument(selector)
        ),
        TargetReference::Id(id) => format!("fragcap deep-capture --id {id} --launch"),
    }
}

pub(crate) fn next_command(
    category: FirstRunRefusal,
    target: Option<TargetReference<'_>>,
    calibration_context: Option<&CalibrationCommandContext>,
) -> String {
    match category {
        FirstRunRefusal::Environment => "fragcap doctor".to_string(),
        FirstRunRefusal::Target => "fragcap targets discover".to_string(),
        FirstRunRefusal::Launch | FirstRunRefusal::Compatibility | FirstRunRefusal::Calibration => {
            target.map_or_else(
                || "fragcap calibrate \"<installed game>\"".to_string(),
                |value| calibration_command(value, calibration_context),
            )
        }
        FirstRunRefusal::Process => target.map_or_else(
            || "fragcap calibrate \"<installed game>\" --restart-warm".to_string(),
            |value| {
                format!(
                    "{} --restart-warm",
                    calibration_command(value, calibration_context)
                )
            },
        ),
        FirstRunRefusal::Recovery => "fragcap doctor --fix".to_string(),
        FirstRunRefusal::Bundle => target.map_or_else(
            || {
                "fragcap deep-capture \"<installed game>\" --launch --bundle \"<new empty directory>\""
                    .to_string()
            },
            |value| {
                format!(
                    "{} --bundle \"<new empty directory>\"",
                    deep_capture_command(value)
                )
            },
        ),
        FirstRunRefusal::Authorization => target.map_or_else(
            || "fragcap deep-capture \"<installed game>\" --launch".to_string(),
            deep_capture_command,
        ),
    }
}

pub(crate) fn classify_pre_session_refusal(message: &str) -> Option<FirstRunRefusal> {
    let message = message.to_ascii_lowercase();
    if message.contains("recovery") || message.contains("prior deep capture") {
        Some(FirstRunRefusal::Recovery)
    } else if message.contains("registration") && message.contains("plan") {
        Some(FirstRunRefusal::Calibration)
    } else if message.contains("compatibility") || message.contains("protocol") {
        Some(FirstRunRefusal::Compatibility)
    } else if message.contains("bundle") {
        Some(FirstRunRefusal::Bundle)
    } else if message.contains("warm") || message.contains("already running") {
        Some(FirstRunRefusal::Process)
    } else if message.contains("launch") || message.contains("executable") {
        Some(FirstRunRefusal::Launch)
    } else if message.contains("authorization")
        || message.contains("authorize")
        || message.contains("interactive terminal")
        || message.contains("plan")
    {
        Some(FirstRunRefusal::Authorization)
    } else if message.contains("target") || message.contains("local store") {
        Some(FirstRunRefusal::Target)
    } else if message.contains("doctor") || message.contains("environment") {
        Some(FirstRunRefusal::Environment)
    } else {
        None
    }
}

pub(crate) fn actionable_error(
    error: CliError,
    target: Option<TargetReference<'_>>,
    fallback: Option<FirstRunRefusal>,
    verb: WorkflowVerb,
    calibration_context: Option<&CalibrationCommandContext>,
) -> CliError {
    if error.message().contains("Next command:") || error.message().contains("Next commands:") {
        return error;
    }
    if error.message().contains("selector is ambiguous") {
        return ambiguous_target_error(error, verb, calibration_context);
    }
    let Some(category) = classify_pre_session_refusal(error.message()).or(fallback) else {
        return error;
    };
    with_next_command(error, next_command(category, target, calibration_context))
}

fn ambiguous_target_error(
    error: CliError,
    verb: WorkflowVerb,
    calibration_context: Option<&CalibrationCommandContext>,
) -> CliError {
    let commands: Vec<_> = error
        .message()
        .lines()
        .filter_map(|line| line.trim().split('\t').nth(1)?.parse::<i64>().ok())
        .map(|id| match verb {
            WorkflowVerb::Calibrate => {
                calibration_command(TargetReference::Id(id), calibration_context)
            }
            WorkflowVerb::DeepCapture => format!("fragcap deep-capture --id {id} --launch"),
        })
        .collect();
    if commands.is_empty() {
        return error;
    }
    let message = format!(
        "{}\nNext commands:\n  Choose one listed target:\n  {}",
        error.message(),
        commands.join("\n  ")
    );
    match error {
        CliError::Usage(_) => CliError::usage(message),
        CliError::Failure(_) => CliError::failure(message),
    }
}

pub(crate) fn with_next_command(error: CliError, command: impl AsRef<str>) -> CliError {
    let message = format!("{}\nNext command:  {}", error.message(), command.as_ref());
    match error {
        CliError::Usage(_) => CliError::usage(message),
        CliError::Failure(_) => CliError::failure(message),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use clap::Parser;

    use super::*;
    use crate::cli::Cli;

    #[test]
    fn every_displayed_example_has_a_unique_identity_and_parses() {
        let mut ids = HashSet::new();
        let help = [
            ROOT_LONG_HELP,
            DEEP_CAPTURE_LONG_HELP,
            CALIBRATE_LONG_HELP,
            DOCTOR_LONG_HELP,
            TARGETS_LONG_HELP,
            TARGETS_DISCOVER_LONG_HELP,
            TARGETS_ADD_LONG_HELP,
            TARGETS_SHOW_LONG_HELP,
            BUNDLE_CLEANUP_LONG_HELP,
            BUNDLE_EXPORT_LONG_HELP,
        ]
        .join("\n");
        for example in COMMAND_EXAMPLES {
            assert!(
                ids.insert(example.id),
                "duplicate example id {}",
                example.id
            );
            assert!(
                example.display.starts_with("fragcap "),
                "displayed example must be a complete command: {}",
                example.display
            );
            Cli::try_parse_from(example.argv)
                .unwrap_or_else(|error| panic!("{} does not parse: {error}", example.display));
            assert!(
                help.contains(example.display),
                "registered example is not displayed: {}",
                example.display
            );
        }
    }

    #[test]
    fn refusal_inventory_is_closed_and_unique() {
        assert_eq!(FIRST_RUN_REFUSALS.len(), 9);
        let registered: HashSet<_> = COMMAND_EXAMPLES
            .iter()
            .map(|example| example.display)
            .collect();
        for (index, category) in FIRST_RUN_REFUSALS.iter().enumerate() {
            assert!(
                !FIRST_RUN_REFUSALS[..index].contains(category),
                "duplicate refusal category {category:?}"
            );
            let command = next_command(*category, Some(TargetReference::Selector("My Game")), None);
            assert!(
                registered.contains(command.as_str()),
                "refusal category {category:?} has an unregistered next command: {command}"
            );
            let error = actionable_error(
                CliError::usage("setup stopped"),
                Some(TargetReference::Selector("My Game")),
                Some(*category),
                WorkflowVerb::Calibrate,
                None,
            );
            assert_eq!(
                error.message().matches("Next command:").count(),
                1,
                "refusal category {category:?} must emit exactly one next command"
            );
            assert!(
                error.message().contains(&command),
                "refusal category {category:?} emitted a different command: {}",
                error.message()
            );
        }
    }

    #[test]
    fn dynamic_target_commands_preserve_one_argument() {
        assert_eq!(
            calibration_command(TargetReference::Selector("Pokémon \"Élan\" `$HOME"), None,),
            "fragcap calibrate \"Pokémon `\"Élan`\" ```$HOME\""
        );
        assert_eq!(
            deep_capture_command(TargetReference::Selector("My Game")),
            "fragcap deep-capture \"My Game\" --launch"
        );
        assert_eq!(
            calibration_command(TargetReference::Id(123), None),
            "fragcap calibrate --id 123"
        );
        assert_eq!(
            deep_capture_command(TargetReference::Id(123)),
            "fragcap deep-capture --id 123 --launch"
        );
    }

    #[test]
    fn ambiguous_target_guidance_preserves_every_durable_choice() {
        let message = "the selector is ambiguous (2 targets match); select by handle or `--id`:\n  first\t76001\tFirst\n  second\t76002\tSecond";
        let calibrate = actionable_error(
            CliError::usage(message),
            None,
            None,
            WorkflowVerb::Calibrate,
            None,
        );
        assert!(calibrate.message().contains("fragcap calibrate --id 76001"));
        assert!(calibrate.message().contains("fragcap calibrate --id 76002"));
        let deep_capture = actionable_error(
            CliError::usage(message),
            None,
            None,
            WorkflowVerb::DeepCapture,
            None,
        );
        assert!(deep_capture
            .message()
            .contains("fragcap deep-capture --id 76001 --launch"));
        assert!(deep_capture
            .message()
            .contains("fragcap deep-capture --id 76002 --launch"));
        Cli::try_parse_from(["fragcap", "calibrate", "--id", "76001"])
            .expect("first durable choice parses");
        Cli::try_parse_from(["fragcap", "calibrate", "--id", "76002"])
            .expect("second durable choice parses");
        Cli::try_parse_from(["fragcap", "deep-capture", "--id", "76001", "--launch"])
            .expect("first durable Deep Capture choice parses");
        Cli::try_parse_from(["fragcap", "deep-capture", "--id", "76002", "--launch"])
            .expect("second durable Deep Capture choice parses");
    }

    #[test]
    fn calibration_guidance_preserves_exact_store_and_case() {
        let context = CalibrationCommandContext {
            catalog_db: None,
            local_db: Some(PathBuf::from(r"C:\Stores\targets one.db")),
            launch_case: Some("direct-exe-cold".to_string()),
            routing_strategy: Some("child-environment".to_string()),
            proxy_family: Some("ipv6".to_string()),
            protocols: vec!["https".to_string()],
        };
        let command = calibration_command(TargetReference::Id(75_000), Some(&context));
        assert_eq!(
            command,
            "fragcap calibrate --id 75000 --local-db \"C:\\Stores\\targets one.db\" --launch-case direct-exe-cold --routing-strategy child-environment --proxy-family ipv6 --protocol https"
        );
        Cli::try_parse_from([
            "fragcap",
            "calibrate",
            "--id",
            "75000",
            "--local-db",
            r"C:\Stores\targets one.db",
            "--launch-case",
            "direct-exe-cold",
            "--routing-strategy",
            "child-environment",
            "--proxy-family",
            "ipv6",
            "--protocol",
            "https",
        ])
        .expect("exact-case calibration continuation parses");
    }

    #[test]
    fn registration_plan_refusal_returns_to_calibration() {
        let error = actionable_error(
            CliError::usage(
                "target registration confirmation did not match the exact current plan identifier; no target was registered",
            ),
            Some(TargetReference::Selector("Portal 2")),
            None,
            WorkflowVerb::Calibrate,
            None,
        );
        assert!(error
            .message()
            .contains("Next command:  fragcap calibrate \"Portal 2\""));
        assert!(!error.message().contains("fragcap deep-capture"));
        Cli::try_parse_from(["fragcap", "calibrate", "Portal 2"])
            .expect("registration retry parses");
    }
}
