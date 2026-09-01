# S107 Research

## Client-facing TLS key logs

**Decision**: Implement a session-owned rustls `KeyLog` that receives an already protected open file, allowlists documented NSS labels, serializes concurrent records, flushes each complete line, and exposes counters and the first write failure. Attach it only to client-facing `ServerConfig` values.

**Rationale**: rustls `KeyLogFile` reads process-global state, silently disables failures, and does not live-flush. A dedicated owner provides explicit authorization and exact outcomes while making upstream secret inclusion structurally impossible.

## Operator-supplied client identity

**Decision**: Read only the two explicit operator paths in the facade, zeroize owned input buffers, and parse the certificate chain plus exactly one PKCS#1, PKCS#8, or SEC1 key through `rustls-pki-types`. Configure the existing upstream client builder with `with_client_auth_cert`; preserve the no-identity builder.

**Rationale**: Existing dependencies already parse the supported PEM and DER forms and validate key compatibility. No target or certificate-store discovery is needed or permitted.

## TLS refusal evidence

**Decision**: Downcast Tokio-rustls I/O causes to `rustls::Error` and map variants and alerts to stable structured categories. Preserve exact non-secret alert or certificate subtypes. Treat client-facing `BadCertificateHashValue` as conclusive pinning evidence and every ambiguous trust refusal as unknown.

**Rationale**: Parsing error display text is unstable. Most peer closures cannot distinguish pinning, ordinary trust refusal, or application behavior, so claiming pinning would violate honest reporting.

## Artifact preparation boundary

**Decision**: Add an authorized artifact preparation effect immediately after plan emission and before proxy startup. Establish the protected bundle root and sensitive journal there.

**Rationale**: The current native application writer opens plaintext during proxy start, before CLI finalization. Protection performed only during finalization is necessarily late.

## Windows access control

**Decision**: Create Windows bundle directories and strict files with protected owner-and-SYSTEM discretionary access control using native security descriptors. Supply security attributes at file creation where sensitive bytes are written. Portable tests use owner-only permissions as the non-Windows model.

**Rationale**: Applying an ACL after ordinary file creation exposes a race. Native creation-time security avoids helpers and retains an auditable current-user boundary.

## Sensitive action journal and cleanup

**Decision**: Use a bounded `.sensitive-actions.jsonl` journal with a header and append, flush, sync intent/result records over normalized relative paths. Replay only pending sensitive deletes and abandoned share staging. Completed-bundle cleanup removes only manifest-declared sensitive roles and is idempotent.

**Rationale**: #322 needs crash-safe sensitive operations, but general proxy, trust, launch, and capture recovery belongs to #320 and remains open.

## Share preparation

**Decision**: Copy validated regular artifacts into a protected sibling staging directory, omit secret-adjacent roles, write an exhaustive transformation manifest, sync, then atomically rename to an absent destination. Never rewrite the source.

**Rationale**: Transform-on-copy preserves evidentiary originals. Staging prevents a partial copy from appearing complete.

## Doctor behavior

**Decision**: Restrict automatic doctor cleanup to unfinished manifests and pending sensitive journal residue. Completed retained bundles require the targeted confirmed cleanup command.

**Rationale**: The current broad cleanup candidate scan can delete retained evidence from completed bundles, which conflicts with explicit retention.

## Platform workflow coverage

**Decision**: Remove `paths` filters from the platform workflow and add a repository lint that refuses future `paths` or `paths-ignore` filters on that whole-workspace job.

**Rationale**: The job executes `cargo test --workspace`; any narrower static input list can drift as tests and workspace members change. Main-only push scope still prevents branch and pull-request duplication.

## Dependency Impact

No new registry package or lockfile package is required. Existing rustls, rustls-pki-types, Tokio-rustls, ring, zeroize, serde_json, and windows-sys APIs cover the design. Additional windows-sys feature flags are additive on the exact existing 0.36 pin.
