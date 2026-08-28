# Data Model: Deep Capture Compatibility Bootstrap

## CalibrationInvocation

The parsed optional calibration extension to `DeepCaptureArgs`.

- `phase`: `reachability` or `tls`.
- `declared_launch_case`: one existing `CompatibilityLaunchCase` token.
- `preconfirmed`: whether `--yes` was supplied.
- Existing target, capture, bundle, proxy, trust, HAR, and key-log fields remain authoritative.

Validation requires both calibration fields together, keeps `--launch` mandatory, rejects unsupported launch cases before mutation, rejects trust and decrypted-output flags during reachability, and requires explicit trust intent for TLS.

## CalibrationPlan

The immutable, side-effect-free preview emitted before confirmation.

- Target id, handle, and display name.
- Calibration phase.
- Declared and observed launch case.
- Backend name and known version state.
- Loopback-only proxy mode and scoped environment variables.
- Bundle destination.
- Launch, observation, proxy-shutdown, and cleanup deadlines.
- Planned fact families.
- Trust action, `none` for reachability.
- Cleanup obligations and explicit non-actions.

States are `ready`, `confirmed`, or `declined`. Only `confirmed` may enter execution.

## CalibrationObservation

One direct observation used by classification and fact mapping.

- Observation time.
- Proxy connection identity where present.
- Packet flow identity and process attribution where present.
- Process image, role, and ancestry/handoff evidence where known.
- Protocol and inspectability.
- Explicit backend trust diagnostic where available.
- Self-reported proxy environment evidence for the controlled target only.

Absence is not represented as an affirmative observation.

## CalibrationOutcome

The terminal result of one phase, distinct from compatibility facts.

Reachability values include `reached-client`, `launcher-only`, `escaped-tree`, `proxy-not-reached`, `no-relevant-traffic`, `inconclusive`, `unsupported-protocol`, `interrupted`, and `failed`.

TLS values include `local-ca-accepted`, `certificate-pinned`, `unknown-trust`, `metadata-only`, `unsupported-protocol`, `proxy-not-reached`, `interrupted`, and `failed`.

The outcome also carries a reason, timestamps, and whether observation, fact persistence, bundle finalization, and cleanup completed.

## PendingCompatibilityFact

One validated observed fact awaiting append to the existing store.

- Target id.
- Fact key and value.
- Launch case.
- `observed-run` provenance.
- One run observation timestamp and fragcap version.
- Backend name, version, and `launch-scoped-env` mode.
- Final owner executable and handoff only when observed.
- Fresh stale marker, initially false.
- Scrubbed note naming calibration phase and evidence kind.

Each insert produces a separate `performed` or `failed` write result. Repeated and conflicting history is retained.

## CalibrationResourceLedger

Accounts for every planned effect and artifact.

- Proxy process.
- Loopback listener.
- Managed launch.
- CA material.
- Current-user trust entry.
- Session artifacts.
- Compatibility fact writes.
- Cleanup report.
- Final manifest.

Each entry ends as `performed`, `skipped`, `failed`, or `not-applicable`, with a reason. The aggregate never replaces individual results.

## Existing Entities Reused

- `TargetEntry` and existing selector resolution remain the target authority.
- `CompatibilityFact` and `deep_capture_facts` remain durable evidence authority.
- `DeepCaptureSession`, `ProxyBackend`, `RunningProxy`, and `TrustManager` remain execution machinery.
- `Observation`, `FlowRegistry`, and packet attribution remain correlation inputs.
- `manifest.json`, `compatibility.json`, and `cleanup.json` remain bundle authorities.
