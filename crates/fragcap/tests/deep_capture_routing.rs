// SPDX-License-Identifier: Apache-2.0

#![cfg(feature = "deep-capture")]

use fragcap::deep_capture::{
    RouteValueSource, RoutingAvailability, RoutingPlan, RoutingStrategyKind,
};

#[test]
fn every_strategy_is_declared_but_only_available_ones_can_authorize() {
    let available = RoutingPlan::child_environment();
    assert_eq!(available.strategy, RoutingStrategyKind::ChildEnvironment);
    assert_eq!(available.availability, RoutingAvailability::Implemented);
    assert!(available.validate().is_ok());

    let future = RoutingPlan::planned(RoutingStrategyKind::TargetConfiguration);
    assert_eq!(future.availability, RoutingAvailability::Planned);
    assert!(future.validate().is_err());
}

#[test]
fn child_routing_clears_inherited_proxy_bypass() {
    let plan = RoutingPlan::child_environment();
    let no_proxy = plan
        .effects
        .iter()
        .find(|effect| effect.destination == "NO_PROXY")
        .expect("child routing owns proxy bypass");
    assert_eq!(no_proxy.scope, "managed-child-only");
    assert_eq!(no_proxy.value, RouteValueSource::Literal(String::new()));
}
