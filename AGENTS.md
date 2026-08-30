# Agent guide (provider-agnostic)

This is the canonical, agent-neutral instruction file for this repository. Any
coding agent that reads `AGENTS.md` (Codex, Cursor, opencode, and others)
should treat this as the source of truth. Claude Code reads it through
`CLAUDE.md`, which imports this file.

These instructions OVERRIDE any default behavior. Follow them exactly.

## What fragcap is

fragcap is a Windows game-network observability tool, written in Rust. Its
shipped **Capture** mode is passive, process-attributed packet capture. Packet
capture is a solved problem; attribution is not. Standard tooling captures at
the network driver, below the socket layer, where the association between a
packet and the process that produced it has already been discarded. fragcap
reconstructs that association for game clients launched indirectly through
platform and publisher launchers, and writes it into an extended pcapng profile
that unmodified analyzers still read as ordinary pcapng.

The planned **Deep Capture** mode extends that product with explicit, scoped,
authorized local proxy inspection for selected targets whose traffic can be
routed through the proxy. Capture remains passive. Deep Capture is active by
design, and is permitted only when it is selected deliberately, visible to the
operator, reversible, and auditable. Neither mode reaches inside a target
process.

## Reference documents

Read these before acting. They are ordered by authority.

- **Constitution** (governing principles, versioned):
  `.specify/memory/constitution.md`
- **AI authorized-use context**:
  `AI_CONTEXT.md`. Read this before cybersecurity-sensitive work, and any time
  terms such as packet capture, MITM, proxying, TLS inspection, certificate
  authority, decryption, or traffic inspection are relevant. It records the
  project's authorized, defensive, game-development research context.
- **Master specification** (architecture of record):
  `docs/fragcap-specification.md`. Every feature traces to it. Section
  references in the constitution and in slice specs point here.
- **Specification outline** (a map of the above, useful for navigation):
  `docs/fragcap-spec-outline.md`
- **Slice ordering and dependencies**: `docs/plans/README.md`
- **Repository mechanical rules**: `CONVENTIONS.md`
- **Contributor workflow**: `CONTRIBUTING.md`
- **Active feature directory**: recorded in `.specify/feature.json`
  (`feature_directory`). Read that feature's `plan.md` before implementing; it
  carries the technologies, project structure, shell commands, and
  slice-specific context for the current work.

## Current state

**The authority for what has landed is `specs/`, `CHANGELOG.md`, and
`changelog.d/`, not this file.** Every completed slice leaves a directory under
`specs/`, so the highest-numbered one shows where the work has reached, and
`.specify/feature.json` shows what is in flight. For the narrative:
`CHANGELOG.md` carries every released change, and `changelog.d/` carries only
what has not shipped yet, because `cargo xtask changelog --release` consumes the
fragments and deletes them at release time. Looking for a landed slice in
`changelog.d/` alone will usually find nothing.

This file deliberately names no slice number as a completion marker, because any
number written here is wrong one slice later and a reader will quote it anyway.

The architectural summary below is written as of S11 and is extended by those
same records rather than rewritten here every slice. Notably, since S17 the
`fragcap-steam` crate is no longer a skeleton: it reads Steam's local
installation metadata to enumerate installed titles, scaffolds a validating
profile from one (`fragcap steam profile <app_id>`), and starts a title through
Steam's protocol handler under capture (`fragcap run --launch`), all with no
capture logic, no attribution logic, and no process handle.

The Cargo workspace exists with the eight
crates from the architecture of record, a task runner carrying the repository's
own checks, and six workflow files. `fragcap-core` carries the type and trait
vocabulary from specification sections 8.4 and 8.5, a `parse` module
implementing sections 12.5 and 12.6, a `pipeline` module implementing
sections 8.6 and 12.4, a `duration` module carrying the literal grammar three
later slices share, and, since S11, a `process` module carrying the process tree
of section 10. `fragcap-profile` carries section 15 in full: the schema, the
validation set, and the resolution order. `fragcap-capture` reads classic pcap
and replays it as a `PacketSource`. `fragcap-attr` answers attribution from a
declared script and, since S10, from the operating system socket table, and
since S11 watches process start and exit through Event Tracing for Windows.
`fragcap-sink` writes both output formats: pcapng carrying attribution in packet
comments, and JSON Lines. `fixtures/` holds the committed corpus of section 25.3
and, since S06, a golden per fixture per format.

**fragcap attributes flows to processes.** S10 filled in specification section
11. `SocketTableAttributor` snapshots the socket table, joins captured flows
against it by 5-tuple, keeps a closing connection's tail attributed through a
thirty second retention window, and publishes each snapshot as an immutable
value every capture thread reads without locking. Every attributor before it
answered from a text file a test wrote.

**The join's order is total, and that is load-bearing rather than tidy.**
Competing entries rank by exactness, then by the latest socket creation instant
at or before the packet, then by a declared tiebreak whose only job is to make
the order total. An implementation that iterates the platform's rows and takes
the first hit passes an ordinary test and produces answers that change between
runs over identical traffic; the permutation test in `index.rs` is what fails
it. Do not replace that test with one that resolves a single unambiguous flow.

**A socket created after a packet cannot have owned it**, and that filter is
the only mechanism available that distinguishes the previous owner of a reused
port from the current one. Both tables are therefore read by owning module
rather than by owning process identifier, which is the class that carries a
creation instant. Appendix D attributes the timestamp to TCP alone; it is on
both, and the correction is in the S10 decisions fragment. It matters more for
UDP, whose key is the local endpoint alone.

**A retained answer is marked, and the window's origin is exact.** Retention
runs from the instant an endpoint was last observed present in a table, not
from the refresh that noticed it gone; those differ by up to one interval, and
measuring from the later one would make thirty seconds silently thirty-one. A
retained answer can be wrong in exactly one way, a port reassigned inside the
window, which is why `Fidelity::Retained` exists and why widening the window
quietly is a P-9 problem rather than a tuning question.

