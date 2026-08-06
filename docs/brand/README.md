# Brand

**Status: placeholder.** Identity is developed in a dedicated session. The
guardrails below are from specification section 23.3 and are sufficient to
build the site against in the interim.

Two open questions remain, both from specification section 29:

| ID | Question | Blocks |
| --- | --- | --- |
| Q-7 | Monospace face selection | S18 |
| Q-8 | Parent brand visual relationship | S18 |

Brand is not on the critical path and is deliberately not a v0.1.0 priority.
It is also not forgettable: S18 ships the documentation site, and a site cannot
be built against an undecided identity. Resolve it before S18 starts, not
during.

## Governing principle: instrument, not weapon

Every visual and verbal decision reads as laboratory equipment.

This is accurate positioning, since fragcap is a passive observation tool. It
is also load-bearing rather than decorative: an identity that reads as cheat
tooling attracts platform removal, security software heuristics, and community
moderation regardless of what the software actually does. The brand is part of
the security posture, not a layer applied on top of it.

Treat a proposed design that fails this test the way you would treat a
constitution violation, because the consequence is the same order of severity.

## Excluded

Each of these signals cheat tooling to precisely the audiences whose assessment
matters most:

- Saturated multi-color peripheral palettes
- Angular chrome logotypes
- Skull and crosshair imagery
- Exploit vocabulary

## Directional

- Instrumentation visual language, drawn from oscilloscopes, spectrum
  analyzers, and network topology diagrams
- A restrained dark base with a single signal accent
- Density and precision over decoration

## Typography carries functional weight

The monospace face is a **selection requirement, not a stylistic preference**,
because users read packet payloads in it. Candidates are evaluated against
hex-reading criteria:

- Unambiguous zero against capital O
- Unambiguous one against lowercase L and capital I
- Clear separation among hexadecimal digits

A face that fails any of these is rejected regardless of how it looks in a
heading. Evaluate candidates on a real hex dump at the size the documentation
will actually use, not on a specimen sheet.

This is open question Q-7.

## Voice

Precise, dry, assumes technical competence. Links unfamiliar vocabulary to the
glossary rather than simplifying it.

The glossary exists so that prose does not have to condescend. Constitution
principle P-6 keeps it current, which is what makes this affordable: writing
precisely is only viable if the reader has somewhere to go when a term is new.

No marketing register. The landing page states what fragcap is in one sentence,
shows one worked invocation with its output, names the prerequisite plainly,
and links onward. No testimonials, no feature grids, no calls to action. The
audience arrives already knowing they need a capture tool.

## Parent brand relationship

fragcap is a ShruggieTech sub-brand carrying its own visual identity, alongside
the other product sub-brands rather than inheriting the parent system.

The nature of the visible relationship is unresolved. This is open question
Q-8.

## Domain

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

## Before the brand session

Worth having on hand:

- A real `.fcapng` hex dump, for evaluating monospace candidates against actual
  content rather than a specimen
- The other ShruggieTech sub-brand identities, for the Q-8 relationship
  decision
- The landing page copy, since the one-sentence statement constrains the
  visual composition more than the reverse
