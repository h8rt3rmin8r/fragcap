### Changed

- **`cargo xtask notes` now stops at the `### Decisions` section.** Release notes
  are derived from `CHANGELOG.md`, and the decisions section records verbose
  internal rationale for changing pinned artifacts; a single decision fragment
  runs to kilobytes. Excluding it keeps the release page to the curated
  Highlights and the user-facing Added, Changed, and Fixed sections, while the
  full detail, decisions included, stays in `CHANGELOG.md`.
