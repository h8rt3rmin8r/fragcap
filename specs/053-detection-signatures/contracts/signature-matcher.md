# Contract: the generic signature matcher

The pure, data-driven matcher in `fragcap-profile`. It replaces the embedded
`CompiledRuleset`. It takes a caller-provided signature set and a filesystem
location; it never reads a database and never calls a platform API (D1, D3).

## Inputs and outputs

- A per-directory match primitive: given a directory's immediate listing (and, for
  a `pe-version-string` signature, the candidate binary), return the products
  matched there, each as a `DetectionFinding` carrying category, product, evidence,
  and fidelity.
- A full-inventory entry point: given a root, walk the bounded subtree, apply every
  applied signature, and return all findings deduplicated per (category, product),
  grouped by category, plus the unreadable-subtree list. This is what the
  `technologies` command calls.

## Behavior

- **Data-driven**: matching is a function of the provided signatures only. There are
  no per-product code branches (FR-003). Adding a signature of an implemented kind
  changes behavior with no code change (FR-004).
- **Kinds**: `filename` and `directory-shape` match over the listing;
  `pe-version-string` reads the candidate binary's PE version resource (D3);
  `binary-marker` is inert and never matches this slice.
- **Fidelity**: each finding's fidelity is derived from its signature's confidence
  (D4). A definitive local marker stamps `verified`.
- **Neutral**: a finding carries no status, risk, or gate value (D9, FR-011).
- **No silent loss**: an unreadable subtree is surfaced, distinct from an empty
  clean scan; an unreadable root is an error (as the existing detector already
  does).

## P-1 boundary

The matcher opens no process handle, reads no process memory, and reads no file
contents beyond an on-disk PE version resource for a `pe-version-string` signature
(FR-008b). It launches nothing and makes no network call.
