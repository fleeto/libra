//! Windows capture filesystem capabilities. Every component is opened without
//! following reparse points. Ordinary directory pins deny write/delete sharing;
//! private mutation directories use handle-relative operations and deny deletion.

use std::{
    ffi::{OsStr, OsString},
    fs::File,
    io,
    os::windows::{
        ffi::{OsStrExt, OsStringExt},
        io::{AsRawHandle, FromRawHandle, OwnedHandle},
    },
    path::{Component, Path, PathBuf, Prefix},
    ptr::{null, null_mut},
};

use windows_sys::{
    Wdk::{
        Foundation::OBJECT_ATTRIBUTES,
        Storage::FileSystem::{
            FILE_CREATE, FILE_DIRECTORY_FILE, FILE_DISPOSITION_INFORMATION,
            FILE_NON_DIRECTORY_FILE, FILE_OPEN, FILE_OPEN_REPARSE_POINT, FILE_RENAME_INFORMATION,
            FILE_SYNCHRONOUS_IO_NONALERT, FileDispositionInformation, FileRenameInformation,
            NtCreateFile, NtSetInformationFile,
        },
    },
    Win32::{
        Foundation::{
            ERROR_LOCK_VIOLATION, GENERIC_READ, GENERIC_WRITE, INVALID_HANDLE_VALUE, LocalFree,
            OBJ_CASE_INSENSITIVE, OBJ_DONT_REPARSE, RtlNtStatusToDosError, UNICODE_STRING,
        },
        Security::{
            ACCESS_ALLOWED_ACE, ACE_HEADER, ACL,
            Authorization::{
                ConvertSidToStringSidW, ConvertStringSecurityDescriptorToSecurityDescriptorW,
                GetSecurityInfo, SE_FILE_OBJECT,
            },
            DACL_SECURITY_INFORMATION, EqualSid, GetAce, GetSecurityDescriptorControl,
            GetTokenInformation, IsWellKnownSid, OWNER_SECURITY_INFORMATION, SE_DACL_PROTECTED,
            SECURITY_ATTRIBUTES, TOKEN_QUERY, TOKEN_USER, TokenUser, WinLocalSystemSid,
        },
        Storage::FileSystem::{
            BY_HANDLE_FILE_INFORMATION, CreateDirectoryW, CreateFileW, DELETE,
            FILE_ATTRIBUTE_REPARSE_POINT, FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OPEN_REPARSE_POINT,
            FILE_SHARE_DELETE, FILE_SHARE_READ, FILE_SHARE_WRITE, FILE_TYPE_DISK,
            GetFileInformationByHandle, GetFileType, GetFinalPathNameByHandleW,
            LOCKFILE_EXCLUSIVE_LOCK, LOCKFILE_FAIL_IMMEDIATELY, LockFileEx, OPEN_EXISTING,
            SYNCHRONIZE,
        },
        System::{
            IO::{IO_STATUS_BLOCK, OVERLAPPED},
            Threading::{GetCurrentProcess, OpenProcessToken},
        },
    },
};

fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message)
}

pub(crate) fn wide(value: &OsStr) -> io::Result<Vec<u16>> {
    let mut result: Vec<_> = value.encode_wide().collect();
    if result.contains(&0) {
        return Err(invalid("Windows path contains NUL"));
    }
    result.push(0);
    Ok(result)
}

pub(crate) fn validate_component(name: &OsStr) -> io::Result<()> {
    let units: Vec<_> = name.encode_wide().collect();
    let normal = matches!(
        Path::new(name).components().next(),
        Some(Component::Normal(_))
    ) && Path::new(name).components().count() == 1;
    if !normal
        || units.is_empty()
        || units
            .iter()
            .any(|c| *c < 32 || b"\\/:*?\"<>|".contains(&(*c as u8)) && *c < 128)
        || matches!(units.last(), Some(32 | 46))
    {
        return Err(invalid(
            "Windows child name must be one normal component without streams or aliases",
        ));
    }
    let stem = name
        .to_string_lossy()
        .split('.')
        .next()
        .unwrap_or_default()
        .to_ascii_uppercase();
    if matches!(
        stem.as_str(),
        "CON" | "PRN" | "AUX" | "NUL" | "CONIN$" | "CONOUT$"
    ) || (stem.len() == 4
        && (stem.starts_with("COM") || stem.starts_with("LPT"))
        && stem.as_bytes()[3].is_ascii_digit())
    {
        return Err(invalid("Windows device names cannot be capture sources"));
    }
    Ok(())
}

