# Real-Machine Validation: Target Discovery Integrity

**Status**: Complete
**Date**: 2026-09-09

This record accepts aggregate counts only. It must never contain a title name, application id, local path, account, hostname, volume identity, or machine identifier.

| Count | Value |
| --- | ---: |
| Candidates considered | 63 |
| Produced candidates | 56 |
| Eligible for automatic registration | 55 |
| Refused by precision policy | 1 |
| Parse failures | 0 |
| Not-a-game outcomes | 5 |
| Interactive candidates declined | 0 |
| Containers descended | 1 |
| Container descent truncated | 1 |
| Volumes skipped | 0 |
| Access errors | 0 |
| Warnings | 1 |

The automatic-registration execution counts are validated from the synthetic CLI fixture rather than the operator's existing local store, so this non-target-persistent real-machine run does not create or modify target rows. Ordinary first-run catalog bootstrapping and volume-eligibility seeding may still occur.
