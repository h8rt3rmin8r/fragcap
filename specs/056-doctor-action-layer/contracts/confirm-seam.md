# Contract: the ActionConfirm seam

The interface through which `--fix` asks the operator to confirm an action, with a
console implementation and a scripted test double, so the confirm loop is exercised
without a real terminal (FR-017).

## Trait

```
trait ActionConfirm {
    // Ask whether to perform `action`. true = perform, false = skip.
    fn confirm(&self, action: &Action) -> bool;
}
```

## Implementations

| Impl | Behavior | Used by |
| --- | --- | --- |
| ConsoleConfirm | Prints the action label and a `[y/N]` prompt, reads a line from stdin, returns true only on an affirmative answer | the real `--fix` run in a terminal |
| YesConfirm | Always returns true | `--fix --yes` |
| ScriptedConfirm | Returns answers from a fixed list (or a constant), in order | tests |

## Contract

- The driver calls `confirm` once per offered action, in report order.
- A `false` result skips the action (records `Skipped`) and advances to the next.
- The seam decides only yes/no; it performs no action and prints no outcome. The
  driver performs and reports.
- ConsoleConfirm reads stdin; the refusal gate (non-terminal stdout) is checked by
  the shell before the loop, so ConsoleConfirm is only ever constructed in an
  interactive context.
- The default answer on an empty line or a non-affirmative token is No (safe default
  for a tool that may run elevated).

## Testability

`cli_doctor.rs` drives the whole action phase with a `ScriptedConfirm` and an
injected `Report` plus `Capabilities`, asserting: the offered set equals the report's
carried actions; a declined action records `Skipped` and changes nothing; the
degraded label appears when `net` is false; and an action whose check is absent is
never offered. No terminal, driver, elevation, or network is involved.
