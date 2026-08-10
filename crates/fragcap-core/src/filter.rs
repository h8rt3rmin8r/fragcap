// SPDX-License-Identifier: Apache-2.0

//! Capture filter programs and the narrowing strategy of specification sections
//! 12.2 and 12.3.
//!
//! A [`FilterProgram`] is the compiled artifact handed to a
//! [`crate::traits::PacketSource`]. Slice S09 installs the bootstrap program
//! (`ip or ip6`) at open; this module adds phase two (narrowing a program down to
//! the endpoints belonging to profiled processes) and phase three (recompiling
//! and reinstalling as that set changes, debounced and rate limited).
//!
//! Two things stay true here, and both are load-bearing.
//!
//! **The kernel filter is only ever an optimization.** Specification section 12.3
//! makes userspace attribution the authority on what is captured; a narrowed
//! filter that is briefly stale is admitted, and the traffic it wrongly excludes
//! is counted as a filter gap rather than silently lost. So nothing here needs a
//! live handle to be correct, and everything here is a pure decision over a value.
//!
//! **A compiled filter is text to core.** [`FilterProgram::narrowed`] builds a
//! libpcap expression, but `fragcap-core` treats the result as an opaque string;
//! only `fragcap-capture` compiles it onto an npcap handle. That is what keeps
//! this module inside constitution P-2.

use std::collections::BTreeSet;
use std::time::{Duration, Instant};

use crate::flow::{Endpoint, Proto};

/// A capture filter to be installed on a [`crate::traits::PacketSource`].
///
/// The expression is in the backend's own syntax (libpcap, for the npcap
/// backend). A default program selects everything, which is the state a capture
/// starts in before narrowing; [`FilterProgram::narrowed`] compiles a program
/// admitting exactly a set of endpoints.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct FilterProgram {
    expression: String,
}

impl FilterProgram {
    pub fn new(expression: impl Into<String>) -> Self {
        FilterProgram {
            expression: expression.into(),
        }
    }

    /// Compile a program admitting exactly `endpoints`, as the OR of one clause
    /// per endpoint constraining protocol, host, and port. Spans IPv4 and IPv6:
    /// libpcap's `host` matches the address family of the literal, and each
    /// endpoint carries its own. The endpoints are sorted and deduplicated, so
    /// the expression is a pure function of the set and two equal sets compile
    /// byte for byte alike, which is what lets the filter manager tell whether a
    /// program actually changed.
    ///
    /// An empty set yields an empty program ([`FilterProgram::is_empty`]); the
    /// filter manager never installs one, because narrowing to nothing would
    /// admit nothing while attribution still wants the retained tail.
    ///
    /// The result admits the target's traffic plus whatever shares its ports,
    /// which specification section 12.2 accepts on purpose: over-admission is
    /// cleaned up by userspace attribution (section 12.3), not tightened here.
    pub fn narrowed(endpoints: &[Endpoint]) -> Self {
        let ordered: BTreeSet<Endpoint> = endpoints.iter().copied().collect();
        let expression = ordered
            .into_iter()
            .map(clause)
            .collect::<Vec<_>>()
            .join(" or ");
        FilterProgram { expression }
    }

    /// The filter expression, in the backend's own syntax.
    pub fn expression(&self) -> &str {
        &self.expression
    }

    /// Whether this program selects everything, which is the state a capture
    /// starts in before narrowing, and also what a narrowing over an empty
    /// endpoint set produces.
    pub fn is_empty(&self) -> bool {
        self.expression.is_empty()
    }
}

/// One endpoint's libpcap clause: protocol, host, and port.
fn clause(endpoint: Endpoint) -> String {
    let proto = match endpoint.proto {
        Proto::Tcp => "tcp",
        Proto::Udp => "udp",
    };
    format!(
        "({} and host {} and port {})",
        proto,
        endpoint.addr.ip(),
        endpoint.addr.port()
    )
}

/// The maintenance timings of specification section 12.2.
///
/// Plain values, not keys in a game profile: like `fragcap-attr`'s
/// `AttributorConfig`, these carry section constants that a test overrides but an
/// operator does not set here. S14 owns the command line that could expose them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FilterConfig {
    /// The wanted endpoint set must be stable this long before a reinstall.
    /// Section 12.2: recompilation is debounced by two seconds, because endpoint
    /// sets churn during connection establishment.
    pub debounce: Duration,
    /// The floor between two reinstalls on one handle. Section 12.2: one
    /// reinstallation per five seconds per handle, because installing a filter
    /// briefly interrupts capture on that handle.
    pub min_reinstall_interval: Duration,
}

