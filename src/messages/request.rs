use std::fmt::{self, Display};
use std::sync::Arc;

use log::error;
use zerocopy::FromBytes;

use crate::error::Errno;
use crate::messages::argument::get_vec;
use crate::messages::fuse_abi::*;
use crate::{messages::argument::get_string, ReplyTx};
use tokio::sync::Mutex;

use super::reply::NotifyReply;

pub struct Request {
    pub header: fuse_in_header,
    pub operation: Operation,
    pub reply_to: ReplyTx,
}

impl Request {
    pub fn from_op(op: Operation, reply_to: &ReplyTx) -> Self {
        Self {
            header: fuse_in_header {
                uid: 0,
                gid: 0,
                pid: 0,
                len: 0,
                nodeid: 0,
                unique: 0,
                padding: 0,
                opcode: op.get_opcode(),
            },
            operation: op,
            reply_to: reply_to.clone(),
        }
    }

    pub fn parse(buffer: &mut [u8], reply_to: &ReplyTx) -> Result<Self, Errno> {
        let (header, rest) = match fuse_in_header::mut_from_prefix(buffer) {
            Err(_e) => return Err(Errno::EIO),
            Ok((h, r)) => (*h, r),
        };

        let operation = match fuse_opcode::try_from(header.opcode) {
            Err(_e) => {
                error!("invalid op code {:?}", _e);
                return Err(Errno::EIO);
            }
            Ok(opcode) => match opcode {
                // TODO error handling - these will panic
                fuse_opcode::FUSE_LOOKUP => {
                    let (name, _rest) = get_string(rest);
                    Operation::Lookup(Lookup { name })
                }
                fuse_opcode::FUSE_FORGET => Operation::Forget(Forget {
                    arg: *fuse_forget_in::ref_from_prefix(buffer).unwrap().0,
                }),
                fuse_opcode::FUSE_GETATTR => Operation::GetAttr(GetAttr {
                    #[cfg(feature = "abi-7-9")]
                    arg: *fuse_getattr_in::ref_from_prefix(rest).unwrap().0,
                }),
                fuse_opcode::FUSE_SETATTR => Operation::SetAttr(SetAttr {
                    arg: fuse_setattr_in::ref_from_prefix(rest).unwrap().0.clone(),
                }),
                fuse_opcode::FUSE_READLINK => Operation::ReadLink(ReadLink {}),
                fuse_opcode::FUSE_SYMLINK => {
                    let (name, rest) = get_string(rest);
                    let (target, _rest) = get_string(rest);
                    Operation::SymLink(SymLink { name, target })
                }
                fuse_opcode::FUSE_MKNOD => {
                    let (arg, rest) = fuse_mknod_in::mut_from_prefix(rest).unwrap();
                    let name = get_string(rest).0;
                    Operation::MkNod(MkNod { arg: *arg, name })
                }
                fuse_opcode::FUSE_MKDIR => {
                    let (arg, rest) = fuse_mkdir_in::mut_from_prefix(rest).unwrap();
                    let (name, _rest) = get_string(rest);
                    Operation::MkDir(MkDir { arg: *arg, name })
                }
                fuse_opcode::FUSE_UNLINK => {
                    let (name, _rest) = get_string(rest);
                    Operation::Unlink(Unlink { name })
                }
                fuse_opcode::FUSE_RMDIR => {
                    let (name, _rest) = get_string(rest);
                    Operation::RmDir(RmDir { name })
                }
                fuse_opcode::FUSE_RENAME => {
                    let (arg, rest) = fuse_rename_in::mut_from_prefix(rest).unwrap();
                    let (name, rest) = get_string(rest);
                    let (newname, _rest) = get_string(rest);
                    Operation::Rename(Rename {
                        arg: *arg,
                        name,
                        newname,
                    })
                }
                fuse_opcode::FUSE_LINK => {
                    let (arg, rest) = fuse_link_in::mut_from_prefix(rest).unwrap();
                    let (name, _rest) = get_string(rest);
                    Operation::Link(Link { arg: *arg, name })
                }
                fuse_opcode::FUSE_OPEN => {
                    let (arg, _rest) = fuse_open_in::ref_from_prefix(rest).unwrap();
                    Operation::Open(Open { arg: *arg })
                }
                fuse_opcode::FUSE_READ => {
                    let (arg, _rest) = fuse_read_in::ref_from_prefix(rest).unwrap();
                    Operation::Read(Read { arg: *arg })
                }
                fuse_opcode::FUSE_WRITE => {
                    let (arg, rest2) = fuse_write_in::ref_from_prefix(rest).unwrap();
                    Operation::Write(Write {
                        arg: *arg,
                        data: Arc::new(Mutex::new(rest2[..arg.size as usize].to_vec())),
                    })
                }
                fuse_opcode::FUSE_STATFS => Operation::StatFs(StatFs {}),
                fuse_opcode::FUSE_RELEASE => {
                    let (arg, _rest) = fuse_release_in::ref_from_prefix(rest).unwrap();
                    Operation::Release(Release { arg: *arg })
                }
                fuse_opcode::FUSE_FSYNC => {
                    let (arg, _rest) = fuse_fsync_in::ref_from_prefix(rest).unwrap();
                    Operation::FSync(FSync { arg: *arg })
                }
                fuse_opcode::FUSE_SETXATTR => {
                    let (arg, rest) = fuse_setxattr_in::mut_from_prefix(rest).unwrap();
                    let (name, rest) = get_string(rest);
                    let value = Arc::new(get_vec(rest, arg.size as usize));
                    Operation::SetXAttr(SetXAttr { arg: *arg, name, value })
                }
                fuse_opcode::FUSE_GETXATTR => {
                    let (arg, rest) = fuse_getxattr_in::mut_from_prefix(rest).unwrap();
                    let (name, _rest) = get_string(rest);
                    Operation::GetXAttr(GetXAttr { arg: *arg, name })
                }
                fuse_opcode::FUSE_LISTXATTR => {
                    let (arg, _rest) = fuse_getxattr_in::ref_from_prefix(rest).unwrap();
                    Operation::ListXAttr(ListXAttr { arg: *arg })
                }
                fuse_opcode::FUSE_REMOVEXATTR => {
                    let (name, _rest) = get_string(rest);
                    Operation::RemoveXAttr(RemoveXAttr { name })
                }
                fuse_opcode::FUSE_FLUSH => {
                    let (arg, _rest) = fuse_flush_in::ref_from_prefix(rest).unwrap();
                    Operation::Flush(Flush { arg: *arg })
                }
                fuse_opcode::FUSE_INIT => {
                    let (arg, _rest) = fuse_init_in::ref_from_prefix(rest).unwrap();
                    Operation::Init(Init { arg: *arg })
                }
                fuse_opcode::FUSE_OPENDIR => {
                    let (arg, _rest) = fuse_open_in::ref_from_prefix(rest).unwrap();
                    Operation::OpenDir(OpenDir { arg: *arg })
                }
                fuse_opcode::FUSE_READDIR => {
                    let (arg, _rest) = fuse_read_in::ref_from_prefix(rest).unwrap();
                    Operation::ReadDir(ReadDir { arg: *arg })
                }
                fuse_opcode::FUSE_RELEASEDIR => {
                    let (arg, _rest) = fuse_release_in::ref_from_prefix(rest).unwrap();
                    Operation::ReleaseDir(ReleaseDir { arg: *arg })
                }
                fuse_opcode::FUSE_FSYNCDIR => {
                    let (arg, _rest) = fuse_fsync_in::ref_from_prefix(rest).unwrap();
                    Operation::FSyncDir(FSyncDir { arg: *arg })
                }
                fuse_opcode::FUSE_GETLK => {
                    let (arg, _rest) = fuse_lk_in::ref_from_prefix(rest).unwrap();
                    Operation::GetLk(GetLk { arg: *arg })
                }
                fuse_opcode::FUSE_SETLK => {
                    let (arg, _rest) = fuse_lk_in::ref_from_prefix(rest).unwrap();
                    Operation::SetLk(SetLk { arg: *arg })
                }
                fuse_opcode::FUSE_SETLKW => {
                    let (arg, _rest) = fuse_lk_in::ref_from_prefix(rest).unwrap();
                    Operation::SetLkW(SetLkW { arg: *arg })
                }
                fuse_opcode::FUSE_ACCESS => {
                    let (arg, _rest) = fuse_access_in::ref_from_prefix(rest).unwrap();
                    Operation::Access(Access { arg: *arg })
                }
                fuse_opcode::FUSE_CREATE => {
                    let (arg, rest) = fuse_create_in::mut_from_prefix(rest).unwrap();
                    let (name, _rest) = get_string(rest);
                    Operation::Create(Create { arg: *arg, name })
                }
                fuse_opcode::FUSE_INTERRUPT => {
                    let (arg, _rest) = fuse_interrupt_in::ref_from_prefix(rest).unwrap();
                    Operation::Interrupt(Interrupt { arg: *arg })
                }
                fuse_opcode::FUSE_BMAP => {
                    let (arg, _rest) = fuse_bmap_in::ref_from_prefix(rest).unwrap();
                    Operation::BMap(BMap { arg: *arg })
                }
                fuse_opcode::FUSE_DESTROY => Operation::Destroy(Destroy {}),
                // TODO
                #[cfg(feature = "abi-7-11")]
                fuse_opcode::FUSE_IOCTL => {
                    let (arg, rest) = fuse_ioctl_in::ref_from_prefix(rest).unwrap();
                    let data = rest.to_vec();

                    Operation::IoCtl(IoCtl { arg: *arg, data })
                }
                #[cfg(feature = "abi-7-11")]
                fuse_opcode::FUSE_POLL => {
                    let (arg, _rest) = fuse_poll_in::ref_from_prefix(rest).unwrap();
                    Operation::Poll(Poll { arg: *arg })
                }
                #[cfg(feature = "abi-7-15")]
                fuse_opcode::FUSE_NOTIFY_REPLY => Operation::NotifyReply(NotifyReply {}),
                #[cfg(feature = "abi-7-16")]
                fuse_opcode::FUSE_BATCH_FORGET => {
                    let (arg, rest) = fuse_batch_forget_in::ref_from_prefix(rest).unwrap();
                    let nodes = get_vec::<fuse_forget_one>(rest, arg.count as usize);
                    Operation::BatchForget(BatchForget { arg: *arg, nodes })
                }
                #[cfg(feature = "abi-7-19")]
                fuse_opcode::FUSE_FALLOCATE => {
                    let (arg, _rest) = fuse_fallocate_in::ref_from_prefix(rest).unwrap();
                    Operation::FAllocate(FAllocate { arg: *arg })
                }
                #[cfg(feature = "abi-7-21")]
                fuse_opcode::FUSE_READDIRPLUS => {
                    let (arg, _rest) = fuse_read_in::ref_from_prefix(rest).unwrap();
                    Operation::ReadDirPlus(ReadDirPlus { arg: *arg })
                }
                #[cfg(feature = "abi-7-23")]
                fuse_opcode::FUSE_RENAME2 => {
                    let (arg, rest) = fuse_rename2_in::mut_from_prefix(rest).unwrap();
                    let (name, rest) = get_string(rest);
                    let (newname, _rest) = get_string(rest);
                    Operation::Rename2(Rename2 {
                        arg: *arg,
                        name,
                        newname,
                        old_parent: header.nodeid,
                    })
                }
                #[cfg(feature = "abi-7-24")]
                fuse_opcode::FUSE_LSEEK => {
                    let (arg, _rest) = fuse_lseek_in::ref_from_prefix(rest).unwrap();
                    Operation::LSeek(LSeek { arg: *arg })
                }
                #[cfg(feature = "abi-7-28")]
                fuse_opcode::FUSE_COPY_FILE_RANGE => {
                    let (arg, _rest) = fuse_copy_file_range_in::ref_from_prefix(rest).unwrap();
                    Operation::CopyFileRange(CopyFileRange { arg: *arg })
                }
                // TODO complete mappings
                #[cfg(feature = "abi-7-31")]
                fuse_opcode::FUSE_SETUPMAPPING => Operation::SetupMapping(SetupMapping::default()),
                #[cfg(feature = "abi-7-31")]
                fuse_opcode::FUSE_REMOVEMAPPING => Operation::RemoveMapping(RemoveMapping::default()),
                #[cfg(feature = "abi-7-34")]
                fuse_opcode::FUSE_SYNCFS => Operation::SyncFs(SyncFs::default()),
                #[cfg(feature = "abi-7-37")]
                fuse_opcode::FUSE_TMPFILE => Operation::TmpFile(TmpFile::default()),
                #[cfg(feature = "abi-7-39")]
                fuse_opcode::FUSE_STATX => Operation::StatX(StatX::default()),
                #[cfg(target_os = "macos")]
                fuse_opcode::FUSE_SETVOLNAME => Operation::SetVolName(SetVolName {}),
                #[cfg(target_os = "macos")]
                fuse_opcode::FUSE_GETX_TIMES => Operation::GetXTimes(GetXTimes {}),
                #[cfg(target_os = "macos")]
                fuse_opcode::FUSE_EXCHANGE => Operation::Exchange(Exchange {}),
                fuse_opcode::CUSE_INIT => {
                    let (arg, _rest) = fuse_init_in::ref_from_prefix(rest).unwrap();
                    Operation::CuseInit(CuseInit { arg: *arg })
                }
            },
        };

        let request = Request {
            header,
            operation,
            reply_to: reply_to.clone(),
        };

        Ok(request)
    }

