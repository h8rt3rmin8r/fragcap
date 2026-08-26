# Progress Quality Checklist: Doctor Progress And Timing

**Purpose**: Validate that progress and timing behavior is specified before implementation.
**Created**: 2026-08-26
**Feature**: `specs/079-doctor-progress-timing/spec.md`

## Contract Coverage

- [x] CHK001 Progress stream is separate from final report output
- [x] CHK002 JSON suppression is explicit
- [x] CHK003 Redirected stdout suppression is explicit
- [x] CHK004 Quiet and silent suppression is explicit
- [x] CHK005 `--fix` behavior is not mixed with progress output
- [x] CHK006 Probe labels are named in a stable contract
- [x] CHK007 Timing output is diagnostic and not a stable JSON schema change

## Measurement Quality

- [x] CHK008 Timings are measured around real probe work
- [x] CHK009 Probe timing order follows probe execution order
- [x] CHK010 Dominant local cost evidence is required before optimization

## Output Stability

- [x] CHK011 Existing doctor goldens remain authoritative
- [x] CHK012 Redirected human output remains byte-stable
- [x] CHK013 `doctor --json` remains byte-stable