**The pipeline no longer locks per packet.** S08 held the attributor behind a
mutex and said in `pipeline/mod.rs` that the lock was not the destination; S10
is where it went. `FlowAttributor` gained `Sync` through the deviation process,
the pipeline holds `Arc<dyn FlowAttributor>`, and `Pipeline::new` kept its
`Box` parameter because `Arc<dyn T>` is constructible from one, so no caller
changed.

**The socket table backend has actually run**, which the live capture source of
S09 still has not. A real socket was opened, found in this machine's real
socket table, attributed to the process that opened it, then closed and
observed to survive as a retained attribution. That was possible because the
backend needs no capture driver and no elevation: the IP Helper API ships with
the operating system. Its feature is `socket-table` and deliberately not
`live`, and folding the two together would make attribution unavailable to
anyone without an npcap software development kit it never calls.

**Nothing in this project opens a process handle.** Image names come from a
toolhelp enumeration, which returns them in the snapshot. `OpenProcess` with
query-limited rights would also satisfy P-1, and the point of choosing
otherwise is that a handle request is a thing a reviewer has to check while
having no handle is a thing `cargo xtask lint` can assert. It now does, for
`OpenProcess`, `ReadProcessMemory`, and `WriteProcessMemory`, case
insensitively.

**The section 25.1 claim is now demonstrated rather than asserted.**
`crates/fragcap/tests/pipeline.rs` reads a fixture, parses every packet, and
resolves every flow, with no capture driver, no elevated privilege, and no game.
Every slice from here is testable the day it is written.

**There is a pipeline, and the loss counters carry real values.** S08 composes
a source, the parser, an attributor, and any number of sinks across two threads
with a bounded drop-oldest buffer between them, and
`crates/fragcap/tests/corpus_pipeline.rs` runs the whole corpus through it with
both writers attached, reproducing the committed goldens. The
`CaptureStats` a writer receives is now the run's own rather than a snapshot a
test composed by hand.

The assertion that matters is conservation, not reachability: for every sink,
received plus `buffer_dropped` plus refusals equals `packets_captured`. It is
checked in every pipeline test, and a discard path added later with no counter
fails there rather than passing quietly. Prefer extending it over adding a
counter-specific test.

**A failed sink is retired, not fatal.** Every packet after the failure
advances `sink_dropped` for that sink, which is what section 12.4 already
defines the counter as; the run ends only when every sink has retired. This
reverses the first answer the slice wrote down, and the reasoning for the
reversal is in the S08 decisions fragment. Do not relitigate it without reading
that first.

**The pipeline runs one capture thread per interface.** S09 added `Send` to
`PacketSource` through the deviation process, which is what S08 predicted it
would have to. All capture threads feed the single bounded buffer of section
12.4; there is no second buffer and no multiplexing source, and a proposal to
add either should read the S09 clarification session first.

**Selection is a pure decision over a value, and that is load-bearing.**
`fragcap-core::interface::select` takes an inventory and returns the chosen
interfaces plus a named reason for every interface it passed over. It opens
nothing, so the whole section 12.1 precedence is tested on any machine. The
accounting invariant, that chosen plus passed-over equals the inventory, is
asserted for every case: capturing on the wrong interface produces a run that
exits zero and contains nothing, which is invisible unless the decision is
reported.

**Both writers now record more than one interface.** S06's blanket refusal of a
second is replaced by the narrower rule that was actually needed: every
interface must be declared before the first packet, because section 13.3 settles
the annotation `iface` key from the interface count and a written block cannot
be revised. A single-interface capture is byte-identical to what S06 and S07
produced, checked against the committed goldens.

Note that the two writers differ on the single-interface case and the difference
is deliberate: pcapng omits the `iface` key, JSON Lines always writes it. S09's
specification initially claimed both omit it, and the goldens caught that
during implementation.

**Loss accounting is per-interface where the cause is.** `CaptureStats` holds
one backend report per interface and computes the capture-wide view, so a kernel
drop names the driver buffer that is undersized. `buffer_dropped` and
`sink_dropped` stay capture-wide, because the buffer and the sinks are.

**A retired interface is not a lost packet.** A capture thread that fails
retires its interface, the run continues on the others, and the report names the
interface and the reason. It advances no drop counter: nothing was observed and
then discarded, and counting it as loss would report packets that were never
observed as packets that were thrown away.

**A profile cannot exist unvalidated, and being wrong well is the deliverable.**
S05 filled in `fragcap-profile`. `Profile::parse` returns either a validated
profile or every diagnostic found, and there is no other constructor, so section
15.4's requirement that validation run before every capture cannot be forgotten
by a later caller. Nothing on a diagnostic path uses `?`: a profile with four
mistakes reports four, which is what section 15.4 asks for and what an author
working against a game update needs.

Two of its checks exist because the failures they catch are invisible. A stage
bound to the wrong process among several sharing an image name, and a
`capture.roles` entry naming a role nothing declares, both produce a run that
exits zero, writes a well-formed file, and contains no gameplay. That is the
configuration-side form of the loss P-4 forbids: every packet lost, none
counted. The ambiguity decision is therefore exact rather than approximate, by a
reachability walk over the two patterns. Three further checks were added beyond
the section 15.4 list for the same reason and are recorded in the S05 decisions
fragment as candidates for promotion.

**Unknown keys in a profile are refused rather than ignored**, and the schema
version is what makes that safe. An author who writes `payloads = false`
intending `payload = false` is told, rather than handed a capture containing
contents they meant to exclude. Do not relax this to be helpful; read the S05
decisions fragment first.

**A packet with no flow key advances neither attribution counter.** Never
attempted is not attempted and failed, and `AttributionState` has kept the two
apart since S02. S07's corpus helper conflated them and put a wrong count into
the `malformed` golden, which stood for a whole slice because the goldens were
self-consistent and the definition lived in another crate. S08 found it by
driving the same writers from the pipeline. Corrected in both.

Attribution fidelity is carried on `Attribution`, not derived from whether an
attribution exists. S06 initially derived it and review caught that every
golden was claiming a live socket-table hit for a resolution that came from a
text file.

