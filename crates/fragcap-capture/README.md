# fragcap-capture

Packet acquisition backends for fragcap.

## Status

Functional as of 0.2.0, the first functional release. This crate reads classic
pcap and replays it as a `PacketSource`, and, behind the `live` feature, binds
the npcap capture driver for live acquisition. The offline replay path needs no
driver and no elevation; live capture links against the npcap import library and
runs where that driver is installed.

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

Backends implement the `PacketSource` seam. Acquisition is kept separate
from attribution by constitution principle P-3: the two have different
platform requirements, different failure modes, and different upgrade paths,
and separating them is what makes the pipeline testable offline.

## License

Apache-2.0. See [`LICENSE`](LICENSE) and [`NOTICE`](NOTICE).

npcap is required at runtime but is never bundled, downloaded, or installed by
fragcap, and its Software Development Kit is never vendored. Obtain it from
[npcap.com](https://npcap.com) and review its license terms there.

[repo]: https://github.com/h8rt3rmin8r/fragcap
