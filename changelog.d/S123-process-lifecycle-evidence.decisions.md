<!-- spec-impact: 10, 13.7, 19, 25, 28.1 -->
Define an observed process instance by PID plus creation event time. A snapshot
remains a weaker query-only authority, and PID reuse or out-of-order delivery
cannot transfer stage or socket ownership across lifetimes.

Retain raw process events under a fixed in-memory bound during capture, then
write only launch, stage, flow-owner, and ancestor-relevant records. Socket
ownership comes exclusively from the existing packet flow registry. Kernel
event loss, buffer loss, ignored rundown, retention overflow, missing exits,
and unresolved joins remain explicit. S123 adds no process handle, memory
right, second attribution pass, dependency, or Deep Capture completion claim.
