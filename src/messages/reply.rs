//! Reply message, interior operations, and serializers.
//!
//! # TODO
//! * Create a derive macro to implement the write_too function

use std::{ffi::OsString, os::unix::ffi::OsStrExt};

use zerocopy::{Immutable, IntoBytes, KnownLayout, TryFromBytes};

use crate::error::Errno;
#[allow(unused)]
use crate::messages::fuse_abi::*;
#[allow(unused)]
use crate::messages::request::Request;

/// For objects that can write themselves as a byte array to an [io::Write]r
pub trait IWrite {
    fn write(&mut self, buffer: &mut [u8]) -> usize;
}

/// Reply to [Filesystem] [Request]s
pub struct Reply {
    /// Common header for all operations
    pub header: fuse_out_header,
    /// Operations by type
    pub operation: Option<Operation>,
}

impl Reply {
    pub fn new(unique: u64, error: i32, operation: Option<Operation>) -> Self {
        let header: fuse_out_header = fuse_out_header { error, len: 0, unique };

        Self { header, operation }
    }

    pub fn set_error(&mut self, error: Errno) {
        self.header.error = error.into()
    }
}

impl From<&Request> for Reply {
    fn from(request: &Request) -> Self {
        Self {
            header: fuse_out_header {
                error: 0,
                len: 0,
                unique: request.header.unique,
            },
            operation: None,
        }
    }
}

impl IWrite for Reply {
    fn write(&mut self, buffer: &mut [u8]) -> usize {
        let mut count = 0usize;

        let len = self.header.as_bytes().len();

        buffer[..len].copy_from_slice(self.header.as_bytes());

        count += len;

        if let Some(ref mut op) = self.operation {
            count += op.write(&mut buffer[count..]);
        }

        // Update the header length
        let (header, _rest) = fuse_out_header::try_mut_from_prefix(buffer).unwrap();
        header.len = count as u32;

        count
    }
}

/// [fuse_lowlevel.c](https://github.com/libfuse/libfuse/blob/6cdb65047f60057724d0939836c261bb40433e53/lib/fuse_lowlevel.c#L301)
#[derive(Debug)]
pub struct DirectoryEntry {
    pub entry: fuse_dirent,
    /// Serialized as an array of bytes
    pub name: String,
}

impl IWrite for DirectoryEntry {
    fn write(&mut self, buffer: &mut [u8]) -> usize {
        self.entry.namelen = self.name.as_bytes().len() as u32;

        let mut count = 0;
        buffer[0..self.entry.as_bytes().len()].copy_from_slice(self.entry.as_bytes());
        count += self.entry.as_bytes().len();
        buffer[count..count + self.name.len()].copy_from_slice(self.name.as_bytes());
        count += self.name.as_bytes().len();

        // Align the output to 8 byte boundary
        let r = count % 8;
        if r > 0 {
            let diff = 8 - r;
            buffer[count..count + diff].fill(0);
            count += diff;
        }

        count
    }
}

pub struct DirectoryEntryPlus {
    entry: fuse_direntplus,
    name: OsString,
}

impl IWrite for DirectoryEntryPlus {
    fn write(&mut self, buffer: &mut [u8]) -> usize {
        self.entry.dirent.namelen = self.name.as_bytes().len() as u32;

        let mut count = 0;
        buffer[0..self.entry.as_bytes().len()].copy_from_slice(self.entry.as_bytes());
        count += self.entry.as_bytes().len();
        buffer[count..count + self.name.len()].copy_from_slice(self.name.as_bytes());
        count += self.name.as_bytes().len();

        // Align the output to 8 byte boundary
        let r = count % 8;
        if r > 0 {
            let diff = 8 - r;
            buffer[count..count + diff].fill(0);
            count += diff;
        }

        count
    }
}

#[derive(IntoBytes, KnownLayout, Immutable, Debug)]
#[repr(transparent)]
pub struct Lookup {
    pub arg: fuse_entry_out,
}

impl IWrite for Lookup {
    fn write(&mut self, buffer: &mut [u8]) -> usize {
        let count = self.as_bytes().len();
        buffer[0..count].copy_from_slice(self.as_bytes());
        count
    }
}

#[derive(IntoBytes, Immutable, KnownLayout)]
#[repr(transparent)]
pub struct Forget {}

impl IWrite for Forget {
    fn write(&mut self, buffer: &mut [u8]) -> usize {
        let count = self.as_bytes().len();
        buffer[0..count].copy_from_slice(self.as_bytes());
        count
    }
}

#[derive(IntoBytes, Immutable, KnownLayout, Debug)]
#[repr(transparent)]
pub struct GetAttr {
    pub arg: fuse_attr_out,
}

impl IWrite for GetAttr {
    fn write(&mut self, buffer: &mut [u8]) -> usize {
        let count = self.as_bytes().len();
        buffer[0..count].copy_from_slice(self.as_bytes());
        count
    }
}

