# Contract: S202 Release Boundary

## Candidate

The candidate PR presents one v0.11.0 source tree containing S197 through S201 and release preparation. Its description names local and hosted checks, every actionable review disposition, and the current public v0.10.14 baseline. It does not claim publication. The owner performs final review and merge.

## Tag and public assets

Only an owner-authorized annotated v0.11.0 tag may select the accepted merge. The existing release workflow builds and certifies the exact Windows ZIP, MSI, catalog, and three checksum sidecars, creates the public GitHub release, then waits for normal registry approval. A tag, workflow start, or GitHub release alone does not establish complete publication.

## Independent readback

After tagged jobs finish, independently retrieve all six public files and compare checksums, published API digests, and the certification report. Query all ten versioned registry records, require non-yanked status, and match downloaded archive hashes. Record each exact result before changing current-version pointers.

## Current documentation

Before readback, public documentation identifies v0.10.14 as published and v0.11.0 as candidate. After readback, the reviewed records change advances current public markers, removes S201 candidate wording, keeps Npcap separate and WinDivert bundled, and preserves the distinction between routed traffic and readable application content. Site deployment then confirms the public guidance.