pub(crate) fn information(file: &File) -> io::Result<BY_HANDLE_FILE_INFORMATION> {
    // SAFETY: the output is initialized by the successful call, and the handle is live.
    unsafe {
        let mut info = std::mem::zeroed();
        if GetFileInformationByHandle(file.as_raw_handle(), &mut info) == 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(info)
    }
}

pub(crate) fn identity(file: &File) -> io::Result<(u64, u64)> {
    let info = information(file)?;
    Ok((
        info.dwVolumeSerialNumber as u64,
        ((info.nFileIndexHigh as u64) << 32) | info.nFileIndexLow as u64,
    ))
}

pub(crate) fn handle_path(file: &File) -> io::Result<PathBuf> {
    let mut buffer = vec![0u16; 512];
    loop {
        // SAFETY: the output buffer has the declared capacity and the file is live.
        let size = unsafe {
            GetFinalPathNameByHandleW(
                file.as_raw_handle(),
                buffer.as_mut_ptr(),
                buffer.len() as u32,
                0,
            )
        };
        if size == 0 {
            return Err(io::Error::last_os_error());
        }
        if (size as usize) < buffer.len() {
            buffer.truncate(size as usize);
            return Ok(PathBuf::from(OsString::from_wide(&buffer)));
        }
        if size > 32768 {
            return Err(invalid("Windows capture path exceeds its length limit"));
        }
        buffer.resize(size as usize + 1, 0);
    }
}

fn open(
    path: &Path,
    directory: bool,
    access: u32,
    disposition: u32,
    attributes: *const SECURITY_ATTRIBUTES,
) -> io::Result<File> {
    open_with_share(
        path,
        directory,
        access,
        disposition,
        attributes,
        FILE_SHARE_READ,
    )
}

fn open_with_share(
    path: &Path,
    directory: bool,
    access: u32,
    disposition: u32,
    attributes: *const SECURITY_ATTRIBUTES,
    sharing: u32,
) -> io::Result<File> {
    let path = wide(path.as_os_str())?;
    // SAFETY: path is terminated, security attributes (when supplied) live for this call.
    let handle = unsafe {
        CreateFileW(
            path.as_ptr(),
            access,
            sharing,
            attributes,
            disposition,
            FILE_FLAG_OPEN_REPARSE_POINT | FILE_FLAG_BACKUP_SEMANTICS,
            null_mut(),
        )
    };
    if handle == INVALID_HANDLE_VALUE {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: CreateFileW returned a new owned handle; ownership transfers exactly once.
    let file = unsafe { File::from_raw_handle(handle) };
    let info = information(&file)?;
    if info.dwFileAttributes & FILE_ATTRIBUTE_REPARSE_POINT != 0
        || file.metadata()?.is_dir() != directory
        // SAFETY: GetFileType only inspects this live handle.
        || unsafe { GetFileType(file.as_raw_handle()) } != FILE_TYPE_DISK
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "capture source must be a disk file/directory, without reparse points",
        ));
    }
    Ok(file)
}

/// Retain the entire ancestry until a child has been opened. Denying write and
/// delete sharing prevents junction conversion and rename/swap during traversal.
pub(crate) fn pin_tree(path: &Path) -> io::Result<Vec<File>> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()?.join(path)
    };
    let mut parts = absolute.components();
    let Some(Component::Prefix(prefix)) = parts.next() else {
        return Err(invalid(
            "capture directory must have an absolute Windows drive",
        ));
    };
    if !matches!(prefix.kind(), Prefix::Disk(_) | Prefix::VerbatimDisk(_)) {
        return Err(invalid(
            "capture requires a local Windows drive; UNC and device paths are unsupported",
        ));
    }
    if !matches!(parts.next(), Some(Component::RootDir)) {
        return Err(invalid("capture directory must be absolute"));
    }
    let mut current = PathBuf::from(prefix.as_os_str());
    current.push("\\");
    let mut handles = vec![open(&current, true, GENERIC_READ, OPEN_EXISTING, null())?];
    for component in parts {
        let name = match component {
            Component::CurDir => continue,
            Component::Normal(name) => name,
            _ => return Err(invalid("capture path contains a non-normal component")),
        };
        validate_component(name)?;
        current.push(name);
        handles.push(open(&current, true, GENERIC_READ, OPEN_EXISTING, null())?);
    }
    Ok(handles)
}

