# Contract: Targets Finding Fidelity

## Human Listing

Command:

```text
fragcap targets
```

For each target row:

- A verified-or-stronger technology product renders as its product name, for example `Unreal`.
- A below-verified, missing-fidelity, or malformed-fidelity technology product renders with `?`, for example `Unreal?`.
- ENGINE contains only engine findings.
- SENSITIVITIES contains only anti-cheat and DRM findings, anti-cheat before DRM.
- Coverage markers remain exactly `-`, `incomplete`, and `not scanned`.

Example:

```text
  #  TARGET    CAPTURE  ENGINE   SENSITIVITIES
  1  verified  ready    Unreal   Steam DRM
  2  guessed   ready    Unreal?  Steam DRM?
```

## Target Export

Command:

```text
fragcap targets export
```

Each exported evidence finding preserves its raw fidelity:

```json
[
  {
    "stable_id": 1,
    "handle": "guessed",
    "name": "Guessed",
    "classification": "game",
    "classification_source": "user",
    "fidelity": "authored",
    "evidence": [
      {
        "category": "engine",
        "product": "Unreal",
        "evidence": "Binaries/Win64",
        "fidelity": "heuristic-unverified"
      }
    ]
  }
]
```

Importing and exporting this record again keeps the same `evidence[0].fidelity` token.
