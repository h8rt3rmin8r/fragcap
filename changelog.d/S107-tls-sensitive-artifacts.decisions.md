<!-- spec-impact: 13.7, 19, 25, 26.3, 28.1 -->
### Decisions

- **2026-09-01:** S107 combines issues #300, #304, and #322 because TLS key logs and client credentials introduce the strict artifact class whose protection, retention, deletion, and sharing lifecycle must ship with their producer.
- Key logging is attached only to client-facing rustls server configurations. Upstream TLS configurations cannot emit secrets into the bundle.
- Client credentials come only from an explicit paired certificate and private-key input. Ambiguous trust refusal remains unknown, and fragcap never searches for target-owned keys or bypasses pinning.
- Authorized bundle protection now runs before proxy startup. This deliberately corrects the prior finalization-only placement, which occurred after the native application writer opened plaintext evidence.
- The sensitive-action journal recovers only artifact operations. General proxy, trust, launch, and capture recovery remains open under #320.
- Completed retained bundles are no longer broad cleanup candidates for `doctor --fix`; destructive cleanup is targeted and confirmed through `fragcap bundle cleanup`.
