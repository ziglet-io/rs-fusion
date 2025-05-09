//! TODO Init flags
//! Emit FUSE_PARALLEL_DIROPS on init

use constants::*;
use messages::{reply::Reply, request::Request};

pub mod builder;
pub mod constants;
pub mod error;
pub mod messages;
pub mod mount;
pub mod session;

pub const MEBI: u64 = 2u64.pow(20);
pub const SIZE_CHANNEL: usize = 32;
pub const SIZE_BUFFER: usize = 16 * MEBI as usize;

pub type RequestTx = tokio::sync::mpsc::Sender<Request>;
pub type RequestRx = tokio::sync::mpsc::Receiver<Request>;

pub type ReplyTx = tokio::sync::mpsc::Sender<Reply>;
pub type ReplyRx = tokio::sync::mpsc::Receiver<Reply>;

pub fn create_request_channel() -> (RequestTx, RequestRx) {
    tokio::sync::mpsc::channel::<Request>(SIZE_CHANNEL)
}

pub fn create_reply_channel() -> (ReplyTx, ReplyRx) {
    tokio::sync::mpsc::channel::<Reply>(SIZE_CHANNEL)
}

pub const INIT_FLAGS: u32 = FUSE_ASYNC_READ
    | FUSE_BIG_WRITES
    | FUSE_ASYNC_DIO
    | FUSE_FSYNC_FDATASYNC
    | FUSE_FILE_OPS
    | FUSE_ATOMIC_O_TRUNC
    | FUSE_EXPORT_SUPPORT
    | FUSE_BIG_WRITES;

pub fn supported_init_flags() -> u32 {
    let mut init = INIT_FLAGS;

    #[cfg(feature = "abi-7-12")]
    {
        init |= FUSE_DONT_MASK
    }

    #[cfg(feature = "abi-7-17")]
    {
        init |= FUSE_FLOCK_LOCKS
    }

    #[cfg(feature = "abi-7-28")]
    {
        init |= FUSE_MAX_PAGES;
    }

    #[cfg(target_os = "macos")]
    {
        init |= FUSE_ASYNC_READ | FUSE_VOL_RENAME | FUSE_XTIMES;
    }

    init
}
