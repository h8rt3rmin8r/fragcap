# Quickstart: Native Supply-Chain and Compatibility Gate

## Offline policy gate

```text
cargo xtask supply-chain
```

Expected result: all three graph views match the reviewed policy; every source, checksum, declared Rust floor, critical pin, compatibility-line exception, unsafe review, and finite exception is classified.

## Full repository gate

```text
cargo xtask ci
```

Expected result: the supply-chain gate runs as part of the ordinary local sequence before documentation completion.

## Network-backed dependency audit

```text
cargo deny check --all-features --target x86_64-pc-windows-msvc advisories licenses bans sources
```

Expected result: fresh advisory data loads and all advisory, yanked, unmaintained, unsound, license, ban, and source checks pass. An unavailable database is an infrastructure failure, not a pass.

## Generate release evidence

Install the exact tool versions declared in `supply-chain/policy-v1.json`, set `SOURCE_DATE_EPOCH` to a fixed source revision time, and generate CycloneDX JSON plus third-party notices from `crates/fragcap-cli/Cargo.toml` for `x86_64-pc-windows-msvc` with the shipped `live,socket-table,etw` features. The release workflow contains the canonical commands.

Validate the results:

```text
cargo xtask supply-chain validate-evidence target/release/fragcap.cdx.json target/release/THIRD-PARTY-NOTICES.txt
```

Expected result: both files reconcile exactly to the locked shipped closure and contain no absolute workspace path.

## Mutation checks

- Change one expected graph digest and require `graph-drift`.
- Add an expired, unused, malformed, wildcard, or duplicate exception and require refusal.
- Change one critical dependency pin, feature, default-feature state, or review date and require the matching finding.
- Add a Windows-only or optional edge without refreshing policy and require drift on a non-Windows host.
- Add an unapproved compatibility line and require `duplicate-compatibility`.
- Remove or duplicate one SBOM component, alter its license, source, or checksum metadata, and require evidence refusal.
- Remove one third-party package marker and require evidence refusal.
- Remove either evidence file from WiX or archive assembly and require `artifact-wiring`.
- Move validation after packaging or publication and require `workflow-order`.

## Text and repository hygiene

```text
git diff --check
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
```

Confirm UTF-8 without BOM, LF endings, no mojibake, no Markdown hard wrapping, and no staged `.specify/feature.json`.
