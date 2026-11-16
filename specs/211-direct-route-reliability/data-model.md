# Data Model: Direct Route Reliability

| Entity | Required fields | Authority |
| --- | --- | --- |
| Pending helper | Exact tuple, endpoint generation, flow ID, receipt, start/deadline, first selected packet | Direct route worker |
| Helper outcome | Flow ID, stage, elapsed milliseconds, success or I/O kind/proxy refusal/timeout | Helper admission task |
| Selected disposition | Consumed, reinjected, or lost, with conserved count | WinDivert worker |
| Original packet evidence | Full or checksum-normalized datagram digest, passive presence or absence, history completeness; broader near-match counts are diagnostic only | Npcap flow registry |
| Client transfer | TCP acknowledged returned data or UDP peer reply | Direct bridge and native proxy |

Pending entries are finite and cannot outlive session cleanup. The route stream remains protected bundle content. Older route records remain readable.
