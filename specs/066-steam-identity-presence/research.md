# Phase 0 Research: S066

Nine questions, resolved against the constitution, the three source issues' own
stated tradeoffs, and the existing code (`fragcap-steam`, `fragcap-targets`,
`fragcap-cli`). Five of the nine were already settled in the spec's
Clarifications section (the Music-type exclusion, the CAPTURE label, the next-command
skip, the `&` expansion, and the semantic-divergence rule); the remaining four are
resolved here because they are implementation-shaped rather than requirement-shaped.

| ID | Question | Answer |
| --- | --- | --- |
| R-1 | How does the walk learn a Steam app's install-directory type? | Read `common/type` from `appcache/appinfo.vdf`, which `fragcap-steam` already parses for launch entries; extend the same parsed tree's extraction rather than a second pass. |
| R-2 | Which app types redirect off `steamapps/common/`? | Only `Music` (case-insensitive). Every other type, and an unknown type, keeps the existing `common/<installdir>` assumption, which is the minimal change that fixes the reported defect without touching the dominant case (FR-003). |
| R-3 | What happens when the type cannot be determined at all? | Fall back to `common/<installdir>`, the current behavior. An appinfo read or parse failure degrades to the status quo, never a new failure mode. |
| R-4 | Where does the appid→type map live, and how does it reach `read_manifest`? | `discover_in(root)` reads `appcache/appinfo.vdf` once per call and threads a lookup down through `read_library_titles` into `read_manifest`, which already receives the per-title context needed to join the install root. A missing or unreadable appinfo cache yields an empty map (matches `read_appinfo`'s existing "no cache is not an error" contract), so every title falls back to R-3's default. |
| R-5 | Does `InstalledTitle` need new fields, and what shape? | Three verbatim, observed fields, all optional except the required raw installdir string: `installdir: String` (the raw manifest value, independent of which subdirectory it was joined into), `app_type: Option<String>` (the raw appinfo `common/type`), `launch_executable: Option<String>` (the first appinfo `config/launch` entry's executable). None is reconstructed from another (P-9); each is read once, at the same pass, and carried through. |
| R-6 | How does a Music-type candidate get excluded from registration without breaking discovery-account conservation? | It is counted through the existing `DiscoveryAccount::considered_not_a_game` outcome (already used by the known-roots walk for a directory matching no signature), incrementing `considered` and `considered_not_a_game` and never reaching `CandidateTarget`. No new outcome bucket, so `is_conserved()` needs no change. |
| R-7 | How are the three names stored on `TargetEntry` without duplicating `launch_entries`' socket-holder semantics? | Two new nullable columns, not a reuse of `launch_entries`: `folder_name TEXT` (the raw installdir/folder-identifying value) and `executable_hint TEXT` (the raw observed launch executable name). `launch_entries` stays exactly what it is today, a resolved-or-unresolved *capture* chain gated by the socket-holder decision (P-9); folding the executable hint into it would either fabricate a socket-holder claim fragcap never made, or require touching `capture_readiness`/`entry_windows_clients`/the real capture path, none of which this slice's scope (findability, not capture behavior) calls for. |
| R-8 | How does selector resolution add substring matching without regressing existing exact-match behavior? | Three tiers, in order: exact handle (unchanged) → case-insensitive exact name (unchanged, so an existing exact-name collision still resolves exactly as it does today) → only if neither hits, a case-insensitive substring match across `name`, `folder_name`, and `executable_hint`, deduplicated by target. A naive single-tier substring match was rejected: it would turn today's unambiguous "Portal 2" (an exact match) into an ambiguity against "Portal 2 Beta" the moment both exist, which is a real regression the acceptance scenarios do not ask for and FR-009's no-drift guarantee forbids. |
| R-9 | Where does the missing-install-root color palette come from, given `doctor`'s `use_color()` and its ANSI Warn code are both private to `crates/fragcap-cli/src/commands/doctor.rs` and `crates/fragcap-cli/src/doctor/mod.rs` respectively? | Promote both to one small, shared module (`crates/fragcap-cli/src/color.rs`, `pub(crate)`) that `doctor` and `targets` both import: `use_color()` unchanged in behavior, plus a `WARN`/`RESET` constant pair matching doctor's existing values exactly. `doctor.rs`/`doctor/mod.rs` are refactored to call the shared module instead of keeping a private copy, a zero-behavior-change extraction, so the two surfaces cannot independently drift on what a warning looks like (the repository's own precedent for shared vocabulary, P-6/P-10 in spirit). |

## Note text and rendering (R-10, implementation detail)

The missing-install-root note is prefixed to the SENSITIVITIES cell value, the one
column `render_table`'s doc comment already documents as free-running and exempt from
padding or truncation (`crates/fragcap-cli/src/commands/targets.rs:216-233`), so
adding a prefix for an affected row cannot perturb the TARGET/CAPTURE/ENGINE column
widths computed from the other rows. The unaffected row's cell is the untouched
return value of `sensitivities_summary`, so FR-009's byte-identical guarantee holds by
construction rather than by a runtime branch that could drift. The exact prefix text
is `install folder not found`, joined to any existing sensitivities content with `; `
(and replacing the `-` clean marker outright, since `install folder not found; -` reads
worse than `install folder not found` alone). In color mode the whole cell is wrapped
in the shared Warn color; in plain mode it is exactly the same text with no escapes.

## Handle derivation vector update (R-11)

The `&` expansion (Clarifications) is implemented as one new step in
`crate::handle::normalize`, before the existing apostrophe/quote deletion step:
replace each literal `&` with the three characters ` and ` (padded so it does not
glom onto adjacent letters: `Ivy & Piper` → `Ivy and Piper`, not `IvyandPiper`). The
existing unit test vectors in `crates/fragcap-targets/src/handle.rs` and any
reference vector table in `docs/fragcap-specification.md` Appendix A that exercises
an `&`-bearing name are updated in the same change (a task locates every such vector
by searching for `&` in the test module and the appendix, rather than trusting this
research document to enumerate them exhaustively).
