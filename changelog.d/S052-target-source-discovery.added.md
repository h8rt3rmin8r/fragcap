<!-- spec-impact: 7 -->

Installed games can now be discovered rather than described by hand. Every origin
of a capture target, a Steam walk, a scan of the known game-install roots, or a
directory the user points at, is a discovery source behind one seam, so single
authoring and bulk platform walking are the same operation at different batch
sizes: adding Epic, GOG, Xbox, Battle.net, or an emulator ROM directory later is a
new implementor with no downstream change.

`fragcap targets discover` walks Steam (each installed title joined to the shipped
catalog for its classification) and, on Windows, the known game-install roots
across every eligible fixed volume, and lists what it finds. A machine with no
Steam still lists games whenever a known root exists, on any drive, not only the
system drive. `fragcap targets scan <dir>` points discovery straight at one folder.
Each listing surfaces a conserved account, so an excluded volume or an unparsable
title is counted and shown rather than dropped silently.

The cross-volume walk is kept safe by a persistent volume eligibility allowlist in
`local.db`, seeded permissively with the fixed volumes present at first run and
requiring an explicit opt-in for any volume that appears later or misreports itself
as fixed (a userspace or FUSE mount). Volumes are keyed on their stable GUID
identity, not a reassignable drive letter. The known-roots walk classifies a
directory by its shape and stops descending on a hit rather than enumerating every
executable on the machine; the signature matcher that generalizes this beyond
curated roots lands in a later slice, along with deep filesystem scanning.

Two operator decisions shaped the slice and are recorded in the spec's
clarifications: the eligibility table is permissive-seeded then behaves as an
allowlist, and discovery surfaces candidates live and persists one only when the
user acts on it, so a scan result becomes a stored target through the same entry
model every source shares. The pure discovery model (the seam, the tiers, the
account, the classifier seam, and the eligibility store) lives in `fragcap-targets`
and adds no new dependency and no new inter-crate edge; the two platform adapters
(the Steam walk and the Win32 volume inventory) live in the `fragcap` facade, the
one crate that already depends on both leaf crates. Wiring the interactive,
one-step scan-confirm-author flow into the command line is deferred to the S055
targets hero command; the `InteractiveSource` seam it uses ships here.
