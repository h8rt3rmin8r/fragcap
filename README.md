<h1 align="center">
  <a href="https://fragcap.com">
    <picture>
      <source media="(prefers-color-scheme: dark)" srcset="brand/logos/png/fragcap-horizontal-dark-2400.png">
      <source media="(prefers-color-scheme: light)" srcset="brand/logos/png/fragcap-horizontal-light-2400.png">
      <img alt="fragcap" src="brand/logos/png/fragcap-horizontal-dark-2400.png" width="620">
    </picture>
  </a>
</h1>

<p align="center">
  <em>Passive, process-attributed network capture for game clients on Windows.</em>
</p>

<p align="center">
  <a href="https://github.com/h8rt3rmin8r/fragcap/actions/workflows/ci.yml"><img alt="CI" src="https://img.shields.io/github/actions/workflow/status/h8rt3rmin8r/fragcap/ci.yml?branch=main&label=CI&logo=github"></a>
  <a href="https://github.com/h8rt3rmin8r/fragcap/releases"><img alt="Release" src="https://img.shields.io/github/v/release/h8rt3rmin8r/fragcap?label=release&color=27C7E7&sort=semver"></a>
  <a href="LICENSE"><img alt="License: Apache-2.0" src="https://img.shields.io/badge/license-Apache--2.0-blue.svg"></a>
  <a href="rust-toolchain.toml"><img alt="Rust 1.82+" src="https://img.shields.io/badge/rust-1.82%2B-dea584?logo=rust&logoColor=white"></a>
  <img alt="Platform: Windows" src="https://img.shields.io/badge/platform-Windows-0078D6?logo=windows&logoColor=white">
  <a href="https://fragcap.com"><img alt="Docs: fragcap.com" src="https://img.shields.io/badge/docs-fragcap.com-27C7E7"></a>
</p>

---

fragcap watches the network traffic leaving and entering your machine and labels
every connection with the program that made it. Ordinary capture tools can tell
you a conversation happened; fragcap tells you which process on your machine had
it, even when that process is a game client started indirectly by a launcher.

