# Quickstart: Native Protocol Conformance

## Portable validation

```text
cargo xtask conformance
```

This validates the closed matrix, runs or checks the portable harness evidence, verifies integrated artifact reconciliation, checks deterministic drift and sanitization, and rejects every skipped or missing required row.

## External analyzer validation

```text
cargo xtask conformance --analyzer
```

This mode requires `tshark` on `PATH`. Missing TShark is an error. The command opens the committed pcapng with the committed TLS key log, checks nonzero packets and declared protocol fields, and records the exact analyzer version.

## Update evidence

The analyzer pcapng is derived byte for byte from the generated synthetic
loopback golden. Update the source fixture through its documented generator,
copy the reviewed golden to the conformance directory, and update the normalized
matrix and report only when executable evidence changes. `cargo xtask
conformance` rejects drift, unresolved test identifiers, and prohibited
material. Read the complete diff before committing.

## Full verification

```text
cargo xtask ci
cargo xtask msrv
```

The pull request must also pass Windows portable execution and the dedicated TShark CI job before issue #305 can close.
