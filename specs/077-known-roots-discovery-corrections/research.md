# Research: Known-Roots Discovery Corrections

## D1. Represent containers as a classifier control verdict

**Decision**: Add `ClassifierVerdict::Container` and have known-root signature classification select it when findings contain more than one distinct engine product.

**Rationale**: The classifier owns interpretation of detection findings; the walker owns what each verdict means for traversal. Encoding the state in the existing verdict makes the distinction exhaustive and prevents every walker consumer from reinterpreting evidence independently.

**Alternatives considered**: Recount engines inside `KnownRootsSource` was rejected because it duplicates classification policy in traversal code. Returning `Miss` was rejected because a recognized organizational container is not an ordinary non-game miss and P-4 requires the distinction. Persisting a container target was rejected because the directory is specifically not one capturable title.

## D2. Count distinct canonical engine products

**Decision**: Container detection considers only findings in the engine category and counts unique product strings emitted by the signature set. Two or more unique products yield `Container`; duplicate markers for one product remain one engine.

**Rationale**: Signature product strings are the catalog's canonical identity at this boundary. Counting findings would misclassify one engine with several markers, while including anti-cheat or DRM would confuse supporting technology with game-engine identity.

**Alternatives considered**: Case-folding or alias inference was rejected because the signature catalog, not this walk, owns product normalization. Folder-name rules were rejected as a broader heuristic unsupported by the issue evidence.

## D3. Preserve the shallow bound and expose both container outcomes

**Decision**: Add `container_descended` and `container_descent_truncated` to `DiscoveryAccount`. A container increments exactly one. The latter also emits a warning naming the directory whose descendants may remain undiscovered.

**Rationale**: The directory itself is one considered item and needs one terminal account outcome. A traversable container leads to separately counted child items. A terminal-depth container cannot state how many unseen children exist without listing them, so the truthful quantity is the number of known containers whose descent was truncated, accompanied by a coverage warning.

**Alternatives considered**: Reusing `considered_not_a_game` loses the reason and cannot expose truncation. Increasing `MAX_DESCENT` expands cost and filesystem risk beyond the observed defect. Counting hypothetical descendants would invent a quantity the walk did not observe, violating P-9.

## D4. Compose paths component by component at the real boundary

**Decision**: Keep the generic walk and `KNOWN_ROOTS` in their shared separator-neutral form. `FsDirectoryLister` converts forward slashes to the Windows native separator immediately before `read_dir`; children returned by that lister are then native and are used unchanged.

**Rationale**: `KNOWN_ROOTS` deliberately uses one representation shared with fixture trees. Converting in `KnownRootsSource` would also convert fixture lookup keys on Windows, coupling a pure test seam to host spelling. The real lister is the first boundary that needs a native path and the source of child `PathBuf` values, so normalization there fixes both the read input and the emitted path prefix without affecting fixtures or applying filesystem canonicalization.

**Alternatives considered**: Component-wise composition in the generic walker was the initial plan and was rejected during implementation because it breaks the platform-neutral fixture keys on Windows. Replacing separators on every emitted child spreads normalization beyond the boundary. `std::fs::canonicalize` was rejected because it requires existence, resolves links, changes path spelling, and adds I/O unrelated to joining.

## D5. Correct future discovery without migrating historical rows

**Decision**: S077 does not rewrite previously stored mixed-separator paths.

**Rationale**: Existing rows are user-owned historical identities. A migration needs collision, case, missing-path, and rollback policy that issues #209 and #210 do not define. Newly discovered candidates are corrected before any later persist-on-first-use operation.

**Alternatives considered**: Opportunistic rewrite during listing was rejected as a hidden database mutation and a P-9 risk. A schema migration cannot safely decide semantic identity from separator replacement alone.

## D6. No new dependency or crate edge

**Decision**: Use `HashSet` and `PathBuf` from the standard library in the existing `fragcap-targets` crate.

**Rationale**: Both operations are small and already expressible with the standard library. Classification and traversal remain behind established seams.

**Alternatives considered**: A path-normalization crate adds graph and MSRV risk without solving the deliberately excluded case, link, and historical-row policies.
