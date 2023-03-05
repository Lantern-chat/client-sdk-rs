use super::*;

command! {
    +struct GetInvite -> Invite: GET("invite" / code) {
        pub code: SmolStr,
    }

    +struct RevokeInvite -> (): DELETE("invite" / code) {
        pub code: SmolStr,
    }

    +struct RedeemInvite -> (): POST("invite" / code / "redeem") {
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
