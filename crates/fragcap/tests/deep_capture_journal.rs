// SPDX-License-Identifier: Apache-2.0

#![cfg(feature = "deep-capture")]

use std::io::Write;

use fragcap::deep_capture::{
    read_resource_journal, recover_resource_journal, JournalStatus, ResourceJournal, ResourceKind,
    ResourceState, ResourceTransition,
};

fn transition(id: &str, state: ResourceState) -> ResourceTransition {
    ResourceTransition::new(
        id,
        ResourceKind::Trust,
        "sha1:0123456789abcdef0123456789abcdef01234567",
        "session:owned",
        "remove-current-user-root",
        state,
        "test transition",
    )
}

#[test]
fn synchronized_prefix_is_recoverable_before_the_effect_result() {
    let temp = tempfile::tempdir().unwrap();
    let mut journal = ResourceJournal::create(temp.path(), "session", "plan").unwrap();
    journal
        .append(transition("trust", ResourceState::Pending))
        .unwrap();

    let prefix = read_resource_journal(journal.path()).unwrap();
    assert_eq!(prefix.status, JournalStatus::CrashPrefix);
    assert_eq!(prefix.transitions.len(), 1);
    assert_eq!(prefix.recovery_plan().actions.len(), 1);
    assert!(prefix.recovery_plan().refusals.is_empty());
}

#[test]
fn applied_owned_effect_produces_one_exact_idempotent_recovery_action() {
    let temp = tempfile::tempdir().unwrap();
    let mut journal = ResourceJournal::create(temp.path(), "session", "plan").unwrap();
    journal
        .append(transition("trust", ResourceState::Pending))
        .unwrap();
    journal
        .append(transition("trust", ResourceState::Applied))
        .unwrap();

    let recovery = read_resource_journal(journal.path())
        .unwrap()
        .recovery_plan();
    assert_eq!(recovery.actions.len(), 1);
    assert_eq!(
        recovery.actions[0].target,
        "sha1:0123456789abcdef0123456789abcdef01234567"
    );
    assert!(recovery.refusals.is_empty());
}

#[test]
fn completed_resource_and_completed_journal_need_no_recovery() {
    let temp = tempfile::tempdir().unwrap();
    let mut journal = ResourceJournal::create(temp.path(), "session", "plan").unwrap();
    for state in [
        ResourceState::Pending,
        ResourceState::Applied,
        ResourceState::CleanupPending,
        ResourceState::Released,
    ] {
        journal.append(transition("trust", state)).unwrap();
    }
    journal.finish().unwrap();

    let prefix = read_resource_journal(journal.path()).unwrap();
    assert_eq!(prefix.status, JournalStatus::Complete);
    assert!(prefix.recovery_plan().actions.is_empty());
}

#[test]
fn recovery_terminalizes_a_header_only_crash_prefix() {
    let temp = tempfile::tempdir().unwrap();
    let path = {
        let journal = ResourceJournal::create(temp.path(), "session", "plan").unwrap();
        journal.path().to_path_buf()
    };

    let before = read_resource_journal(&path).unwrap();
    assert_eq!(before.status, JournalStatus::CrashPrefix);
    assert!(before.transitions.is_empty());

    let plan = recover_resource_journal(&path, |_| unreachable!("no resource action")).unwrap();

    assert!(plan.actions.is_empty());
    assert!(plan.refusals.is_empty());
    assert_eq!(
        read_resource_journal(&path).unwrap().status,
        JournalStatus::Complete
    );
}

#[test]
fn corrupt_and_noncontiguous_records_fail_closed() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("resource-journal.jsonl");
    std::fs::write(
        &path,
        b"{\"type\":\"resource-journal.header\",\"schema_version\":1,\"session_id\":\"s\",\"plan_id\":\"p\"}\n{broken\n",
    )
    .unwrap();
    assert_eq!(
        read_resource_journal(&path).unwrap_err().kind(),
        std::io::ErrorKind::InvalidData
    );
}

#[test]
fn a_resource_identity_cannot_change_after_its_obligation() {
    let temp = tempfile::tempdir().unwrap();
    let mut journal = ResourceJournal::create(temp.path(), "session", "plan").unwrap();
    journal
        .append(transition("trust", ResourceState::Pending))
        .unwrap();
    let mut changed = transition("trust", ResourceState::Applied);
    changed.target = "sha1:ffffffffffffffffffffffffffffffffffffffff".into();
    assert_eq!(
        journal.append(changed).unwrap_err().kind(),
        std::io::ErrorKind::InvalidData
    );
}

