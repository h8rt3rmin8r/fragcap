# Steam launcher proxy inheritance research

**Status:** measurement protocol for issue #215.\
**Date:** 2026-08-24.\
**Audience:** maintainers, reviewers, future Deep Capture slice authors.

This document defines how fragcap will measure whether target-scoped proxy routing works across Steam and publisher-launcher handoffs, and how to avoid confusing observed routing with proven environment inheritance. It does not implement Deep Capture, add a proxy backend, or require any trust-store change. Its job is to turn the launch-path uncertainty into committable, privacy-preserving evidence.

## Decision

The repository will record only scrubbed derivative findings for this research. Raw captures, process logs, socket logs, command lines, installed-title names, local paths, account identifiers, tokens, IP addresses, hostnames, and private notes stay outside git under ignored capture directories.

Every measured title receives a neutral alias before any artifact is written for publication:

| Private fact | Committable replacement |
| --- | --- |
| Real game title | `title-a`, `title-b`, `title-c` |
| Steam app id | `steam-app-a`, `steam-app-b`, `steam-app-c` |
| Install directory | `<install-root-a>`, `<install-root-b>` |
| Operator profile path | `<user-profile>` |
| Local machine name | `<host>` |
| Account id, session id, ticket, token, nonce | `<redacted-secret>` |
| Public IP address or hostname | `<remote-endpoint-a>`, `<remote-endpoint-b>` |
| Private LAN address | `<local-address>` |

A private alias map may exist locally inside `captures/recon/` while the work is active, because that directory is gitignored. It must not be copied into `docs/`, `specs/`, `changelog.d/`, issue bodies, PR bodies, or review comments.

## Public documentation baseline

Public documentation answers the operating-system and platform primitives, but it does not answer the runtime compatibility question.

Windows documents that each process has an environment block and that child processes inherit the parent's environment by default. `CreateProcess` can also receive an explicit environment block, which means a launcher can preserve, alter, or replace inherited variables at process creation time.

Valve documents Steam protocol launch forms such as `steam://run/` and `steam://rungameid/`, and Steamworks documents that applications define launch options as an executable path plus optional arguments. Steam Support documents user-facing launch options in the Steam client.

Those facts explain why inheritance can work, but they do not prove what a specific Steam title, publisher launcher, anti-cheat launcher, or crash-and-relaunch path does. The only reliable answer for Deep Capture is measured behavior on the local machine, recorded as sanitized facts.

## Research question

For each launch path, answer this question:

Did traffic from the relevant socket-owning process route through the proxy supplied by fragcap, and is there separate evidence that launch-scoped proxy configuration propagated to that process?

The answer must be one of:

| Routing verdict | Meaning |
| --- | --- |
| `reached-client` | The final socket-owning client connected through the local proxy. |
| `launcher-only-routing` | A launcher or platform process used the proxy, but the socket-owning client did not. |
| `escaped-tree` | The socket-owning client was observed, but its launch path was not connected to the scoped process tree strongly enough to claim inheritance. |
| `no-proxy-traffic` | No measured process used the local proxy. |
| `not-applicable` | The launch path did not start the target or produced no relevant network traffic. |
| `inconclusive` | The run lacks enough evidence to choose one of the above. |

Each run also records a separate propagation finding:

| Propagation finding | Meaning |
| --- | --- |
| `confirmed` | Independent non-invasive evidence shows the proxy configuration reached the socket-owning process. |
| `not-confirmed` | Routing evidence exists, but propagation itself was not independently proved. |
| `not-tested` | The run did not attempt to measure propagation separately from routing. |

An inconclusive verdict is acceptable. A guessed positive is not. A routing verdict is not an inheritance verdict unless propagation is independently confirmed.

## Measurement matrix

Measure these launch paths with aliases rather than real title names:

