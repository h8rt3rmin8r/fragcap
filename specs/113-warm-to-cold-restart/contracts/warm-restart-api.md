# Contract: Warm Restart API

The facade exposes immutable policy values for a warm restart plan, its supported warm-to-cold mapping, and terminal outcome vocabulary. Construction rejects cold or unsupported input, empty image sets, and unbounded deadlines.

The CLI adapter supplies:

1. Current target resolution and declared images.
2. Explicit consent.
3. Repeated complete process-image inventories until cold or deadline.
4. Fresh target resolution and launch preparation.
5. Separate authorization for the resulting prepared session.

The adapter has no process-control operation. The contract cannot request termination, signaling, window messages, relaunch, or a target process handle.

Structured events expose `deep_capture.restart_plan` and `deep_capture.restart` with stable stage, outcome, warm case, optional cold case, deadline, and reason fields.
