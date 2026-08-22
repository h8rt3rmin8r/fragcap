# Feature Specification: Anti-cheat detection and machine-scope presence

**Feature Branch**: `068-anticheat-machine-scope`

**Created**: 2026-08-22

**Status**: Draft

**Input**: User description: "S068: anti-cheat and machine scope (issue #170).
Not one row in `fragcap targets` reports an anti-cheat product, including two
titles that demonstrably ship Easy Anti-Cheat. Two independent causes: (1)
the in-tree signature rows don't match the actual bootstrapper artifacts, and
(2) modern EAC installs machine-wide (a service plus driver under Program
Files, outside any game's install tree), which is structurally invisible to a
directory-scoped detector no matter how many signature rows are added. Fix
both, add a machine-wide scope distinct from title scope, and extract the
anti-cheat signals already sitting unused in the appinfo.vdf launch entries
fragcap already parses. See the issue body for the full measured evidence:
https://github.com/h8rt3rmin8r/fragcap/issues/170"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - See anti-cheat a title actually ships (Priority: P1)

An operator runs `fragcap targets` against a machine with EAC-protected
titles installed (measured cases: Arc Raiders, Tom Clancy's The Division 2).
Today neither reports anti-cheat at all, even though both ship a visible EAC
bootstrapper in their install tree. The operator needs to see this before
deciding whether and how to capture the title.

**Why this priority**: This is the entire measured defect in issue #170.
Every other part of the issue exists to make this reporting complete and
trustworthy, not to replace it.

**Independent Test**: Run detection against a synthetic install tree shaped
like the measured Division 2 and Arc Raiders layouts (an `EasyAntiCheat/`
directory, an `EACLaunch.exe`, an `AntiCheatInstaller.exe`). Verify the
title's SENSITIVITIES column reports "Easy Anti-Cheat".

**Acceptance Scenarios**:

1. **Given** an install tree containing `EasyAntiCheat/EasyAntiCheat_EOS_Setup.exe`
   and `EACLaunch.exe`, **When** the directory is scanned, **Then** the scan
   reports a finding for Easy Anti-Cheat.
2. **Given** an install tree containing `Installers/AntiCheatInstaller.exe`,
   **When** the directory is scanned, **Then** the scan reports a finding for
   Easy Anti-Cheat.
3. **Given** an install tree containing `EOSSDK-Win64-Shipping.dll` and
   nothing else anti-cheat-shaped, **When** the directory is scanned, **Then**
   the scan reports no anti-cheat finding at all: the Epic Online Services SDK
   is not, by itself, evidence of anti-cheat (issue #170's explicit
   false-positive warning).

---

### User Story 2 - Corroborate from Steam's own launch metadata (Priority: P2)

Steam's `appinfo.vdf` cache, which fragcap already parses for launch entries,
carries anti-cheat signals in the `arguments` and `description` fields of a
protected launch entry, independent of anything on disk. An operator whose
directory scan is inconclusive (a title that has not yet downloaded its EAC
bootstrapper, or whose files this scan cannot reach) still gets the correct
answer from data fragcap is already reading and discarding.

**Why this priority**: Depends on User Story 1's finding vocabulary and
rendering existing first, but is independently valuable: it is a second,
zero-new-I/O source that corroborates or, in some cases, is the only source
available.

**Independent Test**: Feed a synthetic set of Steam launch entries (matching
the issue's measured `appinfo.vdf` strings) through the classifier in
isolation. Verify a `-anticheat_settings=`-bearing entry, a
`start_protected_game.exe` entry, and an `"eac-release"`-described entry each
produce an Easy Anti-Cheat finding, and that an explicitly EAC-disabled
launch variant (`-no-eac`, description mentioning "Anti-Cheat Disabled") does
not.

**Acceptance Scenarios**:

1. **Given** a launch entry with `arguments` containing
   `-anticheat_settings=SettingsProfile.json`, **When** the entry is
   classified, **Then** it yields an Easy Anti-Cheat finding whose evidence
   quotes the matched argument.
2. **Given** a launch entry with `executable` equal to
   `start_protected_game.exe`, **When** the entry is classified, **Then** it
   yields an Easy Anti-Cheat finding.
3. **Given** a launch entry whose `arguments` contains `-no-eac` and whose
   `description` reads "Halo: MCC Anti-Cheat Disabled (Mods and Limited
   Services)" (the issue's own measured counter-example), **When** the entry
   is classified, **Then** it yields no anti-cheat finding: a description
   that merely contains the words "anti-cheat" is not itself evidence, and a
   `-no-eac` flag is explicit evidence against.
4. **Given** both a directory-scan finding and an appinfo-classifier finding
   for the same product on the same title, **When** the two are combined,
   **Then** the title reports the product once, at whichever finding's
   fidelity is stronger.

---

### User Story 3 - Know when anti-cheat is present on the machine itself (Priority: P3)

Modern EAC (and, per the issue's research, BattlEye and Vanguard by the same
deployment model) installs once per machine as a service and driver outside
every game's tree. An operator needs to be able to tell "this machine has an
anti-cheat runtime installed" apart from "this specific title ships one":
conflating the two would misattribute a machine-wide fact to a title that may
have nothing to do with it, which is exactly the false-positive class issue
#170 warns against for `EOSSDK-Win64-Shipping.dll`.

**Why this priority**: Lowest priority because User Story 1 alone already
satisfies the issue's two headline acceptance titles (both ship an in-tree
bootstrapper). This story is the structural fix for titles that ship no
on-disk trace at all, which the issue frames as the harder, second cause.

**Independent Test**: Inject a fake machine probe that reports "Easy
Anti-Cheat (Epic Online Services) service present" and run the hero listing.
Verify the fact appears once, under a machine-scope heading, and no target
row's SENSITIVITIES cell changes because of it.

**Acceptance Scenarios**:

1. **Given** a machine probe that finds the `EasyAntiCheat_EOS` service
   registered, **When** `fragcap targets` runs, **Then** the output includes
   a machine-scope line naming Easy Anti-Cheat, separate from every target
   row.
2. **Given** a machine probe that finds nothing, **When** `fragcap targets`
   runs, **Then** no machine-scope section appears at all.
3. **Given** the machine-scope finding from scenario 1, **When** the operator
   reads a specific target's row, **Then** nothing in that row changes as a
   result of the machine-scope finding alone: a title's row reports only what
   its own install-tree scan or appinfo evidence supports.

---

### Edge Cases

- A title ships an EAC bootstrapper artifact that a signature row now
  matches, **and** the machine-wide probe also finds the EAC service: the
  title's row and the machine-scope line both report Easy Anti-Cheat,
  independently, with no cross-reference between them (User Story 3,
  Scenario 3 generalized).
- The appinfo cache is entirely absent (a fresh Steam install with nothing
  cached yet): the classifier runs over zero launch entries and produces no
  findings, exactly as today's "no cache is not an error" contract already
  behaves for the rest of appinfo reading.
- A launch entry's `arguments` or `description` is `None` (never observed by
  the appinfo parser but structurally possible): the classifier treats an
  absent field as no match for that field, never a panic.
