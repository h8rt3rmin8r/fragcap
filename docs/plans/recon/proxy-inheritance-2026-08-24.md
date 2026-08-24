# Steam proxy inheritance findings, 2026-08-24

**Status:** scrubbed derivative findings for issue #215.\
**Date:** 2026-08-24.\
**Audience:** maintainers and Deep Capture slice authors.

## Scope

Seven alias-only launch-path measurements were reduced from private local Steam runs. Two aliased titles were used: one baseline Steam title and one publisher-launcher candidate. The raw title names, paths, app ids, process names, command lines, endpoints, screenshots, and account-state observations remain in the ignored private workspace.

## Verdicts

| Title alias | Launch case | Routing verdict | Propagation finding | Confidence | Product consequence |
| --- | --- | --- | --- | --- | --- |
| title-a | steam-protocol-warm | no-proxy-traffic | not-confirmed | observed | unsupported |
| title-a | direct-exe-warm | no-proxy-traffic | not-confirmed | observed | unsupported |
| title-a | steam-protocol-cold | reached-client | not-confirmed | observed | supported |
| title-a | direct-exe-cold | reached-client | not-confirmed | observed | supported |
| title-y | publisher-launcher-warm | no-proxy-traffic | not-confirmed | observed | needs fallback |
| title-y | publisher-launcher-game-start-clean-warm | no-proxy-traffic | not-confirmed | observed | needs fallback |
| title-y | publisher-launcher-cold | launcher-only-routing | not-confirmed | observed | needs fallback |

## Findings

### title-a, steam-protocol-warm

| Field | Value |
| --- | --- |
| Session alias | run-006 |
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
- Target socket-table events: 45.
- Mitmproxy flow output: zero.
- Target socket events touching the proxy listener: 0.
- Operator observed successful login while no target-owned proxy-port sockets were recorded.

### title-a, direct-exe-warm

| Field | Value |
| --- | --- |
| Session alias | run-007 |
| Platform alias | steam-app-a |
| Steam pre-state | running |
| Invoked alias | client-a |
| Final socket owner alias | client-a |
| Observed ancestry | shell -> client-a |
| Proxy listener | loopback:port-a |
| Proxy traffic observed | no |
| Relevant sockets observed | yes |

Evidence:
- Target alias process count: 1.
- Target socket-table events: 31.
- Mitmproxy flow output: zero.
- Target socket events touching the proxy listener: 0.
- This real-proxy result differs from an earlier dummy-listener warm direct-executable run, so the dummy-listener evidence is retained only as a network-sensitivity warning.

### title-a, steam-protocol-cold

| Field | Value |
| --- | --- |
| Session alias | run-008 |
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
- Target socket-table events: 38.
- Mitmproxy flow output: nonzero.
- Target socket events touching the proxy listener: 4.
- Operator observed the platform client come up and login complete successfully from a cold platform state.
- No independent non-invasive propagation evidence was captured, so propagation remains not-confirmed.

### title-a, direct-exe-cold

| Field | Value |
| --- | --- |
| Session alias | run-009 |
| Platform alias | steam-app-a |
| Steam pre-state | not-running |
| Invoked alias | client-a |
| Final socket owner alias | client-a |
| Observed ancestry | shell -> client-a -> platform-a -> client-a |
| Proxy listener | loopback:port-a |
| Proxy traffic observed | yes |
| Relevant sockets observed | yes |

Evidence:
- Target alias process count: 2.
- Target socket-table events: 46.
- Mitmproxy flow output: nonzero.
- Target socket events touching the proxy listener: 8.
- The first target process touched the proxy listener, exited through the platform handoff, and the relaunched target process also touched the proxy listener.
- Operator observed pressing login close the first target process, start the platform client, and reopen the target to login again.
- No independent non-invasive propagation evidence was captured, so propagation remains not-confirmed.

### title-y, publisher-launcher-warm

| Field | Value |
| --- | --- |
| Session alias | run-010 |
| Platform alias | steam-app-y |
| Steam pre-state | running |
| Invoked alias | platform-protocol |
| Final socket owner alias | launcher-a |
| Observed ancestry | shell -> platform-a -> launcher-a |
| Proxy listener | loopback:port-a |
| Proxy traffic observed | no |
| Relevant sockets observed | yes |

