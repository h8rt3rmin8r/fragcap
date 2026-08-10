// SPDX-License-Identifier: Apache-2.0

//! The socket table, through the IP Helper API.
//!
//! Four tables are read per snapshot: TCP and UDP, each over IPv4 and IPv6.
//! All four are requested by *owning module* rather than by owning process
//! identifier, and that choice is the interesting one.
//!
//! The owning-process classes are the common example in the platform's own
//! documentation, and they are what the reconnaissance session of Appendix D
//! read. Their rows carry no creation instant. The owning-module classes carry
//! `liCreateTimestamp` on both protocols, which is what requirement FR-009 is
//! built on: a socket created after a packet cannot have owned that packet, and
//! rejecting it is the only mechanism available that distinguishes the previous
//! owner of a reused port from the current one.
//!
//! Appendix D records the creation timestamp as a property of the TCP table
//! alone. It is on both, and this slice records the correction. It matters more
//! for UDP than for TCP: specification section 8.4 keys UDP attribution on the
//! local endpoint alone, because the table reports no remote for a datagram
//! socket, which makes it the weaker join and the one where a reused port is
//! least distinguishable.
//!
//! The module information itself is ignored. Resolving it needs
//! `GetOwnerModuleFromTcpEntry`, a separate call per row, and image names come
//! more cheaply from [`super::ToolhelpNamer`].
//!
//! # Cost
//!
//! An owning-module row is roughly 150 bytes larger than an owning-process one,
//! because `OwningModuleInfo` is sixteen `u64` values. Against the roughly 1800
//! sockets Appendix D measured, that is about 270 kilobytes of extra copying
//! per snapshot, once per second, against a measured budget of one to three
//! milliseconds. It is not a consideration.

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};

use fragcap_core::error::AttrError;
use fragcap_core::packet::Timestamp;

use windows_sys::Win32::NetworkManagement::IpHelper::{
    GetExtendedTcpTable, GetExtendedUdpTable, MIB_TCP6ROW_OWNER_MODULE, MIB_TCPROW_OWNER_MODULE,
    MIB_UDP6ROW_OWNER_MODULE, MIB_UDPROW_OWNER_MODULE, TCP_TABLE_OWNER_MODULE_ALL,
    UDP_TABLE_OWNER_MODULE,
};
use windows_sys::Win32::Networking::WinSock::{AF_INET, AF_INET6};

use crate::seam::{Clock, SocketTableSource, SystemClock};
use crate::table::{SocketTable, SocketTableEntry};

/// `ERROR_INSUFFICIENT_BUFFER`. The table grew between the sizing call and the
/// reading call, which is ordinary on a busy machine.
const ERROR_INSUFFICIENT_BUFFER: u32 = 122;
const NO_ERROR: u32 = 0;

/// How many times a size negotiation is retried before giving up.
///
/// Bounded rather than looping until success. A machine opening sockets faster
/// than fragcap can size a buffer is a machine where this read should fail and
/// be reported, not one where a capture thread spins.
const SIZE_RETRIES: usize = 8;

/// Reads the operating system socket table.
///
/// Holds no handle. Each read is a self-contained pair of calls into the IP
/// Helper API, which owns whatever kernel state is involved.
#[derive(Clone, Debug, Default)]
pub struct IpHelperTable {
    /// Reused between reads so a steady-state snapshot does not reallocate.
    buffer: Vec<u8>,
}

impl IpHelperTable {
    pub fn new() -> Self {
        IpHelperTable::default()
    }

