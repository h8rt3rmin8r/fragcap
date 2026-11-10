# S199 Verification Quickstart

1. Run Spec-Kit requirements and consistency analysis over this directory, the constitution and master specification. Resolve every finding before component edits.
2. Run `cargo xtask package-certification` for the static package authority and `cargo xtask ci` for repository CI parity.
3. In the Windows package workflow, acquire the two exact official archives, verify hashes, stage the four component entries, and assemble one MSI and one ZIP.
4. Certify both packages on a disposable Windows runner: exact archive/member hashes, x64 PE machine, driver Authenticode signer, matching installed and portable bytes, ordinary startup, Doctor, and all required installer lifecycle rows.
5. Review the report and exact-head CI, then inspect every bot review and PR reaction. Fix actionable findings, request at most one second `@Codex` review, and leave final merge to the owner.

Do not use a private title or change the workstation's installed Fragcap. A successful S199 package is ready to supply the later route adapter; it is not evidence that any direct game socket reached the proxy.
