# Contract: Native Windows Package Certification

## Static command

`cargo xtask package-certification` validates the versioned package contract, exact artifact and lifecycle registries, WiX ownership and upgrade rules, release workflow ordering, build-identity wiring, tool pins, current documentation, size ceilings, PE allowlists, signature policy, and mutation-test ownership. It performs no installer, Defender, trust, route, capture, proxy, network, or publication effect.

## Report command

`cargo xtask package-certification validate-report <report.json>` validates a Windows-produced report against the current contract. It refuses an unknown schema, contract digest, release identity, artifact, entry, import, signature state, lifecycle case, terminal outcome, missing row, duplicate row, skipped required row, unbounded field, absolute host path, credential-like value, or incomplete result.

## Artifact contract

The primary release artifacts are the versioned x86_64 portable ZIP, versioned x86_64 MSI, and standalone `catalog.db`. Each primary artifact has one adjacent lowercase SHA-256 sidecar whose line contains exactly its digest and basename. Sidecars are not recursively checksummed.

ZIP and MSI containers expose exactly `fragcap.exe`, `catalog.db`, `LICENSE`, `NOTICE`, `fragcap.cdx.json`, and `THIRD-PARTY-NOTICES.txt`. Shared roles are byte-identical across all applicable surfaces. Entry names are relative, canonical, case-unique, separator-stable, non-link paths that cannot escape the package root.

Every entry and primary artifact remains within its reviewed size ceiling. The executable is x86_64 PE, carries the exact release version resource and machine-readable build identity, and imports only the closed ordinary and delayed DLL sets.

## Signature contract

The current executable and MSI policy is `not_signed`, established only by a determinate Authenticode inspection with no signer. ZIP, catalog, documents, evidence, and checksum sidecars are `not_applicable`. Any other state or any prose claim that the package is signed fails.

## Packaged smoke contract

The exact extracted and installed executables must report the certified build identity, start with Npcap absent, report the native Deep Capture backend, and complete the existing controlled loopback native smoke under a sanitized environment. No undeclared child process, non-loopback connection, network fetch, package manager, Python executable, external proxy executable, first-use installer, trust residue, route residue, or target mutation is allowed.

## Installer lifecycle contract

All Windows Installer processes are hidden, non-interactive, finite, exit-accounted, and scoped to fresh CI roots. Clean install creates exactly the owned payload and effects. Repair restores one deleted and one modified owned file. Exact-byte reinstall remains idempotent. Upgrade replaces the digest-pinned v0.8.0 predecessor with one current product. Downgrade is refused without changing the newer installation. Uninstall removes only owned state and is idempotently absent on repetition.

Seeded per-user catalog/local database, capture, bundle, and separately managed extcap fixtures remain byte-identical in every case. Defender installation is best effort and explicitly classified; any exact exclusion created for the installed binary is absent after uninstall. Local MSI logs remain unuploaded scratch because they may contain runner paths.

## Publication contract

The enforced order is tag/workspace identity, official build, S130 evidence, package assembly, checksum generation, package certification, certified-bundle upload, independent bundle revalidation, GitHub release creation, and crate publication. No `continue-on-error`, conditional bypass, warning-only result, rebuild-after-certification, or alternate publication path can authorize a release.
