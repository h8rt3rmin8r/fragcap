<!-- spec-impact: 17.2.1, 29 -->

S134 replaces Deep Capture's generic `--trust-ca` and `--yes` authorization with a versioned canonical plan digest. The exact session CA is generated only in memory before review and is handed unchanged to the native runtime; concrete loopback listener reservation moves after authorization; hidden legacy flags return migration errors for one tagged release without retaining their former authority.
