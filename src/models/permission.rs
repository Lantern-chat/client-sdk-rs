use super::*;

/// Constructs a `Permissions` set from a list of permissions by name.
///
/// Can be used in `const` contexts.
///
/// # Example
/// ```
/// # fn main() { use client_sdk::perms;
/// let perms = perms!(MANAGE_ROOMS | MANAGE_ROLES);
/// # }
/// ```
#[macro_export]
macro_rules! perms {
    () => { $crate::models::Permissions::empty() };

    ($perm:ident $(| $rperm:ident)*) => {{
        use $crate::models::permission::Permissions;

        // enforce const
        const VALUE: Permissions = Permissions::$perm$(.union(Permissions::$rperm))*;

        VALUE
    }}
}

bitflags2! {
    /// Party/Room Permissions
    ///
    /// This type is 16-byte aligned to ensure consistent alignment
    /// of the inner `u128`` across all platforms.
    #[repr(C, align(16))]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct Permissions: u128 {
        const ADMINISTRATOR         = 1u128 << 0;
        const CREATE_INVITE         = 1u128 << 1;
        const KICK_MEMBERS          = 1u128 << 2;
        const BAN_MEMBERS           = 1u128 << 3;
        const VIEW_AUDIT_LOG        = 1u128 << 4;
        const VIEW_STATISTICS       = 1u128 << 5;
        const MANAGE_PARTY          = 1u128 << 6;
        const MANAGE_ROOMS          = 1u128 << 7;
        const MANAGE_NICKNAMES      = 1u128 << 8;
        const MANAGE_ROLES          = 1u128 << 9;
        const MANAGE_WEBHOOKS       = 1u128 << 10;
        /// Allows members to add or remove custom emoji, stickers or sounds.
        const MANAGE_EXPRESSIONS    = 1u128 << 11;
        const MOVE_MEMBERS          = 1u128 << 12;
        const CHANGE_NICKNAME       = 1u128 << 13;
        const MANAGE_PERMS          = 1u128 << 14;

        const DEFAULT_ONLY          = 1u128 << 20;

        const VIEW_ROOM             = 1u128 << 30;
        const READ_MESSAGE_HISTORY  = 1u128 << 31 | Self::VIEW_ROOM.bits();
        const SEND_MESSAGES         = 1u128 << 32 | Self::VIEW_ROOM.bits();
        const MANAGE_MESSAGES       = 1u128 << 33;
        const MUTE_MEMBERS          = 1u128 << 34;
        const DEAFEN_MEMBERS        = 1u128 << 35;
        const MENTION_EVERYONE      = 1u128 << 36;
        const USE_EXTERNAL_EMOTES   = 1u128 << 37;
        const ADD_REACTIONS         = 1u128 << 38;
        const EMBED_LINKS           = 1u128 << 39;
        const ATTACH_FILES          = 1u128 << 40;
        const USE_SLASH_COMMANDS    = 1u128 << 41;
        const SEND_TTS_MESSAGES     = 1u128 << 42;
        /// Allows a user to add new attachments to
        /// existing messages using the "edit" API
        const EDIT_NEW_ATTACHMENT   = 1u128 << 43;

        /// Allows a user to broadcast a stream to this room
        const STREAM                = 1u128 << 60;
        /// Allows a user to connect and watch/listen to streams in a room
        const CONNECT               = 1u128 << 61;
        /// Allows a user to speak in a room without broadcasting a stream
        const SPEAK                 = 1u128 << 62;
        /// Allows a user to acquire priority speaker
        const PRIORITY_SPEAKER      = 1u128 << 63;

        /// Just something to fit in the top half for now during tests
        const TEST                  = 1u128 << 127;

        // place aggregate permissions last for iterator reasons
        const DEFAULT = 0
            | Self::CHANGE_NICKNAME.bits()
            | Self::VIEW_ROOM.bits()
            | Self::READ_MESSAGE_HISTORY.bits()
            | Self::SEND_MESSAGES.bits()
            | Self::USE_EXTERNAL_EMOTES.bits()
            | Self::ADD_REACTIONS.bits()
            | Self::EMBED_LINKS.bits()
            | Self::ATTACH_FILES.bits()
            | Self::SEND_TTS_MESSAGES.bits()
            | Self::CONNECT.bits()
            | Self::SPEAK.bits();
    }
}

impl_rkyv_for_bitflags!(pub Permissions: u128);
impl_schema_for_bitflags!(Permissions);

impl Default for Permissions {
    fn default() -> Self {
        Self::DEFAULT
    }
}

const _: () = {
    use serde::de::{self, Deserialize, Deserializer};
    use serde::ser::{Serialize, Serializer};

    impl Serialize for Permissions {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            if serializer.is_human_readable() {
                serializer.serialize_str(itoa::Buffer::new().format(self.bits()))
            } else {
                self.bits().serialize(serializer)
            }
        }
    }

    impl<'de> Deserialize<'de> for Permissions {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            return deserializer.deserialize_any(PermissionsVisitor);

            struct PermissionsVisitor;

            impl de::Visitor<'_> for PermissionsVisitor {
                type Value = Permissions;

                fn expecting(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                    f.write_str("128-bit integer or numeric string")
                }

                fn visit_u128<E>(self, v: u128) -> Result<Self::Value, E>
                where
                    E: de::Error,
                {
                    Ok(Permissions::from_bits_truncate(v))
                }

                fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
                where
                    E: de::Error,
                {
                    Ok(Permissions::from_bits_truncate(v as _))
                }

                fn visit_i128<E>(self, v: i128) -> Result<Self::Value, E>
                where
                    E: de::Error,
                {
                    Ok(Permissions::from_bits_truncate(v as _))
                }

                fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
                where
                    E: de::Error,
                {
                    Ok(Permissions::from_bits_truncate(v as _))
                }

                fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
                where
                    E: de::Error,
                {
                    match v.parse() {
                        Ok(bits) => Ok(Permissions::from_bits_truncate(bits)),
                        Err(e) => Err(E::custom(e)),
                    }
                }
            }
        }
    }
};

