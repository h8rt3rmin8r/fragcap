<!-- spec-impact: 29 -->

### Decisions

- [#274](https://github.com/shruggietech/fragcap/issues/274) closes the Deep
  Capture proxy backend research
  after an isolated `http-mitm-proxy 0.18.0` Windows spike. The smaller
  fallback passed controlled HTTP/1.1, client-facing HTTPS and HTTP/2,
  handshake, HAR-source, CA-separation, bounded-cache, and ten-run listener
  shutdown proofs. Its exact graph is not parseable by Cargo 1.82, and its
  public API provides neither client-facing TLS key logging nor bounded
  ownership of spawned connection tasks. Deep Capture retains external
  `mitmdump`; no further speculative backend path is opened.
