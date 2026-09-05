# Research: Native Windows Integration Matrix

## Decision 1: One closed registry is the release authority

**Decision**: Add a versioned JSON registry that names every required Windows row, its execution tier, capability predicate, exact expected outcome, production evidence, effects, cleanup obligation, and publication policy. `cargo xtask windows-integration` validates this registry during the ordinary gate.

**Rationale**: Workflow YAML and prose lists drift silently. A source-checked registry makes missing, duplicated, stale, skipped, or conditionally disabled rows executable failures and gives #334 one stable authority to consume.

**Alternatives considered**:

- Keep the matrix only in workflow YAML. Rejected because YAML cannot establish semantic coverage or bind rows to product inventories.
- Keep a manual checklist. Rejected because skipped rows and stale evidence would still pass review unnoticed.

## Decision 2: Separate hosted and physical execution without allowing skips

**Decision**: Every row is required, but rows have either `hosted` or `physical` execution authority. Hosted rows run on every pull request. Physical rows are satisfied by a separately executed report whose validated public-safe summary is committed with a bounded review expiry. The release gate fails when either authority is absent, stale, incomplete, or mismatched to the registry and product revision.

**Rationale**: GitHub-hosted Windows runners do not promise Npcap runtime, Wireshark, interactive UAC transitions, or the exact non-admin state. Pretending otherwise creates flaky or skipped evidence. This model retains real physical evidence without weakening the pull-request gate.

**Alternatives considered**:

- Install Npcap and Wireshark in CI. Rejected because Npcap redistribution and unattended installation conflict with project policy, and the runner image remains outside project control.
- Simulate all platform effects. Rejected because #327 explicitly requires real Windows integration evidence.
- Require a self-hosted runner. Rejected because no durable self-hosted runner is part of the repository contract and it would delay the release path.

## Decision 3: Break the #327/#329 dependency cycle at a staged layout

**Decision**: S129 builds the same feature-complete production executable intended for distribution, copies it into an isolated staged install layout, records its digest, and executes Windows rows against that copy. Issue #329 retains MSI/archive composition, installation, upgrade, repair, uninstall, checksum, signature, and distribution-content certification.

**Rationale**: Issue #327 says it depends on packaging while #329 says it depends on Windows integration. A staged binary proves package-independent runtime behavior and supplies #329 with a reusable matrix without prematurely claiming final packaging.

**Alternatives considered**:

- Merge #327 and #329. Rejected because it expands an XL integration slice into release packaging and weakens review focus.
- Defer all installed-layout evidence. Rejected because binary relocation and delay-load behavior are Windows integration concerns that packaging needs before it can close.

## Decision 4: Run existing authorities, add only missing Windows seams

**Decision**: The matrix invokes existing conformance, failure, Doctor, trust, ACL, process, capture, and analyzer paths. New code is limited to registry validation, finite orchestration, report hygiene, staged-binary identity, and narrowly missing real Windows integration probes.

**Rationale**: Reimplementing production behavior in a test harness would prove the harness. Existing S103 through S128 authorities are the intended seams and already own exact loss, cleanup, and evidence truth.

**Alternatives considered**:

- Build a second synthetic proxy. Rejected because it would not test the shipped backend.
- Add general product instrumentation. Rejected because S129 needs test authority, not a new runtime capability.

## Decision 5: Child execution is finite and hidden on Windows

**Decision**: The orchestrator launches direct argv vectors with redirected standard streams, fixed timeouts, bounded output capture, and `CREATE_NO_WINDOW` on Windows. It never invokes a shell, scheduled task, service, background monitor, or recurring automation.

**Rationale**: This satisfies the repository desktop rule, avoids quoting injection, prevents focus-stealing consoles, and ensures a hung child becomes an explicit failed row.

**Alternatives considered**:

- PowerShell orchestration. Rejected because matrix semantics belong in tested Rust and wrappers must remain thin.
- Background polling. Rejected because it obscures process ownership and previously caused unacceptable repeated activity.

## Decision 6: Raw evidence stays local; summaries are denylist validated

**Decision**: Raw command output and scratch bundles remain under ignored build output and are never uploaded. A derived summary contains closed typed fields only. Validation rejects secrets, raw certificates, payloads, host/user names, and absolute operator paths before upload or commit.

**Rationale**: Failure evidence must remain useful without publishing the sensitive observations the product exists to collect. Closed construction plus adversarial seeded tests is stronger than best-effort text redaction.

**Alternatives considered**:

- Upload raw logs with regex redaction. Rejected because unknown secrets and platform paths cannot be exhaustively recognized after the fact.
- Omit failure details. Rejected because it would make failed rows unactionable and conflict with truthful evidence.

## Primary References

- Microsoft, [Windows application security model](https://learn.microsoft.com/windows/security/application-security/application-control/user-account-control/how-it-works)
- Microsoft, [System certificate stores](https://learn.microsoft.com/windows-hardware/drivers/install/certificate-stores)
- Microsoft, [CreateProcess flags](https://learn.microsoft.com/windows/win32/procthread/process-creation-flags)
- GitHub, [GitHub-hosted runner images](https://docs.github.com/actions/using-github-hosted-runners/about-github-hosted-runners)
- Wireshark, [TLS key log file](https://wiki.wireshark.org/TLS)
- Npcap, [Guide and license](https://npcap.com/guide/)
