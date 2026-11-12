# Data Model: Packet Correlation Integrity

| Entity | Identity | Required facts | Boundary |
| --- | --- | --- | --- |
| Causal generation | PID plus creation instant and ancestry to exact launch root | Start, exit, role/stage when proven, event authority | PID alone never proves ownership |
| Packet flow | Protocol and canonical endpoint tuple | Capture instant, attribution, direction, retained state | Driver observation is distinct from proxy or route receipt |
| Omission interval | Flow ID, first/last observation time, cause | Count and precision status | Unknown or global loss cannot be assigned to a flow |
| Proxy connection | Session-local connection ID and exact window | Peer pair, open/close time, terminal, transfer | Overlap is checked against the same flow's interval |
| Direct receipt | One-use original tuple, generation, route outcome | Client response and nonzero two-way transfer | Positive route fact does not create passive packet truth |
| Compatibility fact | Case key and evidence lineage | Verdict, freshness, route applicability | Later admission checks exact current case |
