# fragcap Brand System

**Status:** Approved identity, version 1.1.0\
**Parent:** ShruggieTech\
**Repository domain:** `fragcap.com`\
**Audience:** Product, engineering, documentation, and communications teams

fragcap provides passive process-attributed network capture for games. Its
identity combines packet structure, capture framing, and controlled
fragmentation without borrowing the visual language of cheat tooling.

## Contents

- [Brand foundation](#brand-foundation)
- [Logo system](#logo-system)
- [Color](#color)
- [Typography](#typography)
- [Visual language](#visual-language)
- [Voice and writing](#voice-and-writing)
- [Parent brand relationship](#parent-brand-relationship)
- [Digital implementation](#digital-implementation)
- [Asset inventory](#asset-inventory)
- [What changed in 1.1.0](#what-changed-in-110)

## Brand foundation

### Governing principle: see what your game is actually saying

Your machine is already exchanging thousands of packets a second while you
play. fragcap makes them readable - which process, which endpoint, how many
bytes, right now.

That is the whole pitch. There is no claim about what you will find and no
promise that it will be useful. Some people open fragcap once out of
curiosity and close it again. Some find the protocol quirk that has been
bugging them for months. Both are fine outcomes, and the brand should never
imply that the second one is waiting for everybody.

### Positioning

**Category:** Game network capture and analysis tooling.\
**Role:** Passive capture and decode. fragcap reads; it does not write.\
**Audience:** Curious players first, then modders, server operators,
developers, protocol researchers, security engineers, and tool authors. The
technical floor is real - you need to be comfortable reading a table - but
being paid to do this is not a requirement.\
**Functional descriptor:** Passive process-attributed network capture for
games.

### Brand idea

**Captured signal. Preserved evidence.**

The brand treats packets as observable material. Every row fragcap shows you
is something that actually crossed the wire, timestamped and attributed to a
process, and it stays readable after the session ends.

### Personality

fragcap is precise, restrained, skeptical, and technically literate. It is
confident enough to state prerequisites and limitations plainly. It does not
perform excitement, posture as dangerous, or simplify away important detail.

| Trait | Expression | Avoid |
| --- | --- | --- |
| Precise | Exact nouns, units, formats, and commands | Vague claims |
| Observational | Observation, capture, decode, traces | Combat metaphors |
| Dry | Calm statements and sparse wit | Hype, performed excitement |
| Competent | Assumes technical ability | Condescending tutorials |
| Transparent | Names limitations, prerequisites, and sharp edges | Implied magic |

fragcap is aimed at gamers, so it talks about games: ports, servers, tick
rates, the process that owns the socket. It does not perform gamer voice to
prove it belongs, and it does not retreat into enterprise register either.

### Brand promises

- Capture is passive and inspectable.
- Every row is a packet that actually happened.
- Technical boundaries are stated before convenience claims.
- Terminology remains consistent across CLI, docs, schemas, and interfaces.
- Unknowns are labeled as unknowns.

## Logo system

### Mark construction

The mark contains three coordinated ideas:

- A triple-bladed **F** represents parallel packet lanes.
- Four separated corner segments create a capture frame without using a
  weapon-like circular crosshair.
- Orange packet terminals represent the bytes isolated by capture. The lower
  lane is deliberately shorter, keeping the silhouette legible as an F rather
  than an E.

The wordmark is custom vector lettering, shipped as **filled outlines**. It was
originally constructed from 22 px strokes with square caps and bevel joins; the
silhouette is identical, but the filled form no longer depends on how a given
engine resolves stroke geometry, fills correctly with `currentColor`, and
survives expand-to-outlines workflows in print, vinyl, and font tooling. Do not
replace it with live text or attempt to reproduce it with a typeface.

### Approved lockups

Use the horizontal lockup for headers, repository artwork, and wide surfaces.
Use the stacked lockup for square compositions and title pages. Use the mark
alone for application icons, favicons, avatars, and compact controls. Use the
wordmark alone only where another nearby element already establishes product
identity.

In every lockup the wordmark's ascender and baseline bracket the mark's inner
F, so the two elements share an optical top and an optical bottom.

### Clear space

Maintain clear space equal to the width of one orange terminal square around
the outer edge of any lockup. No text, border, icon, or crop may enter this
area. On the mark's 512-unit grid the terminal is 34 units, which is roughly
9 percent of the artwork width.

### Minimum size

| Asset | Minimum digital size |
| --- | --- |
| Mark | 24 px wide |
| Horizontal lockup | 160 px wide |
| Stacked lockup | 96 px wide |
| Wordmark | 120 px wide |

At and below **32 px** the four reticle corners and three terminals collapse
into noise. Small icons use the **reduced mark** shipped in `favicons/` and
`logos/svg/fragcap-mark-reduced.svg`: the F with its terminals, corners
dropped. The reduced mark is an approved variant with its own master file. Do not
rasterize a large logo down to browser-icon size at runtime.

### Backgrounds

The primary presentation is Signal Cyan and Capture Orange on Void. The light
variant uses the deeper accessible colors on the Light Surface token. Use the
single-color white or black variants when reproduction supports only one ink.

### Prohibited treatments

- Do not rotate, skew, stretch, outline, bevel, or add glow.
- Do not recolor individual packet lanes.
- Do not move the orange terminals or make them appear explosive.
- Do not close the four reticle corners into a box or circle.
- Do not set the wordmark in live text or a substitute typeface.
- Do not place the logo over busy imagery.
- Do not combine the fragcap and ShruggieTech marks into one lockup.
- Do not add skulls, weapons, controllers, shields, or exploit imagery.

The last rule is the load-bearing one, and it deserves the real reason rather
than a slogan.

Reading game traffic is where protocol reverse-engineering starts, and
protocol knowledge is how cheats get written. That road exists, and fragcap
stands at one end of it. Anyone technical enough to use this will work that
out in about a minute, so the brand does not pretend otherwise. What fragcap
answers with is scope: it captures and decodes, and it will not inject,
modify, automate, or hide. The project draws that line; the technology does
not enforce it.

The visual rules follow from the same fact. Anti-cheat vendors, storefronts,
and platform moderators triage on appearance long before they read a manifest,
and a skull or a crosshair tells them which end of that road you are standing
on. The four separated corners frame without aiming for exactly this reason.

## Color

The palette is dark-first and close to monochrome. Cyan is the observed
signal. Orange is the captured terminal or state requiring attention. Orange
must remain scarce enough to retain its meaning.

| Token | Hex | Role | Contrast |
| --- | --- | --- | --- |
| Signal Cyan | `#27C7E7` | Primary identity, links, focus, active data | 10.01:1 on Void |
| Capture Orange | `#FF5300` | Captured terminal, warning emphasis | 6.24:1 on Void |
| Fault | `#E9505F` | Failed capture, hard error | 5.55:1 on Void |
| Void | `#050708` | Primary background | - |
| Surface | `#0B1115` | Panels and code surfaces | - |
| Surface Raised | `#101A20` | Elevated controls and selected rows | - |
| Line | `#21323A` | Borders, separators, inactive diagrams | - |
| Text | `#F2F7F8` | Primary dark-mode text | 18.68:1 on Void |
| Text Muted | `#94A8B0` | Secondary dark-mode text | 8.16:1 on Void |
| Light Surface | `#F5F8F9` | Light reading background | - |
| Light Text | `#102027` | Primary light-mode text | 15.64:1 on Light Surface |
| Light Text Muted | `#47606B` | Secondary light-mode text | 6.24:1 on Light Surface |
| Light Line | `#D5DFE3` | Borders on light surfaces | - |
| Light Cyan | `#006F82` | Accessible cyan on light surfaces | 5.47:1 on Light Surface |
| Light Orange | `#C24100` | Accessible orange on light surfaces | 4.86:1 on Light Surface |
| Fault Deep | `#C0293A` | Error on light surfaces | 5.44:1 on Light Surface |

Every ratio here is measured from the hex values and re-derived on each
build. They are also carried in `tokens/brand.tokens.json` so they can be
asserted in a test.

### The one hard rule

Signal Cyan measures **1.89:1** on Light Surface. Never set it as text on a
pale background - use Light Cyan. The same applies to Capture Orange, which
needs Light Orange, and to Fault, which needs Fault Deep.

### Color ratio

Use approximately 80 percent neutral surfaces, 15 percent cyan, and no more
than 5 percent orange in a typical view. Orange is not a general CTA color.

### Semantic use

Cyan identifies selection, capture readiness, active filters, links, and focus.
Success uses cyan. Orange identifies captured boundaries, warnings, dropped
data, or a state that requires inspection. Fault identifies a capture that
failed, and always appears with explicit text and an error icon.

Capture Orange and Fault sit only about 25 degrees apart in hue, so they are
not reliably separable under color-vision deficiency. That is acceptable only
because status is never carried by color alone - every state ships a label or a
shape as well.

## Typography

Typography is shared selectively with ShruggieTech. This creates lineage
through craft rather than through copied layouts or parent-brand color.

| Function | Typeface | Weights |
| --- | --- | --- |
| Display and headings | Space Grotesk | 500, 700 |
| Body and interface | Geist | 400, 500 |
| Packet data and code | Geist Mono | 400 |
| Product wordmark | Custom vector lettering | Filled outlines, fixed artwork |

Geist ships 400 and 500 only, and Geist Mono ships 400 only. Emphasis must
resolve to a real face - asking for 700 makes the renderer synthesize a faux
bold, which prints badly and forces outlined glyphs into PDFs. In mono
contexts, carry emphasis with color rather than weight.

### Monospace decision

Geist Mono keeps `0 O`, `1 l I`, `8 B`, `5 S`, and `2 Z` distinguishable at the
actual interface size, which is what matters in a packet payload. Its family
relationship to Geist also prevents technical specimens from feeling detached
from surrounding documentation.

Evaluate future replacements against the included real-format specimen in
`specimens/`, not a decorative alphabet.

### Type behavior

Display headings use tight tracking near `-0.025em`. Body copy uses a line
height near `1.65`. Metadata and labels use Geist Mono with `0.08em` tracking.
Hex dumps never use ligatures and never apply smart quotes or automatic
character substitution - `tokens/base.css` disables both on `code`, `pre`,
`kbd`, and `samp`.

Sentence case is the default. Uppercase is reserved for compact labels,
capture states, and table metadata.

## Visual language

### Instrumentation

Visual references come from oscilloscopes, protocol analyzers, packet lanes,
capture gates, timing marks, and network topology. Use exact alignment and
quiet negative space. Every graphic should do a job: explain a state, or explain a relationship.

### Iconography

Use simple line icons with 1.5 px to 2 px strokes, square or lightly chamfered
terminals, and minimal rounding. Prefer direct technical symbols such as
filter, file, clock, interface, endpoint, and search. Avoid mascots, weapons,
controllers, shields, hooded figures, and generic circuit-board decoration.

Six starter icons ship in `icons/` on a 24 grid with a 1.5 stroke, butt caps,
and mitre joins. They stroke with `currentColor`, so an icon inherits whatever
semantic token its container sets.

### Surfaces and geometry

Panels use 1 px Line borders and radii between 4 px and 8 px. Avoid glass
effects, large shadows, chrome, and decorative gradients. Selected states may
use a subtle cyan border or a low-opacity cyan fill. Orange backgrounds should
be rare and small.

### Data visualization

Prefer lanes, traces, timelines, byte grids, and topology diagrams. Cyan marks
the active observation path. Orange marks the selected terminal, discontinuity,
or warning. Gray remains the default for context.

### Motion

Motion communicates capture state, filtering, or continuity. Keep interface
transitions between 120 ms and 240 ms on `cubic-bezier(.2,.6,.2,1)`. Avoid
pulsing glow, glitch effects, screen shake, and decorative scan lines. Respect
`prefers-reduced-motion`.

## Voice and writing

### Voice

Write precisely and dryly. Assume technical competence. Link unfamiliar terms
to the glossary instead of weakening the language around them.

Use active voice, concrete nouns, and observable outcomes. State prerequisites
before instructions. Distinguish supported behavior, inferred behavior, and
unknown behavior.

Address the reader as someone playing a game on their own machine. "Your
traffic" and "your game" beat "the operator" and "the end user" every time,
and they cost nothing in precision.

Where a capability has an obvious misuse, name it and say where the project's
line falls. Readers technical enough to run fragcap will see what it enables
whether or not the copy admits it, and hedging in front of that audience only
costs credibility.

### Register

fragcap does not use conventional marketing structure. The landing page should
state what the tool is, show one worked invocation with representative output,
name the prerequisite plainly, and link to documentation. Avoid testimonials,
feature grids, urgency, and generalized calls to action.
`ui_kits/fragcap-web/` demonstrates this register.

### Preferred language

| Prefer | Avoid |
| --- | --- |
| your game, your machine, your traffic | the operator, the end user |
| capture, observe, inspect, decode | intercept, attack, exploit |
| packet, frame, trace, session | traffic magic, secret data |
| supported, experimental, unknown | flawless, revolutionary, effortless |
| requires, emits, records, filters | unlocks, dominates, supercharges |

### Rhetorical devices to avoid

Do not build sentences out of the "X, not Y" contrast - *evidence, not
theatre*; *an instrument, not a weapon*. It reads as borrowed confidence, it
is the most recognisable tell of machine-written copy, and it usually
substitutes a shape for an argument. The same applies to its relatives: *X
over Y*, *rather than merely Z*, and *never decorate* tacked onto the end of a
sentence that had already finished.

State the thing you mean. If the contrast genuinely carries information, give
it its own sentence. `build/verify.py` fails the build when this pattern
appears in brand copy.

### Examples

**Product statement:** Passive process-attributed network capture for games.

**Prerequisite:** Packet capture requires an Npcap-compatible capture driver.

**Limitation:** Encrypted payloads remain opaque unless a supported decoder can
derive the required session context.

**Empty state:** No packets matched the current filter.

**Error:** Capture stopped. Interface `Ethernet 2` is no longer available.

### Casing and terminology

The product name is always lowercase: **fragcap**. Begin a sentence with
`fragcap` rather than capitalizing it. File formats, protocol names, and command
flags preserve their canonical casing. Add new domain terms to the glossary
when they first enter user-facing documentation.

## Parent brand relationship

fragcap is an independent ShruggieTech product identity. It shares Space
Grotesk, Geist, Geist Mono, dark-first discipline, and the parent's exact
`#FF5300` orange. It does not inherit ShruggieTech green, the shruggie mark,
marketing layouts, or verbal flourish.

The approved endorsement is **A ShruggieTech project**. Set it in Geist Mono,
uppercase, with positive tracking. Keep it visually subordinate and outside
the fragcap logo's clear space. The endorsement may appear in the footer,
About page, repository metadata, title-page colophon, and social preview.

Do not create a combined parent-product logo.

## Digital implementation

### CSS

Link `styles.css` to load fonts, tokens, and base element styles in one file:

```html
<link rel="stylesheet" href="styles.css">
<link rel="stylesheet" href="components/components.css">
```

The default surface is dark. Add `class="fc-light"` to any container to switch
that subtree to the light reading surface and the deeper accent values - no
second stylesheet, no component changes.

| File | Provides |
| --- | --- |
| `tokens/colors.css` | Brand colors, transparent fills, semantic tokens, `.fc-light` |
| `tokens/typography.css` | Families, weights, display and body scales, tracking, leading |
| `tokens/spacing.css` | Spacing scale, radii, stroke widths, shadows, motion, layout |
| `tokens/base.css` | Element resets, focus ring, ligature suppression, reduced motion |

Variables use the `--fc-` prefix. The `--fragcap-*` names published in v1.0.0
are kept as aliases so existing product code keeps working; new work should use
`--fc-`.

### Components

`components/components.css` holds the styles and the `.jsx` files are thin
wrappers. `CaptureRow` is the signature component: one observed packet, with
the attributed process and a capture state rendered as a labelled badge.

### Favicons

The `favicons/` directory includes SVG, a genuine multi-resolution ICO
(16, 24, 32, 48, 64, 128, 256), standard browser PNGs, an Apple touch icon,
Android icons, and a web manifest. Entries at and below 32 px use the reduced
mark; larger entries use the full mark. Keep the supplied Void background in
small icons because it protects the reticle corners and orange terminals
across browser themes.

### Accessibility

- Do not use the bright Signal Cyan as body text on a light surface. Use Light
  Cyan.
- Do not communicate capture status through color alone. Pair it with text or a
  shape.
- Every interactive element requires a visible 2 px focus ring and a non-color
  state change.
- Interface text should meet WCAG AA contrast at its rendered size.
- Respect `prefers-reduced-motion` on every transition and animation.

## Asset inventory

| Directory | Contents |
| --- | --- |
| `logos/svg/` | Vector masters and all approved lockups. Filled paths throughout; no live text, no stroked lettering |
| `logos/png/` | High-resolution raster exports and social preview |
| `favicons/` | Browser, Apple, Android, SVG, ICO, and manifest assets |
| `fonts/` | WOFF2, TTF, CSS declarations, and OFL licenses |
| `tokens/` | CSS and JSON design tokens, with measured contrast ratios |
| `icons/` | Six starter line icons on a 24 grid |
| `components/` | `components.css` plus React core and form components |
| `guidelines/index.html` | Visual system reference, rendered live from the system |
| `ui_kits/fragcap-web/` | Demo product page built from the system |
| `specimens/` | Typography and hex-readability specimen, fully outlined |
| `styles.css` | Single CSS entry point |
| `brand-guide.pdf` | Printable reference manual |
| `VERIFY.md` | Generated dimensions, ICO entries, and SHA-256 checksums |

Use SVG in product interfaces and documentation whenever the surface supports
it. Use PNG for social platforms, raster-only systems, and external listings.
Treat the files in `logos/svg/` as the source of truth.

## What changed in 1.1.0

Repositioning:

- **The governing principle changed.** 1.0.0 led with "instrument, not weapon".
  It leaned on the "X, not Y" contrast, which is the most recognisable tell of
  machine-written copy, and it framed the product around a defensive claim
  rather than around what the tool does. The principle is now *see what your
  game is actually saying*.
- **Aimed at gamers, explicitly.** The audience now reads "curious players
  first". The old personality table forbade "gamer slang" while addressing
  "the operator" and "the end user"; the voice now talks about games and
  addresses the reader as someone playing one. The technical floor stays where
  it was.
- **The dual-use question is answered instead of avoided.** Reading game
  traffic is where protocol reverse-engineering starts, and fragcap says so.
  This also gives the no-skulls rule a real argument in place of a slogan.
- **The contrasting device is banned and enforced.** `build/verify.py` fails
  the build if "X, not Y" and its relatives appear in brand copy.

Fixes to defects in the 1.0.0 kit:

- **Wordmark converted from strokes to filled outlines.** The silhouette is
  unchanged; verified pixel-identical against the 1.0.0 rendering.
- **Wordmark clipping fixed.** The f crossbar previously extended to x = -1 in
  a viewBox starting at 0, so it was cut off by its own canvas, with 0 px of
  padding on the left against 60 px on the right.
- **Social preview rebuilt with outlined type.** The 1.0.0 SVG used live text
  with `font-family="Geist,Arial,sans-serif"` and a `textLength` override; the
  shipped PNG rendered its tagline in Arial instead of Geist, with subpixel color
  fringing baked in.
- **Type specimen rebuilt with outlined type.** The 1.0.0 SVG declared bare
  font families with no fallback and no embedded font, so it rendered in the
  default serif on any machine without the fonts installed.
- **brand-guide.pdf rebuilt.** The 1.0.0 guide was a light, landscape document
  for a dark-first brand; it referenced a non-embedded Helvetica, embedded only
  one Space Grotesk weight, showed 6 of 12 color tokens with no contrast
  ratios, and omitted the visual-language, accessibility, and inventory
  chapters entirely.
- **Lockups re-aligned** so the wordmark brackets the mark's inner F.
- **ICO bundle fixed** - it now carries seven real entries instead of one.

Additions:

- Reduced mark for icons at and below 32 px.
- Semantic token layer and `.fc-light`, so light mode is deliverable in code.
- `tokens/spacing.css` and `tokens/base.css`; spacing, radii, stroke, and
  motion previously existed only in JSON and could not be consumed from CSS.
- Fault color, filling a gap where the system had no failure state at all.
- Published contrast ratios for every text-bearing token.
- `components/`, `icons/`, `guidelines/index.html`, `ui_kits/fragcap-web/`,
  `SKILL.md`, and `VERIFY.md`.
- Single-ink horizontal black lockup, stacked white lockup, and the missing
  white and black wordmark PNG exports.
