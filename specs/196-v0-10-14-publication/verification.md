# Verification: S196 v0.10.14 Publication

## Source Freeze, 2026-11-06 UTC

- Accepted merge and local `main`: `5efb38ee93daff140e1ba79f57eea0a93d5b572f`.
- Release candidate PR: [#568](https://github.com/h8rt3rmin8r/fragcap/pull/568), merged at `2026-11-06T08:40:07Z`.
- Local tag `v0.10.14`: absent before publication.
- Remote tag `v0.10.14`: absent before publication.
- Public GitHub release `v0.10.14`: absent before publication.
- The records branch starts at the accepted merge. The later records commit is not release source.

## Accepted Merge Workflows

Every merge-triggered workflow on the exact accepted merge completed successfully:

| Workflow | Run |
| --- | ---: |
| Push on main | [42189327949](https://github.com/h8rt3rmin8r/fragcap/actions/runs/42189327949) |
| audit | [42189328188](https://github.com/h8rt3rmin8r/fragcap/actions/runs/42189328188) |
| Windows package certification | [42189328209](https://github.com/h8rt3rmin8r/fragcap/actions/runs/42189328209) |
| docs | [42189328226](https://github.com/h8rt3rmin8r/fragcap/actions/runs/42189328226) |
| ci | [42189328280](https://github.com/h8rt3rmin8r/fragcap/actions/runs/42189328280) |
| Published review candidate | [42189328282](https://github.com/h8rt3rmin8r/fragcap/actions/runs/42189328282) |
| Windows native integration | [42189328287](https://github.com/h8rt3rmin8r/fragcap/actions/runs/42189328287) |
| platform | [42189328288](https://github.com/h8rt3rmin8r/fragcap/actions/runs/42189328288) |
| fuzz | [42189328314](https://github.com/h8rt3rmin8r/fragcap/actions/runs/42189328314) |
| Native performance | [42189328432](https://github.com/h8rt3rmin8r/fragcap/actions/runs/42189328432) |

## Spec-Kit Analysis

- Functional requirements: 12.
- Measurable outcomes: 6.
- Implementation tasks: 36.
- Requirement and outcome coverage: 100 percent.
- Ambiguities: 0.
- Duplications: 0.
- Constitution conflicts: 0.
- Unmapped tasks: 0.
- Critical, high, medium, and low findings: 0.

The specification, clarification session, requirements checklists, plan, research, data model, publication contract, quickstart, and tasks agree that S196 completes the entire publication and current-record journey while retaining distinct evidence phases.

## Tagged Publication

- Annotated tag object: `1e7b4a875b59177fa537bbb2ce88dd9b3ba51a0f`.
- Peeled source: `5efb38ee93daff140e1ba79f57eea0a93d5b572f`.
- Tag type: annotated `tag` object.
- Tag message: `fragcap v0.10.14`.
- The single tag ref was pushed without a branch push or force operation.
- Tagged release workflow: [42193284424](https://github.com/h8rt3rmin8r/fragcap/actions/runs/42193284424).
- Identity job `127810242104` completed successfully and validated version/tag agreement plus normal registry protection.
- Final Windows package certification job `127810429336` completed successfully in 15 minutes 24 seconds.
- GitHub release job `127816992451` completed successfully in 42 seconds.
- Protected crates.io publication job `127817309949` completed successfully in 2 minutes 33 seconds.
- The protected environment presented deployment `7596807011` for normal owner review. The authenticated owner approved it under the explicit S196 delegation without a protection change or bypass.

## Public GitHub Asset Readback

- Release: [v0.10.14](https://github.com/h8rt3rmin8r/fragcap/releases/tag/v0.10.14), release ID `436312514`.
- Published at: `2026-11-06T09:29:10Z`.
- State: public, not draft, not prerelease.
- Immutable tag source: `5efb38ee93daff140e1ba79f57eea0a93d5b572f`.

Each asset was downloaded independently after publication. Its local size and SHA-256 matched the GitHub API digest and the published checksum sidecar where applicable.

| Asset | Bytes | SHA-256 |
| --- | ---: | --- |
| `catalog.db` | 90112 | `b26bb350099d416f4caf9950241b05745ea28141f788da78ee31a68ecdc5be74` |
| `catalog.db.sha256` | 77 | `f51899002c536b9571c66c93190c63723f13f5e61db96b71e7bfb5fd3a944bd3` |
| `fragcap-0.10.14-x86_64-pc-windows-msvc.zip` | 7612217 | `1653a8061be070f77110161ab21d4c412f949047ba5b6b28e284fee5b714236c` |
| `fragcap-0.10.14-x86_64-pc-windows-msvc.zip.sha256` | 109 | `09e319cb586e8039225bfc20186b9884bbd78f41a43d2ca7f109a6dd6b76f23c` |
| `fragcap-0.10.14-x86_64.msi` | 7860224 | `cf66979357c6946de444237c1bb56e520e6b36d872ebb4f108df4c87cdf35c67` |
| `fragcap-0.10.14-x86_64.msi.sha256` | 93 | `88302b18926cf3392ced70e6a9811ae79a9cdf9bec84dc54873135794f56abf9` |

The downloaded tagged schema-4 certification report passed `cargo xtask package-certification validate-report` against these six files. It records official v0.10.14, the exact tagged source, the official target and feature closure, native backend, six complete package contents, two complete controlled loopback smokes, six complete lifecycle cases, a complete fresh start, and zero findings. The MSI signature field is `not_signed`.

## Public crates.io Readback

The primary registry API reported all ten 0.10.14 packages as non-yanked. Each independently downloaded `.crate` archive matched the registry checksum.

| Crate | Bytes | SHA-256 |
| --- | ---: | --- |
| `fragcap-core` | 140428 | `d2e79eb2792e7fc3419000326784a7eac9d9301e20f6df2b9dd75db75435cc39` |
| `fragcap-profile` | 112361 | `49a199f9f574430ecf36073d32a3261fe639bed793705633916ef74fec14e0e0` |
| `fragcap-capture` | 39474 | `544a32a9d4257923f435052ba484e01454c0ed1632a32bbc87199a10669e6904` |
| `fragcap-attr` | 83126 | `7d3df06cd3cb186f8584f762c7248c2d8eb03e7147b294716e7bd81646f7f975` |
| `fragcap-sink` | 69899 | `9bfea70d1bc24710a77149f9f8de2077f247f8e98d8f7e6f483894830833f9dd` |
| `fragcap-steam` | 51102 | `a42c4c384f8b3be2de66bad0353e8d2cacbdcd0b8eb37d46f9f7b1e2c481617f` |
| `fragcap-targets` | 167745 | `482d943dc5ba1e04f955bbb17ee21f55b4891cbfd841be0c33a1c278b79ac2bb` |
| `fragcap-proxy` | 187485 | `f3dd563009e6291c6a313333ea9b36316614ed4e417f85529fa1014aaafb00c2` |
| `fragcap` | 336671 | `e7b9ab8ac26ba1f01ec40345bbd9d238e25e07ed896713113085e0e934cf6447` |
| `fragcap-cli` | 585394 | `e08241e3593c3cc9a73a7060e1645bd18f65d6641f6bd10aa53367b3d74ea8dc` |

## Installed Public Product Observation

- The public ZIP contained the exact six certified package entries. Its `fragcap.exe` SHA-256 is `6596411c321ab7726ef4b750000043117106bcd90e2aedd892bd518b42fe06b6`, matching the tagged certification report.
- The public portable executable reported `fragcap 0.10.14` and official build identity bound to source `5efb38ee93daff140e1ba79f57eea0a93d5b572f`.
- `doctor --json` ran through a hidden, noninteractive process with no storage override. It selected `C:\Users\<account>\.fragcap\roaming\catalog.db`, `C:\Users\<account>\.fragcap\roaming\local.db`, `C:\Users\<account>\.fragcap\roaming\sessions`, and `C:\Users\<account>\.fragcap\roaming\sessions\session-owners` as current-user defaults.
- The selected `.fragcap` root was absent on this account, so no effective opened-handle destination or native session bundle existed to inspect or clean up. Doctor continued to report retained legacy AppData stores at their existing paths.
- The workstation began with the machine-wide v0.10.13 product in `C:\Program Files\fragcap`. A hidden noninteractive upgrade using the verified public v0.10.14 MSI exited 1603. Windows Installer recorded that a silent install could not elevate and administrator authority was required to remove the existing machine-wide application. The v0.10.13 installation remained unchanged.
- Hosted certification, which ran with the required authority on a disposable Windows runner, completed clean install, repair, same-version reinstall, upgrade, downgrade refusal, uninstall, installed smoke, portable smoke, and fresh-start checks.
- A private real game title was not run. The workstation limitation is absent elevation, not a demonstrated product defect. No v0.10.14 issue was filed.

## Publication Records Pull Request

The local records candidate passes:

- `cargo xtask ci`
- `cargo xtask notes 0.10.14`
- `cargo xtask spec`
- `cargo xtask review-record`
- `cargo xtask docs check`
- `cargo xtask docs build`
- `cargo xtask package-certification`
- `cargo xtask supply-chain`
- `cargo fmt --all -- --check`
- all six isolated Rust graph `cargo deny --locked --all-features --manifest-path <manifest> --config deny.toml check advisories` invocations
- `git diff --check`
- UTF-8/mojibake scan across the changed documentation and S196 records

The immutable registry validator reports that the v0.10.14 published candidate is valid and that independent review has not been performed. PR identity, exact-head hosted checks, hosted reviews, merge, and Pages deployment remain pending.

The first hosted docs run exposed a retained accessibility-contract expectation for the `S154` and `S158` lineage markers on the Native product contract page. The page now keeps those exact historical anchors while presenting v0.10.14 as current. The failed run remains part of the diagnostic history; exact-head rerun evidence supersedes it only after success.

## Final Public Reconciliation

Pending.
