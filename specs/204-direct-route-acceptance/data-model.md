# Data Model: S204 Acceptance

S204 reuses the S203 native schema. It does not create a new persistent entity.

| Entity | Existing authority | Acceptance invariant |
| --- | --- | --- |
| Selected route flow | One-use receipt, exact process generation, original tuple, route flow ID | Only exact owned traffic may enter the proxy. |
| Client delivery | Synthetic reply, observed ACK or datagram response, two-way transfer | Upstream connection or send success alone is insufficient. |
| Passive packet | Npcap flow registry and packet evidence state | Absence after diversion remains unavailable, never synthesized. |
| Calibration decision | Compatibility observation, verdict, stored route fact, ordinary admission | All four surfaces use the same applicability and reason. |
| Route stream | Protected route.jsonl, sequence, header/trailer, loss | A valid prefix is partial; queue and writer loss remain counted. |
| Debug report | Derived JSON projection of native artifacts | Source bytes remain unchanged; missing authority is explicit. |

No target-store migration or artifact schema bump is planned. If a failing test requires a schema change, update this model, the contract, master specification, and analyze gate before implementation.
