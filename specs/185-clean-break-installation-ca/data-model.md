# Data model: S185

## Installation identity

- `scheme = 1`: distinguishes the new installation scheme from pre-S185 session identities.
- `generation`: monotonically incremented only by explicit administrative rotation.
- `provenance`: machine installation identifier and product origin; never a user account or session.
- `certificate_der`: exact public root bytes.
- `sha256_fingerprint`, `sha1_thumbprint`: recomputed from DER on every read.
- `not_before`, `not_after`: certificate validity bounds.
- `key_path`: fixed name in the protected PKI directory; no caller-supplied path.

## Health state

`ready`, `not-provisioned`, `missing-cert`, `missing-key`, `mismatch`, `expired`, or `unreadable`. Read-only startup distinguishes resource presence and bytes, while privileged Deep Capture preflight also validates ACL, decrypts, signs, and verifies. A startup `ready` means public identity is ready; key usability remains unverified until privileged proof.

## Cutover state

An installation transaction records a bounded inventory of exact canonical profile roots, session manifests, trust-store entries identified by their DER and thumbprint, and PKI files. Every removal has `observed`, `removed`, or `refused` status and readback. A nonempty refused set keeps the new Deep Capture gate closed. No pre-S185 state is translated into the new schema.

## Session reference

Plans and manifests carry `installation_ca_sha256`, `installation_ca_generation`, and `installation_ca_store=LocalMachine/Root`. They never own the root trust lease. Session-specific routing, proxy, capability, artifacts, and authorization remain independent.
