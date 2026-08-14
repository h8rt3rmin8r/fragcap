# Research: extcap registration

Resolved from the existing code; no external research required. The one materially
irreversible choice (the CLI shape) is recorded here in full because it was the
slice's stated risk (R-5).

## D-1: CLI shape for install/uninstall (R-5)

**Decision**: Add the two operations as an optional subcommand on the existing
`extcap` command: `ExtcapArgs` gains `#[command(subcommand)] action:
Option<ExtcapAction>`, where `ExtcapAction` is `Install(ExtcapInstallArgs) |
Uninstall(ExtcapInstallArgs)` and `ExtcapInstallArgs` carries `--dir`.

**Rationale**: This gives the operator the requested `fragcap extcap install`
spelling with the lowest regression risk, for three concrete reasons found in the
code:
- `ExtcapArgs` is entirely `#[arg(long)]` flags plus a flattened `OfflineArgs`.
  There is no positional argument for the `install`/`uninstall` barewords to
  collide with, so clap disambiguates cleanly: a leading bareword is the
  subcommand, a leading `--extcap*`/`--capture`/`--fifo` means no subcommand.
- The top-level `route_extcap` shim (`lib.rs`) injects the `extcap` subcommand
  only when the first token is not a known subcommand and the invocation leads
  with an extcap protocol flag. `fragcap extcap install` leads with the known
  `extcap` token, so the shim passes it through unchanged; the bare protocol
  forms (`fragcap --extcap-interfaces`, etc.) still route as before. Neither form
  is affected by adding the subcommand.
- The four protocol invocations are already covered in both forms by
  `cli_extcap.rs` (`a_direct_extcap_interfaces_invocation_is_routed` and
  siblings, added by the PR #34 Codex review). This slice adds an explicit
  parser-regression assertion so a future change to `ExtcapArgs` that broke the
  no-subcommand form fails loudly.

**Alternatives considered**: separate top-level commands (e.g. a `register` noun)
- zero risk to the protocol path but deviates from the requested spelling and
splits a naturally-grouped surface; rejected because the optional-subcommand
shape is unambiguous here. A positional `mode` argument on `ExtcapArgs` - would
genuinely collide with the protocol parsing; rejected.

## D-2: Single-sourced binary name (R-6)

**Decision**: Move the extcap binary name to a public `paths::EXTCAP_BINARY`
constant (`fragcap.exe` on Windows, `fragcap` elsewhere) and have both the
install command and the doctor probe reference it.

**Rationale**: The doctor probe (`doctor/probe.rs`) currently has a private
`EXTCAP_BINARY` const it checks for. If install wrote a different name, doctor
would never report the integration as installed. Single-sourcing the constant
makes the tool and the readiness check agree by construction. This is a
behavior-preserving refactor: doctor still probes the same name and location, so
it is within the slice's scope-out ("no change to how doctor probes").

## D-3: --dir override and the test seam

**Decision**: `--dir` overrides the target directory; when omitted the command
uses `paths::extcap_dir()`, which already honors the `FRAGCAP_EXTCAP_DIR`
environment override. Tests use `--dir <tempdir>` (and, for the doctor
end-to-end, `FRAGCAP_EXTCAP_DIR` so the probe reads the same directory).

**Rationale**: `paths::extcap_dir()` already has the env seam; `--dir` gives the
operator an explicit knob and the test a direct argument. No new mechanism.

## D-4: MSI registration mechanism (DEFERRED to a dedicated installer slice)

**Decision (2026-08-14, operator)**: Split the Windows installer option out of
this slice into its own slice, and offer both scopes there (per-user by default;
a documented machine-wide option for administrators). This slice ships the tested
CLI command and its documentation only, so no WiX and no release-adjacent artifact
is touched here.

**Why the split**: the installer option cannot be built or install-tested in this
environment (no WiX toolchain), it touches the just-stabilized release installer,
and operator feedback grew it (a real installer checkbox, an at-install note that
the registration is per user, and the "otherwise run `fragcap extcap install`"
guidance). That is enough surface to deserve its own WiX-validated slice rather
than riding unbuilt into this PR.

**Design carried forward for that slice** (recorded here so it is not re-derived):
run the installed binary's own `extcap install` as a deferred, user-impersonated
WiX custom action so the per-user target resolves to the installing user's
profile rather than SYSTEM (a default non-impersonated action would register into
the wrong profile). Offer a machine-wide option by registering into Wireshark's
system extcap directory when Wireshark is detected. The per-user path is already
reachable from the command; the machine-wide path is reachable now via `--dir`,
which the CLI reference documents, so an administrator is not blocked in the
meantime.

## D-5: Where the command result is written

**Decision**: The register/uninstall result line (the destination path, or the
no-op notice) goes to standard output, like other command results; errors go
through the emitter to standard error, per FR-019.
