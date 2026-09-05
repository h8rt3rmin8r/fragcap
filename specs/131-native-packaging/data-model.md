# Data Model: Native Windows Packaging Certification

## PackageContract

- `schema_version`: Exact supported contract schema.
- `release_identity`: Expected product name, target triple, architecture, official features, native backend, and semantic-version rules.
- `primary_artifacts`: Exact portable archive, installer, and standalone catalog declarations.
- `checksum_sidecars`: One exact SHA-256 sidecar per primary artifact.
- `shared_entries`: Exact six-file package payload with role, ownership, multiplicity, and size ceiling.
- `prohibited_content`: Closed filename, extension, import, and semantic role rules.
- `pe_policy`: Expected PE machine plus ordinary and delayed import allowlists.
- `signature_policy`: Expected signature state by artifact role.
- `installer_effects`: Exact owned program and machine state.
- `user_owned_fixtures`: State that must survive every lifecycle transition byte-identically.
- `lifecycle_cases`: Complete required case registry with deadlines and allowed outcomes.
- `predecessor`: Exact version, URL, size, and SHA-256 of the supported upgrade input.
- `report_policy`: Finding, path, string, collection, and byte bounds.
- `workflow_policy`: Required triggers and publication dependency order.

## ReleaseBuildIdentity

- `schema_version`: Exact machine-readable identity schema.
- `product`: Exact `fragcap` identity.
- `version`: Workspace semantic version.
- `source_revision`: Full tagged commit identity supplied by official packaging.
- `target`: Rust target triple.
- `architecture`: Canonical `x86_64` architecture.
- `features`: Sorted exact explicit official feature set.
- `deep_capture_backend`: Exact `fragcap-native` value.
- `official`: True only when the packaging workflow supplies and later certifies the required identity inputs.

## PrimaryArtifact

- `role`: Portable archive, installer, or standalone catalog.
- `filename_pattern`: Closed version and architecture-bound filename.
- `size`: Observed final byte count.
- `size_ceiling`: Reviewed maximum byte count.
- `sha256`: Independently calculated lowercase hexadecimal digest.
- `signature_state`: `not_signed` for installer or executable surfaces, otherwise `not_applicable`.
- `entries`: Sorted exact package entries where the artifact is a container.
- `complete`: True only after all identity, content, size, checksum, and signature rows reconcile.

## PackageEntry

- `artifact`: Parent artifact identity.
- `relative_path`: Canonical slash-separated relative path.
- `case_folded_path`: Collision key.
- `role`: Binary, catalog, project license, project notice, software bill of materials, or third-party notices.
- `ownership`: Installer-owned immutable program file.
- `size`: Observed byte count.
- `size_ceiling`: Reviewed maximum.
- `sha256`: Digest of exact entry bytes.
- `shared_identity`: Digest that must agree across ZIP, MSI, and standalone surfaces where applicable.

## PeInspection

- `machine`: Canonical PE machine identifier.
- `ordinary_imports`: Sorted case-normalized DLL names.
- `delayed_imports`: Sorted case-normalized DLL names.
- `version_resource`: FileVersion, ProductVersion, ProductName, and OriginalFilename.
- `unknown_imports`: Any import outside the closed allowlist; must be empty.
- `complete`: False when parsing or version-resource inspection is indeterminate.

## InstallerEffect

- `id`: Stable effect identifier.
- `kind`: Program file, product registration, PATH element, or Defender exclusion.
- `ownership_key`: Exact product/component/path identity rather than a display-name heuristic.
- `install_expectation`: Present, absent, or best-effort classified.
- `repair_expectation`: Restored, unchanged, or classified.
- `upgrade_expectation`: Replaced, preserved, or classified.
- `uninstall_expectation`: Absent.
- `observed_state`: Present, absent, unavailable, ambiguous, or error.

## LifecycleCase

- `id`: Clean install, repair, reinstall, upgrade, downgrade refusal, or uninstall.
- `preconditions`: Exact starting package and seeded state.
- `invocation`: Installer operation and arguments without host-specific paths.
- `deadline_seconds`: Maximum elapsed time.
- `expected_exit`: Closed accepted Windows Installer result set.
- `effect_results`: One result for every applicable installer effect.
- `user_state_results`: One byte-identity result for every seeded user-owned fixture.
- `terminal`: Passed, refused-as-expected, failed, timed-out, or could-not-run.
- `complete`: True only when every required observation exists.

## CertificationReport

- `schema_version`: Exact report schema.
- `contract_digest`: SHA-256 of the contract bytes.
- `release_identity`: Certified release build identity.
- `artifact_results`: One row per primary artifact and checksum sidecar.
- `pe_results`: One row per executable surface.
- `smoke_result`: Exact packaged-binary offline native result.
- `lifecycle_results`: One row per required lifecycle case.
- `findings`: Bounded sorted public-safe failures.
- `complete`: True only when every required row is present and successful.

## State Transitions

```text
contract unavailable -> could not run
contract parsed -> static validation pending
static contract mismatch -> failed
static contract valid -> package build eligible
final artifacts assembled -> certification pending
final-byte mismatch -> failed
final bytes valid -> packaged smoke pending
packaged smoke valid -> lifecycle pending
lifecycle incomplete or residue present -> failed
all required rows reconciled -> certified bundle
certified bundle reverified -> GitHub release eligible
GitHub release published -> crate publication eligible
```
