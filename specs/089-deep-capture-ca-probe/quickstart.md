# Quickstart: Verify the CA Trust-State Probe

## Automated Verification

```powershell
cargo test -p fragcap-cli doctor
cargo test -p fragcap-cli --test cli_doctor
cargo xtask ci
```

The unit tests inject certificate inventories. They do not install or remove host
certificates.

## Windows Manual Demonstration

Run the shipped doctor against a session bundle produced through the documented
Deep Capture flow and record the date and CA check for:

1. before trust is installed;
2. while the session CA is in current-user Root;
3. after session cleanup removes it.

If an authorized disposable target, mitmdump installation, or explicit permission
to mutate current-user trust is unavailable, do not manufacture the states. Record
the exact missing prerequisite and demonstrate only the read-only state available.

### Demonstration Record

- **2026-08-28, before/cleaned-equivalent**: `cargo run -q -p fragcap-cli --
  doctor --json` reported `no fragcap Deep Capture CA trust found` with status
  `ok` on Windows.
- **Trusted and post-removal transition not run**: mitmdump is installed, but this
  development run had no operator authorization to install a new certificate into
  the real current-user Root store solely to exercise a diagnostic. The injected
  inventory tests cover the trusted and cleaned transitions without host mutation.

## Safety Inspection

Confirm the ordinary probe uses only certificate enumeration/property APIs. Trust
installation and deletion remain confined to explicit Deep Capture lifecycle code
and the confirmation-gated fix performer.
