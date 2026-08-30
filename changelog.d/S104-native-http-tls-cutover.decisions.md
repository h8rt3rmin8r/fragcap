<!-- spec-impact: 8.3.1, 13.7, 17.2.1, 19, 24, 25, 26, 28.1, 29 -->
### Decisions

- S104 combines issues #290, #292, and #293. Cutting the CLI over under #290 without functional HTTP and TLS would have replaced a working path with a listener-only backend.
- HTTP/1.1 uses a bounded wire-level codec around `httparse` because the selected Hyper server API cannot relay every upstream informational response required by the protocol contract. Hyper remains pinned for later HTTP/2 work.
- Every proxy connection uses standard Basic proxy authentication with a fresh random session password. The raw capability is compared in constant time, redacted from diagnostics, excluded from artifacts, and released with the lease.
- Client-facing and upstream TLS remain separate typed boundaries. The proxy-issued leaf authenticates the inspected destination to the authorized client, while the upstream connection always performs independent chain and hostname verification.
- Production external proxy names, commands, and embedded Python are rejected by a source-policy gate. Historical specifications remain permitted so the repository keeps an honest architectural record.
- Steam protocol dispatch is not treated as child-scoped merely because Steam is cold. The native cutover refuses it before side effects until issue #308 supplies a routing mechanism with demonstrable ownership.
