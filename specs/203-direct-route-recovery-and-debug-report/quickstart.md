# Quickstart: Validate S203

## Prerequisites

Use the pinned Rust toolchain and the repository's existing signed WinDivert and Npcap prerequisites. Run controlled Windows route probes only on an authorized host with a disposable cold client and owned non-loopback peer. Use native default storage for any installed-product claim; controlled fixture tests may use their own test directories.

## Driver-free regression

Run the focused fragcap and fragcap-cli tests covering selected route state, Npcap original present and absent, TCP and UDP positive and negative facts, route queue and crash-prefix behavior, report source immutability, and scope help. The old source behavior must fail the #593 and #594 regression assertions before repair.

## Controlled route and package matrix

Exercise signed-driver selected TCP over IPv4 and IPv6 to a non-loopback owned peer, asserting SYN/ACK acknowledgement and exact client-to-origin and origin-to-client bytes. Run active unrelated sentinels and failure/cancellation paths with diagnostics enabled and disabled. Repeat through certified MSI and ZIP entry points where applicable; inspect route.jsonl, manifest, compatibility, calibration, and cleanup. If an installed default-path claim is made, record exact canonical native paths and artifacts as the constitution requires.

## Report and help

Export a debug report from complete and crash-prefix native bundles with the new bundle command. Confirm source hashes are unchanged, exact endpoint omissions are declared, normal and --json summaries agree, and no missing fact appears as complete. Check installed deep-capture --help and active docs for auto and explicit game scope.

## Full gate

Run cargo fmt --all -- --check, cargo clippy --all-targets --all-features -- -D warnings, cargo test --all --locked, cargo xtask ci, and the dedicated controlled Windows and package tiers used by the pull request. Run spec-kit analyze before implementation and again after material requirements changes. Hosted CI and reviews must be green on the exact pushed head before owner handoff.
