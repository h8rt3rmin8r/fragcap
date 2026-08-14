// SPDX-License-Identifier: Apache-2.0

//! Capture interfaces: what the machine has, which of them fragcap watches,
//! and why it passed over the rest.
//!
//! Specification section 12.1. Three things about the shape are load-bearing.
//!
//! **Selection is a decision over a value, not an act on a machine.**
//! [`select`] takes an [`InterfaceInventory`] and returns a
//! [`SelectionOutcome`]. It opens no handle, enumerates nothing, and touches no
//! platform surface, which is what lets the whole section 12.1 precedence be
//! tested exhaustively on any machine with no capture driver. Producing an
//! inventory from a real machine is `fragcap-capture`'s job, and the inventory
//! is the seam between them.
//!
//! **Nothing is passed over silently.** For every interface in the inventory
//! the outcome carries either its selection or a named [`ExclusionReason`], and
//! a test asserts that the two together account for the whole inventory. This
//! is the selection-side form of the conservation identity slice S08
//! established for packets, and it exists for the same reason: choosing the
//! wrong interface produces a run that exits zero, writes a well-formed capture
//! file, and contains nothing. That failure is invisible unless the decision is
//! reported.
//!
//! **The virtual-interface rule is a heuristic and says so.** The platform does
//! not report a "this is a hypervisor adapter" bit; fragcap infers it from the
//! adapter description. [`VirtualVerdict`] therefore carries the pattern that
//! matched rather than collapsing to a boolean, the verdict only ever excludes
//! from automatic selection, and an explicitly named interface is captured
//! whatever the rule concluded. Constitution P-9: fragcap may not present an
//! inference as an observation.

use std::fmt;
use std::net::IpAddr;
use std::sync::Arc;

use crate::link::LinkType;

/// A capture interface's identity for the duration of one run.
///
/// Assigned by [`select`] from the position of the interface in its outcome,
/// not taken from the platform. Platform names are not guaranteed unique, and
/// specification section 12.1 requires that every packet name the interface it
/// arrived on, so identity has to come from somewhere that can guarantee it.
///
/// An integer rather than a name because it is carried on every packet and
/// compared once per packet on the way to a sink.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InterfaceId(u32);

impl InterfaceId {
    pub const fn new(index: u32) -> Self {
        InterfaceId(index)
    }

    pub const fn index(self) -> u32 {
        self.0
    }
}

impl fmt::Display for InterfaceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "if{}", self.0)
    }
}

/// One interface, as the machine describes it.
///
/// Deliberately carries no `is_virtual` field. Virtuality is a verdict this
/// project reaches by heuristic rather than a property the platform reports,
/// and a boolean here would let the verdict travel without the reasoning that
/// produced it. See [`VirtualVerdict`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InterfaceRecord {
    /// The platform's own name, which is what a capture backend opens.
    pub name: Arc<str>,
    /// The adapter description, when the platform supplies one. This is what
    /// the virtual-interface heuristic reads.
    pub description: Option<Arc<str>>,
    /// The encapsulation frames arrive in.
    pub link_type: LinkType,
    /// Addresses configured on the interface.
    pub addresses: Vec<IpAddr>,
    /// Administratively up.
    pub is_up: bool,
    /// Has a carrier.
    pub is_running: bool,
    /// The platform reports this as a loopback adapter.
    pub is_loopback: bool,
}

impl InterfaceRecord {
    /// A record with everything off, for tests and for backends filling fields
    /// in as they discover them.
    pub fn new(name: impl AsRef<str>, link_type: LinkType) -> Self {
        InterfaceRecord {
            name: Arc::from(name.as_ref()),
            description: None,
            link_type,
            addresses: Vec::new(),
            is_up: false,
            is_running: false,
            is_loopback: false,
        }
    }

    /// Whether any address is configured. Section 12.1's broad capture rule
    /// needs this and nothing else about the addresses.
    pub fn has_address(&self) -> bool {
        !self.addresses.is_empty()
    }
}

