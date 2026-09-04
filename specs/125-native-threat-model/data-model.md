# Data Model: Native Deep Capture Threat Registry

## Registry

- `schema_version`: closed integer version
- `reviewed_on`: date of the security review
- `scope`: stable product scope statement
- `protocol_families`: sorted unique shipped protocol identifiers
- `proxy_dependencies`: sorted unique direct runtime dependency identifiers
- `protocol_reviews`: exactly one threat and executable-test mapping per shipped
  protocol family
- `boundaries`: unique boundary records
- `assets`: unique sensitive-asset records
- `threats`: unique threat records

## Boundary

- `id`: stable kebab-case identifier
- `description`: what crosses and which authority validates it
- `owner`: component responsible for enforcement

## Sensitive Asset

- `id`: stable kebab-case identifier
- `description`: secret, authority, or evidence requiring protection
- `owner`: component responsible for lifecycle and access

## Threat

- `id`: stable identifier
- `title`: concise abuse-case name
- `severity`: `high`, `medium`, or `low`
- `categories`: one or more issue-mandated abuse categories
- `boundaries`: one or more registry boundary identifiers
- `assets`: one or more registry asset identifiers
- `prevention`: nonempty owned controls
- `detection`: nonempty named signals or counters
- `containment`: nonempty failure boundary
- `evidence`: nonempty artifact or report authorities
- `tests`: one or more executable evidence references for every high-risk row
- `residual_risk`: absent for fully tested rows; otherwise explicit accepted
  disposition and authority, which S125 does not create

## Executable Evidence Reference

- `path`: tracked Rust source path beneath the repository
- `function`: exact non-ignored test function in that file
- `proves`: threat behavior demonstrated by the test

## Protocol Review

- `family`: exact shipped protocol-family identifier
- `threats`: one or more declared threat rows applicable to the family
- `tests`: one or more exact executable abuse-case references for the family

## Validation Invariants

- Identifiers are unique, nonempty, and use the closed syntax.
- References resolve to declared boundaries and assets.
- Every threat has all five control/evidence dimensions.
- Every high-risk threat has executable negative evidence.
- Every evidence path is tracked, inside the repository, and contains the exact
  attributed non-ignored test function.
- Protocol and direct proxy dependency inventories equal current source truth.
