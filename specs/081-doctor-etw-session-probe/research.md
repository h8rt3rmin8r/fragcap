# Research: Doctor ETW Session Probe

## Issue Evidence

Issue #204 identifies `crates/fragcap-cli/src/doctor/probe.rs:tracing_availability` as calling `EtwWatcher::start("fragcap-doctor-probe")` and then immediately stopping it. `EtwWatcher::start` starts an ETW session, opens a consumer, spawns a thread running `ProcessTrace`, and takes a process snapshot. Doctor needs only the boolean runtime answer that the session can open and enable the process provider.

## Prior Slice Context

S079 added interactive progress and hidden timings to make slow doctor probes visible. S080 removed the duplicate npcap device-list enumeration. S081 continues the same doctor performance chain and must preserve S079's stable report-surface contract.

## Relevant Existing Boundary

`Session::start` is private to the ETW module and already starts the session and enables the process provider. Dropping `Session` stops the provider and the trace session. That is the correct implementation primitive for a probe-only watcher-level entry point.

## Constraints

- Do not export raw ETW session internals to the CLI.
- Do not replace runtime readiness with a compile-time feature check.
- Do not alter `EtwWatcher::start` capture semantics.
- Do not add dependencies.
- Do not claim a timing improvement without local evidence.

## Open Research Questions

No open research question blocks implementation. Local platform checks will determine how much before/after timing and `logman query -ets` evidence can be recorded in the slice.
