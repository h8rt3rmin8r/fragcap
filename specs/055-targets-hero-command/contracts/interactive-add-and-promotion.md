# Contract: interactive `targets add` and capture promotion

**Feature**: S055 | **Date**: 2026-08-18

## Interactive flow (stdin is a terminal)

1. **Executable**: user supplies a path argument, or presses Enter to browse
   (guided path entry). The chosen executable's directory is the scan root.
2. **Inline scan**: run detection (`SignatureSet::compile` + `detect`) and print
   the engine / anti-cheat / drm findings inline, before dependent prompts
   (FR-009). Unreadable paths are surfaced, not hidden (P-4/P-9).
3. **Name + handle**: prompt for a display name; prompt for a handle offering the
   derived default; a colliding handle is disambiguated, never overwriting an
   existing target (FR-010).
4. **Socket-holder question**: ask exactly
   `Is the executable above the process that holds the sockets? [Y/n/unsure]`.
   Accept `Y`, `n`, `unsure` (case-insensitive; empty = default Y is NOT assumed
   silently -- an empty answer re-prompts, since the honest default is unknown).
5. **Persist**: build a `TargetEntry` and store via `insert_target` (P-10),
   with `evidence` set from the scan, `classification_source = user`,
   `fidelity = Authored`.

### Answer -> stored launch chain (D5, P-9)

| Answer | Meaning | Stored `launch_entries` | CAPTURE |
| --- | --- | --- | --- |
| `Y` | this exe holds the sockets | resolved client entry for the exe | `ready` |
| `n` | a different, unknown process holds them | exe as a non-client stage, holder unresolved | `needs a target` |
| `unsure` | unknown | unresolved marker, no holder claim | `needs a target` |

No answer records a socket holder the tool did not observe (FR-012). `unsure` is a
required, first-class outcome (FR-011); it is the reason the prompt exists.

## Non-interactive flow (stdin not a terminal)

Uses the existing flag-driven form: `--name` (or `--steam`), `--exe`, `--anchor`,
`--handle`. A required-but-missing value is a usage error (exit 2), never a
blocking prompt (FR-015). The `Y`/`n`/`unsure` decision, when needed, is supplied
by a flag (e.g. `--socket-holder yes|no|unsure`) so every branch is reachable in
tests without a terminal (CHK022). All three outcomes persist through the same
`insert_target` path as the interactive flow.

## `add --steam <app_id>`

Scaffolds and registers a `steam:<app_id>`-anchored entry for an installed title
(the capability formerly `steam profile`). Not installed -> usage error naming the
app id. Already registered (anchor present) -> reports the existing handle/id and
exits 0, no duplicate (existing behavior, targets.rs:260-271).

## Capture promotion (the `unsure` -> `verified` write-back)

- When `capture` runs against a target whose launch chain is unresolved, after the
  run it takes the observed dominant socket-holding image from the run's
  attributions and writes it back via `promote_target_launch`, rewriting
  `launch_entries` to the resolved client and raising `fidelity` to `Verified`
  (FR-013).
- If the run observes no socket-holding process, the entry is left unresolved --
  no fabrication (P-9). A capture never invents a holder to "complete" a row.
- Testability: the promotion function (observed image + unresolved entry ->
  resolved entry + fidelity bump) is unit-tested directly; the end-to-end
  promotion runs over the fixture pipeline (no live driver, spec 25.1). If
  implementation shows promotion needs a live backend, the store method + pure
  function still land and are unit-tested and the live demo is Tier 2 (not CI),
  stated as such, never reported as a passing check it is not.
