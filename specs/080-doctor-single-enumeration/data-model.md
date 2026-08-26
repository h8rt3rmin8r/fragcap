# Data Model: Doctor Single Enumeration

## Live Interface Inventory

Existing value returned by live interface enumeration.

- `interfaces`: ordered interface records already rendered in the doctor report.
- `error`: optional enumeration failure captured by the doctor probe path.

## Interface Record

Existing interface entry used to derive loopback support.

- `is_loopback`: explicit loopback evidence when the live backend reports it.
- `description`: optional description text that can carry the npcap loopback marker.
- Other existing fields remain unchanged and continue feeding the interface report.

## Loopback Support Verdict

Existing three-valued doctor fact.

- `Some(true)`: enumeration succeeded and at least one interface matched the loopback predicate.
- `Some(false)`: enumeration succeeded and no interface matched the loopback predicate.
- `None`: loopback support was not determined because enumeration did not run or failed.

## State Transitions

```text
backend unavailable -> None
wpcap unavailable -> None
enumeration failure -> None + interface error
enumeration success with loopback evidence -> Some(true)
enumeration success without loopback evidence -> Some(false)
```
