# Harness Contract

## Commands

The isolated binary exposes bounded commands for `candidate`, `baseline`, and `compare`. Each accepts an output path rooted in a temporary directory and writes one sanitized evidence document on success or partial completion.

The candidate command starts the native proxy on an operating-system-assigned loopback port, runs the controlled matrix, signals cancellation, waits for bounded drain, records residue, and exits nonzero only when the harness itself cannot produce an authoritative result.

The baseline command applies the same lifecycle to the installed `mitmdump` child. Absence or unsupported baseline behavior is an evidence result unless process creation itself prevents the run from being described.

The compare command joins candidate and baseline results by scenario and proof point. It never upgrades a missing or unsupported row to parity.

## Side-Effect Boundary

- Bind addresses are loopback only.
- Proxy routing is applied to harness-owned clients only.
- CA trust is passed directly to those clients and never installed.
- Key-log environment is scoped to the owned backend process or owned TLS configuration.
- Private material and raw logs live only in the supplied temporary output root.
- Cleanup attempts every listener, child, connection, cache owner, and temporary sensitive path once.

## Exit Classes

`0` means authoritative evidence was produced, including evidence containing failed, unsupported, or not-measured results. A nonzero exit means the harness could not preserve or serialize an authoritative run result. Backend capability failure alone is data, not a harness crash.