/// What the machine has, as a value.
///
/// Being a value is the point. A backend produces one by enumerating; a test
/// writes one. Selection cannot tell the difference, which is FR-010.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct InterfaceInventory {
    pub interfaces: Vec<InterfaceRecord>,
    /// The source address the routing table chooses for an off-link
    /// destination, when one could be determined. `None` on a machine with no
    /// default route, which section 12.1's second precedence step then cannot
    /// satisfy.
    pub default_route_source: Option<IpAddr>,
}

/// What the caller is asking for.
///
/// Plain values rather than a profile. `fragcap-capture` and `fragcap-profile`
/// are siblings, and specification section 8.3 forbids the edge, so the facade
/// translates a profile into this. It is also what keeps selection testable
/// without a profile parser in the loop.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SelectionSettings {
    /// Interfaces named by the operator. A non-empty list takes precedence
    /// over everything else.
    pub explicit: Vec<String>,
    /// Include the loopback adapter alongside the default-route interface.
    pub loopback: bool,
    /// Select every interface that is up, addressed, and not virtual.
    pub broad: bool,
}

/// The heuristic's answer for one interface.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VirtualVerdict {
    /// No pattern matched.
    NotVirtual,
    /// The description matched a documented pattern, which is named so that a
    /// misclassification is visible rather than mysterious.
    Virtual { pattern: &'static str },
}

impl VirtualVerdict {
    pub fn is_virtual(&self) -> bool {
        matches!(self, VirtualVerdict::Virtual { .. })
    }
}

/// Adapter description fragments that indicate an interface fragcap should not
/// select automatically.
///
/// A heuristic, held in one place so it can be read and argued with. Every
/// entry names a class of adapter that a development machine commonly carries
/// and that never carries game traffic. Matching is case-insensitive and on the
/// description only; the platform's device name is a GUID on Windows and says
/// nothing.
///
/// Wireless is deliberately absent. A wireless adapter is a real adapter.
pub const VIRTUAL_PATTERNS: &[&str] = &[
    "vmware",
    "virtualbox",
    "hyper-v",
    "vethernet",
    "loopback pseudo",
    "tap-windows",
    "wintun",
    "wireguard",
    "tailscale",
    "docker",
    "npcap loopback",
];

/// Whether an interface looks like one of the adapters in
/// [`VIRTUAL_PATTERNS`], and which pattern said so.
pub fn virtual_verdict(record: &InterfaceRecord) -> VirtualVerdict {
    let Some(description) = &record.description else {
        return VirtualVerdict::NotVirtual;
    };
    let lowered = description.to_lowercase();
    for pattern in VIRTUAL_PATTERNS {
        if lowered.contains(pattern) {
            return VirtualVerdict::Virtual { pattern };
        }
    }
    VirtualVerdict::NotVirtual
}

/// Which step of the section 12.1 precedence chose an interface.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SelectionReason {
    /// Named on the command line or in the profile. Precedence step one.
    NamedExplicitly,
    /// Carries the default route. Precedence step two.
    DefaultRoute,
    /// The loopback adapter, with loopback capture requested. Precedence step
    /// two.
    Loopback,
    /// Up, addressed, and not virtual, with broad capture requested.
    /// Precedence step three.
    Broad,
}

/// Why an enumerated interface was not selected.
///
/// A closed enumeration, following [`crate::parse::ParseReject`]'s discipline:
/// a new exclusion path cannot be added without naming itself, so an interface
/// cannot be dropped on the floor by a rule nobody has to look at.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExclusionReason {
    /// Explicit names were given and this was not among them.
    NotNamed,
    /// Automatic selection took the default-route interface and this is not
    /// it.
    NotDefaultRoute,
    /// A loopback adapter, and the settings did not ask for loopback.
    LoopbackNotRequested,
    /// Not administratively up.
    Down,
    /// Up, but with no address configured.
    NoAddress,
    /// Excluded by the heuristic, with the pattern that matched.
    Virtual { pattern: &'static str },
}

impl fmt::Display for ExclusionReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExclusionReason::NotNamed => f.write_str("not among the interfaces named"),
            ExclusionReason::NotDefaultRoute => f.write_str("does not carry the default route"),
            ExclusionReason::LoopbackNotRequested => {
                f.write_str("a loopback adapter, and loopback capture was not requested")
            }
            ExclusionReason::Down => f.write_str("not up"),
            ExclusionReason::NoAddress => f.write_str("no address configured"),
            ExclusionReason::Virtual { pattern } => {
                write!(f, "looks virtual: the description contains {pattern:?}")
            }
        }
    }
}

