# Publication Contract: S180

1. Candidate source equals the exact checked and reviewed PR head accepted through main integration.
2. Annotated tag `v0.10.8` peels to that accepted source. Tag/version mismatch, existing divergent tag or source drift blocks publication.
3. Tagged Windows certification succeeds before GitHub release creation. Six public files match certified final bytes and three sidecars verify their primary downloads.
4. Normal protected approval precedes registry execution. All ten product crates must be non-yanked at 0.10.8 and archive checksums must match independently downloaded bytes.
5. Only after conditions 2 through 4 hold may current documentation, published-release JSON, frozen review candidate and site move to v0.10.8 through a separate checked PR.
6. A failed or unavailable external operation remains incomplete and must not be represented as a published release.
