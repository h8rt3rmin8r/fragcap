# S182 validation guide

1. Run targets tests with a disposable store and synthetic moved platform metadata. Check proposal, exact confirmation, unchanged ID/roles, and stale-workflow refusal.
2. Run real-adapter timing and production flow-correlation regressions for early withholding, overlapping omission, capacity exhaustion, tuple reuse, IPv4/IPv6, and both orientations.
3. Run a cold managed launch with an ordinary helper. Verify owner close, helper exit observation, route release, and incomplete outcomes for stuck helpers or lost watcher authority.
4. Run stop-timeout with later nonzero cleanup and application trailer. Verify terminal totals and failure while observation eligibility stays bounded.
5. Run repository source, native protocol, physical Windows, clean MSI/ZIP, advisory, and continuous two-hour soak gates. Retain exact hashes and validator reports.
6. Compare getting-started, CLI, compatibility, package, and troubleshooting pages against tested commands and lifecycle. Run site checks.

Private real-title field testing uses published bytes only. Candidate qualification cannot claim unpublished real-title compatibility or S182 publication.

## Source qualification (2026-10-27)

- `cargo xtask ci` passed after the calibration fixture was corrected to use its disposable install root. The first run had exposed a nonexistent synthetic `C:\Games\Fixture` root; the correction changes test setup only.
- The real CLI relocation preview, stale-plan refusal, apply, identity preservation, and second-preview refusal passed in `cli_targets`.
- Omission-history saturation passed with all 65 missing packet counts retained and a conservative unavailable correlation result.
- `cargo xtask conformance --analyzer` passed all 21 required protocol rows and the live TShark 4.6.8 analyzer check.
- A clean Windows Sandbox guest without the Visual C++ runtime or Npcap reproduced the published v0.10.8 portable startup failure (process exit `-1073741515`). The initial static-linked candidate reached version, help, and Doctor; Doctor separately reported missing Npcap. This was portable-only preliminary evidence. Exact final candidate and MSI lifecycle evidence are required before completion.