#[derive(IntoBytes, Immutable, KnownLayout)]
#[repr(transparent)]
pub struct SetAttr {
    pub arg: fuse_attr_out,
}

impl IWrite for SetAttr {
    fn write(&mut self, buffer: &mut [u8]) -> usize {
        let count = self.as_bytes().len();
        buffer[0..count].copy_from_slice(self.as_bytes());
        count
    }
}

pub struct ReadLink {
    pub data: String,
}

impl IWrite for ReadLink {
    fn write(&mut self, buffer: &mut [u8]) -> usize {
        let count = self.data.as_bytes().len();
        buffer[0..count].copy_from_slice(self.data.as_bytes());
        count
    }
}

#[derive(IntoBytes, Immutable, KnownLayout)]
#[repr(transparent)]
pub struct SymLink {
    pub arg: fuse_entry_out,
}

impl IWrite for SymLink {
    fn write(&mut self, buffer: &mut [u8]) -> usize {
        let count = self.as_bytes().len();
        buffer[0..count].copy_from_slice(self.as_bytes());
        count
    }
}

#[derive(IntoBytes, Immutable, KnownLayout)]
#[repr(transparent)]
pub struct MkNod {}

impl IWrite for MkNod {
    fn write(&mut self, buffer: &mut [u8]) -> usize {
        let count = self.as_bytes().len();
        buffer[0..count].copy_from_slice(self.as_bytes());
        count
    }
}

#[derive(IntoBytes, Immutable, KnownLayout)]
#[repr(transparent)]
pub struct MkDir {
    pub arg: fuse_entry_out,
}

impl IWrite for MkDir {
    fn write(&mut self, buffer: &mut [u8]) -> usize {
        let count = self.as_bytes().len();
        buffer[0..count].copy_from_slice(self.as_bytes());
        count
    }
}

#[derive(IntoBytes, Immutable, KnownLayout)]
#[repr(transparent)]
pub struct Unlink {}

impl IWrite for Unlink {
    fn write(&mut self, buffer: &mut [u8]) -> usize {
        let count = self.as_bytes().len();
        buffer[0..count].copy_from_slice(self.as_bytes());
        count
    }
}

#[derive(IntoBytes, Immutable, KnownLayout)]
#[repr(transparent)]
pub struct RmDir {}

impl IWrite for RmDir {
    fn write(&mut self, buffer: &mut [u8]) -> usize {
        let count = self.as_bytes().len();
        buffer[0..count].copy_from_slice(self.as_bytes());
        count
    }
}

#[derive(IntoBytes, KnownLayout, Immutable)]
#[repr(transparent)]
pub struct Rename {}

impl IWrite for Rename {
    fn write(&mut self, buffer: &mut [u8]) -> usize {
        let count = self.as_bytes().len();
        buffer[0..count].copy_from_slice(self.as_bytes());
        count
    }
}

#[derive(IntoBytes, KnownLayout, Immutable)]
#[repr(transparent)]
pub struct Link {
    pub arg: fuse_entry_out,
}

impl IWrite for Link {
    fn write(&mut self, buffer: &mut [u8]) -> usize {
        let count = self.as_bytes().len();
        buffer[0..count].copy_from_slice(self.as_bytes());
        count
    }
}

#[derive(IntoBytes, KnownLayout, Immutable)]
#[repr(transparent)]
pub struct Open {
    pub arg: fuse_open_out,
}

impl IWrite for Open {
    fn write(&mut self, buffer: &mut [u8]) -> usize {
        let count = self.as_bytes().len();
        buffer[0..count].copy_from_slice(self.as_bytes());
        count
    }
}

pub struct Read {
    pub data: Vec<u8>,
}

impl IWrite for Read {
    fn write(&mut self, buffer: &mut [u8]) -> usize {
        let count = self.data.as_bytes().len();
        buffer[0..count].copy_from_slice(self.data.as_bytes());
        count
    }
}

#[derive(IntoBytes, KnownLayout, Immutable)]
#[repr(transparent)]
pub struct Write {
    pub arg: fuse_write_out,
}

impl IWrite for Write {
    fn write(&mut self, buffer: &mut [u8]) -> usize {
        let count = self.as_bytes().len();
        buffer[0..count].copy_from_slice(self.as_bytes());
        count
    }
}

/// See [statfs](https://man7.org/linux/man-pages/man2/statfs.2.html)
#[derive(Debug, IntoBytes, KnownLayout, Immutable)]
#[repr(transparent)]
pub struct StatFs {
    pub arg: fuse_statfs_out,
}

impl IWrite for StatFs {
    fn write(&mut self, buffer: &mut [u8]) -> usize {
        let count = self.as_bytes().len();
        buffer[0..count].copy_from_slice(self.as_bytes());
        count
    }
}

#[derive(IntoBytes, KnownLayout, Immutable)]
#[repr(transparent)]
pub struct Release {}

impl IWrite for Release {
    fn write(&mut self, buffer: &mut [u8]) -> usize {
        let count = self.as_bytes().len();
        buffer[0..count].copy_from_slice(self.as_bytes());
        count
    }
}

