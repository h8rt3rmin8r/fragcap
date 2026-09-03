# Research: Exhaustive Protocol Classification

## Decision 1: Separate Raw Evidence From Public Classification

**Decision**: Keep raw protocol, TLS, parser, transport, retention, and writer evidence in its owning records. Derive one typed, versioned public classification in the facade and carry it alongside raw evidence.

**Rationale**: A stable product vocabulary is required for artifacts and compatibility policy, but raw evidence must remain available under P-9. The facade already owns public session policy and artifact assembly, while `fragcap-proxy` owns protocol engines and raw observations.

**Alternatives considered**: Replacing raw reasons was rejected because it erases diagnostic truth. Publishing proxy-internal error strings as the stable contract was rejected because it couples product compatibility semantics to implementation details. Moving classification policy into the CLI was rejected because library consumers require the same result.

## Decision 2: Model Orthogonal Axes Rather Than One Outcome Enum

**Decision**: A classification contains traffic family, detection state, inspectability state, and optional stable reason. Construction validates permitted combinations against a closed matrix for schema version 1.

**Rationale**: Detection, inspectability, and failure authority answer different questions. A single flat enum either explodes combinatorially or collapses meaningful distinctions. Orthogonal bounded axes preserve facts while matrix validation prevents impossible combinations.

**Alternatives considered**: One flat status list was rejected because HTTPS can be identified while encrypted-opaque, pinned, metadata-only, or fully inspectable. Independent unvalidated strings were rejected because contradictory combinations would serialize successfully.

## Decision 3: Unknown, Unsupported, And Failed Are Evidence States

**Decision**: Unknown means no supported identification could be made from retained evidence. Unsupported means a family or version was identified and deliberately has no supported handler. Failed means a supported handler was attempted and direct processing evidence records failure.

**Rationale**: These meanings prevent silence, parser defects, and product boundaries from becoming interchangeable. They also make issue #317 calibration facts safe to consume.

**Alternatives considered**: Treating unknown as unsupported was rejected because absence of identification is not a product decision. Treating parser failure as unsupported was rejected because a defect or malformed message does not prove target incompatibility.

## Decision 4: Compatibility Promotion Uses Explicit Eligibility Predicates

**Decision**: Define evidence predicates for each fact family. Positive protocol and inspectability facts require an identified supported family and an eligible observed inspectability state. Routing and trust facts retain their existing correlation and phase requirements. Failure and omission reasons are never promoted as positive support facts.

**Rationale**: Compatibility facts are durable and later control eligibility. They need stronger proof than a summary label. Per-fact predicates preserve the existing authority split and prepare #317 without implementing its expanded matrix or stale-evidence rules.

**Alternatives considered**: Promoting every retained observation was rejected because parser and writer failures would become durable target judgments. Suppressing all negative observations was rejected because append-only evidence must remain visible.

## Decision 5: Summaries Are Derived And Conserved

**Decision**: Count classifications by stable detection, inspectability, and reason labels from retained detailed observations. Record bounded lost-observation totals separately. Human and JSON renderers consume the same summary value.

**Rationale**: One derived value prevents renderer drift and makes conservation testable. Lost observations cannot be assigned a protocol classification because their detailed evidence is unavailable.

**Alternatives considered**: Independent renderer counting was rejected because current inspectability counters already duplicate filtering logic. Assigning lost events to unknown was rejected because it invents an observation.

## Decision 6: Keep Artifact Omission Authority Distinct

**Decision**: Use a typed omission reason vocabulary and severity mapping when building manifest entries, but do not infer a protocol classification from an artifact omission or vice versa.

**Rationale**: `writer-failed` means an artifact could not be produced, while `parser-failed` means supported traffic processing failed. Both can occur during one otherwise successful transport session. The manifest owns artifact truth, not protocol truth.

**Alternatives considered**: One shared terminal status was rejected because it loses partial success. A manifest version bump was rejected because the existing version 2 schema already permits additive stable reason values.
