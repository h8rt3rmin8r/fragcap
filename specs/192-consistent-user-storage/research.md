# Research: Consistent User Storage

## R-1: Windows virtualization and the selected root

Microsoft documents that new writes under a virtualized desktop app's AppData are redirected to a private package location, while writes elsewhere in the user profile are allowed when the user has permission. Its selective AppData exclusion is declared by the package manifest, which fragcap cannot change for an arbitrary launching host. The owner selected `%USERPROFILE%\.fragcap` on 2026-11-04. This is a product storage change, so the previous AppData roots must be inventoried and preserved. Sources: [packaged desktop file system behavior](https://learn.microsoft.com/en-us/windows/msix/desktop/desktop-to-uwp-behind-the-scenes#appdata-operations-on-windows-10-version-1903-and-later), [flexible virtualization](https://learn.microsoft.com/en-us/windows/msix/desktop/flexible-virtualization).

## R-2: Identity and effective path

The production resolver should derive the current account's profile from the Windows profile known folder and verify account/session identity for supported elevation. An environment value alone is insufficient authority because a launcher can supply another path. Existing-file effective destinations are obtained from opened handles; an absent path is reported as selected but unverified. An inaccessible or mismatched account is an error rather than another root selection. Source: [known-folder API](https://learn.microsoft.com/en-us/windows/win32/api/shlobj_core/nf-shlobj_core-shgetknownfolderpath).

## R-3: Migration and recovery

Blind file copying of a live SQLite database can omit committed WAL data. Use SQLite's consistent backup operation into a staged destination, validate it, then atomically publish without replacing an existing destination. Retain source bytes and a durable transition record until independent readback succeeds. Active or conflicting roots remain untouched with a named disposition. Old session bundles with path-bound recovery records remain at their original paths and are indexed for discovery; moving them without rewriting every authority would be unsound. Source: [SQLite online backup](https://www.sqlite.org/backup.html).

## R-4: Scope and alternatives

Keeping `%APPDATA%\fragcap` would require a separate unvirtualized process or control over each launching host's package manifest. Neither is part of the MSI/portable product contract. A junction, lexical path trick, or environment-only check does not prove the effective file destination. The chosen profile-root approach needs no new broker or service.

## R-5: Verification boundary

Source tests can exercise selection, migration, conflict, and cleanup with disposable roots. A controlled Windows package row observes the opened-file destination and independently reads a synthetic target from another ordinary process under the same account with altered AppData variables. Controlled native session creation and recovery remain separate package cases. An actual arbitrary packaged-host readback follows release, and later defects are separate GitHub issues.
