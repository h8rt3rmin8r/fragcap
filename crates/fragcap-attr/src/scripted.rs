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
//! # The instant comes through the seam
//!
//! [`FlowAttributor::resolve`] takes the packet's own timestamp. This slice
//! first tried to keep it off the seam, on the reasoning that a real attributor
//! reads a socket table that is already current so the instant is implicit, and
//! carried the clock as an inherent method on this type instead.
//!
//! That was wrong twice over, and review of pull request 7 found both. It does
//! not survive specification section 11.4, which says capture and socket table
//! observation are not synchronized and that a closing connection produces
//! final packets processed after the socket has left the table: the question is
//! always who owned the flow *then*. And it does not survive the pipeline,
//! which holds a `Box<dyn FlowAttributor>` and therefore could never have
//! called an inherent method, leaving every time-windowed fixture stuck at the
//! epoch and resolving nothing.

use fragcap_core::attribution::Attribution;
use fragcap_core::error::AttrError;
use fragcap_core::flow::{Endpoint, FlowKey};
use fragcap_core::packet::Timestamp;
use fragcap_core::traits::FlowAttributor;

use crate::script::AttributionScript;

/// Answers attribution questions from a script.
///
/// Holds no clock of its own. Every answer is a function of the flow and the
/// instant the caller passes, which is what lets a pipeline holding this behind
/// a trait object drive a time-windowed script at all.
#[derive(Clone, Debug)]
pub struct ScriptedAttributor {
    script: AttributionScript,
}

impl ScriptedAttributor {
    pub fn new(script: AttributionScript) -> Self {
        ScriptedAttributor { script }
    }

    pub fn script(&self) -> &AttributionScript {
        &self.script
    }
}

impl FlowAttributor for ScriptedAttributor {
    /// The owner the script declares for this flow at `at`.
    ///
    /// `None` covers both "declared unowned" and "not mentioned", which is the
    /// same distinction a real attributor cannot make either: both are
    /// attempted and unresolved, and the packet is retained and marked per
    /// constitution P-4.
    fn resolve(&self, key: &FlowKey, at: Timestamp) -> Option<Attribution> {
        self.script.resolve(key, at)
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

    fn at(n: i64) -> Timestamp {
        Timestamp::from_nanos(n)
    }

    #[test]
    fn an_always_entry_resolves_at_any_instant() {
        let a = attributor("flow tcp 192.0.2.10:51000 198.51.100.5:443 always owner 42 game.exe");
        for t in [i64::MIN, 0, 1_700_000_000_000_000_000, i64::MAX] {
            assert_eq!(
                a.resolve(&tcp_key(), at(t)).expect("always resolves").pid,
                42
            );
        }
    }

    // SC-006 through the seam rather than through the script directly.
    #[test]
    fn the_instant_selects_the_window() {
        let a = attributor(
            "flow tcp 192.0.2.10:51000 198.51.100.5:443 100..200 owner 1 first.exe\n\
             flow tcp 192.0.2.10:51000 198.51.100.5:443 200..300 owner 2 second.exe\n",
        );
        assert_eq!(a.resolve(&tcp_key(), at(150)).expect("in the first").pid, 1);
        assert_eq!(
            a.resolve(&tcp_key(), at(250)).expect("in the second").pid,
            2
        );
        assert_eq!(a.resolve(&tcp_key(), at(500)), None, "outside both");
    }

    #[test]
    fn an_unresolved_flow_is_not_an_error() {
        let a = attributor("# nothing declared\n");
        assert_eq!(
            a.resolve(&tcp_key(), at(0)),
            None,
            "attempted and unresolved, which P-4 says is retained and marked"
        );
    }

    #[test]
    fn refreshing_succeeds_and_changes_nothing() {
        let mut a = attributor("flow tcp 192.0.2.10:51000 198.51.100.5:443 always owner 1 a.exe");
        let before = a.resolve(&tcp_key(), at(0));
        assert!(a.refresh().is_ok());
        assert_eq!(a.resolve(&tcp_key(), at(0)), before);
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

    // SC-006b, inverted by review of pull request 7. The property that matters
    // is not that the seam is narrow but that a time-windowed script is drivable
    // through it, because a pipeline holds this as a trait object and can reach
    // nothing else.
    #[test]
    fn a_time_windowed_script_is_drivable_through_the_seam_alone() {
        let a = attributor(
            "flow tcp 192.0.2.10:51000 198.51.100.5:443 100..200 owner 1 first.exe\n\
             flow tcp 192.0.2.10:51000 198.51.100.5:443 200..300 owner 2 second.exe\n",
        );
        let seam: Box<dyn FlowAttributor> = Box::new(a);
        // No inherent method is reachable here. If the instant were not on the
        // seam, this test could not be written and S08 could not drive
        // port-reuse.pcap at all.
        assert_eq!(seam.resolve(&tcp_key(), at(150)).expect("first").pid, 1);
        assert_eq!(seam.resolve(&tcp_key(), at(250)).expect("second").pid, 2);
        assert_eq!(seam.resolve(&tcp_key(), at(50)), None);

        fn assert_send<T: Send + ?Sized>() {}
        assert_send::<dyn FlowAttributor>();
    }

    #[test]
    fn a_boxed_attributor_still_takes_the_wildcard_bind_allowance() {
        let boxed: Box<dyn FlowAttributor> =
            Box::new(attributor("flow udp 0.0.0.0:30000 * always owner 9 g.exe"));
        let udp = FlowKey::new(
            Proto::Udp,
            addr("192.0.2.10:30000"),
            addr("198.51.100.5:5055"),
        );
        assert_eq!(
            boxed
                .resolve(&udp, at(0))
                .expect("the wildcard bind owns it")
                .pid,
            9
        );
    }
}
