# S107 Verification Quickstart

1. Run focused proxy tests for TLS 1.2/TLS 1.3 key logs, concurrency, write failures, mutual TLS, and refusal classification.
2. Run facade tests for protected preparation ordering, journal bounds and replay, exact idempotent cleanup, and immutable atomic share copies.
3. Run CLI tests for paired credential arguments, plan confirmation, key-log readiness and manifest truth, bundle cleanup confirmation, and bundle export.
4. On Windows, inspect a real session bundle and key-log DACL and confirm only the owner and SYSTEM have inherited or direct access as declared.
5. Verify `.github/workflows/platform.yml` has no path filter and run the repository lint contract.
6. Run `cargo xtask fmt`, `cargo xtask lint`, `cargo xtask deps`, `cargo xtask test`, `cargo xtask msrv`, `cargo xtask doc`, and the remaining repository gates named in `CONTRIBUTING.md`.

Expected result: requested client-facing key logs decrypt the controlled TLS capture, explicit mutual TLS succeeds, ambiguous refusals remain unknown, sensitive cleanup and sharing are exact and recoverable, and the Windows platform workflow is selected for every pull request.
