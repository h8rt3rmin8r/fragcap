# Data Model: Warm-To-Cold Restart

## Warm Restart Plan

| Field | Meaning |
| --- | --- |
| target | Selected stored target identity shown to the operator |
| warm_case | Observed direct, platform, publisher-launcher-only, or publisher-chain warm class |
| images | Deduplicated declared image names that must all become absent |
| deadline | Effective finite wait bound |
| identity | Always `image-name-observation-only` |
| process_control | Always `none` |

## State Transitions

`not-requested -> warm-observed -> wait-authorized -> cold-observed -> reprepared -> launch-authorized -> ordinary-session`

Terminal pre-effect outcomes are `not-warm`, `wait-declined`, `timeout`, `inventory-failed`, `changed-state`, `reprepare-failed`, and `launch-declined`.

## Invariants

- No transition invokes process control.
- `cold-observed` requires every declared image absent in one complete snapshot.
- `reprepared` contains no value retained from the warm launch plan except audit context.
- Session effects require `launch-authorized` for the exact re-prepared plan.
