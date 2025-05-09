//! FUSE kernel interface.
//!
//! Types and definitions used for communication between the kernel driver and the userspace
//! part of a FUSE filesystem. Since the kernel driver may be installed independently, the ABI
//! interface is versioned and capabilities are exchanged during the initialization (mounting)
//! of a filesystem.
//!
//! OSXFUSE (macOS): <https://github.com/osxfuse/fuse/blob/master/include/fuse_kernel.h>
//! - supports ABI 7.8 in OSXFUSE 2.x
//! - supports ABI 7.19 since OSXFUSE 3.0.0
//!
//! libfuse (Linux/BSD): <https://github.com/libfuse/libfuse/blob/master/include/fuse_kernel.h>
//! - supports ABI 7.8 since FUSE 2.6.0
//! - supports ABI 7.12 since FUSE 2.8.0
//! - supports ABI 7.18 since FUSE 2.9.0
//! - supports ABI 7.19 since FUSE 2.9.1
//! - supports ABI 7.26 since FUSE 3.0.0
//!
//! Items without a version annotation are valid with ABI 7.8 and later

#![warn(missing_debug_implementations)]
#![allow(missing_docs)]

#[cfg(feature = "abi-7-9")]
use crate::constants::{FATTR_ATIME_NOW, FATTR_MTIME_NOW};
use std::convert::TryFrom;
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout, TryFromBytes};

pub const FUSE_KERNEL_VERSION: u32 = 7;

#[cfg(not(feature = "abi-7-9"))]
pub const FUSE_KERNEL_MINOR_VERSION: u32 = 8;
#[cfg(all(feature = "abi-7-9", not(feature = "abi-7-10")))]
pub const FUSE_KERNEL_MINOR_VERSION: u32 = 9;
#[cfg(all(feature = "abi-7-10", not(feature = "abi-7-11")))]
pub const FUSE_KERNEL_MINOR_VERSION: u32 = 10;
#[cfg(all(feature = "abi-7-11", not(feature = "abi-7-12")))]
pub const FUSE_KERNEL_MINOR_VERSION: u32 = 11;
#[cfg(all(feature = "abi-7-12", not(feature = "abi-7-13")))]
pub const FUSE_KERNEL_MINOR_VERSION: u32 = 12;
#[cfg(all(feature = "abi-7-13", not(feature = "abi-7-14")))]
pub const FUSE_KERNEL_MINOR_VERSION: u32 = 13;
#[cfg(all(feature = "abi-7-14", not(feature = "abi-7-15")))]
pub const FUSE_KERNEL_MINOR_VERSION: u32 = 14;
#[cfg(all(feature = "abi-7-15", not(feature = "abi-7-16")))]
pub const FUSE_KERNEL_MINOR_VERSION: u32 = 15;
#[cfg(all(feature = "abi-7-16", not(feature = "abi-7-17")))]
pub const FUSE_KERNEL_MINOR_VERSION: u32 = 16;
#[cfg(all(feature = "abi-7-17", not(feature = "abi-7-18")))]
pub const FUSE_KERNEL_MINOR_VERSION: u32 = 17;
#[cfg(all(feature = "abi-7-18", not(feature = "abi-7-19")))]
pub const FUSE_KERNEL_MINOR_VERSION: u32 = 18;
#[cfg(all(feature = "abi-7-19", not(feature = "abi-7-20")))]
pub const FUSE_KERNEL_MINOR_VERSION: u32 = 19;
#[cfg(all(feature = "abi-7-20", not(feature = "abi-7-21")))]
pub const FUSE_KERNEL_MINOR_VERSION: u32 = 20;
#[cfg(all(feature = "abi-7-21", not(feature = "abi-7-22")))]
pub const FUSE_KERNEL_MINOR_VERSION: u32 = 21;
#[cfg(all(feature = "abi-7-22", not(feature = "abi-7-23")))]
pub const FUSE_KERNEL_MINOR_VERSION: u32 = 22;
#[cfg(all(feature = "abi-7-23", not(feature = "abi-7-24")))]
pub const FUSE_KERNEL_MINOR_VERSION: u32 = 23;
#[cfg(all(feature = "abi-7-24", not(feature = "abi-7-25")))]
pub const FUSE_KERNEL_MINOR_VERSION: u32 = 24;
#[cfg(all(feature = "abi-7-25", not(feature = "abi-7-26")))]
pub const FUSE_KERNEL_MINOR_VERSION: u32 = 25;
#[cfg(all(feature = "abi-7-26", not(feature = "abi-7-27")))]
pub const FUSE_KERNEL_MINOR_VERSION: u32 = 26;
#[cfg(all(feature = "abi-7-27", not(feature = "abi-7-28")))]
pub const FUSE_KERNEL_MINOR_VERSION: u32 = 27;
#[cfg(all(feature = "abi-7-28", not(feature = "abi-7-29")))]
pub const FUSE_KERNEL_MINOR_VERSION: u32 = 28;
#[cfg(all(feature = "abi-7-29", not(feature = "abi-7-30")))]
pub const FUSE_KERNEL_MINOR_VERSION: u32 = 29;
#[cfg(all(feature = "abi-7-30", not(feature = "abi-7-31")))]
pub const FUSE_KERNEL_MINOR_VERSION: u32 = 30;
#[cfg(feature = "abi-7-31")]
pub const FUSE_KERNEL_MINOR_VERSION: u32 = 31;

pub const FUSE_ROOT_ID: u64 = 1;

