// SPDX-License-Identifier: Apache-2.0

//! A [`FlowAttributor`] backed by a declared script rather than a socket table.
//!
//! The other half of the claim specification section 25.1 makes. With the
//! replay source in `fragcap-capture`, the pipeline becomes a deterministic
//! function from fixture input to output.
//!
//! Contains no packet acquisition, per constitution P-3, and could not:
//! `fragcap-attr` has no dependency edge to `fragcap-capture`, which
//! `cargo xtask deps` enforces.
//!
//! # The clock is here and not on the seam
//!
//! [`FlowAttributor::resolve`] takes a flow key and no timestamp. A real
//! attributor needs none, because it reads a socket table that is already
//! current: "now" is implicit in the data. Only a scripted one has to be told,
//! and port reuse is exactly the case that depends on it.
//!
//! Widening the seam would have been the easy fix and is refused. Slice S02
//! fixed those five traits as the part of the surface intended to reach 1.0.0
//! unchanged, and a test double is a poor reason to hand every real
//! implementation a parameter it does not want. [`ScriptedAttributor::set_now`]
//! is inherent to this type, and the asymmetry stays where it belongs.

use fragcap_core::attribution::Attribution;
use fragcap_core::error::AttrError;
use fragcap_core::flow::{Endpoint, FlowKey};
use fragcap_core::packet::Timestamp;
use fragcap_core::traits::FlowAttributor;

use crate::script::AttributionScript;

/// Answers attribution questions from a script.
#[derive(Clone, Debug)]
pub struct ScriptedAttributor {
    script: AttributionScript,
    now: Timestamp,
}

impl ScriptedAttributor {
    /// Defaults to the epoch, so a script of `always` entries resolves without
    /// the caller ever setting a clock.
    pub fn new(script: AttributionScript) -> Self {
        ScriptedAttributor {
            script,
            now: Timestamp::from_nanos(0),
        }
    }

    /// Tell the attributor what time it is.
    ///
    /// Not on the [`FlowAttributor`] seam, and must not be. The caller knows
    /// the timestamp of the packet it is about to attribute; in tier 1 that is
    /// a test, and in S08 it will be the pipeline.
    pub fn set_now(&mut self, now: Timestamp) {
        self.now = now;
    }

    pub fn now(&self) -> Timestamp {
        self.now
    }

    pub fn script(&self) -> &AttributionScript {
        &self.script
    }
}

impl FlowAttributor for ScriptedAttributor {
    /// The owner the script declares for this flow at the instant last set.
    ///
    /// `None` covers both "declared unowned" and "not mentioned", which is the
    /// same distinction a real attributor cannot make either: both are
    /// attempted and unresolved, and the packet is retained and marked per
    /// constitution P-4.
    fn resolve(&self, key: &FlowKey) -> Option<Attribution> {
        self.script.resolve(key, self.now)
    }

    /// Succeeds and does nothing. There is no table behind this to re-read.
    fn refresh(&mut self) -> Result<(), AttrError> {
        Ok(())
    }

