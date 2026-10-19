# S173 research and decisions

## 2026-10-19 token acquisition

Decision: use the exact desktop shell's real ordinary token after exact WTS/producer/session, elevation and integrity validation. An identification-only linked token is not impersonation authority.

Evidence: a hidden read-only local probe compared current process pseudo-token, own process token and ImpersonateSelf/OpenThreadToken acquisition. All returned the ordinary split account's linked full token as TokenImpersonation/SecurityIdentification; DuplicateTokenEx(SecurityImpersonation) failed with 1346. This measures the platform failure mechanism, not the owner's elevated-to-ordinary incident. A subsequent genuine elevated Sandbox regression also reproduced 1346 specifically at DuplicateTokenEx on the linked ordinary token. The fixed exact-shell path passed the same full writer/read fixture with a real unrelated standard principal denied.

Rationale: Microsoft requires SecurityImpersonation for impersonation. TokenLinkedToken promises only a handle. GetShellWindow identifies the exact shell; OpenProcessToken requires PROCESS_QUERY_LIMITED_INFORMATION. Validate account/session and ordinary elevation/integrity, then duplicate actual ordinary authority.

Alternatives considered: changing the current source token did not fix the measured linked-token limitation; restricted equivalents cannot prove genuine ordinary authority; a shell COM helper adds unnecessary launch/authentication orchestration; manual self-SID handoff violates the requested default.

Sources: [impersonation requirements](https://learn.microsoft.com/en-us/windows/win32/api/securitybaseapi/nf-securitybaseapi-impersonateloggedonuser), [linked token](https://learn.microsoft.com/en-us/windows/win32/api/winnt/ns-winnt-token_linked_token), [shell window](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getshellwindow), [token query rights](https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-openprocesstoken).

## Disposable evidence environment

Decision: inspect the owner-offered installed Windows Sandbox CLI for unattended elevated same-account evidence. Map only synthetic tools/results. Keep host UAC, accounts, trust and owner data unchanged. Missing ordinary desktop authority is an unavailable prerequisite, never a passed required case.

Source: [Windows Sandbox CLI](https://learn.microsoft.com/en-us/windows/security/application-security/application-isolation/windows-sandbox/windows-sandbox-cli).

## Retirement

Decision: delete historical inspection/repair and exclusive support; retain access-authorize and shared legacy operation machinery. Maintained docs describe current source while historical release/spec records retain prior behavior. No migration or historical permission changes are needed.

## 2026-10-19 required advisory patch review (#478)

Fresh all-feature cargo-deny identified RUSTSEC-2026-0338, 0340 and 0343 in quinn-proto 0.11.17 plus RUSTSEC-2026-0346 through 0349 in Rustls 0.23.45. The fixed graph passes advisories, bans, licenses and sources without a new waiver. Update all seven affected lockfiles; preserve historical release records and outer Quinn 0.11.11.

Downloaded normalized manifests for Rustls 0.23.45/0.23.46 and quinn-proto 0.11.17/0.11.18 differ only in package version. Dependencies, features, licenses and MSRV declarations remain identical (1.71 and 1.85, both below workspace 1.88). Selected ring/std/tls12 and Quinn runtime/ring feature declarations remain fixed. Upstream patches add or remove no unsafe lines. Existing unchecked VarInt sites in changed Quinn files retain their surrounding logic. This supports scoped source/compatibility review, not an independent security audit.

The [Rustls release](https://github.com/rustls/rustls/releases/tag/v%2F0.23.46) removes secret session Debug fields, rejects ECH rejection before application data, bounds Brotli certificate windows and authenticates key-exchange hints. It also tightens record limits, key updates, writer error propagation and handshake validation. The [quinn-proto release](https://github.com/quinn-rs/quinn/releases/tag/quinn-proto-0.11.18) guards empty unordered-read ranges, clamps ACK-delay bounds and fixes datagram eviction accounting, plus send-storage, reset/0-RTT credit and transport-parameter bounds. The package manifests remain compatible with Quinn 0.11.11. Its separate 0.11.12 outer RecvStream::read_to_end hardening is not imported or claimed here.

## 2026-10-19 measured Windows import closure

After vendor TLS recovery, hosted final-package PE inspection measured the original closed ordinary DLL set plus exactly user32.dll. The [GetShellWindow contract](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getshellwindow) and [GetWindowThreadProcessId contract](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getwindowthreadprocessid) both name User32.dll. These are the two query calls already confined to access/windows/shell.rs; this is the operating system component rather than a shipped file. The correction adds that exact system import to the inventory and does not alter the harness, exact comparison or delayed wpcap.dll policy. The existing full-report mutation regression first failed because the old inventory lacked User32; it now requires evidence to contain that import and rejects both its omission and an additional unreviewed DLL.
