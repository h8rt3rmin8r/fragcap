# Validation Guide: S181

## Source qualification

Use Rust 1.96.0, MSRV 1.88, Node 24, and pnpm 10.26.0. Windows console child processes require hidden redirected noninteractive execution.

```sh
cargo fetch --locked
cargo xtask ci
cd site
pnpm install --frozen-lockfile
pnpm audit --audit-level low
pnpm test:unit
cd ..
cargo xtask docs build
cd site
pnpm test:accessibility
```

Run pinned cargo-deny 0.20.2 against the root full policy and each isolated graph in [data-model.md](data-model.md), with locked all-feature resolution and fresh advisory data.

## Hosted qualification

Require applicable PR and final-main checks. Dispatch docs on final main with `cold_cache=true`; require empty isolated Cargo home/target, locked fetch, and offline root validation. Require successful Pages deployment and fresh audit/links.

## Final reconciliation

Read back Dependabot alerts and open bot PRs. Require zero outstanding items, successful graph reevaluation, explicit #510 supersession, and #511 evidence bound to final main.
