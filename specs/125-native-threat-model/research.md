# Research: Native Deep Capture Threat Model

## R-1: Canonical representation

**Decision**: Use versioned JSON as the canonical threat registry and Markdown
for reviewer guidance.

**Rationale**: The existing workspace already validates versioned JSON evidence
with serde_json in xtask. JSON permits deterministic closed-vocabulary and
cross-reference checks without adding a parser package. Markdown remains the
readable entry point but cannot silently satisfy the executable gate.

**Alternatives rejected**: Markdown tables alone cannot reliably enforce field
or test completeness. Rust constants would make security review unnecessarily
code-centric and couple data changes to compilation.

## R-2: Executable evidence identity

**Decision**: Reference each negative test as a tracked Rust file plus exact
function name, and require the validator to find a test attribute and reject an
ignored test.

**Rationale**: Function-only scanning can collide across files and can mistake
a helper or comment for a test. Path plus function is reviewable, deterministic,
and survives test-binary naming differences.

**Alternatives rejected**: Running every named test individually would create a
slow process-per-row gate. Cargo test already executes the suite; this gate
proves ownership and lets ordinary CI prove execution.

## R-3: Protocol review currency

**Decision**: Compare the registry's closed protocol-family list with the
facade's exhaustive classification vocabulary.

**Rationale**: S120 makes that vocabulary the shipped product authority. A
change there is exactly the event that must force a security review.

## R-4: Dependency review currency

**Decision**: Compare a sorted registry inventory with every direct normal and
Windows-target dependency declared by `fragcap-proxy`, excluding dev-only
dependencies.

**Rationale**: Direct runtime edges express the proxy's reviewed attack surface.
Lockfile transitives remain governed by later supply-chain work (#328), while a
direct promotion or addition must update this model now.

## R-5: Residual risk

**Decision**: Accept no implicit residual-risk disposition in S125.

**Rationale**: The user authorized implementation, not acceptance of an
unmitigated high-risk condition. Existing negative evidence covers the shipped
high-risk paths, and any newly discovered gap must receive a test.

## R-6: Scope boundary

**Decision**: Review functionality shipped through S124 and leave fuzzing,
performance, Windows integration, packaging, supply-chain automation, produced
artifact validation, and final completion to #324 through #334.

**Rationale**: Those issues require different environments and acceptance
authorities. Pulling them into S125 would weaken rather than strengthen the
review boundary.
