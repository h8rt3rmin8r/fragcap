// SPDX-License-Identifier: Apache-2.0

//! The Windows socket table and process enumeration backends.
//!
//! Present only when the `socket-table` feature is enabled and the target is
//! Windows. Absent otherwise, rather than stubbed into something that compiles
//! and reports fabricated contents: a stub that returns an empty table would
//! make every packet unattributed on a platform where attribution is simply not
//! implemented, and it would do it silently, which constitution P-9 forbids.
//!
//! # Why a binding crate rather than hand-written declarations
//!
//! The same argument slice S09 made for `pcap`. The alternative to a binding
//! here is not arithmetic over a byte slice, as it was in S03 and S06, but a C
//! ABI whose struct layouts must be transcribed by hand with nothing checking
//! them against the header. A wrong offset in `MIB_TCPROW_OWNER_MODULE` yields
//! a plausible process identifier that is wrong, which is the P-9 failure no
//! test over synthetic data catches.
//!
//! # Why this is not the `live` feature
//!
//! `fragcap-capture`'s `live` feature means "links against the npcap import
//! library". Nothing here does. The IP Helper API and the toolhelp snapshot
//! both ship with the operating system, so this backend builds and runs on a
//! bare Windows machine with no capture driver and no software development kit.
//! Folding the two features together would make attribution unavailable to
//! anyone without a driver they never call.
//!
//! # Constitution P-1
//!
//! Both facilities here are on the specification section 19.2 permitted list:
//! IP Helper socket tables, and query-only process enumeration. Neither opens a
//! handle against a target process. That is not merely compliant, it is the
//! reason the toolhelp path was chosen over `OpenProcess` plus
//! `QueryFullProcessImageNameW`: the latter would also comply, since
//! `PROCESS_QUERY_LIMITED_INFORMATION` carries no memory rights, but complying
//! is something a reviewer has to check, and having no handle at all is
//! something `cargo xtask lint` can assert.

mod iphelper;
mod toolhelp;

pub use iphelper::IpHelperTable;
pub use toolhelp::ToolhelpNamer;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::seam::{Clock, SystemClock};
    use crate::socket::{AttributorConfig, SocketTableAttributor};
    use fragcap_core::attribution::Fidelity;
    use fragcap_core::flow::{FlowKey, Proto};
    use fragcap_core::traits::FlowAttributor;
    use std::net::{SocketAddr, UdpSocket};
    use std::sync::Arc;

    /// The whole product claim, against a real machine: open a socket, and ask
    /// fragcap which process owns the flow on it.
    ///
    /// Tier 2 by specification section 25.2. Needs Windows and nothing else:
    /// no capture driver, no elevation, no game. This is the one test in the
    /// project that exercises attribution end to end against the operating
    /// system rather than against a table a test wrote down.
    #[test]
    #[ignore = "tier 2: opens a real socket and reads the machine's socket table"]
    fn a_real_socket_attributes_to_this_process() {
        // A real UDP socket, bound to a real ephemeral port on loopback.
        let socket = UdpSocket::bind("127.0.0.1:0").expect("binding a loopback socket");
        let local: SocketAddr = socket.local_addr().expect("the socket reports its address");

        let mut attributor = SocketTableAttributor::new(
            Box::new(IpHelperTable::new()),
            Box::new(ToolhelpNamer::new()),
            Arc::new(SystemClock) as Arc<dyn Clock>,
            AttributorConfig::default(),
        );
        attributor
            .refresh()
            .expect("the machine's socket table reads");

        // A flow on that socket, as the header parser would key it. The peer is
        // arbitrary: specification section 8.4 keys UDP on the local endpoint
        // alone, because the table carries no remote for a datagram socket.
        let key = FlowKey::new(
            Proto::Udp,
            local,
            "198.51.100.5:5055".parse().expect("a test peer"),
        );

        let a = attributor
            .resolve(&key, SystemClock.now())
            .expect("the socket this test just opened is in the table");

        assert_eq!(
            a.pid,
            std::process::id(),
            "the flow belongs to the process that opened it"
        );
        assert!(
            a.process.to_ascii_lowercase().contains("fragcap"),
            "the image name came from enumeration, got {:?}",
            a.process
        );
        assert_eq!(
            a.fidelity,
            Fidelity::Live,
            "the endpoint was in the table, so the answer is observed rather than inferred"
        );
        assert!(a.role.is_none(), "roles arrive with S12");

        // And the endpoint appears in the active set.
        assert!(
            attributor
                .active_endpoints()
                .iter()
                .any(|e| e.addr == local && e.proto == Proto::Udp),
            "the socket is reported active"
        );
    }

    /// Retention, against the real table: close the socket, refresh, and the
    /// answer should survive as inferred rather than vanishing.
    ///
    /// This is the section 11.4 behavior that keeps the tail of a connection
    /// attributed, demonstrated rather than asserted.
    #[test]
    #[ignore = "tier 2: opens a real socket and reads the machine's socket table"]
    fn a_closed_socket_stays_resolvable_and_is_marked_retained() {
        let socket = UdpSocket::bind("127.0.0.1:0").expect("binding a loopback socket");
        let local: SocketAddr = socket.local_addr().expect("the socket reports its address");

        let mut attributor = SocketTableAttributor::new(
            Box::new(IpHelperTable::new()),
            Box::new(ToolhelpNamer::new()),
            Arc::new(SystemClock) as Arc<dyn Clock>,
            AttributorConfig::default(),
        );
        attributor.refresh().expect("the first read");

        let key = FlowKey::new(
            Proto::Udp,
            local,
            "198.51.100.5:5055".parse().expect("a test peer"),
        );
        assert_eq!(
            attributor
                .resolve(&key, SystemClock.now())
                .expect("live")
                .fidelity,
            Fidelity::Live
        );

        drop(socket);
        attributor.refresh().expect("the second read");

        let a = attributor
            .resolve(&key, SystemClock.now())
            .expect("still inside the thirty second grace period");
        assert_eq!(
            a.fidelity,
            Fidelity::Retained,
            "the socket has gone; the answer is inference and says so"
        );
        assert_eq!(a.pid, std::process::id());
    }
}
