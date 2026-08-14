`fragcap doctor` now points at where to get Wireshark. When the analyzer extcap
integration is not registered, the guidance names the Wireshark download URL
alongside `fragcap extcap install`, and notes that the Wireshark installer also
provides npcap, so one download resolves both the recommended analyzer and the
required capture driver. The URL is single-sourced in a new
`fragcap_core::interface::WIRESHARK_DOWNLOAD_URL` constant, the sibling of
`DRIVER_DOWNLOAD_URL`, and the npcap-absent remediation takes its Wireshark URL
from the same constant rather than a second literal. doctor stays granular (the
per-option npcap precision is unchanged) and the integration check remains a
non-blocking optional warning; a ready environment's output is unchanged.
