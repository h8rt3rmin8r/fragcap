### Added

- **Extcap analyzer integration: fragcap is a capture source in Wireshark**
  (specification section 14.5, roadmap slice S18). `fragcap extcap` implements the
  four-invocation extcap contract: `--extcap-interfaces` lists the one `fragcap`
  interface, `--extcap-dlts` its link type, `--extcap-config` the configurable
  options, and `--capture --fifo <path>` streams pcapng to the analyzer's FIFO.
  The analyzer renders a native configuration dialog from the declaration, so a
  full graphical interface exists with no graphical code in fragcap.
- **The configurable options are profile, roles, direction, and loopback**, and
  they select the capture through the same overlay the `run` command uses: the
  option call names are the `run` flag names, so the analyzer's dialog and the
  command line select capture identically. A profile that fails to validate is a
  configuration error reported before any capture starts, not a started-but-empty
  stream.
- **The extcap stream is the same bytes a file capture produces.** The FIFO sink
  reuses the pcapng writer through the existing sink factory, so an unmodified
  analyzer reads a process-attributed live capture; a single-interface extcap
  stream is byte-comparable to a plain `--out` file capture of the same input, and
  the pipeline conservation identity (received + buffer_dropped + refusals =
  captured) holds exactly as for a file capture.
- **`fragcap doctor` reports the analyzer extcap integration.** It names the
  analyzer's extcap directory and reports whether a fragcap binary is installed
  there, in both the installed and not-installed states. The probe reads the
  directory read-only and installs, downloads, and copies nothing; installation
  is an operator action (copy the binary into the reported directory).
- **A `fifo:` sink scheme** streams pcapng to a FIFO or named-pipe path, opened
  for writing (a named-pipe client on Windows, an opened FIFO elsewhere). It is
  the transport the extcap capture uses and is available to `--sink` as well.
