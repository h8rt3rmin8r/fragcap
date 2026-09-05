# Quickstart: Native Windows Packaging Certification

## Offline package contract gate

```text
cargo xtask package-certification
```

Expected result: contract, WiX, workflow, build identity, documentation, size, PE, signature, lifecycle, and report authorities reconcile without changing machine state.

## Full repository gate

```text
cargo xtask ci
```

Expected result: package certification runs in the ordinary offline gate after S130 supply-chain validation.

## Windows final-artifact certification

```text
pwsh -NoLogo -NoProfile -NonInteractive -File scripts/Test-PackageCertification.ps1 -ArtifactDirectory target/package-certification -ReportPath target/package-certification/report.json -Confirm:$false
cargo xtask package-certification validate-report target/package-certification/report.json
```

Expected result: final bytes, checksums, unsigned state, PE identity/imports, packaged native smoke, and every lifecycle row pass; only a bounded public-safe report is retained.

## Required mutation checks

- Add, remove, rename, duplicate by case, traverse, or alter one package entry and require exact artifact-content refusal.
- Exceed one size ceiling and require size-budget refusal.
- Change PE machine, version resource, ordinary import, delayed import, build feature, target, source revision, or backend identity and require binary-identity refusal.
- Remove, duplicate, alter, recursively checksum, or stale one checksum sidecar and require checksum refusal.
- Change `not_signed` to signed, invalid, unknown, or misleading prose and require signature-policy refusal.
- Omit, duplicate, skip, time out, or alter one lifecycle row and require report refusal.
- Remove one WiX component, ownership condition, tool pin, workflow trigger, certification dependency, or post-transfer revalidation and require static refusal.
- Introduce Python, mitmdump, mitmproxy, OpenSSL, Npcap, shell wrapper, repository document, user state, or undeclared child behavior and require exact prohibited-content refusal.

## Text and repository hygiene

```text
git diff --check
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
```

Run the repository PowerShell compliance wrapper against `scripts/Test-PackageCertification.ps1`. Confirm UTF-8 without BOM, LF endings, no mojibake, no Markdown hard wrapping, and no staged `.specify/feature.json`.
