# Quickstart: Session Causal Diagnostics

1. Confirm `codex/s190-session-causal-diagnostics` and `.specify/feature.json` select this slice.
2. Run focused controlled proxy cases for client TLS alert, reset, upstream TCP failure, upstream TLS failure after client TLS completion, and post-response idle timeout. Inspect retained stage, boundary, code, and loss facts against [the diagnostic contract](contracts/session-diagnostics.md).
3. Run controlled lifecycle cases for late client acquisition, acquisition timeout, full observation, and interrupted cleanup. Compare actual phase elapsed and remaining milliseconds across terminal and manifest projections.
4. Run repository formatting, Clippy, locked tests, documentation and conventions checks, and the applicable shell compliance checks. CI on the pushed PR is the exact-head result.
5. Review the PR diff for private values, encoding corruption, finite diagnostic bounds, scope drift, and truthful unknowns. Installed-title behavior is observed only after release; a concrete released-product failure is filed as its own issue.
