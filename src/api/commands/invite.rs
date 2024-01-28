use super::*;

command! {
    +struct GetInvite -> One Invite: GET("invite" / code) {
        pub code: SmolStr,
    }

    +struct RevokeInvite -> One (): DELETE("invite" / code) {
        pub code: SmolStr,
    }

    +struct RedeemInvite -> One (): POST("invite" / code / "redeem") {
        pub code: SmolStr,

        ;
        #[cfg_attr(feature = "builder", derive(typed_builder::TypedBuilder))]
        struct RedeemInviteBody {
            #[serde(default, skip_serializing_if = "Option::is_none")]
            #[cfg_attr(feature = "builder", builder(default, setter(into)))]
            pub nickname: Option<SmolStr>,
        }
    }
}
