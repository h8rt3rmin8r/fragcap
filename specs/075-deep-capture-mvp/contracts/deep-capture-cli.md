# Contract: Deep Capture CLI

## Command shape

The final command spelling is allowed to follow clap ergonomics during implementation, but it must expose Deep Capture as a first-class command. The planned shape is:

```powershell
fragcap deep-capture <selector> --launch --bundle .\out\session --duration 5m
```

Machine-readable status uses the existing global flag:

```powershell
fragcap --json deep-capture <selector> --launch --bundle .\out\session --duration 5m
```

## Required arguments

- `<selector>` or `--target <selector>`: stored target selector.
- `--id <stable-id>`: stored target stable id. Mutually exclusive with selector forms.
- `--launch`: required for MVP, because Deep Capture must own the scoped launch environment.
- `--bundle <dir>`: optional bundle root. Defaults to fragcap session storage when omitted.

## Supported capture controls

Deep Capture may reuse these `capture` controls where they preserve bundle accounting:

- `--duration`
- `--wait`
- `--max-packets`
- `--max-bytes`
- `--interface`
- `--no-payload`

Deep Capture always includes loopback because the local proxy path requires it.
Role and direction filtering remain downstream concerns for the MVP so the
session does not discard useful traffic before analysis.

## Deep Capture controls

- `--trust-ca`: ask to trust the fragcap Deep Capture CA when HTTPS inspection requires it.
- `--yes`: pre-confirm trust and cleanup prompts only where the command is otherwise interactive-safe.
- `--har`: write HAR when HTTP semantics are observable.
- `--key-log`: create and announce a proxy-owned analyzer key log before proxy traffic, append secrets during the session, and mark a nonempty result secret-adjacent in the manifest.
- `--proxy-backend mitmdump`: select the MVP backend. Other values are refused until implemented.

## Refusals

The command exits before launch and before mutation when:

- no stored target resolves;
- raw `--process` is used;
- `--launch` is omitted;
- the selected proxy backend is unavailable;
- the target lacks stored facts proving scoped proxy propagation to the final client;
- trust mutation is required but not confirmed;
- the requested launch path requires system-wide proxy configuration;
- stale fragcap-owned proxy state blocks the selected local port and cannot be cleaned safely.

## Output contract

Human output names phases, blockers, warnings, bundle path, manifest path, inspectability summary, and cleanup summary. It never prints decrypted bodies, key-log contents, certificate private material, tokens, account names, or local absolute paths beyond paths the user explicitly supplied.

`--json` emits newline-delimited records through the existing event path. New event kinds use the `deep_capture.` prefix.
