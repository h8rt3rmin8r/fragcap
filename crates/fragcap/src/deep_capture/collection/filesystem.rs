// SPDX-License-Identifier: Apache-2.0

//! Filesystem identities and Windows object-handle deletion capabilities.

use serde_json::{json, Value};
use std::fs::File;
#[cfg(windows)]
use std::io::Write;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use super::invalid;

pub(super) struct Object {
    pub path: PathBuf,
    pub identity: Value,
    pub directory: bool,
    pub bytes: u64,
    pub modified: u64,
    file: File,
}

impl Object {
    pub fn open(path: &Path, deleting: bool) -> io::Result<Self> {
        #[cfg(windows)]
        let file = {
            use std::os::windows::fs::OpenOptionsExt;
            use windows_sys::Win32::Storage::FileSystem::{
                FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OPEN_REPARSE_POINT, FILE_SHARE_READ,
                FILE_SHARE_WRITE,
            };
            // Share neither replacement nor deletion; all reading uses this handle.
            // Ancestor guards are held separately, never duplicated below a DELETE pin.
            let access = 0x8000_0000 | 0x0002_0000 | if deleting { 0x0001_0000 } else { 0 };
            std::fs::OpenOptions::new()
                .access_mode(access)
                .share_mode(if deleting {
                    FILE_SHARE_READ
                } else {
                    FILE_SHARE_READ | FILE_SHARE_WRITE
                })
                .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
                .open(path)?
        };
        #[cfg(not(windows))]
        let file = {
            let _ = deleting;
            let metadata = std::fs::symlink_metadata(path)?;
            if metadata.file_type().is_symlink() {
                return Err(invalid("collection refuses symlinks"));
            }
            File::open(path)?
        };
        let (identity, directory, bytes, modified) = snapshot(&file)?;
        Ok(Self {
            path: path.into(),
            identity,
            directory,
            bytes,
            modified,
            file,
        })
    }

    #[cfg(windows)]
    pub fn open_for_append(path: &Path) -> io::Result<Self> {
        use std::os::windows::fs::OpenOptionsExt;
        use windows_sys::Win32::Storage::FileSystem::{
            FILE_FLAG_OPEN_REPARSE_POINT, FILE_SHARE_READ,
        };
        let file = std::fs::OpenOptions::new()
            .access_mode(0x8000_0000 | 0x4000_0000 | 0x0002_0000)
            .share_mode(FILE_SHARE_READ)
            .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
            .open(path)?;
        let (identity, directory, bytes, modified) = snapshot(&file)?;
        if directory {
            return Err(invalid("retirement progress target is not a regular file"));
        }
        Ok(Self {
            path: path.into(),
            identity,
            directory,
            bytes,
            modified,
            file,
        })
    }

    #[cfg(windows)]
    pub fn append_at(&mut self, prefix: u64, bytes: &[u8]) -> io::Result<()> {
        self.verify()?;
        // Discard only an uncommitted torn final line under this exclusive writer.
        self.file.set_len(prefix)?;
        self.file.seek(SeekFrom::Start(prefix))?;
        self.file.write_all(bytes)?;
        self.file.sync_all()
    }

    pub fn read(&mut self, limit: u64) -> io::Result<Vec<u8>> {
        if self.directory || self.bytes > limit {
            return Err(invalid(
                "collection provenance exceeds its bound or is not a file",
            ));
        }
        self.file.seek(SeekFrom::Start(0))?;
        let mut bytes = Vec::new();
        (&mut self.file).take(limit + 1).read_to_end(&mut bytes)?;
        if bytes.len() as u64 > limit {
            return Err(invalid("collection provenance exceeds its bound"));
        }
        Ok(bytes)
    }

    pub fn verify(&self) -> io::Result<()> {
        let (identity, directory, bytes, modified) = snapshot(&self.file)?;
        if identity != self.identity
            || directory != self.directory
            || bytes != self.bytes
            || (!directory && modified != self.modified)
        {
            return Err(invalid("collection object changed after preview"));
        }
        Ok(())
    }

