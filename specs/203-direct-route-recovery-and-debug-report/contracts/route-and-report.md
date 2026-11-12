# Contract: Route Stream and Debug Report

## Native route stream

The protected route.jsonl is a versioned newline-delimited JSON stream. It begins with a route header naming schema version, session, product and signed driver versions, loopback family, route scope, and clock anchors. Records have a strict ascending sequence, selected flow ID, event kind, bounded metadata, and timestamp uncertainty. Event kinds include selected, proxy-attempt, proxy-connected, proxy-refused, synthetic-reply-generated, synthetic-reply-sent, synthetic-reply-failed, client-response, transfer, terminal, and gap. TCP and supported UDP use one stable vocabulary with protocol-specific optional fields. The trailer reconciles records, queue drops, writer failures, selected flows, no-byte closures, and packet accounting. A readable prefix without trailer is partial. Unknown schema is unavailable, not guessed.

The native stream may contain original endpoint and process-generation metadata and is sensitive. It is registered in manifest.json with source authority, finalization, completeness, and loss, protected under the existing recipient policy, and excluded or transformed by bundle share export. The secret route receipt is never serialized.

## Correlation and calibration

Each route flow ID joins an exact one-use receipt and the proxy leg. Packet-flow ID is optional and never synthesized from a route receipt. A positive direct reachability fact needs exact final-client generation and original destination, a client-visible response and two-way transfer. Passive packet absence is recorded independently as a packet-evidence omission. Competing owners, reused generations, missing delivery, no payload, partial route events, or failed cleanup prevent a positive fact. Compatibility, verdict, target store, and ordinary admission use this same decision.

## CLI debug report

Command: fragcap bundle debug-report BUNDLE --out REPORT.json. The command accepts completed or partial native bundles, verifies access and exact paths, reads only bounded native artifacts, never mutates BUNDLE, and atomically creates a distinct REPORT.json. Human output prints the destination and summary through the shared display layout; --json emits the same summary in a stable machine object. The report is a JSON object with schema_version, product_version, source_manifest, artifact_status, route_scope, launch_scope, driver_version, flows, calibration, cleanup, omissions, and transformation. Each flow has route_flow_id, protocol, milestones, owner, proxy_leg, packet_evidence, client_delivery, transfer, terminal, and evidence_status. Exact endpoints and account paths are omitted in the exported projection with a declared transformation; native source retains them for an authorized maintainer's optional capture join. Unknown or partial source fields are explicit and never defaulted to success.

## Scope help

The active --inspect-scope help says auto resolves to launch-chain for every supported topology and explicit game chooses game-only inspection. The help conformance test calls the resolver for direct and launcher-mediated topologies and compares both result and help text.
