# fragcap v0.1.0 Technical Specification (Outline)

**Status:** Draft outline for review, not the specification itself \
**Audience:** Human-facing (operator review, later contributor reference) \
**Author:** Drafted with Claude Opus 4.5, owned by William Thompson \
**Date:** 2026-08-06 \
**Repository:** `github.com/h8rt3rmin8r/fragcap` (planned) \
**License:** Apache-2.0

This outline enumerates every section the v0.1.0 specification will
contain, with a short annotation describing that section's job and the
decisions it must resolve. Annotations are scaffolding for review and do
not survive into the specification itself. Sections marked **(gate)**
block downstream work until resolved; sections marked **(deferrable)**
may ship as a stub in v0.2.0.

## Table of Contents

- [1. Document Control](#1-document-control)
- [2. Purpose and Problem Statement](#2-purpose-and-problem-statement)
- [3. Goals, Non-Goals, and Success Criteria](#3-goals-non-goals-and-success-criteria)
- [4. Glossary and Terminology Policy](#4-glossary-and-terminology-policy)
- [5. Domain Background](#5-domain-background)
- [6. Constraints and Assumptions](#6-constraints-and-assumptions)
- [7. Functional Requirements](#7-functional-requirements)
- [8. System Architecture](#8-system-architecture)
- [9. Platform Strategy](#9-platform-strategy)
- [10. Process Detection and Lifecycle](#10-process-detection-and-lifecycle)
- [11. Flow Attribution](#11-flow-attribution)
- [12. Capture Pipeline](#12-capture-pipeline)
- [13. Output Formats](#13-output-formats)
- [14. Sinks and Streaming](#14-sinks-and-streaming)
- [15. Game Profiles](#15-game-profiles)
- [16. Steam Integration](#16-steam-integration)
- [17. Command Line Interface](#17-command-line-interface)
- [18. Shell Wrappers](#18-shell-wrappers)
- [19. Security Posture and Anti-Cheat Interaction](#19-security-posture-and-anti-cheat-interaction)
- [20. Licensing and Third-Party Obligations](#20-licensing-and-third-party-obligations)
- [21. Repository Layout](#21-repository-layout)
- [22. Documentation System](#22-documentation-system)
- [23. Website and Brand](#23-website-and-brand)
- [24. Build, CI, and Release](#24-build-ci-and-release)
- [25. Testing Strategy](#25-testing-strategy)
- [26. Observability and Diagnostics](#26-observability-and-diagnostics)
- [27. Spec Kit Decomposition](#27-spec-kit-decomposition)
- [28. Roadmap Beyond v0.2.0](#28-roadmap-beyond-v020)
- [29. Open Questions](#29-open-questions)
- [30. Appendices](#30-appendices)

## 1. Document Control

Version, date, author, revision history, and the definition of what
"v0.1.0 specification" governs. States explicitly that this document is
the input to the Spec Kit constitution and feature slices rather than a
replacement for them, so that a reader arriving from `specs/` knows the
relationship between the two artifacts.

## 2. Purpose and Problem Statement

The one-paragraph statement of what fragcap is, followed by the problem
framing. The core assertion to defend here: packet capture is a solved
problem and attribution is not. Standard tooling captures at the network
driver, below the socket layer, where the association between a packet
and the process that produced it has already been discarded. fragcap
exists to reconstruct that association reliably for game clients that
are launched indirectly through platform and publisher launchers.

Also states the learning-oriented motivation honestly, since it shapes
priorities: this is a harness for observing where network theory and
shipped game networking diverge.

## 3. Goals, Non-Goals, and Success Criteria

Goals as testable statements rather than aspirations. Non-goals matter
more than usual here and deserve equal prominence:

- No process injection, memory reading, or hooking of the target.
- No packet modification, injection, or replay against a live server.
- No game-specific protocol logic in core (dissectors are a plugin
  seam, not a core concern).
- Not a cheat, not a proxy, not a latency optimizer.

Success criteria for the functional releases should be concrete enough
to test, for
example: capture a complete ESO session from launcher start through
client exit, with every packet correctly attributed to a named role, and
the resulting file opening cleanly in an unmodified Wireshark.

## 4. Glossary and Terminology Policy

**(gate)** Defines the glossary as a first-class deliverable with an
enforced contract rather than an appendix. Specifies the per-entry
template (blurb, detail paragraph, "why it matters here", see-also,
references), the category taxonomy, the cross-linking requirement, the
primary-source preference for references, and the CI linter that
enforces all of it. Also states the rule that no term may appear in any
other project document without a glossary entry existing first.

## 5. Domain Background

Enough shared context that a contributor from any one of the five
constituent domains can follow the rest of the document. Written as
prose with heavy glossary cross-linking, covering the capture stack on
Windows, the socket-table attribution problem, the platform-launcher
process chain, and what modern game anti-cheat does and does not
observe. This section is where the project earns the right to make
architectural claims later.

## 6. Constraints and Assumptions

Hard constraints (Windows-first, npcap dependency, administrative
privilege requirement, no redistribution of npcap) separated from
working assumptions that may prove false (game traffic is attributable
by 5-tuple, sessions are long-lived enough for poll-based attribution,
focal titles do not tunnel gameplay through a relay service). Each
assumption gets a validation method and a fallback, because several of
them are load-bearing.

## 7. Functional Requirements

Numbered, testable requirements grouped by capability. The two headline
modes from the original brief, plus the additions agreed during design:

- **FR-1 Bounded capture.** Detect target, capture for a bounded
  window, write an attributed capture file.
- **FR-2 Live streaming.** Capture and stream to a socket with optional
  directional filtering, consumable by arbitrary downstream processes.
- **FR-3 Ring capture.** Maintain a rolling in-memory window and dump on
  trigger. Added because fixed-duration capture requires predicting when
  the interesting event occurs.
- **FR-4 Role-aware capture.** Distinguish launcher, client, and
  platform-service traffic, and allow selecting a subset.
- **FR-5 Managed launch.** Start the target through fragcap so the
  detector is attached before the process chain begins.

## 8. System Architecture

The crate topology, the trait seams, and the data flow. The central
architectural claim to state and justify: `PacketSource` and
`FlowAttributor` are independent traits because capture and attribution
have different platform requirements, different failure modes, and
different upgrade paths. A Mermaid diagram of the pipeline belongs here.

Covers the facade crate, the dependency direction rule (core depends on
nothing platform-specific), the leaf `fragcap-proxy` crate and its bounded
native runtime, the public Deep Capture prepared-plan and session
coordinator, its immutable routing strategy and synchronized recovery journal,
its narrow effect adapters, and the plugin seam for dissectors.

The native HTTP and TLS conformance gate is a closed versioned matrix. It binds
each required protocol row to independent peer implementations, exact expected
and observed outcomes, executable evidence, production artifact authorities,
and a required CI tier. Missing or skipped required rows fail, and unmodified
TShark consumption is a separate analyzer proof.

## 9. Platform Strategy

Windows 11 is the v0.2.0 target. Documents why the capture binary must
be a native `x86_64-pc-windows-msvc` build and why WSL2 cannot host it
(NAT'd virtual NIC, and no access to Windows PIDs or socket tables from
a Linux kernel, which makes attribution structurally impossible rather
than merely inconvenient). Linux and macOS are named as future targets
with their intended backends sketched so the trait boundaries are not
accidentally Windows-shaped.

## 10. Process Detection and Lifecycle

**(gate)** The launcher-chain problem and its resolution. Covers why
polling and parent-chain walking are both inadequate on Windows (PPID
recycling, and launcher handoffs that complete faster than a poll
interval), and specifies ETW `Microsoft-Windows-Kernel-Process` as the
mechanism for building a creation-time process tree. Defines the
lifecycle states a tracked process moves through and how transient
launchers and persistent platform services are handled differently.

## 11. Flow Attribution

The join between captured packets and owning processes. Specifies the
v0.2.0 mechanism (periodic socket-table snapshots via `netstat2`), the
known race window, why that window is acceptable for long-lived game
sessions, and the upgrade path if it proves otherwise. Defines the
`FlowKey` type, the handling of unattributed packets (retained and
marked, never silently dropped), and the retention policy for closed
flows so that late-arriving packets still resolve.

## 12. Capture Pipeline

Interface selection, the dynamic BPF strategy (start coarse, tighten
once the flow set is known, re-tighten on change), buffer sizing, and
the backpressure policy. Backpressure needs an explicit, documented
decision rather than an emergent one: bounded ring, drop-oldest,
counted, and surfaced. Silent drops in a capture tool are indefensible.

Also covers loopback capture, since the launcher-to-client token handoff
and platform service chatter are local and invisible on a normal
adapter.

## 13. Output Formats

**(gate)** Defines `.fcapng`, the extended pcapng profile carrying
process attribution in Enhanced Packet Block options. States the design
principle that unmodified tools must read the file as ordinary pcapng
and ignore the annotations, so compatibility is never traded for
richness. Specifies the exact option encoding, the Interface Statistics
Block usage for drop accounting, and the timestamp anchoring scheme for
correlating captures with external event logs.

Second format: JSON Lines, for scripting and downstream analysis where
pcapng tooling is overkill.

## 14. Sinks and Streaming

The transport abstraction covering named pipes, Unix domain sockets, and
TCP, with the rationale for supporting all three (named pipe for
host-local Wireshark, TCP for containerized or remote consumers). Covers
the Wireshark extcap integration, which lets fragcap appear inside
Wireshark's own interface picker, and the directional filtering options
from the original brief.

## 15. Targets And Compatibility Evidence

The single local target store, source and fidelity model, resolution order, discovery signatures, and append-only Deep Capture compatibility facts. Unknown targets can produce initial evidence only through an explicit compatibility calibration, while `targets show` remains read-only and never selects an aggregate verdict.

## 16. Steam Integration

Scoped to its own crate so core stays platform-neutral. Covers library
folder and app manifest parsing, installed-title enumeration, profile
scaffolding from an installed game, and managed launch via either the Steam
protocol handler or one exact stored direct executable. Direct launch retains
its path, working directory, argument vector, and child-only environment before
effects; Steam environment handoff remains compatibility-dependent.

## 17. Command Line Interface

Full command surface, argument grammar, exit-code contract, and the machine-readable output mode that makes thin wrappers possible. Includes Capture, the library-backed Deep Capture adapter, the two-phase compatibility calibration plan and confirmation contract, target management, doctor, and integration commands.

## 18. Shell Wrappers

**(deferrable)** Bash and PowerShell 7 wrappers built to the house
scripting standards. States the scope discipline explicitly: wrappers
handle privilege checks, npcap presence detection, interface
enumeration, path translation, and output templating. They contain no
parsing or capture logic. Any wrapper that grows parsing logic indicates
a missing feature in core.

Notes the WSL2 interop case, where a Bash wrapper invokes the Windows
binary and translates paths across the boundary.

## 19. Security Posture and Anti-Cheat Interaction

Framed as engineering constraint rather than policy. Documents the mode-specific allowlist and absolute denylist. Capture remains passive. Deep Capture and compatibility calibration are explicit, target-scoped, plan-visible, confirmed, reversible, and audited local proxy inspection with no system-proxy fallback or pinning bypass. Managed direct launch creates an explicit child with scoped environment but performs no target inspection or memory access.

## 20. Licensing and Third-Party Obligations

**(gate)** Apache-2.0 for the project, with the NOTICE file convention
and the SPDX header policy. The critical item: **npcap is not
redistributable**. Its license permits free use but restricts bundling,
so fragcap must detect npcap rather than ship it, and installation
guidance must be a documented prerequisite. This constraint shapes
packaging, CI, and the getting-started flow, so it belongs in the
specification rather than buried in a README.

Also covers the dual-license question, since Rust ecosystem convention
is `MIT OR Apache-2.0` and choosing Apache-2.0 alone is a deliberate
deviation worth recording.

## 21. Repository Layout

The public monorepo structure covering the Rust workspace, the web
application, documentation content, profiles, scripts, and Spec Kit
directories. States the tooling decision (Cargo workspace plus a package
manager workspace for the web side, coordinated by a task runner) and
the reasoning for not reaching for a heavier monorepo orchestrator at
this scale.

Documents the separation between documentation content and web
application source, so docs remain editable without touching the site.

## 22. Documentation System

The documentation framework choice, the information architecture, the
glossary implementation, search configuration, and the local development
workflow. Also specifies the documentation linter: entry completeness,
internal anchor resolution, external link liveness on a schedule, and
regeneration of the alphabetical index.

## 23. Website and Brand

**(deferrable)** Site structure, hosting, custom domain configuration,
and the brand guardrails. Brand identity is resolved (the approved kit is
vendored in `brand/`); the guardrails belong in the specification because
they constrain the site build.

## 24. Build, CI, and Release

Toolchain pinning, the workflow set, the Windows runner requirements for
capture-dependent tests, artifact production, the crates.io publishing
order for a multi-crate workspace, and versioning policy. Also covers
site deployment and the custom-domain configuration.

## 25. Testing Strategy

**(gate)** The replay and scripted seams that keep the packet pipeline offline-testable, plus the controlled Deep Capture target that proves calibration orchestration, local fact persistence, bundle contracts, and cleanup without a game account, remote service, or real trust mutation. Real launcher inheritance remains a manual evidence tier.

## 26. Observability and Diagnostics

Structured lifecycle events, human progress, capture statistics, calibration phase outcomes, and the diagnostic command that reports Capture and Deep Capture readiness, exact CA trust state, target stores, interfaces, ETW, proxy support, and owned cleanup residue.

## 27. Spec Kit Decomposition

The mapping from this specification into the agentic workflow. Names
which sections become constitution principles (the non-goals, the
technique denylist, the dependency direction rule, the glossary
requirement) and which become numbered feature slices, with a proposed
slice ordering and the dependency edges between slices. This section is
what turns the specification into executable work.

## 28. Roadmap Beyond the Current Release

Explicitly deferred capability and the native Deep Capture completion
contract. Issue #278 owns four ordered milestones. S102 establishes the owned
`fragcap-proxy` leaf and bounded runtime. S103 completes authenticated admission,
upstream policy, certificate and trust ownership, raw observations, and the
controlled protocol lab. S104 adds native HTTP/1.1, CONNECT, and HTTPS and
removes the production external proxy path. S105 adds native HTTP/2,
protocol-faithful metadata, bounded streaming bodies and decoding, and the live
version 2 application stream. S106 adds WebSocket frame and message evidence,
incremental Server-Sent Events, and schema-free gRPC envelope evidence. S107
adds proxy-owned TLS key logs, explicit upstream client identities, stable TLS
refusal evidence, and protected sensitive-artifact cleanup and sharing. S108
adds final packet/process correlation, bounded evidence-derived HAR 1.2, and
manifest version 2 artifact authority. S109 adds immutable target-scoped routing
and crash recovery, S110 closes native HTTP and TLS conformance, and S111 adds
exact cold publisher-launcher chains. Broader launch and transport coverage,
packaging, independent review, and the final #334 gate remain open. Additional
platforms, richer attribution backends, dissector
plugins, platform integrations, and analysis tooling remain deferred.

## 29. Open Questions

Tracked unknowns with owners and resolution methods. S100's two turnkey
candidate results remain historical evidence, but S102 supersedes their external
end state after native completion became an explicit requirement. The owned
Tokio/Hyper/rustls/rcgen path uses a measured Rust 1.88 floor, bounded task
ownership, the S103 secure foundation, and S104's native production cutover.
The section also retains the crate-naming reservation and reconnaissance
findings.

## 30. Appendices

- Appendix A: Crate and module inventory.
- Appendix B: Profile schema reference.
- Appendix C: `.fcapng` option encoding reference.
- Appendix D: Reconnaissance findings for focal titles.
- Appendix E: Referenced standards and specifications.
