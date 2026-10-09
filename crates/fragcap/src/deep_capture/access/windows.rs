// SPDX-License-Identifier: Apache-2.0

use super::*;
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::os::windows::fs::{MetadataExt, OpenOptionsExt};
use std::os::windows::io::{AsRawHandle, FromRawHandle};
use std::ptr;
use std::sync::Arc;
use std::thread;
use std::time::Instant;
use windows_sys::Win32::Foundation::*;
use windows_sys::Win32::Security::Authorization::*;
use windows_sys::Win32::Security::*;
use windows_sys::Win32::Storage::FileSystem::*;
use windows_sys::Win32::System::Memory::LocalFree;
use windows_sys::Win32::System::Pipes::*;
use windows_sys::Win32::System::RemoteDesktop::*;
use windows_sys::Win32::System::SystemServices::{
    GENERIC_READ, GENERIC_WRITE, SECURITY_MANDATORY_MEDIUM_RID, SE_DACL_PROTECTED,
    SE_GROUP_INTEGRITY, WRITE_DAC,
};
use windows_sys::Win32::System::Threading::OpenThreadToken;

const CURRENT_TOKEN: isize = -4;
const CURRENT_EFFECTIVE_TOKEN: isize = -6;
const CURRENT_THREAD: isize = -2;
const RIGHTS: u32 = TOKEN_QUERY | TOKEN_DUPLICATE | TOKEN_IMPERSONATE | TOKEN_ADJUST_DEFAULT;

#[derive(Debug)]
struct Handle(isize);
// SAFETY: owned token and pipe handles may be transferred between threads. Their
// lifetimes are owned by this wrapper; no process-wide token state is changed.
unsafe impl Send for Handle {}
unsafe impl Sync for Handle {}
impl Drop for Handle {
    fn drop(&mut self) {
        unsafe { CloseHandle(self.0) };
    }
}
#[derive(Debug)]
pub(super) struct RecipientToken {
    token: Handle,
    session: u32,
    desktop: bool,
}

