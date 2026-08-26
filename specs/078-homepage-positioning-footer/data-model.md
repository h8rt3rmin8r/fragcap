# Data Model: Homepage Positioning And Next-Command Footer

S078 introduces no persisted data. Its model is a set of presentation contracts over existing observations.

## NextCommandSuggestion

- `row`: Existing one-based listing row selected by readiness and install-presence precedence.
- `command`: `fragcap capture <row>`.
- `label`: Literal `Next command:`.
- `boundary`: Exactly one blank line before the labelled line.

### Invariants

- Selection is unchanged by S078.
- Exactly one suggestion exists for a populated listing.
- No suggestion exists for an empty listing.
- Bare invocation adds only its existing help footer after the suggestion.

## HomepagePositioning

- `outcome`: Process-attributed game traffic.
- `capture_mode`: Passive packet capture and process attribution.
- `deep_capture_mode`: Explicit target-scoped local proxy inspection for compatible targets.
- `mechanism`: Correlation of packet flows with separate socket and process-lifecycle observations.
- `limits`: Unresolved attribution and unsupported or uninspectable Deep Capture traffic remain valid outcomes.

### Prohibited Claims

- The operating system destroyed all ownership information.
- Every launcher chain has a fixed number of hops.
- Every flow is attributed.
- Every encrypted flow is inspectable or decryptable.
- The entire product is passive-only.

## DependencyGuidance

- `npcap`: Required for live packet capture in WinPcap-compatible mode; never bundled or redistributed.
- `wireshark`: Recommended analyzer, not a prerequisite.
- `doctor`: Readiness authority for Capture and Deep Capture.

## HomepageSpecimen

- `columns`: `TARGET`, `CAPTURE`, `ENGINE`, `SENSITIVITIES`.
- `rows`: Two synthetic target rows.
- `next_command`: Exact `NextCommandSuggestion` text.
- `privacy`: No actual title, account, host, filesystem, or endpoint data.
