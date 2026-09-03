# Bypass Safety Checklist: Proxy Bypass and Local-Destination Policy

**Purpose**: Test whether S122 completely specifies scope, recursion, local-destination, DNS, and evidence safety
**Created**: 2026-09-03
**Feature**: [spec.md](../spec.md)

## Requirement Completeness

- [x] CHK001 Are every supported rule kind, address family, and port behavior explicit? [Completeness, FR-001]
- [x] CHK002 Are normalization, duplicate, ordering, and unsafe-syntax outcomes explicit? [Completeness, FR-002, FR-003]
- [x] CHK003 Are ambient uppercase and lowercase environment variables fully displaced? [Completeness, FR-004, FR-005]
- [x] CHK004 Are listener, controlled origin, operator bypass, refused local service, and undetermined cases distinct? [Completeness, FR-006 through FR-008, FR-012]
- [x] CHK005 Are plan and evidence surfaces plus conservation requirements identified? [Completeness, FR-011 through FR-014]

## Requirement Clarity

- [x] CHK006 Are bare and leading-dot apex-and-descendant semantics defined without wildcard ambiguity? [Clarity, Clarifications]
- [x] CHK007 Is empty explicit policy distinguished from inherited policy? [Clarity, FR-004]
- [x] CHK008 Is requested-name matching separated from post-resolution address policy? [Clarity, FR-009]
- [x] CHK009 Is intentional bypass distinguished from every loss class? [Clarity, FR-013]
- [x] CHK010 Are built-in infrastructure exclusions separated from operator rules? [Clarity, Key Entities]

## Security and Scenario Coverage

- [x] CHK011 Are wildcard-all and infrastructure-colliding policies refused before effects? [Security, FR-003, FR-015]
- [x] CHK012 Are self-proxy recursion and mapped listener aliases covered? [Security, FR-006, FR-016]
- [x] CHK013 Are unrelated local, private, link-local, unique-local, multicast, and unspecified destinations denied implicit permission? [Security, FR-007]
- [x] CHK014 Are controlled origins kept on the proxy path with exact grants? [Security, FR-008]
- [x] CHK015 Are mixed answers, rebinding, ordering, and repeated resolution covered? [Security, FR-009, FR-010, FR-016]
- [x] CHK016 Are malformed DNS, CIDR host bits, ports, bracketed IPv6, and non-ASCII inputs covered? [Coverage, Edge Cases, FR-016]
- [x] CHK017 Is unsupported target projection required to refuse instead of weakening semantics? [Security, FR-015, Assumptions]
- [x] CHK018 Are system proxy mutation, silent trust, transparent fallback, and target process access prohibited? [Scope, FR-018]

## Acceptance Quality

- [x] CHK019 Can parser correctness be measured across input-order permutations? [Measurability, SC-001]
- [x] CHK020 Can pre-effect refusal and environment isolation be demonstrated objectively? [Measurability, SC-002, SC-003]
- [x] CHK021 Can local correctness and rebinding resistance be demonstrated without remote services? [Measurability, SC-004, SC-005]
- [x] CHK022 Can observable decision conservation, unavailable bypass truth, and zero bypass loss be computed exactly? [Measurability, SC-006]
- [x] CHK023 Can dependency neutrality and repository health be checked mechanically? [Measurability, SC-007]

## Notes

- All 23 requirement-quality checks pass. This checklist is a formal PR-review gate for issue #318.