#[repr(C)]
#[derive(Debug, IntoBytes, Clone, Copy, KnownLayout, Immutable)]
pub struct fuse_attr {
    /// Inode number
    pub ino: u64,
    /// Size of the object in bytes
    pub size: u64,
    /// Number of 512B blocks allocated
    /// See [stat](https://linux.die.net/man/2/stat)
    pub blocks: u64,
    // NOTE: this field is defined as u64 in fuse_kernel.h in libfuse. However, it is treated as signed
    // to match stat.st_atime
    pub atime: i64,
    // NOTE: this field is defined as u64 in fuse_kernel.h in libfuse. However, it is treated as signed
    // to match stat.st_mtime
    pub mtime: i64,
    // NOTE: this field is defined as u64 in fuse_kernel.h in libfuse. However, it is treated as signed
    // to match stat.st_ctime
    pub ctime: i64,
    #[cfg(target_os = "macos")]
    pub crtime: u64,
    pub atimensec: u32,
    pub mtimensec: u32,
    pub ctimensec: u32,
    #[cfg(target_os = "macos")]
    pub crtimensec: u32,
    /// See [st_mode](https://www.man7.org/linux/man-pages/man2/stat.2.html)
    pub mode: u32,
    /// See [unlink](https://man7.org/linux/man-pages/man2/unlink.2.html)
    /// See [stat](https://linux.die.net/man/2/stat)
    ///
    /// Number of hard links
    pub nlink: u32,
    pub uid: u32,
    pub gid: u32,
    /// Device ID (if special file)
    /// See [stat](https://linux.die.net/man/2/stat)
    pub rdev: u32,
    #[cfg(target_os = "macos")]
    pub flags: u32, // see chflags(2)
    #[cfg(feature = "abi-7-9")]
    pub blksize: u32,
    #[cfg(feature = "abi-7-9")]
    pub padding: u32,
}

#[repr(C)]
#[derive(Debug, IntoBytes, KnownLayout, Immutable, Clone, Copy)]
/// See [statfs man page](https://www.man7.org/linux/man-pages/man2/statfs.2.html)
pub struct fuse_kstatfs {
    /// Total number of blocks (in units of frsize)
    pub blocks: u64,
    /// Free blocks
    pub bfree: u64, // Free blocks
    /// Free blocks for unprivileged users
    pub bavail: u64,
    /// Total allocated inodes
    pub files: u64,
    /// Number of free inodes
    pub ffree: u64,
    /// Filesystem block size
    pub bsize: u32,
    pub namelen: u32, // Maximum filename length
    /// "Fragment Size" Fundamental file system block size
    pub frsize: u32,
    pub padding: u32,
    pub spare: [u32; 6],
}

#[repr(C)]
#[derive(Debug, IntoBytes, FromBytes, KnownLayout, Immutable, Clone, Copy)]
pub struct fuse_file_lock {
    pub start: u64,
    pub end: u64,
    // NOTE: this field is defined as u32 in fuse_kernel.h in libfuse. However, it is treated as signed
    pub typ: i32,
    pub pid: u32,
}

/// Invalid opcode error.
#[derive(Debug)]
pub struct InvalidOpcodeError;

#[repr(C)]
#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum fuse_opcode {
    FUSE_LOOKUP          = 1,
    FUSE_FORGET          = 2, // no reply
    FUSE_GETATTR         = 3,
    FUSE_SETATTR         = 4,
    FUSE_READLINK        = 5,
    FUSE_SYMLINK         = 6,
    FUSE_MKNOD           = 8,
    FUSE_MKDIR           = 9,
    FUSE_UNLINK          = 10,
    FUSE_RMDIR           = 11,
    FUSE_RENAME          = 12,
    FUSE_LINK            = 13,
    FUSE_OPEN            = 14,
    FUSE_READ            = 15,
    FUSE_WRITE           = 16,
    FUSE_STATFS          = 17,
    FUSE_RELEASE         = 18,
    FUSE_FSYNC           = 20,
    FUSE_SETXATTR        = 21,
    FUSE_GETXATTR        = 22,
    FUSE_LISTXATTR       = 23,
    FUSE_REMOVEXATTR     = 24,
    FUSE_FLUSH           = 25,
    FUSE_INIT            = 26,
    FUSE_OPENDIR         = 27,
    FUSE_READDIR         = 28,
    FUSE_RELEASEDIR      = 29,
    FUSE_FSYNCDIR        = 30,
    FUSE_GETLK           = 31,
    FUSE_SETLK           = 32,
    FUSE_SETLKW          = 33,
    FUSE_ACCESS          = 34,
    FUSE_CREATE          = 35,
    FUSE_INTERRUPT       = 36,
    FUSE_BMAP            = 37,
    FUSE_DESTROY         = 38,
    #[cfg(feature = "abi-7-11")]
    FUSE_IOCTL           = 39,
    #[cfg(feature = "abi-7-11")]
    FUSE_POLL            = 40,
    #[cfg(feature = "abi-7-15")]
    FUSE_NOTIFY_REPLY    = 41,
    #[cfg(feature = "abi-7-16")]
    FUSE_BATCH_FORGET    = 42,
    #[cfg(feature = "abi-7-19")]
    FUSE_FALLOCATE       = 43,
    #[cfg(feature = "abi-7-21")]
    FUSE_READDIRPLUS     = 44,
    #[cfg(feature = "abi-7-23")]
    FUSE_RENAME2         = 45,
    #[cfg(feature = "abi-7-24")]
    FUSE_LSEEK           = 46,
    #[cfg(feature = "abi-7-28")]
    FUSE_COPY_FILE_RANGE = 47,
    #[cfg(feature = "abi-7-31")]
    FUSE_SETUPMAPPING    = 48,
    #[cfg(feature = "abi-7-31")]
    FUSE_REMOVEMAPPING   = 49,
    #[cfg(feature = "abi-7-34")]
    FUSE_SYNCFS          = 50,
    #[cfg(feature = "abi-7-37")]
    FUSE_TMPFILE         = 51,
    #[cfg(feature = "abi-7-39")]
    FUSE_STATX           = 52,
    #[cfg(target_os = "macos")]
    FUSE_SETVOLNAME      = 61,
    #[cfg(target_os = "macos")]
    FUSE_GETXTIMES       = 62,
    #[cfg(target_os = "macos")]
    FUSE_EXCHANGE        = 63,

    #[cfg(feature = "abi-7-12")]
    CUSE_INIT            = 4096,
}

