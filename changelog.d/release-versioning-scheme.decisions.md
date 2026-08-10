**2026-08-10** The release-versioning scheme is codified. `v0.1.0` is the
crates.io namespace-reservation stub already published and carries no
functionality; the first functional release is `v0.2.0`, the usable
capture-to-file CLI reached at the end of slice S14; `v0.3.0` completes slices
S15 through S18. The specification is corrected to match: section 3.3 success
criteria are partitioned across the two functional releases, section 28 is
retitled "Roadmap Beyond v0.3.0", and the scope prose that used `v0.1.0` as a
synonym for the deliverable is updated in the specification, its outline, the
brand notes, and the plan documents. The workspace manifest stays at `0.1.0`;
the bump to `0.2.0` is a release-time `cargo release minor` action when S14
lands, so no code artifact changes here.
