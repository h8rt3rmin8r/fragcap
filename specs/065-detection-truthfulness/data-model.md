# Phase 1 Data Model: Detection truthfulness and the column split

**Slice**: S065 | **Date**: 2026-08-20 |
**Spec**: [spec.md](./spec.md) | **Research**: [research.md](./research.md)

Three things change shape: the signature vocabulary gains a matchable
binary-marker form, the scan outcome gains a truncation count, and the target
entry gains a coverage field that is plumbed from every producing source.

## Signature vocabulary (`fragcap-profile::signature`)

### `SignatureKind::BinaryMarker` becomes matchable in one form

The kind is unchanged. What changes is that a pattern of the form
`section:<glob>` now compiles and matches, while every other pattern of this
kind stays inert.

| Pattern | Disposition | Why |
| --- | --- | --- |
| `section:.bind` | applied | recognized form, compiles to an anchored, case-insensitive glob over a PE section name |
| `denuvo-anti-tamper-marker` | inert | a byte-sequence form this build does not implement |
| `arxan-guard-marker` | inert | as above |
| `vmprotect-section-marker` | inert | as above |
| `section:` | skipped | recognized form with an empty pattern, a malformed row |

Matchability moves from the kind to the row, because that is where it now
lives. `SignatureKind::is_implemented(&self)` is replaced by
`Signature::is_matchable(&self)`, which reads both the kind and the pattern.
This is a breaking change to a pre-1.0 public API and is recorded in the
changelog.

The accounting invariant is unchanged and still asserted:
`applied_count() + inert_count() + skipped_count() == total_count()`.

### `ScanOutcome` gains a truncation count

| Field | Type | Meaning |
| --- | --- | --- |
| `findings` | `Vec<DetectionFinding>` | unchanged |
| `unreadable` | `Vec<PathBuf>` | unchanged; a candidate executable that could not be read is now recorded here too |
| `marker_candidates_skipped` | `usize` | **new**: candidates dropped by the section-scan cap |

New derived accessor:

```text
ScanOutcome::is_complete() -> bool
    unreadable.is_empty() && marker_candidates_skipped == 0
```

### Scan bounds (new public constants)

| Constant | Value | Role |
| --- | --- | --- |
| `MARKER_SCAN_MAX_DEPTH` | 4 | defines the candidate set; not a truncation, so exclusions are not counted |
| `MARKER_SCAN_MAX_CANDIDATES` | 64 | truncates the candidate set, so exclusions are counted |

## PE reader (`fragcap-profile::pe`)

New function alongside `version_strings`:

```text
pe::section_names(bytes: &[u8]) -> Vec<String>
```

Returns the section-table names of a PE image, or an empty vector when the
bytes are not a PE image, when the section table lies outside the supplied
bytes, or when the header is malformed. Never errors, never guesses.

The caller reads a bounded prefix of the file (64 KiB) rather than the whole
file, so a large executable costs one small read. `version_strings` is
unchanged.

Test support gains `minimal_pe_with_sections(&[&str]) -> Vec<u8>`, beside the
existing `minimal_pe_with_version_string`, so the two fixture executables the
spec calls for are generated rather than hand-made
(`fixtures/README.md`'s rule).

## Engine declaration (`fragcap-profile::engine_rule`)

`Engine` gains two members, both additive:

| Member | Type | Purpose |
| --- | --- | --- |
| `Engine::ALL` | `[Engine; 4]` | the structural iteration source for the subset check |
| `Engine::product_name()` | `&'static str` | the exact product string the signature set uses |

`product_name()` differs from the existing `as_str()`, which returns a
lower-case diagnostic label. The mapping is `Unreal -> "Unreal"`,
`Unity -> "Unity"`, `Godot -> "Godot"`, `RenPy -> "Ren'Py"`.

## Coverage state (`fragcap-targets`)

### `DetectionScan`

A new enum in `fragcap-targets::entry`, mirroring the shape of
`TargetClassification` and `ClassificationSource`: a stored string, a `parse`
that rejects an out-of-set value, and no permissive fallback.

| Variant | Stored | Meaning |
| --- | --- | --- |
| `Complete` | `complete` | the directory was scanned and the scan covered everything it set out to |
| `Incomplete` | `incomplete` | the directory was scanned and coverage was reduced (something unreadable, or the candidate cap truncated the scan) |

Absence is modeled by `Option<DetectionScan>` being `None`, stored as SQL
`NULL`. There is deliberately no `NotScanned` variant: a variant would allow a
row to assert "no scan happened", which is a claim, where `NULL` is the absence
of a claim.

### `TargetEntry`

| Field | Type | Change |
| --- | --- | --- |
| `detection_scan` | `Option<DetectionScan>` | **new**, `None` for every existing row |

### Store schema

`SCHEMA_VERSION` moves from 6 to 7.

```sql
-- in DDL (fresh stores) and in MIGRATE_6_TO_7 (existing stores)
detection_scan TEXT CHECK (detection_scan IS NULL
                           OR detection_scan IN ('complete', 'incomplete'))
```

`MIGRATE_6_TO_7` is a single additive `ALTER TABLE targets ADD COLUMN`, applied
in its own transaction stamping version 7, following the five migrations already
in the ladder. Existing rows read the column as `NULL`, which is exactly the
"never scanned" state, so no backfill is needed and no row changes meaning.

### Export contract

One new optional key on the target-entry export object, emitted only when
present, and rejected at import when out of set:

```json
{ "handle": "...", "detection_scan": "complete" }
```

### Producing sources

The value is carried from the scan to the stored row through the existing
candidate types. Every source that can produce a target is plumbed; a source
left unplumbed is the defect FR-015 names.

| Producer | Carrier | Value |
| --- | --- | --- |
| `SteamSource` (facade `discovery.rs`) | `CandidateTarget::detection_scan` | from the scan outcome, or `None` when the root could not be read at all |
| `KnownRootsSource` via `SignatureClassifier` | `ClassifierVerdict::Hit { detection_scan, .. }` | from the scan outcome |
| `KnownRootChildIsGame` (no signatures) | `ClassifierVerdict::Hit { detection_scan: None, .. }` | no scan ran |
| `DirectorySource::with_signatures` | `CandidateTarget::detection_scan` | from the scan outcome |
| `DirectorySource::new` (no signatures) | `CandidateTarget::detection_scan` | `None` |
| `targets add` (`scan_exe_evidence`) | written directly onto the entry | from the scan outcome, `None` when no catalog resolved |

An install root that could not be read at all is a scan that produced nothing
and covered nothing. It records `Incomplete`, not `None`: a scan was attempted
and failed, which is a different fact from no scan.

## Presentation (`fragcap-targets::readiness`)

`known_summary` is replaced by two functions. It is removed rather than kept as
a deprecated alias, because leaving both would let the flattened form survive in
some caller and reintroduce the conflation this slice removes.

| Function | Reads | Falls back to |
| --- | --- | --- |
| `engine_summary(&TargetEntry) -> String` | findings with category `engine` | the coverage marker |
| `sensitivities_summary(&TargetEntry) -> String` | findings with category `anti-cheat` or `drm`, in that declared order | the coverage marker |

Coverage markers, used by both when the column has no products:

| State | Marker | Width |
| --- | --- | --- |
| `Some(Complete)` | `-` | 1 |
| `Some(Incomplete)` | `incomplete` | 10 |
| `None` | `not scanned` | 11 |

`capture_readiness` and `CaptureReadiness::label` are unchanged: `ready` and
`needs a target`. The two fallback sentences `no online mode recorded` and
`no launch data known` are removed from the codebase entirely.
