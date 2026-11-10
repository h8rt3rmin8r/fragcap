<!-- spec-impact: 2, 3, 5, 19, 20, 21, 24, 27, 28 -->
2026-11-09: Owner-approved #572 and #577 replace the absolute interception-driver
ban with a precise separation between passive Npcap Capture and explicitly
selected, bounded Deep Capture direct diversion. Constitution 2.0.0 retains no
target memory access, injection, hooks, in-place game-file edits, silent global
proxy changes, or disruption of unrelated traffic. Detected anti-cheat products
remain evidence, not eligibility gates. Voluntary non-memory target key export
is research #580; proxy-owned key logs remain separate.

2026-11-09: Select official WinDivert 2.2.2-A x64 under its LGPLv3 option as an
external packaged component, not a new Rust dependency. Pin the upstream
binary and corresponding source archives and selected DLL/SYS hashes, preserve
dynamic replacement and source rights, and keep fragcap's Apache-2.0 source
identity. Stage the component only in Windows distribution assembly, certify
the exact driver signature on a fresh runner, and defer `WinDivertOpen` to the
later authorized route adapter. Static Doctor readiness cannot prove a future
driver open or game compatibility.

2026-11-09: Select a future session-scoped dynamic load of the exact packaged
DLL after direct-route authorization. S199 introduces no binding or driver
handle. Upstream installs the driver on demand at `WinDivertOpen` with
Administrator rights and says an already-running driver may remain until the
next reboot after files are removed. The S199 installer matrix proves owned
file removal and does not claim active-service teardown. Read-only Doctor may
report a disabled existing service; otherwise runtime blocking remains
unverified until the later route adapter attempts its authorized open.

2026-11-09: Hosted audit reported RUSTSEC-2026-0459 in the bundled SQLite
version selected by rusqlite 0.40.2. Update the existing rusqlite edge to
0.41.1 and libsqlite3-sys to 0.39.1, whose bundled source declares SQLite
3.54.1. The newer rusqlite requires Rust 1.89, so raise the workspace MSRV,
minimum-toolchain gate, audit toolchain and reviewed graph digests together.
No new package or graph edge is introduced by this repair.
The two isolated `fuzz` and `performance/native-proxy` lockfiles also carry
the patched versions so their locked audit and short campaign can run.

2026-11-09: The first complete hosted package assembly reached lifecycle
certification and exposed a verifier error: Doctor emits JSON Lines, but the
new component filter treated each line as a parsed object. Parse each nonempty
record before selecting the component check; a local exact-component Doctor
fixture confirms the `ok` readiness record without opening the driver.

2026-11-09: The documentation-closeout head's Ubuntu test run exposed an
existing timing-sensitive offline-capture assertion. A five-packet volume
stop can interrupt the source before all 24 fixture packets are observed;
the file's exact five retained packets and `volume-reached` reason were
already correct. Assert the reported observation count covers those five and
cannot exceed the 24-packet source, preserving the test's actual contract.
