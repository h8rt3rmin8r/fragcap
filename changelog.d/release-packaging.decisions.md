**2026-08-12** Changed the distribution archive in `.github/workflows/release.yml`
to contain only the binary, `LICENSE`, and `NOTICE`. It previously bundled all of
`scripts/` (shipping the maintainer tooling `cut-release.sh`, `New-Release.ps1`,
and `lint-docs.sh` alongside the user wrappers), the repo-oriented README, and a
permanent false `INCOMPLETE.txt` fired by an empty `profiles/` directory that
fragcap never populates (issue #54). The shell wrappers are no longer shipped in
the archive at all: since #56 the binary detects elevation itself and `doctor`
covers driver detection, so a wrapper adds little where it would ship, and the
wrappers remain in the repository for people who clone it (issue #57).
Specification section 24.5 is reconciled to match.
