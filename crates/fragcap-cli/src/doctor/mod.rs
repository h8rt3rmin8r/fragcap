// SPDX-License-Identifier: Apache-2.0

//! The `doctor` diagnostic of specification section 26.3, as a pure classifier.
//!
//! The whole command is a function from an [`Inputs`] of raw environment facts
//! to a [`Report`] of classified checks and a single exit code. Modelling it
//! this way is what makes the section 26.3 matrix, npcap present or absent, each
//! non-default option present or absent, elevated or not, interfaces up or down,
//! testable with hand-built inputs on any target, without the environment it
//! describes. The thin [`probe`] gathers real inputs on Windows and is not unit
//! tested; everything that decides or renders is here and in [`checks`].

pub mod action;
pub mod checks;
pub mod fix;
pub mod probe;
pub mod progress;
pub mod residue;

use fragcap::write_json_string;

use crate::display::{display_width, pad_display};
use crate::doctor::action::Action;
use crate::exit::Exit;

/// Whether the tool is running on native Windows or a subsystem.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Subsystem {
    /// Native Windows.
    Native,
    /// The Windows Subsystem for Linux, where capture is constrained.
    Wsl,
}

/// Whether the session is elevated.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Privilege {
    /// Running elevated.
    Elevated,
    /// Not elevated.
    NotElevated,
}

/// What is known about an installed npcap driver.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NpcapInfo {
    /// The version string the driver reports.
    pub version: String,
    /// Whether loopback capture is available, as three-valued driver truth:
    /// `Some(true)` present, `Some(false)` determined absent, `None` not
    /// determined (for example on a build without the live capture backend). It
    /// comes from enumerating the loopback adapter, never from a proxy file.
    pub loopback_supported: Option<bool>,
    /// Whether WinPcap API compatibility mode is installed (a non-default
    /// option).
    pub winpcap_api_mode: bool,
}

/// One network interface as the machine describes it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IfaceInfo {
    /// The interface name.
    pub name: String,
    /// A representative address, when it has one.
    pub addr: Option<String>,
    /// Whether the interface is up.
    pub up: bool,
    /// Whether it is judged virtual.
    pub is_virtual: bool,
}

/// Deep Capture proxy backend availability.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProxyBackendInfo {
    /// Backend name, currently `fragcap-native`.
    pub name: String,
    /// Version string, when the backend reported one.
    pub version: Option<String>,
}

/// Read-only readiness of one exact Deep Capture loopback family.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LoopbackReadiness {
    /// An ephemeral exact loopback bind succeeded.
    Ready,
    /// The operating system refused or could not create the bind.
    Unavailable(String),
    /// No implemented probe could determine readiness.
    Undetermined,
}

/// What doctor knows about fragcap-owned Deep Capture CA trust.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DeepCaptureCa {
    /// No fragcap-owned Deep Capture CA trust was found.
    Absent,
    /// Trust exists in the expected current-user store.
    CurrentUser { thumbprint: String },
    /// Trust exists in a broader or unexpected store.
    WrongStore { store: String, thumbprint: String },
    /// The manifest and trust store disagree.
    Mismatched {
        expected: String,
        actual: String,
        store: Option<String>,
    },
    /// The probe could not determine trust state.
    Unknown(String),
}

/// Raw Deep Capture environment and residue facts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeepCaptureInputs {
    /// The session-bundle root doctor scanned.
    pub session_dir: Option<std::path::PathBuf>,
    /// Whether the session-bundle root exists.
    pub session_dir_present: bool,
    /// Native Deep Capture backend availability.
    pub proxy_backend: Option<ProxyBackendInfo>,
    /// Probe error for the proxy backend, when detection failed after finding it.
    pub proxy_backend_error: Option<String>,
    /// Exact IPv4 loopback listener readiness.
    pub ipv4_loopback: LoopbackReadiness,
    /// Exact IPv6 loopback listener readiness.
    pub ipv6_loopback: LoopbackReadiness,
    /// Whether analyzer key-log configuration is visible to this process.
    pub analyzer_keylog_configured: bool,
    /// The fragcap-owned CA trust state.
    pub ca: DeepCaptureCa,
    /// The bounded native resource and session-owner inventory.
    pub native_residue: residue::NativeResidueInventory,
}

