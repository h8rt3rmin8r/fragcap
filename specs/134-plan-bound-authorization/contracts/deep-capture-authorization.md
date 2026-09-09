# Deep Capture Authorization Contract

## Command surface

```text
fragcap deep-capture <SELECTOR> --launch [SESSION OPTIONS]
fragcap deep-capture <SELECTOR> --launch --authorize-stdin [SESSION OPTIONS]
fragcap --json deep-capture <SELECTOR> --launch --authorize-stdin [SESSION OPTIONS]
```

Interactive human output needs no authorization flag. `--authorize-stdin` selects the prompt-free exact-identifier input contract. Deep Capture `--trust-ca` and Deep Capture `--yes` are hidden migration inputs for one tagged release and always exit 2 with replacement guidance. They cannot be combined with or substitute for `--authorize-stdin`.

## Human plan

Human output prints one headed authorization plan before any session effect. It includes:

- the complete plan identifier;
- selected target and exact stored identity;
- requested and observed launch case plus the declared chain, fully resolved profile, exact client executable, and managed platform root and dispatch where applicable;
- native backend and fragcap versions;
- IPv4 or IPv6 loopback-only proxy scope, target-scoped environment, normalized bypass policy, and no system fallback;
- current-user Root-store action, exact certificate thumbprint and fingerprint, validity, and removal promise, or `none` for reachability;
- bundle and requested artifact paths plus sensitivity;
- exact millisecond effective launch, observation, shutdown, and cleanup deadlines;
- possible append-only compatibility facts;
- cleanup obligations and important refusal boundaries.

Values may wrap at narrow terminal widths but may not be truncated or omitted.

The interactive prompt names the exact plan identifier and accepts only the existing documented affirmative answer grammar on one complete LF-terminated line. Every other response declines. The plan and prompt writes must succeed, and the complete plan is flushed before input is read.

## Structured events

The JSON stream emits `deep_capture.authorization_plan` with `schema: 1`, every canonical field, and `plan_id` before reading authorization. It emits one `deep_capture.authorization` outcome with the same `plan_id` and one of `authorized`, `declined`, `invalid`, `closed`, `interrupted`, `drifted`, or `expired`. Secret key bytes, proxy credentials, unrelated targets, and captured payloads never appear.

JSON without `--authorize-stdin` exits 2 after request validation but before session effects, with guidance that structured execution requires the dedicated exact-plan input.

## Dedicated standard-input protocol

After the structured or human plan has been emitted and flushed, the caller writes:

```text
plan-v1:<64 lowercase hexadecimal digits>\n
```

Only the exact current identifier matches. The line ending is removed. No other leading or trailing whitespace is removed. Input is bounded to the identifier length plus the line ending. Extra bytes, a second line, EOF, malformed UTF-8, I/O error, abbreviation, case change, stale id, or different id refuses before effects. Identifier comparison is exact and authorization-sensitive.

The caller must keep the process running while reading the plan and returning the identifier. No plan private material is written for a later invocation.

Before any plan is emitted, a bounded read-only inspection checks prior Deep Capture owner records and resource journals. Pending recovery actions refuse with `fragcap doctor --fix` guidance. A new plan never authorizes replay, trust removal, artifact deletion, or registry retirement belonging to an older session. After approval, stored authority is checked immediately, then the fully resolved profile and managed launch are independently prepared and compared inside the facade's final resolver before endpoint selection. That exact retained Capture preparation is consumed during execution. The prepared CA must remain within its displayed validity period.

## Reachability calibration

Reachability uses the same session-plan authorization because it may start the local proxy, Capture, and managed target. Its trust action is `none`, its HAR and key-log requests are false, and its human wording does not ask the operator to approve certificate trust.

## Warm restart

`--restart-warm` first displays the existing no-process-control observation plan and may ask whether fragcap should wait while the operator closes the application normally. `--authorize-stdin` does not bypass that interaction, so noninteractive warm restart is refused. After cold state and exact target re-resolution, fragcap builds and requests authorization for one fresh complete session plan. No separate cold-plan or trust prompt follows.

## Exit contract

| Outcome | Exit | Effects |
| --- | --- | --- |
| Exact plan authorized and complete | 0 | Exact displayed plan only |
| Interactive decline or EOF | 0 | None |
| Missing structured authorization mode | 2 | None |
| Invalid, mismatched, stale, or replayed identifier | 2 | None |
| Legacy Deep Capture authorization flag | 2 | None |
| Plan drift before realization | 2 | None |
| Post-authorization runtime or cleanup failure | 1 | Existing truthful partial-result and cleanup contract |

## Compatibility boundary

The hidden migration behavior lasts for the first tagged release containing S134. The following tagged release may remove parser recognition. Doctor repair, bundle cleanup, and target reconciliation retain their own `--yes` contracts.
