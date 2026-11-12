# Acceptance Design Checklist: Installed Direct Route

**Purpose**: Check that S205 defines truthful, executable acceptance before implementation.
**Created**: 2026-11-12
**Feature**: [spec.md](../spec.md)

## Route and package

- [x] The candidate route requires client-visible two-way bytes rather than CONNECT success.
- [x] MSI, ZIP, IPv4, IPv6, required route stream, separate library diagnostics states, unrelated sentinels, refusal, cancellation, and cleanup are named.
- [x] Published v0.11.0 is a comparison baseline, not an assumed reproducible failure.
- [x] Installed default native paths and package-harness paths have separate evidence authority.

## Correlation

- [x] Physical original-packet presence and absence require passive Npcap evidence.
- [x] TCP and UDP positives require exact route and client proof.
- [x] Reuse, competing owner, helper identity, incomplete history, refusal, and no payload have named negative or independent outcomes.
- [x] Compatibility, verdict, stored fact, and subsequent admission are one consistency contract.

## Diagnostics and access

- [x] Complete and partial reports, queue and writer gaps, source preservation, and cleanup are named.
- [x] Manifest, Doctor, retention, export, and recipient access are included.
- [x] Human and JSON summary parity is measurable.
- [x] Private route identity and unrelated payload remain outside shareable output.

## Notes

These checkboxes validate the specification's coverage. Passing implementation evidence is tracked in tasks.md and the slice evidence record.
