# S155 Quickstart

## Local static verification

Do not run the installed product on the owner's workstation.

```powershell
cargo test -p xtask review_record
cargo test -p xtask package_certification
cargo xtask review-handoff
cargo xtask ci
```

## Validate an externally supplied completed record

```powershell
cargo xtask review-record C:\review\native-product-review.json
```

A zero exit means the record has no detected mechanical blocker. Maintainers still verify reviewer authenticity, independence and issue acceptance outside the command.

## Hosted published-build replay

Run the `Published review candidate` GitHub Actions workflow. It downloads the six immutable v0.10.1 release files, verifies the registry identities, installs the MSI on disposable hosted Windows, runs separate portable and installed controlled smoke, reconciles cleanup and uploads only the bounded sanitized report.

The replay does not perform the source-only installed QUIC retest required by #413. That issue remains open until an independent reviewer supplies the missing evidence.
