# Contract: Production UX And Accessibility Audit Report

The S094 report is the reviewable interface between the audit run and maintainers who triage its findings.

## Required sections

The report MUST contain, in order:

1. Scope and production artifact identity.
2. Environment and exact commands.
3. Reconciled route inventory.
4. Route coverage matrix.
5. Keyboard and focus journey.
6. Semantic and automated accessibility results.
7. Responsive, zoom, theme, contrast, and complex-content results.
8. Search, internal-anchor, and external-link results.
9. Findings and dispositions.
10. Checks not run and confidence limits.
11. Gate results and conclusion.

## Result vocabulary

- **Pass**: The stated check ran and no defect was observed within its declared scope.
- **Fail**: The stated check ran and produced a finding.
- **Not run**: The check did not run. A reason and confidence impact are mandatory.
- **Not applicable**: The route or surface does not contain the examined element. The inventory still records that determination.

The report MUST NOT use pass for a check inferred from source, another viewport, another theme, or a different interaction mode.

## Coverage reconciliation

Every public route MUST appear exactly once in the inventory. Every documentation route MUST have desktop, 768 px, 320 px, semantic, and automated-result entries. Informational home routes MUST have desktop coverage and shared responsive coverage. The not-found probe is recorded separately and does not count as a public route.

The report MUST state:

```text
expected routes = generated routes = observed routes
```

or record each set difference as a finding.

## Finding record

Each finding MUST include:

```text
ID:
Title:
Severity:
Route or shared surface:
Viewport or access mode:
Reproduction:
Observed evidence:
User impact:
Disposition:
```

Severity follows FR-013. A material finding is any critical, high, or medium defect. Low findings may be recorded without a new issue only when the report provides a reasoned no-action disposition.

## Follow-up contract

Before creating an issue, record the overlap search and candidate results. A new issue MUST:

- describe one defect;
- include production-export reproduction;
- define testable acceptance criteria;
- receive the appropriate repository labels;
- receive the `Post-v0.7.0 documentation` milestone;
- link issue #249 and the S094 report.

S094 MUST NOT implement the follow-up inside the audit branch.

## Completion rule

The report is complete when route arithmetic balances, all required checks have one result, every material finding has one issue disposition, all limitations are explicit, and the full repository gate passes. Issue #255 remains open until issue #249 and any milestone-owned follow-ups reach their own closure conditions.
