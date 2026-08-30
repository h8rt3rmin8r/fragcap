# fragcap-proxy

`fragcap-proxy` owns the native loopback runtime for fragcap Deep Capture.

S102 provides the library boundary and bounded listener, connection, task, and
shutdown ownership. It intentionally does not forward, decrypt, parse, or claim
inspectability for traffic. The functional v0.8 command-line path continues to
use external mitmdump until the native integration issue is complete.

This crate never configures an ambient system proxy and never reaches inside a
target process.
