# Feature Specification: CLI Reference Gate

**Feature Branch**: `codex/093-cli-reference-gate`

**Created**: 2026-08-28

**Status**: Approved for implementation

**Input**: User description: "Kick off S093", implementing GitHub issue #246.

## User Scenarios & Testing

### User Story 1 - Command Drift Fails Before Merge (Priority: P1)

As a maintainer, I want the public CLI reference checked against the command definition so a new, removed, or renamed command, subcommand, flag, allowed value, or declared default cannot merge while the website still describes the old surface.

**Why this priority**: Repeated drift is the defect this slice exists to prevent. Correcting today's page without a standing gate would leave the same failure mode intact.

**Independent Test**: Compare the complete public command definition with the reference contract, then introduce a synthetic unmatched command or flag and confirm the comparison reports the exact missing or stale item.

**Acceptance Scenarios**:

1. **Given** the current public command definition and CLI reference, **When** the documentation gate runs, **Then** every visible command path and flag agrees exactly and the gate passes.
2. **Given** a public command or flag exists in only one side, **When** the gate runs, **Then** it fails and names the command path plus the mismatched item.
3. **Given** an internal or parser-generated command or flag is intentionally excluded, **When** the gate runs, **Then** the exclusion is structural or carries a checked reason rather than relying on an unverified name list.

### User Story 2 - Operators Read the Shipped Surface (Priority: P2)

As an operator, I want one accurate v0.7.0 reference for every public command and option so I can compose an invocation without discovering stale schemes, required-looking override paths, or missing value constraints at runtime.

**Why this priority**: The current page is useful but incomplete. It omits accepted sink forms and modifiers, summarizes several nested commands without their complete option sets, and can imply that managed store paths are required.

**Independent Test**: Read the reference command by command and account for every visible option, its accepted enumerated values, its parser-declared default, and its availability without requiring a local store, capture driver, elevated privilege, or game.

**Acceptance Scenarios**:

1. **Given** the v0.7.0 command tree, **When** an operator opens the reference, **Then** each public command and subcommand appears once in a stable command section.
2. **Given** an option with an enumerated value set or parser-declared default, **When** the option is documented, **Then** the reference states the same values and default.
3. **Given** a store-path option, **When** it appears in the reference, **Then** it is described as an override of managed resolution rather than a prerequisite.
4. **Given** the sink grammar, **When** an operator reads `--sink`, **Then** every accepted scheme and modifier is named with the combinations and platform limits needed to avoid a rejected invocation.

### User Story 3 - Examples and JSON Routing Are Trustworthy (Priority: P3)

As an automation author, I want every worked invocation to pass command parsing and the global JSON routing rules to be explicit so I can distinguish command results, lifecycle events, capture bytes, diagnostics, and errors without scraping the wrong stream.

**Why this priority**: A command can be spelled correctly while its output is still routed differently than an automation expects. Both the grammar and stream contract must be accurate.

**Independent Test**: Extract every executable `fragcap` example from the reference, parse it without dispatching any command, and inspect the JSON-routing section for every command-result and capture-event class.

**Acceptance Scenarios**:

1. **Given** each executable reference example, **When** the no-side-effect parser validates it, **Then** it is accepted without running the command.
2. **Given** a malformed or retired command in an example, **When** the gate runs, **Then** it fails before any capture, store, network, trust, or process side effect occurs.
3. **Given** `--json`, **When** an operator reads the global option and output section, **Then** command results are assigned to standard output, capture and Deep Capture lifecycle events to standard error, capture bytes to sinks, and warnings or errors to diagnostic output.

### Edge Cases

- Hidden harness arguments and the hidden controlled-target command are excluded structurally and never become public documentation obligations.
- Parser-generated help and version controls are identified separately from authored public options so they neither disappear silently nor force duplicate rows on every command.
- Global options are documented once and are not counted as local options on every subcommand after propagation.
- Feature-gated public options are either verified in their enabled command tree or documented as excluded with the availability reason.
- A command with no local flags still requires its own command section and remains covered by the command-path gate.
- Positional arguments are allowed in worked examples and are validated by the parser even though the flag-set comparison covers named options.
- Quoted Windows paths, inline comments, and line continuations in worked examples must not cause the extractor to run a command or mis-tokenize an otherwise valid example.

## Requirements

### Functional Requirements