/// The raw environment facts `doctor` classifies. Entirely constructible in a
/// test.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Inputs {
    /// The fragcap version that produced this report. Carried as a field (not
    /// read in the classifier) so a test fixture supplies a fixed value and the
    /// goldens do not churn on a release bump.
    pub fragcap_version: String,
    /// The absolute path of the running binary, when it can be determined.
    pub binary_path: Option<std::path::PathBuf>,
    /// The default catalog store path, reported regardless of existence.
    pub catalog_db_path: Option<std::path::PathBuf>,
    /// Whether the catalog store exists on disk (created on first run).
    pub catalog_db_present: bool,
    /// The default local store path, reported regardless of existence.
    pub local_db_path: Option<std::path::PathBuf>,
    /// Whether the local store exists on disk (created on first run).
    pub local_db_present: bool,
    /// The operating system description.
    pub os: String,
    /// Native or a subsystem.
    pub subsystem: Subsystem,
    /// Elevated or not.
    pub privilege: Privilege,
    /// The npcap driver, when one is installed.
    pub npcap: Option<NpcapInfo>,
    /// Whether the process-event tracing session could open, or `None` when the
    /// tracing capability is not built into this binary.
    pub etw_available: Option<bool>,
    /// Whether the live capture backend is compiled into this binary. `None`
    /// means it is not: the binary cannot capture at all, which is a blocking
    /// problem rather than a downstream "no interfaces" symptom.
    pub live_available: Option<bool>,
    /// Whether the socket-table attribution backend is compiled into this
    /// binary. `None` means it is not: attribution is degraded (ETW may still
    /// cover it), which is a non-blocking concern.
    pub socket_table_available: Option<bool>,
    /// The interfaces found.
    pub interfaces: Vec<IfaceInfo>,
    /// The reason interface enumeration failed, when it was attempted and could
    /// not run. `None` covers both a clean enumeration (see `interfaces`) and a
    /// build where enumeration was never attempted; the classifier tells those
    /// apart by `live_available`. A failed probe is reported distinctly so it is
    /// never presented as a successfully observed empty machine (P-9).
    pub interface_error: Option<String>,
    /// Whether the analyzer extcap integration is installed for the current user
    /// (a fragcap binary is present in the per-user extcap directory).
    pub extcap_installed: bool,
    /// The analyzer's per-user extcap directory, when the platform location can
    /// be determined. Reported by the integration check so an operator knows
    /// where to copy the binary.
    pub extcap_dir: Option<std::path::PathBuf>,
    /// Whether the integration is installed machine-wide (a fragcap binary is
    /// present in Wireshark's system extcap directory, as the MSI's machine-wide
    /// option installs it). A second user sees a machine-wide registration even
    /// though it is not in their own profile.
    pub extcap_system_installed: bool,
    /// The machine-wide (system) extcap directory, when it can be determined.
    pub extcap_system_dir: Option<std::path::PathBuf>,
    /// How many target entries are registered in the local store, when it could be
    /// read. `None` means the count could not be determined (for example the store
    /// could not be opened); it is never presented as zero, so a probe failure is
    /// not reported as an observed-empty store (P-9). `Some(0)` is a real empty
    /// store and carries the "run discovery" action.
    pub target_entry_count: Option<usize>,
    /// Deep Capture readiness and cleanup facts.
    pub deep_capture: DeepCaptureInputs,
}

/// The classification of one check.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    /// Ready.
    Ok,
    /// A non-blocking concern.
    Warn,
    /// Not applicable or not built in.
    Skip,
    /// A blocking problem that must be fixed before capture is possible.
    Fail,
}

/// Which capture mode owns a diagnostic check.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModeScope {
    /// An ordinary Capture prerequisite, also required by Deep Capture.
    Capture,
    /// A native Deep Capture-only prerequisite.
    DeepCapture,
}

impl ModeScope {
    fn as_str(self) -> &'static str {
        match self {
            Self::Capture => "capture",
            Self::DeepCapture => "deep_capture",
        }
    }
}

impl Status {
    /// The word this status renders as.
    pub fn as_str(self) -> &'static str {
        match self {
            Status::Ok => "ok",
            Status::Warn => "warn",
            Status::Skip => "skip",
            Status::Fail => "fail",
        }
    }
}

