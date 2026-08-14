The documentation site now renders Mermaid diagrams. A ```mermaid fence is
rewritten to a client-side renderer that follows the page theme, so the same
fenced source renders on the site and on GitHub. Three seed diagrams are authored
with it (the pieces and how they relate, the runtime data flow, and how npcap is
acquired) on the Architecture page and in the master specification.

The Getting Started guide gains an annotated install walkthrough built from real
Wireshark and Npcap installer screenshots, ending in a `fragcap doctor`
verification step that shows the readiness output. Screenshots are served from
`site/public/screenshots/`.