- **FR-001**: The repository MUST expose a deterministic documentation check that compares the public CLI reference with the runtime command definition.
- **FR-002**: The check MUST derive every public command and subcommand path recursively from the command definition rather than from a hand-maintained expected-command list.
- **FR-003**: The reference MUST identify each public command and subcommand exactly once in a machine-checkable, human-visible section.
- **FR-004**: The check MUST exclude hidden commands and hidden options structurally and MUST exclude parser-generated controls through one documented policy.
- **FR-005**: The check MUST compare every public named option on its owning command, including short aliases, while treating propagated global options as one root-level contract.
- **FR-006**: The check MUST fail with the owning command path and both sides of any command or option mismatch.
- **FR-007**: The check MUST compare every enumerated option value and parser-declared default with the reference entry for that option.
- **FR-008**: Feature-gated public options MUST be exercised under each documented availability variant or carry a checked exclusion reason.
- **FR-009**: The check MUST derive accepted sink schemes and modifier names from the sink parser source of truth and compare them with the `--sink` reference.
- **FR-010**: The reference MUST document the complete current sink scheme and modifier vocabulary plus the transport-specific constraints needed to interpret it.
- **FR-011**: The reference MUST describe every managed store path flag as an optional override and state the applicable environment or per-user fallback without presenting an owned store path as required.
- **FR-012**: The reference MUST state the complete global `--json`, `--quiet`, and `--silent` routing and suppression contract without implying that every JSON record uses the same stream.
- **FR-013**: The check MUST discover every executable `fragcap` invocation in the reference and validate it through parsing only, without dispatching the command.
- **FR-014**: The example validator MUST handle the quoting, comments, and line continuation forms used by the reference and MUST fail with the source line plus parser diagnostic.
- **FR-015**: The check MUST run through `cargo xtask docs check` and the ordinary repository CI aggregate.
- **FR-016**: The gate MUST be hermetic and deterministic and MUST require no capture driver, elevation, game, user store, trust change, proxy backend, external service, or network access.
- **FR-017**: The implementation MUST add no runtime dependency and MUST preserve the existing public `fragcap_cli::command()` seam as the command-definition authority.
- **FR-018**: Existing CLI reference prose MUST be corrected to the current v0.7.0 command, store, sink, and output-routing behavior in the same change.
- **FR-019**: Failures MUST distinguish an invalid reference contract, command-tree drift, sink-grammar drift, and an invalid worked invocation.
- **FR-020**: S093 MUST NOT change command grammar, runtime dispatch, capture behavior, output formats, trust behavior, storage schemas, workflow files, release configuration, or the master specification.

### Key Entities

- **Command path**: A public root-to-leaf sequence such as `targets add`, with its owning command section, local named options, positionals, and availability.
- **Option contract**: One named option's long flag, optional short alias, enumerated values, parser-declared default, owning command path, and availability.
- **Reference section**: The single human-visible CLI reference section that declares a command path and contains its mechanical option contract.
- **Worked invocation**: An executable `fragcap` command line in a fenced example, identified by source line and validated without dispatch.
- **Sink grammar**: The schemes, aliases, modifiers, value sets, and transport constraints accepted by the sink parser.
- **Exclusion policy**: The structural rule or checked reason that keeps a non-public command or option outside the reference contract.

## Success Criteria

### Measurable Outcomes

- **SC-001**: One documentation check accounts for 100 percent of public command and subcommand paths in both the reference and the runtime command definition.
- **SC-002**: The same check accounts for 100 percent of public named options, short aliases, enumerated values, and parser-declared defaults on their owning command paths.
- **SC-003**: Adding one public command, flag, enumerated value, or declared default without changing the reference produces a deterministic failure naming the mismatch.
- **SC-004**: Every executable `fragcap` invocation in the CLI reference parses successfully through a no-side-effect validation path, and a deliberately invalid specimen fails.
- **SC-005**: Every accepted sink scheme and modifier appears in the reference, with zero stale or invented scheme and modifier names.
- **SC-006**: The reference contains zero claims that managed store-path overrides are required and assigns each global JSON output class to its shipped stream.
- **SC-007**: `cargo xtask docs check`, the production documentation build, and the complete repository CI aggregate pass with no driver, elevation, game, proxy, trust mutation, network, or user-store dependency.

## Assumptions

- S093 implements issue #246 after the entry-point, first-run, architecture, and bundle-reference corrections have landed.
- The shipped default v0.7.0 command tree is the primary public contract. The network-capable maintainer variant is checked separately where it adds public flags.
- `fragcap_cli::command()` is already the stable, recursively enumerable command-definition seam and remains preferable to introducing a second CLI schema.
- Hidden offline harness controls and `__controlled-target` are test infrastructure, not public commands.
- Parser-generated help and version controls remain available through clap but need not be repeated in each command's authored option table.
- The production UX and accessibility audit remains assigned to issue #249. S093 builds the site but does not claim responsive or assistive-technology review.
- The issue's request for a documentation gate authorizes the narrow task-runner change needed to include the gate in `cargo xtask docs check`; it does not authorize workflow changes.
