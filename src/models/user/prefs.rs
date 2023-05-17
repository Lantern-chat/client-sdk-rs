use std::collections::HashMap;
use std::fmt;

use serde_json::Value;

use super::*;

#[derive(Default, Debug, Clone, Copy, Hash, serde_repr::Serialize_repr, serde_repr::Deserialize_repr)]
#[allow(non_camel_case_types)]
#[repr(u16)]
pub enum Locale {
    #[default]
    enUS = 0,

    __MAX_LOCALE,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde_repr::Serialize_repr, serde_repr::Deserialize_repr)]
#[repr(u16)]
#[allow(non_camel_case_types)]
pub enum Font {
    SansSerif = 0,
    Serif = 1,
    Monospace = 2,
    Cursive = 3,
    ComicSans = 4,

    // third-party fonts
    OpenDyslexic = 30,

    AtkinsonHyperlegible = 31,

    __MAX_FONT,
}

bitflags::bitflags! {
    pub struct UserPrefsFlags: i32 {
        /// Reduce movement and animations in the UI
        const REDUCE_ANIMATIONS                 = 1 << 0;
        /// Pause animations on window unfocus
        const UNFOCUS_PAUSE                     = 1 << 1;
        const LIGHT_MODE                        = 1 << 2;

        /// Allow direct messages from shared server memmbers
        const ALLOW_DMS                         = 1 << 3;
        /// Show small lines between message groups
        const GROUP_LINES                       = 1 << 4;
        const HIDE_AVATARS                      = 1 << 5;

        /// Display dark theme in an OLED-compatible mode
        const OLED_MODE                         = 1 << 6;

        /// Mute videos/audio by default
        const MUTE_MEDIA                        = 1 << 7;

        /// Hide images/video with unknown dimensions
        const HIDE_UNKNOWN_DIMENSIONS           = 1 << 8;

        const COMPACT_VIEW                      = 1 << 9;

        /// Prefer browser/platform emojis rather than twemoji
        const USE_PLATFORM_EMOJIS               = 1 << 10;
        const ENABLE_SPELLCHECK                 = 1 << 11;
        const LOW_BANDWIDTH_MODE                = 1 << 12;
        const FORCE_COLOR_CONSTRAST             = 1 << 13;

        /// Displays information like mime type and file size
        const SHOW_MEDIA_METADATA               = 1 << 14;
        const DEVELOPER_MODE                    = 1 << 15;
        const SHOW_DATE_CHANGE                  = 1 << 16;

        const HIDE_LAST_ACTIVE                  = 1 << 17;

        /// Show grey background color for images
        /// (helps keep transparent pixels consistent)
        const SHOW_GREY_IMAGE_BG                = 1 << 18;

        /// When multiple attachments are present, condense them
        /// into a grid to avoid cluttering the channel
        const SHOW_ATTACHMENT_GRID              = 1 << 19;

        const SMALLER_ATTACHMENTS               = 1 << 20;

        const HIDE_ALL_EMBEDS                   = 1 << 21;
        const HIDE_NSFW_EMBEDS                  = 1 << 22;

        const DEFAULT_FLAGS = 0
            | Self::ALLOW_DMS.bits
            | Self::GROUP_LINES.bits
            | Self::ENABLE_SPELLCHECK.bits
            | Self::SHOW_MEDIA_METADATA.bits
            | Self::SHOW_DATE_CHANGE.bits
            | Self::SHOW_GREY_IMAGE_BG.bits
            | Self::SHOW_ATTACHMENT_GRID.bits;
    }
}

serde_shims::impl_serde_for_bitflags!(UserPrefsFlags);
common::impl_sql_for_bitflags!(UserPrefsFlags);

impl From<u64> for UserPrefsFlags {
    fn from(value: u64) -> Self {
        UserPrefsFlags::from_bits_truncate(value as _)
    }
}

impl Default for UserPrefsFlags {
    fn default() -> Self {
        Self::DEFAULT_FLAGS
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Hash, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum UserPreference {
    Locale,

    Flags,

    /*
        PRIVACY
    */
    /// Who can add you as a friend,
    /// number 0-3 where 0 = no one, 1 = friends of friends, 2 = server members, 3 = anyone
    FriendAdd,

    /*
        ACCESSIBILITY
    */

    /*
        APPEARANCE
    */
    /// Color temperature
    Temp,
    /// Chat font
    ChatFont,
    /// UI Font
    UiFont,
    /// Font size
    ChatFontSize,
    /// UI Font Size
    UiFontSize,
    /// Message Tab Size (in spaces)
    TabSize,
    /// Time format
    TimeFormat,
    /// Group padding
    Pad,

    /*
        Advanced
    */
    #[serde(other)]
    InvalidField,
}

impl fmt::Display for UserPreference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use serde::Serialize;
        self.serialize(f)
    }
}

