# Contract: Container Discovery Accounting

Every considered directory still lands in exactly one named outcome.

For a `Container` verdict:

- If `depth + 1 < MAX_DESCENT`, increment `container_descended` and enumerate its immediate children.
- Otherwise, increment `container_descent_truncated`, do not enumerate beneath it, and append one warning naming the directory and stating that descendants may remain undiscovered.

Both fields are included in `DiscoveryAccount::is_conserved`, merged by `discover_all`, and rendered by the CLI account summary. A warning describes the consequence but is not an additional account outcome.

The account never estimates the number of unseen descendants. It counts the observed container at which coverage stopped.
