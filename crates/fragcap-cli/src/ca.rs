// SPDX-License-Identifier: Apache-2.0

//! Installation-owned Windows certificate authority. Public status is read-only;
//! the signer is opened only by elevated setup and Deep Capture preflight.

use std::path::{Path, PathBuf};
#[cfg(any(windows, test))]
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(any(windows, test))]
use fragcap::deep_capture::api::NativeProxyAdapter;
#[cfg(windows)]
use fragcap::deep_capture::api::TrustMutation;
use fragcap::deep_capture::api::{NativeCertificateStore, PreparedNativeAuthority, TrustState};
use serde_json::{json, Value};

use crate::cli::CaCommand;
use crate::CliError;
use crate::Exit;
use std::io::Write;

const KEY_NAME: &str = "signer.dpapi";
const IDENTITY_NAME: &str = "ca.json";
const CUTOVER_NAME: &str = "cutover-v1.json";
#[cfg(any(windows, test))]
const MSI_RECEIPT_NAME: &str = "msi-ca-creation-v1.json";
#[cfg(any(windows, test))]
const CA_LIFETIME: Duration = Duration::from_secs(10 * 365 * 24 * 60 * 60);

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Health {
    pub state: &'static str,
    pub detail: String,
    pub fingerprint: Option<String>,
}

impl Health {
    fn new(state: &'static str, detail: impl Into<String>, fingerprint: Option<String>) -> Self {
        Self {
            state,
            detail: detail.into(),
            fingerprint,
        }
    }

    pub fn ready(&self) -> bool {
        self.state == "ready"
    }

    pub fn guidance(&self) -> String {
        format!(
            "installation CA {} in LocalMachine/Root: {}; run elevated `fragcap ca repair`",
            self.state, self.detail
        )
    }
}

#[derive(Clone, Debug)]
struct Identity {
    #[cfg_attr(not(windows), allow(dead_code))]
    generation: u64,
    der: Vec<u8>,
    fingerprint: String,
    thumbprint: String,
    not_before: u64,
    not_after: u64,
    #[cfg_attr(not(windows), allow(dead_code))]
    provenance: String,
}

impl Identity {
    #[cfg(any(windows, test))]
    fn from_authority(
        authority: &PreparedNativeAuthority,
        provenance: String,
    ) -> Result<Self, CliError> {
        Ok(Self {
            generation: authority.generation(),
            der: authority.der().to_vec(),
            fingerprint: authority.sha256_fingerprint().to_string(),
            thumbprint: authority.sha1_thumbprint().to_string(),
            not_before: seconds(authority.not_before())?,
            not_after: seconds(authority.not_after())?,
            provenance,
        })
    }

    fn read(path: &Path) -> Result<Self, CliError> {
        let bytes = std::fs::read(path).map_err(|error| {
            CliError::failure(format!("installation identity unreadable: {error}"))
        })?;
        if bytes.len() > 1024 * 1024 {
            return Err(CliError::failure("installation identity oversized"));
        }
        let value: Value = serde_json::from_slice(&bytes).map_err(|error| {
            CliError::failure(format!("installation identity invalid: {error}"))
        })?;
        if value["scheme"].as_u64() != Some(1) {
            return Err(CliError::failure("installation identity scheme mismatch"));
        }
        let text = |name: &str| -> Result<String, CliError> {
            value[name]
                .as_str()
                .map(str::to_string)
                .ok_or_else(|| CliError::failure(format!("installation identity missing {name}")))
        };
        let der = decode_hex(&text("der")?)?;
        let fingerprint = text("sha256")?;
        let thumbprint = text("sha1")?;
        if digest_hex(&ring::digest::SHA256, &der) != fingerprint
            || digest_hex(&ring::digest::SHA1_FOR_LEGACY_USE_ONLY, &der) != thumbprint
        {
            return Err(CliError::failure(
                "installation identity fingerprint mismatch",
            ));
        }
        let (remaining, certificate) =
            x509_parser::parse_x509_certificate(&der).map_err(|error| {
                CliError::failure(format!("installation certificate invalid: {error}"))
            })?;
        if !remaining.is_empty()
            || !certificate.is_ca()
            || certificate.subject() != certificate.issuer()
            || certificate.verify_signature(None).is_err()
        {
            return Err(CliError::failure(
                "installation certificate is not one exact CA",
            ));
        }
        let actual_not_before = u64::try_from(certificate.validity().not_before.timestamp())
            .map_err(|_| CliError::failure("installation certificate starts before Unix epoch"))?;
        let actual_not_after = u64::try_from(certificate.validity().not_after.timestamp())
            .map_err(|_| CliError::failure("installation certificate ends before Unix epoch"))?;
        let not_before = value["not_before"]
            .as_u64()
            .ok_or_else(|| CliError::failure("installation identity missing not_before"))?;
        let not_after = value["not_after"]
            .as_u64()
            .ok_or_else(|| CliError::failure("installation identity missing not_after"))?;
        if not_before != actual_not_before
            || not_after != actual_not_after
            || not_after <= not_before
        {
            return Err(CliError::failure("installation identity validity mismatch"));
        }
        Ok(Self {
            generation: value["generation"]
                .as_u64()
                .ok_or_else(|| CliError::failure("installation identity missing generation"))?,
            der,
            fingerprint,
            thumbprint,
            not_before,
            not_after,
            provenance: text("provenance")?,
        })
    }

