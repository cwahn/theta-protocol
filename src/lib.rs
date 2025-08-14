#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "codec")]
pub mod codec;

pub mod core;
pub mod error;
pub mod protocol;
