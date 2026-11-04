# Research: Session Causal Diagnostics

## Decision 1: Extend retained proxy connection identity

The native runtime already assigns a connection ID at accept and retains a bounded terminal diagnostic per closed task. The S187 inception collector separately binds exact socket and process evidence after collection. Extend the existing terminal diagnostic and keyed observation path rather than creating a second connection registry. A parallel registry would make loss accounting and identity joins disagree.

## Decision 2: Record protocol milestones at their owning boundary

The HTTP CONNECT path knows when authentication, requested authority, upstream TCP, tunnel response, client TLS, upstream TLS, protocol selection, and first request occur. Record reached and completed times there before the next fallible operation. Preserve a client TLS completion record before attempting upstream TLS. Infer neither a completed stage nor a TLS cause from a missing application event. SOCKS5, cleartext HTTP/2, and generic paths use the stages they actually observe.

## Decision 3: Stable failures and bounded evidence

Keep the existing stable code and category while adding boundary, alert when the TLS library exposes it, and numeric OS error where available. Do not copy arbitrary error text into the compact diagnostic. One fixed stage vocabulary and bounded terminal record per connection preserve finite storage; the existing dropped-oldest and unavailable counters account for lost detail. Local raw artifacts keep their existing authority.

## Decision 4: Phase timing belongs to the lifecycle owner

The session facade owns most phase origins and the CLI owns final bundle cleanup and manifest publication. Record monotonic elapsed at each actual phase boundary and project budget, elapsed, remaining, and status from those observations. Wall times may be retained for chronology but are not used to subtract budgets. A missing phase has no measured elapsed or remaining value.

## Decision 5: Scope of related issues

#551 is the proxy diagnostic defect and #553 has a remaining concrete reporting requirement after S189. They share the operator's session timeline and terminal projection. #542 needs post-release evidence to identify the publisher failure and stays open; #552 needs post-release managed handoff and cleanup observation and stays open. No new release or installed-title run belongs to this implementation slice.