Two placements are load-bearing and worth not relitigating. The parser lives in
`fragcap-core` rather than `fragcap-capture`, because the capture thread that
calls it belongs to the pipeline, which specification section 8.2 places in
core; the other way round would invert section 8.3. The end-to-end test lives in
the `fragcap` facade rather than in either backend crate, because the facade is
the only crate that legitimately depends on both, and a dev-dependency between
capture and attribution would create exactly the edge P-3 exists to prevent
while slipping past `cargo xtask deps` unnoticed.

**Fixtures are generated, not hand-made.** The generator in
`crates/fragcap-capture/tests/corpus.rs` is the readable record of what each
one contains, and a drift check in the ordinary gate fails if a committed file
stops matching it. Regenerate with `FRAGCAP_UPDATE_FIXTURES=1 cargo test -p
fragcap-capture --test corpus`, then read the diff. See `fixtures/README.md`.

**Dependency inventory.** The table below lists the workspace's dependencies and
which slice added each; the runtime, optional-runtime, and dev distinction is
load-bearing rather than bookkeeping.

| Crate | Kind | Added by | Why |
| --- | --- | --- | --- |
| `bytes` | runtime | S02 | Reference-counted payload clones |
| `pcap` | runtime, optional | S09 | The capture driver binding, behind the `live` feature |
| `regex` | runtime | S05 | Compiles the `path_regex` match predicate |
| `arc-swap` | runtime | S10 | Lock-free publication of the attribution snapshot |
| `windows-sys` | runtime, optional | S10 | The IP Helper socket table, behind the `socket-table` feature |
| `serde_json` | runtime (in `fragcap-profile`), dev elsewhere | S07, promoted S25 | Parses target JSON for the master-schema validator (S25); parses the JSON writer's output in tests (S07) |
| `rusqlite` | runtime, optional | S034 | The embedded SQLite store the targets hint database is built on, behind the `targets` feature |
| `http_req` | runtime, optional | S035 | The HTTP client the live catalog seeder uses, behind the `net` feature; since S056 also `fragcap-cli`'s `doctor --fix` npcap installer fetch, same `net` gate (already in the graph, no `Cargo.lock` package added) |
| `winresource` | build, windows-only | S048 | Stamps the exe's PE FileVersion from `CARGO_PKG_VERSION` (issue #104), behind `[target.'cfg(windows)'.build-dependencies]` |
| `blake3` | runtime | S051 | The 63-bit anchor identifier, a durable exported contract; `default-features = false` |
| `unicode-normalization` | runtime | S051 | The NFKD step of handle normalization |
| `unicode-properties` | runtime | S051 | The `So`/`Sk`/`Cf`/`Mn` general-category tests of handle normalization |
| `getrandom` | runtime | S051 | OS entropy for the unanchored 63-bit identifier (already in the graph; a direct edge only) |
| `terminal_size` | runtime, transitive | S062 | Terminal dimensions for clap's `wrap_help`; not a direct dependency, it arrives with the feature (issue #177) |
| `tokio`, `hyper`, `hyper-util`, `http-body-util`, `rustls`, `tokio-rustls`, `rcgen`, `rustls-native-certs` | runtime | S102 | Exact-pinned native Deep Capture runtime and future HTTP/TLS/certificate stack in `fragcap-proxy`; default features off, ring is the sole crypto provider, Windows roots come from Schannel |
| `ring`, `subtle`, `zeroize` | runtime, direct | S103 | Capability entropy and constant-time comparison plus explicit private-material zeroization; all were already transitive, so no lock packages were added |
| `quinn`, `x509-parser` | dev | S103 | Real loopback QUIC coverage and independent certificate semantic inspection in the controlled protocol lab; never linked into the product path |

S102 raises the workspace MSRV from 1.82 to 1.88 and adds the ninth product
crate, `fragcap-proxy`. This deliberately supersedes S100's external-backend
end state after native completion became an explicit product requirement under
issue #278. The two turnkey-candidate measurements remain valid: fragcap owns
the stack instead. S102 implements only loopback listener, finite connection
and task ownership, typed observation, and bounded idempotent cleanup. It does
not forward, decrypt, parse protocols, or claim inspectability, and the shipped
CLI continues to select mitmdump until #290. The exact dependency and provider
arguments are in the S102 decisions fragment and research document.

S103 completes the native foundation beneath that unchanged production boundary.
It adds session capability admission, bounded and policy-checked upstream work,
per-session certificate authority and leaf ownership, direct current-user
CryptoAPI trust effects, a loss-accounted raw observation stream, and an offline
protocol matrix. Quinn is dev-only and supplies the real QUIC endpoint in that
matrix. The CLI still selects mitmdump until #290, so S103 is not a production
protocol or feature-completion claim.

