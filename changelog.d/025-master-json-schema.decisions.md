**2026-08-12** The master JSON Schema is authored, embedded, and published as a
standard Draft 2020-12 document, but fragcap validates against it by hand over
`serde_json::Value` rather than embedding a JSON Schema validator crate. The
candidate crate `boon` was evaluated and rejected: adding it to `fragcap-profile`
pulls 42 new transitive crates into `Cargo.lock` (the ICU4X stack via
`url`/`idna`) to service `$ref` and `format` machinery this schema does not use,
which is irreconcilable with the project's dependency discipline. The ecosystem
value of JSON Schema comes from publishing the document (editors, agents, and the
submission pipeline validate against it natively), not from consuming a validator
crate in the binary, and fragcap already hand-rolls this class of work (the glob
matcher, the pcap parser, the profile validator). A conformance corpus test binds
the published schema to the hand-rolled validator so they cannot drift. The only
new runtime dependency is `serde_json`, promoted from dev-only to a runtime
dependency of `fragcap-profile`; it was already in the build graph, so no crate is
added to `Cargo.lock`. MSRV stays 1.82.
