# Quickstart: Validate S202

1. Confirm `main` contains merged S197 through S201, public v0.10.14 remains the latest release, and the release branch starts from the exact accepted tree.
2. Run the repository's release-preparation dry run. Assemble v0.11.0 candidate identities, changelog, notes, and documentation, then run complete local release and repository checks.
3. Open the official candidate PR. Confirm exact-head Windows packages, applicable checks, actual review findings, and owner-ready merge state. Do not tag or claim publication at this point.
4. After the owner merges and authorizes publication, verify the annotated tag selects that merge. Watch the tagged workflow and normal registry approval to completion.
5. Independently verify six public files, three checksum sidecars, the package report, and ten registry archives. Only then advance published-version records and site guidance through a reviewed records change.

The [release boundary](contracts/publication.md) defines what each stage proves. A private-title run and #580 research are separate from this sequence.
