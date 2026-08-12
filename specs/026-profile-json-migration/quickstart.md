# Quickstart: Profile JSON migration

Runnable scenarios proving the migration. Assumes a built workspace.

## Scenario 1: a valid JSON profile validates

```bash
cargo run -p fragcap-cli -- profile validate crates/fragcap-cli/tests/data/game.json
```

Expected: `is valid`, exit 0. (The former `game.toml` is now `game.json`.)

## Scenario 2: a profile with mixed faults reports them all in one pass

Author a JSON profile with a structural fault (an unknown key) and a semantic
fault (two terminal stages), then:

```bash
cargo run -p fragcap-cli -- profile validate ./bad.json
```

Expected: both problems reported in one run, each located by JSON pointer, exit 2.

## Scenario 3: the former TOML is refused, not half-parsed

```bash
cargo run -p fragcap-cli -- profile validate ./old-profile.toml
```

Expected: refused as invalid JSON, exit 2 (a leftover TOML profile is not
silently accepted).

## Scenario 4: scaffold emits JSON with a machine-readable heuristic warning

```bash
cargo run -p fragcap-cli -- steam profile <app_id> > scaffold.json
cargo run -p fragcap-cli -- schema validate scaffold.json
```

Expected: `scaffold.json` validates; it carries `"fidelity": "heuristic-unverified"`
and a `"notes"` string with the verification warning (no TOML comment).

## Scenario 5: capture behavior is unchanged

Load an equivalent profile and run the corpus pipeline; the committed goldens
reproduce byte-for-byte (the profile format changed, not the capture output).

## Gate

```bash
cargo xtask ci
cargo xtask msrv     # builds at 1.82, now with one fewer dependency
```

Expected: green. `toml-span` no longer appears in the dependency graph.
