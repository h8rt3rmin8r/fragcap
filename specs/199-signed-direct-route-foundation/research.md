# Research: Signed Direct Route Foundation

Date: 2026-11-09. Inputs: issues #572, #577, #579, current repository, and official WinDivert 2.2 documentation and v2.2.2 release.

## R1. Governing change

Current P-1, specification section 19.3, `deny.toml`, contributor prose and the feature template prohibit every interception driver. The owner explicitly approved an official signed component for deliberately selected Deep Capture while retaining no target memory access, no game-file edits and continuity of unrelated workstation traffic. This is a planned governance amendment, not an ad hoc implementation exception. Removing an absolute P-1 prohibition materially redefines that principle, so the constitution version advances from 1.8.0 to 2.0.0 before implementation. Passive Capture, target-memory and file boundaries remain.

The anti-cheat rationale currently asserts universal monitoring of every prohibited technique. The actual approved rule concerns product behavior, not a brand-specific compatibility claim. Revise absolute claims without adding automatic brand gates. Application-supported, non-memory TLS key export is research #580; proxy-owned key logging is already shipped.

## R2. Exact upstream component

Official release: [WinDivert v2.2.2](https://github.com/basil00/WinDivert/releases/tag/v2.2.2), selected asset `WinDivert-2.2.2-A.zip` (official release URL `https://github.com/basil00/WinDivert/releases/download/v2.2.2/WinDivert-2.2.2-A.zip`). Measured archive SHA-256: `63cb41763bb4b20f600b6de04e991a9c2be73279e317d4d82f237b150c5f3f15`. Official tag source archive URL: `https://codeload.github.com/basil00/WinDivert/zip/refs/tags/v2.2.2`, measured SHA-256: `65ec79c9e6afa99f648a3f4d1f6db794640b40d0b65bd438770ea503ee14ecb7`.

The selected x64 `WinDivert.dll` is 47,616 bytes, SHA-256 `c1e060ee19444a259b2162f8af0f3fe8c4428a1c6f694dce20de194ac8d7d9a2`, and Authenticode `NotSigned`. The selected x64 `WinDivert64.sys` is 94,144 bytes, SHA-256 `8da085332782708d8767bcace5327a6ec7283c17cfb85e40b03cd2323a90ddc2`, and Authenticode `Valid` on this Windows host. Its signer certificate thumbprint is `043589F75FCE2795E7F2CC3E526D46784D5DDAB3`; signer subject is the vendor-documented Mistiny Technology sponsor. These observations are repeated on the disposable Windows package runner. Both PE files must report AMD64 machine `8664`.

The binary archive includes a `LICENSE` containing the LGPLv3 additional terms and embedded GPLv3/GPLv2 copies. It does not contain the full source. Ship the exact source archive and component license alongside the DLL and SYS in both MSI and ZIP. Keep files replaceable and do not statically link the DLL. Include component identity and source obligation in the package notice and SBOM.

## R3. Loading and lifecycle

The [official documentation](https://reqrypt.org/windivert-doc.html) describes placing the x64 DLL and SYS beside the application, dynamically loading or linking the DLL, and the Administrator requirement for `WinDivertOpen`. The driver is installed on demand at that call. Deleting the files does not prove the service is immediately unloaded; it may remain until reboot. S199 does not make that call. The later route adapter should load the exact side-by-side DLL only after session plan authorization using a reviewed absolute path and should keep ordinary Capture independent of it. No Rust binding crate is added in S199, so the dependency graph requires no new copyleft exception. Reassess any future binding separately against the narrow policy.

Read-only Doctor can verify presence, hash, PE and Authenticode state. It cannot prove that a future elevated `WinDivertOpen` will succeed or that security software will permit the driver. Report that distinction instead of inventing a blocked diagnosis from static files.

## R4. Packaging authority

The current package contract has six shared entries and requires determinate unsigned state for fragcap's own executable and MSI. WinDivert adds distinct signature requirements to the two component files and source/license entries. The existing Windows certification workflow assembles both formats once, compares installed files to portable bytes, tests install/repair/reinstall/upgrade/uninstall and emits a report. Extend this authority rather than create a second packaging path. Keep Fragcap's own unsigned marker and installation-CA lifecycle unchanged.

The package workflow obtains Npcap SDK at build time and never bundles Npcap. WinDivert's binary and source archives are downloaded only from the pinned official URLs and verified before staging. Static contract validation must reject an unpinned or structurally inconsistent component, while Windows certification tests actual PE and signature facts.

## R5. Scope decisions

- Include #577 and #579 in S199 because policy, license and package evidence must agree on the selected component.
- Leave #578 and #581 through #584 for route, bridge and integrated operation. Packaging readiness does not claim direct final-client reachability.
- Treat a package with no driver handle as install-ready only; route readiness and inspection success require later controlled tests.
