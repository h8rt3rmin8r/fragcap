# Helper and Packet Evidence Contract

An admitted selected flow records `selected`, `proxy-attempt`, and a one-use receipt. Helper completion records its exact stage and elapsed time. Success records `proxy-connected`; failure records a typed reason and the actual I/O kind where applicable. A helper registered before receipt consumption reports `unconsumed` if admission fails; only actual ledger consumption reports `consumed`. A pending first packet lost after owner change or shutdown records both aggregate loss and `selected-packet-disposition: lost`.

The worker does not wait for SOCKS CONNECT. Its pending state is bounded by exact tuple and endpoint generation. A duplicate selected packet is either retained under the same pending owner or explicitly accounted; it cannot begin a second helper. Session stop closes pending helper work within its finite deadline and retires receipts.

Route reachability requires actual client transfer. Passive original-packet presence is an independent field. Original-absent and incomplete-history states never become original-present because a helper packet or later same-tuple packet was observed.

The portable package test checks full route journal, compatibility, terminal result, and manifest readback for original-present, original-absent, truncated, refusal, no-payload, reuse, ambiguity, and TCP/UDP cases. A controlled test result does not assert an installed ESO result.
