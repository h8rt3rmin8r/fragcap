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
convenience: packet interception drivers, code injection, function hooking,
process handles carrying memory-read rights, layered service providers, and
executable image modification.

{: .matters }
> Constitution principle P-1 makes the list absolute, and it is enforced at
> three points: as a constitution principle inherited by every agent session,
> as a dependency policy checked in continuous integration, and as a code
> review gate on process handle access rights.

**See also:** [Anti-cheat](anti-cheat-and-security.md#anti-cheat)
