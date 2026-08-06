# Reconnaissance protocol

**Status: open. Gates S09, S10, and S17.**

Six open questions from specification section 29 are answered here. They need
no fragcap code, no Rust toolchain, and nothing beyond an analyzer and the
tools already on a Windows machine. Budget one session per focal title.

This is the only work in the project that can invalidate work already
completed. Sections 11 and 12 of the specification are built on five working
assumptions (section 6.2), and each has a fallback path that is expensive to
retrofit and cheap to plan for. Running reconnaissance first means those
fallbacks get built only if they are actually needed.

## What is being asked

| ID | Question | Validates | Blocks |
| --- | --- | --- | --- |
| Q-1 | Are focal title gameplay flows attributable by 5-tuple? | A-1 | S10 |
| Q-2 | Is either focal title relay-tunneled? | A-3 | S10 |
| Q-3 | Connection lifetime distribution per title | A-2 | S10 |
| Q-4 | Exact process topology and image names per title | A-4 | S17 profiles |
| Q-5 | Is the launcher handoff visible on loopback? | A-5 | S09 |
| Q-6 | Transport encryption posture per title | expectations | Documentation |

## Prerequisites

- npcap installed **with loopback traffic capture support** and **WinPcap API
  compatibility mode**. Q-5 cannot be answered without the first, and the
  analyzer will not see a loopback adapter at all if it is missing.
- An analyzer capable of reading pcapng and applying display filters.
- An administrative shell. Socket table enumeration and capture both need it.
- A focal title installed, and the willingness to play it for long enough to
  produce a representative gameplay sample. Fifteen minutes of real gameplay
  beats an hour sitting in a menu.

## Safety and scope

This session is observation only. It uses no fragcap code and requires nothing
that constitution principle P-1 prohibits. Capture, socket table enumeration,
and process enumeration are all read-only system queries.

**Scrub everything before it lands in the repository.** Findings are recorded
without account identifiers, session tokens, or addresses attributable to the
operator. Do not commit the raw capture. Record derived facts (protocol,
direction, port ranges, endpoint ownership, timing distributions) rather than
the capture itself. If a fixture is genuinely needed for the S04 corpus, it is
scrubbed and reviewed before it lands.

## Tooling

`recon/Start-ReconSession.ps1` runs all four recorders described below and is
the intended way to execute this protocol. It performs the preflight checks,
starts the recorders in the right order, and tears them down cleanly.

```powershell
pwsh -File docs/plans/recon/Start-ReconSession.ps1 -Title eso
```

The manual procedure below documents what the script does and why, so that the
protocol survives the script and so a step can be run by hand when something
misbehaves.

## Procedure

Run these in order. Steps 1 and 2 must be started before the launcher, which
is the entire point.

### Step 1: start the process tree recorder

Before touching the launcher, begin recording process creation events so the
launch chain is captured at creation time rather than reconstructed afterward.
Reconstruction does not work; that is the problem fragcap exists to solve, and
it applies to reconnaissance too.

Any ETW consumer for `Microsoft-Windows-Kernel-Process` will do. Record, for
every process created during the session: process identifier, parent process
identifier, image name, full image path, command line, and creation timestamp.

Leave it running for the entire session, including shutdown.

### Step 2: start two captures

Start both before the launcher, and leave both running to the end.

- **Primary adapter**, no capture filter. Filtering early risks discarding the
  thing you did not predict.
- **Loopback adapter**, no capture filter. This one answers Q-5.

Note the wall-clock time at which each capture started. Q-3 and the correlation
between the two captures depend on it.

### Step 3: start the socket table sampler

Poll the socket table on a fixed interval for the whole session, recording each
socket when it first appears and again when it disappears.

**Use the direct IP Helper call, not the CIM path.** Measured on a busy machine
with roughly 1800 sockets, `Get-NetTCPConnection` costs 1400 to 2000
milliseconds per snapshot, while `GetExtendedTcpTable` costs 1 to 3. See PF-4 in
`recon/README.md`. A one second cadence is not a design choice on the CIM path;
it is the floor, and it is too coarse to characterize connection lifetimes. The
default is 250 milliseconds and the interval is a parameter.

Each sample records, for every TCP and UDP endpoint: protocol, local address and
port, remote address and port where one exists, state, and owning process
identifier.

Note that **UDP endpoints have no remote address or port**, because
`GetExtendedUdpTable` does not carry one. UDP attribution keys on the local
endpoint alone. See PF-3.

This log is the join key for everything else. Q-1 and Q-3 are answered from it
directly, and it is the only artifact that cannot be reconstructed afterward.

### Step 4: run the full session

Launch the title the way a user would, through the platform client. Then:

1. Let the platform client settle before starting the title, so its background
   traffic is characterized separately from the launch.
