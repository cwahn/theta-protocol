#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

pub mod base;
pub mod core;
pub mod dto;
pub mod error;
pub mod protocol;