- The machine probe runs on a non-Windows host, or the registry key cannot be
  read for a permission reason: the probe reports no findings rather than a
  false claim; a probe that could not run is not evidence of absence, and the
  rendering must not print "no anti-cheat products found" as if the check had
  actually completed.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The detection signature set MUST match the Easy Anti-Cheat
  bootstrapper artifacts measured in issue #170: an `EasyAntiCheat/`
  directory, an `EasyAntiCheat_EOS/` directory, and the executables
  `EasyAntiCheat*.exe`, `EACLaunch.exe`, `AntiCheatInstaller.exe`, and
  `start_protected_game.exe`.
- **FR-002**: `EOSSDK-Win64-Shipping.dll` MUST NOT be treated as anti-cheat
  evidence by any signature row, now or by future addition to
  `signatures.json`, unless the row also requires other co-located evidence.
- **FR-003**: fragcap MUST classify each Steam title's already-parsed launch
  entries (`arguments`, `description`, `executable`) for anti-cheat signals,
  independent of any directory scan.
- **FR-004**: The launch-entry classifier MUST only match specific,
  unambiguous tokens (an enabling command-line flag, the canonical launcher
  executable name, an exact description string), never a broad substring
  match on words like "anti-cheat" that also appear in a launch entry that
  explicitly disables anti-cheat.
- **FR-005**: When both a directory-scan finding and a launch-entry-classifier
  finding name the same product for the same title, the title MUST report
  that product exactly once, at the stronger of the two findings' fidelity.
