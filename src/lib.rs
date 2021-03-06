#[macro_use]
extern crate serde;

pub use models::Snowflake;

#[macro_use]
pub mod util {
    #[macro_use]
    pub mod schema;
    pub mod fixed;

    #[macro_use]
    pub mod flags;
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
