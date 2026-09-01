<!-- spec-impact: 2.1, 13.7, 17.2, 19, 25, 26.3, 28.1 -->
### Added

- Added explicit native client-facing TLS 1.2 and TLS 1.3 key logs at the protected final bundle path, with live flushing and exact session status.
- Added operator-supplied upstream mutual-TLS identities and evidence-backed refusal categories without target key discovery or certificate-pinning bypass.
- Added protected bundle preparation, bounded sensitive-action recovery, confirmed exact cleanup, and atomic share-on-copy with a transformation manifest.
- Added `fragcap bundle cleanup` and `fragcap bundle export` for completed Deep Capture evidence.
