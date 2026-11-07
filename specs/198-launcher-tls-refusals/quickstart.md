# Quickstart: Launcher TLS Refusal Diagnosis

## Local controlled verification

Run the focused classifier and native proxy tests without a game, Npcap, administrator rights, or machine trust changes:

```text
cargo test -p fragcap-proxy tls_refusal_tests --lib
cargo test -p fragcap-proxy --test https_proxy
cargo test -p fragcap-proxy --test http1_proxy
cargo test -p fragcap --test loopback_correlation
```

Run the repository CI parity gate after the focused red/green tests:

```text
cargo xtask ci
```

The controlled fixture uses loopback clients and local origins, exact session capability authentication, per-test generated CA material, and the existing bounded artifact collectors. It does not inspect or launch a private title.

## Evidence readback

For each controlled connection, compare the native diagnostic's accepted connection ID, tunnel and TLS stage chronology, terminal code, alert, transport category, and completed exchange count with application correlation under the same ID. A successful launcher/updater pair proves only its two controlled routed paths. A failed client-facing handshake must have no full-inspection application fact or blind-relay transition. A generic EOF remains a transport ending with unknown cause.

## Hosted handoff

The official PR closes #574 only after local checks, exact-head hosted CI, and every received external review comment are resolved. Owner final review and merge remain separate. Direct final-client sockets are #572, and real-title compatibility is optional post-release evidence rather than a source acceptance gate.
