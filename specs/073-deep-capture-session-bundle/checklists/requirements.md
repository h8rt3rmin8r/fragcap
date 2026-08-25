# Requirements Checklist: Deep Capture session bundle

**Purpose**: Validate the design before future writer and doctor implementation.

**Created**: 2026-08-25

## Content Quality

- [X] Bundle artifacts are named without implementing writers.
- [X] The design distinguishes artifact authority from artifact presence.
- [X] Sensitive and secret-adjacent material is labeled.
- [X] Unsupported or omitted artifacts require reasons.
- [X] Examples use placeholders and contain no local PII.

## Requirement Completeness

- [X] `.fcapng`, JSONL, HAR, TLS key log, proxy log, process trace, compatibility updates, cleanup report, and manifest are covered.
- [X] HAR behavior is defined for both Capture and Deep Capture.
- [X] Correlation anchors are named.
- [X] Doctor cleanup consumers have stable resource names.
- [X] The MVP dependency chain is recorded.
