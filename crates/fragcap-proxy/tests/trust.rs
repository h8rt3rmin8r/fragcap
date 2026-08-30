// SPDX-License-Identifier: Apache-2.0

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use fragcap_proxy::{
    CertificateStore, TrustController, TrustError, TrustMutation, TrustState, CURRENT_USER_ROOT,
};

#[derive(Clone, Default)]
struct FakeStore {
    entries: Rc<RefCell<BTreeMap<String, Vec<u8>>>>,
    wrong_store: Rc<RefCell<bool>>,
    denied: Rc<RefCell<bool>>,
}

impl CertificateStore for FakeStore {
    fn observe(&self, der: &[u8], thumbprint: &str) -> Result<TrustState, TrustError> {
        if *self.denied.borrow() {
            return Err(denied("observe"));
        }
        Ok(match self.entries.borrow().get(thumbprint) {
            Some(value) if value == der => TrustState::PresentExact,
            Some(_) => TrustState::Mismatch,
            None if *self.wrong_store.borrow() => TrustState::PresentWrongStore,
            None => TrustState::Absent,
        })
    }

    fn add_exact(&self, der: &[u8], thumbprint: &str) -> Result<TrustMutation, TrustError> {
        if *self.denied.borrow() {
            return Err(denied("add"));
        }
        match self.observe(der, thumbprint)? {
            TrustState::PresentExact => Ok(TrustMutation::AlreadyPresent),
            TrustState::Mismatch => Err(mismatch("add")),
            TrustState::Absent | TrustState::PresentWrongStore => {
                self.entries
                    .borrow_mut()
                    .insert(thumbprint.to_string(), der.to_vec());
                Ok(TrustMutation::Added)
            }
        }
    }

    fn remove_exact(&self, der: &[u8], thumbprint: &str) -> Result<TrustMutation, TrustError> {
        if *self.denied.borrow() {
            return Err(denied("remove"));
        }
        match self.observe(der, thumbprint)? {
            TrustState::PresentExact => {
                self.entries.borrow_mut().remove(thumbprint);
                Ok(TrustMutation::Removed)
            }
            TrustState::Mismatch => Err(mismatch("remove")),
            _ => Ok(TrustMutation::AlreadyAbsent),
        }
    }
}

fn denied(operation: &'static str) -> TrustError {
    TrustError {
        code: "access-denied",
        operation,
        store: CURRENT_USER_ROOT,
        os_error: Some(5),
        detail: "synthetic denial".to_string(),
    }
}

fn mismatch(operation: &'static str) -> TrustError {
    TrustError {
        code: "trust-certificate-mismatch",
        operation,
        store: CURRENT_USER_ROOT,
        os_error: None,
        detail: "synthetic mismatch".to_string(),
    }
}

#[test]
fn explicit_authorization_adds_and_cleanup_removes_only_owned_entry() {
    let store = FakeStore::default();
    store
        .entries
        .borrow_mut()
        .insert("unrelated".into(), b"other".to_vec());
    let mut controller =
        TrustController::new(store.clone(), b"certificate".to_vec(), "A".repeat(40));
    assert_eq!(
        controller.authorize_add(false).unwrap_err().code,
        "trust-authorization-required"
    );
    assert_eq!(
        controller.authorize_add(true).unwrap(),
        TrustMutation::Added
    );
    assert!(controller.owns_cleanup());
    assert_eq!(controller.cleanup().unwrap(), TrustMutation::Removed);
    assert_eq!(store.entries.borrow().get("unrelated").unwrap(), b"other");
    assert_eq!(controller.cleanup().unwrap(), TrustMutation::AlreadyAbsent);
}

#[test]
fn duplicate_wrong_store_mismatch_and_denial_are_distinct() {
    let store = FakeStore::default();
    let thumbprint = "B".repeat(40);
    store.wrong_store.replace(true);
    assert_eq!(
        store.observe(b"cert", &thumbprint).unwrap(),
        TrustState::PresentWrongStore
    );
    assert_eq!(
        store.add_exact(b"cert", &thumbprint).unwrap(),
        TrustMutation::Added
    );
    assert_eq!(
        store.add_exact(b"cert", &thumbprint).unwrap(),
        TrustMutation::AlreadyPresent
    );
    assert_eq!(
        store.observe(b"different", &thumbprint).unwrap(),
        TrustState::Mismatch
    );
    assert_eq!(
        store
            .remove_exact(b"different", &thumbprint)
            .unwrap_err()
            .code,
        "trust-certificate-mismatch"
    );
    store.denied.replace(true);
    assert_eq!(
        store.observe(b"cert", &thumbprint).unwrap_err().os_error,
        Some(5)
    );
}

#[test]
fn failed_cleanup_preserves_discoverable_obligation() {
    let store = FakeStore::default();
    let mut controller = TrustController::new(store.clone(), b"cert".to_vec(), "C".repeat(40));
    controller.authorize_add(true).unwrap();
    store.denied.replace(true);
    assert!(controller.cleanup().is_err());
    assert!(controller.owns_cleanup());
}
