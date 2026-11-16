# Data Model: Verified Test Cleanup

## Test run owner

- **Identity**: One unique, unpredictable run token and exact canonical root.
- **Lifetime**: Created before a test child starts; active while the child runs; finalized after its exit and artifact inspection.
- **State**: Active, complete-clean, failed-with-residue, or interrupted-unresolved.
- **Invariant**: One root is never reused by another run; the runner never deletes a sibling or a product-managed native root.

## Fixture owner

- **Identity**: Unique root beneath the effective run root or independently owned direct-test temp location.
- **Contents**: Input files and output artifacts retained until assertions consume them.
- **State**: Created, in-use, close-attempted, absent, or residual.
- **Invariant**: A successful test result requires fallible close and verified absence. A failed assertion retains its original failure and names any cleanup residue.

## Residual inventory

- **Identity**: Exact canonical path, owning run, observed state, and cause.
- **Bound**: Entries within the one exact run root only; a saturated inventory reports overflow and fails instead of silently truncating.
- **Invariant**: A same-prefix path, replacement, reparse point, or unresolved native session is not admitted as a test fixture by name alone.

## Product-managed session

- **Identity**: Product session manifest, journal, registry, and recovery obligations.
- **Relationship**: May be produced by a controlled application test, but is not a fixture owned by the test runner solely because it appears beneath a temporary-looking path.
- **Invariant**: Supported product cleanup and exact recovery authority govern it independently of test scratch.
