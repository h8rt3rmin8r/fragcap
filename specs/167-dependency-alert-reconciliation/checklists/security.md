# Security Requirements Checklist: S167 dependency alert reconciliation

**Purpose**: Check that the dependency correction requirements cover every affected inventory and retain the existing verification boundary.

**Created**: 2026-10-06

**Feature**: [spec.md](../spec.md)

## Advisory Coverage

- [x] Does the spec identify the advisory package and fixed version for every affected inventory?
- [x] Does the spec require zero vulnerable resolved instances, including duplicate transitive versions?
- [x] Does the spec distinguish the already fixed product graph from the vulnerable historical spike graphs?

## Regression Boundary

- [x] Does the spec require frozen site installation and a production build?
- [x] Does the spec require locked spike checks and the ordinary product gate?
- [x] Does the spec avoid claiming GitHub default branch alert closure before merge and rescan?
- [x] Does the spec keep the owner merge decision explicit?
