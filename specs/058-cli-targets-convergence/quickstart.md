# Quickstart / Validation Guide: S058

Local build/test on the GNU toolchain; CI runs the MSVC gate.

## Prerequisites

- Branch `058-cli-targets-convergence`.
- Rust GNU toolchain `1.96.0-x86_64-pc-windows-gnu`.
- `bash` (for `xtask docs check` and grep). `pnpm` optional (docs build).

## 1. Default `--db` (#157)

```bash
cargo +1.96.0-x86_64-pc-windows-gnu test -p fragcap-cli --test cli_targets
```

Expected: green, including the new no-`--db` default-store case. Manual check
(isolated store):

```bash
FRAGCAP_LOCAL_DB="$TEMP/s058.db" cargo +1.96.0-x86_64-pc-windows-gnu run -p fragcap-cli -- targets add "My Game" --exe game.exe --socket-holder unsure
FRAGCAP_LOCAL_DB="$TEMP/s058.db" cargo +1.96.0-x86_64-pc-windows-gnu run -p fragcap-cli -- targets list
```

Expected: `add` (no `--db`) registers into the env store; `list` (no `--db`) shows
it. An explicit `--db` still overrides.

## 2. extcap uses target selection (#156)

```bash
cargo +1.96.0-x86_64-pc-windows-gnu test -p fragcap-cli --test cli_extcap
```

Expected: green with the target-selection surface. Confirm the config block:

```bash
cargo +1.96.0-x86_64-pc-windows-gnu run -p fragcap-cli -- extcap --extcap-config --extcap-interface fragcap
```

Expected: four arg lines; the number=0 arg is `{call=--target}` (a target selector),
not `{call=--profile}`.

## 3. No profile-file resolution remains in extcap, one shared seam (#156)

```bash
grep -n "search_path\|paths::bundled\|resolve(" crates/fragcap-cli/src/commands/extcap.rs
grep -rn "fn resolve_stored\|fn setup_stores\|fn synthesize_named_profile" crates/fragcap-cli/src/commands/
```

Expected: the extcap capture handler no longer calls the profile-file
`resolve(...)`/`search_path`/`bundled` cascade; the resolution functions live once,
in `commands/target_resolve.rs`, called by both `capture` and `extcap`.

## 4. Docs + spec reconciled

```bash
grep -rn "legacy" site/content/docs/reference/cli.mdx
cargo +1.96.0-x86_64-pc-windows-gnu xtask spec
bash scripts/lint-docs.sh check
```

Expected: no extcap "legacy" callout remains; `xtask spec` reports the Applies-To
lockstep holds; the glossary/P-6 linter passes.

## 5. No Cargo.lock delta + full gate

```bash
git diff --stat Cargo.lock            # expect: no output
cargo +1.96.0-x86_64-pc-windows-gnu xtask ci
```

Expected: `Cargo.lock` unchanged; `cargo xtask ci` green (fmt, clippy, workspace
tests, lint, deps, license, docs check). The MSVC `--all-features` gate runs in CI.
