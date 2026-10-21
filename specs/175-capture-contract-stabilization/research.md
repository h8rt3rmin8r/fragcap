# Research: Capture Contract Stabilization

## R1. Deadline time domain

**Decision**: Keep `CaptureSession::on_tick` on the existing zero-based arm-relative domain. In live Capture, both packet and idle notifications supply `Instant::elapsed` mapped onto that domain. In replay, packet timestamps retain ordering authority, while a virtual elapsed clock subtracts the first acquired replay packet timestamp recorded before the drop-oldest queue and saturates backward jumps at zero. Process-event timestamps never enter the deadline check.

**Rationale**: Live Npcap and ETW timestamps are absolute, while the session attaches at zero. `SessionGate` must continue comparing absolute packet and process timestamps to classify buffered pre-acquisition and post-terminal packets. Separating the deadline input at the driver keeps this window correct. Replay needs a deterministic virtual interval so duration tests do not depend on machine speed.

**Alternatives considered**: Converting all session timestamps to wall time would keep clock-domain mixing and expose wall-clock jumps. Replacing packet evidence timestamps with elapsed values would corrupt pcapng, process ordering, and correlation. Using real elapsed time for replay would make fast fixture playback unable to exercise duration bounds.

## R2. Direction and role scope

**Decision**: Validate role spelling against `Profile::stage(role)` during effective-configuration construction, after argument parsing and before components or sinks are opened. Keep exact case-sensitive roles. Pass the requested direction into `SessionConfig`; decide it in `SessionGate::admit` after the existing window and role-scope checks and before volume bounds. Count known opposite direction separately from unknown direction.

**Rationale**: The gate is the one shared admission point before pcapng, JSON Lines, ring, and streaming sinks. The existing parser already produces typed inbound, outbound, or unknown direction. Distinct counters prevent an unknown packet from being represented as a known opposite-direction packet or an attribution failure.

**Alternatives considered**: Per-sink filtering duplicates decisions and counters. A warning or rejection for `in` and `out` would preserve the known defect instead of delivering the documented Capture capability. Case folding would reinterpret custom profile vocabulary and could hide a typo.

## R3. Metadata-only pcapng

**Decision**: Add a pcapng payload mode to the sink factory and writer, keeping the current constructor as full-payload compatibility. A shared core parser helper computes a conservative header-only prefix for recognized Ethernet, raw IP, and BSD loopback frames with IPv4/IPv6 and TCP/UDP. It uses the existing link and IP parsing decisions, validates the full transport header, and returns zero bytes when the boundary is uncertain. The writer uses retained prefix length as EPB captured length and the unchanged `orig_len` as original length.

**Rationale**: The sink factory is common to file, stream, rotation, and ring, so mode selection cannot be bypassed by a transport. `fragcap-core` already owns frame parsing and remains platform neutral; duplicating byte arithmetic in the sink would create a second inconsistent parser. Zero captured bytes are the only truthful bounded representation for malformed or unsupported frames whose header boundary cannot be established. JSON Lines already omits `data` in metadata-only mode.

**Alternatives considered**: Zeroing payload bytes and leaving captured length unchanged would falsely report observed bytes and could preserve sensitive lengths or malformed content. Dropping whole packets would break packet accounting. Rejecting all pcapng metadata-only output would narrow an expressly supported operator option.

## R4. Sink comma grammar

**Decision**: Use doubled commas inside a destination to mean one literal comma, for example `jsonl:Capture,, QA.jsonl,format=jsonl`. A single comma introduces a `key=value` modifier. Reject a dangling single comma or malformed option with an exact syntax message. Preserve every existing specification without doubled commas.

**Rationale**: Windows paths use backslashes, so a backslash escape would make ordinary paths ambiguous. Doubled comma requires no shell-specific quoting beyond the usual argument quoting and preserves existing modifier boundaries.

**Alternatives considered**: Backslash escapes would conflict with path separators; guessing whether a comma starts a known modifier would make unknown-modifier diagnostics misleading and could silently reinterpret paths.

## R5. Verification and issue closure

**Decision**: Use controlled replay and live-source tests, byte-marker checks, independent pcapng reading where available, the repository's `cargo xtask ci` gate, and issue-specific CLI tests. Real-game calibration after a published build remains separately labeled compatibility evidence.

**Rationale**: The constitution and contributor workflow close implementation issues on repository-controlled gates. The affected active-release observations are valid regression inputs, but private game traffic is not required or appropriate as a committed fixture.

**Alternatives considered**: Waiting on a future real-game run as a standing issue gate conflicts with the project's release-first defect workflow.
