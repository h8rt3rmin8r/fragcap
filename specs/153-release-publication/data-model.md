# S153 Publication Evidence Model

## Release Identity

The existing schema_version=1 record has version, source_revision (exact forty-hex merged tag source), published_date and canonical release_url. It changes from 0.10.0 only after complete 0.10.1 verification. Candidate workspace version remains independently validated.

## Certified Bundle

Six exact asset names have sizes and SHA-256 values, reconciled against official sidecars and source/version-bound report.build_identity. Require official and complete flags, zero findings and every artifact complete. The report's run ID and summary artifact ID identify provenance independently of later documentation commits.

## Registry and Gates

Ten exact product crate version records identify version=0.10.1, yanked=false and public checksums. Each source/release workflow has run ID, exact head SHA, status and conclusion. Owner deployment decisions retain reviewer identity or explicit bypass distinction; no agent decision is represented.

## State Transitions

Prepared, source-green, tagged, certifying, release-created, awaiting-owner, registry-publishing, fully-verified and reconciled-PR are separate states. Failed and partial outcomes preserve their last successful boundary. Release-created alone is not fully-verified. Reconciled-PR does not mean human-merged or independently accepted.