impl TryFrom<u32> for fuse_opcode {
    type Error = InvalidOpcodeError;

    fn try_from(n: u32) -> Result<Self, Self::Error> {
        match n {
            1 => Ok(fuse_opcode::FUSE_LOOKUP),
            2 => Ok(fuse_opcode::FUSE_FORGET),
            3 => Ok(fuse_opcode::FUSE_GETATTR),
            4 => Ok(fuse_opcode::FUSE_SETATTR),
            5 => Ok(fuse_opcode::FUSE_READLINK),
            6 => Ok(fuse_opcode::FUSE_SYMLINK),
            8 => Ok(fuse_opcode::FUSE_MKNOD),
            9 => Ok(fuse_opcode::FUSE_MKDIR),
            10 => Ok(fuse_opcode::FUSE_UNLINK),
            11 => Ok(fuse_opcode::FUSE_RMDIR),
            12 => Ok(fuse_opcode::FUSE_RENAME),
            13 => Ok(fuse_opcode::FUSE_LINK),
            14 => Ok(fuse_opcode::FUSE_OPEN),
            15 => Ok(fuse_opcode::FUSE_READ),
            16 => Ok(fuse_opcode::FUSE_WRITE),
            17 => Ok(fuse_opcode::FUSE_STATFS),
            18 => Ok(fuse_opcode::FUSE_RELEASE),
            20 => Ok(fuse_opcode::FUSE_FSYNC),
            21 => Ok(fuse_opcode::FUSE_SETXATTR),
            22 => Ok(fuse_opcode::FUSE_GETXATTR),
            23 => Ok(fuse_opcode::FUSE_LISTXATTR),
            24 => Ok(fuse_opcode::FUSE_REMOVEXATTR),
            25 => Ok(fuse_opcode::FUSE_FLUSH),
            26 => Ok(fuse_opcode::FUSE_INIT),
            27 => Ok(fuse_opcode::FUSE_OPENDIR),
            28 => Ok(fuse_opcode::FUSE_READDIR),
            29 => Ok(fuse_opcode::FUSE_RELEASEDIR),
            30 => Ok(fuse_opcode::FUSE_FSYNCDIR),
            31 => Ok(fuse_opcode::FUSE_GETLK),
            32 => Ok(fuse_opcode::FUSE_SETLK),
            33 => Ok(fuse_opcode::FUSE_SETLKW),
            34 => Ok(fuse_opcode::FUSE_ACCESS),
            35 => Ok(fuse_opcode::FUSE_CREATE),
            36 => Ok(fuse_opcode::FUSE_INTERRUPT),
            37 => Ok(fuse_opcode::FUSE_BMAP),
            38 => Ok(fuse_opcode::FUSE_DESTROY),
            #[cfg(feature = "abi-7-11")]
            39 => Ok(fuse_opcode::FUSE_IOCTL),
            #[cfg(feature = "abi-7-11")]
            40 => Ok(fuse_opcode::FUSE_POLL),
            #[cfg(feature = "abi-7-15")]
            41 => Ok(fuse_opcode::FUSE_NOTIFY_REPLY),
            #[cfg(feature = "abi-7-16")]
            42 => Ok(fuse_opcode::FUSE_BATCH_FORGET),
            #[cfg(feature = "abi-7-19")]
            43 => Ok(fuse_opcode::FUSE_FALLOCATE),
            #[cfg(feature = "abi-7-21")]
            44 => Ok(fuse_opcode::FUSE_READDIRPLUS),
            #[cfg(feature = "abi-7-23")]
            45 => Ok(fuse_opcode::FUSE_RENAME2),
            #[cfg(feature = "abi-7-24")]
            46 => Ok(fuse_opcode::FUSE_LSEEK),
            #[cfg(feature = "abi-7-28")]
            47 => Ok(fuse_opcode::FUSE_COPY_FILE_RANGE),
            #[cfg(feature = "abi-7-31")]
            48 => Ok(fuse_opcode::FUSE_SETUPMAPPING),
            #[cfg(feature = "abi-7-31")]
            49 => Ok(fuse_opcode::FUSE_REMOVEMAPPING),
            #[cfg(feature = "abi-7-34")]
            50 => Ok(fuse_opcode::FUSE_SYNCFS),
            #[cfg(feature = "abi-7-37")]
            51 => Ok(fuse_opcode::FUSE_TMPFILE),
            #[cfg(feature = "abi-7-39")]
            52 => Ok(fuse_opcode::FUSE_STATX),
            #[cfg(target_os = "macos")]
            61 => Ok(fuse_opcode::FUSE_SETVOLNAME),
            #[cfg(target_os = "macos")]
            62 => Ok(fuse_opcode::FUSE_GETXTIMES),
            #[cfg(target_os = "macos")]
            63 => Ok(fuse_opcode::FUSE_EXCHANGE),

            #[cfg(feature = "abi-7-12")]
            4096 => Ok(fuse_opcode::CUSE_INIT),

            _ => Err(InvalidOpcodeError),
        }
    }
}

/// Invalid notify code error.
#[cfg(feature = "abi-7-11")]
#[derive(Debug)]
pub struct InvalidNotifyCodeError;

#[cfg(feature = "abi-7-11")]
#[repr(C)]
#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum fuse_notify_code {
    #[cfg(feature = "abi-7-11")]
    FUSE_POLL            = 1,
    #[cfg(feature = "abi-7-12")]
    FUSE_NOTIFY_INVAL_INODE = 2,
    #[cfg(feature = "abi-7-12")]
    FUSE_NOTIFY_INVAL_ENTRY = 3,
    #[cfg(feature = "abi-7-15")]
    FUSE_NOTIFY_STORE    = 4,
    #[cfg(feature = "abi-7-15")]
    FUSE_NOTIFY_RETRIEVE = 5,
    #[cfg(feature = "abi-7-18")]
    FUSE_NOTIFY_DELETE   = 6,
}