    #[cfg(windows)]
    pub fn remove(self) -> io::Result<()> {
        #[cfg(windows)]
        {
            use std::os::windows::io::AsRawHandle;
            use windows_sys::Win32::Storage::FileSystem::{
                FileDispositionInfo, SetFileInformationByHandle, FILE_DISPOSITION_INFO,
            };
            self.verify()?;
            let disposition = FILE_DISPOSITION_INFO { DeleteFileA: 1 };
            if unsafe {
                SetFileInformationByHandle(
                    self.file.as_raw_handle() as isize,
                    FileDispositionInfo,
                    &disposition as *const _ as *const _,
                    std::mem::size_of::<FILE_DISPOSITION_INFO>() as u32,
                )
            } == 0
            {
                return Err(io::Error::last_os_error());
            }
            let path = self.path.clone();
            drop(self);
            match std::fs::symlink_metadata(&path) {
                Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
                Err(error) => Err(error),
                Ok(_) => Err(invalid(
                    "collection disposition did not remove the exact selected object",
                )),
            }
        }
    }
}

#[cfg(windows)]
fn store_descriptor() -> io::Result<String> {
    let producer = super::super::OutputRecipient::current_account()?;
    let sid = producer.producer_sid();
    Ok(format!("O:{sid}D:P(A;OICI;FA;;;{sid})(A;OICI;FA;;;SY)"))
}

#[cfg(windows)]
pub(super) fn create_private_store(path: &Path) -> io::Result<()> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Security::Authorization::ConvertStringSecurityDescriptorToSecurityDescriptorW;
    use windows_sys::Win32::Security::SECURITY_ATTRIBUTES;
    use windows_sys::Win32::Storage::FileSystem::CreateDirectoryW;
    use windows_sys::Win32::System::Memory::LocalFree;
    let text = store_descriptor()?
        .encode_utf16()
        .chain(Some(0))
        .collect::<Vec<_>>();
    let mut descriptor = std::mem::MaybeUninit::<*mut core::ffi::c_void>::uninit();
    if unsafe {
        ConvertStringSecurityDescriptorToSecurityDescriptorW(
            text.as_ptr(),
            1,
            descriptor.as_mut_ptr(),
            std::ptr::null_mut(),
        )
    } == 0
    {
        return Err(io::Error::last_os_error());
    }
    // The documented output is initialized only on success; still reject null.
    let descriptor = unsafe { descriptor.assume_init() };
    if descriptor.is_null() {
        return Err(invalid("private store descriptor output is null"));
    }
    let security = SECURITY_ATTRIBUTES {
        nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
        lpSecurityDescriptor: descriptor,
        bInheritHandle: 0,
    };
    let name = path
        .as_os_str()
        .encode_wide()
        .chain(Some(0))
        .collect::<Vec<_>>();
    let created = unsafe { CreateDirectoryW(name.as_ptr(), &security) };
    let failure = if created == 0 {
        Some(io::Error::last_os_error())
    } else {
        None
    };
    unsafe { LocalFree(descriptor as isize) };
    if let Some(error) = failure {
        if error.kind() != io::ErrorKind::AlreadyExists {
            return Err(error);
        }
    }
    validate_private_store(path)
}

#[cfg(windows)]
pub(super) fn validate_private_store(path: &Path) -> io::Result<()> {
    use std::os::windows::io::AsRawHandle;
    use windows_sys::Win32::Security::Authorization::{GetSecurityInfo, SE_FILE_OBJECT};
    use windows_sys::Win32::Security::{DACL_SECURITY_INFORMATION, OWNER_SECURITY_INFORMATION};
    use windows_sys::Win32::System::Memory::LocalFree;
    let object = Object::open(path, false)?;
    if !object.directory {
        return Err(invalid(
            "retirement store must be a producer-private directory",
        ));
    }
    let producer = super::super::OutputRecipient::current_account()?;
    let mut descriptor = std::mem::MaybeUninit::<*mut core::ffi::c_void>::uninit();
    let flags = OWNER_SECURITY_INFORMATION | DACL_SECURITY_INFORMATION;
    let result = unsafe {
        GetSecurityInfo(
            object.file.as_raw_handle() as isize,
            SE_FILE_OBJECT,
            flags,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            descriptor.as_mut_ptr(),
        )
    };
    if result != 0 {
        return Err(io::Error::from_raw_os_error(result as i32));
    }
    let descriptor = unsafe { descriptor.assume_init() };
    if descriptor.is_null() {
        return Err(invalid("private store security descriptor output is null"));
    }
    let result = validate_store_descriptor(descriptor, producer.producer_sid());
    unsafe { LocalFree(descriptor as isize) };
    result
}

