# Quickstart: Filter Management

## What S13 lets you do

Once the attribution map holds endpoints belonging to profiled processes, fragcap
narrows the kernel capture filter to just those endpoints and keeps it current as
connections open and close, so a capture on a busy machine stops paying for
unrelated background traffic. The narrowed filter is only ever an optimization:
userspace attribution still decides what is captured, and any traffic a briefly
stale filter excludes is counted as a filter gap and surfaced.

## Tier-1: compile a filter from an endpoint set (no driver, no game)

```rust
use fragcap_core::filter::FilterProgram;
use fragcap_core::flow::{Endpoint, Proto};

let endpoints = [
    Endpoint::new("198.51.100.5:443".parse().unwrap(), Proto::Tcp),
    Endpoint::new("[2001:db8::1]:5055".parse().unwrap(), Proto::Udp),
];
let program = FilterProgram::narrowed(&endpoints);
// Admits exactly those two endpoints, across IPv4 and IPv6, deterministically.
assert!(!program.is_empty());
```

## Tier-1: drive the maintenance policy with a synthetic clock

```rust
use std::time::{Duration, Instant};
use fragcap_core::filter::{FilterConfig, FilterManager};

let mut mgr = FilterManager::new(1, FilterConfig::PRODUCTION);
let t0 = /* a base Instant */;
let wanted = /* one endpoint */;

// Debounce: nothing installs until the set has been stable for two seconds.
assert!(mgr.poll(&wanted, t0).is_empty());
let installs = mgr.poll(&wanted, t0 + Duration::from_secs(2));
assert_eq!(installs.len(), 1);           // narrowed onto handle 0

// A new endpoint while narrowed, past debounce and the rate limit, records a gap.
```

## Tier-1: end to end through the pipeline with a recording source double

```rust
// A PacketSource double records the sequence of FilterPrograms it is asked to
// install. A scripted attributor returns a fixed active-endpoint set. With a
// zero-debounce FilterConfig injected via Pipeline::set_filter_config, run the
// pipeline and assert the recorded programs go bootstrap -> narrowed, that every
// packet is still attributed regardless of the filter, and that CaptureStats
// reports filter_gaps separately from the drop counters with conservation intact.
```

No capture driver, no elevation, and no game are required for any of the above.
Installing a program on a live npcap handle is tier 2 and out of scope for this
slice; the application path itself is S09 and already exercised.
