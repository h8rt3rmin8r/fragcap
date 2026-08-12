### Changed

- **The landing page presents the value proposition** (issue #42, specification
  section 23.1 as amended). It leads with the problem fragcap solves that standard
  tooling does not: capture below the socket layer has already discarded the
  packet-to-process association, and for a client started indirectly through a
  launcher the owning process is not the launcher. It keeps the one worked
  invocation and the prerequisite honesty ("detects npcap, never installs it"),
  and adds a small number of concrete capability statements, each a plain fact
  linking into the documentation that proves it rather than an adjective. It still
  carries no testimonials, no feature grid, and no call to action.
