# Data Model: Deep Capture CA Trust-State Probe

## Owned CA Identity

- `recorded_thumbprint`: normalized 40-digit uppercase SHA-1 identifier from a
  Deep Capture manifest
- `material_thumbprint`: optional normalized identifier derived from bundled CA
  material
- `manifest`: source path for diagnostics

Valid when the recorded value has exactly 40 hexadecimal digits. Repeated recorded
identities are deduplicated.

## Certificate Inventory

- `store`: `CurrentUser/Root` or `LocalMachine/Root`
- `thumbprints`: normalized identifiers actually read from certificate contexts

The inventory is complete only if both stores open and enumerate successfully.

## Trust Classification

- `Absent`: complete evidence contains no observed owned identity
- `CurrentUser`: one exact owned identity is observed in `CurrentUser/Root`
- `WrongStore`: one exact owned identity is observed in `LocalMachine/Root`
- `Mismatched`: a manifest record differs from its bundled material
- `Unknown`: evidence is incomplete, malformed, or ambiguous

An actionable wrong-store or mismatch state carries the exact observed store and
thumbprint. Unknown never carries cleanup.
