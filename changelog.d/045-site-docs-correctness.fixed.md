The documentation website's profile pages described the wrong format. The
profile-schema reference and the writing-a-profile guide documented and
demonstrated the profile as TOML, a format the tool stopped accepting when the
profile format moved to JSON; both pages are now JSON, and their examples carry
the schema's required top-level keys (`schema`, `kind`, `fidelity`) and validate
against the published schema. Command examples name a `.json` profile path rather
than `.toml`.

The verbatim game slug `eso` no longer appears throughout the docs as if it were
a shipped profile. One concrete slug is kept, as the example value under the CLI
`--profile` reference; every other occurrence is a typed placeholder (`<game-id>`,
`<client>.exe`, `<profile>.json`), and the sentence apologizing that the example
was illustrative and not a shipped profile is gone.

The two wide Mermaid diagrams on the Architecture page (the dependency model and
the runtime data flow) are laid out top to bottom instead of left to right, so
they fit the content column and are legible at normal widths instead of running
off the side.

The site footer no longer detaches on documentation pages. It rendered once at
the body level, a sibling of the fumadocs docs layout whose grid forces a full
viewport of height, which parked the footer a full viewport below the content on
every docs page. The footer now renders in flow: the home group renders it after
its layout, and docs pages render it inside the docs content column after the
body, so it sits directly under the content, exactly one footer per page, with
the home page unchanged.
