# Data Model: Managed Direct-Executable Launch

## PreparedManagedLaunch

An immutable launch selected during side-effect-free preparation.

Variants:

- `Steam(LaunchRequest)`: the existing Steam application identifier and protocol URL.
- `Direct(DirectExecutableLaunch)`: the exact direct child process configuration.

Only one variant can exist for a session.

## DirectExecutableLaunch

Fields:

- `executable`: canonical absolute path to an existing file beneath the stored install root.
- `working_directory`: canonical absolute stored install root, or the parent of a legacy authored absolute executable.
- `arguments`: ordered operating-system argument values.
- `environment`: ordered explicit environment additions for this child only.

Invariants:

- `executable` and `working_directory` are absolute.
- `executable` is beneath `working_directory` after canonicalization.
- Every argument is a distinct value. No command string exists.
- Environment keys are non-empty and contain neither `=` nor NUL.
- Applying an environment overlay does not change executable, working directory, or arguments.

## Target Relationship

The existing `TargetEntry` remains authoritative for:

- stable identity and selector
- install root, derived from an authored absolute executable when needed
- resolved client launch entries
- compatibility facts

No launch row, direct-launch table, or absolute-path cache is added.

## Lifecycle

```text
stored target
  -> side-effect-free launch preparation
  -> immutable prepared launch
  -> arm watcher and Capture pipeline
  -> optional Deep Capture proxy and trust effects
  -> apply target-scoped environment to retained direct launch
  -> create child
  -> observe process and descendants through existing watcher
  -> bounded stop, cleanup, and terminal truth
```
