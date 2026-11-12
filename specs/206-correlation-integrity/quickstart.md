# Quickstart: S206 verification

1. Run focused deterministic gate, native correlation, and calibration tests without a driver.
2. Run `cargo xtask ci` for repository parity.
3. Use the existing disposable Windows integration matrix for signed-driver behavior and packaged Npcap/WinDivert packet-order observations. Keep original-present and original-absent results separate.
4. Inspect exact-head native bundle, compatibility, process trace, manifest, and terminal summaries. Record private bundle locations only in local evidence; publish sanitized outcomes.
5. Validate hosted checks and review comments before the owner merge handoff. Never equate a controlled fixture with a physical packet-order observation.
