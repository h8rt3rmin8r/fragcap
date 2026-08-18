<!-- spec-impact: 15.5, 26.3 -->

### Landing page and getting-started rewrite, and site docs convergence (slice S057)

The public site and the first-run documentation now describe the tool that
shipped in S054/S055 rather than the profile-file tool that preceded it. The
landing page opens with the problem fragcap solves that standard capture does not,
stated for a visitor who has never thought about attribution, shows the real
`fragcap targets` hero listing as its worked example, carries the dependency-model
diagram (npcap required, Wireshark recommended, extcap optional), and directs the
visitor to getting started with a single call to action. The getting-started guide
is rewritten to end at a capture file on disk using `fragcap targets` then
`fragcap capture <n>`: prerequisites are acquired in "Before you begin" (framed
conditionally), the install step links the releases page and names the `.msi` /
`.zip` / `.sha256` assets, the verify step tells the reader to run the terminal as
Administrator and is the single home of the optional `fragcap extcap install`, and
the target step presents automatic discovery as the happy path and defines a Steam
App ID inline. The npcap narrative across the guide, the installer exit-dialog
prompt, and the S056 `doctor --fix` action now tell one coherent story
(detection-only, with the user-confirmed vendor-installer fetch permitted).

This resolves the getting-started QA batch: issues #130 (extcap command surfaced
before fragcap is installed), #131 (no download link), #132 (prerequisites
installed as a numbered step), #134 (run as Administrator; extcap home), #135 ("Get
a profile" unclear), and the documentation half of #133 (installer npcap
exit-dialog contradicting the docs narrative).

The reference set is converged onto the shipped command surface: the CLI reference
is rewritten to `capture`, `replay`, `targets`, `technologies`, `steam`, `doctor`,
`extcap`, `catalog`, and `schema` (no `run`/`tap`/`watch`/`profile`/`steam
profile`); the capture-modes guide uses `fragcap capture`; and the two pages that
taught authoring profile files (`guides/writing-a-profile`,
`reference/profile-schema`) are removed, with their navigation and inbound links
rerouted. No documentation page references the retired verbs, the retired profile
directory, the `--profile` selector, or a profile slug that no longer exists.

`fragcap capture` now accepts a stored-target selector positionally, so the
`fragcap capture <n>` form the `targets` listing hints (and the README and site
docs show) works as advertised: the positional is equivalent to `--target` and
mutually exclusive with it, resolving a handle, name, or row index the same way.
Before this, only the `--target` flag form parsed, so the listing's own hint was
rejected as an unexpected argument.

A small companion change removes the leftover `profile dir` identity row and the
`Profiles` section from `fragcap doctor` (in both the human report and `--json`),
so the getting-started sample is faithful to the binary and free of the retired
directory. The bundled profile set was already permanently empty and the user
profile directory unwritable after S054, so the rows reported dead surface; the
classifier keeps its exit status and every other row unchanged, and the internal
`Profile` capture-config type is untouched. Specification sections 15.5 and 26.3
are reconciled to match.

IGDB enrichment and its credential walkthrough (issue #144 stretch goal) are
deferred to a dedicated slice: the codebase has no IGDB or credential-storage
plumbing (the S050 local.db columns the handoff plan assumed were never built), so
documenting a credential-registration flow with no consumer would describe unbuilt
functionality (P-11). A dedicated slice should carry the storage, the fetch, and
the walkthrough together.