#[cfg(windows)]
fn validate_store_descriptor(
    descriptor: *mut core::ffi::c_void,
    producer_sid: &str,
) -> io::Result<()> {
    use std::mem::{offset_of, MaybeUninit};
    use windows_sys::Win32::Security::Authorization::ConvertStringSidToSidW;
    use windows_sys::Win32::Security::{
        EqualSid, GetAce, GetLengthSid, GetSecurityDescriptorControl, GetSecurityDescriptorDacl,
        GetSecurityDescriptorOwner, IsValidAcl, IsValidSecurityDescriptor, IsValidSid,
        ACCESS_ALLOWED_ACE, ACE_HEADER, ACL,
    };
    use windows_sys::Win32::Storage::FileSystem::FILE_ALL_ACCESS;
    use windows_sys::Win32::System::Memory::LocalFree;
    use windows_sys::Win32::System::SystemServices::SE_DACL_PROTECTED;

    struct LocalSid(*mut core::ffi::c_void);
    impl Drop for LocalSid {
        fn drop(&mut self) {
            unsafe { LocalFree(self.0 as isize) };
        }
    }
    fn sid(text: &str) -> io::Result<LocalSid> {
        let text = text.encode_utf16().chain(Some(0)).collect::<Vec<_>>();
        let mut output = MaybeUninit::<*mut core::ffi::c_void>::uninit();
        if unsafe { ConvertStringSidToSidW(text.as_ptr(), output.as_mut_ptr()) } == 0 {
            return Err(io::Error::last_os_error());
        }
        let output = unsafe { output.assume_init() };
        if output.is_null() {
            return Err(invalid("private store expected SID output is null"));
        }
        let sid = LocalSid(output);
        if unsafe { IsValidSid(sid.0) } == 0 {
            return Err(invalid("private store expected SID is invalid"));
        }
        Ok(sid)
    }
    let refused = || {
        invalid(
            "retirement store ownership or permissions are not the exact producer-private contract",
        )
    };
    if descriptor.is_null() || unsafe { IsValidSecurityDescriptor(descriptor) } == 0 {
        return Err(refused());
    }
    let producer = sid(producer_sid)?;
    let system = sid("S-1-5-18")?;
    let mut owner = MaybeUninit::<*mut core::ffi::c_void>::uninit();
    let mut defaulted = MaybeUninit::<i32>::uninit();
    if unsafe { GetSecurityDescriptorOwner(descriptor, owner.as_mut_ptr(), defaulted.as_mut_ptr()) }
        == 0
    {
        return Err(io::Error::last_os_error());
    }
    let owner = unsafe { owner.assume_init() };
    if owner.is_null()
        || unsafe { IsValidSid(owner) } == 0
        || unsafe { EqualSid(owner, producer.0) } == 0
    {
        return Err(refused());
    }
    let mut control = MaybeUninit::<u16>::uninit();
    let mut revision = MaybeUninit::<u32>::uninit();
    if unsafe {
        GetSecurityDescriptorControl(descriptor, control.as_mut_ptr(), revision.as_mut_ptr())
    } == 0
    {
        return Err(io::Error::last_os_error());
    }
    if u32::from(unsafe { control.assume_init() }) & SE_DACL_PROTECTED == 0 {
        return Err(refused());
    }
    let mut present = MaybeUninit::<i32>::uninit();
    let mut dacl = MaybeUninit::<*mut ACL>::uninit();
    let mut defaulted = MaybeUninit::<i32>::uninit();
    if unsafe {
        GetSecurityDescriptorDacl(
            descriptor,
            present.as_mut_ptr(),
            dacl.as_mut_ptr(),
            defaulted.as_mut_ptr(),
        )
    } == 0
    {
        return Err(io::Error::last_os_error());
    }
    let dacl = unsafe { dacl.assume_init() };
    if unsafe { present.assume_init() } == 0 || dacl.is_null() || unsafe { IsValidAcl(dacl) } == 0 {
        return Err(refused());
    }
    // Compare the security contract, not SDDL rendering. Windows can render a
    // numeric SID as LA/BA/SY and protected ACE order is not part of ownership.
    if unsafe { (*dacl).AceCount } != 2 {
        return Err(refused());
    }
    let mut producer_seen = false;
    let mut system_seen = false;
    for index in 0..2 {
        let mut ace = MaybeUninit::<*mut core::ffi::c_void>::uninit();
        if unsafe { GetAce(dacl, index, ace.as_mut_ptr()) } == 0 {
            return Err(io::Error::last_os_error());
        }
        let ace = unsafe { ace.assume_init() };
        if ace.is_null() {
            return Err(refused());
        }
        let header = unsafe { &*ace.cast::<ACE_HEADER>() };
        let sid_offset = offset_of!(ACCESS_ALLOWED_ACE, SidStart);
        if header.AceType != 0
            || header.AceFlags != 3
            || usize::from(header.AceSize) < sid_offset + 8
        {
            return Err(refused());
        }
        let allowed = unsafe { &*ace.cast::<ACCESS_ALLOWED_ACE>() };
        if allowed.Mask != FILE_ALL_ACCESS {
            return Err(refused());
        }
        let principal = (&allowed.SidStart as *const u32).cast_mut().cast();
        if unsafe { IsValidSid(principal) } == 0
            || unsafe { GetLengthSid(principal) } as usize
                > usize::from(header.AceSize) - sid_offset
        {
            return Err(refused());
        }
        if unsafe { EqualSid(principal, producer.0) } != 0 && !producer_seen {
            producer_seen = true;
        } else if unsafe { EqualSid(principal, system.0) } != 0 && !system_seen {
            system_seen = true;
        } else {
            return Err(refused());
        }
    }
    // A SYSTEM producer creates the same two exact ACEs; no other duplicate
    // principal may stand in for the separately required SYSTEM grant.
    if !producer_seen || !system_seen {
        return Err(refused());
    }
    Ok(())
}

