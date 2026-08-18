<!-- spec-impact: none -->

**2026-08-18** npcap license determination for the `doctor --fix` fetch action
(slice S056, issue #143). The npcap LICENSE
(https://github.com/nmap/npcap/blob/master/LICENSE) grants free use (install and
use five copies, unlimited when used only with Nmap, Wireshark, or Microsoft
Defender for Identity) and prohibits redistribution and transfer of the Software
Product ("not open source software and may not be redistributed or used in other
software without special permission"; a licensee "may not ... redistribute,
encumber, sell, rent, lease, sublicense, or otherwise transfer" it). No clause
restricts a user, or a tool acting on the user's machine, from downloading the
vendor's official installer and running it. Determination: fragcap fetching the
vendor's own signed installer from the official location and launching it, while
embedding, copying, hosting, or caching nothing in any fragcap artifact, does not
redistribute npcap and is permitted. Guardrails carried into the implementation:
fetch only from the vendor's official location, store nothing as fragcap's own,
and act only under an explicit interactive confirmation.

**2026-08-18** Constitution amendment authorized by the operator (slice S056):
Licensing rule 2 changed from an absolute ("It never downloads, installs, or
invokes an installer") to a narrow, user-confirmed carve-out permitting fragcap to
fetch and launch the vendor's own signed installer under an explicit interactive
confirmation (as in `doctor --fix`), storing nothing in any fragcap artifact and
redistributing nothing. Rules 1 (no bundling), 3 (documented prerequisite), and 4
(no SDK vendoring) stay absolute; P-1 and P-9 are untouched. Constitution version
1.2.0 -> 1.3.0 (MINOR: an existing section materially expanded). Recorded here per
the amendment policy; the reasoning is in the constitution's Sync Impact Report.

**2026-08-18** `http_req` added as an optional dependency of `fragcap-cli` behind
the `net` feature, for the npcap installer fetch. It is already in the workspace
graph via `fragcap-targets`, so this adds no package to `Cargo.lock`; the `net`
feature is off by default, so the shipped end-user build never compiles it, and
`cargo xtask msrv` (default features) never sees it.
