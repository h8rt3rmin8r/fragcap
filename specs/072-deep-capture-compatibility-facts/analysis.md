# Analysis: Deep Capture compatibility facts

## Review-sensitive points

- Compatibility facts are not inferred from platform metadata. The migration creates an empty table and never backfills.
- Proxy backend identity is structured because backend behavior is part of the observation.
- Final owner executable is structured because role alone can collapse distinct executables.
- Final-owner handoff is not a launch case. The actual launch path remains queryable.
- No real local title names, account details, filesystem paths, endpoints, or screenshots are included in this slice's public artifacts.

## Schema impact

Schema version advances from 8 to 9. This is safe before merge because no released build has written v9 yet. Review fixes are applied to the v9 table definition directly rather than adding a v10 migration inside the same PR.

## Verification strategy

- Model tests prove enum parsing and invalid value rejection.
- Store tests prove migration, round-trip fidelity, key/value CHECK enforcement, and cascade delete behavior.
- Full workspace tests prove existing target-store behavior still passes.
- `cargo xtask deps` proves the dependency graph is unchanged.
- `cargo xtask spec` proves the changelog fragment carries spec impact.
