# Quickstart / Validation Guide: Technology-Detection Surface

This validates the slice end to end, offline, with no game and no capture driver.
It is a run/validation guide; implementation lives in `tasks.md`.

## Prerequisites

- The workspace builds (`cargo build --workspace`).
- The vendored asset is present:
  `crates/fragcap-profile/assets/steamdb/rules.ini`, its
  `rules.lock.json`, and `THIRD_PARTY_NOTICES.md`.

## 1. The vendored asset is faithful and locked

```sh
cargo test -p fragcap-profile sha256
cargo test -p fragcap-profile ruleset
```

Expected: the SHA-256 self-tests pass against the NIST vectors, and the embedded
`rules.ini` hashes to the value recorded in `rules.lock.json`. Independently, a
plain `sha256sum crates/fragcap-profile/assets/steamdb/rules.ini` reproduces the
locked hash (bytes are LF / UTF-8 / no BOM).

## 2. Incompatible patterns are skipped, counted, and conserved

```sh
cargo test -p fragcap-profile technologies
```

Expected: the ruleset loads; `compiled + skipped == total` holds; the skipped
count is exposed and the affected technologies are identifiable. (The applied
categories compile in the large majority; any incompatible pattern is counted,
never silently dropped.)

## 3. Detection reports engine and anti-cheat from an install layout

The unit tests build a temporary install directory containing marker files (for
example an `EasyAntiCheat/` directory and a `.../Binaries/Win64/Game-Win64-Shipping.exe`)
and assert the report lists the anti-cheat and the engine under their categories,
each with the matched marker path and `heuristic-unverified` fidelity, and that a
technology revealed by several files is reported once.

```sh
cargo test -p fragcap-profile technologies
```

## 4. Unreadable target is distinct from an empty scan

Expected (unit test): a directory with no markers yields empty findings and empty
unreadable; an unreadable subtree yields a surfaced unreadable path; the two are
never conflated.

## 5. The CLI surface prints a grouped report

```sh
# Point at any local game install directory:
cargo run -p fragcap-cli -- technologies --path "<INSTALL_DIR>"
```

Expected: technologies grouped by category, each naming its marker path, with a
heuristic-unverified banner; a skipped-patterns note when non-zero; exit 0 on a
clean scan, a surfaced non-zero only if the target directory itself is unreadable.

## 6. The metadata validates against the master schema

```sh
cargo test -p fragcap-profile --test schema_conformance
```

Expected: a target carrying a `technologies` array validates; a malformed item
(missing/invalid `category`) is rejected; an empty `technologies` array
validates. The embedded and `docs/schema` copies remain byte-identical (the
existing drift check passes).

## 7. The scaffold carries technologies

Expected (fragcap-steam test): scaffolding a target from an install directory
with technology markers produces an artifact whose `technologies` array reflects
the detected set, and the artifact still validates against the schema.

## 8. Full gate

```sh
cargo xtask ci
cargo xtask msrv     # MSRV 1.82 stays green
```

Expected: fmt, clippy, tests, lint, deps (no new dependency; core allowlist
unchanged), and license all pass; MSRV build is green.
