# Quickstart: Validate Complete Process Lifecycle Evidence

## Preconditions

- Use the committed controlled or offline harness. No game, account, Internet access, capture driver, or elevation is required.
- Keep `.specify/feature.json` pointed at `specs/123-process-lifecycle-evidence`.

## Focused validation

```powershell
cargo test -p fragcap process_trace
cargo test -p fragcap-cli process_trace
cargo test -p fragcap-cli --test cli_deep_capture
```

Expected outcomes:

- Cold direct, platform, and publisher fixtures retain distinct launch and stage chronologies.
- PID reuse and event-order permutations never transfer identity.
- Flow-owner intervals use the same `flow_id` as packet and application evidence.
- Injected watcher and retention loss produces partial truth with exact counters.
- A complete stream has one header and one trailer; a crash prefix has no completion claim.

## Full gate

```powershell
cargo xtask ci
```

The gate must finish with `ci: all checks passed`. Review the diff to confirm `Cargo.lock` is unchanged and no prohibited process-handle API or memory right was introduced.
