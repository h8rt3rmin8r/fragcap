# Contract: `mitmdump` Backend Adapter

## Backend responsibilities

The adapter owns exactly one `mitmdump` child process per Deep Capture session.

It must:

- discover the executable through the same PATH-based logic doctor uses;
- record backend name and version;
- bind to a loopback address and a session-selected local port;
- configure CA material and key-log output through backend-supported options;
- stream or collect application observations into fragcap's application JSONL model;
- report backend startup, readiness, abnormal exit, timeout, and cleanup outcomes;
- stop the child process at session end or interrupt.

## Backend non-responsibilities

The adapter must not:

- install Python packages;
- mutate system proxy settings;
- read target memory;
- extract target TLS keys;
- modify target binaries;
- hide from the user that it is on the data path.

## Test doubles

The implementation must include a fake backend for deterministic CI tests. The fake backend emits predictable HTTP, HTTPS, metadata-only, unsupported, failure, and cleanup events without requiring `mitmdump`.

Local demonstration tests may use real `mitmdump` when present, but those tests must skip clearly when the executable is unavailable.
