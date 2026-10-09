# Data Model: S172

## Release Candidate

Fields: version 0.10.6, exact merged base, dependency head, local commits, release identity inventory, graph digests, assembled fragments, notes and gate results. States: specified -> analyzed -> prepared -> locally verified -> committed -> authorized remote handoff -> hosted verified -> owner merged -> authorized tagged -> verified publication. This run completes through committed; later states are handoff steps, not incomplete implementation tasks.

## Published Baseline

Fields: version 0.10.5, exact source/tag, verified assets and registry checksums, frozen review-candidate record. Candidate preparation does not mutate these fields; independent publication reconciliation belongs to a separate reviewed records PR.

## Gate Result

Fields: command, source state, environment, exit code, evidence and limitations. Labels: passed, failed, unavailable, not run. Older-source success and local contract checks cannot establish current hosted package evidence.

## Dependency Disposition

Fields: PR #470, exact source, two candidate files, Next.js 16.3.8, retained overrides, current-source validation and pending external disposition. Local changes do not close or merge the existing PR.
