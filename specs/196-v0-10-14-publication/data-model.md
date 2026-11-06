# Data Model: v0.10.14 Publication

## Release Source

- Version: `0.10.14`
- Accepted merge revision: `5efb38ee93daff140e1ba79f57eea0a93d5b572f`
- Annotated tag name and object identifier
- Peeled source revision
- Candidate tree relationship
- Merge-check receipts

State: accepted merge -> annotated tag -> tagged workflow identity accepted.

## Tagged Workflow Receipt

- Workflow run and job identifiers
- Tag and source revision
- Identity result
- Windows package-certification result
- GitHub release result
- Protected crates.io deployment and approval result
- Terminal conclusion and timestamps

State: queued -> identity -> certification -> GitHub release -> protected approval -> registry publication -> complete.

## Public GitHub Asset Receipt

- Filename
- Public URL
- API size and digest
- Downloaded byte count
- Computed SHA-256
- Sidecar relationship and validation result
- Package-certification membership

Exactly six receipts exist: three primary files and three sidecars.

## Registry Archive Receipt

- Crate name
- Version `0.10.14`
- Yank state
- Registry checksum
- Public archive URL
- Computed archive SHA-256

Exactly ten receipts exist and each checksum agrees.

## Installed Observation

- Public MSI filename, size, and SHA-256
- Installed product version and embedded source identity
- Effective canonical target-store location
- Effective session-registry and native bundle locations
- Commands and product operations actually exercised
- Native artifacts retained
- Supported cleanup performed
- Remaining native state and reason
- Unverified or unavailable behavior
- Concrete defect issue references, if any

State: verified public MSI -> installed -> observed -> supported cleanup -> recorded.

## Publication Record

- Current public version and source
- Release URL and publication date
- Tag object and workflow receipt
- Six asset receipts
- Ten registry receipts
- Installed observation boundary
- Frozen published-review candidate
- Pages deployment receipt
- Issue, milestone, and project completion state

State: blocked before public readback -> authored -> reviewed -> merged -> publicly deployed.
