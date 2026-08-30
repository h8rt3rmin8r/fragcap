// SPDX-License-Identifier: Apache-2.0

use std::fmt;

pub const CURRENT_USER_ROOT: &str = "CurrentUser/Root";
pub const LOCAL_MACHINE_ROOT: &str = "LocalMachine/Root";

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TrustState {
    Absent,
    PresentExact,
    PresentWrongStore,
    Mismatch,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TrustMutation {
    Added,
    AlreadyPresent,
    Removed,
    AlreadyAbsent,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrustError {
    pub code: &'static str,
    pub operation: &'static str,
    pub store: &'static str,
    pub os_error: Option<u32>,
    pub detail: String,
}

impl TrustError {
    pub(crate) fn new(
        code: &'static str,
        operation: &'static str,
        store: &'static str,
        os_error: Option<u32>,
        detail: impl Into<String>,
    ) -> Self {
        Self {
            code,
            operation,
            store,
            os_error,
            detail: detail.into(),
        }
    }
}
impl fmt::Display for TrustError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.code, self.detail)
    }
}
impl std::error::Error for TrustError {}

pub trait CertificateStore {
    fn observe(&self, der: &[u8], thumbprint: &str) -> Result<TrustState, TrustError>;
    fn add_exact(&self, der: &[u8], thumbprint: &str) -> Result<TrustMutation, TrustError>;
    fn remove_exact(&self, der: &[u8], thumbprint: &str) -> Result<TrustMutation, TrustError>;
}

pub struct TrustController<S> {
    store: S,
    der: Vec<u8>,
    thumbprint: String,
    owned: bool,
}

impl<S: CertificateStore> TrustController<S> {
    pub fn new(store: S, der: Vec<u8>, thumbprint: impl Into<String>) -> Self {
        Self {
            store,
            der,
            thumbprint: thumbprint.into(),
            owned: false,
        }
    }

    pub fn observe(&self) -> Result<TrustState, TrustError> {
        self.store.observe(&self.der, &self.thumbprint)
    }

    pub fn authorize_add(&mut self, confirmed: bool) -> Result<TrustMutation, TrustError> {
        if !confirmed {
            return Err(TrustError::new(
                "trust-authorization-required",
                "add",
                CURRENT_USER_ROOT,
                None,
                "explicit operator authorization is required",
            ));
        }
        let mutation = self.store.add_exact(&self.der, &self.thumbprint)?;
        self.owned = mutation == TrustMutation::Added;
        Ok(mutation)
    }

    pub fn cleanup(&mut self) -> Result<TrustMutation, TrustError> {
        if !self.owned {
            return Ok(TrustMutation::AlreadyAbsent);
        }
        let mutation = self.store.remove_exact(&self.der, &self.thumbprint)?;
        self.owned = false;
        Ok(mutation)
    }

    pub fn owns_cleanup(&self) -> bool {
        self.owned
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct NativeCertificateStore;

impl NativeCertificateStore {
    /// Remove a legacy owned entry identified by its recorded thumbprint.
    ///
    /// New sessions use [`CertificateStore::remove_exact`] with certificate
    /// bytes as well. This path exists for doctor cleanup of older manifests.
    pub fn remove_current_user_thumbprint(
        &self,
        thumbprint: &str,
    ) -> Result<TrustMutation, TrustError> {
        native_remove_thumbprint(thumbprint)
    }
}

impl CertificateStore for NativeCertificateStore {
    fn observe(&self, der: &[u8], thumbprint: &str) -> Result<TrustState, TrustError> {
        native_observe(der, thumbprint)
    }
    fn add_exact(&self, der: &[u8], thumbprint: &str) -> Result<TrustMutation, TrustError> {
        native_add(der, thumbprint)
    }
    fn remove_exact(&self, der: &[u8], thumbprint: &str) -> Result<TrustMutation, TrustError> {
        native_remove(der, thumbprint)
    }
}

#[cfg(windows)]
fn native_observe(der: &[u8], thumbprint: &str) -> Result<TrustState, TrustError> {
    crate::windows::trust::observe(der, thumbprint)
}
#[cfg(windows)]
fn native_add(der: &[u8], thumbprint: &str) -> Result<TrustMutation, TrustError> {
    crate::windows::trust::add(der, thumbprint)
}
#[cfg(windows)]
fn native_remove(der: &[u8], thumbprint: &str) -> Result<TrustMutation, TrustError> {
    crate::windows::trust::remove(der, thumbprint)
}
#[cfg(windows)]
fn native_remove_thumbprint(thumbprint: &str) -> Result<TrustMutation, TrustError> {
    crate::windows::trust::remove_thumbprint(thumbprint)
}

#[cfg(not(windows))]
fn unavailable(operation: &'static str) -> TrustError {
    TrustError::new(
        "trust-platform-unavailable",
        operation,
        CURRENT_USER_ROOT,
        None,
        "the native certificate store is available only on Windows",
    )
}
#[cfg(not(windows))]
fn native_observe(_: &[u8], _: &str) -> Result<TrustState, TrustError> {
    Err(unavailable("observe"))
}
#[cfg(not(windows))]
fn native_add(_: &[u8], _: &str) -> Result<TrustMutation, TrustError> {
    Err(unavailable("add"))
}
#[cfg(not(windows))]
fn native_remove(_: &[u8], _: &str) -> Result<TrustMutation, TrustError> {
    Err(unavailable("remove"))
}
#[cfg(not(windows))]
fn native_remove_thumbprint(_: &str) -> Result<TrustMutation, TrustError> {
    Err(unavailable("remove"))
}
