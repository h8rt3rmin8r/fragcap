# S130 Research: Native Supply-Chain and Compatibility Gate

## Decision 1: Close issue #328 as one release-critical policy boundary

S130 closes dependency-policy drift and release dependency evidence together. The two outputs share one exact locked Windows release graph, and separating them would allow either a gate with no distributable proof or an SBOM with no enforced policy behind it. Runtime behavior, final package installation certification, and the final Deep Capture completion gate remain outside S130.

The existing baseline is materially incomplete: `audit.yml` runs only weekly and manually despite the master specification naming push coverage; its stale S01 comment still claims there are no external dependencies; duplicate versions are warning-only; exceptions have no owner or expiry model; and no release artifact contains an SBOM or third-party notices.

## Decision 2: Separate offline repository policy from network-backed ecosystem intelligence

`cargo xtask supply-chain` becomes the deterministic, offline authority for the reviewed graph shape, release root, target, features, sources, checksums, declared Rust requirements, compatibility-line duplication, critical dependency pins, tool pins, exceptions, and release-workflow wiring. It derives normalized graphs through stable `cargo metadata --locked --offline` invocations and compares them with one versioned policy.

`cargo-deny` remains authoritative for RustSec advisories, yanked and unmaintained classifications, SPDX license evaluation, package bans, and registry or Git source policy. These facts can change without a repository change and therefore remain a network-backed workflow responsibility. The workflow runs on pull requests, `main`, manual dispatch, and one weekly schedule. It does not create a local automation or background process.

This split makes an unavailable or stale advisory database an audit infrastructure failure, never a clean result. It also keeps the ordinary local gate reproducible and fast.

## Decision 3: Pin the existing audit action and make findings blocking

The dependency audit uses `EmbarkStudios/cargo-deny-action` v2.1.1 at immutable commit `3c6349835b2b7b196a839186cb8b78e02f7b5f25`, with cargo-deny 0.20.2 and Rust 1.88.0 explicit. The action is invoked with all features and the supported Windows target. Advisory, yanked, unmaintained, unsound, license, ban, and source violations remain blocking.

The action documentation recommends a non-blocking advisory pattern for projects that do not want newly published advisories to break unrelated pull requests. Issue #328 explicitly requires the opposite. S130 deliberately keeps the result blocking and supplies an emergency patch procedure plus finite exception governance instead.

Primary sources: https://github.com/EmbarkStudios/cargo-deny-action/releases/tag/v2.1.1, https://embarkstudios.github.io/cargo-deny/checks/advisories/cfg.html, https://embarkstudios.github.io/cargo-deny/checks/bans/cfg.html, https://embarkstudios.github.io/cargo-deny/checks/licenses/index.html, and https://embarkstudios.github.io/cargo-deny/checks/sources/cfg.html.

## Decision 4: Fingerprint normalized graphs rather than duplicate Cargo resolution

The policy stores normalized digests and counts for three graph views: the complete Windows all-feature workspace, the complete Linux all-feature workspace, and the exact shipped Windows closure rooted at `fragcap-cli` with `live`, `socket-table`, and `etw`. Each normalized package identity includes name, version, source, checksum, declared license, declared Rust requirement, activated features, dependency kind, target expression, and exact dependency identity. Machine-local workspace paths are represented by stable workspace-relative identities.

Any graph or feature change therefore fails until the reviewed policy is updated. The validator separately emits actionable findings for common causes so a digest mismatch is not the only diagnostic. Windows-only, optional, development, and build edges remain covered in the all-feature policy even when the current host does not compile them. Release evidence walks only normal runtime edges from the shipped root; build dependencies remain policy-audited but are not falsely represented as distributed libraries.

Cargo metadata exposes declared `rust_version`; it does not prove the true minimum for crates that omit or misstate it. S130 rejects a declared floor above Rust 1.88, while the existing Rust 1.88 build remains the authoritative compatibility proof. Primary sources: https://doc.rust-lang.org/cargo/commands/cargo-metadata.html and https://doc.rust-lang.org/stable/cargo/reference/rust-version.html.

## Decision 5: Define duplicate compatibility lines precisely

For versions at or above 1.0, a compatibility line is the major version. For pre-1.0 versions, it is `0.minor`, matching Cargo compatibility semantics. Multiple exact versions inside one compatibility line remain visible in the report but are not mislabeled as duplicate-major violations. Every additional compatibility line requires an exact policy record with owner, rationale, expiry, and removal condition.

