use super::*;

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

bitflags::bitflags! {
    /// Permissions that make sense with party-wide roles
    pub struct Permissions: u128 {
        const DEFAULT = 0
            | Self::CHANGE_NICKNAME.bits
            | Self::VIEW_ROOM.bits
            | Self::READ_MESSAGE_HISTORY.bits
            | Self::SEND_MESSAGES.bits
            | Self::USE_EXTERNAL_EMOTES.bits
            | Self::ADD_REACTIONS.bits
            | Self::EMBED_LINKS.bits
            | Self::ATTACH_FILES.bits
            | Self::SEND_TTS_MESSAGES.bits
            | Self::CONNECT.bits
            | Self::SPEAK.bits;

        const ADMINISTRATOR         = 1 << 0;
        const CREATE_INVITE         = 1 << 1;
        const KICK_MEMBERS          = 1 << 2;
        const BAN_MEMBERS           = 1 << 3;
        const VIEW_AUDIT_LOG        = 1 << 4;
        const VIEW_STATISTICS       = 1 << 5;
        const MANAGE_PARTY          = 1 << 6;
        const MANAGE_ROOMS          = 1 << 7;
        const MANAGE_NICKNAMES      = 1 << 8;
        const MANAGE_ROLES          = 1 << 9;
        const MANAGE_WEBHOOKS       = 1 << 10;
        const MANAGE_EMOJIS         = 1 << 11;
        const MOVE_MEMBERS          = 1 << 12;
        const CHANGE_NICKNAME       = 1 << 13;
        const MANAGE_PERMS          = 1 << 14;

        const VIEW_ROOM             = 1 << 30;
        const READ_MESSAGE_HISTORY  = 1 << 31 | Self::VIEW_ROOM.bits;
        const SEND_MESSAGES         = 1 << 32 | Self::VIEW_ROOM.bits;
        const MANAGE_MESSAGES       = 1 << 33;
        const MUTE_MEMBERS          = 1 << 34;
        const DEAFEN_MEMBERS        = 1 << 35;
        const MENTION_EVERYONE      = 1 << 36;
        const USE_EXTERNAL_EMOTES   = 1 << 37;
        const ADD_REACTIONS         = 1 << 38;
        const EMBED_LINKS           = 1 << 39;
        const ATTACH_FILES          = 1 << 40;
        const USE_SLASH_COMMANDS    = 1 << 41;
        const SEND_TTS_MESSAGES     = 1 << 42;
        /// Allows a user to add new attachments to
        /// existing messages using the "edit" API
        const EDIT_NEW_ATTACHMENT   = 1 << 43;

        /// Allows a user to broadcast a stream to this room
        const STREAM                = 1 << 60;
        /// Allows a user to connect and watch/listen to streams in a room
        const CONNECT               = 1 << 61;
        /// Allows a user to speak in a room without broadcasting a stream
        const SPEAK                 = 1 << 62;
        /// Allows a user to acquire priority speaker
        const PRIORITY_SPEAKER      = 1 << 63;
    }
}

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

            impl<'de> de::Visitor<'de> for PermissionsVisitor {
                type Value = Permissions;

                fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
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
            }
        }
    }
};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Overwrite {
    /// Role or user ID
    ///
    /// If it doesn't exist in the role list, then it's a user, simple as that
    pub id: Snowflake,

    #[serde(default, skip_serializing_if = "Permissions::is_empty")]
    pub allow: Permissions,
    #[serde(default, skip_serializing_if = "Permissions::is_empty")]
    pub deny: Permissions,
}

use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not};

impl Overwrite {
    #[inline]
    pub fn combine(&self, other: Self) -> Overwrite {
        Overwrite {
            id: self.id,
            allow: self.allow | other.allow,
            deny: self.deny | other.deny,
        }
    }

    #[inline]
    pub fn apply(&self, base: Permissions) -> Permissions {
        (base & !self.deny) | self.allow
    }
}

impl Permissions {
    #[inline(always)]
    pub fn from_i64(low: i64, high: i64) -> Self {
        unsafe { std::mem::transmute([low, high]) }
    }

    #[inline(always)]
    pub fn to_i64(self) -> [i64; 2] {
        unsafe { std::mem::transmute(self) }
    }

    pub fn compute_overwrites(mut self, overwrites: &[Overwrite], roles: &[Snowflake], user_id: Snowflake) -> Permissions {
        if self.contains(Permissions::ADMINISTRATOR) {
            return Permissions::all();
        }

        let mut allow = Permissions::empty();
        let mut deny = Permissions::empty();

        let mut user_overwrite = None;

        // overwrites are always sorted role-first
        for overwrite in overwrites {
            if roles.contains(&overwrite.id) {
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
}
