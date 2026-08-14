// SPDX-License-Identifier: Apache-2.0

//! Windows-MSVC build steps: stamp the exe's version resource, and delay-load
//! npcap's `wpcap.dll` when the `live` feature is linked.
//!
//! Version resource: without an embedded VERSIONINFO the shipped `fragcap.exe`
//! reports a FileVersion of `0.0.0.0` (issue #104), even though `fragcap
//! --version` prints the real crate version. This stamps FileVersion and
//! ProductVersion from `CARGO_PKG_VERSION`, the same `[workspace.package]
//! version` source clap's `--version` uses, so the two cannot disagree. It runs
//! on every windows-msvc build, `live` or not.
//!
//! Delay-load: linking the live capture backend makes `wpcap.dll` a load-time
//! import, and a binary with an unresolved load-time import exits with
//! STATUS_DLL_NOT_FOUND before `main` on a machine where npcap is absent (AGENTS.md
//! records this from slice S09). That would make the single most important recovery
//! path, `fragcap doctor`, unable to run and tell the operator to install npcap,
//! and npcap is deliberately never bundled. Delay-loading defers the DLL load to
//! the first live-capture call, so `doctor` and every offline command start and run
//! with no npcap present, while live capture still resolves once it is installed.
//! Scoped to the binary target on the windows-msvc host with the `live` feature;
//! every other build emits no linker argument and is unaffected.

fn main() {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let target_env = std::env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();

    if target_os == "windows" && target_env == "msvc" {
        // Stamp the PE version resource on every windows-msvc build, live or not.
        #[cfg(windows)]
        stamp_version_resource();

        // The npcap delay-load helper is only needed when the live backend is
        // linked; `/DELAYLOAD` needs the helper, which lives in delayimp.lib.
        if std::env::var_os("CARGO_FEATURE_LIVE").is_some() {
            println!("cargo:rustc-link-arg-bins=/DELAYLOAD:wpcap.dll");
            println!("cargo:rustc-link-arg-bins=delayimp.lib");
        }
    }

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=CARGO_PKG_VERSION");
}

/// Embed a VERSIONINFO resource carrying the crate version, so the exe's
/// FileVersion is the real version rather than `0.0.0.0`. The version fields come
/// from `CARGO_PKG_VERSION` (the workspace version); a resource-compiler failure
/// is a warning, not an error, so a dev machine without `rc.exe` still links (it
/// simply ships unstamped).
#[cfg(windows)]
fn stamp_version_resource() {
    // "0.3.0" -> (0, 3, 0). Split on the semver separators; a missing or
    // non-numeric component becomes 0 rather than failing the build.
    let pkg = std::env::var("CARGO_PKG_VERSION").unwrap_or_default();
    let mut parts = pkg
        .split(['.', '-', '+'])
        .map(|component| component.parse::<u16>().unwrap_or(0));
    let major = parts.next().unwrap_or(0);
    let minor = parts.next().unwrap_or(0);
    let patch = parts.next().unwrap_or(0);
    let file_version = format!("{major}.{minor}.{patch}.0");
    // VS_FIXEDFILEINFO packs the four 16-bit fields into a u64: the most
    // significant word is major/minor, the least significant is patch/build.
    let packed = ((major as u64) << 48) | ((minor as u64) << 32) | ((patch as u64) << 16);

    let mut res = winresource::WindowsResource::new();
    res.set_version_info(winresource::VersionInfo::FILEVERSION, packed);
    res.set_version_info(winresource::VersionInfo::PRODUCTVERSION, packed);
    res.set("FileVersion", &file_version);
    res.set("ProductVersion", &pkg);
    res.set("ProductName", "fragcap");
    res.set(
        "FileDescription",
        "Passive, process-attributed network capture for Windows",
    );
    res.set("OriginalFilename", "fragcap.exe");
    res.set("LegalCopyright", "Licensed under Apache-2.0");
    if let Err(e) = res.compile() {
        println!("cargo:warning=fragcap version resource not stamped: {e}");
    }
}