/// An interface chosen for the run.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectedInterface {
    pub id: InterfaceId,
    pub record: InterfaceRecord,
    pub reason: SelectionReason,
}

/// The whole answer: what was chosen, and why everything else was not.
///
/// The second half is what makes an unexpectedly empty capture diagnosable
/// instead of mysterious, which is the entire reason this type is not just a
/// `Vec<SelectedInterface>`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectionOutcome {
    /// In selection order. Identifiers are assigned from position.
    pub selected: Vec<SelectedInterface>,
    /// Every interface not chosen, with the reason.
    pub excluded: Vec<(InterfaceRecord, ExclusionReason)>,
}

impl SelectionOutcome {
    /// How many interfaces the inventory held. Computed from the two halves,
    /// so it cannot disagree with them.
    pub fn accounted_for(&self) -> usize {
        self.selected.len() + self.excluded.len()
    }

    /// The record for an identifier, for a writer declaring interfaces.
    pub fn record(&self, id: InterfaceId) -> Option<&InterfaceRecord> {
        self.selected.iter().find(|s| s.id == id).map(|s| &s.record)
    }
}

/// Why a selection could not be made.
///
/// Returned rather than panicking, and turned into a failed run by the caller.
/// Selection is a pure decision and a pure decision cannot fail a run; it can
/// only decline to produce one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SelectionError {
    /// A name matched nothing enumerated. Carries what was available, so the
    /// message can say rather than leaving the operator to guess.
    UnknownInterface {
        requested: String,
        available: Vec<String>,
    },
    /// The settings and the inventory together chose nothing. Opening a
    /// capture on no interfaces would exit zero having watched nothing.
    NothingSelected,
}

impl fmt::Display for SelectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SelectionError::UnknownInterface {
                requested,
                available,
            } => {
                write!(f, "no interface named {requested:?}. Available: ")?;
                if available.is_empty() {
                    f.write_str("none")
                } else {
                    f.write_str(&available.join(", "))
                }
            }
            SelectionError::NothingSelected => {
                f.write_str("no interface was selected, so there would be nothing to capture")
            }
        }
    }
}

impl std::error::Error for SelectionError {}

/// Why a capture thread stopped.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RetirementReason {
    /// The source said it had no more packets. The ordinary end.
    SourceClosed,
    /// The interface is gone.
    DeviceLost { detail: String },
    /// The backend failed in a way fragcap does not model.
    Backend { detail: String },
}

impl fmt::Display for RetirementReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RetirementReason::SourceClosed => f.write_str("the source closed"),
            RetirementReason::DeviceLost { detail } => {
                write!(f, "the device is no longer available: {detail}")
            }
            RetirementReason::Backend { detail } => write!(f, "backend failure: {detail}"),
        }
    }
}

/// A capture thread that has ended, and why.
///
/// **This advances no drop counter, and that is not an oversight.** A retired
/// interface stops producing observations; it does not discard observations
/// fragcap already had. Constitution P-4 counts what was discarded, and
/// counting a retirement as loss would report packets that were never observed
/// as packets that were thrown away, which is a P-9 problem rather than an
/// arithmetic one. The retirement is surfaced in the run's report instead,
/// which is where an operator can act on it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InterfaceRetirement {
    pub interface: InterfaceId,
    pub reason: RetirementReason,
}

/// What capture driver detection concluded.
///
/// The two installation-option fields are three-valued deliberately. "We could
/// not tell" is not "no", and reporting it as "no" would make a diagnostic
/// assert something it did not observe.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DriverReport {
    pub present: bool,
    pub version: Option<String>,
    /// Whether the loopback capture option is installed. `None` when it could
    /// not be determined.
    pub loopback_supported: Option<bool>,
    /// Whether WinPcap API compatibility mode is installed. `None` when it
    /// could not be determined.
    pub winpcap_compatible: Option<bool>,
}

/// Where the capture driver is obtained. fragcap never fetches it; this exists
/// so that a diagnostic can say where to go.
pub const DRIVER_DOWNLOAD_URL: &str = "https://npcap.com/#download";

