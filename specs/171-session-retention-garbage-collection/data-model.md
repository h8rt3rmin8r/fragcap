# Data Model: S171

## Retention policy

Versioned policy distinguishes managed-history and explicit-retention. Managed defaults carry max age 30 days, max count 20 and max logical bytes 2147483648, with oldest eligible ordering. Creation/completion timestamps use exact integer epoch units. Allocation intent determines class, not path containment.

## Collection proposal

Proposal binds exact root identity, session identity, provenance, complete recognized child identities/lengths, selected scope and policy. Per-session eligibility distinguishes collectable, retained, active, recovery-required, empty and unresolved. Aggregate proposal identity includes all selected sessions and purge/include-retained intent.

## Retirement transaction

External versioned authority binds exact root path/identity, policy and selected population before deletion. Projected phases are collecting, collected, purge-pending and purged. Synchronized progress records removals and phase changes through the exact transaction object. Missing original children can be reconciled; replaced children refuse. A collected record recognizes the preserved actually empty root. Obsolete owner records retire only after content removal; owner retirement failure is reported independently and interruption retains retry authority.

## Collection report

Per-object results separate removed, failed, already absent and not attempted. Removed logical bytes sum actual removed bundle files only; owner-registry retirement outcomes are separate and exclude registry metadata bytes from content totals. Empty roots preserved/purged and retained/unresolved sessions remain independent counts. Bounds and incomplete inventory cannot become success.
