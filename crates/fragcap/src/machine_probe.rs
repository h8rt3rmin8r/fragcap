// SPDX-License-Identifier: Apache-2.0

//! The real Windows machine-wide anti-cheat probe (slice S068, issue #170).
//!
//! Modern Easy Anti-Cheat installs once per machine as a service under
//! `HKLM\SYSTEM\CurrentControlSet\Services\<name>`, outside any game's install
//! tree. This checks the one service name issue #170 measured on a real machine
//! (`EasyAntiCheat_EOS`) for existence, the same registry route (over the
//! service-control-manager alternative) the issue itself prefers, and the same
//! access-rights class `fragcap-steam`'s Steam install-path lookup already uses.
//! The seam and the model this implements live in
//! `fragcap_targets::machine_probe`, so `fragcap-targets` itself carries no
//! platform dependency; the tests drive that seam with
//! [`fragcap_targets::FixtureMachineAntiCheatProbe`] instead, so this adapter is
//! exercised only on a real Windows machine (mirrors
//! [`crate::WindowsVolumeInventory`]). The module declaration itself is
//! `#[cfg(windows)]` (`crates/fragcap/src/lib.rs`), the same posture
//! `fragcap-attr`'s `socket-table` platform module takes, so a non-Windows
//! build never compiles this file and never needs a fallback stub.
//!
//! Only Easy Anti-Cheat is checked. BattlEye and Vanguard's machine-wide service
//! names are named in issue #170 as unverified ("not installed on this machine...
//! should be verified before being relied on"); adding an unverified entry would
//! be exactly the kind of unmeasured claim principle P-9 exists to avoid, even
//! though a wrong name would simply fail closed (no finding, never a false one).

use fragcap_targets::{MachineAntiCheatFinding, MachineAntiCheatProbe};

/// One (product, registered service name) pair this probe checks for.
struct KnownProduct {
    product: &'static str,
    service_name: &'static str,
}

/// Every product this probe checks, all backed by a measurement recorded in issue
/// #170 rather than inferred from a deployment model.
const KNOWN_PRODUCTS: &[KnownProduct] = &[KnownProduct {
    product: "Easy Anti-Cheat",
    service_name: "EasyAntiCheat_EOS",
}];

/// The real Windows machine-wide anti-cheat probe.
pub struct WindowsMachineAntiCheatProbe;

impl WindowsMachineAntiCheatProbe {
    /// Build the Windows machine-wide anti-cheat probe.
    pub fn new() -> Self {
        WindowsMachineAntiCheatProbe
    }
}

impl Default for WindowsMachineAntiCheatProbe {
    fn default() -> Self {
        Self::new()
    }
}

impl MachineAntiCheatProbe for WindowsMachineAntiCheatProbe {
    fn detect(&self) -> Vec<MachineAntiCheatFinding> {
        KNOWN_PRODUCTS
            .iter()
            .filter(|p| service_registered(p.service_name))
            .map(|p| MachineAntiCheatFinding {
                product: p.product.to_string(),
                evidence: format!("service {} registered", p.service_name),
            })
            .collect()
    }
}

/// Whether a service key exists under
/// `HKLM\SYSTEM\CurrentControlSet\Services\<name>`. Existence only, no value
/// read: the fact this probe needs is presence, not any particular field. A
/// failure to open (the key is absent, or access is denied) is `false`, never a
/// panic and never a claim that the service is absent for a reason other than
/// "this probe could not confirm it present" (FR-008: the caller treats an empty
/// result set as "nothing to report," not as a confirmed clean scan).
fn service_registered(name: &str) -> bool {
    use windows_sys::Win32::System::Registry::{
        RegCloseKey, RegOpenKeyExW, HKEY, HKEY_LOCAL_MACHINE, KEY_READ,
    };

    let subkey = format!("SYSTEM\\CurrentControlSet\\Services\\{name}");
    let subkey_w: Vec<u16> = subkey.encode_utf16().chain(std::iter::once(0)).collect();

    // SAFETY: `subkey_w` is a live, null-terminated UTF-16 string; `hkey` is
    // written only on success and closed only when it was opened.
    unsafe {
        let mut hkey: HKEY = 0;
        let opened = RegOpenKeyExW(
            HKEY_LOCAL_MACHINE,
            subkey_w.as_ptr(),
            0,
            KEY_READ,
            &mut hkey,
        ) == 0;
        if opened {
            RegCloseKey(hkey);
        }
        opened
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_service_name_that_cannot_exist_is_never_registered() {
        assert!(!service_registered(
            "fragcap-test-service-name-that-does-not-exist-9c2f"
        ));
    }
}
