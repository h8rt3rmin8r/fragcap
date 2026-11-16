// SPDX-License-Identifier: Apache-2.0

//! Current first-run Deep Capture guidance and pasteable recovery commands.

use std::path::{Path, PathBuf};

use clap::ValueEnum;

use crate::{
    cli::{DeepCaptureArgs, DeepCaptureProxyFamilyArg},
    exit::CliError,
};

pub(crate) const ROOT_LONG_HELP: &str = r#"First Deep Capture session:
  1. Check the environment:    fragcap doctor
  2. Find the installed game:  fragcap targets discover
  3. Inspect or register it:   fragcap targets show "<installed game>"
                                fragcap targets add --steam 620
  4. Capture the game:         fragcap deep-capture "<installed game>" --launch

One authorized capture measures compatibility while it runs. The default
routes and retains only the exact final game. Launcher traffic keeps its
ordinary route. A full launch-chain proxy route is an explicit advanced choice."#;

pub(crate) const DEEP_CAPTURE_LONG_HELP: &str = r#"First session:
  fragcap doctor
  fragcap targets discover
  fragcap targets show "<installed game>"
  fragcap deep-capture "<installed game>" --launch

Steam example:
  fragcap targets add --steam 620
  fragcap deep-capture "Portal 2" --launch

Direct or publisher example:
  fragcap targets scan "C:\Games\My Game"
  fragcap deep-capture "My Game" --launch

Explicit advanced launcher inspection:
  fragcap deep-capture "My Game" --launch --inspect-scope launch-chain --launcher-proxy socks-only

Explicit output recipient:
  fragcap deep-capture 1 --launch --bundle .\bundle --output-recipient <SID>

The default selects the exact final game and gives the managed launcher no
Fragcap proxy variables. The signed direct route requires Administrator and
the packaged component. Advanced launch-chain inspection changes the
launcher environment and must be authorized as a distinct plan.
An already-running Steam process can receive one title dispatch after exact
query-only identity verification. Fragcap leaves that process and its game
running; existing routed connections may close when capture ends. A reviewed
plan may name one direct client alternate if a cold Steam root cannot start
before title dispatch. `targets show` reports bounded chronological attempts.

Use `fragcap doctor --fix` for an interrupted prior session. A captured
protocol fact states what this attempt observed; no prior measurement is
needed. HAR, key logs, and client identities can contain sensitive material.
The default bundle belongs to the exact ordinary desktop-session user."#;

pub(crate) const DOCTOR_LONG_HELP: &str = r#"Doctor checks machine readiness for Capture and Deep Capture. A ready
environment does not register a game or prove its protocol inspectability.

Next commands:
  fragcap targets discover
  fragcap deep-capture "<installed game>" --launch

Use `fragcap doctor --fix` for the exact remediations Doctor reports,
including recovery of interrupted Deep Capture sessions. Read-only Doctor
never deletes history."#;

pub(crate) const TARGETS_LONG_HELP: &str = r#"Deep Capture target path:
  fragcap targets discover
  fragcap targets add --steam 620
  fragcap targets show "<installed game>"
  fragcap deep-capture "<installed game>" --launch

Registration identifies the installed target. Capture records its observed
route and protocol compatibility without a separate measurement step."#;

pub(crate) const TARGETS_DISCOVER_LONG_HELP: &str = r#"Discovery may create or update the local store for volume eligibility.
It does not register candidate targets or claim compatibility. Results keep
distinct technology findings under their target; --summary prints counts.

Next commands:
  fragcap targets add --steam 620
  fragcap targets show "<installed game>"
  fragcap deep-capture "<installed game>" --launch"#;

pub(crate) const TARGETS_ADD_LONG_HELP: &str = r#"Registering creates or updates one stored target.

Steam example:
  fragcap targets add --steam 620

Next commands:
  fragcap targets show "Portal 2"
  fragcap deep-capture "Portal 2" --launch"#;

pub(crate) const TARGETS_SHOW_LONG_HELP: &str = r#"Target detail reports stored launch authority, exact historical
compatibility facts, and bounded chronological capture outcomes. Prior facts
and advisory drift never authorize a new capture.

Next command:
  fragcap deep-capture "<installed game>" --launch"#;

pub(crate) const BUNDLE_COLLECT_LONG_HELP: &str = r#"A preview changes nothing. Applying its exact reviewed proposal deletes
eligible bundle contents while preserving the container. --include-retained
expands deletion to retained or custom evidence selected by --bundle.
--purge-empty separately selects eligible exact empty containers."#;

pub(crate) const BUNDLE_CLEANUP_LONG_HELP: &str = r#"This operation removes only sensitive artifacts declared by a completed
bundle. Review the bundle before passing --yes. If cleanup fails, retain
manifest.json, cleanup.jsonl, resources.jsonl, and the session-owner record
so `fragcap doctor --fix` can retry exact recovery.

Confirmed cleanup:
  fragcap bundle cleanup <bundle> --yes"#;

