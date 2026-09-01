# Security Requirements Checklist: S107

**Purpose**: Verify the authorized local TLS and sensitive-artifact boundary before implementation
**Created**: 2026-09-01
**Feature**: [spec.md](../spec.md)

## Authorization and Scope

- [x] Key logging is disabled unless present in the immutable confirmed plan
- [x] Client credentials come only from an explicit operator-supplied pair
- [x] No target process key discovery, extraction, or copying exists
- [x] No certificate-pinning bypass is implemented or claimed
- [x] Credentials are scoped to the selected target's upstream session

## Secret Handling

- [x] Key-log records contain only client-facing proxy TLS secrets
- [x] Private-key and session-secret bytes are absent from diagnostics and debug output
- [x] Concurrent key-log records are complete, serialized, and live-flushed
- [x] Sensitive files and directories are protected before exposure
- [x] Partial permission or write failures fail closed and remain visible

## Retention, Deletion, and Sharing

- [x] Retention is explicit in the confirmed plan and defaults to preserve
- [x] Deletion is exact, confirmed, idempotent, journaled, and audited
- [x] Interrupted sensitive deletion can be recovered without claiming general recovery
- [x] Share preparation creates a separate atomic copy and never mutates the source
- [x] Share manifests list included and omitted artifacts without secret contents

## Failure Classification

- [x] TLS refusal categories are stable and evidence-backed
- [x] Ambiguous client trust or possible pinning remains unknown
- [x] Missing, invalid, expired, and rejected identities are distinguishable where observable
- [x] No unsupported protocol path is upgraded to inspected

## Verification

- [x] Controlled TLS 1.2 and TLS 1.3 analyzer compatibility is demonstrated
- [x] Controlled mutual-TLS acceptance and refusal cases are covered
- [x] Windows access-control behavior is exercised on a real Windows filesystem
- [x] Pending, malformed, bounded, and partial-failure journal paths are covered
- [x] Repository denylist and full gate suite pass
