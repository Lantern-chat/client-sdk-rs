#[macro_use]
pub mod macros;

pub mod types {
    //! Types used in models
    //!
    //! These types/modules are dual licensed under MIT or Apache 2.0

    pub mod fixed_str;
    pub mod thin_str;
}

pub use types::*;
