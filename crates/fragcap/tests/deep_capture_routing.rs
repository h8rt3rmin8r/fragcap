// SPDX-License-Identifier: Apache-2.0

#![cfg(feature = "deep-capture")]

use fragcap::deep_capture::{
    LoopbackEndpoint, RouteValueSource, RoutingAvailability, RoutingPlan, RoutingStrategyKind,
};

fn plan(inputs: &[String]) -> RoutingPlan {
    RoutingPlan::child_environment(
        LoopbackEndpoint::new("127.0.0.1:8080".parse().unwrap()).unwrap(),
        inputs,
    )
    .unwrap()
}

#[test]
fn every_strategy_is_declared_but_only_available_ones_can_authorize() {
    let available = plan(&[]);
    assert_eq!(available.strategy, RoutingStrategyKind::ChildEnvironment);
    assert_eq!(available.availability, RoutingAvailability::Implemented);
    assert!(available.validate().is_ok());

    let future = RoutingPlan::planned(RoutingStrategyKind::TargetConfiguration);
    assert_eq!(future.availability, RoutingAvailability::Planned);
    assert!(future.validate().is_err());
}

#[test]
fn child_routing_owns_proxy_bypass_and_infrastructure() {
    let plan = plan(&[".example.com".to_string()]);
    for name in ["NO_PROXY", "no_proxy"] {
        let no_proxy = plan
            .effects
            .iter()
            .find(|effect| effect.destination == name)
            .expect("child routing owns proxy bypass");
        assert_eq!(no_proxy.scope, "managed-child-only");
        assert_eq!(
            no_proxy.value,
            RouteValueSource::Literal(".example.com,127.0.0.1:8080".to_string())
        );
    }
}

#[test]
fn child_routing_uses_distinct_http_and_proxy_resolved_socks_values() {
    let plan = plan(&[]);
    for name in ["HTTP_PROXY", "HTTPS_PROXY", "http_proxy", "https_proxy"] {
        assert_eq!(
            plan.effects
                .iter()
                .find(|effect| effect.destination == name)
                .expect("HTTP route is declared")
                .value,
            RouteValueSource::SessionProxyUrl
        );
    }
    for name in ["ALL_PROXY", "all_proxy"] {
        assert_eq!(
            plan.effects
                .iter()
                .find(|effect| effect.destination == name)
                .expect("SOCKS route is declared")
                .value,
            RouteValueSource::SessionSocks5hUrl
        );
    }
}
