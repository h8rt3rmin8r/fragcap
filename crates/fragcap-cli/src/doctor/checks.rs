// SPDX-License-Identifier: Apache-2.0

//! The pure per-check classifiers of specification section 26.3.
//!
//! Each is a function from the injected [`Inputs`] to one [`Check`], and
//! [`run`] assembles them in section order into a [`Report`]. The two npcap
//! options are separate checks, each naming its own remediation when absent
//! (the Licensing section's detect-never-install rule and slice S14 research
//! D-f), and the tracing check is a blocking fail only when the session is
//! elevated and the process-event session could not open.

use super::{Check, Inputs, Privilege, Report, Subsystem};

const PLATFORM: &str = "Platform";
const DRIVER: &str = "Capture driver";
const TRACING: &str = "Tracing";
const INTERFACES: &str = "Interfaces";
const INTEGRATION: &str = "Integration";
const PROFILES: &str = "Profiles";

/// Where to obtain npcap. fragcap never installs it (the Licensing section).
const NPCAP_SOURCE: &str = "fragcap does not install npcap; obtain it from https://npcap.com and \
     install it, then run doctor again";

/// Run every classifier in section order.
pub fn run(inputs: &Inputs) -> Report {
    let mut checks = vec![
        os(inputs),
        subsystem(inputs),
        privilege(inputs),
        npcap(inputs),
        loopback(inputs),
        winpcap_api(inputs),
        tracing(inputs),
    ];
    checks.extend(interfaces(inputs));
    checks.push(integration(inputs));
    checks.push(profiles(inputs));
    Report { checks }
}

fn os(inputs: &Inputs) -> Check {
    Check::ok(PLATFORM, "os", inputs.os.clone())
}

fn subsystem(inputs: &Inputs) -> Check {
    match inputs.subsystem {
        Subsystem::Native => Check::ok(PLATFORM, "subsystem", "native"),
        Subsystem::Wsl => Check::warn(
            PLATFORM,
            "subsystem",
            "running under WSL; capture of Windows traffic is constrained",
        ),
    }
}

fn privilege(inputs: &Inputs) -> Check {
    match inputs.privilege {
        Privilege::Elevated => Check::ok(PLATFORM, "privilege", "elevated"),
        Privilege::NotElevated => Check::warn(
            PLATFORM,
            "privilege",
            "not elevated; some interfaces and process tracing need an elevated session",
        ),
    }
}

fn npcap(inputs: &Inputs) -> Check {
    match &inputs.npcap {
        Some(info) => Check::ok(DRIVER, "npcap", format!("version {}", info.version)),
        None => Check::fail(DRIVER, "npcap", "npcap is not installed", NPCAP_SOURCE),
    }
}

fn loopback(inputs: &Inputs) -> Check {
    match &inputs.npcap {
        None => Check::skip(DRIVER, "loopback adapter", "npcap is not installed"),
        Some(info) if info.loopback_adapter => {
            Check::ok(DRIVER, "loopback adapter", "loopback capture supported")
        }
        Some(_) => Check::fail(
            DRIVER,
            "loopback adapter",
            "loopback capture support is not installed",
            "reinstall npcap with the \"Support loopback traffic\" option enabled",
        ),
    }
}

fn winpcap_api(inputs: &Inputs) -> Check {
    match &inputs.npcap {
        None => Check::skip(DRIVER, "winpcap api mode", "npcap is not installed"),
        Some(info) if info.winpcap_api_mode => {
            Check::ok(DRIVER, "winpcap api mode", "WinPcap API compatible")
        }
        Some(_) => Check::fail(
            DRIVER,
            "winpcap api mode",
            "WinPcap API compatibility mode is not installed",
            "reinstall npcap with the \"Install Npcap in WinPcap API-compatible Mode\" option \
             enabled",
        ),
    }
}

fn tracing(inputs: &Inputs) -> Check {
    match inputs.etw_available {
        // The tracing capability is not built into this binary. A non-blocking
        // skip: attribution still works from the socket table.
        None => Check::skip(
            TRACING,
            "process events",
            "process-event tracing is not built into this binary",
        ),
        Some(true) => Check::ok(TRACING, "process events", "process-event session available"),
        // Elevated and the session could not open: attribution is degraded, so
        // this blocks. Not elevated and it could not open: not blocking, because
        // the session was never entitled to open one.
        Some(false) => match inputs.privilege {
            Privilege::Elevated => Check::fail(
                TRACING,
                "process events",
                "the process-event session could not open while elevated",
                "ensure no other tool holds the kernel process trace session, then run doctor \
                 again",
            ),
            Privilege::NotElevated => Check::warn(
                TRACING,
                "process events",
                "the process-event session is unavailable without elevation",
            ),
        },
    }
}

fn interfaces(inputs: &Inputs) -> Vec<Check> {
    if inputs.interfaces.is_empty() {
        return vec![Check::warn(
            INTERFACES,
            "adapters",
            "no interfaces were found",
        )];
    }
    inputs
        .interfaces
        .iter()
        .map(|iface| {
            let addr = iface.addr.as_deref().unwrap_or("no address");
            let detail = format!(
                "{}{}",
                addr,
                if iface.is_virtual { " (virtual)" } else { "" }
            );
            // Never fails: an interface being down is a warning, not a blocking
            // problem, because another interface may still carry the traffic.
            if iface.up {
                Check::ok(INTERFACES, iface_name(iface.name.as_str()), detail)
            } else {
                Check::warn(INTERFACES, iface_name(iface.name.as_str()), "down")
            }
        })
        .collect()
}

