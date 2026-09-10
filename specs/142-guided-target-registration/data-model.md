# Data Model: Guided Target Discovery and Registration

S142 adds transient orchestration values and two additive CLI events. It does not change the target-store, compatibility-fact, bundle, manifest, or capture schemas.

## DiscoverySelection

| Field | Meaning | Rule |
| --- | --- | --- |
| stored | Resolved, ambiguous, or no match | Resolved and ambiguous are terminal before discovery |
| selector | Positional/`--target` token or `--id` | `--id` never selects an unregistered candidate |
| candidates | Exact discovered matches | Match by exact Steam app id or Unicode-aware case-insensitive display name only |
| discovery account | Conserved S133 counts and warnings | Plan-bound and visible before confirmation, including on success |

## RegistrationPlan

| Field | Meaning | Rule |
| --- | --- | --- |
| schema | `fragcap.target-registration-plan.v1` | Exact version string |
| operation | `register-candidate-v1` | Names the shared persistence contract |
| local store | Effective absolute destination path | Included in canonical authority |
| candidate | Complete canonical candidate projection | Includes identity, name, source, fidelity, classification, evidence, scan state, install root, folder name, and executable hint |
| discovery | Conserved account plus warning list | Complete visibility and plan authority before confirmation |
| predicted stable id | Deterministic anchored id when available | Null for an unregistered path candidate |
| plan id | `target-registration-v1:` plus BLAKE3 digest | Domain-separated over compact canonical JSON |

## RegistrationOutcome

| Field | Meaning | Rule |
| --- | --- | --- |
| plan id | Registration plan being decided | Always present after a plan is emitted |
| status | `declined`, `closed`, `invalid`, `interrupted`, `drifted`, `registered`, `already-present`, or `failed` | Independent from calibration status |
| reason | Stable machine reason | Display prose is not authority |
| target id | Resulting stable identifier | Present only for registered or exact already-present outcomes |
| continued | Whether S139-S141 evaluation began | Never implies a session started |

## State Transitions

```text
parsed -> stored-selection
  -> stored-resolved -> existing-guided-calibration
  -> stored-ambiguous -> refused
  -> stored-miss -> discovery
      -> no-match
      -> discovered-ambiguous
      -> exact-candidate -> registration-plan-emitted
          -> declined-or-invalid
          -> confirmed -> rediscovery
              -> drifted
              -> unchanged -> shared-registration
                  -> durable-target-resolved -> existing-guided-calibration
```

No transition before shared registration inserts a target row. No registration transition authorizes a later session effect.

## Validation Rules

- Canonical projections preserve every candidate field and sort evidence deterministically.
- Every discovery account must conserve before a plan can be emitted; its warnings are sorted deterministically and remain plan-bound.
- The plan binds the effective store path rather than the raw optional flag.
- Interactive affirmative input is a complete case-insensitive `y` or `yes` line; every other response declines.
- Structured input is exact plan id plus one newline and constant-time compared.
- Revalidation recomputes the plan from a new discovery result and requires equality.
- Steam result lookup uses the canonical anchor. Path result lookup uses exact install root and requires one result.
- Durable handoff invokes the existing guided implementation without mutating its proposal, authorization, or one-session behavior.
