**2026-08-14** Corrected the MSI build in the release workflow
(`.github/workflows/release.yml`) after the re-pointed v0.3.0 run reached the MSI
step for the first time and failed on the `cargo wix` invocation. The step passed
`--ext WixUtilExtension`, which cargo-wix's `wix` subcommand does not accept, and
it omitted `--target-bin-dir`, which is required alongside `--no-build` so the
hand-authored `main.wxs` can resolve `$(var.CargoTargetBinDir)` to the staged
payload. The invocation now passes no extension flags (cargo-wix already links
both `WixUIExtension` and `WixUtilExtension`, so adding either again makes light
fail with duplicate-table and duplicate-symbol collisions) and sets
`--target-bin-dir` to the release output directory.

Two `main.wxs` defects were fixed in the same change (that file is not a pinned
artifact): an explicit `ARPNOMODIFY` property collided with the one
`WixUI_InstallDir` already defines, and the `WixUILicenseRtf` path was relative to
the package directory rather than the repository root where `cargo wix` runs.

The whole MSI build was then validated end to end offline against WiX 3.14: the
installer compiles and links with no errors, and an administrative extract
confirms it carries `fragcap.exe`, `hint.db`, `LICENSE`, and `NOTICE` under a
`fragcap` directory with product name `fragcap`, version `0.3.0`, and manufacturer
`ShruggieTech`. This is a fix-forward: the v0.3.0 tag builds from the workflow as
it was at the tag, so consuming it requires re-pointing the tag.
