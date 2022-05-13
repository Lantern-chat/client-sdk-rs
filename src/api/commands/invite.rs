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

    +struct CreateInvite -> Invite: POST("invite") {
        ;
        /// Infinite parameters may only be used with appropriate permissions
        struct CreateInviteBody {
            /// If `None`, invite has infinite uses
            #[serde(default)]
            pub max_uses: Option<u16>,

            /// If `None`, invite has infinite duration
            #[serde(default)]
            pub duration: Option<u64>,

            #[serde(default)]
            pub description: Option<SmolStr>,
        }
    }
}
