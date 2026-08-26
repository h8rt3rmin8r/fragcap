# Contract: Traffic Support Reference

## Required Columns

The public table contains these columns for every required traffic family:

- traffic family;
- Capture visibility;
- Deep Capture outcome;
- prerequisites or blockers;
- outputs and analyzer use.

## Required Rows

| Traffic family | Capture | Deep Capture | Required limitation |
| --- | --- | --- | --- |
| HTTP | Packet and payload bytes when enabled | HTTP method, URL, and status through the proxy | Current application records omit headers and bodies |
| HTTPS | Encrypted packets | HTTP semantics only when proxy-routed and CA-accepted | No pinning bypass or target key extraction |
| WebSocket | Packets | HTTP upgrade handshake only | No WebSocket frame records or payload retention |
| Non-HTTP TLS | Encrypted packets | Metadata-only proxy observation | No custom protocol dissection |
| QUIC | UDP packets | Unsupported by current proxy path | No QUIC routing or decryption claim |
| UDP | Packets | Unsupported by current proxy path | No generic UDP proxy inspection |
| Plaintext | Packet and payload bytes when enabled | HTTP follows HTTP row; custom protocols unsupported at application layer | No generic custom protocol dissection |

## Required Explanations

The reference must explain:

- Capture is passive and Deep Capture is explicit active proxy inspection;
- traffic must actually reach the scoped proxy;
- TLS inspection requires acceptance of the fragcap-owned local CA;
- `full` means HTTP semantics were observed, not that all payload material was
  retained;
- TLS key logs are proxy-owned analyzer aids and not keys extracted from the
  target;
- HAR is emitted only from observable HTTP semantics;
- compatibility facts are local evidence, not a public title verdict;
- `targets show` is read-only and explicit measurement is the refresh path.