S048 added `winresource`, the workspace's first build-dependency, to stamp the
Windows exe's version resource so `Get-Command fragcap` reports the real version
rather than `0.0.0.0` (issue #104). It is taken with `default-features = false`,
which is the load-bearing choice: the default `toml` feature pulls the `toml` crate,
which declares Rust 1.85 and would break the 1.82 MSRV gate exactly as it did in
S05, so it is dropped and every VERSIONINFO field is set through the API instead.
The delta to `Cargo.lock` is then exactly two packages, `winresource` and its one
transitive `version_check`, both MIT or Apache-2.0; no `toml`. It lives only under
`fragcap-cli`'s `cfg(windows)` build-dependencies, so a non-windows or cross build
never compiles it, and `cargo xtask deps` ignores build-dependencies, so the first
`[build-dependencies]` does not touch the runtime-graph gate; the only new gate
interaction is MSRV, verified green under 1.82. `embed-resource` was rejected because
its `toml` dependency is unconditional (no feature switch) and it carries a larger
graph; a hand-rolled `.rc` plus the SDK resource compiler was kept only as the
fallback had `winresource` failed the 1.82 floor, which it did not.

S051 added four, all on `fragcap-targets` alone and all non-optional there (the
crate is the unit gated at the facade behind `targets`, the same arrangement as
`rusqlite`, so a default build and the 1.82 MSRV gate compile none of them). The
lock delta is nine crates: `blake3` and its graph (`arrayref`, `arrayvec`,
`constant_time_eq`, `cpufeatures`), `unicode-normalization` with `tinyvec` and
`tinyvec_macros`, and `unicode-properties`; `getrandom` was already present, so
it adds only a direct edge. `blake3` computes the 63-bit anchor identifier, a
durable exported contract the handoff plan names it for; reusing the hand-rolled
SHA-256 was rejected because a durable id contract deserves the algorithm named,
and hand-rolling BLAKE3 was rejected because a wrong identity hash is a P-9 defect
that ships silently. NFKD and the Unicode general categories are large generated
tables, so `unicode-normalization` and `unicode-properties` are the
absurd-transcription case dependencies exist to avoid; a slug crate was rejected
because it would not reproduce the exact Appendix A vectors. `blake3` is
`CC0-1.0 OR Apache-2.0` (the OR resolves to the allowlisted Apache-2.0); every
other crate offers MIT/Apache/BSD/Zlib. `default-features = false` on `blake3`
drops the `std` feature the one-shot `blake3::hash` does not need; `rayon` is a
separate non-default feature never enabled.

S03, S04, S06, S08, S052, and S053 added none. The parser is arithmetic over a byte
slice, a pcap file is a header and a run of records, the attribution script format
is deliberately trivial, and pcapng is length-prefixed binary over a byte sink.

S053 (the data-driven detection signatures) not only added no dependency, it
removed one thing worth recording: the vendored SteamDB `FileDetectionRuleSets`
asset (`crates/fragcap-profile/assets/steamdb/`, MIT, hash-locked) and the
`fragcap-profile::technologies` module that compiled it, along with the crate's
hand-rolled `sha256` module that existed only to integrity-lock those bytes.
Detection moved from that embedded ruleset to a `signature` table in the catalog
database, matched by a generic matcher. The matcher and the `Signature` value type
live in `fragcap-profile` (the one crate both `fragcap-steam` and `fragcap-targets`
already depend on, so both detection consumers reach it with no new edge); the
table, its seed, and the discovery classifier live in `fragcap-targets`;
`fragcap-steam`'s scaffold takes an injected finding set rather than a new edge to
its sibling. The catalog schema advanced from version 4 to version 5 for the
`signature` table. Filename, directory-shape, and PE-version-string matching are
evaluated; the PE reader hand-parses a binary's version resource over its own bytes
(no `goblin`/`object` crate, no OS call), consistent with the hand-rolled pcap and
pcapng parsers. The binary-marker kind is carried but inert.

S052 (the TargetSource discovery seam and tiers) is the other worth noting for what
it did not add. The discovery model, the seam, the account, the known-roots and
user-pointed sources, the directory-shape classifier seam, and the volume
eligibility store, is pure computation and SQLite over crates already in the graph,
so it lands in `fragcap-targets` with no new dependency. Its two platform adapters,
the Steam walk and the Win32 fixed-volume inventory, land in the `fragcap` facade,
the one crate that already depends on both `fragcap-steam` and `fragcap-targets`, so
they add no new inter-crate edge either; the volume inventory reuses the
already-pinned `windows-sys` 0.36 (as a direct facade dependency behind
`cfg(windows)`, adding no `Cargo.lock` package). The schema advanced from version 3
to version 4 for the `volume_eligibility` table, and `TargetsError` gained a
`Discovery` variant for a whole-run source failure.

S08 is the one worth spelling out, because a concurrency crate is the obvious
reach and it would not have helped. Section 12.4 needs bounded, drop-oldest, and
a producer that never waits, together. The standard library's channels are
either unbounded or blocking, and their non-blocking form fails rather than
evicting, which is drop-newest. A third-party bounded channel has the same two
shapes and would still leave the eviction to be written by hand. The buffer is
therefore a `VecDeque` behind a `Mutex` and a `Condvar`, and a proposal to add
`crossbeam` or an async runtime here should say which of those three properties
it thinks the dependency supplies.

S05 is the other one worth spelling out, because it added two and the obvious
choice was not available. `toml` declares Rust 1.85 against this workspace's
1.82 minimum, and pinning it to `~1.0` does not help: `toml_parser` resolves to
1.1.3 underneath and declares 1.85 too. `toml-span` declares 1.70, brings one
transitive crate, has no serde in its graph, and carries the byte spans the
diagnostics are built on. A serde-derived deserializer was never available on
its own terms either: it returns the first error and stops, and section 15.4
requires every problem in one report. `regex` is taken with default features off
because the engine that validates a `path_regex` must be the engine that
evaluates it in S12, and because `aho-corasick` and `memchr` accelerate scanning
large haystacks while a haystack here is one image path. `regex-lite` was
rejected for reduced Unicode support against paths that can carry non-ASCII.

The `exe` glob matcher stays hand-rolled despite both, and the pairing only
looks inconsistent: section 15.4 needs glob intersection, every glob crate
answers glob matching, and a dependency would leave the harder half to be
written anyway. A proposal to replace it should say how it decides whether two
patterns can match one name.

`toml-span` does not implement TOML datetimes, which its own documentation
states and which the S05 analyze gate caught contradicting that slice's first
requirement. No key in schema version 1 has a datetime type, so the divergence
is confined to profiles that are invalid anyway; it is pinned by a test rather
than left in prose.

S09 is the third worth spelling out, and it broke the project's usual pattern of
adding nothing. The alternative to a dependency here is not arithmetic over a
byte slice, as it was in S03 and S06, but a C ABI whose struct layouts must be
transcribed by hand with nothing checking them against the header. A wrong
offset in the packet header yields plausible timestamps that are wrong, which is
the P-9 failure no test over synthetic data catches. `pcap` is MIT or Apache-2.0
across its whole graph and declares Rust 1.64.

Two things about it are worth keeping in working memory. **`libloading` is
pinned to the 0.8 line by `pcap`, and `libloading` 0.9 declares Rust 1.88**, so
taking it directly would break `cargo xtask msrv` in a check most contributors
cannot run locally. And **`pcap` can transmit, and fragcap never does**: `cargo
xtask lint` fails if any fragcap source names a transmit call, so the P-1
argument is mechanical rather than remembered.

S10 added two and both need their argument kept.

**`arc-swap` supplies one property: a reader that a writer cannot block.**
Specification section 11.6 requires the capture thread to read the current
attribution snapshot without locking while the control thread replaces it. The
tempting alternative is `RwLock<Arc<Index>>`, and it is a lock: a reader can
block behind a writer, and the reader here is the acquisition path section 11.6
exists to keep unblocked. It would satisfy a test and not the requirement,
which is worse than failing both because it looks like the requirement was met.
A hand-rolled `AtomicPtr` is correct and needs a reclamation scheme in `unsafe`,
in a workspace that has none outside a platform binding. A proposal to drop it
should answer whether a reader may be blocked at all, not whether a read lock is
fast enough.

Note that it adds **two** packages, not one: it has a build dependency on
`rustversion`. The planning research said one, from reading an empty
`[dependencies]` table without looking at `[build-dependencies]`. Recorded
because an audit that makes that mistake under-reports every proc macro in the
graph.

**`windows-sys` is pinned to 0.36 because `pcap` already resolves it there**, so
it adds no package to `Cargo.lock` at all. Taking the current line would put a
second complete `windows-sys` tree in the graph for declarations that have not
changed. If `pcap` later requires a newer line the graph gains a second copy,
which is Cargo working correctly rather than a defect.

Two further things about it. It is **unrelated to npcap**: the IP Helper API
ships with the operating system, which is why its feature is `socket-table`
rather than `live` and why that backend runs on a machine with no capture
driver. And the alternative to it is the same C ABI transcription S09 rejected:
a wrong offset in `MIB_TCPROW_OWNER_MODULE` yields a plausible process
identifier that is wrong.

S07's writer is hand-rolled and its `serde_json` was test-only on purpose:
verification is worth more the less it shares with what it verifies. S25 promoted
`serde_json` to a runtime dependency of `fragcap-profile`, and the argument
survives because the roles do not overlap: the pcapng and JSON Lines writers are
still hand-rolled and still verified by a test-only `serde_json`, while the new
runtime use only parses an input target file to a `Value` for the hand-rolled
master-schema validator. S25 evaluated taking a JSON Schema validator crate
(`boon`) instead and rejected it: it adds 42 transitive crates (the ICU4X stack
via `url`/`idna`) for machinery the schema does not use. The ecosystem value is
in publishing the schema document, not in consuming a validator, so validation is
hand-rolled like the glob matcher and the writers, and `serde_json` is the only
new crate (already in the graph, so `Cargo.lock` is unchanged).

S026 moved the profile format itself from TOML to JSON, which removed `toml-span`
from `fragcap-profile` entirely (the format it parsed no longer exists) and
promoted `serde_json` to a runtime dependency of `fragcap-steam` and `fragcap-cli`
as well, so the Steam scaffold and the ad-hoc `tap` build their JSON through it
rather than by hand. No crate is added to `Cargo.lock` by any of these, and one
(`toml-span`, and its one transitive crate) is removed. The profile-load path
reuses the S025 `jsonschema` validator for structural conformance and keeps only
the checks a schema cannot express (glob, regex, and duration compilation and the
semantic graph checks), so there is one structural implementation bound to the
published schema.

S034 added `rusqlite`, the first embedded-database dependency, and it is worth
spelling out because the obvious way to take it is wrong. rusqlite 0.40's default
features enable a WebAssembly FFI backend that drags roughly fourteen packages
(the wasm-bindgen stack, `js-sys`, `thiserror`, `bumpalo`) into `Cargo.lock` for
machinery this project never runs; taken with `default-features = false` and only
`bundled`, the delta is six packages: `rusqlite`, `libsqlite3-sys`,
`fallible-iterator`, `fallible-streaming-iterator`, `smallvec`, and `vcpkg`, with
`cc`, `bitflags`, `shlex`, `find-msvc-tools`, and `pkg-config` already in the graph
via `pcap`. `bundled` compiles the SQLite amalgamation through `cc`, so the store
needs no system libsqlite3 and the build is deterministic on a bare Windows
runner; the bundled SQLite carries JSON1. The alternative to the dependency is not
arithmetic over a byte slice but a hand-rolled indexed, transactional on-disk
format, which leaves the harder half to be written; `sqlx` was rejected for
bringing an async runtime and a far larger graph for a synchronous single-file
store. Every crate in the delta is MIT or Apache-2.0; the bundled SQLite
amalgamation is public-domain C compiled by the MIT `libsqlite3-sys`, so
`cargo deny` reads it as MIT and it imposes no attribution obligation. None of the
six declares a `rust-version`, and all compile under Rust 1.82, verified by
building through `rustup run 1.82`; it is taken as a `0.40` range with
`cargo xtask msrv` as the standing gate rather than exact-pinned, because unlike
`clap` nothing in the graph actively breaks the floor today. It is optional and
off at the facade behind the `targets` feature, so a default library build
compiles no SQLite engine, and it lands only in `fragcap-targets`, never in
`fragcap-core`.

S035 added `http_req`, the first HTTP client, for the targets hint database's live
catalog seeder, and the choice was forced by two constraints most 2025-era clients
fail. The license allowlist eliminated the rustls path: `minreq` with its `https`
feature is otherwise the minimal choice, but it forces `webpki-roots`, whose bundled
Mozilla root store is CDLA-Permissive-2.0, outside the allowed set; native-tls uses
the operating system trust store (schannel on Windows) and bundles no roots. The
no-gratuitous-graph rule eliminated every client built on the `url` crate: `ureq`
and `attohttpc` pull `idna` 1.x and the whole ICU4X stack, measured at 42 packages,
the same graph S025 rejected `boon` for. `http_req` does its own URL parsing, so
with `default-features = false, features = ["native-tls"]` it adds 18 packages, no
ICU4X and no `ring`, every one MIT or Apache-2.0. MSRV turned out not to bind it:
the `net` feature is off by default and `cargo xtask msrv` builds default features
only, so `http_req` (and a transitive `zeroize` that declares edition 2024) is never
compiled under 1.82, exactly as `pcap` behind `live` is not. Verified by building
net-off under 1.82 and net-on under the pinned toolchain. `http_req` is smaller and
less widely used than `ureq`; the risk is bounded because it sits behind the
`CatalogSource` trait (replacing it is a one-module change) and is compiled but never
run in continuous integration, so a client regression can break neither the default
build nor the tested pipeline. It lands only in `fragcap-targets`, never in
`fragcap-core`.

`fragcap-core` may depend only on crates named in the allowlist in
`xtask/src/deps.rs`, which is checked mechanically. Note that `cargo xtask
deps` ignores `[dev-dependencies]` by design, so a dev-dependency on a sibling
crate would pass the gate; S06 and S07 both keep their corpus tests in the
`fragcap` facade for that reason.

The remote is `origin`, at `https://github.com/h8rt3rmin8r/fragcap`. S01
integrated through pull request #1.

**A check that has not run is not a check that passed, and neither is to be
reported as green until someone has watched it.** That rule is standing and does
not expire when the list below empties. What follows is the current state of the
checks and demonstrations that rule has governed, each with the evidence that
discharged it or with what would discharge it. Verify before repeating any of
it.

Two kinds of claim appear below and they carry evidence differently. A claim
about something **observed once** (a workflow run, a manual demonstration) must
name its date, and one without a date is a claim to distrust. A claim about how
a check **behaves** is invariant and carries no date; it names instead how to
see the behavior for yourself. Do not read a missing date on the second kind as
the defect the first kind's date exists to prevent.

Discharged:

- **`platform` and `audit` have both run, and both are green.** They were red
  for a runner reason rather than a code reason during the GitHub incident of
  2026-08-06, and stayed unwatched for some time after. As of 2026-08-20 `audit`
  has two scheduled runs, 2026-08-10 and 2026-08-17, both green; `platform` has
  85 runs, 79 green, most recently 2026-08-19. Read as an accounting of the
  workflows, not of what runs inside them: `platform` going green does **not**
  mean its Tier 2 steps executed, for the reason in the outstanding item below.
- **The minimum-toolchain check runs for real.** Until S02 it built with the
  pinned toolchain and reported success, which said nothing about the declared
  minimum. It now builds through the workspace's declared minimum (1.88 as of
  S102) and exits 2 when that
  toolchain is absent, so a check that did not run can no longer look like one
  that passed. This is the clearest illustration of the rule above.
