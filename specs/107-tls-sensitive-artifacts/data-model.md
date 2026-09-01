# S107 Data Model

## SessionKeyLog

Owns one protected open file, the allowlisted label set, serialization lock, written-record and written-byte counters, flush count, rejected-label count, and first failure. Debug output contains counters only.

## ClientIdentity

Owns a nonempty ordered certificate chain and exactly one zeroizing private key. It has no displayable secret representation. Provenance is the confirmed operator-supplied path pair, carried separately from bytes.

## TlsRefusal

Carries boundary, stable class, evidence token, optional TLS alert token, optional certificate-error token, whether an operator identity was configured, and pinning state. It carries no certificate body, subject, path, key, or session secret.

## SensitiveArtifactPolicy

An immutable plan value of `retain-until-explicit-cleanup`. Deletion is effective only through the targeted completed-bundle cleanup command and its confirmation boundary.

## SensitiveArtifact

One normalized relative bundle path, role, sensitivity class, required state, and lifecycle status. Secret-adjacent material includes the CA private key and TLS key log. Payload-sensitive material remains distinct from ordinary evidence.

## SensitiveAction

One bounded journal record with format version, monotonic action id, operation, normalized relative path, intent/result phase, outcome, and non-secret error token. Pending intent without a result is recoverable.

## ShareTransformation

One source identity, destination identity, ordered included artifact records, ordered omitted artifact records with reasons, final status, and completion stamp. It describes a separate copy only.

## State Transitions

```text
key log:    unrequested | requested -> protected-ready -> active -> nonempty/empty/failed -> retained/removed
identity:   absent | selected -> loaded -> validated -> configured -> dropped/zeroized
refusal:    observed -> classified -> emitted -> persisted
artifact:   declared -> protected -> written -> retained | delete-pending -> removed/already-absent/failed
journal:    created -> intent-synced -> effect -> result-synced -> replayed/complete
share copy: absent -> staging -> copied -> manifest-synced -> published | abandoned-cleaned
```