/// Permissions Overwrite for a role or user in a room.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "ts", derive(ts_bindgen::TypeScriptDef))]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize),
    rkyv(compare(PartialEq))
)]
pub struct Overwrite {
    /// Role or User ID
    ///
    /// If it doesn't exist in the role list, then it's a user, simple as that.
    pub id: Snowflake,

    /// Permissions to allow.
    #[serde(default, skip_serializing_if = "Permissions::is_empty")]
    pub allow: Permissions,

    /// Permissions to deny.
    #[serde(default, skip_serializing_if = "Permissions::is_empty")]
    pub deny: Permissions,
}

impl Overwrite {
    /// Takes the Union of two overwrites, assuming the same ID.
    ///
    /// # Panics
    ///
    /// With debug assertions enabled, this function will panic if the IDs do not match.
    #[inline]
    #[must_use]
    pub const fn combine(&self, other: Self) -> Overwrite {
        //debug_assert_eq!(self.id, other.id);
        #[cfg(debug_assertions)]
        if self.id.to_u64() != other.id.to_u64() {
            panic!("Overwrite IDs do not match");
        }

        Overwrite {
            id: self.id,
            allow: self.allow.union(other.allow),
            deny: self.deny.union(other.deny),
        }
    }

    /// Applies the overwrite to a base set of permissions.
    ///
    /// Equivalent to `(base & !deny) | allow`.
    #[inline]
    #[must_use]
    pub const fn apply(&self, base: Permissions) -> Permissions {
        // self.allow(base & !self.deny) | self.allow
        base.difference(self.deny).union(self.allow)
    }
}

impl Permissions {
    /// Constructs a new `Permissions` from two `i64` values.
    #[inline(always)]
    #[must_use]
    pub const fn from_i64(low: i64, high: i64) -> Self {
        Permissions::from_bits_truncate(low as u64 as u128 | ((high as u64 as u128) << 64))
    }

    /// Constructs a new `Permissions` from two `Option<i64>` values, defaulting to `0` if `None` on either.
    #[inline(always)]
    #[must_use]
    pub const fn from_i64_opt(low: Option<i64>, high: Option<i64>) -> Self {
        // TODO: Replace with `.unwrap_or(0)` when that's const-stable
        Permissions::from_i64(
            match low {
                Some(low) => low,
                None => 0,
            },
            match high {
                Some(high) => high,
                None => 0,
            },
        )
    }

    /// Converts the `Permissions` into two `i64` values.
    #[inline(always)]
    #[must_use]
    pub const fn to_i64(self) -> [i64; 2] {
        let bits = self.bits();
        let low = bits as u64 as i64;
        let high = (bits >> 64) as u64 as i64;
        [low, high]
    }

    /// Returns `true` if the permissions contain the `ADMINISTRATOR` permission.
    #[inline(always)]
    #[must_use]
    pub const fn is_admin(self) -> bool {
        self.contains(Permissions::ADMINISTRATOR)
    }

    /// Takes cerrtain flags into account and normalizes the permissions to obey them.
    #[must_use]
    pub const fn normalize(self) -> Self {
        if self.contains(Permissions::DEFAULT_ONLY) {
            return Permissions::DEFAULT;
        }

        if self.contains(Permissions::ADMINISTRATOR) {
            return Permissions::all();
        }

        self
    }

    /// Computes the final permissions for a user in a room given the overwrites and roles.
    #[must_use]
    pub fn compute_overwrites(mut self, overwrites: &[Overwrite], user_roles: &[RoleId], user_id: UserId) -> Permissions {
        if self.contains(Permissions::ADMINISTRATOR) {
            return Permissions::all();
        }

        if self.contains(Permissions::DEFAULT_ONLY) {
            self = Permissions::DEFAULT;
        }

        let mut allow = Permissions::empty();
        let mut deny = Permissions::empty();

        let mut user_overwrite = None;

        // overwrites are always sorted role-first
        for overwrite in overwrites {
            if user_roles.contains(&overwrite.id) {
                deny |= overwrite.deny;
                allow |= overwrite.allow;
            } else if overwrite.id == user_id {
                user_overwrite = Some((overwrite.deny, overwrite.allow));
                break;
            }
        }

        self &= !deny;
        self |= allow;

        if let Some((user_deny, user_allow)) = user_overwrite {
            self &= !user_deny;
            self |= user_allow;
        }

        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_print_admin() {
        println!("{:?}", Permissions::default().to_i64());
        println!("{:?}", Permissions::ADMINISTRATOR.to_i64());
    }

    #[test]
    fn test_perm_cast() {
        let [low, high] = Permissions::all().to_i64();
        assert_eq!(Permissions::from_i64(low, high), Permissions::all());
    }
}
