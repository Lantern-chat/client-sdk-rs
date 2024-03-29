#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "schemars")]
extern crate alloc;

#[cfg(feature = "rkyv")]
pub extern crate rkyv;

#[cfg(feature = "rkyv")]
pub extern crate rend;

pub extern crate bitflags_serde_shim;

pub mod db;
pub mod fixed;
pub mod schema;
pub mod ser;
