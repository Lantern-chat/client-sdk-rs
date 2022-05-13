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

        ; struct RedeemInviteBody {
            #[serde(default, skip_serializing_if = "Option::is_none")]
            pub nickname: Option<SmolStr>,
        }
    }
}
