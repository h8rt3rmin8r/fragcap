// SPDX-License-Identifier: Apache-2.0

//! The action layer's value types and the pure selection of offered actions.
//!
//! This module is the machine-facing counterpart of the classifier's
//! human-readable remediations (slice S056). An [`Action`] is what the `--fix`
//! layer may perform for a [`Check`](super::Check); [`offered_actions`] turns a
//! [`Report`](super::Report) into the ordered list of actions `--fix` will offer.
//! The whole module is pure: it opens nothing, reads no environment, and takes no
//! side effect, so the selection, ordering, and capability degradation are tested
//! with hand-built reports on any target (FR-017).

use super::Report;

/// The compile-time and platform capabilities that shape which actions can be
/// offered and in which form. Injected (rather than read from `cfg!`) so the
/// selection is tested every way without a build matrix.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Capabilities {
    /// Whether the network-fetch capability (the `net` feature) is present.
    pub net: bool,
    /// Whether relaunching elevated is possible on this platform (Windows). When
    /// false, a `RelaunchElevated` action is not offered at all rather than offered
    /// only to fail: the non-Windows probe reports every session as not elevated,
    /// so without this the action would surface on platforms that cannot perform
    /// it.
    pub elevation: bool,
}

/// Which Wireshark extcap directory an `InstallExtcap` action targets.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExtcapScope {
    /// The per-user extcap directory (the default).
    User,
    /// The machine-wide extcap directory (needs the privilege to write it).
    Machine,
}

/// One remediation the `--fix` layer can perform, identified by kind. Every variant
/// maps to exactly one finding in the action catalog.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActionKind {
    /// npcap is absent: obtain it.
    ObtainNpcap,
    /// WinPcap API compatibility mode is off: relaunch the npcap installer.
    RelaunchNpcapInstaller,
    /// The session is not elevated: relaunch `doctor --fix` elevated.
    RelaunchElevated,
    /// The analyzer extcap integration is not registered: install it.
    InstallExtcap(ExtcapScope),
    /// The catalog store is missing: fetch the published catalog.
    FetchCatalog,
    /// No target entries are registered: run discovery.
    RunDiscovery,
}

impl ActionKind {
    /// Whether the primary form of this action needs the network-fetch capability.
    fn net_required(self) -> bool {
        matches!(
            self,
            ActionKind::ObtainNpcap | ActionKind::RelaunchNpcapInstaller | ActionKind::FetchCatalog
        )
    }

    /// The sentence printed before performing the primary (capable) form.
    fn primary_label(self) -> String {
        match self {
            ActionKind::ObtainNpcap => {
                "Fetch the vendor's Wireshark installer (which provides npcap) and launch it"
                    .to_string()
            }
            ActionKind::RelaunchNpcapInstaller => {
                "Fetch and launch the Wireshark installer, which runs npcap setup where the \
                 WinPcap API compatible mode can be enabled"
                    .to_string()
            }
            ActionKind::RelaunchElevated => "Relaunch doctor --fix elevated".to_string(),
            ActionKind::InstallExtcap(ExtcapScope::User) => {
                "Register the analyzer extcap integration for the current user".to_string()
            }
            ActionKind::InstallExtcap(ExtcapScope::Machine) => {
                "Register the analyzer extcap integration machine-wide".to_string()
            }
            ActionKind::FetchCatalog => "Fetch the current published catalog".to_string(),
            ActionKind::RunDiscovery => {
                "Run discovery (tiers 1 and 2) to register installed titles".to_string()
            }
        }
    }

    /// The sentence printed for the degraded (no-network) form. Only the
    /// net-required kinds have a distinct degraded label; the rest reuse the
    /// primary one because they do not degrade.
    fn degraded_label(self) -> String {
        match self {
            ActionKind::ObtainNpcap | ActionKind::RelaunchNpcapInstaller => {
                "Open the official download page for npcap (npcap.com, or the Wireshark installer \
                 that provides it); this build cannot fetch the installer"
                    .to_string()
            }
            ActionKind::FetchCatalog => {
                "This build cannot fetch; run `fragcap catalog update` with a net-enabled build"
                    .to_string()
            }
            _ => self.primary_label(),
        }
    }
}

