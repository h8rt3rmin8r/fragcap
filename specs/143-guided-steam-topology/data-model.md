# Data Model: Guided Steam Client Setup

S143 adds one transient plan, one typed conditional-update result, and two additive CLI events. It changes no SQLite, compatibility-fact, bundle, manifest, capture, or discovery schema.

## SteamClientSetupEligibility

| Field | Meaning | Rule |
| --- | --- | --- |
| target | Exact stored `TargetEntry` | Must carry a positive canonical Steam anchor and `launch_entries == None` |
| app id | Positive Steam application identifier | Parsed only from the canonical target anchor |
| candidate | Current discovery candidate | Exactly one candidate with the same Steam application identifier |
| install authority | Stored and discovered install roots | Both present and exactly equal |
| executable proposal | Candidate `executable_hint` | Verbatim value must identify one suitable Windows executable image |
| decision | preserve, unavailable, ambiguous, or offer | Only offer can produce a plan |

## SteamClientSetupPlan

| Field | Meaning | Rule |
| --- | --- | --- |
| schema | `fragcap.steam-client-setup-plan.v1` | Exact version string |
| operation | `author-target-client-if-unchanged-v1` | Names the shared conditional target operation |
| local store | Effective absolute destination path | Canonical plan authority |
| target | Complete planned target row | Every persistent field represented, including nulls and carried JSON |
| candidate | Complete canonical candidate projection | Includes source, identity, name, fidelity, classification, evidence, coverage, install root, folder name, and executable hint |
| discovery | Conserved account plus sorted warnings | Makes incomplete acquisition visible even on the offered path |
| proposed executable | Original candidate value | Never normalized or silently repaired |
| resulting launch declaration | One resolved client entry | Derived only for display and conditional persistence after attestation |
| no-effect boundary | Explicit list of effects not authorized | Separates setup from a later session |
| plan id | `steam-client-setup-v1:` plus digest | Domain-separated over compact canonical JSON |

## SteamClientSetupOutcome

| Field | Meaning | Rule |
| --- | --- | --- |
| plan id | Setup plan being decided | Always present after a plan is emitted |
| status | `declined`, `closed`, `invalid`, `interrupted`, `drifted`, `changed`, `applied`, or `failed` | Independent from registration and calibration status |
| reason | Stable machine reason | Human prose is a projection |
| target id | Durable target identifier | Present for every emitted plan outcome |
| continued | Whether S139 proposal evaluation resumed | Never implies a session started |

## AuthorTargetClientOutcome

| Variant | Meaning | Store effect |
| --- | --- | --- |
| `Applied` | Current row exactly matched the planned row | Sets resolved client declaration and authored fidelity |
| `Changed` | Row exists but any planned field differs | None |
| `Missing` | Durable row no longer exists | None |

## State Transitions

```text
durable-target-resolved
  -> existing-launch-present -> existing-S139-proposal
  -> non-steam-or-invalid-anchor -> existing-S139-limitation
  -> missing-launch-steam -> bounded-discovery
      -> no-or-multiple-candidate -> existing-S139-limitation
      -> install-or-executable-ineligible -> existing-S139-limitation
      -> exact-proposal -> setup-plan-emitted
          -> declined-or-invalid -> unchanged target
          -> confirmed -> fresh-target-plus-discovery
              -> rebuilt-plan-differs -> drifted
              -> rebuilt-plan-matches -> conditional-store-update
                  -> missing-or-changed -> unchanged target
                  -> applied -> durable-target-reread -> existing-S139-proposal
```

Registration, setup, and session plans are sequential independent authorities. No setup transition creates a new target, starts a session, or appends a compatibility fact.

## Validation Rules

- Canonical target projection includes every `TargetEntry` field in a fixed object shape and preserves carried JSON values exactly.
- Candidate projection and discovery projection use the S142 canonical ordering rules; evidence and warnings sort deterministically.
- Discovery accounting must conserve before a plan can be created.
- The target anchor must be exactly canonical `steam:<positive-u32>`.
- Stored and candidate install roots compare exactly and remain unmodified.
- The executable proposal is trimmed only to decide emptiness; the bound and stored value is verbatim.
- The executable must have one `.exe` image component and contain no quotes, URI marker, command placeholder, shell separator, or parent traversal.
- Interactive affirmative input is a complete case-insensitive `y` or `yes` line; every other complete line declines.
- Structured input is exact plan id plus one newline and constant-time compared.
- The post-confirmation plan must be byte-equivalent to the emitted canonical plan.
- The immediate transaction compares the complete current target to the planned target before changing launch declaration and fidelity.
- The post-update stable-id lookup must return the exact expected authored result before calibration continues.
