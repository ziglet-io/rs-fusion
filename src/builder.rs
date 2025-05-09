use std::{
    os::{
        fd::{AsFd, AsRawFd, FromRawFd},
        unix::fs::FileTypeExt,
    },
    path::PathBuf,
};

use log::error;
use tokio_util::sync::CancellationToken;

use crate::{
    error::Errno,
    mount::{mount_options::MountOption, Mount},
    session::{Inner, Session},
    RequestTx, SIZE_BUFFER,
};

pub struct Builder {
    /// Path to the device file (e.g. /dev/fuse)
    device_path: PathBuf,
    /// Path to the mount point
    mount_path: Option<PathBuf>,
    /// Mount options
    mount_options: Vec<MountOption>,

    outbound_fs_request_tx: Option<RequestTx>,

    cancellation_token: CancellationToken,
}

impl Builder {
    pub fn new() -> Self {
        let default_mount_options = vec![
            MountOption::AllowOther,
            MountOption::DefaultPermissions,
            MountOption::NoDev,
            MountOption::NoAtime,
        ];

        Self {
            device_path: PathBuf::from("/dev/fuse"),
            mount_path: None,
            mount_options: default_mount_options,
            outbound_fs_request_tx: None,
            cancellation_token: CancellationToken::new(),
        }
    }

    /// Supply your own cancellation token to control the long-running actor.
    ///
    /// Can be used to synchronize cancellation with other processes/actors in a broader system.
    pub fn set_cancellation_token(&mut self, token: &CancellationToken) -> &mut Self {
        self.cancellation_token = token.clone();
        self
    }

    /// Supply the channel on which [Session] will forward requests to the filesystem.
    ///
    /// REQUIRED
    pub fn set_outbound_fs_request_tx(&mut self, tx: &RequestTx) -> &mut Self {
        self.outbound_fs_request_tx = Some(tx.clone());
        self
    }

    /// The device path defaults to `/dev/fuse`. Overwrite that here.
    pub fn set_device_path(&mut self, path: PathBuf) -> &mut Self {
        self.device_path = path;
        self
    }

    /// Set the path where the filesystem will be (attempted to be) mounted
    pub fn set_mount_path(&mut self, path: PathBuf) -> &mut Self {
        self.mount_path = Some(path);
        self
    }

    pub fn set_mount_options(&mut self, options: &[MountOption]) -> &mut Self {
        self.mount_options = options.to_vec();
        self
    }

    pub async fn build(&mut self) -> Result<Session, Errno> {
        if self.outbound_fs_request_tx.is_none() {
            error!("outbound fs request channel required");
            return Err(Errno::EINVAL);
        }

        if !self.device_path.exists() {
            error!("device path {:?} does not exist", self.device_path);
            return Err(Errno::ENOENT);
        }

        if self.mount_path.is_none() {
            error!("mount path required");
            return Err(Errno::EINVAL);
        }

        if !tokio::fs::metadata(&self.device_path)
            .await?
            .file_type()
            .is_char_device()
        {
            error!("path {:?} exists but is not a block device", self.device_path);
            return Err(Errno::ENODEV);
        }

        // Create the mount
        // TODO debugging EPERM
        self.mount_options.push(MountOption::AllowOther);
        // if self.mount_options.contains(&MountOption::AutoUnmount)
        //     && !(self.mount_options.contains(&MountOption::AllowRoot)
        //         || self.mount_options.contains(&MountOption::AllowOther))
        // {
        //     self.mount_options.push(MountOption::AllowOther);
        // };

        let (file, mount) = Mount::new(self.mount_path.as_ref().unwrap(), &self.mount_options)?;

        let writer = unsafe { std::fs::File::from_raw_fd(file.as_fd().as_raw_fd()) };

        let (reply_tx, reply_rx) = crate::create_reply_channel();

        let mut inner = Inner {
            _mount: mount,
            file,
            writer,
            buffer: vec![0u8; SIZE_BUFFER],
            cancellation_token: self.cancellation_token.clone(),
            inbound_fs_reply_tx: reply_tx,
            inbound_fs_reply_rx: reply_rx,
            outbound_fs_request_tx: self.outbound_fs_request_tx.as_ref().unwrap().clone(),
        };

        let session = Session {
            cancellation_token: self.cancellation_token.clone(),
            outbound_fs_request_tx: self.outbound_fs_request_tx.as_ref().unwrap().clone(),
        };

        // Start the actor
        tokio::spawn(async move {
            match inner.run().await {
                Err(e) => {
                    error!("session failed with {:?}", e);
                }
                Ok(_) => {}
            }
        });

        Ok(session)
    }
}
