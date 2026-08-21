<!-- spec-impact: 26.3 -->
`fragcap catalog update` is gone. It could not run in any released binary, since
the release build has never enabled the feature that compiles it, and what it
actually fetched was a third-party title list rather than any fragcap artifact,
so its description of "the current published catalog" named something that does
not exist. `fragcap doctor` no longer points at it either: the remediation it
offers for a missing catalog store is now to create the store and load the
bundled detection signatures, which any build can do with no network, replacing
guidance that told users to rebuild fragcap from source with a build flag.