#[derive(IntoBytes, KnownLayout, Immutable)]
#[repr(transparent)]
pub struct FSync {}

impl IWrite for FSync {
    fn write(&mut self, buffer: &mut [u8]) -> usize {
        let count = self.as_bytes().len();
        buffer[0..count].copy_from_slice(self.as_bytes());
        count
    }
}

#[derive(IntoBytes, KnownLayout, Immutable)]
#[repr(transparent)]
pub struct SetXAttr {}

impl IWrite for SetXAttr {
    fn write(&mut self, buffer: &mut [u8]) -> usize {
        let count = self.as_bytes().len();
        buffer[0..count].copy_from_slice(self.as_bytes());
        count
    }
}

pub struct GetXAttr {
    arg: fuse_getxattr_out,
    data: Vec<u8>,
}

impl IWrite for GetXAttr {
    fn write(&mut self, buffer: &mut [u8]) -> usize {
        let mut count = 0usize;
        buffer[..self.arg.as_bytes().len()].copy_from_slice(self.arg.as_bytes());
        count += self.arg.as_bytes().len();

        buffer[count..count + self.data.as_bytes().len()].copy_from_slice(self.data.as_bytes());
        count += self.data.len();

        count
    }
}

pub struct ListXAttr {
    data: Vec<u8>,
}

impl IWrite for ListXAttr {
    fn write(&mut self, buffer: &mut [u8]) -> usize {
        let count = self.data.as_bytes().len();
        buffer[0..count].copy_from_slice(self.data.as_bytes());
        count
    }
}

#[derive(IntoBytes, KnownLayout, Immutable)]
#[repr(transparent)]
pub struct RemoveXAttr {}

impl IWrite for RemoveXAttr {
    fn write(&mut self, buffer: &mut [u8]) -> usize {
        let count = self.as_bytes().len();
        buffer[0..count].copy_from_slice(self.as_bytes());
        count
    }
}

#[derive(IntoBytes, KnownLayout, Immutable)]
#[repr(transparent)]
pub struct Flush {}

impl IWrite for Flush {
    fn write(&mut self, buffer: &mut [u8]) -> usize {
        let count = self.as_bytes().len();
        buffer[0..count].copy_from_slice(self.as_bytes());
        count
    }
}

#[derive(IntoBytes, KnownLayout, Immutable)]
#[repr(transparent)]
pub struct Init {
    pub arg: fuse_init_out,
}

impl IWrite for Init {
    fn write(&mut self, buffer: &mut [u8]) -> usize {
        let count = self.as_bytes().len();
        buffer[0..count].copy_from_slice(self.as_bytes());
        count
    }
}

#[derive(IntoBytes, KnownLayout, Immutable)]
#[repr(transparent)]
pub struct OpenDir {
    pub arg: fuse_open_out,
}

impl IWrite for OpenDir {
    fn write(&mut self, buffer: &mut [u8]) -> usize {
        let count = self.as_bytes().len();
        buffer[0..count].copy_from_slice(self.as_bytes());
        count
    }
}

pub struct ReadDir {
    /// list of directory entry names
    ///
    /// Converted during write
    pub entries: Vec<DirectoryEntry>,
}

impl IWrite for ReadDir {
    fn write(&mut self, buffer: &mut [u8]) -> usize {
        let mut count = 0usize;

        for entry in self.entries.as_mut_slice() {
            count += entry.write(&mut buffer[count..]);
        }

        count
    }
}

#[derive(IntoBytes, KnownLayout, Immutable)]
#[repr(transparent)]
pub struct ReleaseDir {}

impl IWrite for ReleaseDir {
    fn write(&mut self, buffer: &mut [u8]) -> usize {
        let count = self.as_bytes().len();
        buffer[0..count].copy_from_slice(self.as_bytes());
        count
    }
}

#[derive(IntoBytes, KnownLayout, Immutable)]
#[repr(transparent)]
pub struct FSyncDir {}

impl IWrite for FSyncDir {
    fn write(&mut self, buffer: &mut [u8]) -> usize {
        let count = self.as_bytes().len();
        buffer[0..count].copy_from_slice(self.as_bytes());
        count
    }
}

#[derive(IntoBytes, KnownLayout, Immutable)]
#[repr(transparent)]
pub struct GetLk {
    arg: fuse_lk_out,
}

impl IWrite for GetLk {
    fn write(&mut self, buffer: &mut [u8]) -> usize {
        let count = self.as_bytes().len();
        buffer[0..count].copy_from_slice(self.as_bytes());
        count
    }
}

#[derive(IntoBytes, KnownLayout, Immutable)]
#[repr(transparent)]
pub struct SetLk {
    arg: fuse_lk_out,
}

impl IWrite for SetLk {
    fn write(&mut self, buffer: &mut [u8]) -> usize {
        let count = self.as_bytes().len();
        buffer[0..count].copy_from_slice(self.as_bytes());
        count
    }
}

