# S174 release research

## 2026-10-19 decisions

**Decision**: v0.10.7 publishes current merged main b4cb29a and every remaining fragment since v0.10.6. **Rationale**: S173 corrections are accepted source but absent from published v0.10.6. **Alternatives**: IGDB is explicitly rejected; no feature expansion or unrelated dependency update is bundled.

**Decision**: Reuse S172's version-only candidate and separate publication-records process. **Rationale**: The wrapper requires clean main while the mandatory spec gate is already committed on a release branch. Direct cargo-release preserves all tool verification and disables tag/push/publication until deliberate later steps. **Alternatives**: Editing versions manually or running the clean-main wrapper after planning would bypass the established coherent mechanism or fail preconditions.

**Decision**: Advance current/public authorities only after independent actual-byte verification. **Rationale**: P-11 and existing spec gate distinguish workspace applicability from published baseline. **Alternatives**: Early blanket replacement of 0.10.6 would corrupt history and falsely advertise publication.

**Decision**: Normal protected approval under owner delegation. **Rationale**: Fresh environment metadata requires exact owner reviewer, prevent_self_review=false, can_admins_bypass=true and a single v* tag policy. No bypass or policy change is needed. **Alternatives**: Removing protection or pretending verification is approval is rejected.

The speckit-plan research agent independently inspected S172 and current candidate surfaces without mutations or Cargo execution. Its confirmed minimum candidate inventory is reflected in plan.md and tasks.md. Existing S173 full local/elevated evidence is inherited product evidence; S174 still requires fresh candidate and tagged release gates.

## Confirmed publication surfaces

The actual primary authority is docs/published-release.json. Current baseline markers are the fourteen surfaces enumerated in xtask/src/spec.rs. docs/audits/native-documentation-coverage.v2.json binds actual version/source. docs/security/native-product-review-candidate.v1.json and six literal PUBLISHED_ASSETS in xtask/src/review_record.rs advance together only after actual downloads reconcile. The replay job label in .github/workflows/published-review-candidate.yml follows that authority in the records phase with a dated decision. The frozen v0.8.0 predecessor and historical measured evidence remain unchanged. Site generated version/changelog content is ignored and derives from prebuild, so it is not edited manually.