pub(crate) fn open_tree(path: &Path) -> io::Result<File> {
    pin_tree(path)?
        .pop()
        .ok_or_else(|| invalid("capture directory is empty"))
}

pub(crate) struct PinnedParent {
    _ancestry: Vec<File>,
    path: PathBuf,
}

impl PinnedParent {
    pub(crate) fn new(parent: &File) -> io::Result<Self> {
        let path = handle_path(parent)?;
        let ancestry = pin_tree(&path)?;
        let reopened = ancestry
            .last()
            .ok_or_else(|| invalid("capture parent has no directory"))?;
        if identity(reopened)? != identity(parent)? {
            return Err(io::Error::other(
                "capture parent changed while opening; retry the command",
            ));
        }
        Ok(Self {
            _ancestry: ancestry,
            path,
        })
    }

    pub(crate) fn child(&self, name: &OsStr) -> io::Result<PathBuf> {
        validate_component(name)?;
        Ok(self.path.join(name))
    }
}

pub(crate) fn open_at(parent: &File, name: &OsStr, directory: bool) -> io::Result<File> {
    open_relative(
        parent,
        name,
        directory,
        GENERIC_READ,
        FILE_OPEN,
        FILE_SHARE_READ,
        null(),
    )
}

// RootDirectory is a capability, not a pathname. This also keeps private
// mutations safe while their parent permits write sharing for atomic rename.
#[allow(clippy::too_many_arguments)]
fn open_relative(
    parent: &File,
    name: &OsStr,
    directory: bool,
    access: u32,
    disposition: u32,
    sharing: u32,
    descriptor: *const windows_sys::Win32::Security::SECURITY_DESCRIPTOR,
) -> io::Result<File> {
    validate_component(name)?;
    let mut name = wide(name)?;
    let length = u16::try_from((name.len() - 1) * 2)
        .map_err(|_| invalid("Windows child name exceeds its length limit"))?;
    let unicode = UNICODE_STRING {
        Length: length,
        MaximumLength: length,
        Buffer: name.as_mut_ptr(),
    };
    let attributes = OBJECT_ATTRIBUTES {
        Length: std::mem::size_of::<OBJECT_ATTRIBUTES>() as u32,
        RootDirectory: parent.as_raw_handle(),
        ObjectName: &unicode,
        Attributes: OBJ_CASE_INSENSITIVE | OBJ_DONT_REPARSE,
        SecurityDescriptor: descriptor,
        SecurityQualityOfService: null(),
    };
    let mut handle = null_mut();
    let mut status_block = IO_STATUS_BLOCK::default();
    // SAFETY: all buffers and the root directory remain live for this synchronous
    // call. Only a validated single component can be resolved beneath the root.
    let status = unsafe {
        NtCreateFile(
            &mut handle,
            access | SYNCHRONIZE,
            &attributes,
            &mut status_block,
            null(),
            0,
            sharing,
            disposition,
            FILE_OPEN_REPARSE_POINT
                | FILE_SYNCHRONOUS_IO_NONALERT
                | if directory {
                    FILE_DIRECTORY_FILE
                } else {
                    FILE_NON_DIRECTORY_FILE
                },
            null(),
            0,
        )
    };
    if status < 0 {
        return Err(nt_error(status));
    }
    // SAFETY: the successful call transferred one fresh handle to us.
    let file = unsafe { File::from_raw_handle(handle) };
    if information(&file)?.dwFileAttributes & FILE_ATTRIBUTE_REPARSE_POINT != 0
        || file.metadata()?.is_dir() != directory
        || unsafe { GetFileType(file.as_raw_handle()) } != FILE_TYPE_DISK
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "capture source must be a disk file/directory, without reparse points",
        ));
    }
    Ok(file)
}

fn nt_error(status: i32) -> io::Error {
    // SAFETY: this pure conversion takes no pointers.
    io::Error::from_raw_os_error(unsafe { RtlNtStatusToDosError(status) } as i32)
}

pub(crate) fn open_private_directory(path: &Path) -> io::Result<File> {
    let parent = path
        .parent()
        .ok_or_else(|| invalid("private directory has no parent"))?;
    let name = path
        .file_name()
        .ok_or_else(|| invalid("private directory has no filename"))?;
    let parent = open_tree(parent)?;
    let file = open_relative(
        &parent,
        name,
        true,
        GENERIC_READ,
        FILE_OPEN,
        FILE_SHARE_READ | FILE_SHARE_WRITE,
        null(),
    )?;
    validate_private(&file)?;
    Ok(file)
}

