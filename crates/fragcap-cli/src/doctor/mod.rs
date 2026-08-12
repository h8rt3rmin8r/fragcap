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

pub mod checks;
pub mod probe;

use fragcap::write_json_string;

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
    /// Whether loopback capture support is installed (a non-default option).
    pub loopback_adapter: bool,
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

/// The raw environment facts `doctor` classifies. Entirely constructible in a
/// test.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Inputs {
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
    /// Whether the analyzer extcap integration is installed (a fragcap binary is
    /// present in the extcap directory).
    pub extcap_installed: bool,
    /// The analyzer's extcap directory, when the platform location can be
    /// determined. Reported by the integration check so an operator knows where
    /// to copy the binary.
    pub extcap_dir: Option<std::path::PathBuf>,
    /// How many bundled profiles ship.
    pub bundled_count: usize,
    /// How many user profiles are available.
    pub user_count: usize,
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
    pub name: &'static str,
    /// A one-line detail.
    pub detail: String,
    /// The classification.
    pub status: Status,
    /// The remediation, mandatory when the status is `Fail`.
    pub remediation: Option<String>,
}

impl Check {
    /// An `Ok` check.
    pub fn ok(section: &'static str, name: &'static str, detail: impl Into<String>) -> Check {
        Check {
            section,
            name,
            detail: detail.into(),
            status: Status::Ok,
            remediation: None,
        }
    }

    /// A `Warn` check.
    pub fn warn(section: &'static str, name: &'static str, detail: impl Into<String>) -> Check {
        Check {
            section,
            name,
            detail: detail.into(),
            status: Status::Warn,
            remediation: None,
        }
    }

    /// A `Skip` check.
    pub fn skip(section: &'static str, name: &'static str, detail: impl Into<String>) -> Check {
        Check {
            section,
            name,
            detail: detail.into(),
            status: Status::Skip,
            remediation: None,
        }
    }

    /// A `Fail` check, which must carry a remediation.
    pub fn fail(
        section: &'static str,
        name: &'static str,
        detail: impl Into<String>,
        remediation: impl Into<String>,
    ) -> Check {
        Check {
            section,
            name,
            detail: detail.into(),
            status: Status::Fail,
            remediation: Some(remediation.into()),
        }
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

    /// Render the human report as aligned columns grouped by section, ending
    /// with the readiness verdict.
    pub fn render_human(&self) -> String {
        let mut out = String::new();
        let mut section = "";
        for check in &self.checks {
            if check.section != section {
                out.push_str(check.section);
                out.push('\n');
                section = check.section;
            }
            out.push_str(&format!(
                "  {:<22} {:<5} {}\n",
                check.name,
                check.status.as_str(),
                check.detail
            ));
            if let Some(remediation) = &check.remediation {
                out.push_str(&format!("    remediation: {remediation}\n"));
            }
        }
        if self.ready() {
            out.push_str("Ready to capture.\n");
        } else {
            let failing = self
                .checks
                .iter()
                .filter(|c| c.status == Status::Fail)
                .count();
            out.push_str(&format!(
                "Not ready: {failing} blocking problem(s); fix the failing checks above.\n"
            ));
        }
        out
    }

    /// Render the report as one JSON record per check, newline-delimited.
    pub fn render_json(&self) -> String {
        let mut out = String::new();
        for check in &self.checks {
            let mut line = String::from("{\"section\":");
            write_json_string(check.section, &mut line);
            line.push_str(",\"name\":");
            write_json_string(check.name, &mut line);
            line.push_str(",\"detail\":");
            write_json_string(&check.detail, &mut line);
            line.push_str(",\"status\":");
            write_json_string(check.status.as_str(), &mut line);
            if let Some(remediation) = &check.remediation {
                line.push_str(",\"remediation\":");
                write_json_string(remediation, &mut line);
            }
            line.push_str("}\n");
            out.push_str(&line);
        }
        out
    }
}
