# Release Requirements Checklist: S202 v0.11.0

**Purpose**: Test whether S202 requirements cover the release journey and evidence boundaries.

**Created**: 2026-11-10

**Feature**: [spec.md](../spec.md)

## Candidate Identity

- [x] CHK001 Does the spec name every accepted source change since the prior public release and require one release identity?
- [x] CHK002 Does it separate candidate version claims from current-public version claims?
- [x] CHK003 Does it require both installer and portable package identities to agree with the candidate?
- [x] CHK004 Does it require all pending release fragments to enter the changelog exactly once?

## Publication And Recovery

- [x] CHK005 Does it place merge, tag push, and registry approval under the owner's control?
- [x] CHK006 Does it distinguish a GitHub release from completed registry publication?
- [x] CHK007 Does it require independent public-file and registry readback before current-version pointers move?
- [x] CHK008 Does it define the prior verified release as the current record if publication is incomplete?

## Documentation And Review

- [x] CHK009 Does it require current first-use and compatibility guidance to identify the actual public version after publication?
- [x] CHK010 Does it preserve routing, trust, and plaintext as separate outcomes in public claims?
- [x] CHK011 Does it require a response to every actual review finding without inventing an absent bot verdict?
- [x] CHK012 Does it keep #580 and future private-title observations out of the release gate?