    #[cfg(any(windows, test))]
    fn write_new(&self, path: &Path) -> Result<(), CliError> {
        use std::io::Write;
        let value = json!({
            "scheme": 1, "generation": self.generation, "der": encode_hex(&self.der),
            "sha256": self.fingerprint, "sha1": self.thumbprint,
            "not_before": self.not_before, "not_after": self.not_after,
            "provenance": self.provenance,
        });
        let bytes = serde_json::to_vec_pretty(&value)
            .map_err(|error| CliError::failure(error.to_string()))?;
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .map_err(|error| {
                CliError::failure(format!("installation identity create failed: {error}"))
            })?;
        file.write_all(&bytes)
            .and_then(|()| file.sync_all())
            .map_err(|error| {
                CliError::failure(format!("installation identity write failed: {error}"))
            })
    }
}

fn seconds(time: SystemTime) -> Result<u64, CliError> {
    time.duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .map_err(|_| CliError::failure("installation CA validity precedes Unix epoch"))
}

fn digest_hex(algorithm: &'static ring::digest::Algorithm, bytes: &[u8]) -> String {
    ring::digest::digest(algorithm, bytes)
        .as_ref()
        .iter()
        .map(|byte| format!("{byte:02X}"))
        .collect()
}

#[cfg(any(windows, test))]
fn encode_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02X}")).collect()
}

fn decode_hex(text: &str) -> Result<Vec<u8>, CliError> {
    if !text.len().is_multiple_of(2)
        || text.len() > 1024 * 1024
        || !text.bytes().all(|byte| byte.is_ascii_hexdigit())
    {
        return Err(CliError::failure("installation DER hex invalid"));
    }
    (0..text.len())
        .step_by(2)
        .map(|index| {
            u8::from_str_radix(&text[index..index + 2], 16)
                .map_err(|_| CliError::failure("installation DER hex invalid"))
        })
        .collect()
}

#[cfg(windows)]
fn root() -> Result<PathBuf, CliError> {
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;
    use windows_sys::Win32::System::Com::CoTaskMemFree;
    use windows_sys::Win32::UI::Shell::{FOLDERID_ProgramData, SHGetKnownFolderPath};
    let mut pointer = std::ptr::null_mut();
    let result = unsafe { SHGetKnownFolderPath(&FOLDERID_ProgramData, 0, 0, &mut pointer) };
    if result < 0 || pointer.is_null() {
        return Err(CliError::failure(format!(
            "ProgramData lookup failed: 0x{:08X}",
            result as u32
        )));
    }
    let mut length = 0;
    while unsafe { *pointer.add(length) } != 0 {
        length += 1;
    }
    let path = PathBuf::from(OsString::from_wide(unsafe {
        std::slice::from_raw_parts(pointer, length)
    }))
    .join("fragcap");
    unsafe { CoTaskMemFree(pointer.cast()) };
    Ok(path)
}

#[cfg(not(windows))]
fn root() -> Result<PathBuf, CliError> {
    Err(CliError::failure("installation CA requires Windows"))
}

fn identity_path(root: &Path) -> PathBuf {
    root.join(IDENTITY_NAME)
}
fn key_path(root: &Path) -> PathBuf {
    root.join("pki").join(KEY_NAME)
}
fn cutover_path(root: &Path) -> PathBuf {
    root.join(CUTOVER_NAME)
}

#[cfg(any(windows, test))]
fn msi_receipt_path(root: &Path) -> PathBuf {
    root.join(MSI_RECEIPT_NAME)
}

#[cfg(any(windows, test))]
fn validate_msi_transaction(value: &str) -> Result<(), CliError> {
    if value.len() > 128
        || value.is_empty()
        || !value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '{' | '}' | '-'))
    {
        return Err(CliError::usage("invalid MSI transaction identity"));
    }
    Ok(())
}

#[cfg(any(windows, test))]
fn read_msi_receipt(root: &Path) -> Result<Option<(String, String)>, CliError> {
    let path = msi_receipt_path(root);
    if !ordinary_file_presence(&path).map_err(CliError::failure)? {
        return Ok(None);
    }
    let bytes = std::fs::read(&path)
        .map_err(|error| CliError::failure(format!("MSI CA receipt unreadable: {error}")))?;
    if bytes.len() > 4096 {
        return Err(CliError::failure("MSI CA receipt oversized"));
    }
    let value: Value = serde_json::from_slice(&bytes)
        .map_err(|error| CliError::failure(format!("MSI CA receipt invalid: {error}")))?;
    if value["scheme"].as_u64() != Some(1) {
        return Err(CliError::failure("MSI CA receipt scheme mismatch"));
    }
    let transaction = value["transaction"]
        .as_str()
        .ok_or_else(|| CliError::failure("MSI CA receipt transaction missing"))?;
    validate_msi_transaction(transaction)?;
    let fingerprint = value["fingerprint"]
        .as_str()
        .ok_or_else(|| CliError::failure("MSI CA receipt fingerprint missing"))?;
    if fingerprint.len() != 64 || !fingerprint.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(CliError::failure("MSI CA receipt fingerprint invalid"));
    }
    Ok(Some((transaction.to_string(), fingerprint.to_string())))
}

