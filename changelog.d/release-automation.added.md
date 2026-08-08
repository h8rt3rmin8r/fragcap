`cargo xtask publish` publishes the workspace to crates.io in dependency
order, asserts that order against the dependency graph before uploading
anything, and skips a crate whose version is already on the registry so an
interrupted release can be resumed by rerunning it. It prints the plan and
changes nothing unless `--execute` is passed.

`cargo xtask notes <version>` prints that version's release notes from
`CHANGELOG.md`, falling back to the `Unreleased` section.

A tagged release now builds a checksummed Windows archive carrying the binary,
the license, and the notice, and creates a GitHub release with notes derived
from the changelog. `release.toml` configures the version bump that precedes
all of it.
