# Data Model: Deep Capture MVP

## DeepCaptureInvocation

Represents the parsed command after clap validation and before side effects.

Fields:

- `target_ref`: selector or stable id for one stored target.
- `local_db`: optional local store path.
- `catalog_db`: optional catalog store path.
- `bundle_root`: optional output bundle root.
- `duration`, `wait`, `max_packets`, `max_bytes`, `interfaces`, and `no_payload`: capture controls reused from `CaptureArgs`.
- `trust`: requested trust behavior, such as confirm interactively, pre-confirm, or refuse without mutation.
- `proxy_backend`: selected backend, initially `mitmdump`.
- `emit_key_log`: whether to produce a proxy-owned analyzer key log.
- `emit_har`: whether to write HAR when HTTP semantics are observable.

Validation:

- Exactly one stored target reference is required.
- Raw process names are invalid for MVP Deep Capture.
- Non-interactive trust mutation requires an explicit pre-confirmed option.
- Unsupported sink shapes are refused until bundle output can account for them.

## DeepCapturePreflight

Represents the blocking decision made before launch.

Fields:

- `target_id`, `target_handle`, and optional platform anchor.
- `proxy_backend` and `proxy_backend_version`.
- `session_storage_state`.
- `trust_state`.
- `launch_case`.
- `routing_fact`.
- `propagation_fact`.
- `inspectability_fact`.
- `required_confirmations`.
- `blockers`.
- `warnings`.

Rules:

- Any missing backend, unsupported launch path, unknown scoped proxy propagation, required but unconfirmed trust mutation, or unsafe residue that blocks the requested port becomes a blocker.
- Unknown facts are blockers for real targets and permitted only for the controlled local harness.

## ProxyBackend

Represents an owned local proxy process.

Fields:

- `backend_name`: `mitmdump` for MVP.
- `backend_version`.
- `executable_path`: local path kept out of committed fixtures.
- `listen_addr`.
- `listen_port`.
- `ca_material_ref`.
- `key_log_path`.
- `event_stream`.
- `process_id`.
- `started_at`.
- `stopped_at`.
- `stop_status`.

Rules:

- The proxy is owned by the Deep Capture session.
- The proxy must bind only to local interfaces.
- Process exit, kill, timeout, and cleanup outcomes are recorded.
- The controlled adapter binds a real loopback listener and receives requests from a placeholder child process; it does not fabricate process identity.

## TrustState

Represents fragcap-owned CA material and trust visibility.

Fields:

- `ca_id`.
- `thumbprint`.
- `material_path`.
- `current_user_trusted`.
- `wrong_store_present`.
- `created_this_session`.
- `trusted_this_session`.
- `cleanup_policy`.

Rules:

- Trust is current-user scoped for MVP.
- Wrong-store or mismatched trust is a warning or blocker, never treated as success.
- No trust mutation occurs without explicit confirmation.

## DeepCaptureSession

Represents the running session coordinator.

Fields:

- `session_id`.
- `target_id` or stable target handle.
- `mode`: `deep-capture`.
- `bundle_root`.
- `proxy`.
- `trust`.
- `capture_config`.
- `started_at`.
- `stopped_at`.
- `stop_reason`.
- `cleanup_status`.

Rules:

- The same `session_id` appears in manifest, application JSONL, HAR metadata, proxy log, process trace, compatibility update sidecar, cleanup report, and status events.
- Session state is `complete`, `partial`, or `failed`; observations collected before a failure remain eligible for scrubbed local fact updates.

## ApplicationObservation

Represents one proxy-derived application-layer record.

Fields:

- `session_id`.
- `flow_id`.
- `proxy_connection_id`.
- `timestamp`.
- `direction`.
- `protocol`.
- `inspectability`: `full`, `metadata-only`, `unsupported`, or `unknown`.
- `process_id`.
- `process_image`.
- `role`.
- `method`, `url`, `status`, `headers`, `body_ref`, or omission fields when HTTP semantics are present.
- `reason` when metadata-only or unsupported.

Rules:

- Sensitive payload material is never printed to terminal status.
- URL, host, and body fields in committed fixtures must use local placeholder values only.

## CompatibilityObservation

Represents scrubbed facts written after a run.

Fields:

- `target_id`.
- `launch_case`.
- `proxy_routing`.
- `proxy_propagation`.
- `tls_trust_behavior`.
- `protocol_behavior`.
- `inspectability`.
- `final_socket_owner_role`.
- `proxy_backend`.
- `proxy_backend_version`.
- `proxy_mode`.
- `observed_at`.
- `note`.

Rules:

- Values must pass the closed vocabularies in `fragcap-targets/src/compatibility.rs`.
- Notes are optional and scrubbed before storage or export.

## ControlledTargetHarness

Represents the test target used to verify Deep Capture without a game account.

Fields:

- `launcher_image`: placeholder launcher name.
- `client_image`: placeholder client name.
- `http_endpoint`: loopback placeholder URL.
- `https_endpoint`: loopback placeholder URL.
- `trust_mode`.
- `expected_requests`.

Rules:

- Harness names and endpoints are synthetic.
- No remote service, user account, real title name, or captured third-party payload is used.
