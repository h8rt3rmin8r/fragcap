# Calibration Case Contract

## CLI

Calibration retains `--calibrate <reachability|tls>` and `--launch-case <case>`, and adds:

```text
--calibration-protocol <routing|http1|http2|https|http3|websocket|sse|grpc|non-http-tls|socks5-tcp|socks5-udp|generic-tcp|generic-udp|quic>
```

The option is required when `--calibrate` is present and refused without calibration. `routing` is valid only for reachability. TLS requires a concrete protocol family. Reachability remains incompatible with trust, HAR, and key-log options.

## Plan and events

`deep_capture.calibration_plan` adds stable string fields:

- `routing_strategy`
- `address_family`
- `protocol`
- `fragcap_version`
- `target_version` (nullable)

The human plan displays the same fields before confirmation. The case is copied without reinterpretation into calibration phase events, `compatibility.json`, the manifest calibration object, and the terminal summary.

## Observation selection

A concrete selected protocol matches only the same S120 traffic family. `routing` permits routing facts but no protocol-specific positive fact. Other observed families remain in application and proxy evidence and contribute to an explicit mismatch outcome, but do not produce a selected-family positive fact.

## Refusal boundary

Invalid phase/protocol combinations, unavailable routing strategies, missing exact prerequisites, and plan-to-observation case changes refuse before any new external effect or fact append.
