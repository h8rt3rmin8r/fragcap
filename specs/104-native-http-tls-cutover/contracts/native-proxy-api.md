# Native Proxy Cutover API Contract

## Start and borrowed access

`NativeProxyBackend::start` binds loopback, creates one fresh capability and session authority, prepares the native-root upstream configuration, and starts the owned runtime. It returns only after the listener can accept authenticated HTTP proxy traffic.

`NativeProxyLease::access` returns a borrowed view:

```text
ProxySessionAccess
  endpoint: LoopbackEndpoint
  route: ProxyLaunchRoute<'lease>
  trust: ProxyTrustMaterial<'lease>
```

The route can construct child-only proxy environment values but cannot be serialized, displayed with credentials, or retained beyond the lease borrow. Trust material exposes public DER and exact identity only.

## Facade ordering

```text
prepare effect-free plan
authorize exact plan
start native proxy
borrow access
acquire exact current-user trust when planned
apply route to exact retained managed launch
start selected launch and ordinary Capture
observe
stop Capture and proxy
drain native observations
cleanup launch, trust, and proxy obligations
finalize facts and artifacts
```

Any failure runs bounded cleanup for resources already acquired. No access material exists on preflight, decline, stale authorization, or proxy-start failure.

## Authentication

- Scheme: HTTP Basic proxy authorization.
- Username: exact ASCII `fragcap`.
- Password: URL-safe unpadded Base64 of the 32-byte session capability.
- Input: exactly one bounded `Proxy-Authorization` field per request.
- Failure: HTTP 407 with a stable proxy-authenticate challenge, no DNS/connect/certificate/body-retention work, and one refusal outcome.
- Success: strict outer Basic decode, strict inner capability decode, constant-time proof comparison, immediate zeroization of temporary secret buffers, and removal of the proxy-only field before upstream forwarding.

## Shutdown

The lease is the sole owner of the listener, capability, authority, leaf cache, connection supervisors, upstream transports, and observation stream. Stop rejects new accepts, signals every supervisor, drains within the smaller configured or caller budget, aborts and joins remainder, zeroizes private state, and returns one reconciled report. Repeated stop/cleanup returns the same terminal truth.

## Refusals

- No standard authorization: `proxy-auth-required`.
- Wrong or stale capability: `proxy-auth-refused`.
- Ambiguous framing: `http-framing-ambiguous`.
- Message limit exceeded: a specific `*-limit-exceeded` code.
- Invalid or disallowed destination: existing authority/policy code.
- Client TLS identity mismatch or issue failure: client-boundary TLS code.
- Upstream certificate/name failure: upstream-boundary TLS code.
- Unsupported negotiated protocol: explicit unsupported outcome, never implicit fallback.
