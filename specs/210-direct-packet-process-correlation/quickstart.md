# Quickstart: S210 Controlled Qualification

## Prerequisites

- Windows build environment with the repository's signed direct-route component and Npcap for the packaged route matrix.
- Rust toolchain and repository dependencies already required by `cargo xtask ci`.
- A controlled loopback TCP/UDP client and origin; no private game is required.

## Validation

1. Run focused core, attribution, facade, and direct-route hosted tests through the standard Cargo commands. Confirm original-present, original-absent, history-overflow, and conflicting-owner cases report different facts.
2. Run the controlled packaged Windows WinDivert/Npcap route matrix with its exact test fixture. Read the route journal, packet capture, application correlation, process trace, terminal result, and manifest as one bundle. Confirm the consumed-original case has positive route transfer and unavailable original packet evidence.
3. Run post-exit reset and ACK/FIN continuity plus reuse, event-loss, empty-image, and out-of-bound negative cases. Confirm true exit time and explicit tail basis.
4. Run `cargo xtask ci` in the foreground and inspect its exact-root cleanup report. Record any packaged or physical gate that the local host cannot execute as unverified rather than as passed.