#[cfg(any(windows, test))]
fn rollback_owns_identity(
    receipt: Option<&(String, String)>,
    transaction: &str,
    identity: &Identity,
) -> bool {
    receipt.is_some_and(|(created_by, fingerprint)| {
        created_by == transaction
            && fingerprint == &identity.fingerprint
            && identity.provenance == format!("fragcap S185 MSI {transaction}")
    })
}

#[cfg(windows)]
fn write_msi_receipt(root: &Path, transaction: &str, fingerprint: &str) -> Result<(), CliError> {
    let path = msi_receipt_path(root);
    let bytes = serde_json::to_vec(
        &json!({"scheme":1,"transaction":transaction,"fingerprint":fingerprint}),
    )
    .map_err(|error| CliError::failure(error.to_string()))?;
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .map_err(|error| CliError::failure(format!("MSI CA receipt create failed: {error}")))?;
    if let Err(error) = file.write_all(&bytes).and_then(|()| file.sync_all()) {
        let _ = std::fs::remove_file(&path);
        return Err(CliError::failure(format!(
            "MSI CA receipt write failed: {error}"
        )));
    }
    Ok(())
}

pub(crate) fn health() -> Health {
    let root = match root() {
        Ok(root) => root,
        Err(error) => return Health::new("unreadable", error.message(), None),
    };
    health_at(&root, |der, thumbprint| {
        NativeCertificateStore.observe_machine(der, thumbprint)
    })
}

fn health_at(
    root: &Path,
    observe: impl FnOnce(&[u8], &str) -> Result<TrustState, fragcap::deep_capture::api::TrustError>,
) -> Health {
    let identity_path = identity_path(root);
    let key_path = key_path(root);
    let identity_present = match ordinary_file_presence(&identity_path) {
        Ok(present) => present,
        Err(detail) => return Health::new("unreadable", detail, None),
    };
    let key_present = match ordinary_file_presence(&key_path) {
        Ok(present) => present,
        Err(detail) => return Health::new("unreadable", detail, None),
    };
    let cutover_present = match ordinary_file_presence(&cutover_path(root)) {
        Ok(present) => present,
        Err(detail) => return Health::new("unreadable", detail, None),
    };
    if identity_present && !cutover_present {
        return Health::new("mismatch", "S185 cutover receipt missing", None);
    }
    if !identity_present {
        return if key_present {
            Health::new("mismatch", "key exists without public identity", None)
        } else {
            Health::new("not-provisioned", "no installation identity", None)
        };
    }
    let identity = match Identity::read(&identity_path) {
        Ok(identity) => identity,
        Err(error) => return Health::new("unreadable", error.message(), None),
    };
    if !key_present {
        return Health::new(
            "missing-key",
            "protected signer absent",
            Some(identity.fingerprint),
        );
    }
    let key_info = match std::fs::symlink_metadata(&key_path) {
        Ok(info) => info,
        Err(error) => {
            return Health::new(
                "unreadable",
                format!("protected signer inaccessible: {error}"),
                Some(identity.fingerprint),
            )
        }
    };
    if !key_info.is_file() || key_info.file_type().is_symlink() {
        return Health::new(
            "unreadable",
            "protected signer is not an ordinary file",
            Some(identity.fingerprint),
        );
    }
    let now = seconds(SystemTime::now()).unwrap_or_default();
    if now < identity.not_before || now >= identity.not_after {
        return Health::new(
            "expired",
            "root outside its validity interval",
            Some(identity.fingerprint),
        );
    }
    match observe(&identity.der, &identity.thumbprint) {
        Ok(TrustState::PresentExact) => Health::new(
            "ready",
            "public trust and protected key resource present; signing unverified",
            Some(identity.fingerprint),
        ),
        Ok(TrustState::Absent) => Health::new(
            "missing-cert",
            "exact root absent from LocalMachine/Root",
            Some(identity.fingerprint),
        ),
        Ok(TrustState::PresentWrongStore) => Health::new(
            "mismatch",
            "exact root is present only in CurrentUser/Root",
            Some(identity.fingerprint),
        ),
        Ok(TrustState::Mismatch) => Health::new(
            "mismatch",
            "root thumbprint resolves to different certificate bytes",
            Some(identity.fingerprint),
        ),
        Err(error) => Health::new("unreadable", error.to_string(), Some(identity.fingerprint)),
    }
}

fn ordinary_file_presence(path: &Path) -> Result<bool, String> {
    match std::fs::symlink_metadata(path) {
        Ok(info) if info.is_file() && !info.file_type().is_symlink() => Ok(true),
        Ok(_) => Err(format!(
            "installation resource is not an ordinary file: {}",
            path.display()
        )),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(format!(
            "installation resource inaccessible: {}: {error}",
            path.display()
        )),
    }
}