#[derive(IntoBytes, KnownLayout, Immutable)]
#[repr(transparent)]
pub struct Access {}

impl IWrite for Access {
    fn write(&mut self, buffer: &mut [u8]) -> usize {
        let count = self.as_bytes().len();
        buffer[0..count].copy_from_slice(self.as_bytes());
        count
    }
}

#[derive(IntoBytes, KnownLayout, Immutable)]
#[repr(transparent)]
pub struct Create {
    pub arg: fuse_create_out,
}

impl IWrite for Create {
    fn write(&mut self, buffer: &mut [u8]) -> usize {
        let count = self.as_bytes().len();
        buffer[0..count].copy_from_slice(self.as_bytes());
        count
    }
}

#[derive(IntoBytes, KnownLayout, Immutable)]
#[repr(transparent)]
pub struct Interrupt {}

impl IWrite for Interrupt {
    fn write(&mut self, buffer: &mut [u8]) -> usize {
        let count = self.as_bytes().len();
        buffer[0..count].copy_from_slice(self.as_bytes());
        count
    }
}

#[derive(IntoBytes, KnownLayout, Immutable)]
#[repr(transparent)]
pub struct BMap {
    arg: fuse_bmap_out,
}

impl IWrite for BMap {
    fn write(&mut self, buffer: &mut [u8]) -> usize {
        let count = self.as_bytes().len();
        buffer[0..count].copy_from_slice(self.as_bytes());
        count
    }
}

#[derive(IntoBytes, KnownLayout, Immutable)]
#[repr(transparent)]
pub struct Destroy {}

impl IWrite for Destroy {
    fn write(&mut self, buffer: &mut [u8]) -> usize {
        let count = self.as_bytes().len();
        buffer[0..count].copy_from_slice(self.as_bytes());
        count
    }
}

#[derive(IntoBytes, KnownLayout, Immutable)]
#[repr(transparent)]
pub struct IoCtl {
    arg: fuse_ioctl_out,
}

impl IWrite for IoCtl {
    fn write(&mut self, buffer: &mut [u8]) -> usize {
        let count = self.as_bytes().len();
        buffer[0..count].copy_from_slice(self.as_bytes());
        count
    }
}

#[derive(IntoBytes, KnownLayout, Immutable)]
#[repr(transparent)]
pub struct Poll {
    #[cfg(feature = "abi-7-11")]
    arg: fuse_poll_out,
}

impl IWrite for Poll {
    fn write(&mut self, buffer: &mut [u8]) -> usize {
        let count = self.as_bytes().len();
        buffer[0..count].copy_from_slice(self.as_bytes());
        count
    }
}

#[derive(IntoBytes, KnownLayout, Immutable)]
#[repr(transparent)]
pub struct NotifyReply {}

impl IWrite for NotifyReply {
    fn write(&mut self, buffer: &mut [u8]) -> usize {
        let count = self.as_bytes().len();
        buffer[0..count].copy_from_slice(self.as_bytes());
        count
    }
}

#[derive(IntoBytes, KnownLayout, Immutable)]
#[repr(transparent)]
pub struct BatchForget {}

impl IWrite for BatchForget {
    fn write(&mut self, buffer: &mut [u8]) -> usize {
        let count = self.as_bytes().len();
        buffer[0..count].copy_from_slice(self.as_bytes());
        count
    }
}

#[derive(IntoBytes, KnownLayout, Immutable)]
#[repr(transparent)]
pub struct FAllocate {}

impl IWrite for FAllocate {
    fn write(&mut self, buffer: &mut [u8]) -> usize {
        let count = self.as_bytes().len();
        buffer[0..count].copy_from_slice(self.as_bytes());
        count
    }
}

pub struct ReadDirPlus {
    entries: Vec<DirectoryEntryPlus>,
}

impl IWrite for ReadDirPlus {
    fn write(&mut self, buffer: &mut [u8]) -> usize {
        let mut count = 0usize;

        for e in self.entries.as_mut_slice() {
            count += e.write(&mut buffer[count..]);
        }
        count
    }
}

#[derive(IntoBytes, KnownLayout, Immutable)]
#[repr(transparent)]
pub struct Rename2 {}

impl IWrite for Rename2 {
    fn write(&mut self, buffer: &mut [u8]) -> usize {
        let count = self.as_bytes().len();
        buffer[0..count].copy_from_slice(self.as_bytes());
        count
    }
}

#[derive(IntoBytes, KnownLayout, Immutable)]
#[repr(transparent)]
pub struct Lseek {
    pub arg: fuse_lseek_out,
}

impl IWrite for Lseek {
    fn write(&mut self, buffer: &mut [u8]) -> usize {
        let count = self.as_bytes().len();
        buffer[0..count].copy_from_slice(self.as_bytes());
        count
    }
}

#[derive(IntoBytes, KnownLayout, Immutable)]
#[repr(transparent)]
pub struct CopyFileRange {
    pub arg: fuse_write_out,
}

