use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Serialize, Deserialize)]
pub struct HostDto(Url);

#[derive(Serialize, Deserialize)]
pub struct ActorRefDto(Url);
