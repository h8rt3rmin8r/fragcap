# Quickstart: Validating the Workspace Scaffold

How to confirm this slice does what it claims. Every step is a command whose
output is the evidence; nothing here asks you to take a claim on trust.

## Prerequisites

- `rustup` installed. Nothing else. The repository names the toolchain it
  needs and `rustup` fetches it on first build.
- No capture driver and no software development kit. This slice must build
  without them, and that is part of what these steps verify.

## 1. Build from a clean state

```bash
cargo build --workspace
```

**Expected**: the pinned toolchain is selected automatically and all nine
members compile.

Verifies SC-001, FR-001, FR-003.

## 2. Run the tests

```bash
cargo test --workspace --locked
```

**Expected**: tests pass, including those covering the conventions and
dependency checks. `--locked` proves the committed lockfile is consistent.

Verifies User Story 1 scenario 2.

## 3. Run the full local check set

```bash
cargo xtask ci
```

**Expected**: format, lints, tests, conventions, and dependency direction all
pass. This is the same set the automated check set runs.

Verifies SC-003, FR-013, FR-015.

## 4. Confirm the crate graph matches the architecture

```bash
cargo xtask deps
```

**Expected**: no unexpected edges, no missing edges.

To see the graph rather than only its verdict:

```bash
cargo tree --workspace --depth 1
```

Verifies SC-005, FR-004, and validation rules V-1 through V-4 in
`data-model.md`.

## 5. Prove platform neutrality

```bash
cargo xtask neutral
```

**Expected**: `fragcap-core` builds for a non-host target. If the target is not
installed, the command exits 2 and prints the `rustup target add` line. **It
does not exit 0.** A skipped check reporting success is the failure this
project's constitution names most often.

Verifies SC-006, FR-005, FR-014, and constitution P-2.

## 6. Confirm the checks can actually fail

This is the step that distinguishes a working check set from one whose matchers
never fire. Introduce each violation, confirm the named failure, then revert.

One per check category, matching the four scenarios in User Story 2.

```bash
# a. Misformatted code -> the formatter
printf 'fn  x( )  {   }\n' >> crates/fragcap-core/src/lib.rs
cargo fmt --all -- --check   # expect: non-zero, names the file
git checkout crates/fragcap-core/src/lib.rs

# b. Missing license identifier -> the conventions check
printf 'pub fn y() {}\n' > crates/fragcap-core/src/scratch.rs
cargo xtask lint             # expect: exit 1, names scratch.rs and the rule
rm crates/fragcap-core/src/scratch.rs

# c. Forbidden dependency edge -> the dependency check
cargo add --package fragcap-core --path crates/fragcap-sink
cargo xtask deps             # expect: exit 1, names the core-to-sink edge
git checkout crates/fragcap-core/Cargo.toml Cargo.lock

# d. Platform-specific dependency in core -> the neutrality check
cargo add --package fragcap-core windows-sys
cargo xtask neutral          # expect: non-zero, core no longer builds portably
git checkout crates/fragcap-core/Cargo.toml Cargo.lock
```

**Expected**: each check fails, names the specific violation, and exits
non-zero.

Verifies SC-004 and User Story 2 in full. Without this step, User Story 2 is
asserted rather than demonstrated, which is the failure mode this project's
constitution names most often.

Step (d) needs the non-host target installed:

```bash
rustup target add x86_64-unknown-linux-gnu
```

The copyleft-dependency scenario (US2 scenario 3) is exercised by the license
check rather than here, since it needs a real copyleft crate in the graph. If
the license tool is unavailable, that scenario is recorded as scaffolded rather
than counted as demonstrated.

## 7. Check the minimum supported toolchain

```bash
cargo xtask msrv
```

**Expected**: builds at the declared minimum, **and prints that the result does
not yet constrain anything**, because the workspace has no external
dependencies. The caveat is the point: this check is scaffolded now so it is in
place when it starts to mean something at S02.

Verifies FR-012, FR-012a.

## What this slice does not verify

Stated so that a reader does not infer more than was demonstrated.

- **No workflow has executed.** There is no git remote. The six workflow files
  are validated as well-formed only. Their first real run happens when a remote
  exists.
- **The capture library acquisition step is unexercised.** No crate links
  against it until S09.
- **The minimum-toolchain check is currently vacuous**, per step 7.

These are recorded in the slice's completion report as scaffolded rather than
counted among its passing checks.
