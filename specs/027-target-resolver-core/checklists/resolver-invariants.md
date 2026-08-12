# Correctness Checklist: Target Resolution Cascade -- Resolver Core

**Purpose**: Guard the subtle, correctness-critical invariants of the resolver so
they are verified rather than assumed. These map directly to required tests and to
the analyze gate.
**Created**: 2026-08-12
**Feature**: [spec.md](../spec.md)

## Precedence and total order

- [ ] CHK001 The provider precedence is a fixed order, and the highest-precedence
  available answer wins (FR-001).
- [ ] CHK002 Resolution is deterministic and independent of provider registration
  or iteration order, proven by a permutation test over the provider order (FR-004,
  SC-001).
- [ ] CHK003 A lower-precedence provider answers when every higher provider is
  silent, and is not shadowed by that silence (US1 scenario 4, edge case).
- [ ] CHK004 The fidelity tier enum has a total rank, and provider precedence never
  inverts it; the invariant is asserted, not merely intended (FR-003).

## Fidelity stamping and honesty (P-9)

- [ ] CHK005 Every resolved target carries exactly one fidelity tier and a
  provenance; no resolved target is unstamped (FR-002, SC-002).
- [ ] CHK006 No answer is stamped at a higher fidelity than its source; an observed
  answer is never labeled verified or authored (FR-007, SC-006).
- [ ] CHK007 A profile answer is stamped with the profile's own declared fidelity,
  not a fixed tier (FR-006).

## The two fidelity axes stay separate

- [ ] CHK008 Targeting fidelity (authored/verified/heuristic-unverified/observed)
  and attribution fidelity (live/retained/none) are distinct types; neither is
  renamed and neither is derived from the other (FR-010, US3 scenario 3).
- [ ] CHK009 The attribution Fidelity enum is untouched by this slice.

## No silent loss (P-4)

- [ ] CHK010 When no provider can answer, resolution returns a distinct, named
  not-resolved outcome, never a silent empty success (FR-011, SC-007).
- [ ] CHK011 The observation provider consulted before any matching process exists
  returns the not-resolved outcome, not an error (edge case).

## Placement and passivity (P-1, P-2)

- [ ] CHK012 The resolver, providers, and Target live in fragcap-profile; nothing
  is added to fragcap-core (allowlist ["bytes"]); cargo xtask deps stays green
  (FR-012).
- [ ] CHK013 The observation provider uses only image name and path from the
  process snapshot and opens no process handle; cargo xtask lint stays green
  (FR-007).
- [ ] CHK014 No new external crate is added; cargo xtask license and the dependency
  inventory are consistent (FR-012).

## Surfacing and integration

- [ ] CHK015 A loaded profile exposes its declared fidelity, provenance, and kind;
  these are no longer discarded after validation (FR-009, SC-004).
- [ ] CHK016 The stub providers (hint, engine, walker) are registered and return no
  answer, so their data slices are additive (FR-008, SC-005).
- [ ] CHK017 The run command's existing profile path flows through the resolver
  with byte-identical capture output (SC-008), verified against the committed
  goldens.
- [ ] CHK018 The spec cascade section and the four glossary entries (provider,
  target resolver, resolution cascade, target) exist and pass the docs linter
  (FR-013, P-6).

## Notes

- Every item maps to an FR/SC or a constitution principle; the analyze gate should
  find each covered by a task and a test.
- CHK002 and CHK006 are the two most likely to pass an ordinary test while being
  wrong; they need the permutation test and an explicit tier-comparison test
  respectively.
