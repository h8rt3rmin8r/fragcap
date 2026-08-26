# Contract: Known-Root Path Composition

`KNOWN_ROOTS` remains the one separator-neutral list for fixture and live discovery.

For each live volume and root:

1. The generic known-roots walk joins the volume mount and root in the existing separator-neutral form used by fixtures.
2. `FsDirectoryLister` converts forward slashes to the Windows native separator immediately before calling the real filesystem. Non-Windows hosts need no conversion.
3. Use each real lister child unchanged for candidate identity and install root.

On Windows, emitted real paths contain backslashes and no forward slashes. On Unix-like test hosts, emitted real paths contain forward slashes and no backslashes. Fixture trees retain their established normalized strings because they do not cross `FsDirectoryLister`.
