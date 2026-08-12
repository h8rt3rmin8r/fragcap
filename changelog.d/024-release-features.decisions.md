**2026-08-12** Changed the release workflow (`.github/workflows/release.yml`) to
build the distributed binary with the `live`, `socket-table`, and `etw`
capability features and to acquire the npcap SDK for the link step, rather than a
bare `cargo build --release` that shipped a binary unable to capture (issue #62).
npcap itself remains unbundled; only its SDK import library is used at build time.
