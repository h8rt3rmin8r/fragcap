// SPDX-License-Identifier: Apache-2.0

//! The bounded buffer of specification section 12.4.
//!
//! Fixed capacity, drop-oldest eviction, a producer that never waits on the
//! consumer, and a consumer that waits without spinning. Section 12.4 states
//! why the producer must not block: "blocking a capture thread stalls the
//! kernel buffer behind it and converts a fragcap drop into a kernel drop,
//! which is both less visible and less controllable."
//!
//! The property this claims is that the producer never waits for the consumer
//! to make progress, not that the producer never blocks. The second is false of
//! any shared structure and would be a claim the code could not honor. The
//! producer does take a lock, and the critical section it waits for is a push
//! and at most one pop, held by a consumer that is itself never waiting on
//! anything outside this module. Sink slowness is therefore not expressible as
//! producer latency, which is what section 12.4 actually requires.
//!
//! Two details are load-bearing and easy to get wrong.
//!
//! **The eviction count lives here, not in the producer.** A producer that
//! unwinds takes its stack frame with it, and the count of packets that went
//! missing is exactly what an operator needs in that case. Slice S08 research
//! decision R-5.
//!
//! **The terminal item is exempt from the capacity bound.** Evicting an
//! observed packet to make room for fragcap's own bookkeeping would be loss
//! caused by the tool's shutdown rather than by a slow sink, and constitution
//! P-4 would then require counting it under a name that means something else.
//! The queue holds at most capacity plus one, and the extra is never a packet.
//! Research decision R-6.
//!
//! Crate-private. A bounded queue with eviction is not a seam the project
//! promises to anyone; [`super::Pipeline`] is.

use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex};

use crate::packet::CapturedPacket;
use crate::stats::CaptureStats;

/// What crosses the buffer.
#[derive(Debug)]
pub(crate) enum Item {
    /// One observation, parsed and attributed.
    Packet(Box<CapturedPacket>),
    /// The acquisition side finished, carrying its final counters. Sent exactly
    /// once, and exempt from the capacity bound.
    End(Box<CaptureStats>),
}

/// The state both halves share, guarded by one mutex.
#[derive(Debug)]
struct Shared {
    queue: VecDeque<Item>,
    capacity: usize,
    /// Advanced once per eviction, under the lock that performs it. Saturating,
    /// because a count that wrapped would understate loss.
    evicted: u64,
    /// False once the producer is gone, for any reason including unwinding.
    open: bool,
}

/// A buffer's two halves, created together.
///
/// # Panics
///
/// If `capacity` is zero. [`super::Pipeline::new`] rejects that with a named
/// error and is the only caller today, but this function is reachable from
/// anywhere in the crate and a zero capacity is not merely useless here: every
/// push would find the queue full, pop nothing, and still advance the eviction
/// count, so the buffer would grow without bound while reporting losses that
/// did not happen. A counter that lies is the one failure this module exists to
/// prevent, so the precondition is enforced rather than documented.
pub(crate) fn channel(capacity: usize) -> (Producer, Consumer) {
    assert!(capacity > 0, "a bounded buffer needs a non-zero capacity");
    let shared = Arc::new((
        Mutex::new(Shared {
            queue: VecDeque::new(),
            capacity,
            evicted: 0,
            open: true,
        }),
        Condvar::new(),
    ));
    (
        Producer {
            shared: Arc::clone(&shared),
        },
        Consumer { shared },
    )
}

type Channel = Arc<(Mutex<Shared>, Condvar)>;

/// The acquisition side's handle. Not `Clone`: there is one producer, so
/// closing the buffer when it drops is unambiguous.
#[derive(Debug)]
pub(crate) struct Producer {
    shared: Channel,
}

