# Data Model: Deep Capture Embedded Workflow Help

S148 adds no persistent product data. It defines an in-process documentation and guidance authority used by the CLI renderer and tests.

## Workflow Registry

| Field | Type | Invariant |
| --- | --- | --- |
| `schema_version` | integer | Exactly `1` |
| `stages` | ordered stage set | Doctor, discovery, registration or selection, detail, calibration, ordinary Deep Capture, recovery, cleanup, and export appear in navigation order |
| `audited_help_paths` | unique command paths | Covers the closed S148 command set |
| `examples` | unique command examples | Every displayed command has representative parse arguments |
| `refusals` | closed refusal categories | Every category has one or a finite set of next commands |

## Workflow Stage

| Field | Type | Invariant |
| --- | --- | --- |
| `id` | stable token | Unique and suitable for documentation synchronization |
| `purpose` | concise human text | Distinguishes this stage from every other stage |
| `prerequisite` | stage or initial condition | Forms an acyclic ordered journey |
| `example_ids` | non-empty reference set | References parseable commands in the same registry |

## Command Example

| Field | Type | Invariant |
| --- | --- | --- |
| `id` | stable token | Unique across the registry |
| `display` | exact command string | Begins with `fragcap` and appears verbatim in at least one help or guidance surface |
| `parse_argv` | representative argument vector | Accepted by the production Clap command model without executing the command |
| `help_paths` | one or more audited paths | Every referenced path exists in the Clap tree |
| `sensitivity` | ordinary or sensitive | Sensitive examples remain explicit and are never recommended silently |

## First-Run Refusal Guidance

| Field | Type | Invariant |
| --- | --- | --- |
| `category` | closed enum | Environment, target, launch, process, compatibility, recovery, bundle, authorization, or calibration |
| `summary` | human explanation | States the observed reason without claiming compatibility |
| `next_example_ids` | one or more references | Finite, ordered, parseable alternatives |
| `effect_state` | fixed token | Always `no-session-effects` for this registry |

## Validation States

- **Valid**: Every audited path renders short and long help, every example is displayed and parses, every refusal category has bounded guidance, and every referenced identifier resolves exactly once.
- **Missing navigation**: A workflow stage or audited page cannot reach its required next stage.
- **Grammar drift**: A displayed example no longer parses against the production command model.
- **Presentation drift**: A required section, safety boundary, or exact command disappears or changes order.
- **Unbounded refusal**: A closed first-run category has no next command or falls back to vague prose.

## State Transitions

```text
installed name -> Doctor -> discover/register -> target detail -> calibration -> ordinary Deep Capture
                                      |                 |              |
                                      +---- refusal ----+---- next ----+
prior residue -> Doctor recovery -> fresh plan
completed bundle -> retain original -> export share copy
completed bundle -> confirmed sensitive cleanup -> retain recovery truth on failure
```