/// Atomically rename the held file beneath the held target directory, without
/// replacing a pre-existing key and without resolving any ancestor pathname.
pub(crate) fn publish_private_file(file: &File, target: &File, name: &OsStr) -> io::Result<()> {
    validate_component(name)?;
    let name: Vec<_> = name.encode_wide().collect();
    let length = std::mem::size_of::<FILE_RENAME_INFORMATION>() + name.len() * 2;
    let mut buffer = vec![0usize; length.div_ceil(std::mem::size_of::<usize>())];
    let info = buffer.as_mut_ptr().cast::<FILE_RENAME_INFORMATION>();
    let mut status_block = IO_STATUS_BLOCK::default();
    // SAFETY: aligned, zeroed storage is large enough for the variable-length
    // name. ReplaceIfExists remains false and both handles are held throughout.
    let status = unsafe {
        (*info).RootDirectory = target.as_raw_handle();
        (*info).FileNameLength = (name.len() * 2) as u32;
        std::ptr::copy_nonoverlapping(name.as_ptr(), (*info).FileName.as_mut_ptr(), name.len());
        NtSetInformationFile(
            file.as_raw_handle(),
            &mut status_block,
            info.cast(),
            length as u32,
            FileRenameInformation,
        )
    };
    if status < 0 {
        return Err(nt_error(status));
    }
    file.sync_all()
}

pub(crate) fn remove_private_file_at(parent: &File, name: &OsStr) -> io::Result<()> {
    let file = open_relative(
        parent,
        name,
        false,
        GENERIC_READ | DELETE,
        FILE_OPEN,
        FILE_SHARE_READ,
        null(),
    )?;
    validate_private(&file)?;
    if information(&file)?.nNumberOfLinks != 1 {
        return Err(invalid("private staging file has unexpected hard links"));
    }
    let info = FILE_DISPOSITION_INFORMATION { DeleteFile: true };
    let mut status_block = IO_STATUS_BLOCK::default();
    // SAFETY: the synchronous request references a live owned regular file.
    let status = unsafe {
        NtSetInformationFile(
            file.as_raw_handle(),
            &mut status_block,
            (&info as *const FILE_DISPOSITION_INFORMATION).cast(),
            std::mem::size_of_val(&info) as u32,
            FileDispositionInformation,
        )
    };
    if status < 0 {
        return Err(nt_error(status));
    }
    Ok(())
}

/// Live transcript writers can retain and rotate their own file handles. The
/// reader authorizes this opened file once and consumes only its held handle.
/// Keys, locks and trusted executables continue to use the stricter opener.
pub(crate) fn open_live_source_at(parent: &File, name: &OsStr) -> io::Result<File> {
    open_relative(
        parent,
        name,
        false,
        GENERIC_READ,
        FILE_OPEN,
        FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
        null(),
    )
}

pub(crate) fn create_directory_at(parent: &File, name: &OsStr, private: bool) -> io::Result<File> {
    if private {
        let descriptor = private_descriptor()?;
        let open_directory = |disposition| {
            open_relative(
                parent,
                name,
                true,
                GENERIC_READ,
                disposition,
                FILE_SHARE_READ | FILE_SHARE_WRITE,
                descriptor.0.cast(),
            )
        };
        let file = match open_directory(FILE_CREATE) {
            Ok(file) => file,
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
                open_directory(FILE_OPEN)?
            }
            Err(error) => return Err(error),
        };
        validate_private(&file)?;
        return Ok(file);
    }
    let pin = PinnedParent::new(parent)?;
    let path = pin.child(name)?;
    let result = std::fs::create_dir(&path);
    match result {
        Ok(()) => {}
        Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {}
        Err(e) => return Err(e),
    }
    let file = open(&path, true, GENERIC_READ, OPEN_EXISTING, null())?;
    Ok(file)
}

struct LocalAllocation(*mut std::ffi::c_void);
impl Drop for LocalAllocation {
    fn drop(&mut self) {
        // SAFETY: all instances own a LocalAlloc allocation returned by a Windows security API.
        unsafe {
            LocalFree(self.0);
        }
    }
}

