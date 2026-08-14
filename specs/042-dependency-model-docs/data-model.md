# Data Model: Slice 042

This slice adds no code entities. Its "data" is the reader-facing content and its
single-sourcing relationships.

## Dependency tier

The canonical model, defined once in `docs/glossary/platform-and-distribution.md`.

| Field | Value for each tier |
| --- | --- |
| tier | required / recommended / optional |
| component | npcap / Wireshark / the Wireshark extcap integration |
| provides | the capture driver / the analyzer (and, via its installer, npcap) / opening fragcap captures live inside Wireshark |
| acquired | user installs npcap (detection-only by fragcap) / user installs Wireshark, which bundles Npcap, by the Nmap Project / ships with Wireshark, enabled by `fragcap extcap install` |
| doctor severity | required / recommended / optional |

Consumers (must link, not restate): `README.md`, `site/content/docs/getting-started.mdx`.

Invariant: the tier language matches the `fragcap doctor` severities and
`changelog.d/dependency-taxonomy.decisions.md`. A change to one is a change to the
source, propagated by link.

## Seed diagram

Three diagrams, each authored once as an identical `mermaid` fence in two places.

| id | subject | lives in |
| --- | --- | --- |
| pieces | fragcap, npcap, Wireshark, extcap and how they relate | `architecture.mdx` and `docs/fragcap-specification.md` |
| dataflow | interface -> npcap -> fragcap capture -> attribution -> pcapng/JSONL, plus extcap into Wireshark | same two |
| acquisition | how npcap arrives; detection-only; never bundled by fragcap | same two |

Invariant: the site copy and the GitHub copy are the same source; core Mermaid
syntax only (renders on both).

## Install screenshot

Five images, vendored under `site/public/screenshots/`, referenced `/screenshots/<name>.png`.

| order | file | step it illustrates |
| --- | --- | --- |
| 1 | 01_wireshark_choose-components_extcap.png | Wireshark component selection, extcap present |
| 2 | 02_wireshark_choose-install-location.png | choose install location |
| 3 | 03_wireshark_packet-capture_install-npcap.png | the packet-capture / install-Npcap prompt |
| 4 | 04_wireshark_npcap-setup.png | the Npcap setup dialog |
| 5 | 05_wireshark_npcap-setup_options.png | the Npcap setup options |

Each carries dash-free alt text and a step caption. The walkthrough ends with a
`fragcap doctor` output block (not an image).
