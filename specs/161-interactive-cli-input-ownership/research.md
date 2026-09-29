# Research: S161 Interactive CLI Input Ownership

## R1: Rust stdin lock

**Decision**: Hold `std::io::Stdin` across dispatch and take `StdinLock` only within each authorization read.

**Evidence**: [Rust standard library stdio source](https://github.com/rust-lang/rust/blob/main/library/std/src/io/stdio.rs) defines stdin as one static `Mutex<BufReader<StdinRaw>>`; `Stdin::read_line` calls `self.lock().read_line(buf)`. The current `run` and `run_with` retain `stdin.lock()` through dispatch, while Doctor calls `stdin().read_line()`. Re-entering that mutex blocks before any console read. The tagged `v0.10.2` source contains both calls.

**Alternatives considered**: Route one locked reader through all commands (broader interface change), or use a second OS handle (would split buffering and complicate exact authorization). Short lived locking is the narrow fix.

## R2: Authorization contract

**Decision**: Preserve the current bounded `fill_buf` and `consume` algorithm under the short lived guard.

**Evidence**: `StdinAuthorizationInput::read_response` limits the response to the plan identifier plus two bytes, checks for a complete line, and marks extra already buffered bytes. Deep Capture and calibration consume one response per plan and independently reject incomplete or mismatched input. The shared global `BufReader` survives guard release, retaining subsequent bytes without creating an alternate input source.

**Alternatives considered**: Replace plan input with `read_line` (would discard boundedness and exact extra-input detection), or keep the command-wide lock (recreates #437).

## R3: Honest prompt failures

**Decision**: Treat Doctor prompt write, flush, EOF, and read failure as an error that stops further actions; distinguish a completed empty or nonaffirmative line as No. Apply the same no-false-decline rule to warm restart. Flush the target question and propagate write/flush/read errors.

**Evidence**: `ActionConfirm::confirm` returns only bool; `ConsoleConfirm` converts `Err` to false and `drive_actions` prints `skipped`. The master specification section 26.3 requires honest outcomes and explicit confirmation before effects. `CliError::Failure` maps an expected I/O failure to exit 1.

**Alternatives considered**: Keep error-as-No for fail-closed effects, but that falsely attributes a broken input channel to an operator decision. Continue after error, which could execute later actions without reliable prompting.

## R4: Regression evidence

**Decision**: Test both the global-lock lifetime and prompt outcome/effect mapping with bounded controlled tests. Exercise Doctor through the integrated headless terminal when an action is offered.

**Evidence**: Existing Doctor tests use `ScriptedConfirm`; they never instantiate `ConsoleConfirm` through the production input owner. A unit test that verifies another stdin lock can be acquired while the owner object lives fails with the previous design and passes after the correction. Independent prompt tests cover exact decisions and zero performer calls after failure.

**Alternatives considered**: A manual Windows Terminal run is useful field evidence but cannot be the only completion gate. A naive subprocess with redirected pipes cannot pass Doctor's terminal gate.

## R5: Print Screen

**Decision**: Record the focus-dependent hotkey observation separately from the confirmed input defect. Do not claim a hotkey correction in S161.

**Evidence**: The operator reported Print Screen fails while the elevated Terminal has focus and works with another app focused. A [Microsoft elevated-app report](https://learn.microsoft.com/en-us/answers/questions/3916266/snippet-tool-does-not-work-on-admin-app) describes a similar symptom. The inspected Doctor path has no keyboard hook, hotkey registration, or console mode change. No controlled evidence currently connects the hotkey to the stdin mutex.

**Alternatives considered**: Treat shared timing as shared cause (unsupported), or ignore the operator report (would lose relevant context).
