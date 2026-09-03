# Quickstart: Validate Complete IPv6 Parity

## Prerequisites

- Work on `codex/119-ipv6-parity`.
- `.specify/feature.json` points to `specs/119-ipv6-parity` and remains unstaged.
- Use synthetic loopback peers, certificates, routes, and payloads only.

## Focused Validation

```sh
cargo test -p fragcap-proxy --test upstream
cargo test -p fragcap-proxy --test ipv6_parity
cargo test -p fragcap --test deep_capture_session
cargo test -p fragcap-cli --test cli_doctor
```

Expected results:

- IPv4 and IPv6 plans bind exactly their authorized loopback endpoints.
- IPv6 HTTP, HTTPS, SOCKS, TCP, UDP, and QUIC rows pass.
- Scoped literals and mapped addresses either preserve exact identity or receive stable refusal.
- A dual-stack race yields one selected peer and no duplicate application observation.
- Doctor reports IPv4 and IPv6 loopback readiness separately.

## Full Gate

```sh
cargo xtask ci
```

Review the final diff for issue #315 only. Confirm no wildcard or external bind, process access, target key extraction, global proxy mutation, hidden family fallback, unbounded owner, Unicode dash punctuation, BOM, mojibake, or staged `.specify/feature.json`.
