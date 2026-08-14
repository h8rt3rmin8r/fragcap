# Quickstart / validation: doctor truthfulness and presentation

Runnable checks that prove the slice. The first block is what CI runs; the
machine-dependent checks are the pre-push manual smoke tests (they need a Windows
machine with npcap and the live backend).

## Prerequisites

- Rust toolchain (workspace MSRV 1.82) for the default build.
- For the live checks: a Windows machine with npcap installed and a build with
  `--features live` (the shipped release enables it).

## CI-parity checks (run in the foreground)

```sh
# The whole gate the project's automated checks run.
cargo xtask ci

# The default (no-feature) build and test must still compile and pass. This is
# the R-1 regression guard: the enumerate() call must be cfg-gated.
cargo test --workspace --locked

# Regenerate the two doctor goldens after the output change, then confirm clean.
FRAGCAP_UPDATE_GOLDENS=1 cargo test -p fragcap-cli --test cli_doctor
cargo test -p fragcap-cli --test cli_doctor
```

Expected: `cargo xtask ci` passes; the golden diff shows the new Identity section,
blank-line section separation, and the extra identity records in the ndjson; the
`the_json_form_is_one_record_per_check` test stays green.

## Unit-level expectations (in the doctor test suites)

- Loopback classifier: `Some(true)` -> ok, `Some(false)` -> warn,
  `None` -> warn with "could not be determined".
- Identity classifiers: the leading section carries the four fixture values;
  an unresolvable path renders as an undetermined note, still ok.
- Interfaces classifier: a populated fixture lists adapters; an empty fixture
  warns only then.

## Machine-dependent smoke tests (pre-push, on Windows with npcap)

```sh
# Real interfaces appear; loopback reported correctly; identity header present.
fragcap doctor

# Piped output is byte-plain (no color codes).
fragcap doctor | cat

# NO_COLOR suppresses color even in a terminal.
NO_COLOR=1 fragcap doctor

# Machine-readable form is one record per check and never colorized.
fragcap doctor --json
```

Expected: on a machine with adapters and the live backend, the Interfaces section
lists the real adapters (no "no interfaces were found"); the loopback line
reflects the real state; the report opens with the Identity section (version,
binary path, profile dir, hint-db path); color shows only in the interactive
terminal run.