#[cfg(feature = "abi-7-11")]
impl TryFrom<u32> for fuse_notify_code {
    type Error = InvalidNotifyCodeError;

    fn try_from(n: u32) -> Result<Self, Self::Error> {
        match n {
            #[cfg(feature = "abi-7-11")]
            1 => Ok(fuse_notify_code::FUSE_POLL),
            #[cfg(feature = "abi-7-12")]
            2 => Ok(fuse_notify_code::FUSE_NOTIFY_INVAL_INODE),
            #[cfg(feature = "abi-7-12")]
            3 => Ok(fuse_notify_code::FUSE_NOTIFY_INVAL_ENTRY),
            #[cfg(feature = "abi-7-15")]
            4 => Ok(fuse_notify_code::FUSE_NOTIFY_STORE),
            #[cfg(feature = "abi-7-15")]
            5 => Ok(fuse_notify_code::FUSE_NOTIFY_RETRIEVE),
            #[cfg(feature = "abi-7-18")]
            6 => Ok(fuse_notify_code::FUSE_NOTIFY_DELETE),

            _ => Err(InvalidNotifyCodeError),
        }
    }
}

#[repr(C)]
#[derive(Debug, IntoBytes, KnownLayout, Immutable, Clone, Copy)]
pub struct fuse_entry_out {
    /// Reference into a fixed size inode table
    pub nodeid: u64,
    pub generation: u64,
    pub entry_valid: u64,
    pub attr_valid: u64,
    pub entry_valid_nsec: u32,
    pub attr_valid_nsec: u32,
    pub attr: fuse_attr,
}

#[repr(C)]
#[derive(Debug, FromBytes, KnownLayout, Immutable, Copy, Clone)]
pub struct fuse_forget_in {
    pub nlookup: u64,
}

#[cfg(feature = "abi-7-16")]
#[repr(C)]
#[derive(Debug, FromBytes, KnownLayout, Immutable, Copy, Clone)]
pub struct fuse_forget_one {
    pub nodeid: u64,
    pub nlookup: u64,
}

#[cfg(feature = "abi-7-16")]
#[repr(C)]
#[derive(Debug, FromBytes, KnownLayout, Immutable, Clone, Copy)]
pub struct fuse_batch_forget_in {
    pub count: u32,
    pub dummy: u32,
}

#[cfg(feature = "abi-7-9")]
#[repr(C)]
#[derive(Debug, FromBytes, KnownLayout, Immutable, Clone, Copy)]
pub struct fuse_getattr_in {
    pub getattr_flags: u32,
    pub dummy: u32,
    pub fh: u64,
}

#[repr(C)]
#[derive(Debug, IntoBytes, KnownLayout, Immutable, Copy, Clone)]
pub struct fuse_attr_out {
    pub attr_valid: u64,
    pub attr_valid_nsec: u32,
    pub dummy: u32,
    pub attr: fuse_attr,
}

#[cfg(target_os = "macos")]
#[repr(C)]
#[derive(Debug, IntoBytes, KnownLayout, Immutable, Copy, Clone)]
pub struct fuse_getxtimes_out {
    pub bkuptime: u64,
    pub crtime: u64,
    pub bkuptimensec: u32,
    pub crtimensec: u32,
}

#[repr(C)]
#[derive(Debug, IntoBytes, FromBytes, KnownLayout, Immutable, Clone, Copy)]
#[allow(non_camel_case_types)]
/// See [man](https://man7.org/linux/man-pages/man2/mknod.2.html)
pub struct fuse_mknod_in {
    /// `mode` and `umask` together (`mode & ~umask`) specify the file mode
    pub mode: u32,
    /// If the file type is S_IFCHR or S_IFBLK then `dev` specifies the major and minor
    /// numbers of the newly created device special file.
    ///
    /// See [man](https://man7.org/linux/man-pages/man2/mknod.2.html)
    pub rdev: u32,
    #[cfg(feature = "abi-7-12")]
    /// `mode` and `umask` together (`mode & ~umask`) specify the file mode
    pub umask: u32,
    #[cfg(feature = "abi-7-12")]
    pub padding: u32,
}

#[repr(C)]
#[derive(Debug, FromBytes, IntoBytes, KnownLayout, Immutable, Clone, Copy)]
pub struct fuse_mkdir_in {
    pub mode: u32,
    #[cfg(not(feature = "abi-7-12"))]
    pub padding: u32,
    #[cfg(feature = "abi-7-12")]
    pub umask: u32,
}

#[repr(C)]
#[derive(Debug, IntoBytes, FromBytes, KnownLayout, Immutable, Clone, Copy)]
pub struct fuse_rename_in {
    pub newdir: u64,
    #[cfg(feature = "macfuse-4-compat")]
    pub flags: u32,
    #[cfg(feature = "macfuse-4-compat")]
    pub padding: u32,
}

#[repr(C)]
#[derive(Debug, IntoBytes, FromBytes, KnownLayout, Immutable, Clone, Copy)]
pub struct fuse_rename2_in {
    pub newdir: u64,
    pub flags: u32,
    pub padding: u32,
}

#[cfg(target_os = "macos")]
#[repr(C)]
#[derive(Debug, FromBytes, KnownLayout, Immutable, Clone, Copy)]
pub struct fuse_exchange_in {
    pub olddir: u64,
    pub newdir: u64,
    pub options: u64,
}

#[repr(C)]
#[derive(Debug, IntoBytes, FromBytes, KnownLayout, Immutable, Clone, Copy)]
pub struct fuse_link_in {
    pub oldnodeid: u64,
}

