# Data Model: S167 dependency alert reconciliation

## Advisory finding

A finding has an alert number, manifest path, package name, affected resolved version, first fixed version, and severity. The eight current findings are grouped by five tracked lockfiles. Alert numbers remain GitHub-owned state and are not stored as a product schema.

## Locked inventory

A locked inventory maps package identities to exact versions, checksums or integrity values, and dependency edges. Four Cargo locks are independent spike inventories. The site pnpm lock may hold multiple versions of one package; acceptance examines all resolved instances.

## Transition

An affected inventory moves from vulnerable to fixed only when each advisory package instance satisfies its first fixed version and the owning package manager accepts the lockfile without mutation. GitHub alert state moves separately after the owner merges and GitHub scans default branch.
