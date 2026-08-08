**2026-08-08** The `ci` workflow gains a publication licensing step, and its
header note is corrected to record that the `check` matrix has now run and
passed while `neutrality` and `msrv` have not. The previous note claimed the
workflow had never executed because there was no remote, which stopped being
true when slice S01 integrated through pull request #1.

**2026-08-08** The eight crate names are reserved on crates.io at 0.1.0, the
version already declared in the workspace manifest, rather than at a 0.0.0
placeholder. The facade crate cannot be published without its six dependencies
already in the registry, so reserving the headline name means publishing the
whole graph; each crate's README states that the release is a skeleton so the
listing does not overclaim.
