#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "codec")]
pub mod codec;

pub mod core;
pub mod error;
pub mod protocol;

#[cfg(feature = "iroh")]
pub mod implementations;
