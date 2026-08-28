# Quickstart: Verify First Capture and Deep Capture Journeys

## Audit the Retired Baseline

The rewritten guide must contain none of the known v0.5-era claims:

```powershell
rg -n "0\.5\.0|KNOWN|the_elder_scrolls_online|the_division_2|--db pointing|Payloads are encrypted|streaming and ring capture" site/content/docs/getting-started.mdx
```

Expected: no matches.

## Audit Required Current Contracts

```powershell
rg -n "0\.7\.0|Deep Capture|Next command:|ENGINE|SENSITIVITIES|sample-target|steam-protocol-cold|mitmdump|current-user|unknown|partial|failed|manifest|doctor --fix" site/content/docs/getting-started.mdx
```

Expected: each contract appears in the guide with its scope and refusal conditions intact.

## Compare Current Commands

```powershell
cargo run -q -p fragcap-cli -- --help
cargo run -q -p fragcap-cli -- doctor --help
cargo run -q -p fragcap-cli -- targets --help
cargo run -q -p fragcap-cli -- targets show --help
cargo run -q -p fragcap-cli -- capture --help
cargo run -q -p fragcap-cli -- deep-capture --help
```

Confirm every command in [contracts/journey-contract.md](contracts/journey-contract.md) uses an accepted command, selector, and flag. Do not run the live Capture or Deep Capture examples during documentation verification.

## Run Focused CLI Contracts

```powershell
cargo test -p fragcap-cli --test cli_args
cargo test -p fragcap-cli --test cli_help
cargo test -p fragcap-cli --test cli_targets
cargo test -p fragcap-cli --test cli_doctor
cargo test -p fragcap-cli --test cli_deep_capture
```

Compare the doctor specimen with `crates/fragcap-cli/tests/goldens/doctor-ready.txt`. The guide uses `0.7.0` in place of the test-only version token and keeps every machine-specific value synthetic.

## Audit Privacy and Scope

```powershell
rg -n -i "account|token|private endpoint|real title|system-wide|pinning|target.*key|warm steam|direct executable|unknown" site/content/docs/getting-started.mdx
```

Review every match in context. Sensitive values must appear only as exclusions or handling guidance. System-wide proxying, pinning bypass, target key extraction, warm Steam, direct execution, and unknown compatibility must never be presented as supported v0.7.0 behavior.

## Run Documentation Gates

```powershell
cargo xtask docs check
cargo xtask docs build
```

## Run the Full Repository Gate

```powershell
cargo fmt --all -- --check
cargo xtask ci
```

Review `git diff --check`, changed-file punctuation, UTF-8 decoding, mojibake, local links, and the final file inventory. Confirm the implementation changes only the getting-started page, the changelog fragment, and S090 artifacts.