    /// Send an error reply based on the [Error] value
    ///
    /// [Error] is automatically matches to appropriate [Errno]
    pub async fn send_error(&self, error: Errno) -> Result<(), Errno> {
        let reply = super::reply::Reply {
            header: fuse_out_header {
                error: Errno::from(error).into(),
                len: 0,
                unique: self.header.unique,
            },
            operation: None,
        };

        if let Err(_e) = self.reply_to.send(reply).await {
            error!("channel send");
            return Err(Errno::EIO);
        }

        Ok(())
    }

    pub async fn send(&self, reply: super::reply::Reply) -> Result<(), Errno> {
        if let Err(_e) = self.reply_to.send(reply).await {
            error!("channel send");
            return Err(Errno::EIO);
        }

        Ok(())
    }

    pub async fn send_ok(&self) -> Result<(), Errno> {
        let reply = crate::messages::reply::Reply::from(self);

        if let Err(_e) = self.reply_to.send(reply).await {
            error!("channel send");
            return Err(Errno::EIO);
        }

        Ok(())
    }
}

/// Lookup a directory to get its attributes
pub struct Lookup {
    pub name: String,
}

/// Forget about an inode.
///
/// [Forget::lookup_count] indicates the number of lookups previously performed on this inode.
/// If the [Filesystem] implements inode lifetimes, it is recommended that inodes increment a
/// single reference on each lookup and decrement references on each forget. The filesystem
/// may ignore forget calls if the inodes do not need a limited lifetime.
///
/// On unmount, it is not guaranteed that all referenced inodes will receive a forget message.
pub struct Forget {
    pub arg: fuse_forget_in,
}

