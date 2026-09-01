// SPDX-License-Identifier: Apache-2.0

#![cfg(feature = "deep-capture")]

use fragcap::deep_capture::{RoutingAvailability, RoutingPlan, RoutingStrategyKind};

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