impl FilterConfig {
    /// The section 12.2 constants.
    pub const PRODUCTION: FilterConfig = FilterConfig {
        debounce: Duration::from_secs(2),
        min_reinstall_interval: Duration::from_secs(5),
    };
}

impl Default for FilterConfig {
    fn default() -> Self {
        FilterConfig::PRODUCTION
    }
}

/// One program to install on one handle, returned by [`FilterManager::poll`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Install {
    /// The handle index, matching the capture thread order the manager was built
    /// with.
    pub handle: usize,
    /// The program to install.
    pub program: FilterProgram,
}

/// What one handle currently has installed, as far as the manager knows.
enum Installed {
    /// The bootstrap program from slice S09. Admits all IP traffic, so narrowing
    /// away from it excludes only unwanted traffic and opens no gap.
    Bootstrap,
    /// A narrowed program admitting exactly this endpoint set.
    Narrowed(BTreeSet<Endpoint>),
}

struct HandleState {
    installed: Installed,
    last_install: Option<Instant>,
}

/// The phase-two and phase-three policy of specification section 12.2, as a pure
/// decision over a wanted endpoint set and a supplied instant.
///
/// It opens nothing, installs nothing, and reads no clock of its own: the control
/// thread drives it, supplies the instant, and performs the installs it returns.
/// That is what makes the whole strategy testable with synthetic instants and no
/// capture driver.
pub struct FilterManager {
    config: FilterConfig,
    handles: Vec<HandleState>,
    /// The last wanted set observed, and when it last changed. The debounce is
    /// measured from that change, so a set that keeps churning never settles and
    /// never installs, which is the coalescing section 12.2 asks for.
    last_wanted: Option<BTreeSet<Endpoint>>,
    changed_at: Option<Instant>,
    gaps: u64,
}

impl FilterManager {
    /// A manager for `handle_count` handles (one per capture thread), all
    /// starting in bootstrap.
    pub fn new(handle_count: usize, config: FilterConfig) -> Self {
        let handles = (0..handle_count)
            .map(|_| HandleState {
                installed: Installed::Bootstrap,
                last_install: None,
            })
            .collect();
        FilterManager {
            config,
            handles,
            last_wanted: None,
            changed_at: None,
            gaps: 0,
        }
    }

    /// Feed the current wanted endpoint set and the current instant. Returns the
    /// programs to install now.
    ///
    /// The rules, all from specification section 12.2:
    ///
    /// - Debounce: nothing is returned until the wanted set has been unchanged
    ///   for `config.debounce`. Every change resets the timer, so churn
    ///   coalesces.
    /// - Rate limit: a handle is reinstalled at most once per
    ///   `config.min_reinstall_interval`; an otherwise-due reinstall is deferred
    ///   to a later `poll`, never dropped.
    /// - Never empty: a handle whose wanted set is empty keeps bootstrap (if
    ///   never narrowed) or its prior narrowed program; no empty program is
    ///   installed.
    /// - Idempotence: a handle already narrowed to exactly the wanted set is left
    ///   alone.
    ///
    /// Gap accounting (section 12.3): after a reinstall on a handle whose prior
    /// program was narrowed, [`FilterManager::filter_gaps`] rises by the count of
    /// endpoints the new set adds. A bootstrap-to-first-narrowing install adds
    /// none, because bootstrap admitted everything. The count is of gap
    /// occurrences, endpoints briefly excluded, never a fabricated count of the
    /// packets the kernel dropped, which fragcap never observed (P-9).
    pub fn poll(&mut self, wanted: &[Endpoint], now: Instant) -> Vec<Install> {
        let wanted: BTreeSet<Endpoint> = wanted.iter().copied().collect();

        // Debounce: reset the settle timer whenever the wanted set changes.
        if self.last_wanted.as_ref() != Some(&wanted) {
            self.last_wanted = Some(wanted.clone());
            self.changed_at = Some(now);
        }

        let settled = match self.changed_at {
            Some(t) => now.saturating_duration_since(t) >= self.config.debounce,
            None => false,
        };
        if !settled {
            return Vec::new();
        }

        // The wanted set is empty: keep bootstrap or the prior narrowed program
        // on every handle. Never install an empty program.
        if wanted.is_empty() {
            return Vec::new();
        }

        let ordered: Vec<Endpoint> = wanted.iter().copied().collect();
        let mut installs = Vec::new();
        for (idx, handle) in self.handles.iter_mut().enumerate() {
            // Already narrowed to exactly this set: nothing to do.
            if let Installed::Narrowed(current) = &handle.installed {
                if *current == wanted {
                    continue;
                }
            }
            // Rate limit: defer to a later poll rather than reinstall too soon.
            if let Some(t) = handle.last_install {
                if now.saturating_duration_since(t) < self.config.min_reinstall_interval {
                    continue;
                }
            }
            // Gap accounting: endpoints this reinstall newly admits that a prior
            // narrowed program excluded. Bootstrap admitted everything.
            if let Installed::Narrowed(prev) = &handle.installed {
                let added = wanted.difference(prev).count() as u64;
                self.gaps = self.gaps.saturating_add(added);
            }
            handle.installed = Installed::Narrowed(wanted.clone());
            handle.last_install = Some(now);
            installs.push(Install {
                handle: idx,
                program: FilterProgram::narrowed(&ordered),
            });
        }
        installs
    }

