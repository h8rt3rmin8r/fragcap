// SPDX-License-Identifier: Apache-2.0

//! When to read the socket table again. Specification section 11.2.
//!
//! Three rules, and the interesting part is that two of them are recorded on a
//! thread that cannot act on them.
//!
//! The default interval is one second, set by measurement rather than caution:
//! Appendix D found a snapshot costs one to three milliseconds against roughly
//! 1800 sockets through the table interface, so the cadence is bounded by what
//! is useful rather than by what is affordable.
//!
//! A process start matching a profile stage triggers an immediate refresh,
//! because a newly matched process is about to open sockets.
//!
//! An unattributed packet on a previously unseen endpoint triggers one too,
//! rate limited to one per two hundred milliseconds. That one arrives on the
//! acquisition path, where [`fragcap_core::traits::FlowAttributor::resolve`]
//! holds only `&self` and where reading a table is exactly what section 11.6
//! forbids. So it records a request rather than performing a refresh, through
//! atomics, and whoever drives the cadence acts on it.

use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};
use std::time::Duration;

use fragcap_core::packet::Timestamp;

/// A sentinel meaning "never", distinguishable from any real instant.
///
/// `i64::MIN` rather than zero, because zero is the Unix epoch and is a
/// perfectly good instant for a test to use.
const NEVER: i64 = i64::MIN;

/// The refresh cadence and its two triggers.
///
/// `Send + Sync` and shared behind an `Arc`, because the unseen-endpoint
/// trigger is recorded on a capture thread and read by whoever owns the
/// attributor.
#[derive(Debug)]
pub struct RefreshSchedule {
    last_refresh: AtomicI64,
    last_request: AtomicI64,
    requested: AtomicBool,
}

impl Default for RefreshSchedule {
    fn default() -> Self {
        RefreshSchedule {
            last_refresh: AtomicI64::new(NEVER),
            last_request: AtomicI64::new(NEVER),
            requested: AtomicBool::new(false),
        }
    }
}

impl RefreshSchedule {
    pub fn new() -> Self {
        RefreshSchedule::default()
    }

    /// Whether the interval has elapsed since the last refresh.
    ///
    /// True before any refresh has happened, because a schedule that has never
    /// read the table is as due as one can be.
    pub fn is_due(&self, now: Timestamp, interval: Duration) -> bool {
        let last = self.last_refresh.load(Ordering::SeqCst);
        if last == NEVER {
            return true;
        }
        elapsed_at_least(last, now, interval)
    }

    /// Record that a refresh completed at `now`. Clears any pending request,
    /// because the refresh it was asking for has happened.
    pub fn mark_refreshed(&self, now: Timestamp) {
        self.last_refresh.store(now.as_nanos(), Ordering::SeqCst);
        self.requested.store(false, Ordering::SeqCst);
    }

    /// Ask for a refresh because a packet arrived on an endpoint the current
    /// snapshot does not carry. Rate limited.
    ///
    /// Returns whether the request was recorded. Reporting that rather than
    /// silently dropping it is what makes the rate limit observable in a test
    /// without reading private state, and it is the only way a caller can tell
    /// a refused request from an accepted one.
    ///
    /// The limit bounds how often fragcap reads the platform's table, which is
    /// why it is measured against `now` and not against the packet's instant: a
    /// replay of an hour of traffic in one second would otherwise request
    /// thousands of refreshes, and a quiet interface would request none.
    pub fn request_triggered(&self, now: Timestamp, limit: Duration) -> bool {
        let last = self.last_request.load(Ordering::SeqCst);
        if last != NEVER && !elapsed_at_least(last, now, limit) {
            return false;
        }
        self.last_request.store(now.as_nanos(), Ordering::SeqCst);
        self.requested.store(true, Ordering::SeqCst);
        true
    }

    /// Ask for a refresh because a process matching a profile stage started.
    ///
    /// Not rate limited. Specification section 11.2 rate limits the
    /// unattributed-packet trigger and not this one, and the reason is in the
    /// asymmetry between them: an unattributed packet can arrive thousands of
    /// times a second from traffic fragcap will never attribute, while a
    /// matched process start is a rare event that is always followed by sockets
    /// worth catching.
    pub fn request_immediate(&self) {
        self.requested.store(true, Ordering::SeqCst);
    }

    /// Whether a refresh has been requested, clearing the request.
    pub fn take_request(&self) -> bool {
        self.requested.swap(false, Ordering::SeqCst)
    }

    /// Whether a refresh has been requested, without clearing it.
    pub fn is_requested(&self) -> bool {
        self.requested.load(Ordering::SeqCst)
    }
}