use crate::util::prefs::*;

pub type UserPreferences = PreferenceMap<UserPreference, Hasher>;
pub type UserPreferenceError = PreferenceError<UserPreference>;

impl Preference for UserPreference {
    type Flags = UserPrefsFlags;

    const FLAGS_KEY: Self = UserPreference::Flags;

    fn validate(&self, value: &Value) -> Result<(), UserPreferenceError> {
        let mut kind = PreferenceErrorKind::InvalidType;

        let valid_type = match *self {
            // NULL values are not allowed
            _ if value.is_null() => false,

            Self::InvalidField => false,

            // The locale just has to be in the list of enums, and since
            // they are numbered it's easy to check
            Self::Locale => match value.as_u64() {
                Some(value) => {
                    kind = PreferenceErrorKind::InvalidValue;
                    value < Locale::__MAX_LOCALE as u64
                }
                _ => false,
            },
            // Check docs for this, but values can only be from 0-3 inclusive
            Self::FriendAdd => match value.as_u64() {
                Some(value) => {
                    kind = PreferenceErrorKind::InvalidValue;
                    value <= 3
                }
                _ => false,
            },
            Self::Flags => match value.as_u64() {
                Some(value) => {
                    kind = PreferenceErrorKind::InvalidValue;

                    // contained within 2^32 AND a valid flag
                    value <= (u32::MAX as u64) && UserPrefsFlags::from_bits(value as i32).is_some()
                }
                _ => false,
            },
            // Color temperature in kelvin degrees
            Self::Temp => match value.as_f64() {
                Some(temp) => {
                    kind = PreferenceErrorKind::InvalidValue;
                    (965.0..=12000.0).contains(&temp)
                }
                _ => false,
            },
            Self::TimeFormat => match value {
                // TODO: Properly validate format string
                Value::String(_format) => true,
                Value::Bool(_) => true,
                _ => false,
            },
            // Fonts must be in the list, which is easily checked by parsing the enum
            Self::ChatFont | Self::UiFont => match value.as_u64() {
                Some(value) => {
                    kind = PreferenceErrorKind::InvalidValue;
                    value < Font::__MAX_FONT as u64
                }
                _ => false,
            },
            Self::TabSize => match value.as_u64() {
                Some(value) => {
                    kind = PreferenceErrorKind::InvalidValue;
                    value > 0 && value < 64
                }
                _ => false,
            },
            // Font sizes can be floats for smooth scaling, but must be positive
            Self::ChatFontSize | Self::UiFontSize => match value.as_u64() {
                Some(value) => {
                    kind = PreferenceErrorKind::InvalidValue;
                    (8..=32).contains(&value)
                }
                _ => false,
            },
            Self::Pad => match value.as_u64() {
                Some(value) => {
                    kind = PreferenceErrorKind::InvalidValue;
                    value <= 32
                }
                _ => false,
            },
        };

        if !valid_type {
            Err(PreferenceError { field: *self, kind })
        } else {
            Ok(())
        }
    }

    fn is_default(&self, value: &Value, flags: UserPrefsFlags) -> bool {
        match *self {
            Self::Flags => value.as_u64() == Some(UserPrefsFlags::DEFAULT_FLAGS.bits() as u64),
            Self::ChatFontSize | Self::UiFontSize => value.as_u64() == Some(16),
            Self::Temp => value.as_f64() == Some(7500.0),
            Self::FriendAdd => value.as_u64() == Some(3),
            Self::Locale => value.as_u64() == Some(Locale::enUS as u64),
            Self::ChatFont | Self::UiFont => value.as_u64() == Some(0),
            Self::TabSize => value.as_u64() == Some(4),
            Self::Pad => {
                let value = value.as_u64();

                if flags.contains(UserPrefsFlags::COMPACT_VIEW) {
                    value == Some(0)
                } else {
                    value == Some(16)
                }
            }
            _ => false,
        }
    }
}