#[repr(C)]
#[derive(Debug, FromBytes, KnownLayout, Immutable, Clone)]
pub struct fuse_setattr_in {
    pub valid: u32,
    pub padding: u32,
    pub fh: u64,
    pub size: u64,
    #[cfg(not(feature = "abi-7-9"))]
    pub unused1: u64,
    #[cfg(feature = "abi-7-9")]
    pub lock_owner: u64,
    // NOTE: this field is defined as u64 in fuse_kernel.h in libfuse. However, it is treated as signed
    // to match stat.st_atime
    pub atime: i64,
    // NOTE: this field is defined as u64 in fuse_kernel.h in libfuse. However, it is treated as signed
    // to match stat.st_mtime
    pub mtime: i64,
    #[cfg(not(feature = "abi-7-23"))]
    pub unused2: u64,
    #[cfg(feature = "abi-7-23")]
    // NOTE: this field is defined as u64 in fuse_kernel.h in libfuse. However, it is treated as signed
    // to match stat.st_ctime
    pub ctime: i64,
    pub atimensec: u32,
    pub mtimensec: u32,
    #[cfg(not(feature = "abi-7-23"))]
    pub unused3: u32,
    #[cfg(feature = "abi-7-23")]
    pub ctimensec: u32,
    pub mode: u32,
    pub unused4: u32,
    pub uid: u32,
    pub gid: u32,
    pub unused5: u32,
    #[cfg(target_os = "macos")]
    pub bkuptime: u64,
    #[cfg(target_os = "macos")]
    pub chgtime: u64,
    #[cfg(target_os = "macos")]
    pub crtime: u64,
    #[cfg(target_os = "macos")]
    pub bkuptimensec: u32,
    #[cfg(target_os = "macos")]
    pub chgtimensec: u32,
    #[cfg(target_os = "macos")]
    pub crtimensec: u32,
    #[cfg(target_os = "macos")]
    pub flags: u32, // see chflags(2)
}

impl fuse_setattr_in {
    #[cfg(feature = "abi-7-9")]
    pub fn atime_now(&self) -> bool {
        self.valid & FATTR_ATIME_NOW != 0
    }

    #[cfg(not(feature = "abi-7-9"))]
    pub fn atime_now(&self) -> bool {
        false
    }

    #[cfg(feature = "abi-7-9")]
    pub fn mtime_now(&self) -> bool {
        self.valid & FATTR_MTIME_NOW != 0
    }

    #[cfg(not(feature = "abi-7-9"))]
    pub fn mtime_now(&self) -> bool {
        false
    }
}

#[repr(C)]
#[derive(Debug, FromBytes, KnownLayout, Immutable, Clone, Copy)]
pub struct fuse_open_in {
    // NOTE: this field is defined as u32 in fuse_kernel.h in libfuse. However, it is then cast
    // to an i32 when invoking the filesystem's open method and this matches the open() syscall
    // TODO should be u32
    pub flags: i32,
    // TODO should be called 'open_flags` to match fuse_kernel.h
    // Options specified by FUSE_OPEN_ ... constants
    pub unused: u32,
}

#[repr(C)]
#[derive(Debug, IntoBytes, FromBytes, KnownLayout, Immutable, Clone, Copy, Default)]
pub struct fuse_create_in {
    /// Open flags from constants like [libc::O_DIRECT]
    ///
    /// See [open](https://man7.org/linux/man-pages/man2/open.2.html)
    // TODO should be u32 to match fuse_kernel.h
    pub flags: i32,
    /// File type and permissions
    ///
    /// e.g. [libc::S_IFREG] and [libc::S_IRWXU] [libc::S_IWUSER]
    ///
    /// See [open](https://man7.org/linux/man-pages/man2/open.2.html)
    /// See [creat](https://www.man7.org/linux/man-pages/man3/creat.3p.html)
    pub mode: u32,
    /// Set of permissions for a process that indicate what the process CAN NOT do
    #[cfg(feature = "abi-7-12")]
    pub umask: u32,
    #[cfg(feature = "abi-7-12")]
    pub padding: u32,
}

#[repr(C)]
#[derive(Debug, IntoBytes, KnownLayout, Immutable, Clone, Copy)]
pub struct fuse_create_out {
    pub entry: fuse_entry_out,
    pub open: fuse_open_out,
}

#[repr(C)]
#[derive(Debug, IntoBytes, KnownLayout, Immutable, Clone, Copy)]
pub struct fuse_open_out {
    /// File descriptor
    pub fh: u64,
    /// FUSE flags starting with FUSE_OPEN_ ...
    pub open_flags: u32,
    pub padding: u32,
}

#[repr(C)]
#[derive(Debug, FromBytes, KnownLayout, Immutable, Clone, Copy)]
pub struct fuse_release_in {
    pub fh: u64,
    // NOTE: this field is defined as u32 in fuse_kernel.h in libfuse. However, it is then cast
    // to an i32 when invoking the filesystem's read method
    pub flags: i32,
    pub release_flags: u32,
    pub lock_owner: u64,
}

#[repr(C)]
#[derive(Debug, FromBytes, KnownLayout, Immutable, Clone, Copy)]
pub struct fuse_flush_in {
    pub fh: u64,
    pub unused: u32,
    pub padding: u32,
    pub lock_owner: u64,
}

#[repr(C)]
#[derive(Debug, FromBytes, KnownLayout, Immutable, Clone, Copy)]
pub struct fuse_read_in {
    pub fh: u64,
    // NOTE: this field is defined as u64 in fuse_kernel.h in libfuse. However, it is then cast
    // to an i64 when invoking the filesystem's read method
    pub offset: i64,
    pub size: u32,
    #[cfg(feature = "abi-7-9")]
    pub read_flags: u32,
    #[cfg(feature = "abi-7-9")]
    pub lock_owner: u64,
    #[cfg(feature = "abi-7-9")]
    // NOTE: this field is defined as u32 in fuse_kernel.h in libfuse. However, it is then cast
    // to an i32 when invoking the filesystem's read method
    pub flags: i32,
    #[cfg(feature = "abi-7-9")]
    pub padding: u32,
}

