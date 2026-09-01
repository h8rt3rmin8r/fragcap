# Contract: Deep Capture Manifest Version 2

- `manifest_version` is 2; `$schema` and `product.version` are distinct.
- One declaration exists for every expected role, produced or omitted.
- Authority is typed and projections name their source role.
- Finalization and evidence completeness are separate.
- Version 1 is read conservatively and never rewritten.
- Paths are canonical contained relative paths and unique under Windows comparison.
- A durable prefix is crash-prefix truth; only atomic final publication claims final state.
