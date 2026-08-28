# Data Model: Production Accessibility Remediation

S095 adds no runtime persistence. Its model consists of rendered accessibility subjects and browser observations used by the production-export gate.

## Public Route Case

| Field | Meaning | Validation |
| --- | --- | --- |
| `route` | Public path derived from one exported HTML file | Unique; excludes only not-found build artifacts |
| `layout` | Home or documentation layout family | Exactly one value |
| `viewport` | Browser width under test | 320, 768, or 1440 pixels |
| `primary` | The route-specific primary content element | Exactly one; identity is `main-content` |
| `skip` | The shared bypass control | Exactly one and precedes persistent navigation |

## Heading Observation

| Field | Meaning | Validation |
| --- | --- | --- |
| `route` | Generated changelog route | Belongs to the complete generated changelog set |
| `text` | Visible heading text | Preserved from the source release record |
| `level` | Rendered heading level | One through six |
| `id` | Generated anchor identity | Unchanged when only level changes |
| `previous_level` | Prior visible content heading level | Current level may exceed it by at most one |

## Contrast Observation

| Field | Meaning | Validation |
| --- | --- | --- |
| `subject` | Visible muted or syntax text element | Population must be nonempty |
| `foreground` | Computed opaque text color | Matches the corrected shared or syntax value |
| `background` | Nearest computed opaque background | One of the shipped surfaces used by the subject |
| `ratio` | Relative-luminance contrast ratio | At least 4.5:1 |
| `theme` | Active site theme | Light |

## Diagram Observation

| Field | Meaning | Validation |
| --- | --- | --- |
| `route` | Architecture documentation route | `/docs/architecture` |
| `index` | Document-order diagram position | Exactly two observations |
| `role` | Hydrated Mermaid SVG role | Graphics-document semantics preserved |
| `name` | Resolved programmatic name | Nonempty, distinct, and equal to its expected purpose-specific title |

## State Transitions

```text
static export -> loopback server -> browser load -> hydration complete
                                            -> route observations pass
                                            -> any missing or failed observation fails the gate
```

No empty subject population may transition to pass.
