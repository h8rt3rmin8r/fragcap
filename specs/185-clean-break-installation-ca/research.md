# Research and decisions: S185

## R1. Trust scope and time

Decision: one machine-unique self-signed CA with finite validity lives in `LocalMachine/Root`. Its signer is reused for all Deep Capture sessions. Installation is the disclosed trust operation; later session authorization still names target and routing. Explicit repair rotates only after exact identity checks. This resolves the contradiction in prior session-only P-1 guidance under issues #528 and #529.

Rejected: leaving a fresh session CA trusted after cleanup. That would accumulate roots and still cause a new trust event each run. Rejected: embedding one shared product CA in the release, because every installation would share the same signer.

## R2. Protected signer

Decision: `%ProgramData%\fragcap\pki` is the sole signer home, with an exact non-inherited SYSTEM/Administrators ACL. Machine DPAPI encrypts the key; file and parent access rules are checked before decryption. `LocalMachine/Root` holds only the public DER. The full DER, SHA-256 fingerprint, SHA-1 thumbprint, validity, generation, and provenance form a durable public identity. Signing verification is privileged and separate from read-only startup status.

Rejected: relying on DPAPI machine scope without ACLs, because anyone able to obtain the blob can decrypt it on the same machine. Rejected: placing key material in the install tree or a session bundle.

## R3. Clean cutover

Decision: the first S185 install inventories canonical pre-S185 Fragcap roots and exact legacy trust identities from their manifests, removes that state and verifies absence before enabling the new scheme. It does not migrate or preserve old state for compatibility. Unknown or ambiguous certificates are never guessed to be Fragcap-owned; they cause a visible incomplete-cutover result instead of deletion. Repeated S185 installation detects the generation marker and retains the current CA and current state.

## R4. MSI and portable boundary

Decision: WiX deferred elevated custom actions call the product's `ca install` backend after installing files. A new CA writes an exact creation receipt; the MSI commit action disarms it, and rollback requires that uncommitted receipt plus the exact identity. Repair, reinstall, and upgrade reuse a healthy root without arming rollback, even when ProductCode repeats. Provision failure is an installer error with a logged reason. Portable extraction cannot run setup, so an explicit elevated `fragcap ca install` provides the same backend. Ordinary commands check read-only health and warn; Deep Capture refuses before effects.

## R5. Session continuity

Decision: remove session CA generation and trust mutation from production. The prepared authority is loaded from the installed signer and bound by public fingerprint to the plan. HTTP CONNECT and QUIC/H3 already consume the same prepared authority interface. Session cleanup reports installation trust as retained and never removes it. Route-owner and proxy failures remain independent partial results. This fixes the #528 trust loss without claiming to fix `tls-tunnel-authority-mismatch`.
