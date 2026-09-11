# Command Contract: Durable Guided Calibration Resume

## Start

```text
fragcap calibrate <TARGET> [--protocol <PROTOCOL>]... [SESSION BOUNDS]
```

A successfully prepared target creates one workflow checkpoint before the first
effectful attempt. Guidance includes its identifier and exact resume command.

## Resume

```text
fragcap calibrate --resume <WORKFLOW_ID> --local-db <PATH> [SESSION BOUNDS]
```

Resume selects exactly one row from exactly one local store. It cannot be combined
with a target selector or `--protocol`. It re-resolves target authority and current
workflow state before proposing remaining work.

## Explicit Pause

```text
fragcap calibrate --resume <WORKFLOW_ID> --local-db <PATH> \
  --pause-for <login|eula|gameplay|shutdown|interrupted>
```

This is a no-effect checkpoint operation. It emits the new revision, exact reason,
and the resume command without entering registration, setup, proposal execution, or
Deep Capture.

## Guidance Additions

The existing `calibration.guidance` event gains nullable fields:

```json
{
  "workflow_id": 17,
  "workflow_revision": 4,
  "workflow_state": "paused",
  "pause_reason": "gameplay",
  "resume_command": "fragcap calibrate --resume 17 --local-db C:\\\\path\\\\local.db"
}
```

Fresh pre-workflow registration and setup guidance may retain null workflow fields.
Once the workflow exists, every terminal guidance carries the five fields. Human
output projects the same values.

## Refusals

The command starts no new effect for:

- missing workflow
- unsupported record version
- invalid stored vocabulary or non-canonical protocol set
- changed or missing target authority
- stale expected revision
- incompatible resume arguments
- a refused workflow

## Authorization

Resume never supplies authorization. Every selected attempt uses the unchanged S134
plan and confirmation flow. Plan identifiers and responses are not checkpoint fields.

## Effect Recovery

An `in-flight` checkpoint means only that the previous process crossed the durable
pre-delegation boundary. Existing resource-obligation journals and Doctor remain the
only authorities for deciding whether cleanup is required before another attempt.
