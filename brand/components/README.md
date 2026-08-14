# fragcap components

`components.css` holds the styles; the `.jsx` files are thin wrappers that
apply the class names. Load `../styles.css` first, then `components.css`.

Every component reads semantic tokens (`--fc-bg`, `--fc-fg`, `--fc-accent`,
...), so wrapping any subtree in `class="fc-light"` switches it to the light
reading surface without a second stylesheet.

## core

| Component | Class | Notes |
| --- | --- | --- |
| `Button` | `.fc-btn` | Variants: `primary`, `ghost`, `warning`. Orange is not a general CTA color; `warning` is for the inspection-required case only. |
| `Badge` | `.fc-badge` | Tones: `signal`, `capture`, `fault`. Each toned badge prints a glyph as well as a color. |
| `Card` | `.fc-card` | `raised` variant for elevated surfaces. |
| `Divider` | `.fc-divider` | Hairline rule on the Line token. |
| `SectionHeading` | `.fc-section` | Mono eyebrow, Space Grotesk title, muted sub. |
| `CaptureRow` | `.fc-capture-row` | Signature component - one observed packet. |

## forms

`Input`, `Select` and `Textarea` all take `label`, `hint` and `error`. Setting
`error` sets `aria-invalid` and prints the message; the border color changes
too, but the message is what carries the meaning.

## Rules these encode

- Status never depends on color alone.
- Every interactive element has a visible 2 px focus ring.
- Payload and hex text disables ligatures and character substitution.
- Motion respects `prefers-reduced-motion`.