- **The npcap SDK acquisition step has run, and the live source links.** Both
  were first exercised on pull request 12, watched to completion. What that
  proves is that the kit is acquired at build time and that `fragcap-capture
  --features live` compiles and links against `wpcap.lib`.
- **`cargo deny` has run.** The `audit` workflow owns it, and that workflow ran
  green on 2026-08-10 and 2026-08-17. Before those runs its licenses had only
  been verified by hand against the allowlist.
- **Live capture has been executed, manually, on a developer machine with npcap
  installed.** On 2026-08-20 a `fragcap capture --launch` run against a Steam
  title ran 16 minutes wall clock, captured 18,234 packets, and wrote 16,427 of
  them to a pcapng. Managed launch, stage matching, the ETW process watch,
  socket-table attribution, kernel filter narrowing (observed engaging at
  t+22.5s), and a graceful `terminal-stage-exited` shutdown all ran. That run is
  also what produced issues #184, #185, and #186, so it demonstrated the
  pipeline and found real defects in the same pass.
- **The socket table backend has run.** S10's tier 2 tests were executed to
  completion on a Windows developer machine: a real socket opened, found in the
  real socket table, attributed to the process that opened it, and then closed
  and observed as a retained attribution. It was cheap for one reason worth
  remembering: the backend needs no capture driver and no elevation, so there is
  no external dependency between the test and the machine. Its workflow step is
  likewise the first in `platform.yml` that can go green on a bare runner.

