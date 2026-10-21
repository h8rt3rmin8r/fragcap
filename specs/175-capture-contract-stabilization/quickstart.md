# Validation Guide: Capture Contract Stabilization

## Prerequisites

Use the workspace Rust toolchain and committed replay fixtures. Controlled source tests need no Npcap, elevation, target process, proxy, or real-game traffic. Do not commit capture artifacts from a real session.

## Focused checks

1. Run the Capture CLI integration tests. The epoch-stamped replay with `--duration 45s` must finish for its terminal-stage cause; a `100ms` replay retains the first seven fixture packets and stops for duration. A controlled live-driver tick simulation covers busy, sparse, silent, and pending-terminal waits without spending real wall time.
2. Replay `fixtures/udp-gameplay.pcap` through `--direction in` and `out`, pcapng and JSON Lines sinks, and verify zero opposite-direction records and reconciled omission counters.
3. Replay the fixture's known payload filler through ordinary `--out`, explicit pcapng `payload=false`, and ring output. Confirm marker absence in metadata-only artifacts, marker presence in full output, truthful EPB lengths, and Deep Capture's inheritance of the same packet artifact setting. The shared factory and resolver cover rotation, stream, and FIFO choices. On the current Windows host, installed TShark independently reads the controlled metadata-only output as 24 frames with a 42-byte captured and 74-byte original first frame.
4. Exercise empty, comma-only, interior-empty, case-mismatched, and valid custom roles; invalid values must exit 2 before creating output.
5. Exercise `,,` in sink destinations with and without modifiers, spaces plus commas, and malformed single-comma or option syntax.
6. Compare Capture help snapshots and the controlled ring and five-packet examples against [the CLI contract](contracts/capture.md).

## Repository gate

Run `cargo xtask ci` in the foreground and read the terminal result. Run additional focused Windows or feature-gated checks where the changed paths require them. Report controlled verification separately from any later installed-game compatibility observation.