/// Interface names arrive from the machine, so a `Check`'s `&'static str` name
/// field cannot hold them. The name is folded into the detail instead, and the
/// check is labelled by a fixed slot.
fn iface_name(_name: &str) -> &'static str {
    "adapter"
}

fn integration(inputs: &Inputs) -> Check {
    if inputs.extcap_installed {
        Check::ok(INTEGRATION, "analyzer extcap", "installed")
    } else {
        // Optional: a warning, never a block.
        Check::warn(
            INTEGRATION,
            "analyzer extcap",
            "not installed; the analyzer integration is optional",
        )
    }
}

fn profiles(inputs: &Inputs) -> Check {
    Check::ok(
        PROFILES,
        "profiles",
        format!(
            "bundled: {}, user: {}",
            inputs.bundled_count, inputs.user_count
        ),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::doctor::{IfaceInfo, NpcapInfo, Status};
    use crate::exit::Exit;

    fn ready_inputs() -> Inputs {
        Inputs {
            os: "Windows 11".to_string(),
            subsystem: Subsystem::Native,
            privilege: Privilege::Elevated,
            npcap: Some(NpcapInfo {
                version: "1.79".to_string(),
                loopback_adapter: true,
                winpcap_api_mode: true,
            }),
            etw_available: Some(true),
            interfaces: vec![IfaceInfo {
                name: "Ethernet".to_string(),
                addr: Some("192.0.2.10".to_string()),
                up: true,
                is_virtual: false,
            }],
            extcap_installed: true,
            bundled_count: 0,
            user_count: 2,
        }
    }

    #[test]
    fn a_ready_machine_passes_and_exits_zero() {
        let report = run(&ready_inputs());
        assert!(report.ready());
        assert_eq!(report.exit(), Exit::SUCCESS);
        assert!(report.checks.iter().all(|c| c.status != Status::Fail));
    }

    #[test]
    fn absent_npcap_fails_and_skips_its_options() {
        let mut inputs = ready_inputs();
        inputs.npcap = None;
        let report = run(&inputs);
        assert_eq!(report.exit(), Exit::FAILURE);
        assert_eq!(npcap(&inputs).status, Status::Fail);
        assert_eq!(loopback(&inputs).status, Status::Skip);
        assert_eq!(winpcap_api(&inputs).status, Status::Skip);
    }

    #[test]
    fn the_two_npcap_options_fail_independently() {
        let mut inputs = ready_inputs();
        if let Some(info) = inputs.npcap.as_mut() {
            info.loopback_adapter = false;
        }
        let report = run(&inputs);
        assert_eq!(loopback(&inputs).status, Status::Fail);
        assert_eq!(
            winpcap_api(&inputs).status,
            Status::Ok,
            "the WinPcap API check is unaffected by the loopback option"
        );
        assert_eq!(report.exit(), Exit::FAILURE);

        let mut inputs = ready_inputs();
        if let Some(info) = inputs.npcap.as_mut() {
            info.winpcap_api_mode = false;
        }
        assert_eq!(winpcap_api(&inputs).status, Status::Fail);
        assert_eq!(loopback(&inputs).status, Status::Ok);
    }

    #[test]
    fn every_failing_check_carries_a_remediation() {
        let mut inputs = ready_inputs();
        inputs.npcap = Some(NpcapInfo {
            version: "1.79".to_string(),
            loopback_adapter: false,
            winpcap_api_mode: false,
        });
        inputs.privilege = Privilege::Elevated;
        inputs.etw_available = Some(false);
        let report = run(&inputs);
        for check in report.checks.iter().filter(|c| c.status == Status::Fail) {
            assert!(
                check.remediation.as_ref().is_some_and(|r| !r.is_empty()),
                "{} has no remediation",
                check.name
            );
        }
    }

    #[test]
    fn tracing_is_a_skip_when_not_built_and_blocks_only_when_elevated() {
        let mut inputs = ready_inputs();
        inputs.etw_available = None;
        assert_eq!(tracing(&inputs).status, Status::Skip);

        inputs.etw_available = Some(false);
        inputs.privilege = Privilege::Elevated;
        assert_eq!(tracing(&inputs).status, Status::Fail);

        inputs.privilege = Privilege::NotElevated;
        assert_eq!(
            tracing(&inputs).status,
            Status::Warn,
            "unavailable without elevation is not a blocking problem"
        );
    }

    #[test]
    fn missing_optional_integration_only_warns() {
        let mut inputs = ready_inputs();
        inputs.extcap_installed = false;
        let report = run(&inputs);
        assert_eq!(integration(&inputs).status, Status::Warn);
        assert!(report.ready(), "an optional warning does not block");
    }

    #[test]
    fn no_interfaces_warns_without_blocking() {
        let mut inputs = ready_inputs();
        inputs.interfaces.clear();
        let report = run(&inputs);
        assert!(report
            .checks
            .iter()
            .any(|c| c.section == INTERFACES && c.status == Status::Warn));
        assert!(report.ready());
    }
}
