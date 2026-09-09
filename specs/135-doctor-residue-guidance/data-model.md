# Data Model: Doctor Residue Guidance

## HumanPresentation

Presentation-only values attached to a Doctor check.

| Field | Type | Rule |
| --- | --- | --- |
| `name` | string | Short stable human label; native residue uses `native residue` |
| `detail` | string | Plain-language observed condition, ownership proof, and readiness consequence |
| `remediation` | optional string | Plain-language next action; exact cleanup guidance only when an action already exists |

This value cannot select or execute an action. JSON does not serialize it.

## NativeResourceContext

Exact non-secret machine context copied from one inventory finding.

| Field | Type | Rule |
| --- | --- | --- |
| `session_id` | string | Stable recorded or bundle-derived session identity |
| `resource_id` | string | Stable resource identity within the session |
| `kind` | string | Existing residue resource-kind vocabulary |
| `state` | string | Existing lifecycle state |
| `health` | string | Existing residue health vocabulary |
| `ownership_authority` | string | Existing observation or recovery authority identifier |
| `recovery_eligible` | boolean | Whether this check actually offers Doctor's shared cleanup action |

The context excludes bundle paths, secrets, capabilities, payloads, and cleanup implementation state.

## Check extensions

The existing Doctor `Check` retains `section`, machine `name`, machine `detail`, `status`, `scope`, `remediation`, and `action`. It gains optional `HumanPresentation` and optional `NativeResourceContext` values. Non-residue checks leave both absent and render exactly through the common path.

### Invariants

- Human presentation never changes status, scope, action, verdict, or exit code.
- A native resource context is present on every native residue check and absent elsewhere.
- `recovery_eligible` equals `Check.action.is_some()`; an active resource can retain an exact journal plan while remaining ineligible for Doctor cleanup.
- `Action::Cleanup` is offered under the same condition as before S135.
- Common JSON fields preserve their existing machine values.

## HumanReportLayout

| Variant | Selection | Shape |
| --- | --- | --- |
| `Aligned` | Selected width leaves useful fixed-column detail space | name and status share a row; detail and continuation use computed hanging indent |
| `Compact` | Narrow supported width | name and status share the heading; detail and remediation are stacked with compact indentation |

The selected width is 40 through 80 display cells. An indivisible exact token may exceed it; ordinary wrap opportunities may not. ANSI sequences have zero display width.

## State-to-diagnosis rules

| Health or state | Required human meaning |
| --- | --- |
| Healthy | Completed retained history; no cleanup required; non-blocking; terminal resource state alone does not determine live session ownership |
| Active | Current active ownership is proven; no cleanup offered |
| Stale abandoned session owner | Earlier session ended without retiring owner record; no active owner proven; Deep Capture blocked pending review and confirmed cleanup of all eligible inactive records |
| Other stale | Earlier session left cleanup incomplete; no active owner proven; blocking consequence and exact eligibility |
| Cleanup failed | A cleanup attempt failed; retained record and exact retry eligibility remain visible |
| Unknown or ambiguous | Required lifecycle or ownership fact could not be proven; no safety claim |
| Unsupported | Record cannot be interpreted by this version; no inferred cleanup safety |

## Transitions

S135 adds no lifecycle transition. Inventory discovery, confirmation, recovery planning, cleanup execution, journal reconciliation, and exit semantics remain unchanged.
