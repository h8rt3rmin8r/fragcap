# Research: Target Discovery Integrity

## D1. Separate discovery production from automatic-registration admission

**Decision**: Add one pure decision over `CandidateTarget` with an exhaustive accepted/refused reason, plus one batch operation that proves `produced = eligible + refused` before passing only eligible candidates through the existing registration function.

**Rationale**: Explicit discovery and automatic registration have different confidence thresholds. Reusing `CandidateTarget` preserves P-10's source seam, while a separate admission decision prevents the generic source from hiding low-confidence candidates merely because the hero command may not persist them. One combined operation keeps the hero and Doctor paths from drifting.

**Alternatives considered**: Changing every source to omit low-confidence candidates was rejected because explicit discovery would lose them. Encoding a boolean on `CandidateTarget` was rejected because it is not self-explanatory and forces pointed, interactive, and future sources to claim a default policy that may not apply. Filtering only in the CLI was rejected because Doctor already consumes the same behavior and a second policy would recur.

## D2. Define the two positive admission grants exactly

**Decision**: A `SteamAppId` identity grants authoritative-platform admission. A path identity grants local-evidence admission only when its findings contain an engine-category result at verified-or-stronger fidelity. Anti-cheat, DRM, classification, source name, known-root membership, and path shape do not grant admission.

**Rationale**: Steam manifests identify installed applications even when fragcap does not recognize the engine. For an unanchored path, the engine category is the existing title-class signal, and its fidelity carries the classifier's evidence strength. The other findings can occur in launchers, browsers, and infrastructure and therefore cannot prove one game title.

**Alternatives considered**: Requiring a known engine for Steam was rejected as systematic false negatives for proprietary engines. Accepting every `classification == Game` was rejected because the current structural prior itself assigns that value. Accepting any verified finding was rejected because security or DRM evidence does not establish a title.

## D3. Prune the exact platform-client subtree before classification

**Decision**: Extend `KnownRootsSource` with excluded subtrees. Compare separator-normalized, case-folded path components, treating equality or a component-boundary descendant as excluded. Check both each configured known root and every child before listing, classification, or descent. The facade injects the exact discovered or explicitly supplied Steam root.

**Rationale**: A content classifier cannot reliably distinguish Steam UI assets from a title, and the issue demonstrates a false Unreal marker. The platform root is stronger structural evidence and is already known to the facade. Pruning before classification prevents both false hits and unnecessary traversal. Component boundaries prevent `C:/Games/Steam` from excluding `C:/Games/SteamVRGame`.

**Alternatives considered**: A folder-name denylist was rejected because names are mutable, incomplete, and violate the issue's ownership rule. Filesystem canonicalization was rejected because it adds I/O, link resolution, existence requirements, and failure cases unrelated to the already resolved root. Excluding only immediate infrastructure names was rejected because future Steam layout changes would reopen the defect.

## D4. Keep broad discovery explicit and add a privacy-safe summary

**Decision**: `targets discover` remains the read-only broad inspection surface. Its detailed table gains an automatic-registration decision and reason for each candidate, followed by a separate admission account. `--summary` emits aggregate source, admission, coverage, and warning counts only and suppresses store paths, candidate identities, application ids, title names, evidence strings, and warning paths.

**Rationale**: The existing explicit command already has the correct non-persistent authority. Annotating it explains why a candidate will not appear after a hero run. A count-only mode is the smallest way to satisfy real-machine validation without leaking the private installed-title inventory that motivated the issue report.

**Alternatives considered**: A new `discover broad` command was rejected as duplicate surface. Recording a scrubbed copy of detailed output was rejected because path and name redaction is harder to audit than never rendering those fields. Omitting real-machine validation was rejected because the defect depends on actual nested platform layouts.

## D5. Reconciliation removes only rows with exact tool ownership and structural proof

**Decision**: A pure reconciliation planner accepts current target rows and an injected platform inventory containing exact client roots and authoritative app installs. A row is removable only when all ownership facts agree: classification source `platform`, provenance exactly names `known-roots`, no anchor, and a present install root. It then also needs one structural reason: it equals/is infrastructure beneath an exact platform client root, duplicates an exact anchored authoritative install already stored, or contains findings for multiple distinct engine products. Every other row is preserved with a stable reason.

**Rationale**: Provenance identifies who created the row; platform inventory and existing anchored rows identify what the path means. Multi-engine evidence is the already approved S077 aggregate signal. User-authored, missing-provenance, conflicting, anchored, and location-only rows remain ambiguous and are never deleted.

**Alternatives considered**: Removing every old no-engine known-roots row was rejected because some may be valid unsupported-engine games. Inferring ownership from a path or handle was rejected explicitly by issue #375. A schema migration was rejected because it would mutate local stores without preview or consent.

## D6. Bind confirmation to an immutable preview and apply atomically

**Decision**: `targets reconcile` without `--yes` is preview-only and prints the exact stable id, handle, and reason for each removable row plus preserved-reason counts. `--yes` repeats planning against the current store, then calls one store transaction that reloads and compares every previewed `TargetEntry` before deleting any. Any missing or changed row aborts the transaction. An empty plan is a successful no-op.

**Rationale**: The command itself is the preview boundary; rerunning with `--yes` is explicit, scriptable confirmation. Equality over the complete stored value is a natural fingerprint already available from `TargetEntry`. Transactional compare-and-delete prevents time-of-check/time-of-use partial cleanup.

**Alternatives considered**: Interactive stdin prompting was rejected because it complicates automation and is unnecessary when the command can require a second explicit flag. Deleting row-by-row was rejected because failure after the first deletion violates the all-or-nothing acceptance criterion. Hashing selected fields was rejected because full value equality is simpler and stronger.

## D7. Preserve the store schema and dependency graph

**Decision**: Add no database column, schema version, crate edge, or package. Reconciliation previews are ephemeral values. Existing JSON provenance and evidence carry every ownership and aggregate fact needed.

**Rationale**: The defect is policy, not missing durable data. A migration would add compatibility and recovery risk while providing no stronger ownership than the already stored source provenance.

**Alternatives considered**: A persistent discovery-ownership column was rejected because historical rows could not be populated truthfully and future rows already retain provenance. A deletion journal was rejected because one local transaction is fully atomic and the command has no external effects.

## D8. S133 intentionally supersedes the stale roadmap assignment

**Decision**: S133 closes issue #375. The roadmap's prior sentence assigning S133 to #331 is updated, and #331 remains open for a later slice code.

**Rationale**: The operator explicitly selected S133 after a complete backlog audit identified #375 as the sole standalone critical correctness issue and an unblocker for #376 and #380. Reusing the stale #331 assignment would contradict the authorized work and prioritize documentation over durable data integrity.

**Alternatives considered**: Renumbering this work S134 was rejected because the operator supplied S133 explicitly. Bundling #331 was rejected as unrelated scope and would violate single-slice cohesion.
