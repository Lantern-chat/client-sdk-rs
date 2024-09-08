use super::*;

enum_codes! {
    #[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash, serde_repr::Serialize_repr, serde_repr::Deserialize_repr)]
    #[allow(non_camel_case_types)]
    pub enum Locale: u16 = enUS {
        #[default]
        0 = enUS,
    }
}

enum_codes! {
    #[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash, serde_repr::Serialize_repr, serde_repr::Deserialize_repr)]
    pub enum Font: u16 = SansSerif {
        #[default]
        0 = SansSerif,
        1 = Serif,
        2 = Monospace,
        3 = Cursive,
        4 = ComicSans,

        // third-party fonts
        30 = OpenDyslexic,

        31 = AtkinsonHyperlegible,
    }
}

enum_codes! {
    #[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash, serde_repr::Serialize_repr, serde_repr::Deserialize_repr)]
    pub enum FriendAddability: u8 = None {
        #[default]
        0  = None,
        10 = FriendsOfFriends,
        20 = ServerMembers,
        30 = Anyone,
    }
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
            | Self::ALLOW_DMS.bits()
            | Self::GROUP_LINES.bits()
            | Self::ENABLE_SPELLCHECK.bits()
            | Self::SHOW_MEDIA_METADATA.bits()
            | Self::SHOW_DATE_CHANGE.bits()
            | Self::SHOW_GREY_IMAGE_BG.bits()
            | Self::SHOW_ATTACHMENT_GRID.bits();
    }
}

impl_rkyv_for_bitflags!(pub UserPrefsFlags: i32);
impl_serde_for_bitflags!(UserPrefsFlags);
impl_sql_for_bitflags!(UserPrefsFlags);

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

pub mod preferences {
    decl_newtype_prefs! {
        Temperature: u16 = 7500u16,
        FontSize: f32 = 16.0f32,
        TabSize: u8 = 4u8,
        Padding: u8 = 16u8,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "rkyv", derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize))]
pub struct UserPreferences {
    #[serde(default, skip_serializing_if = "is_default", alias = "locale")]
    pub l: Locale,
    #[serde(default, skip_serializing_if = "is_default", alias = "flags")]
    pub f: UserPrefsFlags,
    #[serde(default, skip_serializing_if = "is_default", alias = "friend_add")]
    pub friend: FriendAddability,
    #[serde(default, skip_serializing_if = "is_default", alias = "temperature")]
    pub temp: preferences::Temperature,
    #[serde(default, skip_serializing_if = "is_default", alias = "chat_font")]
    pub cf: Font,
    #[serde(default, skip_serializing_if = "is_default", alias = "ui_font")]
    pub uf: Font,
    #[serde(default, skip_serializing_if = "is_default", alias = "chat_font_size")]
    pub cfs: preferences::FontSize,
    #[serde(default, skip_serializing_if = "is_default", alias = "ui_font_size")]
    pub ufs: preferences::FontSize,
    #[serde(default, skip_serializing_if = "is_default", alias = "padding")]
    pub pad: preferences::Padding,
    #[serde(default, skip_serializing_if = "is_default", alias = "tab_size")]
    pub tab: preferences::TabSize,
}

impl UserPreferences {
    pub fn clean(&mut self) {
        use core::ops::Range;

        #[inline]
        fn clamp_range<T: PartialOrd>(value: &mut T, range: Range<T>) {
            if *value < range.start {
                *value = range.start;
            } else if *value > range.end {
                *value = range.end;
            }
        }

        #[inline]
        fn round_2(value: &mut f32) {
            *value = (*value * 100.0).round() / 100.0;
        }

        round_2(&mut self.cfs);
        round_2(&mut self.ufs);

        clamp_range::<u16>(&mut self.temp, 965..12000);
        clamp_range::<f32>(&mut self.cfs, 8.0..32.0);
        clamp_range::<f32>(&mut self.ufs, 8.0..32.0);
        clamp_range::<u8>(&mut self.pad, 0..32);
        clamp_range::<u8>(&mut self.tab, 1..64);
    }
}
