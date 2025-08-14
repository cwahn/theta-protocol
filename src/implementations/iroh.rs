use crate::codec::postcard_prefix::PostcardPrefixCodec;
use crate::core::{Network, Receiver, Sender, Transport};
use crate::error::Error;
use alloc::boxed::Box;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;
use ed25519_compact::PublicKey;
use futures::future::BoxFuture;
use futures::{FutureExt, SinkExt, StreamExt};
use iroh::endpoint::{Connection, RecvStream, SendStream};
use iroh::{Endpoint, NodeAddr, PublicKey as IrohPublicKey};
use std::collections::HashMap;
use std::sync::Mutex;
use tokio_util::codec::{FramedRead, FramedWrite};
use url::Url;

#[derive(Debug)]
pub struct IrohNetwork {
    endpoint: Endpoint,
}

#[derive(Debug)]
pub struct IrohTransport {
    conn: Connection,
}

#[derive(Debug)]
pub struct IrohReceiver(FramedRead<RecvStream, PostcardPrefixCodec<Vec<u8>>>);

#[derive(Debug)]
pub struct IrohSender(FramedWrite<SendStream, PostcardPrefixCodec<Vec<u8>>>);

// Implementation

impl Network for IrohNetwork {
    fn is_supported_scheme(&self, addr: &Url) -> bool {
        addr.scheme() == "iroh"
    }

    fn connect(&self, remote_addrs: &Url) -> Result<Arc<dyn Transport>, Error> {

        let conn_fut = self.endpoint.connect(addr);
    }

    fn run(&self, _on_accept: fn(PublicKey, Arc<dyn Transport>)) {
        // Implementation for accepting connections would go here
    }
}
