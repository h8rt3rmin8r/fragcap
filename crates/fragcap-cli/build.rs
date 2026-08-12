// SPDX-License-Identifier: Apache-2.0

//! Delay-load npcap's `wpcap.dll` so a binary built with the `live` feature
//! still starts on a machine where npcap is not installed.
//!
//! Linking the live capture backend makes `wpcap.dll` a load-time import, and a
//! binary with an unresolved load-time import exits with STATUS_DLL_NOT_FOUND
//! before `main` on a machine where npcap is absent (AGENTS.md records this from
//! slice S09). That would make the single most important recovery path,
//! `fragcap doctor`, unable to run and tell the operator to install npcap, and
//! npcap is deliberately never bundled. Delay-loading defers the DLL load to the
//! first live-capture call, so `doctor` and every offline command start and run
//! with no npcap present, while live capture still resolves once it is installed.
//!
//! Scoped to the binary target on the windows-msvc host with the `live` feature
//! enabled; every other build emits no linker argument and is unaffected.

fn main() {
    let live = std::env::var_os("CARGO_FEATURE_LIVE").is_some();
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let target_env = std::env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();

    if live && target_os == "windows" && target_env == "msvc" {
        // `/DELAYLOAD` needs the delay-load helper, which lives in delayimp.lib.
        println!("cargo:rustc-link-arg-bins=/DELAYLOAD:wpcap.dll");
        println!("cargo:rustc-link-arg-bins=delayimp.lib");
    }

    println!("cargo:rerun-if-changed=build.rs");
}
