# Quickstart: Doctor Progress And Timing

## Focused Tests

```powershell
cargo test -p fragcap-cli --test cli_doctor
cargo test -p fragcap-cli doctor
```

## Interactive Checks

Run a human doctor report from a terminal:

```powershell
cargo run -p fragcap-cli -- doctor
```

Expected:

- Progress appears on stderr before the final report.
- The final report still appears on stdout.

Run with timings:

```powershell
cargo run -p fragcap-cli -- doctor --timings
```

Expected:

- Completed progress lines include elapsed milliseconds.
- The final report body remains unchanged.

## Suppression Checks

```powershell
cargo run -p fragcap-cli -- doctor --json
cargo run -p fragcap-cli -- doctor --timings > doctor.txt
```

Expected:

- JSON output contains only the existing doctor JSON records.
- Redirected human output matches the existing report format.
- Progress and timings are absent when stdout is redirected.

## Full Gate

```powershell
cargo xtask ci
```
