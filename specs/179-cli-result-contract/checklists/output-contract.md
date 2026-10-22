# Output Contract Checklist: Consistent CLI Result Output

**Purpose**: Check the precision of channel and verbosity requirements for PR review.

## Format and Channel

- [x] CHK001 Does each JSON result have a named command/result identity and typed fields? [Clarity, Spec Key Entities]
- [x] CHK002 Are document and protocol outputs kept free of an added envelope? [Consistency, Spec US1 and US3]
- [x] CHK003 Is stdout reserved for non-capture command results and stderr for lifecycle diagnostics? [Clarity, Spec FR-002]
- [x] CHK004 Are one-record-per-diagnostic and terminal-summary semantics stated? [Completeness, Spec FR-005]

## Verbosity and Failures

- [x] CHK005 Does the spec resolve default, quiet, silent, JSON, and JSON plus silent behavior? [Completeness, Spec US2]
- [x] CHK006 Are required prompts and cleanup obligations retained under silent? [Coverage, Spec FR-006]
- [x] CHK007 Are error exit classes and failed-output cases retained without false success records? [Coverage, Spec Edge Cases]
- [x] CHK008 Is an unsupported combination refused before side effects with an actionable reason? [Clarity, Spec FR-009]
