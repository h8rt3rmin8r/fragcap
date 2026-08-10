# Quickstart: Stage Matching and Session Lifecycle

## What S12 lets you do

Given a validated profile and a stream of process events, decide which process is
which role, and drive a capture session that arms before the target, keeps only
gameplay, and stops cleanly.

## Tier-1 example (no driver, no elevation, no game)

Stage matching over a scripted tree:

```rust
use fragcap::profile::matching::{bind_stages, stage_for};
// build a ProcessTree from a scripted event stream, then:
bind_stages(&profile, &mut tree);
// every node that matches a stage under causal ordering now carries a StageId
```

A capture session over scripted events and packets:

```rust
use fragcap::session::{CaptureSession, SessionConfig, PacketDisposition, StopReason};

let mut s = CaptureSession::new(profile, SessionConfig {
    acquisition_timeout: Some(Duration::from_secs(60)),
    duration: None, packet_bound: None, byte_bound: None,
});
s.attach(t0);                              // Arming -> Watching
assert_eq!(s.on_packet(100), PacketDisposition::Discarded); // counted
s.on_process_event(launcher_started);      // launcher matches -> Capturing
s.on_process_event(client_started);        // client binds too
assert_eq!(s.on_packet(100), PacketDisposition::Retained);
s.on_interrupt();                          // -> Draining
s.finalize();                              // -> Complete
assert_eq!(s.stop_reason(), Some(StopReason::Interrupt));
assert_eq!(s.stats().watching_discarded, 1);
assert_eq!(s.stats().retained, 1);
```

## Running the tests

```bash
cargo test -p fragcap-profile --test matching
cargo test -p fragcap --test session
cargo xtask ci
```

`cargo xtask ci` runs format, clippy, the whole test suite, the conventions
lint, the dependency-direction check (which confirms no `fragcap-attr` to
`fragcap-profile` edge appeared), and the license check.
