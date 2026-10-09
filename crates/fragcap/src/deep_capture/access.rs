// SPDX-License-Identifier: Apache-2.0

//! Exact retained-output recipients and actual filesystem access verification.

use serde_json::{json, Value};
use std::fs::File;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::time::Duration;

#[cfg(windows)]
mod windows;

#[derive(Clone, Debug)]
pub struct OutputRecipient {
    sid: String,
    producer_sid: String,
    proof_kind: String,
    #[cfg(windows)]
    platform: std::sync::Arc<windows::RecipientToken>,
}

impl OutputRecipient {
    /// Controlled group-denied equivalent for repository-owned ACL fixtures.
    #[cfg(windows)]
    #[doc(hidden)]
    pub fn group_denied_fixture() -> io::Result<Self> {
        windows::group_denied_fixture()
    }
    /// Explicit current-account contract for library and controlled callers.
    pub fn current_account() -> io::Result<Self> {
        #[cfg(windows)]
        {
            windows::recipient(false)
        }
        #[cfg(not(windows))]
        {
            Ok(Self {
                sid: "current-posix-user".into(),
                producer_sid: "current-posix-user".into(),
                proof_kind: "current-platform-account".into(),
            })
        }
    }
    /// Resolve the exact Windows desktop session account before session effects.
    pub fn desktop_session() -> io::Result<Self> {
        #[cfg(windows)]
        {
            windows::recipient(true)
        }
        #[cfg(not(windows))]
        {
            Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "Windows desktop recipient proof is unavailable",
            ))
        }
    }
    pub fn sid(&self) -> &str {
        &self.sid
    }
    pub fn producer_sid(&self) -> &str {
        &self.producer_sid
    }
    pub fn proof_kind(&self) -> &str {
        &self.proof_kind
    }
    pub fn revalidate(&self) -> io::Result<()> {
        #[cfg(windows)]
        {
            windows::revalidate(self)
        }
        #[cfg(not(windows))]
        {
            Ok(())
        }
    }
    /// Read bounded synthetic fixture bytes in the retained ordinary context.
    #[doc(hidden)]
    pub fn read_fixture(&self, path: &Path) -> io::Result<Vec<u8>> {
        #[cfg(windows)]
        {
            windows::fixture_read(self, path)
        }
        #[cfg(not(windows))]
        {
            std::fs::read(path)
        }
    }
    /// Enumerate synthetic fixture entries in the retained ordinary context.
    #[doc(hidden)]
    pub fn enumerate_fixture(&self, path: &Path) -> io::Result<Vec<PathBuf>> {
        #[cfg(windows)]
        {
            windows::fixture_enumerate(self, path)
        }
        #[cfg(not(windows))]
        {
            std::fs::read_dir(path)?
                .map(|e| e.map(|v| v.path()))
                .collect()
        }
    }
    pub fn verify_paths(
        &self,
        root: &Path,
        paths: &[PathBuf],
    ) -> io::Result<BundleAccessVerification> {
        self.revalidate()?;
        #[cfg(windows)]
        {
            windows::verify(self, root, paths)
        }
        #[cfg(not(windows))]
        {
            Ok(verify_current(root, paths, self.sid(), self.proof_kind()))
        }
    }
}

#[derive(Clone, Debug)]
pub struct AccessPathVerification {
    pub path: PathBuf,
    pub state: String,
    pub reason: String,
}

#[derive(Clone, Debug)]
pub struct BundleAccessVerification {
    pub recipient_sid: String,
    pub proof_kind: String,
    pub state: String,
    pub paths: Vec<AccessPathVerification>,
    pub limitations: Vec<String>,
}
impl BundleAccessVerification {
    pub fn is_verified(&self) -> bool {
        self.state == "verified"
    }
    pub fn as_json(&self) -> Value {
        json!({"version":1,"recipient_sid":self.recipient_sid,"proof_kind":self.proof_kind,"state":self.state,"paths":self.paths.iter().map(|p|json!({"path":p.path,"state":p.state,"reason":p.reason})).collect::<Vec<_>>(),"limitations":self.limitations})
    }
}

