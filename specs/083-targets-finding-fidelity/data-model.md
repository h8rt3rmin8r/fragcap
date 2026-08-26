# Data Model: Targets Finding Fidelity

## Technology Finding

Stored inside a target entry's evidence payload.

Fields used by this slice:

- `category`: Technology category token. `engine` maps to ENGINE; `anti-cheat` and `drm` map to SENSITIVITIES.
- `product`: Product label rendered in the target table.
- `evidence`: Evidence detail preserved for export and detailed surfaces.
- `fidelity`: Trust tier for this finding.

Validation rules:

- Missing or unknown `category` means the finding belongs to neither listing column.
- Missing or empty `product` means the finding contributes no product label.
- `fidelity: "verified"` or stronger renders as an unmarked product.
- Any other fidelity token, missing fidelity, or malformed fidelity renders as uncertain.

## Technology Summary Product

Derived at listing time, never stored.

Fields:

- `category`: One requested technology column category.
- `product`: Display label.
- `verified_or_stronger`: Whether the strongest matching finding for this product is verified or stronger.

Rules:

- Products are ordered by category order, then first-seen order within that category.
- Duplicate product labels collapse to one summary product.
- The strongest duplicate fidelity wins. Authored and verified outrank every uncertain or malformed finding.
- An uncertain summary product renders as `<product>?`.

## Target Entry Export

Existing JSON array surface emitted by `targets export`.

Rules for this slice:

- The `evidence` array remains the machine source of truth for technology category, product, evidence detail, and fidelity.
- Export emits each evidence object without dropping its `fidelity` token.
- Import preserves each evidence object without normalizing the per-finding fidelity token.
- No schema migration is required.
