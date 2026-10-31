# Security requirements checklist: S185

- [x] Is the machine-wide trust scope and its duration explicit before installation? (FR-001, FR-009)
- [x] Are public certificate and private signing key stored separately with exact access rules? (FR-003, FR-004)
- [x] Is machine DPAPI insufficient without ACLs stated as a requirement? (FR-004)
- [x] Are exact identity and unrelated-certificate refusal conditions specified for cutover, rollback, repair, and remove? (FR-002, FR-005, FR-009)
- [x] Is the ordinary read-only health result distinct from a privileged signing proof? (FR-006, FR-007)
- [x] Does every Deep Capture entry path have a pre-effect refusal contract? (FR-007)
- [x] Are package, child environment, log, and artifact key-exclusion requirements explicit? (FR-004, SC-005)