    /// Fill `self.buffer` with one table, negotiating its size.
    ///
    /// The platform's contract: call with a size, receive the required size and
    /// `ERROR_INSUFFICIENT_BUFFER` if it was too small, call again. The table
    /// can grow in between, hence the retry.
    ///
    /// # Safety
    ///
    /// `GetExtendedTcpTable` and `GetExtendedUdpTable` write at most
    /// `*size` bytes into the pointer they are given. The buffer is resized to
    /// `size` before the pointer is taken, and the returned length is only
    /// trusted after the call reports `NO_ERROR`.
    fn fill(&mut self, tcp: bool, family: u32) -> Result<(), AttrError> {
        let mut size: u32 = self.buffer.len().max(4096) as u32;
        for _ in 0..SIZE_RETRIES {
            self.buffer.resize(size as usize, 0);
            let rc = unsafe {
                if tcp {
                    GetExtendedTcpTable(
                        self.buffer.as_mut_ptr().cast(),
                        &mut size,
                        0,
                        family,
                        TCP_TABLE_OWNER_MODULE_ALL,
                        0,
                    )
                } else {
                    GetExtendedUdpTable(
                        self.buffer.as_mut_ptr().cast(),
                        &mut size,
                        0,
                        family,
                        UDP_TABLE_OWNER_MODULE,
                        0,
                    )
                }
            };
            match rc {
                NO_ERROR => {
                    self.buffer.truncate(size as usize);
                    return Ok(());
                }
                ERROR_INSUFFICIENT_BUFFER => continue,
                other => {
                    return Err(AttrError::RefreshFailed {
                        detail: format!(
                            "reading the {} table for address family {family} returned {other}",
                            if tcp { "TCP" } else { "UDP" }
                        ),
                    })
                }
            }
        }
        Err(AttrError::RefreshFailed {
            detail: format!(
                "the {} table for address family {family} kept growing across {SIZE_RETRIES} attempts",
                if tcp { "TCP" } else { "UDP" }
            ),
        })
    }

    /// The row count and a pointer to the first row, from a filled buffer.
    ///
    /// Every one of these tables is `{ dwNumEntries: u32, table: [Row; 1] }`,
    /// so the rows begin at the first offset satisfying the row's alignment.
    ///
    /// # Safety
    ///
    /// The caller must have filled `self.buffer` with a table of rows of type
    /// `R` through [`Self::fill`]. The count is clamped to what the buffer can
    /// actually hold, so a table reporting more rows than it delivered yields
    /// the rows present rather than reading past the end.
    unsafe fn rows<R: Copy>(&self) -> &[R] {
        if self.buffer.len() < std::mem::size_of::<u32>() {
            return &[];
        }
        let count = u32::from_ne_bytes([
            self.buffer[0],
            self.buffer[1],
            self.buffer[2],
            self.buffer[3],
        ]) as usize;
        let offset = std::mem::align_of::<R>().max(std::mem::size_of::<u32>());
        let available = self
            .buffer
            .len()
            .saturating_sub(offset)
            .checked_div(std::mem::size_of::<R>())
            .unwrap_or(0);
        let n = count.min(available);
        if n == 0 {
            return &[];
        }
        std::slice::from_raw_parts(self.buffer.as_ptr().add(offset).cast::<R>(), n)
    }
}

/// A `FILETIME`, as a count of 100 nanosecond intervals since 1601-01-01, as
/// the project's [`Timestamp`], which counts nanoseconds since 1970-01-01.
///
/// The epoch difference is 11,644,473,600 seconds. Getting this wrong produces
/// plausible instants that are wrong by 369 years, which is the P-9 failure no
/// test over synthetic data catches, so it is pinned by a test below.
///
/// A non-positive value means the platform reported no creation time, which is
/// distinct from a socket created at the epoch and is reported as such.
fn filetime_to_timestamp(ft: i64) -> Option<Timestamp> {
    if ft <= 0 {
        return None;
    }
    const EPOCH_DIFFERENCE_SECS: i64 = 11_644_473_600;
    let unix_100ns = ft.checked_sub(EPOCH_DIFFERENCE_SECS.checked_mul(10_000_000)?)?;
    Some(Timestamp::from_nanos(unix_100ns.checked_mul(100)?))
}

/// The platform reports ports in network byte order in the low half of a `u32`.
fn port_of(raw: u32) -> u16 {
    u16::from_be((raw & 0xffff) as u16)
}

fn v4(addr: u32, port: u32) -> SocketAddr {
    SocketAddr::new(
        IpAddr::V4(Ipv4Addr::from(u32::from_be(addr))),
        port_of(port),
    )
}

fn v6(addr: [u8; 16], port: u32) -> SocketAddr {
    SocketAddr::new(IpAddr::V6(Ipv6Addr::from(addr)), port_of(port))
}

