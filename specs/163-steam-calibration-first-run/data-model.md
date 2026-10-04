# Data Model: S163 Steam Calibration First Run

## Owned platform authority

Fields: prepared executable path, expected basename, launch receipt PID, expected parent PID, lower and upper launch timestamps, bound process node identity. Before a receipt, no platform stage can bind. A matching creation event binds once; an exit or identity conflict does not transfer authority to another PID. The raw observed image remains in the process tree.

## Client setup evidence

Fields: target stable ID, Steam app ID, install root, launch hint, observed descendant process instances, socket table row counts, selected executable, evidence source (`observed-socket-owner` or `operator-declaration`), observation completeness. Zero or several candidates do not select a client. A socket row does not prove packet traffic. A manual declaration has no observed socket claim.

## Client-authoring plan

Existing canonical plan fields remain: target and discovery snapshots, proposed executable, resulting launch entry, local store identity, operation, and no-effect declarations. The selected evidence source and candidate identity become plan inputs. Confirmation is followed by fresh resolution and compare-and-swap persistence. Decline, invalid input, interruption, and drift cause no write.

## Session diagnosis

Fields: known failed stage, launch receipt and binding state, final-client acquisition state, target packet retention, proxy accepted count, exact compatibility writes, workflow revision/state, and cleanup results. Each can be known or unavailable. Artifact presence is reported separately and does not imply capture success.
