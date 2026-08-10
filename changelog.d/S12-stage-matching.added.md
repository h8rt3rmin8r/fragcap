### Added

- **Stage matching.** `fragcap-profile::matching` evaluates a profile's stage
  predicates against an observed process tree and binds each process to the
  first stage, in declaration order, all of whose predicates hold. Specification
  section 10.3. The five predicates behave as the section defines: `exe` a
  case-insensitive file-name glob, `path_contains` and `path_regex` over the
  full image path, `cmdline_contains` which never matches a command line that was
  not observed, and `descends_from` resolved over the synthetic process tree
  rather than the operating system parent chain. It opens nothing and names no
  platform type, so section 10.3 is tested at tier 1 against a scripted event
  stream.
- **A stage binding is recorded on the node.** `fragcap-core` gains
  `ProcessTree::bind_stage`, which writes the stage field slice S11 reserved. A
  node binds to at most one stage. The decision of which stage a node binds to
  stays in `fragcap-profile`; only the recording is in core, which keeps the
  profile schema out of `fragcap-core`.
- **The capture session lifecycle.** The `fragcap` facade gains a `session`
  module carrying the five-state machine of specification section 10.5: it arms
  before any target exists, discards and counts packets while watching, retains
  on the first stage match with nothing lost at the boundary because the handle
  is already open, and drains to a valid capture on any stop condition.
- **Every stop condition, and one shutdown.** Specification section 10.6's six
  conditions (the duration bound, the volume bound, a terminal-stage exit, all
  matched processes having exited, an operator interrupt, and a sink error) each
  end capture through the same drain, and an acquisition timeout completes a
  session that never acquired a target.
- **Packets discarded before acquisition are counted.** A
  `SessionStats::watching_discarded` counter records every packet dropped while
  watching, and the session's conservation identity, that observed equals
  retained plus watching-discards, is asserted in the tests. Constitution P-4.
