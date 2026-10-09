# Research: S169

## 2026-10-09 acquisition research

Decision: Require recognized loopback, provide exact session listener locality and retain its TCP endpoint through per-handle BPF narrowing.

Rationale: Core explicit selection bypasses loopback inclusion (interface.rs:432); automatic selection uses only its flag (516), while Doctor recognizes Npcap descriptions (136). Assembly uses enumerated addresses (assemble.rs:553); no local address yields no flow key (parse/direction.rs:80), so gate rejection cannot create a flow summary (pipeline/mod.rs:1078). Dynamic filters have a two-second debounce/five-second reinstall cadence without an immutable listener allowance.

Alternatives considered: Broad capture or disabled filtering admits unrelated traffic; globally loosening locality alters physical-source semantics. Prefer CLI private exact listener configuration, additive Pipeline/FilterManager methods and unchanged public configuration literals.

## 2026-10-09 ownership research

Decision: Resolve both exact TCP loopback orientations in RoleStampingAttributor using one binding snapshot and select only a unique observed bound owner.

Rationale: Canonical parser keys and native correlation sort endpoints, while TCP table matching requires its local endpoint orientation (index.rs:67). The lower server port can resolve the proxy socket. The facade has stage authority; inner lookup keeps creation-time, retention and total ranking.

Alternatives considered: Changing canonical identity breaks durable joins; selecting an arbitrary endpoint/PID has no target authority; merging acquisition and attribution violates P-3. Two bound owners remain unresolved. Consistent same-PID results preserve weaker fidelity; conflicting process identities remain unresolved.

## 2026-10-09 gating and verification research

Decision: Preserve watching retention and exact timestamp-window correlation. Use real synthetic TCP endpoints with synthesized frames plus declared owned platform/client observations.

Rationale: Pipeline starts before launch and opens retention only from terminal-client acquisition. Parsed rejected flows already create unretained summaries, so they yield packet-history-bound-exceeded rather than packet-flow-not-observed. Registry sharing is correctly wired. DLT_NULL already parses Windows AF2/23. Existing native tests cover absent, conflicting, reused and history-bounded cases; mapped endpoints are not a confirmed field cause.

Alternatives considered: Proxy accepts or HTTP success cannot supply process ownership. Real games, real trust mutation and future operator verification are unnecessary for controlled acceptance. Synthetic frames are not Npcap-acquired traffic and the field cause remains unverified.
