# Data Model: Target Discovery Integrity

## AutomaticRegistrationDecision

- `candidate`: the existing `CandidateTarget` value.
- `disposition`: `Eligible` or `Refused`.
- `reason`: an exhaustive stable reason.
  - `AuthoritativePlatformIdentity`
  - `VerifiedLocalTitleEvidence`
  - `InsufficientTitleEvidence`

### Invariants

- Every produced candidate has exactly one decision.
- `eligible + refused == produced candidates`.
- Only eligible candidates reach automatic registration.
- Explicit user actions such as `targets scan` retain their existing authority and do not become automatic registration.

## AutomaticRegistrationAccount

- `produced`: candidates evaluated.
- `eligible`: candidates allowed into automatic registration.
- `refused`: candidates withheld by the precision policy.
- `registered`: eligible candidates newly inserted.
- `already_present`: eligible candidates already represented.

### Conservation

- `produced == eligible + refused`.
- `eligible == registered + already_present` after a successful registration run.

## ExcludedSubtree

- `normalized_root`: exact platform-client root normalized for separators, trailing separators, and Windows case.

### Match Rule

A path matches when it equals the root or begins with the root followed by a path-component separator. Prefix-only string matches are forbidden.

## PlatformInventory

- `client_roots`: exact platform-client roots.
- `authoritative_installs`: application anchor plus exact install root for every currently observed platform title.

This is an injected value. `fragcap-targets` does not depend on Steam.

## ReconciliationDisposition

### Removable reasons

- `PlatformClientRoot`
- `PlatformInfrastructure`
- `DuplicateAuthoritativeInstall`
- `MultiTitleAggregate`

### Preserved reasons

- `UserAuthored`
- `AnchoredAuthoritative`
- `OwnershipUnproven`
- `InstallRootAbsent`
- `LocationOnlyAmbiguous`
- `NoRemovalReason`

## ReconciliationItem

- `entry`: complete existing `TargetEntry`, including database row id and stable id.
- `reason`: one removable or preserved reason.

The complete entry is the apply-time fingerprint. No separate hash or persistent token is introduced.

## ReconciliationPlan

- `removable`: exact items eligible for deletion.
- `preserved_counts`: count per stable preserved reason.

### State Transitions

```text
current rows -> preview plan -> no confirmation -> unchanged
current rows -> preview plan -> --yes -> reload all removable rows
reload mismatch -> rollback/refusal -> unchanged
reload exact -> one transaction deletes exact rows -> committed
```

User-authored or ambiguous rows have no transition to deletion.

## CountOnlyValidationRecord

- timestamp/date (no machine id)
- produced
- eligible
- refused
- registered
- already present
- parse failed
- not a game
- containers descended
- container descent truncated
- volumes skipped
- access errors
- warnings count

No title name, application id, path, account, hostname, volume identity, or machine identifier is allowed.
