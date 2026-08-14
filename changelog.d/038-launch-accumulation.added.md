Each copy of fragcap now learns its own Steam games' launch executables locally
and privately. When a hint database is configured, a capture run first walks the
installed Steam library and, for each installed title, reads that title's launch
configuration from the machine's own application-info cache
(`appcache/appinfo.vdf`) into the local store's launch columns, so the hint
provider can name a socket-holding client the engine rule and platform walker
would miss. A hand-rolled parser reads the binary appinfo format (a different
format from the text VDF of `libraryfolders.vdf` and the `.acf` manifests),
framing each application's section by size so one malformed section is isolated
rather than losing the file. Titles already current are skipped by their appinfo
change-number, so the first run is slower and later runs are mostly skips, and a
user accumulates a growing personal collection over time. Nothing about any user's
library is shipped or shared; the distributed database still carries only the
public catalog and engine data. The read is passive: no network, no process
handle, only a file Steam already wrote. Every considered title lands in exactly
one counted outcome and the account is surfaced, so a partial walk cannot read as
complete.
