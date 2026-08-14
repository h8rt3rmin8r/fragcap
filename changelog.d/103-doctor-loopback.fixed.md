`fragcap doctor` now reports loopback capture support from the actual loopback
adapter rather than an unrelated driver file, so a machine that has loopback
support installed is no longer told it is missing; when the state cannot be
determined the report says so rather than claiming loopback is absent.
