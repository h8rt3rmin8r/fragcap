<!--
Read CONTRIBUTING.md before opening. The governing rules are in
.specify/memory/constitution.md; the mechanical ones in CONVENTIONS.md.
-->

## What this changes

<!-- One or two sentences, written for someone who has not read the diff. -->

Closes #

## How it was verified

<!--
Paste what you ran and what it said. Do not assert that tests pass; show it.
An unverified success claim is worse than a known failure, because it removes
the reviewer's ability to trust anything else in this description.
-->

```text

```

## Slice

<!--
If this implements a feature, name the slice: specs/NNN-slug/. Every feature
traces to docs/fragcap-specification.md and goes through the spec-kit sequence
first. Bug fixes and documentation corrections do not need a slice; say so.
-->

## Checklist

- [ ] Constitution checked. In particular, nothing here injects, hooks, reads
      target memory, modifies traffic, or bundles npcap.
- [ ] Any process handle states its requested access rights explicitly at the
      call site, and none carry memory rights.
- [ ] Every new discard path has a named counter that is surfaced in
      statistics.
- [ ] Any new term has a glossary entry in this same change.
- [ ] `cargo fmt --all -- --check` clean.
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` clean.
- [ ] `cargo test --all --locked` passing, run in the foreground.
- [ ] A `changelog.d/` fragment is included. `CHANGELOG.md` itself is
      untouched.
- [ ] `.specify/feature.json` is not staged.
- [ ] No pinned artifact changed, or a dated decision is recorded in
      `CHANGELOG.md`.
- [ ] Text hygiene: UTF-8 without BOM, LF, no trailing whitespace, no
      em-dashes or en-dashes anywhere including comments.

## Deviations from the specification

<!--
Any divergence discovered while implementing. Record it here and in the slice;
it gets promoted to specification section 29 at the next version. Silent
divergence between the specification and the code is a defect in both. Write
"none" if there are none.
-->
