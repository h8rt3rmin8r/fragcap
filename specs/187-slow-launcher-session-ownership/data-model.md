# Data Model: S187

## Deadline request and effective plan

- `launch`: requested optional duration, default 30 seconds, accepted range (0, 300 seconds].
- `observation`: requested optional duration, default 60 seconds, accepted range (0, 600 seconds].
- `capture_return_allowance`: separate finite budget after observation, never subtracted from the requested span.
- `route_owner_release`, `shutdown`, `cleanup`: separately finite existing budgets with preflight ceiling validation.
- Plan and terminal evidence retain both requested state (`explicit` or `default`) and effective milliseconds.

## Managed chain

- One exact cold platform root has a launch receipt and immutable canonical executable.
- Zero or more declared publisher stages have unique roles, canonical executable paths, and ordered parent roles; exactly one terminal client ends the declaration.
- A transient shim or helper may have an observed process instance and ancestry, but no declared role or control authority.
- Shutdown scope is frozen before effects and contains only exact declared canonical paths and the platform root.

## Process instance and connection

- Process instance identity combines PID, creation time, observed path, account, session, and parent generation when available.
- Process trace includes birth and exit records for retained managed descendants; source and projection omissions are counted separately.
- Each proxy connection stores its initial exact, ambiguous, or unavailable process-instance relation, with reason and observation time. A later packet-flow relation cannot overwrite an unavailable inception claim without retaining the distinction.

## Release obligation

- A declared exact owner with a retained receipt may receive bounded graceful and then forced close after fresh identity checks.
- An observed undeclared descendant is an exit obligation only.
- A missing receipt, ambiguous ancestry, live helper at deadline, or incomplete event authority yields explicit unresolved recovery state.
