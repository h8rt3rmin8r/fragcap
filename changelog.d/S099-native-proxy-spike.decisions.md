<!-- spec-impact: 29 -->

### Decisions

- [#253](https://github.com/shruggietech/fragcap/issues/253) records the
  isolated native proxy spike: `hudsucker 0.23.0` passed the controlled
  protocol, lifecycle, HAR-source, CA-separation, bounded-cache, and
  client-facing key-log proofs, but no measured dependency resolution was both
  advisory-clean and parseable by Rust 1.82's Cargo. The shipped
  `mitmdump` backend remains unchanged while one smaller native fallback is
  evaluated.
