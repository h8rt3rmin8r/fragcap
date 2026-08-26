# Quickstart: Known-Roots Discovery Corrections

## Scenario 1 - Multi-Engine Container Descends

Create a temporary `Games/Collection` directory with synthetic markers for `EngineAlpha` and `EngineBeta`, then place one synthetic title under each immediate child. Run known-roots discovery with the production signature classifier. Assert that `Collection` is absent, both child titles are candidates, `container_descended` is one, and the account conserves.

## Scenario 2 - One Engine Still Stops

Create one title directory with two markers that both map to `EngineAlpha` and a nested decoy directory. Assert the title is emitted once, the decoy is not emitted, and no container counter advances.

## Scenario 3 - Depth-Limited Container Is Visible

Drive the fixture walker with a container verdict at the terminal shallow depth. Assert no candidate is emitted for it, `container_descent_truncated` is one, the warning names only the synthetic path, and the account conserves.

## Scenario 4 - Native Paths Stay Native

Run `FsDirectoryLister` against a temporary volume path and a component-wise joined `Games` root. Assert candidate identity equals install root and neither value contains both slash styles. On Windows, assert no forward slash remains.

## Verification

```powershell
cargo test -p fragcap-targets --test known_roots --locked
cargo test -p fragcap-targets --test detection_walk --locked
cargo test -p fragcap-cli targets --all-features --locked
cargo xtask ci
```
