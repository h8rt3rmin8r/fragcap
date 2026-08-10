// SPDX-License-Identifier: Apache-2.0

//! The two launcher chains reconnaissance actually observed, replayed.
//!
//! Specification section 5.4 and Appendix D record both, and they are the
//! reason this project exists in the shape it does. Everything here runs at
//! tier 1: no elevation, no capture driver, no game, no Windows.
//!
//! The Division 2 chain is the one to read first. Three of its seven processes
//! share the image name `TheDivision2.exe` and only the last holds sockets, so
//! a detector that matches on image name binds to a process that transmits
//! nothing and reports an empty session as a success. That is the failure
//! specification section 15.4 makes a validation error and section 10.3's
//! `descends_from` exists to avoid, and it is answerable only against a tree.

use fragcap_attr::{ProcessScript, ScriptedWatcher};
use fragcap_core::packet::Timestamp;
use fragcap_core::traits::ProcessWatcher;
use fragcap_core::{Ancestry, ProcessId, ProcessTree};

/// Play a script through a watcher and fold it, exactly as a live run would.
fn tree_from(script: ProcessScript) -> ProcessTree {
    let watcher = ScriptedWatcher::new(script);
    let rx = watcher.subscribe();
    let snapshot = watcher.snapshot();
    watcher.play();

    let mut tree = ProcessTree::new();
    tree.apply_snapshot(&snapshot);
    for event in rx.try_iter() {
        tree.apply(event);
    }
    tree
}

/// `explorer.exe -> steam.exe -> zosSteamStarter.exe ->
/// Bethesda.net_Launcher.exe -> eso64.exe`
///
/// Appendix D.2. The shim holds no sockets, the launcher holds sixteen flows on
/// 443, and the client holds four on 24120 to 24131.
fn eso_chain() -> ProcessScript {
    ProcessScript::new()
        .started(1000, 4, "C:\\Windows\\explorer.exe", "explorer.exe", 1)
        .started(
            1100,
            1000,
            "C:\\Program Files (x86)\\Steam\\steam.exe",
            "steam.exe",
            2,
        )
        .started(
            1200,
            1100,
            "C:\\Games\\ESO\\zosSteamStarter.exe",
            "zosSteamStarter.exe",
            3,
        )
        .started(
            1300,
            1200,
            "C:\\Games\\ESO\\Launcher\\Bethesda.net_Launcher.exe",
            "Bethesda.net_Launcher.exe",
            4,
        )
        .started(
            1400,
            1300,
            "C:\\Games\\ESO\\client\\eso64.exe",
            "eso64.exe -viewer_id 0",
            5,
        )
        // The shim hands off and terminates, which is what makes polling
        // unworkable and this whole slice necessary.
        .exited(1200, 6)
}

/// `explorer.exe -> steam.exe -> TheDivision2.exe(A) ->
/// UbisoftGameLauncher.exe -> TheDivision2.exe(B) -> EACLaunch.exe ->
/// TheDivision2.exe(C)`
///
/// Appendix D.3. Two shims and the anti-cheat launcher hold no sockets, the
/// platform service holds ten flows on 443, and the client holds thirty-one.
fn div2_chain() -> ProcessScript {
    ProcessScript::new()
        .started(2000, 4, "C:\\Windows\\explorer.exe", "explorer.exe", 1)
        .started(
            2100,
            2000,
            "C:\\Program Files (x86)\\Steam\\steam.exe",
            "steam.exe",
            2,
        )
        .started(
            2200,
            2100,
            "C:\\Games\\Div2\\TheDivision2.exe",
            "TheDivision2.exe",
            3,
        )
        .started(
            2300,
            2200,
            "C:\\Program Files\\Ubisoft\\UbisoftGameLauncher.exe",
            "UbisoftGameLauncher.exe",
            4,
        )
        .started(
            2400,
            2300,
            "C:\\Games\\Div2\\TheDivision2.exe",
            "TheDivision2.exe -uplay",
            5,
        )
        .started(
            2500,
            2400,
            "C:\\Games\\Div2\\EACLaunch.exe",
            "EACLaunch.exe",
            6,
        )
        .started(
            2600,
            2500,
            "C:\\Games\\Div2\\TheDivision2.exe",
            "TheDivision2.exe -eac",
            7,
        )
}