The current graph contains justified parallel lines including `bitflags` 1 and 2, `getrandom` 0.2 and 0.4, `syn` 2 and 3, and several Windows binding families. `cargo-deny` reports every duplicate version rather than this narrower semantic, so its general duplicate setting remains informational while the repository validator enforces the compatibility-line rule.

## Decision 6: Treat unsafe usage as review inventory, not an insecurity verdict

Unsafe Rust is necessary in Windows FFI, cryptography, allocators, and optimized protocol implementations. Cargo metadata cannot detect unsafe code, and cargo-geiger explicitly states that its counts do not determine whether code is insecure. S130 therefore binds unsafe review to the immutable complete-graph fingerprint and records reviewed security-critical or platform package families with their containment rationale. Any package, checksum, version, feature, or target-edge change invalidates that review and fails the static gate until the policy is deliberately refreshed.

This is narrower and more truthful than claiming a lexical scan or unstable statistic proves safety. The project continues to forbid unsafe code in its own non-platform modules through existing source policy and linting. Primary source: https://github.com/geiger-rs/cargo-geiger/blob/master/README.md.

## Decision 7: Use exact-pinned mature generators for release evidence

The release job installs exact `cargo-cyclonedx` 0.5.9 and `cargo-about` 0.9.2 with locked tool graphs. `cargo-cyclonedx` generates CycloneDX JSON 1.5 from the `fragcap-cli` manifest for the exact Windows target and shipped feature set, with binary description and `SOURCE_DATE_EPOCH` derived from the tag commit for reproducibility. `cargo-about` evaluates the same target and manifest through a committed configuration and template and emits complete third-party license texts.

A first-party CycloneDX writer was rejected because checksums, dependency relationships, target-specific features, PackageURL identities, and standard-version rules are already implemented by the official ecosystem tool. A first-party license-file scraper was rejected after the exact release graph showed packages that declare valid SPDX metadata but do not ship a discoverable local license file. The repository-owned validator remains independent: it parses the SBOM, checks graph binding and exact component coverage, validates notices package markers, and refuses stale, missing, duplicate, or unexpected evidence.

Primary sources: https://github.com/CycloneDX/cyclonedx-rust-cargo/releases/tag/cargo-cyclonedx-0.5.9, https://github.com/CycloneDX/cyclonedx-rust-cargo/blob/main/cargo-cyclonedx/README.md, https://github.com/EmbarkStudios/cargo-about/releases/tag/0.9.2, and https://embarkstudios.github.io/cargo-about/cli/generate/output.html.

## Decision 8: Embed evidence without changing the three-download release contract

The release stages `fragcap.cdx.json` and `THIRD-PARTY-NOTICES.txt` beside `fragcap.exe`, validates both before packaging, copies both into the portable archive, and adds fixed WiX components so the MSI installs both. They are not published as two new loose downloads. The existing ZIP and MSI checksums cover their exact bytes, while section 24.5's three-download contract remains intact.

S130 validates workflow and WiX ownership statically and adds a cheap installed-file assertion to the existing smoke step. S131 remains responsible for authoritative archive/MSI extraction, install, upgrade, repair, uninstall, checksum, signature, and final-content certification.

The current official cargo-cyclonedx 0.5.9 locked tool graph emits a Cargo warning because it retains yanked `xml-rs` 0.8.19. No RustSec advisory applies, the package is used only by the build-time SBOM generator, and it is neither linked into fragcap nor distributed. S130 records this as finite exception `S130-TOOL-001`, expiring after 90 days or when an official cargo-cyclonedx release removes that package. Using an unlocked generator graph was rejected because it would make release evidence depend on mutable transitive resolution.

## Decision 9: Make exceptions finite and procedures executable

Every exception is JSON data with a unique identifier, exact rule and package scope, owner, rationale, creation date, expiry date, and removal condition. A tool-graph exception additionally binds the exact governed tool and version that resolves the package. The validator rejects unknown fields, invalid or future dates, expired exceptions, duplicates, unused entries, and a package wildcard. An exception can narrow a finding but cannot disable the entire gate or turn an unavailable audit into success.

The maintainer procedure documents routine one-package updates and emergency advisory response. Contract tests seed valid update, advisory, stale-data, expired-exception, feature-drift, Windows-only drift, and rollback cases. The process preserves pull requests, CI, tag authorization, and the crates.io environment approval.

## Decision 10: Add no product or runtime dependency

All new implementation lives in the existing `xtask`, policy/configuration files, documentation, and workflows. `xtask` reuses its existing `serde_json` and `ring` dependencies. The evidence generators are exact-pinned CI tools and are not linked into fragcap. S130 performs no capture, proxy, routing, trust, process, or target effect.
