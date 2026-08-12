# Research: CLI readiness, help, and output-contract polish

Phase 0 findings. Each item resolves an implementation unknown by reading the
current code; nothing here is speculative.

## R1. The #68 exit-code split is one line, and the issue's framing is imprecise

**Decision**: Reclassify `ResolveError::InvalidReference` from `CliError::Usage`
(exit 2) to `CliError::Failure` (exit 1) in `crates/fragcap-cli/src/exit.rs`
(the `From<ResolveError>` impl, ~line 138). Keep `Load { LoadError::Invalid(_) }`
at exit 2. No change to `commands/profile.rs` is required.

**Rationale (from `crates/fragcap-profile/src/resolve.rs`)**: The resolver has
three failure variants:
- `InvalidReference` (resolve.rs:266) - the reference is neither an existing
  file nor a valid game-id slug (`GameId::is_valid`: lowercase, digits, hyphen,
  underscore). A path-shaped reference such as `missing.toml` fails the slug
  test and lands here.
- `NotFound` (resolve.rs:305) - a valid slug that matched no candidate.
- `Load { Invalid }` (resolve.rs:250, 340) - a candidate file was found but
  failed to read/parse/validate (a genuinely invalid *profile*).

Both `profile show` and `profile validate` call the same `resolve(...)` with the
same arguments, so for identical input they cannot disagree. The apparent split
in the bug report comes from *different* inputs: `show missing` (a bare slug →
`NotFound` → 1) versus `validate missing.toml` (a path that is not a file and
not a slug → `InvalidReference` → 2). Under the current mapping, an unresolvable
*path-shaped* reference is exit 2 while an unresolvable *slug* reference is exit
1 - that is the real inconsistency.

Per the Session 2026-08-12 clarification (resolves-to-nothing → 1),
`InvalidReference` and `NotFound` both describe "this reference names no
resolvable profile" and MUST both exit 1. Only `Load { Invalid }` - a profile
file that exists but is invalid - is a configuration error (exit 2). Reclassing
`InvalidReference → Failure` achieves this for both commands at once and leaves
the invalid-profile-content path untouched.

**Spec impact**: FR-011 and the exit-codes contract are corrected to say
"any reference that resolves to no profile → exit 1 (both commands); a profile
file that exists but fails validation → exit 2." The earlier "malformed
reference stays 2" phrasing is dropped - the code has no such separate class.

**Alternatives rejected**: intercepting per-command in `profile.rs` (more code,
two sites, and `show`'s existing `NotFound` arm already only adds a
"searched locations" printout - it is redundant for exit purposes).

## R2. npcap version: read the wpcap.dll FileVersion, no new crate, no new feature

**Decision**: Replace the hardcoded `NpcapInfo.version = "installed"`
(`crates/fragcap-cli/src/doctor/probe.rs:127`) with the FileVersion resource of
the already-located `wpcap.dll`, read via `GetFileVersionInfoSizeW` /
`GetFileVersionInfoW` / `VerQueryValueW`. Fall back to `"installed"` when the
version cannot be read, and reword the fallback rendering so it does not print
"version installed".

**Rationale**:
- The probe already computes the `wpcap.dll` path (`probe.rs:113-118`), so the
  source is in hand.
- These version-info APIs are gated behind the `Win32_Storage_FileSystem`
  windows-sys feature, which is already effectively enabled for the single
  windows-sys 0.36.1 instance the `cfg(windows)` crates resolve to (it is in the
  workspace `windows-sys` feature list). To avoid relying on cross-crate feature
  unification implicitly, add `Win32_Storage_FileSystem` explicitly to
  `crates/fragcap-cli/Cargo.toml`'s windows-sys features. This is a feature
  addition on an existing dependency, not a new crate - dependency inventory
  unchanged.
- No process handle, no elevation: a version-resource read on a DLL path is
  read-only (P-1 clean, consistent with `probe.rs:157-159` and the existing
  `is_elevated` current-process-token read). The unsafe FFI block sits alongside
  the existing `is_elevated` unsafe block in the same file.

**Alternatives considered**:
- Registry `HKLM\SOFTWARE\WOW6432Node\Npcap` via the copyable `read_reg_sz`
  pattern in `crates/fragcap-steam/src/lib.rs:170-235` (uses `Win32_System_Registry`
  + `KEY_WOW64_32KEY`). Rejected as primary because the version value name under
  that key is unconfirmed (npcap stores the install dir there; a version REG_SZ
  is not guaranteed), so the FileVersion source is more robust. The registry
  pattern remains a fallback option if FileVersion proves unavailable in
  practice.
- Version pinned: windows-sys stays 0.36.1 (`Cargo.lock`); no bump.

## R3. New doctor checks need no exit-logic changes

**Decision**: Add the live/socket-table capability checks as ordinary `Check`
instances; rely on the existing `Report::exit()`/`ready()` logic.

**Rationale (from `crates/fragcap-cli/src/doctor/mod.rs`)**:
- `render_json` (mod.rs:250-269) emits one NDJSON object per check
  (`section`/`name`/`detail`/`status`, plus `remediation` when present); it is a
  per-check shape, distinct from the section 17.5 event stream - so the profile
  `--json` work (R-none; see contracts/profile-json.md) reuses the `events.rs`
  emitter, not this.
- `Report::exit()` (mod.rs:196-211) returns FAILURE iff any check is `Fail`;
  `ready()` derives from that. A new `Check::fail(...)` for an absent live
  backend therefore blocks with no special-casing, and the downgraded loopback
  `Warn` stops blocking automatically.
- `checks::npcap` already renders `format!("version {}", info.version)`
  (checks.rs:68-71), so a real version string from R2 surfaces in both human and
  JSON output with zero change to the check or exit code.
- Confirmed: `doctor --json` currently emits **no** terminal readiness/summary
  record. Adding one is an adjacent nicety (see #65 "related"), not required by
  this slice; left out unless trivial.

## R4. Profile `--json` reuses the existing section 17.5 emitter

**Decision**: Thread the existing `Emitter` (`emit.rs`) / `Event` (`events.rs`)
NDJSON machinery into the `profile` dispatch (currently `commands/profile.rs`
receives no `json` flag; `lib.rs` drops it for that path). Add `diagnostic` and
`summary` event variants and profile-list count events.

**Rationale**: `run`/`tap`/`steam`/`extcap` already route through the `Emitter`;
`profile` is the outlier. Reusing the same emitter keeps one structured-output
contract (the clarified §17.5 choice) and one escaping path
(`fragcap::write_json_string`). No new dependency; `serde_json` stays dev-only
for test-side parsing.

## Dependency summary

No new runtime crate. One additive `windows-sys` feature
(`Win32_Storage_FileSystem`) on an already-present, already-pinned (0.36.1)
dependency. `serde_json` remains dev-only. The dependency inventory in
`AGENTS.md` is unchanged in crate count.
