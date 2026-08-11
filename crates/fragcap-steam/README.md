# fragcap-steam

Steam platform integration for fragcap.

## Status

Implemented as of slice S17: library discovery, profile scaffolding, and managed
launch. The Windows-only internals (the registry read for the Steam install path
and the `steam://` protocol handler) are behind `#[cfg(windows)]`, so the crate
builds on every target; the parser and the scaffolding classifier are portable.

## About fragcap

fragcap is a passive, process-attributed network capture tool for Windows,
written in Rust. Packet capture is a solved problem; attribution is not.
Standard tooling captures at the network driver, below the socket layer, where
the association between a packet and the process that produced it has already
been discarded. fragcap reconstructs that association for game clients
launched indirectly through platform and publisher launchers, and writes it
into an extended pcapng profile that unmodified analyzers still read as
ordinary pcapng.

It observes. It does not modify traffic, and it does not reach inside the
processes it names.

## This crate

Library discovery, profile scaffolding, and managed launch. Contains no
capture logic and no attribution logic, and opens no process handle.

- **Discovery** reads Steam's local metadata (`libraryfolders.vdf` and every
  `appmanifest_*.acf`, in Valve's key-value text format) to enumerate installed
  titles with their application identifiers and install paths.
- **Scaffolding** (`fragcap steam profile <app_id>`) generates a profile skeleton
  for an installed title. It is a heuristic starting point, marked as such, and
  is validated before it is emitted.
- **Managed launch** (`fragcap run --launch`) starts a title through Steam's
  protocol handler once fragcap is already watching, so the launch chain is
  observed without an acquisition race. It reads no Steam metadata that requires a
  running Steam client and installs nothing.

## License

Apache-2.0. See [`LICENSE`](LICENSE) and [`NOTICE`](NOTICE).

npcap is required at runtime but is never bundled, downloaded, or installed by
fragcap, and its Software Development Kit is never vendored. Obtain it from
[npcap.com](https://npcap.com) and review its license terms there.

[repo]: https://github.com/h8rt3rmin8r/fragcap
