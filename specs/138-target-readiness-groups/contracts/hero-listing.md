# Contract: Grouped Hero Target Listing

## Human Output

For each non-empty readiness class, output its fixed heading followed immediately by the existing target table schema. Output the ready class before the setup-needed class. Separate two present groups with one blank line. Omit a missing class completely.

```text
Ready to capture:
  #  TARGET  CAPTURE  ENGINE       SENSITIVITIES
  1  alpha   ready    not scanned  not scanned

Needs setup:
  #  TARGET  CAPTURE         ENGINE       SENSITIVITIES
  2  zeta    needs a target  not scanned  not scanned

Next command:  fragcap capture 1
```

Rows retain the existing cells and no-truncation behavior. Each table derives widths from only its own rows and headings. Row numbers are global and continuous across groups.

## Ordering and Snapshot

The complete order is all ready rows sorted by handle, followed by all setup-needed rows sorted by handle. The listing snapshot stores stable identifier and handle pairs in exactly that order. Bare-integer selectors continue to resolve through the snapshot.

## Footer

When the ready group exists, the footer selects its first row whose install is present or unrecorded, otherwise its first row. When no ready group exists, the same preference runs over the setup-needed group. Empty listings emit no next-command footer.

## Stable Interfaces

`targets export`, target-entry storage, target stable identifiers, capture-readiness derivation, discovery, registration, diagnostics, exit codes, and machine-wide findings remain unchanged.