Still outstanding, with what would discharge it:

- **Live capture is not exercised in continuous integration, and a green
  `platform` run does not demonstrate it.** The npcap SDK supplies the import
  library; `wpcap.dll` ships with the npcap driver, and a binary linked against
  `wpcap.lib` will not start without it. A runner with no npcap installed exits
  with STATUS_DLL_NOT_FOUND before `main`, which is how S09 found this. Tier 2
  tests therefore do not run in continuous integration, and the workflow says so
  rather than appearing green over nothing, which is correct and should stay.
  **Installing npcap on a runner would not, on its own, discharge this.** The
  Tier 2 tests degrade gracefully rather than failing when the environment is
  not there: `crates/fragcap-capture/tests/live.rs` prints a reason and returns,
  because Rust's harness has no skip and a hard failure would make the `live`
  feature unusable for local development. A test that returned early still
  passes, so a green Tier 2 step can mean the driver was found and used, or that
  every test declined to run. Discharging this therefore needs all of: npcap
  present, installed **with loopback capture support**, a runner with enough
  privilege to open the interface, and the test output read to confirm packets
  were actually captured rather than skipped. Another manual run discharges
  nothing here, because the claim is about continuous integration. Installing
  npcap on a runner remains a licensing decision for the operator.

## Mermaid diagram layout

- Lay out Mermaid diagrams from top to bottom. For flowcharts, use `flowchart TB`; for nested subgraphs with an explicit direction, use `direction TB`.
- Do not use left-to-right, right-to-left, or bottom-to-top flow directions (`LR`, `RL`, or `BT`) unless the user explicitly requests one for a specific diagram.
- Keep the primary reading order vertically stacked. When a diagram would become too wide, split it into multiple focused diagrams or vertically arranged subgraphs instead of changing it to a horizontal layout.
- Treat top-to-bottom layout as a required project convention for every new or modified Mermaid diagram, not as an optional formatting preference.
- Apply this convention only to project-authored diagrams. Do not reject, rewrite, or warn about a different valid direction contained in a user-owned file being processed by the product.

