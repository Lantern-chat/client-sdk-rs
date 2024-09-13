bitflags::bitflags! {
    /// NOTE: Formats are as individual bitflags (rather than some integer value) so we can do
    /// simpler queries when matching valid formats. A user can select formats A, B and C, and testing for a match
    /// can be done with a single bitwise-AND operation, rather than many comparisons or an `IN ARRAY` operation.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct AssetFlags: i16 {
        /// 7-bit unsigned integer for quality from `[0-128)`
        ///
        /// A quality value greater then 100 indicates some lossless encoding
        const QUALITY  = 0x7F;

        /// Indicates if the encoded image has an alpha channel
        const HAS_ALPHA = 1 << 8;

        /// Indicates if the encoded image is animated
        const ANIMATED = 1 << 9;

        /// PNG format
        const FORMAT_PNG  = 1 << 10;
        /// JPEG format
        const FORMAT_JPEG = 1 << 11;
        /// GIF format
        const FORMAT_GIF  = 1 << 12;
        /// AVIF format
        const FORMAT_AVIF = 1 << 13;
        /// WebM format
        const FORMAT_WEBM = 1 << 14;
        /// Jpeg XL format
        const FORMAT_JXL  = 1 << 15;

        /// All supported formats
        const FORMATS = 0
            | Self::FORMAT_PNG.bits()
            | Self::FORMAT_JPEG.bits()
            | Self::FORMAT_GIF.bits()
            | Self::FORMAT_AVIF.bits()
            | Self::FORMAT_WEBM.bits()
            | Self::FORMAT_JXL.bits();

        /// These formats don't have widespread support yet, so don't include them by default
        const MAYBE_UNSUPPORTED_FORMATS = 0
            | Self::FORMAT_AVIF.bits()
            | Self::FORMAT_JXL.bits();

        /// All qualifier flags
        const FLAGS = 0
            | Self::HAS_ALPHA.bits()
            | Self::ANIMATED.bits();

        /// All formats and flags
        const FORMATS_AND_FLAGS = 0
            | Self::FORMATS.bits()
            | Self::FLAGS.bits();
    }
}

impl_rkyv_for_bitflags!(pub AssetFlags: i16);
impl_serde_for_bitflags!(AssetFlags);
impl_schema_for_bitflags!(AssetFlags);
impl_sql_for_bitflags!(AssetFlags);

impl AssetFlags {
    /// Sets the quality of the asset, clamping to `[0-128)`
    #[must_use]
    pub const fn with_quality(self, q: u8) -> Self {
        self.intersection(Self::QUALITY.complement()).union(if q < 128 {
            AssetFlags::from_bits_truncate(q as i16)
        } else {
            AssetFlags::QUALITY
        })
    }

    /// Sets the alpha channel flag
    #[must_use]
    pub const fn with_alpha(&self, has_alpha: bool) -> Self {
        if has_alpha {
            self.union(Self::HAS_ALPHA)
        } else {
            self.difference(Self::HAS_ALPHA)
        }
    }

    /// Gets the quality value from the asset flags
    #[must_use]
    pub const fn quality(&self) -> u8 {
        self.intersection(Self::QUALITY).bits() as u8
    }

    /// Constructs a new `AssetFlags` from the given extension
    #[must_use]
    pub fn from_ext(ext: &str) -> Self {
        static FORMAT_EXTS: &[(AssetFlags, &str)] = &[
            (AssetFlags::FORMAT_PNG, "png"),
            (AssetFlags::FORMAT_JPEG, "jpeg"),
            (AssetFlags::FORMAT_JPEG, "jpg"),
            (AssetFlags::FORMAT_GIF, "gif"),
            (AssetFlags::FORMAT_AVIF, "avif"),
        ];

        if (3..5).contains(&ext.len()) {
            for &(f, e) in FORMAT_EXTS {
                if ext.eq_ignore_ascii_case(e) {
                    return f;
                }
            }
        }

        AssetFlags::empty()
    }
}
