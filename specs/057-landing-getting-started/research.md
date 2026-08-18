# Phase 0 Research: Landing page and getting-started rewrite

All unknowns are resolved by reading the shipped source (S054/S055/S056) rather
than by external research; this is a convergence slice, so "research" is
establishing the shipped reality the docs must mirror.

## Decision 1: The command surface the CLI reference must document

**Decision**: Document exactly the nine commands declared in
`crates/fragcap-cli/src/cli.rs`: `capture`, `replay`, `targets`, `technologies`,
`steam`, `doctor`, `extcap`, `catalog`, `schema`, grouped as the binary's own
`--help` groups them (Capture / Targets / Environment / Data). No `run`, `tap`,
`watch`, `profile`, or `steam profile`.

**Rationale**: `cli.rs` is the single source of truth (clap-derived, so the help
and the doc cannot drift). The README quickstart (updated by S054) already mirrors
this surface and is a correct model. See
[contracts/cli-reference-surface.md](contracts/cli-reference-surface.md).

**Alternatives considered**: Copying the binary's `--help` verbatim into the page
(rejected: the page is the readable version with per-flag tables and prose, per the
existing page's own framing); leaving cli.mdx and only fixing the retired lines
(rejected: the page documents whole retired commands `run`/`tap`/`profile`/`steam
profile`, so a line fix is not possible, a rewrite is required).

## Decision 2: What to do with the two profile-file pages

**Decision**: Delete `guides/writing-a-profile.mdx` and
`reference/profile-schema.mdx`. Reroute inbound links: the landing page capability
bullet and `index.mdx` Guides list point at getting started / discovery;
`architecture.mdx`'s "writing a profile" link is replaced with the same
conceptual anchor it already carries (the process-tree / `descends_from`
explanation lives on that same Architecture page); `meta.json` drops both nav
entries.

**Rationale**: Profiles are no longer user-authored files after S054 (no `profile`
command, no `--profile` selector, no profile directory). A page teaching authoring
of a file the tool no longer reads is worse than absent (P-11). The conceptual
content (stages, `descends_from`, match predicates) already lives on
`architecture.mdx` under "Naming an indirectly launched client," and the current
master schema is documented by `reference/target-schema.mdx`, which stays. So no
information is lost by deletion.

**Alternatives considered**: Redirect stubs (rejected: the static export has no
configured redirect mechanism, and a pre-1.0 docs site can drop routes); reframing
the pages around the internal Profile type (rejected: the internal type is not a
user artifact and documenting it as one re-introduces the confusion the slice
removes).

## Decision 3: The doctor companion change scope

**Decision**: Remove the `profile dir` identity row (checks.rs) and the entire
`Profiles` section (the `PROFILES` constant, the `profiles(inputs)` check, and its
`push`), plus the now-dead `Inputs` fields `profile_dir`, `bundled_count`,
`user_count` and the probe plumbing that computes them (`count_profiles`, the
`user_profile_dir` count call, the `bundled().len()` call). Keep `paths.rs`
`user_profile_dir`/`bundled()` only if still referenced by non-doctor code; if the
only remaining references are the doctor probe and their own tests, remove them
too. The identity section becomes `version`, `binary`, `catalog db`, `local db`.

**Rationale**: The bundled set is permanently empty (`paths.rs` says so and
`bundled()` returns `BundledSet::empty()`), and the user profile directory is
unwritable after S054 (no command puts a file there), so both counts are dead
surface reporting the retired directory. FR-017/FR-018 require removing the
reporting, not the internal `Profile`/`BundledSet`/`SearchPath` capture-config
types, which `capture.rs` still uses.

**Alternatives considered**: Leaving the rows and trimming them from the doc
sample only (rejected: the sample would then lie about the binary, failing FR-019,
and the binary would still emit the retired directory, failing FR-017/SC-002);
keeping `profiles: bundled: 0` as a forward-looking row for a future bundled-profile
feature (rejected: reporting a permanently-zero count of a retired concept is noise
the slice exists to remove, and a future feature can re-add a truthful row then).

## Decision 4: The landing worked example

**Decision**: Render the S055 hero listing as the persuasive asset: numbered `# /
TARGET / CAPTURE / KNOWN` columns with two example rows and the trailing `fragcap
capture 1` hint, in the monospace face on the dark ground, matching the shipped
`fragcap targets` output and the README quickstart.

**Rationale**: Section 23.1 (amended) names the worked command-and-output as the
page's primary persuasive asset, and S055 section 9.5 specifies this listing as
the hero output. Using the real output keeps the page honest and current.

**Alternatives considered**: A `fragcap capture` run transcript (rejected: capture
output is less legible as a first-glance asset and depends on a running game; the
targets listing is the S055-designated hero); keeping a synthetic `run --profile`
transcript (rejected: retired verb and slug).

## Decision 5: Verifying the retired-token criterion

**Decision**: Verify SC-002 by grepping `site/` for the enumerated tokens
(`fragcap run`, `fragcap tap`, `fragcap watch`, `steam profile`, `profile
validate`, `--profile`, `\bprofiles\b` as a directory, and the `eso`/`<game-id>`
profile slugs used as `--profile` args) and by building the site (no broken
links). The documentation linter (`cargo xtask docs check`) covers the glossary
and canonical docs, not the site MDX, so it is a necessary but not sufficient gate;
the grep is the SC-002 gate. See
[contracts/retired-token-inventory.md](contracts/retired-token-inventory.md).

**Rationale**: Establishes an objective, repeatable check for the site-wide
acceptance criterion that the linter does not itself enforce.

**Alternatives considered**: Relying on the linter alone (rejected: it does not
scan site MDX); a bespoke site-content linter (rejected: out of scope and would
touch pinned tooling patterns; a grep in the quickstart validation is sufficient).
