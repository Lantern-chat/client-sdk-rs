#![allow(async_fn_in_trait)]

use super::*;

bitflags2! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct Intent: u32 where "gateway" {
        /// - PARTY_CREATE
        /// - PARTY_UPDATE
        /// - PARTY_DELETE
        /// - PARTY_ROLE_CREATE
        /// - PARTY_ROLE_UPDATE
        /// - PARTY_ROLE_DELETE
        /// - CHANNEL_CREATE
        /// - CHANNEL_UPDATE
        /// - CHANNEL_DELETE
        /// - CHANNEL_PINS_UPDATE
        const PARTIES                   = 1 << 0;

        /// - PARTY_MEMBER_ADD
        /// - PARTY_MEMBER_UPDATE
        /// - PARTY_MEMBER_REMOVE
        const PARTY_MEMBERS             = 1 << 1;

        /// - PARTY_BAN_ADD
        /// - PARTY_BAN_REMOVE
        const PARTY_BANS                = 1 << 2;

        /// - PARTY_EMOJIS_UPDATE
        const PARTY_EMOTES              = 1 << 3;

        /// - PARTY_INTEGRATIONS_UPDATE
        /// - INTEGRATION_CREATE
        /// - INTEGRATION_UPDATE
        /// - INTEGRATION_DELETE
        const PARTY_INTEGRATIONS        = 1 << 4;

        /// - WEBHOOKS_UPDATE
        const PARTY_WEBHOOKS            = 1 << 5;

        /// - INVITE_CREATE
        /// - INVITE_DELETE
        const PARTY_INVITES             = 1 << 6;

        /// - VOICE_STATE_UPDATE
        const VOICE_STATUS              = 1 << 7;

        /// - PRESENCE_UPDATE
        const PRESENCE                  = 1 << 8;

        /// - MESSAGE_CREATE
        /// - MESSAGE_UPDATE
        /// - MESSAGE_DELETE
        /// - MESSAGE_DELETE_BULK
        const MESSAGES                  = 1 << 9;

        /// - MESSAGE_REACTION_ADD
        /// - MESSAGE_REACTION_REMOVE
        /// - MESSAGE_REACTION_REMOVE_ALL
        /// - MESSAGE_REACTION_REMOVE_EMOTE
        const MESSAGE_REACTIONS         = 1 << 10;

        /// - TYPING_START
        const MESSAGE_TYPING            = 1 << 11;

        /// - MESSAGE_CREATE
        /// - MESSAGE_UPDATE
        /// - MESSAGE_DELETE
        /// - CHANNEL_PINS_UPDATE
        const DIRECT_MESSAGES           = 1 << 12;

        /// - MESSAGE_REACTION_ADD
        /// - MESSAGE_REACTION_REMOVE
        /// - MESSAGE_REACTION_REMOVE_ALL
        /// - MESSAGE_REACTION_REMOVE_EMOTE
        const DIRECT_MESSAGE_REACTIONS  = 1 << 13;

        /// - TYPING_START
        const DIRECT_MESSAGE_TYPING     = 1 << 14;

        const PROFILE_UPDATES           = 1 << 15;
    }
}

impl_rkyv_for_bitflags!(pub Intent: u32);
impl_serde_for_bitflags!(Intent);
impl_schema_for_bitflags!(Intent);
impl_sql_for_bitflags!(Intent);

pub mod commands {
    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
    #[cfg_attr(feature = "rkyv", derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize))]
    #[cfg_attr(feature = "ts", derive(ts_bindgen::TypeScriptDef), ts(tag = "gateway"))]
    pub struct Identify {
        pub auth: AuthToken,
        pub intent: Intent,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
    #[cfg_attr(feature = "rkyv", derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize))]
    #[cfg_attr(feature = "ts", derive(ts_bindgen::TypeScriptDef), ts(tag = "gateway"))]
    pub struct SetPresence {
        #[serde(flatten)]
        pub presence: UserPresence,
    }
}

pub mod events {
    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
    #[cfg_attr(feature = "rkyv", derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize))]
    #[cfg_attr(feature = "ts", derive(ts_bindgen::TypeScriptDef), ts(tag = "gateway"))]
    pub struct Hello {
        /// Number of milliseconds between heartbeats
        pub heartbeat_interval: u32,
    }

