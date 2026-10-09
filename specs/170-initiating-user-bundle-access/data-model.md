# S170 data model

## Output recipient

Exact recipient SID, producer SID, proof kind (session-linked, ordinary-current, explicit authenticated helper or controlled equivalent), retained ordinary impersonation context and destination binding. Context handles are transient, never serialized. Production revalidation compares recipient/session/path before effects. Default ambiguity and helper refusal are terminal preflight outcomes.

## Access verification

Versioned non-secret projection of recipient SID, proof kind, directory enumeration, retained file open/read outcomes, checked population and limits. Written status and semantic completeness remain independent. Overall states are verified, unresolved and unsupported; unresolved paths retain exact named reasons.

## Access inspection

Exact root, provenance/session identity, recipient, bounded recognized population, pinned object identities, current descriptors and immutable inspection digest. The population includes at most eight necessary proven fragcap-owned parent containers, each with a non-propagating traversal/enumeration-only correction; sibling bundles and arbitrary profile/custom ancestors are excluded. CLI supplies exact owned-container authority and current owner-lease disposition. Unknown population, reparse escape, hard-link ambiguity, unsupported authority or scan overflow make the inspection non-repairable. Inspection changes no descriptors or content.

## Access repair

The exact current inspection identifier authorizes one DACL-only attempt. Per-object outcomes distinguish unchanged, applied, failed, verification-failed and not-attempted. Final verification runs under the recipient context. Retry requires fresh inspection and preserves content, timestamps used for retention, manifest semantics and recovery state. No collected-byte or cleanup authority exists in this entity.
