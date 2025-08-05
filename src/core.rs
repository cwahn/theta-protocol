use alloc::borrow::Cow;
use alloc::boxed::Box;
use alloc::vec::Vec;
use ed25519_compact::PublicKey;

use crate::error::Error;
use futures::future::BoxFuture;

/// Composable OSI layer 3 implementation
pub trait Network: Send + Sync {
    /// Check if supported scheme
    fn is_supported_scheme(&self, scheme: Scheme) -> bool;

    /// Bind to a local address.
    fn bind(&self, local_addr: &HostAddr) -> Result<(), Error>;

    /// Free the local address, allowing it to be reused.
    fn free(&self, local_addr: &HostAddr) -> Result<(), Error>;

    /// Connect to a remote address.
    fn connect(&self, remote_addrs: &HostAddr) -> BoxFuture<Result<Box<dyn Transport>, Error>>;

    /// Accept a connection from a remote address.
    /// Should spawn tasks for each network that supports the scheme.
    fn run(&self, on_accept: fn(PublicKey, Box<dyn Transport>));

    // ! Currently, there is no way to recover if the run method fails.
    // todo: Might need to find a way for graceful shutdown
}

/// OSI layer 4 implementation
pub trait Transport: Send + Sync + Sender + Receiver {
    fn open_uni(&self) -> BoxFuture<Result<Box<dyn Sender>, Error>>;

    fn accept_uni(&self) -> BoxFuture<Result<Box<dyn Receiver>, Error>>;
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

pub struct ActorAddr {
    ident: ActorIdent,
    host: HostAddr,
}

pub type ActorIdent = Cow<'static, [u8]>;

pub struct HostAddr {
    pub scheme: Scheme,          // e.g. "tcp", "udp", "ws", "wss"
    pub host: Cow<'static, str>, // e.g. "example.com"
}

pub type Scheme = &'static str;
