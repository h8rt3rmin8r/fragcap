# Phase 1 Data Model: Non-Profile Production Capture Path

This slice is command wiring; it adds no persisted data and no new schema. The
"entities" are the argument surface and the small in-memory shapes the branch
uses. Rust names are illustrative.

## RunArgs input group (changed)

`--profile`, `--install-dir`, `--steam` form one required, mutually-exclusive
clap `ArgGroup`.

- `profile: Option<String>` (was `String`) - a profile path, name, or game id.
- `install_dir: Option<PathBuf>` - an install directory to resolve the cascade
  over.
- `steam: Option<String>` - a Steam app id, resolved to an install directory.
- Group rule: exactly one present; violation is a usage error (exit 2).
- All existing `RunArgs` capture options are unchanged and apply to every input.

## Target::identity (new accessor)

A pure accessor on the existing `Target`:

- `Target::identity(&self) -> Option<&MatchPredicates>`
- Returns the resolved identity for a non-profile origin (`Observed`,
  `EngineRule`, `PlatformWalker`); returns `None` for a `Profile` origin.
- No new type; reads the origins' existing `identity()`.

## Synthesized profile (in-memory, transient)

Built in `run` from a resolved non-profile identity, then parsed and captured;
never persisted.

- `schema: 1`, `kind: "profile"`, `fidelity: "heuristic-unverified"`.
- `game`: a generic id/name placeholder; `app_id` set to the Steam app id when
  the input was `--steam` (a fact), otherwise absent.
- `stage`: exactly one - `role: "target"`, `lifecycle: "session"`,
  `terminal: true`, `match`: the identity's predicates (`exe`, `path_contains`,
  `path_regex`, `cmdline_contains`, `descends_from`, whichever are present),
  serialized from the `MatchPredicates`.
- Constructed via `Profile::parse` so an invalid identity surfaces as a profile
  diagnostic (exit 2), the same path `watch`/`tap`/authored profiles take.
- **Invariant**: fidelity is `heuristic-unverified`, never `authored` (P-9); the
  schema permits it on a profile and refuses only `observed`.

## Resolution request (reused)

- `--install-dir <path>` -> `ResolutionRequest::for_install(path, search, bundled)`.
- `--steam <app_id>` -> `install_root_for(app_id)` -> the resolved directory ->
  the same `for_install` request.
- `--profile <ref>` -> `ResolutionRequest::for_reference(ref, search, bundled)`
  (unchanged).

## Decline outcome (reused, newly rendered)

- `ResolutionError::Unresolved(u)` carries `ResolutionNotes`
  (engine-rule ambiguity, walker ambiguity, unreadable path, profile-not-found).
- The non-profile branch renders those notes into a surfaced failure message
  (exit 1); nothing is captured. The profile branch keeps the existing mapping.

## Relationships and boundaries

- The install-location inputs produce an install root; the resolver consumes it;
  a non-profile resolved target yields a `MatchPredicates` via `Target::identity`;
  the synthesizer turns that into a one-stage profile; the orchestrator captures
  it. Nothing here is a `PacketSource` or `FlowAttributor`; no schema changes; no
  persisted artifact.