impl IWrite for CopyFileRange {
    fn write(&mut self, buffer: &mut [u8]) -> usize {
        let count = self.as_bytes().len();
        buffer[0..count].copy_from_slice(self.as_bytes());
        count
    }
}

#[derive(IntoBytes, KnownLayout, Immutable)]
#[repr(transparent)]
pub struct SetVolName {}

impl IWrite for SetVolName {
    fn write(&mut self, buffer: &mut [u8]) -> usize {
        let count = self.as_bytes().len();
        buffer[0..count].copy_from_slice(self.as_bytes());
        count
    }
}

#[derive(IntoBytes, KnownLayout, Immutable)]
#[repr(transparent)]
#[cfg(target_os = "macos")]
pub struct GetXTimes {
    arg: fuse_getxtimes_out,
}

#[cfg(target_os = "macos")]
impl IWrite for GetXTimes {
    fn write_too(&mut self, buffer: &mut [u8]) -> usize {
        let count = self.as_bytes().len();
        buffer[0..count].copy_from_slice(self.as_bytes());
        count
    }
}

#[derive(IntoBytes, KnownLayout, Immutable)]
#[repr(transparent)]
pub struct Exchange {}

impl IWrite for Exchange {
    fn write(&mut self, buffer: &mut [u8]) -> usize {
        let count = self.as_bytes().len();
        buffer[0..count].copy_from_slice(self.as_bytes());
        count
    }
}

#[cfg(feature = "abi-7-31")]
#[derive(IntoBytes, KnownLayout, Immutable)]
#[repr(transparent)]
pub struct SetupMapping {}

impl IWrite for SetupMapping {
    fn write(&mut self, buffer: &mut [u8]) -> usize {
        let count = self.as_bytes().len();
        buffer[0..count].copy_from_slice(self.as_bytes());
        count
    }
}

#[cfg(feature = "abi-7-31")]
#[derive(IntoBytes, KnownLayout, Immutable)]
#[repr(transparent)]
pub struct RemoveMapping {}

impl IWrite for RemoveMapping {
    fn write(&mut self, buffer: &mut [u8]) -> usize {
        let count = self.as_bytes().len();
        buffer[0..count].copy_from_slice(self.as_bytes());
        count
    }
}

#[cfg(feature = "abi-7-34")]
#[derive(IntoBytes, KnownLayout, Immutable)]
#[repr(transparent)]
pub struct SyncFs {}

impl IWrite for SyncFs {
    fn write(&mut self, buffer: &mut [u8]) -> usize {
        let count = self.as_bytes().len();
        buffer[0..count].copy_from_slice(self.as_bytes());
        count
    }
}

#[cfg(feature = "abi-7-37")]
#[derive(IntoBytes, KnownLayout, Immutable)]
#[repr(transparent)]
pub struct TmpFile {}

impl IWrite for TmpFile {
    fn write(&mut self, buffer: &mut [u8]) -> usize {
        let count = self.as_bytes().len();
        buffer[0..count].copy_from_slice(self.as_bytes());
        count
    }
}

#[cfg(feature = "abi-7-39")]
#[derive(IntoBytes, KnownLayout, Immutable)]
#[repr(transparent)]
pub struct StatX {
    pub arg: fuse_statx_out,
}

impl IWrite for StatX {
    fn write(&mut self, buffer: &mut [u8]) -> usize {
        let count = self.as_bytes().len();
        buffer[0..count].copy_from_slice(self.as_bytes());
        count
    }
}

#[derive(IntoBytes, KnownLayout, Immutable)]
#[repr(transparent)]
pub struct CuseInit {}

impl IWrite for CuseInit {
    fn write(&mut self, buffer: &mut [u8]) -> usize {
        let count = self.as_bytes().len();
        buffer[0..count].copy_from_slice(self.as_bytes());
        count
    }
}

