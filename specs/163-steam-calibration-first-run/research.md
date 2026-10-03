# Research: S163 Steam Calibration First Run

## Owned root binding

**Decision**: Bind the exact managed root in `CaptureSession` using a receipt-gated authority, matching the event PID, observed parent, creation interval, expected basename, and any reported full path.

**Rationale**: ETW `ImageFileName` is ordinarily a basename. The synthesized platform stage requires an exact full-path regex, so generic profile matching cannot bind the root. Snapshot matching occurs before launch, so deleting the regex would allow an already-running same-name process to bind before the receipt. The process tree already distinguishes PID instances.

**Alternatives considered**: Global basename-only stage rule was rejected because it discards the prepared path and admits foreign roots. A second launch ownership state machine was rejected because the existing session owns stage binding and ancestry.

## Client evidence and authoring

**Decision**: Provide a bounded read-only ETW observation of the owned launch chain joined to IP Helper socket table rows, plus an explicitly labeled manual declaration fallback. Reuse the exact client-authoring plan and CAS after a selected executable is known.

**Rationale**: Steam app metadata names a launch executable, which can be an intermediate launcher. The current `calibrate` setup creates a plan from that hint and asks the user to assert socket ownership. ETW supplies the owned ancestry and IP Helper directly reports socket-owning PIDs without a target handle. This is client-identification evidence, not packet traffic or Deep Capture reachability. Ordinary unresolved Capture does not fit this case: Steam resolution enters the install-layout path first, and its observe mode automatically promotes a dominant image without the setup plan's CAS.

**Alternatives considered**: Automatically trusting the hint would repeat the observed failure. Asking a network-socket yes/no question would require knowledge the user does not have. Reusing ordinary Capture promotion would silently author a target without review.

## Session reporting

**Decision**: Use typed process, Capture, proxy, compatibility-write, and workflow evidence to produce a single prioritized failure summary. Emit exact cleanup results once.

**Rationale**: The current terminal report prints every artifact and cleanup result but has no causal stage. `lifecycle_progress` prints the same generic cleanup sentence for every resource. `calibrate` collapses delegated failures to one reason. A zero-observation failure may write a launch-case fact while writing no proxy-routing row, so a generic compatibility claim is inaccurate.

**Alternatives considered**: Parsing console prose or guessing from artifact existence was rejected because neither carries the required authority. Suppressing structured lifecycle events was rejected because those remain the audit stream.

## Documentation and controlled evidence

**Decision**: Document the actual setup, authorization, launch, reachability, protocol, failure, and resume states with synthetic examples. Test the full controlled chain and negative identity cases without a real title.

**Rationale**: A future operator field measurement is useful but is not an implementation gate under constitution 1.5. Public examples must not disclose the operator's title or retained capture data.
