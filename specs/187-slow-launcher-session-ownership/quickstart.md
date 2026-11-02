# Quickstart: S187 controlled validation

1. Check the S187 feature selector and branch: `.specify/feature.json` must name `specs/187-slow-launcher-session-ownership`, and `git branch --show-current` must name `codex/s187-slow-launcher-session-ownership`.
2. Run focused deadline tests in `fragcap-cli` and `fragcap` and verify 120000/600000 millisecond plans, explicit 300000 millisecond launch, unsupported no-effect refusal, and the full requested observation span.
3. Run controlled Steam chain, process trace, connection correlation, and shutdown tests. The fixture must exercise slow and absent client, a helper born during release, same-name unrelated process, missing or ambiguous socket evidence, and exact recovery obligations.
4. Run `cargo xtask ci` in the foreground. Run isolated Windows package certification with the installed binary and controlled slow launcher under the runner account's documented default target and session locations; read back the actual canonical paths. The test must not take active desktop focus or launch a real game.
5. After push, verify exact-head CI and external review threads. At most one additional `@Codex` review request is permitted. Handoff only after every applicable check is green and every comment is answered or resolved.

Source-level controlled fixtures may use temporary directories. They do not establish installed-product default-path behavior; the installed-binary certification row requires constitution 1.8.0 evidence of the actual canonical target and session locations.
