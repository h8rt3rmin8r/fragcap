# Contract: Guided Calibration Acceptance Registry v1

## Command

```text
cargo xtask guided-calibration-acceptance
```

## Success

- Exit `0`.
- Reports schema version, thirteen criteria, and the number of executable evidence references.
- Confirms that implementation evidence is controlled and automated.

## Validation Failure

- Exit `1`.
- Emits one diagnostic per registry or evidence-reference defect.
- Never treats a skipped, ignored, conditional, missing, untracked, or documentary test as acceptance evidence.

## Infrastructure Failure

- Exit `2`.
- Used when the registry, a referenced source, or Git tracking inventory cannot be read.

## Registry Shape

```json
{
  "schema_version": 1,
  "reviewed_on": "YYYY-MM-DD",
  "scope": "guided calibration issue #380",
  "implementation_evidence": "controlled-automated",
  "live_compatibility": {
    "status": "not-demonstrated",
    "owner": "operator",
    "timing": "published-release-only"
  },
  "criteria": [
    {
      "id": "AC-01",
      "statement": "...",
      "evidence_class": "controlled-automated",
      "tests": [
        {
          "path": "crates/.../tests/example.rs",
          "function": "exact_test_name",
          "proves": "..."
        }
      ]
    }
  ]
}
```

## CI Composition

The full gate first runs the workspace test suite, which executes every accepted reference, and later runs this registry validator, which proves inventory and source currency. Neither half substitutes for the other.
