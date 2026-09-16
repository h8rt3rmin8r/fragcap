# Data model: S154

## Controlled consumer state

A shared test-only control carries armed, blocked, released and stopped flags. Header acceptance precedes arming; blocked acknowledgement follows the first packet write; release permits draining and stop refuses pending writes. All observation waits have finite deadlines. No production type changes.

A final consumer report preserves id, offered, written, dropped and reason. Exact conservation is offered = written + dropped. The independent file retains all submitted packet payloads in original order.

## Documentation inventory

Schema version 1 carries eleven closed topic rows. Each row has id, page and a non-empty tests array of path, function and explicit features. IDs are unique and exhaustive. Page paths are confined current site MDX files; test paths are confined Rust crate/xtask evidence files with an actual non-ignored Cargo-discoverable test function. Neither row presence nor validation asserts independent security acceptance.

Published version and source stay in the existing publication authority, not a second mutable release identity.