- **FR-006**: fragcap MUST provide a machine-wide anti-cheat presence check,
  distinct in kind from a per-title install-directory scan.
- **FR-007**: A machine-wide finding MUST be rendered separately from every
  target row, so a reader can distinguish "this title ships product X" from
  "this machine has product X installed", and MUST NOT be merged into, or
  used to infer, any specific target's evidence.
- **FR-008**: When the machine-wide probe finds nothing, or cannot run at
  all (non-Windows, a permission failure), fragcap MUST NOT render a
  machine-scope section that asserts a negative ("no anti-cheat found"); it
  MUST simply omit the section, so an unreported absence is never rendered
  as a confirmed clean result.
- **FR-009**: The machine-wide probe MUST be implemented behind a seam
  (a trait or equivalent) that a test can substitute with a synthetic
  result, so the machine-scope path is testable on a runner with no
  anti-cheat software installed.
- **FR-010**: Every anti-cheat finding, from any source (directory scan,
  launch-entry classifier, machine-wide probe), MUST carry evidence that
  names what was actually matched (the file path, the argument string, or
  the registry key), so a claim always traces to an inspectable fact.

### Key Entities

- **Detection finding** (existing type, reused): a category, product,
  evidence string, and fidelity. This slice adds new producers of this type;
  it does not change its shape.
- **Launch-entry classification**: a pure function over a title's parsed
  Steam launch entries, producing zero or more detection findings, run once
  per title alongside (not instead of) the existing directory scan.
- **Machine-wide anti-cheat finding**: a product name and evidence string,
  scoped to the whole machine rather than to any title, produced by a
  probe seam with exactly one real (Windows) implementation and any number
  of test implementations.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Both of issue #170's measured titles (an install tree shaped
  like Arc Raiders, one shaped like The Division 2) report Easy Anti-Cheat
  after this slice, where neither did before.
- **SC-002**: No install tree containing only `EOSSDK-Win64-Shipping.dll`
  (and no other anti-cheat-shaped artifact) ever reports anti-cheat, verified
  by a standing regression test.
- **SC-003**: An operator reading `fragcap targets` output can always tell,
  without cross-referencing any other command, whether a reported anti-cheat
  product is attributed to a specific title or to the machine as a whole.
- **SC-004**: The launch-entry classifier correctly abstains on the issue's
  own measured negative example (the Halo: MCC EAC-disabled launch variant),
  verified by a standing regression test.

## Assumptions

- Only Easy Anti-Cheat is implemented for the machine-wide probe and the
  launch-entry classifier in this slice. The issue's own text marks BattlEye
  and Vanguard's machine-wide deployment as "not installed on this machine...
  should be verified before being relied on"; adding an unverified probe
  entry would be exactly the kind of unmeasured claim this project's
  practice avoids. The probe's seam is designed to accept more products
  without a structural change, so a follow-up issue can add BattlEye and
  Vanguard once someone measures them the same way this issue measured EAC.
- Source D from the issue (Steam Deck compatibility-test localization tokens,
  `#SteamDeckVerified_TestResult_UnsupportedAntiCheat*`) is out of scope. The
  issue frames it as "worth investigating," not as part of the acceptance
  criteria, and it is product-agnostic (it says "has anti-cheat," never
  which one), so it would not by itself resolve the issue's headline defect.
- The machine-wide probe is wired into the `fragcap targets` hero listing
  (bare `fragcap` and `fragcap targets`), the surface the issue's acceptance
  criteria are phrased against. Wiring it into `targets discover` or other
  surfaces is not required by the acceptance criteria and is left for a
  follow-up if wanted.
- No local-store or catalog-store schema change is needed. The new signature
  rows use existing, already-valid `kind` values (`filename`,
  `directory-shape`); the launch-entry classifier's findings are stored in
  the same per-target `evidence` JSON column directory-scan findings already
  use; the machine-wide finding is computed fresh each run and never
  persisted.
- "Machine-wide" is checked via a registry key read
  (`HKLM\SYSTEM\CurrentControlSet\Services\<name>`), the same mechanism and
  access-rights class already used elsewhere in this codebase (the Steam
  install-path lookup), not a service-control-manager handle, matching the
  issue's own stated preference ("Registry route preferred over SCM").
