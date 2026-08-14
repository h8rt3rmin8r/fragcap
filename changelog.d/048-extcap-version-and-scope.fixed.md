The `fragcap.exe` binary now carries its real version. Its Windows PE version
resource was never stamped, so `Get-Command fragcap` (and Explorer file
properties, and inventory tools) reported a FileVersion of `0.0.0.0` even though
`fragcap --version` printed the true version (issue #104). The build now embeds a
VERSIONINFO resource stamped from the crate version, so the FileVersion and
ProductVersion match `fragcap --version` and track releases automatically. The
version is single-sourced from the workspace version, so the two can never
disagree. This is a Windows build-time change only; no runtime behavior changes.
