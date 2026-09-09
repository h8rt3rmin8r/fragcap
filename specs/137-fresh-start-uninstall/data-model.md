# Data Model: Explicit Fresh-Start Uninstall

## FreshStartScope

- `current-user`: exactly one supplied or caller-resolved roaming/local root pair.
- `all-users`: every exact eligible profile pair returned by the Windows profile authority; requires administrator status and preview identifier.

## ProfileIdentity

- Stable local identity used for sorting and reporting.
- Current-user display identity or Windows profile SID for all-users discovery.
- Exact profile base, roaming root, and local root.
- Eligibility or refusal reason.

## OwnedRoot

- Profile identity.
- Root kind: `roaming` or `local`.
- Exact canonical absolute path.
- Category set derived from the root contract.
- Root status: `absent`, `eligible`, or `refused`.

## InventoryEntry

- Root identity.
- Canonical relative path using `/` separators.
- Kind: file or directory.
- Category: catalog, local database, profiles, sessions, settings, cache, logs, captures, or other canonical fragcap state.
- Link/reparse classification.
- Observed length and modification identity used for changed-plan detection.
- Eligibility or exact refusal.

## FreshStartInventory

- Schema version `1`.
- Scope and ordered profile identities.
- Ordered roots and entries.
- Inventory identifier `fresh-start-v1:<64 lowercase hexadecimal characters>`.
- Complete only when every requested profile can be classified safely.

## CleanupItem

- Exact root and relative path.
- Operation: recovery, remove file, remove directory, retain, or refuse.
- Terminal result: removed, absent, retained, refused, or failed.
- Public-safe reason and remediation.

## CleanupReport

- Schema version `1`.
- Inventory identifier and scope.
- Ordered approved roots and categories.
- Ordered item outcomes.
- Recovery summary.
- Overall status: `complete`, `partial`, or `refused`.
- Doctor guidance when recovery authority remains.

## State Transitions

```text
requested -> inventoried -> previewed -> confirmed -> recovery -> deletion -> complete
              |              |             |            |          |
              `-> refused    `-> changed   `-> refused   `-> partial
```

No deletion occurs before `confirmed`. All-users confirmation is valid only for the unchanged inventory identifier. Session deletion is valid only after recovery reaches a safe terminal result.
