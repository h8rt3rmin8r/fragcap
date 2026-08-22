<!-- spec-impact: none -->
**2026-08-21** The observed launch executable a Steam title's appinfo cache
names is stored as a new, dedicated `executable_hint` column, not folded into
the existing `launch_entries` field. `launch_entries` carries the
socket-holder decision's guarantee that fragcap never claims a process holds
the gameplay sockets without having been told so by the user or by observing a
capture; writing an executable there for a title nobody has authored or
captured yet would either fabricate that claim or require touching
`capture_readiness` and the real capture path, neither of which this change's
scope (findability, not capture behavior) called for. The local store schema
advances from version 7 to version 8 for this and for a second new column,
`folder_name` (the raw installdir), both nullable `TEXT` with no CHECK
constraint: unlike `detection_scan`'s enum, both are free-form observed strings
with no closed vocabulary to enforce.

**2026-08-21** Selector resolution gained a third tier, a case-insensitive
substring match against the name, folder name, and executable hint, but only
once an exact handle and an exact name both miss. A single-tier substring match
was rejected: it would turn an existing unambiguous exact-name selection (for
example `Portal 2`) into a reported ambiguity the moment a superstring name
exists (`Portal 2 Beta`), which is a regression the existing exact-match
behavior never had.

**2026-08-21** `doctor`'s `use_color()` predicate and its warning-color ANSI
codes move to a new, shared `crate::color` module in `fragcap-cli`, and
`doctor` itself is repointed at it. `targets` needed the identical palette for
its missing-install-root note, and keeping two independently maintained copies
of the same two escape codes is exactly the drift the issue's own instruction
to "follow the doctor convention" was written to prevent.

**2026-08-21** The `&` handle-derivation change is forward-looking only.
Already-registered targets keep the handle they were given at registration
time; nothing re-derives or migrates a stored handle. `Ratchet & Clank` now
derives `ratchet_and_clank` for a title registered after this change, where it
would previously have derived `ratchet_clank`.
