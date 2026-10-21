# Quickstart and Controlled Verification

1. Render `fragcap doctor` with controlled elevated and unelevated Windows facts. Check matching human, JSON, exit, blocking-check, and `doctor --fix` recheck results. Offline replay remains a separate path.
2. In an isolated local store, run first-time Steam client setup with one exact plan identifier and LF, verify the authored target and setup-complete terminal result, then run its offered command against the same store to reach a fresh calibration decision.
3. Repeat setup with CRLF, incomplete, stale, and declined responses. Verify no target mutation and no setup-complete claim.
4. Render `deep-capture --help` at 40, 60, 80, and 100 columns. Verify mandatory launch, useful wording, a copyable recipient command, and sensitive-output explanations. Check the exact-LF PowerShell example separately.
5. Run focused CLI tests and `cargo xtask ci` on the final source. Report real-title and active-release behavior as unverified unless separately observed.
