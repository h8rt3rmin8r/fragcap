# fragcap

Passive, process-attributed network capture for game clients on Windows.

[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![Status](https://img.shields.io/badge/status-pre--implementation-orange.svg)](docs/plans/README.md)

> **Status: pre-implementation.** This repository currently holds the
> specification, the constitution, and the plan. There is no Cargo workspace
> and no Rust code yet; slice S01 creates them. Nothing here is installable or
> runnable. See [`docs/plans/README.md`](docs/plans/README.md) for what is
> being built and in what order.

## The problem

Packet capture is solved. Attribution is not.

Capture on every mainstream operating system happens at the network driver
layer, below the socket layer. By the time a packet reaches a capture handle,
the operating system has already discarded the association between that packet
and the process that produced it. A capture file records that a UDP datagram
left the machine on port 51834. It does not record that the game client sent
it.

Recovering that association means joining each packet's 5-tuple against a
separately maintained table of open sockets and their owning process
identifiers. Games make that join harder than average in three specific ways:

**Clients are launched indirectly.** A title acquired through Steam is
typically started by Steam, which starts a publisher launcher, which starts the
actual client. The launcher performs authentication and server selection, then
hands off. Detection that waits for the client process to appear has already
missed the authentication exchange, which is frequently the most
information-dense traffic of the entire session.

**Process ancestry is unreliable on Windows.** Parent process identifiers are
recorded but not maintained, and the values are recycled. By the time a client
process is observed, its recorded parent may be dead and the number reassigned
to something unrelated. Reconstructing the launch chain after the fact does not
work.

**Platform services generate constant background traffic.** A persistent
publisher client maintains connections for presence, overlay, entitlement
checks, and telemetry for the whole session. Capturing a process tree without
discrimination buries gameplay traffic under service noise.

fragcap addresses all three: an ETW-backed process watcher that builds the
creation-time process tree so the launcher is attached before it hands off,
socket-table attribution that survives identifier recycling, and profile-driven
roles that separate launcher, client, and platform-service traffic.

## What it will do

- Attribute every captured packet to the process that produced it, from
  launcher start through client exit.
- Write `.fcapng`, an extended pcapng profile carrying attribution in Enhanced
  Packet Block options. **Unmodified analyzers read it as ordinary pcapng** and
  ignore the annotations. Compatibility is never traded for richness.
- Stream live to named pipes, Unix domain sockets, or TCP, including directly
  into an analyzer through the extcap interface.
- Maintain a rolling in-memory window and dump it on trigger, so you do not
  have to predict when the interesting event happens.
- Distinguish launcher, client, and platform-service traffic by role, and let
  you capture a subset.
- Count every discarded packet in a named counter and surface it. A capture
  tool that loses data without saying so produces conclusions the user cannot
  check.

The library is the product. The command line tool is one consumer of it, and
the shell wrappers are consumers of the tool. Anything reachable through the
CLI is reachable through the public Rust API.

Planned invocations, from specification section 17.3:

```bash
# Bounded capture, launched by fragcap, written to a dated file
fragcap run --profile eso --launch --duration 30m

# Client traffic only, streamed to an analyzer through a named pipe
fragcap run --profile div2 --mode stream --roles client --sink pipe:fragcap

# Rolling ten-minute window, dumped on interrupt
fragcap run --profile eso --mode ring --ring 10m --out captures/eso.fcapng

# Ad-hoc capture of a running process, no profile
fragcap tap --process eso64.exe --duration 5m
```

## What it will not do

The non-goals matter as much as the goals, and they are enforced by
[constitution principle P-1](.specify/memory/constitution.md), not merely
documented:

- **No process injection, memory reading, or hooking** of any target.
- **No packet modification, injection, or replay** against a live server.
- **No game-specific protocol logic in core.** Dissectors are a plugin seam.
- **Not a cheat, not a proxy, not a latency optimizer.**

Where a game uses transport encryption, payloads are captured as ciphertext.
fragcap does not decrypt them and does not attempt key recovery. What you get
is timing, sizing, endpoints, and attribution. That is stated plainly here so
expectations are correct before you install anything.

## Prerequisite: npcap

**fragcap requires [npcap](https://npcap.com) to be installed separately,
before any other step.**

npcap is not redistributable under its standard license, so fragcap does not
and will not bundle it. fragcap detects it and reports its absence with the
download location; it never downloads, installs, or invokes an installer.

Two installation options are required and **not both default**:

| Option | Why it is required |
| --- | --- |
| Support loopback traffic capture | The launcher-to-client handoff and platform service chatter are local, and invisible on a normal adapter |
| Install in WinPcap API-compatible mode | The `pcap` crate links against the WinPcap-compatible interface |

`fragcap doctor` will verify each and name the specific option when it is
missing.

Building fragcap additionally requires the npcap Software Development Kit,
which is likewise not redistributed and is acquired at build time.

Capture requires administrative privilege.

## Repository map

```text
.specify/memory/constitution.md   Governing principles, versioned
docs/fragcap-specification.md     Architecture of record
docs/fragcap-spec-outline.md      Navigable map of the specification
docs/plans/README.md              Slice ordering S01 to S18
docs/plans/reconnaissance.md      Protocol for open questions Q-1 to Q-6
docs/brand/README.md              Brand guardrails, identity pending
specs/                            Spec Kit feature slices
.agents/skills/                   Vendored portable agent skills
skills/                           First-party agent skills
AGENTS.md                         Canonical agent instructions
CONVENTIONS.md                    Mechanical rules for every file
CONTRIBUTING.md                   Contributor workflow
```

## Documentation and development

The specification is the architecture of record, and every feature traces to
it. Development runs through [GitHub Spec Kit](https://github.com/github/spec-kit):
each slice is spec'd, planned, and analyzed before implementation, and lands as
a numbered `specs/NNN-slug/` directory.

Four agent surfaces are installed (Claude Code, Codex, Cursor, opencode), all
driving the same agent-neutral `.specify/` engine. See
[`AGENTS.md`](AGENTS.md).

Contributions are welcome under the Apache-2.0 inbound-equals-outbound model;
no contributor license agreement is required. See
[`CONTRIBUTING.md`](CONTRIBUTING.md).

## Versioning

Semantic versioning. Below 1.0.0, minor version increments may carry breaking
changes. This is stated here rather than discovered.

The `.fcapng` annotation profile carries its own version, independent of the
fragcap version, because the annotation grammar and the software change on
different schedules.

## License

Apache License, Version 2.0. See [`LICENSE`](LICENSE) and [`NOTICE`](NOTICE).

Apache-2.0 is chosen over the Rust ecosystem's conventional
`MIT OR Apache-2.0` deliberately: the express patent grant has real value in a
domain where technique patents exist, and the attribution requirements are
appropriate for a project whose correctness claims matter.

## Disclaimer

fragcap is an independent open source project. It is not affiliated with,
endorsed by, or sponsored by any game publisher, developer, distribution
platform, or software vendor. Product names, game titles, company names, and
trademarks appearing in this repository or its documentation are the property
of their respective owners and are used solely to identify software that
fragcap has been tested against or designed to observe. Their mention implies
no relationship between those owners and this project.

fragcap is built and published for demonstration, research, and educational
purposes. It is a passive observation tool: it does not modify, inject, or
replay network traffic, and it does not read, write, or attach to the memory of
any other process. The authors cannot control what third parties choose to do
with an open source utility. Use of fragcap may nonetheless violate the terms
of service of a given game or platform, and determining that is the user's
responsibility rather than the project's.

fragcap is provided "as is", without warranty of any kind, as set out in
sections 7 and 8 of the Apache License, Version 2.0.