#[repr(C)]
#[derive(Debug, FromBytes, KnownLayout, Immutable, Clone, Copy)]
pub struct fuse_write_in {
    pub fh: u64,
    // NOTE: this field is defined as u64 in fuse_kernel.h in libfuse. However, it is then cast
    // to an i64 when invoking the filesystem's write method
    pub offset: i64,
    pub size: u32,
    pub write_flags: u32,
    #[cfg(feature = "abi-7-9")]
    pub lock_owner: u64,
    #[cfg(feature = "abi-7-9")]
    // NOTE: this field is defined as u32 in fuse_kernel.h in libfuse. However, it is then cast
    // to an i32 when invoking the filesystem's read method
    pub flags: i32,
    #[cfg(feature = "abi-7-9")]
    pub padding: u32,
}

#[repr(C)]
#[derive(Debug, IntoBytes, KnownLayout, Immutable, Clone, Copy)]
pub struct fuse_write_out {
    pub size: u32,
    pub padding: u32,
}

#[repr(C)]
#[derive(Debug, IntoBytes, KnownLayout, Immutable, Clone, Copy)]
pub struct fuse_statfs_out {
    pub st: fuse_kstatfs,
}

#[repr(C)]
#[derive(Debug, FromBytes, KnownLayout, Immutable, Clone, Copy)]
pub struct fuse_fsync_in {
    pub fh: u64,
    pub fsync_flags: u32,
    pub padding: u32,
}

#[repr(C)]
#[derive(Debug, IntoBytes, FromBytes, KnownLayout, Immutable, Clone, Copy)]
pub struct fuse_setxattr_in {
    pub size: u32,
    // NOTE: this field is defined as u32 in fuse_kernel.h in libfuse. However, it is then cast
    // to an i32 when invoking the filesystem's setxattr method
    pub flags: i32,
    #[cfg(target_os = "macos")]
    pub position: u32,
    #[cfg(target_os = "macos")]
    pub padding: u32,
}

#[repr(C)]
#[derive(Debug, IntoBytes, FromBytes, KnownLayout, Immutable, Clone, Copy)]
pub struct fuse_getxattr_in {
    pub size: u32,
    pub padding: u32,
    #[cfg(target_os = "macos")]
    pub position: u32,
    #[cfg(target_os = "macos")]
    pub padding2: u32,
}

#[repr(C)]
#[derive(Debug, IntoBytes, KnownLayout, Immutable, Clone, Copy)]
pub struct fuse_getxattr_out {
    pub size: u32,
    pub padding: u32,
}

#[repr(C)]
#[derive(Debug, FromBytes, KnownLayout, Immutable, Clone, Copy)]
pub struct fuse_lk_in {
    pub fh: u64,
    pub owner: u64,
    pub lk: fuse_file_lock,
    #[cfg(feature = "abi-7-9")]
    pub lk_flags: u32,
    #[cfg(feature = "abi-7-9")]
    pub padding: u32,
}

#[repr(C)]
#[derive(Debug, IntoBytes, KnownLayout, Immutable, Clone, Copy)]
pub struct fuse_lk_out {
    pub lk: fuse_file_lock,
}

#[repr(C)]
#[derive(Debug, FromBytes, KnownLayout, Immutable, Clone, Copy)]
pub struct fuse_access_in {
    // NOTE: this field is defined as u32 in fuse_kernel.h in libfuse. However, it is then cast
    // to an i32 when invoking the filesystem's access method
    pub mask: i32,
    pub padding: u32,
}

#[repr(C)]
#[derive(Debug, FromBytes, KnownLayout, Immutable, Clone, Copy)]
pub struct fuse_init_in {
    pub major: u32,
    pub minor: u32,
    pub max_readahead: u32,
    pub flags: u32,
}

#[repr(C)]
#[derive(Debug, IntoBytes, KnownLayout, Immutable)]
/// Iniitialization parameters see: [fuse_common.h](https://github.com/libfuse/libfuse/blob/master/include/fuse_common.h)
///
/// Also see [crate::constants]
pub struct fuse_init_out {
    /// ABI version major
    pub major: u32,
    /// ABI version minor
    pub minor: u32,
    pub max_readahead: u32,
    /// See init flags in [crate::constants]
    pub flags: u32,
    #[cfg(not(feature = "abi-7-13"))]
    pub unused: u32,
    /// Maximum number of background (pending) requests the kernel should submit at a time
    #[cfg(feature = "abi-7-13")]
    pub max_background: u16,
    #[cfg(feature = "abi-7-13")]
    pub congestion_threshold: u16,
    /// Maximum number of bytes in a single write operation
    pub max_write: u32,
    /// Supported time granularity
    #[cfg(feature = "abi-7-23")]
    pub time_gran: u32,
    #[cfg(all(feature = "abi-7-23", not(feature = "abi-7-28")))]
    pub reserved: [u32; 9],
    /// Maximum number of pages to submit in a single request
    #[cfg(feature = "abi-7-28")]
    pub max_pages: u16,
    #[cfg(feature = "abi-7-28")]
    pub unused2: u16,
    #[cfg(feature = "abi-7-28")]
    pub reserved: [u32; 8],
}

#[cfg(feature = "abi-7-12")]
#[repr(C)]
#[derive(Debug, FromBytes, KnownLayout, Immutable)]
pub struct cuse_init_in {
    pub major: u32,
    pub minor: u32,
    pub unused: u32,
    pub flags: u32,
}

#[cfg(feature = "abi-7-12")]
#[repr(C)]
#[derive(Debug, KnownLayout, Immutable)]
pub struct cuse_init_out {
    pub major: u32,
    pub minor: u32,
    pub unused: u32,
    pub flags: u32,
    pub max_read: u32,
    pub max_write: u32,
    pub dev_major: u32, // chardev major
    pub dev_minor: u32, // chardev minor
    pub spare: [u32; 10],
}