| Case | Steam state | Invoked path | Expected value |
| --- | --- | --- | --- |
| `steam-protocol-warm` | Steam already running | `steam://rungameid/<app>` | Baseline for managed launch when the platform client is already alive. |
| `steam-protocol-cold` | Steam not running | `steam://rungameid/<app>` | Determines whether proxy routing works after a cold Steam start, and whether separate propagation evidence exists. |
| `direct-exe-warm` | Steam already running | Installed executable | Measures direct executable behavior when platform services already exist. |
| `direct-exe-cold` | Steam not running | Installed executable | Measures the suspected crash, Steam start, and relaunch path. |
| `publisher-launcher` | Steam already running | Steam-managed title with launcher chain | Measures an extra handoff layer. |
| `final-owner-differs` | Any | Invoked executable differs from socket owner | Measures whether proxy routing works for the process that matters, not merely the first process. |

The same title may cover more than one case. The report should prefer a small number of carefully measured aliases over a large matrix of weak observations.

## Evidence model

Each run has three evidence levels.

| Level | Storage | Contents | Commit status |
| --- | --- | --- | --- |
| Raw | `captures/recon/<session>/` | Captures, socket logs, ETW process logs, local proxy logs, raw command lines, raw endpoints | Never committed |
| Private summary | `captures/recon/<session>/PRIVATE.md` | Real title mapping, operator notes, raw path notes, unresolved sensitive context | Never committed |
| Public finding | `docs/plans/recon/proxy-inheritance-YYYY-MM-DD.md` | Alias-only derivative table, verdicts, sanitized timelines, no raw endpoints | Committable |

The public finding is the only artifact expected to land in the repository.

## Run card

Each measured run should be reduced to this committable shape:

| Field | Value |
| --- | --- |
| Session alias | `run-001` |
| Title alias | `title-a` |
| Platform alias | `steam-app-a` |
| Launch case | One value from the measurement matrix |
| Steam pre-state | `running`, `not-running`, or `unknown` |
| Invoked alias | `launcher-a`, `client-a`, `platform-protocol` |
| Final socket owner alias | `client-a`, `launcher-a`, `platform-service-a`, or `unknown` |
| Observed ancestry | Alias-only chain, with PIDs omitted unless needed as local run labels |
| Proxy listener | `loopback:<port-alias>` |
| Proxy traffic observed | `yes`, `no`, or `partial` |
| Relevant sockets observed | `yes`, `no`, or `unknown` |
| Routing verdict | One value from the routing verdict taxonomy |
| Propagation finding | `confirmed`, `not-confirmed`, or `not-tested` |
| Confidence | `observed`, `inferred`, or `inconclusive` |
| Product consequence | `supported`, `unsupported`, `needs fallback`, or `needs more data` |

PIDs may appear in a public finding only as run-local labels such as `p1`, `p2`, and `p3`. Real PIDs are not stable and add little value. If a PID is needed to explain image-name recurrence, map it before publication.

## Sanitized process timeline

The public process timeline records shape, not identity:

| Order | Process alias | Parent alias | Role | Created during run | Held relevant sockets | Used proxy |
| ---: | --- | --- | --- | --- | --- | --- |
| 1 | `platform-a` | `shell` | platform | no | yes | unknown |
| 2 | `launcher-a` | `platform-a` | launcher | yes | yes | yes |
| 3 | `client-a` | `launcher-a` | client | yes | yes | no |

Do not include command-line arguments in public findings. If an argument shape matters, describe it structurally, for example: "client received a launch ticket argument, value redacted." Do not preserve argument names if the name itself reveals a service, account system, or title.

## Proxy proof

This research should prove Deep Capture compatibility by behavior. It should not claim environment inheritance unless there is independent non-invasive propagation evidence.

Acceptable routing proof:

- A local proxy listener records a CONNECT or HTTP request from a process later identified as the socket-owning client.
- Socket ownership shows the client connecting to the local proxy address and port.
- A process timeline shows a launcher used the proxy, then a later client opened non-proxy external sockets, supporting `launcher-only-routing`.

Acceptable propagation proof:

- A test wrapper or controlled helper launched through the same path reports receipt of the proxy variables without reading another process.
- A target-controlled diagnostic mode reports its own proxy configuration in a user-visible or log-visible way.
- A launcher records, in its own public log or documented diagnostics, that it passed proxy configuration to the child process.

Insufficient proof:

