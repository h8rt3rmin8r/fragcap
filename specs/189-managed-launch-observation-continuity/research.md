# Research: Managed Launch Observation Continuity

## First-client exit

`CaptureSession::note_bound_exit` previously stopped on the first terminal exit, and `match_and_bind` rejected a second process for the same role. The process tree retains exited ancestors, so a later exact descendant can be recognized without broadening ordinary Capture. A managed-only provisional state preserves that causal chain. A missing successor ends with a named incomplete outcome.

## Process identity

The previous packet role map and CLI stage display used PID-only history. A reused PID could inherit a departed client's role. Publishing creation and exit intervals makes a packet's own timestamp select only the correct process instance. The S188 event-to-process path proof remains the stage matching authority.

## Time origins

The previous Capture duration began at arm and the outer Deep Capture observation budget omitted acquisition. Both could end before a client arriving within `--wait` received `--duration`. Live ETW events use Unix wall timestamps while deadline ticks use monotonic time since arm; an explicit anchor translates phase decisions. Packet and process artifacts retain their original timestamps.

Simple and multi-stage launch defaults are resolved from the prepared launch case. The accepted plan remains immutable. Post-authorization shortening is a refusal, and the bounded startup and return allowances remain visible.

## #547

S188/v0.10.13 shipped the basename-only stage path fix and precise acquisition timeout reporting. Installed behavior is observed after release, and any defect is reported as a GitHub issue.
