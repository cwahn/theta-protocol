#[cfg(feature = "alloc")]
mod connection_impl {
    use alloc::boxed::Box;
    #[cfg(all(feature = "alloc", not(feature = "std")))]
    use alloc::vec::Vec;
    #[cfg(feature = "std")]
    use std::vec::Vec;

    use crate::error::Error;
    use futures::future::BoxFuture;

    // Should be one per host
    pub trait Connection: Send + Sync + Sender + Receiver {
        fn open_uni(&self) -> BoxFuture<Result<Box<dyn Sender>, Error>>;

        fn accept_uni(&self) -> BoxFuture<Result<Box<dyn Receiver>, Error>>;
    }

    // Logical sender
    // It could be actual stream in case of WebSocket like transport, or internally wrap message with stream_id and send to single internal stream.
    pub trait Sender: Send + Sync {
        /// Send exactly one datagram to the connected peer.
        /// The transport guarantees integrity‐checked, at‐most‐once delivery.
        fn send_datagram(&self, payload: Vec<u8>) -> BoxFuture<Result<(), Error>>;
    }

    // Logical receiver
    // It could be actual stream in case of WebSocket like transport, or internally wrap message from single internal stream.
    pub trait Receiver: Send + Sync {
        /// Receive the next datagram from the peer.
        fn recv_datagram(&self) -> BoxFuture<Result<Vec<u8>, Error>>;
    }
}

#[cfg(feature = "alloc")]
pub use connection_impl::Connection;