/// Get attributes for an inode
pub struct GetAttr {
    #[cfg(feature = "abi-7-9")]
    pub arg: fuse_getattr_in,
}

/// Set attributes for an inode
pub struct SetAttr {
    pub arg: fuse_setattr_in,
}

/// Read a symbolic link
pub struct ReadLink {}

/// Create a symbolic link
#[derive(Debug)]
pub struct SymLink {
    pub name: String,
    pub target: String,
}

/// Create a regular file, block device, fifo, or socket node
///
/// See [man](https://man7.org/linux/man-pages/man2/mknod.2.html)
pub struct MkNod {
    pub arg: fuse_mknod_in,
    pub name: String,
}

/// Make a directory
pub struct MkDir {
    pub arg: fuse_mkdir_in,
    pub name: String,
}

/// Remove a file or directory
pub struct Unlink {
    pub name: String,
}

/// Remove a directory
pub struct RmDir {
    pub name: String,
}

/// Rename a file or directory
pub struct Rename {
    pub arg: fuse_rename_in,
    pub name: String,
    pub newname: String,
}

/// Create a hard link
pub struct Link {
    pub arg: fuse_link_in,
    pub name: String,
}

/// Open a file
///
/// See [open man page](https://man7.org/linux/man-pages/man2/open.2.html)
pub struct Open {
    pub arg: fuse_open_in,
}

