# Research: doctor truthfulness and presentation

All unknowns were resolvable from the existing code and the approved plan; no
external research was required. The decisions below are recorded for the analyze
gate and for slice 042, which depends on the taxonomy decision.

## D-1: Colorization approach

**Decision**: Hand-rolled ANSI escapes gated by `std::io::IsTerminal` plus a
`NO_COLOR` environment check, applied in `commands/doctor.rs` around the plain
string returned by `render_human`.

**Rationale**: The CLI has no color code and no color dependency today. The
surface is tiny (four status words and a bold heading). `anstyle`/`anstream` are
present transitively via clap but taking them as direct dependencies adds a
supply surface and a manifest change for no real gain here. `IsTerminal` is
stable well under the 1.82 MSRV. Keeping color out of `render_human` is what
lets the golden test (which calls `render_human` directly) stay byte-plain.

**Alternatives considered**: take `anstyle`/`anstream` as direct deps (rejected:
unnecessary supply surface); colorize inside `render_human` gated on a passed-in
choice (rejected: risks color bytes leaking into the golden and the JSON path).

## D-2: Line wrapping

**Decision**: Wrap detail and remediation at a fixed 80 columns inside
`render_human`, emitting overflow as an indented continuation aligned under the
detail column.

**Rationale**: Golden determinism. Reading the real terminal width would make the
golden machine-dependent. 80 is the stated standard width (spec Assumptions).

**Alternatives considered**: dynamic terminal width (rejected: non-deterministic
goldens); no wrapping (rejected: fails FR-008 and the original #105 complaint).

## D-3: Loopback three-valued state

**Decision**: Feed `DriverReport::loopback_supported: Option<bool>` into
`loopback()`. `Some(true)` -> ok "loopback capture supported"; `Some(false)` ->
warn (not needed unless capturing loopback, non-blocking, matching the existing
severity from issue #69); `None` -> warn "loopback support could not be
determined". Delete the `npcap_wifi.sys` line entirely.

**Rationale**: P-9. The old signal answered "is WiFi support installed" and
mislabeled it as loopback. `detect_driver()` already computes loopback from the
enumerated adapter (`flags.is_loopback()` or a description containing
"loopback"). `None` preserves the "undetermined" distinction P-9 requires rather
than collapsing it into "absent".

**Alternatives considered**: per-interface `is_loopback` from the enumerated set
(equivalent; `DriverReport` already encapsulates the same test and is the
single-purpose value, so it is preferred); keep a bool and collapse None to false
(rejected: reintroduces the "undetermined reported as absent" P-9 problem).

## D-4: Identity facts as check rows

**Decision**: Emit version, binary path, profile dir, and hint-db path as
ordinary `Check` rows in a new leading `Identity` section, not as a separate
JSON object or header.

**Rationale**: The machine-readable form asserts one record per check
(`the_json_form_is_one_record_per_check`). Modeling identity as checks keeps that
invariant automatically and reuses the existing `Inputs field -> classifier ->
Check` pattern. Version is carried on `Inputs` (not read from
`CARGO_PKG_VERSION` inside the classifier) so the golden fixture supplies a fixed
version and does not churn on release bumps (R-2).

**Alternatives considered**: a distinct header object in JSON (rejected: breaks
the record-count invariant and needs a new consumer contract); reading the
version in the classifier (rejected: golden churns every release).

## D-5: cfg-gating the enumeration

**Decision**: A `#[cfg(all(feature = "live", windows))]` probe helper calls
`fragcap::enumerate()` and `fragcap::detect_driver()`; the
`#[cfg(not(all(feature = "live", windows)))]` fallback returns an empty interface
vector and `None` loopback.

**Rationale**: `enumerate`/`detect_driver` are re-exported only under
`live`+`windows` (`crates/fragcap/src/lib.rs`). An ungated call fails the default
`cargo test --workspace` (no features) and the Linux `fragcap-core` neutrality
build. This mirrors the existing `live_availability()`/`tracing_availability()`
shape exactly (R-1, the single most likely regression).

## D-6: virtual_verdict re-export

**Decision**: Add `virtual_verdict` and `VirtualVerdict` to the `fragcap::core`
re-export in `crates/fragcap/src/lib.rs`.

**Rationale**: The doctor maps `InterfaceRecord.is_virtual` via the existing
`virtual_verdict` heuristic (`fragcap-core::interface`), which is not currently
reachable through the facade. Re-exporting single-sources `VIRTUAL_PATTERNS`
rather than replicating the substring list in the CLI.

**Alternatives considered**: replicate the heuristic in `probe.rs` (rejected:
duplicates a maintained pattern list, drift risk).

## D-7: Dependency taxonomy (shared with slice 042)

**Decision**: Record the required/recommended/optional dependency model as a
`changelog.d/dependency-taxonomy.decisions.md` fragment: npcap REQUIRED (driver,
for live capture); Wireshark RECOMMENDED (analyzer, bundles npcap); extcap
OPTIONAL (not a separate download, ships with Wireshark). Doctor severities in
this slice follow it (npcap absent is blocking; loopback and integration are
non-blocking); slice 042 rolls the same model into the docs so tool and docs
cannot drift.

**Rationale**: Prevents the doctor severities (040) and the docs prose (042) from
diverging. The taxonomy is the through-line of the whole post-release work
stream.
