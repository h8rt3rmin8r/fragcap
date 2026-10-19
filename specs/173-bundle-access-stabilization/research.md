# S173 research and decisions

## 2026-10-19 token acquisition

Decision: use the exact desktop shell's real ordinary token after exact WTS/producer/session, elevation and integrity validation. An identification-only linked token is not impersonation authority.

Evidence: a hidden read-only local probe compared current process pseudo-token, own process token and ImpersonateSelf/OpenThreadToken acquisition. All returned the ordinary split account's linked full token as TokenImpersonation/SecurityIdentification; DuplicateTokenEx(SecurityImpersonation) failed with 1346. This measures the platform failure mechanism, not the owner's elevated-to-ordinary incident. Stage diagnostics and genuine elevated regression remain required.

Rationale: Microsoft requires SecurityImpersonation for impersonation. TokenLinkedToken promises only a handle. GetShellWindow identifies the exact shell; OpenProcessToken requires PROCESS_QUERY_LIMITED_INFORMATION. Validate account/session and ordinary elevation/integrity, then duplicate actual ordinary authority.

Alternatives considered: changing the current source token did not fix the measured linked-token limitation; restricted equivalents cannot prove genuine ordinary authority; a shell COM helper adds unnecessary launch/authentication orchestration; manual self-SID handoff violates the requested default.

Sources: [impersonation requirements](https://learn.microsoft.com/en-us/windows/win32/api/securitybaseapi/nf-securitybaseapi-impersonateloggedonuser), [linked token](https://learn.microsoft.com/en-us/windows/win32/api/winnt/ns-winnt-token_linked_token), [shell window](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getshellwindow), [token query rights](https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-openprocesstoken).

## Disposable evidence environment

Decision: inspect the owner-offered installed Windows Sandbox CLI for unattended elevated same-account evidence. Map only synthetic tools/results. Keep host UAC, accounts, trust and owner data unchanged. Missing ordinary desktop authority is an unavailable prerequisite, never a passed required case.

Source: [Windows Sandbox CLI](https://learn.microsoft.com/en-us/windows/security/application-security/application-isolation/windows-sandbox/windows-sandbox-cli).

## Retirement

Decision: delete historical inspection/repair and exclusive support; retain access-authorize and shared legacy operation machinery. Maintained docs describe current source while historical release/spec records retain prior behavior. No migration or historical permission changes are needed.
