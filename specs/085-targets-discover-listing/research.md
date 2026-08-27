# Research: Targets Discover Listing

## Decision: Render discovery as a human listing, not a tab dump

**Rationale**: `targets discover` is documented as a read-only inspection command, not as a machine interface. The current tab-separated output has no field names and does not align across variable-width identities. A headed table matches the existing `fragcap targets` surface and makes the command usable without adding a new output contract.

**Alternatives considered**:

- Keep tab-separated output and document it. Rejected because no `--format` or `--json` flag declares a stable machine contract, and human operators still get unlabeled fields.
- Add `--json` in this slice. Rejected because the issue is the human listing. A machine-readable discovery stream needs a separate schema and compatibility contract.

## Decision: Omit classification from the human discovery table

**Rationale**: On a stock barebones catalog, the classification column commonly prints `unknown` for every row. That costs width while teaching the operator little. Source, identity, fidelity, display name, and evidence are the useful human fields in this inspection listing. The underlying discovery candidate still carries classification for registration and internal behavior.

**Alternatives considered**:

- Keep classification with a header. Rejected because it fixes the unlabeled-field problem but keeps a noisy all-unknown column for normal users.
- Print a separate explanation of `unknown`. Rejected because it adds prose to explain a column that does not need to be present in the first place.

## Decision: Render account totals as a labelled block

**Rationale**: The account is the discovery conservation record. `considered` and `produced` should always be visible, non-zero outcome counters should be individually readable, and zero-valued outcomes should not bury the useful signal. Grouping zero names keeps the invariant visible while making unusual outcomes stand out.

**Alternatives considered**:

- Omit zero-valued outcomes entirely. Rejected because the current output names every bucket, and a grouped zero line preserves the bucket set without dominating the output.
- Keep one `account:` line. Rejected because it recreates the exact readability failure this slice fixes.

## Decision: Keep warnings on the existing emitter path

**Rationale**: S082 established that warnings must not contaminate command-result stdout. S085 changes the command-result layout only. Diagnostics remain on the diagnostic stream and retain quiet, silent, and JSON behavior from the emitter.

**Alternatives considered**:

- Inline warnings under discovery rows. Rejected because it would mix diagnostics with result output and undo S082's stream separation.