#[repr(C)]
#[derive(Debug, FromBytes, KnownLayout, Immutable, Clone, Copy)]
pub struct fuse_interrupt_in {
    pub unique: u64,
}

#[repr(C)]
#[derive(Debug, FromBytes, KnownLayout, Immutable, Clone, Copy)]
pub struct fuse_bmap_in {
    pub block: u64,
    pub blocksize: u32,
    pub padding: u32,
}

#[repr(C)]
#[derive(Debug, IntoBytes, KnownLayout, Immutable, Clone, Copy)]
pub struct fuse_bmap_out {
    pub block: u64,
}

#[cfg(feature = "abi-7-11")]
#[repr(C)]
#[derive(Debug, FromBytes, KnownLayout, Immutable, Clone, Copy)]
pub struct fuse_ioctl_in {
    pub fh: u64,
    pub flags: u32,
    pub cmd: u32,
    pub arg: u64, // TODO: this is currently unused, but is defined as a void* in libfuse
    pub in_size: u32,
    pub out_size: u32,
}

#[cfg(feature = "abi-7-16")]
#[repr(C)]
#[derive(Debug, KnownLayout, Immutable, Clone, Copy)]
pub struct fuse_ioctl_iovec {
    pub base: u64,
    pub len: u64,
}

#[repr(C)]
#[derive(Debug, IntoBytes, KnownLayout, Immutable, Clone, Copy)]
pub struct fuse_ioctl_out {
    pub result: i32,
    pub flags: u32,
    pub in_iovs: u32,
    pub out_iovs: u32,
}

#[cfg(feature = "abi-7-11")]
#[repr(C)]
#[derive(Debug, FromBytes, KnownLayout, Immutable, Clone, Copy)]
pub struct fuse_poll_in {
    pub fh: u64,
    pub kh: u64,
    pub flags: u32,
    #[cfg(not(feature = "abi-7-21"))]
    pub padding: u32,
    #[cfg(feature = "abi-7-21")]
    pub events: u32,
}

#[cfg(feature = "abi-7-11")]
#[repr(C)]
#[derive(Debug, IntoBytes, KnownLayout, Immutable, Clone, Copy)]
pub struct fuse_poll_out {
    pub revents: u32,
    pub padding: u32,
}

#[cfg(feature = "abi-7-11")]
#[repr(C)]
#[derive(Debug, IntoBytes, KnownLayout, Immutable, Clone, Copy)]
pub struct fuse_notify_poll_wakeup_out {
    pub kh: u64,
}

#[cfg(feature = "abi-7-19")]
#[repr(C)]
#[derive(Debug, FromBytes, KnownLayout, Immutable, Clone, Copy)]
pub struct fuse_fallocate_in {
    pub fh: u64,
    // NOTE: this field is defined as u64 in fuse_kernel.h in libfuse. However, it is treated as signed
    pub offset: i64,
    // NOTE: this field is defined as u64 in fuse_kernel.h in libfuse. However, it is treated as signed
    pub length: i64,
    // NOTE: this field is defined as u32 in fuse_kernel.h in libfuse. However, it is treated as signed
    pub mode: i32,
    pub padding: u32,
}

#[repr(C)]
#[derive(Debug, IntoBytes, FromBytes, KnownLayout, Immutable, Clone, Copy)]
pub struct fuse_in_header {
    pub len: u32,
    pub opcode: u32,
    pub unique: u64,
    pub nodeid: u64,
    pub uid: u32,
    pub gid: u32,
    pub pid: u32,
    pub padding: u32,
}

#[repr(C)]
#[derive(Debug, IntoBytes, KnownLayout, Immutable, Clone, Copy, TryFromBytes)]
pub struct fuse_out_header {
    /// Length of the entire message including the header
    ///
    /// If using [crate::messages::reply::Reply], will be automatically calculated during write
    pub len: u32,
    /// Must be the negative of the error value from `libc` e.g. [libc::EIO] should be `-5'
    pub error: i32,
    /// Unique id for the request/reply pair. See [fuse_in_header]
    pub unique: u64,
}

impl fuse_out_header {
    pub fn set_error(&mut self, error: crate::error::Errno) {
        self.error = error.into();
    }
}

impl From<fuse_in_header> for fuse_out_header {
    fn from(value: fuse_in_header) -> Self {
        Self {
            len: 0,
            unique: value.unique,
            error: 0,
        }
    }
}

#[repr(C)]
#[derive(Debug, IntoBytes, KnownLayout, Immutable, Clone, Copy)]
pub struct fuse_dirent {
    pub ino: u64,
    // NOTE: this field is defined as u64 in fuse_kernel.h in libfuse. However, it is treated as signed
    pub off: i64,
    pub namelen: u32,
    /// See [st_mode](https://man7.org/linux/man-pages/man7/inode.7.html)
    /// Only stores the type, not the permissions. Shift off the low 12 bits from `st_mode`
    ///
    /// To set correctly, use [fuse_dirent::set_type]
    pub typ: u32,
    // followed by name of namelen bytes
}

impl fuse_dirent {
    /// kind is the file type from libc e.g. [libc::S_IFREG] for a regular file.
    pub fn set_type(&mut self, kind: u32) {
        self.typ = kind >> 12;
    }
}

#[repr(C)]
#[derive(Debug, IntoBytes, KnownLayout, Immutable, Clone, Copy)]
pub struct fuse_direntplus {
    pub entry_out: fuse_entry_out,
    pub dirent: fuse_dirent,
}

#[cfg(feature = "abi-7-12")]
#[repr(C)]
#[derive(Debug, IntoBytes, KnownLayout, Immutable, Clone, Copy)]
pub struct fuse_notify_inval_inode_out {
    pub ino: u64,
    pub off: i64,
    pub len: i64,
}

