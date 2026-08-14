**2026-08-14** Adopted a single required/recommended/optional model for
fragcap's external dependencies: npcap is required (the capture driver),
Wireshark is recommended (the analyzer, whose installer also provides npcap), and
the Wireshark extcap integration is optional (it ships with Wireshark and only
needs fragcap registered as a source). `fragcap doctor` severities follow this
model, and the documentation pass in a later slice single-sources the same
wording so the tool and the docs cannot drift.
