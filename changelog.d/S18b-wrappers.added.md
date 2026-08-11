### Added

- **Shell wrappers: `Invoke-FragCap.ps1` and `fragcap.sh`** (specification section
  18, roadmap slice S18). Two thin wrappers handle the environment concerns that
  belong outside the binary: the PowerShell wrapper verifies elevation and
  relaunches elevated when needed, detects the capture driver and reports the
  download location when it is absent (installing nothing), filters virtual
  adapters from the interface list, and expands an output-path template; the Bash
  wrapper bridges the WSL2 subsystem boundary, invoking the native Windows binary
  through interop and translating paths in both directions, and reports capture
  unavailable and exits 1 on a Linux host with no reachable binary.
- **The wrappers are thin and honest** (constitution P-7). They contain no capture
  logic and never parse fragcap's human-readable output; they react to the
  section 17.5 structured event stream, which they add (`--json`) to every
  invocation, and pass unrecognized options through to fragcap unchanged. A
  `--dry-run` (`-DryRun`) seam prints the assembled invocation and exits without
  capturing, which previews the expanded template and the pass-through.
- **Both wrappers are held to their ShruggieTech house standards in CI.** A new
  `cargo xtask wrappers` gate runs the vendored PowerShell compliance checker on
  `Invoke-FragCap.ps1`, an authored Bash structural checker on `fragcap.sh`, a
  syntax check of each, and each script's help and dry-run, returning the 0/1/2
  contract. It is part of `cargo xtask ci` and the `ci.yml` workflow, so a wrapper
  that drifts from its standard fails the build. This is the section 18.4 gate,
  previously unmet: no shell-lint ran before.
