**2026-08-10** The release-versioning scheme is codified, and there is one first
public release. `v0.1.0` is the crates.io namespace-reservation stub already
published and carries no functionality. The first public release is `v0.2.0`, and
it comprises the whole roadmap: all eighteen slices, S01 through S18. It, and the
crates.io publication of the functional crates, happen only after every slice is
complete and operational; there is no earlier functional release. This reverses an
earlier same-day decision that split the roadmap across two functional releases
(`v0.2.0` at the end of S14, `v0.3.0` for S15 through S18); the split is retired.
The specification is corrected to match: section 3.3 success criteria are no longer
partitioned (`v0.2.0` is complete when SC-1 through SC-7 hold), section 27.3's
release table collapses to a single functional release, section 28 is retitled
"Roadmap Beyond v0.2.0", and the scope prose in the specification, its outline, and
the plan documents is updated. The workspace manifest stays at `0.1.0`; the bump to
`0.2.0` is a release-time `cargo release minor` action taken only once every slice
is complete, so no code artifact changes here.

**2026-08-10** The `0.2.0` version bump is necessary but not sufficient at
release time, and the release runbook must say so. The pcapng `USER_APPL` and
the JSON Lines `VERSION` are both `concat!("fragcap/", env!("CARGO_PKG_VERSION"))`,
so `cargo release minor` changes them from `fragcap/0.1.0` to `fragcap/0.2.0`.
That moves the value embedded in every committed golden and the two assertions
that pin it (`crates/fragcap-sink/src/pcapng/mod.rs` and
`crates/fragcap-sink/src/json/mod.rs`), so the release commit must also
regenerate the golden corpus and update those assertions, or `cargo xtask ci`
fails on the release branch. Recorded now so the obligation is not discovered
during the release.
