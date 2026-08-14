# fragcap brand kit - build

Every asset in this kit is generated from these scripts, so the identity can be
rebuilt from source rather than re-drawn by hand. v1.0.0 shipped no build step,
which is why several assets had drifted from their masters.

Run in this order from the kit root:

```
python3 build/build_logos.py      # SVG masters: lockups, marks, wordmarks, social preview
python3 build/build_icons.py      # icons/
python3 build/build_specimen.py   # specimens/ (outlined type)
python3 build/build_raster.py     # logos/png/ and favicons/, including the ICO bundle
node    build/print_pdf.js build/brand-guide.html brand-guide.pdf
python3 build/finish_pdf.py       # PDF metadata and outline
python3 build/verify.py           # manifest.json + VERIFY.md, and asserts the kit's claims
```

## What each piece does

| File | Role |
| --- | --- |
| `outline.py` | Expands the wordmark's stroked polylines into filled outlines. Square caps and bevel joins are reproduced exactly, so the silhouette is unchanged. |
| `geometry.py` | Single source of truth for mark paths, wordmark geometry, and palette. |
| `typeset.py` | Shapes text through HarfBuzz and emits outlined SVG paths, so no shipped asset depends on an installed font. |
| `build_logos.py` | All SVG masters and lockup metrics. |
| `build_raster.py` | PNG exports and favicons. Hand-assembles the multi-resolution ICO, because Pillow's ICO writer only ever emits one directory entry. |
| `build_icons.py` | The six starter icons. |
| `build_specimen.py` | The type specimen, fully outlined. |
| `brand-guide.html` | Print source for `brand-guide.pdf`. |
| `print_pdf.js` | Prints the guide via headless Chromium at the CSS page size. |
| `finish_pdf.py` | Sets document metadata and builds the bookmark outline. |
| `verify.py` | Regenerates `manifest.json` and `VERIFY.md`, and fails the build if a claim drifts. |

## verify.py can fail the build

It re-derives every published contrast ratio from the hex values, scans every
SVG for live text and font-family declarations, renders each logo against a
widened viewBox to prove nothing is clipped, checks the PDF's fonts are all
embedded kit faces with no Type 3 fallbacks, and scans every text file for
mojibake and BOMs. A non-zero exit means something in the kit is claiming
something untrue.

Requires: `shapely`, `fonttools`, `uharfbuzz`, `cairosvg`, `pillow`, `pikepdf`,
`numpy`, and `playwright` (Node) for the PDF step.
