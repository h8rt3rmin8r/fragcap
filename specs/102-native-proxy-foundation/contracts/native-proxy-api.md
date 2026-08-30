# Native Proxy API Contract

The publishable `fragcap-proxy` crate owns native effects and depends on no fragcap crate. The `fragcap` facade depends on it behind the existing `deep-capture` feature and adapts its values to `ProxyBackend` and `ProxyLease`. `fragcap-proxy` never depends on `fragcap-cli`.

## Construction

`NativeProxyBackend::new(config)` validates all non-effectful constraints. `identity()` returns the stable typed identity and capability set.

## Start and observation

`start()` binds the exact loopback endpoint, starts one owned runtime thread, and returns `NativeProxyLease`. A failure returns a typed `StartError` and leaves no listener or task. `observation(budget)` returns lifecycle and accounting state within the caller budget. The S102 facade adapter returns no compatibility observation because the foundation performs no application inspection.

## Stop and cleanup

`stop(budget)` closes the listener, signals all connection tasks, drains within the smaller configured/caller timeout, forces remaining tasks down, joins the runtime thread, and returns a complete `ShutdownReport`. `cleanup(budget)` is idempotent and returns the cached terminal result after stop.

## Refusals

- Non-loopback bind addresses are invalid.
- Zero capacity, buffer, or timeout values are invalid.
- Saturation closes the newly accepted socket and increments the saturation counter.
- The foundation never reports forwarding, HTTP observation, TLS inspection, or target compatibility.
