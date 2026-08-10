### Decisions

**2026-08-09: three deviations from the architecture of record, for promotion to
specification section 29.**

- **`PacketSource` requires `Send`.** Section 8.5 declares it without the bound,
  and slice S08 relied on its absence: it acquired on the calling thread and
  spawned only the sink thread, so a trait meant to reach 1.0.0 unchanged did
  not have to change for one slice. Section 12.1's one thread per interface ends
  that. There is no arrangement of a single thread reading several handles that
  does not need either this bound or a second buffer, and section 12.4 specifies
  exactly one buffer.
- **`CapturedPacket` carries a non-optional interface identifier.** Section
  8.4's packet vocabulary predates any capture with more than one interface.
  Non-optional because every packet arrived somewhere; an `Option` would let a
  real capture ship with the question unanswered. `RawPacket` is unchanged: a
  source knows only its own interface, so the identifier is attached by the
  pipeline at the lift.
- **`CaptureStats::source` becomes per-interface, with the total computed.**
  Each handle has its own driver buffer, so a kernel drop was always a
  per-interface quantity; there was simply never a second interface to reveal
  it. Folding them would tell an operator a driver buffer is undersized without
  saying which. Found during planning rather than before it.

**2026-08-09: the `pcap` crate binds the capture driver.** Measured rather than
assumed: MIT or Apache-2.0 across its whole transitive graph, a declared Rust
1.64 against this workspace's 1.82 floor, and a released-2025 line still
maintained. Its `Stat` maps one to one onto `SourceStats`, and its counts are
cumulative from the start of the run, so relaying them unaltered is a copy with
no arithmetic in which an alteration could hide.

The alternative to a dependency here is not arithmetic over a byte slice, as it
was in S03 and S06, but a C ABI whose struct layouts must be transcribed by hand
with nothing checking them against the header. A wrong offset in the packet
header yields plausible timestamps that are wrong, which is the constitution P-9
failure that no test over synthetic data catches.

Note for anyone adding to the graph later: `libloading` is pinned to the 0.8
line by `pcap`, and `libloading` 0.9 declares Rust 1.88. Taking it directly at
`"0.9"` would break `cargo xtask msrv`, in a check most contributors cannot run
locally.

**2026-08-09: the transmit capability is answered with a lint, not an
argument.** `pcap` exposes packet transmission on an active capture, and the
constitution says a dependency providing a prohibited capability fails the
dependency audit. Transmission is not on the section 19.3 denylist, which names
interception drivers, code injection, function hooking, process handles carrying
memory rights, layered service providers, and image modification; npcap's NDIS
capture driver is explicitly permitted by section 19.2. That argument is
correct, and it is also the kind of argument that decays, so `cargo xtask lint`
now fails if any fragcap source names a transmit call. The check was verified by
introducing a call and watching it fire.

**2026-08-09: the feature is named `live`, not `platform-tests`.** The
`platform` workflow anticipated the latter name in a comment. The feature gates
a capability rather than a test suite, and a capability named for its tests
invites someone to enable it in order to run tests and be surprised that the
library changed.

**2026-08-09: `.github/workflows/platform.yml` gains real triggers.** A pinned
artifact, changed because this slice is the first to give it a subject: until
now no crate linked against the capture library, so its software development kit
acquisition step had never run. It now triggers on changes to the capture
crates and builds the live source, because `cargo check` does not link and a
missing `wpcap.lib` appears only at the link step. Its first run was watched to
completion on pull request 12; what it found is the entry dated 2026-08-10
below.

**2026-08-09: the default route is determined with `std::net`.** A UDP socket
bound and connected to a documentation-range address reports the source address
the routing table chose; `connect` on UDP transmits nothing. The alternative was
`GetBestRoute2` through `windows-sys`, which would add a platform dependency and
a second major version of `windows-sys` to the graph, since `pcap` pins the 0.36
line.

**2026-08-09: device loss is determined by observation, not by string
matching.** `pcap::Error` has thirteen variants and none names a device that has
gone away; a removed adapter arrives as the general `PcapError(String)`. On a
terminal error the live source re-enumerates and asks whether its interface is
still present. Matching the message text would work until a driver update or a
non-English locale changed it, and would then downgrade a lost device to an
unmodelled failure silently.

**2026-08-09: the virtual-interface rule is a heuristic and is presented as
one.** No platform reports a "this is a hypervisor adapter" bit, so fragcap
matches the adapter description against a documented pattern list. The verdict
only ever excludes from automatic selection, never from explicit selection, and
it is recorded with the pattern that matched, so a misclassification is visible
rather than surfacing as an empty capture.

**2026-08-09: two facts the binding cannot supply are reported as unknown.** The
`pcap` crate exposes no libpcap version string, and WinPcap API compatibility
mode is indistinguishable from an ordinary npcap installation through libpcap.
Both are reported as `None` rather than guessed or inferred, because "not
determined" and "absent" are different statements. Slice S14's `doctor` command
can query the installed service and is where that capability belongs.

**2026-08-09: the attributor is shared behind a mutex, as an interim.** Several
capture threads ask one attributor, and `FlowAttributor` is `Send` without being
`Sync`. Section 8.6's control thread publishes a snapshot the capture threads
read without blocking, which is the arrangement that removes the lock; it
arrives with S11 and S13. Adding `Sync` to the trait would have been a fourth
deviation to buy something the control thread makes moot.

**2026-08-09: the pcapng writer's blanket refusal of a second interface is
replaced by a narrower rule.** S06 refused every second interface because
`CapturedPacket` carried no identifier, so every packet would have routed to the
first declaration. S09 supplies the identifier, and what remains necessary is
only that all interfaces be declared before the first packet: section 13.3
settles the annotation `iface` key from the interface count, and a written block
cannot be revised.

**2026-08-10: the software development kit is enough to build and not enough to
run, and this slice learned it the hard way.** The `platform` workflow's first
ever run acquired the kit and built the live source successfully, then failed
running the test suite with STATUS_DLL_NOT_FOUND. A binary linked against
`wpcap.lib` needs `wpcap.dll` at load time, and that DLL ships with the npcap
driver installation rather than with the kit.

The consequence falsifies a claim this slice's plan made. Tier 2 tests were
designed to detect a missing driver at runtime and print a reason rather than
failing; on Windows the process never starts, so that design gets no chance to
run. The workflow now checks for the driver before choosing which test command
to issue, and says plainly when live capture was not exercised.

Installing npcap on a runner would make tier 2 tests real and is a licensing
decision rather than a technical one, so it is left to the operator. Until it is
taken, the `platform` workflow proves that the live source compiles and links,
and proves nothing about whether it captures.

**2026-08-09: `cargo xtask neutral` now builds `fragcap-capture` as well as
`fragcap-core`.** It only ever built core, while the specification claimed both
build for a target with no capture backend. The claim was true and nothing
checked it. Found by this slice's analyze gate.