It only observes. It never modifies, injects, or replays traffic, and it never
reads or attaches to the memory of another process. It writes an extended
[pcapng](https://pcapng.com) file that unmodified analyzers such as Wireshark
read as an ordinary packet trace.

Two prerequisites sit outside fragcap. Live capture needs the
[npcap](https://npcap.com) driver installed separately (fragcap detects it and
never installs, downloads, or bundles it), and reading a capture needs
[Wireshark](https://www.wireshark.org) or another pcapng-aware analyzer. You
capture with fragcap, then open the result in Wireshark.

**The full documentation, including a first-run guide, lives at
[fragcap.com](https://fragcap.com).**

## Who is this for?

- **Curious players and tinkerers** who want to see what a game client actually
  talks to. Start at [the getting-started guide](https://fragcap.com/docs/getting-started);
  it walks the whole first capture. You do not need to read Rust to use the tool.
- **Network and security researchers** who need attribution-grade captures:
  every packet tied to its owning process and role, in a format existing
  analysis tooling already understands.
- **Rust developers and contributors.** The library is the product; the
  command-line tool is one consumer of it. See [Building](#building) and
  [`CONTRIBUTING.md`](CONTRIBUTING.md).

## Quick links

| | |
| --- | --- |
| Documentation and first-run guide | [fragcap.com](https://fragcap.com) |
| Glossary of every term | [fragcap.com/docs/glossary](https://fragcap.com/docs/glossary) |
| Architecture of record | [`docs/fragcap-specification.md`](docs/fragcap-specification.md) |
| Contributor workflow | [`CONTRIBUTING.md`](CONTRIBUTING.md) |
| Windows installer | [Windows installer (MSI)](#windows-installer-msi) |
| Releases | [github.com/h8rt3rmin8r/fragcap/releases](https://github.com/h8rt3rmin8r/fragcap/releases) |

## Status

All eighteen roadmap slices (**S01 through S18**) are complete and merged: the
crate graph, the capture pipeline, both output writers, the profile schema, the
socket-table attributor, the ETW process watcher, transports and streaming
sinks, ring mode, Steam integration and managed launch, the extcap analyzer
integration, the shell wrappers, and the documentation site. **v0.2.0 is the
first public release** and packages the whole roadmap; see the
[Releases](https://github.com/h8rt3rmin8r/fragcap/releases) page.

Live capture requires the npcap driver (below) and administrative privilege; the
socket-table attribution path and the offline replay substrate run without
either.

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

## What it does

- Attributes captured packets to the process that produced them, from launcher
  start through client exit, and marks the ones it cannot resolve as
  unattributed rather than dropping them or guessing. A frame with no flow key
  or a socket-table miss is retained and labeled, not silently assigned an
  owner.
- Writes `.fcapng`, an extended pcapng profile carrying attribution in packet
  comments. **Unmodified analyzers read it as ordinary pcapng** and ignore the
  annotations. Compatibility is never traded for richness.
- Streams live to named pipes or TCP, including directly into an analyzer
  through the extcap interface.
- Maintains a rolling in-memory window and dumps it on trigger, so you do not
  have to predict when the interesting event happens.
- Distinguishes launcher, client, and platform-service traffic by role, and
  lets you capture a subset.
- Counts every discarded packet in a named counter and surfaces it. A capture
  tool that loses data without saying so produces conclusions the user cannot
  check.

The command-line tool exposes one capture verb, `capture`, alongside `targets`,
`technologies`, `steam`, `catalog`, `schema`, `doctor`, and `extcap`. Anything
reachable through the CLI is reachable through the public Rust API.

Start with `fragcap targets`: it lists the capturable titles on your machine and
ends by naming the next command.

```bash
# The hero command: discover and list your own targets, then follow the hint
fragcap targets
#   #  TARGET                     CAPTURE          KNOWN
#   1  the_elder_scrolls_online   ready            no online mode recorded
#   2  the_division_2             ready            Denuvo, EasyAntiCheat
#
#   fragcap capture 1

# Register an installed Steam title, or author one interactively
fragcap targets add --steam 306130
fragcap targets add "My Game" --exe game.exe        # asks who holds the sockets
```

The row numbers `fragcap targets` shows are the numbers `fragcap capture <n>`
honors, so `fragcap capture 1` captures what you just saw.

Example capture invocations (see [the CLI reference](https://fragcap.com/docs/reference/cli)
for the full surface):

```bash
# Bounded capture of a registered title, launched by fragcap
fragcap capture --target eso --launch --duration 30m --out capture.fcapng

# Client traffic only, streamed to an analyzer through a named pipe
fragcap capture --target div2 --mode stream --roles client --sink pipe:fragcap,format=pcapng

# Rolling ten-minute window, dumped on interrupt
fragcap capture --target eso --mode ring --ring 10m --out captures/eso.fcapng

# Ad-hoc capture of a running process, no stored target
fragcap capture --process eso64.exe --duration 5m --out capture.fcapng
```

`eso` and `div2` stand in for target selectors. A target is registered in your
local store; register an installed Steam title with `fragcap targets add --steam
<APP_ID>`, then capture it by handle, name, or row index. A bare `fragcap` lists
your registered targets. Non-file sinks (`pipe:`, `tcp://`) have no extension to
infer a format from, so they name it explicitly with `,format=pcapng` or
`,format=jsonl`.

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

## Windows installer (MSI)

**Each release offers three downloads, and you choose: a portable `.zip`, a
Windows installer (`.msi`), and the targets hint database on its own.** Every
download has a `.sha256` checksum beside it. The installer is the simplest path
for a new user; the portable archive needs no installation.

The installer places fragcap under `Program Files`, adds it to your `PATH` (so
`fragcap` works in any new terminal), installs the barebones hint database, and
excludes its own install directory from Windows Defender on a best-effort basis
(removed when you uninstall). On completion it offers to open the npcap download
page, because npcap is still a separate install (see below). It never downloads,
bundles, or installs npcap itself.

**The installer is not code-signed.** Windows will show a SmartScreen warning
("Windows protected your PC") and an "Unknown Publisher" prompt. This is expected
for this release, not a sign the download is bad. To proceed, choose **More info**
and then **Run anyway**. Because there is no signature, the integrity check is the
checksum: compute the SHA-256 of the downloaded `.msi` and confirm it matches the
published `.sha256`. Code signing is planned and tracked separately (issue #79).

| Download | Use it when |
| --- | --- |
| `fragcap-<version>-x86_64-pc-windows-msvc.zip` | You want a portable copy with no installer; unzip and run |
| `fragcap-<version>-x86_64.msi` | You want a normal install (PATH, uninstall entry, bundled database) |
| `hint.db` | You only want to refresh the targets hint database |

The `PATH` entry takes effect in newly opened terminals, not the one that was
already open. The bundled hint database is empty on release; fragcap grows it from
your own machine as you capture.

## Prerequisite: npcap

**fragcap requires [npcap](https://npcap.com) to be installed separately,
before any capture.** Run `fragcap doctor` to check your environment; it reports
npcap's presence and names any missing option, and captures nothing. Run
`fragcap doctor --fix` to be walked through the remediations it names, one at a
time, under your confirmation (register the analyzer integration, fetch the
catalog, run discovery, and, with your explicit yes, obtain npcap). `--fix` is
interactive: it is refused with `--json` and when the session is not a terminal,
and it acts only on what `doctor` first printed.

npcap is by the [Nmap Project](https://nmap.org) and is not redistributable
under its standard license, so fragcap does not and will not bundle it. fragcap
detects it and reports its absence with the download location, and only fetches
and launches the vendor's own signed installer when you explicitly confirm it in
`fragcap doctor --fix`, storing nothing of it in any fragcap artifact. The
simplest way to obtain npcap is the [Wireshark](https://www.wireshark.org/)
installer, which bundles it; the
[Getting started guide](https://fragcap.com/docs/getting-started) walks through
that install with screenshots.

npcap and Wireshark are two of the three tiers in fragcap's dependency model
(npcap required, Wireshark recommended, the Wireshark extcap integration
optional); the model is defined once in the
[glossary](docs/glossary/platform-and-distribution.md#dependency-model).

One installation option matters:

| Option | Why it matters |
| --- | --- |
| Install Npcap in WinPcap API-compatible mode | The `pcap` crate links against the WinPcap-compatible interface |

Current Npcap installs loopback capture support automatically, so there is no
longer a separate loopback option to enable.

Building fragcap with the live capture backend additionally requires the npcap
Software Development Kit, which is likewise not redistributed and is acquired at
build time. Capture requires administrative privilege.

## Building

fragcap is a Rust workspace. `rustup` is the only prerequisite; the repository
names the toolchain it needs and `rustup` fetches it on first build. **npcap is
not required to build** the workspace, and the capture backend is behind a
feature flag, so the logic is tested on any platform.

```bash
cargo build --workspace
cargo test --workspace --locked
cargo xtask ci          # the full local check set, same as CI runs
```

To verify platform neutrality locally you need one extra target:

```bash
rustup target add x86_64-unknown-linux-gnu
cargo xtask neutral
```

The documentation site under [`site/`](site/) is a separate build:

```bash
cargo xtask docs build  # static export into site/out
cargo xtask docs        # local dev server with hot reload
```

## Repository map

```text
site/                             Documentation website (fragcap.com), Fumadocs
docs/fragcap-specification.md     Architecture of record
docs/fragcap-spec-outline.md      Navigable map of the specification
docs/glossary/                    Glossary, one page per category (single source)
docs/plans/README.md              Slice ordering S01 to S18
brand/                            Brand identity kit (fonts, logos, tokens)
specs/                            Spec Kit feature slices
scripts/                          Shell wrappers and the documentation linter
.specify/memory/constitution.md   Governing principles, versioned
.agents/skills/                   Vendored portable agent skills
AGENTS.md                         Canonical agent instructions
CONVENTIONS.md                    Mechanical rules for every file
CONTRIBUTING.md                   Contributor workflow
```

## Documentation and development

User-facing documentation is at [fragcap.com](https://fragcap.com). The
[specification](docs/fragcap-specification.md) is the architecture of record,
and every feature traces to it. Development runs through
[GitHub Spec Kit](https://github.com/github/spec-kit): each slice is spec'd,
planned, and analyzed before implementation, and lands as a numbered
`specs/NNN-slug/` directory.

Four agent surfaces are installed (Claude Code, Codex, Cursor, opencode), all
driving the same agent-neutral `.specify/` engine. See [`AGENTS.md`](AGENTS.md).

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

The brand assets under [`brand/`](brand/) are project marks rather than code;
the bundled fonts carry their own SIL Open Font License texts.

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