/// One readiness check.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Check {
    /// The section it belongs to.
    pub section: &'static str,
    /// The check name.
    pub name: String,
    /// A one-line detail.
    pub detail: String,
    /// The classification.
    pub status: Status,
    /// The mode boundary used to derive independent readiness verdicts.
    pub scope: ModeScope,
    /// The remediation, mandatory when the status is `Fail`.
    pub remediation: Option<String>,
    /// The structured, machine-facing counterpart of `remediation`: the action the
    /// `--fix` layer may perform for this check. `None` for informational checks and
    /// for checks whose remedy has no automatable form. When present it is
    /// constructed together with `remediation` (see [`Check::warn_action`] and
    /// [`Check::fail_action`]) so the printed remediation and the offered action
    /// cannot drift.
    pub action: Option<Action>,
    /// Human-only wording when the machine-facing name and detail should remain
    /// stable for automation.
    #[doc(hidden)]
    pub human: Option<HumanPresentation>,
    /// Exact non-secret native residue facts for structured output.
    #[doc(hidden)]
    pub native_resource: Option<NativeResourceContext>,
}

/// Human-only presentation for a check whose machine identity stays unchanged.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HumanPresentation {
    /// Short stable label.
    pub name: String,
    /// Plain-language diagnosis.
    pub detail: String,
    /// Plain-language remediation, when the machine check has one.
    pub remediation: Option<String>,
}

/// Exact non-secret facts attached to a native Deep Capture resource finding.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeResourceContext {
    pub session_id: String,
    pub resource_id: String,
    pub kind: String,
    pub state: String,
    pub health: String,
    pub ownership_authority: String,
    /// Whether this check offers Doctor's shared cleanup action.
    pub recovery_eligible: bool,
}

impl Check {
    fn scope(section: &'static str) -> ModeScope {
        if section == "Deep Capture" {
            ModeScope::DeepCapture
        } else {
            ModeScope::Capture
        }
    }

    /// An `Ok` check.
    pub fn ok(section: &'static str, name: impl Into<String>, detail: impl Into<String>) -> Check {
        Check {
            section,
            name: name.into(),
            detail: detail.into(),
            status: Status::Ok,
            scope: Self::scope(section),
            remediation: None,
            action: None,
            human: None,
            native_resource: None,
        }
    }

    /// A `Warn` check.
    pub fn warn(
        section: &'static str,
        name: impl Into<String>,
        detail: impl Into<String>,
    ) -> Check {
        Check {
            section,
            name: name.into(),
            detail: detail.into(),
            status: Status::Warn,
            scope: Self::scope(section),
            remediation: None,
            action: None,
            human: None,
            native_resource: None,
        }
    }

    /// A `Skip` check.
    pub fn skip(
        section: &'static str,
        name: impl Into<String>,
        detail: impl Into<String>,
    ) -> Check {
        Check {
            section,
            name: name.into(),
            detail: detail.into(),
            status: Status::Skip,
            scope: Self::scope(section),
            remediation: None,
            action: None,
            human: None,
            native_resource: None,
        }
    }

    /// A `Fail` check, which must carry a remediation.
    pub fn fail(
        section: &'static str,
        name: impl Into<String>,
        detail: impl Into<String>,
        remediation: impl Into<String>,
    ) -> Check {
        Check {
            section,
            name: name.into(),
            detail: detail.into(),
            status: Status::Fail,
            scope: Self::scope(section),
            remediation: Some(remediation.into()),
            action: None,
            human: None,
            native_resource: None,
        }
    }

    /// A `Warn` check that carries both a human remediation and the structured
    /// action `--fix` can perform for it. The two are set together here so they
    /// cannot drift (FR-004).
    pub fn warn_action(
        section: &'static str,
        name: impl Into<String>,
        detail: impl Into<String>,
        remediation: impl Into<String>,
        action: Action,
    ) -> Check {
        Check {
            section,
            name: name.into(),
            detail: detail.into(),
            status: Status::Warn,
            scope: Self::scope(section),
            remediation: Some(remediation.into()),
            action: Some(action),
            human: None,
            native_resource: None,
        }
    }

    /// A `Fail` check that carries both a human remediation and the structured
    /// action `--fix` can perform for it, set together so they cannot drift
    /// (FR-004).
    pub fn fail_action(
        section: &'static str,
        name: impl Into<String>,
        detail: impl Into<String>,
        remediation: impl Into<String>,
        action: Action,
    ) -> Check {
        Check {
            section,
            name: name.into(),
            detail: detail.into(),
            status: Status::Fail,
            scope: Self::scope(section),
            remediation: Some(remediation.into()),
            action: Some(action),
            human: None,
            native_resource: None,
        }
    }

