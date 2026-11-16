# S211 Verification Quickstart

1. Run the focused helper latency, refusal, timeout, cancellation, and packet-conservation tests.
2. Run `cargo xtask ci` in the foreground and inspect the terminal result.
3. Build the exact candidate portable package using the repository package workflow. Verify executable and signed driver identities.
4. In the owner-scheduled network window, run the controlled external peer and elevated client matrix once. Keep the native bundle and route/compatibility/result/manifest readback linked to the candidate identity. Check exact helper and peer cleanup.
5. Push the verified branch, open the official PR, resolve first and optional second Codex review plus security review, and wait for green applicable CI. Owner performs the final merge.
