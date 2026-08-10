# Brand

**Status: resolved (brand session, 2026-08-10).** The approved fragcap identity,
version 1.0.0, is vendored in the top-level [`brand/`](../../brand/) directory:
the full brand system (`brand/README.md`), the logo lockups (`brand/logos/`),
favicons, fonts with their OFL licenses (`brand/fonts/`), design tokens
(`brand/tokens/`), a type and hex-readability specimen (`brand/specimens/`), and
the printable guide (`brand/brand-guide.pdf`). Treat `brand/logos/svg/` as the
source of truth. This directory carries the repository-specific notes that the
kit does not: the security-posture framing that governs acceptance, the two
resolved open questions, and the site deployment target for S18.

The two former open questions from specification section 29 are both resolved:

| ID | Question | Resolution |
| --- | --- | --- |
| Q-7 | Monospace face selection | **Geist Mono.** Its `0 O`, `1 l I`, `8 B`, `5 S`, and `2 Z` stay distinguishable in packet payloads at the interface size, and its family relationship to Geist keeps technical specimens from feeling detached from the surrounding documentation. Evaluate any future replacement against `brand/specimens/`, not a decorative alphabet. |
| Q-8 | Parent brand visual relationship | fragcap is an independent **ShruggieTech** sub-brand with its own identity. It shares Space Grotesk, Geist, Geist Mono, dark-first discipline, and the parent's exact `#FF5300` orange, but not ShruggieTech green, the shruggie mark, or marketing layouts. The endorsement is **A ShruggieTech project**, set in Geist Mono, uppercase, subordinate, and outside the logo's clear space. Do not create a combined parent-product logo. |

With both resolved, S18 (the documentation site) is unblocked: the site can be
built against a decided identity.

## Governing principle: instrument, not weapon

Every visual and verbal decision reads as laboratory equipment. fragcap observes
traffic; it does not alter game state, conceal activity, automate play, or
promise advantage.

This is load-bearing rather than decorative: an identity that reads as cheat
tooling attracts platform removal, security software heuristics, and community
moderation regardless of what the software actually does. The brand is part of
the security posture, not a layer applied on top of it. Treat a proposed design
that fails this test the way you would treat a constitution violation, because
the consequence is the same order of severity. The full excluded-signal list,
color ratios, voice rules, and prohibited treatments are in `brand/README.md`
(specification section 23.3 remains the governing reference).

## Voice

Precise, dry, assumes technical competence. Links unfamiliar vocabulary to the
glossary rather than simplifying it. Constitution principle P-6 keeps the
glossary current, which is what makes precise prose affordable: the reader has
somewhere to go when a term is new. No marketing register. The landing page
states what fragcap is in one sentence, shows one worked invocation with its
output, names the prerequisite plainly, and links onward.

## Domain and deployment

`fragcap.com` is registered. Deployment targets GitHub Pages from a static
export built by continuous integration: the apex resolves through address
records to the static host's published addresses, the `www` subdomain through
an alias to the repository's default site host, and HTTPS enforcement is
enabled.

No base path is configured, because the site is served from a domain root
rather than a repository subpath. A `.nojekyll` marker must be emitted into the
output root; without it the static host's legacy processing removes the
framework's asset directory and the site loads with no styling or
interactivity.
