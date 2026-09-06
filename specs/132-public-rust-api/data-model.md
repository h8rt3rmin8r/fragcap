# Data Model: Stable Public Rust API

## StableApiSurface

- `version`: Exact `DEEP_CAPTURE_API_VERSION` integer.
- `module`: Canonical `fragcap::deep_capture::api` path.
- `exports`: Reviewed stable type, trait, function, and constant names grouped by capability.
- `legacy_exports`: Existing top-level compatibility paths excluded from the curated promise.
- `feature`: Required Cargo feature and availability rules.
- `compatibility`: Allowed additive and breaking changes.

## SessionConfigBuilder

- `target`: Required stored target handle or identifier.
- `bundle`: Required session bundle destination.
- `mode`: Ordinary Deep Capture or one calibration phase.
- `launch_case`: Optional exact managed launch shape.
- `calibration_protocol`: Optional exact calibration protocol.
- `controlled`: Whether the committed local controlled target is selected.
- `trust_ca`: Explicit current-user trust authorization.
- `artifact_requests`: HAR, key-log, payload, and sensitive-retention intent.
- `proxy_bypass`: Ordered operator-owned bypass rules.
- `client_identity`: Whether an explicit upstream identity is bound.
- `deadlines`: Effective finite stage deadlines.

Validation produces one immutable `SessionConfig` or a typed construction refusal before preflight.

## AdapterSetBuilder

- `targets`: Required side-effect-free target resolution.
- `endpoints`: Required exact loopback allocation.
- `clock`: Required wall and monotonic time source.
- `identifiers`: Required plan and session identity source.
- `proxy`: Required proxy backend, with `NativeProxyAdapter` as the production entry.
- `trust`: Required explicit trust manager.
- `routing`: Required target-scoped routing adapter.
- `launch`: Required managed launch adapter.
- `capture`: Required ordinary Capture runner.
- `facts`: Required append-only compatibility repository.
- `artifacts`: Required bundle authority.
- `events`: Required typed event sink.
- `cancellation`: Optional caller-owned cooperative cancellation handle supplied separately to the session.

Build fails with a typed missing-capability result and never installs the hidden boundary fault controller.

## CancellationToken

- `requested`: Monotonic false-to-true atomic state shared by cloned handles.
- `request()`: Idempotently requests cancellation.
- `is_requested()`: Reads the current state without blocking.
- `ownership`: Caller and adapters may retain clones; the session owns one clone.
- `effect`: Prevents later non-cleanup lifecycle effects when observed between calls and produces an interrupted terminal outcome.
- `limit`: Does not preempt an in-flight arbitrary adapter call; `Budget` remains mandatory for that call.

## ApiCompatibilityPolicy

- `stable`: Items named by the curated inventory.
- `additive`: Builder methods, optional capabilities, non-exhaustive variants, and new implementation helpers outside the inventory.
- `breaking`: Removal, rename, signature change, stronger trait bound, changed ownership, changed cancellation semantics, or changed machine-state meaning of a stable item.
- `exception`: A correctness or security defect may require a break only with an explicit changelog decision and crate-version action.
- `wire_independence`: Rust API version changes do not silently change artifact schemas or compatibility-fact vocabularies.

## PublicApiContractReport

- `api_version`: Observed constant.
- `stable_exports`: Sorted exact names compiled by the external consumer.
- `cli_capabilities`: Sorted command-consumed capability groups.
- `non_exhaustive_types`: Reviewed evolvable enum set.
- `construction_paths`: Required builders and constructors.
- `concurrency_assertions`: Positive and negative type-property checks.
- `example_result`: Controlled native run outcome and cleanup state.
- `complete`: True only when every required row passes.

## State Transitions

```text
configuration builder incomplete -> typed build refusal
configuration built -> side-effect-free preflight eligible
prepared session -> authorization or cancellation
cancellation before authorization -> no-effect interrupted terminal path
authorization accepted -> serial lifecycle effects
cancellation between effects -> later non-cleanup effects withheld
in-flight bounded adapter -> cooperative budget or cancellation return
stopped or failed -> complete safe cleanup and finalization
terminal report returned -> no further session reuse
stable inventory changed -> compatibility review required
```
