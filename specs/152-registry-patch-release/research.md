# S152 Research

## Environment Authority

**Decision**: Reuse `crates-io`, require owner `h8rt3rmin8r` (verified user id 46768484), allow self-review, retain administrator bypass and add one tag-only `v*` allowance.

**Rationale**: Live read-only metadata confirms environment id 21294112472 with no protection, null deployment policy and administrator bypass allowed. The owner is also the initiator, so self-review prevention blocks the sole reviewer. A named environment is not proof of a gate. [GitHub environment reference](https://docs.github.com/en/actions/reference/workflows-and-actions/deployments-and-environments) describes separate required review and bypass behavior; [REST environment API](https://docs.github.com/en/rest/deployments/environments) and [tag-policy API](https://docs.github.com/en/rest/deployments/branch-policies) support scoped reviewer and tag policy changes.

**Alternatives considered**: Owner-only self-review prevention is unapprovable; adding a reviewer invents authority; branch main allowance does not match tag-triggered publication; trusting documentation repeats the verified gap. The current REST request and GraphQL mutation schemas omit administrator bypass, although readback exposes it. Treat an ignored API field as failure and retain a supported operator settings step if required. Never weaken the guard to hide this limitation.

**Updated evidence and operator decision**: A bounded REST request accepted the undocumented bypass field, and separate public readback confirmed its effect. The operator then explicitly rejected disabling bypass; S152 restored and verified true while preserving normal owner review and the tag-only rule. Current policy follows that instruction, not the research agent's initial false recommendation. Any later deliberate owner bypass remains distinct from reviewer approval; no deployment approval or bypass occurred here.

## Verification Boundary

**Decision**: Use bounded public API documents with a pure xtask validator and fresh fetching at identity and publish boundaries.

**Rationale**: Public read endpoints work with read-only workflow authority and need no environment secrets. Keep ordinary CI offline and fail closed on more than one complete 100-entry page rather than develop a generic paginated policy engine for one rule. Harmless service fields are allowed, unknown protection types and incomplete policy facts are refused.

**Alternatives considered**: Static environment-name checking is insufficient; repository snapshots alone become stale; approving via the owner token violates the manual approval boundary; a new token or generic network client is unnecessary.

**Final-review permission correction (20:35 UTC)**: Primary [environment read documentation](https://docs.github.com/en/rest/deployments/environments#get-an-environment) and [allowance read documentation](https://docs.github.com/en/rest/deployments/branch-policies#list-deployment-branch-policies) specify Actions read for authenticated installation-token access. Public unauthenticated access does not establish the authority of `github.token` used in the workflow. [Workflow permission semantics](https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax#jobsjob_idpermissions) make omitted scopes none when a job overrides permissions. Explicit Actions read in defaults and the release override is the least-privilege reproducible path, with negative tests covering future job overrides. No third review is requested.

## Patch Preparation

**Decision**: Prepare `release/0.10.1` using the installed version-only cargo-release command and existing release generators, without orchestrator execution, tags or publication.

**Rationale**: Pinned `release.toml` permits only release branches. Existing scripts define embedded writer assertions and three synthetic golden generators; the PowerShell implementation additionally identifies portable conformance and staged Windows assertion surfaces. Independent-review record fixtures and candidate identity must match the candidate version, but the review state remains not-started. Published v0.10.0 identity, assets, history and unsigned policy remain immutable.

**Alternatives considered**: Updating actual publication to an unshipped version violates P-9/P-11. Running a whole release orchestrator from a dirty feature branch fails its preconditions and uses a Windows launcher not verified hidden. Aggregate cargo-release commands are broader than the version-only step required.

## Research Coordination

The plan skill explicitly requested independent read-only research agents. Environment research verified API limitations and public read capability; release research identified candidate identity and static workflow-gate surfaces. No agent modified repository files, environment settings, secrets, issues or deployment approval during research.