/// Read bytes from a file
pub struct Read {
    pub arg: fuse_read_in,
}

/// Write bytes to a file
pub struct Write {
    pub arg: fuse_write_in,
    // TODO make this [Option] so we can take it without copying it
    pub data: Arc<Mutex<Vec<u8>>>,
}

/// Get filesystem statistics
pub struct StatFs {}

/// Release an open file
///
/// See [close man page](https://man7.org/linux/man-pages/man2/close.2.html)
pub struct Release {
    pub arg: fuse_release_in,
}

/// Synchronize file contents
///
/// See [fsync man page](https://man7.org/linux/man-pages/man2/fsync.2.html)
pub struct FSync {
    pub arg: fuse_fsync_in,
}

/// Set an extended attribute
pub struct SetXAttr {
    pub arg: fuse_setxattr_in,
    pub name: String,
    pub value: Arc<Vec<u8>>,
}

pub struct GetXAttr {
    pub arg: fuse_getxattr_in,
    pub name: String,
}

/// List extended attribute names
pub struct ListXAttr {
    pub arg: fuse_getxattr_in,
}

/// Remove an extended attribute
pub struct RemoveXAttr {
    pub name: String,
}

/// Flush a file to disk
///
/// See [fuse](https://libfuse.github.io/doxygen/structfuse__operations.html#a6bfecd61ddd58f74820953ee23b19ef3)
pub struct Flush {
    pub arg: fuse_flush_in,
}

