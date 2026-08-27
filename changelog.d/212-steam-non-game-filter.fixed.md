<!-- spec-impact: 16.2, 17.7 -->
Steam discovery no longer offers Steam utility, application, configuration, or video records as capture targets. Those entries are counted as not-a-game, while demos and titles with unknown app type remain eligible.
Composed discovery also passes current Steam non-game install roots to known-roots so those directories are not reintroduced as path candidates, and the target listing hides existing platform-created rows for current Steam non-game installs without deleting user-authored entries.
