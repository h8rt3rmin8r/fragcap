# Phase 0 Research: surface a Wireshark download link in doctor

The slice is small; two facts were established against current source before
implementation.

## Fact 1: how the CLI reaches a core constant

`fragcap-cli` depends on the `fragcap` facade, not directly on `fragcap-core`
(`crates/fragcap-cli/Cargo.toml` lists only `fragcap = { workspace = true,
features = ["targets"] }`). The facade re-exports selected `fragcap-core::interface`
items into `fragcap::core` (`crates/fragcap/src/lib.rs:33-36`), and the CLI
imports core items as `fragcap::core::...`. `DRIVER_DOWNLOAD_URL` is defined at
`crates/fragcap-core/src/interface.rs:386` but is not currently re-exported.

**Decision**: Define `WIRESHARK_DOWNLOAD_URL` in `fragcap-core::interface` beside
`DRIVER_DOWNLOAD_URL`, add it to the facade re-export block, and import it in
`checks.rs` as `fragcap::core::WIRESHARK_DOWNLOAD_URL`. This keeps the dependency
direction unchanged (CLI -> facade -> core) and matches how every other core item
reaches the CLI.

**Alternative considered**: add a direct `fragcap-core` dependency to
`fragcap-cli` and import `fragcap_core::interface::...`. Rejected: it introduces a
new edge the workspace does not have and is not needed; the facade already exists
for exactly this.

## Fact 2: goldens and tests affected

The only doctor golden is `doctor-ready.{txt,ndjson}` (the all-ok ready state).
The npcap-absent remediation and the extcap not-registered guidance are the two
strings this slice changes, and neither appears in the ready state (ready has
npcap ok and extcap installed). So **no golden regeneration is needed**.

Existing tests remain valid: `absent_npcap_blocks_with_a_remediation...` asserts
the npcap remediation contains `npcap.com`, which the unchanged npcap literal
still satisfies; the npcap URL is not changed by this slice. `Check::ok/warn/fail`
take `impl Into<String>`, so turning `NPCAP_SOURCE` (a `const &str`) into
`npcap_source() -> String` needs no call-site shape change.

**Decision**: Convert `NPCAP_SOURCE` to `fn npcap_source() -> String` that formats
using `WIRESHARK_DOWNLOAD_URL` for the Wireshark URL (single-sourced) and keeps
the existing `https://npcap.com` npcap literal. Add unit tests: the integration
not-registered detail contains `WIRESHARK_DOWNLOAD_URL`, and the npcap remediation
contains it too. Keep the integration check an optional `Warn`.

## Constant value

`WIRESHARK_DOWNLOAD_URL = "https://www.wireshark.org/download.html"`, the download
page named in issue #107 (more actionable than the site root the string used
before). The getting-started prose links the site root; that is documentation
prose, a separate source from the tooling constant, and is left unchanged.
