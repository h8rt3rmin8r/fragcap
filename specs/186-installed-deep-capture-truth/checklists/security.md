# Security Checklist: S186 Installed Deep Capture truth

**Purpose**: Confirm the trust and tunnel boundaries before implementation
**Created**: 2026-10-31
**Feature**: [spec.md](../spec.md)

- [x] Ordinary-user health never gains private signer read permission.
- [x] Deep Capture still proves exact trust and signing before effects.
- [x] HTTPS authority normalization does not silently accept a distinct destination.
- [x] Upstream TLS verification and target scope remain mandatory.
- [x] A timeout or partial artifact cannot claim full HTTPS.
- [x] Public evidence excludes private field identities and traffic.

## Notes

The checklist describes required design boundaries. Controlled tests and review must verify implementation against them.
