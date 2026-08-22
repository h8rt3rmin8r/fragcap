<!-- spec-impact: 16.3 -->
`fragcap steam list` now prints a header naming its columns, instead of two
tab-separated fields a reader had to guess the meaning of. Each row is joined
against the local store by its exact `steam:<app_id>` anchor (never by name,
which two different installed titles can share) and shows one of three
distinct states: registered and positioned in the most recent `fragcap
targets` listing (its handle and row index), registered but not positioned
(its handle only), or not registered at all. `steam list` only ever reads that
listing snapshot; it never writes it, so running it does not change what
`fragcap capture <n>` resolves to. Rows sort by title name, case-insensitive,
tie-broken by app id, so the order is the same on repeated runs instead of the
prior incidental app-id-as-string ordering. With no local store, or an
unopenable one, the listing still succeeds and still names every installed
title, with a warning that identity could not be joined.
