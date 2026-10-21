# Data Model: Managed Shutdown and Doctor History

## Owned launch receipt

- Session and accepted plan identity.
- Exact created root process object and creation instant.
- Canonical image path, user identity, Windows session, and declared launch role.
- Bounded descendant instances each linked by creation-time ancestry and role.
- Live minimal-rights process handle, never serialized into a bundle.

## Shutdown decision

- One monotonic origin and 120-second ceiling.
- Per-owner state: observed, identity-verified, graceful-requested, exited, grace-expired, force-requested, force-failed, identity-refused, or unresolved.
- Each transition has a timestamp, owner identity, bounded reason, and route/proxy order.
- Route released only when every exact route-owning process is verified exited.

## Doctor historical projection

- Terminal manifest state and cleanup status remain immutable historical facts.
- `cleanup.json` identifies failed resource and reason when present and valid.
- Resource journal describes current tangible resources and exact recoverable actions.
- Human and JSON findings carry both authorities without converting a past failure into a current blocker when the owner has exited.
