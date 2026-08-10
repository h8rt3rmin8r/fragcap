// SPDX-License-Identifier: Apache-2.0

//! Turning what the machine has into an [`InterfaceInventory`].
//!
//! This is the seam that makes specification section 12.1's precedence testable.
//! Everything platform-specific happens here and produces a plain value;
//! `fragcap_core::interface::select` then decides over that value with no
//! machine involved. A test writes an inventory by hand and exercises the whole
//! precedence; this function is the only part that needs a real adapter.
//!
//! Nothing here opens a capture handle. FR-003: an operator can be told what
//! exists before anything is captured, which matters because the most common
//! first question is "what can I watch" and the most common first failure is
//! "the driver is not installed".

use fragcap_core::error::SourceError;
use fragcap_core::interface::{InterfaceInventory, InterfaceRecord};
use fragcap_core::link::LinkType;

use super::route;

/// Every capture-capable interface, plus the default route's source address.
///
/// The link type is reported as Ethernet for every interface, and that is a
/// placeholder rather than an observation: libpcap only reveals an interface's
/// data link type once a handle is open on it, and opening one here would
/// violate FR-003 and would need privilege this call does not have. The value
/// each capture actually parses against comes from
/// [`super::LiveSource::link_type`], which reads it from the open handle, so
/// nothing downstream depends on the placeholder.
pub fn enumerate() -> Result<InterfaceInventory, SourceError> {
    let devices = pcap::Device::list().map_err(|e| SourceError::Backend {
        detail: format!("interface enumeration failed: {e}"),
    })?;

    let interfaces = devices
        .into_iter()
        .map(|device| InterfaceRecord {
            description: device.desc.as_deref().map(Into::into),
            addresses: device.addresses.iter().map(|a| a.addr).collect(),
            is_up: device.flags.is_up(),
            is_running: device.flags.is_running(),
            is_loopback: device.flags.is_loopback(),
            ..InterfaceRecord::new(&device.name, LinkType::ETHERNET)
        })
        .collect();

    Ok(InterfaceInventory {
        interfaces,
        default_route_source: route::default_route_source(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // Runs on any machine with the driver installed and asserts only what is
    // true of every machine. Asserting a particular adapter exists would be
    // asserting something about the runner.
    #[test]
    fn enumeration_answers_without_opening_a_handle() {
        let Ok(inventory) = enumerate() else {
            eprintln!("skipped: interface enumeration failed, most likely no capture driver");
            return;
        };
        for record in &inventory.interfaces {
            assert!(
                !record.name.is_empty(),
                "an interface with no name is unusable"
            );
        }
    }
}
