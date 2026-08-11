### Decisions

**2026-08-10: Steam integration and managed launch (slice S17), decisions worth
recording for promotion to specification section 29.**

- **The registry read and the protocol handler go through the workspace's
  already-resolved `windows-sys` 0.36, not `winreg`** (a deviation from the
  specification crate table, which names `winreg`). The additive features
  `Win32_System_Registry` (the Steam install-path read) and `Win32_UI_Shell`
  (`ShellExecuteW` for the `steam://` handler) add no package to `Cargo.lock` and
  change no resolved version, whereas `winreg` would add a second Windows-binding
  package tree. This mirrors the recorded S10 decision to reuse `windows-sys`
  rather than take a second binding, and `fragcap-attr` already carries the
  `unsafe` FFI pattern this follows. No new runtime dependency is added.
- **`fragcap-steam` compiles on every target; only its Windows internals are
  cfg-gated.** The public API (discover, scaffold, launch request) is
  cfg-independent, so the facade and CLI build on the neutral non-Windows target
  (P-2, FR-014). The registry read and `ShellExecuteW` are `#[cfg(windows)]`, with
  the non-Windows arm returning an "only supported on Windows" error. The VDF
  parser, the scaffolding classifier, and the launch-URL and launch-config
  decisions are portable and unit-tested on the CI host whatever its OS. Gating
  the whole crate on `cfg(windows)` was rejected: it would break the neutral
  facade build and hide the portable logic from non-Windows CI.
- **A scaffold proves its own validity by round-trip.** The renderer builds TOML
  text and parses it back through `fragcap_profile::Profile::parse` before
  emitting, so FR-008 (the scaffold passes section 15.4 unedited) holds by
  construction rather than by a separate assertion. Emitting untested text was
  rejected as a P-9 risk.
- **Scaffolded stage rules are `exe` image-name predicates and never inferred
  `descends_from`.** Runtime process topology, including the observed case where
  three processes share the image name `TheDivision2.exe` (Q-4), is invisible to a
  static install-directory scan, so ancestry cannot be inferred at scaffold time.
  The heuristic header comment and the existing section 15.4 runtime warning cover
  that case. Where two proposed stages would share a basename, the renderer adds a
  `path_contains` predicate so the output passes the ambiguous-image-match check.
- **Managed launch uses `steam://run/<app_id>`** (`steam://rungameid/<app_id>` is
  the noted alternative; a mutable detail). The launch is issued after the session
  reaches its watching state and the sinks are open, which is what removes the
  acquisition race. Because live capture is never executed in CI, the tests assert
  the launch decision (URL, ordering, refusals); the actual `ShellExecuteW` call
  is Windows-and-live-gated and tier-2/manual, and is not asserted as run in CI.
- **Section 16.5 (environment inheritance) is deferred, not implemented.** Reading
  another process's environment block requires a process handle carrying
  memory-read rights, which the constitution's technique denylist and the
  `OpenProcess` lint forbid. It is a corroborating signal only, and section 10
  ancestry already attributes reliably, so deferring it costs no capability.
