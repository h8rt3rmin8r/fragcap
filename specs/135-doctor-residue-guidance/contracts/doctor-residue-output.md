# Doctor Residue Output Contract

## Human report

Every native resource check uses the stable human label `native residue`. Its diagnosis states:

- what condition was observed;
- whether active ownership was proven;
- the consequence for Deep Capture readiness;
- stable session and resource identity as secondary context; and
- the safe next action, when one exists.

Findings with an available Doctor cleanup action use this remediation meaning:

```text
Run fragcap doctor --fix to review and confirm cleanup of all eligible inactive Deep Capture records.
```

An abandoned session-owner diagnosis states that an earlier session ended without retiring its owner record, no active owner was proven, and Deep Capture remains blocked until all eligible inactive Deep Capture records are reviewed and confirmed for cleanup. Active records never receive abandoned wording, cleanup guidance, or `recovery_eligible: true`. Unknown and unsupported records never claim safety.

The aligned and compact layouts preserve the existing section order, status words, Capture verdict, and Deep Capture verdict. Output supports 40 through 80 display columns. Values are not truncated. An indivisible exact token may exceed the selected width.

## JSON Lines check record

Existing common fields remain present. A native residue check additionally contains:

```json
{
  "section": "Deep Capture",
  "name": "native resource s1/trust",
  "detail": "existing machine detail",
  "status": "fail",
  "scope": "deep_capture",
  "remediation": "existing machine remediation",
  "native_resource": {
    "session_id": "s1",
    "resource_id": "trust",
    "kind": "trust",
    "state": "installed",
    "health": "stale",
    "ownership_authority": "cleanup-journal",
    "recovery_eligible": true
  }
}
```

The nested object is omitted for non-native checks. `recovery_eligible` is true exactly when the check offers Doctor's cleanup action, not merely when the underlying journal contains a recovery plan. Records remain one valid JSON value per line. The object never includes bundle paths, certificate material, capabilities, credentials, payloads, or cleanup confirmation state.

## Recovery compatibility

The set of offered cleanup actions is equivalent to the pre-S135 set for the same controlled findings. `fragcap doctor --fix`, `--yes`, interactive confirmation, exact recovery planning, active-resource refusal, partial failures, and exit codes are unchanged.

## Width compatibility

- Non-terminal human output defaults to 80 display columns.
- Terminal human output uses the detected width, clamped to 40 through 80.
- Plain and color output have identical visible text and wrapping.
- Continuation indentation comes from the selected layout.
- Unicode scalars remain intact; combining and wide characters use display-cell accounting.
