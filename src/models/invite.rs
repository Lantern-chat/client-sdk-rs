use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Invite {
    pub code: SmolStr,

    pub party: PartialParty,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inviter: Option<Snowflake>,

    #[serde(default, skip_serializing_if = "is_none_or_empty")]
    pub description: Option<SmolStr>,

    pub expires: Option<Timestamp>,

    /// Number of remaining uses this invite has left.
    ///
    /// Only users with the `MANAGE_INVITES` permission can see this.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remaining: Option<u16>,
}
