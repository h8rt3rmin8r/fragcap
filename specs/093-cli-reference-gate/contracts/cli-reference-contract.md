# CLI Reference Contract

## Authority Order

1. `fragcap_cli::command()` defines public command paths, locally owned options, aliases, finite values, and parser defaults.
2. `crates/fragcap-cli/src/args.rs` defines accepted `--sink` schemes, aliases, modifiers, and combination rules.
3. `site/content/docs/reference/cli.mdx` is the public explanation and worked-example authority.
4. `crates/fragcap-cli/tests/cli_reference.rs` compares these authorities without executing a command.

## Command Sections

- Each visible clap subcommand MUST have exactly one code-formatted command heading.
- Parent commands and commands with no local named options MUST still have a heading.
- A heading that names no visible clap command MUST fail as stale documentation.
- Hidden commands MUST be excluded through clap's hidden metadata.
- The root-level global option table MUST be checked separately and MUST NOT be repeated as locally owned subcommand options.

## Option Tables

- Each command section MUST carry one table whose option rows declare the long flag and any short alias.
- Finite clap value sets MUST be represented exactly after stable normalization.
- Clap-declared defaults MUST be represented exactly. Runtime fallback prose MUST NOT be promoted into the default column.
- Rows conditional on `net` MUST declare their availability and MUST compare successfully in both default and enabled test runs.
- Generated help and version actions are governed by one documented exclusion policy and are not authored repeatedly.

## Sink Grammar

- The accepted schemes and aliases MUST equal the parser-derived set.
- The accepted modifier names MUST equal the parser-derived set.
- The reference MUST explain platform limits for named-pipe and Unix-domain transports.
- The reference MUST explain applicable modifier constraints, including format selection, payload policy, rotation, queue capacity, and timeout.

## Worked Examples

- Every executable line beginning with `fragcap` in a supported command fence MUST be discovered.
- The extractor MUST support the quoted paths, comments, and line continuations that occur in the page.
- Each logical invocation MUST parse through `fragcap_cli::command().try_get_matches_from()`.
- Validation MUST NOT call `run`, `run_with`, a command handler, or an external `fragcap` process.
- Failures MUST include the one-based source line and clap diagnostic.

## Output Routing

The reference MUST distinguish:

- command result records written to standard output when the command supports structured results;
- Capture and Deep Capture lifecycle events written to standard error;
- capture bytes written only to configured sinks;
- warnings and errors written to diagnostic output;
- `--quiet` suppression of progress without suppressing warnings or errors;
- `--silent` suppression of progress, warnings, and summaries without suppressing errors or sink output.

## Gate Composition

`cargo xtask docs check` MUST run:

1. the existing documentation glossary and link checks;
2. the CLI-reference test with default features;
3. the CLI-reference test with `net` enabled.

The aggregate MUST be hermetic and deterministic. It MUST require no network, capture driver, elevation, game, proxy, trust mutation, user store, or pre-existing local configuration.

## Required Diagnostics

- **Invalid reference contract**: malformed, duplicate, or unrecognized heading or table data, with source line.
- **Command-tree drift**: commands or options present on only one side, with owning command path and both sets.
- **Sink-grammar drift**: parser and reference scheme or modifier sets differ, with both sets.
- **Invalid worked invocation**: parser rejection, with source line and clap diagnostic.
