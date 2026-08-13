# Phase 0 Research: Technology-Detection Surface

All items below were open questions in the Technical Context or in the spec's
plan-time assumptions. Each is resolved here with a decision, its rationale, and
the alternatives considered.

## R1. Which upstream commit to pin, and how to acquire it

- **Decision**: Pin the vendored `rules.ini` to SteamDB
  `FileDetectionRuleSets` commit `243cf741921d2c8fd6b844f83831edf4692cf788`
  (authored 2026-04-01, subject "Add support for detecting usage of Mono or
  .NET"). Acquire the exact file at that commit
  (`https://raw.githubusercontent.com/SteamDatabase/FileDetectionRuleSets/243cf741921d2c8fd6b844f83831edf4692cf788/rules.ini`)
  and store it verbatim, normalized only to LF line endings and UTF-8 without BOM
  per the house text-hygiene rule.
- **Rationale**: Pinning a commit, not `master`, makes the lock hash meaningful
  and the vendored version reproducible and auditable. The file is fetched at the
  pinned commit so the committed bytes and the recorded commit agree.
- **Alternatives considered**: Tracking `master` (rejected: a moving target makes
  the hash and the "faithful copy of a known version" property meaningless). A git
  submodule (rejected: pulls the whole repo and its history for one file, and the
  constitution's spirit is a committed, self-contained asset with an attribution
  and a lock, like `skills-lock.json`).

## R2. Rust regex (RE2) incompatibility with the PCRE-style ruleset

- **Decision**: Compile each pattern independently with the `regex` crate. On a
  compile error, skip that single pattern, increment a skipped counter, and record
  the technology and category it belonged to. Never edit the vendored bytes to
  make a pattern compile. Assert `compiled + skipped == total_patterns` in a test.
- **Rationale**: The ruleset is authored for a PCRE-style engine and its own
  header says it uses constructs the RE2-family `regex` crate does not support:
  possessive quantifiers appear directly (for example `mus\_\w++\.ogg` and
  `snd?_\w++\.ogg` in `[Evidence]`), and other PCRE-only constructs may appear
  elsewhere. RE2 guarantees linear-time matching by excluding backreferences and
  possessive/atomic groups, so these cannot be compiled and must not be silently
  dropped (P-4). A per-pattern compile isolates the failure to one rule instead of
  discarding a whole section.
- **Normalization note**: The applied categories this slice targets (`Engine`,
  `AntiCheat`, `SDK`, `Emulator`, `Container`, `Launcher`) are dominated by simple
  anchored path/extension patterns (`(?:^|/)...`, `\.ext$`) that compile cleanly;
  the possessive-quantifier cases observed are concentrated in `[Evidence]`, which
  this slice does not apply. The skip machinery still runs for the applied
  categories so any incompatible pattern there is counted and surfaced rather than
  assumed absent.
- **Alternatives considered**: Pre-transforming patterns (for example rewriting
  `++` to `+`) to coerce compilation (rejected: it silently changes the ruleset's
  meaning, defeats the faithful-copy property, and is exactly the "normalize a
  value that looked malformed" move P-9 warns against; a skipped pattern that is
  counted is honest, a rewritten pattern that matches differently is not).
  Swapping in a PCRE crate such as `fancy-regex` or `pcre2` (rejected: a new
  dependency and lockfile crate, violating the slice constraint, and `pcre2`
  brings a C library; the value is detecting the common technologies, which the
  compatible majority already covers).

## R3. How the INI-ish ruleset is parsed

- **Decision**: A minimal hand-rolled line parser, not an INI-library dependency.
  Lines are: a section header `[Name]`; a comment beginning with `;`; a blank
  line; or a rule `Key = pattern` where `Key` may end in `[]` to denote one of
  several markers for the same technology. A trailing `; comment` on a rule line
  is stripped. The pattern is the text between `=` and an unescaped trailing
  comment. Section and key names are treated case-sensitively as written; the
  regex itself is compiled case-insensitively (the ruleset header states matching
  is case-insensitive).
- **Rationale**: The format is trivial and fully described by the file's own
  header comment; a parser is a few dozen lines. Adding an INI crate would add a
  lockfile crate for no benefit, against the slice constraint and the project's
  established "hand-roll the trivial format" pattern (the glob matcher, the pcapng
  writer, the schema validator).
- **Edge**: A `; ` inside a regex character class is not a comment. The parser
  only treats a `;` as a comment delimiter when it is preceded by whitespace and
  not inside the pattern's leading run, matching how the upstream inline comments
  are written (`KEY = pattern$ ; note`). Patterns that would be ambiguous are rare
  in the applied categories; any that mis-split simply fail to compile and are
  counted as a skip rather than silently mis-detected.
- **Alternatives considered**: `rust-ini`/`configparser` (rejected: new crate;
  and the `Key[]` array form and inline-comment rules are ruleset-specific enough
  that a generic INI parser would need post-processing anyway).

## R4. The directory scan and its bound

- **Decision**: A depth-bounded recursive walk of the install directory using
  `std::fs::read_dir`, collecting relative file paths with `/` separators (to
  match the ruleset's path convention) and matching each compiled pattern against
  each relative path. Bound the recursion at a fixed maximum depth (proposed: 8
  levels below the install root) and continue past an unreadable subdirectory by
  recording its path as an unreadable condition rather than aborting.
- **Rationale**: The ruleset matches on the full depot path, so the scan must see
  relative paths, not just filenames. A depth bound keeps the walk affordable on a
  large install and mirrors S029's bounded engine-rule scan; depth 8 comfortably
  covers real marker locations (`game/bin/win64/...`, `EasyAntiCheat/...`,
  `*_Data/...`) while excluding pathological deep trees. Surfacing an unreadable
  subtree (not swallowing it) is the S029 `Unreadable` precedent and P-4.
- **Open for tasks**: the exact depth constant and whether to also cap total
  entries visited are tuning details finalized in implementation; the bound's
  existence and the unreadable-surfacing behavior are fixed here.
- **Alternatives considered**: Unbounded walk (rejected: a symlink loop or a
  pathological install could make detection run unboundedly; also inconsistent
  with S029). Filename-only matching (rejected: the ruleset keys on paths, so
  filename-only matching would both miss path-anchored rules and mis-fire
  filename rules that upstream anchors to a directory).

## R5. Deduplication and the reported marker path

- **Decision**: Report each detected technology once per category, keyed by
  (category, technology name). When several marker files match one technology, keep
  the first match encountered in a deterministic scan order (sorted directory
  traversal) as the representative marker path. Distinct technologies that share a
  marker file are each reported.
- **Rationale**: An operator wants "this game uses EasyAntiCheat", not one line
  per EAC file. Deterministic ordering makes the representative path stable across
  runs (important for the scaffold artifact's reproducibility and for golden
  tests). Reporting per technology, not per file, is FR-011.
- **Alternatives considered**: Listing every matching path (rejected: noisy and
  unstable); reporting only the technology with no path (rejected: the marker path
  is the auditable evidence that keeps the heuristic honest, P-9/CHK013).

## R6. SHA-256 without a new dependency

- **Decision**: Hand-roll SHA-256 in a small `sha256.rs` in `fragcap-profile`,
  validated against published NIST test vectors (empty string, "abc", and the
  56-byte multi-block vector) plus a round-trip on the committed asset. A test
  hashes the embedded `rules.ini` and asserts equality with the hash recorded in
  `rules.lock.json`.
- **Rationale**: The slice constraint is no new lockfile crate; `sha2` (or any
  crypto crate) would add one, and adding it only as a dev-dependency still writes
  it into `Cargo.lock`. SHA-256 is a fixed, well-specified algorithm (~200 lines)
  that is fully testable against standard vectors, exactly the kind of trivial-
  but-load-bearing code the project already hand-rolls to avoid dependencies. The
  check lives in `cargo test`, which is already in the gate, so no workflow or
  toolchain (pinned-artifact) change is needed.
- **Reproducibility**: The hash is computed over the committed bytes as stored
  (LF, UTF-8, no BOM). The lock's `note` states this so an external `sha256sum` on
  the committed file reproduces the recorded value, and a re-vendor from the pinned
  commit (normalized identically) reproduces it too.
- **Alternatives considered**: `sha2` crate (rejected: new lockfile crate).
  Computing the hash only in `xtask` with a dependency (rejected: still a lockfile
  crate, and splits the check out of the default test gate). A non-cryptographic
  checksum such as CRC32 (rejected: the operator asked for SHA-256, and a
  cryptographic hash is the right integrity primitive for a supply-chain lock).

## R7. Third-party attribution placement vs the license xtask

- **Decision**: Store the SteamDB attribution as a distinctly named nested file,
  `crates/fragcap-profile/assets/steamdb/THIRD_PARTY_NOTICES.md`, containing the
  MIT license text and the `Copyright (c) 2021 SteamDB` line. Do not create a bare
  `NOTICE` file in the asset directory or anywhere the `license` xtask scans.
- **Rationale**: `cargo xtask license` requires every publishable crate to carry a
  root `LICENSE`/`NOTICE`/`README.md`, with `LICENSE` and `NOTICE` byte-identical
  to the repository-root originals (the Apache-2.0 texts). A second file named
  `NOTICE` in the crate, even nested, invites confusion and risks the check's
  intent; a distinctly named third-party notice keeps the Apache-2.0 crate notice
  and the MIT asset attribution unambiguous and separate. MIT only requires the
  notice to travel with the copy, which a file beside the asset (published inside
  the crate package) satisfies.
- **Alternatives considered**: A single combined `NOTICE` merging Apache-2.0 and
  the SteamDB MIT text (rejected: breaks the byte-identical mirror the license
  check enforces). A repository-root `THIRD-PARTY-LICENSES` directory (rejected:
  the attribution must travel inside the published crate that carries the asset;
  a root-only file would not be packaged with `fragcap-profile`).

## R8. Text-hygiene linter vs a verbatim third-party file

- **Decision**: The vendored `rules.ini` is stored LF, UTF-8, no BOM (a
  normalization the MIT license permits and that does not change regex meaning). It
  may contain characters the house linter would flag in first-party prose (for
  example non-ASCII in a comment, or a long line). Confirm during implementation
  whether the documentation/convention linters scan `assets/**`; if they do and
  the vendored content trips a rule, add a scoped exclusion for the vendored asset
  path (recorded as a decision) rather than editing the third-party bytes.
- **Rationale**: Editing a vendored file to satisfy a first-party style rule
  breaks the faithful-copy and lock-hash properties. The correct move is to scope
  the linter to exclude the clearly-marked third-party asset directory, if needed.
- **Alternatives considered**: Editing the file to pass the linter (rejected:
  breaks the lock and the license faithful-copy property). Leaving a failing gate
  (rejected: the gate must be green).

## R9. Scope of the schema change

- **Decision**: Add an optional top-level `technologies` array and a
  `$defs/technology` object to `target-schema.v1.json`, in both the embedded copy
  and the `docs/schema` copy (kept byte-identical, per the existing drift check),
  and extend the hand-rolled variant validator in `variants.rs` to accept and
  shape-check it. No schema-version bump.
- **Rationale**: The addition is backward compatible under a previously
  `additionalProperties: false` schema (old artifacts still validate; new ones now
  validate). The two schema copies have a drift check, so both must change
  together. The validator is hand-rolled and enforces the closed property set
  itself, so it must learn the new property or it would reject a conformant
  artifact.
- **Alternatives considered**: A new schema version 2 (rejected: an additive
  optional field does not warrant a version bump and would ripple through every
  validator and fixture). A separate sidecar schema for technologies (rejected:
  the technologies belong to a target artifact the master schema already governs;
  a sidecar would fragment the single-vocabulary property S025 established).

## R10. CLI surface shape

- **Decision**: A new top-level subcommand that takes an install-directory path
  and prints the grouped report; findings are ordered by category (a fixed
  category order) and, within a category, by technology name. Exit code 0 on a
  successful scan (including an empty result); a surfaced non-zero path for an
  unreadable target directory, following the CLI's existing exit contract. The
  exact command name and flag are fixed in the CLI contract
  (`contracts/cli-technologies.md`).
- **Rationale**: A dedicated subcommand matches the existing command surface
  (`doctor`, `schema`, `profile`, `steam`, `tap`, `watch`) and keeps detection
  independent of `run` (FR-013a). A stable ordering makes the output testable.
- **Alternatives considered**: Folding detection into `doctor` or `profile`
  (rejected: those have distinct jobs; a first-class capability deserves a
  first-class command and a clean test surface). Running detection inside `run`
  (rejected by clarification: puts a filesystem walk on the capture path and would
  tempt writing into the packet stream, against P-5).
