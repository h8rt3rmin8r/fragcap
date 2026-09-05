# Contract: Native Windows Integration Gate

## Commands

```text
cargo xtask windows-integration
cargo xtask windows-integration --run hosted --binary <path> --report <path>
cargo xtask windows-integration --run physical --binary <path> --report <path>
cargo xtask windows-integration --validate-report <path>
cargo xtask windows-integration --release
```

The first form validates static registry, source, workflow, and committed summary authority. `--run` is Windows-only, finite, and writes a local append-safe report plus a separate public-safe summary. `--validate-report` validates either report form without executing effects. `--release` additionally requires current physical evidence for the exact registry and product revision.

## Exit Contract

- `0`: Requested validation or execution completed and every required condition passed.
- `1`: A row, report, cleanup, capability, source reference, or release authority failed.
- `2`: Command usage is invalid or the requested execution tier is unsupported on this platform.

No successful exit may contain a skipped required row.

## Registry Contract

The registry is `integration/windows-native-matrix-v1.json`. Its digest is computed over the committed bytes. Schema version 1 uses closed vocabularies for tiers, capabilities, authorities, outcomes, effects, cleanup, and publication.

Every executable test reference names a repository-relative UTF-8 source path and exact attributed test function. Static validation rejects missing files, missing functions, ignored or conditionally disabled hosted tests, and references outside the repository.

Physical tests may carry an explicit tier marker that prevents ordinary test discovery from mutating the host. They execute only through the `physical` runner after immutable preflight succeeds.

## Execution Contract

1. Validate the registry and compute its digest.
2. Validate the staged binary path, version, digest, and source revision.
3. Capture one immutable host capability snapshot.
4. Refuse the whole run if the tier's required capability shape is not present.
5. Capture the normalized owned-effect inventory.
6. Execute rows in registry order with direct argv, redirected streams, `CREATE_NO_WINDOW`, and the row deadline.
7. After each row, record exact outcome and cleanup evidence.
8. Re-snapshot capabilities and effects.
9. Emit one terminal record. Capability drift, missing rows, or residue makes it incomplete.

The runner never invokes a shell, installs software, changes firewall rules, changes system proxy settings, schedules work, or reaches a non-loopback destination.

## Evidence Contract

Raw reports and scratch bundles remain ignored local artifacts. The public-safe summary is constructed only from typed closed fields and must pass prohibited-value tests before it can be committed or uploaded.

Physical evidence authorizes release only when:

- its registry digest matches current committed bytes;
- its source revision is an ancestor of or equal to the release candidate according to the recorded policy;
- its product version and staged binary identity are present;
- every physical row passed exactly once;
- residue is zero;
- evidence age is within the registry maximum;
- no prohibited field class is present.

## Workflow Contract

The required pull-request workflow:

- runs only on `pull_request`, `push` to `main`, and explicit dispatch;
- contains no schedule;
- builds the production-feature Windows executable using a build-time Npcap SDK that is never uploaded;
- stages the executable outside Cargo output;
- runs static and hosted tiers;
- validates the summary before upload;
- uploads only the public-safe summary;
- has no conditional that can skip a required hosted row.

## Packaging Handoff

S129 proves the relocated staged executable and package-independent Windows behavior. Issue #329 must replace or supplement the staged-layout row with final MSI and archive install, upgrade, repair, uninstall, checksum, signature, and content evidence before #334 closes.
