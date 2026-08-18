# Contract: the `doctor --fix` CLI surface

The command grammar and its refusal rules. This is the external contract users and
tests bind to.

## Grammar

```
fragcap doctor                 # unchanged: read-only classifier, exit 0/1
fragcap doctor --json          # unchanged: one JSON record per check, exit 0/1
fragcap doctor --fix           # NEW: classifier, then interactive action phase
fragcap doctor --fix --yes     # NEW: action phase with per-action prompts pre-confirmed
```

`DoctorArgs` gains two booleans: `--fix` and `--yes`.

## Exit codes

| Invocation | Condition | Exit |
| --- | --- | --- |
| `doctor` / `doctor --json` | no blocking check | 0 |
| `doctor` / `doctor --json` | any blocking check | 1 |
| `doctor --fix` | ready machine, nothing to fix | 0 |
| `doctor --fix` | actions offered and run; machine still blocked after | 1 |
| `doctor --fix` | actions offered and run; machine ready after | 0 |
| `doctor --fix --json` | refused (usage error) | 2 |
| `doctor --fix` with non-terminal stdout | refused (usage error) | 2 |
| `doctor --yes` without `--fix` | refused (usage error) | 2 |

The post-action exit reflects the re-run verdict (FR-010): the same 0/1 rule the
read-only report uses, evaluated after the confirmed actions ran.

## Refusal rules (FR-007, FR-008, FR-009)

- `--fix` + `--json`: exit 2, no action, a message stating `--fix` is interactive and
  incompatible with `--json`.
- `--fix` + non-terminal stdout (piped/redirected): exit 2, no action, a message
  stating `--fix` needs an interactive terminal. Holds even with `--yes`.
- `--fix` without `--yes` + non-terminal stdin: exit 2, no action (the prompt reads
  stdin; a redirected stdin would read end-of-file and skip everything). `--yes`
  supplies the answers and does not require a terminal stdin.
- `--yes` without `--fix`: exit 2, no action.

All three are decided before the classifier's action phase begins, so a refused
invocation performs nothing.

## Behavioral contract

- `doctor` without `--fix` is byte-for-byte and exit-code identical to before this
  slice for the same `Inputs` (SC-001), except that new fixtures exercising an absent
  catalog or zero target entries show two additional warning rows (both non-blocking).
- `--fix` prints the same report first, then the action phase.
- `--fix` offers only actions carried by checks in the printed report, in report
  order (FR-003).
- Each offered action is named before it runs and runs only after confirmation
  (or `--yes`).
- After the action phase, the classifier is re-run and the updated verdict printed.
- No action is performed when the invocation is refused.
