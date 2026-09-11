# Quickstart: Durable Guided Calibration Resume

## 1. Verify the store migration and checkpoint model

```bash
cargo test -p fragcap-targets calibration_workflow
cargo test -p fragcap-targets a_v10_store_gains_empty_calibration_workflows
```

Expected: a version-10 store migrates additively, existing targets and facts remain
unchanged, and workflow creation, validation, revision, and cascade behavior pass.

## 2. Verify command and event contracts

```bash
cargo test -p fragcap-cli --lib calibrate
cargo test -p fragcap-cli --test cli_calibrate
cargo test -p fragcap-cli --test cli_reference
```

Expected: fresh starts emit workflow identity, resume accepts no competing target or
protocol input, explicit pauses start no effect, and stable output contains the
checkpoint projection.

## 3. Verify cross-process controlled resume

Start a controlled workflow with a bounded protocol candidate, stop it at an
incomplete or explicit pause boundary, then invoke the emitted `--resume` command in
a new process. Verify:

- the same target and protocol intent are retained
- current facts and process state are reread
- the next plan identifier differs from every earlier plan
- no earlier response is accepted as authority
- attempt bundle ordinals continue without reuse

## 4. Verify crash and drift boundaries

Exercise an in-flight checkpoint, a changed target row, an unsupported record
version, and a stale revision. Every case must stop before a new effect, and the
in-flight case must be described as interrupted rather than completed.

## 5. Run repository verification

```bash
cargo xtask ci
```

Then run UTF-8, no-BOM, LF, punctuation, mojibake, dependency, diff, and clean
worktree checks described by the repository conventions.
