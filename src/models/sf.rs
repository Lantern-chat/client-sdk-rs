use super::*;

use time::{OffsetDateTime, PrimitiveDateTime};

/// Arbitrarily chosen starting epoch to offset the clock by
pub const LANTERN_EPOCH: u64 = 1550102400000;

pub const LANTERN_EPOCH_ODT: OffsetDateTime = time::macros::datetime!(2019 - 02 - 14 00:00 +0);
pub const LANTERN_EPOCH_PDT: PrimitiveDateTime = time::macros::datetime!(2019 - 02 - 14 00:00);

pub use snowflake::Snowflake;

#[cfg(feature = "rkyv")]
pub use snowflake::NicheSnowflake;
