# Quickstart: Socket Table Attributor

**Slice**: S10 | **Date**: 2026-08-09 | **Plan**: [plan.md](plan.md)

How to exercise what this slice adds, on any machine, with no capture driver,
no elevation, and no game.

## Run the checks

```sh
cargo xtask ci
```

The platform backend is behind a feature that is off by default, so this passes
with no Windows, no npcap, and no software development kit. That is SC-010, and
it is the property S09 established and this slice must not cost.

## Resolve a flow against a declared table

```rust
use std::sync::Arc;
use fragcap_attr::{
    AttributorConfig, DeclaredNames, DeclaredTable, SocketTableAttributor,
    SocketTable, SocketTableEntry, TestClock,
};
use fragcap_core::flow::{FlowKey, Proto};
use fragcap_core::packet::Timestamp;
use fragcap_core::traits::FlowAttributor;

let clock = Arc::new(TestClock::at(Timestamp::from_nanos(1_000)));

let table = SocketTable::new(
    Timestamp::from_nanos(1_000),
    vec![SocketTableEntry {
        proto: Proto::Tcp,
        local: "192.0.2.10:51000".parse().unwrap(),
        remote: Some("198.51.100.5:443".parse().unwrap()),
        pid: 4242,
        created: Some(Timestamp::from_nanos(500)),
    }],
);

let mut attributor = SocketTableAttributor::new(
    Box::new(DeclaredTable::once(table)),
    Box::new(DeclaredNames::from([(4242, "eso64.exe")])),
    clock.clone(),
    AttributorConfig::default(),
);
attributor.refresh().expect("the declared table reads");

let key = FlowKey::new(
    Proto::Tcp,
    "192.0.2.10:51000".parse().unwrap(),
    "198.51.100.5:443".parse().unwrap(),
);
let a = attributor.resolve(&key, Timestamp::from_nanos(1_500)).unwrap();
assert_eq!(a.pid, 4242);
assert_eq!(&*a.process, "eso64.exe");
```

The attribution carries `Fidelity::Live`, no role, and no stage. Roles arrive
with S12.

## Watch an endpoint go from live to retained to gone

Advance the clock, hand the source a table without the entry, refresh, and
resolve at three instants:

```rust
clock.set(Timestamp::from_nanos(2_000));
attributor.refresh().expect("the second table reads");

// Inside the thirty second window, measured from when it was last seen.
let retained = attributor.resolve(&key, Timestamp::from_nanos(5_000)).unwrap();
assert_eq!(retained.fidelity, Fidelity::Retained);

// Past it.
assert!(attributor.resolve(&key, Timestamp::from_nanos(40_000_000_000)).is_none());
```

Nothing sleeps. The window is thirty seconds of declared time.

## Drive the cadence

```rust
let schedule = attributor.schedule();

// The interval.
assert!(!schedule.is_due(now, Duration::from_secs(1)));

// The unseen endpoint trigger, and its rate limit.
assert!(schedule.request_triggered(t0, Duration::from_millis(200)));
assert!(!schedule.request_triggered(t0_plus_100ms, Duration::from_millis(200)));
assert!(schedule.request_triggered(t0_plus_300ms, Duration::from_millis(200)));

// A process start ignores the limit.
schedule.request_immediate();
assert!(schedule.take_request());
```

## Read a real socket table

Only on Windows, and only with the feature on:

```sh
cargo test -p fragcap-attr --features socket-table -- --ignored
```

The feature is `socket-table` and not `live`. `live` belongs to
`fragcap-capture` and means "links against the npcap import library". This
backend links against nothing of the sort: the IP Helper API ships with the
operating system, so this builds on a bare Windows machine with no capture
driver and no software development kit.

These are tier 2 by specification section 25.2. They do not run in
`cargo xtask ci` and they are not green anywhere yet; the `platform` workflow
is where they will run when it has a machine.

## What is not here

- **Roles and stages.** Every attribution this slice produces has `role: None`
  and `stage: None`. S12 matches profile stages.
- **A control thread.** Nothing calls `refresh` on a schedule yet. The seam is
  `published()` and `schedule()`, and S13 attaches a thread to them.
- **A command line.** S14.
