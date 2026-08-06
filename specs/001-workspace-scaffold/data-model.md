# Phase 1 Data Model: The Workspace Graph

This slice defines no runtime data. Its structural model is the crate graph:
the entities are crates, and the relationships are dependency edges. Both are
verified mechanically, which is what makes this a model rather than a diagram.

## Entities

### Crate

A unit of compilation with one purpose and a fixed position in the dependency
direction.

| Attribute | Source | Notes |
| --- | --- | --- |
| name | Appendix A | Exactly the eight names, plus the task runner |
| purpose | Appendix A | One line, carried into the manifest description |
| version | workspace | Inherited; all crates share one version below 1.0.0 |
| edition | workspace | Inherited |
| license | workspace | Inherited; every crate declares Apache-2.0 |
| rust-version | workspace | Inherited; the declared minimum, not the build pin |
| dependencies | this document | The edge set below is normative for the slice |

| Crate | Purpose | Publishes |
| --- | --- | --- |
| `fragcap-core` | Types, traits, pipeline | library |
| `fragcap-profile` | Profile schema and matching | library |
| `fragcap-capture` | Acquisition backends | library |
| `fragcap-attr` | Attribution and process watching | library |
| `fragcap-sink` | Sinks and transports | library |
| `fragcap-steam` | Steam integration | library |
| `fragcap` | Facade | library |
| `fragcap-cli` | Binary | binary |
| `xtask` | Repository task runner | not published |

### Edge

A dependency from one workspace crate to another. The complete permitted set:

| From | To |
| --- | --- |
| `fragcap-cli` | `fragcap` |
| `fragcap` | `fragcap-core`, `fragcap-profile`, `fragcap-capture`, `fragcap-attr`, `fragcap-sink`, `fragcap-steam` |
| `fragcap-capture` | `fragcap-core` |
| `fragcap-attr` | `fragcap-core` |
| `fragcap-sink` | `fragcap-core` |
| `fragcap-profile` | `fragcap-core` |
| `fragcap-steam` | `fragcap-profile` |
| `fragcap-core` | none |
| `xtask` | none within the workspace |

## Validation Rules

Derived from section 8.3 and enforced by the task runner's dependency check.

- **V-1**: The observed edge set equals the permitted edge set exactly. An edge
  present but not listed fails; an edge listed but absent fails.
- **V-2**: No crate depends on `fragcap-cli`.
- **V-3**: No crate below the facade depends on a sibling at its own level.
  Siblings are `fragcap-capture`, `fragcap-attr`, `fragcap-sink`, and
  `fragcap-steam`. The `fragcap-steam` to `fragcap-profile` edge is not a
  sibling edge: `fragcap-profile` sits below them, adjacent to core.
- **V-4**: `fragcap-core` has no dependencies at all, within the workspace or
  outside it.
- **V-5**: Every crate declares the project license.
- **V-6**: The workspace member set equals the crates directory contents plus
  the task runner.

V-1 is stated as equality rather than containment deliberately. A missing edge
is as much a defect as an extra one, because it means a crate that should have
been wired up was not, and the failure would otherwise surface only when a
later slice tried to use it.

## State Transitions

Crates have one lifecycle state in this slice: skeleton. A skeleton compiles,
passes lints, declares its metadata and edges, and contains no capability.

The transition out of skeleton belongs to the slice that owns each crate:
`fragcap-core` at S02, `fragcap-profile` at S05, `fragcap-capture` at S09,
`fragcap-attr` at S10, `fragcap-sink` at S15, `fragcap-steam` at S17, the
facade and binary at S14.

Recording this here means a later reader can tell an intentionally empty crate
from an abandoned one.

## Non-Entities

Explicitly not modelled in this slice, to prevent speculative structure:

- Packet, flow, attribution, and profile types. These are S02 and S05.
- Any trait. The seams named in section 8.5 are S02's.
- Configuration schema. The profile schema is S05's.
- Output format structures. These are S06 and S07's.