/// One offered remediation: its kind, the sentence to print, whether its primary
/// form needs the network, and whether only the degraded form is available in this
/// build.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Action {
    /// Which remediation this is.
    pub kind: ActionKind,
    /// The human sentence `--fix` prints before performing it.
    pub label: String,
    /// Whether the primary form needs the `net` capability.
    pub net_required: bool,
    /// Whether only the no-network fallback is available (set by
    /// [`offered_actions`] when the capability is absent).
    pub degraded: bool,
}

impl Action {
    /// An action in its primary (non-degraded) form, with the label derived from
    /// the kind so the two cannot diverge.
    pub fn new(kind: ActionKind) -> Action {
        Action {
            kind,
            label: kind.primary_label(),
            net_required: kind.net_required(),
            degraded: false,
        }
    }

    /// Whether this action, in its currently-offered form, is presented as guidance
    /// only rather than as a performable step. A degraded `FetchCatalog` has no
    /// performable form in a default build, so it is surfaced as guidance and not
    /// offered as a confirm prompt (FR-016).
    pub fn guidance_only(&self) -> bool {
        self.degraded && self.kind == ActionKind::FetchCatalog
    }
}

/// The honest result of attempting one action (P-9, FR-011).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ActionOutcome {
    /// The action ran to success.
    Performed,
    /// The operator declined it.
    Skipped,
    /// A capability-limited fallback ran (reported as what happened, not as success
    /// of the primary form).
    Degraded,
    /// The action was attempted and failed; never reported as performed.
    Failed(String),
}

