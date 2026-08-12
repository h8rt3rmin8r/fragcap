# Phase 1 Data Model: Profile Format Migration

The typed in-memory model is unchanged; only the on-disk syntax and the parse
path change. This document records the JSON shape a profile now takes and the
mapping from the former TOML.

## The JSON profile (the `profile` variant of the master schema)

```json
{
  "schema": 1,
  "kind": "profile",
  "fidelity": "verified",
  "game": { "id": "eso", "name": "The Elder Scrolls Online", "platform": "steam", "app_id": "306130" },
  "capture": { "mode": "file", "duration": "30m", "roles": ["launcher", "client"], "loopback": true, "payload": true },
  "stage": [
    { "role": "launcher", "lifecycle": "transient", "match": { "exe": "*Launcher.exe", "path_contains": "Elder Scrolls Online" } },
    { "role": "client", "lifecycle": "session", "terminal": true, "match": { "exe": "eso64.exe" } }
  ]
}
```

## Mapping from TOML

| TOML | JSON |
| --- | --- |
| `schema = 1` | `"schema": 1` |
| (none) | `"kind": "profile"` (new, required by the master schema) |
| (none) | `"fidelity": "verified"` or `"heuristic-unverified"` (new, required) |
| `[game]` table | `"game"` object |
| `[capture]` table | `"capture"` object |
| `[[stage]]` array of tables | `"stage"` array of objects |
| `match = { exe = "..." }` inline table | `"match": { "exe": "..." }` object |
| TOML header comment (scaffold warning) | `"notes"` string (structured) |

## Typed model (unchanged)

`Profile`, `Game`, `GameId`, `CaptureDefaults`, `CaptureMode`, `Stage`,
`Lifecycle`, `MatchPredicates`, `PathRegex` in `schema.rs` are unchanged.
`SCHEMA_VERSION` stays 1. The `Draft` / `DraftGame` / `DraftStage` in `parse.rs`
are retained as the lenient intermediate, but are populated from a
`serde_json::Value` instead of a `toml_span` table, and their byte-span fields
become unnecessary (locations are JSON pointers).

## Diagnostics

`Diagnostic`, `DiagnosticCode`, `Diagnostics` are retained. `location` holds a
JSON pointer; `offset` and `position` are `None` on the profile-load path.
`DiagnosticCode::Syntax` now denotes invalid JSON. The full validation coverage
is unchanged: structural codes (`MissingField`, `WrongType`, `UnknownKey`,
`UnsupportedSchema`, `InvalidSlug`, `InvalidLifecycle`, `InvalidMode`,
`EmptyMatch`, `NoStages`, `TooManyStages`) plus semantic codes (`DuplicateRole`,
`MultipleTerminal`, `TerminalLifecycle`, `UnknownDescendsFrom`,
`DescendsFromCycle`, `UndeclaredCaptureRole`, `AllServices`,
`AmbiguousImageMatch`) plus compilation codes (`InvalidGlob`, `InvalidRegex`,
`InvalidDuration`) and `FileTooLarge`, `EmptyRoles`.

## Resolution

`<ref>.json` replaces `<ref>.toml` in the resolution order (explicit path,
command-line profile directory, user directory, bundled by game id).
