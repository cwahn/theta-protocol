use std::sync::Arc;

use futures::future::{BoxFuture, Shared};
use futures::{FutureExt, SinkExt, StreamExt};
use iroh::endpoint::{self, Connection, RecvStream, SendStream};
use iroh::{Endpoint, NodeAddr, PublicKey as IrohPublicKey};

use crate::{
    codec::postcard_prefix::PostcardPrefixCodec,
    core::{Network, Receiver, Sender, Transport},
    error::Error,
};

use tokio_util::codec::{FramedRead, FramedWrite};
use url::Url;

#[derive(Debug)]
pub struct IrohNetwork {
    endpoint: Endpoint,
}

#[derive(Debug, Clone)]
pub struct IrohTransport {
    inner: Shared<BoxFuture<'static, Result<Connection, Error>>>,
}

#[derive(Debug)]
pub struct IrohReceiver(FramedRead<RecvStream, PostcardPrefixCodec<Vec<u8>>>);

#[derive(Debug)]
pub struct IrohSender(FramedWrite<SendStream, PostcardPrefixCodec<Vec<u8>>>);

// Implementation

impl Network for IrohNetwork {
    type Transport = IrohTransport;

    fn is_supported_scheme(&self, addr: &Url) -> bool {
        addr.scheme() == "iroh"
    }

    fn run(&self, on_accept: fn(Self::Transport)) {
        // Implementation for accepting connections would go here
        tokio::spawn({
            let endpoint = self.endpoint.clone();

            async move {
                loop {
                    let Some(incomming) = endpoint.accept().await else {
                        // Possible wrong packet log and proceed
                        continue;
                    };

                    let connecting = match incomming.accept() {
                        Ok(conn) => conn,
                        Err(_) => continue, // Handle connection error
                    };

                    let inner = async move { connecting.await.map_err(|_| Error::Custom) }
                        .boxed()
                        .shared();

                    on_accept(IrohTransport { inner });
                }
            }
        });
    }

    fn connect(&self, host_addr: Url) -> Result<Self::Transport, Error> {
        let first_segment = host_addr
            .path_segments()
            .and_then(|mut segments| segments.next())
            .ok_or_else(|| Error::Custom)?;

        let public_key: IrohPublicKey = first_segment.parse().map_err(|_| Error::Custom)?;

        let node_addr = NodeAddr::new(public_key);

        let endpoint = self.endpoint.clone();

        let shared = async move {
            endpoint
                .connect(node_addr, b"theta")
                .await
                .map_err(|_| Error::Custom)
        }
        .boxed()
        .shared();

        Ok(IrohTransport { inner: shared })
    }
}

impl IrohTransport {
    async fn get_conn(&self) -> Result<Connection, Error> {
        self.inner.clone().await
    }
}

impl Transport for IrohTransport {
    type Sender = IrohSender;
    type Receiver = IrohReceiver;

    fn send_datagram(&self, payload: Vec<u8>) -> BoxFuture<'_, Result<(), Error>> {
        async move {
            let conn = self.get_conn().await?;

            conn.send_datagram(payload.into())
                .map_err(|_| Error::Custom)
        }
        .boxed()
    }

    fn recv_datagram(&self) -> BoxFuture<'_, Result<Vec<u8>, Error>> {
        async move {
            let conn = self.get_conn().await?;

            conn.read_datagram()
                .await
                .map(|data| data.into())
                .map_err(|_| Error::Custom)
        }
        .boxed()
    }

    fn open_uni(&self) -> BoxFuture<'_, Result<Self::Sender, Error>> {
        async move {
            let conn = self.get_conn().await?;

            let send_stream = conn.open_uni().await.map_err(|_| Error::Custom)?;

            Ok(IrohSender(FramedWrite::new(
                send_stream,
                PostcardPrefixCodec::default(),
            )))
        }
        .boxed()
    }

    fn accept_uni(&self) -> BoxFuture<'_, Result<Self::Receiver, Error>> {
        async move {
            let conn = self.get_conn().await?;

            let recv_stream = conn.accept_uni().await.map_err(|_| Error::Custom)?;

            Ok(IrohReceiver(FramedRead::new(
                recv_stream,
                PostcardPrefixCodec::default(),
            )))
        }
        .boxed()
    }

    fn host_addr(&self) -> BoxFuture<'_, Result<Url, Error>> {
        async move {
            let conn = self.get_conn().await?;

            let public_key = conn.remote_node_id().expect("Suppose to be infallible");

            let mut url = Url::parse("iroh://").expect("Known valid scheme");
            url.set_path(&format!("/{}", public_key));

            Ok(url)
        }
        .boxed()
    }
}

impl Sender for IrohSender {
    fn send_frame(&mut self, payload: Vec<u8>) -> BoxFuture<'_, Result<(), Error>> {
        async move { self.0.send(payload).await.map_err(|_| Error::Custom) }.boxed()
    }
}

impl Receiver for IrohReceiver {
    fn recv_frame(&mut self) -> BoxFuture<'_, Result<Vec<u8>, Error>> {
        async move {
            match self.0.next().await {
                Some(Ok(frame)) => Ok(frame),
                Some(Err(_)) => Err(Error::Custom),
                None => Err(Error::Custom), // Stream ended
            }
        }
        .boxed()
    }
}