/// The pure selection: the actions `--fix` will offer for a report.
///
/// Walks the report in order, collecting each check's action; drops an action the
/// platform cannot perform (elevation off this platform); degrades a net-required
/// action when the capability is absent; and moves a
/// [`ActionKind::RelaunchElevated`] to the front so escalation precedes any
/// privilege-gated action (FR-014). The result is always a subset of the actions
/// carried by checks in `report` (FR-003): there is no other source of actions.
pub fn offered_actions(report: &Report, caps: Capabilities) -> Vec<Action> {
    let mut actions: Vec<Action> = Vec::new();
    for check in &report.checks {
        let Some(action) = &check.action else {
            continue;
        };
        // Never offer an action the platform cannot perform: the non-Windows probe
        // reports every session as not elevated, so a RelaunchElevated action would
        // otherwise surface only to fail.
        if action.kind == ActionKind::RelaunchElevated && !caps.elevation {
            continue;
        }
        let mut action = action.clone();
        if action.net_required && !caps.net {
            action.degraded = true;
            action.label = action.kind.degraded_label();
        }
        actions.push(action);
    }
    // Elevation first: escalate before any privilege-gated work.
    if let Some(pos) = actions
        .iter()
        .position(|a| a.kind == ActionKind::RelaunchElevated)
    {
        let elevated = actions.remove(pos);
        actions.insert(0, elevated);
    }
    actions
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::doctor::{Check, Report};

    const S: &str = "Section";

    fn report_with(actions: Vec<Option<Action>>) -> Report {
        let checks = actions
            .into_iter()
            .enumerate()
            .map(|(i, action)| match action {
                Some(a) => {
                    let name = Box::leak(format!("check{i}").into_boxed_str());
                    Check::warn_action(S, name, "detail", "remediation", a)
                }
                None => {
                    let name = Box::leak(format!("check{i}").into_boxed_str());
                    Check::ok(S, name, "detail")
                }
            })
            .collect();
        Report { checks }
    }

    #[test]
    fn offered_is_a_subset_of_the_reports_actions() {
        // Only checks that carry an action contribute; an ok check contributes none.
        let report = report_with(vec![
            None,
            Some(Action::new(ActionKind::InstallExtcap(ExtcapScope::User))),
            None,
            Some(Action::new(ActionKind::RunDiscovery)),
        ]);
        let offered = offered_actions(
            &report,
            Capabilities {
                net: true,
                elevation: true,
            },
        );
        let kinds: Vec<ActionKind> = offered.iter().map(|a| a.kind).collect();
        assert_eq!(
            kinds,
            vec![
                ActionKind::InstallExtcap(ExtcapScope::User),
                ActionKind::RunDiscovery
            ]
        );
    }

    #[test]
    fn an_action_whose_check_is_absent_is_never_offered() {
        // A report with no npcap action never yields ObtainNpcap (SC-003).
        let report = report_with(vec![Some(Action::new(ActionKind::RunDiscovery))]);
        let offered = offered_actions(
            &report,
            Capabilities {
                net: true,
                elevation: true,
            },
        );
        assert!(offered.iter().all(|a| a.kind != ActionKind::ObtainNpcap));
    }

    #[test]
    fn net_required_actions_degrade_when_the_capability_is_absent() {
        let report = report_with(vec![
            Some(Action::new(ActionKind::ObtainNpcap)),
            Some(Action::new(ActionKind::FetchCatalog)),
        ]);

        let capable = offered_actions(
            &report,
            Capabilities {
                net: true,
                elevation: true,
            },
        );
        assert!(capable.iter().all(|a| !a.degraded));
        assert!(capable[0]
            .label
            .contains("Fetch the vendor's Wireshark installer"));

        let degraded = offered_actions(
            &report,
            Capabilities {
                net: false,
                elevation: true,
            },
        );
        assert!(degraded.iter().all(|a| a.degraded));
        assert!(degraded[0].label.contains("download page"));
        // The degraded catalog action is guidance, not a performable prompt.
        let catalog = degraded
            .iter()
            .find(|a| a.kind == ActionKind::FetchCatalog)
            .unwrap();
        assert!(catalog.guidance_only());
    }

    #[test]
    fn non_net_actions_never_degrade() {
        let report = report_with(vec![
            Some(Action::new(ActionKind::InstallExtcap(ExtcapScope::Machine))),
            Some(Action::new(ActionKind::RunDiscovery)),
        ]);
        for caps in [
            Capabilities {
                net: true,
                elevation: true,
            },
            Capabilities {
                net: false,
                elevation: true,
            },
        ] {
            let offered = offered_actions(&report, caps);
            assert!(offered.iter().all(|a| !a.degraded), "caps {caps:?}");
        }
    }

    #[test]
    fn relaunch_elevated_is_offered_first() {
        // Even when the elevation finding appears after others in the report, its
        // action is offered first so escalation precedes privilege-gated work.
        let report = report_with(vec![
            Some(Action::new(ActionKind::InstallExtcap(ExtcapScope::User))),
            Some(Action::new(ActionKind::RelaunchElevated)),
            Some(Action::new(ActionKind::RunDiscovery)),
        ]);
        let offered = offered_actions(
            &report,
            Capabilities {
                net: true,
                elevation: true,
            },
        );
        assert_eq!(offered[0].kind, ActionKind::RelaunchElevated);
        // The others keep their relative order after the elevation move.
        assert_eq!(
            offered[1].kind,
            ActionKind::InstallExtcap(ExtcapScope::User)
        );
        assert_eq!(offered[2].kind, ActionKind::RunDiscovery);
    }

    #[test]
    fn relaunch_elevated_is_dropped_when_the_platform_cannot_elevate() {
        // On a platform without elevation (the non-Windows probe reports every
        // session as not elevated), the elevation action is not offered at all
        // rather than offered only to fail.
        let report = report_with(vec![
            Some(Action::new(ActionKind::RelaunchElevated)),
            Some(Action::new(ActionKind::RunDiscovery)),
        ]);
        let offered = offered_actions(
            &report,
            Capabilities {
                net: true,
                elevation: false,
            },
        );
        let kinds: Vec<ActionKind> = offered.iter().map(|a| a.kind).collect();
        assert_eq!(kinds, vec![ActionKind::RunDiscovery]);
    }

    #[test]
    fn a_report_with_no_actions_offers_nothing() {
        let report = report_with(vec![None, None]);
        assert!(offered_actions(
            &report,
            Capabilities {
                net: true,
                elevation: true
            }
        )
        .is_empty());
    }
}
