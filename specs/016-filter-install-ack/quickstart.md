# Quickstart: Filter Manager Install Acknowledgement

**Slice**: 016 | **Date**: 2026-08-10

Tier-1 validation. No capture driver, no elevation, no game.

## The gate (foreground, watch to completion)

```bash
cargo xtask ci
```

Then, each exits 2 if its target/toolchain is absent (report which):

```bash
cargo xtask neutral
```

```bash
cargo xtask msrv
```

## What the tests demonstrate

- **SC-001 (rejection retried)** - `filter.rs` unit test: after an issued install,
  a `acknowledge(handle, false)` leaves the handle not considered installed, and a
  later poll (after the rate-limit interval) re-issues the same program.
- **SC-002 (success committed once)** - `filter.rs` unit tests: `acknowledge(handle,
  true)` commits the program; a later poll with the same set installs nothing; the
  debounce, rate limit, and gap accounting still hold. The existing filter-manager
  tests are updated to acknowledge their installs and otherwise assert the same.
- **SC-003 (pipeline retries)** - `pipeline/mod.rs` test: a source double that
  rejects maintenance `set_filter` calls records more than one attempt of the same
  program (the control thread retried through the ack channel), and the run ends on
  its own.
- **SC-004 (goldens)** - `corpus_pipeline` and `cli_run` reproduce byte-identically
  (the replay source accepts every filter, so acknowledgement is always success).

## Goldens must not move

```bash
cargo test -p fragcap --test goldens
cargo test -p fragcap-cli --test cli_run
```

Both pass with no regeneration.

## Expected outcome

`cargo xtask ci` green; `neutral` and `msrv` exit 0; corpus goldens byte-identical;
the rejection path retries and the success path is unchanged from S13.
