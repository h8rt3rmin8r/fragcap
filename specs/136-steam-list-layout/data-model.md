# Data Model: Readable Steam Title Listing

S136 adds no persisted data and no schema migration. The model below exists only for deterministic human presentation.

## Installed title row

An existing enumerated Steam title paired with its existing local identity.

| Field | Meaning | Constraint |
| --- | --- | --- |
| app id | Steam application identity | Preserved from enumeration and used for numeric ordering ties |
| name | Installed title display name | Preserved in JSON; only layout controls receive visible human representation |
| identity | Positioned, unpositioned, or unregistered | Resolved by the existing exact `steam:<app_id>` anchor join |

## Human presentation row

The four complete display strings calculated once from an installed title row.

| Field | Value rule |
| --- | --- |
| APP ID | Human-safe representation of app id |
| NAME | Human-safe representation of title name |
| STATE | `registered` or `unregistered` |
| TARGET | `<handle> (#<position>)`, `<handle> (no position)`, or empty |

## Listing layout

| State | Entry condition | Output form |
| --- | --- | --- |
| Aligned | Maximum header-or-row table width is at most the selected width | One header plus one padded row per title, with two-space column gaps |
| Vertical | Maximum complete table width exceeds the selected width | One listing heading plus four labeled lines per title, with a blank line between records |

The layout is selected once after rows are sorted. It does not change row identity, ordering, or structured output.

## Selected display width

Interactive stdout uses the reported terminal width clamped to 40 through 80 display columns. Redirected output, unavailable measurement, and unsupported hosts use 80. Width is presentation input only and is never persisted.

## State transitions

None. S136 reads existing title and target state without changing it.
