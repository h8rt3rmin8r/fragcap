# Steam proxy inheritance findings, 2026-08-24

**Status:** scrubbed derivative findings for issue #215.\
**Date:** 2026-08-24.\
**Audience:** maintainers and Deep Capture slice authors.

## Scope

Five alias-only launch-path measurements were reduced from private local Steam runs. Two aliased titles were used: one baseline Steam title and one publisher-launcher candidate. The raw title names, paths, app ids, process names, command lines, endpoints, and screenshots remain in the ignored private workspace.

## Verdicts

| Title alias | Launch case | Routing verdict | Propagation finding | Confidence | Product consequence |
| --- | --- | --- | --- | --- | --- |
| title-a | steam-protocol-warm | inconclusive | not-tested | inconclusive | needs more data |
| title-y | publisher-launcher | not-applicable | not-tested | observed | needs more data |
| title-a | steam-protocol-warm | no-proxy-traffic | not-confirmed | observed | unsupported |
| title-a | direct-exe-warm | reached-client | not-confirmed | observed | supported |
| title-a | steam-protocol-cold | reached-client | not-confirmed | observed | supported |

## Findings

### title-a, steam-protocol-warm

| Field | Value |
| --- | --- |
| Session alias | run-001 |
| Platform alias | steam-app-a |
| Steam pre-state | unknown |
| Invoked alias | platform-protocol |
| Final socket owner alias | unknown |
| Observed ancestry | shell -> platform-a -> client-a |
| Proxy listener | loopback:port-a |
| Proxy traffic observed | no |
| Relevant sockets observed | no |

Evidence:
- Target alias process count: 1.
- Target socket-table events: 0.
- Local proxy accepted connection count: 0.
- Target socket events touching the proxy listener: 0.
- Operator corrected this run as not a clean warm case because the platform client started in the background during the attempt.

### title-y, publisher-launcher

| Field | Value |
| --- | --- |
| Session alias | run-002 |
| Platform alias | steam-app-y |
| Steam pre-state | running |
| Invoked alias | platform-protocol |
| Final socket owner alias | launcher-a |
| Observed ancestry | shell -> platform-a -> launcher-a |
| Proxy listener | loopback:port-a |
| Proxy traffic observed | no |
| Relevant sockets observed | yes |

Evidence:
- Target alias process count: 6.
- Target socket-table events: 41.
- Local proxy accepted connection count: 0.
- Target socket events touching the proxy listener: 0.
- Operator observed launcher network content failing to load and hanging without reaching a playable client state.
- Because a playable client state was not reached, this run is recorded as not applicable rather than as a title-level incompatibility verdict.

### title-a, steam-protocol-warm

| Field | Value |
| --- | --- |
| Session alias | run-003 |
| Platform alias | steam-app-a |
| Steam pre-state | running |
| Invoked alias | platform-protocol |
| Final socket owner alias | client-a |
| Observed ancestry | shell -> platform-a -> client-a |
| Proxy listener | loopback:port-a |
| Proxy traffic observed | no |
| Relevant sockets observed | yes |

Evidence:
- Target alias process count: 1.
- Target socket-table events: 53.
- Local proxy accepted connection count: 0.
- Target socket events touching the proxy listener: 0.
- Operator observed an unstable-network dialog and login retry loop.

### title-a, direct-exe-warm

| Field | Value |
| --- | --- |
| Session alias | run-004 |
| Platform alias | steam-app-a |
| Steam pre-state | running |
| Invoked alias | client-a |
| Final socket owner alias | client-a |
| Observed ancestry | shell -> client-a |
| Proxy listener | loopback:port-a |
| Proxy traffic observed | yes |
| Relevant sockets observed | yes |

Evidence:
- Target alias process count: 1.
- Target socket-table events: 94.
- Local proxy accepted connection count: 60.
- Target socket events touching the proxy listener: 94.
- Operator observed an unstable-network retry loop while the proxy listener accepted target-routed connections.

### title-a, steam-protocol-cold

| Field | Value |
| --- | --- |
| Session alias | run-005 |
| Platform alias | steam-app-a |
| Steam pre-state | not-running |
| Invoked alias | platform-protocol |
| Final socket owner alias | client-a |
| Observed ancestry | shell -> platform-a -> client-a |
| Proxy listener | loopback:port-a |
| Proxy traffic observed | yes |
| Relevant sockets observed | yes |

Evidence:
- Target alias process count: 1.
- Target socket-table events: 144.
- Local proxy accepted connection count: 90.
- Target socket events touching the proxy listener: 144.
- Operator observed the platform client restart, launch the target, and show the unstable-network dialog while proxy connections were accepted.

## Compatibility facts proposed

- deep_capture.proxy_routing: title-a direct-exe-warm reached-client; title-a steam-protocol-cold reached-client; title-a steam-protocol-warm no-proxy-traffic; title-y publisher-launcher not-applicable.
- deep_capture.proxy_propagation: not-confirmed for runs with routing evidence because routing was observed behaviorally but environment propagation was not independently proved; not-tested for run-001 and run-002.
- deep_capture.direct_exe_supported: yes for title-a when the platform client is already running.
- deep_capture.steam_protocol_supported: yes for title-a from a cold platform state; no clean warm-protocol success was confirmed in this set.
- deep_capture.publisher_launcher_present: yes for title-y; publisher-launcher path requires a client-reaching rerun before caching compatibility.
- deep_capture.requires_steam_running: no for title-a cold Steam protocol path in this set.
- deep_capture.proxy_variables_tested: `HTTP_PROXY`, `HTTPS_PROXY`, `ALL_PROXY`, `http_proxy`, `https_proxy`, and `all_proxy`, all pointing at the local loopback proxy listener for the run.
- deep_capture.evidence_fragcap_version: workspace version 0.6.0, measurement base commit `2b3c976`.

## Open questions

- Measure a clean warm Steam-protocol path again after fully resetting the title state, because run-003 showed target sockets but no proxy routing.
- Measure a direct-executable cold path separately, because run-001 was not a clean direct-executable cold case.
- Identify whether publisher-launcher candidates need a launcher-specific proxy method, because the measured publisher-launcher path opened launcher-side target sockets without reaching the local proxy.

## Sanitization record

- Source evidence was reduced to public aliases before writing.
- The generated report was scanned for common private-data patterns.
- Raw captures, logs, command lines, endpoints, and title names remain uncommitted.
