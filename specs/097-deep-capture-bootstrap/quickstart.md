# Quickstart: Deep Capture Compatibility Bootstrap

This maintainer flow uses only the controlled local target. It does not require a game account, real CA trust mutation, Npcap, or a remote service.

## Contract And Classifier Tests

```powershell
cargo test -p fragcap-cli cli_deep_capture --quiet
cargo test -p fragcap-cli deep_capture::tests --quiet
```

Verify that invalid flag pairs, unsupported launch cases, absent confirmation, insufficient TLS prerequisites, and declined plans all stop before bundle, proxy, trust, launch, or fact mutation.

## Controlled Reachability

```powershell
cargo test -p fragcap-cli --test cli_deep_capture calibration_reachability -- --nocapture
```

Expected evidence includes a visible plan, no trust event or trust mutation, bounded phase events, current final-client routing, independently self-reported controlled propagation, owner context, a finalized local bundle, and complete cleanup accounting.

## Controlled TLS

```powershell
cargo test -p fragcap-cli --test cli_deep_capture calibration_tls -- --nocapture
```

The test first seeds or produces current same-case reachability, then verifies accepted, pinned, metadata-only, unsupported, proxy-not-reached, interrupted, and failed outcomes through controlled adapters. No real trust store is touched.

## Ordinary Gate Reuse

```powershell
cargo test -p fragcap-cli --test cli_deep_capture ordinary_deep_capture_consumes_calibration_routing -- --nocapture
```

The ordinary command must read the exact rows written by calibration. Stale, conflicting, another-target, and another-launch-case rows remain insufficient.

## Full Verification

```powershell
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all --locked
cargo xtask ci
```

Review the final diff for private evidence, disallowed punctuation, encoding corruption, and unexpected dependencies before committing.
