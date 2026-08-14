# fragcap-targets

The targets hint database for fragcap.

## Status

Foundation implemented as of slice S034 (issue #78): an embedded SQLite store of
known game binaries and launch patterns, the three-tier seeding model (public
catalog, launch metadata, community engine data), and a schema-conformant JSON
export. This slice adds no network fetching; the store is populated offline from
a seed document, and the seeders that fill it from the Steam Web API, PICS, and
PCGamingWiki arrive in later slices.

The database is the provider at precedence 2 of the resolution cascade (issue
#77), stamping every hint `heuristic-unverified`: it is never a source of truth,
and a live runtime observation always overrides it. The JSON export conforms to
the `kind: "export"` variant of the published target schema, so an unmodified
schema validator reads it.

## About fragcap

fragcap is a passive, process-attributed network capture tool for Windows,
written in Rust. Packet capture is a solved problem; attribution is not.
Standard tooling captures at the network driver, below the socket layer, where
the association between a packet and the process that produced it has already
been discarded. fragcap reconstructs that association for game clients launched
indirectly through platform and publisher launchers, and writes it into an
extended pcapng profile that unmodified analyzers still read as ordinary
pcapng.

## License

Apache-2.0. See `LICENSE` and `NOTICE`.
