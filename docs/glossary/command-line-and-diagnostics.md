# Command Line and Diagnostics

## Readiness check

One line of the `fragcap doctor` report: a section, a name, a detail, a status,
and, when it fails, a remediation. The status vocabulary is exactly four words:
**ok** (ready), **warn** (a non-blocking concern), **skip** (not applicable or
not built into this binary), and **fail** (a blocking problem that must be fixed
before capture is possible). The report exits 1 if any check is `fail` and 0
otherwise.

{: .matters }
> `skip` and `fail` are deliberately distinct. A process-tracing session that is
> not built into the binary is a `skip`, because attribution still works from
> the socket table; a session that could not open while elevated is a `fail`,
> because attribution is then degraded. Collapsing the two would either block a
> capture that would have worked or pass one that will not.

**See also:** [npcap](platform-and-distribution.md#npcap), [Attribution fidelity](file-and-wire-formats.md#attribution-fidelity)

## Lifecycle event

One record in the machine-readable event stream `fragcap` emits on standard
error under `--json`, over a capture's life. There are five: **session.armed**
(the handle is open and the watcher attached), **stage.matched** (a stage bound
a process), **stage.exited** (a bound process exited), **filter.narrowed** (the
capture filter narrowed to a set of active endpoints), and **session.complete**
(the run ended, carrying the headline counters). Each carries an RFC3339 `Z`
timestamp.

{: .matters }
> The event stream is what lets a wrapper react to a capture without parsing
> human-readable progress, which is what keeps a wrapper thin under constitution
> principle P-7. It is newline-delimited JSON on standard error, so capture data
> written to a sink, even one on standard output, is never contaminated by it.

**See also:** [Completion summary](command-line-and-diagnostics.md#completion-summary)

## Completion summary

The end-of-run accounting an operator reads: the captured and attributed counts,
the stop reason, and every discard counter, the packets discarded while watching
before a target was acquired, those discarded out of the capture window, buffer
drops, and per-sink drops.

{: .matters }
> The summary surfaces the counters the pipeline and session already maintain
> and invents none, which is what constitution principle P-4 requires: a bare
> success that hid a watch-time discard or a buffer drop is exactly the silent
> loss the principle forbids.

**See also:** [Lifecycle event](command-line-and-diagnostics.md#lifecycle-event)

## Shell wrapper

A thin script that handles the environment concerns fragcap's binary leaves
outside itself: `Invoke-FragCap.ps1` on Windows (PowerShell) and `fragcap.sh` for
a Linux or WSL2 shell (Bash), specification section 18.

{: .matters }
> A wrapper does privilege elevation, capture-driver detection, interface
> enumeration, [path translation](command-line-and-diagnostics.md#path-translation), and [output
> template](command-line-and-diagnostics.md#output-template) expansion, and nothing else. It reacts to the
> [lifecycle event](command-line-and-diagnostics.md#lifecycle-event) stream rather than parsing human-readable
> output, which is what keeps it thin under constitution principle P-7. A wrapper
> that needs to grow past those concerns is a missing capability in the binary.

**See also:** [Lifecycle event](command-line-and-diagnostics.md#lifecycle-event),
[WSL2 interop](command-line-and-diagnostics.md#wsl2-interop), [Path translation](command-line-and-diagnostics.md#path-translation)

## WSL2 interop

The mechanism by which a script in a Windows Subsystem for Linux shell invokes a
native Windows executable and exchanges data with it across the subsystem
boundary.

{: .matters }
> The Bash wrapper's distinguishing job is this boundary: capture runs in the
> native Windows binary, so `fragcap.sh` under WSL2 invokes it through interop and
> translates paths in both directions. On a Linux host with no reachable Windows
> binary it reports capture unavailable and exits 1, rather than failing
> obscurely.

**See also:** [Shell wrapper](command-line-and-diagnostics.md#shell-wrapper), [Path translation](command-line-and-diagnostics.md#path-translation)

## Path translation

Rewriting a filesystem path between the form one environment uses and the form
another expects, here between a Linux or WSL2 path and a Windows path.

{: .matters }
> A relative output path given in a WSL2 shell must resolve to the intended
> Windows location for the native binary, and the resulting file path must be
> reported back in Linux form. The Bash wrapper does this with the subsystem's
> own path tool; it is a small pure function, checkable without a capture driver.

**See also:** [WSL2 interop](command-line-and-diagnostics.md#wsl2-interop), [Output template](command-line-and-diagnostics.md#output-template)

## Output template

An output-path string carrying tokens a [shell wrapper](command-line-and-diagnostics.md#shell-wrapper) expands
before capture: `{profile}` to the profile name, `{date}` to the capture date,
and `{time}` to the capture time.

{: .matters }
> Templating and directory preparation are an environment concern the wrapper
> handles so the binary does not have to. The expansion is pure and deterministic
> given its inputs, which is what lets a `--dry-run` preview it with no capture.

**See also:** [Shell wrapper](command-line-and-diagnostics.md#shell-wrapper), [Path translation](command-line-and-diagnostics.md#path-translation)

## Effective configuration

The capture options actually used, formed by overlaying the command-line options
onto a profile's `[capture]` defaults. The command line wins, and an option
absent from both stays absent, so a profile that chose a value and a profile that
said nothing remain distinguishable.

{: .matters }
> The overlay preserves the declared-versus-absent distinction the profile schema
> depends on, rather than substituting a default the moment a value is missing.
> Substituting one would destroy the information an operator supplied and make a
> later override behave differently than they wrote.

**See also:** [Game profile](platform-and-distribution.md#game-profile), [Completion summary](command-line-and-diagnostics.md#completion-summary)

## Diagnostic record

One structured `--json` record describing a single problem `fragcap profile
validate` found in a profile: its stable `code`, the configuration key `path`, a
`line` and `col` into the source text, and a human `message`. Validation emits
one record per problem, followed by a terminal `summary` record, on standard
output.

{: .matters }
> The record preserves every field the human formatter renders rather than
> collapsing all problems into one string. A consumer keys on the `code` and
> `path` to act on a specific problem; re-parsing a rendered line would tie the
> automation to prose that may be reworded without notice.

**See also:** [Lifecycle event](command-line-and-diagnostics.md#lifecycle-event), [Readiness check](command-line-and-diagnostics.md#readiness-check)

## Hero listing

The output of `fragcap targets` (and a bare `fragcap`): a numbered table of the
user's registered, capturable targets, each row showing a 1-based index, the
target handle, a capture-readiness status, and neutral known evidence, ending by
naming the next command to run. Producing it runs discovery across its tiers and
registers any newly found titles first, so a fresh install lists the user's own
software; an empty result prints the commands that populate the store instead of
an empty table.

{: .matters }
> The listing is the one command a new user runs successfully on their own
> machine that makes attribution concrete using their own data. Every listed row
> is capturable in principle; the readiness column reports how close, never
> whether the row is valid.

**See also:** [Listing snapshot](command-line-and-diagnostics.md#listing-snapshot), [Capture readiness](command-line-and-diagnostics.md#capture-readiness)

## Listing snapshot

The ordered set of targets the most recent hero listing displayed, persisted to
`local.db` so a bare-integer selector resolves to the row the user saw. A row
index resolves through the snapshot (position to stable identifier to entry), not
through the live store order, so `fragcap capture 3` names the row that occupied
position 3 in the listing even after an intervening add or remove shifts the live
order. A new listing replaces the snapshot; a position past it, or one taken
before any listing has run, is an out-of-range usage error.

{: .matters }
> Without the snapshot a row number would silently change meaning between the
> listing and the capture whenever the target set changed, attributing a capture
> to a different target than the one the user pointed at.

**See also:** [Hero listing](command-line-and-diagnostics.md#hero-listing), [Stable identifier](process-and-attribution.md#stable-identifier)

## Capture readiness

The presentational status a hero listing shows for a target in its CAPTURE
column: `ready` when the entry names a Windows client executable or carries an
anchor a capture can resolve, or `needs a target` when its launch chain is
unresolved and no anchor gives a client. Derived from the entry at listing time
and stored nowhere.

{: .matters }
> Readiness reports how close a row is to a capture, never whether the row is
> valid: every registered target is capturable in principle, and a `needs a
> target` row becomes ready once a capture observes its socket holder.

**See also:** [Unresolved launch chain](process-and-attribution.md#unresolved-launch-chain), [Hero listing](command-line-and-diagnostics.md#hero-listing)

## Target-entry export

A dedicated JSON array of target-entry objects, each carrying an entry's identity
(its stable identifier and handle) alongside its classification, fidelity,
anchor, launch chain, install root, and evidence, produced by `targets export`
and consumed by `targets import`. Import merges each element on its stable
identifier, so an export round-trips through an import with identical identifiers
and no duplicate rows. It is deliberately not the published capture schema, whose
export records are catalog games and omit the entry identity that merge-on-id
requires.

{: .matters }
> The identity travels with the record, so the same target moved between two
> machines converges to one row rather than duplicating. A representation that
> dropped the stable identifier, as the capture-schema export does, could not
> merge and would multiply the target on every import.

**See also:** [Stable identifier](process-and-attribution.md#stable-identifier), [Anchor](process-and-attribution.md#anchor)
