# Contract: Platform Launch Ownership

## Preparation

```text
PlatformAdapter::prepare(stored target, resolved profile)
    -> PlatformLaunchPlan
    | PlatformPreparationRefusal
```

Preparation is side-effect-free. Success guarantees:

- one supported platform token;
- one exact canonical platform executable and root;
- one non-empty application dispatch;
- one validated exact platform-rooted Capture profile;
- one finite terminal acquisition deadline;
- no running same-named platform image was observed.

Refusals include `platform-unsupported`, `platform-warm`, `platform-install-missing`, `platform-executable-invalid`, `platform-application-missing`, `platform-profile-invalid`, and `platform-identity-uncertain`.

## Root Start

```text
PlatformLaunchPlan::start_root(environment)
    -> PlatformLaunchReceipt(root_process_id, dispatch=pending)
    | PlatformStartFailure
```

The root start creates only the retained executable, in the retained working directory, with retained arguments and the session route environment. It invokes no shell and performs no title dispatch.

## Ownership Transition

```text
Capture process event
    + exact platform path
    + created root identity
    -> platform role bound
    -> dispatch authorized
```

Image-name equality alone cannot authorize dispatch. A warm startup-snapshot process cannot satisfy this transition.

## Title Dispatch

```text
PlatformLaunchPlan::dispatch(receipt with observed root)
    -> receipt(dispatch=issued)
    | PlatformDispatchFailure
```

Steam dispatch invokes the retained `steam.exe` directly with the selected application identifier. It does not re-read the registry, resolve another executable, invoke a shell, or mutate routing.

Dispatch is at-most-once. Repeated platform events cannot issue it again.

## Terminal Ownership

```text
exact client process event
    + creation-time ancestry beneath bound platform role
    -> terminal client acquired
```

A same-image process outside that ancestry yields `escaped-descendant`. A competing qualifying process yields `ambiguous-stage-match`. Neither acquires the terminal role.

## Evidence

```text
proxy observation + flow ownership + platform ancestry
    -> routing outcome
    -> independent propagation outcome
```

`proxy-routing = reached-client` requires final-client correlation. `proxy-propagation = confirmed` additionally requires that client to descend from the exact owned platform root in the same session. Platform-only and launcher-only traffic cannot satisfy either positive client claim.

## Compatibility

- Ordinary Capture keeps its existing Steam protocol-handler path.
- Direct and publisher Deep Capture plans keep their existing preparation and execution.
- Unsupported platforms refuse before effects.
- No existing artifact reader or database schema is removed.
