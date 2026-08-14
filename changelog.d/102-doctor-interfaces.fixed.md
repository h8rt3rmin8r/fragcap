`fragcap doctor` now lists the real capture-capable network interfaces, naming
each adapter beside its address, instead of always reporting that none were
found. The empty-set warning appears only when enumeration genuinely finds no
interfaces, and an enumeration that fails is reported as a failure rather than
presented as a successfully observed empty machine.