#[cfg(feature = "abi-7-12")]
#[repr(C)]
#[derive(Debug, IntoBytes, KnownLayout, Immutable, Clone, Copy)]
pub struct fuse_notify_inval_entry_out {
    pub parent: u64,
    pub namelen: u32,
    pub padding: u32,
}

#[cfg(feature = "abi-7-18")]
#[repr(C)]
#[derive(Debug, IntoBytes, KnownLayout, Immutable, Clone, Copy)]
pub struct fuse_notify_delete_out {
    pub parent: u64,
    pub child: u64,
    pub namelen: u32,
    pub padding: u32,
}

#[cfg(feature = "abi-7-15")]
#[repr(C)]
#[derive(Debug, IntoBytes, KnownLayout, Immutable, Clone, Copy)]
pub struct fuse_notify_store_out {
    pub nodeid: u64,
    pub offset: u64,
    pub size: u32,
    pub padding: u32,
}

#[cfg(feature = "abi-7-15")]
#[repr(C)]
#[derive(Debug, KnownLayout, Immutable, Clone, Copy)]
pub struct fuse_notify_retrieve_out {
    pub notify_unique: u64,
    pub nodeid: u64,
    pub offset: u64,
    pub size: u32,
    pub padding: u32,
}

#[cfg(feature = "abi-7-15")]
#[repr(C)]
#[derive(Debug, FromBytes, KnownLayout, Immutable, Clone, Copy)]
pub struct fuse_notify_retrieve_in {
    // matches the size of fuse_write_in
    pub dummy1: u64,
    pub offset: u64,
    pub size: u32,
    pub dummy2: u32,
    pub dummy3: u64,
    pub dummy4: u64,
}

#[repr(C)]
#[derive(Debug, FromBytes, KnownLayout, Immutable, Clone, Copy)]
pub struct fuse_lseek_in {
    pub fh: u64,
    pub offset: u64,
    // NOTE: this field is defined as u32 in fuse_kernel.h in libfuse. However, it is treated as signed
    pub whence: u32,
    pub padding: u32,
}

#[repr(C)]
#[derive(Debug, IntoBytes, KnownLayout, Immutable, Clone, Copy)]
pub struct fuse_lseek_out {
    pub offset: u64,
}

#[repr(C)]
#[derive(Debug, FromBytes, KnownLayout, Immutable, Clone, Copy)]
pub struct fuse_copy_file_range_in {
    pub fh_in: u64,
    // NOTE: this field is defined as u64 in fuse_kernel.h in libfuse. However, it is treated as signed
    pub off_in: i64,
    pub nodeid_out: u64,
    pub fh_out: u64,
    // NOTE: this field is defined as u64 in fuse_kernel.h in libfuse. However, it is treated as signed
    pub off_out: i64,
    pub len: u64,
    pub flags: u64,
}

#[cfg(feature = "abi-7-31")]
#[repr(C)]
#[derive(Debug, FromBytes, KnownLayout, Immutable, Clone, Copy, Default)]
pub struct fuse_setupmapping_in {
    pub fh: u64,
    pub foffset: u64,
    pub len: u64,
    pub flags: u64,
    pub moffset: u64,
}

#[cfg(feature = "abi-7-31")]
#[repr(C)]
#[derive(Debug, FromBytes, KnownLayout, Immutable, Clone, Copy, Default)]
pub struct fuse_removemapping_in {
    pub count: u32,
}

#[cfg(feature = "abi-7-31")]
#[repr(C)]
#[derive(Debug, FromBytes, KnownLayout, Immutable, Clone, Copy)]
pub struct fuse_removemapping_one {
    moffset: u64,
    len: u64,
}

#[cfg(feature = "abi-7-34")]
#[repr(C)]
#[derive(Debug, FromBytes, KnownLayout, Immutable, Clone, Copy)]
pub struct fuse_syncfs_in {
    padding: u64,
}

impl Default for fuse_syncfs_in {
    fn default() -> Self {
        Self { padding: 0 }
    }
}

#[cfg(feature = "abi-7-39")]
#[repr(C)]
#[derive(Debug, IntoBytes, FromBytes, KnownLayout, Immutable, Clone, Copy, Default)]
pub struct fuse_sx_time {
    pub tv_sec: i64,
    pub tv_nsec: u32,
    pub __reserved: i32,
}

#[cfg(feature = "abi-7-39")]
#[repr(C)]
#[derive(Debug, IntoBytes, FromBytes, KnownLayout, Immutable, Clone, Copy, Default)]
pub struct fuse_statx {
    pub mask: u32,
    pub blksize: u32,
    pub attributes: u64,
    pub nlink: u32,
    pub uid: u32,
    pub gid: u32,
    pub mode: u16,
    pub __spare0: [u16; 1],
    pub ino: u64,
    pub size: u64,
    pub blocks: u64,
    pub attributes_mask: u64,
    pub atime: fuse_sx_time,
    pub btime: fuse_sx_time,
    pub ctime: fuse_sx_time,
    pub mtime: fuse_sx_time,
    pub rdev_major: u32,
    pub rdev_minor: u32,
    pub dev_major: u32,
    pub dev_minor: u32,
    pub __spare2: [u64; 14],
}

#[cfg(feature = "abi-7-39")]
#[repr(C)]
#[derive(Debug, IntoBytes, FromBytes, KnownLayout, Immutable, Clone, Copy, Default)]
pub struct fuse_statx_in {
    pub getattr_flags: u32,
    pub reserved: u32,
    pub fh: u64,
    pub sx_flags: u32,
    pub sx_mask: u32,
}

#[cfg(feature = "abi-7-39")]
#[repr(C)]
#[derive(Debug, IntoBytes, FromBytes, KnownLayout, Immutable, Clone, Copy)]
pub struct fuse_statx_out {
    pub attr_valid: u64,
    pub attr_valid_nsec: u32,
    pub flags: u32,
    pub spare: [u64; 2],
    pub stat: fuse_statx,
}