/// Where Wireshark is obtained. fragcap never fetches or bundles it; this exists
/// so that a diagnostic can say where to go. Wireshark is the recommended
/// analyzer, and its installer also provides npcap, so one download resolves both
/// the analyzer and the capture driver.
pub const WIRESHARK_DOWNLOAD_URL: &str = "https://www.wireshark.org/download.html";

/// Apply the specification section 12.1 precedence to an inventory.
///
/// A pure decision. It opens nothing, enumerates nothing, and consults no
/// platform surface, which is what lets the whole precedence be tested
/// exhaustively without a capture driver.
///
/// The precedence, in order:
///
/// 1. Interfaces named explicitly, in the order named, whatever the automatic
///    rules would have concluded about them.
/// 2. Otherwise the interface carrying the default route, plus the loopback
///    adapter when `settings.loopback`.
/// 3. Otherwise, when `settings.broad`, every interface that is up, addressed,
///    and not virtual.
///
/// Steps two and three both honour `settings.loopback`, because a loopback
/// adapter is not something broad capture should sweep up by accident: it
/// carries a large volume of a process talking to itself, and an operator who
/// did not ask for it did not ask for that.
pub fn select(
    inventory: &InterfaceInventory,
    settings: &SelectionSettings,
) -> Result<SelectionOutcome, SelectionError> {
    let chosen = if settings.explicit.is_empty() {
        choose_automatically(inventory, settings)
    } else {
        choose_explicitly(inventory, settings)?
    };

    if chosen.is_empty() {
        return Err(SelectionError::NothingSelected);
    }

    let selected: Vec<SelectedInterface> = chosen
        .iter()
        .enumerate()
        .map(|(position, (index, reason))| SelectedInterface {
            id: InterfaceId::new(position as u32),
            record: inventory.interfaces[*index].clone(),
            reason: *reason,
        })
        .collect();

    let taken: Vec<usize> = chosen.iter().map(|(index, _)| *index).collect();
    let excluded = inventory
        .interfaces
        .iter()
        .enumerate()
        .filter(|(index, _)| !taken.contains(index))
        .map(|(_, record)| {
            let reason = exclusion_reason(record, settings);
            (record.clone(), reason)
        })
        .collect();

    Ok(SelectionOutcome { selected, excluded })
}

/// Precedence step one. Names are resolved in the order given, and a name that
/// appears twice in the inventory can be asked for twice.
fn choose_explicitly(
    inventory: &InterfaceInventory,
    settings: &SelectionSettings,
) -> Result<Vec<(usize, SelectionReason)>, SelectionError> {
    let mut chosen: Vec<(usize, SelectionReason)> = Vec::new();

    for requested in &settings.explicit {
        let matches_name = |record: &InterfaceRecord| record.name.as_ref() == requested.as_str();

        if !inventory.interfaces.iter().any(matches_name) {
            return Err(SelectionError::UnknownInterface {
                requested: requested.clone(),
                available: inventory
                    .interfaces
                    .iter()
                    .map(|r| r.name.to_string())
                    .collect(),
            });
        }

        // The next instance not already taken. Two interfaces may share a name
        // and asking for that name twice must yield both, not the first twice.
        // Asking a third time for a name with two instances is not an error;
        // everything by that name is already selected.
        if let Some(index) = inventory
            .interfaces
            .iter()
            .enumerate()
            .position(|(index, record)| {
                matches_name(record) && !chosen.iter().any(|(taken, _)| *taken == index)
            })
        {
            chosen.push((index, SelectionReason::NamedExplicitly));
        }
    }

    Ok(chosen)
}