fn error() -> io::Error {
    io::Error::last_os_error()
}
fn bad(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::PermissionDenied, message)
}
fn wide(value: impl AsRef<OsStr>) -> Vec<u16> {
    value.as_ref().encode_wide().chain(Some(0)).collect()
}
fn string(pointer: *const u16) -> io::Result<String> {
    if pointer.is_null() {
        return Err(bad("missing Windows string"));
    }
    let mut length = 0;
    while length < 32768 && unsafe { *pointer.add(length) } != 0 {
        length += 1;
    }
    if length == 32768 {
        return Err(bad("Windows string exceeds limit"));
    }
    String::from_utf16(unsafe { std::slice::from_raw_parts(pointer, length) })
        .map_err(io::Error::other)
}
fn info(token: isize, class: i32) -> io::Result<Vec<usize>> {
    let mut needed = 0;
    unsafe { GetTokenInformation(token, class, ptr::null_mut(), 0, &mut needed) };
    if needed == 0 || needed > 65536 {
        return Err(error());
    }
    let mut data = vec![0_usize; (needed as usize).div_ceil(std::mem::size_of::<usize>())];
    if unsafe { GetTokenInformation(token, class, data.as_mut_ptr().cast(), needed, &mut needed) }
        == 0
    {
        return Err(error());
    }
    Ok(data)
}
fn scalar(token: isize, class: i32) -> io::Result<u32> {
    let data = info(token, class)?;
    Ok(unsafe { *data.as_ptr().cast::<u32>() })
}
fn sid_string(sid: *mut core::ffi::c_void) -> io::Result<String> {
    let mut text = ptr::null_mut();
    if unsafe { ConvertSidToStringSidW(sid, &mut text) } == 0 {
        return Err(error());
    }
    let result = string(text);
    unsafe { LocalFree(text as isize) };
    result
}
fn user(token: isize) -> io::Result<String> {
    let data = info(token, TokenUser)?;
    sid_string(unsafe { (*data.as_ptr().cast::<TOKEN_USER>()).User.Sid })
}
fn individual(value: &str) -> io::Result<()> {
    let sid = Sid::parse(value)?;
    let mut name_len = 0;
    let mut domain_len = 0;
    let mut kind = 0;
    unsafe {
        LookupAccountSidW(
            ptr::null(),
            sid.0,
            ptr::null_mut(),
            &mut name_len,
            ptr::null_mut(),
            &mut domain_len,
            &mut kind,
        )
    };
    if name_len == 0 || name_len > 32768 || domain_len > 32768 {
        return Err(bad(
            "recipient SID does not resolve to an individual account",
        ));
    }
    let mut name = vec![0_u16; name_len as usize];
    let mut domain = vec![0_u16; domain_len as usize];
    if unsafe {
        LookupAccountSidW(
            ptr::null(),
            sid.0,
            name.as_mut_ptr(),
            &mut name_len,
            domain.as_mut_ptr(),
            &mut domain_len,
            &mut kind,
        )
    } == 0
    {
        return Err(error());
    }
    if kind != SidTypeUser {
        return Err(bad(
            "recipient SID identifies a group or unsupported principal",
        ));
    }
    Ok(())
}
fn session(token: isize) -> io::Result<u32> {
    scalar(token, TokenSessionId)
}
fn ordinary(token: isize) -> io::Result<()> {
    if scalar(token, TokenElevation)? != 0 {
        return Err(bad("output recipient token is elevated"));
    }
    let data = info(token, TokenIntegrityLevel)?;
    let sid = unsafe { (*data.as_ptr().cast::<TOKEN_MANDATORY_LABEL>()).Label.Sid };
    let count = unsafe { *GetSidSubAuthorityCount(sid) };
    if count == 0
        || unsafe { *GetSidSubAuthority(sid, u32::from(count) - 1) }
            > SECURITY_MANDATORY_MEDIUM_RID as u32
    {
        return Err(bad(
            "output recipient integrity exceeds ordinary medium level",
        ));
    }
    Ok(())
}
fn isolated<T: Send + 'static>(
    operation: impl FnOnce() -> io::Result<T> + Send + 'static,
) -> io::Result<T> {
    thread::spawn(operation)
        .join()
        .map_err(|_| io::Error::other("recipient verification thread failed"))?
}
fn current_owned() -> io::Result<Handle> {
    isolated(|| {
        if unsafe { ImpersonateSelf(SecurityImpersonation) } == 0 {
            return Err(error());
        }
        let mut result = 0;
        let ok = unsafe { OpenThreadToken(CURRENT_THREAD, RIGHTS, 1, &mut result) };
        let failure = if ok == 0 { Some(error()) } else { None };
        if unsafe { RevertToSelf() } == 0 {
            return Err(bad("failed to restore token acquisition thread"));
        }
        match failure {
            Some(e) => Err(e),
            None => Ok(Handle(result)),
        }
    })
}
fn duplicate(token: isize) -> io::Result<Handle> {
    let mut result = 0;
    if unsafe {
        DuplicateTokenEx(
            token,
            RIGHTS,
            ptr::null(),
            SecurityImpersonation,
            TokenImpersonation,
            &mut result,
        )
    } == 0
    {
        return Err(error());
    }
    Ok(Handle(result))
}
fn session_user(id: u32) -> io::Result<String> {
    fn query(id: u32, class: i32) -> io::Result<String> {
        let mut data = ptr::null_mut();
        let mut size = 0;
        if unsafe { WTSQuerySessionInformationW(0, id, class, &mut data, &mut size) } == 0 {
            return Err(error());
        }
        let result = if size > 65536 {
            Err(bad("session identity exceeds limit"))
        } else {
            string(data)
        };
        unsafe { WTSFreeMemory(data.cast()) };
        result
    }
    let name = query(id, WTSUserName)?;
    let domain = query(id, WTSDomainName)?;
    if name.is_empty() || domain.is_empty() {
        return Err(bad("exact Windows session user is unavailable; use an explicit authenticated recipient handoff"));
    }
    let qualified = wide(format!("{domain}\\{name}"));
    let mut size = 0;
    let mut domain_size = 0;
    let mut kind = 0;
    unsafe {
        LookupAccountNameW(
            ptr::null(),
            qualified.as_ptr(),
            ptr::null_mut(),
            &mut size,
            ptr::null_mut(),
            &mut domain_size,
            &mut kind,
        )
    };
    if size == 0 || size > 65536 || domain_size > 32768 {
        return Err(error());
    }
    let mut sid = vec![0_usize; (size as usize).div_ceil(std::mem::size_of::<usize>())];
    let mut found_domain = vec![0_u16; domain_size as usize];
    if unsafe {
        LookupAccountNameW(
            ptr::null(),
            qualified.as_ptr(),
            sid.as_mut_ptr().cast(),
            &mut size,
            found_domain.as_mut_ptr(),
            &mut domain_size,
            &mut kind,
        )
    } == 0
    {
        return Err(error());
    }
    if kind != SidTypeUser {
        return Err(bad("exact session account is not an individual user"));
    }
    sid_string(sid.as_mut_ptr().cast())
}
fn restricted(token: isize) -> io::Result<Handle> {
    let groups = info(token, TokenGroups)?;
    let group_info = unsafe { &*groups.as_ptr().cast::<TOKEN_GROUPS>() };
    let groups = unsafe {
        std::slice::from_raw_parts(group_info.Groups.as_ptr(), group_info.GroupCount as usize)
    };
    let mut disabled = Vec::new();
    for group in groups {
        if unsafe { IsWellKnownSid(group.Sid, WinBuiltinAdministratorsSid) } != 0 {
            disabled.push(*group);
        }
    }
    let mut result = 0;
    if unsafe {
        CreateRestrictedToken(
            token,
            DISABLE_MAX_PRIVILEGE | LUA_TOKEN,
            disabled.len() as u32,
            disabled.as_ptr(),
            0,
            ptr::null(),
            0,
            ptr::null(),
            &mut result,
        )
    } == 0
    {
        return Err(error());
    }
    let result = Handle(result);
    let label_sid = Sid::parse("S-1-16-8192")?;
    let label = TOKEN_MANDATORY_LABEL {
        Label: SID_AND_ATTRIBUTES {
            Sid: label_sid.0,
            Attributes: SE_GROUP_INTEGRITY as u32,
        },
    };
    if unsafe {
        SetTokenInformation(
            result.0,
            TokenIntegrityLevel,
            (&label as *const TOKEN_MANDATORY_LABEL).cast(),
            (std::mem::size_of::<TOKEN_MANDATORY_LABEL>() as u32) + GetLengthSid(label_sid.0),
        )
    } == 0
    {
        return Err(error());
    }
    ordinary(result.0)?;
    Ok(result)
}
pub(super) fn recipient(desktop: bool) -> io::Result<OutputRecipient> {
    let producer = user(CURRENT_TOKEN)?;
    let session = session(CURRENT_TOKEN)?;
    if desktop && session_user(session)? != producer {
        return Err(bad("session user differs from elevated producer; provide --output-recipient with --bundle and authenticate from the intended ordinary session"));
    }
    let owned = current_owned()?;
    let (token, proof) = if scalar(owned.0, TokenElevation)? == 0 {
        (owned, "ordinary-current")
    } else if scalar(owned.0, TokenElevationType)? == TokenElevationTypeFull as u32 {
        let linked = info(owned.0, TokenLinkedToken)?;
        let linked = Handle(unsafe { (*linked.as_ptr().cast::<TOKEN_LINKED_TOKEN>()).LinkedToken });
        if user(linked.0)? != producer || self::session(linked.0)? != session {
            return Err(bad("linked ordinary token identity mismatch"));
        }
        ordinary(linked.0)?;
        (duplicate(linked.0)?, "session-linked")
    } else if !desktop {
        (restricted(owned.0)?, "controlled-restricted-equivalent")
    } else {
        return Err(bad("elevated producer has no provable ordinary linked context; use an authenticated recipient handoff"));
    };
    ordinary(token.0)?;
    Ok(OutputRecipient {
        sid: producer.clone(),
        producer_sid: producer,
        proof_kind: proof.into(),
        platform: Arc::new(RecipientToken {
            token,
            session,
            desktop,
        }),
    })
}
pub(super) fn revalidate(recipient: &OutputRecipient) -> io::Result<()> {
    if user(recipient.platform.token.0)? != recipient.sid
        || user(CURRENT_TOKEN)? != recipient.producer_sid
    {
        return Err(bad("retained recipient/producer identity changed"));
    }
    ordinary(recipient.platform.token.0)?;
    if recipient.platform.desktop && session_user(recipient.platform.session)? != recipient.sid {
        return Err(bad("exact session recipient association changed"));
    }
    Ok(())
}
fn with_token<T: Send + 'static>(
    token: Arc<RecipientToken>,
    f: impl FnOnce() -> io::Result<T> + Send + 'static,
) -> io::Result<T> {
    isolated(move || {
        if unsafe { ImpersonateLoggedOnUser(token.token.0) } == 0 {
            return Err(error());
        }
        let result = f();
        if unsafe { RevertToSelf() } == 0 {
            return Err(bad("failed to restore recipient verification thread"));
        }
        result
    })
}
pub(super) fn verify(
    recipient: &OutputRecipient,
    root: &Path,
    paths: &[PathBuf],
) -> io::Result<BundleAccessVerification> {
    let root = root.to_path_buf();
    let paths = paths.to_vec();
    let sid = recipient.sid.clone();
    let proof = recipient.proof_kind.clone();
    with_token(recipient.platform.clone(), move || {
        Ok(verify_current(&root, &paths, &sid, &proof))
    })
}

