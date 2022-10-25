use super::*;

bitflags::bitflags! {
    /// NOTE: Remember to clear flag caches when they change
    pub struct UserFlags: i32 {
        //const SYSTEM                = 1 << 2;
        //const BOT                   = 1 << 3;
        //const STAFF                 = 1 << 4;

        const DELETED               = 1 << 0;
        const VERIFIED              = 1 << 1;
        const MFA_ENABLED           = 1 << 2;
        const NEEDS_PASSWORD_RESET  = 1 << 3;

        const RESERVED_1            = 1 << 4;
        const RESERVED_2            = 1 << 5;

        // 3-bit integer
        const ELEVATION_1           = 1 << 6;
        const ELEVATION_2           = 1 << 7;
        const ELEVATION_3           = 1 << 8;

        // 3-bit integer
        const PREMIUM_1             = 1 << 9;
        const PREMIUM_2             = 1 << 10;
        const PREMIUM_3             = 1 << 11;

        const RESERVED_3            = 1 << 12;

        // 2-bit integer
        const EXTRA_STORAGE_1       = 1 << 13;
        const EXTRA_STORAGE_2       = 1 << 14;

        const RESERVED_4            = 1 << 15;

        const RESERVED = Self::RESERVED_1.bits | Self::RESERVED_2.bits | Self::RESERVED_3.bits | Self::RESERVED_4.bits;

        /// Always strip these from public responses
        const PRIVATE_FLAGS = Self::VERIFIED.bits | Self::MFA_ENABLED.bits | Self::DELETED.bits | Self::NEEDS_PASSWORD_RESET.bits | Self::EXTRA_STORAGE.bits | Self::RESERVED.bits;

        /// elevation level integer
        const ELEVATION     = Self::ELEVATION_1.bits | Self::ELEVATION_2.bits | Self::ELEVATION_3.bits;

        /// premium level integer
        const PREMIUM       = Self::PREMIUM_1.bits | Self::PREMIUM_2.bits | Self::PREMIUM_3.bits;

        /// extra storage level integer
        const EXTRA_STORAGE = Self::EXTRA_STORAGE_1.bits | Self::EXTRA_STORAGE_2.bits;
    }
}

serde_shims::impl_serde_for_bitflags!(UserFlags);
impl_schema_for_bitflags!(UserFlags);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum ElevationLevel {
    None = 0,
    Bot = 1,

    Reserved = 2,

    Staff = 3,
    System = 4,
}

impl UserFlags {
    #[inline]
    pub const fn from_bits_truncate_public(bits: i32) -> Self {
        Self::from_bits_truncate(bits).difference(Self::PRIVATE_FLAGS)
    }

    pub fn elevation(self) -> ElevationLevel {
        match (self & Self::ELEVATION).bits() >> 6 {
            1 => ElevationLevel::Bot,
            3 => ElevationLevel::Staff,
            4 => ElevationLevel::System,
            _ => ElevationLevel::None,
        }
    }

    pub fn with_elevation(self, ev: ElevationLevel) -> Self {
        self.difference(Self::ELEVATION) | Self::from_bits_truncate(((ev as u8) as i32) << 6)
    }

    pub fn premium_level(self) -> u8 {
        ((self & Self::PREMIUM).bits() >> 9) as u8
    }

    pub fn extra_storage_tier(self) -> u8 {
        ((self & Self::EXTRA_STORAGE).bits() >> 13) as u8
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct DateOfBirth {
    pub year: i32,
    pub month: u8,
    pub day: u8,
}

impl From<time::Date> for DateOfBirth {
    fn from(d: time::Date) -> Self {
        let (year, month, day) = d.to_calendar_date();
        DateOfBirth {
            year,
            month: month as u8,
            day,
        }
    }
}

impl TryFrom<DateOfBirth> for time::Date {
    type Error = time::error::ComponentRange;

    fn try_from(d: DateOfBirth) -> Result<Self, Self::Error> {
        time::Date::from_calendar_date(d.year, time::Month::try_from(d.month)?, d.day)
    }
}

bitflags::bitflags! {
    pub struct UserProfileBits: i32 {
        const AVATAR_ROUNDNESS = 0x7F; // 127, lower 7 bits
        const OVERRIDE_COLOR = 0x80; // 8th bit
        const COLOR = 0xFF_FF_FF_00u32 as i32; // top 24 bits
    }
}

serde_shims::impl_serde_for_bitflags!(UserProfileBits);
impl_schema_for_bitflags!(UserProfileBits);
impl_sql_for_bitflags!(UserProfileBits);

impl Default for UserProfileBits {
    fn default() -> Self {
        UserProfileBits::empty()
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct UserProfile {
    pub bits: UserProfileBits,

    #[serde(default, skip_serializing_if = "Nullable::is_undefined")]
    pub avatar: Nullable<SmolStr>,

    #[serde(default, skip_serializing_if = "Nullable::is_undefined")]
    pub banner: Nullable<SmolStr>,

    #[serde(default, skip_serializing_if = "Nullable::is_undefined")]
    pub status: Nullable<SmolStr>,

    #[serde(default, skip_serializing_if = "Nullable::is_undefined")]
    pub bio: Nullable<SmolStr>,
}

impl UserProfile {
    pub fn roundedness(&self) -> f32 {
        (self.bits & UserProfileBits::AVATAR_ROUNDNESS).bits() as f32 / 127.0
    }

    pub fn override_color(&self) -> bool {
        self.bits.contains(UserProfileBits::OVERRIDE_COLOR)
    }

    pub fn color(&self) -> u32 {
        self.bits.bits() as u32 >> 8
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct User {
    pub id: Snowflake,
    pub username: SmolStr,

    /// Unsigned 16-bit integer
    pub discriminator: i32,
    pub flags: UserFlags,

    #[serde(default, skip_serializing_if = "Nullable::is_undefined")]
    pub profile: Nullable<UserProfile>,

    /// Not present when user isn't self
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub email: Option<SmolStr>,

    /// Not present when user isn't self
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preferences: Option<UserPreferences>,
}

bitflags::bitflags! {
    pub struct FriendFlags: i16 {
        /// Pins the user to the top of their friendlist
        const FAVORITE = 1 << 0;
    }
}

serde_shims::impl_serde_for_bitflags!(FriendFlags);
impl_schema_for_bitflags!(FriendFlags);
impl_sql_for_bitflags!(FriendFlags);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Friend {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<SmolStr>,
    pub flags: FriendFlags,
    pub user: User,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_elevation_flags() {
        let f = UserFlags::ELEVATION_3;
        assert_eq!(f.elevation(), ElevationLevel::System);

        for &ev in &[
            ElevationLevel::None,
            ElevationLevel::Bot,
            ElevationLevel::Staff,
            ElevationLevel::System,
        ] {
            assert_eq!(UserFlags::empty().with_elevation(ev).elevation(), ev);
            assert_eq!(UserFlags::all().with_elevation(ev).elevation(), ev);
        }

        println!("SYSTEM {}", f.bits());
        println!("BOT: {}", f.with_elevation(ElevationLevel::Bot).bits());
    }
}
