# S173 controlled validation

## Prerequisites

Use synthetic temporary bundles only. Ordinary tests exercise real tokens/filesystem access; required elevated evidence needs a genuinely elevated same-account producer and ordinary session shell. Use the owner-offered Windows Sandbox CLI if it provides that context. Missing prerequisites are not success.

## Focused checks

Run cargo test -p fragcap --features deep-capture --test bundle_access --locked for parents, independent writers, atomic publication, ordinary full reads and unrelated-authority denial. Run the dedicated elevated test in the disposable elevated desktop using --ignored --exact elevated_desktop_production_recipient_reads_every_final_artifact. It requires an elevated UAC split token, FRAGCAP_TEST_PUBLIC_ROOT and disposable unrelated standard-user credentials in FRAGCAP_TEST_UNRELATED_USER/FRAGCAP_TEST_UNRELATED_PASSWORD. These are generated inside the disposable guest and never stored in repository source or public evidence. Run cargo test -p fragcap-cli --test cli_bundle --locked and parser/session guidance unit tests. Retired verbs fail usage before fixture changes.

## Full gates and delivery

Run cargo xtask ci in the foreground. Inspect final-head hosted Windows/Linux, docs, dependency, security and package gates. Record elevated, ordinary and equivalent evidence separately. Publish one official PR with Closes #476, Closes #477 and Closes #478; reconcile all threads with at most one requested second round. Human merge follows satisfied reviews and final-head green CI.