2. Launch the title. Let the publisher launcher fully load.
3. Authenticate. This exchange is frequently the most information-dense
   traffic of the whole session, and it is what naive detection misses.
4. Reach character or server selection, and make a selection.
5. Play for at least fifteen minutes of genuine gameplay, including a zone or
   map transition if the title has one. Transitions frequently open new
   connections and are where A-2 is most likely to fail.
6. Exit cleanly through the game's own menu.
7. Stop the captures, the sampler, and the process recorder, in that order.

## Answering each question

### Q-4: process topology (answer this first)

From the process tree recording, write down the full chain, in creation order:
platform client, publisher launcher, game client, and any helper or
anti-cheat processes. For each, record the image name, full path, whether it
persisted after the client started, and whether it exited before the client
did.

Everything else is easier once you know which process identifiers matter.

This directly produces the stage definitions and match rules for the game
profile (specification section 15.2), so record image names exactly, including
case and architecture suffix.

**A-4 holds** if each stage holds its own sockets. **A-4 fails** if stages
share an inherited handle, in which case roles collapse to a single attributed
process and role separation becomes advisory.

### Q-1: 5-tuple attributability

Join the capture against the socket table log. For each flow in the capture,
find the socket table sample covering its lifetime and read the owning process
identifier.

Record: what fraction of captured packets resolve to exactly one process, and
what fraction of gameplay traffic resolves to the game client rather than the
platform process.

**A-1 holds** if gameplay flows resolve to the client. **A-1 fails** if the
title multiplexes gameplay through a shared platform socket, in which case
attribution resolves to the platform process, and the profile marks that role
`attribution = "coarse"`.

### Q-2: relay tunneling

Look at the remote endpoints carrying gameplay traffic. Determine ownership of
each: publisher-operated, platform-operated, or a third-party network provider.

Relay-characteristic patterns to look for: gameplay traffic to an endpoint
owned by neither the publisher nor the platform; a single long-lived
connection carrying traffic whose volume and timing look like several logical
streams; gameplay endpoints that change on zone transition while the transport
connection does not.

**A-3 holds** if gameplay goes to publisher-operated endpoints directly.
**A-3 fails** if it is relayed, in which case the traffic attributes to the
relay-owning process, the profile declares the relay, and section 11 gains a
documented coarse-attribution path.

### Q-3: connection lifetime distribution

From the socket table log, compute for each connection the interval between
first and last appearance.

Record the distribution, and specifically: what fraction of connections lived
shorter than one poll interval, and what fraction of total packets rode on
those connections. The second number is the one that matters. Many short
connections carrying almost no traffic is a different situation from a few
carrying a lot.

**A-2 holds** if the packet fraction on sub-interval connections is below the
SC-7 threshold. **A-2 fails** if it is above, in which case the poll interval
drops, and event-driven socket tracking is promoted from the roadmap if that
is still insufficient.

Note that this measurement determines the default poll cadence in section 11.2,
so it is worth doing carefully rather than approximately.

### Q-5: loopback handoff visibility

Look at the loopback capture across the launch window, specifically between
the launcher's authentication completing and the client's first external
connection.

**A-5 holds** if the handoff is visible as loopback socket traffic. **A-5
fails** if the launcher and client communicate by named pipe, shared memory,
or command line argument, none of which a network capture tool can observe. If
it fails, that is a documented scope boundary rather than a defect, and it
should be stated plainly in the getting-started documentation so users do not
expect something the tool cannot deliver.

### Q-6: encryption posture

Sample payloads from each traffic class (launcher authentication, gameplay,
platform service) and characterize them: cleartext, recognizable TLS, or opaque
with high entropy.

This answers no architectural question. It sets user expectations, and
specification section 19.6 requires stating it plainly. A user who installs
fragcap expecting readable game packets and finds ciphertext has been
misled by omission.

Record the transport protocol per class as well (TCP, UDP, or both, and
whether anything runs over QUIC), because that determines the kernel filter
expressions in section 12.2.

## Recording the findings

Findings go into specification Appendix D, one entry per title, each recording:

- Title, date of the session, and the game build or client version
- Observed process topology, with exact image names
- Transport protocols per traffic class
- Endpoint ownership
- Encryption posture per traffic class
- Connection lifetime distribution
- **Verdict on each of A-1 through A-5: holds, fails, or inconclusive**

Update the section 29 table with the resolution of Q-1 through Q-6, and if any
assumption failed, record the consequence in the affected slice before that
slice starts.

An inconclusive result is a real result. Record it as inconclusive rather than
rounding it to whichever answer is more convenient; a wrong "holds" is far more
expensive than an honest "unknown", because it produces confident code built on
a false premise.
