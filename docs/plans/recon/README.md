# Reconnaissance tooling

Scripts supporting the protocol in
[`../reconnaissance.md`](../reconnaissance.md), which answers open questions
Q-1 through Q-6.

These live here rather than in `scripts/` because that directory is reserved by
specification section 21.1 for shell wrappers and linters, and is owned by
slice S01. This tooling is neither: it is a working artifact of a documented
plan, and it stops being run once Appendix D is populated. If it proves durable
past that, promoting it into `scripts/` is a spec deviation to record at the
next version.

| Script | Purpose |
| --- | --- |
| `Start-ReconSession.ps1` | Runs all four recorders for one session |

## Running a session

Administrative privilege is required, for the process tree recorder only.

```powershell
pwsh -File docs/plans/recon/Start-ReconSession.ps1 -Title eso
```

Start it before launching the platform client, and press Ctrl+C after the game
has exited cleanly. Output lands in `captures/recon/<title>-<timestamp>/`, which
is gitignored.

`-Help` prints the full parameter documentation.

## Output

| File | Contents |
| --- | --- |
| `session.json` | Manifest: times, tool versions, interface, poll interval |
| `processes.jsonl` | Process start and stop events with image path and command line |
| `sockets.jsonl` | Delta-encoded socket table: one record per open, one per close |
| `primary.pcapng` | Capture on the primary adapter |
| `loopback.pcapng` | Capture on the npcap loopback adapter |

Expect roughly 2 GB for a 45 minute session at the default snap length, most of
it capture files. Captures are deliberately unfiltered: filtering early risks
discarding the thing you did not predict.

**These artifacts contain addresses, and for some titles session identifiers.
They are gitignored and MUST NOT be committed.** Only derived findings go into
Appendix D, scrubbed per the protocol.

## Preflight findings

Established while building and validating the tooling, before any gameplay.
Each of these bears on the specification and should be promoted to section 29
at the next version.

### PF-1. npcap configuration is readable from the registry

`HKLM\SOFTWARE\WOW6432Node\Npcap` exposes `WinPcapCompatible` and `AdminOnly` as
DWORD values, and the loopback adapter appears in the interface list as
`\Device\NPF_Loopback`.

Both options that section 20.3 requires are therefore verifiable without opening
a capture handle. This is a direct input to `fragcap doctor` (section 26.3),
which must name the specific missing option rather than reporting a generic
failure.

### PF-2. Capture does not always require administrative privilege

Section 19.5 states that fragcap requires administrative privilege for capture
handle creation and ETW session consumption. The first half is conditional
rather than absolute: when npcap is installed with `AdminOnly = 0`, an
unprivileged process opens a capture handle successfully. This was confirmed by
capturing 576 loopback packets with zero drops from a non-elevated shell.

ETW consumption does still require elevation.

Consequence: `doctor` should report the two privilege requirements separately,
and the getting-started documentation should not tell every user to elevate when
their npcap install may not require it for capture. A user who only needs
capture, on an `AdminOnly = 0` install, does not need an elevated shell.

### PF-3. The UDP socket table carries no remote endpoint

`GetExtendedUdpTable` exposes local address, local port, owning process
identifier, and creation time. It carries no remote address or port, because a
UDP socket generally has none. The CIM projection (`MSFT_NetUDPEndpoint`) shows
the same fields.

**UDP attribution therefore keys on (protocol, local address, local port), not
on a 5-tuple.** Since gameplay traffic is predominantly UDP in most titles, this
is load-bearing rather than an edge case.

This is not a defect and not a surprise to the design, but the framing in
sections 2.2 and 11.1, which describes the mechanism as joining each packet's
5-tuple against the socket table, is accurate only for TCP. The `FlowKey` type
in section 8.4 needs to express an asymmetric join: full 5-tuple for TCP, local
endpoint for UDP. Worth settling before S02 defines the type, not after.

### PF-4. Socket table access cost differs by three orders of magnitude

Measured against a table of roughly 1800 rows on a busy machine:

| Access path | Time per snapshot |
| --- | --- |
| `Get-NetTCPConnection` (CIM) | 1400 to 2000 ms |
| `netstat -ano` | 30 to 40 ms |
| `GetExtendedTcpTable` (direct) | 1 to 3 ms |

fragcap's selected crate wraps the direct call, which is the fast path.

Consequence for section 11.2: the snapshot cadence can be far tighter than a
naive reading suggests. A 50 or 100 millisecond interval is affordable, which
makes the race window in section 11.3 correspondingly smaller and reduces how
much weight assumption A-2 has to carry. Before concluding from a Q-3
measurement that polling is insufficient and event-driven tracking is needed,
check whether simply shortening the interval resolves it.

The recon sampler polls at 250 ms by default, and the interval is a parameter so
the effect can be measured directly.

### PF-5. TCP socket creation timestamps are available

`MIB_TCPROW_OWNER_MODULE` carries `liCreateTimestamp`, surfaced by
`Get-NetTCPConnection` as `CreationTime` and confirmed populated and accurate.

This matters for section 11.3. A socket observed in a snapshot reports when it
was created, so attribution can be back-dated to socket creation rather than
starting from first observation. The race window then covers only sockets that
both open and close within a single interval, rather than every socket younger
than one interval.

The recon sampler uses the `OWNER_PID` table variant, which omits the timestamp,
because the delta encoding already brackets lifetimes to within one interval.
fragcap itself should evaluate `OWNER_MODULE`, which carries the timestamp and
the owning module path at the cost of a larger, variable-length row.

## Development notes

Two defects were found by smoke testing rather than by review, both worth
remembering because both produce plausible-looking success:

**An array `-notmatch` is a filter, not a boolean.** `$interfaces -notmatch 'x'`
returns every element that does not match, which is a non-empty array, which is
truthy. The loopback preflight check inverted itself and rejected a correctly
configured machine. Use `-not ($array -match 'x')`.

**`Write-Log ""` throws.** The house fixture declares `Message` as
`Mandatory = $true`, and an empty string cannot bind to a mandatory parameter.
Three cosmetic spacer calls aborted the recording loop one second after it
started, after every recorder had reported success. The session looked healthy
in the log and produced a zero-byte socket table.

The second is the more instructive one: every recorder logged `SUCCESS`, the
script exited 0, and the output directory contained files. Only the byte counts
revealed that nothing had been recorded. This is exactly the failure mode
constitution principle P-4 exists to prevent in fragcap itself, met here in the
tooling built to validate it.
