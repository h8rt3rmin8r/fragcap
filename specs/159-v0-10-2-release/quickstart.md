# Quickstart: S159 v0.10.2 release verification

Do not install or execute a released fragcap binary, a real game, production Doctor, real trust mutation or sensitive live capture on the local machine.

## Candidate checks

```powershell
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all --locked
cargo xtask ci
cargo xtask msrv
cargo xtask neutral
cargo xtask changelog --check
cargo xtask notes 0.10.2
```

## Publication checks

```powershell
git rev-parse v0.10.2^{}
gh run view <release-run-id>
gh release view v0.10.2
```

Download public files only into an ignored temporary directory for checksum and certification validation. Do not unpack, install or execute them locally.

## Registry checks

Query crates.io for all ten package version records and require exact 0.10.2, `yanked=false` and a nonempty checksum. Registry propagation may be delayed, so bounded rechecks are observation rather than retries of publication.
