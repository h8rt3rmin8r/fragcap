# Research: Deep Capture Bundle and Artifact Reference

## Use the Existing Output Page as the Contract Home

**Decision**: Rewrite `site/content/docs/reference/output-formats.mdx` in place, preserving its URL and ordinary Capture material before adding the Deep Capture bundle contract.

**Rationale**: The page is already linked as the output authority. Adding a competing bundle page would force readers and later maintainers to decide which output reference is canonical, while renaming it would break stable links.

**Alternatives considered**:

- Add a separate bundle page: rejected because authority would be split and substantial handling guidance would be duplicated.
- Replace the page with Deep Capture only: rejected because ordinary Capture pcapng and packet JSON Lines remain shipped public contracts.
- Put the complete matrix on the architecture page: rejected because S091 deliberately leaves exhaustive artifact detail to issue #248.

## Prefer Shipped Vocabulary Over the Forward-Looking Superset

**Decision**: Document the four omission reasons emitted by the current finalized-manifest writer: `writer-failed`, `no-http-semantics`, `not-requested`, and `not-produced`. Identify broader reason examples in specification section 13.7 as future vocabulary, not current output.

**Rationale**: Constitution P-11 and issue #248 require a v0.7.0 reference. Publishing tokens the parser never emits would encourage consumers to build against a contract that has not shipped.

**Alternatives considered**:

- Copy every reason listed in specification section 13.7: rejected because `not-observable`, `unsupported-protocol`, `proxy-not-reached`, `certificate-pinned`, and `backend-unavailable` are not current manifest omission tokens.
- Omit exact reason tokens and describe them generically: rejected because the acceptance criteria explicitly require all current reasons and machine consumers benefit from an exact vocabulary.
- Change runtime output to match the broader specification: rejected because S092 is documentation-only and such a schema expansion needs its own implementation slice.

**Explicit deviation**: This slice deliberately narrows the public omission vocabulary from the master specification's forward-looking examples to the values current code emits. The deviation is necessary because the previous prose describes planned breadth, while the public release contract must describe shipped behavior.

## Separate Three Similar Vocabularies

**Decision**: Present manifest omission reasons, application observation `reason` text, and cleanup resource statuses as three distinct contracts.

**Rationale**: They differ in structure and authority. Manifest omissions explain absent artifact roles, application reasons explain one proxy observation or failed packet correlation, and cleanup statuses report resource disposition. Combining them would create false equivalence.

**Alternatives considered**:

- One global reason table: rejected because only manifest omission reasons are a fixed top-level token set.
- Document only manifest omissions: rejected because readers could still mistake `application.unsupported` or cleanup `not-produced` values for the same vocabulary.

## Treat Manifest Sensitivity Labels as Classification, Not Sharing Permission

**Decision**: Reproduce the exact manifest labels (`ordinary`, `sensitive`, and `secret-adjacent`) and add handling guidance that no label means safe to publish without review.

**Rationale**: `capture.fcapng`, `compatibility.json`, `cleanup.json`, and `manifest.json` are currently labeled `ordinary`, but they can still reveal packet payloads, target identity, paths, ports, or operational context. P-9 requires preserving the label while explaining its limit.

**Alternatives considered**:

- Relabel packet truth as sensitive in documentation: rejected because that would disagree with the emitted manifest.
- Treat `ordinary` as public: rejected because it would give unsafe sharing guidance.

## Describe Actual Retention and Cleanup

**Decision**: State that finalized artifacts remain in the bundle until the operator removes them. Explain that session cleanup removes active proxy, port, trust, and private material where possible, while confirmation-gated `doctor --fix` can later remove known sensitive sidecars and unfinished manifests under fragcap-owned session storage.

**Rationale**: The runtime intentionally retains evidence. `doctor --fix` does not erase every completed bundle file, and it preserves manifest ownership evidence when CA trust cleanup is incomplete.

**Alternatives considered**:

- Promise automatic expiry: rejected because no retention policy or timer exists.
- Say cleanup deletes the bundle: rejected because normal session cleanup does not, and later doctor cleanup is selective and confirmed.
- Avoid lifecycle guidance: rejected because sensitivity without disposition guidance is incomplete.

## Keep Synthetic Examples Small and Structurally Honest

**Decision**: Use one shortened manifest example with placeholder target identity, relative artifact paths, non-secret correlation ids, and no key-log content.

**Rationale**: The example should teach the index shape and joins without resembling a complete generated schema or risking local and account material.

**Alternatives considered**:

- Paste a full controlled-test bundle: rejected because it is too large and would duplicate details already covered by tables.
- Show a sample TLS secret: rejected because even a fake line normalizes copying secret-adjacent content into documentation and adds no explanatory value.
- Use a real game name or local path: rejected by the issue's synthetic-data requirement.
