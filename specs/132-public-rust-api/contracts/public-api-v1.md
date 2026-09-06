# Contract: Stable Deep Capture Rust API Version 1

## Canonical path

The supported integration boundary is `fragcap::deep_capture::api` with Cargo feature `deep-capture`. `DEEP_CAPTURE_API_VERSION` is `1`. Visibility elsewhere does not create an additional stability promise. Existing `fragcap::deep_capture::*` compatibility re-exports remain available for the current pre-1.0 line, but new consumers use the canonical path.

## Capability groups

The stable inventory covers session configuration and builders, side-effect-free preparation, immutable plans, plan-bound authorization, checked lifecycle control, explicit cancellation, adapter-set construction, target and endpoint resolution, native proxy selection, explicit trust and routing, managed launch, ordinary Capture reuse, application observations, protocol classification, calibration, compatibility facts, artifact results, recovery planning, readiness inputs required by session consumers, lifecycle events, failures, cleanup, terminal snapshots, and terminal reports.

The inventory excludes raw protocol engines, runtime leases, certificate caches, parser and serializer helpers, artifact writer implementations, journal encoders, controlled fault switches, CLI argument and presentation types, and direct `fragcap-proxy` types except for facade-owned production values intentionally re-exported under the facade contract.

## Compatibility policy

Within the current pre-1.0 completion line, patch releases preserve the names, signatures, ownership, observable semantics, feature availability, and trait requirements of version 1 stable items. Compatible additions use builders, optional capabilities, non-exhaustive variants, or new names. A removal, rename, stronger trait bound, changed construction requirement, changed cancellation boundary, or changed machine-state meaning is a breaking change and requires an API-version increment plus the appropriate crate-version action.

A correctness or security defect may require a breaking correction only when the change is explicit in a dated decision fragment, the affected behavior is identified, migration guidance is supplied, and the release version reflects the break. Wire and artifact schemas retain their own independent versions.

## Construction

New consumers construct session intent through `SessionConfigBuilder` and adapter ownership through `AdapterSetBuilder`. Builders validate required inputs and return typed errors. Existing public fields remain temporarily readable and constructible for source compatibility, but field-literal construction is not the additive evolution mechanism promised by version 1.

## Enum evolution

Status, reason, event, operation, route, protocol-classification, outcome, cleanup, artifact, fact, recovery, and lifecycle enums that may grow are non-exhaustive. Consumers include a fallback arm. A closed enum may remain exhaustive only when this contract names the fixed set as a durable protocol vocabulary.

Version 1 keeps the following sets closed and exhaustive: calibration phases are `Reachability` and `Tls`; correlation states are `Matched`, `FlowOnly`, `Ambiguous`, and `Unavailable`; bypass outcomes are `Proxied`, `Bypassed`, and `Infrastructure`; sensitive retention is `Retain`; stage transitions are `Matched` and `Exited`; trust state and mutation use the exact states exported by `fragcap-proxy`; and compatibility fact keys and protocols use the exact closed stored vocabularies documented by `fragcap-targets`. Adding to any closed set is an API-version change. All enums marked `#[non_exhaustive]`, including launch case, session mode, authorization, operation, stage, routing, classification, result, event, recovery, and lifecycle families, may grow compatibly and require a fallback arm. The API module carries a compile-fail example enforcing that consumer rule.

## Concurrency and ownership

`SessionConfig`, `SessionPlan`, `PreparedSession`, `CancellationToken`, immutable observation and result values, and the production native backend configuration are transferable when their contained standard-library values are transferable. `CancellationToken` is cloneable, `Send`, and `Sync`.

`DeepCaptureSession`, `AdapterSet`, and injected adapter trait objects are serial, uniquely owned, and thread-confined by version 1. No trait method is invoked concurrently. This is a guarantee, not an omission: consumers may use non-`Send` adapters. A future movable runner requires a new explicit API and cannot silently strengthen these trait bounds.

Leases are owned by one coordinator, invoked at most once for each terminal effect, and receive bounded cleanup attempts before terminal freeze. Dropping an unfinished session cannot promise arbitrary blocking cleanup; callers use the checked lifecycle or end-to-end runner to receive authoritative cleanup results.

## Cancellation

Cancellation is monotonic, idempotent, explicit, and cooperative. A caller may request it through any token clone. The coordinator checks before authorization and between later non-cleanup adapter calls. Once observed, it starts no later non-cleanup effect, records interruption, and still attempts every safe cleanup, fact, artifact, and terminal-report obligation.

An in-flight arbitrary adapter call cannot be preempted safely. Every blocking adapter receives a finite `Budget`, must honor it, and may share the cancellation token for earlier return. A late return remains deadline-accounted. Cancellation never erases observations already collected.

## Production native example

The shipped example uses this canonical surface and `NativeProxyAdapter` against the committed controlled loopback lab. It invokes no CLI, reaches no Internet origin, installs no certificate, requires no Npcap driver, uses no game account, inspects no target process, and reports exact cleanup. Its executable test is part of the ordinary all-target gate.
