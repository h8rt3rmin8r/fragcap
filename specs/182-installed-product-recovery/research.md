# S182 research and decisions

## Relocation

**Decision**: Keep anchored identity immutable. Present a unique, validated old-to-new root proposal and apply it only after exact confirmation and a fresh store/platform read.
**Rationale**: `register_candidate` intentionally inserts only; making it silently update an existing anchor would hide changed launch authority.
**Alternatives**: Re-registration loses authored roles; export/edit/import is an emergency workaround; silent update makes ambiguity authoritative.

## Observation deadline

**Decision**: The facade owns the outer wall clock. Real Capture uses remaining time after setup and reserves finite completion margin. Genuine overruns still fail; later owner-release traffic never enters calibration.
**Rationale**: The adapter currently gives Capture the full outer duration after setup, so normal return work exceeds the outer deadline.
**Alternatives**: Removing the deadline permits stuck work; suppressing the facade error would certify late work.

## Packet omissions

**Decision**: Retain bounded cause and time intervals per flow. Gate rejection and history overflow use distinct causes. Unlocalized global loss remains conservative. Only a proven nonoverlap stops an omission from vetoing exact ownership.
**Rationale**: The current untimed count erases cause/time and invalidates later complete traffic.
**Alternatives**: Discarding omissions violates P-4; trusting every later packet invents certainty; retaining all omitted packets is unbounded.

## Managed shutdown

**Decision**: Select independently proven owners before passive descendant completion checks. Request owner close while observing helpers; release only after verified exit of all potentially routed descendants.
**Rationale**: The current helper pre-scan blocks owner close, which may itself retire helpers.
**Alternatives**: Global helper control expands rights; ignoring helpers falsely declares release.

## Final evidence

**Decision**: Treat stop, drain, cleanup, application trailer, and terminal as separate authorities. Timeout placeholders mean unavailable until real later runtime/artifact evidence reconciles the terminal. Later counts do not change observation eligibility.
**Rationale**: A stop-timeout zero survives despite retained traffic.
**Alternatives**: Copying final counts into calibration invents facts; leaving zeros hides observed traffic.

## Windows package

**Decision**: Statically link the MSVC C runtime for shipped executables, update PE contract from actual imports, and test MSI and ZIP startup in a clean guest. Npcap remains a separately diagnosed runtime requirement.
**Rationale**: Clean Windows lacks the v0.10.8 ordinary `VCRUNTIME140.dll` import. Rust documents `-C target-feature=+crt-static` for MSVC.
**Alternatives**: App-local runtime DLLs add license/servicing and MSI ownership complexity; a manual redist prerequisite fails the agreed portable startup contract. Sources: [Rust linkage](https://doc.rust-lang.org/stable/reference/linkage.html), [Microsoft CRT deployment](https://learn.microsoft.com/en-us/cpp/windows/deployment-in-visual-cpp?view=msvc-170), [Microsoft redistribution](https://learn.microsoft.com/cpp/windows/redistributing-visual-cpp-files).

## Physical reference

**Decision**: Correct the ordinary-user assertion, then generate a fresh exact official v0.10.8 physical reference with the corrected harness. Keep old record historical and candidate evidence separate.
**Rationale**: The 0.8.0 record is expired and nonancestral. Static checks cannot replace physical execution.
**Alternatives**: Widening freshness, relabeling old data, or skipping a row suppresses the failure.
