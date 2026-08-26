# Quickstart: Verify Homepage Positioning And Target Footer

## Verify The CLI Footer

Run the focused CLI tests:

```powershell
cargo test -p fragcap-cli targets --locked
cargo test -p fragcap-cli --test cli_targets --locked
```

Confirm populated output contains one blank-line-separated line:

```text
Next command:  fragcap capture 1
```

Confirm empty output still contains `Add one:` and `Scan a folder:` and no `Next command:`.

## Verify Homepage Claims

Search the authored page for retired phrases:

```powershell
rg -n "40,000 packets|thrown away|three hops|passive network capture tool|Two prerequisites" 'site/app/(home)/page.tsx'
```

The command must return no matches. Inspect the page for `Capture`, `Deep Capture`, `Npcap`, `Wireshark`, and `fragcap doctor`.

## Build The Site

```powershell
cargo xtask docs build
```

Confirm the generated homepage carries `.nojekyll`, `CNAME`, the revised positioning, synthetic specimen, and exact labelled next command.

## Run Repository Gates

```powershell
cargo xtask ci
```

Scan changed lines for PII, actual local game titles, personal paths, credentials, endpoints, mojibake, and unplanned files before committing.
