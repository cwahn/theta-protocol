use alloc::{boxed::Box, vec::Vec};
use ed25519_compact::PublicKey;
use futures::{FutureExt, future::BoxFuture};

use crate::{
    base::HashMap,
    core::{HostAddr, Network, Transport},
};

pub type Routes = HashMap<PublicKey, HostAddr>;

pub struct RouteEntry {
    pub addr: HostAddr,
    pub mb_conn: Option<Box<dyn Transport>>,
}

// Assume that no duplicated scheme support

impl Network for Vec<Box<dyn Network>> {
    fn is_supported_scheme(&self, scheme: crate::core::Scheme) -> bool {
        self.iter().any(|n| n.is_supported_scheme(scheme))
    }

    fn bind(&self, local_addr: &HostAddr) -> Result<(), crate::error::Error> {
        for network in self {
            if network.is_supported_scheme(local_addr.scheme) {
                return network.bind(local_addr);
            }
        }
        // todo Fix error
        Err(crate::error::Error::Simple(
            crate::error::ErrorKind::SendError,
        ))
    }

    fn free(&self, local_addr: &HostAddr) -> Result<(), crate::error::Error> {
        for network in self {
            if network.is_supported_scheme(local_addr.scheme) {
                return network.free(local_addr);
            }
        }
        // todo Fix error
        Err(crate::error::Error::Simple(
            crate::error::ErrorKind::RecvError,
        ))
    }

    fn run(&self, on_accept: fn(PublicKey, Box<dyn Transport>)) {
        for network in self {
            network.run(on_accept);
        }
    }
}
