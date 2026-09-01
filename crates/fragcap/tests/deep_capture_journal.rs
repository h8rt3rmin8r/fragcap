// SPDX-License-Identifier: Apache-2.0

#![cfg(feature = "deep-capture")]

use fragcap::deep_capture::{
    read_resource_journal, JournalStatus, ResourceJournal, ResourceKind, ResourceState,
    ResourceTransition,
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
    assert!(prefix.recovery_plan().actions.is_empty());
    assert_eq!(prefix.recovery_plan().refusals.len(), 1);
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
