`cargo xtask publish` publishes the workspace to crates.io in dependency
order, and asserts that order against the dependency graph before uploading
anything. It prints the plan and changes nothing unless `--execute` is passed.
`release.toml` configures the version bump that precedes it.