/// Whether at least `d` has elapsed from `from` to `now`, in nanoseconds.
///
/// Saturating, and false for a `now` before `from`. Time going backwards is not
/// a reason to read the table.
fn elapsed_at_least(from: i64, now: Timestamp, d: Duration) -> bool {
    let elapsed = now.as_nanos().saturating_sub(from);
    if elapsed < 0 {
        return false;
    }
    let want = i64::try_from(d.as_nanos()).unwrap_or(i64::MAX);
    elapsed >= want
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(n: i64) -> Timestamp {
        Timestamp::from_nanos(n)
    }

    const SECOND: Duration = Duration::from_secs(1);
    const LIMIT: Duration = Duration::from_millis(200);

    // FR-011, FR-012. The interval, against declared instants. Nothing here
    // sleeps, which is the whole reason the clock is injected.
    #[test]
    fn a_schedule_that_has_never_refreshed_is_due() {
        let s = RefreshSchedule::new();
        assert!(s.is_due(at(0), SECOND));
    }

    #[test]
    fn the_interval_governs_when_the_next_refresh_is_due() {
        let s = RefreshSchedule::new();
        s.mark_refreshed(at(1_000_000_000));
        assert!(!s.is_due(at(1_500_000_000), SECOND), "half a second later");
        assert!(
            s.is_due(at(2_000_000_000), SECOND),
            "exactly one second later"
        );
        assert!(s.is_due(at(9_000_000_000), SECOND));
    }

    #[test]
    fn time_going_backwards_does_not_make_a_refresh_due() {
        let s = RefreshSchedule::new();
        s.mark_refreshed(at(5_000_000_000));
        assert!(!s.is_due(at(1_000_000_000), SECOND));
    }

    // FR-014, FR-015. The trigger and its rate limit. SC-005.
    #[test]
    fn the_unseen_endpoint_trigger_is_rate_limited() {
        let s = RefreshSchedule::new();
        assert!(s.request_triggered(at(0), LIMIT), "the first is accepted");
        assert!(
            !s.request_triggered(at(100_000_000), LIMIT),
            "100 ms later, inside the limit"
        );
        assert!(
            !s.request_triggered(at(199_999_999), LIMIT),
            "just inside the limit"
        );
        assert!(
            s.request_triggered(at(200_000_000), LIMIT),
            "exactly 200 ms later"
        );
        assert!(
            s.request_triggered(at(400_000_000), LIMIT),
            "and again after another 200 ms"
        );
    }

    #[test]
    fn a_burst_of_unattributable_traffic_records_one_request() {
        // The property the limit exists for. Ten thousand unattributed packets
        // inside one interval must not become ten thousand table reads.
        let s = RefreshSchedule::new();
        let mut accepted = 0;
        for i in 0..10_000i64 {
            if s.request_triggered(at(i * 10_000), LIMIT) {
                accepted += 1;
            }
        }
        // 10,000 packets spread over 100 ms of declared time.
        assert_eq!(accepted, 1, "one request per 200 ms, no matter the volume");
    }

    // FR-013, FR-016.
    #[test]
    fn a_matched_process_start_ignores_the_rate_limit() {
        let s = RefreshSchedule::new();
        assert!(s.request_triggered(at(0), LIMIT));
        assert!(s.take_request());

        // Inside the limit, so a triggered request would be refused.
        assert!(!s.request_triggered(at(50_000_000), LIMIT));
        assert!(!s.is_requested());

        s.request_immediate();
        assert!(
            s.take_request(),
            "a matched process start is not rate limited"
        );
    }

    #[test]
    fn taking_a_request_clears_it() {
        let s = RefreshSchedule::new();
        s.request_immediate();
        assert!(s.take_request());
        assert!(!s.take_request(), "a request is taken once");
    }

    // Checklist CHK024. A trigger arriving just before the interval elapses
    // must not produce two reads.
    #[test]
    fn refreshing_clears_a_pending_request() {
        let s = RefreshSchedule::new();
        s.mark_refreshed(at(0));

        // A trigger just before the interval is up.
        assert!(s.request_triggered(at(990_000_000), LIMIT));
        assert!(s.is_requested());

        // Whoever drives the cadence refreshes, for whichever reason.
        s.mark_refreshed(at(1_000_000_000));
        assert!(
            !s.take_request(),
            "the refresh satisfied the request; a second read would be for nothing"
        );
        assert!(!s.is_due(at(1_000_000_000), SECOND));
    }

    #[test]
    fn a_schedule_is_shareable_across_threads() {
        use std::sync::Arc;
        use std::thread;

        let s = Arc::new(RefreshSchedule::new());
        let handles: Vec<_> = (0..4)
            .map(|_| {
                let s = Arc::clone(&s);
                thread::spawn(move || {
                    for i in 0..1_000i64 {
                        s.request_triggered(at(i), LIMIT);
                    }
                })
            })
            .collect();
        for h in handles {
            h.join().expect("a recorder finishes");
        }
        // The only claim: recording from several threads is sound and the
        // schedule survives it. Which requests won is deliberately not asserted,
        // because it depends on interleaving and asserting it would be a flaky
        // test dressed as a strict one.
        assert!(s.is_requested());
    }
}
