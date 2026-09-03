# Protocol Classification Requirements Checklist

**Purpose**: Review classification honesty, security-boundary coverage, and authority separation before implementation

**Created**: 2026-09-03

## Requirement Completeness

- [x] CHK001 Are every published traffic family and packet-only boundary enumerated? [Completeness, Spec FR-002]
- [x] CHK002 Are trust, client-authentication, parsing, retention, routing, and writer reasons separately required? [Completeness, Spec FR-005 to FR-007]
- [x] CHK003 Are detailed evidence owners preserved when a stable classification is added? [Completeness, Spec FR-006]
- [x] CHK004 Are compatibility fact eligibility rules specified for every fact family? [Completeness, Spec FR-014]

## Requirement Clarity

- [x] CHK005 Are unknown, unsupported, and failed defined as mutually distinct states? [Clarity, Spec FR-003 and FR-008]
- [x] CHK006 Is inspectability separated from protocol detection and forwarding success? [Clarity, Spec FR-004 and FR-019]
- [x] CHK007 Is parser-failed prohibited from becoming a routing, trust, support, or artifact verdict? [Clarity, Spec FR-007]
- [x] CHK008 Is packet-only unrouted traffic distinguished from proxy loss? [Clarity, Spec Edge Cases]

## Requirement Consistency

- [x] CHK009 Do summary, application, manifest, and compatibility requirements share one versioned vocabulary without sharing authority? [Consistency, Spec FR-010 to FR-014]
- [x] CHK010 Do loss reconciliation requirements agree with retained-record and bounded-loss ownership? [Consistency, Spec FR-013]
- [x] CHK011 Do the exclusions preserve target scope, trust, and anti-instrumentation boundaries? [Consistency, Spec FR-019 and FR-020]

## Scenario And Edge Coverage

- [x] CHK012 Are partial parsing, unknown ALPN, pinning, upstream client authentication, truncation, and writer retirement addressed? [Coverage, Spec Edge Cases]
- [x] CHK013 Are invalid classification combinations and future schema versions addressed? [Coverage, Spec FR-009 and FR-017]
- [x] CHK014 Are conflicting observations retained without an invented winner? [Coverage, Spec FR-016]
- [x] CHK015 Are controlled security and transition tests mandatory without live targets or secrets? [Coverage, Spec FR-018]

## Acceptance Criteria Quality

- [x] CHK016 Can matrix completeness, transition coverage, fact suppression, and summary reconciliation be measured exactly? [Measurability, Spec SC-001 to SC-005]
- [x] CHK017 Is existing forwarding and evidence compatibility an explicit success condition? [Measurability, Spec SC-006]