struct Sid(*mut core::ffi::c_void);
impl Sid {
    fn parse(value: &str) -> io::Result<Self> {
        let text = wide(value);
        let mut sid = ptr::null_mut();
        if unsafe { ConvertStringSidToSidW(text.as_ptr(), &mut sid) } == 0 {
            return Err(error());
        }
        Ok(Self(sid))
    }
}
impl Drop for Sid {
    fn drop(&mut self) {
        unsafe { LocalFree(self.0 as isize) };
    }
}
struct Descriptor(*mut core::ffi::c_void);
impl Descriptor {
    fn parse(value: &str) -> io::Result<Self> {
        let text = wide(value);
        let mut pointer = ptr::null_mut();
        let mut size = 0;
        if unsafe {
            ConvertStringSecurityDescriptorToSecurityDescriptorW(
                text.as_ptr(),
                1,
                &mut pointer,
                &mut size,
            )
        } == 0
        {
            return Err(error());
        }
        Ok(Self(pointer))
    }
    fn dacl(&self) -> io::Result<*mut ACL> {
        let mut acl = ptr::null_mut();
        let mut present = 0;
        let mut defaulted = 0;
        if unsafe { GetSecurityDescriptorDacl(self.0, &mut present, &mut acl, &mut defaulted) } == 0
            || present == 0
            || acl.is_null()
        {
            return Err(bad("private access descriptor has no DACL"));
        }
        Ok(acl)
    }
}
impl Drop for Descriptor {
    fn drop(&mut self) {
        unsafe { LocalFree(self.0 as isize) };
    }
}
pub(super) fn contract(recipient: &OutputRecipient) -> Vec<String> {
    let mut result = vec![
        recipient.sid.clone(),
        recipient.producer_sid.clone(),
        "S-1-5-18".into(),
    ];
    result.sort();
    result.dedup();
    result
}
fn sddl(principals: &[String], directory: bool) -> String {
    let mut result = "D:P".to_string();
    for sid in principals {
        result.push_str(&format!(
            "(A;{};FA;;;{sid})",
            if directory { "OICI" } else { "" }
        ));
    }
    result
}
fn reject_reparse_ancestors(path: &Path) -> io::Result<()> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()?.join(path)
    };
    for ancestor in absolute.ancestors() {
        if let Ok(meta) = std::fs::symlink_metadata(ancestor) {
            if meta.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
                return Err(bad("access refuses reparse-point ancestors"));
            }
        }
    }
    Ok(())
}
pub(super) fn pin(path: &Path, write_dacl: bool) -> io::Result<PinnedAccessPath> {
    let ancestors = pin_ancestors(path)?;
    let meta = std::fs::symlink_metadata(path)?;
    let access = READ_CONTROL | if write_dacl { WRITE_DAC } else { 0 };
    let file = std::fs::OpenOptions::new()
        .access_mode(access)
        .share_mode(if write_dacl && meta.is_dir() {
            0
        } else if write_dacl {
            FILE_SHARE_READ
        } else {
            FILE_SHARE_READ | FILE_SHARE_WRITE
        })
        .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
        .open(path)?;
    let (identity, descriptor) = snapshot(&file)?;
    if !identity.directory && identity.links != 1 {
        return Err(bad("access refuses ambiguous external hard links"));
    }
    Ok(PinnedAccessPath {
        path: path.to_path_buf(),
        identity,
        descriptor,
        file,
        _ancestors: ancestors,
    })
}
fn pin_ancestors(path: &Path) -> io::Result<Vec<File>> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()?.join(path)
    };
    if absolute
        .components()
        .any(|c| matches!(c, std::path::Component::ParentDir))
    {
        return Err(bad("access path contains a parent component"));
    }
    let mut ancestors = absolute
        .parent()
        .into_iter()
        .flat_map(|p| p.ancestors())
        .collect::<Vec<_>>();
    ancestors.reverse();
    let mut pinned = Vec::new();
    for ancestor in ancestors {
        if pinned.len() >= 128 {
            return Err(bad("access ancestor depth exceeds limit"));
        }
        let file = match std::fs::OpenOptions::new()
            // Windows permits rename past a zero-access directory handle despite
            // no delete sharing. Real read access makes this an identity guard.
            .access_mode(GENERIC_READ)
            .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE)
            .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
            .open(ancestor)
        {
            Ok(v) => v,
            Err(e) if e.kind() == io::ErrorKind::NotFound => break,
            Err(e) => return Err(e),
        };
        let meta = file.metadata()?;
        if meta.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 || !meta.is_dir() {
            return Err(bad("access ancestor is a reparse point or not a directory"));
        }
        pinned.push(file);
    }
    Ok(pinned)
}
pub(super) fn snapshot(file: &File) -> io::Result<(AccessObjectIdentity, String)> {
    let handle = file.as_raw_handle() as isize;
    let mut identity = unsafe { std::mem::zeroed::<BY_HANDLE_FILE_INFORMATION>() };
    if unsafe { GetFileInformationByHandle(handle, &mut identity) } == 0 {
        return Err(error());
    }
    if identity.dwFileAttributes & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
        return Err(bad("access refuses a pinned reparse point"));
    }
    let mut sd = ptr::null_mut();
    let code = unsafe {
        GetSecurityInfo(
            handle,
            SE_FILE_OBJECT,
            OWNER_SECURITY_INFORMATION | DACL_SECURITY_INFORMATION,
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            &mut sd,
        )
    };
    if code != 0 {
        return Err(io::Error::from_raw_os_error(code as i32));
    }
    let descriptor = Descriptor(sd);
    let mut text = ptr::null_mut();
    let mut size = 0;
    if unsafe {
        ConvertSecurityDescriptorToStringSecurityDescriptorW(
            descriptor.0,
            1,
            OWNER_SECURITY_INFORMATION | DACL_SECURITY_INFORMATION,
            &mut text,
            &mut size,
        )
    } == 0
    {
        return Err(error());
    }
    let value = string(text);
    unsafe { LocalFree(text as isize) };
    Ok((
        AccessObjectIdentity {
            volume_serial: u64::from(identity.dwVolumeSerialNumber),
            file_index: (u64::from(identity.nFileIndexHigh) << 32)
                | u64::from(identity.nFileIndexLow),
            links: identity.nNumberOfLinks,
            directory: identity.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY != 0,
        },
        value?,
    ))
}
pub(super) fn apply(file: &File, directory: bool, principals: &[String]) -> io::Result<()> {
    let descriptor = Descriptor::parse(&sddl(principals, directory))?;
    let code = unsafe {
        SetSecurityInfo(
            file.as_raw_handle() as isize,
            SE_FILE_OBJECT,
            DACL_SECURITY_INFORMATION | PROTECTED_DACL_SECURITY_INFORMATION,
            ptr::null_mut(),
            ptr::null_mut(),
            descriptor.dacl()?,
            ptr::null_mut(),
        )
    };
    if code == 0 {
        Ok(())
    } else {
        Err(io::Error::from_raw_os_error(code as i32))
    }
}
pub(super) fn traversal(file: &File, recipient: &OutputRecipient) -> io::Result<()> {
    let (identity, text) = snapshot(file)?;
    if !identity.directory {
        return Err(bad("traversal grant requires a directory"));
    }
    let descriptor = Descriptor::parse(&text)?;
    let sid = Sid::parse(&recipient.sid)?;
    let entry = EXPLICIT_ACCESS_W {
        grfAccessPermissions: FILE_GENERIC_READ | FILE_GENERIC_EXECUTE,
        grfAccessMode: GRANT_ACCESS,
        grfInheritance: 0,
        Trustee: TRUSTEE_W {
            pMultipleTrustee: ptr::null_mut(),
            MultipleTrusteeOperation: NO_MULTIPLE_TRUSTEE,
            TrusteeForm: TRUSTEE_IS_SID,
            TrusteeType: TRUSTEE_IS_USER,
            ptstrName: sid.0.cast(),
        },
    };
    let mut acl = ptr::null_mut();
    let code = unsafe { SetEntriesInAclW(1, &entry, descriptor.dacl()?, &mut acl) };
    if code != 0 {
        return Err(io::Error::from_raw_os_error(code as i32));
    }
    let code = unsafe {
        SetSecurityInfo(
            file.as_raw_handle() as isize,
            SE_FILE_OBJECT,
            DACL_SECURITY_INFORMATION,
            ptr::null_mut(),
            ptr::null_mut(),
            acl,
            ptr::null_mut(),
        )
    };
    unsafe { LocalFree(acl as isize) };
    if code == 0 {
        Ok(())
    } else {
        Err(io::Error::from_raw_os_error(code as i32))
    }
}
fn parent_contract(path: &Path) -> io::Result<Vec<String>> {
    let parent = path
        .parent()
        .ok_or_else(|| bad("private child has no parent"))?;
    if let Some(principals) = private_contract(parent)? {
        return Ok(principals);
    }
    // An inherited private child may preserve the exact protected ancestor
    // contract only when every intermediate directory has the same ACE set.
    if let Some(principals) = inherited_contract(parent)? {
        return Ok(principals);
    }
    if protected_directory(parent)? {
        return Err(bad("protected parent has an unsupported recipient contract; inspect the exact bundle before repair"));
    }
    for ancestor in parent.ancestors().skip(1) {
        if private_contract(ancestor)?.is_some() {
            return Err(bad(
                "nested private output directory lost the exact protected ancestor contract",
            ));
        }
    }
    // Standalone public library calls establish their explicit current-account
    // contract when no selected private parent contract exists.
    Ok(contract(&recipient(false)?))
}
fn private_contract(path: &Path) -> io::Result<Option<Vec<String>>> {
    let pinned = pin(path, false)?;
    if !pinned.identity.directory {
        return Err(bad("private output parent must be a directory"));
    }
    let descriptor = Descriptor::parse(&pinned.descriptor)?;
    let acl = descriptor.dacl()?;
    let mut control = 0;
    let mut revision = 0;
    if unsafe { GetSecurityDescriptorControl(descriptor.0, &mut control, &mut revision) } == 0 {
        return Err(error());
    }
    if control & SE_DACL_PROTECTED as u16 == 0 {
        return Ok(None);
    }
    acl_principals(acl)
}
fn acl_principals(acl: *mut ACL) -> io::Result<Option<Vec<String>>> {
    if acl.is_null() || unsafe { IsValidAcl(acl) } == 0 {
        return Err(bad("private output has an invalid ACL"));
    }
    let count = unsafe { (*acl).AceCount };
    let mut principals = Vec::new();
    let mut private = true;
    for index in 0..u32::from(count) {
        let mut ace = std::mem::MaybeUninit::<*mut core::ffi::c_void>::uninit();
        if unsafe { GetAce(acl, index, ace.as_mut_ptr()) } == 0 {
            return Err(error());
        }
        // SAFETY: successful GetAce initializes its documented output pointer;
        // the descriptor owns its storage throughout this bounded decode.
        let ace = unsafe { ace.assume_init() };
        if ace.is_null() {
            return Err(bad("private output has a missing ACE"));
        }
        let header = unsafe { &*ace.cast::<ACE_HEADER>() };
        let sid_offset = std::mem::offset_of!(ACCESS_ALLOWED_ACE, SidStart);
        // Check the common header before interpreting a variable-size ACE. The
        // SID header itself occupies eight bytes before any subauthorities.
        if header.AceType != 0 || usize::from(header.AceSize) < sid_offset + 8 {
            private = false;
            break;
        }
        let allowed = unsafe { &*ace.cast::<ACCESS_ALLOWED_ACE>() };
        if allowed.Mask != FILE_ALL_ACCESS
            || allowed.Header.AceFlags & 3 != 3
            || allowed.Header.AceFlags & !(3 | 16) != 0
        {
            private = false;
            break;
        }
        let sid_pointer = (&allowed.SidStart as *const u32).cast_mut().cast();
        if unsafe { IsValidSid(sid_pointer) } == 0
            || unsafe { GetLengthSid(sid_pointer) } as usize
                > usize::from(header.AceSize) - sid_offset
        {
            return Err(bad("private output has an invalid ACE SID"));
        }
        let sid = sid_string(sid_pointer)?;
        if sid != "S-1-5-18" && !sid.starts_with("S-1-5-21-") && !sid.starts_with("S-1-12-1-") {
            private = false;
            break;
        }
        if sid != "S-1-5-18" {
            individual(&sid)?;
        }
        principals.push(sid);
    }
    principals.sort();
    principals.dedup();
    if private
        && principals.len() >= 2
        && principals.len() <= 3
        && principals.iter().any(|s| s == "S-1-5-18")
    {
        if !principals.contains(&user(CURRENT_TOKEN)?) {
            return Err(bad(
                "private parent contract does not name the exact producer",
            ));
        }
        return Ok(Some(principals));
    }
    Ok(None)
}
fn inherited_contract(path: &Path) -> io::Result<Option<Vec<String>>> {
    let pinned = pin(path, false)?;
    let descriptor = Descriptor::parse(&pinned.descriptor)?;
    let Some(principals) = acl_principals(descriptor.dacl()?)? else {
        return Ok(None);
    };
    let Some(parent) = path.parent() else {
        return Ok(None);
    };
    let ancestor = match private_contract(parent)? {
        Some(v) => Some(v),
        None => inherited_contract(parent)?,
    };
    Ok(ancestor.filter(|v| v == &principals))
}
pub(super) fn preserve_directory(path: &Path) -> io::Result<bool> {
    if !path.exists() {
        return Ok(false);
    }
    let selected = private_contract(path)?.is_some();
    if !selected && protected_directory(path)? {
        return Err(bad(
            "existing protected bundle requires exact access inspection/repair",
        ));
    }
    Ok(selected)
}
fn protected_directory(path: &Path) -> io::Result<bool> {
    let pinned = pin(path, false)?;
    let descriptor = Descriptor::parse(&pinned.descriptor)?;
    let mut control = 0;
    let mut revision = 0;
    if unsafe { GetSecurityDescriptorControl(descriptor.0, &mut control, &mut revision) } == 0 {
        return Err(error());
    }
    Ok(control & SE_DACL_PROTECTED as u16 != 0)
}
pub(super) fn protect_directory(path: &Path, recipient: &OutputRecipient) -> io::Result<()> {
    recipient.revalidate()?;
    let path = std::path::absolute(path)?;
    if path.components().count() > 128 {
        return Err(bad("recipient output path depth exceeds limit"));
    }
    reject_reparse_ancestors(&path)?;
    let _ancestors = pin_ancestors(&path)?;
    let mut missing = Vec::new();
    let mut cursor = path.as_path();
    loop {
        match std::fs::symlink_metadata(cursor) {
            Ok(meta) => {
                if !meta.is_dir() || meta.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
                    return Err(bad(
                        "recipient output ancestor is not an ordinary directory",
                    ));
                }
                break;
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                if missing.len() >= 128 {
                    return Err(bad("recipient output creation depth exceeds limit"));
                }
                missing.push(cursor.to_path_buf());
                cursor = cursor
                    .parent()
                    .ok_or_else(|| bad("recipient output has no existing ancestor"))?;
            }
            Err(error) => return Err(error),
        }
    }
    let ancestor = if missing.is_empty() {
        path.parent()
            .ok_or_else(|| bad("recipient output has no parent"))?
    } else {
        cursor
    };
    let verification = recipient.verify_paths(ancestor, &[ancestor.to_path_buf()])?;
    if !verification.is_verified() {
        return Err(bad("ordinary recipient cannot enumerate the existing output ancestor; inspect/repair the exact owned container or choose an accessible explicit destination"));
    }
    if missing.is_empty() {
        return pin(&path, true)?.apply(recipient);
    }
    let principals = contract(recipient);
    let descriptor = Descriptor::parse(&sddl(&principals, true))?;
    let attributes = SECURITY_ATTRIBUTES {
        nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
        lpSecurityDescriptor: descriptor.0,
        bInheritHandle: 0,
    };
    let mut created = Vec::new();
    for directory in missing.into_iter().rev() {
        let name = wide(&directory);
        // Existing objects are never adopted after a concurrent creation. Every
        // new intermediate gets the selected contract at its creation boundary.
        if unsafe { CreateDirectoryW(name.as_ptr(), &attributes) } == 0 {
            return Err(error());
        }
        let normalization = pin(&directory, true)?;
        let created_descriptor = Descriptor::parse(&normalization.descriptor)?;
        let mut control = 0;
        let mut revision = 0;
        if unsafe {
            GetSecurityDescriptorControl(created_descriptor.0, &mut control, &mut revision)
        } == 0
        {
            return Err(error());
        }
        if control & SE_DACL_PROTECTED as u16 == 0
            || acl_principals(created_descriptor.dacl()?)?.as_ref() != Some(&principals)
        {
            return Err(bad(
                "new output directory changed before permission normalization",
            ));
        }
        if std::fs::read_dir(&directory)?.next().is_some() {
            return Err(bad("new output directory changed during creation"));
        }
        // Normalize Windows' inheritable-ACE state before creating descendants.
        // Creation already used the same protected exact-principal descriptor.
        normalization.apply(recipient)?;
        drop(normalization);
        let pinned = pin(&directory, false)?;
        if private_contract(&directory)?.as_ref() != Some(&principals) {
            return Err(bad(
                "new output directory lost the exact recipient contract",
            ));
        }
        if std::fs::read_dir(&directory)?.next().is_some() {
            return Err(bad("new output directory changed during creation"));
        }
        let verification = recipient.verify_paths(&directory, std::slice::from_ref(&directory))?;
        if !verification.is_verified() {
            return Err(bad(
                "ordinary recipient cannot enumerate a newly protected output directory",
            ));
        }
        // No-delete pins retain every created ancestor through the whole walk.
        created.push(pinned);
    }
    Ok(())
}
pub(super) fn protect_child(path: &Path, directory: bool) -> io::Result<()> {
    let principals = parent_contract(path)?;
    let pinned = pin(path, true)?;
    apply(&pinned.file, directory, &principals)
}
pub(super) fn create_private_file(path: &Path) -> io::Result<File> {
    let _ancestors = pin_ancestors(path)?;
    let principals = parent_contract(path)?;
    create_with_principals(path, &principals)
}
pub(super) fn create_producer_file(path: &Path) -> io::Result<File> {
    let _ancestors = pin_ancestors(path)?;
    let mut principals = vec![user(CURRENT_TOKEN)?, "S-1-5-18".into()];
    principals.sort();
    principals.dedup();
    create_with_principals(path, &principals)
}
fn create_with_principals(path: &Path, principals: &[String]) -> io::Result<File> {
    let descriptor = Descriptor::parse(&sddl(principals, false))?;
    let attributes = SECURITY_ATTRIBUTES {
        nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
        lpSecurityDescriptor: descriptor.0,
        bInheritHandle: 0,
    };
    let name = wide(path);
    let handle = unsafe {
        CreateFileW(
            name.as_ptr(),
            GENERIC_READ | GENERIC_WRITE,
            FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
            &attributes,
            CREATE_NEW,
            FILE_ATTRIBUTE_NORMAL,
            0,
        )
    };
    if handle == INVALID_HANDLE_VALUE {
        return Err(error());
    }
    // SAFETY: CreateFileW returned a new owned handle, transferred once to File.
    Ok(unsafe { File::from_raw_handle(handle as *mut core::ffi::c_void) })
}

