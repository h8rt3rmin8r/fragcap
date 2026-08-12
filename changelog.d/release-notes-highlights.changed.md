`cargo xtask notes` now generates a release body from a version's curated
Highlights plus a link to the full changelog at the tag, rather than the entire
Added/Changed/Fixed list. A version with no Highlights block still falls back to
the fuller body, so a release without curated highlights is never empty (issue
#53).