Evidence:
- Target alias process count: 7.
- Target socket-table events: 52.
- Mitmproxy flow output: zero.
- Target socket events touching the proxy listener: 0.
- Operator observed publisher-launcher network content failing to load before the run ended.
- Operator observed that the game start progressed immediately after the proxy listener went down, which marks this path as proxy-sensitive.

### title-y, publisher-launcher-game-start-clean-warm

| Field | Value |
| --- | --- |
| Session alias | run-012 |
| Platform alias | steam-app-y |
| Steam pre-state | running |
| Invoked alias | platform-protocol |
| Final socket owner alias | launcher-a |
| Observed ancestry | shell -> platform-a -> launcher-a |
| Proxy listener | loopback:port-a |
| Proxy traffic observed | no |
| Relevant sockets observed | yes |

Evidence:
- Target alias process count: 7.
- Target socket-table events: 94.
- Mitmproxy flow output: zero.
- Target socket events touching the proxy listener: 0.
- The clean warm publisher retry used a longer window and still produced no target-owned proxy-port sockets.

### title-y, publisher-launcher-cold

| Field | Value |
| --- | --- |
| Session alias | run-013 |
| Platform alias | steam-app-y |
| Steam pre-state | not-running |
| Invoked alias | platform-protocol |
| Final socket owner alias | launcher-a |
| Observed ancestry | shell -> platform-a -> launcher-a |
| Proxy listener | loopback:port-a |
| Proxy traffic observed | yes |
| Relevant sockets observed | yes |

Evidence:
- Target alias process count: 7.
- Target socket-table events: 67.
- Mitmproxy flow output: nonzero.
- Target socket events touching the proxy listener: 25.
- Operator observed the platform client launch from cold state, the publisher launcher load slowly, and the manual play control become available before post-click capture completed.
- The final socket-owner alias for this run is launcher-a, so this run proves launcher-side proxy routing but does not prove client-owned proxy routing.

## Compatibility facts proposed

- deep_capture.proxy_routing: title-a steam-protocol-cold reached-client; title-a direct-exe-cold reached-client; title-y publisher-launcher-cold launcher-only-routing; title-a steam-protocol-warm no-proxy-traffic; title-a direct-exe-warm no-proxy-traffic; title-y publisher-launcher-warm no-proxy-traffic.
- deep_capture.proxy_propagation: not-confirmed for all runs because no independent non-invasive propagation evidence was captured; target-owned proxy-port sockets are routing evidence, not propagation proof.
- deep_capture.final_owner_differs: observed for title-a direct-exe-cold, where the first client process closed through a platform handoff and a later client process owned the final socket activity.
- deep_capture.publisher_launcher_present: yes for title-y; warm publisher launch paths showed proxy sensitivity and no target-owned proxy-port sockets, while cold publisher launch routed launcher-owned sockets to the proxy listener.
- deep_capture.requires_platform_cold_start_for_proxy: yes for the successful baseline client-routing measurements in this set; warm starts did not route target-owned sockets through mitmproxy.
- deep_capture.proxy_variables_tested: HTTP_PROXY, HTTPS_PROXY, ALL_PROXY, http_proxy, https_proxy, and all_proxy, all pointing at the local loopback mitmproxy listener for the run.
- deep_capture.proxy_backend_tested: mitmproxy 12.2.3 in regular mode with ignored TLS hosts, lazy upstream connection strategy, and loopback-only listener.
- deep_capture.evidence_fragcap_version: workspace version 0.6.0, measurement base commit cf21a1b.

## Open questions

- Determine whether warm platform sessions can be made to inherit proxy settings without restarting the platform client, or whether Deep Capture must explicitly require a managed cold platform launch for reliable routing.
- Determine why the dummy listener and real mitmproxy produced different warm direct-executable results before using either as a compatibility fact.
- Determine whether publisher-launcher warm paths require a launcher-specific fallback, a longer startup grace period, or an explicit cold-platform requirement.
- Promote final-owner handoff observations into the local compatibility database so the user does not need to rediscover them for the same title.
- Define the Deep Capture UX wording for proxy-sensitive launchers so users understand when restarting the platform client is expected behavior.

## Sanitization record

- Source evidence was reduced to public aliases before writing.
- The generated report was scanned for common private-data patterns.
- Raw captures, logs, command lines, endpoints, and title names remain uncommitted.