    fn active_endpoints(&self) -> Vec<Endpoint> {
        self.script.endpoints().to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fragcap_core::flow::Proto;
    use std::net::SocketAddr;

    fn addr(s: &str) -> SocketAddr {
        s.parse().expect("test address must parse")
    }

    fn tcp_key() -> FlowKey {
        FlowKey::new(
            Proto::Tcp,
            addr("192.0.2.10:51000"),
            addr("198.51.100.5:443"),
        )
    }

    fn attributor(text: &str) -> ScriptedAttributor {
        ScriptedAttributor::new(AttributionScript::parse(text).expect("the script loads"))
    }

    #[test]
    fn an_always_entry_resolves_with_no_clock_ever_set() {
        let a = attributor("flow tcp 192.0.2.10:51000 198.51.100.5:443 always owner 42 game.exe");
        assert_eq!(a.now(), Timestamp::from_nanos(0));
        assert_eq!(a.resolve(&tcp_key()).expect("always resolves").pid, 42);
    }

    // SC-006 through the seam rather than through the script directly.
    #[test]
    fn the_clock_selects_the_window() {
        let mut a = attributor(
            "flow tcp 192.0.2.10:51000 198.51.100.5:443 100..200 owner 1 first.exe\n\
             flow tcp 192.0.2.10:51000 198.51.100.5:443 200..300 owner 2 second.exe\n",
        );
        a.set_now(Timestamp::from_nanos(150));
        assert_eq!(a.resolve(&tcp_key()).expect("in the first").pid, 1);
        a.set_now(Timestamp::from_nanos(250));
        assert_eq!(a.resolve(&tcp_key()).expect("in the second").pid, 2);
        a.set_now(Timestamp::from_nanos(500));
        assert_eq!(a.resolve(&tcp_key()), None, "outside both");
    }

    #[test]
    fn an_unresolved_flow_is_not_an_error() {
        let a = attributor("# nothing declared\n");
        assert_eq!(
            a.resolve(&tcp_key()),
            None,
            "attempted and unresolved, which P-4 says is retained and marked"
        );
    }

    #[test]
    fn refreshing_succeeds_and_changes_nothing() {
        let mut a = attributor("flow tcp 192.0.2.10:51000 198.51.100.5:443 always owner 1 a.exe");
        let before = a.resolve(&tcp_key());
        assert!(a.refresh().is_ok());
        assert_eq!(a.resolve(&tcp_key()), before);
    }

    // FR-023, the requirement the analyze gate found uncovered.
    #[test]
    fn active_endpoints_reports_what_the_script_declares() {
        let a = attributor(
            "endpoint tcp 192.0.2.10:51000\n\
             endpoint udp 192.0.2.10:30000\n",
        );
        let endpoints = a.active_endpoints();
        assert_eq!(endpoints.len(), 2);
        assert_eq!(endpoints[0].addr, addr("192.0.2.10:51000"));
        assert_eq!(endpoints[0].proto, Proto::Tcp);
        assert_eq!(endpoints[1].proto, Proto::Udp);
    }

    #[test]
    fn a_script_with_no_endpoints_reports_none() {
        let a = attributor("flow tcp 192.0.2.10:51000 198.51.100.5:443 always unowned");
        assert!(a.active_endpoints().is_empty());
    }

    // SC-006b. The seam is unchanged: this compiles only while `resolve` takes
    // a flow key and nothing else, and while the trait stays dyn compatible.
    #[test]
    fn the_flow_attributor_seam_is_unwidened_and_still_dyn_compatible() {
        let mut a = attributor("flow tcp 192.0.2.10:51000 198.51.100.5:443 always owner 5 a.exe");
        a.set_now(Timestamp::from_nanos(1));
        let seam: &mut dyn FlowAttributor = &mut a;
        assert_eq!(seam.resolve(&tcp_key()).expect("resolves").pid, 5);
        assert!(seam.refresh().is_ok());
        assert!(seam.active_endpoints().is_empty());

        // The clock is reachable only off the seam. If `set_now` ever moves
        // onto the trait, this comment is the record that it was deliberate.
        fn assert_send<T: Send + ?Sized>() {}
        assert_send::<dyn FlowAttributor>();
    }

    #[test]
    fn a_scripted_attributor_is_usable_as_a_boxed_trait_object() {
        let boxed: Box<dyn FlowAttributor> =
            Box::new(attributor("flow udp 0.0.0.0:30000 * always owner 9 g.exe"));
        let udp = FlowKey::new(
            Proto::Udp,
            addr("192.0.2.10:30000"),
            addr("198.51.100.5:5055"),
        );
        assert_eq!(
            boxed.resolve(&udp).expect("the wildcard bind owns it").pid,
            9
        );
    }
}
