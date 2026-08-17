// SPDX-License-Identifier: Apache-2.0

//! Volume eligibility: permissive first-run seeding, then an allowlist (S052
//! spec FR-016/FR-016a/FR-017, US4). The store is exercised in memory; every
//! decision's reason is asserted recoverable.

use fragcap_targets::{DriveType, EligibilityReason, Store, Volume};

fn vol(id: &str, mount: &str) -> Volume {
    Volume {
        identity: id.to_string(),
        mount_point: mount.to_string(),
        drive_type: DriveType::Fixed,
    }
}

#[test]
fn first_run_seeds_present_volumes_then_is_an_allowlist() {
    let mut store = Store::open_in_memory().unwrap();
    assert!(store.volume_eligibility_is_empty().unwrap());

    let c = vol("vol-c-guid", "C:");
    let d = vol("vol-d-guid", "D:");
    store
        .seed_volume_eligibility(&[c.clone(), d.clone()])
        .unwrap();

    // Both present volumes are recorded eligible with the seeding reason.
    let eligible = store.eligible_volumes().unwrap();
    assert_eq!(eligible.len(), 2);
    for e in &eligible {
        assert!(e.eligible);
        assert_eq!(e.reason, EligibilityReason::SeededFirstRun);
    }

    // A volume that appears AFTER seeding is unseen (not eligible) until an
    // explicit opt-in: re-seeding is a no-op because the table is non-empty.
    let e = vol("vol-e-guid", "E:");
    store
        .seed_volume_eligibility(std::slice::from_ref(&e))
        .unwrap();
    assert!(
        store.volume_eligibility("vol-e-guid").unwrap().is_none(),
        "a later-appearing volume must not be auto-added (FR-016a)"
    );
    assert_eq!(store.eligible_volumes().unwrap().len(), 2);
}

#[test]
fn a_user_added_volume_becomes_eligible() {
    let mut store = Store::open_in_memory().unwrap();
    store
        .seed_volume_eligibility(&[vol("vol-c-guid", "C:")])
        .unwrap();

    let e = vol("vol-e-guid", "E:");
    store
        .set_volume_eligibility(&e, true, EligibilityReason::UserAdded)
        .unwrap();

    let recorded = store.volume_eligibility("vol-e-guid").unwrap().unwrap();
    assert!(recorded.eligible);
    assert_eq!(recorded.reason, EligibilityReason::UserAdded);
    assert_eq!(recorded.mount_point.as_deref(), Some("E:"));
    assert_eq!(store.eligible_volumes().unwrap().len(), 2);
}

#[test]
fn an_excluded_volume_is_never_eligible_and_re_include_works() {
    let mut store = Store::open_in_memory().unwrap();
    let c = vol("vol-c-guid", "C:");
    let d = vol("vol-d-guid", "D:");
    store
        .seed_volume_eligibility(&[c.clone(), d.clone()])
        .unwrap();

    // Exclude D.
    store
        .set_volume_eligibility(&d, false, EligibilityReason::UserExcluded)
        .unwrap();
    let eligible = store.eligible_volumes().unwrap();
    assert_eq!(eligible.len(), 1);
    assert_eq!(eligible[0].volume_id, "vol-c-guid");
    let excluded = store.volume_eligibility("vol-d-guid").unwrap().unwrap();
    assert!(!excluded.eligible);
    assert_eq!(excluded.reason, EligibilityReason::UserExcluded);

    // Re-include D.
    store
        .set_volume_eligibility(&d, true, EligibilityReason::UserAdded)
        .unwrap();
    assert_eq!(store.eligible_volumes().unwrap().len(), 2);
}
