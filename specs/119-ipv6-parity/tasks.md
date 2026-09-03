# Tasks: Complete IPv6 Parity

**Input**: Design documents from `specs/119-ipv6-parity/`

## Phase 1: Endpoint Contract

- [x] T001 Add failing exact IPv4/IPv6 endpoint, wildcard refusal, URL formatting, and CLI parsing tests
- [x] T002 Replace the port-only facade endpoint with one validated exact loopback socket
- [x] T003 Add the public IPv4/IPv6 CLI family selection and exact family-specific reservation
- [x] T004 Carry the exact endpoint through native bind, plan, routing, lifecycle, journal, controlled harness, and rendering

## Phase 2: Authority And Canonical Identity

- [x] T005 Add failing bracketed IPv6, numeric scope, malformed scope, and mapped identity tests
- [x] T006 Extend destination authority with scoped IPv6 socket construction and scope-free TLS identity
- [x] T007 Centralize observed-to-canonical socket normalization for policy, grants, SOCKS peer ownership, and correlation

## Phase 3: One-Winner Dual-Stack Connection

- [x] T008 Add failing deterministic candidate interleaving, stagger, timeout, cancellation, and one-winner tests
- [x] T009 Bound and canonically deduplicate resolved TCP and UDP candidates
- [x] T010 Implement the finite 250 ms staggered TCP race under one deadline and cancellation owner
- [x] T011 Expose exact selected peer and local socket facts from successful upstream streams

## Phase 4: IPv6 Transport Parity

- [x] T012 Add failing IPv6 HTTP, HTTPS, SOCKS, generic TCP, generic UDP, and QUIC controlled lab rows
- [x] T013 Generalize controlled protocol peers and SOCKS address encoders for exact IPv4 or IPv6 loopback
- [x] T014 Preserve exact family-bearing endpoints through application evidence, HAR, lifecycle, manifest, and correlation
- [x] T015 Add mapped-address and observation-order tests proving stable flow and socket ownership

## Phase 5: Doctor Readiness

- [x] T016 Add failing Doctor classifier, JSON, human, and probe tests for independent family readiness
- [x] T017 Add exact ephemeral IPv4 and IPv6 bind probes and separate readiness rows
- [x] T018 Update Doctor goldens and progress-safe integration expectations

## Phase 6: Documentation And Completion

- [x] T019 Update the master specification and outline with the exact listener, scoped-address, race, evidence, and readiness contracts
- [x] T020 Add glossary entries and update plans README, proxy README, AGENTS, issue #315 notes, and changelog fragment
- [x] T021 Run focused tests, formatting, clippy, text hygiene, dependency gates, and full `cargo xtask ci`
- [x] T022 Run convergence audit, resolve every finding, mark artifacts complete, and verify the final diff
