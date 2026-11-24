# Research: Withdrawn Site Advisory Cleanup

## Decision 1: Withdraw the obsolete exception

The [GitHub advisory record](https://github.com/advisories/GHSA-fc5q-39xc-vjpf) states that GHSA-fc5q-39xc-vjpf was withdrawn on 2026-11-18 because it is not a security issue. [Issue #630](https://github.com/h8rt3rmin8r/fragcap/issues/630) separates that current disposition from the valid v0.11.2 publication record. At the start of S213, the repository still carried the exact exception in the site workspace, hosted workflow, offline validator, policy file, and current guidance.

**Selected**: Remove the exception and run the original unfiltered full-graph site audit. Preserve the locked picomatch version unless another independent reason requires a dependency change.

**Rejected**: Extend the exception until its 2026-12-01 expiry or upgrade picomatch merely to satisfy a withdrawn security report. Either would misstate the present advisory authority or add unrelated lock churn.

## Decision 2: Retain a small bypass guard

Deleting the former exception validator would also remove its check that the pnpm workspace has no additional audit ignores. An accidental future auditConfig subtree or audit disablement could make a green site job misleading.

**Selected**: Replace the exception-specific logic with offline and hosted guards that reject top-level `auditConfig` or `audit` keys in the site workspace. Accept LF and CRLF lines and canonical root block-mapping keys, using ASCII plain names matching `[A-Za-z][A-Za-z0-9_-]*` with optional whitespace before the colon. An indentless `- ` sequence item may follow a root mapping key such as `packages`; a standalone root sequence is refused. Refuse bare CR, root flow mappings, explicit keys, merge or alias constructs, tags, and quoted or escaped keys that a small line guard cannot classify. Nested overrides and comments remain accepted. Keep the same full audit command and low threshold. Controlled fixtures cover the four known workspace ignore or disable forms, noncanonical root syntax, and bare CR, with an indentless `packages` sequence as a positive fixture.

**Rejected**: Retain the old exception record as a sentinel or add a generic dependency-policy framework. The first preserves false current authority; the second adds architecture beyond this site-only cleanup.

## Decision 2a: Close the pinned pnpm configuration gap found in PR review

The second-round security review of [PR #642](https://github.com/h8rt3rmin8r/fragcap/pull/642) identified a separate source for advisory ignores. In pinned pnpm 10.26.0, [`getOptionsFromRootManifest`](https://github.com/pnpm/pnpm/blob/v10.26.0/config/config/src/getOptionsFromRootManifest.ts) reads `auditConfig` from `site/package.json#pnpm`. The [configuration loader](https://github.com/pnpm/pnpm/blob/v10.26.0/config/config/src/index.ts) applies that manifest setting before workspace settings, and the [audit handler](https://github.com/pnpm/pnpm/blob/v10.26.0/lockfile/plugin-commands-audit/src/audit.ts) filters CVE and GHSA advisories using the resulting value. Removing the workspace ignore alone therefore leaves a real bypass if a future change adds `pnpm.auditConfig` to the root package manifest. The current `site/package.json` has no such field.

Pinned pnpm also reads settings from [`.npmrc` files and environment variables](https://pnpm.io/10.x/settings). Its [audit option declarations and handler](https://github.com/pnpm/pnpm/blob/v10.26.0/lockfile/plugin-commands-audit/src/audit.ts) accept `dev`, `production`, `optional`, `fix`, `ignore-registry-errors`, `ignore`, `ignore-unfixable`, and `registry`. These can narrow dependency classes, suppress registry errors, or take a branch that returns before the normal advisory verdict. The explicit `--audit-level low` command protects only the severity threshold. Neither `.npmrc` exists in the current checkout, and the hosted site job currently declares no audit environment override.

**Selected**: Parse `site/package.json` as JSON in both gates and reject any root `pnpm.auditConfig` property, including escaped JSON key spellings. Reject `.npmrc` at the repository root or `site/.npmrc` because neither is needed by the current site audit. Check the hosted audit step's environment for pnpm/npm configuration overrides immediately before executing the unchanged pinned audit command, in that same step, so a variable scoped only to the audit step is observable. Add negative fixtures for CVE and GHSA manifest ignores, both `.npmrc` locations, and audit-step-only environment overrides; retain benign manifest siblings as positive cases.

**Rejected**: Rely on `pnpm config get` alone for effective settings, because the pinned loader assigns root-manifest options to runtime config without necessarily reflecting them in the raw configuration queried by that command. A separate prior hosted step is also insufficient for environment checks, because the audit step can declare its own variables. Replacing the pinned audit command or adding a general-purpose pnpm policy engine would enlarge S213 without addressing this bounded gap more directly.

## Decision 2b: Isolate hosted global configuration paths

The cross-artifact audit found an indirect route past a prefix-only environment check. Pinned pnpm's [`getConfigDir`](https://github.com/pnpm/pnpm/blob/v10.26.0/config/config/src/dirs.ts) follows XDG_CONFIG_HOME on Linux, and the [configuration loader](https://github.com/pnpm/pnpm/blob/v10.26.0/config/config/src/index.ts) reads the resulting global `rc`. Node also resolves its user home for user-level configuration. A workflow step could point one of those variables at a committed config file without setting an `npm_config_*` or `pnpm_config_*` variable.

**Selected**: In the hosted audit step, first reject direct pnpm/npm configuration environment variables, then create one fresh temporary directory and set HOME and XDG_CONFIG_HOME to it before running the unchanged pinned audit command. This removes the repository-controlled indirect config path in the hosted runner without claiming to inspect arbitrary global files on a developer workstation. Add workflow-wiring mutations for the creation, exports, and order; final-head hosted CI executes the resulting step.

**Rejected**: Only adding XDG_CONFIG_HOME and HOME to a denylist. HOME is normally present on Linux, and a path allowlist would be more fragile than running the audit with a fresh home and config directory.

## Decision 3: Keep source and history separate

The v0.11.2 release did ship when the advisory was published and disclosed a finite exception. That was a true observation at the time. Current maintainer guidance and the plan chronology must say the report was subsequently withdrawn.

**Selected**: Correct current guidance and add a dated S213 account. Leave the v0.11.2 handoff, published notes, release identity, and binaries unchanged. Its publication-time statements remain true in their historical context; current guidance explains the later withdrawal.

**Rejected**: Rewrite the release record to say that v0.11.2 had an unfiltered audit or never carried the exception.

## Decision 4: Separate Rust replacement

The nine heapless advisories and smoltcp parent-scoped exception are independent of the withdrawn npm GHSA. [Issue #640](https://github.com/h8rt3rmin8r/fragcap/issues/640) owns their replacement before 2026-12-15. S213 changes neither the Rust graph nor that policy.

## Evidence boundary

The official withdrawal establishes the status of this one GHSA. It does not establish that all future site dependency advisories are absent, that every pathological picomatch pattern has bounded runtime, or that the Windows product was field-tested. Only a fresh unfiltered audit on the final S213 head can support the new site audit result.
