# Contract: hero listing and the durable row index

**Feature**: S055 | **Date**: 2026-08-18

## Populated listing

```
  #  TARGET                     CAPTURE          KNOWN
  1  the_elder_scrolls_online   ready            launcher-mediated -> eso64.exe
  2  the_division_2             ready            Denuvo, EasyAntiCheat
  3  rust                       ready            EasyAntiCheat (kernel-mode)
  4  stardew_valley             ready            no online mode recorded
  5  some_indie_thing           needs a target   no launch data known

  fragcap capture 1
```

Obligations:
- Columns: 1-based `#`, `TARGET` (handle), `CAPTURE` (`ready` | `needs a target`),
  `KNOWN` (neutral evidence).
- Rows ordered deterministically **by handle** (FR-003), so the same store numbers
  identically every run.
- The final line names the next command (`fragcap capture <n>`), n being the first
  `ready` row when one exists (FR-006).
- CAPTURE derivation (D2): `ready` when a resolved launch chain reduces to a client
  image or a resolved anchor gives one; `needs a target` otherwise.
- KNOWN derivation (D3): evidence products, else launcher-mediation + client image,
  else "no online mode recorded" / "no launch data known". Neutral (FR-021).
- Non-destructive: writes only the listing snapshot; the registered target set is
  byte-identical afterward (SC-007).

## Empty listing (no targets AND discovery finds nothing)

Prints actionable next commands instead of an empty table (FR-006, SC-006), e.g.:

```
  No targets yet.

  Add one:      fragcap targets add
  Scan a folder: fragcap targets scan <dir>
```

Ends by naming a next command (hero criterion 5 in the empty case). Exits 0.

## Listing snapshot and row-index resolution

- Every listing path (bare `fragcap`, `targets`, `targets list`) replaces the
  `listing_snapshot` table with the ordered rows it displayed
  (position, stable_id, handle).
- A bare-integer selector (`capture <n>`, `targets show <n>`, `targets remove <n>`,
  `targets export <n>`) resolves through the snapshot: position -> stable_id ->
  `target_by_stable_id`.
- `n` refers to the row the user last saw, even after an intervening add/remove
  shifts the live order (FR-004, SC-003).
- A position past the snapshot length, or one whose target was since removed, is a
  no-match on the row-index path -> usage error, exit 2 (FR-005), distinct from a
  clean handle/name miss (exit 0).
- If no listing has been run (no snapshot), a bare-integer selector is out of
  range (exit 2), not a silent hit against live order.
- Handle, name, and `--id` resolution are unchanged and independent of the
  snapshot.

## Consistency with S054

`capture <n>` (capture.rs `resolve_stored` -> `resolve_positional`) inherits the
snapshot resolution with no call-site change, because the change is inside
`resolve_positional`'s row-index branch. This intentionally changes the S054
behavior where `capture <n>` resolved over live `targets()` order; the change is
recorded in the spec Clarifications and is the load-bearing behavior of the slice.
