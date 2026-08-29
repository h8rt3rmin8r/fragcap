# Research: Deep Capture Compatibility Bootstrap

## R-1: Extend the Existing Command With Optional Calibration Flags

**Decision**: Use `fragcap deep-capture <TARGET> --launch --calibrate <reachability|tls> --launch-case <CASE> [OPTIONS]`.

**Rationale**: The existing command already owns target selection, capture controls, scoped launch, proxy selection, bundle output, and trust intent. Optional flags preserve every ordinary invocation. A nested `deep-capture calibrate` command would collide with the positional target grammar and force a broader parser change.

**Alternatives rejected**:

- A separate `targets calibrate` command places active interception under a read-mostly target-management surface and duplicates Deep Capture controls.
- A new top-level command duplicates the proxy, bundle, output, and target contract.
- Silently treating unknown ordinary Deep Capture as calibration weakens the existing safety refusal.

## R-2: Require an Explicit Launch Case and Verify Actual Launch State

**Decision**: `--launch-case` is mandatory with `--calibrate` and invalid without it. All existing launch-case values parse, but S097 permits only `steam-protocol-cold` for real targets. The declared case must match observed Steam state before any mutation.

**Rationale**: Calibration evidence is keyed by launch case. Inferring the operator's intended case from a machine snapshot hides the experiment being performed. Parsing unsupported cases lets the command issue precise refusal diagnostics without claiming support.

**Alternatives rejected**:

- Inferring the case makes a warm/cold race look like an intended measurement.
- Accepting direct and publisher launches would absorb issue #254 and launch paths the product does not own.

## R-3: Display a Complete Plan Before Confirmation

**Decision**: Build the plan after side-effect-free target, launch-state, bundle, capture, and backend-name validation. Emit it in human or structured form, then use an injected confirmation seam. `--yes` preconfirms but never suppresses the plan. JSON or noninteractive input without `--yes` refuses before mutation.

**Rationale**: Explicit visibility and consent are product requirements for active inspection. The `doctor --fix` confirmer supplies the local pattern, but calibration writes through the existing emitter because Deep Capture status belongs on standard error.

**Alternatives rejected**:

- Confirmation before the plan cannot authorize specific effects.
- Prompting in JSON mode corrupts the machine-readable event stream.
- Requiring terminal stdout copies a doctor-specific constraint that does not apply to Deep Capture's stderr event stream.

## R-4: Keep Reachability and TLS as Separate Executions

**Decision**: Reachability rejects trust, HAR, and key-log options and never constructs a trust manager. TLS requires current same-target, same-launch `proxy-routing=reached-client` evidence before any mutation, then follows the confirmed current-user CA path.

**Rationale**: Scoped reachability can be learned without trust. Delaying trust reduces effects and ensures TLS testing occurs only after the target is known to reach the loopback proxy.

**Alternatives rejected**:

- One combined run mutates trust before routing is established.
- A TLS request based on stale, conflicting, or another-case evidence repeats the circular safety problem.

## R-5: Correct Routing and Propagation Semantics

**Decision**: A proxy event correlated to a client-owned packet flow proves `proxy-routing=reached-client`. It does not prove `proxy-propagation=confirmed`. Real calibration writes `not-tested` or `not-confirmed` unless a separate non-invasive source proves propagation. The self-reporting controlled target may write `confirmed`. Ordinary Deep Capture eligibility requires current same-case final-client routing, which is the safety condition it actually consumes.

**Rationale**: The current MVP writer equates correlated routing with independent environment propagation. The governing Steam inheritance plan explicitly forbids that inference. Retaining both as gate requirements would make real calibration unable to open the gate honestly.

**Alternatives rejected**:

- Preserving the current writer keeps a false public fact.
- Requiring propagation confirmation for eligibility incentivizes the same false claim and measures an implementation mechanism rather than the product safety condition.
- Dropping propagation facts loses useful diagnostic evidence; they remain distinct and append-only.

## R-6: Classify Negative Outcomes Only From Affirmative Evidence

**Decision**: Reachability classification may use proxy observations plus a narrow read-only completed-flow view. `launcher-only-routing`, `escaped-tree`, and `no-proxy-traffic` require the corresponding launcher, ancestry, or direct target-traffic evidence. Silence alone becomes `inconclusive` or `no-relevant-traffic`. TLS pinning requires an explicit backend diagnostic; a generic TLS failure remains unknown.

**Rationale**: A deadline proves that a condition was not observed, not why it was absent. Outcome labels remain separate from fact rows.

**Alternatives rejected**:

- Inferring no-proxy or pinning from silence fabricates negative evidence.
- Classifying from proxy records alone cannot see escaped or direct traffic.

## R-7: Reuse the Fact Schema and Extend Existing Sidecar Authority

**Decision**: Use the existing `CompatibilityFact` and append-only store with no migration. Build phase-aware pending facts carrying one observation timestamp, backend provenance, actual owner context, and freshness. Extend `compatibility.json` with the plan, phase outcome, observations, omissions, and individual fact-write results. Keep `cleanup.json` authoritative for resource cleanup.

**Rationale**: The existing row already carries every required provenance field. A second table, target resolver, or sidecar would violate P-10 and create conflicting authorities.

**Alternatives rejected**:

- Adding a calibration table duplicates durable evidence.
- Encoding phase outcomes as compatibility facts turns a run result into a target verdict.

## R-8: Finalize Facts, Audit Artifacts, and Cleanup Independently

**Decision**: After confirmation, route every path through one bounded finalizer. Attempt cleanup, observed fact appends, compatibility sidecar, cleanup report, and manifest independently, recording each result before returning the combined error.

**Rationale**: The current sequence writes the bundle before facts, so a writer failure can discard already observed facts. Early proxy and trust failures also produce incomplete audit state, and some waits are unbounded. The new ledger reconciles every planned resource and write.

**Alternatives rejected**:

- Returning on the first finalization error loses later cleanup or evidence.
- Treating bundle success as fact-write success hides partial persistence.

## R-9: Keep Extraction and Backend Replacement Out of S097

**Decision**: Add private models and narrow seams within `fragcap-cli::commands::deep_capture`. Add a platform-neutral completed-flow accessor only if the outcome classifier needs it. Keep `mitmdump` and the controlled adapter.

**Rationale**: Issue #252 owns the library-first extraction, #253 owns the native backend spike, and #254 owns direct launch. Pulling them into S097 would enlarge the risk surface without improving the bootstrap contract.

## R-10: Verification Proves Orchestration, Not Real Steam Inheritance

**Decision**: Controlled tests self-report inherited proxy variables, drive synthetic outcome variants, avoid the real trust store, and exercise production persistence, bundle, event, and cleanup paths. A manual cold Steam calibration remains the evidence tier for real launcher behavior and stays private unless scrubbed.

**Rationale**: Continuous integration has no account, title, Npcap driver, or trustworthy Steam state. It can prove state-machine behavior without making a platform claim it did not observe.
