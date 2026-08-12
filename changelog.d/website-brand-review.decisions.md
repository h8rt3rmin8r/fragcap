### Decisions

Dated 2026-08-11. The 2026-08-11 landing-page and brand review (issues #39
through #43), a website-only change ahead of the v0.2.0 release.

- **Specification section 23.1 is amended to permit a value proposition.** The
  section previously held the landing page to exactly one sentence of definition
  with no capability statements, on the premise that the audience arrives already
  knowing they need a capture tool. That premise is retired: the page should still
  reach a technically competent visitor who does not yet know that attribution is
  the hard part. The amended section leads with the problem, allows a small number
  of capability statements each linking into the documentation, and retains the
  prohibitions on testimonials, feature grids, and calls to action, with the
  section 23.3 voice as the guardrail. The corresponding slice spec
  (`specs/023-docsite/spec.md`, FR-004, FR-005, SC-006, and User Story 1) is
  updated so the two do not contradict, including the getting-started ordering,
  which now names obtaining a profile between verify and capture (issue #43).

- **The disclaimer and the wordmark are wired without touching pinned CI.** The
  disclaimer is single-sourced from `README.md` by extending the existing
  `site/scripts/prebuild.mjs` render step, which already generates the glossary
  content tree; the generated module is gitignored and excluded from the
  conventions linter, matching the glossary precedent. No workflow, release
  configuration, toolchain pin, or repository-root script changed. `prebuild.mjs`
  is a site build script under `site/scripts/`, not the constitution's pinned
  repository-root `scripts/`; the extension is recorded here regardless, since it
  is a build-affecting change.
