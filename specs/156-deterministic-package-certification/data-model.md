# S156 Data Model

## PackageCertificationReportV4

- `schema_version`: exact integer `4`.
- All package, release identity, artifact, entry, PE inspection, lifecycle, fresh-start, findings and completion fields retain their schema-3 meaning.
- `smokes`: exactly two unique rows, one `portable` and one `installed`.
- Each smoke row is closed and validates independently.

## ControlledSmokeV4

- `surface`: `portable` or `installed`.
- `executable_sha256`: exact certified `fragcap.exe` digest.
- `backend`: exact `fragcap-native`.
- `network`: exact `loopback-only`.
- `firewall_containment`: exact `outbound-non-loopback-blocked`.
- `process_observation`: exact `complete`.
- `structured_reachability`: exact `proxy-started-and-reached-client`.
- `session_evidence`: structured identity summary containing one matching session, exact proxy loopback family, positive port, reachability phase, routing protocol, controlled launch case and terminal status.
- `socket_observation`: diagnostic-only sample count, product and permitted system-process counts, endpoint count, non-loopback count and exact-loopback sample Boolean.
- `cleanup`: exact `reconciled`.
- `complete`: exact `true`.

`socket_observation.endpoint_count` may be zero and `loopback_socket_observed` may be false. Sample count and product process count remain positive. Non-loopback count remains exactly zero.

## SessionEvidenceV1

- `schema_version`: exact integer `1`.
- `session_id_sha256`: lowercase SHA-256 of the nonempty shared session identifier, so public evidence binds identity without publishing it.
- `proxy_event`: exact `deep_capture.proxy_started`.
- `proxy_event_count`: exact integer `1`.
- `proxy_backend`: exact `fragcap-native`.
- `proxy_address_family`: `ipv4` or `ipv6`, derived from an exact loopback address.
- `proxy_port_valid`: exact `true` after a port in 1 through 65535 is observed.
- `phase_event`: exact `deep_capture.calibration_phase`.
- `terminal_event_count`: exact integer `1`.
- `phase`: exact `reachability`.
- `launch_case`: exact `direct-exe-warm` for the controlled package target.
- `routing_strategy`: exact `child-environment`.
- `protocol`: exact `routing`.
- `stage`: exact `complete`.
- `status`: exact `reached-client`.

Exactly one startup and one terminal reached-client event must exist. Their `session_id` values must match before hashing.

## PredicateDiagnostic

- A closed stable identifier naming one failed invariant.
- At most sixteen identifiers are emitted in deterministic contract order.
- The combined message is at most 1 KiB.
- No raw event field or host observation value is included.

## Legacy compatibility

- Schema 2 retains the single historical smoke object and historical rules.
- Schema 3 retains two historical smoke rows and requires its original socket-observed positive authority.
- Schema 4 uses deterministic structured evidence and diagnostic socket observations.
- Unknown versions and cross-version field mixtures fail closed.