pub fn verify_bundle_access(
    root: &Path,
    recipient: &OutputRecipient,
) -> io::Result<BundleAccessVerification> {
    let mut paths = Vec::new();
    inventory(root, root, 0, &mut paths)?;
    #[cfg(windows)]
    {
        let absolute = if root.is_absolute() {
            root.to_path_buf()
        } else {
            std::env::current_dir()?.join(root)
        };
        if let Some(parent) = absolute.parent() {
            paths.push(parent.to_path_buf());
        }
    }
    recipient.verify_paths(root, &paths)
}
fn inventory(root: &Path, path: &Path, depth: usize, output: &mut Vec<PathBuf>) -> io::Result<()> {
    if depth > 8 || output.len() >= 256 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "bundle access inventory exceeds bounds",
        ));
    }
    let meta = std::fs::symlink_metadata(path)?;
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if meta.file_attributes() & 0x400 != 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "bundle access refuses a reparse point",
            ));
        }
    }
    if meta.file_type().is_symlink() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "bundle access refuses a linked path",
        ));
    }
    output.push(path.strip_prefix(root).unwrap_or(path).to_path_buf());
    if meta.is_dir() {
        for entry in std::fs::read_dir(path)? {
            inventory(root, &entry?.path(), depth + 1, output)?;
        }
    }
    Ok(())
}
fn verify_current(
    root: &Path,
    paths: &[PathBuf],
    sid: &str,
    proof: &str,
) -> BundleAccessVerification {
    let outcomes = paths
        .iter()
        .map(|relative| {
            let path = if relative.is_absolute() {
                relative.clone()
            } else {
                root.join(relative)
            };
            let result = (|| -> io::Result<()> {
                let meta = std::fs::symlink_metadata(&path)?;
                if meta.is_dir() {
                    let _: Vec<_> = std::fs::read_dir(&path)?.collect::<io::Result<_>>()?;
                } else if meta.is_file() {
                    let mut probe = [0_u8; 1];
                    let _count = File::open(&path)?.read(&mut probe)?;
                } else {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "access verification requires a regular object",
                    ));
                }
                Ok(())
            })();
            AccessPathVerification {
                path: relative.clone(),
                state: if result.is_ok() {
                    "verified"
                } else {
                    "unresolved"
                }
                .into(),
                reason: result.err().map_or_else(
                    || "actual enumeration or read-open and bounded read succeeded".into(),
                    |e| e.to_string(),
                ),
            }
        })
        .collect::<Vec<_>>();
    BundleAccessVerification { recipient_sid:sid.into(),proof_kind:proof.into(),state:if outcomes.iter().all(|v|v.state=="verified"){"verified"}else{"unresolved"}.into(),paths:outcomes,limitations:vec!["Read verification probes at most one byte per file; payload contents are never emitted.".into()] }
}

