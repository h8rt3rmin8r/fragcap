# Research: S165 Route Owner Lifetime

## Observed defect

The native session starts a managed root with child-scoped proxy variables, while `Session::stop` stops the proxy as soon as packet capture returns. `LibraryLaunchLease::cleanup` then reports `NotNeeded`. Managed Steam is long lived, so later title launchers can inherit an endpoint that no longer exists. The v0.10.3 incident and normal Steam-exit recovery are recorded in #452; the same code remains in v0.10.4. This is a strong causal inference without reading target-process environment.

## Decisions

1. **Use a bounded route-owner release phase before proxy stop.** The operator closes remaining applications normally while the proxy and trust are still active. Query-only process inventory checks declared image names. Successful absence is positive release evidence; a remaining image or unavailable inventory at the deadline is unresolved and makes the session partial. A foreign same-named image can extend the wait or produce a conservative false unresolved result, but cannot falsely authorize cleanup.
2. **Keep an exact deadline in the authorized plan.** A new route-owner release budget is separate from observation, proxy shutdown, and cleanup. It defaults to two minutes for an ordinary operator shutdown action. The session does not charge this interval against the finite proxy stop budget.
3. **Apply the barrier to all child-environment managed cases.** Steam, direct, and each declared publisher stage can retain the inherited route. A failed capture after a launch attempt still enters the barrier. If no launch was attempted, the barrier reports not needed.
4. **Do not control the target.** Process termination, target handles with memory rights, process environment writes, and system proxy mutation would violate project constraints. A detached always-on guardian would require moving the proxy, session secrets, trust, and artifact ownership into a separate process. That is a much larger architecture change with new crash and authorization boundaries. The bounded release plus explicit recovery meets #452 without inventing process control or claiming that abrupt fragcap termination is solved.
5. **Preserve failure truth.** Expiry does not produce a successful cleanup claim. The human report must tell the operator to exit the application normally and relaunch outside the old session. The versioned cleanup stream and terminal report carry the unresolved result before proxy stop. The read-only release check creates no external resource obligation in the resource journal.

## Existing authorities

- `crates/fragcap/src/deep_capture/session.rs` orders Capture, proxy, route, trust, and bundle cleanup.
- `crates/fragcap-cli/src/commands/deep_capture.rs` supplies the shipped managed-launch and query-only ToolHelp adapters.
- `crates/fragcap/src/managed_launch.rs` puts the environment on the root and does not retain a process controller.
- Constitution P-1 permits query-only enumeration and forbids target memory access; P-9 requires truthful outcomes.

## Verification boundary

Use pure owner-inventory decisions and controlled adapter lifecycle tests, then the full repository CI gate. No installed game, account, real title, or live Deep Capture is part of this work slice's acceptance.