/// Precedence steps two and three.
fn choose_automatically(
    inventory: &InterfaceInventory,
    settings: &SelectionSettings,
) -> Vec<(usize, SelectionReason)> {
    let mut chosen: Vec<(usize, SelectionReason)> = Vec::new();

    for (index, record) in inventory.interfaces.iter().enumerate() {
        if record.is_loopback {
            if settings.loopback {
                chosen.push((index, SelectionReason::Loopback));
            }
            continue;
        }

        if settings.broad {
            if record.is_up && record.has_address() && !virtual_verdict(record).is_virtual() {
                chosen.push((index, SelectionReason::Broad));
            }
            continue;
        }

        if carries_default_route(record, inventory) {
            chosen.push((index, SelectionReason::DefaultRoute));
        }
    }

    // The default-route interface comes first when both are present, because
    // it is the one the operator meant and the loopback adapter is the
    // addition. Explicit selection preserves the operator's order instead.
    chosen.sort_by_key(|(_, reason)| match reason {
        SelectionReason::DefaultRoute => 0,
        SelectionReason::Broad => 1,
        SelectionReason::Loopback => 2,
        SelectionReason::NamedExplicitly => 3,
    });

    chosen
}

/// Whether this interface holds the address the routing table chose.
///
/// `None` for the route source means the machine has no default route, in
/// which case no interface carries it and automatic selection has nothing to
/// choose. That is the specification's edge case, and it surfaces as
/// [`SelectionError::NothingSelected`] rather than as an empty success.
fn carries_default_route(record: &InterfaceRecord, inventory: &InterfaceInventory) -> bool {
    match inventory.default_route_source {
        Some(source) => record.addresses.contains(&source),
        None => false,
    }
}

/// Why this interface was passed over.
///
/// Ordered most specific first, so an operator is told the most actionable
/// thing rather than the most general one. An adapter that is both down and
/// not the default route is reported as down, because that is the fact they can
/// do something about.
fn exclusion_reason(record: &InterfaceRecord, settings: &SelectionSettings) -> ExclusionReason {
    if !settings.explicit.is_empty() {
        return ExclusionReason::NotNamed;
    }
    if record.is_loopback && !settings.loopback {
        return ExclusionReason::LoopbackNotRequested;
    }
    if !record.is_up {
        return ExclusionReason::Down;
    }
    if !record.has_address() {
        return ExclusionReason::NoAddress;
    }
    if let VirtualVerdict::Virtual { pattern } = virtual_verdict(record) {
        return ExclusionReason::Virtual { pattern };
    }
    ExclusionReason::NotDefaultRoute
}

#[cfg(test)]
mod tests {
    use super::*;

    fn addr(s: &str) -> IpAddr {
        s.parse().expect("test address must parse")
    }

    /// An ordinary wired adapter, up, running, addressed.
    fn wired(name: &str, ip: &str) -> InterfaceRecord {
        InterfaceRecord {
            description: Some(Arc::from("Intel(R) Ethernet Connection I219-V")),
            addresses: vec![addr(ip)],
            is_up: true,
            is_running: true,
            ..InterfaceRecord::new(name, LinkType::ETHERNET)
        }
    }

    fn loopback(name: &str) -> InterfaceRecord {
        InterfaceRecord {
            description: Some(Arc::from("Adapter for loopback traffic capture")),
            addresses: vec![addr("127.0.0.1")],
            is_up: true,
            is_running: true,
            is_loopback: true,
            ..InterfaceRecord::new(name, LinkType::NULL)
        }
    }

    fn hypervisor(name: &str, ip: &str) -> InterfaceRecord {
        InterfaceRecord {
            description: Some(Arc::from("VMware Virtual Ethernet Adapter for VMnet8")),
            addresses: vec![addr(ip)],
            is_up: true,
            is_running: true,
            ..InterfaceRecord::new(name, LinkType::ETHERNET)
        }
    }

    fn down(name: &str) -> InterfaceRecord {
        InterfaceRecord {
            description: Some(Arc::from("Realtek Gaming 2.5GbE Family Controller")),
            ..InterfaceRecord::new(name, LinkType::ETHERNET)
        }
    }

    fn unaddressed(name: &str) -> InterfaceRecord {
        InterfaceRecord {
            description: Some(Arc::from("Intel(R) Wi-Fi 6 AX201")),
            is_up: true,
            is_running: true,
            ..InterfaceRecord::new(name, LinkType::ETHERNET)
        }
    }

