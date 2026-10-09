# Quickstart: S169 controlled verification

Run in the repository with a hidden redirected foreground launcher on Windows. No real game, driver installation or real trust mutation is required.

```text
cargo test -p fragcap-core --lib --locked
cargo test -p fragcap --test loopback_correlation --locked
cargo test -p fragcap --lib --locked
cargo test -p fragcap-cli --lib --locked
cargo xtask ci
```

Expected: explicit-interface Deep Capture retains required loopback; address-less recognized loopback frames keep canonical keys; listener filters survive churn; real endpoint identities in controlled frames identify the unique bound owner in either port order. Platform, unrelated, ambiguous, withheld and unavailable outcomes cannot authorize client facts. These prove controlled implementation, not live Npcap acquisition or title compatibility.
