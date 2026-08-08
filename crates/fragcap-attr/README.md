# fragcap-attr

Flow attribution and process tree watching for fragcap.

## Status

**This release is a skeleton. It contains no functionality.**

Version 0.1.0 reserves the name and fixes the crate boundary.
The socket table attributor arrives in slice S10 and the process watcher in
slice S11.
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

Attribution reads the system socket table and watches process lifetime
events. It never opens a process handle carrying memory-read rights, never
injects code, and never hooks a function; the technique denylist in
constitution principle P-1 is absolute.

## License

Apache-2.0. See [`LICENSE`](LICENSE) and [`NOTICE`](NOTICE).

npcap is required at runtime but is never bundled, downloaded, or installed by
fragcap, and its Software Development Kit is never vendored. Obtain it from
[npcap.com](https://npcap.com) and review its license terms there.

[repo]: https://github.com/h8rt3rmin8r/fragcap
