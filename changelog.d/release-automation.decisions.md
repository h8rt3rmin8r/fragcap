**2026-08-08** `release.toml` is added, and configured to move the version
number and nothing else: no tag, no push, no publish. A release tool that
commits, tags, pushes, and publishes in one step cannot coexist with an
integration workflow where nobody pushes to `main` directly. Splitting the
steps keeps the tag and the publish as separate authorized acts rather than
side effects of a version bump.

**2026-08-08** The `release` workflow stops exiting non-zero and becomes real.
Its jobs run in the order specification section 24.4 states: build artifacts,
generate checksums, create the release with notes from the changelog, then
publish to the registry. Publication is last because it is the only step that
cannot be undone, so a failure leaves a release that was never created rather
than crate versions with nothing to download.

**2026-08-08** Publication is gated on the `crates-io` environment, which
requires a human to approve the run. That makes the constitution's rule
against publishing without explicit authorization mechanical rather than
remembered. Registry credentials reach the job as `CARGO_REGISTRY_TOKEN` from
that environment, scoped to `publish-update` on `fragcap-*`.

**2026-08-08** Publication order lives in `cargo xtask publish` rather than in
the workflow, as section 24.4 requires. crates.io rejects a crate whose
dependencies are not already in the registry, so the order is load-bearing;
encoding it in Rust means a unit test asserts it against the dependency graph,
and a reordering that would break a release fails a check instead.

**2026-08-08** `cargo xtask publish` treats an already-published version as a
skip rather than an error, which makes a run resumable. Uploading eight crates
is eight network operations, and an interruption after the third would
otherwise leave a release permanently half published, because rerunning would
fail on the first crate and stop. Detection matches cargo's message text,
which is unpleasant but is what cargo offers: there is no skip-existing flag
and no distinct exit code, and querying the registry over HTTP would put a
network client into a crate that deliberately has no external dependencies.

**2026-08-08** Release artifacts are knowingly incomplete against
specification section 24.5, which specifies an archive carrying the binary,
both shell wrappers, the bundled profiles, the license, and the notice. The
wrappers arrive with slice S18 and the profiles with S05, so those directories
are empty and the archive cannot yet be complete. Rather than ship a partial
archive that looks finished, the packaging step names the absent components,
writes them to `INCOMPLETE.txt` inside the archive, and warns in the log. The
record is generated from what is actually on disk, so it retires itself when
the components land.
