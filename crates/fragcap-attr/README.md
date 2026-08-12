# fragcap-attr

Flow attribution and process tree watching for fragcap.

## Status

Functional as of 0.2.0, the first functional release.

`SocketTableAttributor` implements specification section 11: it snapshots the
operating system socket table, joins captured flows against it by 5-tuple,
keeps a closing connection's tail attributed through a retention window, and
publishes each snapshot as an immutable value that any number of capture
threads read without locking.

The process watcher tracks process start and exit through Event Tracing for
Windows and maintains the process tree, so an attribution can carry a process
identifier, an image name, and a role from profile stage matching.

`ScriptedAttributor` remains, and remains useful: it answers from a declared
script rather than a socket table, which is what makes the whole pipeline
testable with no capture driver, no elevation, and no game.

## Features

| Feature | Default | What it gates |
| --- | --- | --- |
| `socket-table` | off | The Windows backends: the IP Helper socket table and query-only process enumeration |

The feature is off by default so the ordinary check set passes on any machine.
It is deliberately not named `live`, which is `fragcap-capture`'s feature and
means "links against the npcap import library". This one needs no npcap at all:
the IP Helper API ships with the operating system, so the backend builds and
runs on a bare Windows machine with no capture driver installed.

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

In this crate that is stronger than it sounds. Image names come from a toolhelp
process enumeration, which returns them in the snapshot, so no process handle
is opened at all and there are no access rights to audit. `cargo xtask lint`
fails if any fragcap source names a process-opening call, which makes the
argument mechanical rather than remembered.

## License

Apache-2.0. See [`LICENSE`](LICENSE) and [`NOTICE`](NOTICE).

npcap is required at runtime but is never bundled, downloaded, or installed by
fragcap, and its Software Development Kit is never vendored. Obtain it from
[npcap.com](https://npcap.com) and review its license terms there.

[repo]: https://github.com/h8rt3rmin8r/fragcap
