### Decisions

**2026-08-10. The brand session resolved Q-7 and Q-8, and the approved kit is
vendored into the repository.**

- **Q-7 is Geist Mono.** The monospace face is a selection requirement, not a
  style choice, because users read packet payloads in it. Geist Mono keeps `0 O`,
  `1 l I`, `8 B`, `5 S`, and `2 Z` distinguishable at the interface size, and its
  family relationship to Geist keeps technical specimens from feeling detached
  from surrounding documentation. Any future replacement is evaluated against the
  real-format specimen in `brand/specimens/`, not a decorative alphabet.
- **Q-8 is an independent ShruggieTech sub-brand.** fragcap shares Space Grotesk,
  Geist, Geist Mono, dark-first discipline, and the parent's exact `#FF5300`
  orange, but not ShruggieTech green, the shruggie mark, or marketing layouts. The
  visible relationship is the endorsement "A ShruggieTech project" in Geist Mono,
  uppercase, subordinate, and outside the logo's clear space. There is no combined
  parent-product logo. This satisfies the section 23.3 "instrument, not weapon"
  posture, which is part of the security posture rather than decoration.
- **The kit is vendored under `brand/`, not embedded in a crate.** The assets are
  not part of any published crate, so the per-crate license discipline is
  unaffected. The fonts remain under the SIL Open Font License 1.1, with the
  license texts carried in `brand/fonts/licenses/`; the logos and the custom
  wordmark are the project's brand marks, usable under the treatment rules in
  `brand/README.md` rather than under the code's Apache-2.0 grant.
- **`brand` is added to the linter's excluded-directory list** (`xtask/src/lint.rs`),
  matching the vendored-content rationale already used for the skills directories.
  The binary fonts and images are skipped by content sniffing, but the PDF guide
  is text-like in its first bytes and the vector art is machine-generated, so the
  whole directory is treated as vendored content re-imported wholesale rather than
  edited in place. Text files were normalized to LF on import so the exclusion is
  a policy choice rather than a workaround for dirty bytes.
