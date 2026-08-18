# Implementation Plan: Landing page and getting-started rewrite

**Branch**: `057-landing-getting-started` | **Date**: 2026-08-18 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/057-landing-getting-started/spec.md`

## Summary

Converge fragcap's public site and first-run documentation onto the command
surface that shipped in S054/S055, and reach a first-time visitor with a strong
opening. Rewrite the landing page (settled opening copy, a real `fragcap targets`
worked example, the dependency diagram, one call to action), rewrite getting
started to end at a completed capture using `fragcap targets` then `fragcap
capture <n>` (folding the getting-started QA batch #130-#135 and the docs half of
#133), and purge every retired verb/command/selector/directory/slug across the
site (rewriting the CLI reference to the real grammar, fixing the capture-modes
guide, and retiring the two obsolete profile-file pages). One small companion code
change removes the leftover `profile dir` row and `Profiles` section from `fragcap
doctor` so the getting-started sample is both faithful and free of the retired
directory. IGDB enrichment and its credential walkthrough are deferred to a
dedicated slice (no plumbing exists; documenting it would violate P-11).

## Technical Context

**Language/Version**: Rust 1.82 MSRV (workspace pins 1.96 toolchain; local GNU
toolchain `1.96.0-x86_64-pc-windows-gnu` for build/test here); site is
TypeScript/React (Next.js + Fumadocs) built with pnpm.

**Primary Dependencies**: no new dependency in any crate; no `Cargo.lock` change.
The site build (pnpm) and the documentation linter (`scripts/lint-docs.sh`) are
unchanged tooling. The doctor change is internal to `fragcap-cli`.

**Storage**: N/A (documentation slice; the doctor change removes reporting of a
directory count, adds no storage).

**Testing**: `cargo test -p fragcap-cli` (doctor unit tests), `cargo xtask ci`
(fmt, clippy, workspace tests, lint, deps, license, docs check), `cargo xtask docs
check` (glossary/P-6 linter), site build via `cargo xtask docs build` (pnpm,
optional locally). Local build/test uses the GNU toolchain
(`cargo +1.96.0-x86_64-pc-windows-gnu ...`); CI runs the MSVC gate.

**Target Platform**: Windows (the tool); the site is static-exported to GitHub
Pages.

**Project Type**: documentation + CLI (a Rust workspace with a Next.js docs site
under `site/`).

**Performance Goals**: N/A.

**Constraints**: UTF-8 without BOM, LF line endings, no em-dashes or en-dashes
anywhere (including code comments). `scripts/**` and `.github/workflows/**` are
pinned artifacts and are not touched by this slice. The documentation linter scans
`docs/glossary/` and canonical `README.md`/`docs/*.md`, not the site MDX, so the
retired-token criterion is verified by grep and the site build.

**Scale/Scope**: ~8 site files changed (2 rewritten, 2 deleted, 4 edited), 2
`app/(home)` pages edited, ~1 crate touched (`fragcap-cli` doctor: checks.rs,
mod.rs, probe.rs, paths.rs, tests), spec section 26.3 updated via `cargo xtask
spec`, one changelog fragment added.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **P-1 (technique denylist / observe-only)**: Not engaged. No capture,
  attribution, process-handle, injection, or hooking code is added or changed. The
  doctor change only removes reporting rows. PASS.
- **P-4 (every discard counted)**: Not engaged (no capture path change). The
  removed doctor rows counted a profile directory, not packets. PASS.
- **P-6 (a new term gets a glossary entry in the same change)**: The rewrites use
  existing vocabulary (target, handle, capture readiness, listing snapshot,
  attribution fidelity, dependency model), all already in the glossary. No new term
  is introduced; if one is, it gets an entry in this change. PASS (verified in
  tasks).
- **P-9 (no fabrication; honest reporting)**: The doctor sample must match the
  binary (FR-019), and the IGDB walkthrough is deferred precisely because
  documenting unbuilt credential plumbing would fabricate a shipped capability.
  PASS.
- **P-11 (the specification describes what shipped)**: The whole slice exists to
  make the docs describe what shipped. Spec section 23.1/26.3 are reconciled;
  `cargo xtask spec` keeps the Applies-To binding. PASS.
- **Compatibility outranks richness / wrappers stay thin**: Not engaged (no output
  format or wrapper change). PASS.
- **Pinned-artifact discipline**: `scripts/lint-docs.sh`, workflows,
  `rust-toolchain.toml`, `release.toml` are not modified. No dated CHANGELOG
  decision is required. PASS.
- **Licensing rule (npcap)**: The #133 narrative reconciliation states the amended
  posture exactly (detection-only, plus the user-confirmed vendor-installer fetch
  from S056, bundling/hosting/SDK-vendoring still absolute). It documents the
  shipped posture, changes no policy. PASS.
- **Text encoding / no dashes**: Enforced across all edited files. PASS (verified
  in tasks).

No violations. Complexity Tracking is empty.

## Project Structure

### Documentation (this feature)

```text
specs/057-landing-getting-started/
├── plan.md              # This file
├── spec.md              # Feature spec (/speckit-specify)
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output (doctor report + retired-token inventory)
├── quickstart.md        # Phase 1 output (validation guide)
├── contracts/           # Phase 1 output
│   ├── doctor-report.md         # The corrected doctor row/section contract
│   ├── cli-reference-surface.md # The command surface the CLI reference must mirror
│   └── retired-token-inventory.md # The exact tokens SC-002 forbids, per file
└── checklists/
    ├── requirements.md
    └── docs-convergence.md
```

### Source Code (repository root)

```text
site/
├── app/(home)/page.tsx              # Landing page: rewrite (US2)
├── app/(home)/brand/page.tsx        # Brand specimen line: fix retired demo (US2)
└── content/docs/
    ├── getting-started.mdx          # Rewrite (US1): folds #130-#135, #133 docs half
    ├── index.mdx                    # Update Guides/Reference links (US3)
    ├── architecture.mdx             # Update the writing-a-profile inbound link (US3)
    ├── meta.json                    # Nav: drop the two retired pages (US3)
    ├── guides/
    │   ├── capture-modes.mdx        # Fix retired verbs (US3)
    │   └── writing-a-profile.mdx    # DELETE (US3)
    └── reference/
        ├── cli.mdx                  # Rewrite to real surface (US3)
        ├── profile-schema.mdx       # DELETE (US3)
        └── target-schema.mdx        # Unchanged (current master schema doc)

crates/fragcap-cli/src/
├── doctor/checks.rs                 # Remove profile dir row + Profiles section + tests (US4)
├── doctor/mod.rs                    # Remove profile_dir/bundled_count/user_count Inputs fields (US4)
├── doctor/probe.rs                  # Remove count_profiles + profile_dir gathering (US4)
└── paths.rs                         # Remove now-unused user_profile_dir/bundled count usage (US4)

docs/fragcap-specification.md        # Section 26.3 doctor rows reconciled (via xtask spec binding)
changelog.d/S057-*.md               # Changelog fragment
```

**Structure Decision**: The repository is a Rust workspace with a co-located
Next.js documentation site under `site/`. This slice touches the site (the bulk)
and one crate (`fragcap-cli`, the doctor companion change). No new module or crate
is created; two site pages are deleted and their references rerouted.

## Complexity Tracking

No constitution violations; no entries.
