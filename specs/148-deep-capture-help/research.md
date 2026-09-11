# Research: Deep Capture Embedded Workflow Help

## Decision 1: Own workflow prose and examples in one CLI registry

**Decision**: Add one typed, ordered registry for the first-run stages, audited help paths, displayed command examples, representative parse arguments, and first-run refusal guidance. Clap help, human refusals, semantic tests, and the later public documentation work will consume or validate against this authority.

**Rationale**: The current help is assembled from isolated doc comments while calibration already creates exact next commands in a separate path. A registry makes the whole journey inspectable and prevents a command from being edited in prose without its parser-backed example changing with it.

**Alternatives considered**:

- Duplicating a tutorial into every command was rejected because the pages would drift and short help would become noisy.
- Keeping examples only in tests was rejected because users need the exact commands at the point of refusal.
- Making issue #331's future website page the authority was rejected because embedded help must work offline and the CLI grammar owns whether an example parses.

## Decision 2: Use Clap's short and long help distinction deliberately

**Decision**: Keep enum-variant summaries and option first sentences concise for `-h`, add full workflow and examples only to `--help`, and assign options to stable required, common, advanced, sensitive, storage, and troubleshooting headings.

**Rationale**: Clap already supports separate short and long descriptions plus argument headings. Using its command model preserves one parser and one renderer while providing progressive disclosure.

**Alternatives considered**:

- A separate `help deep-capture-workflow` command was rejected because the journey would no longer be discoverable from the commands users already invoke.
- A hand-rendered help page was rejected because it would duplicate Clap's actual grammar and default rendering.

## Decision 3: Bound refusal work at the post-parse, pre-session boundary

**Decision**: Define a closed category set covering environment, target, launch, process, compatibility, prior recovery, bundle, authorization, and guided-calibration stops. Human errors and outcomes in those categories receive one registry-backed `Next command:` or a finite `Next commands:` block. Structured records, exit codes, and effect behavior do not change.

**Rationale**: This is the exact point where a first-time operator can become stranded. Including parser grammar errors would restate Clap usage, while including failures after session effects begin would turn a help slice into a runtime redesign.

**Alternatives considered**:

- Appending the same generic calibration command to every error was rejected because target discovery, recovery, and unsafe bundle destinations need different action.
- Changing structured error schemas was rejected because exact next commands are a human navigation contract and machine output already has typed state.

## Decision 4: Test semantics and parser agreement instead of freezing bytes

**Decision**: Walk the real Clap tree for every audited `-h` and `--help` page, render at 40, 60, and 80 columns, assert required concepts and section order, parse every registry command with representative values, and exercise refusal-category guidance. Use targeted snapshots only if semantic assertions cannot express a stable contract.

**Rationale**: Whole-page snapshots are brittle across wrapping changes and obscure meaningful omissions. Parser-backed and order-aware semantic tests fail on workflow drift while permitting wording improvements.

**Alternatives considered**:

- Golden files for every width and color mode were rejected because formatting churn would dominate review without increasing grammar confidence.
- Source-text scanning was rejected because it does not prove what Clap renders or accepts.

## Decision 5: Keep implementation acceptance controlled and effect-free

**Decision**: Validate S148 through command-model rendering, argument parsing, temporary stores, existing controlled refusal seams, and the repository gate. Do not launch a real game, mutate a real trust store, start sensitive live capture, or infer compatibility.

**Rationale**: Help and refusal navigation can be proved completely without exercising sensitive software. This preserves the operator-owned, published-release-only live-validation boundary established in S147.

**Alternatives considered**:

- A live first-run walkthrough was rejected because the operator explicitly prohibited agent-run sensitive software and it would test compatibility rather than the help contract.

## Audit findings

- Root help exposes the relevant commands but does not connect them into a first-run sequence.
- `deep-capture` and `calibrate` present a flat argument list with no stable help headings.
- Target, Doctor, bundle, and calibration doc comments are individually accurate but do not explain their place in the same workflow.
- Existing help tests enumerate the complete command tree for long help, leaks, and a 100-column ceiling, but they do not cover `-h`, the S148 audited journey, 40 through 80 columns, or example parsing.
- Prior-recovery refusal already names `fragcap doctor --fix`; compatibility and target-selection paths need the same exact-command discipline.
- Calibration already carries typed `next_command` values, which should be validated against the shared command registry rather than replaced.
- Bundle cleanup says deletion is sensitive but does not state which recovery records survive an incomplete cleanup, and bundle export help does not clearly distinguish the unchanged source from the share copy.
