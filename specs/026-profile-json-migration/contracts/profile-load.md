# Contract: profile loading (JSON)

## `Profile::parse(text: &str) -> Result<Profile, Diagnostics>`

The only constructor for a `Profile`. Unchanged signature; the `text` is now JSON.

- **Input**: JSON text intended to be a profile.
- **Behavior**:
  1. Parse to a `serde_json::Value`. A JSON syntax error yields exactly one
     `Diagnostic` (code `Syntax`, message from serde_json, located at the
     document root) and stops, because a document that did not parse has no
     structure to check.
  2. Run structural validation via `jsonschema::validate_json`; map every
     `SchemaDiagnostic` into a `Diagnostic`. An unsupported `schema` version
     suppresses the rest (as under TOML), because later faults are likely
     consequences of reading a newer schema under this build's rules.
  3. Run the lenient fragcap pass: extract an all-optional `Draft`, compile the
     `exe` glob, the `path_regex`, and the `capture.duration`, and run the
     semantic checks in `validate.rs`. Accumulate.
  4. If any diagnostic was produced, return `Err(Diagnostics)` with every problem,
     sorted deterministically. Otherwise return the validated `Profile`.
- **Guarantees**:
  - Every problem across both layers in one pass (never only the first).
  - No unvalidated `Profile` can be constructed.
  - The returned `Profile` is byte-for-byte equivalent in behavior to the one the
    equivalent TOML produced (same game, capture defaults, stages).
- **Locations**: JSON pointers (RFC 6901). Byte positions are not populated.

### Acceptance

| Given | Then |
| --- | --- |
| a valid JSON profile | `Ok(Profile)` with the expected game, defaults, and stages |
| N mixed structural and semantic faults | `Err` carrying exactly N diagnostics, one pass |
| a missing `kind` or `fidelity` | a `MissingField` diagnostic, reported with any others |
| a cyclic `descends_from` (structurally valid) | the semantic cycle is reported |
| a non-string `exe` | reported structurally; glob compilation is skipped (no double-report) |
| a `path_regex` that does not compile | reported as a fragcap-specific fault, not deferred |
| not valid JSON | one `Syntax` diagnostic, nothing else |
| the former TOML content | refused as invalid JSON (not half-parsed) |

## `load(path: &Path) -> Result<Profile, LoadError>`

Unchanged behavior: refuse a non-file, refuse a file over `MAX_PROFILE_BYTES`
before reading, otherwise read and defer to `Profile::parse`. Resolution finds
`<ref>.json`.

## Scaffold output contract (`fragcap-steam` render)

The generated profile is JSON validating against the master schema's profile
variant, carrying:

- `"schema": 1`, `"kind": "profile"`, `"fidelity": "heuristic-unverified"`.
- a `"notes"` string containing the warning that the stage classification is
  heuristic and must be verified against a live capture session.
- `game` and the `stage` array as classified.

### Acceptance

| Given | Then |
| --- | --- |
| an installed title | `render` output parses and loads via `Profile::parse` |
| the scaffold output | carries `fidelity: heuristic-unverified` and the `notes` warning as data |
