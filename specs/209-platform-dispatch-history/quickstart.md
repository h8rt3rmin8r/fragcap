# Quickstart: S209 Controlled Verification

1. Run the unit and integration tests for platform identity, dispatch gating, store migration, history comparison, and native evidence projection. The controlled process source supplies cold and warm platform generations, a publisher handoff, and unrelated sentinels.
2. Run the repository full gate with cargo xtask ci. The S208 test-clean wrapper verifies exact temporary-root cleanup for the workspace test suite.
3. Confirm a clean first-run history, a comparable repeated run, a failed association, a changed build or scope, and a rollback. Read the local store and native bundle outputs independently; neither historical facts nor history rows may authorize a new effect.
4. Check the package and hosted Windows gates on the final branch head. Record any installed-product observation separately from controlled qualification.