fn current_user() -> io::Result<Vec<usize>> {
    // SAFETY: token and output buffers are initialized and owned for each call.
    unsafe {
        let mut token = null_mut();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) == 0 {
            return Err(io::Error::last_os_error());
        }
        let token = OwnedHandle::from_raw_handle(token);
        let mut size = 0;
        GetTokenInformation(token.as_raw_handle(), TokenUser, null_mut(), 0, &mut size);
        let mut buffer = vec![0usize; (size as usize).div_ceil(std::mem::size_of::<usize>())];
        if GetTokenInformation(
            token.as_raw_handle(),
            TokenUser,
            buffer.as_mut_ptr().cast(),
            size,
            &mut size,
        ) == 0
        {
            return Err(io::Error::last_os_error());
        }
        Ok(buffer)
    }
}

fn private_descriptor() -> io::Result<LocalAllocation> {
    let user = current_user()?;
    // SAFETY: user is an aligned TOKEN_USER buffer, SID and converted strings remain live.
    unsafe {
        let user = &*user.as_ptr().cast::<TOKEN_USER>();
        let mut sid_text = null_mut();
        if ConvertSidToStringSidW(user.User.Sid, &mut sid_text) == 0 {
            return Err(io::Error::last_os_error());
        }
        let _sid_text = LocalAllocation(sid_text.cast());
        let mut len = 0;
        while *sid_text.add(len) != 0 {
            len += 1;
        }
        let sid = String::from_utf16_lossy(std::slice::from_raw_parts(sid_text, len));
        let sddl = wide(OsStr::new(&format!(
            "O:{sid}D:P(A;OICI;FA;;;{sid})(A;OICI;FA;;;SY)"
        )))?;
        let mut descriptor = null_mut();
        if ConvertStringSecurityDescriptorToSecurityDescriptorW(
            sddl.as_ptr(),
            1,
            &mut descriptor,
            null_mut(),
        ) == 0
        {
            return Err(io::Error::last_os_error());
        }
        Ok(LocalAllocation(descriptor))
    }
}

fn security_attributes(descriptor: &LocalAllocation) -> SECURITY_ATTRIBUTES {
    SECURITY_ATTRIBUTES {
        nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
        lpSecurityDescriptor: descriptor.0,
        bInheritHandle: 0,
    }
}

pub(crate) fn create_private_directory(path: &Path) -> io::Result<()> {
    let descriptor = private_descriptor()?;
    let attrs = security_attributes(&descriptor);
    let path = wide(path.as_os_str())?;
    // SAFETY: both path and descriptor live throughout CreateDirectoryW.
    if unsafe { CreateDirectoryW(path.as_ptr(), &attrs) } == 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

pub(crate) fn create_private_file_at(parent: &File, name: &OsStr) -> io::Result<File> {
    create_private_file_at_with_access(parent, name, GENERIC_READ | GENERIC_WRITE | DELETE)
}

pub(crate) fn open_private_lock_at(parent: &File, name: &OsStr) -> io::Result<File> {
    match create_private_file_at_with_access(parent, name, GENERIC_READ) {
        Ok(file) => Ok(file),
        Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {
            let file = open_at(parent, name, false)?;
            validate_private(&file)?;
            Ok(file)
        }
        Err(e) => Err(e),
    }
}

fn create_private_file_at_with_access(
    parent: &File,
    name: &OsStr,
    access: u32,
) -> io::Result<File> {
    let descriptor = private_descriptor()?;
    let file = open_relative(
        parent,
        name,
        false,
        access,
        FILE_CREATE,
        FILE_SHARE_READ,
        descriptor.0.cast(),
    )?;
    validate_private(&file)?;
    Ok(file)
}

/// Refuse unprotected, foreign-owned or broadly readable key material. SYSTEM
/// is the sole additional principal allowed by our creation descriptor.
pub(crate) fn validate_private(file: &File) -> io::Result<()> {
    let user = current_user()?;
    // SAFETY: the security API returns one owned descriptor; all SID/ACL pointers
    // remain within it until validation completes. GetAce checks each ACE index.
    unsafe {
        let user = &*user.as_ptr().cast::<TOKEN_USER>();
        let mut owner = null_mut();
        let mut acl: *mut ACL = null_mut();
        let mut descriptor = null_mut();
        let result = GetSecurityInfo(
            file.as_raw_handle(),
            SE_FILE_OBJECT,
            OWNER_SECURITY_INFORMATION | DACL_SECURITY_INFORMATION,
            &mut owner,
            null_mut(),
            &mut acl,
            null_mut(),
            &mut descriptor,
        );
        if result != 0 {
            return Err(io::Error::from_raw_os_error(result as i32));
        }
        let _descriptor = LocalAllocation(descriptor);
        let mut control = 0;
        let mut revision = 0;
        if GetSecurityDescriptorControl(descriptor, &mut control, &mut revision) == 0 {
            return Err(io::Error::last_os_error());
        }
        if control & SE_DACL_PROTECTED == 0 {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "capture key requires a protected Windows ACL without inherited permissions",
            ));
        }
        if owner.is_null() || EqualSid(owner, user.User.Sid) == 0 || acl.is_null() {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "capture key must have an owner-only Windows ACL; run libra agent doctor",
            ));
        }
        for index in 0..(*acl).AceCount {
            let mut ace = null_mut();
            if GetAce(acl, index as u32, &mut ace) == 0 {
                return Err(io::Error::last_os_error());
            }
            let header = &*ace.cast::<ACE_HEADER>();
            // DENY ACEs grant no access. Reject unfamiliar allow/object/callback ACEs.
            if header.AceType == 1 {
                continue;
            }
            if header.AceType != 0 {
                return Err(io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    "capture key contains an unsupported Windows ACL entry",
                ));
            }
            let allowed = &*ace.cast::<ACCESS_ALLOWED_ACE>();
            let sid = std::ptr::addr_of!(allowed.SidStart).cast_mut().cast();
            if EqualSid(sid, user.User.Sid) == 0 && IsWellKnownSid(sid, WinLocalSystemSid) == 0 {
                return Err(io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    "capture key grants access to another Windows user; use an owner-only ACL",
                ));
            }
        }
    }
    Ok(())
}

