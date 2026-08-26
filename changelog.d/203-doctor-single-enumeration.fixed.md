<!-- spec-impact: none -->

Fixed `fragcap doctor` doing duplicate npcap device-list enumeration by deriving
the loopback-adapter verdict from the interface inventory it already gathered.
