# Anti-Cheat and Security

## Anti-cheat

Software that detects or prevents unauthorized modification of a game client
or its environment.

Modern implementations observe process handles, loaded modules, memory
integrity, and inline code modification.

{: .matters }
> fragcap is designed so that nothing it does resembles what anti-cheat
> watches for. Specification section 19.3's denylist exists because those
> techniques are the primitives detection systems monitor, and because none of
> them are needed for anything fragcap claims. Reconnaissance observed an
> anti-cheat launcher in one chain; fragcap records the relationship from
> creation-time telemetry and does not interact with the process.

**See also:** [Technique denylist](anti-cheat-and-security.md#technique-denylist)

## Technique denylist

The enumerated set of techniques fragcap will not use, regardless of
convenience. Each one is a way of reaching into a target process or interposing
on its traffic, and fragcap's whole posture is that it does neither. Every item
below therefore names a permitted alternative that recovers the same
information by observing from the outside.

- **Packet interception drivers.** A driver placed in the network stack to
  intercept or filter packets while they are in flight. Off-limits because it
  can alter or drop traffic, which is the opposite of observing it. fragcap
  reads from the NDIS capture driver, which copies packets without sitting in
  their path.
- **Code injection.** Writing fragcap's own code into a target process so it
  runs inside that process. Off-limits because it modifies the very process
  fragcap only means to name, and it is a primitive anti-cheat watches for
  directly. There is no alternative; it is out of scope.
- **Function hooking.** Redirecting a target's own function calls through
  fragcap's code. Off-limits for the same reason, reaching inside the process.
  Attribution comes from the socket table, read entirely from outside.
- **Process handles carrying memory-read rights.** Opening a handle to a target
  with the access needed to read its memory. Off-limits because reading another
  process's memory is exactly what detection systems look for. fragcap recovers
  which process owns a flow from creation-time ETW telemetry, and any handle it
  does open declares query-only rights at the call site.
- **Layered service providers.** Inserting fragcap into the Winsock call chain
  that a socket's traffic passes through. Off-limits because it interposes on
  live traffic. The socket table yields the same process-to-flow association
  after the fact, without being in the path.
- **Executable image modification.** Patching a target's binary, on disk or in
  memory. Off-limits because it changes the thing fragcap claims only to
  observe. There is no alternative; it is out of scope.

{: .matters }
> Constitution principle P-1 makes the list absolute, and it is enforced at
> three points: as a constitution principle inherited by every agent session,
> as a dependency policy checked in continuous integration, and as a code
> review gate on process handle access rights. Naming each technique precisely
> is deliberate. It records, in plain terms, exactly what fragcap will not do,
> which is what lets the project be published as a passive observer rather than
> read as a tool for reaching into a game.

**See also:** [Anti-cheat](anti-cheat-and-security.md#anti-cheat)
