**2026-08-12** The target resolution cascade (issue #77) landed its resolver
core, and three decisions were recorded rather than left implicit. First, the
targeting fidelity tier and the attribution fidelity are kept as separate
mechanisms on separate types in separate crates: the new `FidelityTier`
(`authored`/`verified`/`heuristic-unverified`/`observed`) lives in
`fragcap-profile` and is what the resolver ranks by, while the pre-existing
`Fidelity` (`Live`/`Retained`/`None`) in `fragcap-core::attribution` describes how
a packet was attributed. Issue #77's prose draws an analogy between them that is a
trap; conflating them would let an observed target read as a live attribution.
`FidelityTier`'s variants are declared in ascending trust order so the derived
`Ord` makes the more trusted tier the greater value, and a provider precedence that
inverted the order would be caught by comparing against it. Second, precedence is a
provider ordering imposed by the resolver (it sorts providers before querying), not
a property the caller's registration order supplies; a permutation test proves the
result does not depend on registration order, the same discipline the attribution
join in `fragcap-attr` already holds. The issue's top two chain positions (authored
package, verified profile) collapse into one profile provider, because the section
15.3 lookup already returns one profile and the authored-versus-verified
distinction is carried by the fidelity the file declares. Third, the resolver, the
`Target` type, and the providers live in `fragcap-profile`, the crate that already
reads both the profile schema and the process tree; nothing is added to
`fragcap-core` (allowlist stays `["bytes"]`) and no external crate is added. The
CLI `run` path flows through the resolver and maps its outcomes onto the existing
`CliError` classes so exit codes and messages are unchanged; capture output is
byte-identical. MSRV stays 1.82.

Four refinements landed from PR review, each tightening an invariant the first
cut left to convention. First, a capture profile may no longer declare
`fidelity: observed`: the profile-load path refuses it with a new
`observed-profile-fidelity` diagnostic, because `observed` is the observation
provider's runtime stamp and allowing it on a profile would let the
top-precedence provider answer below a lower one's fidelity, inverting the rank
the resolver is built to hold. Second, `TargetResolver::new` now returns a
`Result` and refuses two providers at one `Precedence` (a `DuplicatePrecedence`
error, mirroring `BundledSet::new`'s duplicate-`game.id` refusal); one provider
per position is what makes the order total, so registration-order independence
is guaranteed by construction rather than by the built-in set happening to be
unique. Third, an observed `Target` now retains the `MatchPredicates` identity
that selected it, not only the process that was live at resolution, so the
target carries its match rules (section 15.7) and a later capture can re-match
after a restart. Fourth, the observation matcher (`first_live_match`) evaluates
the image-and-path predicates only (`exe`, `path_contains`, `path_regex`) and
treats an identity with no image-or-path anchor as matching nothing rather than
every process; `cmdline_contains` and `descends_from` are stage-matching
concerns outside the section 15.7 observation identity, which reads only the
image name and path.
