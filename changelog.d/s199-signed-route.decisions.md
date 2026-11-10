<!-- spec-impact: 2, 3, 5, 19, 20, 27, 28 -->
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
