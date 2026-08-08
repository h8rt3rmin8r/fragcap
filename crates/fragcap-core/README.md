# fragcap-core

Types, traits, and the capture pipeline for fragcap. Platform-neutral.

## Status

**This release is a skeleton. It contains no functionality.**

Version 0.1.0 reserves the name and fixes the crate boundary.
The types and traits arrive in slice S02.
Depending on this version buys nothing. Follow [the repository][repo]
for progress.

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

This crate is platform-neutral by constitution principle P-2: it takes no
platform-specific dependency, no I/O crate, and no capture library.
Continuous integration proves it by building this crate for a target where no
capture backend exists.

## License

Apache-2.0. See [`LICENSE`](LICENSE) and [`NOTICE`](NOTICE).

npcap is required at runtime but is never bundled, downloaded, or installed by
fragcap, and its Software Development Kit is never vendored. Obtain it from
[npcap.com](https://npcap.com) and review its license terms there.

[repo]: https://github.com/h8rt3rmin8r/fragcap
