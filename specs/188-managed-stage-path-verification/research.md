# Research: S188 Managed stage path verification

## Event-to-process identity

**Decision**: For a basename-only managed stage candidate, use the existing query-only `ProcessIdentity::observe` process-object path and creation time at event handling. Accept a verified path for stage matching only when PID agrees, canonical image basename agrees with the ETW basename, account and session agree with the initiating session, and the OS creation instant precedes the ETW start timestamp by no more than one second. Preserve the raw event separately. Query only declared managed stage names, leaving ordinary Capture and unrelated ETW processes untouched.

**Rationale**: ETW and `GetProcessTimes` use the same Windows system-time epoch. A later process that reused the PID has a creation instant after the older ETW event and is rejected even if reuse occurs inside the one-second lag bound. The existing shutdown receipt uses a symmetric five-second proximity check and cannot alone authorize a stage match. Query-only rights are already the reviewed P-1 technique; no target memory rights or process control is added.

**Alternatives considered**: Removing exact `path_regex` regresses S187 ownership. Parsing the command line cannot prove the loaded image. Replacing the event's image with a queried path would falsify the source observation. Querying every ETW process at the callback would add system-wide work and could delay event delivery. Accepting the shutdown receipt without an event join would confuse control and matching authority.

## Stage and limitation representation

**Decision**: Keep `ProcessEvent::Started.image` as the exact source text. Attach a separately verified optional path to the corresponding process-tree node before the stage matcher runs; exact path predicates evaluate that path when present and otherwise retain existing full-path event behavior. Managed candidate failures receive a bounded reason code in process evidence and a concise diagnostic, with no false stage transition.

**Rationale**: The process tree already holds one generation per creation event and centralizes matching. A separate field preserves provenance and avoids modifying the public `ProcessEvent` variant or broadening ordinary Capture's process access. A bounded reason inventory lets timeout diagnostics identify path or ancestry uncertainty without exposing private title paths.

**Alternatives considered**: A parallel matcher in the CLI would duplicate stage semantics and risk diverging from profile matching. Overwriting the tree image would make a queried value look ETW-observed. A global basename fallback would admit wrong-path processes.

## Capture and CLI outcomes

**Decision**: Reuse the production orchestrator and session gate for the controlled Windows row, with a synthetic owned chain and real loopback traffic; verify the output with unmodified TShark. Preserve `CaptureOutcome.stop_reason` on every no-acquisition path and map it to the composed Deep Capture error and structured result instead of replacing it with a numeric exit alone.

**Rationale**: A matcher-only unit test would miss the S187 synthesis-to-acquisition gap and a valid zero-frame pcapng. The existing outcome already has a typed stop reason, but the no-acquisition live branch currently discards it and the composed command emits a generic error. Keeping the native packet, proxy, manifest, and cleanup authorities separate avoids inventing traffic from proxy evidence.

**Alternatives considered**: A private installed-title retest is unnecessary as a repository implementation gate and would disclose sensitive evidence if copied into CI. Treating proxy authentication as packet ownership would repeat the field-report error. Changing the pcapng format is unrelated.