#[cfg(windows)]
pub(crate) fn load_for_deep_capture() -> Result<PreparedNativeAuthority, CliError> {
    let observed = health();
    if !observed.ready() {
        return Err(CliError::failure(observed.guidance()));
    }
    let root = root()?;
    let identity = Identity::read(&identity_path(&root))?;
    let authority = NativeProxyAdapter::load_installation_authority(identity.generation, &identity.der, &key_path(&root))
        .map_err(|error| CliError::failure(format!("installation CA unreadable in LocalMachine/Root: {error}; run elevated `fragcap ca repair`")))?;
    if authority.sha256_fingerprint() != identity.fingerprint
        || authority.sha1_thumbprint() != identity.thumbprint
    {
        return Err(CliError::failure(
            "installation CA mismatch in LocalMachine/Root; run elevated `fragcap ca repair`",
        ));
    }
    Ok(authority)
}

#[cfg(not(windows))]
pub(crate) fn load_for_deep_capture() -> Result<PreparedNativeAuthority, CliError> {
    Err(CliError::failure("installation CA requires Windows"))
}

#[cfg(windows)]
pub(crate) fn install() -> Result<String, CliError> {
    install_new(1, None)
}

#[cfg(windows)]
fn install_new(generation: u64, transaction: Option<&str>) -> Result<String, CliError> {
    if let Some(transaction) = transaction {
        validate_msi_transaction(transaction)?;
    }
    let observed = health();
    if observed.ready() {
        let authority = load_for_deep_capture()?;
        let root = root()?;
        if let Some((prior_transaction, prior_fingerprint)) = read_msi_receipt(&root)? {
            if transaction != Some(prior_transaction.as_str())
                || prior_fingerprint != authority.sha256_fingerprint()
            {
                return Err(CliError::failure(
                    "existing MSI CA creation receipt is unresolved; refuse reuse",
                ));
            }
            std::fs::remove_file(msi_receipt_path(&root)).map_err(|error| {
                CliError::failure(format!("MSI CA receipt disarm failed: {error}"))
            })?;
        }
        return Ok(format!(
            "reused installation CA {}",
            authority.sha256_fingerprint()
        ));
    }
    if observed.state != "not-provisioned" {
        return Err(CliError::failure(format!(
            "{}; explicit `fragcap ca repair` required",
            observed.guidance()
        )));
    }
    let root = root()?;
    if read_msi_receipt(&root)?.is_some() {
        return Err(CliError::failure(
            "unresolved MSI CA creation receipt blocks a new installation",
        ));
    }
    let pki = root.join("pki");
    NativeProxyAdapter::prepare_installation_directories(&root, &pki)
        .map_err(|error| CliError::failure(format!("installation storage refused: {error}")))?;
    if !cutover_path(&root).exists() {
        let result = crate::commands::fresh_start::cutover_all_users()?;
        use std::io::Write;
        let mut receipt = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(cutover_path(&root))
            .map_err(|error| {
                CliError::failure(format!("cutover receipt creation failed: {error}"))
            })?;
        writeln!(receipt, "{}", json!({"scheme":1,"result":result}))
            .and_then(|()| receipt.sync_all())
            .map_err(|error| CliError::failure(format!("cutover receipt write failed: {error}")))?;
    }
    let authority = NativeProxyAdapter::generate_installation_authority(
        generation,
        SystemTime::now(),
        CA_LIFETIME,
    )
    .map_err(|error| CliError::failure(format!("installation CA generation failed: {error}")))?;
    let provenance = match transaction {
        Some(value) => format!("fragcap S185 MSI {value}"),
        None => "fragcap S185 portable installation".to_string(),
    };
    let identity = Identity::from_authority(&authority, provenance)?;
    let key = key_path(&root);
    authority
        .persist_installation_key(&key)
        .map_err(|error| CliError::failure(format!("installation signer write failed: {error}")))?;
    let path = identity_path(&root);
    if let Err(error) = identity.write_new(&path) {
        let _ = std::fs::remove_file(&key);
        return Err(error);
    }
    let store = NativeCertificateStore;
    let added = match store.add_machine_exact(&identity.der, &identity.thumbprint) {
        Ok(added) => added,
        Err(error) => {
            let _ = std::fs::remove_file(&path);
            let _ = std::fs::remove_file(&key);
            return Err(CliError::failure(format!(
                "installation trust add failed: {error}"
            )));
        }
    };
    if let Err(error) = load_for_deep_capture() {
        if added == TrustMutation::Added {
            let _ = store.remove_machine_exact(&identity.der, &identity.thumbprint);
        }
        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_file(&key);
        return Err(error);
    }
    if let Some(transaction) = transaction {
        if let Err(error) = write_msi_receipt(&root, transaction, &identity.fingerprint) {
            let cleanup = remove(&identity.fingerprint, true);
            return Err(CliError::failure(format!(
                "{error}; newly created CA rollback: {}",
                if cleanup.is_ok() {
                    "complete"
                } else {
                    "incomplete"
                }
            )));
        }
    }
    Ok(format!(
        "installed CA {} in LocalMachine/Root",
        identity.fingerprint
    ))
}

#[cfg(not(windows))]
pub(crate) fn install() -> Result<String, CliError> {
    Err(CliError::failure("installation CA requires Windows"))
}

