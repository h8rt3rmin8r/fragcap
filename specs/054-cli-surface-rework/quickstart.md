# Quickstart / validation: CLI surface rework

Validation scenarios that prove the slice end to end. Capture scenarios run through
the hidden offline substrate (no capture driver, no elevation, no game). Local
builds use the GNU-host toolchain; CI runs the real MSVC gate.

## Prerequisites

- Workspace builds: `cargo +1.96.0-x86_64-pc-windows-gnu build -p fragcap-cli`
- Tests: `cargo +1.96.0-x86_64-pc-windows-gnu test -p fragcap-cli`

## The five captures (SC-001)

Each expressible with one `capture` invocation and covered by a test driving the
offline substrate:

1. Target + ring: `capture --target <t> --mode ring --ring 10m --out cap.fcapng`
2. Named process + ring: `capture --process eso64.exe --mode ring --ring 10m --out cap.fcapng`
3. Named process + wait: `capture --process unknown.exe --wait 5m --out cap.fcapng`
4. Target launched under capture: `capture --target eso --launch --duration 30m --out cap.fcapng`
5. Target + give-up timeout: `capture --target eso --wait 5m --out cap.fcapng`

Expected: each parses, assembles an `EffectiveConfig`, and runs the offline pipeline
to a written capture. The `--process --launch` negative exits 2.

## Removal negatives (SC-002)

Expected exit 2 / unknown-subcommand for each:

- `fragcap run --profile x`
- `fragcap tap --process x`
- `fragcap watch --exe x`
- `fragcap profile validate x`

`fragcap schema validate some.json` still works.

## Namespace moves (SC-005)

- `fragcap catalog seed-signatures --db catalog.db` seeds the catalog signature table.
- `fragcap targets seed-signatures …` no longer resolves.
- `fragcap targets add "Elden Ring" --db local.db --steam 1245620` registers a target
  in `local.db` with a `steam:1245620` anchor, equivalent to the old
  `steam profile 1245620` scaffold.
- `fragcap steam profile 1245620` no longer resolves.

## Presentation (SC-003, SC-004)

- `fragcap --help` shows the four headings (Capture / Targets / Environment / Data)
  with every command present, nothing hidden.
- `fragcap` (no args) prints the targets listing plus the `--help` footer.
- `fragcap targets` prints the same listing without the footer; diff the two outputs
  and confirm they differ only by the footer line.

## Documentation coherence (SC-006)

- Scan the docs tree and master-spec section 17: no example names `run`, `tap`,
  `watch`, `profile …`, or a catalog op under `targets`.

## Full gate (SC-007)

- `cargo xtask ci` (MSVC, in CI) plus `cargo xtask spec` (Applies-To + spec-impact)
  pass. Locally, run the GNU-host equivalents that do not need MSVC.
- The changelog fragment `changelog.d/S054-*.md` carries a `spec-impact:` header
  naming section 17 (and 15.7 if the relocated signature-seed prose names a command).
- Glossary entries exist for any new term introduced by this change.
