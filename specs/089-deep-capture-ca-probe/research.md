# Research: Deep Capture CA Trust-State Probe

## Decision 1: Ownership Comes From Manifests

Use normalized `manifest.json` `trust.thumbprint` values under fragcap-owned
session storage as the durable identity set.

**Rationale**: The MVP writes the exact SHA-1 store identifier it installs. Subject,
issuer, and friendly-name matching can collide with unrelated certificates.

**Rejected**: Search for names containing `fragcap` or `mitmproxy`. Those names are
not unique ownership proof and could authorize removal of user-managed material.

## Decision 2: Read Stores Through CryptoAPI

Enumerate current-user Root and local-machine Root with read-only Windows
CryptoAPI calls already exposed by the pinned `windows-sys` dependency.

**Rationale**: Direct enumeration returns actual certificate contexts and their
SHA-1 properties without parsing localized `certutil` output or launching a child.

**Rejected**: `certutil -store`. Its textual output is localization-sensitive and
less directly testable. Adding a certificate crate is unnecessary graph growth.

## Decision 3: Pure Classification Seam

Separate evidence collection from `classify_ca(identities, inventories)`. Tests
inject identities and inventories and never access the host trust store.

**Rationale**: The state matrix and cleanup eligibility are policy. Keeping them
pure makes absence, ambiguity, mismatch, and unrelated material reproducible.

## Decision 4: Bounded Store Scope

The supported store is current-user Root. Local-machine Root is the wrong store
that can broaden trust. This slice does not scan personal or intermediate stores.

**Rationale**: The MVP installs only a trust anchor into Root. A copy in `My` or
`CA` is certificate material but not the shipped trust state.

## Decision 5: Mismatch Precedes Store Classification

When a bundled CA certificate is still available, derive its actual thumbprint and
compare it with the manifest. A mismatch is reported before store placement.

**Rationale**: The manifest and material disagree even if one value also appears in
a store. Silently choosing either would violate P-9.

## Decision 6: Cleanup Boundary

The existing cleanup action may remove a trust entry only when the probe found an
exact manifest-backed thumbprint in a named store. File residue cleanup remains as
before. Any removal is still confirmation-gated by `doctor --fix`.

**Rationale**: An offered action must be executable and narrowly scoped. Unknown or
malformed evidence cannot authorize trust mutation.