/// LockFileEx locks are released by handle close and process teardown.
pub(crate) fn try_lock(file: &File) -> io::Result<bool> {
    // SAFETY: OVERLAPPED is zeroed for byte zero; synchronous disk handle stays live.
    let result = unsafe {
        let mut offset: OVERLAPPED = std::mem::zeroed();
        LockFileEx(
            file.as_raw_handle(),
            LOCKFILE_EXCLUSIVE_LOCK | LOCKFILE_FAIL_IMMEDIATELY,
            0,
            1,
            0,
            &mut offset,
        )
    };
    if result != 0 {
        return Ok(true);
    }
    let error = io::Error::last_os_error();
    if error.raw_os_error() == Some(ERROR_LOCK_VIOLATION as i32) {
        return Ok(false);
    }
    Err(error)
}

pub(crate) fn ensure_directory_not_publicly_writable(path: &Path) -> io::Result<()> {
    use windows_sys::Win32::Security::WinBuiltinAdministratorsSid;
    let file = open_tree(path)?;
    let user = current_user()?;
    // SAFETY: ACL/SID pointers belong to the held descriptor's security buffer.
    unsafe {
        let user = &*user.as_ptr().cast::<TOKEN_USER>();
        let mut acl: *mut ACL = null_mut();
        let mut descriptor = null_mut();
        let result = GetSecurityInfo(
            file.as_raw_handle(),
            SE_FILE_OBJECT,
            DACL_SECURITY_INFORMATION,
            null_mut(),
            null_mut(),
            &mut acl,
            null_mut(),
            &mut descriptor,
        );
        if result != 0 {
            return Err(io::Error::from_raw_os_error(result as i32));
        }
        let _descriptor = LocalAllocation(descriptor);
        if acl.is_null() {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "trusted Windows directory has an unrestricted ACL",
            ));
        }
        for index in 0..(*acl).AceCount {
            let mut ace = null_mut();
            if GetAce(acl, index as u32, &mut ace) == 0 {
                return Err(io::Error::last_os_error());
            }
            let header = &*ace.cast::<ACE_HEADER>();
            if header.AceType == 1 {
                continue;
            }
            if header.AceType != 0 {
                return Err(io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    "trusted Windows directory has an unsupported ACL",
                ));
            }
            let allowed = &*ace.cast::<ACCESS_ALLOWED_ACE>();
            let sid = std::ptr::addr_of!(allowed.SidStart).cast_mut().cast();
            const WRITE_RIGHTS: u32 = 0x4000_0000 | 0x1000_0000 | 0x000D_0046;
            if allowed.Mask & WRITE_RIGHTS != 0
                && EqualSid(sid, user.User.Sid) == 0
                && IsWellKnownSid(sid, WinLocalSystemSid) == 0
                && IsWellKnownSid(sid, WinBuiltinAdministratorsSid) == 0
            {
                return Err(io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    "trusted Windows directory is writable by another user; move the binary to a protected directory",
                ));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_traversal_streams_and_windows_device_aliases() {
        for name in [
            "..",
            "a/b",
            "a\\b",
            "key:secret",
            "NUL",
            "con.txt",
            "COM1",
            "key.",
            "key ",
        ] {
            assert!(
                validate_component(OsStr::new(name)).is_err(),
                "accepted {name}"
            );
        }
        assert!(validate_component(OsStr::new("捕获-key.json")).is_ok());
    }

    #[test]
    fn private_acl_and_lock_exclude_concurrent_owners() {
        let root = tempfile::tempdir().unwrap();
        let parent = open_tree(root.path()).unwrap();
        let private = create_directory_at(&parent, OsStr::new("private"), true).unwrap();
        validate_private(&private).unwrap();
        let first = open_private_lock_at(&private, OsStr::new("lock")).unwrap();
        let second = open_private_lock_at(&private, OsStr::new("lock")).unwrap();
        assert!(try_lock(&first).unwrap());
        assert!(!try_lock(&second).unwrap());
        drop(first);
        assert!(try_lock(&second).unwrap());
    }

    #[test]
    fn private_material_rejects_inheritable_acl_even_when_current_grants_are_private() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("private");
        create_private_directory(&path).unwrap();
        std::fs::write(path.join("inherited"), b"secret").unwrap();
        let directory = open_tree(&path).unwrap();
        validate_private(&directory).unwrap();
        let file = open_at(&directory, OsStr::new("inherited"), false).unwrap();
        assert!(
            validate_private(&file)
                .unwrap_err()
                .to_string()
                .contains("protected Windows ACL")
        );
    }

    #[test]
    fn private_publication_never_replaces_an_existing_key() {
        use std::io::Write;

        let root = tempfile::tempdir().unwrap();
        let parent = open_tree(root.path()).unwrap();
        let private = create_directory_at(&parent, OsStr::new("private"), true).unwrap();
        let mut existing = create_private_file_at(&private, OsStr::new("key")).unwrap();
        existing.write_all(b"original").unwrap();
        existing.sync_all().unwrap();
        drop(existing);
        let mut staged = create_private_file_at(&private, OsStr::new("staged")).unwrap();
        staged.write_all(b"replacement").unwrap();
        staged.sync_all().unwrap();
        assert_eq!(
            publish_private_file(&staged, &private, OsStr::new("key"))
                .unwrap_err()
                .kind(),
            io::ErrorKind::AlreadyExists
        );
        assert_eq!(
            std::fs::read(root.path().join("private/key")).unwrap(),
            b"original"
        );
    }

    #[test]
    fn live_source_allows_a_writer_and_retains_identity_after_rotation() {
        use std::io::{Read, Write};
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("session.jsonl");
        let mut writer = std::fs::File::create(&path).unwrap();
        writer.write_all(b"first\n").unwrap();
        let parent = open_tree(root.path()).unwrap();
        let mut reader = open_live_source_at(&parent, OsStr::new("session.jsonl")).unwrap();
        writer.write_all(b"second\n").unwrap();
        drop(writer);
        drop(parent);
        std::fs::rename(&path, root.path().join("old.jsonl")).unwrap();
        std::fs::write(&path, b"replacement\n").unwrap();
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).unwrap();
        assert_eq!(bytes, b"first\nsecond\n");
    }

    #[test]
    fn rejects_junction_and_pins_parent_against_rename() {
        let root = tempfile::tempdir().unwrap();
        let external = tempfile::tempdir().unwrap();
        let junction = root.path().join("junction");
        let status = std::process::Command::new("cmd.exe")
            .args(["/C", "mklink", "/J"])
            .arg(&junction)
            .arg(external.path())
            .stdout(std::process::Stdio::null())
            .status()
            .unwrap();
        assert!(status.success(), "create junction fixture");
        assert!(open_tree(&junction).is_err());
        let parent = open_tree(root.path()).unwrap();
        assert!(open_at(&parent, OsStr::new("junction"), true).is_err());
        assert!(std::fs::rename(root.path(), root.path().with_extension("moved")).is_err());
        assert!(open_at(&parent, OsStr::new("../outside"), false).is_err());
        drop(parent);
        std::fs::remove_dir(&junction).unwrap();
    }
}
