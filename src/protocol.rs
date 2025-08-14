use alloc::{sync::Arc, vec::Vec};
use ed25519_compact::PublicKey;
use futures::FutureExt;
use url::Url;

use crate::core::{Network, Transport};

// pub type Routes = HashMap<PublicKey, Url>;

pub struct RouteEntry {
    pub addr: Url,
    pub mb_conn: Option<Arc<dyn Transport>>,
}

// Assume that no duplicated scheme support

impl Network for Vec<Arc<dyn Network>> {
    fn is_supported_scheme(&self, addr: &Url) -> bool {
        self.iter().any(|n| n.is_supported_scheme(addr))
    }

    fn connect(
        &self,
        remote_addrs: Url,
    ) -> futures::future::BoxFuture<'_, Result<Arc<dyn Transport>, crate::error::Error>> {
        async move {
            for network in self {
                if network.is_supported_scheme(&remote_addrs) {
                    return network.connect(remote_addrs).await;
                }
            }

            // todo Fix error
            Err(crate::error::Error::Simple(
                crate::error::ErrorKind::SendError,
            ))
        }
        .boxed()
    }

    fn run(&self, on_accept: fn(PublicKey, Arc<dyn Transport>)) {
        for network in self {
            network.run(on_accept);
        }
    }
}