- The first invoked process received environment variables.
- Steam was launched from a shell with proxy variables.
- The final client is a descendant of a process that should have inherited the environment.
- A packet capture contains encrypted traffic, but no proxy connection.
- The client opens direct external sockets after a launcher used the proxy. That proves only that client routing did not use the proxy; it does not prove whether the client failed to inherit proxy variables, inherited them but ignored them, or used a non-proxyable protocol such as UDP or QUIC.

The product outcome that matters is whether the socket-owning process routed traffic through the proxy. The implementation reason may still be inheritance, proxy-awareness, protocol choice, or target behavior. Public findings must keep those apart.

## Sensitive-data handling

Before any artifact is committed, run the following manual checks:

- No real game title names.
- No Steam app ids.
- No publisher names tied to a measured title.
- No local usernames, profile paths, machine names, volume labels, or install roots.
- No public IP addresses, hostnames, DNS names, or URLs.
- No account identifiers, session identifiers, one-time tokens, tickets, cookies, nonces, or bearer-like values.
- No raw command lines.
- No capture files, socket logs, process logs, proxy logs, or HAR files.

If a fact cannot be made useful without revealing one of these values, keep it private and record only the product consequence.

## Findings report shape

The final committable report for this issue should use this outline:

```markdown
# Steam proxy inheritance findings, YYYY-MM-DD

**Status:** scrubbed derivative findings for issue #215.\
**Date:** YYYY-MM-DD.\
**Audience:** maintainers and Deep Capture slice authors.

## Scope

Describe how many aliased titles and launch paths were measured. Do not name titles.

## Verdicts

| Title alias | Launch case | Routing verdict | Propagation finding | Confidence | Product consequence |
| --- | --- | --- | --- | --- | --- |

## Findings

One subsection per title alias, with sanitized timelines and proxy proof.

## Compatibility facts proposed

List the local SQLite facts these observations should later store.

## Open questions

List anything unresolved without guessing.
```

## Compatibility facts to feed issue #217

This issue should produce local-store requirements, even before the store is implemented:

| Fact | Values |
| --- | --- |
| `deep_capture.proxy_routing` | `reached-client`, `launcher-only-routing`, `escaped-tree`, `no-proxy-traffic`, `inconclusive` |
| `deep_capture.proxy_propagation` | `confirmed`, `not-confirmed`, `not-tested` |
| `deep_capture.launch_case` | One value from the measurement matrix |
| `deep_capture.final_socket_owner` | Executable alias in public docs, real executable in local store |
| `deep_capture.publisher_launcher_present` | `yes`, `no`, `unknown` |
| `deep_capture.requires_steam_running` | `yes`, `no`, `unknown` |
| `deep_capture.direct_exe_supported` | `yes`, `no`, `unknown` |
| `deep_capture.steam_protocol_supported` | `yes`, `no`, `unknown` |
| `deep_capture.proxy_variables_tested` | Set of variables tested |
| `deep_capture.evidence_time` | Timestamp |
| `deep_capture.evidence_fragcap_version` | Version or commit |

The public findings should use aliases. The future local SQLite store may keep real title and executable facts because it lives on the user's machine rather than in the public repository.

## Acceptance criteria for issue #215

- A scrubbed findings report exists under `docs/plans/recon/`.
- The report uses title aliases and contains no actual measured game titles.
- Every verdict names the evidence used to reach it.
- At least one warm Steam protocol path and one cold Steam protocol path are measured.
- At least one direct executable path is measured.
- At least one publisher-launcher path is measured, if a suitable local title is available.
- The report identifies at least one reliable Deep Capture launch path, or states that none was confirmed.
- Unsupported paths are explicitly reported rather than converted into a recommendation for system-wide proxy settings.
- Proposed compatibility facts are ready for issue #217.

## Source notes

- Microsoft Windows environment variables: <https://learn.microsoft.com/en-us/windows/win32/procthread/environment-variables>
- Microsoft `CreateProcess`: <https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-createprocessa>
- Valve Steam browser protocol: <https://developer.valvesoftware.com/wiki/Steam_browser_protocol>
- Steamworks upload and launch-option setup: <https://partner.steamgames.com/doc/sdk/uploading>
- Steam Support launch options: <https://help.steampowered.com/en/faqs/view/7D01-D2DD-D75E-2955>
