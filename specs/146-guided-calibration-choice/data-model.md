# Data Model: Explicit Guided Calibration Choice

## Calibration Candidate Choice

| Field | Rule |
| --- | --- |
| `schema` | Exact `fragcap.calibration-candidate.v1` |
| `id` | `candidate-v1:` plus 64 lowercase hexadecimal BLAKE3 characters |
| `identity` | Steam app identifier or exact path identity |
| `source` | Non-empty discovery source |
| `display_name` | Candidate display name |
| `classification` | Existing closed target classification |
| `fidelity` | Existing closed fidelity tier |
| `install_root` | Optional exact install root |
| `folder_name` | Optional discovered folder name |
| `executable_hint` | Optional executable hint |
| `detection_scan` | Optional existing detection coverage token |
| `evidence` | Canonically sorted complete evidence projections |

The canonical JSON value excludes its digest. Serialization is compact and deterministic before hashing.

## Choice Set

| Field | Rule |
| --- | --- |
| `scope` | `target-registration` or `steam-client` |
| `selector` | Original user selector for registration, or `steam:<app_id>` for Steam client setup |
| `target_id` | Absent before registration; required for Steam client setup |
| `choices` | Two or more candidate projections in stable identifier order |
| `discovery_considered` | Exact current candidate count considered |
| `discovery_produced` | Exact current candidate count produced |
| `discovery_warning_count` | Exact current warning count |

No choice-set state is persisted. A rerun reconstructs it from current authority.

## Exact-Case Workflow Intent

Schema version 12 extends each S145 workflow row:

| Field | Rule |
| --- | --- |
| `selected_launch_case` | Nullable existing compatibility launch-case token; null means inferred |
| `routing_strategy` | Required existing compatibility routing token; default `child-environment` during migration |
| `address_family` | Required existing compatibility address-family token; default `ipv4` during migration |

Record version 1 remains current because this is an additive extension with exact defaults. Reads of a different record version remain refusals. Workflow updates never mutate these three fields.

## State Transitions

Candidate selection has no stored state:

```text
ambiguous current candidates -> choice required -> rerun with exact identity -> current candidate matched -> existing plan and confirmation
                                                                                   | mismatch or duplicate
                                                                                   v
                                                                                refusal
```

Workflow lifecycle remains S145's `ready`, `in-flight`, `paused`, `completed`, and `refused`. Exact-case intent is fixed at creation and consulted on every transition that rebuilds a proposal.

## Validation Invariants

- Candidate identity parses strictly and matches exactly one current canonical candidate.
- A supplied candidate is consumed exactly once before workflow creation.
- Routing strategy and address family always parse through their closed vocabularies.
- A selected launch case is cold and equals the current proposal's inferred cold case before any plan.
- Resume cannot replace workflow intent.
- Checkpoints contain progress only and cannot update intent fields.