#[test]
fn the_eso_chain_reports_five_levels_from_the_shell_to_the_client() {
    let tree = tree_from(eso_chain());

    let client = tree
        .resolve(ProcessId(1400), Timestamp::from_nanos(10))
        .unwrap();
    let path = tree.ancestry(client);
    assert_eq!(path.len(), 5, "Appendix D.2 records five levels");

    let names: Vec<_> = path
        .iter()
        .map(|id| tree.node(*id).unwrap().image_name())
        .collect();
    assert_eq!(
        names,
        vec![
            "explorer.exe",
            "steam.exe",
            "zosSteamStarter.exe",
            "Bethesda.net_Launcher.exe",
            "eso64.exe",
        ]
    );
}

#[test]
fn the_eso_client_stays_attributable_after_the_shim_exits() {
    let tree = tree_from(eso_chain());

    let shim = tree
        .resolve(ProcessId(1200), Timestamp::from_nanos(4))
        .unwrap();
    assert!(!tree.node(shim).unwrap().is_live());

    // The transient launcher is gone and the chain is still answerable, which
    // is the property retention exists for.
    let client = tree
        .resolve(ProcessId(1400), Timestamp::from_nanos(10))
        .unwrap();
    assert!(tree.descends_from(client, shim));
    assert_eq!(tree.ancestry(client).len(), 5);
}

#[test]
fn the_div2_chain_holds_seven_nodes_and_three_share_one_image_name() {
    let tree = tree_from(div2_chain());

    let client = tree
        .resolve(ProcessId(2600), Timestamp::from_nanos(10))
        .unwrap();
    let path = tree.ancestry(client);

    // Seven, not six. Specification section 5.4's prose says six levels while
    // its own diagram and Appendix D.3's topology both list seven processes.
    // The count here follows the two that agree. Recorded as a deviation in
    // the S11 slice and promoted to section 29.
    assert_eq!(path.len(), 7);

    let sharing: Vec<_> = tree
        .nodes()
        .filter(|n| n.image_name() == "TheDivision2.exe")
        .map(|n| n.id())
        .collect();
    assert_eq!(sharing.len(), 3, "Appendix D.3: three processes, one name");
}

#[test]
fn ancestry_distinguishes_the_three_processes_sharing_one_image_name() {
    let tree = tree_from(div2_chain());

    let a = tree
        .resolve(ProcessId(2200), Timestamp::from_nanos(10))
        .unwrap();
    let b = tree
        .resolve(ProcessId(2400), Timestamp::from_nanos(10))
        .unwrap();
    let c = tree
        .resolve(ProcessId(2600), Timestamp::from_nanos(10))
        .unwrap();
    assert_ne!(a, b);
    assert_ne!(b, c);

    // The anti-cheat launcher is the distinguishing ancestor Appendix D.3
    // names, and it separates the client that holds sockets from the two that
    // do not. fragcap observes this relationship and never touches the process.
    let eac = tree
        .resolve(ProcessId(2500), Timestamp::from_nanos(10))
        .unwrap();
    assert!(tree.descends_from(c, eac));
    assert!(!tree.descends_from(a, eac));
    assert!(!tree.descends_from(b, eac));

    // The whole point: image name alone binds to the wrong one.
    let first_by_name = tree
        .nodes()
        .find(|n| n.image_name() == "TheDivision2.exe")
        .unwrap()
        .id();
    assert_eq!(first_by_name, a, "matching on name alone finds the shim");
    assert_ne!(first_by_name, c, "which is not the process holding sockets");
}

