# Acceptance Checklist: Direct Route and Evidence

**Purpose**: Keep the four issue outcomes independently verifiable during S204.
**Created**: 2026-11-12
**Feature**: [spec.md](../spec.md)

## Route and correlation

- [x] Non-loopback IPv4 and IPv6 selected TCP exchange exact two-way bytes.
- [x] Active unrelated sentinels meet the current matrix bounds with diagnostics on and off.
- [x] Zero-byte, refusal, cancellation, and cleanup have distinct outcomes.
- [ ] Original-present and original-absent route facts reconcile without fabricated passive packets.
- [ ] Ambiguous, reused, incomplete, helper, and no-payload cases remain non-positive.

## Report and access

- [ ] Route event queue, writer, prefix, terminal, and manifest loss reconcile.
- [ ] Report fixtures include partial sources and leave the source unchanged.
- [ ] Access, Doctor, recovery, sharing, and human/JSON summaries remain consistent.
- [ ] No unrelated payload or secret receipt appears in exported diagnostics.

## Packaging and wording

- [ ] Required Windows and MSI/ZIP rows pass without skips.
- [ ] Packaged help, guided plan, resolver, and active docs agree.
- [ ] Any installed default-path claim records actual canonical native paths and retained artifacts.

## Notes

Check items only after reading actual command or test output. A repository test is not evidence of a private title or published release.
