<!-- spec-impact: 2.1, 8.3.1, 13.7, 17.2.1, 19, 24, 25, 26, 28.1, 29 -->
### Added

- Native Deep Capture now owns authenticated HTTP/1.1 forwarding, CONNECT, client-facing TLS under the exact session authority, and a separately verified upstream TLS connection.
- The controlled protocol path now drives real native HTTP and HTTPS traffic through the public facade and CLI without Python, a capture driver, a game account, remote service, or operating-system trust mutation.
- Native protocol observations retain bounded loss accounting and distinguish HTTP, CONNECT, client TLS, upstream TLS, and terminal failure boundaries.

### Changed

- Deep Capture now selects `fragcap-native` as its sole production proxy backend. The `--proxy-backend` option and the production mitmdump/Python orchestration path are removed.
- `fragcap doctor` reports the compiled native proxy version instead of searching for an external executable.
- Session proxy credentials and CA identity now flow only through the authorized post-start facade route and the exact managed child environment.

### Fixed

- Session bundle directory creation is now an explicit artifact-owner operation instead of an accidental side effect of starting the former external proxy.
- Steam protocol launches are refused before effects while native child-scoped routing cannot be guaranteed; issue #308 owns the platform-client routing strategy.
- Native observations again join the packet-side flow registry for real targets, while controlled observations carry the exact spawned child process identifier rather than the parent CLI identifier.
- HTTP/1.1 clients using `Expect: 100-continue` no longer deadlock behind a withheld request body, and every parsed request retains metadata evidence when later forwarding or response handling fails.
