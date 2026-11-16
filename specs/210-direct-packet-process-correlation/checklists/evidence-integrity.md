# Evidence Integrity Checklist: S210

**Purpose**: Validate the direct route and process-tail proof boundary
**Created**: 2026-11-16
**Feature**: [spec.md](../spec.md)

- [x] Route receipt, client transfer, passive packet observation, and helper leg have separate authorities.
- [x] Original packet absence and finite history loss cannot become a synthetic packet claim.
- [x] Unique process generation, socket creation, image, flags, lag, and event completeness are required for tail attribution.
- [x] PID and tuple reuse, missing image, and partial history have explicit negative cases.
- [x] Controlled package and private installed-product evidence are distinguished.