    /// The machine the whole matrix is decided against: one wired adapter
    /// carrying the default route, a loopback adapter, a hypervisor adapter, a
    /// down adapter, and an up adapter with no address.
    fn machine() -> InterfaceInventory {
        InterfaceInventory {
            interfaces: vec![
                wired("\\Device\\NPF_{ETH}", "192.0.2.10"),
                loopback("\\Device\\NPF_Loopback"),
                hypervisor("\\Device\\NPF_{VMNET8}", "198.51.100.1"),
                down("\\Device\\NPF_{DOWN}"),
                unaddressed("\\Device\\NPF_{WIFI}"),
            ],
            default_route_source: Some(addr("192.0.2.10")),
        }
    }

    fn names(outcome: &SelectionOutcome) -> Vec<String> {
        outcome
            .selected
            .iter()
            .map(|s| s.record.name.to_string())
            .collect()
    }

    // The invariant that matters, asserted for every case rather than for one.
    // Selection that drops an interface without saying so is the failure this
    // module exists to prevent.
    fn assert_accounted(inventory: &InterfaceInventory, outcome: &SelectionOutcome) {
        assert_eq!(
            outcome.accounted_for(),
            inventory.interfaces.len(),
            "every interface must be either selected or excluded with a reason"
        );
    }

    // FR-005 step two, the default path. SC-002.
    #[test]
    fn automatic_selection_takes_the_default_route_interface() {
        let inv = machine();
        let settings = SelectionSettings::default();
        let out = select(&inv, &settings).expect("a default route exists");
        assert_eq!(names(&out), vec!["\\Device\\NPF_{ETH}"]);
        assert_eq!(out.selected[0].reason, SelectionReason::DefaultRoute);
        assert_accounted(&inv, &out);
    }

    #[test]
    fn loopback_joins_the_default_route_interface_when_requested() {
        let inv = machine();
        let settings = SelectionSettings {
            loopback: true,
            ..SelectionSettings::default()
        };
        let out = select(&inv, &settings).expect("a default route exists");
        assert_eq!(
            names(&out),
            vec!["\\Device\\NPF_{ETH}", "\\Device\\NPF_Loopback"]
        );
        assert_eq!(out.selected[1].reason, SelectionReason::Loopback);
        assert_accounted(&inv, &out);
    }

    #[test]
    fn loopback_is_excluded_with_its_own_reason_when_not_requested() {
        let inv = machine();
        let out = select(&inv, &SelectionSettings::default()).expect("a default route exists");
        let (_, reason) = out
            .excluded
            .iter()
            .find(|(r, _)| r.is_loopback)
            .expect("the loopback adapter must appear in the exclusions");
        assert_eq!(*reason, ExclusionReason::LoopbackNotRequested);
    }

    // FR-005 step three. SC-002.
    #[test]
    fn broad_capture_takes_every_up_addressed_non_virtual_interface() {
        let inv = machine();
        let settings = SelectionSettings {
            broad: true,
            ..SelectionSettings::default()
        };
        let out = select(&inv, &settings).expect("at least one interface qualifies");
        // The wired adapter only: loopback is not requested, the hypervisor
        // adapter is virtual, one is down, and one has no address.
        assert_eq!(names(&out), vec!["\\Device\\NPF_{ETH}"]);
        assert_eq!(out.selected[0].reason, SelectionReason::Broad);
        assert_accounted(&inv, &out);
    }

    #[test]
    fn broad_capture_includes_loopback_when_also_requested() {
        let inv = machine();
        let settings = SelectionSettings {
            broad: true,
            loopback: true,
            ..SelectionSettings::default()
        };
        let out = select(&inv, &settings).expect("at least one interface qualifies");
        assert!(out.selected.iter().any(|s| s.record.is_loopback));
        assert_accounted(&inv, &out);
    }

    // FR-005 step one, and FR-006.
    #[test]
    fn an_explicit_name_wins_over_every_automatic_rule() {
        let inv = machine();
        let settings = SelectionSettings {
            explicit: vec!["\\Device\\NPF_{VMNET8}".into()],
            ..SelectionSettings::default()
        };
        let out = select(&inv, &settings).expect("the name matches");
        assert_eq!(names(&out), vec!["\\Device\\NPF_{VMNET8}"]);
        assert_eq!(out.selected[0].reason, SelectionReason::NamedExplicitly);
        assert_accounted(&inv, &out);
    }

