# Validation Quickstart: S163

## Prerequisites

Use the pinned Rust toolchain and a clean local store in controlled tests. The public test identity is synthetic. No real game, elevated terminal, proxy trust change, or operator capture bundle is required.

## Controlled scenarios

1. Run the owned-root tests with a prelaunch same-name snapshot, a foreign creation event, and a matching basename-only event for the launch receipt. Expect one platform binding and one title dispatch. Repeat with stale PID, wrong parent, changed path, reused identity, early exit, and watcher loss; expect no unauthorized dispatch.
2. Run the first-run setup tests with a socketless launcher and one socket-owning descendant. Expect one evidence-backed client proposal, one exact authoring plan, and no write until confirmation. Run zero, ambiguous, declined, interrupted, and drift cases; expect zero target changes.
3. Run the controlled reachability chain after authoring using the reserved synthetic direct-launch proxy harness. Expect a fresh plan and a directly observed proxy reachability fact. This verifies setup-to-plan authority and the proxy path, not real Steam child-environment propagation. An artifact with zero target traffic or zero proxy accepts must remain unsuccessful.
4. Run target-detail and report fixtures. Expect distinct hint and active-client labels, one exact causal failure stage, exact fact disposition, and no repeated generic cleanup lines.
5. Run the full repository gate: `cargo xtask ci`. Review its format, lint, test, documentation, and shell results before committing.
