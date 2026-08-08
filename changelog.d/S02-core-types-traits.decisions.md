**2026-08-08** The dependency direction check no longer requires
`fragcap-core` to have zero dependencies. It now checks against a named
allowlist, currently one crate. The empty-set rule was stricter than the
principle it enforces: constitution P-2 forbids a platform-specific dependency,
an I/O crate, and a capture library, not every dependency. The rule would have
blocked a pure-Rust buffer crate on a reading the constitution does not support.
The check still fails closed, so anything not on the list is a problem and
adding to the list is a deliberate edit a reviewer sees.

**2026-08-08** Recorded for promotion to specification section 29:
sections 8.4 and 8.5 reference eight types in their signatures and define none
of them. They are `Timestamp`, `Bytes`, `StageId`, `LinkType`, `Endpoint`,
`FilterProgram`, `ProcessEvent`, and `ProcessRecord`. Slice S02 defines all
eight, with the three that later slices own documented as provisional:
`FilterProgram` is settled by S13, and `ProcessEvent` and `ProcessRecord` by
S11. This is a gap in the architecture of record rather than a decision it made,
and the constitution requires the divergence be recorded rather than resolved
silently.

**2026-08-08** The minimum supported toolchain check builds at the declared
minimum instead of with the pinned toolchain. It previously ran an ordinary
build and reported success, which checked the pinned toolchain and said nothing
about the minimum. That was harmless while the dependency graph was empty and
every declared minimum passed trivially, and stopped being harmless the moment a
real dependency arrived. It now builds through `rustup run`, into a separate
target directory so it does not try to replace the running task runner binary,
and exits 2 when the minimum toolchain is not installed.