pub(super) struct Handoff {
    pipe: Handle,
    request: String,
    sid: String,
    bundle: PathBuf,
}
fn absolute(path: &Path) -> io::Result<PathBuf> {
    if !path.is_absolute()
        || path
            .components()
            .any(|c| matches!(c, std::path::Component::ParentDir))
    {
        return Err(bad(
            "recipient destination must be an exact absolute path without parent components",
        ));
    }
    reject_reparse_ancestors(path)?;
    Ok(path.to_path_buf())
}
fn encode(bytes: &[u8]) -> String {
    bytes.iter().map(|v| format!("{v:02x}")).collect()
}
fn decode(value: &str) -> io::Result<Vec<u8>> {
    if value.len() > 8192
        || !value.len().is_multiple_of(2)
        || !value.bytes().all(|b| b.is_ascii_hexdigit())
    {
        return Err(bad(
            "recipient request exceeds bounds or has invalid encoding",
        ));
    }
    (0..value.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&value[i..i + 2], 16).map_err(io::Error::other))
        .collect()
}
fn message(request: &str) -> io::Result<Value> {
    let bytes = decode(request)?;
    serde_json::from_slice(&bytes).map_err(io::Error::other)
}
pub(super) fn handoff(sid: &str, bundle: &Path) -> io::Result<RecipientHandoff> {
    let parsed = Sid::parse(sid)?;
    let normalized = sid_string(parsed.0)?;
    if normalized != sid || (!sid.starts_with("S-1-5-21-") && !sid.starts_with("S-1-12-1-")) {
        return Err(bad("recipient must be an exact individual account SID"));
    }
    individual(sid)?;
    let bundle = absolute(bundle)?;
    let mut entropy = [0_u8; 16];
    getrandom::fill(&mut entropy).map_err(io::Error::other)?;
    let name = format!("\\\\.\\pipe\\fragcap-output-recipient-{}", encode(&entropy));
    let request = encode(
        &serde_json::to_vec(&json!({"version":1,"pipe":name,"sid":sid,"bundle":bundle}))
            .map_err(io::Error::other)?,
    );
    if request.len() > 8192 {
        return Err(bad("recipient request exceeds bounds"));
    }
    let producer = user(CURRENT_TOKEN)?;
    let descriptor = Descriptor::parse(&format!(
        "D:P(A;;FA;;;{producer})(A;;FA;;;SY)(A;;0x00120183;;;{sid})"
    ))?;
    let attributes = SECURITY_ATTRIBUTES {
        nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
        lpSecurityDescriptor: descriptor.0,
        bInheritHandle: 0,
    };
    let wide_name = wide(&name);
    let pipe = unsafe {
        CreateNamedPipeW(
            wide_name.as_ptr(),
            PIPE_ACCESS_DUPLEX | FILE_FLAG_FIRST_PIPE_INSTANCE,
            PIPE_TYPE_MESSAGE | PIPE_READMODE_MESSAGE | PIPE_NOWAIT | PIPE_REJECT_REMOTE_CLIENTS,
            1,
            4096,
            4096,
            0,
            &attributes,
        )
    };
    if pipe == INVALID_HANDLE_VALUE {
        return Err(error());
    }
    Ok(RecipientHandoff {
        request_id: request.clone(),
        platform: Handoff {
            pipe: Handle(pipe),
            request,
            sid: sid.into(),
            bundle,
        },
    })
}
fn deadline(timeout: Duration) -> Instant {
    Instant::now() + timeout.min(Duration::from_secs(60))
}
fn retry(deadline: Instant) -> io::Result<()> {
    if Instant::now() >= deadline {
        return Err(io::Error::new(
            io::ErrorKind::TimedOut,
            "ordinary recipient handoff timed out before session effects",
        ));
    }
    thread::sleep(Duration::from_millis(20));
    Ok(())
}
fn read_message(pipe: isize, deadline: Instant) -> io::Result<Vec<u8>> {
    loop {
        let mut data = vec![0_u8; 4096];
        let mut read = 0;
        let ok = unsafe {
            ReadFile(
                pipe,
                data.as_mut_ptr().cast(),
                4096,
                &mut read,
                ptr::null_mut(),
            )
        };
        if ok != 0 && read > 0 {
            data.truncate(read as usize);
            return Ok(data);
        }
        let code = unsafe { GetLastError() };
        if ok == 0 && code != ERROR_NO_DATA && code != ERROR_PIPE_LISTENING {
            return Err(io::Error::from_raw_os_error(code as i32));
        }
        retry(deadline)?;
    }
}
fn write_message(pipe: isize, bytes: &[u8]) -> io::Result<()> {
    if bytes.len() > 4096 {
        return Err(bad("recipient message exceeds limit"));
    }
    let mut count = 0;
    if unsafe {
        WriteFile(
            pipe,
            bytes.as_ptr().cast(),
            bytes.len() as u32,
            &mut count,
            ptr::null_mut(),
        )
    } == 0
    {
        return Err(error());
    }
    if count as usize != bytes.len() {
        return Err(bad("recipient message was not written completely"));
    }
    Ok(())
}
pub(super) fn accept(handoff: Handoff, timeout: Duration) -> io::Result<OutputRecipient> {
    isolated(move || {
        let end = deadline(timeout);
        loop {
            if unsafe { ConnectNamedPipe(handoff.pipe.0, ptr::null_mut()) } != 0 {
                break;
            }
            let code = unsafe { GetLastError() };
            if code == ERROR_PIPE_CONNECTED {
                break;
            }
            if code != ERROR_PIPE_LISTENING && code != ERROR_NO_DATA {
                return Err(io::Error::from_raw_os_error(code as i32));
            }
            retry(end)?;
        }
        let request = read_message(handoff.pipe.0, end)?;
        if request != decode(&handoff.request)? {
            return Err(bad("recipient handoff request/destination mismatch"));
        }
        if unsafe { ImpersonateNamedPipeClient(handoff.pipe.0) } == 0 {
            return Err(error());
        }
        let result = (|| -> io::Result<_> {
            let mut token = 0;
            if unsafe { OpenThreadToken(CURRENT_THREAD, RIGHTS, 1, &mut token) } == 0 {
                return Err(error());
            }
            let token = Handle(token);
            if user(token.0)? != handoff.sid {
                return Err(bad("recipient helper identity does not match selected SID"));
            }
            ordinary(token.0)?;
            let session = session(token.0)?;
            let token = duplicate(token.0)?;
            Ok((token, session))
        })();
        if unsafe { RevertToSelf() } == 0 {
            return Err(bad("failed to restore recipient handoff thread"));
        }
        let (token, session) = result?;
        absolute(&handoff.bundle)?;
        write_message(handoff.pipe.0, b"accepted")?;
        Ok(OutputRecipient {
            sid: handoff.sid,
            producer_sid: user(CURRENT_TOKEN)?,
            proof_kind: "explicit-authenticated-helper".into(),
            platform: Arc::new(RecipientToken {
                token,
                session,
                desktop: false,
            }),
        })
    })
}
pub(super) fn authorize(request: &str, bundle: &Path, timeout: Duration) -> io::Result<()> {
    ordinary(CURRENT_EFFECTIVE_TOKEN)?;
    let value = message(request)?;
    let bundle = absolute(bundle)?;
    if value["version"] != 1
        || value["sid"].as_str() != Some(user(CURRENT_EFFECTIVE_TOKEN)?.as_str())
        || value["bundle"].as_str() != bundle.to_str()
    {
        return Err(bad(
            "ordinary helper identity or exact destination mismatch",
        ));
    }
    let name = value["pipe"]
        .as_str()
        .ok_or_else(|| bad("recipient request has no pipe"))?;
    let suffix = name
        .strip_prefix("\\\\.\\pipe\\fragcap-output-recipient-")
        .ok_or_else(|| bad("recipient request is not a local fragcap pipe"))?;
    if suffix.len() != 32 || !suffix.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(bad("recipient pipe identity is invalid"));
    }
    let end = deadline(timeout);
    let name = wide(name);
    let handle = loop {
        let handle = unsafe {
            CreateFileW(
                name.as_ptr(),
                0x00120183,
                0,
                ptr::null(),
                OPEN_EXISTING,
                SECURITY_SQOS_PRESENT | SECURITY_IMPERSONATION,
                0,
            )
        };
        if handle != INVALID_HANDLE_VALUE {
            break Handle(handle);
        }
        let code = unsafe { GetLastError() };
        if code != ERROR_PIPE_BUSY && code != ERROR_FILE_NOT_FOUND {
            return Err(io::Error::from_raw_os_error(code as i32));
        }
        retry(end)?;
    };
    let mode = PIPE_READMODE_MESSAGE | PIPE_NOWAIT;
    if unsafe { SetNamedPipeHandleState(handle.0, &mode, ptr::null_mut(), ptr::null_mut()) } == 0 {
        return Err(error());
    }
    write_message(handle.0, &decode(request)?)?;
    if read_message(handle.0, end)? != b"accepted" {
        return Err(bad("recipient server did not accept exact handoff"));
    }
    Ok(())
}
pub(super) fn denied_user_fixture(path: &Path, directory: bool) -> io::Result<()> {
    let producer = user(CURRENT_TOKEN)?;
    let descriptor = Descriptor::parse(&format!(
        "D:P(A;;FA;;;{producer})(A;;FA;;;SY)(A;;FRFX;;;AN)(A;;FRFX;;;WD)"
    ))?;
    let pin = pin(path, true)?;
    let _ = directory;
    let code = unsafe {
        SetSecurityInfo(
            pin.file.as_raw_handle() as isize,
            SE_FILE_OBJECT,
            DACL_SECURITY_INFORMATION | PROTECTED_DACL_SECURITY_INFORMATION,
            ptr::null_mut(),
            ptr::null_mut(),
            descriptor.dacl()?,
            ptr::null_mut(),
        )
    };
    if code == 0 {
        Ok(())
    } else {
        Err(io::Error::from_raw_os_error(code as i32))
    }
}
pub(super) fn verify_denied_user(
    root: &Path,
    paths: &[PathBuf],
) -> io::Result<BundleAccessVerification> {
    let root = root.to_path_buf();
    let paths = paths.to_vec();
    isolated(move || {
        let owned = current_owned()?;
        let user_info = info(owned.0, TokenUser)?;
        let user_entry = unsafe { (*user_info.as_ptr().cast::<TOKEN_USER>()).User };
        let anonymous = Sid::parse("S-1-5-7")?;
        let restriction = SID_AND_ATTRIBUTES {
            Sid: anonymous.0,
            Attributes: 0,
        };
        let mut token = 0;
        if unsafe {
            CreateRestrictedToken(
                owned.0,
                DISABLE_MAX_PRIVILEGE,
                1,
                &user_entry,
                0,
                ptr::null(),
                1,
                &restriction,
                &mut token,
            )
        } == 0
        {
            return Err(error());
        }
        let token = Handle(token);
        if unsafe { ImpersonateLoggedOnUser(token.0) } == 0 {
            return Err(error());
        }
        let mut result = verify_current(
            &root,
            &paths,
            &user(CURRENT_TOKEN)?,
            "controlled-denied-user-equivalent",
        );
        result.limitations.push("The synthetic token denies its individual user SID and restricts access to the anonymous SID; it is not a different authenticated human account.".into());
        if unsafe { RevertToSelf() } == 0 {
            return Err(bad("failed to restore anonymous verification thread"));
        }
        Ok(result)
    })
}
pub(super) fn group_denied_fixture() -> io::Result<OutputRecipient> {
    let mut result = recipient(false)?;
    // Restrict the already validated ordinary context. Restricting the elevated
    // producer instead retains its high integrity level on hosted runners.
    let ordinary_token = result.platform.token.0;
    ordinary(ordinary_token)?;
    let groups = info(ordinary_token, TokenGroups)?;
    let group_info = unsafe { &*groups.as_ptr().cast::<TOKEN_GROUPS>() };
    let groups = unsafe {
        std::slice::from_raw_parts(group_info.Groups.as_ptr(), group_info.GroupCount as usize)
    };
    let disabled = groups
        .iter()
        .filter(|g| {
            g.Attributes & 0xc0000000 == 0xc0000000
                || unsafe { IsWellKnownSid(g.Sid, WinBuiltinAdministratorsSid) != 0 }
        })
        .copied()
        .collect::<Vec<_>>();
    let mut token = 0;
    if unsafe {
        CreateRestrictedToken(
            ordinary_token,
            DISABLE_MAX_PRIVILEGE | LUA_TOKEN,
            disabled.len() as u32,
            disabled.as_ptr(),
            0,
            ptr::null(),
            0,
            ptr::null(),
            &mut token,
        )
    } == 0
    {
        return Err(error());
    }
    let token = Handle(token);
    ordinary(token.0)?;
    result.proof_kind = "controlled-group-denied-equivalent".into();
    result.platform = Arc::new(RecipientToken {
        token,
        session: result.platform.session,
        desktop: false,
    });
    Ok(result)
}
pub(super) fn fixture_logon_sid() -> io::Result<String> {
    let groups = info(CURRENT_TOKEN, TokenGroups)?;
    let group_info = unsafe { &*groups.as_ptr().cast::<TOKEN_GROUPS>() };
    let groups = unsafe {
        std::slice::from_raw_parts(group_info.Groups.as_ptr(), group_info.GroupCount as usize)
    };
    let group = groups
        .iter()
        .find(|g| g.Attributes & 0xc0000000 == 0xc0000000)
        .ok_or_else(|| bad("controlled token has no logon group SID"))?;
    sid_string(group.Sid)
}
pub(super) fn fixture_read(recipient: &OutputRecipient, path: &Path) -> io::Result<Vec<u8>> {
    let path = path.to_path_buf();
    with_token(recipient.platform.clone(), move || {
        let file = File::open(path)?;
        let mut bytes = Vec::new();
        file.take(1024 * 1024 + 1).read_to_end(&mut bytes)?;
        if bytes.len() > 1024 * 1024 {
            return Err(bad("synthetic fixture read exceeds limit"));
        }
        Ok(bytes)
    })
}
pub(super) fn fixture_enumerate(
    recipient: &OutputRecipient,
    path: &Path,
) -> io::Result<Vec<PathBuf>> {
    let path = path.to_path_buf();
    with_token(recipient.platform.clone(), move || {
        std::fs::read_dir(path)?
            .take(257)
            .map(|e| e.map(|v| v.path()))
            .collect()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn group_denied_fixture_retains_ordinary_identity_and_denies_the_logon_group() {
        let base = recipient(false).unwrap();
        let fixture = group_denied_fixture().unwrap();
        ordinary(fixture.platform.token.0).unwrap();
        assert_eq!(user(fixture.platform.token.0).unwrap(), base.sid);
        assert_eq!(fixture.platform.session, base.platform.session);
        assert_eq!(fixture.proof_kind, "controlled-group-denied-equivalent");
        let logon_sid = fixture_logon_sid().unwrap();
        let data = info(fixture.platform.token.0, TokenGroups).unwrap();
        let group_info = unsafe { &*data.as_ptr().cast::<TOKEN_GROUPS>() };
        let groups = unsafe {
            std::slice::from_raw_parts(group_info.Groups.as_ptr(), group_info.GroupCount as usize)
        };
        let logon = groups
            .iter()
            .find(|group| sid_string(group.Sid).unwrap() == logon_sid)
            .expect("ordinary fixture must retain its denied logon SID");
        // SE_GROUP_USE_FOR_DENY_ONLY is 0x10; SE_GROUP_ENABLED is 0x4.
        assert_ne!(logon.Attributes & 0x10, 0);
        assert_eq!(logon.Attributes & 0x4, 0);
        for group in groups {
            if unsafe { IsWellKnownSid(group.Sid, WinBuiltinAdministratorsSid) } != 0 {
                assert_ne!(group.Attributes & 0x10, 0);
                assert_eq!(group.Attributes & 0x4, 0);
            }
        }
    }
    #[test]
    fn real_local_handoff_retains_authenticated_ordinary_context() {
        let root = tempfile::tempdir().unwrap();
        let bundle = root.path().join("retained");
        let recipient = recipient(false).unwrap();
        let handoff = handoff(recipient.sid(), &bundle).unwrap();
        let request = handoff.request_id().to_string();
        let server = thread::spawn(move || handoff.accept(Duration::from_secs(3)));
        let path = bundle.clone();
        with_token(recipient.platform.clone(), move || {
            authorize(&request, &path, Duration::from_secs(3))
        })
        .unwrap();
        let accepted = server.join().unwrap().unwrap();
        assert_eq!(accepted.sid(), recipient.sid());
        assert_eq!(accepted.proof_kind(), "explicit-authenticated-helper");
        super::super::super::artifacts::prepare_bundle_for_recipient(&bundle, &accepted).unwrap();
        let report = super::super::verify_bundle_access(&bundle, &accepted).unwrap();
        assert!(report.is_verified(), "{report:?}");
    }
    #[test]
    fn wrong_path_and_missing_helper_refuse_before_output_creation() {
        let root = tempfile::tempdir().unwrap();
        let bundle = root.path().join("retained");
        let recipient = recipient(false).unwrap();
        let handoff = handoff(recipient.sid(), &bundle).unwrap();
        let request = handoff.request_id().to_string();
        let wrong = root.path().join("wrong");
        assert!(with_token(recipient.platform.clone(), move || authorize(
            &request,
            &wrong,
            Duration::from_millis(40)
        ))
        .is_err());
        assert_eq!(
            handoff
                .accept(Duration::from_millis(40))
                .unwrap_err()
                .kind(),
            io::ErrorKind::TimedOut
        );
        assert!(!bundle.exists());
    }
    #[test]
    fn group_sid_and_malformed_request_are_refused() {
        let root = tempfile::tempdir().unwrap();
        assert!(handoff("S-1-5-32-544", root.path()).is_err());
        assert!(decode("\u{00e9}\u{00e9}").is_err());
        assert!(decode("00zz").is_err());
    }
    #[test]
    fn authenticated_helper_with_mismatched_expected_identity_is_refused() {
        let root = tempfile::tempdir().unwrap();
        let bundle = root.path().join("retained");
        let recipient = recipient(false).unwrap();
        let mut handoff = handoff(recipient.sid(), &bundle).unwrap();
        let request = handoff.request_id().to_string();
        handoff.platform.sid = "S-1-5-21-1-2-3-999999".into();
        let server = thread::spawn(move || handoff.accept(Duration::from_secs(3)));
        let path = bundle.clone();
        assert!(with_token(recipient.platform.clone(), move || authorize(
            &request,
            &path,
            Duration::from_secs(3)
        ))
        .is_err());
        assert_eq!(
            server.join().unwrap().unwrap_err().kind(),
            io::ErrorKind::PermissionDenied
        );
        assert!(!bundle.exists());
    }
    #[test]
    fn full_elevated_token_is_not_ordinary_helper_authority() {
        let ordinary_recipient = recipient(false).unwrap();
        ordinary(ordinary_recipient.platform.token.0).unwrap();
        let elevation_type = scalar(CURRENT_TOKEN, TokenElevationType).unwrap();
        if scalar(CURRENT_TOKEN, TokenElevation).unwrap() != 0 {
            assert!(ordinary(CURRENT_TOKEN).is_err());
        } else if elevation_type == TokenElevationTypeLimited as u32 {
            let linked = info(CURRENT_TOKEN, TokenLinkedToken).unwrap();
            let linked =
                Handle(unsafe { (*linked.as_ptr().cast::<TOKEN_LINKED_TOKEN>()).LinkedToken });
            assert_eq!(
                scalar(linked.0, TokenElevationType).unwrap(),
                TokenElevationTypeFull as u32
            );
            assert!(ordinary(linked.0).is_err());
        } else {
            assert_eq!(elevation_type, TokenElevationTypeDefault as u32);
            ordinary(CURRENT_TOKEN).unwrap();
            println!("ordinary default token has no elevated linked fixture; ordinary acceptance verified, elevated rejection is not exercised on this host");
        }
    }
}