#[cfg(windows)]
pub(crate) fn repair(rotate: bool) -> Result<String, CliError> {
    let observed = health();
    if observed.ready() && !rotate {
        let authority = load_for_deep_capture()?;
        return Ok(format!(
            "installation CA {} is healthy",
            authority.sha256_fingerprint()
        ));
    }
    if observed.state == "mismatch" && observed.detail.contains("cutover receipt") {
        return Err(CliError::failure(
            "cutover receipt missing; refuse CA rotation until installation state is inspected",
        ));
    }
    if observed.state == "not-provisioned" {
        return install();
    }
    let root = root()?;
    let identity = Identity::read(&identity_path(&root))?;
    let store = NativeCertificateStore;
    if observed.state == "missing-cert" {
        // Prove the existing signer before restoring public trust.
        NativeProxyAdapter::load_installation_authority(
            identity.generation,
            &identity.der,
            &key_path(&root),
        )
        .map_err(|error| {
            CliError::failure(format!(
                "cannot restore trust without a valid signer: {error}"
            ))
        })?;
        store
            .add_machine_exact(&identity.der, &identity.thumbprint)
            .map_err(|error| {
                CliError::failure(format!("installation trust repair failed: {error}"))
            })?;
        load_for_deep_capture()?;
        return Ok(format!(
            "restored exact installation CA {} in LocalMachine/Root",
            identity.fingerprint
        ));
    }
    let existing = store
        .observe_machine(&identity.der, &identity.thumbprint)
        .map_err(|error| {
            CliError::failure(format!("installation trust inventory failed: {error}"))
        })?;
    if matches!(
        existing,
        TrustState::Mismatch | TrustState::PresentWrongStore
    ) {
        return Err(CliError::failure(
            "installation root is mismatched or only in the wrong store; refuse automatic rotation",
        ));
    }
    rotate_existing(&root, &identity, existing)
}

#[cfg(windows)]
fn rotate_existing(root: &Path, old: &Identity, old_store: TrustState) -> Result<String, CliError> {
    let generation = old
        .generation
        .checked_add(1)
        .ok_or_else(|| CliError::failure("installation CA generation exhausted"))?;
    let replacement = NativeProxyAdapter::generate_installation_authority(
        generation,
        SystemTime::now(),
        CA_LIFETIME,
    )
    .map_err(|error| CliError::failure(format!("replacement CA generation failed: {error}")))?;
    let next = Identity::from_authority(
        &replacement,
        "fragcap S185 explicit repair rotation".to_string(),
    )?;
    let key = key_path(root);
    let identity = identity_path(root);
    let staged_key = root.join("pki").join(format!("signer-{generation}.next"));
    let staged_identity = root.join(format!("ca-{generation}.next"));
    let saved_key = root
        .join("pki")
        .join(format!("signer-{generation}.previous"));
    let saved_identity = root.join(format!("ca-{generation}.previous"));
    for path in [&staged_key, &staged_identity, &saved_key, &saved_identity] {
        if ordinary_file_presence(path).map_err(CliError::failure)? {
            return Err(CliError::failure(format!(
                "rotation residue requires inspection: {}",
                path.display()
            )));
        }
    }
    replacement
        .persist_installation_key(&staged_key)
        .map_err(|error| {
            CliError::failure(format!("replacement signer staging failed: {error}"))
        })?;
    if let Err(error) = next.write_new(&staged_identity) {
        let _ = std::fs::remove_file(&staged_key);
        return Err(error);
    }
    // Prove the staged pair before trusting it or touching the old identity.
    if let Err(error) =
        NativeProxyAdapter::load_installation_authority(generation, &next.der, &staged_key)
    {
        let _ = std::fs::remove_file(&staged_key);
        let _ = std::fs::remove_file(&staged_identity);
        return Err(CliError::failure(format!(
            "replacement signer proof failed: {error}"
        )));
    }
    let store = NativeCertificateStore;
    if let Err(error) = store.add_machine_exact(&next.der, &next.thumbprint) {
        let _ = std::fs::remove_file(&staged_key);
        let _ = std::fs::remove_file(&staged_identity);
        return Err(CliError::failure(format!(
            "replacement trust staging failed: {error}"
        )));
    }
    let had_key = ordinary_file_presence(&key).map_err(CliError::failure)?;
    let mut saved_key_done = false;
    let mut saved_identity_done = false;
    let mut new_key_done = false;
    let mut new_identity_done = false;
    let committed = (|| -> Result<(), CliError> {
        if had_key {
            std::fs::rename(&key, &saved_key).map_err(|error| {
                CliError::failure(format!("old signer staging failed: {error}"))
            })?;
            saved_key_done = true;
        }
        std::fs::rename(&identity, &saved_identity)
            .map_err(|error| CliError::failure(format!("old identity staging failed: {error}")))?;
        saved_identity_done = true;
        std::fs::rename(&staged_key, &key)
            .map_err(|error| CliError::failure(format!("new signer activation failed: {error}")))?;
        new_key_done = true;
        std::fs::rename(&staged_identity, &identity).map_err(|error| {
            CliError::failure(format!("new identity activation failed: {error}"))
        })?;
        new_identity_done = true;
        load_for_deep_capture()?;
        if old_store == TrustState::PresentExact {
            store
                .remove_machine_exact(&old.der, &old.thumbprint)
                .map_err(|error| {
                    CliError::failure(format!("old trust retirement failed: {error}"))
                })?;
        }
        Ok(())
    })();
    if let Err(error) = committed {
        // Preserve the previous working identity on any activation failure.
        let mut rollback_errors = Vec::new();
        if new_identity_done {
            if let Err(problem) = std::fs::remove_file(&identity) {
                rollback_errors.push(format!("new identity: {problem}"));
            }
        }
        if new_key_done {
            if let Err(problem) = std::fs::remove_file(&key) {
                rollback_errors.push(format!("new signer: {problem}"));
            }
        }
        if saved_identity_done {
            if let Err(problem) = std::fs::rename(&saved_identity, &identity) {
                rollback_errors.push(format!("old identity: {problem}"));
            }
        }
        if saved_key_done {
            if let Err(problem) = std::fs::rename(&saved_key, &key) {
                rollback_errors.push(format!("old signer: {problem}"));
            }
        }
        if old_store == TrustState::PresentExact {
            match store.observe_machine(&old.der, &old.thumbprint) {
                Ok(TrustState::PresentExact) => {}
                Ok(TrustState::Absent) => {
                    if let Err(problem) = store.add_machine_exact(&old.der, &old.thumbprint) {
                        rollback_errors.push(format!("old root: {problem}"));
                    }
                }
                Ok(other) => rollback_errors.push(format!("old root: unexpected state {other:?}")),
                Err(problem) => rollback_errors.push(format!("old root: {problem}")),
            }
        }
        if let Err(problem) = store.remove_machine_exact(&next.der, &next.thumbprint) {
            rollback_errors.push(format!("new root: {problem}"));
        }
        if staged_key.exists() {
            if let Err(problem) = std::fs::remove_file(&staged_key) {
                rollback_errors.push(format!("staged signer: {problem}"));
            }
        }
        if staged_identity.exists() {
            if let Err(problem) = std::fs::remove_file(&staged_identity) {
                rollback_errors.push(format!("staged identity: {problem}"));
            }
        }
        return if rollback_errors.is_empty() {
            Err(error)
        } else {
            Err(CliError::failure(format!(
                "{}; rotation rollback incomplete: {}",
                error.message(),
                rollback_errors.join("; ")
            )))
        };
    }
    if saved_key_done {
        std::fs::remove_file(&saved_key)
            .map_err(|error| CliError::failure(format!("old signer retirement failed: {error}")))?;
    }
    std::fs::remove_file(&saved_identity)
        .map_err(|error| CliError::failure(format!("old identity retirement failed: {error}")))?;
    Ok(format!(
        "rotated installation CA {} to {} in LocalMachine/Root",
        old.fingerprint, next.fingerprint
    ))
}

