# Data Model: Profile Schema, Parsing, and Validation

**Slice**: S05 | **Date**: 2026-08-09 | **Plan**: [plan.md](plan.md)

Phase 1 output. The types this slice adds, what each one guarantees, and where
the guarantee comes from. Field names are the Rust ones; the TOML surface they
correspond to is in [contracts/profile-schema.md](contracts/profile-schema.md).

## The validated profile

Every type in this section is constructible only by parsing. There is no public
constructor, no public field mutation, and no `Default`, because a default
profile would be a profile that passed no check. That is FR-011, and it is the
reason S12 can treat a `Profile` as a set of facts rather than a set of claims.

### `Profile`

| Field | Type | Guarantee |
| --- | --- | --- |
| `schema` | `u32` | Exactly 1. A value the crate does not support never reaches here. |
| `game` | `Game` | Present. |
| `capture` | `CaptureDefaults` | Present, possibly entirely absent values. |
| `stages` | `Vec<Stage>` | Non-empty, declaration order preserved, at least one non-service, at most one terminal, role names unique, `descends_from` acyclic and resolving within the set, no ambiguous image match. |

The stage-set guarantees are the semantic checks, and stating them here rather
than only in `validate.rs` is deliberate: a reader asking what a `Profile` is
worth should find the answer beside the type.

### `Game`

| Field | Type | Guarantee |
| --- | --- | --- |
| `id` | `GameId` | A validated slug. |
| `name` | `String` | Non-empty, verbatim. |
| `platform` | `Option<String>` | Absent means the profile declared none. |
| `app_id` | `Option<String>` | Absent means the profile declared none. |

`platform` and `app_id` stay strings in this slice. Section 16 gives `platform`
meaning for managed launch, and constraining it to a known set now would be
guessing at S17's vocabulary from outside it.

### `GameId`

A newtype over `String` holding a non-empty slug of lowercase ASCII
alphanumerics, hyphen, and underscore. It exists as a type rather than a
validated `String` because it becomes a filename component during resolution,
and a type is what stops a later slice from joining an unvalidated reference to
a directory. The charset rule is FR-025; the reason it is a security-relevant
rule rather than a tidiness rule is research R-5.

### `CaptureDefaults`

| Field | Type | Guarantee |
| --- | --- | --- |
| `mode` | `Option<CaptureMode>` | One of the three section 17.2 modes when present. |
| `duration` | `Option<Duration>` | A parsed, non-zero `std::time::Duration` when present. |
| `roles` | `Option<Vec<String>>` | Non-empty when present, every entry declared by a stage. |
| `loopback` | `Option<bool>` | |
| `payload` | `Option<bool>` | |

Every field is an `Option` and none carries a substituted default. The
distinction matters to S14: a profile that chose `payload = true` and a profile
that said nothing are different inputs to the command line's override logic, and
collapsing them here would destroy information the operator supplied. That is
FR-004's "report absence rather than a substituted value" applied to the capture
table.

The duration is the typed span and not the literal text. Decision recorded in
the spec's clarifications: nothing in this slice or the three that follow
round-trips a profile back to TOML, and a literal that fails to parse never
produces a profile, so the text has no second consumer.

### `CaptureMode`

`File`, `Stream`, `Ring`. The three names section 17.2 gives `--mode`. A closed
enumeration rather than a string, so that S14 matches on a variant and a
misspelling is a parse fault here rather than an unhandled case there.

### `Stage`

| Field | Type | Guarantee |
| --- | --- | --- |
| `role` | `String` | Non-empty, unique within the profile, verbatim. |
| `lifecycle` | `Lifecycle` | One of three. |
| `terminal` | `bool` | False when absent. True implies `lifecycle == Session`. |
| `predicates` | `MatchPredicates` | At least one predicate present. |

`terminal` is the one place a default is applied, and it is not a substitution:
absent and `false` mean the same thing to every consumer, section 15.2 marks the
key optional, and there is no override logic that needs to tell them apart. This
is the difference between a default that discards information and one that does
not.

### `Lifecycle`

`Transient`, `Session`, `Service`, with section 10.4's meanings. The variant
governs two things beyond S05: whether an exit is significant, and whether
acquisition waits. This slice only enforces the pairing with `terminal`, and it
does so because section 10.4 defines a transient exit as normal and expected, so
a terminal transient would end a capture at the moment the launcher hands off.

### `MatchPredicates`

| Field | Type | Guarantee |
| --- | --- | --- |
| `exe` | `Option<ImagePattern>` | Well-formed and non-empty when present. |
| `path_contains` | `Option<String>` | Verbatim. |
| `path_regex` | `Option<PathRegex>` | Compiled successfully when present. |
| `cmdline_contains` | `Option<String>` | Verbatim. |
| `descends_from` | `Option<String>` | Names a role declared in the same profile. |

All five of section 10.3's predicates and no others. At least one must be
present, because an empty predicate set matches every process on the system.
Nothing here evaluates a predicate; S12 does.

`pinned()` reports whether the stage carries any predicate other than `exe`. It
is the ambiguity check's input and is a method rather than a derived field so
that adding a sixth predicate cannot leave a stale flag behind.

### `ImagePattern`

The author's `exe` text, verbatim. Matching and intersection fold case on copies
at comparison time, which is why the stored text can stay unaltered under
FR-009. Every string is a well-formed pattern in this syntax (research R-2), so
the only rejection is the empty pattern, which matches only the empty image
name.

### `PathRegex`