pub struct RecipientHandoff {
    request_id: String,
    #[cfg(windows)]
    platform: windows::Handoff,
}
impl RecipientHandoff {
    pub fn new(sid: &str, bundle: &Path) -> io::Result<Self> {
        #[cfg(windows)]
        {
            windows::handoff(sid, bundle)
        }
        #[cfg(not(windows))]
        {
            let _ = (sid, bundle);
            Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "Windows recipient handoff is unavailable",
            ))
        }
    }
    pub fn request_id(&self) -> &str {
        &self.request_id
    }
    pub fn accept(self, timeout: Duration) -> io::Result<OutputRecipient> {
        #[cfg(windows)]
        {
            windows::accept(self.platform, timeout)
        }
        #[cfg(not(windows))]
        {
            let _ = timeout;
            Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "Windows recipient handoff is unavailable",
            ))
        }
    }
}
pub fn authorize_output_recipient(
    request: &str,
    bundle: &Path,
    timeout: Duration,
) -> io::Result<()> {
    #[cfg(windows)]
    {
        windows::authorize(request, bundle, timeout)
    }
    #[cfg(not(windows))]
    {
        let _ = (request, bundle, timeout);
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "Windows recipient handoff is unavailable",
        ))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AccessObjectIdentity {
    pub volume_serial: u64,
    pub file_index: u64,
    pub links: u32,
    pub directory: bool,
}
pub(crate) struct PinnedAccessPath {
    pub path: PathBuf,
    pub identity: AccessObjectIdentity,
    pub descriptor: String,
    file: File,
    #[cfg(windows)]
    _ancestors: Vec<File>,
}
pub(crate) fn pin_access_path(path: &Path, write_dacl: bool) -> io::Result<PinnedAccessPath> {
    #[cfg(windows)]
    {
        windows::pin(path, write_dacl)
    }
    #[cfg(not(windows))]
    {
        let _ = (path, write_dacl);
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "Windows pinned permissions are unavailable",
        ))
    }
}
impl PinnedAccessPath {
    pub(crate) fn refresh(&self) -> io::Result<(AccessObjectIdentity, String)> {
        #[cfg(windows)]
        {
            windows::snapshot(&self.file)
        }
        #[cfg(not(windows))]
        {
            let _ = &self.file;
            Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "Windows pinned permissions are unavailable",
            ))
        }
    }
    pub(crate) fn apply(&self, recipient: &OutputRecipient) -> io::Result<()> {
        if self.refresh()? != (self.identity.clone(), self.descriptor.clone()) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "pinned output identity or descriptor changed before permission application",
            ));
        }
        #[cfg(windows)]
        {
            windows::apply(
                &self.file,
                self.identity.directory,
                &windows::contract(recipient),
            )
        }
        #[cfg(not(windows))]
        {
            let _ = recipient;
            Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "Windows pinned permissions are unavailable",
            ))
        }
    }
    pub(crate) fn apply_traversal(&self, recipient: &OutputRecipient) -> io::Result<()> {
        if self.refresh()? != (self.identity.clone(), self.descriptor.clone()) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "pinned traversal identity or descriptor changed before permission application",
            ));
        }
        #[cfg(windows)]
        {
            windows::traversal(&self.file, recipient)
        }
        #[cfg(not(windows))]
        {
            let _ = recipient;
            Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "Windows pinned permissions are unavailable",
            ))
        }
    }
}

#[cfg(windows)]
pub(crate) fn protect_recipient_directory(
    path: &Path,
    recipient: &OutputRecipient,
) -> io::Result<()> {
    windows::protect_directory(path, recipient)
}
#[cfg(windows)]
pub(crate) fn preserve_private_directory(path: &Path) -> io::Result<bool> {
    windows::preserve_directory(path)
}
#[cfg(windows)]
pub(crate) fn open_private_file(path: &Path) -> io::Result<File> {
    windows::create_private_file(path)
}
#[cfg(windows)]
pub(crate) fn protect_private_path(path: &Path, directory: bool) -> io::Result<()> {
    windows::protect_child(path, directory)
}

/// Controlled synthetic-fixture helper, never used by production session resolution.
#[cfg(windows)]
#[doc(hidden)]
pub fn protect_denied_user_fixture_path(path: &Path, directory: bool) -> io::Result<()> {
    windows::denied_user_fixture(path, directory)
}
#[cfg(windows)]
#[doc(hidden)]
pub fn verify_denied_user_fixture_access(
    root: &Path,
    paths: &[PathBuf],
) -> io::Result<BundleAccessVerification> {
    windows::verify_denied_user(root, paths)
}
#[cfg(windows)]
#[doc(hidden)]
pub fn current_fixture_logon_sid() -> io::Result<String> {
    windows::fixture_logon_sid()
}