#[cfg(not(windows))]
pub(crate) fn repair(_: bool) -> Result<String, CliError> {
    Err(CliError::failure("installation CA requires Windows"))
}

#[cfg(windows)]
pub(crate) fn remove(fingerprint: &str, yes: bool) -> Result<String, CliError> {
    let root = root()?;
    let has_msi_receipt = read_msi_receipt(&root)?.is_some();
    let identity = Identity::read(&identity_path(&root))?;
    if !yes || fingerprint != identity.fingerprint {
        return Err(CliError::usage(format!("removing the installation CA revokes machine-wide trust for all Windows users; review the exact SHA-256 fingerprint {} and run `fragcap ca remove --fingerprint {} --yes`", identity.fingerprint, identity.fingerprint)));
    }
    let store = NativeCertificateStore;
    let observed = store
        .observe_machine(&identity.der, &identity.thumbprint)
        .map_err(|error| {
            CliError::failure(format!("installation trust inventory failed: {error}"))
        })?;
    if observed == TrustState::Mismatch {
        return Err(CliError::failure(
            "refusing to remove a different root in LocalMachine/Root",
        ));
    }
    if observed == TrustState::PresentExact {
        store
            .remove_machine_exact(&identity.der, &identity.thumbprint)
            .map_err(|error| {
                CliError::failure(format!("installation CA removal failed: {error}"))
            })?;
    }
    if key_path(&root).exists() {
        std::fs::remove_file(key_path(&root)).map_err(|error| {
            CliError::failure(format!("installation signer removal failed: {error}"))
        })?;
    }
    std::fs::remove_file(identity_path(&root)).map_err(|error| {
        CliError::failure(format!("installation identity removal failed: {error}"))
    })?;
    if has_msi_receipt {
        std::fs::remove_file(msi_receipt_path(&root)).map_err(|error| {
            CliError::failure(format!("MSI CA receipt removal failed: {error}"))
        })?;
    }
    if health().state != "not-provisioned" {
        return Err(CliError::failure(
            "installation CA removal readback incomplete",
        ));
    }
    Ok(format!(
        "removed exact installation CA {} from LocalMachine/Root and protected storage",
        identity.fingerprint
    ))
}