#[repr(u32)]
pub enum Operation {
    Lookup(Lookup) = 1,
    Forget(Forget) = 2,
    GetAttr(GetAttr) = 3,
    SetAttr(SetAttr) = 4,
    ReadLink(ReadLink) = 5,
    SymLink(SymLink) = 6,
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
    Lseek(Lseek)  = 46,
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

impl IWrite for Operation {
    fn write(&mut self, buffer: &mut [u8]) -> usize {
        let count = match self {
            Operation::Lookup(lookup) => lookup.write(buffer),
            Operation::Forget(forget) => forget.write(buffer),
            Operation::GetAttr(get_attr) => get_attr.write(buffer),
            Operation::SetAttr(set_attr) => set_attr.write(buffer),
            Operation::ReadLink(read_link) => read_link.write(buffer),
            Operation::SymLink(sym_link) => sym_link.write(buffer),
            Operation::MkNod(mk_nod) => mk_nod.write(buffer),
            Operation::MkDir(mk_dir) => mk_dir.write(buffer),
            Operation::RmDir(rm_dir) => rm_dir.write(buffer),
            Operation::Rename(rename) => rename.write(buffer),
            Operation::Link(link) => link.write(buffer),
            Operation::Open(open) => open.write(buffer),
            Operation::Read(read) => read.write(buffer),
            Operation::Write(write) => write.write(buffer),
            Operation::StatFs(stat_fs) => stat_fs.write(buffer),
            Operation::Release(release) => release.write(buffer),
            Operation::FSync(fsync) => fsync.write(buffer),
            Operation::SetXAttr(set_xattr) => set_xattr.write(buffer),
            Operation::GetXAttr(get_xattr) => get_xattr.write(buffer),
            Operation::ListXAttr(list_xattr) => list_xattr.write(buffer),
            Operation::RemoveXAttr(remove_xattr) => remove_xattr.write(buffer),
            Operation::Flush(flush) => flush.write(buffer),
            Operation::Init(init) => init.write(buffer),
            Operation::OpenDir(open_dir) => open_dir.write(buffer),
            Operation::ReadDir(read_dir) => read_dir.write(buffer),
            Operation::ReleaseDir(release_dir) => release_dir.write(buffer),
            Operation::FSyncDir(fsync_dir) => fsync_dir.write(buffer),
            Operation::GetLk(get_lk) => get_lk.write(buffer),
            Operation::SetLk(set_lk) => set_lk.write(buffer),
            Operation::Access(access) => access.write(buffer),
            Operation::Create(create) => create.write(buffer),
            Operation::Interrupt(interrupt) => interrupt.write(buffer),
            Operation::BMap(bmap) => bmap.write(buffer),
            Operation::Destroy(destroy) => destroy.write(buffer),
            Operation::IoCtl(io_ctl) => io_ctl.write(buffer),
            Operation::Poll(poll) => poll.write(buffer),
            Operation::NotifyReply(notify_reply) => notify_reply.write(buffer),
            Operation::BatchForget(batch_forget) => batch_forget.write(buffer),
            Operation::FAllocate(fallocate) => fallocate.write(buffer),
            Operation::ReadDirPlus(read_dir_plus) => read_dir_plus.write(buffer),
            Operation::Rename2(rename2) => rename2.write(buffer),
            Operation::Lseek(lseek) => lseek.write(buffer),
            Operation::CopyFileRange(copy_file_range) => copy_file_range.write(buffer),
            Operation::CuseInit(cuse_init) => cuse_init.write(buffer),
            Operation::Unlink(unlink) => unlink.write(buffer),
            #[cfg(feature = "abi-7-31")]
            Operation::SetupMapping(op) => op.write(buffer),
            #[cfg(feature = "abi-7-31")]
            Operation::RemoveMapping(op) => op.write(buffer),
            #[cfg(feature = "abi-7-34")]
            Operation::SyncFs(op) => op.write(buffer),
            #[cfg(feature = "abi-7-37")]
            Operation::TmpFile(op) => op.write(buffer),
            #[cfg(feature = "abi-7-39")]
            Operation::StatX(statx) => statx.write(buffer),
        };

        count
    }
}

/*
#[cfg(test)]
mod test {
    use std::time::Duration;

    use libc::{S_IFDIR, S_IFREG};

    use crate::messages::reply::{
        fuse_attr, fuse_dirent, fuse_entry_out, fuse_open_out, fuse_out_header, DirectoryEntry, FSync, IWrite, Lookup,
        Open, Operation, Reply,
    };

    use super::{fuse_create_out, Create};

    #[test]
    fn reply_error() {
        let mut r = Reply::new(0xdeadbeef, -libc::EREMOTE, Some(Operation::FSync(FSync {})));
        r.update_length();

        let mut buffer = Vec::new();
        r.write(&mut buffer).expect("write");

        assert_eq!(
            buffer,
            vec![0x10, 0x00, 0x00, 0x00, 0xbe, 0xff, 0xff, 0xff, 0xef, 0xbe, 0xad, 0xde, 0x00, 0x00, 0x00, 0x00,],
        );
    }

    #[test]
    fn reply_read() {
        let data = [0xde, 0xad, 0xbe, 0xef];
        let mut r = Reply::new(
            0xdeadbeef,
            0,
            Some(Operation::Read(super::Read { data: data.to_vec() })),
        );
        r.update_length();

        let mut buffer = Vec::new();
        r.write(&mut buffer).expect("write");
    }

    #[test]
    fn reply_entry() {
        let mut expected = if cfg!(target_os = "macos") {
            vec![
                0x98, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xef, 0xbe, 0xad, 0xde, 0x00, 0x00, 0x00, 0x00, 0x11,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xaa, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x65, 0x87,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x65, 0x87, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x21, 0x43, 0x00,
                0x00, 0x21, 0x43, 0x00, 0x00, 0x11, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x22, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x33, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x34, 0x12, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x34, 0x12, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x34, 0x12, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x34, 0x12, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x78, 0x56, 0x00, 0x00, 0x78, 0x56, 0x00,
                0x00, 0x78, 0x56, 0x00, 0x00, 0x78, 0x56, 0x00, 0x00, 0xa4, 0x81, 0x00, 0x00, 0x55, 0x00, 0x00, 0x00,
                0x66, 0x00, 0x00, 0x00, 0x77, 0x00, 0x00, 0x00, 0x88, 0x00, 0x00, 0x00, 0x99, 0x00, 0x00, 0x00,
            ]
        } else {
            vec![
                0x88, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xef, 0xbe, 0xad, 0xde, 0x00, 0x00, 0x00, 0x00, 0x11,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xaa, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x65, 0x87,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x65, 0x87, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x21, 0x43, 0x00,
                0x00, 0x21, 0x43, 0x00, 0x00, 0x11, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x22, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x33, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x34, 0x12, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x34, 0x12, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x34, 0x12, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x78, 0x56, 0x00, 0x00, 0x78, 0x56, 0x00, 0x00, 0x78, 0x56, 0x00, 0x00, 0xa4, 0x81, 0x00,
                0x00, 0x55, 0x00, 0x00, 0x00, 0x66, 0x00, 0x00, 0x00, 0x77, 0x00, 0x00, 0x00, 0x88, 0x00, 0x00, 0x00,
            ]
        };

        if cfg!(feature = "abi-7-9") {
            expected.extend(vec![0xbb, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
        }
        expected[0] = (expected.len()) as u8;

        let time_secs = Duration::new(0x1234, 0x5678).as_secs() as i64;
        let time_nsecs = Duration::new(0x1234, 0x5678).subsec_nanos();
        let ttl = Duration::new(0x8765, 0x4321);

        let entry = Lookup {
            arg: fuse_entry_out {
                nodeid: 0x11,
                generation: 0xaa,
                entry_valid: ttl.as_secs(),
                attr_valid: ttl.as_secs(),
                entry_valid_nsec: ttl.subsec_nanos(),
                attr_valid_nsec: ttl.subsec_nanos(),
                attr: fuse_attr {
                    nlink: 0x55,
                    size: 0x22,
                    blocks: 0x33,
                    atime: time_secs,
                    mtime: time_secs,
                    ctime: time_secs,
                    mode: 0o644 | S_IFREG,
                    uid: 0x66,
                    gid: 0x77,
                    rdev: 0x88,
                    blksize: 0xbb,
                    atimensec: time_nsecs,
                    ctimensec: time_nsecs,
                    mtimensec: time_nsecs,
                    ino: 0x11,
                    padding: 0,
                },
            },
        };

        let operation = Operation::Lookup(entry);

        let mut reply = Reply {
            header: fuse_out_header {
                unique: 0xdeadbeef,
                error: 0,
                len: 0,
            },
            operation: Some(operation),
        };

        let mut buffer = Vec::new();
        reply.write(&mut buffer).expect("write");

        // assert_eq!(expected.len(), buffer.len());
        assert_eq!(expected, buffer);
    }

    #[test]
    fn reply_open() {
        let expected = vec![
            0x20, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xef, 0xbe, 0xad, 0xde, 0x00, 0x00, 0x00, 0x00, 0x22, 0x11,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x33, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];

        let operation = Operation::Open(Open {
            arg: fuse_open_out {
                fh: 0x1122,
                open_flags: 0x33,
                padding: 0,
            },
        });

        let mut reply = Reply {
            header: fuse_out_header {
                unique: 0xdeadbeef,
                error: 0,
                len: 0,
            },
            operation: Some(operation),
        };

        let mut buffer = Vec::new();
        reply.write(&mut buffer).expect("write");

        assert_eq!(expected, buffer);
    }

    #[test]
    fn reply_create() {
        let mut expected = if cfg!(target_os = "macos") {
            vec![
                0xa8, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xef, 0xbe, 0xad, 0xde, 0x00, 0x00, 0x00, 0x00, 0x11,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xaa, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x65, 0x87,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x65, 0x87, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x21, 0x43, 0x00,
                0x00, 0x21, 0x43, 0x00, 0x00, 0x11, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x22, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x33, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x34, 0x12, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x34, 0x12, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x34, 0x12, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x34, 0x12, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x78, 0x56, 0x00, 0x00, 0x78, 0x56, 0x00,
                0x00, 0x78, 0x56, 0x00, 0x00, 0x78, 0x56, 0x00, 0x00, 0xa4, 0x81, 0x00, 0x00, 0x55, 0x00, 0x00, 0x00,
                0x66, 0x00, 0x00, 0x00, 0x77, 0x00, 0x00, 0x00, 0x88, 0x00, 0x00, 0x00, 0x99, 0x00, 0x00, 0x00, 0xbb,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xcc, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            ]
        } else {
            vec![
                0x98, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xef, 0xbe, 0xad, 0xde, 0x00, 0x00, 0x00, 0x00, 0x11,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xaa, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x65, 0x87,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x65, 0x87, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x21, 0x43, 0x00,
                0x00, 0x21, 0x43, 0x00, 0x00, 0x11, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x22, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x33, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x34, 0x12, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x34, 0x12, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x34, 0x12, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x78, 0x56, 0x00, 0x00, 0x78, 0x56, 0x00, 0x00, 0x78, 0x56, 0x00, 0x00, 0xa4, 0x81, 0x00,
                0x00, 0x55, 0x00, 0x00, 0x00, 0x66, 0x00, 0x00, 0x00, 0x77, 0x00, 0x00, 0x00, 0x88, 0x00, 0x00, 0x00,
                0xbb, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xcc, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            ]
        };

        if cfg!(feature = "abi-7-9") {
            let insert_at = expected.len() - 16;
            expected.splice(
                insert_at..insert_at,
                vec![0xdd, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
            );
        }
        expected[0] = (expected.len()) as u8;

        let time_secs = Duration::new(0x1234, 0x5678).as_secs() as i64;
        let time_nsecs = Duration::new(0x1234, 0x5678).subsec_nanos();
        let ttl = Duration::new(0x8765, 0x4321);

        let operation = Operation::Create(Create {
            arg: fuse_create_out {
                entry: fuse_entry_out {
                    nodeid: 0x11,
                    generation: 0xaa,
                    entry_valid: ttl.as_secs(),
                    attr_valid: ttl.as_secs(),
                    entry_valid_nsec: ttl.subsec_nanos(),
                    attr_valid_nsec: ttl.subsec_nanos(),
                    attr: fuse_attr {
                        nlink: 0x55,
                        size: 0x22,
                        blocks: 0x33,
                        atime: time_secs,
                        mtime: time_secs,
                        ctime: time_secs,
                        mode: 0o644 | S_IFREG,
                        uid: 0x66,
                        gid: 0x77,
                        rdev: 0x88,
                        blksize: 0xdd,
                        atimensec: time_nsecs,
                        ctimensec: time_nsecs,
                        mtimensec: time_nsecs,
                        ino: 0x11,
                        padding: 0,
                    },
                },
                open: fuse_open_out {
                    fh: 0xbb,
                    open_flags: 0xcc,
                    padding: 0,
                },
            },
        });

        let mut buffer = Vec::new();
        let mut reply = Reply::new(0xdeadbeef, 0, Some(operation));
        reply.write(&mut buffer).expect("write");

        assert_eq!(expected, buffer);
    }

    #[test]
    fn reply_directory() {
        let expected = vec![
            0x50, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xef, 0xbe, 0xad, 0xde, 0x00, 0x00, 0x00, 0x00, 0xbb, 0xaa,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00,
            0x04, 0x00, 0x00, 0x00, 0x68, 0x65, 0x6c, 0x6c, 0x6f, 0x00, 0x00, 0x00, 0xdd, 0xcc, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x08, 0x00, 0x00, 0x00, 0x08, 0x00, 0x00, 0x00,
            0x77, 0x6f, 0x72, 0x6c, 0x64, 0x2e, 0x72, 0x73,
        ];

        let n1 = String::from("hello");

        let e1 = DirectoryEntry {
            entry: fuse_dirent {
                ino: 0xaabb,
                off: 1,
                typ: S_IFDIR >> 12,
                namelen: n1.len() as u32,
            },
            name: n1,
        };

        let n2 = String::from("world.rs");
        let e2 = DirectoryEntry {
            entry: fuse_dirent {
                ino: 0xccdd,
                off: 2,
                typ: S_IFREG >> 12,
                namelen: n2.len() as u32,
            },
            name: n2,
        };

        let operation = Operation::ReadDir(super::ReadDir { entries: vec![e1, e2] });

        let mut reply = Reply::new(0xdeadbeef, 0, Some(operation));

        let mut buffer = Vec::new();
        reply.write(&mut buffer).expect("write");

        assert_eq!(expected, buffer);
    }

    #[test]
    fn reply_bmap() {
        let expected = vec![
            0x18, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xef, 0xbe, 0xad, 0xde, 0x00, 0x00, 0x00, 0x00, 0x34, 0x12,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];

        let operation = Operation::BMap(crate::messages::reply::BMap {
            arg: crate::messages::reply::fuse_bmap_out { block: 0x1234 },
        });

        let mut reply = Reply::new(0xdeadbeef, 0, Some(operation));

        let mut buffer = Vec::new();
        reply.write(&mut buffer).expect("write");

        assert_eq!(expected, buffer);
    }

    #[test]
    fn reply_write() {
        let expected = vec![
            0x18, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xef, 0xbe, 0xad, 0xde, 0x00, 0x00, 0x00, 0x00, 0x22, 0x11,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];

        let operation = Operation::Write(crate::messages::reply::IWrite {
            arg: crate::messages::reply::fuse_write_out {
                size: 0x1122,
                padding: 0,
            },
        });

        let mut reply = Reply::new(0xdeadbeef, 0, Some(operation));

        let mut buffer = Vec::new();
        reply.write(&mut buffer).expect("write");

        assert_eq!(expected, buffer);
    }
}
*/
