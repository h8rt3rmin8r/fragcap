# Research: Managed Direct-Executable Launch

## Decision 1: One public launch enum

**Decision**: Add a public immutable launch enum in the `fragcap` facade with existing Steam protocol and new direct-executable variants.

**Rationale**: Capture currently carries a Steam-specific request while Deep Capture has a launch adapter that does no launch itself. A shared enum gives both modes one prepared decision without moving target-store or platform concerns into core.

**Alternatives considered**:

- Add a second direct launch field beside Steam. Rejected because callers could construct contradictory launches.
- Put launch in `fragcap-core`. Rejected because core cannot depend on target storage, Steam, or process I/O.
- Keep direct launch in the CLI. Rejected because the public API acceptance criterion would fail and Deep Capture would retain duplicate policy.

## Decision 2: Resolve beneath the stored install root

**Decision**: Build the direct executable path by joining the single resolved Windows client from `launch_entries` to the stored install root, then normalize and verify that it remains beneath the canonical install root and names a file.

**Rationale**: The target entry already owns both facts. This preserves P-10 and refuses traversal or stale paths before capture, proxy, or trust effects.

**Alternatives considered**:

- Use `executable_hint`. Rejected because it is explicitly a findability hint, not a client identity.
- Search the machine for the executable. Rejected because it would create a second resolution path and could launch the wrong install.
- Store a new absolute launch path. Rejected because it duplicates existing identity facts and requires a migration.

## Decision 3: Explicit process creation, no shell

**Decision**: Execute direct launches through `std::process::Command` with a program path, `current_dir`, individual arguments, and individual environment additions. Drop the returned child immediately and observe lifecycle through existing watchers.

**Rationale**: `Command` preserves argument boundaries and does not invoke a command shell. Keeping no long-lived child object avoids introducing a second lifecycle observer or process-control owner.

**Alternatives considered**:

- `cmd.exe /C`, PowerShell, or a shell string. Rejected because quoting becomes executable policy and enables shell interpretation.
- `ShellExecuteW`. Rejected for direct executables because it accepts one parameter string and routes through the shell association layer.
- Retain and poll `Child`. Rejected because existing ETW and session lifecycle already own observation, and a second observer would drift.

## Decision 4: Environment is an immutable launch overlay

**Decision**: Direct launches accept explicit environment additions on the prepared value. Deep Capture adds only the proxy variables required for its selected loopback endpoint and executes that derived value without target or path re-resolution.

**Rationale**: Environment inheritance is the scoped routing mechanism. Treating it as a value makes the exact effect reviewable and testable and prevents ambient system proxy changes.

**Alternatives considered**:

- Mutate fragcap's own process environment. Rejected because concurrent work could observe it and unrelated child processes could inherit it.
- Set Windows Internet Options or WinHTTP proxy state. Rejected because those are machine or user ambient effects outside the selected target.

## Decision 5: Cold direct launch only for Deep Capture

**Decision**: Classify a non-Steam stored target selected with `--launch` as `direct-exe-cold` only when its resolved client image is absent from the process snapshot, and continue to refuse `direct-exe-warm`.

**Rationale**: A child created by the session can inherit its environment. An already-running target cannot be changed retroactively, and claiming otherwise would fabricate routing certainty.

## Decision 6: No dependency or schema change

**Decision**: Use existing target JSON and standard library APIs only.

**Rationale**: The required path checks, values, environment overlay, and process creation need no new crate. Existing storage already carries the target identity and install root.
