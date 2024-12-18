use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "rkyv", derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize))]
#[cfg_attr(feature = "ts", derive(ts_bindgen::TypeScriptDef))]
pub struct Invite {
    /// Invite code, which is either an encrypted Snowflake or a custom vanity code.
    pub code: SmolStr,

    pub party: PartialParty,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "rkyv", rkyv(with = NicheSnowflake))]
    pub inviter: Option<UserId>,

    #[serde(default, skip_serializing_if = "is_none_or_empty")]
    pub description: Option<SmolStr>,

    pub expires: Option<Timestamp>,

    /// Number of remaining uses this invite has left.
    ///
    /// Only users with the `MANAGE_INVITES` permission can see this.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remaining: Option<u16>,
}
