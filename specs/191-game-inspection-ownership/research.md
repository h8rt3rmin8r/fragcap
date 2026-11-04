# Research: Exact Game Inspection Ownership

## Existing authority

`CaptureSession` already makes the S189 provisional handoff decision and retains process-instance role intervals. `CaptureProcessEvidence` retains raw start/exit events, stage transitions, launch receipt, exact managed instances, and loss. The native adapter's socket inception collector reads the Windows socket table asynchronously for reporting, but its process evidence is currently published only after packet capture returns. That timing cannot authorize pre-inspection decisions.

## Decision boundary

The proxy needs a typed decision before creating a leaf certificate for CONNECT. Its interface cannot depend on Windows types or the CLI. The facade can publish live process evidence and reuse `select_inception_socket_owner` and `bind_inception_process`, then classify the resulting exact role against the selected-game contract. The lookup must remain finite; a missing event or socket is uncertainty, not middleware proof.

## Alternatives considered

- **Infer from destination or Basic capability**: rejected because middleware and game use the same hosts and inherit the same route.
- **Use final application correlation**: rejected because it runs after protocol handling and cannot undo TLS termination.
- **Read IP Helper synchronously in the accept loop**: rejected because it can delay all accepted connections and has no finite queue semantics.
- **Inspect the first connection and reclassify later**: rejected because inspection and key logging are irreversible for already-forwarded bytes.
- **Share a bounded live evidence snapshot with a proxy-facing interface**: chosen because it preserves dependency direction and exact identity authority.

## #552 coverage gap

The S189 source has a provisional client and proven successor path. PR #554 explicitly left #552 open for its full production-path Windows regression and cleanup/recovery behavior. A deterministic chain should verify the current implementation, then any discovered source gap should be corrected in this slice. No installed private game run is required before release.

## Security and retention

Decision evidence contains stable action/reason codes, timing, and process-instance proof or uncertainty. It does not contain raw credentials, TLS keys, payload bytes, or private paths in public summaries. The current session admission and destination policy remain separate.
