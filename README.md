# FUSE (Filesystem in Userspace)

Re-write of [fuser](https://docs.rs/fuser/latest/fuser/) to provide `async`, message-oriented facilities.

There is no interface for the Filesystem to implement. Instead, the filesystem provides on input channel, [RequestTx], processes messages, and sends [Reply]s back to the [Session].

**Note** this is somewhat less efficient that the implementation in [fuser](https://docs.rs/fuser/latest/fuser/) in that it allocates and deallocates memory on a per-request basis. However, it allows the filesystem to process messages in a fully asynchronous manner.

## Acknowledgements

This library borrows heavily from [fuser](https://docs.rs/fuser/latest/fuser/), especially the low-level ABI compatibility code.
