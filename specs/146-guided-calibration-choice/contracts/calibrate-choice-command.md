# Command Contract: Guided Calibration Choices and Overrides

## Fresh workflow

```text
fragcap calibrate <SELECTOR> [--candidate <CANDIDATE_ID>] [--launch-case <CASE>] [--routing-strategy <STRATEGY>] [--proxy-family <ipv4|ipv6>] [--protocol <PROTOCOL> ...]
```

Defaults:

- `--launch-case` is absent and the current exact topology infers the cold case.
- `--routing-strategy` is `child-environment`.
- `--proxy-family` is `ipv4`.

`--candidate` is valid only when current discovery consumes it during target registration or Steam client setup. Supplying it to an already complete exact target is an error.

## Resume

```text
fragcap calibrate --resume <WORKFLOW_ID> --local-db <PATH>
```

Resume conflicts with `--candidate`, `--launch-case`, `--routing-strategy`, `--proxy-family`, and `--protocol`. Operational bounds remain selectable because each new plan binds them.

## Candidate identifier

```text
candidate-v1:<64 lowercase hexadecimal characters>
```

The identifier is stable for the same canonical candidate authority. It is not authorization, a stored target identifier, a workflow identifier, or compatibility evidence.

## Structured choice event

Event type: `calibration.choice_required`

```json
{
  "event": "calibration.choice_required",
  "scope": "target-registration",
  "selector": "Sample Target",
  "target_id": null,
  "choices": [
    {
      "id": "candidate-v1:<digest>",
      "source": "steam",
      "identity": "steam:75000",
      "display_name": "Sample Target",
      "classification": "game",
      "fidelity": "observed",
      "install_root": "C:\\Games\\Sample Target",
      "executable_hint": "client.exe"
    }
  ],
  "discovery_considered": 2,
  "discovery_produced": 2,
  "discovery_warning_count": 0,
  "process_control": "none"
}
```

Choices are ordered by identifier. Human output carries the same fields and an exact continuation suffix `--candidate '<ID>'`.

## Guidance additions

`calibration.guidance` adds:

- `routing_strategy`
- `address_family`
- `launch_case_assertion`, null when the launch case remains inferred

These values describe workflow intent and current validated selection. They do not claim compatibility.

## Refusal contract

Malformed, unknown, stale, duplicate, unused, resume-conflicting, unsupported, and topology-inconsistent choices exit with usage failure before any new low-level authorization plan or effect. Existing registration and Steam client plans retain their own terminal outcome events after they have been emitted.
