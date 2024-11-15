use super::*;

command! { Invite;

    +struct GetInvite -> One Invite: GET("invite" / code) {
        pub code: SmolStr,
    }

    +struct RevokeInvite -> One (): DELETE("invite" / code) {
        pub code: SmolStr,
    }

    +struct RedeemInvite -> One (): POST("invite" / code / "redeem") {
        pub code: SmolStr,

        ;
        #[cfg_attr(feature = "typed-builder", derive(typed_builder::TypedBuilder))]
        #[cfg_attr(feature = "bon", derive(bon::Builder))]
        struct RedeemInviteBody {
            #[serde(default, skip_serializing_if = "Option::is_none")]
            #[cfg_attr(feature = "typed-builder", builder(default, setter(into)))]
            #[cfg_attr(feature = "bon", builder(into))]
            pub nickname: Option<SmolStr>,
        }
    }
}
