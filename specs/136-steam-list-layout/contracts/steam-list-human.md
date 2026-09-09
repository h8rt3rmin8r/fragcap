# Contract: Readable `fragcap steam list` Human Output

## Shared rules

- Human title output contains no tab characters.
- Rows retain case-insensitive title-name order with numeric app id as tiebreak.
- `APP ID`, `NAME`, `STATE`, and `TARGET` values are complete and associated with one title.
- Tab, carriage return, and line feed inside a human value are shown as `\\t`, `\\r`, and `\\n`; all other characters are preserved.
- Positioned target text is `<handle> (#<position>)`; unpositioned target text is `<handle> (no position)`; an unregistered target is empty.
- Layout selection uses visible display-cell width and occurs once for the complete listing.

## Aligned layout

When the widest complete table line fits the selected width, output is one header and one row per title with two ASCII spaces between padded columns:

```text
APP ID   NAME                                  STATE         TARGET
1190600  Captain Hardcore                      registered    captain_hardcore (#4)
1203620  Enshrouded                            unregistered
228980   Steamworks Common Redistributables    registered    steamworks_common (no position)
```

The final `TARGET` field is not padded. Earlier columns are padded by visible display cells to the maximum of header and row values.

## Vertical layout

When the complete table would exceed the selected width, the complete result uses labeled records:

```text
INSTALLED STEAM TITLES

APP ID: 1190600
NAME: Captain Hardcore: The Complete Collection
STATE: registered
TARGET: captain_hardcore (#4)

APP ID: 1203620
NAME: Enshrouded
STATE: unregistered
TARGET:
```

No value is wrapped or truncated. A blank line separates records.

## Width policy

An interactive reported width is clamped to 40 through 80 display columns. Redirected output, unavailable measurement, and unsupported-host measurement use 80.

## Compatibility boundary

The existing zero-title sentence, store-unavailable warning, JSON Lines records, diagnostics, field-presence rules, exit codes, ordering, and read-only storage behavior do not change. The original S067 contract is updated to point to these human layout rules while retaining its JSON contract.
