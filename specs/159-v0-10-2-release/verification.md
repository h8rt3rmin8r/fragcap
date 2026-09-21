# Verification Record: S159 v0.10.2 release

**Date**: 2026-09-20

## Candidate preparation

The S159 specification gate was committed before any candidate mutation. The established version-only cargo-release command moved the workspace to 0.10.2 without tagging, pushing or publishing. Embedded writer assertions, native conformance evidence, staged Windows identity and specification Applies-To moved to the same version. The three owning golden generators regenerated every version-bearing capture output.

Release assembly consumed fourteen accumulated S153 through S159 fragments into the dated v0.10.2 section and reset Unreleased. Assembly identified two existing documentation fragments whose unsupported `docs` suffix had never been accepted by the closed changelog grammar. Their unchanged content was reclassified to the supported `changed` section before successful assembly. No changelog schema or release tool changed.

The bounded v0.10.2 highlights validate and link to the tagged complete changelog. The candidate handoff preserves v0.10.1 as actual publication and keeps independent review, Doctor field validation and final Deep Capture acceptance open.

## Local source evidence

The following passed on the prepared candidate without installing or running released fragcap bytes:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --all --locked`
- `cargo xtask ci`
- `cargo xtask msrv`
- `cargo xtask neutral`
- `cargo xtask docs build`
- `cargo xtask spec`
- `cargo xtask supply-chain`
- `cargo xtask review-handoff`
- `cargo xtask review-record`
- `cargo xtask notes 0.10.2`

The aggregate gate reports thirteen documentation topics, thirteen guided-calibration criteria, ten threat-model rows, six fuzz targets, fifteen failure boundaries, fourteen performance cases, a closed Windows integration registry, complete package certification, current release-guard wiring and all source tests passing. Independent review remains explicitly not performed.

## Safety boundary

No installed product, real game, production Doctor, real trust mutation or sensitive live capture ran locally. Controlled source tests use synthetic loopback and repository-owned fixtures. Packaged execution remains the responsibility of disposable hosted Windows certification after push.

## Candidate review and merge

Official candidate pull request [#432](https://github.com/h8rt3rmin8r/fragcap/pull/432) completed on exact final head `3002bbf55ad1405cb7e54519f8535c9cd620e173`. Every required hosted check passed. The automatic Codex review and the one permitted manual second round both completed with no findings, and no inline review comments existed. The owner then merged the candidate as `13c9230d1e91b1fddd34de1f279963fa0e711ef4`.

## Public release evidence

Annotated tag object `70bd22034f686c727df4cd7a6d8e0078984fd11f` peels to exact merged source `13c9230d1e91b1fddd34de1f279963fa0e711ef4`. [Release run 35557886659](https://github.com/h8rt3rmin8r/fragcap/actions/runs/35557886659) completed without rerun or policy change. All four jobs passed:

- identity: 106204995789;
- final Windows package certification: 106205065384;
- GitHub release creation: 106207026652; and
- dependency-ordered crates.io publication: 106207107075.

GitHub held the registry job for the existing protected environment. Approval history records owner `h8rt3rmin8r` (user id 46768484), state `approved`, for `crates-io` environment 21294112472. The agent neither approved nor bypassed the deployment. Administrator bypass remains enabled and unchanged at the owner's direction.

The public [fragcap v0.10.2 release](https://github.com/h8rt3rmin8r/fragcap/releases/tag/v0.10.2) is neither draft nor prerelease and was published at `2026-09-21T03:46:16Z`. The downloaded files and sidecars reconcile exactly:

| File | Bytes | SHA-256 |
| --- | ---: | --- |
| `catalog.db` | 90112 | `d1da331be8e1c33b58f53b35e6a331755bfbc315d75be811214cc83488104fff` |
| `catalog.db.sha256` | 77 | `bc7cf4fb3499d1fdc605d23e1bc51b904273d7eada20a959a7b140d756293901` |
| `fragcap-0.10.2-x86_64-pc-windows-msvc.zip` | 6534526 | `246c34ff29b8e07ca5ac35e89534d51ce61bfa5d8384552e194dee062dc7e535` |
| `fragcap-0.10.2-x86_64-pc-windows-msvc.zip.sha256` | 108 | `30f11db62a87b117c67635148674eb60f0d8d52072845d0ab774ff6e83d18991` |
| `fragcap-0.10.2-x86_64.msi` | 6828032 | `ac279c6da241f5b50e97b97a76c7e373c72dcdd331d767a030645d4c650e2fac` |
| `fragcap-0.10.2-x86_64.msi.sha256` | 92 | `d54625e27fd44f072d1153f36002a291598cdaf153c18edb78693f2acbc4a368` |

Certification summary artifact 10621475367 has workflow digest `58ad1d70e3a5cb623957d104a11af6ffe78025ef558a3ce91504f62da38e670c`. Its downloaded schema-version-4 report binds exact official version, source, target, native backend and feature set. Both controlled surfaces require structured proxy-start and same-session reached-client evidence, retain strict firewall and process ownership checks, observe no non-loopback attempt, reconcile cleanup and complete with zero findings.

All ten crates.io version records report exact 0.10.2 and `yanked=false`:

| Crate | Registry checksum |
| --- | --- |
| `fragcap-core` | `082bc05b5c8a16d3f7b755e015aa90f42b0e8e3cbc2e17f7b81774d5a47e3de8` |
| `fragcap-profile` | `bc860c00c28e3dc22f7bdb26f75173e29d8b3250806948a67d63b82bada1d00a` |
| `fragcap-capture` | `fdb0d0b159e74dee87a2f1a93cc17ddefecd77060323b2a77dbd6248a3b99053` |
| `fragcap-attr` | `fc35047b901744db6d0187ccd61fd8be35df9c3adc95b2c6fa9089d07edc9217` |
| `fragcap-sink` | `5812b816f4b5606c45c5db4515e0334d5529e29d3d1c25424805f9cefc3812b1` |
| `fragcap-steam` | `bce7524ab7bbe9b920e2dedb03d26c9b450f299cf12eade4d5a5b8d6065b4ad7` |
| `fragcap-targets` | `4a661d4ff3e9fb3e71b809b8222329f67deeb61753496ecb458110741ed0734d` |
| `fragcap-proxy` | `a88cccd4cfa09853cf01ac6b24de8be927304044edb521e692e022fd762e60cb` |
| `fragcap` | `ff91b2caa7e6f1eb74d8a72ba1b7b7cff2d0b04e9c11f45f15aaee28990b06a3` |
| `fragcap-cli` | `b9d9349269af81a574201c27611428d22a2e2cabe4dc3ec2ac2dc852cd7361b6` |

## Records reconciliation boundary

The records branch changes only observed publication identity, immutable review-candidate data, current documentation and S159 evidence. It does not alter the released tag, package bytes, product version, dependencies, release workflow or environment policy. Historical v0.10.1 evidence remains intact. Independent review #333, Doctor field validation #372, documentation reconciliation #331 and final #334/#278 acceptance remain open. No installed fragcap binary, production Doctor, real game, trust mutation or sensitive capture ran locally.