#[cfg(all(test, windows))]
mod descriptor_tests {
    use super::*;

    fn check(sddl: &str, producer: &str) -> io::Result<()> {
        use windows_sys::Win32::Security::Authorization::ConvertStringSecurityDescriptorToSecurityDescriptorW;
        use windows_sys::Win32::System::Memory::LocalFree;
        let text = sddl.encode_utf16().chain(Some(0)).collect::<Vec<_>>();
        let mut descriptor = std::mem::MaybeUninit::<*mut core::ffi::c_void>::uninit();
        assert_ne!(
            unsafe {
                ConvertStringSecurityDescriptorToSecurityDescriptorW(
                    text.as_ptr(),
                    1,
                    descriptor.as_mut_ptr(),
                    std::ptr::null_mut(),
                )
            },
            0
        );
        let descriptor = unsafe { descriptor.assume_init() };
        assert!(!descriptor.is_null());
        let result = validate_store_descriptor(descriptor, producer);
        unsafe { LocalFree(descriptor as isize) };
        result
    }

    #[test]
    fn numeric_producer_sid_and_windows_alias_are_the_same_exact_principal() {
        // Built-in Administrators is only a synthetic descriptor producer here;
        // production always compares against its actual token's individual SID.
        assert!(check("O:BAD:P(A;OICI;FA;;;BA)(A;OICI;FA;;;SY)", "S-1-5-32-544").is_ok());
    }

    #[test]
    fn exact_allow_ace_order_and_auto_inherit_control_do_not_change_contract() {
        let producer = super::super::super::OutputRecipient::current_account().unwrap();
        let sid = producer.producer_sid();
        assert!(check(
            &format!("O:{sid}D:PAI(A;OICI;FA;;;SY)(A;OICI;FA;;;{sid})"),
            sid
        )
        .is_ok());
    }

    #[test]
    fn equivalent_rendering_never_admits_extra_grants_wrong_owner_or_weaker_acl() {
        for descriptor in [
            "O:BAD:P(A;OICI;FA;;;BA)(A;OICI;FA;;;SY)(A;OICI;FA;;;WD)",
            "O:SYD:P(A;OICI;FA;;;BA)(A;OICI;FA;;;SY)",
            "O:BAD:(A;OICI;FA;;;BA)(A;OICI;FA;;;SY)",
            "O:BAD:P(A;OI;FA;;;BA)(A;OICI;FA;;;SY)",
            "O:BAD:P(A;OICI;GR;;;BA)(A;OICI;FA;;;SY)",
            "O:BAD:P(D;OICI;FA;;;BA)(A;OICI;FA;;;SY)",
            "O:BAD:P(A;OICIID;FA;;;BA)(A;OICI;FA;;;SY)",
            "O:BAD:P(A;OICI;FA;;;BA)(A;OICI;FA;;;BA)",
        ] {
            assert!(check(descriptor, "S-1-5-32-544").is_err(), "{descriptor}");
        }
    }

    #[test]
    fn system_producer_matches_its_two_exact_creation_aces() {
        assert!(check("O:SYD:P(A;OICI;FA;;;SY)(A;OICI;FA;;;SY)", "S-1-5-18").is_ok());
    }
}