## GitHub planning and slice composition

- Bundle as many compatible GitHub Issues into one implementation slice as can be completed, validated, and reviewed coherently.
- Do not default to one GitHub Issue per implementation slice. Split work only when dependencies, risk, reviewability, platform boundaries, conflicting validation needs, or independent delivery value make separate slices materially clearer or safer.
- Preserve issue-level traceability inside a bundled slice by naming every included issue, satisfying each issue's acceptance criteria, and reporting any issue that remains incomplete instead of closing it implicitly.
- Prefer slices that deliver a meaningful end-to-end capability or release increment over artificially narrow issue-by-issue churn.
- Do not close an issue merely because it was included in a slice. Close it only after its individual acceptance criteria and required verification are complete.
- Treat native GitHub Issues, labels, and milestones as the authoritative project-management surface unless the repository explicitly designates another system.
- When approved specifications, plans, requirements, review findings, or implementation discoveries create actionable work, proactively create or update the corresponding GitHub planning records. Do not leave durable work tracked only in chat, temporary notes, or an implementation plan.
- Before creating or modifying GitHub records, verify that the local Git remote identifies the intended repository and inspect the repository's existing issues, milestones, labels, title prefixes, sibling repositories, and organizational conventions.
- Search open and closed issues before creating anything. Reuse or update an existing issue when it already represents the work, and never create a duplicate merely because its wording differs.
- Organize substantial work under outcome-oriented or release-oriented milestones. Give every planned issue the appropriate milestone unless the repository convention explicitly permits an unmilestoned backlog.
- Use coordinator or epic issues for work spanning several child issues or milestones. Each child issue must identify its parent, and each coordinator issue must maintain a linked child-issue index and a completion checklist.
- Record dependencies with explicit GitHub issue references. State what an issue depends on, what it blocks when useful, and the order in which dependent work can safely proceed.
- Give every issue a specific outcome, bounded scope, acceptance criteria, dependency section, and links to the specifications, decisions, incidents, or review findings that authorize it.
- Apply the repository's established label taxonomy consistently. Prefer structured label families for work type, functional area, priority, and estimated effort when those families exist.
- Treat "tags" as GitHub labels. Do not create synonymous, differently punctuated, or differently capitalized labels when an existing label already represents the same category.
- Create or revise labels when the roadmap introduces a durable new category. Every label must have a concise description, an intentional color, and a scope that does not overlap confusingly with existing labels.
- Follow the repository's established title-prefix convention, such as lifecycle, stability, release-stage, or work-type prefixes. Inspect existing usage before selecting a prefix, apply it consistently, and do not invent a new prefix for a single issue.
- Do not invent milestone deadlines, release dates, priorities, assignees, or completion claims without supporting project authority.
- Keep planned capabilities visibly distinct from shipped capabilities. Closing an issue or milestone must not cause documentation, associations, release notes, or public claims to advertise behavior that has not passed its release gates.
- When completed historical work must be represented, create a clearly identified retrospective issue or milestone, attach the available implementation and verification evidence, and close it as completed without rewriting project history.
- After bulk creation or reorganization, audit the complete planning surface for duplicate titles, missing milestones, missing required labels, broken issue references, missing parent links, invalid dependency ordering, and incorrect open or closed states.
- Prefer idempotent automation for bulk GitHub management. Match existing records by stable identity or exact canonical title so an interrupted run can resume without creating duplicates.
- Never create, modify, close, or reorganize issues in a repository that does not match the verified Git remote or the repository explicitly named by the user.

## Spec-driven development workflow (spec-kit)

Every slice MUST be spec'd through the spec-kit framework before
implementation. The slice ordering document scopes a slice but never
substitutes for its spec.

The engine is shared and agent-neutral; drive it the same way regardless of
which agent you are:

- Templates: `.specify/templates/` (`spec-template.md`, `plan-template.md`,
  `tasks-template.md`, `checklist-template.md`, `constitution-template.md`)
- Scripts: `.specify/scripts/bash/` (`create-new-feature.sh`, `setup-plan.sh`,
  `setup-tasks.sh`, `check-prerequisites.sh`, `common.sh`)
- Workflow registry: `.specify/workflows/workflow-registry.json`
- Constitution (the gate every phase checks against):
  `.specify/memory/constitution.md`

The full cycle, run end to end per slice:

1. **specify** - create or update the feature spec from the slice intent.
2. **clarify** - resolve underspecified areas; encode answers back into the
   spec.
3. **checklist** - generate a slice-appropriate quality checklist.
4. **plan** - produce design artifacts into the feature directory.
5. **tasks** - generate a dependency-ordered `tasks.md`.
6. **analyze** - non-destructive cross-artifact consistency check. This gate
   MUST pass and MUST NOT be weakened or skipped.
7. **implement** - execute `tasks.md`.
8. **verify** - run the full gate set in the foreground (see below).
9. **commit** - stage only the slice's files, add a changelog fragment under
   `changelog.d/`, and commit. `.specify/feature.json` is local, gitignored
   state; never stage it.

Agents with native spec-kit command wrappers may invoke those. Four surfaces
are installed and all drive the same `.specify/` engine:

| Agent | Command surface |
| --- | --- |
| Claude Code | `.claude/skills/speckit-*` |
| Codex | `.agents/skills/speckit-*` |
| Cursor | `.cursor/skills/speckit-*` |
| opencode | `.opencode/commands/speckit.*` |

Agents without a wrapper should follow the phases above directly against the
templates and scripts. The result is identical; the wrappers are convenience,
not capability.

Do not re-point or hand-edit `.specify/integration.json` or
`.specify/init-options.json`. Those record the generated command surfaces and
are regenerated by the spec-kit CLI.

## Skills

`skills/README.md` is the authority for this surface: the admission test, the
add and remove procedures, and the gate. What follows is the policy summary,
not a substitute for it.

