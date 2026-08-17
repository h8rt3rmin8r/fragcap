<!-- spec-impact: none -->

2026-08-17: The release workflow (`.github/workflows/release.yml`) now seeds the
detection signature table into the shipped `catalog.db`. Before slice S053 the
release built a barebones catalog by importing the committed empty hint seed, which
under the S053 schema (version 5) creates an empty `signature` table; the shipped
catalog would then detect no engine, anti-cheat, or DRM until an operator ran
`targets seed-signatures` by hand. The build step now runs `fragcap targets
seed-signatures --db target/release/catalog.db` immediately after the import, so the
archive and the MSI both ship a catalog whose detection works out of the box. The
seed is offline (from the bundled Appendix B document embedded in the binary) and
idempotent, so it adds no network dependency and re-running is safe. This is a
dated decision recorded per the pinned-artifact rule (constitution: workflows change
only with a `changelog.d/*.decisions.md` fragment).