#[test]
fn every_node_in_both_chains_carries_creation_time_ancestry() {
    for tree in [tree_from(eso_chain()), tree_from(div2_chain())] {
        let mut roots = 0;
        for node in tree.nodes() {
            match node.ancestry() {
                // The shell's own parent was created before the watcher
                // started, so it resolves to nothing and says so.
                Ancestry::Unresolved => roots += 1,
                Ancestry::Observed => {}
                Ancestry::Snapshot => panic!("nothing here came from a snapshot"),
            }
        }
        assert_eq!(roots, 1, "exactly one node has no parent in the tree");
    }
}

#[test]
fn a_tree_is_the_same_whichever_valid_order_the_events_arrive_in() {
    let ordered = tree_from(div2_chain());

    // The same session, with every exit delivered before every start, which a
    // trace consumer reading from several buffers is entitled to do.
    let script = div2_chain();
    let mut reordered = ProcessTree::new();
    let events: Vec<_> = script.events().to_vec();
    for e in events
        .iter()
        .filter(|e| matches!(e, fragcap_core::ProcessEvent::Exited { .. }))
    {
        reordered.apply(e.clone());
    }
    for e in events
        .iter()
        .filter(|e| !matches!(e, fragcap_core::ProcessEvent::Exited { .. }))
    {
        reordered.apply(e.clone());
    }

    assert_eq!(ordered.len(), reordered.len());
    assert_eq!(ordered.unmatched_exits(), reordered.unmatched_exits());

    let client = reordered
        .resolve(ProcessId(2600), Timestamp::from_nanos(10))
        .unwrap();
    assert_eq!(reordered.ancestry(client).len(), 7);
}

#[test]
fn a_process_running_before_the_watcher_started_is_told_apart_from_one_observed() {
    // The shell and the platform client were already running, which is the
    // ordinary case: section 10.4 calls a platform client a service and says
    // it may predate the session.
    let script = ProcessScript::new()
        .with_snapshot(vec![
            fragcap_core::ProcessRecord::new(2000, 4, "C:\\Windows\\explorer.exe"),
            fragcap_core::ProcessRecord::new(2100, 2000, "C:\\Steam\\steam.exe"),
        ])
        .started(
            2200,
            2100,
            "C:\\Games\\Div2\\TheDivision2.exe",
            "TheDivision2.exe",
            3,
        );

    let tree = tree_from(script);
    assert_eq!(tree.len(), 3);

    let shell = tree
        .resolve(ProcessId(2000), Timestamp::from_nanos(1))
        .unwrap();
    let steam = tree
        .resolve(ProcessId(2100), Timestamp::from_nanos(1))
        .unwrap();
    let client = tree
        .resolve(ProcessId(2200), Timestamp::from_nanos(4))
        .unwrap();

    assert_eq!(tree.node(shell).unwrap().ancestry(), Ancestry::Unresolved);
    assert_eq!(tree.node(steam).unwrap().ancestry(), Ancestry::Snapshot);
    assert_eq!(tree.node(client).unwrap().ancestry(), Ancestry::Observed);

    // Neither snapshot node has a command line, and the observed one does.
    assert!(!tree.node(steam).unwrap().command_line().is_available());
    assert!(tree.node(client).unwrap().command_line().is_available());

    // The relation still spans both kinds.
    assert!(tree.descends_from(client, shell));
}

#[test]
fn a_chain_with_a_lost_event_says_the_tree_may_have_a_hole() {
    let mut tree = tree_from(eso_chain());
    assert!(tree.is_complete());

    tree.note_lost(1);

    assert!(!tree.is_complete());
    // The ancestry it did observe is still answerable. An incomplete tree is
    // not a useless one; it is one whose silence means less than it looks.
    let client = tree
        .resolve(ProcessId(1400), Timestamp::from_nanos(10))
        .unwrap();
    assert_eq!(tree.ancestry(client).len(), 5);
}
