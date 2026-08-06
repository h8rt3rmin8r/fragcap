# Brand Rules Reference

Compressed reference for the values and rules the skill enforces. Authoritative source is [`brand-identity.md`](https://github.com/) in the ShruggieTech knowledge base; this file restates the slice the skill needs so it stays self-contained.

## Five Immutable Brand Color Tokens

These five hex values do not change at the token level under any circumstance.

| Token | Hex | Role |
|---|---|---|
| `--color-black` | `#000000` | Primary background (dark mode), text (light mode) |
| `--color-green-bright` | `#2BCC73` | Primary accent, interactive elements, links, highlights |
| `--color-green-deep` | `#00AB21` | Logo mark, secondary accent, hover states, light-mode accent text |
| `--color-orange` | `#FF5300` | CTA emphasis, alerts, energy accents, link hover color |
| `--color-gray-light` | `#D1D3D4` | Borders, secondary text, dividers |

Derived tokens (extended palette, section surface tokens) may deviate. Brand tokens may not.

## Light-Mode Accessibility Rule

Green Bright (`#2BCC73`) hits roughly 8.9:1 on black (passes WCAG AA), but only 2.4:1 on white (fails for body text). In any light-mode surface, green accent text substitutes to Green Deep (`#00AB21`, roughly 3.6:1, acceptable for large text and decorative use). Body text on white uses near-black (`#0A0A0A`, roughly 19.5:1).

Orange is used directly in both modes without a darkened variant.

## Typography

| Role | Family | Weights | When |
|---|---|---|---|
| Display, headings, logotype | Space Grotesk | 500, 700 | `h1`-`h6`, hero headlines, button labels |
| Body, UI | Geist | 400, 500 | Paragraphs, lists, nav, general prose |
| Monospace, eyebrows, labels | Geist Mono | 400 | Code blocks, eyebrow labels, metadata |

System fonts are graceful-degradation fallbacks only. Never substitute system fonts in production output. Geist Pixel variants exist but are decorative accents only; do not use for body or general UI.

Fonts load via `@import url("https://cdn.shruggie.tech/brand/typography.css");` at the top of the inlined `<style>` block. Never inline `@font-face` rules; the CDN stylesheet is the source of truth and may evolve.

## Tagline

The exact tagline string is:

```
¯\_(ツ)_/¯ We'll figure it out.
```

Do not paraphrase. When rendering, mark the emoticon `aria-hidden="true"` and color it `var(--color-green-bright)` (Green Deep in light mode). The tagline element uses the `.tagline` class and renders at `body-xs` scale.

## Logo URLs (CDN)

These CDN URLs are the source of truth. Generated HTML never embeds them as `<img src>`; instead, the skill substitutes inline base64 data URIs from `inline-assets.md` so output is fully self-contained. The URLs below exist so the bundled assets stay traceable to their canonical origin.

| Variant | URL | Use when |
|---|---|---|
| Dark background | `https://cdn.shruggie.tech/brand/logo/logo-darkbg.png` | Dark or black container backgrounds |
| Light background | `https://cdn.shruggie.tech/brand/logo/logo-lightbg.png` | White or near-white container backgrounds |
| Social share | `https://cdn.shruggie.tech/brand/logo/github-social-share.png` | OG image, Twitter card image |

The kawaii character may not be used where the platform requires authentic photography (Google Business Profile is the canonical example). On those surfaces, use real photographs of people and work, never the illustrated mark.

## Favicon URLs (CDN)

Same source-of-truth rule as the logo URLs above: generated HTML embeds these as inline base64 from `inline-assets.md`; the CDN URLs document the canonical origin of the bundled bytes.

| Asset | URL |
|---|---|
| Multi-size ICO (primary) | `https://cdn.shruggie.tech/brand/favicon/icon.ico` |
| PNG 512 | `https://cdn.shruggie.tech/brand/favicon/icon-512.png` |
| PNG 256 | `https://cdn.shruggie.tech/brand/favicon/icon-256.png` |
| PNG 192 | `https://cdn.shruggie.tech/brand/favicon/icon-192.png` |
| PNG 144 | `https://cdn.shruggie.tech/brand/favicon/icon-144.png` |
| PNG 92 | `https://cdn.shruggie.tech/brand/favicon/icon-92.png` |
| PNG 64 | `https://cdn.shruggie.tech/brand/favicon/icon-64.png` |

The `.ico` is the primary `<link rel="icon">` target. PNG sizes are for PWA manifests, Apple touch icons, and other surfaces that need a specific raster size.

## Voice Rules

These apply to all copy the skill emits, not just headlines.

1. **Second person.** Speak to the visitor (`You have a business to run`). The visitor is the subject of the page, not the company.
2. **No competitive jabs.** Do not name competitors. Vendor-displacement narratives (`rentership to ownership`) are fine as positioning; named attacks are not.
3. **No unsubstantiated claims.** A capability appears in copy only if it can be backed by concrete work.
4. **Confidence without arrogance.** State plainly. Avoid `we might be able to help` and avoid `industry-leading experts`.
5. **Plain English over consulting-speak.** No `synergy`, `leverage`, `best-in-class`, `turnkey`, `solutioning`.
6. **Humor is permitted in small doses.** The shruggie identity invites a wry self-aware tone; do not force it.

## Brand Architecture Scope

This skill covers the **ShruggieTech parent brand only** (and Shruggie LLC legal-entity attribution). Product sub-brands carry their own visual identities and are out of scope:

- metadexer
- shruggie-indexer
- shruggie-feedtools
- Covarity
- SparkPlan
- Knox.Dance
- rustif

If the user asks for a page that is primarily about a sub-brand, flag the scope conflict and ask whether to (a) produce a parent-brand page that mentions the sub-brand, or (b) defer to the sub-brand's own identity system. Do not invent visual identity for a sub-brand inside this skill.

## "By ShruggieTech" Attribution

Standard formats:

- `By ShruggieTech` (default, footers and about pages)
- `A ShruggieTech product` (marketing and press)
- `A ShruggieTech LLC Company` (Covarity specifically)

The legal name `Shruggie LLC` appears only in license headers, copyright notices, and legal agreements; not in consumer-facing marketing surfaces unless a disclosure requires it.

## Motion

Permitted animations: scroll reveal, staggered reveal, hover feedback (border, shadow, scale up to 1.02), page transition cross-fade, mobile-nav slide, link underline scaleX, SVG draw-on.

Prohibited: parallax on content elements, autoplaying video backgrounds, infinite-loop animations (one exception exists on the website hero CTA pulse), scroll-jacking, single-element durations over 800ms.

Every animation is wrapped in `prefers-reduced-motion: reduce` and reduces to zero duration when the user opts out. For single-file HTML one-offs, ship without animation unless the user explicitly asks; restraint is the brand default.

## Output File Rules

Every file the skill writes:

- UTF-8 encoding, no BOM
- LF line endings (not CRLF)
- No trailing whitespace on any line
- Single trailing newline at end of file
- No em-dashes and no en-dashes anywhere in the document, including comments and `alt` text
- Every `<img>` has meaningful `alt` text (decorative-only images get `alt=""` plus `aria-hidden="true"`)
