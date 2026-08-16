<!-- spec-impact: none -->

`cargo xtask spec` checks that the specification's `Applies-To` field matches the
workspace version and that every changelog fragment declares its `spec-impact`,
and the changelog release step now refuses to assemble a release whose fragment
claims a specification change the release diff does not contain.