impl Producer {
    /// Admit an item, evicting the oldest packet first if the buffer is full.
    ///
    /// Never waits for the consumer to remove anything.
    pub(crate) fn push(&self, item: Item) {
        let (lock, cvar) = &*self.shared;
        let mut shared = lock.lock().expect("the buffer mutex is never poisoned");
        // A terminal item is admitted regardless of length. See the module
        // documentation and research decision R-6.
        if matches!(item, Item::Packet(_)) && shared.queue.len() >= shared.capacity {
            shared.queue.pop_front();
            shared.evicted = shared.evicted.saturating_add(1);
        }
        shared.queue.push_back(item);
        drop(shared);
        cvar.notify_one();
    }
}

impl Drop for Producer {
    /// Close the buffer and wake the consumer.
    ///
    /// This runs during an unwinding panic as well as on an ordinary return,
    /// which is the whole point: the consumer observes an ending however the
    /// producer terminated, and cannot wait forever for a terminal item that
    /// will never arrive.
    fn drop(&mut self) {
        let (lock, cvar) = &*self.shared;
        if let Ok(mut shared) = lock.lock() {
            shared.open = false;
        }
        cvar.notify_all();
    }
}

/// The output side's handle.
#[derive(Debug)]
pub(crate) struct Consumer {
    shared: Channel,
}

impl Consumer {
    /// The next item, waiting while the buffer is empty and still open.
    ///
    /// `None` means the queue is empty and the producer is gone, which is the
    /// only ending.
    pub(crate) fn next(&self) -> Option<Item> {
        let (lock, cvar) = &*self.shared;
        let mut shared = lock.lock().expect("the buffer mutex is never poisoned");
        loop {
            if let Some(item) = shared.queue.pop_front() {
                return Some(item);
            }
            if !shared.open {
                return None;
            }
            shared = cvar
                .wait(shared)
                .expect("the buffer mutex is never poisoned");
        }
    }

