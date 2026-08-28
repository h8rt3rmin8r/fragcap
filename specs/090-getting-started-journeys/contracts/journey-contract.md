# Contract: First Capture and Deep Capture Journeys

## Command Sequence

The guide uses the following concrete command forms in chronological order:

```text
fragcap doctor
fragcap targets
fragcap capture 1 --duration 5m --out first-capture.fcapng
fragcap targets show sample-target
fragcap deep-capture sample-target --launch --duration 5m --trust-ca --har --key-log
fragcap doctor
fragcap doctor --fix
```

The final `doctor --fix` is conditional. It is offered only when the post-session report names stale trust or session residue, and every action remains individually confirmed.

## Capture Journey Contract

- The operator starts in an elevated terminal after installing fragcap and the separately distributed Npcap driver in WinPcap API-compatible mode.
- `doctor` is read-only and reports Capture and Deep Capture readiness. Database and session-storage paths are observations or defaults, not required command arguments.
- `targets` discovers and lists stored targets with `#`, `TARGET`, `CAPTURE`, `ENGINE`, and `SENSITIVITIES`, followed by a labelled `Next command:` line.
- The documented file-mode Capture is bounded to five minutes and writes `first-capture.fcapng`.
- Capture records observed packets, payload bytes unless `--no-payload` is selected, and process attribution. It does not promise HTTP semantics or decryption of encrypted payloads.
- An unmodified pcapng analyzer opens the output and ignores annotations it does not understand.

## Deep Capture Eligibility Contract

- `sample-target` represents an already stored synthetic target.
- The operator runs `targets show` before Deep Capture and proceeds only with current facts for the exact launch case.
- The shipped real-target case is a cold Steam protocol launch. Warm Steam and direct-executable launch remain refusals.
- Unknown, stale, conflicting, or different-launch evidence stops the journey before Deep Capture side effects. The guide links to the compatibility reference and does not claim an automatic calibration workflow.
- mitmdump is the only shipped proxy backend. Deep Capture does not fall back to system-wide proxy configuration.

## Consent and Cleanup Contract

- `--launch` owns the target-scoped launch and is required for the documented real-target path.
- `--trust-ca` explicitly permits the prompt for a fragcap-owned current-user CA trust change. It does not make the change silent and does not bypass pinning.
- The session is bounded. Cleanup removes session-owned proxy and trust changes when possible and records the exact result in the manifest.
- A post-session `doctor` report is the read-only verification step. `doctor --fix` is used only for named residue and retains per-action confirmation.

## Output Authority Contract

- `.fcapng` remains packet truth.
- Application JSONL and optional HAR contain only semantics observed by the proxy and can be partial.
- `--key-log` requests proxy-owned TLS key-log material for analyzer correlation. It is sensitive and is not extracted from the target.
- The manifest indexes present, omitted, partial, and failed artifacts and the cleanup result.
- Proxy logs, process traces, compatibility updates, omission reasons, and artifact handling receive a concise first-run explanation in this guide. The full artifact and lifetime reference remains assigned to issue #248, and the guide does not present the current stale output-format page as that authority.

## Synthetic Content Contract

The page uses only `sample-target`, `sample.exe`, documentation address ranges, generic Windows user paths, and synthetic output. It contains no real title, account, local machine, private endpoint, host identifier, or captured payload.