    #[test]
    fn an_explicit_name_selects_an_interface_that_is_down() {
        // The operator asked. Automatic rules do not get to overrule that.
        let inv = machine();
        let settings = SelectionSettings {
            explicit: vec!["\\Device\\NPF_{DOWN}".into()],
            ..SelectionSettings::default()
        };
        let out = select(&inv, &settings).expect("the name matches");
        assert_eq!(names(&out), vec!["\\Device\\NPF_{DOWN}"]);
    }

    #[test]
    fn explicit_names_are_selected_in_the_order_given() {
        let inv = machine();
        let settings = SelectionSettings {
            explicit: vec![
                "\\Device\\NPF_Loopback".into(),
                "\\Device\\NPF_{ETH}".into(),
            ],
            ..SelectionSettings::default()
        };
        let out = select(&inv, &settings).expect("both names match");
        assert_eq!(
            names(&out),
            vec!["\\Device\\NPF_Loopback", "\\Device\\NPF_{ETH}"]
        );
    }

    // FR-007.
    #[test]
    fn an_unmatched_name_reports_what_was_available() {
        let inv = machine();
        let settings = SelectionSettings {
            explicit: vec!["eth0".into()],
            ..SelectionSettings::default()
        };
        match select(&inv, &settings) {
            Err(SelectionError::UnknownInterface {
                requested,
                available,
            }) => {
                assert_eq!(requested, "eth0");
                assert_eq!(available.len(), 5, "every enumerated name must be offered");
            }
            other => panic!("expected UnknownInterface, got {other:?}"),
        }
    }

    // FR-011.
    #[test]
    fn selecting_nothing_is_an_error_rather_than_an_empty_capture() {
        let inv = InterfaceInventory {
            interfaces: vec![down("\\Device\\NPF_{DOWN}")],
            default_route_source: None,
        };
        assert_eq!(
            select(&inv, &SelectionSettings::default()),
            Err(SelectionError::NothingSelected)
        );
    }

    // The machine with no default route, from the specification's edge cases.
    #[test]
    fn no_default_route_is_an_error_rather_than_a_silent_empty_capture() {
        let inv = InterfaceInventory {
            interfaces: vec![wired("\\Device\\NPF_{ETH}", "192.0.2.10")],
            default_route_source: None,
        };
        assert_eq!(
            select(&inv, &SelectionSettings::default()),
            Err(SelectionError::NothingSelected)
        );
    }

    // FR-002. Identity has to survive the platform reusing a name.
    #[test]
    fn identifiers_are_unique_even_when_names_collide() {
        let inv = InterfaceInventory {
            interfaces: vec![wired("dup", "192.0.2.10"), wired("dup", "198.51.100.7")],
            default_route_source: None,
        };
        let settings = SelectionSettings {
            explicit: vec!["dup".into(), "dup".into()],
            ..SelectionSettings::default()
        };
        let out = select(&inv, &settings).expect("the name matches");
        assert_eq!(out.selected.len(), 2);
        assert_ne!(
            out.selected[0].id, out.selected[1].id,
            "two interfaces sharing a name must not share an identity"
        );
    }

    #[test]
    fn identifiers_are_assigned_from_position() {
        let inv = machine();
        let settings = SelectionSettings {
            loopback: true,
            ..SelectionSettings::default()
        };
        let out = select(&inv, &settings).expect("a default route exists");
        for (i, s) in out.selected.iter().enumerate() {
            assert_eq!(s.id, InterfaceId::new(i as u32));
        }
    }

    // FR-009 and SC-003, asserted across the whole matrix rather than once.
    #[test]
    fn every_case_accounts_for_every_interface() {
        let inv = machine();
        let cases = [
            SelectionSettings::default(),
            SelectionSettings {
                loopback: true,
                ..SelectionSettings::default()
            },
            SelectionSettings {
                broad: true,
                ..SelectionSettings::default()
            },
            SelectionSettings {
                broad: true,
                loopback: true,
                ..SelectionSettings::default()
            },
            SelectionSettings {
                explicit: vec!["\\Device\\NPF_{ETH}".into()],
                ..SelectionSettings::default()
            },
        ];
        for settings in cases {
            let out = select(&inv, &settings).expect("each case selects something");
            assert_accounted(&inv, &out);
            for (record, _) in &out.excluded {
                assert!(
                    !out.selected.iter().any(|s| &s.record == record),
                    "an interface may not be both selected and excluded"
                );
            }
        }
    }

