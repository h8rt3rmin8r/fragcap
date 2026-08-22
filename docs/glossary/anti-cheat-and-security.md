# Anti-Cheat and Security

## Anti-cheat

Software that detects or prevents unauthorized modification of a game client
or its environment.

Modern implementations observe process handles, loaded modules, memory
integrity, and inline code modification.

{: .matters }
> fragcap is designed so that nothing it does resembles what anti-cheat
> watches for. Specification section 19.3's denylist exists because those
> techniques are the primitives detection systems monitor, and because none of
> them are needed for anything fragcap claims. Reconnaissance observed an
> anti-cheat launcher in one chain; fragcap records the relationship from
> creation-time telemetry and does not interact with the process.

**See also:** [Technique denylist](anti-cheat-and-security.md#technique-denylist)

## Technique denylist

The enumerated set of techniques fragcap will not use, regardless of
convenience. Each one is a way of reaching into a target process or interposing
on its traffic, and fragcap's whole posture is that it does neither. Every item
below therefore names a permitted alternative that recovers the same
information by observing from the outside.

- **Packet interception drivers.** A driver placed in the network stack to
  intercept or filter packets while they are in flight. Off-limits because it
  can alter or drop traffic, which is the opposite of observing it. fragcap
  reads from the NDIS capture driver, which copies packets without sitting in
  their path.
- **Code injection.** Writing fragcap's own code into a target process so it
  runs inside that process. Off-limits because it modifies the very process
  fragcap only means to name, and it is a primitive anti-cheat watches for
  directly. There is no alternative; it is out of scope.
- **Function hooking.** Redirecting a target's own function calls through
  fragcap's code. Off-limits for the same reason, reaching inside the process.
  Attribution comes from the socket table, read entirely from outside.
- **Process handles carrying memory-read rights.** Opening a handle to a target
  with the access needed to read its memory. Off-limits because reading another
  process's memory is exactly what detection systems look for. fragcap never
  reads a target's memory; it recovers process relationships from creation-time
  ETW telemetry and attributes flow ownership from the socket table, and any
  handle it does open declares query-only rights at the call site.
- **Layered service providers.** Inserting fragcap into the Winsock call chain
  that a socket's traffic passes through. Off-limits because it interposes on
  live traffic. The socket table yields the same process-to-flow association
  after the fact, without being in the path.
- **Executable image modification.** Patching a target's binary, on disk or in
  memory. Off-limits because it changes the thing fragcap claims only to
  observe. There is no alternative; it is out of scope.

{: .matters }
> Constitution principle P-1 makes the list absolute, and it is enforced at
> three points: as a constitution principle inherited by every agent session,
> as a dependency policy checked in continuous integration, and as a code
> review gate on process handle access rights. Naming each technique precisely
> is deliberate. It records, in plain terms, exactly what fragcap will not do,
> which is what lets the project be published as a passive observer rather than
> read as a tool for reaching into a game.

**See also:** [Anti-cheat](anti-cheat-and-security.md#anti-cheat)

## Detection signature

A row of data that identifies a game technology (an engine, an anti-cheat, or a
DRM product) from an install directory. It names a category, a match kind (a
filename, a directory shape, a PE version string, or a binary marker), a pattern,
the product, and a confidence. Signatures live in a table in the shipped catalog
database, so `fragcap catalog seed --tier signature` refreshes detection capability as
data rather than through a code change and a release.

{: .matters }
> A detected anti-cheat or DRM product is neutral evidence, never a gate
> (specification section 3.6). fragcap does not restrict, block, warn against, or
> discourage capture based on what it detects, and no output frames a title as off
> limits, risky, or discouraged. A single-player title that produces network
> traffic is one of the most interesting results the tool can surface, so a title
> with no recorded online mode is still fully capturable.

**See also:** [Signature matcher](anti-cheat-and-security.md#signature-matcher), [Binary marker](anti-cheat-and-security.md#binary-marker)

## Binary marker

A detection signature kind whose pattern names something inside an executable
rather than in the directory around it. One form is matched: `section:<glob>`,
which matches a name in the binary's own PE section table, and which is how
fragcap recognizes the Steam DRM wrapper by the `.bind` section it appends. Every
other pattern of this kind names a byte sequence that no shipped build matches;
such a row is carried and counted **inert**, so the signature load still
reconciles applied plus inert plus skipped to the number of rows loaded.

A section-marker scan reads only executables near the install root, bounded in
both depth and count, and reads only a bounded prefix of each one. A candidate
dropped by that count bound is counted and makes the scan incomplete, and a
candidate that could not be opened is recorded unreadable rather than treated as
carrying no marker.

{: .matters }
> Before this kind was matchable, fragcap reported "Steam DRM" for any title
> shipping `steam_api64.dll`, which is the Steamworks SDK redistributable and
> present in nearly every Steam title regardless of DRM. That recorded an
> observation nobody made, which principle P-9 forbids, on 28 of 32 rows.
> Matching the wrapper's own section is what made the label true.

**See also:** [Detection signature](anti-cheat-and-security.md#detection-signature), [Coverage state](command-line-and-diagnostics.md#coverage-state)

## Signature matcher

The single generic routine that evaluates the detection signatures against a
directory's shape. Detection behavior is a function of the signature table's
contents, not of per-product code, so adding a signature of an implemented kind is
honored on the next scan with no code change. A locally detected engine is stamped
`verified`, which outranks the `heuristic-unverified` engine attribution a remote
catalog carries: local evidence outranks a remote claim (principle P-9). The
matcher reads directory entries and, for a PE version string, the version resource
in a binary's own on-disk bytes; it opens no process handle and reads no process
memory (principle P-1).

**See also:** [Detection signature](anti-cheat-and-security.md#detection-signature)
