# Validation Guide: Causal Launch-Chain Coverage

## Controlled source checks

Run `cargo xtask ci` from the repository root. Its relevant regressions must cover gate reasons and packet conservation, default and explicit scope, stale resume, exact causal-generation membership, and post-client helper release. Read failures rather than treating unrelated passing tests as acceptance.

## Hosted Windows checks

At the official PR head, wait for CI, Windows integration, package certification, and external reviews. The controlled cold fixture includes platform, publisher, final client, updater, unexpected utility, unrelated same-name process, and mixed exiting/surviving helpers. Compare packet file, process trace, application stream, manifest, release journal, terminal result, and Doctor against [the evidence contract](contracts/evidence-contract.md).

## Acceptance observations

- Default `auto` selects full-chain application inspection before effects; explicit `game` remains available while packet coverage stays full chain.
- Packets from exactly proven descendants appear before client acquisition, during client observation, and until route release, with zero unrelated-process admission in controlled cases.
- Every withheld packet and correlated flow has the actual reason, and all loss equations reconcile.
- A stale game-only fact or resumed policy cannot qualify full-chain work.
- Proven live helpers exit under finite authority or produce exact incomplete-release evidence. Ambiguous and unrelated processes receive no control.
- Direct raw game sockets and launcher TLS refusals retain separate observed boundaries; no positive inspection result is inferred.

No private-title run or new release is required for S197 implementation acceptance. A later installed-product observation uses published bytes and documented default paths, with any new failure filed against that release.
