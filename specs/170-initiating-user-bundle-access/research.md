# S170 recipient and repair research

## Exact recipient identity and context

Decision: the production desktop default binds TokenSessionId to WTSUserName/WTSDomainName for that exact local session, resolves a fully qualified individual SID and requires equality with the tool token. A linked limited token proves the same account's unelevated context. An ordinary token is obtained through ImpersonateSelf/OpenThreadToken on an isolated thread. A process-token pseudo handle supports query only and cannot simply be duplicated. Every restore result is checked, and impersonation never crosses an async await.

Rationale: current and linked administrator tokens can both belong to credentials supplied for a different desktop user. Exact session association catches that mismatch; active-console/first-user guessing would not establish recipient identity. Alternatives rejected: object owner, Administrators membership, environment-only identity, arbitrary desktop enumeration, producer-context reads and query-only token duplication.

Sources: [token information](https://learn.microsoft.com/en-us/windows/win32/api/winnt/ne-winnt-token_information_class), [process-token rights](https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-getcurrentprocesstoken), [WTS query](https://learn.microsoft.com/en-us/windows/win32/api/wtsapi32/nf-wtsapi32-wtsquerysessioninformationw), [WTS user/domain](https://learn.microsoft.com/en-us/windows/win32/api/wtsapi32/ne-wtsapi32-wts_info_class), [account lookup](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-lookupaccountnamew), [ImpersonateSelf](https://learn.microsoft.com/en-us/windows/win32/api/securitybaseapi/nf-securitybaseapi-impersonateself), [restore](https://learn.microsoft.com/en-us/windows/win32/api/securitybaseapi/nf-securitybaseapi-reverttoself).

## Explicit different-account handoff

Decision: one unpredictable local pipe, exact recipient/producer/SYSTEM ACL, first-instance and remote-client rejection, one 4096-byte versioned request and a finite 60-second deadline. An explicit ordinary helper connects with impersonation SQOS, binds the exact request/destination and is read before server impersonation. Validate exact SID, unelevated token and session, duplicate the accepted context and retain it for filesystem checks. Use individual pipe read/write-data rights so the client ACE does not grant creation of another pipe instance.

Rationale: this supports different-account output without passwords, unrelated process handles or guessed accounts. Missing or contradictory evidence produces an early truthful refusal. A bare selected SID without an authenticated unelevated context cannot establish access. Session association is not a claim of historical causal discovery of a person who clicked a shortcut.

Sources: [pipe impersonation](https://learn.microsoft.com/en-us/windows/win32/api/namedpipeapi/nf-namedpipeapi-impersonatenamedpipeclient), [pipe rights](https://learn.microsoft.com/en-us/windows/win32/ipc/named-pipe-security-and-access-rights), [impersonation context](https://learn.microsoft.com/en-us/windows/win32/api/securitybaseapi/nf-securitybaseapi-impersonateloggedonuser).

## Whole-bundle historical repair

Decision: validate bounded manifest/prefix plus resource-journal provenance and exact CLI owner lease; inventory all declared artifacts and recognized writer-owned auxiliaries, including sensitive-actions and atomic staging names. Pin every object, reject reparses, stale preview, unknown objects and unproven outside hard-link aliases, and apply descriptor changes by handle. Validate the whole population before mutation. Permission results are external command output; retained evidence and recovery journals remain byte-identical. Reverify after session reconciliation.

Rationale: manifest-only repair misses protected lifecycle files. Canonicalize-then-mutate paths permit replacement races, and file ACL changes affect hard-link aliases. Inheritable directory ACE changes can propagate into children, so unchecked directory-first recursive repair is insufficient. Unreadable provenance in the privileged inspector remains an explicit unresolved result, not guessed ownership. Arbitrary inaccessible custom ancestors are not eligible for recursive permission replacement.

Sources: [SetSecurityInfo propagation](https://learn.microsoft.com/en-us/windows/win32/api/aclapi/nf-aclapi-setsecurityinfo), [pinned directory/reparse access](https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-createfilea), [file security rights](https://learn.microsoft.com/en-us/windows/win32/fileio/file-security-and-access-rights), [hard-link effects](https://learn.microsoft.com/en-ca/windows/win32/fileio/hard-links-and-junctions).

## Controlled evidence

Decision: perform actual filesystem operations under linked, ordinary, restricted-equivalent and authenticated helper contexts; distinguish them in reports. Security-descriptor inspection alone is insufficient. Unrelated anonymous-token denied-read probes require a readable positive-control sibling and traversable synthetic parent, so an ancestor failure cannot masquerade as bundle isolation. Restricted tokens preserve TokenUser and are not represented as a separate human account. Historical fixtures reproduce mixed protected sidecars and prove byte conservation, complete access and idempotent partial retry.

Sources: [restricted tokens](https://learn.microsoft.com/en-us/windows/win32/api/securitybaseapi/nf-securitybaseapi-createrestrictedtoken), [anonymous impersonation](https://learn.microsoft.com/en-us/windows/win32/api/securitybaseapi/nf-securitybaseapi-impersonateanonymoustoken). No owner-sensitive bundle or real game is used. Required research-agent findings are incorporated; no technical clarification remains unresolved.
