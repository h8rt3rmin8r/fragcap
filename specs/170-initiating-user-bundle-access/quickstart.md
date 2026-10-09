# S170 controlled validation

Use synthetic files only. The owner historical evidence is outside slice execution.

```text
cargo test -p fragcap --features deep-capture --test bundle_access --locked
cargo test -p fragcap-cli --test cli_bundle --locked
cargo test -p fragcap-cli --lib --locked
cargo xtask ci
```

Required scenarios reproduce denied ordinary reads under the old owner-relative policy, then prove new and repaired bundle enumeration/read, private sidecar inheritance, atomic publication and unchanged content. Recipient mismatch, elevated helper, expired request, inaccessible custom ancestor, active lease, stale inspection, unknown object, reparse and outside-hard-link cases produce named refusals before unrelated changes. Partial repair reports actual results and a fresh-preview retry is idempotent.

A denied proven-owned session container is included in the exact preview and receives only a non-propagating traversal/enumeration grant. A sibling bundle's descriptor and contents remain unchanged. An inaccessible arbitrary profile/custom ancestor is reported without permission changes.

Run `fragcap bundle access-inspect <SYNTHETIC_BUNDLE>` to obtain its exact inspection id, then `fragcap bundle access-repair <SYNTHETIC_BUNDLE> --authorize <INSPECTION_ID>`. Both outputs distinguish written bytes from verified recipient access. Explicit different-account tests use the emitted `bundle access-authorize` request in the controlled ordinary helper context and an exact destination. Actual Windows token/filesystem outcomes are recorded separately from Unix or pure-model tests.

All console commands use verified hidden foreground launch with redirected I/O. The full gate and exact-head hosted checks are watched to completion. External review has an automatic first round and at most one manual second round, followed by owner merge handoff.
