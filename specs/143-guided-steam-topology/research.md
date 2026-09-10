# Research: Guided Steam Client Setup

## Decision: Eligibility is an absent launch field, not a failed proposal

**Rationale**: `TargetEntry::launch_entries` already separates absence from the unresolved object written by `no` or `unsure`, the resolved array written by `yes` or observation, and historical malformed values. S143 may fill absence but must not become an implicit repair or overwrite path.

**Alternatives considered**: Offer setup for every `missing-launch-declaration` limitation, which would overwrite present malformed or unresolved evidence. Offer it for an empty array only, which gives one historical invalid encoding special silent treatment.

## Decision: Reuse bounded discovery and join by canonical Steam application identity

**Rationale**: The S133/S142 composition already reads current local Steam metadata, conserves every source outcome, and carries the first appinfo executable as a deliberately non-authoritative hint. Selecting exactly one `CandidateIdentity::SteamAppId` matching the stored positive anchor avoids a second Steam parser or hidden selector namespace.

**Alternatives considered**: Read `executable_hint` only from the stored row, which can be stale. Call the Steam crate directly from calibration, which duplicates the discovery path and loses cross-source accounting. Select by target display name, which can collide.

## Decision: Exact install-root equality gates the plan

**Rationale**: The target and fresh candidate must describe the same installed instance. Exact equality preserves the observed strings and treats relocation as drift requiring review rather than normalization that could merge distinct paths.

**Alternatives considered**: Case-fold or canonicalize both paths, which can hide changed spelling or mount authority. Accept a missing stored root, which cannot prove which install supplied the executable.

## Decision: Require an executable value that names one Windows image

**Rationale**: The stored declaration becomes a final-client matcher, not a command line. A trimmed non-empty value must end in one `.exe` image component and must not contain quotes, command placeholders, URI syntax, shell separators, or traversal components. The original candidate value remains plan-bound and unmodified; unsuitable values are refused rather than repaired.

**Alternatives considered**: Store any non-empty appinfo string, which can turn a command template into a process identity. Normalize or strip quotes, which would change the observation and violate P-9.

## Decision: Use a separate plan and event family

**Rationale**: Registration authorizes row creation, Steam client setup authorizes a field update, and Deep Capture authorizes session effects. `calibration.steam_client_plan` and `calibration.steam_client` keep those machine contracts distinguishable while using the existing emitter, checked flush, input, constant-time comparison, and canonical plan patterns.

**Alternatives considered**: Extend the registration event, which does not occur for an already registered target. Reuse `deep_capture.authorization_plan`, which would falsely imply session authorization.

## Decision: Put final race protection in the target store

**Rationale**: A CLI re-read followed by an unconditional update still has a time-of-check/time-of-use window. A `Store::author_target_client_if_unchanged` operation can start an immediate SQLite transaction, read and compare the complete current row, then update only the two permitted fields before commit. A typed `Applied`, `Changed`, or `Missing` result preserves exact outcomes.

**Alternatives considered**: Call `update_target`, which overwrites every mutable field and has no expected-row condition. Call `promote_target_launch`, whose contract accepts observed capture evidence and would mislabel an operator assertion. Encode every field in one large SQL `WHERE`, which duplicates row serialization and is harder to audit than the existing row reader inside a write transaction.

## Decision: Raise row fidelity to authored without rewriting provenance

**Rationale**: The project already treats an affirmative socket-holder answer as authored authority and runtime promotion as verified authority. S143 makes the same affirmative assertion against a Steam-proposed executable. Keeping the existing platform classification source and discovery provenance preserves where other row facts came from.

**Alternatives considered**: Stamp verified or observed, neither of which describes a human assertion. Keep heuristic fidelity, which would hide that the decisive client identity was explicitly authored. Add per-field fidelity storage, which requires a schema change disproportionate to this slice.

## Decision: Controlled fixtures model target and Steam drift separately

**Rationale**: The existing controlled discovery seam can change its candidate after the plan is emitted, while a test authorization adapter can update the stored row before returning input. Both paths exercise the production revalidation and conditional update without a real Steam install or concurrent process.

**Alternatives considered**: Mock the plan comparison directly, which misses command sequencing. Use threads against the real file store, which is slower and less deterministic while proving no additional authority.
