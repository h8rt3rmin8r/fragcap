// SPDX-License-Identifier: Apache-2.0

//! Anti-cheat signals in Steam's own launch metadata (slice S068, issue #170).
//!
//! `appinfo.vdf`'s launch entries already carry `arguments`, `description`, and
//! `executable` fields this crate parses for other reasons; some of them name
//! anti-cheat products directly, independent of anything on disk. This is a
//! second, zero-new-I/O evidence source alongside the directory scan
//! `fragcap-profile`'s [`fragcap_profile::signature::SignatureSet`] already runs.
//!
//! **Narrow on purpose.** Every rule here matches a specific, unambiguous token: an
//! enable flag unique to a protected launch, the canonical EAC launcher shim's
//! exact filename, or an exact (not partial) description value. Issue #170's own
//! measured data includes a direct counter-example to a broader match: a Halo: MCC
//! launch entry whose `arguments` includes `-no-eac` and whose `description` reads
//! "Halo: MCC Anti-Cheat Disabled (Mods and Limited Services)". A substring match on
//! the word "anti-cheat" in `description` would report this explicitly-disabled
//! variant as evidence *for* anti-cheat, the exact false-positive class the issue
//! devotes a section to warning against for `EOSSDK-Win64-Shipping.dll` (principle
//! P-9). [`classify_launch_entries`] never does that.

use fragcap_profile::signature::{DetectionFinding, SignatureCategory};
use fragcap_profile::FidelityTier;

use crate::appinfo::SteamLaunchEntry;

const PRODUCT_EAC: &str = "Easy Anti-Cheat";

/// The two command-line flags that appear only on a genuinely protected Easy
/// Anti-Cheat launch, measured in issue #170's own `appinfo.vdf` cache.
const EAC_ARGUMENT_MARKERS: &[&str] = &["-anticheat_settings=", "-force_enable_eac_module"];

/// The canonical Easy Anti-Cheat launcher shim's exact executable name.
const EAC_LAUNCHER_EXECUTABLE: &str = "start_protected_game.exe";

/// The exact (not partial) `description` values measured on a genuinely protected
/// launch entry, already in [`normalize_description`]'s output form (hyphens
/// collapsed to spaces): `eac-release` is a build-configuration label; `easy anti
/// cheat` is the launch-entry description Steam itself shows.
const EAC_DESCRIPTION_MARKERS: &[&str] = &["eac release", "easy anti cheat"];

/// Classify a title's already-parsed Steam launch entries for anti-cheat signals,
/// returning zero or more findings. Pure: no I/O, never panics on any input,
/// including an empty slice or entries whose optional fields are all `None`.
///
/// A single entry may trigger more than one rule; the caller merges duplicates
/// across entries and rules via
/// [`fragcap_profile::signature::merge_finding`], so this function returns every
/// match rather than deduplicating within itself.
pub fn classify_launch_entries(entries: &[SteamLaunchEntry]) -> Vec<DetectionFinding> {
    let mut findings = Vec::new();
    for entry in entries {
        if entry
            .executable
            .eq_ignore_ascii_case(EAC_LAUNCHER_EXECUTABLE)
        {
            findings.push(finding(format!(
                "appinfo launch executable: {}",
                entry.executable
            )));
        }
        if let Some(arguments) = &entry.arguments {
            if let Some(marker) = EAC_ARGUMENT_MARKERS
                .iter()
                .find(|m| arguments.contains(**m))
            {
                findings.push(finding(format!("appinfo launch argument: {marker}")));
            }
        }
        if let Some(description) = &entry.description {
            let normalized = normalize_description(description);
            if EAC_DESCRIPTION_MARKERS.contains(&normalized.as_str()) {
                findings.push(finding(format!(
                    "appinfo launch description: {description}"
                )));
            }
        }
    }
    findings
}

fn finding(evidence: String) -> DetectionFinding {
    DetectionFinding {
        category: SignatureCategory::AntiCheat,
        product: PRODUCT_EAC.to_string(),
        evidence,
        // A launch-entry token is not a byte-exact marker the way a PE section name
        // is; heuristic-unverified matches how a filename/directory-shape signature
        // is stamped, not the stronger `Verified` a `binary-marker` earns.
        fidelity: FidelityTier::HeuristicUnverified,
    }
}

/// Trim, lowercase, and collapse the exact set of punctuation this comparison
/// needs to ignore, without touching anything else: "Easy Anti Cheat" and
/// "Easy Anti-Cheat" both normalize to the same string, but "Anti-Cheat Disabled"
/// does not collapse into it, because normalization never drops words.
fn normalize_description(description: &str) -> String {
    description
        .trim()
        .to_lowercase()
        .replace('-', " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(executable: &str) -> SteamLaunchEntry {
        SteamLaunchEntry {
            executable: executable.to_string(),
            ..Default::default()
        }
    }

    #[test]
    fn an_anticheat_settings_argument_is_evidence() {
        let entries = vec![SteamLaunchEntry {
            arguments: Some(
                "-anticheat_settings=SettingsProfile.json --bundle-dir data --release".to_string(),
            ),
            ..entry("Game.exe")
        }];
        let findings = classify_launch_entries(&entries);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].product, PRODUCT_EAC);
        assert!(findings[0].evidence.contains("-anticheat_settings="));
    }

    #[test]
    fn the_launcher_shim_executable_is_evidence_case_insensitively() {
        let entries = vec![entry("START_PROTECTED_GAME.EXE")];
        let findings = classify_launch_entries(&entries);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].product, PRODUCT_EAC);
    }

    #[test]
    fn multiple_matching_fields_on_one_entry_are_all_returned_for_the_caller_to_merge() {
        let entries = vec![SteamLaunchEntry {
            arguments: Some(
                "-force_enable_eac_module -force_enable_eos_sdk \
                 -anticheat_settings=Settings_Release_PROD.json"
                    .to_string(),
            ),
            description: Some("eac-release".to_string()),
            ..entry("Game.exe")
        }];
        let findings = classify_launch_entries(&entries);
        // Both the arguments rule and the description rule fire on this single
        // entry; classify_launch_entries returns both, uncollapsed.
        assert_eq!(findings.len(), 2, "{findings:?}");
        assert!(findings.iter().all(|f| f.product == PRODUCT_EAC));
    }

    #[test]
    fn the_measured_mcc_disabled_variant_yields_no_finding() {
        // Issue #170's own measured counter-example: -no-eac and a description
        // that contains the words "Anti-Cheat" but explicitly means the opposite.
        let entries = vec![SteamLaunchEntry {
            arguments: Some("-no-eac".to_string()),
            description: Some(
                "Halo: MCC Anti-Cheat Disabled (Mods and Limited Services)".to_string(),
            ),
            ..entry(r"mcc\binaries\win64\mcc-win64-shipping.exe")
        }];
        assert!(
            classify_launch_entries(&entries).is_empty(),
            "an explicitly-disabled launch variant must never report anti-cheat"
        );
    }

    #[test]
    fn empty_and_all_none_entries_yield_no_finding_and_no_panic() {
        assert!(classify_launch_entries(&[]).is_empty());
        assert!(classify_launch_entries(&[entry("Game.exe")]).is_empty());
    }
}
