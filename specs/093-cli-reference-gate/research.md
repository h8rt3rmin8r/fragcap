# Research: CLI Reference Gate

## Decision 1: Put the contract test in `fragcap-cli`

**Decision**: Add `crates/fragcap-cli/tests/cli_reference.rs` and use the public `fragcap_cli::command()` seam.

**Rationale**: The CLI integration-test target can inspect the real clap tree without exposing new runtime APIs or creating a cross-crate dependency. It also has direct access to the package source and repository reference file through compile-time manifest paths.

**Alternatives considered**:

- Add `fragcap-cli` as an `xtask` dependency. Rejected because constitution P-2 explicitly prohibits a crate from depending on `fragcap-cli` and because it would couple repository tooling to the Windows-facing product layer.
- Generate and commit a separate CLI manifest. Rejected because the manifest and prose could drift together while remaining internally consistent.
- Scrape `--help` subprocess output. Rejected because formatted help is a presentation surface, loses owning-command structure, and varies more than the underlying clap model.

## Decision 2: Use visible MDX as the reference contract

**Decision**: Give every public command path one code-formatted heading and every owning command one option table with stable `Option`, `Values`, `Default`, and explanatory columns.

**Rationale**: The reviewer-facing page and the machine-facing contract stay identical. A two-sided set comparison detects both missing runtime entries and stale documentation entries. Commands without local options still remain visible as headings.

**Alternatives considered**:

- Hide JSON or YAML metadata in the page. Rejected because it creates a second description that readers cannot verify naturally.
- Infer command coverage from arbitrary prose mentions. Rejected because token mentions do not establish ownership or exact-once coverage.

## Decision 3: Compare owning-command options from clap

**Decision**: Recursively walk visible clap subcommands, exclude hidden commands and options structurally, exclude generated help/version actions through one policy, and compare locally owned long names, short aliases, enumerated values, and declared defaults. Document propagated globals once at the root.

**Rationale**: Ownership avoids duplicating `--json`, `--quiet`, and `--silent` on every subcommand. Structural visibility follows clap itself, and action-based treatment of generated controls avoids a drifting private-name list.

**Alternatives considered**:

- Compare flattened effective arguments on each command. Rejected because propagated globals would appear repeatedly and obscure their actual owner.
- Maintain explicit hidden-item exclusions by name. Rejected because newly hidden harness controls could silently escape review.

## Decision 4: Exercise default and `net` variants

**Decision**: Run the same integration test once with default features and once with `--features net`. Mark conditional table rows with their availability so each tree compares only its active contract.

**Rationale**: The `catalog seed` source flags are public when `net` is enabled. Running both trees verifies that availability instead of pretending conditional options are always present or ignoring them.

**Alternatives considered**:

- Check only default features. Rejected because public conditional flags would be unguarded.
- Exclude all feature-gated options. Rejected because these options are intentionally public in supported builds.

## Decision 5: Bind sink documentation to the parser source

**Decision**: Extract accepted sink schemes, aliases, and modifier names from the match arms in `crates/fragcap-cli/src/args.rs`, then compare them with the structured `--sink` section in the reference.

**Rationale**: clap sees `--sink` as a repeatable string and cannot expose the internal URI-like grammar. The parser match arms are the current accepted-token authority, and a source-level audit already has precedent in `cli_help.rs`.

**Alternatives considered**:

- Duplicate token constants only for documentation. Rejected because that would introduce a second runtime grammar authority.
- Assert a handwritten expected set in the test. Rejected because parser and test could drift together without failing the page.

## Decision 6: Parse examples without dispatch

**Decision**: Extract `fragcap` invocations from shell and PowerShell fences, combine the line continuations used by the page, strip shell comments outside quotes, tokenize the supported quoted forms, and call `command().try_get_matches_from()` only.

**Rationale**: This catches retired syntax while guaranteeing that no capture, store, network, trust, proxy, or process behavior can run. Diagnostics retain the source line of the example.

**Alternatives considered**:

- Execute examples with temporary paths. Rejected because many commands legitimately mutate state or require platform services.
- Check examples as raw strings. Rejected because quoting, positional arguments, and option arity would remain unverified.

## Decision 7: Compose the gate through `cargo xtask docs check`

**Decision**: Extend `xtask/src/docs.rs` to run the existing documentation linter, then the focused CLI-reference test in both feature variants.

**Rationale**: This gives contributors one documented gate and automatically includes the reference check in `cargo xtask ci`, which already invokes `docs check`. The subprocess composition preserves the existing crate graph.

**Alternatives considered**:

- Edit workflow files directly. Rejected because the existing CI aggregate is already the correct integration point and S093 excludes workflow changes.
- Leave the test discoverable only through `cargo test`. Rejected because a documentation-only contributor must receive the same drift failure through the documentation gate.
