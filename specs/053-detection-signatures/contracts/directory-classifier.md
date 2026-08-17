# Contract: SignatureClassifier (the real DirectoryClassifier)

The `fragcap-targets` implementation of the S052 `DirectoryClassifier` seam,
backed by the signature matcher. Replaces the S052 placeholder
`KnownRootChildIsGame`.

## Interface

Implements the S052 `DirectoryClassifier` trait:
`classify(&self, dir) -> ClassifierVerdict`, unchanged in shape. Constructed from a
loaded `SignatureSet` (from `Store::load_signatures`), so it holds the applied
signatures and delegates matching to the `fragcap-profile` matcher (D1, D8).

## Behavior

- **Hit**: the directory's shape matches an engine signature. Return
  `Hit { classification: Game }`, carrying the detected engine at `verified`
  fidelity. The scan emits one candidate and stops descending into the subtree
  (S052 FR-015 / this slice FR-007). Any anti-cheat or DRM matched on the same
  directory is recorded as additional neutral evidence on the candidate.
- **Miss**: no engine signature matched. Return `Miss`; the directory is
  considered-not-a-game in the S052 discovery account.
- The classifier never enumerates a directory's executables first and then asks
  whether each is a game (S052 FR-009); it tests the directory's shape and decides.

## Wiring

- Every S052 `TargetSource` scan phase runs through this classifier with no separate
  user action (FR-006). The known-roots walk, the user-pointed directory source, and
  the Steam source all classify through the one seam.
- The discovery account stays conserved across every source test (FR-013, SC-007).

## Fidelity ordering

Where a candidate also carries a remote catalog engine attribution
(`heuristic-unverified`), the local `verified` engine from the classifier is the one
presented (FR-008, FR-009, P-9).

## Neutral evidence

Nothing on the verdict or the candidate frames a detected anti-cheat or DRM product
as a reason not to capture; a title with no online multiplayer mode is still a
capturable candidate (FR-011, FR-012, D9).
