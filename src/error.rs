#[cfg(all(feature = "alloc", not(feature = "std")))]
use alloc::boxed::Box;
#[cfg(feature = "std")]
use std::boxed::Box;

use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum Error {
    #[error("TBD")]
    Tbd,
}