    /// How many packets the buffer evicted to admit newer ones.
    ///
    /// Readable after the producer is gone, which is why it lives in the shared
    /// state rather than in the producer's stack frame.
    pub(crate) fn evicted(&self) -> u64 {
        let (lock, _) = &*self.shared;
        lock.lock()
            .expect("the buffer mutex is never poisoned")
            .evicted
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interface::InterfaceId;
    use crate::packet::{Payload, RawPacket, Timestamp};

    fn packet(n: u8) -> Item {
        Item::Packet(Box::new(CapturedPacket::from_raw(
            RawPacket::new(
                Timestamp::from_nanos(n as i64),
                Payload::copy_from_slice(&[n]),
                1,
            ),
            InterfaceId::default(),
        )))
    }

    /// The marker byte a test packet was built with, so a sequence is
    /// identifiable after crossing the buffer.
    fn marker(item: &Item) -> u8 {
        match item {
            Item::Packet(p) => p.data[0],
            Item::End(_) => panic!("expected a packet, found the terminal item"),
        }
    }

    fn drain(c: &Consumer) -> Vec<u8> {
        let mut out = Vec::new();
        while let Some(item) = c.next() {
            match item {
                Item::Packet(p) => out.push(p.data[0]),
                Item::End(_) => break,
            }
        }
        out
    }

    // A zero capacity would make every push find the queue full, pop nothing,
    // and still advance the eviction count. `Pipeline::new` rejects it with a
    // named error; this is the crate-internal backstop for a caller that
    // bypassed it.
    #[test]
    #[should_panic(expected = "non-zero capacity")]
    fn a_zero_capacity_buffer_cannot_be_created() {
        let _ = channel(0);
    }

    // T009. FR-014.
    #[test]
    fn a_buffer_under_capacity_delivers_everything_in_push_order() {
        let (tx, rx) = channel(8);
        for n in 0..5 {
            tx.push(packet(n));
        }
        drop(tx);
        assert_eq!(drain(&rx), vec![0, 1, 2, 3, 4]);
        assert_eq!(rx.evicted(), 0, "nothing was full, so nothing was evicted");
    }

    // T010. FR-012, FR-014, FR-016. The three properties together: the oldest
    // goes, the survivors keep their order, and the count matches.
    #[test]
    fn pushing_beyond_capacity_evicts_the_oldest_and_counts_each_one() {
        let (tx, rx) = channel(3);
        for n in 0..10 {
            tx.push(packet(n));
        }
        drop(tx);
        assert_eq!(
            drain(&rx),
            vec![7, 8, 9],
            "the three newest survive, in order"
        );
        assert_eq!(rx.evicted(), 7);
    }

    // T011. The degenerate but legitimate setting.
    #[test]
    fn a_capacity_of_one_evicts_on_every_push_after_the_first() {
        let (tx, rx) = channel(1);
        tx.push(packet(1));
        tx.push(packet(2));
        tx.push(packet(3));
        drop(tx);
        assert_eq!(drain(&rx), vec![3]);
        assert_eq!(rx.evicted(), 2);
    }

    // T012. FR-030 and research R-6. The terminal item never costs a packet.
    #[test]
    fn the_terminal_item_is_admitted_without_evicting_a_packet() {
        let (tx, rx) = channel(2);
        tx.push(packet(1));
        tx.push(packet(2));
        assert_eq!(rx.evicted(), 0);

        tx.push(Item::End(Box::default()));
        assert_eq!(
            rx.evicted(),
            0,
            "admitting the terminal item must not discard an observation"
        );

        // Both packets are still here, followed by the terminal item.
        assert_eq!(marker(&rx.next().expect("first packet")), 1);
        assert_eq!(marker(&rx.next().expect("second packet")), 2);
        assert!(matches!(rx.next(), Some(Item::End(_))));
    }

    // T013. FR-030. A drained tail survives the producer going away.
    #[test]
    fn items_already_queued_survive_the_producer_being_dropped() {
        let (tx, rx) = channel(8);
        for n in 0..4 {
            tx.push(packet(n));
        }
        drop(tx);
        assert_eq!(drain(&rx), vec![0, 1, 2, 3]);
        assert!(rx.next().is_none(), "and only then does the buffer end");
    }

    // T014. Research R-5. The count an operator needs is exactly the one a
    // producer-side counter would have lost.
    #[test]
    fn the_eviction_count_outlives_the_producer() {
        let (tx, rx) = channel(2);
        for n in 0..6 {
            tx.push(packet(n));
        }
        drop(tx);
        assert_eq!(rx.evicted(), 4, "readable with no producer left to ask");
        assert_eq!(drain(&rx), vec![4, 5]);
        assert_eq!(rx.evicted(), 4, "and draining does not change it");
    }

    // FR-013, observably. A full buffer accepts a push and returns; if the
    // producer waited for the consumer, this would never finish, because there
    // is no consumer running.
    #[test]
    fn a_push_into_a_full_buffer_returns_with_no_consumer_present() {
        let (tx, rx) = channel(1);
        tx.push(packet(1));
        tx.push(packet(2));
        tx.push(packet(3));
        drop(tx);
        assert_eq!(drain(&rx), vec![3]);
    }

    // The ending the consumer sees when the producer never sent a terminal
    // item, which is what an unwinding acquisition side leaves behind.
    #[test]
    fn a_producer_that_sent_no_terminal_item_still_ends_the_buffer() {
        let (tx, rx) = channel(4);
        tx.push(packet(1));
        drop(tx);
        assert!(matches!(rx.next(), Some(Item::Packet(_))));
        assert!(
            rx.next().is_none(),
            "the consumer must not wait for a terminal item that is not coming"
        );
    }

    // The consumer blocks rather than spinning, and is woken by a push from
    // another thread. FR-015.
    #[test]
    fn the_consumer_waits_for_a_push_from_another_thread() {
        let (tx, rx) = channel(4);
        let writer = std::thread::spawn(move || {
            tx.push(packet(42));
            tx.push(Item::End(Box::default()));
        });
        // Blocks here until the spawned thread pushes.
        assert_eq!(marker(&rx.next().expect("the pushed packet arrives")), 42);
        assert!(matches!(rx.next(), Some(Item::End(_))));
        writer.join().expect("the writer thread finishes");
    }
}
