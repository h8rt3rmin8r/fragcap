# Data Model: S181

No product schema or runtime entity changes.

## Finding inventory

Each finding records ecosystem, package, advisory, vulnerable range, patched resolution, owning manifest/lockfile, and observed state. Multiple manifest alerts remain individually reconciled.

## Graph inventory

The site graph is `site/pnpm-lock.yaml`. Rust roots are the repository, `fuzz/`, `performance/native-proxy/`, `spikes/native-proxy/`, `spikes/native-proxy/audit/`, `spikes/http-mitm-proxy/`, and `spikes/http-mitm-proxy/audit/`. Root policy digests change only with an actual root graph change.

## Delivery evidence

Evidence binds source commit, PR, review, main commit, workflow/job, terminal result, deployment, cold-cache execution, and final alert/PR counts. Source qualification and merged observations remain distinct.
