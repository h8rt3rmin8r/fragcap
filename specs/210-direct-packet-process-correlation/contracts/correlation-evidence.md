# Correlation Evidence Contract

## Direct route

Version 2 direct-route reconciliation records bind a one-use receipt to the original tuple and exact process-generation ID. They declare the selected packet disposition, helper admission, client delivery and return acknowledgement or UDP peer reply, proxy transfer, and passive original-packet state separately. An accepted helper connection cannot alone prove client delivery. The packet count represents only passive original-flow observations; a missing first packet remains unavailable even when later packets exist.

The route journal records `receipt-consumed` and `selected-packet-disposition` with evidence version 2. Application and compatibility records carry the nested state, while the manifest carries bounded route and packet-state counts under correlation contract version 3. The curated Rust API advances to major version 2 because its public compatibility observation gains an optional direct-route field. A failed selected-packet delivery is `lost`, distinct from a consumed packet with later unproved transfer.

## Process trace

Version 2 `socket-owner.interval` records may add `socket_created_at_nanos`, `tcp_flags`, and `tail_basis`. A post-exit record names an instance only when exact generation, image, socket creation, finite lag, and complete event history are proven. ACK/FIN adds prior same-flow in-lifetime ownership and no conflicting reuse. Version 1 readers retain their existing meaning; version 2 readers reject missing mandatory trailer fields and mismatched record counts.

## Cross-artifact invariant

Every positive route result has one exact receipt and demonstrated transfer. Every packet-present claim has a matching passive observation. Every post-exit owner link has an explicit basis and preserves the process's true exit timestamp. Terminal and manifest counts derive from these records without inventing gameplay or packet visibility.
