# Research: Withdrawn Site Advisory Cleanup

## Decision 1: Withdraw the obsolete exception

The [GitHub advisory record](https://github.com/advisories/GHSA-fc5q-39xc-vjpf) states that GHSA-fc5q-39xc-vjpf was withdrawn on 2026-11-18 because it is not a security issue. [Issue #630](https://github.com/h8rt3rmin8r/fragcap/issues/630) separates that current disposition from the valid v0.11.2 publication record. At the start of S213, the repository still carried the exact exception in the site workspace, hosted workflow, offline validator, policy file, and current guidance.

**Selected**: Remove the exception and run the original unfiltered full-graph site audit. Preserve the locked picomatch version unless another independent reason requires a dependency change.

**Rejected**: Extend the exception until its 2026-12-01 expiry or upgrade picomatch merely to satisfy a withdrawn security report. Either would misstate the present advisory authority or add unrelated lock churn.

## Decision 2: Retain a small bypass guard

Deleting the former exception validator would also remove its check that the pnpm workspace has no additional audit ignores. An accidental future auditConfig subtree or audit disablement could make a green site job misleading.

**Selected**: Replace the exception-specific logic with an offline gate and a hosted pre-audit step that reject top-level `auditConfig` or `audit` keys in the site workspace. Accept LF and CRLF lines and canonical root block-mapping keys, using ASCII plain names matching `[A-Za-z][A-Za-z0-9_-]*` with optional whitespace before the colon. An indentless `- ` sequence item may follow a root mapping key such as `packages`; a standalone root sequence is refused. Refuse bare CR, root flow mappings, explicit keys, merge or alias constructs, tags, and quoted or escaped keys that a small line guard cannot classify. Nested overrides and comments remain accepted. Keep the same full audit command and low threshold. Controlled fixtures cover the four known ignore or disable forms, noncanonical root syntax, and bare CR, with an indentless `packages` sequence as a positive fixture. The guard owns only `site/pnpm-workspace.yaml`, the configuration surface implicated by this exception.

**Rejected**: Retain the old exception record as a sentinel or add a generic dependency-policy framework. The first preserves false current authority; the second adds architecture beyond this site-only cleanup.

## Decision 3: Keep source and history separate

The v0.11.2 release did ship when the advisory was published and disclosed a finite exception. That was a true observation at the time. Current maintainer guidance and the plan chronology must say the report was subsequently withdrawn.

**Selected**: Correct current guidance and add a dated S213 account. Leave the v0.11.2 handoff, published notes, release identity, and binaries unchanged. Its publication-time statements remain true in their historical context; current guidance explains the later withdrawal.

**Rejected**: Rewrite the release record to say that v0.11.2 had an unfiltered audit or never carried the exception.

## Decision 4: Separate Rust replacement

The nine heapless advisories and smoltcp parent-scoped exception are independent of the withdrawn npm GHSA. [Issue #640](https://github.com/h8rt3rmin8r/fragcap/issues/640) owns their replacement before 2026-12-15. S213 changes neither the Rust graph nor that policy.

## Evidence boundary

The official withdrawal establishes the status of this one GHSA. It does not establish that all future site dependency advisories are absent, that every pathological picomatch pattern has bounded runtime, or that the Windows product was field-tested. Only a fresh unfiltered audit on the final S213 head can support the new site audit result.
