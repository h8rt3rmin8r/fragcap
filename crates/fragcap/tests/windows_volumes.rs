// SPDX-License-Identifier: Apache-2.0
#![cfg(all(feature = "targets", windows))]

//! The real Windows volume inventory adapter (S052 spec US4/T018). A tier-2 test:
//! it calls the live Win32 volume APIs on the machine running it, so it is
//! exercised only on Windows (the fixture inventory drives every other test). It
//! asserts the adapter returns this machine's fixed volumes with the shape the
//! eligibility table and the known-roots walk expect.

use fragcap::targets::{DriveType, VolumeInventory};
use fragcap::WindowsVolumeInventory;

#[test]
fn enumerates_this_machines_fixed_volumes() {
    let inventory = WindowsVolumeInventory::new();
    let volumes = inventory.fixed_volumes();

    // Any Windows host has at least the system volume.
    assert!(
        !volumes.is_empty(),
        "expected at least one fixed volume on a Windows host"
    );
    // The system drive letter is not always C: (unusual installs and CI images
    // differ), so assert against %SystemDrive% rather than a hard-coded letter.
    let system_drive = std::env::var("SystemDrive").unwrap_or_else(|_| "C:".to_string());
    let system_drive = system_drive.trim_end_matches('\\');
    assert!(
        volumes
            .iter()
            .any(|v| v.mount_point.eq_ignore_ascii_case(system_drive)),
        "expected the system volume {system_drive} among the fixed volumes"
    );

    for v in &volumes {
        assert_eq!(
            v.drive_type,
            DriveType::Fixed,
            "only fixed volumes are returned"
        );
        assert!(
            !v.identity.is_empty(),
            "every volume carries a stable identity"
        );
        assert!(
            v.mount_point.ends_with(':'),
            "the mount point is a bare drive letter like C:, got {:?}",
            v.mount_point
        );
    }

    // Identities are stable keys: distinct volumes must not collide on identity.
    let mut ids: Vec<&str> = volumes.iter().map(|v| v.identity.as_str()).collect();
    ids.sort_unstable();
    let before = ids.len();
    ids.dedup();
    assert_eq!(before, ids.len(), "volume identities are unique");
}