    pub(crate) fn with_native_presentation(
        mut self,
        human: HumanPresentation,
        native_resource: NativeResourceContext,
    ) -> Self {
        self.human = Some(human);
        self.native_resource = Some(native_resource);
        self
    }
}

/// The ordered set of checks and the single exit code they yield.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Report {
    /// The checks, in section order.
    pub checks: Vec<Check>,
}

impl Report {
    /// The exit code: 1 if any check failed, else 0.
    pub fn exit(&self) -> Exit {
        if self.checks.iter().any(|c| c.status == Status::Fail) {
            Exit::FAILURE
        } else {
            Exit::SUCCESS
        }
    }

    /// Whether capture is possible (no failing check).
    pub fn ready(&self) -> bool {
        self.exit() == Exit::SUCCESS
    }

    /// Whether ordinary Capture has no blocking check.
    pub fn capture_ready(&self) -> bool {
        !self
            .checks
            .iter()
            .any(|check| check.scope == ModeScope::Capture && check.status == Status::Fail)
    }

    /// Whether Deep Capture and every Capture prerequisite have no blocking check.
    pub fn deep_capture_ready(&self) -> bool {
        !self.checks.iter().any(|check| check.status == Status::Fail)
    }

    /// Render the human report as aligned columns grouped by section, ending
    /// with the readiness verdict. Plain text with no color; this is the
    /// golden-tested form.
    pub fn render_human(&self) -> String {
        self.render_human_with(false)
    }

    /// Render the human report, optionally colorizing status words and bolding
    /// section headings with ANSI. `color` is the caller's choice from whether
    /// the destination is an interactive terminal; the plain form
    /// (`color == false`) is byte-identical to [`Report::render_human`], so the
    /// golden path is never colorized.
    pub fn render_human_with(&self, color: bool) -> String {
        self.render_human_with_width(color, DEFAULT_HUMAN_WIDTH)
    }

    /// Render the human report at a caller-selected terminal width.
    #[doc(hidden)]
    pub fn render_human_with_width(&self, color: bool, width: usize) -> String {
        let width = width.clamp(MIN_HUMAN_WIDTH, DEFAULT_HUMAN_WIDTH);
        let aligned = width >= ALIGNED_LAYOUT_MIN_WIDTH;
        let mut out = String::new();
        let mut section = "";
        for check in &self.checks {
            if check.section != section {
                if !section.is_empty() {
                    out.push('\n');
                }
                if color {
                    out.push_str(ANSI_BOLD);
                    out.push_str(check.section);
                    out.push_str(ANSI_RESET);
                } else {
                    out.push_str(check.section);
                }
                out.push('\n');
                section = check.section;
            }
            let name = check
                .human
                .as_ref()
                .map_or(check.name.as_str(), |human| human.name.as_str());
            let detail = check
                .human
                .as_ref()
                .map_or(check.detail.as_str(), |human| human.detail.as_str());
            let remediation = check.human.as_ref().map_or_else(
                || check.remediation.as_deref(),
                |human| human.remediation.as_deref(),
            );
            if aligned && display_width(name) <= NAME_WIDTH {
                out.push_str("  ");
                out.push_str(&pad_display(name, NAME_WIDTH));
                out.push(' ');
                out.push_str(&status_field(check.status, color));
                out.push(' ');
                out.push_str(&wrap_hanging(detail, DETAIL_INDENT, width));
                out.push('\n');
            } else {
                out.push_str("  ");
                out.push_str(name);
                out.push(' ');
                out.push_str(&status_field(check.status, color));
                out.push('\n');
                out.push_str("    ");
                out.push_str(&wrap_hanging(detail, COMPACT_INDENT, width));
                out.push('\n');
            }
            if let Some(remediation) = remediation {
                out.push_str("    remediation: ");
                out.push_str(&wrap_hanging(remediation, REMEDIATION_INDENT, width));
                out.push('\n');
            }
        }
        out.push('\n');
        out.push_str("Capture: ");
        out.push_str(if self.capture_ready() {
            "ready\n"
        } else {
            "not ready\n"
        });
        out.push_str("Deep Capture: ");
        out.push_str(if self.deep_capture_ready() {
            "ready\n"
        } else {
            "not ready\n"
        });
        out
    }

