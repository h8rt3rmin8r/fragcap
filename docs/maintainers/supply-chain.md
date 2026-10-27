# Supply-chain maintenance

S130 makes the dependency graph a release authority rather than a best-effort report. `cargo xtask supply-chain` is the offline gate over the reviewed Windows, Linux, and exact shipped Windows graphs. The GitHub `audit` workflow adds fresh advisory, yanked, unmaintained, unsound, license, ban, and source intelligence. Neither result may be skipped or converted to a warning for release.

The same audit workflow checks advisories in all six isolated Rust graphs (`fuzz`, `performance/native-proxy`, and both proxy spikes plus their audit projects) using the pinned cargo-deny and root policy. A separate Node 24/pnpm 10.26.0 job installs `site/` with `--frozen-lockfile` and runs `pnpm audit --audit-level low` over production, development, and optional dependencies. PRs, main pushes, weekly runs, and manual dispatch all execute these checks. Registry failures and advisory findings remain blocking.

## Routine update

Update one direct package or one coordinated package family at a time. Refresh the lockfile deliberately, inspect the complete `cargo metadata` and `cargo tree -e features` diff, and run `cargo xtask supply-chain snapshot`. Review every package, checksum, feature, target edge, compatibility line, declared Rust floor, and unsafe containment change before replacing the affected digest and count in `supply-chain/policy-v1.json`.

Inventory every committed manifest and lockfile when reconciling Dependabot findings; the default workspace does not cover isolated graphs. For site changes, update the owning manifest or pnpm workspace override, regenerate the lock with the pinned package manager, and inspect every resolved copy. An override can block Dependabot from selecting a patched version. Preserve explicit supersession links when a consolidated PR incorporates a bot update. After merge, require scanner readback of fixed alerts rather than dismissing them.

The docs workflow runs `cargo fetch --locked` before root content contracts on PRs, main, and manual dispatch. Offline compiled-test discovery remains unchanged. Dispatch docs with `cold_cache=true` to isolate both Cargo home and target, skip cache restoration, fetch the locked graph, and run the original root contracts with network access disabled. This proves the actual dependency preparation and validation path without relying on a warm cache.

If a security-critical package or generator changes, update its exact pin, resolved feature inventory, `last_reviewed` date, compatibility boundary, and the dated decisions fragment in the same pull request. Do not widen a version requirement to make the gate pass. Run the Rust 1.88 build, the exact Windows release build, cargo-deny, evidence generation and validation, and `cargo xtask ci` before merge.

## Emergency advisory

Treat an actionable RustSec advisory as a release blocker. Prefer a patched exact version and update the smallest coordinated family that resolves it. Run the same graph, MSRV, Windows release, license, evidence, and full-CI checks as a routine update. If no patched version exists, stop release work unless a narrowly scoped temporary exception is explicitly approved in a pull request.

An exception must name one rule and one exact package version, owner, rationale, creation date, expiry date, and observable removal condition. A tool-graph exception must also name the exact governed tool and version that resolves the package. It cannot contain a wildcard, disable an entire check, excuse unavailable advisory data, or outlive its stated need. Mirror an advisory exception into cargo-deny's exact advisory ignore only after the repository record exists, and remove both together. The static gate rejects expired and unused records, including a tool exception whose pinned tool version has changed.

## Rollback

If an update fails compatibility or release evidence, revert that update through a new pull request, restore the prior lockfile and policy digests together, regenerate evidence, and run the complete gates. Never keep a new lockfile with the old reviewed digest or restore a vulnerable version after an actionable advisory without an explicitly approved finite exception.

If a tagged release fails before GitHub release creation, fix the release branch and create a new semantic version rather than moving the tag. If crate publication has begun, do not overwrite or republish an existing version; follow the registry's yank and successor-release process. Tag creation and the `crates-io` environment approval remain the two human release authorizations.

## Release evidence

The release workflow installs exact locked cargo-cyclonedx and cargo-about versions from `supply-chain/policy-v1.json`. It generates CycloneDX 1.5 and complete third-party notices from the exact `fragcap-cli` Windows release graph, then `cargo xtask supply-chain stamp-evidence` removes generator-only graph facts, canonicalizes local references, stamps versioned identities, and performs the independent validation before WiX or archive assembly. `cargo xtask supply-chain validate-evidence` can revalidate either completed artifact pair without changing it.

The portable archive and MSI each contain `fragcap.cdx.json` and `THIRD-PARTY-NOTICES.txt`. They remain embedded in the existing downloads; S130 does not add loose release artifacts. S131 owns authoritative installation, extraction, upgrade, repair, uninstall, checksum, signature, and final-content certification.
