// SPDX-License-Identifier: Apache-2.0
#![cfg(all(feature = "targets", windows))]

//! The real Windows machine-wide anti-cheat probe (slice S068, issue #170). A
//! tier-2 test: it calls the live Win32 registry API on the machine running it,
//! so it is exercised only on Windows (the fixture probe drives every other
//! test). It asserts the adapter runs without panicking and returns a shape the
//! rendering contract expects; it is inconclusive on findings content on a
//! runner with no anti-cheat service registered, the same posture
//! `windows_volumes.rs` takes for the volume inventory adapter.

use fragcap::targets::MachineAntiCheatProbe;
use fragcap::WindowsMachineAntiCheatProbe;

#[test]
fn the_real_probe_runs_without_panicking() {
    let probe = WindowsMachineAntiCheatProbe::new();
    let findings = probe.detect();

    // Every finding names a product and evidence; neither is ever empty, whether
    // or not this runner has an anti-cheat service registered.
    for f in &findings {
        assert!(!f.product.is_empty());
        assert!(!f.evidence.is_empty());
    }
}
