# Research: Guided Target Discovery and Registration

## Decision: Inspect stored selection before discovery

**Rationale**: `fragcap-targets::Selection` already distinguishes resolved, ambiguous, and no-match states. Falling back only on no-match preserves existing row, handle, name, and stable-id behavior.

**Alternatives considered**: Merge stored targets and candidates into one ranking, which lets heuristic discovery compete with curated rows. Catch rendered `CliError` text, which is brittle and loses typed ambiguity.

## Decision: Reuse the production discovery composition through an injected seam

**Rationale**: S133 established one composition for Steam and known roots. A small injected provider makes command integration tests deterministic while production still calls that exact composition.

**Alternatives considered**: Reimplement Steam lookup inside calibration, call the `targets` command, or parse its output. Each creates a second discovery path or string protocol.

## Decision: Select by exact application identifier or exact display name

**Rationale**: These are directly observed candidate fields and cover the parent issue's installed-game entry cases. Requiring uniqueness makes ambiguity visible. Display-name equality uses the same Unicode-aware lowercase fold as stored-target name resolution, so non-ASCII titles do not regress at the discovery boundary.

**Alternatives considered**: Substring/fuzzy matching, inferred handles, executable hints, folder names, or path prefixes. All can silently choose the wrong title or source.

## Decision: Bind discovery completeness in a domain-separated canonical registration plan

**Rationale**: The CLI already depends directly on `blake3`, `subtle`, and `serde_json` for S134 plan authorization. A `target-registration-v1:` prefix and sorted canonical object bind the exact candidate, conserved discovery account and warnings, and destination authority without a new dependency. This prevents a successful candidate from concealing incomplete discovery that could have missed an ambiguity.

**Alternatives considered**: A bare `--yes`, candidate index, or name echo. None binds the complete current preview. Reusing the Deep Capture plan identifier would merge distinct authorities.

## Decision: Re-discover after confirmation

**Rationale**: Installed metadata and evidence can change while the operator reviews the plan. Re-acquisition prevents confirmation of one observation from authorizing a different row.

**Alternatives considered**: Persist the in-memory preview directly, which carries a time-of-check/time-of-use gap. Re-stat only the path, which misses metadata and evidence drift.

## Decision: Identify the inserted row from candidate identity

**Rationale**: Steam candidates map to a canonical anchor and deterministic stable identifier. Path candidates map to an exact install root; after the shared operation, exactly one matching row must exist. This retains idempotency and avoids changing `register_candidate`'s public return type.

**Alternatives considered**: Predict the random path stable identifier, change the registration API to return an entry, or select the newest row. The first is impossible, the second broadens the facade contract, and the third is race-prone.

## Decision: Treat registration consent and session authorization as sequential inputs

**Rationale**: `DeepCaptureAuthorizationInput` already provides bounded complete-line input and can be invoked more than once. Structured orchestration can respond to each emitted plan in sequence; interactive mode receives two distinct prompts.

**Alternatives considered**: Let registration approval authorize calibration, which broadens consent. Stop after registration, which fails the single-command handoff goal. Add a second stdin flag and parser, which is redundant.