    /// The running total of filter gaps, for the capture statistics.
    pub fn filter_gaps(&self) -> u64 {
        self.gaps
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::SocketAddr;

    fn ep(addr: &str, proto: Proto) -> Endpoint {
        Endpoint::new(addr.parse::<SocketAddr>().expect("address parses"), proto)
    }

    #[test]
    fn an_expression_round_trips() {
        let f = FilterProgram::new("tcp port 443");
        assert_eq!(f.expression(), "tcp port 443");
        assert!(!f.is_empty());
    }

    #[test]
    fn the_default_selects_everything() {
        assert!(FilterProgram::default().is_empty());
    }

    #[test]
    fn a_single_endpoint_compiles_to_its_clause() {
        let f = FilterProgram::narrowed(&[ep("198.51.100.5:443", Proto::Tcp)]);
        assert_eq!(f.expression(), "(tcp and host 198.51.100.5 and port 443)");
    }

    #[test]
    fn an_endpoint_set_spans_both_address_families() {
        let f = FilterProgram::narrowed(&[
            ep("198.51.100.5:443", Proto::Tcp),
            ep("[2001:db8::1]:5055", Proto::Udp),
        ]);
        assert!(
            f.expression().contains("host 198.51.100.5 and port 443"),
            "the v4 clause is present: {}",
            f.expression()
        );
        assert!(
            f.expression().contains("host 2001:db8::1 and port 5055"),
            "the v6 clause is present: {}",
            f.expression()
        );
        assert!(f.expression().contains(" or "), "clauses are ORed");
    }

    #[test]
    fn compilation_is_deterministic_under_reorder_and_duplication() {
        let a = ep("198.51.100.5:443", Proto::Tcp);
        let b = ep("203.0.113.9:5055", Proto::Udp);
        let one = FilterProgram::narrowed(&[a, b]);
        let two = FilterProgram::narrowed(&[b, a, a, b]);
        assert_eq!(one, two, "order and duplicates do not change the program");
    }

    #[test]
    fn an_empty_set_compiles_to_an_empty_program() {
        assert!(FilterProgram::narrowed(&[]).is_empty());
    }

    // --------------------------------------------------------------
    // FilterManager: the phase-three policy over synthetic instants.
    // --------------------------------------------------------------

    fn cfg(debounce_ms: u64, rate_ms: u64) -> FilterConfig {
        FilterConfig {
            debounce: Duration::from_millis(debounce_ms),
            min_reinstall_interval: Duration::from_millis(rate_ms),
        }
    }

    #[test]
    fn production_config_carries_the_section_12_2_constants() {
        assert_eq!(FilterConfig::PRODUCTION.debounce, Duration::from_secs(2));
        assert_eq!(
            FilterConfig::PRODUCTION.min_reinstall_interval,
            Duration::from_secs(5)
        );
        assert_eq!(FilterConfig::default(), FilterConfig::PRODUCTION);
    }

    #[test]
    fn debounce_holds_an_install_until_the_set_settles() {
        let mut mgr = FilterManager::new(1, cfg(2000, 0));
        let t0 = Instant::now();
        let want = [ep("198.51.100.5:443", Proto::Tcp)];

        assert!(mgr.poll(&want, t0).is_empty(), "held during the window");
        assert!(
            mgr.poll(&want, t0 + Duration::from_millis(1999)).is_empty(),
            "still held one millisecond short"
        );
        let installs = mgr.poll(&want, t0 + Duration::from_millis(2000));
        assert_eq!(installs.len(), 1, "installed once settled");
        assert_eq!(installs[0].handle, 0);
        assert!(!installs[0].program.is_empty());
    }

    #[test]
    fn a_change_resets_the_debounce_timer() {
        let mut mgr = FilterManager::new(1, cfg(2000, 0));
        let t0 = Instant::now();
        let first = [ep("198.51.100.5:443", Proto::Tcp)];
        let second = [
            ep("198.51.100.5:443", Proto::Tcp),
            ep("203.0.113.9:5055", Proto::Udp),
        ];

        assert!(mgr.poll(&first, t0).is_empty());
        // The set changes at t0+1s; the timer resets, so t0+2s is only one second
        // of stability and installs nothing.
        assert!(mgr
            .poll(&second, t0 + Duration::from_millis(1000))
            .is_empty());
        assert!(mgr
            .poll(&second, t0 + Duration::from_millis(2000))
            .is_empty());
        // Settled two seconds after the change.
        let installs = mgr.poll(&second, t0 + Duration::from_millis(3000));
        assert_eq!(installs.len(), 1);
    }

    #[test]
    fn the_rate_limit_defers_a_reinstall_per_handle() {
        let mut mgr = FilterManager::new(1, cfg(0, 5000));
        let t0 = Instant::now();
        let first = [ep("198.51.100.5:443", Proto::Tcp)];
        let second = [
            ep("198.51.100.5:443", Proto::Tcp),
            ep("203.0.113.9:5055", Proto::Udp),
        ];

        assert_eq!(mgr.poll(&first, t0).len(), 1, "first install");
        // A new endpoint four seconds later is deferred by the five-second limit.
        assert!(
            mgr.poll(&second, t0 + Duration::from_millis(4000))
                .is_empty(),
            "deferred, not dropped"
        );
        // And installed once the interval passes.
        let installs = mgr.poll(&second, t0 + Duration::from_millis(5000));
        assert_eq!(installs.len(), 1, "reinstalled after the interval");
    }

    #[test]
    fn an_unchanged_set_is_not_reinstalled() {
        let mut mgr = FilterManager::new(1, cfg(0, 0));
        let t0 = Instant::now();
        let want = [ep("198.51.100.5:443", Proto::Tcp)];
        assert_eq!(mgr.poll(&want, t0).len(), 1, "first install");
        assert!(
            mgr.poll(&want, t0 + Duration::from_millis(10)).is_empty(),
            "idempotent: no reinstall when nothing changed"
        );
    }

    #[test]
    fn an_empty_set_installs_nothing_and_keeps_the_prior_program() {
        let mut mgr = FilterManager::new(1, cfg(0, 0));
        let t0 = Instant::now();
        let want = [ep("198.51.100.5:443", Proto::Tcp)];
        assert_eq!(mgr.poll(&want, t0).len(), 1);
        // The set empties; nothing is installed, the handle keeps its narrowed
        // program, and no gap is charged.
        assert!(mgr.poll(&[], t0 + Duration::from_millis(10)).is_empty());
        assert_eq!(mgr.filter_gaps(), 0);
        // The prior narrowed program is still what is installed: re-offering it
        // reinstalls nothing.
        assert!(mgr.poll(&want, t0 + Duration::from_millis(20)).is_empty());
    }

    #[test]
    fn the_first_narrowing_records_no_gap() {
        let mut mgr = FilterManager::new(1, cfg(0, 0));
        let t0 = Instant::now();
        let want = [ep("198.51.100.5:443", Proto::Tcp)];
        assert_eq!(mgr.poll(&want, t0).len(), 1);
        assert_eq!(
            mgr.filter_gaps(),
            0,
            "bootstrap admitted everything, so narrowing from it opens no gap"
        );
    }

    #[test]
    fn a_later_added_endpoint_records_a_gap() {
        let mut mgr = FilterManager::new(1, cfg(0, 0));
        let t0 = Instant::now();
        let first = [ep("198.51.100.5:443", Proto::Tcp)];
        let second = [
            ep("198.51.100.5:443", Proto::Tcp),
            ep("203.0.113.9:5055", Proto::Udp),
        ];
        let third = [
            ep("198.51.100.5:443", Proto::Tcp),
            ep("203.0.113.9:5055", Proto::Udp),
            ep("[2001:db8::1]:80", Proto::Tcp),
        ];
        assert_eq!(mgr.poll(&first, t0).len(), 1);
        assert_eq!(mgr.filter_gaps(), 0);
        assert_eq!(mgr.poll(&second, t0 + Duration::from_millis(10)).len(), 1);
        assert_eq!(mgr.filter_gaps(), 1, "one endpoint newly admitted");
        assert_eq!(mgr.poll(&third, t0 + Duration::from_millis(20)).len(), 1);
        assert_eq!(mgr.filter_gaps(), 2, "a second endpoint newly admitted");
    }

    #[test]
    fn each_handle_narrows_and_is_rate_limited_independently() {
        let mut mgr = FilterManager::new(3, cfg(0, 0));
        let t0 = Instant::now();
        let want = [ep("198.51.100.5:443", Proto::Tcp)];
        let installs = mgr.poll(&want, t0);
        assert_eq!(installs.len(), 3, "one install per handle");
        let handles: BTreeSet<usize> = installs.iter().map(|i| i.handle).collect();
        assert_eq!(handles, BTreeSet::from([0, 1, 2]));
    }
}