/// Initialize a filesystem
pub struct Init {
    pub arg: fuse_init_in,
}

/// Open a directory
pub struct OpenDir {
    pub arg: fuse_open_in,
}

/// Read a directory
pub struct ReadDir {
    pub arg: fuse_read_in,
}

/// Release an open directory
///
/// For every [OpenDir] call there will be one [ReleaseDir] call
pub struct ReleaseDir {
    pub arg: fuse_release_in,
}

/// Synchronize directory contents
pub struct FSyncDir {
    pub arg: fuse_fsync_in,
}

/// Test for a POSIX lock
pub struct GetLk {
    pub arg: fuse_lk_in,
}

/// Acquire, modify or release a POSIX lock
pub struct SetLk {
    pub arg: fuse_lk_in,
}

pub struct SetLkW {
    pub arg: fuse_lk_in,
}

/// Check file access permissions
pub struct Access {
    pub arg: fuse_access_in,
}

/// Create and open a file
#[derive(Debug)]
pub struct Create {
    pub arg: fuse_create_in,
    pub name: String,
}

/// If a process issuing a FUSE filesystem request is interrupted, the
/// following will happen:
///
///   1) If the request is not yet sent to userspace AND the signal is fatal (SIGKILL or unhandled fatal signal), then
///      the request is dequeued and returns immediately.
///
///   2) If the request is not yet sent to userspace AND the signal is not fatal, then an 'interrupted' flag is set for
///      the request.  When the request has been successfully transferred to userspace and this flag is set, an
///      INTERRUPT request is queued.
///
///   3) If the request is already sent to userspace, then an INTERRUPT request is queued.
///
/// [Interrupt] requests take precedence over other requests, so the
/// userspace filesystem will receive queued [Interrupt]s before any others.
///
/// The userspace filesystem may ignore the [Interrupt] requests entirely,
/// or may honor them by sending a reply to the **original** request, with
/// the error set to [Errno::EINTR].
///
/// It is also possible that there's a race between processing the
/// original request and its [Interrupt] request.  There are two
/// possibilities:
///
/// 1. The [Interrupt] request is processed before the original request is processed
///
/// 2. The [Interrupt] request is processed after the original request has been answered
///
/// If the filesystem cannot find the original request, it should wait for
/// some timeout and/or a number of new requests to arrive, after which it
/// should reply to the [Interrupt] request with an [Errno::EAGAIN] error.
/// In case (1) the [Interrupt] request will be requeued.  In case (2) the
/// [Interrupt] reply will be ignored.
pub struct Interrupt {
    pub arg: fuse_interrupt_in,
}

/// Map the block index within file to block index within the device.
///
/// **Note**: this makes sense only for block device backed filesystems
/// mounted with the `blkdev` option.
pub struct BMap {
    pub arg: fuse_bmap_in,
}

/// Delete the inode
pub struct Destroy {}

/// Control the device
#[cfg(feature = "abi-7-11")]
pub struct IoCtl {
    pub arg: fuse_ioctl_in,
    pub data: Vec<u8>,
}

/// Pole the filesystem for change to the file
#[cfg(feature = "abi-7-11")]
pub struct Poll {
    pub arg: fuse_poll_in,
}

/// Batch forget
#[cfg(feature = "abi-7-16")]
pub struct BatchForget {
    pub arg: fuse_batch_forget_in,
    pub nodes: Vec<fuse_forget_one>,
}

/// Pre-allocate or deallocate space to a file
///
/// See [fallocate man](https://man7.org/linux/man-pages/man2/fallocate.2.html)
#[cfg(feature = "abi-7-19")]
pub struct FAllocate {
    pub arg: fuse_fallocate_in,
}

