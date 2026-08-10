### Added

- **Live packet capture.** `fragcap-capture` gains a `PacketSource` backed by
  the platform capture driver, behind a `live` feature that is off by default.
  Specification sections 12.1 and 12.2. The feature being off is what keeps
  `cargo xtask ci` passing on a machine with neither npcap nor its software
  development kit installed.
- **Interface enumeration and selection.** `fragcap-core::interface` carries the
  whole section 12.1 precedence as a pure decision over an inventory value:
  explicitly named interfaces first, otherwise the default-route interface plus
  the loopback adapter when requested, otherwise every interface that is up,
  addressed, and not virtual. It opens nothing and touches no platform surface,
  so the entire precedence is tested on any machine with no capture driver.
- **Every interface is accounted for.** A selection reports each interface it
  passed over with a named reason, and a test asserts that the chosen and the
  passed-over together account for the whole inventory. Choosing the wrong
  interface produces a run that exits zero and captures nothing, which is
  invisible unless the decision is reported.
- **Multi-interface capture.** The pipeline takes several sources and runs each
  on its own thread, all feeding the single bounded buffer of section 12.4.
  Every packet carries the identity of the interface it arrived on, from
  acquisition through to both writers.
- **Both writers record more than one interface.** The pcapng writer declares
  each with its own link type and references the correct one from every packet
  block; the JSON Lines writer names the interface on every record. A
  single-interface capture produces byte-identical output to before, checked
  against the committed goldens.
- **Per-interface loss accounting.** `CaptureStats` holds one backend report per
  interface, and the capture-wide view is a computed sum. A kernel drop now
  names the driver buffer that is undersized rather than reporting that one of
  several is.
- **Interface retirement.** A capture thread that fails retires its interface
  and the run continues on the others, ending when the last has retired. The
  report names the interface and the reason. It advances no drop counter,
  because nothing was observed and then discarded.
- **Capture driver detection.** Presence and the loopback installation option
  are detected at runtime and reported with the official download location when
  absent. fragcap never downloads, installs, or invokes an installer.
- **A mechanical P-1 check.** `cargo xtask lint` fails if any fragcap source
  names a transmit call, and if any capture driver binary or software
  development kit file reaches the repository.
