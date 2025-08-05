#[cfg(feature = "alloc")]
mod connection_impl {
    #[cfg(all(feature = "alloc", not(feature = "std")))]
    use alloc::vec::Vec;
    #[cfg(feature = "std")]
    use std::vec::Vec;

    use futures::future::BoxFuture;
    use crate::error::Error;

    pub trait Connection: Send + Sync {
        /// Send exactly one datagram to the connected peer.
        /// The transport guarantees integrity‐checked, at‐most‐once delivery.
        fn send_datagram(&self, payload: Vec<u8>) -> BoxFuture<Result<(), Error>>;

        /// Receive the next datagram from the peer.
        fn recv_datagram(&self) -> BoxFuture<Result<Vec<u8>, Error>>;
    }
}

#[cfg(feature = "alloc")]
pub use connection_impl::Connection;