/// Read directory
#[cfg(feature = "abi-7-21")]
pub struct ReadDirPlus {
    pub arg: fuse_read_in,
}

/// Rename a file
#[cfg(feature = "abi-7-23")]
pub struct Rename2 {
    pub arg: fuse_rename2_in,
    pub name: String,
    pub newname: String,
    pub old_parent: u64,
}

/// Reposition read/write file offset
///
/// See [lseek man](https://man7.org/linux/man-pages/man2/lseek.2.html)
#[cfg(feature = "abi-7-24")]
pub struct LSeek {
    pub arg: fuse_lseek_in,
}

/// Copy the specified range from the source inode to the destination inode
#[cfg(feature = "abi-7-28")]
#[repr(transparent)]
pub struct CopyFileRange {
    pub arg: fuse_copy_file_range_in,
}

#[cfg(feature = "abi-7-31")]
#[repr(transparent)]
#[derive(Default)]
pub struct SetupMapping {
    pub arg: fuse_setupmapping_in,
}

#[cfg(feature = "abi-7-31")]
#[derive(Default)]
pub struct RemoveMapping {
    pub arg: fuse_removemapping_in,
    pub mappings: Vec<fuse_removemapping_one>,
}

#[cfg(feature = "abi-7-34")]
#[derive(Default)]
pub struct SyncFs {
    #[allow(unused)]
    arg: fuse_syncfs_in,
}

#[cfg(feature = "abi-7-37")]
#[derive(Default)]
#[repr(transparent)]
pub struct TmpFile {
    arg: fuse_create_in,
}

/// MacOS only: Rename the volume. Set `fuse_init_out.flags` during init to
/// `FUSE_VOL_RENAME` to enable
#[cfg(target_os = "macos")]
pub struct SetVolName {
    name: String,
}

/// macOS only: Query extended times (bkuptime and crtime). Set fuse_init_out.flags
/// during init to FUSE_XTIMES to enable
#[cfg(target_os = "macos")]
pub struct GetXTimes {}

// API TODO: Consider rename2(RENAME_EXCHANGE)
/// macOS only (undocumented)
#[cfg(target_os = "macos")]
pub struct Exchange {
    pub arg: fuse_exchange_in,
    oldname: String,
    newname: String,
}

#[repr(transparent)]
#[derive(Default)]
pub struct StatX {
    pub arg: fuse_statx_in,
}

// TODO document
#[cfg(feature = "abi-7-12")]
#[derive(Debug)]
pub struct CuseInit {
    pub arg: fuse_init_in,
}

#[repr(u32)]
pub enum Operation {
    Lookup(Lookup) = 1,
    Forget(Forget) = 2,
    GetAttr(GetAttr) = 3,
    SetAttr(SetAttr) = 4,
    ReadLink(ReadLink) = 5,
    SymLink(SymLink) = 6,
    /// Create special or ordinary file
    ///
    /// [man](https://man7.org/linux/man-pages/man2/mknod.2.html)
    MkNod(MkNod)  = 8,
    MkDir(MkDir)  = 9,
    Unlink(Unlink) = 10,
    RmDir(RmDir)  = 11,
    Rename(Rename) = 12,
    Link(Link)    = 13,
    Open(Open)    = 14,
    Read(Read)    = 15,
    Write(Write)  = 16,
    StatFs(StatFs) = 17,
    Release(Release) = 18,
    FSync(FSync)  = 20,
    SetXAttr(SetXAttr) = 21,
    GetXAttr(GetXAttr) = 22,
    ListXAttr(ListXAttr) = 23,
    RemoveXAttr(RemoveXAttr) = 24,
    Flush(Flush)  = 25,
    Init(Init)    = 26,
    OpenDir(OpenDir) = 27,
    ReadDir(ReadDir) = 28,
    ReleaseDir(ReleaseDir) = 29,
    FSyncDir(FSyncDir) = 30,
    GetLk(GetLk)  = 31,
    SetLk(SetLk)  = 32,
    SetLkW(SetLkW) = 33,
    Access(Access) = 34,
    Create(Create) = 35,
    Interrupt(Interrupt) = 36,
    BMap(BMap)    = 37,
    Destroy(Destroy) = 38,
    #[cfg(feature = "abi-7-11")]
    IoCtl(IoCtl)  = 39,
    #[cfg(feature = "abi-7-11")]
    Poll(Poll)    = 40,
    #[cfg(feature = "abi-7-15")]
    #[allow(dead_code)]
    NotifyReply(NotifyReply) = 41,
    #[cfg(feature = "abi-7-16")]
    BatchForget(BatchForget) = 42,
    #[cfg(feature = "abi-7-19")]
    FAllocate(FAllocate) = 43,
    #[cfg(feature = "abi-7-21")]
    ReadDirPlus(ReadDirPlus) = 44,
    #[cfg(feature = "abi-7-23")]
    Rename2(Rename2) = 45,
    #[cfg(feature = "abi-7-24")]
    LSeek(LSeek)  = 46,
    #[cfg(feature = "abi-7-28")]
    CopyFileRange(CopyFileRange) = 47,
    #[cfg(feature = "abi-7-31")]
    SetupMapping(SetupMapping) = 48,
    #[cfg(feature = "abi-7-31")]
    RemoveMapping(RemoveMapping) = 49,
    #[cfg(feature = "abi-7-34")]
    SyncFs(SyncFs) = 50,
    #[cfg(feature = "abi-7-37")]
    TmpFile(TmpFile) = 51,
    #[cfg(feature = "abi-7-39")]
    StatX(StatX)  = 52,
    #[cfg(target_os = "macos")]
    SetVolName(SetVolName) = 61,
    #[cfg(target_os = "macos")]
    GetXTimes(GetXTimes) = 62,
    #[cfg(target_os = "macos")]
    Exchange(Exchange) = 63,