#[cfg(not(windows))]
pub(crate) fn remove(_: &str, _: bool) -> Result<String, CliError> {
    Err(CliError::failure("installation CA requires Windows"))
}

#[cfg(windows)]
fn rollback_install(transaction: &str) -> Result<String, CliError> {
    validate_msi_transaction(transaction)?;
    let root = root()?;
    let Some((receipt_transaction, receipt_fingerprint)) = read_msi_receipt(&root)? else {
        return Ok("no CA creation receipt for this MSI execution; retained".to_string());
    };
    if receipt_transaction != transaction {
        return Ok("CA creation receipt belongs to another MSI transaction; retained".to_string());
    }
    if !identity_path(&root).exists() {
        return Err(CliError::failure(
            "MSI CA creation receipt exists without installation identity",
        ));
    }
    let identity = Identity::read(&identity_path(&root))?;
    if !rollback_owns_identity(
        Some(&(receipt_transaction, receipt_fingerprint)),
        transaction,
        &identity,
    ) {
        return Err(CliError::failure(
            "MSI CA receipt does not match the exact installation identity",
        ));
    }
    remove(&identity.fingerprint, true)
}

#[cfg(windows)]
fn commit_install(transaction: &str) -> Result<String, CliError> {
    validate_msi_transaction(transaction)?;
    let root = root()?;
    let Some((receipt_transaction, receipt_fingerprint)) = read_msi_receipt(&root)? else {
        return Ok("no CA creation receipt to commit; installation root retained".to_string());
    };
    if receipt_transaction != transaction {
        return Err(CliError::failure(
            "MSI CA creation receipt belongs to another transaction",
        ));
    }
    let identity = Identity::read(&identity_path(&root))?;
    if identity.fingerprint != receipt_fingerprint || !health().ready() {
        return Err(CliError::failure(
            "MSI CA commit requires the exact healthy installation identity",
        ));
    }
    std::fs::remove_file(msi_receipt_path(&root))
        .map_err(|error| CliError::failure(format!("MSI CA receipt commit failed: {error}")))?;
    Ok(format!(
        "committed installation CA {}",
        identity.fingerprint
    ))
}

#[cfg(not(windows))]
fn commit_install(_: &str) -> Result<String, CliError> {
    Err(CliError::failure("installation CA requires Windows"))
}

#[cfg(not(windows))]
fn rollback_install(_: &str) -> Result<String, CliError> {
    Err(CliError::failure("installation CA requires Windows"))
}

