# Publication Contract: S196

1. Annotated tag `v0.10.14` exists once and peels to accepted merge `5efb38ee93daff140e1ba79f57eea0a93d5b572f`. A divergent existing tag is never overwritten.
2. The tag-driven workflow succeeds for identity, final Windows package certification, GitHub release creation, and protected crates.io publication. Registry approval uses the configured environment without protection changes or bypass.
3. The public GitHub release is neither draft nor prerelease. Exactly six public files are independently downloaded; the three sidecars validate their primary files; API size and digest agree with computed bytes; the public certification report identifies official v0.10.14 from the tagged source with complete required lifecycle evidence.
4. All ten exact v0.10.14 crate records are public and non-yanked. Every independently downloaded archive matches its crates.io checksum. Registry propagation delay is retried without duplicate publication.
5. Installed-product observation begins only after conditions 1 through 4 hold. It uses the verified public MSI and default product storage, retention, bundles, and cleanup. It names effective paths and remaining native state, makes no private target public, and files concrete v0.10.14 defects when observed.
6. Current documentation, published-release JSON, frozen review candidate, site markers, and release handoff advance only from conditions 3 and 4. Candidate or workflow artifacts cannot substitute for public readback.
7. The records pull request receives terminal-successful applicable checks, at most two hosted Codex review rounds, responses to every actual comment, and zero unresolved threads before merge.
8. S196 completes only when the records merge is on main, Pages publicly serves v0.10.14 current state, issue #567 and the milestone are complete, local slice state is removed, and clean local main equals origin/main.