    #[cfg(feature = "abi-7-12")]
    #[allow(dead_code)]
    CuseInit(CuseInit) = 4096,
}

impl Operation {
    pub fn get_opcode(&self) -> u32 {
        unsafe { *<*const _>::from(self).cast::<u32>() }
    }
}

#[allow(const_item_mutation)]
#[cfg(test)]
mod tests {

    use crate::messages::request::Operation;

    use super::Request;

    #[cfg(target_endian = "big")]
    const INIT_REQUEST: [u8; 56] = [
        0x00, 0x00, 0x00, 0x38, 0x00, 0x00, 0x00, 0x1a, // len, opcode
        0xde, 0xad, 0xbe, 0xef, 0xba, 0xad, 0xd0, 0x0d, // unique
        0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, // nodeid
        0xc0, 0x01, 0xd0, 0x0d, 0xc0, 0x01, 0xca, 0xfe, // uid, gid
        0xc0, 0xde, 0xba, 0x5e, 0x00, 0x00, 0x00, 0x00, // pid, padding
        0x00, 0x00, 0x00, 0x07, 0x00, 0x00, 0x00, 0x08, // major, minor
        0x00, 0x00, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, // max_readahead, flags
    ];

    #[cfg(target_endian = "little")]
    const INIT_REQUEST: [u8; 56] = [
        0x38, 0x00, 0x00, 0x00, 0x1a, 0x00, 0x00, 0x00, // len, opcode
        0x0d, 0xf0, 0xad, 0xba, 0xef, 0xbe, 0xad, 0xde, // unique
        0x88, 0x77, 0x66, 0x55, 0x44, 0x33, 0x22, 0x11, // nodeid
        0x0d, 0xd0, 0x01, 0xc0, 0xfe, 0xca, 0x01, 0xc0, // uid, gid
        0x5e, 0xba, 0xde, 0xc0, 0x00, 0x00, 0x00, 0x00, // pid, padding
        0x07, 0x00, 0x00, 0x00, 0x08, 0x00, 0x00, 0x00, // major, minor
        0x00, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // max_readahead, flags
    ];

    #[cfg(target_endian = "big")]
    const MKNOD_REQUEST: [u8; 56] = [
        0x00, 0x00, 0x00, 0x38, 0x00, 0x00, 0x00, 0x08, // len, opcode
        0xde, 0xad, 0xbe, 0xef, 0xba, 0xad, 0xd0, 0x0d, // unique
        0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, // nodeid
        0xc0, 0x01, 0xd0, 0x0d, 0xc0, 0x01, 0xca, 0xfe, // uid, gid
        0xc0, 0xde, 0xba, 0x5e, 0x00, 0x00, 0x00, 0x00, // pid, padding
        0x00, 0x00, 0x01, 0xa4, 0x00, 0x00, 0x00, 0x00, // mode, rdev
        0x66, 0x6f, 0x6f, 0x2e, 0x74, 0x78, 0x74, 0x00, // name
    ];