pub(crate) const BUNDLE_EXPORT_LONG_HELP: &str = r#"Export creates a separate share copy and never modifies the source bundle.
The transformation manifest lists every omission. The source may contain
plaintext, credentials, client identities, or a TLS key log.

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
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TargetReference<'a> {
    Selector(&'a str),
    Id(i64),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorkflowVerb {
    DeepCapture,
}

/// Values needed to reproduce an exact ordinary capture command after refusal.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct CalibrationCommandContext {
    catalog_db: Option<PathBuf>,
    local_db: Option<PathBuf>,
    proxy_family: Option<String>,
}

impl CalibrationCommandContext {
    pub(crate) fn for_deep_capture(
        args: &DeepCaptureArgs,
        effective_local_db: Option<PathBuf>,
    ) -> Self {
        Self {
            catalog_db: args.catalog_db.clone(),
            local_db: effective_local_db.or_else(|| args.local_db.clone()),
            proxy_family: Some(value_name(&args.proxy_family)),
        }
    }

    fn append_to(&self, command: &mut String) {
        append_path_option(command, "--catalog-db", self.catalog_db.as_deref());
        append_path_option(command, "--local-db", self.local_db.as_deref());
        if self.proxy_family.as_deref()
            != Some(value_name(&DeepCaptureProxyFamilyArg::Ipv4).as_str())
        {
            append_value_option(command, "--proxy-family", self.proxy_family.as_deref());
        }
    }
}

fn value_name<T: ValueEnum>(value: &T) -> String {
    value
        .to_possible_value()
        .expect("CLI value has a name")
        .get_name()
        .to_string()
}

fn append_path_option(command: &mut String, flag: &str, path: Option<&Path>) {
    if let Some(path) = path.and_then(Path::to_str) {
        command.push(' ');
        command.push_str(flag);
        command.push(' ');
        command.push_str(&quote_powershell_argument(path));
    }
}

fn append_value_option(command: &mut String, flag: &str, value: Option<&str>) {
    if let Some(value) = value {
        command.push(' ');
        command.push_str(flag);
        command.push(' ');
        command.push_str(value);
    }
}

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
    context: Option<&CalibrationCommandContext>,
) -> String {
    match category {
        FirstRunRefusal::Environment => "fragcap doctor".into(),
        FirstRunRefusal::Target => "fragcap targets discover".into(),
        FirstRunRefusal::Recovery => "fragcap doctor --fix".into(),
        FirstRunRefusal::Launch
        | FirstRunRefusal::Compatibility
        | FirstRunRefusal::Process
        | FirstRunRefusal::Bundle
        | FirstRunRefusal::Authorization => {
            let mut command = target.map_or_else(
                || "fragcap deep-capture \"<installed game>\" --launch".to_string(),
                deep_capture_command,
            );
            if let Some(context) = context {
                context.append_to(&mut command);
            }
            if category == FirstRunRefusal::Bundle {
                command.push_str(" --bundle \"<new empty directory>\"");
            }
            command
        }
    }
}

pub(crate) fn classify_pre_session_refusal(message: &str) -> Option<FirstRunRefusal> {
    let message = message.to_ascii_lowercase();
    if message.contains("recovery") || message.contains("prior deep capture") {
        Some(FirstRunRefusal::Recovery)
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
    _verb: WorkflowVerb,
    context: Option<&CalibrationCommandContext>,
) -> CliError {
    if error.message().contains("Next command:") || error.message().contains("Next commands:") {
        return error;
    }
    if error.message().contains("selector is ambiguous") {
        let commands = error
            .message()
            .lines()
            .filter_map(|line| line.split_whitespace().nth(1)?.parse::<i64>().ok())
            .map(|id| deep_capture_command(TargetReference::Id(id)))
            .collect::<Vec<_>>();
        if !commands.is_empty() {
            return with_next_command(error, commands.join("\n  "));
        }
    }
    let Some(category) = classify_pre_session_refusal(error.message()).or(fallback) else {
        return error;
    };
    with_next_command(error, next_command(category, target, context))
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
    use super::*;
    use crate::cli::Cli;
    use clap::Parser;

    #[test]
    fn current_guidance_has_one_capture_path_and_parses() {
        for help in [
            ROOT_LONG_HELP,
            DEEP_CAPTURE_LONG_HELP,
            DOCTOR_LONG_HELP,
            TARGETS_LONG_HELP,
            TARGETS_DISCOVER_LONG_HELP,
            TARGETS_ADD_LONG_HELP,
            TARGETS_SHOW_LONG_HELP,
        ] {
            assert!(!help.contains("fragcap calibrate"));
        }
        assert!(Cli::try_parse_from(["fragcap", "deep-capture", "My Game", "--launch"]).is_ok());
        assert!(Cli::try_parse_from(["fragcap", "calibrate", "My Game"]).is_err());
    }

    #[test]
    fn retry_preserves_exact_store_and_no_calibration_command() {
        let context = CalibrationCommandContext {
            local_db: Some(PathBuf::from("C:\\data\\local.db")),
            proxy_family: Some("ipv6".into()),
            ..Default::default()
        };
        let command = next_command(
            FirstRunRefusal::Compatibility,
            Some(TargetReference::Selector("My Game")),
            Some(&context),
        );
        assert!(command.starts_with("fragcap deep-capture"));
        assert!(command.contains("--local-db \"C:\\data\\local.db\""));
        assert!(command.contains("--proxy-family ipv6"));
        assert!(!command.contains("calibrate"));
    }
}
