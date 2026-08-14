The external-dependency model is now stated once. The glossary carries a
"Dependency model" entry defining the three tiers (npcap required, Wireshark
recommended, the Wireshark extcap integration optional) to match the `fragcap
doctor` severities; the README and the Getting Started guide summarize and link
to it rather than restating the tiers, so the tool and the docs cannot drift.

The stale loopback framing is corrected across the README, the glossary, and the
master specification: current Npcap installs loopback capture support
automatically, so it is no longer a separate installation option to enable. The
one option that still matters, WinPcap API compatible mode, is kept. The docs now
also state that npcap is by the Nmap Project and that the Wireshark installer
bundles it.