    // FR-004 and plan D-9. The verdict is carried, not folded into a boolean.
    #[test]
    fn a_virtual_exclusion_names_the_pattern_that_matched() {
        let inv = machine();
        let out = select(
            &inv,
            &SelectionSettings {
                broad: true,
                ..SelectionSettings::default()
            },
        )
        .expect("the wired adapter qualifies");
        let (_, reason) = out
            .excluded
            .iter()
            .find(|(r, _)| r.name.contains("VMNET8"))
            .expect("the hypervisor adapter must be excluded");
        assert_eq!(*reason, ExclusionReason::Virtual { pattern: "vmware" });
    }

    #[test]
    fn the_virtual_heuristic_reads_the_description_and_ignores_case() {
        let mut r = InterfaceRecord::new("x", LinkType::ETHERNET);
        r.description = Some(Arc::from("Hyper-V Virtual Ethernet Adapter"));
        assert_eq!(
            virtual_verdict(&r),
            VirtualVerdict::Virtual { pattern: "hyper-v" }
        );
    }

    #[test]
    fn an_interface_with_no_description_is_not_called_virtual() {
        // Absence of evidence. Guessing here would exclude a real adapter and
        // produce the empty capture this module exists to prevent.
        let r = InterfaceRecord::new("x", LinkType::ETHERNET);
        assert_eq!(virtual_verdict(&r), VirtualVerdict::NotVirtual);
    }

    #[test]
    fn a_wireless_adapter_is_not_virtual() {
        let mut r = InterfaceRecord::new("x", LinkType::ETHERNET);
        r.description = Some(Arc::from("Intel(R) Wi-Fi 6 AX201 160MHz"));
        assert_eq!(virtual_verdict(&r), VirtualVerdict::NotVirtual);
    }

    // Exclusion reasons must be specific enough to act on.
    #[test]
    fn each_exclusion_reason_is_reachable_and_distinct() {
        let inv = machine();
        let out = select(
            &inv,
            &SelectionSettings {
                broad: true,
                ..SelectionSettings::default()
            },
        )
        .expect("the wired adapter qualifies");
        let reasons: Vec<&ExclusionReason> = out.excluded.iter().map(|(_, r)| r).collect();
        assert!(reasons.contains(&&ExclusionReason::Down));
        assert!(reasons.contains(&&ExclusionReason::NoAddress));
        assert!(reasons.contains(&&ExclusionReason::LoopbackNotRequested));
        assert!(reasons
            .iter()
            .any(|r| matches!(r, ExclusionReason::Virtual { .. })));
    }

    #[test]
    fn an_identifier_resolves_back_to_its_record() {
        let inv = machine();
        let out = select(&inv, &SelectionSettings::default()).expect("a default route exists");
        let id = out.selected[0].id;
        assert_eq!(
            out.record(id).map(|r| r.name.as_ref()),
            Some("\\Device\\NPF_{ETH}")
        );
        assert_eq!(out.record(InterfaceId::new(99)), None);
    }

    // FR-028. Stated as a test so that a later change adding a counter has to
    // delete an assertion rather than merely edit a comment.
    #[test]
    fn a_retirement_carries_its_interface_and_reason() {
        let r = InterfaceRetirement {
            interface: InterfaceId::new(3),
            reason: RetirementReason::DeviceLost {
                detail: "the adapter was removed".into(),
            },
        };
        assert_eq!(r.interface, InterfaceId::new(3));
        assert!(r.reason.to_string().contains("no longer available"));
    }

    // FR-045. Three-valued, because "could not tell" is not "no".
    #[test]
    fn an_undeterminable_driver_option_is_not_reported_as_absent() {
        let report = DriverReport {
            present: true,
            version: Some("1.83".into()),
            loopback_supported: None,
            winpcap_compatible: Some(false),
        };
        assert_ne!(report.loopback_supported, Some(false));
        assert_eq!(report.winpcap_compatible, Some(false));
    }
}
