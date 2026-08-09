### 2026-08-09: Two runtime dependencies, chosen by measurement rather than reputation

The workspace has added one runtime dependency in eight slices, so a second and
third are an architectural event and are recorded as one. `fragcap-profile`
takes `toml-span` 0.7 and `regex` 1.13 with default features off. Five crates
enter the graph: `toml-span`, `smallvec`, `regex`, `regex-automata`, and
`regex-syntax`. Every license is MIT or Apache-2.0, inside the `deny.toml`
allowlist, and no version specification is a wildcard.

**Why not hand-roll TOML, when pcap, pcapng, and JSON Lines were hand-rolled.**
Those three formats were the deliverable, produced by fragcap or by a tool, and
hand-rolling gave verification something independent to judge against. A profile
is a file a contributor typed. A hand-rolled subset would refuse legal TOML an
author's editor produced, and section 15.1 promises that adding support for a
game means writing a TOML file, which a parser that rejects valid files does not
survive.

**Why not `toml`.** It is unavailable at this workspace's declared minimum
toolchain. Version 1.1 declares Rust 1.85 against a floor of 1.82, and pinning
to `~1.0` does not fix it: `toml_parser` resolves to 1.1.3 underneath and
declares 1.85 as well. Holding the floor would mean a direct dependency on a
crate this slice never calls, purely to constrain it, which one `cargo update`
undoes without anything failing loudly. `toml-span` declares 1.70, brings one
transitive crate rather than four, has no serde in its graph, and carries byte
spans on every value, which is what the diagnostics are built on. Verified by
building under `rustup run 1.82`.

**A serde-derived deserializer was never on offer**, which is worth stating
because it is the obvious ergonomic path. Such a deserializer returns the first
error and stops, and section 15.4 requires every problem in one report. The
requirement rules out the shape, so field extraction is written by hand and the
question of serde at runtime does not arise. S07's `serde_json` remains
test-only and that argument is undisturbed.

**Why `regex` and not `regex-lite`.** Section 15.4 requires compiling
`path_regex`, so an engine is unavoidable, and it must be the engine that
evaluates the pattern in S12: validating with one and matching with another lets
a pattern pass validation and fail during a capture. `regex-lite` is one crate
with no dependencies, which is attractive here, and it was rejected because its
Unicode support is reduced. An image path can carry non-ASCII through a user or
localized directory name, and matching under quietly different Unicode rules
produces a wrong binding rather than an error. Default features are off because
`aho-corasick` and `memchr` accelerate scanning large haystacks, and a haystack
here is one image path matched a few times per session.

**The glob matcher stays hand-rolled** despite the above, and the pairing only
looks inconsistent. Section 15.4 needs to know whether two `exe` patterns can
match a common image name, which is glob intersection; every glob crate answers
glob matching, which is the intersection of a pattern with a literal. A
dependency would supply half the requirement and leave the harder half to be
written anyway, giving two implementations of one syntax to drift apart.

### 2026-08-09: The duration grammar lives in fragcap-core

Section 25.2 lists duration parsing as a tier 0 concern without placing it, and
three consumers are visible: `capture.duration` in a profile, `--duration` and
`--wait` on the command line, and the ring window. Core is the crate all three
reach, and the grammar adds no dependency there, so the allowlist
`cargo xtask deps` enforces is untouched.

Keeping it in `fragcap-profile` was the alternative and fails on ring mode:
that slice would either depend on a sibling, which section 8.3 forbids, or carry
a second grammar. Two implementations of `30m` that disagree produce a capture
of the wrong length, which is a defect an operator cannot see in the output.

Recorded for promotion to specification section 29, since section 25.2 names the
concern without assigning it a crate.

### 2026-08-09: Three validation checks beyond the section 15.4 list

Section 15.4 enumerates its checks, and three more are implemented. Recorded as
additions rather than as readings of the specification, so that a future reader
comparing the code against the document finds the difference explained rather
than having to decide whether it is a defect.

- A `terminal` stage must have lifecycle `session`. Section 10.4 defines a
  transient exit as normal and expected, so a terminal transient ends the
  capture at the moment a launcher hands off, which is the point the launcher
  chain exists to survive.
- The `descends_from` relation must be acyclic. A cycle is unsatisfiable, so
  every stage in it binds nothing.
- Every role named in `capture.roles` must be declared by a stage, and the list
  must not be empty when present. A role nothing declares captures nothing
  under it.

Each is in the failure class the two unusual checks section 15.4 already names
were added for: a run that succeeds, exits zero, and captures nothing. All three
are candidates for promotion into section 15.4 under the deviation process.

### 2026-08-09: Unknown keys are refused, and the schema version is what makes that safe

The `[capture]` table accepts exactly the five keys section 15.2 declares, and
any key outside a table's accepted set is a diagnostic naming the key and the
set. Section 17.2 lists more capture options on the command line, and none is
accepted here: a profile key with no consumer is a key whose behavior is
untested and whose meaning is set by whoever first reads it. S14 owns the
command line and adds the keys it can honor.

Ignoring an unknown key is the silent failure. An author who writes
`payloads = false` intending `payload = false` gets a capture containing full
packet contents they meant to exclude, and nothing in the run says so. That is a
P-9 problem rather than a typo: the instrument was told to narrow what it
recorded and did not.

Strictness is only safe because `schema` exists. A profile written for a later
format declares it and is refused with one version diagnostic rather than a wall
of unknown-key faults.

### 2026-08-09: Resolution takes its search path from the caller

The resolver implements section 15.3's four steps over directories and a bundled
set that it is given, and never consults an environment variable or a platform
configuration location. That keeps a platform-directories dependency out of the
workspace, keeps the ordering testable against directories a test builds, and
leaves the platform question to S14, which is the layer that already has to know
it.

The slug rule applies to steps two through four only, and applies before any
path is joined. Step one is exempt because an operator who types a path has
named a file, and refusing an absolute path there would break the case section
15.3 puts first. The distinction is between naming a file and interpolating a
name into a search path, and only the second is a traversal surface. The check
runs before the join rather than relying on the open failing, because a check
that depends on what happens to be at the target is not a check.

### 2026-08-09: A known divergence from TOML 1.0, found by the analyze gate

`toml-span` does not implement TOML datetimes, which its own documentation
states. The first draft of this slice's FR-002 required a parser that
"implements the language rather than a subset of it", and the analyze gate
measured that claim to be false. The requirement was corrected to name the
constructs a schema version 1 profile can contain, which is both true and
sufficient, rather than the finding being explained away.

The divergence is confined to profiles that are invalid regardless. No key in
schema version 1 has a datetime type, so a datetime can appear only as a
wrong-typed value or under an unknown key. What changes is the message: a syntax
diagnostic rather than a type diagnostic located at the key. That is worse, and
it is worth less than the minimum toolchain the alternative would have cost.

The behavior is pinned by a test rather than left in prose, so a future reader
finds a recorded decision instead of a surprise, and so that the day the parser
gains datetime support the test says so.
