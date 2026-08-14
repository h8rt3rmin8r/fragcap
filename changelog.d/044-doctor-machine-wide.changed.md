`fragcap doctor` now recognizes a machine-wide Wireshark extcap registration, not
only the per-user one. The `analyzer extcap` check reports `ok` when the fragcap
binary is present in either the current user's Wireshark extcap directory or the
machine-wide (system) one, and names which scope registered it. Previously a
second user on a machine where fragcap was registered machine-wide (the MSI's
machine-wide option, slice 043) saw the "not registered" optional warning even
though Wireshark could see the source. Detection stays read-only, and the
not-registered case is still an optional warning.
