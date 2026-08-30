# Harness Contract

The isolated workspace provides contract tests and a bounded `candidate` command. The three-way comparison is a review of the committed S099 evidence because S099 intentionally committed sanitized Markdown rather than raw machine-specific output. The harness binds only to loopback, contacts only harness-owned origins, never reads or writes system proxy or trust configuration, trusts the private CA only in controlled clients, never disables validation, and applies deadlines to startup, traffic, cancellation, and cleanup.

The adapter may use public `http-mitm-proxy` 0.18.0 interfaces, buffer fixed bodies, and minimally inspect public upgraded streams. It must not patch, fork, or access private candidate internals to manufacture a passing result.

Protocol limitations are evidence. Structural incompleteness, unsafe endpoints, missed global deadlines, evidence overwrite, or sanitization failure exit nonzero. Cleanup is always attempted and residue is recorded.
