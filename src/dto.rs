use alloc::borrow::Cow;
use ed25519_compact::PublicKey;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct HostDto {
    pub public_key: [u8; PublicKey::BYTES], // Public key of the host, using array to support serde.
}

#[derive(Serialize, Deserialize)]
pub struct ActorRefDto {
    pub host: HostDto,
    pub ident: Cow<'static, [u8]>,
}
