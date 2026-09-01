# Quickstart: Native Protocol Conformance

## Portable validation

```text
cargo xtask conformance
```

This validates the closed matrix, runs or checks the portable harness evidence, verifies integrated artifact reconciliation, regenerates both analyzer inputs byte for byte from `analyzer-source-v1.txt`, checks sanitization, and rejects every skipped or missing required row.

## External analyzer validation

```text
cargo xtask conformance --analyzer
```

This mode requires `tshark` on `PATH`. Missing TShark is an error. The command opens the committed pcapng with the committed TLS key log, requires decryption of the synthetic `GET /conformance` request for `s110.invalid`, and records the exact analyzer version. Ordinary TCP output cannot satisfy the gate.

## Update evidence

The analyzer fixture contains a bounded synthetic TLS 1.3 client/server
transcript and its matching NSS key log. Replace the pair together only when
the encrypted HTTP assertion changes, and confirm the required analyzer tier
decrypts the declared method and host. Update the normalized matrix and report
only when executable evidence changes. `cargo xtask conformance` rejects a
generic corpus capture, missing traffic secrets, unresolved test identifiers,
and prohibited material. Read the complete diff before committing.

## Full verification

```text
cargo xtask ci
cargo xtask msrv
```

The pull request must also pass Windows portable execution and the dedicated TShark CI job before issue #305 can close.
