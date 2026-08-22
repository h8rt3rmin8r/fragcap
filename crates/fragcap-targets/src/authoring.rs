// SPDX-License-Identifier: Apache-2.0

//! Interactive authoring logic: the socket-holder answer and the launch chain it
//! records (slice S055).
//!
//! The `targets add` prompt asks whether the executable the user pointed at is the
//! process that holds the sockets. The answer is a first-class value with three
//! outcomes, and its only job is to decide what launch chain is stored. No outcome
//! records a socket holder the tool did not observe (P-9): `unsure` and `no` both
//! leave the holder unresolved for a capture to fill, and only `yes` records the
//! executable as the client, on the user's own assertion.

use serde_json::{json, Value};

use crate::entry::TargetEntry;

/// The answer to "Is the executable above the process that holds the sockets?".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SocketHolderAnswer {
    /// The executable holds the sockets: record it as the resolved client.
    Yes,
    /// A different, unknown process holds them: record the executable as a
    /// launcher stage with the holder unresolved.
    No,
    /// Unknown: record the executable observed, with no socket-holder claim.
    Unsure,
}

impl SocketHolderAnswer {
    /// Parse an answer from a prompt or a `--socket-holder` flag. Accepts
    /// `y`/`yes`, `n`/`no`, and `unsure`/`u`, case-insensitively. Anything else is
    /// `None` (the caller re-prompts or reports a usage error).
    pub fn parse(token: &str) -> Option<SocketHolderAnswer> {
        match token.trim().to_ascii_lowercase().as_str() {
            "y" | "yes" => Some(SocketHolderAnswer::Yes),
            "n" | "no" => Some(SocketHolderAnswer::No),
            "u" | "unsure" => Some(SocketHolderAnswer::Unsure),
            _ => None,
        }
    }
}

/// The launch chain a socket-holder answer records for an executable. `Yes` is a
/// resolved client array (which reduces to a capturable client, so CAPTURE reads
/// `ready`); `No` and `Unsure` are unresolved markers (an object, which names no
/// client, so CAPTURE reads `needs a target`). None fabricates a holder (P-9).
pub fn launch_entries_for(answer: SocketHolderAnswer, executable: &str) -> Value {
    match answer {
        SocketHolderAnswer::Yes => resolved_client_launch(executable),
        SocketHolderAnswer::No => json!({
            "role": "launcher",
            "executable": executable,
            "socket_holder": "unresolved",
        }),
        SocketHolderAnswer::Unsure => json!({
            "observed_exe": executable,
            "socket_holder": "unresolved",
        }),
    }
}

/// The resolved launch chain naming one client executable: the shape a capture
/// writes back on promotion, and the shape a `yes` answer records.
pub fn resolved_client_launch(executable: &str) -> Value {
    json!([{ "executable": executable, "role": "client" }])
}

/// Whether an entry's launch chain is unresolved, i.e. it carries the
/// `socket_holder: "unresolved"` marker a `no` or `unsure` authoring answer wrote.
/// A capture against such a target promotes it once it observes the real holder.
pub fn launch_is_unresolved(entry: &TargetEntry) -> bool {
    entry
        .launch_entries
        .as_ref()
        .and_then(|v| v.get("socket_holder"))
        .and_then(|v| v.as_str())
        == Some("unresolved")
}

/// The executable an unresolved launch chain observed, if any (slice S059).
///
/// A `no` answer records the executable under `executable` (it is a launcher
/// stage whose socket-holding child is unknown); an `unsure` answer records it
/// under `observed_exe` (the socket holder is unknown, so only the observed
/// process is named). This reads `observed_exe` first, then `executable`, so
/// either unresolved shape yields the process the operator pointed at. A
/// resolved (array) chain or an object with neither key returns `None`: there is
/// no single observed executable to build an observe-mode capture from.
pub fn observed_executable(entry: &TargetEntry) -> Option<&str> {
    let launch = entry.launch_entries.as_ref()?;
    launch
        .get("observed_exe")
        .or_else(|| launch.get("executable"))
        .and_then(|v| v.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entry::{ClassificationSource, TargetClassification};
    use crate::readiness::{capture_readiness, CaptureReadiness};
    use fragcap_profile::FidelityTier;

    fn entry_with_launch(launch: Value) -> TargetEntry {
        TargetEntry {
            id: None,
            stable_id: 1,
            handle: "g".to_string(),
            name: "G".to_string(),
            classification: TargetClassification::Game,
            classification_source: ClassificationSource::User,
            fidelity: FidelityTier::Authored,
            provenance: None,
            anchor: None,
            launch_entries: Some(launch),
            install_root: None,
            evidence: None,
            detection_scan: None,
            folder_name: None,
            executable_hint: None,
        }
    }

    #[test]
    fn parse_accepts_the_three_answers() {
        assert_eq!(
            SocketHolderAnswer::parse("Y"),
            Some(SocketHolderAnswer::Yes)
        );
        assert_eq!(
            SocketHolderAnswer::parse("no"),
            Some(SocketHolderAnswer::No)
        );
        assert_eq!(
            SocketHolderAnswer::parse("Unsure"),
            Some(SocketHolderAnswer::Unsure)
        );
        assert_eq!(SocketHolderAnswer::parse("maybe"), None);
    }

    #[test]
    fn yes_is_a_ready_resolved_client() {
        let e = entry_with_launch(launch_entries_for(SocketHolderAnswer::Yes, "game.exe"));
        assert_eq!(capture_readiness(&e), CaptureReadiness::Ready);
        assert!(!launch_is_unresolved(&e));
    }

    #[test]
    fn no_and_unsure_are_unresolved_and_need_a_target() {
        for answer in [SocketHolderAnswer::No, SocketHolderAnswer::Unsure] {
            let e = entry_with_launch(launch_entries_for(answer, "game.exe"));
            assert_eq!(
                capture_readiness(&e),
                CaptureReadiness::NeedsTarget,
                "{answer:?} needs a target"
            );
            assert!(launch_is_unresolved(&e), "{answer:?} is unresolved");
        }
    }

    #[test]
    fn observed_executable_reads_both_unresolved_shapes() {
        // The `no` shape records the launcher under `executable`.
        let no = entry_with_launch(launch_entries_for(SocketHolderAnswer::No, "launcher.exe"));
        assert_eq!(observed_executable(&no), Some("launcher.exe"));

        // The `unsure` shape records the observed process under `observed_exe`.
        let unsure = entry_with_launch(launch_entries_for(SocketHolderAnswer::Unsure, "game.exe"));
        assert_eq!(observed_executable(&unsure), Some("game.exe"));

        // A resolved (array) chain has no single observed executable to read.
        let yes = entry_with_launch(launch_entries_for(SocketHolderAnswer::Yes, "game.exe"));
        assert_eq!(observed_executable(&yes), None);

        // An object with neither key names nothing to observe from.
        let empty = entry_with_launch(json!({ "socket_holder": "unresolved" }));
        assert_eq!(observed_executable(&empty), None);
    }

    #[test]
    fn no_answer_fabricates_a_socket_holder() {
        // The unresolved markers name no client executable at all.
        for answer in [SocketHolderAnswer::No, SocketHolderAnswer::Unsure] {
            let value = launch_entries_for(answer, "game.exe");
            assert!(
                value.get("socket_holder").and_then(|v| v.as_str()) == Some("unresolved"),
                "{answer:?} leaves the holder unresolved"
            );
            assert!(
                !value.is_array(),
                "{answer:?} is not a resolved client array"
            );
        }
    }
}
