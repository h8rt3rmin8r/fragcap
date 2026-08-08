**2026-08-08** `release.toml` is added, and configured to move the version
number and nothing else: no tag, no push, no publish. A release tool that
commits, tags, pushes, and publishes in one step cannot coexist with an
integration workflow where nobody pushes to `main` directly. Splitting the
steps keeps the tag and the publish as separate authorized acts rather than
side effects of a version bump.

**2026-08-08** The `release` workflow stops exiting non-zero and becomes real.
It builds the Windows binary, asserts no npcap component is present, checks
that the tag agrees with the workspace version, and then publishes behind the
`crates-io` environment gate. The gate requires a human to approve the run,
which makes the constitution's rule against publishing without explicit
authorization mechanical rather than remembered. Registry credentials reach
the job as `CARGO_REGISTRY_TOKEN` from that environment, scoped to
`publish-update` on `fragcap-*`.

**2026-08-08** Publication order lives in `cargo xtask publish` rather than in
the workflow. crates.io rejects a crate whose dependencies are not already in
the registry, so the order is load-bearing; encoding it in Rust means a unit
test asserts it against the dependency graph, and a reordering that would
break a release fails a check instead.
