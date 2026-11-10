# Research: Direct Route Qualification

## R1. The 10 ms candidate expiry

**Observation**: S200's pending buffer checks expiry before checking available proof. `direct_runtime.rs` sets a 10 ms candidate wait and a 30 ms socket-event history. Intermittent managed UDP bypasses occurred in hosted runs even though subsequent exact-head repetitions passed. The disposable #587 sweep passed 20/20 cold chains at 100, 250, and 500 ms, and 20/20 at 1000 ms after correcting dependent synthetic deadlines. Selected decisions took at most about 17 ms. The cause of the earlier 10 ms misses is still unproven.

**Decision**: Measure receipt, event publication, process evidence, and decision separately. Evaluate exact proof before expiry. Start controlled qualification at 100 ms and retain 250, 500, and 1000 ms as comparison points. The implementation value is chosen only after paired selected-success, unrelated-latency/loss, and capacity evidence.

**Alternatives**: Retain 10 ms, which has observed missed routes; set 1000 ms by intuition, which made unrelated candidates wait a full second in the investigation; select a game owner from an absent event, which violates P-1 and P-9.

## R2. Known unrelated candidates

**Observation**: A complete PID-bearing socket event can identify a nonselected owner before the current candidate deadline. The S200 receive path sends many first packets through the pending buffer, where missing process evidence can make even a known unrelated packet wait.

**Decision**: Use the socket observer's exact current tuple and endpoint ownership to reject a proven helper or unrelated PID at receipt. The pending buffer also applies a proven nonselected process result as soon as evidence arrives. Do not infer an owner from an empty or ambiguous observer state.

**Alternatives**: Wait out the entire candidate deadline, which raises avoidable unrelated latency; classify by destination or image basename, which is not exact process-generation proof.

## R3. Integrated route authority

**Observation**: S200 already defines `RoutingPlan.direct_socket` and `NativeDirectRouting`, but the public CLI still constructs `ChildEnvironmentRouting` and calls ordinary `DeepCapture::preflight`. Compatibility policy currently keys positive reachability to environment-only cases.

**Decision**: Reuse the existing immutable plan and native adapter. Choose a route kind before preflight, bind it to compatibility case identity, and require the same kind in the stored fact. Preserve child-environment routing in the direct plan. A direct positive needs original final-client receipt, native proxy outcome, and independent packet evidence. Legacy facts stay readable but do not authorize the new route.

**Alternatives**: Infer direct route from a generic `reached-client` fact, which confuses old environment-only evidence; add a second unaudited post-authorization route choice, which violates the plan boundary.

## R4. Documentation and package truth

**Observation**: S199 bundles signed WinDivert in MSI/ZIP and S200 uses it only for authorized direct routing. The MSI already provisions the installation CA, while portable setup may be explicit. Npcap remains separate for live Capture. Extcap registration and live execution have distinct privilege paths.

**Decision**: Check command and installer source before stating every privilege row. Keep MSI and portable instructions separate. Describe routed, inspected, opaque, refused, bypassed, and packet-only outcomes from actual evidence.

**Alternatives**: Copy older portable CA instructions into MSI onboarding or imply every command needs Administrator; both misstate the shipped product.