pub(super) fn guard_ancestors(path: &Path) -> io::Result<Vec<File>> {
    let absolute = std::path::absolute(path)?;
    if absolute
        .components()
        .any(|c| matches!(c, std::path::Component::ParentDir))
    {
        return Err(invalid("collection path contains a parent component"));
    }
    let mut paths = absolute
        .parent()
        .into_iter()
        .flat_map(|p| p.ancestors())
        .collect::<Vec<_>>();
    paths.reverse();
    if paths.len() > 128 {
        return Err(invalid("collection ancestor population exceeds bound"));
    }
    let mut result = Vec::new();
    for ancestor in paths {
        #[cfg(windows)]
        let file = {
            use std::os::windows::fs::{MetadataExt, OpenOptionsExt};
            use windows_sys::Win32::Storage::FileSystem::{
                FILE_ATTRIBUTE_REPARSE_POINT, FILE_FLAG_BACKUP_SEMANTICS,
                FILE_FLAG_OPEN_REPARSE_POINT, FILE_SHARE_READ, FILE_SHARE_WRITE,
            };
            // A zero-access directory handle does not participate in the Windows
            // sharing check for rename. A real read access retains the ancestor.
            let file = std::fs::OpenOptions::new()
                .access_mode(0x8000_0000)
                .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE)
                .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
                .open(ancestor)?;
            let meta = file.metadata()?;
            if !meta.is_dir() || meta.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
                return Err(invalid(
                    "collection refuses reparse or non-directory ancestors",
                ));
            }
            file
        };
        #[cfg(not(windows))]
        let file = {
            let meta = std::fs::symlink_metadata(ancestor)?;
            if !meta.is_dir() || meta.file_type().is_symlink() {
                return Err(invalid(
                    "collection refuses linked or non-directory ancestors",
                ));
            }
            File::open(ancestor)?
        };
        result.push(file);
    }
    Ok(result)
}

fn snapshot(file: &File) -> io::Result<(Value, bool, u64, u64)> {
    #[cfg(windows)]
    {
        use std::os::windows::io::AsRawHandle;
        use windows_sys::Win32::Storage::FileSystem::{
            GetFileInformationByHandle, BY_HANDLE_FILE_INFORMATION, FILE_ATTRIBUTE_DIRECTORY,
            FILE_ATTRIBUTE_REPARSE_POINT,
        };
        let mut info: BY_HANDLE_FILE_INFORMATION = unsafe { std::mem::zeroed() };
        if unsafe { GetFileInformationByHandle(file.as_raw_handle() as isize, &mut info) } == 0 {
            return Err(io::Error::last_os_error());
        }
        if info.dwFileAttributes & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
            return Err(invalid("collection refuses pinned reparse objects"));
        }
        let directory = info.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY != 0;
        if !directory && info.nNumberOfLinks != 1 {
            return Err(invalid("collection refuses hard-link aliases"));
        }
        let identity = json!({"volume":info.dwVolumeSerialNumber,"index":(u64::from(info.nFileIndexHigh)<<32)|u64::from(info.nFileIndexLow),"directory":directory});
        let bytes = if directory {
            0
        } else {
            (u64::from(info.nFileSizeHigh) << 32) | u64::from(info.nFileSizeLow)
        };
        let modified = (u64::from(info.ftLastWriteTime.dwHighDateTime) << 32)
            | u64::from(info.ftLastWriteTime.dwLowDateTime);
        Ok((identity, directory, bytes, modified))
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        let meta = file.metadata()?;
        if !(meta.is_file() || meta.is_dir()) || (meta.is_file() && meta.nlink() != 1) {
            return Err(invalid("collection refuses non-regular or aliased objects"));
        }
        let modified = meta
            .modified()?
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(io::Error::other)?
            .as_nanos()
            .min(u128::from(u64::MAX)) as u64;
        Ok((
            json!({"volume":meta.dev(),"index":meta.ino(),"directory":meta.is_dir()}),
            meta.is_dir(),
            if meta.is_dir() { 0 } else { meta.len() },
            modified,
        ))
    }
    #[cfg(not(any(windows, unix)))]
    {
        let _ = file;
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "filesystem identity unavailable",
        ))
    }
}

#[cfg(windows)]
pub(super) fn file_identity(file: &File) -> io::Result<Value> {
    Ok(snapshot(file)?.0)
}
