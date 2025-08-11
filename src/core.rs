use alloc::sync::Arc;
use alloc::vec::Vec;
use alloc::{borrow::Cow, boxed::Box};
use ed25519_compact::PublicKey;
use url::Url;

use crate::error::Error;
use futures::future::BoxFuture;

// * Host and actor address are an URL
// * The last segment of the path is the actor identifier
//
// e.g. "iroh://example.com/e1875890-4b3d-4248-8904-1a7461b9a701"
// e.g. "serial:///dev/ttyUSB0/actor/root"

pub type Ident = Cow<'static, [u8]>;

/// Composable OSI layer 3 implementation
pub trait Network: Send + Sync {
    /// Check if supported scheme
    fn is_supported_scheme(&self, addr: &Url) -> bool;

    /// Bind to a (local) host address.
    fn bind(&self, local_addr: &Url) -> Result<(), Error>;

    /// Free the (local) host address, allowing it to be reused.
    fn free(&self, local_addr: &Url) -> Result<(), Error>;

    /// Connect to a remote host address.
    fn connect(&self, remote_addrs: &Url) -> BoxFuture<Result<Arc<dyn Transport>, Error>>;

    /// Accept a connection from a remote address.
    /// Should spawn tasks for each network that supports the scheme.
    fn run(&self, on_accept: fn(PublicKey, Arc<dyn Transport>));

    // ! Currently, there is no way to recover if the run method fails.
    // todo: Might need to find a way for graceful shutdown
}

/// OSI layer 4 implementation
pub trait Transport: Send + Sync + Sender + Receiver {
    fn open_uni(&self) -> BoxFuture<Result<Box<dyn Sender>, Error>>;

    fn accept_uni(&self) -> BoxFuture<Result<Box<dyn Receiver>, Error>>;

    /// Export address of the given actor identifier.
    /// Commonly used for re-exporting an actor
    fn export_addr(&self, ident: &Ident) -> Url;
}

/// Logical sender
/// It could be actual stream in case of WebSocket like transport, or internally wrap message with stream_id and send to single internal stream.
pub trait Sender: Send + Sync {
    /// - The transport guarantees integrity‐checked, at‐most‐once delivery.
    /// - The transport does not guarantee delivery or ordering
    fn send_datagram(&self, payload: Vec<u8>) -> BoxFuture<Result<(), Error>>;
}

/// Logical receiver
/// It could be actual stream in case of WebSocket like transport, or internally wrap message from single internal stream.
pub trait Receiver: Send + Sync {
    /// Receive the next datagram from the peer.
    fn recv_datagram(&self) -> BoxFuture<Result<Vec<u8>, Error>>;
}
