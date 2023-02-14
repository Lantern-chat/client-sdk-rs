use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "lowercase")]
pub enum EmbedType {
    Img,
    Audio,
    Vid,
    Html,
    Link,
    Article,
}

pub type MaybeEmbedMedia = Option<Box<EmbedMedia>>;

/// An embed is metadata taken from a given URL by loading said URL, parsing any meta tags, and fetching
/// extra information from oEmbed sources.
///
/// Typically, embeds contain title, description, etc. plus a thumbnail. However, direct media
/// may be embedded directly either via a URL (`embed_url`) or arbitrary HTML (`embed_html`), of which
/// should always be properly sandboxed.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct EmbedV1 {
    /// Timestamp when the embed was retreived
    pub ts: Timestamp,

    /// URL fetched
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<SmolStr>,

    /// Embed type
    pub ty: EmbedType,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<SmolStr>,

    /// Description, usually from the Open-Graph API
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub desc: Option<SmolStr>,

    /// Accent Color
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub col: Option<u32>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author: Option<EmbedAuthor>,

    /// oEmbed Provider
    #[serde(default, skip_serializing_if = "EmbedProvider::is_none")]
    pub pro: EmbedProvider,

    /// HTML and similar objects
    ///
    /// See: <https://www.html5rocks.com/en/tutorials/security/sandboxed-iframes/>
    #[serde(default, skip_serializing_if = "EmbedMedia::is_empty")]
    pub obj: MaybeEmbedMedia,
    /// Image media
    #[serde(default, skip_serializing_if = "EmbedMedia::is_empty")]
    pub img: MaybeEmbedMedia,
    #[serde(default, skip_serializing_if = "EmbedMedia::is_empty")]
    pub audio: MaybeEmbedMedia,
    /// Video media
    #[serde(default, skip_serializing_if = "EmbedMedia::is_empty")]
    pub vid: MaybeEmbedMedia,
    #[serde(default, skip_serializing_if = "EmbedMedia::is_empty")]
    pub thumb: MaybeEmbedMedia,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fields: Vec<EmbedField>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub footer: Option<EmbedFooter>,
}

impl EmbedV1 {
    /// Visit each [`EmbedMedia`] to mutate them (such as to generate the proxy signature)
    pub fn visit_media_mut<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut EmbedMedia),
    {
        EmbedMedia::visit_mut(&mut self.obj, &mut f);
        EmbedMedia::visit_mut(&mut self.img, &mut f);
        EmbedMedia::visit_mut(&mut self.audio, &mut f);
        EmbedMedia::visit_mut(&mut self.vid, &mut f);
        EmbedMedia::visit_mut(&mut self.thumb, &mut f);

        if let Some(ref mut footer) = self.footer {
            EmbedMedia::visit_mut(&mut footer.icon, &mut f);
        }

        if let Some(ref mut author) = self.author {
            EmbedMedia::visit_mut(&mut author.icon, &mut f);
        }
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct EmbedFooter {
    pub text: SmolStr,

    #[serde(default, skip_serializing_if = "EmbedMedia::is_empty")]
    pub icon: MaybeEmbedMedia,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct EmbedMedia {
    pub url: SmolStr,

    /// Non-visible description of the embedded media
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub alt: Option<SmolStr>,

    /// Cryptographic signature for use with the proxy server
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sig: Option<SmolStr>,

    /// height
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub h: Option<i32>,

    /// witdth
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub w: Option<i32>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mime: Option<SmolStr>,
}

impl EmbedMedia {
    pub fn is_empty(this: &MaybeEmbedMedia) -> bool {
        match this {
            Some(ref e) => e.url.is_empty(),
            None => true,
        }
    }

    pub fn visit_mut<F>(this: &mut MaybeEmbedMedia, mut f: F)
    where
        F: FnMut(&mut EmbedMedia),
    {
        if let Some(ref mut media) = this {
            f(&mut *media);
        }
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct EmbedProvider {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<SmolStr>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<SmolStr>,
}

impl EmbedProvider {
    pub const fn is_none(&self) -> bool {
        self.name.is_none() && self.url.is_none()
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct EmbedAuthor {
    pub name: SmolStr,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<SmolStr>,

    #[serde(default, skip_serializing_if = "EmbedMedia::is_empty")]
    pub icon: MaybeEmbedMedia,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct EmbedField {
    name: SmolStr,
    value: SmolStr,

    #[serde(default, skip_serializing_if = "is_false")]
    inline: bool,
}

impl EmbedField {
    pub fn is_empty(&self) -> bool {
        self.name.is_empty() || self.value.is_empty()
    }
}

impl Default for EmbedV1 {
    #[inline]
    fn default() -> EmbedV1 {
        EmbedV1 {
            ts: Timestamp::UNIX_EPOCH,
            ty: EmbedType::Link,
            url: None,
            title: None,
            desc: None,
            col: None,
            author: None,
            pro: EmbedProvider::default(),
            img: None,
            audio: None,
            vid: None,
            thumb: None,
            obj: None,
            fields: Vec::new(),
            footer: None,
        }
    }
}