    impl Default for Hello {
        fn default() -> Self {
            Hello {
                heartbeat_interval: 45000, // 45 seconds
            }
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
    #[cfg_attr(feature = "rkyv", derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize))]
    #[cfg_attr(feature = "ts", derive(ts_bindgen::TypeScriptDef), ts(tag = "gateway"))]
    pub struct ReadyParty {
        pub party: Party,

        /// The user's own member object
        pub me: PartyMember,
    }

    impl core::ops::Deref for ReadyParty {
        type Target = Party;

        fn deref(&self) -> &Party {
            &self.party
        }
    }

    impl core::ops::DerefMut for ReadyParty {
        fn deref_mut(&mut self) -> &mut Party {
            &mut self.party
        }
    }

    #[cfg(feature = "rkyv")]
    impl core::ops::Deref for ArchivedReadyParty {
        type Target = rkyv::Archived<Party>;

        fn deref(&self) -> &Self::Target {
            &self.party
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
    #[cfg_attr(feature = "rkyv", derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize))]
    #[cfg_attr(feature = "ts", derive(ts_bindgen::TypeScriptDef), ts(tag = "gateway"))]
    pub struct Ready {
        pub user: User,

        /// The parties the user is in, including DMs.
        pub parties: ThinVec<ReadyParty>,

        /// Contains all rooms the user is in, including DMs.
        pub rooms: ThinVec<Room>,

        /// Gateway session ID, used for resuming sessions
        pub session: Snowflake,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
    #[cfg_attr(feature = "rkyv", derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize))]
    #[cfg_attr(feature = "ts", derive(ts_bindgen::TypeScriptDef), ts(tag = "gateway"))]
    pub struct TypingStart {
        pub room_id: RoomId,
        pub party_id: PartyId,
        pub user_id: UserId,
        pub member: PartyMember,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub parent: Option<MessageId>,
        // maybe timestamp?
        //ts: u32,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
    #[cfg_attr(feature = "rkyv", derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize))]
    #[cfg_attr(feature = "ts", derive(ts_bindgen::TypeScriptDef), ts(tag = "gateway"))]
    pub struct PartyPositionUpdate {
        pub id: PartyId,
        pub position: i16,
    }

    #[derive(Debug, Serialize, Deserialize)]
    #[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
    #[cfg_attr(feature = "rkyv", derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize))]
    #[cfg_attr(feature = "ts", derive(ts_bindgen::TypeScriptDef), ts(tag = "gateway"))]
    pub struct UserPresenceEvent {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub party_id: Option<PartyId>,

        pub user: User,
    }

    #[derive(Debug, Serialize, Deserialize)]
    #[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
    #[cfg_attr(feature = "rkyv", derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize))]
    #[cfg_attr(feature = "ts", derive(ts_bindgen::TypeScriptDef), ts(tag = "gateway"))]
    pub struct MessageDeleteEvent {
        pub id: MessageId,
        pub room_id: RoomId,
        pub party_id: PartyId,
    }

    #[derive(Debug, Serialize, Deserialize)]
    #[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
    #[cfg_attr(feature = "rkyv", derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize))]
    #[cfg_attr(feature = "ts", derive(ts_bindgen::TypeScriptDef), ts(tag = "gateway"))]
    pub struct RoleDeleteEvent {
        pub id: RoleId,
        pub party_id: PartyId,
    }

    #[derive(Debug, Serialize, Deserialize)]
    #[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
    #[cfg_attr(feature = "rkyv", derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize))]
    #[cfg_attr(feature = "ts", derive(ts_bindgen::TypeScriptDef), ts(tag = "gateway"))]
    pub struct RoomDeleteEvent {
        pub id: RoomId,

        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[cfg_attr(feature = "rkyv", rkyv(with = NicheSnowflake))]
        pub party_id: Option<PartyId>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    #[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
    #[cfg_attr(feature = "rkyv", derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize))]
    #[cfg_attr(feature = "ts", derive(ts_bindgen::TypeScriptDef), ts(tag = "gateway"))]
    pub struct PartyMemberEvent {
        pub party_id: PartyId,

        #[serde(flatten)]
        pub member: PartyMember,
    }

    #[derive(Debug, Serialize, Deserialize)]
    #[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
    #[cfg_attr(feature = "rkyv", derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize))]
    #[cfg_attr(feature = "ts", derive(ts_bindgen::TypeScriptDef), ts(tag = "gateway"))]
    #[serde(untagged)]
    pub enum PartyUpdateEvent {
        Position(PartyPositionUpdate),
        Full(Party),
    }

    #[derive(Debug, Serialize, Deserialize)]
    #[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
    #[cfg_attr(feature = "rkyv", derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize))]
    #[cfg_attr(feature = "ts", derive(ts_bindgen::TypeScriptDef), ts(tag = "gateway"))]
    pub struct UserReactionEvent {
        pub user_id: UserId,
        pub room_id: RoomId,
        pub party_id: PartyId,
        pub msg_id: MessageId,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[cfg_attr(feature = "rkyv", rkyv(with = rkyv::with::Niche))]
        pub member: Option<Box<PartyMember>>,
        pub emote: EmoteOrEmoji,
    }

    #[derive(Debug, Serialize, Deserialize)]
    #[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
    #[cfg_attr(feature = "rkyv", derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize))]
    #[cfg_attr(feature = "ts", derive(ts_bindgen::TypeScriptDef), ts(tag = "gateway"))]
    pub struct ProfileUpdateEvent {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub party_id: Option<PartyId>,
        pub user: User,
    }

    //#[derive(Debug, Clone, Serialize, Deserialize)]
    //pub struct PresenceUpdate {
    //    pub user_id: UserId,
    //    pub presence: UserPresence,
    //}
}

pub mod message {
    use super::Snowflake;

    use serde_repr::{Deserialize_repr, Serialize_repr};

    use serde::de::{self, Deserialize, Deserializer, MapAccess, SeqAccess, Visitor};
    use serde::ser::{Serialize, SerializeStruct, Serializer};

    #[inline]
    fn is_default<T>(value: &T) -> bool
    where
        T: Default + Eq,
    {
        *value == T::default()
    }

    macro_rules! decl_msgs {
        (
            $(#[$meta:meta])*
            enum $name:ident {
                $(
                    $(#[$variant_meta:meta])*
                    $code:literal => $opcode:ident $(:$Default:ident)?  {
                        $( $(#[$field_meta:meta])* $field:ident $(*$Deref:ident)? : $ty:ty),*$(,)?
                    }
                ),*$(,)*
            }
        ) => {paste::paste!{
            #[doc = "OpCodes for [`" $name "`]"]
            #[derive(Debug, Clone, Copy, Serialize_repr, Deserialize_repr)]
            #[cfg_attr(feature = "schema", derive(schemars::JsonSchema_repr))]
            #[cfg_attr(feature = "ts", derive(ts_bindgen::TypeScriptDef), ts(tag = "gateway"))]
            #[repr(u8)]
            pub enum [<$name Opcode>] {
                $($opcode = $code,)*
            }

            pub mod [<$name:snake _payloads>] {
                use super::*;

                $(
                    $(#[$variant_meta])*
                    #[doc = ""]
                    #[doc = "Payload struct for [`" $name "::" $opcode "`]"]
                    #[derive(Debug, Serialize, Deserialize)]
                    #[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
                    #[cfg_attr(feature = "rkyv", derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize))]
                    #[cfg_attr(feature = "ts", derive(ts_bindgen::TypeScriptDef), ts(tag = "gateway"))]
                    $(#[derive($Default, PartialEq, Eq)])?
                    pub struct [<$opcode Payload>] {
                        $($(#[$field_meta])* pub $field : $ty,)*
                    }

                    $($(
                        impl core::ops::$Deref for [<$opcode Payload>] {
                            type Target = $ty;

                            #[inline(always)]
                            fn deref(&self) -> &Self::Target {
                                &self.$field
                            }
                        }

                        #[cfg(feature = "rkyv")]
                        impl core::ops::$Deref for [<Archived $opcode Payload>] {
                            type Target = rkyv::Archived<$ty>;

                            #[inline(always)]
                            fn deref(&self) -> &Self::Target {
                                &self.inner
                            }
                        }
                    )?)*
                )*
            }

            #[cfg(feature = "framework")]
            pub struct [<Dynamic $name Handlers>]<C, U = (), S = ()> {
                state: Arc<S>,

                fallback: Box<dyn Fn(Arc<S>, C, $name) -> BoxFuture<'static, U> + Send + Sync>,

                $(
                    [<$opcode:snake _handler>]: Option<Box<dyn Fn(Arc<S>, C, $($ty,)*) -> BoxFuture<'static, U> + Send + Sync>>,
                )*
            }

            #[cfg(feature = "framework")]
            impl<C> Default for [<Dynamic $name Handlers>]<C, (), ()> {
                fn default() -> Self {
                    Self::new(|_, _, _| async {})
                }
            }

            #[cfg(feature = "framework")]
            impl<C, U> [<Dynamic $name Handlers>]<C, U, ()> {
                pub fn new<F, R>(fallback: F) -> Self
                where
                    F: Fn(Arc<()>, C, $name) -> R + Send + Sync + 'static,
                    R: Future<Output = U> + Send + 'static
                {
                    Self::new_with_state((), fallback)
                }
            }

            #[cfg(feature = "framework")]
            impl<C, U, S> [<Dynamic $name Handlers>]<C, U, S> {
                pub fn new_with_state<F, R>(state: S, fallback: F) -> Self
                where
                    F: Fn(Arc<S>, C, $name) -> R + Send + Sync + 'static,
                    R: Future<Output = U> + Send + 'static
                {
                    Self::new_raw_with_state(state, Box::new(move |this, ctx, msg| Box::pin(fallback(this, ctx, msg)) ))
                }

                pub fn new_raw_with_state(state: impl Into<Arc<S>>, fallback: Box<dyn Fn(Arc<S>, C, $name) -> BoxFuture<'static, U> + Send + Sync>) -> Self {
                    Self {
                        state: state.into(),
                        fallback,
                        $([<$opcode:snake _handler>]: None,)*
                    }
                }

                #[inline(always)] #[must_use]
                pub fn state(&self) -> &S {
                    &self.state
                }

                $(
                    pub fn [<on_ $opcode:snake>]<F, R>(&mut self, cb: F) -> &mut Self
                    where
                        F: Fn(Arc<S>, C, $($ty,)*) -> R + Send + Sync + 'static,
                        R: Future<Output = U> + Send + 'static,
                    {
                        assert!(
                            self.[<$opcode:snake _handler>].is_none(),
                            concat!("Cannot have more than one listener for ", stringify!([<on_ $opcode:snake>]))
                        );

                        self.[<$opcode:snake _handler>] = Some(Box::new(move |this, ctx, $($field,)*| Box::pin(cb(this, ctx, $($field,)*))));
                        self
                    }
                )*
            }

            #[cfg(feature = "framework")]
            impl<C: Send + 'static, U, S> [<$name Handlers>]<C, U> for [<Dynamic $name Handlers>]<C, U, S> where S: Send + Sync {
                async fn fallback(&self, ctx: C, msg: $name) -> U {
                    (self.fallback)(self.state.clone(), ctx, msg).await
                }

                $(
                    async fn [<$opcode:snake>](&self, ctx: C, $($field: $ty,)*) -> U {
                        match self.[<$opcode:snake _handler>] {
                            Some(ref cb) => cb(self.state.clone(), ctx, $($field,)*).await,
                            None => self.fallback(ctx, $name::$opcode([<$name:snake _payloads>]::[<$opcode Payload>] { $($field,)* })).await,
                        }
                    }
                )*
            }

            #[doc = "Handler callbacks for [`" $name "`]"]
            #[cfg(feature = "framework")]
            pub trait [<$name Handlers>]<C, U = ()>: Send + Sync where C: Send + 'static {
                /// Dispatches a message to the appropriate event handler
                async fn dispatch(&self, ctx: C, msg: $name) -> U {
                    match msg {
                        $($name::$opcode([<$name:snake _payloads>]::[<$opcode Payload>] { $($field,)* }) => {
                            self.[<$opcode:snake>](ctx, $($field,)*).await
                        })*
                    }
                }

                /// Callback for unhandled messages
                async fn fallback(&self, ctx: C, msg: $name) -> U;

                $(
                    $(#[$variant_meta])*
                    #[doc = ""]
                    #[doc = "Handler callback for [`" $name "::" $opcode "`]"]
                    #[inline(always)]
                    async fn [<$opcode:snake>](&self, ctx: C, $($field: $ty,)*) -> U {
                        self.fallback(ctx, $name::$opcode([<$name:snake _payloads>]::[<$opcode Payload>] { $($field,)* })).await
                    }
                )*
            }

            $(#[$meta])*
            #[derive(Debug)]
            #[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
            #[cfg_attr(feature = "rkyv", derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize))]
            #[repr(u8)]
            pub enum $name {
                $(
                    $(#[$variant_meta])*
                    #[doc = ""]
                    #[doc = "See [`" [<new_ $opcode:snake>] "`](" $name "::" [<new_ $opcode:snake>] ") for an easy way to create this message."]
                    #[cfg_attr(feature = "schema", schemars(description = "" $name "::" $opcode "" ))]
                    $opcode([<$name:snake _payloads>]::[<$opcode Payload>]) = $code,
                )*
            }

            impl $name {
                /// Returns the discrete opcode for the message
                #[must_use]
                pub const fn opcode(&self) -> [<$name Opcode>] {
                    match self {
                        $($name::$opcode(_) => [<$name Opcode>]::$opcode,)*
                    }
                }
            }

            impl From<&$name> for [<$name Opcode>] {
                #[inline]
                fn from(msg: &$name) -> [<$name Opcode>] {
                    msg.opcode()
                }
            }

            impl $name {
                $(
                    #[doc = "Create new [`" $opcode "`](" $name "::" $opcode ") message from payload fields."]
                    #[doc = ""]
                    $(#[$variant_meta])*
                    #[inline] #[must_use]
                    pub fn [<new_ $opcode:snake>]($($field: impl Into<$ty>),*) -> Self {
                        $name::$opcode([<$name:snake _payloads>]::[<$opcode Payload>] {
                            $($field: $field.into()),*
                        })
                    }
                )*
            }

            #[cfg(feature = "ts")]
            const _: () = {
                use ts_bindgen::{TypeScriptDef, TypeScriptType, TypeRegistry};

                impl TypeScriptDef for $name {
                    #[allow(clippy::vec_init_then_push)]
                    fn register(registry: &mut TypeRegistry) -> TypeScriptType {
                        if registry.contains(stringify!($name)) {
                            return TypeScriptType::Named(stringify!($name));
                        }

                        [<$name Opcode>]::register(registry);

                        let mut variants = Vec::new();

                        $(
                            variants.push({
                                let mut members = Vec::new();

                                // o: Opcode.$opcode
                                members.push(( "o".into(), TypeScriptType::EnumValue(stringify!([<$name Opcode>]), stringify!($opcode)), "".into() ));
                                // p: Payload
                                members.push(( "p".into(), <[<$name:snake _payloads>]::[<$opcode Payload>]>::register(registry), "".into() ));

                                TypeScriptType::interface(members, 0)
                            });
                        )*

                        registry.insert(stringify!($name), TypeScriptType::Union(variants), concat!("Union of all ", stringify!($name), " messages"));

                        registry.tag(stringify!($name), "gateway");

                        TypeScriptType::Named(stringify!($name))
                    }
                }
            };

            impl Serialize for $name {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: Serializer,
                {
                    let state = match self {$(
                        $name::$opcode(payload) => {
                            let skip_payload = false $(|| [<is_ $Default:lower>](payload))?;

                            let mut state = serializer.serialize_struct(stringify!($name), 2 - skip_payload as usize)?;

                            state.serialize_field("o", &[<$name Opcode>]::$opcode)?;

                            if !skip_payload {
                                state.serialize_field("p", payload)?;
                            }

                            state
                        }
                    )*};

                    state.end()
                }
            }

            impl<'de> Deserialize<'de> for $name {
                fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: Deserializer<'de>
                {
                    use core::fmt;

                    #[derive(Clone, Copy, Deserialize)]
                    #[serde(rename_all = "lowercase")]
                    enum Field {
                        #[serde(alias = "o")]
                        Opcode,

                        #[serde(alias = "p")]
                        Payload,
                    }

                    struct MessageVisitor;

                    impl<'de> Visitor<'de> for MessageVisitor {
                        type Value = $name;

                        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                            formatter.write_str(concat!("struct ", stringify!($name)))
                        }

                        fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
                        where
                            V: MapAccess<'de>,
                        {
                            let opcode = match map.next_entry()? {
                                Some((Field::Opcode, o)) => o,
                                _ => return Err(de::Error::custom("Missing opcode first")),
                            };

                            match opcode {
                                $(
                                    [<$name Opcode>]::$opcode => Ok($name::$opcode(match map.next_entry()? {
                                        Some((Field::Payload, payload)) => payload,
                                        $(None => $Default::default(),)?

                                        #[allow(unreachable_patterns)]
                                        _ => return Err(de::Error::missing_field("payload")),
                                    })),
                                )*
                                // _ => Err(de::Error::custom("Invalid opcode")),
                            }
                        }

                        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
                        where
                            A: SeqAccess<'de>
                        {
                            let opcode: [<$name Opcode>] = match seq.next_element()? {
                                Some(o) => o,
                                _ => return Err(de::Error::custom("Missing opcode first")),
                            };

                            match opcode {
                                $(
                                    [<$name Opcode>]::$opcode => Ok($name::$opcode(match seq.next_element()? {
                                        Some(payload) => payload,
                                        $(None => $Default::default(),)?

                                        #[allow(unreachable_patterns)]
                                        _ => return Err(de::Error::missing_field("payload")),
                                    })),
                                )*
                            }
                        }
                    }

                    deserializer.deserialize_struct(stringify!($name), &["o", "p"], MessageVisitor)
                }
            }
        }}
    }

    #[cfg(feature = "framework")]
    use futures::future::{BoxFuture, Future};

    #[allow(unused_imports)]
    use crate::models::{
        aliases::*,
        commands::{Identify, SetPresence},
        events::*,
        Arc, Intent, Message as RoomMessage, Party, PartyMember, Relationship, Role, User, UserPresence,
    };

    type Room = (); // TODO

    // TODO: Check that this enum doesn't grow too large, allocate large payloads like Ready
    decl_msgs! {
        /// Messages send from the server to the client
        enum ServerMsg {
            /// The Hello message initiates the gateway session and expects a [ClientMsg::Identify] in return.
            0 => Hello { #[serde(flatten)] inner: Hello },

            /// Acknowledgement of a heartbeat
            1 => HeartbeatAck: Default {},
            2 => Ready { #[serde(flatten)] inner *Deref: Arc<Ready> },

            /// Sent when the session is no longer valid
            3 => InvalidSession: Default {},

            4 => PartyCreate { #[serde(flatten)] inner *Deref: Arc<Party> },
            5 => PartyUpdate { #[serde(flatten)] inner *Deref: Arc<PartyUpdateEvent> },
            6 => PartyDelete { id: PartyId },

            7 => RoleCreate { #[serde(flatten)] inner *Deref: Arc<Role> },
            8 => RoleUpdate { #[serde(flatten)] inner *Deref: Arc<Role> },
            9 => RoleDelete { #[serde(flatten)] inner *Deref: Arc<RoleDeleteEvent> },

            10 => MemberAdd     { #[serde(flatten)] inner *Deref: Arc<PartyMemberEvent> },
            11 => MemberUpdate  { #[serde(flatten)] inner *Deref: Arc<PartyMemberEvent> },
            12 => MemberRemove  { #[serde(flatten)] inner *Deref: Arc<PartyMemberEvent> },
            13 => MemberBan     { #[serde(flatten)] inner *Deref: Arc<PartyMemberEvent> },
            14 => MemberUnban   { #[serde(flatten)] inner *Deref: Arc<PartyMemberEvent> },

            15 => RoomCreate { #[serde(flatten)] inner *Deref: Arc<Room> },
            16 => RoomUpdate { #[serde(flatten)] inner *Deref: Arc<Room> },
            17 => RoomDelete { #[serde(flatten)] inner *Deref: Arc<RoomDeleteEvent> },
            18 => RoomPinsUpdate {},

            19 => MessageCreate { #[serde(flatten)] inner *Deref: Arc<RoomMessage> },
            20 => MessageUpdate { #[serde(flatten)] inner *Deref: Arc<RoomMessage> },
            21 => MessageDelete { #[serde(flatten)] inner *Deref: Arc<MessageDeleteEvent> },

            22 => MessageReactionAdd { #[serde(flatten)] inner *Deref: Arc<UserReactionEvent> },
            23 => MessageReactionRemove { #[serde(flatten)] inner *Deref: Arc<UserReactionEvent> },
            24 => MessageReactionRemoveAll {},
            25 => MessageReactionRemoveEmote {},

            26 => PresenceUpdate { #[serde(flatten)] inner *Deref: Arc<UserPresenceEvent> },
            27 => TypingStart { #[serde(flatten)] inner *Deref: Arc<TypingStart> },
            28 => UserUpdate { user: Arc<User> },

            29 => ProfileUpdate { #[serde(flatten)] inner *Deref: Arc<ProfileUpdateEvent> },
            30 => RelationAdd { #[serde(flatten)] inner *Deref: Arc<Relationship> },
            31 => RelationRemove { user_id: UserId },
        }
    }

    decl_msgs! {
        /// Messages sent from the client to the server
        enum ClientMsg {
            0 => Heartbeat: Default {},
            1 => Identify { #[serde(flatten)] inner *Deref: Box<Identify> },
            2 => Resume { session: Snowflake },
            3 => SetPresence { #[serde(flatten)] inner *Deref: Box<SetPresence> },
            4 => Subscribe { party_id: PartyId },
            5 => Unsubscribe { party_id: PartyId },
        }
    }

    impl ServerMsg {
        #[rustfmt::skip] #[must_use]
        pub fn matching_intent(&self) -> Option<Intent> {
            Some(match *self {
                | ServerMsg::PartyCreate { .. }
                | ServerMsg::PartyDelete { .. }
                | ServerMsg::PartyUpdate { .. }
                | ServerMsg::RoleCreate { .. }
                | ServerMsg::RoleDelete { .. }
                | ServerMsg::RoleUpdate { .. }
                | ServerMsg::RoomPinsUpdate { .. }
                | ServerMsg::RoomCreate { .. }
                | ServerMsg::RoomDelete { .. }
                | ServerMsg::RoomUpdate { .. }
                    => Intent::PARTIES,

                | ServerMsg::MemberAdd { .. }
                | ServerMsg::MemberUpdate { .. }
                | ServerMsg::MemberRemove { .. }
                    => Intent::PARTY_MEMBERS,

                | ServerMsg::MemberBan {..}
                | ServerMsg::MemberUnban {..}
                    => Intent::PARTY_BANS,

                | ServerMsg::MessageCreate { .. }
                | ServerMsg::MessageDelete { .. }
                | ServerMsg::MessageUpdate { .. }
                    => Intent::MESSAGES,

                | ServerMsg::MessageReactionAdd { .. }
                | ServerMsg::MessageReactionRemove { .. }
                | ServerMsg::MessageReactionRemoveAll { .. }
                | ServerMsg::MessageReactionRemoveEmote { .. }
                    => Intent::MESSAGE_REACTIONS,

                ServerMsg::PresenceUpdate { .. }
                    => Intent::PRESENCE,

                ServerMsg::TypingStart { .. }
                    => Intent::MESSAGE_TYPING,

                ServerMsg::ProfileUpdate(ref payload) => match payload.inner.party_id {
                    Some(_) => Intent::PROFILE_UPDATES | Intent::PARTY_MEMBERS,
                    None => Intent::PROFILE_UPDATES,
                }

                | ServerMsg::Hello { .. }
                | ServerMsg::HeartbeatAck { .. }
                | ServerMsg::Ready { .. }
                | ServerMsg::UserUpdate { .. }
                | ServerMsg::InvalidSession { .. }
                | ServerMsg::RelationAdd { .. }
                | ServerMsg::RelationRemove { .. }
                    => return None,
            })
        }

        /// If the event originated from a specific user, get their ID
        #[must_use]
        pub fn user_id(&self) -> Option<UserId> {
            Some(match self {
                ServerMsg::MemberAdd(e) => e.member.user.id,
                ServerMsg::MemberUpdate(e) => e.member.user.id,
                ServerMsg::MemberRemove(e) => e.member.user.id,
                ServerMsg::MemberBan(e) => e.member.user.id,
                ServerMsg::MemberUnban(e) => e.member.user.id,

                ServerMsg::MessageCreate(m) => m.author.id,
                ServerMsg::MessageUpdate(m) => m.author.id,

                ServerMsg::MessageReactionAdd(r) => r.user_id,
                ServerMsg::MessageReactionRemove(r) => r.user_id,

                ServerMsg::PresenceUpdate(p) => p.user.id,
                _ => return None,
            })
        }
    }

    #[cfg(test)]
    mod tests {
        use core::mem::size_of;

        use super::*;

        #[test]
        fn test_client_msg_size() {
            assert_eq!(16, size_of::<ClientMsg>());
        }

        #[cfg(feature = "ts")]
        #[test]
        fn test_server_msg_ts() {
            use ts_bindgen::{TypeRegistry, TypeScriptDef};

            let mut registry = TypeRegistry::default();

            ServerMsg::register(&mut registry);

            println!("{}", registry.fmt_to_string().unwrap());
        }
    }
}
