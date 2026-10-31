# Data Model: S186 Installed Deep Capture truth

## Public installation health

The existing public CA identity and exact machine-root observation determine the read-only health state. A separate signer-proof field distinguishes `unverified` read-only observation from a privileged proof or confirmed private failure. No ordinary-account private key bytes or ACL expansion enter this model.

## Tunnel request authority

The immutable CONNECT destination, client SNI, and each raw inner HTTP request remain distinct observations. The parsed request derives an effective scheme and port from its actual tunnel context, then compares normalized host and port to the CONNECT destination. Raw Host and target bytes remain available for evidence. An upstream connection retains the exact destination whose TLS certificate was verified.

## Terminal and compatibility evidence

Each accepted connection has one terminal cause and separately counted protocol stages. HTTPS inspectability requires a completed supported application exchange. Partial HTTP or timeout records cannot advance it. Existing packet, application, HAR, and cleanup artifact authorities remain independent.
