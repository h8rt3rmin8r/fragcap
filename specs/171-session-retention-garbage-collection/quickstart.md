# Validation Guide: S171

Use synthetic temporary roots only. No owner AppData, game launch, real trust mutation or published-release claim is part of this guide.

1. Run facade collection regressions to prove terminal eligibility, unknown/reparse/replaced-object refusal and interruption recovery.
2. Run CLI session_gc and Doctor regressions to prove active leases, retained/custom policy, automatic boundaries and a 1,000-container backlog.
3. Run CLI bundle preview/apply/purge help and controlled command tests. Compare exact preview identities and logical-byte results.
4. Run `cargo xtask ci` through the verified hidden redirected launcher and read its completed exit status.
5. Push the feature branch, open and attach the official PR, respond to all findings, verify final-head CI and stop for owner merge. Only the automatic first external review plus one optional second request are authorized.
