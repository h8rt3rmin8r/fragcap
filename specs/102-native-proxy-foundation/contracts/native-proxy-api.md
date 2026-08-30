# Native Proxy API Contract

The publishable `fragcap-proxy` crate owns native effects and depends on no fragcap crate. The `fragcap` facade depends on it behind the existing `deep-capture` feature and adapts its values to `ProxyBackend` and `ProxyLease`. `fragcap-proxy` never depends on `fragcap-cli`.

## Construction

`NativeProxyBackend::new(config)` validates all non-effectful constraints. `identity()` returns the stable typed identity and capability set.

## Start and observation

`start()` binds the exact loopback endpoint, starts one owned runtime thread, and returns `NativeProxyLease`. A failure returns a typed `StartError` and leaves no listener or task. `observation(budget)` returns lifecycle and accounting state within the caller budget. The S102 facade adapter returns no compatibility observation because the foundation performs no application inspection.

## Stop and cleanup

`stop(budget)` closes the listener, signals all connection tasks, drains within the smaller configured/caller timeout, and joins the runtime thread within the caller budget. If the owner thread does not finish by that deadline, the report names the residue and the lease retains the join handle so a later `cleanup(budget)` can retry. A successfully joined terminal result is cached, making repeated stop and cleanup calls idempotent.

## Refusals

- Non-loopback bind addresses are invalid.
- Zero capacity, buffer, or timeout values are invalid.
- Saturation closes the newly accepted socket and increments the saturation counter.
- The foundation never reports forwarding, HTTP observation, TLS inspection, or target compatibility.
