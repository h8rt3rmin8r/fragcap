# fragcap

Facade crate re-exporting the fragcap library surface.

## Status

**This release is a skeleton. It contains no functionality.**

Version 0.1.0 reserves the name and fixes the crate boundary.
Re-exports arrive as the crates beneath it gain surface, beginning with
slice S02.
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

The library is the product; the command line tool is one consumer of it.
Anything reachable through the command line is reachable through this crate.
Install the `fragcap` binary from the `fragcap-cli` crate instead.

## License

Apache-2.0. See [`LICENSE`](LICENSE) and [`NOTICE`](NOTICE).

npcap is required at runtime but is never bundled, downloaded, or installed by
fragcap, and its Software Development Kit is never vendored. Obtain it from
[npcap.com](https://npcap.com) and review its license terms there.

[repo]: https://github.com/h8rt3rmin8r/fragcap