impl SocketTableSource for IpHelperTable {
    fn read(&mut self) -> Result<SocketTable, AttrError> {
        // One instant for the whole snapshot. The four tables are read in
        // sequence and a few milliseconds apart, and stamping each with its own
        // read time would imply a precision the snapshot does not have.
        let taken_at = SystemClock.now();
        let mut entries = Vec::new();

        self.fill(true, AF_INET)?;
        for row in unsafe { self.rows::<MIB_TCPROW_OWNER_MODULE>() } {
            let mut e = SocketTableEntry::tcp(
                v4(row.dwLocalAddr, row.dwLocalPort),
                v4(row.dwRemoteAddr, row.dwRemotePort),
                row.dwOwningPid,
            );
            e.created = filetime_to_timestamp(row.liCreateTimestamp);
            entries.push(e);
        }

        self.fill(true, AF_INET6)?;
        for row in unsafe { self.rows::<MIB_TCP6ROW_OWNER_MODULE>() } {
            let mut e = SocketTableEntry::tcp(
                v6(row.ucLocalAddr, row.dwLocalPort),
                v6(row.ucRemoteAddr, row.dwRemotePort),
                row.dwOwningPid,
            );
            e.created = filetime_to_timestamp(row.liCreateTimestamp);
            entries.push(e);
        }

        self.fill(false, AF_INET)?;
        for row in unsafe { self.rows::<MIB_UDPROW_OWNER_MODULE>() } {
            // No remote. Specification section 8.4 forbids inventing one, and
            // `SocketTableEntry::udp` offers no way to.
            let mut e =
                SocketTableEntry::udp(v4(row.dwLocalAddr, row.dwLocalPort), row.dwOwningPid);
            e.created = filetime_to_timestamp(row.liCreateTimestamp);
            entries.push(e);
        }

        self.fill(false, AF_INET6)?;
        for row in unsafe { self.rows::<MIB_UDP6ROW_OWNER_MODULE>() } {
            let mut e =
                SocketTableEntry::udp(v6(row.ucLocalAddr, row.dwLocalPort), row.dwOwningPid);
            e.created = filetime_to_timestamp(row.liCreateTimestamp);
            entries.push(e);
        }

        Ok(SocketTable::new(taken_at, entries))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // The conversion that no synthetic test of the attributor could catch. A
    // wrong epoch here yields instants that are plausible and wrong by 369
    // years, and every downstream test would still pass.
    #[test]
    fn a_filetime_converts_against_the_right_epoch() {
        // The Unix epoch itself, as a FILETIME.
        let unix_epoch_as_filetime = 11_644_473_600i64 * 10_000_000;
        assert_eq!(
            filetime_to_timestamp(unix_epoch_as_filetime),
            Some(Timestamp::from_nanos(0))
        );

        // One second later.
        assert_eq!(
            filetime_to_timestamp(unix_epoch_as_filetime + 10_000_000),
            Some(Timestamp::from_nanos(1_000_000_000))
        );

        // A known instant: 2026-08-09T00:00:00Z is 1786320000 seconds after the
        // Unix epoch.
        let known_unix_secs = 1_786_320_000i64;
        let as_filetime = (known_unix_secs + 11_644_473_600) * 10_000_000;
        assert_eq!(
            filetime_to_timestamp(as_filetime),
            Some(Timestamp::from_nanos(known_unix_secs * 1_000_000_000))
        );
    }

    #[test]
    fn a_missing_filetime_is_reported_as_absent_rather_than_as_the_epoch() {
        assert_eq!(filetime_to_timestamp(0), None);
        assert_eq!(filetime_to_timestamp(-1), None);
    }

    #[test]
    fn ports_come_out_of_network_byte_order() {
        // 443, big endian, in the low half of the u32 the platform supplies.
        assert_eq!(port_of(0x0000_bb01), 443);
        assert_eq!(port_of(0x0000_0000), 0);
    }

    #[test]
    fn addresses_come_out_of_network_byte_order() {
        // 192.0.2.10, as the platform's little-endian u32.
        let raw = u32::from_be_bytes([192, 0, 2, 10]).to_be();
        assert_eq!(
            v4(raw, 0x0000_bb01),
            "192.0.2.10:443".parse::<SocketAddr>().unwrap()
        );
    }

    // Tier 2 by specification section 25.2. Needs a Windows machine, and is
    // requested explicitly rather than failing everywhere else.
    #[test]
    #[ignore = "tier 2: reads the machine's real socket table"]
    fn the_real_table_reads_and_names_this_process() {
        let mut t = IpHelperTable::new();
        let table = t.read().expect("the socket table reads");
        assert!(!table.is_empty(), "a running Windows machine holds sockets");
        assert!(
            table.entries().iter().any(|e| e.created.is_some()),
            "the owning-module classes report a creation instant"
        );
        assert!(
            table
                .entries()
                .iter()
                .filter(|e| e.proto == fragcap_core::flow::Proto::Udp)
                .all(|e| e.remote.is_none()),
            "no UDP entry may carry a remote"
        );
    }
}
