# Contract: Embedded Deep Capture Workflow Help v1

## Audited Commands

```text
fragcap -h
fragcap --help
fragcap doctor -h
fragcap doctor --help
fragcap targets -h
fragcap targets --help
fragcap targets discover -h
fragcap targets discover --help
fragcap targets add -h
fragcap targets add --help
fragcap targets show -h
fragcap targets show --help
fragcap calibrate -h
fragcap calibrate --help
fragcap deep-capture -h
fragcap deep-capture --help
fragcap bundle cleanup -h
fragcap bundle cleanup --help
fragcap bundle export -h
fragcap bundle export --help
```

## Short Help

- Exits `0` and uses the production command model.
- Gives one concise purpose per command or option.
- Exposes every stage needed to discover the first-run journey.
- Does not contain the complete tutorial or multi-paragraph option details.

## Long Help

- Exits `0` and preserves the same grammar as short help.
- Shows the first-run stages in execution order.
- Places safe required and common inputs before advanced, sensitive, custom-storage, and troubleshooting controls.
- Includes parse-backed Steam and direct or publisher examples.
- States that observed compatibility is required, decryption is not universal, certificate pinning is not bypassed, and selected sensitive artifacts require deliberate handling.

## Human Refusal Guidance

```text
<truthful reason>
Next command:  fragcap <parse-backed command>
```

or, only when the state genuinely permits a finite choice:

```text
<truthful reason>
Next commands:
  fragcap <parse-backed choice one>
  fragcap <parse-backed choice two>
```

- The reason remains specific to the observed state.
- The command is actionable and uses a visible placeholder only when the current value is unavailable.
- A placeholder example has a separate concrete parse argument vector in the registry.
- Machine-readable records, exit codes, and effect counts remain unchanged.

## Calibration Outcomes

- Name established reachability or protocol facts exactly.
- Name remaining unknown or requested cases exactly.
- End with the parse-backed ordinary Deep Capture command when current exact evidence is sufficient.
- Otherwise end with the parse-backed resume, choice, warm-retry, or bounded continuation command.

## Bundle And Recovery

- Export creates a separate share copy and never modifies the source bundle.
- Cleanup is destructive only for declared sensitive artifacts and requires confirmation.
- Failed cleanup preserves the manifest, cleanup stream, resource journal, and session-owner recovery authority needed by `fragcap doctor --fix`.
- Help never recommends broad manual deletion of recovery records.

## Compatibility Boundary

- Rendering and parse validation are effect-free.
- No real-game or trust-store run is an S148 acceptance requirement.
- S148 establishes documentation and navigation behavior only and makes no new title-compatibility or Deep Capture completion claim.
