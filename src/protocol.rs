// use alloc::{sync::Arc, vec::Vec};
// use futures::FutureExt;
// use url::Url;

// use crate::{
//     core::{Network, Receiver, Sender, Transport},
//     error::Error,
// };

// // Assume that no duplicated scheme support
// impl Network
//     for Vec<
//         Arc<
//             dyn Network<
//                 Transport = Arc<
//                     dyn Transport<Sender = Box<dyn Sender>, Receiver = Box<dyn Receiver>>,
//                 >,
//             >,
//         >,
//     >
// {
//     type Transport = Arc<dyn Transport<Sender = Box<dyn Sender>, Receiver = Box<dyn Receiver>>>;

//     fn is_supported_scheme(&self, addr: &Url) -> bool {
//         self.iter().any(|n| n.is_supported_scheme(addr))
//     }

//     fn run(&self, on_accept: fn(Self::Transport)) {
//         for network in self {
//             network.run(on_accept);
//         }
//     }

//     fn connect(&self, host_addr: Url) -> Result<Self::Transport, crate::error::Error> {
//         for network in self {
//             if network.is_supported_scheme(&host_addr) {
//                 return network.connect(host_addr);
//             }
//         }
//         Err(Error::Custom)
//     }
// }

// impl<
//     T: ?Sized
//         + Network<
//             Transport = Arc<dyn Transport<Sender = Box<dyn Sender>, Receiver = Box<dyn Receiver>>>,
//         >,
// > Network for Arc<T>
// {
//     type Transport = Arc<dyn Transport<Sender = Box<dyn Sender>, Receiver = Box<dyn Receiver>>>;

//     fn is_supported_scheme(&self, addr: &Url) -> bool {
//         (**self).is_supported_scheme(addr)
//     }

//     fn run(&self, on_accept: fn(Self::Transport)) {
//         (**self).run(on_accept);
//     }

//     fn connect(&self, host_addr: Url) -> Result<Self::Transport, crate::error::Error> {
//         let transport = (**self).connect(host_addr)?;

//         Ok(Arc::new(transport) as Self::Transport)
//     }
// }

// impl<T: ?Sized + Transport> Transport for Arc<T> {
//     type Receiver = Box<dyn Receiver>;
//     type Sender = Box<dyn Sender>;

//     fn send_datagram(
//         &self,
//         payload: Vec<u8>,
//     ) -> futures::future::BoxFuture<'_, Result<(), crate::error::Error>> {
//         (**self).send_datagram(payload)
//     }

//     fn recv_datagram(
//         &self,
//     ) -> futures::future::BoxFuture<'_, Result<Vec<u8>, crate::error::Error>> {
//         (**self).recv_datagram()
//     }

//     fn open_uni(
//         &self,
//     ) -> futures::future::BoxFuture<'_, Result<Self::Sender, crate::error::Error>> {
//         async move {
//             let sender = (**self).open_uni().await?;
//             Ok(Box::new(sender) as Box<dyn Sender>)
//         }
//         .boxed()
//     }

//     fn accept_uni(
//         &self,
//     ) -> futures::future::BoxFuture<'_, Result<Self::Receiver, crate::error::Error>> {
//         async move {
//             let receiver = (**self).accept_uni().await?;
//             Ok(Box::new(receiver) as Box<dyn Receiver>)
//         }
//         .boxed()
//     }

//     fn host_addr(&self) -> futures::future::BoxFuture<'_, Result<Url, crate::error::Error>> {
//         (**self).host_addr()
//     }
// }

// impl<T: ?Sized + Sender> Sender for Box<T> {
//     fn send_frame(
//         &mut self,
//         payload: Vec<u8>,
//     ) -> futures::future::BoxFuture<'_, Result<(), crate::error::Error>> {
//         (**self).send_frame(payload)
//     }
// }

// impl<T: ?Sized + Receiver> Receiver for Box<T> {
//     fn recv_frame(
//         &mut self,
//     ) -> futures::future::BoxFuture<'_, Result<Vec<u8>, crate::error::Error>> {
//         (**self).recv_frame()
//     }
// }
