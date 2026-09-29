# CLI Input Reliability Checklist: S161

**Purpose**: Check the planned prompt contract against the deadlock and authorization boundaries
**Created**: 2026-09-29
**Feature**: [spec.md](../spec.md)

## Prompt ownership

- [x] Does the specification require an operator response to reach Doctor after the question is shown?
- [x] Does it cover every known second-lock prompt, including target registration and warm restart?
- [x] Does it require bounded completion evidence rather than relying only on scripted confirmation doubles?

## Effect boundary

- [x] Does a negative Doctor response leave its action unperformed?
- [x] Do prompt output and input failures stop further effects and avoid a false `skipped` report?
- [x] Are existing noninteractive and `--yes` rules retained?

## Authorization boundary

- [x] Are Deep Capture and calibration responses still exact, bounded, complete, and separate?
- [x] Can already buffered input not authorize a later plan silently?
- [x] Does the scope avoid attributing Print Screen behavior to fragcap without evidence?

## Evidence

- [x] Can the scoped outcomes be demonstrated by controlled tests within the slice?
- [x] Does the slice distinguish repository-controlled verification from future field observations?
