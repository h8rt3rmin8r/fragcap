**2026-08-13** The targets hint database foundation (issue #78, slice S034)
landed, adding the first embedded-database dependency, and the decisions behind it
were recorded rather than left implicit.

First, `rusqlite` is taken with `default-features = false` and only the `bundled`
feature. rusqlite 0.40's default set enables a WebAssembly FFI backend
(`ffi-sqlite-wasm-rs`) that drags roughly fourteen packages into `Cargo.lock` (the
wasm-bindgen stack, `js-sys`, `thiserror`, `bumpalo`) for machinery this project
never runs. With defaults off, the delta is six packages: `rusqlite`,
`libsqlite3-sys`, `fallible-iterator`, `fallible-streaming-iterator`, `smallvec`,
and `vcpkg`. `cc`, `bitflags`, `shlex`, `find-msvc-tools`, and `pkg-config` are
already in the graph via `pcap`, so they add nothing. The delta was measured by
adding the dependency and diffing `Cargo.lock`, not estimated.

Second, the store is bundled, not linked against a system SQLite. `bundled`
compiles the SQLite amalgamation through `cc`, so the build is deterministic on a
bare Windows runner with no external database library, and the bundled SQLite
carries the JSON1 functions a later query slice may use. A hand-rolled indexed,
transactional on-disk format was rejected (it leaves the hard half to be written),
as was `sqlx` (an async runtime and a far larger graph for a synchronous
single-file local store).

Third, the minimum supported toolchain stays 1.82, verified rather than assumed.
None of the six new crates declares a `rust-version`; all compile under Rust 1.82,
confirmed by building through `rustup run 1.82` (the same path `cargo xtask msrv`
takes). The dependency is taken as a `0.40` range rather than exact-pinned, because
unlike `clap` nothing in its graph breaks the floor today; `cargo xtask msrv` is
the standing gate should a future lock update raise it.

Fourth, licensing. Every crate in the delta is MIT or Apache-2.0. The SQLite
amalgamation vendored inside the MIT `libsqlite3-sys` crate is public-domain C, so
`cargo deny` reads the crate's MIT metadata and the license gate passes; public
domain imposes no attribution obligation, recorded here for completeness. The new
`fragcap-targets` crate carries `LICENSE`, `NOTICE`, and `README.md` per the
license gate.

Fifth, placement and gating. The store lives in a new leaf crate `fragcap-targets`
depending only on `fragcap-profile`, never on `fragcap-core` (the `cargo xtask
deps` allowlist gained the two edges and the sibling entry). It is optional and off
at the facade behind a `targets` feature, so a default library build compiles no
SQLite engine; the command-line tool enables it unconditionally so the shipped tool
carries the `targets` subcommand. Unlike `live`, `socket-table`, and `etw`, the
store needs no capture driver and no elevation, only the C toolchain at build time,
which is why it is on for the binary rather than off by default.
