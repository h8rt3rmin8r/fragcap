# Data Model: Production UX And Accessibility Audit

S094 adds no runtime data model. Its evidence model is persisted only in the Markdown audit report.

## Audit Check

| Field | Meaning | Validation |
| --- | --- | --- |
| `id` | Stable check identifier | Unique within the report |
| `subject` | Route, shared surface, or content class examined | Names a route or inventory entry |
| `mode` | Desktop, 768 px, 320 px, keyboard, semantic, zoom, theme, search, or link | Uses the report vocabulary |
| `method` | Observable procedure used | Reproducible without private state |
| `result` | Passed, failed, or not run | Exactly one state |
| `evidence` | Measurement, browser observation, command output, or finding link | Non-empty for passed and failed states |
| `limitation` | Why a check did not run or what it cannot establish | Required for not-run state |

## Route Observation

| Field | Meaning | Validation |
| --- | --- | --- |
| `route` | Public path under the local production origin | Appears in reconciled inventory |
| `route_kind` | Home, informational, documentation, glossary, or not-found probe | One declared kind |
| `viewport` | Width and height or zoom mode | Width is 320, 768, or 1440 px for responsive checks |
| `navigation` | Reachability and shared navigation result | Passed, failed, or not applicable |
| `structure` | Heading and landmark result | Passed or linked finding |
| `overflow` | Horizontal and vertical reachability result | Passed, failed, or not applicable |
| `notes` | Route-specific evidence | Concise and reproducible |

## Finding

| Field | Meaning | Validation |
| --- | --- | --- |
| `id` | Stable report identifier | Sequential and unique |
| `title` | Narrow observed defect | Describes one behavior |
| `severity` | Critical, high, medium, or low | Matches FR-013 |
| `route` | Smallest affected route set | Explicit path or shared surface |
| `viewport_or_mode` | Condition required to reproduce | Explicit value |
| `reproduction` | Ordered steps | Repeats from a clean production export |
| `evidence` | Observable result and relevant expectation | Contains no private data |
| `impact` | User barrier or audit consequence | Specific, not speculative |
| `disposition` | Existing issue, new issue, accepted limitation, or no defect | Exactly one value |

## Follow-up Issue

| Field | Meaning | Validation |
| --- | --- | --- |
| `url` | GitHub issue link | Resolves to this repository |
| `overlap_search` | Search terms and compared candidates | Recorded before creation |
| `boundary` | One defect and affected surface | Excludes unrelated findings |
| `acceptance` | Conditions that close the defect | Testable |
| `labels` | Type, area, priority, and effort | Uses repository labels |
| `milestone` | Planning milestone | `Post-v0.7.0 documentation` |

## State Transitions

```text
required -> performed -> passed
                      -> failed -> finding -> existing issue
                                            -> new issue
required -> not run -> limitation recorded
```

No required check may disappear from the final report, and no failed material observation may reach completion without one disposition.
