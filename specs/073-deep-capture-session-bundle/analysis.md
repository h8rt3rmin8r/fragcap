# Analysis: Deep Capture session bundle

## Review-sensitive points

- The design is intentionally pre-implementation. It defines contracts and does not add writers, CLI flags, proxy code, or dependencies.
- `.fcapng` remains pcapng-compatible packet truth. Sidecars carry decrypted application data and analyzer aids.
- HAR is mode-independent and tied to observable HTTP semantics.
- TLS key logs are sensitive analyzer aids and are not produced silently.
- The manifest gives issue #218 stable cleanup targets and gives issue #219 a bundle contract for the MVP.

## Relationship to open issues

- #216 is resolved by this slice.
- #218 should consume the manifest, cleanup resource list, sensitivity labels, and omission rules.
- #219 should wait for #218 after this slice, because the MVP still needs doctor readiness and cleanup checks.
- #220 can generate user-facing supported-traffic documentation from the artifact authority rules and compatibility fact model.

## Verification strategy

- Markdown and specification gates prove formatting and spec-impact consistency.
- The example bundle contract is the implementation target for future serializer tests.
- `cargo xtask deps` proves the design PR added no dependency.