The author's pattern text and the compiled regular expression, together. Both,
because the compile is the validation and discarding the result would mean S12
compiles again: the same engine on the same input, so no divergence, but work
done twice for no reason. Equality and formatting are defined on the source
text, since a compiled automaton has no useful notion of either.

## Diagnostics

### `Diagnostic`

| Field | Type | Purpose |
| --- | --- | --- |
| `code` | `DiagnosticCode` | The stable surface. What tests and S14 key on. |
| `location` | `String` | A dotted key path, for example `stage[1].match.descends_from`. Human-readable, not a grammar. |
| `offset` | `Option<usize>` | Byte offset from the parser's span. |
| `position` | `Option<Position>` | Line and column derived from the offset. |
| `message` | `String` | For the operator. May be reworded without notice. |

### `Position`

Line and column, both one-based, derived from a byte offset in one place
(decision D-4). One-based because that is what an editor shows and an author is
reading their own file.

### `DiagnosticCode`

A closed enumeration. Every variant is exercised by a test (SC-003), which is
what stops a code from being added and never produced.

| Code | Fires when |
| --- | --- |
| `Syntax` | The document is not valid TOML. Includes a duplicate key, which the parser refuses. |
| `UnsupportedSchema` | `schema` is a supported-format integer other than 1. |
| `MissingField` | A required key is absent. |
| `WrongType` | A key's value has the wrong type. |
| `UnknownKey` | A key not in the accepted set for its table. |
| `FileTooLarge` | The candidate file exceeds the size limit. |
| `InvalidSlug` | `game.id` is empty or outside the slug charset. |
| `InvalidLifecycle` | `lifecycle` is not one of three. |
| `InvalidMode` | `capture.mode` is not one of three. |
| `InvalidDuration` | A duration literal does not parse. |
| `InvalidGlob` | An `exe` pattern is empty. |
| `InvalidRegex` | A `path_regex` does not compile, including exceeding the engine's size limit. |
| `EmptyMatch` | A `match` table carries no predicate. |
| `EmptyRoles` | `capture.roles` is present and empty. |
| `NoStages` | No `[[stage]]` table is declared. |
| `DuplicateRole` | Two stages declare the same role. |
| `MultipleTerminal` | More than one stage is terminal. |
| `TerminalLifecycle` | A terminal stage's lifecycle is not `session`. |
| `UnknownDescendsFrom` | `descends_from` names an undeclared role. |
| `DescendsFromCycle` | The `descends_from` relation contains a cycle. |
| `UndeclaredCaptureRole` | `capture.roles` names an undeclared role. |
| `AllServices` | Every stage is a service. |
| `AmbiguousImageMatch` | Two stages can match one image name and at least one is unpinned. |

Four of these are the additions beyond section 15.4's list: `TerminalLifecycle`,
`DescendsFromCycle`, `UndeclaredCaptureRole`, and `EmptyRoles`. Plan decision
D-10 records them as candidates for promotion rather than as readings of the
specification.

### `Diagnostics`

An ordered set, sorted by offset then code, non-empty whenever parsing failed
(FR-050). Carries the whole report; there is no way to obtain the first fault
without the rest, which is the shape that makes FR-013 hard to regress.

## Loading and resolution

### `LoadError`

`Read(io::Error)` and `Invalid(Diagnostics)`. Two outcomes because they need two
different responses from an operator: fix the path or its permissions, or fix
the profile.

The size refusal is inside `Invalid`, as a `FileTooLarge` diagnostic, rather
than a third variant. FR-046b says the limit is reported as a diagnostic, and a
file that is unusable because of its size and one that is unusable because of
its contents are the same answer to the operator: this file is not a profile,
here is why.

### `SearchPath`

| Field | Type | Corresponds to |
| --- | --- | --- |
| `command_line` | `Vec<PathBuf>` | Section 15.3 step 2. |
| `user` | `Option<PathBuf>` | Section 15.3 step 3. |

Modelled as the two steps rather than as one flat list, so that a successful
resolution can name which step supplied the profile (FR-039) without the caller
having to remember what index meant what. Both are supplied by the caller and
neither is discovered (FR-034).

### `BundledSet`

Already-parsed profiles, section 15.3 step 4. Constructed through a fallible
constructor that refuses two profiles sharing a `game.id` (FR-041), because step
four selects on that identifier and a duplicate makes the step ambiguous.
Holding parsed profiles rather than text means an invalid bundled profile cannot
exist in the set at all.

### `ProfileSource`

`ExplicitPath(PathBuf)`, `CommandLineDirectory(PathBuf)`,
`UserDirectory(PathBuf)`, `Bundled`. The four steps, one variant each, carrying
the path for the three that have one.

### `Resolved`

The profile and its `ProfileSource`. Returned together so that a caller cannot
report a capture without being able to say which file configured it, which is
the same argument S08 used for returning statistics and an end reason together.

### `ResolveError`

`InvalidReference { reference, reason }`, `NotFound { reference, searched:
Vec<PathBuf> }`, `Load { path, source: LoadError }`.

`NotFound` carries every location that was searched, because FR-040 requires it
and because the question an operator asks on this failure is always "where did
you look".

## What this slice deliberately does not model

- Any evaluation state: no bound process, no process tree node, no session. S12.
- Any capture parameter resolved against the command line. S14 owns the override
  logic and this slice hands it declared-or-absent values to work from.
- Size literals. Section 15.2 declares no size key.
- A serializer. Nothing writes a profile.
