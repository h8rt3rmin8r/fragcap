# Contract: Native Supply-Chain Gate and Release Evidence

## Commands

`cargo xtask supply-chain` validates the policy and all required offline graph views. Exit 0 means complete and conforming, exit 1 means the check ran and found policy violations, and exit 2 means required metadata or policy could not be read.

`cargo xtask supply-chain stamp-evidence <sbom-path> <notices-path>` filters standard-generator output to the exact release closure, canonicalizes local package references, stamps release identities, and immediately runs the independent validator. Release assembly uses this command.

`cargo xtask supply-chain validate-evidence <sbom-path> <notices-path>` validates already generated release evidence against the exact locked shipped Windows graph. It never generates evidence or downloads advisory data.

## Required Graph Views

| View | Root | Target | Features | Edge scope |
| --- | --- | --- | --- | --- |
| `windows-all` | Complete workspace | `x86_64-pc-windows-msvc` | All | Normal, build, development |
| `linux-all` | Complete workspace | `x86_64-unknown-linux-gnu` | All | Normal, build, development |
| `windows-release` | `fragcap-cli` | `x86_64-pc-windows-msvc` | `live,socket-table,etw` | Normal runtime closure |

Every resolved package and edge belongs to the canonical digest. Workspace absolute paths, metadata ordering, and host-specific target directories do not.

## Finding Rules

Findings use stable rule identifiers: `graph-drift`, `package-count`, `edge-count`, `source`, `checksum`, `license-metadata`, `declared-msrv`, `critical-pin`, `critical-feature`, `critical-declaration`, `critical-default-features`, `critical-review-expired`, `duplicate-compatibility`, `unsafe-review-drift`, `exception-schema`, `exception-expired`, `exception-unused`, `workflow-trigger`, `workflow-tool-pin`, `workflow-order`, `artifact-wiring`, `sbom-schema`, `sbom-identity`, `sbom-component`, `sbom-dependency`, `notices-identity`, `notices-component`, `notices-license`, and `finding-limit`.

Diagnostics are sorted and bounded. Each includes view, package when applicable, rule, observed fact, expected fact, and remediation class. No diagnostic includes a credential, host name, account name, absolute operator path, certificate, key, or payload.

## Exception Matching

An exception matches one rule and one exact package identity only. A tool-graph exception additionally matches one exact governed tool and version, so changing the tool immediately makes the exception unused. No wildcard, empty package, global disable, or infrastructure-failure exception is valid. The current date must fall on or before `expires`, the exception must be used exactly once, and all governance fields are mandatory.

## Network Audit

The audit workflow invokes the immutable cargo-deny action and exact cargo-deny/Rust versions for the complete Windows all-feature graph. It runs on pull requests, pushes to `main`, manual dispatch, and Monday at 06:00 UTC. Advisory, yanked, unmaintained, unsound, license, ban, and source findings fail the job. Fetch or database failure also fails the job and is not suppressible through the repository exception schema.

## Evidence Generation

The release job may generate evidence only after the locked release binary graph builds and before WiX or archive assembly. It installs exact locked generator versions, sets `SOURCE_DATE_EPOCH` from the tagged commit, runs both generators against `crates/fragcap-cli/Cargo.toml`, the exact Windows target, and the shipped feature set, then calls `stamp-evidence`, which concludes by validating the completed pair.

Validated files are named `fragcap.cdx.json` and `THIRD-PARTY-NOTICES.txt`. Both are staged beside the executable, installed by fixed WiX components, and copied into the portable archive. Neither is published as a fourth or fifth loose download.

## Release Ordering

```text
locked build -> evidence generation -> evidence validation -> MSI/archive assembly -> checksums -> GitHub release -> crates.io approval -> publication
```

No later step may use `continue-on-error` for the supply-chain authority. S131 may strengthen package extraction and installation validation but must not bypass or replace the evidence gate.

## Non-Effects

Both commands are repository checks. They perform no capture, proxy, routing, trust-store, process-control, target-launch, or desktop scheduling effect. Only the network audit fetches mutable advisory/index data, and it runs in GitHub Actions.