    /// Render the report as one JSON record per check, newline-delimited.
    pub fn render_json(&self) -> String {
        let mut out = String::new();
        for check in &self.checks {
            let mut line = String::from("{\"section\":");
            write_json_string(check.section, &mut line);
            line.push_str(",\"name\":");
            write_json_string(&check.name, &mut line);
            line.push_str(",\"detail\":");
            write_json_string(&check.detail, &mut line);
            line.push_str(",\"status\":");
            write_json_string(check.status.as_str(), &mut line);
            line.push_str(",\"scope\":");
            write_json_string(check.scope.as_str(), &mut line);
            if let Some(remediation) = &check.remediation {
                line.push_str(",\"remediation\":");
                write_json_string(remediation, &mut line);
            }
            if let Some(native) = &check.native_resource {
                line.push_str(",\"native_resource\":{\"session_id\":");
                write_json_string(&native.session_id, &mut line);
                line.push_str(",\"resource_id\":");
                write_json_string(&native.resource_id, &mut line);
                line.push_str(",\"kind\":");
                write_json_string(&native.kind, &mut line);
                line.push_str(",\"state\":");
                write_json_string(&native.state, &mut line);
                line.push_str(",\"health\":");
                write_json_string(&native.health, &mut line);
                line.push_str(",\"ownership_authority\":");
                write_json_string(&native.ownership_authority, &mut line);
                line.push_str(",\"recovery_eligible\":");
                line.push_str(if native.recovery_eligible {
                    "true"
                } else {
                    "false"
                });
                line.push('}');
            }
            line.push_str("}\n");
            out.push_str(&line);
        }
        self.push_json_verdict(&mut out, "capture", self.capture_ready(), |check| {
            check.scope == ModeScope::Capture
        });
        self.push_json_verdict(&mut out, "deep_capture", self.deep_capture_ready(), |_| {
            true
        });
        out
    }

    fn push_json_verdict(
        &self,
        out: &mut String,
        mode: &str,
        ready: bool,
        applies: impl Fn(&Check) -> bool,
    ) {
        let mut line = String::from("{\"type\":\"readiness\",\"mode\":");
        write_json_string(mode, &mut line);
        line.push_str(",\"ready\":");
        line.push_str(if ready { "true" } else { "false" });
        line.push_str(",\"blocking_checks\":[");
        let mut first = true;
        for check in self
            .checks
            .iter()
            .filter(|check| applies(check) && check.status == Status::Fail)
        {
            if !first {
                line.push(',');
            }
            first = false;
            write_json_string(&format!("{}/{}", check.section, check.name), &mut line);
        }
        line.push_str("]}\n");
        out.push_str(&line);
    }
}

/// The column a wrapped detail continuation hangs under: "  " + name(22) + " " +
/// status(5) + " ".
const DETAIL_INDENT: usize = 31;
const COMPACT_INDENT: usize = 4;
const NAME_WIDTH: usize = 22;
/// The column a wrapped remediation continuation hangs under: "    remediation: ".
const REMEDIATION_INDENT: usize = 17;
const MIN_HUMAN_WIDTH: usize = 40;
const ALIGNED_LAYOUT_MIN_WIDTH: usize = 60;
const DEFAULT_HUMAN_WIDTH: usize = 80;

/// Reset all ANSI styling.
const ANSI_RESET: &str = crate::color::RESET;
/// Bold, for section headings when color is on.
const ANSI_BOLD: &str = "\x1b[1m";

/// The five-column status field, optionally colored. When color is on only the
/// word is wrapped in an escape; the padding stays plain so the columns keep
/// their visible width.
fn status_field(status: Status, color: bool) -> String {
    let word = status.as_str();
    if !color {
        return format!("{word:<5}");
    }
    let code = match status {
        Status::Ok => "\x1b[32m",
        Status::Warn => crate::color::WARN,
        Status::Skip => "\x1b[2m",
        Status::Fail => "\x1b[1;31m",
    };
    let pad = " ".repeat(5usize.saturating_sub(word.len()));
    format!("{code}{word}{ANSI_RESET}{pad}")
}

