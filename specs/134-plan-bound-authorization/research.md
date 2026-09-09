# Research: Plan-Bound Deep Capture Authorization

## D1. Use one same-process exact-plan authorization exchange

**Decision**: Interactive use displays the complete plan and accepts one affirmative response. Structured or redirected automation selects `--authorize-stdin`, consumes the emitted plan, and writes that plan's complete identifier back to standard input in the same process. JSON requires this dedicated input and never prompts.

**Rationale**: The plan contains a freshly generated session CA. A separate invocation cannot reproduce or safely reconstruct that private identity without persisting sensitive key material or weakening certificate binding. The same-process exchange preserves ephemeral ownership, works for automation with bidirectional pipes, and never introduces a reusable approval switch.

**Alternatives considered**: A generic `--yes` was rejected because it is not plan-bound. A caller-supplied digest was rejected because the caller cannot know the new certificate thumbprint. A plan file containing private key material was rejected because it creates a new sensitive persistent artifact and recovery problem. A deterministic CA was rejected because deriving private identity from public plan fields would destroy key secrecy.

## D2. Canonicalize the review plan and use its digest as the library plan id

**Decision**: Serialize a versioned fixed-field authorization plan with deterministic collection ordering and derive `plan-v1:<hex>` from those bytes using the existing workspace BLAKE3 package. Supply that complete identifier through the existing facade `IdentifierSource`, so `Authorization::Approved` remains bound to the same value the operator reviewed.

**Rationale**: A digest makes every listed authorization field tamper-evident and avoids a second independent identifier. The existing facade coordinator already consumes exactly one `PlanId` and refuses mismatch, so reusing that boundary keeps lifecycle authority singular.

**Alternatives considered**: Keeping the current random plan label was rejected because it proves object identity within one process but does not demonstrate that authorization-relevant content was canonicalized. A second CLI token plus the existing random library id was rejected because two identifiers could drift. Hashing only a subset was rejected because omitted fields could change without invalidating authorization.

## D3. Generate the exact CA before authorization but persist nothing

**Decision**: Add a process-local prepared native authority that exposes only public certificate identity for planning and can be consumed exactly once by the existing native runtime. Key material stays zeroizing and in memory. Decline drops it. Successful execution uses that exact CA for client-facing TLS and current-user trust.

**Rationale**: The operator cannot authorize an exact trust action until the certificate thumbprint exists. In-memory certificate generation changes no trust store, bundle, listener, target, or external system and creates no recovery obligation. Passing the same authority into runtime prevents the displayed and installed certificates from diverging.

**Alternatives considered**: Showing a placeholder thumbprint was rejected as untruthful. Generating the CA after authorization was rejected because the authorization would not bind the certificate. Persisting a prepared private key before authorization was rejected because refusal would already require sensitive cleanup.

## D4. Authorize loopback scope before selecting the concrete port

**Decision**: The canonical plan binds the requested IPv4 or IPv6 loopback family, target-scoped route strategy, bypass policy, environment ownership, and no-fallback rule. The facade reserves the concrete port only after authorization and must realize it within that exact scope.

**Rationale**: Selecting an available ephemeral port without binding cannot be made race-free, while binding before authorization violates issue #382. The port does not broaden authority when it remains an exact loopback endpoint inside the displayed family and is recorded by the existing runtime and artifacts.

**Alternatives considered**: Pre-binding the current retained listener was rejected by the zero-effect authorization boundary. Choosing an unverified fixed port was rejected because routine collisions would make plans unusable. Authorizing any interface or wildcard endpoint was rejected because it broadens scope and violates P-1.

## D5. Revalidate mutable launch authority before the first session effect

**Decision**: Retain a complete target launch-authority snapshot in the plan, resolve it immediately after authorization, and require the facade's final target resolver to compare its independently refreshed result before endpoint or Capture preparation. Stable id, anchor, install root, launch entries, observed launch case, selected paths, bundle, policy, artifacts, deadlines, versions, and prepared certificate identity must still match. The prepared CA must remain inside its displayed validity period when execution begins.

**Rationale**: The in-memory plan is immutable, but the target store and running-process state can change while a person reviews it. Refusing drift before listener reservation preserves the meaning of exact authorization without silently rebuilding a different plan.

**Alternatives considered**: Trusting the earlier snapshot was rejected because the launch could change. Rebuilding and proceeding was rejected because it would execute an unauthorized plan. Re-prompting inside the same invocation was rejected because the one-plan contract is simpler and the operator can rerun.

## D6. Keep one bounded migration release without preserving insecure behavior

**Decision**: Remove Deep Capture `--trust-ca` and `--yes` from normal help immediately. Accept them as hidden parser inputs for one tagged release solely to return an actionable usage error naming interactive authorization and `--authorize-stdin`. They never satisfy authorization. Other commands' `--yes` flags do not change.

**Rationale**: A migration error helps existing scripts without extending the security defect. One release is bounded, testable, and appropriate for the current pre-1.0 command surface.

**Alternatives considered**: Honoring the flags with warnings was rejected because automation could continue generic authorization. Removing parser recognition immediately was rejected because clap's unknown-argument error cannot explain the replacement. Renaming `--yes` globally was rejected because Doctor repair, bundle cleanup, and target reconciliation have separate exact action models.

## D7. Preserve warm restart as a distinct operator preparation step

**Decision**: Warm restart retains the initial prompt that asks whether fragcap should wait while the operator closes the application normally. It cannot be preconfirmed. After cold state and fresh target resolution, the ordinary complete plan is built and authorized once; the current second session-authorization prompt is removed.

**Rationale**: Waiting for an operator-owned normal shutdown is not authorization for proxy, trust, bundle, launch, or fact effects. Combining it with the final plan would authorize a plan whose cold launch authority does not yet exist. Separating preparation from one final effect authorization preserves S113 and issue #382.

## D8. Refuse prior-session recovery before constructing a new plan

**Decision**: Perform one bounded, read-only inspection of the existing Deep Capture owner registry and resource journals before preparing or emitting a new authorization plan. If an inactive prior session has recovery actions or refusals, stop and direct the operator to `fragcap doctor --fix`. Never replay prior recovery from the new session path.

**Rationale**: Automatic recovery can remove a prior CA, terminalize journals, or retire ownership records. Those effects belong to the prior session and are absent from the new session's plan, so accepting the new plan cannot authorize them. This deliberately supersedes the initial assumption that every filesystem scan must wait until after authorization: a bounded read-only authority check is not a session effect and is required to keep unrelated recovery outside the plan.

**Alternatives considered**: Listing every prior recovery action in the new plan was rejected because it couples unrelated sessions and can make the plan change while it is reviewed. Continuing automatic recovery after approval was rejected because it silently widens authority. Ignoring pending recovery was rejected because stale trust or incomplete ownership can conflict with a new run.

**Alternatives considered**: Removing the shutdown interaction was rejected because fragcap cannot safely control the process. Letting automation bypass it was rejected because image-name observation cannot prove ownership. Keeping the current extra cold-plan prompt was rejected because the complete plan now owns that decision.
