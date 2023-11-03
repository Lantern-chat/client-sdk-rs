#![no_std]

#[cfg(feature = "schemars")]
extern crate alloc;

#[cfg(feature = "rkyv")]
pub extern crate rkyv;

pub extern crate serde_shims;

pub mod db;
pub mod fixed;
pub mod schema;
pub mod ser;
