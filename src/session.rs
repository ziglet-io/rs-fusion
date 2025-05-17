use libc::{EAGAIN, EINTR, ENODEV, ENOENT};
use std::io::Write;
use tokio::select;
use tokio::{fs::File, io::AsyncReadExt};
use tokio_util::sync::CancellationToken;
use zerocopy::TryFromBytes;

use crate::error::Errno;
use crate::messages::fuse_abi::fuse_in_header;
use crate::{
    messages::{
        reply::{IWrite, Reply},
        request::Request,
    },
    mount::Mount,
    ReplyRx, ReplyTx, RequestTx,
};

use log::{error, info, trace, warn};

/// Represents a single session between the kernel and a filesystem.
///
/// This is a simple struct holding some data that an application might be interested in having about the "real"
/// long-running process [Inner] such as the channel for the Filesystem where kernel messages will be sent and
/// the [CancellationToken] that can be used to cancel the actor [Inner].
pub struct Session {
    pub(crate) cancellation_token: CancellationToken,
    pub(crate) outbound_fs_request_tx: RequestTx,
}

impl Session {
    pub fn cancel(&mut self) {
        self.cancellation_token.cancel()
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancellation_token.is_cancelled()
    }

    pub fn get_outbound_fs_request_tx(&self) -> &RequestTx {
        &self.outbound_fs_request_tx
    }
}

/// Internal "actor" that represents a long-running process ferrying kernel requests to the filesystem and
/// replies from the filesystem to the kernel.
pub(crate) struct Inner {
    pub(crate) file: File,
    pub(crate) writer: std::fs::File,
    pub(crate) _mount: Mount,
    pub(crate) buffer: Vec<u8>,
    pub(crate) outbound_fs_request_tx: RequestTx,
    pub(crate) inbound_fs_reply_tx: ReplyTx,
    pub(crate) inbound_fs_reply_rx: ReplyRx,
    pub(crate) cancellation_token: CancellationToken,
}

impl Inner {
    // --------------------------------------------------------------------------------
    //  Main loop

    /// Main loop.
    pub(crate) async fn run(&mut self) -> Result<(), Errno> {
        info!("started");

        while !self.cancellation_token.is_cancelled() {
            select! {
                 _ = self.cancellation_token.cancelled() => {}
                 reply = self.inbound_fs_reply_rx.recv() => {
                     self.on_fs_reply(reply).await?;
                 }
                  read_result = self.file.read(&mut self.buffer) => {

                      match read_result {
                     Err(e) => {
                         match e.raw_os_error() {
                             // Operation interrupted
                             Some(ENOENT) => {
                                 info!("ENOENT");
                                 continue;
                             },
                             // Interrupted by syscall, retry
                             Some(EINTR) => {
                                 info!("EINTR");
                                 continue;
                             }
                             // Explicit "try again"
                             Some(EAGAIN) => {
                                 info!("EAGAIN");
                                 continue;
                             },
                             // Unmounted
                             Some(ENODEV) => {
                                 warn!("ENODEV");
                                 self.cancellation_token.cancel();
                             },
                             // Unhandled
                             _ => return Err(e.into()),
                         }
                     }
                     Ok(_bytes) => {
                         let request = Request::parse(&mut self.buffer, &self.inbound_fs_reply_tx)?;
                         if let Err(_e) = self.outbound_fs_request_tx.send(request).await {
                             error!("channel send");
                             return Err(Errno::EIO);
                         }
                     }
                 }
                 }
            }
        }

        fuse_in_header::try_ref_from_prefix(&self.buffer).unwrap().0.opcode;

        info!("done");

        Ok(())
    }

    // --------------------------------------------------------------------------------
    // Event handlers

    pub(crate) async fn on_fs_reply(&mut self, reply: Option<Reply>) -> Result<(), Errno> {
        trace!("on_fs_reply");

        if reply.is_none() {
            error!("channel send");
            return Err(Errno::EIO);
        }

        let mut reply = reply.unwrap();

        let count = reply.write(&mut self.buffer);

        match self.writer.write(&self.buffer[..count]) {
            Err(e) => {
                return Err(e.into());
            }
            Ok(r) => r,
        };

        Ok(())
    }
}
