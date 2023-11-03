#![allow(clippy::bad_bit_mask)]

#[macro_use]
extern crate serde;

pub use models::Snowflake;

#[macro_use]
pub mod util {
    pub mod prefs;
}

#[macro_use]
pub mod models;

#[cfg(feature = "api")]
pub mod api;

#[cfg(feature = "driver")]
pub mod driver;

#[cfg(feature = "client")]
pub mod client;

#[cfg(feature = "gateway")]
pub mod gateway;

#[cfg(feature = "framework")]
pub mod framework;

#[cfg(feature = "framework_utils")]
pub mod framework_utils;
