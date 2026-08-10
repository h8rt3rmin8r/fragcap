// SPDX-License-Identifier: Apache-2.0

//! Is the capture driver installed, and installed how.
//!
//! **fragcap never installs it.** The constitution's licensing section makes
//! that binding rather than advisory: npcap is not redistributable, so fragcap
//! detects and reports, and the operator installs. Nothing in this module
//! downloads a file, spawns a process, or invokes an installer, and a change
//! that added one would be a licensing violation before it was anything else.
//!
//! What a report is for is telling an operator the specific thing that is
//! wrong. "Capture failed" on a machine missing a non-default installation
//! option is the failure mode that makes people give up on a tool, so the two
//! options fragcap needs are named individually.
//!
//! # What this module cannot determine, and says so
//!
//! Two of the four fields on a [`DriverReport`] are answered `None` here, and
//! neither is an oversight.
//!
//! **The driver's version.** The `pcap` crate exposes no binding for
//! libpcap's version string, so there is no version to read through the surface
//! this crate has. Reporting a guess, or inferring one from behaviour, would
//! put a number in a diagnostic that nothing observed. Slice S14's `doctor`
//! command can query the installed service directly, which is where the
//! capability belongs anyway.
//!
//! **WinPcap API compatibility mode.** It cannot be distinguished from an
//! ordinary npcap installation through libpcap, because both present the same
//! functions. It needs the same service query, and waits for the same slice.
//!
//! `None` is therefore load-bearing rather than lazy: it means "not
//! determined", which is a different statement from "absent", and constitution
//! P-9 does not permit collapsing the two.
//!
//! Presentation belongs to slice S14. This module supplies the facts.

use fragcap_core::interface::{DriverReport, DRIVER_DOWNLOAD_URL};

/// What npcap calls its loopback adapter when the loopback capture option is
/// installed. Matched case-insensitively against the adapter description.
const LOOPBACK_MARKER: &str = "loopback";

/// What is installed, as far as can be determined without opening a handle.
///
/// Returns a report rather than a `Result` because absence is an answer. A
/// machine with no capture driver has been observed correctly; wrapping that in
/// an error would make a caller treat a successful observation as a failure.
pub fn detect_driver() -> DriverReport {
    // Enumeration is the available presence test: it is the first call that
    // reaches the driver, and it needs no handle and no privilege.
    let Ok(devices) = pcap::Device::list() else {
        return DriverReport {
            present: false,
            version: None,
            loopback_supported: None,
            winpcap_compatible: None,
        };
    };

    // The loopback option's effect is that an adapter exists, so enumeration is
    // also how that question gets answered. Here the enumeration succeeded, so
    // `Some` is a real observation rather than an assumption.
    let loopback_supported = Some(devices.iter().any(|d| {
        d.flags.is_loopback()
            || d.desc
                .as_deref()
                .is_some_and(|desc| desc.to_lowercase().contains(LOOPBACK_MARKER))
    }));

    DriverReport {
        present: true,
        // See this module's documentation. Not determinable here, and not
        // guessed.
        version: None,
        loopback_supported,
        winpcap_compatible: None,
    }
}

/// What to tell an operator when the driver is absent.
///
/// A function rather than a constant so that the download location has exactly
/// one source in the codebase, which is what keeps the constitution's
/// requirement that absence be reported with the official location from
/// drifting into an invented one.
pub fn absence_message() -> String {
    format!(
        "no packet capture driver was found. fragcap requires npcap, installed \
         separately, with loopback capture support and WinPcap API compatibility \
         mode selected. Both are non-default options. Obtain it from \
         {DRIVER_DOWNLOAD_URL}. fragcap does not download or install it."
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_absence_message_names_the_driver_the_options_and_where_to_get_it() {
        let m = absence_message();
        assert!(m.contains("npcap"));
        assert!(m.contains("loopback"));
        assert!(m.contains("WinPcap API compatibility"));
        assert!(m.contains(DRIVER_DOWNLOAD_URL));
        assert!(
            m.contains("does not download or install"),
            "the message must say what fragcap will not do, because an operator \
             who expects it to install will wait for something that never happens"
        );
    }

    // FR-045, and the reason the fields are three-valued. The middle value has
    // to be reachable or the type is lying about its own precision.
    #[test]
    fn what_cannot_be_determined_is_reported_as_unknown_rather_than_absent() {
        let report = detect_driver();
        assert_eq!(
            report.winpcap_compatible, None,
            "this module cannot determine compatibility mode and must not claim to"
        );
        assert_eq!(
            report.version, None,
            "the binding exposes no version, and a guessed one is worse than none"
        );
        if !report.present {
            assert_eq!(
                report.loopback_supported, None,
                "with no driver, nothing was observed about its options"
            );
        }
    }
}