#[test]
fn a_torn_final_record_preserves_the_synchronized_prefix() {
    let temp = tempfile::tempdir().unwrap();
    let mut journal = ResourceJournal::create(temp.path(), "session", "plan").unwrap();
    journal
        .append(transition("trust", ResourceState::Pending))
        .unwrap();
    std::fs::OpenOptions::new()
        .append(true)
        .open(journal.path())
        .unwrap()
        .write_all(b"{\"type\":\"resource.transition\"")
        .unwrap();
    let prefix = read_resource_journal(journal.path()).unwrap();
    assert_eq!(prefix.status, JournalStatus::CrashPrefix);
    assert_eq!(prefix.transitions.len(), 1);
}

#[test]
fn a_complete_journal_with_an_unresolved_effect_can_resume_recovery() {
    let temp = tempfile::tempdir().unwrap();
    let path = {
        let mut journal = ResourceJournal::create(temp.path(), "session", "plan").unwrap();
        journal
            .append(transition("trust", ResourceState::Pending))
            .unwrap();
        journal
            .append(transition("trust", ResourceState::Applied))
            .unwrap();
        journal.finish().unwrap();
        journal.path().to_path_buf()
    };
    let plan = recover_resource_journal(&path, |_| Ok("removed exact trust entry".into())).unwrap();
    assert_eq!(plan.actions.len(), 1);
    let prefix = read_resource_journal(&path).unwrap();
    assert_eq!(prefix.status, JournalStatus::Complete);
    assert_eq!(
        prefix.transitions.last().unwrap().state,
        ResourceState::Released
    );
}

#[test]
fn completed_journal_compacts_to_latest_audit_state() {
    let temp = tempfile::tempdir().unwrap();
    let mut journal = ResourceJournal::create(temp.path(), "session", "plan").unwrap();
    for state in [
        ResourceState::Pending,
        ResourceState::Applied,
        ResourceState::CleanupPending,
        ResourceState::Released,
    ] {
        journal.append(transition("trust", state)).unwrap();
    }
    journal.compact().unwrap();
    let prefix = read_resource_journal(journal.path()).unwrap();
    assert_eq!(prefix.status, JournalStatus::Complete);
    assert_eq!(prefix.transitions.len(), 1);
    assert_eq!(prefix.transitions[0].state, ResourceState::Released);
}

#[test]
fn uncertain_and_failed_prefixes_produce_only_exact_recovery_decisions() {
    for (state, expected_actions, expected_refusals) in [
        (ResourceState::Pending, 1, 0),
        (ResourceState::Applied, 1, 0),
        (ResourceState::CleanupPending, 1, 0),
        (ResourceState::Failed, 1, 0),
        (ResourceState::TimedOut, 1, 0),
        (ResourceState::NotApplied, 0, 0),
        (ResourceState::Released, 0, 0),
    ] {
        let temp = tempfile::tempdir().unwrap();
        let mut journal = ResourceJournal::create(temp.path(), "session", "plan").unwrap();
        journal.append(transition("trust", state)).unwrap();
        let recovery = read_resource_journal(journal.path())
            .unwrap()
            .recovery_plan();
        assert_eq!(recovery.actions.len(), expected_actions, "{state:?}");
        assert_eq!(recovery.refusals.len(), expected_refusals, "{state:?}");
        for decision in recovery.actions {
            assert_eq!(
                decision.target,
                "sha1:0123456789abcdef0123456789abcdef01234567"
            );
            assert_eq!(decision.action, "remove-current-user-root");
        }
        for refusal in recovery.refusals {
            assert_eq!(refusal.resource_id, "trust");
            assert!(!refusal.reason.is_empty());
        }
    }

    let temp = tempfile::tempdir().unwrap();
    let mut journal = ResourceJournal::create(temp.path(), "session", "plan").unwrap();
    let mut inexact = transition("trust", ResourceState::Failed);
    inexact.target = "current-user-root-without-thumbprint".into();
    journal.append(inexact).unwrap();
    let recovery = read_resource_journal(journal.path())
        .unwrap()
        .recovery_plan();
    assert!(recovery.actions.is_empty());
    assert_eq!(recovery.refusals.len(), 1);
    assert_eq!(recovery.refusals[0].resource_id, "trust");
}