Portable skill content is vendored in `.agents/skills/` and committed, with
provenance in `skills-lock.json`. First-party skills authored for this
repository live in `skills/`. There are none yet.

**The vendored set is the ShruggieTech house standards this constitution binds,
from one upstream, and nothing else.** A skill is admitted only if a named
principle binds this repository to it or a repository gate executes it;
plausible usefulness does not qualify it, and neither does sharing a brand with
something that does. The upstream is <https://github.com/shruggietech/skills>,
Apache-2.0, and each lock entry's `source` names the release tag its content
came from. Slice S071 established this after the set reached 36 entries from an
unrelated repository that nothing here could account for.

A skill is checked against P-1 before it is vendored. A skill that teaches a
denylisted technique does not land here, whatever else it is useful for. It is
then copied in unmodified: an edited vendored copy is no longer the standard it
claims to be, and `skills/README.md` records what that cost last time.

Codex reads `.agents/skills/` directly. Claude Code and Cursor read their own
directories, which an external skills CLI populates with machine-local symlinks;
those are gitignored because they carry absolute paths, the CLI is not part of
this repository, and **a checkout may carry none of them at all**. Nothing here
depends on their existing. Spec-kit's own generated `speckit-*` skills are real
tracked files in every surface and are the exception.

`cargo xtask skills` holds the directory, the lock, and git's index to
agreement. It does not verify hashes; `skills/README.md` says why.

## Non-negotiables

These restate the constitution's sharpest edges. The constitution is
authoritative; this list is the one to keep in working memory.

- **The technique denylist is absolute.** No packet interception drivers, no
  code injection, no function hooking, no process handles carrying memory-read
  rights against a target, no layered service providers, no executable image
  modification, and no target TLS key extraction. A slice that appears to need
  one has been scoped wrong; halt and raise it. Deep Capture does not weaken
  this rule: it may use only the explicit local inspection proxy and certificate
  lifecycle described in the constitution and specification.
- **Any process handle states its access rights explicitly at the call site.**
  A request carrying memory rights fails review.
- **`fragcap-core` takes no platform-specific dependency.** Dependencies flow
  concrete toward abstract, and continuous integration proves core builds for
  a target with no capture backend.
- **Every discard path has a named counter.** A dropped packet that is not
  counted and surfaced is a defect.
- **npcap is never bundled, hosted, embedded, or redistributed by fragcap, and
  its SDK is never vendored.** fragcap detects it and reports its absence with the
  official download location. It may fetch and launch the vendor's own signed
  installer, storing nothing of it in any fragcap artifact, only under an explicit
  interactive confirmation (as in `doctor --fix`); absent that confirmation, and in
  any non-interactive or machine-readable context, it neither fetches nor launches.
  This carve-out was authorized in constitution 1.3.0 (slice S056); the
  redistribution, bundling, and SDK-vendoring prohibitions remain absolute.
- **Compatibility outranks richness.** Output stays readable by unmodified
  analyzers.
- **A new term gets a glossary entry in the same change that introduces it.**
- **Wrappers stay thin.** A wrapper that needs to parse output means a missing
  capability in Rust.
- **Pinned artifacts change only with a dated decision recorded in
  `CHANGELOG.md`:** `.github/workflows/**`, `rust-toolchain.toml`,
  `release.toml`, `scripts/**`, and release documentation. Write the decision
  as a `changelog.d/<key>.decisions.md` fragment; `CHANGELOG.md` is assembled
  from those fragments at release time, and editing it from a feature branch
  conflicts with every other concurrent pull request. `release.toml` now exists
  (added 2026-08-08 with the release automation), so the rule binds it.
- **All text files are UTF-8 without BOM with LF line endings. No em-dashes or
  en-dashes anywhere, including code comments.**

## Verification discipline

Run verification in the foreground and watch it to completion. Never background
it, never infer a result you did not read.

The gate set, all of which `cargo xtask ci` runs in order:

```sh
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --workspace --locked
cargo xtask lint          # repository conventions, CONVENTIONS.md
cargo xtask deps          # dependency direction, specification section 8.3
cargo xtask license       # per-crate license text for registry publication
```

Two further checks are not in `ci` because they need a target or a toolchain
the runner may not have: `cargo xtask neutral` (constitution P-2) and
`cargo xtask msrv`. Both exit 2 rather than 0 when they cannot run.

The documentation linter and the shell wrapper compliance checkers arrive with
the slices that own them.

**Claims require evidence.** Do not report a slice complete, a test passing, or
a defect fixed without having run the command and read its output. If tests
fail, say so and include the output. If a step was skipped, say that. Reporting
an unverified success is worse than reporting a known failure, because it
removes the operator's ability to trust any other report.

## Deciding versus asking

Default to deciding: enumerate the alternatives, evaluate them against the
constitution, the master specification, and the slice scope, pick the best,
proceed, and record the rationale in the slice.

Halt to the operator only when no option is clearly best on an irreversible or
architecture-defining choice, the slice intent is genuinely ambiguous, or a
constitution conflict needs a human call. A P-1 conflict is always a halt.

## Reconnaissance gate

**Closed.** Open questions Q-1 through Q-6 (specification section 29) are
resolved. The findings are recorded in Appendix D and were applied to the
specification; the protocol that produced them is
`docs/plans/reconnaissance.md`.

Slices S09, S10, and S17 were gated on those answers and are now unblocked.
Q-7 and Q-8 (the brand session) are resolved as of 2026-08-10: Geist Mono for
the monospace face, and fragcap as an independent ShruggieTech sub-brand carrying
an "A ShruggieTech project" endorsement rather than a combined logo. The approved
identity is vendored in `brand/` and recorded in `docs/brand/README.md`, so S18 is
unblocked.

## Integration workflow

Work integrates through pull requests reviewed by the operator
(`@h8rt3rmin8r`). Never push directly to `main`, and never merge your own pull
request. See `CONTRIBUTING.md` for the full workflow.

Never push, tag, cut a release, or publish a crate without explicit
authorization.