/// Wrap `text` on word boundaries so no line exceeds `width` columns, hanging
/// continuations under `indent` spaces. A single word longer than the available
/// width is left whole rather than split, so a path or URL stays intact.
fn wrap_hanging(text: &str, indent: usize, width: usize) -> String {
    let avail = width.saturating_sub(indent).max(1);
    let mut lines: Vec<String> = Vec::new();
    let mut cur = String::new();
    for word in text.split_whitespace() {
        if cur.is_empty() {
            cur.push_str(word);
        } else if display_width(&cur) + 1 + display_width(word) <= avail {
            cur.push(' ');
            cur.push_str(word);
        } else {
            lines.push(std::mem::take(&mut cur));
            cur.push_str(word);
        }
    }
    if !cur.is_empty() {
        lines.push(cur);
    }
    let indent_str = " ".repeat(indent);
    lines.join(&format!("\n{indent_str}"))
}

#[cfg(test)]
mod presentation_tests {
    use super::{Check, HumanPresentation, NativeResourceContext, Report};
    use crate::display::display_width;

    fn report() -> Report {
        Report {
            checks: vec![Check::fail(
                "Deep Capture",
                "native resource machine-session/machine-resource",
                "machine detail remains stable",
                "machine remediation remains stable",
            )
            .with_native_presentation(
                HumanPresentation {
                    name: "native residue".to_string(),
                    detail: "An earlier session left cleanup incomplete for 界🎮 evidence. No active owner was proven, so Deep Capture is blocked. Session a-very-long-session-identity; resource trust-record."
                        .to_string(),
                    remediation: Some(
                        "Run `fragcap doctor --fix` to review and confirm cleanup of all eligible inactive Deep Capture records."
                            .to_string(),
                    ),
                },
                NativeResourceContext {
                    session_id: "machine-\"session\\界".to_string(),
                    resource_id: "machine-resource\nline".to_string(),
                    kind: "trust".to_string(),
                    state: "applied".to_string(),
                    health: "stale".to_string(),
                    ownership_authority: "resource-journal".to_string(),
                    recovery_eligible: true,
                },
            )],
        }
    }

    #[test]
    fn aligned_and_compact_layouts_fit_without_truncating_identity() {
        for width in [80, 40] {
            let text = report().render_human_with_width(false, width);
            assert!(text.contains("native residue"));
            assert!(text.contains("a-very-long-session-identity"));
            assert!(text.contains("trust-record"));
            for line in text.lines() {
                assert!(
                    display_width(line) <= width,
                    "line exceeds {width} cells: {line:?}"
                );
            }
        }
    }

    #[test]
    fn color_changes_no_visible_layout() {
        fn strip_ansi(value: &str) -> String {
            let mut plain = String::new();
            let mut chars = value.chars();
            while let Some(c) = chars.next() {
                if c == '\u{1b}' {
                    for escaped in chars.by_ref() {
                        if escaped == 'm' {
                            break;
                        }
                    }
                } else {
                    plain.push(c);
                }
            }
            plain
        }

        for width in [80, 40] {
            let plain = report().render_human_with_width(false, width);
            let colored = report().render_human_with_width(true, width);
            assert_eq!(strip_ansi(&colored), plain);
        }
    }

    #[test]
    fn structured_context_is_additive_and_human_wording_is_not_serialized() {
        let json = report().render_json();
        let record: serde_json::Value = serde_json::from_str(json.lines().next().unwrap()).unwrap();
        assert_eq!(
            record["name"],
            "native resource machine-session/machine-resource"
        );
        assert_eq!(record["detail"], "machine detail remains stable");
        assert_eq!(
            record["native_resource"]["session_id"],
            "machine-\"session\\界"
        );
        assert_eq!(
            record["native_resource"]["resource_id"],
            "machine-resource\nline"
        );
        assert_eq!(record["native_resource"]["kind"], "trust");
        assert_eq!(record["native_resource"]["state"], "applied");
        assert_eq!(record["native_resource"]["health"], "stale");
        assert_eq!(
            record["native_resource"]["ownership_authority"],
            "resource-journal"
        );
        assert!(record["native_resource"]["recovery_eligible"]
            .as_bool()
            .unwrap());
        assert!(!json.contains("native residue"));

        let ordinary = Report {
            checks: vec![Check::ok("Identity", "version", "0.9.0")],
        };
        assert!(!ordinary.render_json().contains("native_resource"));
    }

    #[test]
    fn indivisible_tokens_are_preserved_at_the_supported_boundary() {
        let token = "x".repeat(60);
        let report = Report {
            checks: vec![Check::ok("Identity", "token", &token)],
        };
        let text = report.render_human_with_width(false, 40);
        assert!(text.contains(&token));
    }
}
