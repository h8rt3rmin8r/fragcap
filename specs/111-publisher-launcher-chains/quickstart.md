# Quickstart: Validate Managed Publisher-Launcher Chains

## Prerequisites

- Work from `codex/111-publisher-launcher-chains`.
- `.specify/feature.json` points to `specs/111-publisher-launcher-chains` and remains unstaged.
- Use the pinned Rust 1.96.0 toolchain.
- Real managed launch is Windows-only. Offline profile and process-timeline tests remain available without Npcap, elevation, a publisher account, or a game.

## 1. Validate preparation and value contracts

```powershell
cargo test -p fragcap-targets
cargo test -p fragcap managed_launch
```

Expected: direct launch remains unchanged; an exact launcher-to-client value becomes a publisher launch; missing, duplicate, reordered, or escaping stages refuse before effects.

## 2. Validate shared Capture preparation

```powershell
cargo test -p fragcap-cli target_resolve
cargo test -p fragcap-cli --test cli_capture
```

Expected: Capture synthesizes the declared multi-stage profile with exact canonical-path predicates, permits one process per role, keeps the finite acquisition deadline active until the sole terminal client binds, and preserves existing one-client, Steam, and unresolved-target behavior.

## 3. Validate Deep Capture policy and controlled outcomes

```powershell
cargo test -p fragcap deep_capture
cargo test -p fragcap-cli --test cli_deep_capture
```

Expected: only `publisher-launcher-cold` can pass the launch-case gate. Warm, game-start-clean warm, ambiguous, escaped, missing, and timed-out cases remain distinct and never produce routing success without final-client socket evidence.

## 4. Run the repository gates

```powershell
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --workspace --locked
cargo run --package xtask -- ci
cargo run --package xtask -- msrv
```

Expected: all gates pass, no new lockfile package appears, documentation and specification checks remain green, and the native HTTP/TLS conformance evidence does not drift.

## 5. Review the final change

Review the diff for issue #307 only. Confirm there is no target process handle, shell, executable mutation, global proxy setting, uncounted loss, target-store schema fork, new dependency, em dash, en dash, BOM, mojibake, or staged `.specify/feature.json`. Confirm issues #308 through #334 remain open as applicable.
