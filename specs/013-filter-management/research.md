# Research: Filter Management

Decisions taken while planning S13, weighed against the constitution, the
architecture of record (sections 8.3, 8.6, 11.4, 11.6, 12.1 through 12.3), and the
existing code. Staged here for promotion to the changelog decisions fragment. Two
(D-a, D-b) carry deviation candidates for specification section 29.

## D-a. Narrowing reads the attribution map, not a process-tree flow set

**Decision**: The endpoint set that phase two compiles from is read through
`FlowAttributor::active_endpoints()`, which the socket-table attributor (S10)
publishes lock-free and documents as "the seam slice S13's control thread attaches
to."

**Alternatives**: The section 8.6 pipeline diagram draws a "flow set" flowing from
the process tree to the filter manager, which would make S11's tree the source.

**Why**: Section 12.2 is explicit: "The attribution map is the only reliable
source, which is why phase two depends on it rather than on traffic inspection,"
and gameplay endpoints are reached by address with no preceding name resolution.
The attributor already carries the retention window (section 11.4), so a closing
endpoint stays in the set through teardown, which the filter must respect. The
diagram's "flow set" and the prose's "attribution map" denote the same endpoint
set; the divergence in rendering is a **deviation candidate for section 29**.

## D-b. `filter_gaps` counts gap occurrences, not kernel-excluded packets

**Decision**: A filter gap is an endpoint that is active in the attribution map
while a narrowed filter that does not admit it is installed on a handle. The
`filter_gaps` counter counts these occurrences, computed as the set difference
between the wanted endpoint set and the previously installed narrowed program at
each reinstall, per handle. Bootstrap admits everything, so the first narrowing
per handle records no gap; gaps arise only when an endpoint appears while a
strictly narrowed filter is installed (phase three).

**Alternatives**: Count the packets a stale filter excluded, as section 12.3's
prose ("packets fragcap does want, which is counted as a filter gap") reads
literally.

**Why**: A packet the kernel filter excludes is never delivered to fragcap, so a
literal packet count is unobservable; fabricating one would violate P-9, and
reporting a fabricated loss is exactly the failure P-9 exists to prevent. The
occurrence count is honestly computable in userspace (a set difference), is a
small integer consistent with the section 13 summary line (`Filter gaps  2
(during narrowing)`), and satisfies section 12.3's requirement that gaps be
"counted and reported in statistics." The unit choice (occurrences, not packets)
is a **deviation candidate for section 29**. The existing `filter_gaps` doc
comment ("Packets that passed while a filter was being narrowed") is refined to
this definition in the same change.

## D-c. Per-source delivery is a std mpsc channel, not arc-swap

**Decision**: The control thread hands each capture thread its current
`FilterProgram` over a `std::sync::mpsc::Sender<FilterProgram>`; the capture
thread drains its receiver to the latest value between reads and installs via
`set_filter`.

**Alternatives**: An `arc-swap` cell per source (mirroring S10's lock-free
attribution snapshot); a `Mutex<FilterProgram>` per source.

**Why**: `arc-swap` is not a `fragcap-core` dependency, and adding it would widen
`CORE_ALLOWED_DEPS` from its deliberate single entry (`bytes`), which the
dependency check treats as a P-2 guard a reviewer must consciously loosen. The
filter slot is read between reads, off the per-packet path, so section 11.6's
lock-free mandate (which exists for the per-packet attribution read that a writer
must never block) does not extend to it; a std channel with a non-blocking
`try_recv` drain is the smaller commitment and adds no dependency. A `Mutex` was
rejected for introducing a lock where a channel needs none. `PacketSource` stays
`!Sync` and the trait gains no bound (P-3): only the owning thread touches the
handle.

## D-d. Filter grammar: union of per-endpoint clauses, and empty-set behavior

**Decision**: A narrowed program is the OR of one clause per endpoint, each
constraining protocol, host address, and port, spanning IPv4 and IPv6 by address
family (libpcap `host` matches the family of the literal). Over-admission of
traffic sharing a target's port is accepted and left to userspace attribution, not
tightened in the kernel. A non-empty endpoint set compiles to a strictly narrowed
program; once narrowing has begun, a set that transiently empties keeps the last
narrowed program rather than reverting to bootstrap admit-all.

**Alternatives**: Reverting to `ip or ip6` when the set empties; tightening
shared-port admission with connection-level predicates.

**Why**: Section 12.2 accepts "the target's traffic plus whatever shares its
ports" and names userspace attribution as the scope authority (section 12.3), so
kernel-side tightening buys nothing and risks excluding wanted traffic. Reverting
to admit-all on a transient empty set would re-flood the boundary for endpoints
that are gone; the S10 retention window keeps closing endpoints present for its
grace period, so a truly empty set mid-session is rare, and holding the last
program is the conservative choice. Generating the text is string building over
core types; a rejected program maps to the existing `SourceError::FilterRejected`.

## D-e. Maintenance timing is a pure policy over supplied instants, injectable for tests

**Decision**: The debounce (two seconds) and per-handle rate limit (one reinstall
per five seconds) live in a pure `FilterManager` whose decision method takes the
current `Instant` as a parameter, so tests drive it with synthetic instants. The
production timings are a `FilterConfig` constant; a `Pipeline::set_filter_config`
setter injects a test config without changing `Pipeline::new`'s signature or the
`PipelineConfig` struct literal (so no existing caller or test breaks).

**Alternatives**: A `Clock` trait dependency (as `fragcap-attr` uses); hard-coded
`Instant::now()` inside the policy; adding the timings to `PipelineConfig`.

**Why**: Taking `now` as a parameter keeps the policy a pure function, mirroring
`interface::select`, and needs no clock abstraction in core. A hard-coded
`Instant::now()` would make the debounce and rate limit untestable without waiting
real seconds. Adding fields to `PipelineConfig` would break every struct-literal
construction of it in the existing tests; a setter is the minimal-churn injection
point, consistent with `add_sink`. The two-second and five-second values are the
section 12.2 constants and are not operator knobs here (S14 owns any command
line), mirroring how S10's `AttributorConfig` carries section 11 constants as
plain values.

## D-f. Gap counting is accumulated on the control thread and absorbed by the run

**Decision**: The control thread accumulates `filter_gaps` (it holds the per-handle
installed-endpoint history the count is computed from) and returns a `CaptureStats`
carrying the total, which `Pipeline::run` folds into the merged report with the
existing `CaptureStats::absorb`.

**Alternatives**: Count gaps in each capture thread as it installs a new program.

**Why**: The gap is a set difference against the previously installed narrowed
program, which the control thread already tracks per handle to enforce the rate
limit; computing it there keeps the capture thread's install path a plain
`set_filter` call and avoids teaching the capture thread the endpoint-set history.
`absorb` already sums `filter_gaps` across merged `CaptureStats`, so folding the
control thread's total in alongside the capture threads' stats needs no new
mechanism. The control thread is joined after the capture threads and before the
report is finalized, so its count is present in the final statistics.

## Non-decisions (consumed, not rebuilt)

- The bootstrap filter (`ip or ip6`) and its application on a live handle
  (`LiveSource::open`, `install_filter`, `set_filter`) are S09.
- `FlowAttributor::active_endpoints()` and the lock-free `PublishedIndex` are S10.
- The `filter_gaps` field on `CaptureStats` and its summation in `absorb` predate
  this slice; S13 populates it and refines its doc comment.