pub(crate) fn run_command(
    command: CaCommand,
    json_output: bool,
    out: &mut dyn Write,
) -> Result<Exit, CliError> {
    let status_only = matches!(&command, CaCommand::Status);
    let result = match command {
        CaCommand::Status => {
            let status = health();
            if json_output {
                json!({"state":status.state,"detail":status.detail,"fingerprint":status.fingerprint,"store":"LocalMachine/Root","trust_scope":"all Windows users","key_usable":"unverified-by-read-only-status"}).to_string()
            } else {
                format!("Installation CA: {}\nStore: LocalMachine/Root (all Windows users)\nFingerprint: {}\nDetail: {}\nSigner usability: unverified by read-only status\n", status.state, status.fingerprint.unwrap_or_else(|| "none".to_string()), status.detail)
            }
        }
        CaCommand::Install { msi_transaction } => {
            if let Some(transaction) = msi_transaction.as_deref() {
                #[cfg(windows)]
                {
                    install_new(1, Some(transaction))?
                }
                #[cfg(not(windows))]
                {
                    let _ = transaction;
                    install()?
                }
            } else {
                install()?
            }
        }
        CaCommand::RollbackInstall { msi_transaction } => rollback_install(&msi_transaction)?,
        CaCommand::CommitInstall { msi_transaction } => commit_install(&msi_transaction)?,
        CaCommand::Repair { rotate } => repair(rotate)?,
        CaCommand::Remove { fingerprint, yes } => remove(&fingerprint, yes)?,
    };
    let rendered = if json_output && !status_only {
        json!({"status":"complete","detail":result,"health":health().state}).to_string()
    } else {
        result
    };
    writeln!(out, "{rendered}")
        .map_err(|error| CliError::failure(format!("CA result output failed: {error}")))?;
    Ok(Exit::SUCCESS)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_product_code_cannot_rollback_a_committed_or_reused_ca() {
        let transaction = "{01234567-89AB-CDEF-0123-456789ABCDEF}";
        let authority =
            NativeProxyAdapter::generate_installation_authority(1, SystemTime::now(), CA_LIFETIME)
                .unwrap();
        let identity =
            Identity::from_authority(&authority, format!("fragcap S185 MSI {transaction}"))
                .unwrap();
        let receipt = (transaction.to_string(), identity.fingerprint.clone());
        assert!(rollback_owns_identity(
            Some(&receipt),
            transaction,
            &identity
        ));
        assert!(!rollback_owns_identity(None, transaction, &identity));
        assert!(!rollback_owns_identity(
            Some(&receipt),
            "{FEDCBA98-7654-3210-FEDC-BA9876543210}",
            &identity,
        ));
    }

    #[cfg(windows)]
    #[test]
    fn creation_receipt_is_exact_and_rejects_tampering() {
        let directory = std::env::temp_dir().join(format!(
            "fragcap-s185-msi-receipt-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
        ));
        std::fs::create_dir(&directory).unwrap();
        let transaction = "{01234567-89AB-CDEF-0123-456789ABCDEF}";
        let fingerprint = "A".repeat(64);
        write_msi_receipt(&directory, transaction, &fingerprint).unwrap();
        assert_eq!(
            read_msi_receipt(&directory).unwrap(),
            Some((transaction.to_string(), fingerprint))
        );
        assert!(write_msi_receipt(&directory, transaction, &"B".repeat(64)).is_err());
        std::fs::write(msi_receipt_path(&directory), b"{\"scheme\":1}").unwrap();
        assert!(read_msi_receipt(&directory).is_err());
        std::fs::remove_dir_all(&directory).unwrap();
    }

    #[test]
    fn public_identity_roundtrip_rejects_changed_validity_and_digest() {
        let authority =
            NativeProxyAdapter::generate_installation_authority(7, SystemTime::now(), CA_LIFETIME)
                .unwrap();
        let identity = Identity::from_authority(&authority, "controlled test".to_string()).unwrap();
        let directory = std::env::temp_dir().join(format!(
            "fragcap-s185-ca-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
        ));
        std::fs::create_dir(&directory).unwrap();
        let file = directory.join("ca.json");
        identity.write_new(&file).unwrap();
        assert_eq!(
            Identity::read(&file).unwrap().fingerprint,
            identity.fingerprint
        );
        let mut value: Value = serde_json::from_slice(&std::fs::read(&file).unwrap()).unwrap();
        value["not_after"] = json!(identity.not_after + 1);
        std::fs::write(&file, serde_json::to_vec(&value).unwrap()).unwrap();
        assert!(Identity::read(&file)
            .unwrap_err()
            .message()
            .contains("validity mismatch"));
        value["not_after"] = json!(identity.not_after);
        value["sha256"] = json!("00");
        std::fs::write(&file, serde_json::to_vec(&value).unwrap()).unwrap();
        assert!(Identity::read(&file)
            .unwrap_err()
            .message()
            .contains("fingerprint mismatch"));
        std::fs::remove_dir_all(&directory).unwrap();
    }

    #[test]
    fn public_resource_presence_distinguishes_missing_file_and_directory() {
        let directory = std::env::temp_dir().join(format!(
            "fragcap-s185-presence-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
        ));
        assert_eq!(ordinary_file_presence(&directory), Ok(false));
        std::fs::create_dir(&directory).unwrap();
        assert!(ordinary_file_presence(&directory).is_err());
        let file = directory.join("ca.json");
        std::fs::write(&file, b"{}").unwrap();
        assert_eq!(ordinary_file_presence(&file), Ok(true));
        std::fs::remove_dir_all(&directory).unwrap();
    }

    #[test]
    fn read_only_health_classifies_every_public_state_without_opening_a_signer() {
        let directory = std::env::temp_dir().join(format!(
            "fragcap-s185-health-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
        ));
        std::fs::create_dir(&directory).unwrap();
        let observed = |state| health_at(&directory, |_, _| Ok(state));
        assert_eq!(observed(TrustState::Absent).state, "not-provisioned");
        std::fs::create_dir(directory.join("pki")).unwrap();
        std::fs::write(key_path(&directory), b"unopened test key").unwrap();
        assert_eq!(observed(TrustState::Absent).state, "mismatch");
        std::fs::remove_file(key_path(&directory)).unwrap();
        let now = SystemTime::now();
        let authority =
            NativeProxyAdapter::generate_installation_authority(1, now, CA_LIFETIME).unwrap();
        Identity::from_authority(&authority, "test".into())
            .unwrap()
            .write_new(&identity_path(&directory))
            .unwrap();
        assert_eq!(observed(TrustState::Absent).state, "mismatch");
        std::fs::write(cutover_path(&directory), b"{}").unwrap();
        assert_eq!(observed(TrustState::Absent).state, "missing-key");
        std::fs::create_dir(key_path(&directory)).unwrap();
        assert_eq!(observed(TrustState::Absent).state, "unreadable");
        std::fs::remove_dir(key_path(&directory)).unwrap();
        std::fs::write(key_path(&directory), b"unopened test key").unwrap();
        assert_eq!(observed(TrustState::Absent).state, "missing-cert");
        assert_eq!(observed(TrustState::PresentWrongStore).state, "mismatch");
        assert_eq!(observed(TrustState::Mismatch).state, "mismatch");
        assert_eq!(observed(TrustState::PresentExact).state, "ready");
        std::fs::remove_file(identity_path(&directory)).unwrap();
        let expired = NativeProxyAdapter::generate_installation_authority(
            2,
            now - Duration::from_secs(2 * 365 * 24 * 60 * 60),
            Duration::from_secs(3600),
        )
        .unwrap();
        Identity::from_authority(&expired, "test expired".into())
            .unwrap()
            .write_new(&identity_path(&directory))
            .unwrap();
        assert_eq!(observed(TrustState::PresentExact).state, "expired");
        std::fs::remove_dir_all(&directory).unwrap();
    }
}