    #[cfg(all(target_endian = "little", not(feature = "abi-7-12")))]
    const MKNOD_REQUEST: [u8; 56] = [
        0x38, 0x00, 0x00, 0x00, 0x08, 0x00, 0x00, 0x00, // len, opcode
        0x0d, 0xf0, 0xad, 0xba, 0xef, 0xbe, 0xad, 0xde, // unique
        0x88, 0x77, 0x66, 0x55, 0x44, 0x33, 0x22, 0x11, // nodeid
        0x0d, 0xd0, 0x01, 0xc0, 0xfe, 0xca, 0x01, 0xc0, // uid, gid
        0x5e, 0xba, 0xde, 0xc0, 0x00, 0x00, 0x00, 0x00, // pid, padding
        0xa4, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // mode, rdev
        0x66, 0x6f, 0x6f, 0x2e, 0x74, 0x78, 0x74, 0x00, // name
    ];

    #[cfg(all(target_endian = "little", feature = "abi-7-12"))]
    const MKNOD_REQUEST: [u8; 64] = [
        0x40, 0x00, 0x00, 0x00, 0x08, 0x00, 0x00, 0x00, // len, opcode
        0x0d, 0xf0, 0xad, 0xba, 0xef, 0xbe, 0xad, 0xde, // unique
        0x88, 0x77, 0x66, 0x55, 0x44, 0x33, 0x22, 0x11, // nodeid
        0x0d, 0xd0, 0x01, 0xc0, 0xfe, 0xca, 0x01, 0xc0, // uid, gid
        0x5e, 0xba, 0xde, 0xc0, 0x00, 0x00, 0x00, 0x00, // pid, padding
        0xa4, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // mode, rdev
        0xed, 0x01, 0x00, 0x00, 0xe7, 0x03, 0x00, 0x00, // umask, padding
        0x66, 0x6f, 0x6f, 0x2e, 0x74, 0x78, 0x74, 0x00, // name
    ];

    #[test]
    fn init() {
        let (reply_tx, _reply_rx) = crate::create_reply_channel();
        let request = Request::parse(&mut INIT_REQUEST, &reply_tx).expect("parse");
        assert_eq!(request.header.len, 56);
        assert_eq!(request.header.len, 56);
        assert_eq!(request.header.opcode, 26);
        assert_eq!(request.header.unique, 0xdead_beef_baad_f00d);
        assert_eq!(request.header.nodeid, 0x1122_3344_5566_7788);
        assert_eq!(request.header.uid, 0xc001_d00d);
        assert_eq!(request.header.gid, 0xc001_cafe);
        assert_eq!(request.header.pid, 0xc0de_ba5e);
        match request.operation {
            Operation::Init(init) => {
                assert_eq!(init.arg.major, 7);
                assert_eq!(init.arg.minor, 8);
                assert_eq!(init.arg.max_readahead, 4096);
            }
            _ => panic!("not init"),
        }
    }

    #[test]
    fn mknod() {
        let (reply_tx, _reply_rx) = crate::create_reply_channel();
        let request = Request::parse(&mut MKNOD_REQUEST, &reply_tx).expect("parse");
        #[cfg(not(feature = "abi-7-12"))]
        assert_eq!(req.header.len, 56);
        #[cfg(feature = "abi-7-12")]
        assert_eq!(request.header.len, 64);
        assert_eq!(request.header.opcode, 8);
        assert_eq!(request.header.unique, 0xdead_beef_baad_f00d);
        assert_eq!(request.header.nodeid, 0x1122_3344_5566_7788);
        assert_eq!(request.header.uid, 0xc001_d00d);
        assert_eq!(request.header.gid, 0xc001_cafe);
        assert_eq!(request.header.pid, 0xc0de_ba5e);
        match request.operation {
            Operation::MkNod(x) => {
                assert_eq!(x.arg.mode, 0o644);
                #[cfg(feature = "abi-7-12")]
                assert_eq!(x.arg.umask, 0o755);
            }
            _ => panic!("Unexpected request operation"),
        }
    }
}

/// ABI version
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version(pub u32, pub u32);
impl Version {
    pub fn major(&self) -> u32 {
        self.0
    }
    pub fn minor(&self) -> u32 {
        self.1
    }
}

impl Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.0, self.1)
    }
}
