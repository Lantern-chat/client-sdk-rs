#[macro_use]
extern crate serde;

#[macro_use]
pub mod models;

#[cfg(feature = "client")]
pub mod api;
