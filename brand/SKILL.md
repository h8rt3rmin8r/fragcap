---
name: fragcap-brand
description: Generate on-brand interfaces, assets, and docs for fragcap (passive process-attributed network capture for games, a ShruggieTech sub-brand). Contains colors, type, fonts, logos, icons, design tokens, and UI components.
user-invocable: true
---

Read `README.md` for the full brand system, then explore the files below.

If you are building visual artifacts (mocks, prototypes, slides), copy assets
out and produce static HTML that links `styles.css` and
`components/components.css`. If you are working in production code, copy the
tokens and components and follow the rules in `README.md`.

## Quick reference

- **Governing principle:** see what your game is actually saying. fragcap makes
  the traffic your machine is already exchanging readable. No claim about what
  the reader will find.
- **Sharp edges, stated:** reading game traffic is where protocol
  reverse-engineering starts, and that is how cheats get written. Say so when it
  is relevant; fragcap's answer is scope - capture and decode, never inject,
  modify, automate, or hide. Never add skulls, weapons, controllers, shields, or
  crosshairs: moderators triage on appearance long before they read code.
- **Mode:** dark-first. Void `#050708`; Surface `#0B1115`; Line `#21323A`.
  Light Surface `#F5F8F9` is the reading surface.
- **Accents:** Signal Cyan `#27C7E7` (observed signal, links, focus, success) ·
  Capture Orange `#FF5300` (captured terminal, warning - scarce, never a
  general CTA) · Fault `#E9505F` (failure only). On light surfaces use Light
  Cyan `#006F82`, Light Orange `#C24100`, Fault Deep `#C0293A`. Signal Cyan is
  1.89:1 on Light Surface - never use it as text there.
- **Ratio:** ~80% neutral surfaces, 15% cyan, under 5% orange.
- **Type:** Space Grotesk (display 500/700, tracking -0.025em) · Geist (body
  400/500) · Geist Mono (payloads, code, labels, 0.08em tracking). Geist has no
  700 and Geist Mono has no bold - never ask for a weight that does not exist
  or the renderer synthesises a faux bold.
- **Voice:** precise, dry, technically literate. Aimed at gamers, so talk about
  games - ports, servers, tick rates, the process that owns the socket. Say
  "your game" and "your traffic", not "the operator". Assume technical ability
  without performing gamer voice or retreating into enterprise register. State
  prerequisites before instructions. Label unknowns as unknowns. No
  testimonials, feature grids, or urgency. Product name is always lowercase:
  fragcap.
- **Never write "X, not Y":** *evidence, not theatre*, *an instrument, not a
  weapon*. It is the clearest tell of machine-written copy and it substitutes a
  shape for an argument. Same for *X over Y* and *rather than merely Z*.
- **Logo:** wordmark is filled vector outlines (never live text, never a
  substitute typeface). Mark = triple-bladed F, four separated reticle corners,
  three orange terminals. Clear space = one terminal width. Below 32 px use the
  reduced mark.
- **Accessibility:** status never depends on colour alone; 2 px visible focus
  ring on every interactive element; respect `prefers-reduced-motion`.
- **Load the system:** link `styles.css`, then `components/components.css`. Add
  `class="fc-light"` to a container for the light reading surface.
