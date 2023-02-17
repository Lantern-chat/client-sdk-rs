use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "lowercase")]
pub enum EmbedType {
    #[serde(alias = "image")]
    Img,
    Audio,
    #[serde(alias = "video")]
    Vid,
    Html,
    Link,
    Article,
}

fn is_none_or_empty(value: &Option<SmolStr>) -> bool {
    match value {
        Some(ref value) => value.is_empty(),
        None => true,
    }
}

pub type MaybeEmbedMedia = Option<Box<EmbedMedia>>;

/// An embed is metadata taken from a given URL by loading said URL, parsing any meta tags, and fetching
/// extra information from oEmbed sources.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct EmbedV1 {
    /// Timestamp when the embed was retreived
    pub ts: Timestamp,

    /// Embed type
    #[serde(alias = "type")]
    pub ty: EmbedType,

    /// URL fetched
    #[serde(default, skip_serializing_if = "is_none_or_empty")]
    pub url: Option<SmolStr>,

    /// Canonical URL
    #[serde(default, skip_serializing_if = "is_none_or_empty", alias = "canonical")]
    pub can: Option<SmolStr>,

    #[serde(default, skip_serializing_if = "is_none_or_empty")]
    pub title: Option<SmolStr>,

    /// Description, usually from the Open-Graph API
    #[serde(default, skip_serializing_if = "is_none_or_empty", alias = "description")]
    pub desc: Option<SmolStr>,

    /// Accent Color
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "color")]
    pub col: Option<u32>,

    #[serde(default, skip_serializing_if = "EmbedAuthor::is_none")]
    pub author: Option<EmbedAuthor>,

    /// oEmbed Provider
    #[serde(default, skip_serializing_if = "EmbedProvider::is_none", alias = "provider")]
    pub pro: EmbedProvider,

    /// HTML and similar objects
    ///
    /// See: <https://www.html5rocks.com/en/tutorials/security/sandboxed-iframes/>
    #[serde(default, skip_serializing_if = "EmbedMedia::is_empty")]
    pub obj: MaybeEmbedMedia,
    /// Image media
    #[serde(default, skip_serializing_if = "EmbedMedia::is_empty", alias = "image")]
    pub img: MaybeEmbedMedia,
    #[serde(default, skip_serializing_if = "EmbedMedia::is_empty")]
    pub audio: MaybeEmbedMedia,
    /// Video media
    #[serde(default, skip_serializing_if = "EmbedMedia::is_empty", alias = "video")]
    pub vid: MaybeEmbedMedia,
    #[serde(default, skip_serializing_if = "EmbedMedia::is_empty")]
    pub thumb: MaybeEmbedMedia,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fields: Vec<EmbedField>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub footer: Option<EmbedFooter>,
}

impl EmbedV1 {
    pub fn is_plain_link(&self) -> bool {
        if self.ty != EmbedType::Link
            || self.url.is_none()
            || !is_none_or_empty(&self.can)
            || !is_none_or_empty(&self.title)
            || !is_none_or_empty(&self.desc)
            || self.col.is_some()
            || !EmbedAuthor::is_none(&self.author)
            || !EmbedProvider::is_none(&self.pro)
            || !EmbedMedia::is_empty(&self.obj)
            || !EmbedMedia::is_empty(&self.img)
            || !EmbedMedia::is_empty(&self.audio)
            || !EmbedMedia::is_empty(&self.vid)
            || !EmbedMedia::is_empty(&self.thumb)
            || !self.fields.is_empty()
            || self.footer.is_some()
        {
            return false;
        }

        true
    }

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

pub type UrlSignature = FixedStr<27>;

crate::util::fixed::impl_fixedstr_schema!(UrlSignature, "Base-64 encoded cryptographic signature");

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct EmbedMedia {
    pub url: SmolStr,

    /// Non-visible description of the embedded media
    #[serde(default, skip_serializing_if = "is_none_or_empty")]
    pub alt: Option<SmolStr>,

    /// Cryptographic signature for use with the proxy server
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sig: Option<FixedStr<27>>,

    /// height
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "height")]
    pub h: Option<i32>,

    /// witdth
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "width")]
    pub w: Option<i32>,

    #[serde(default, skip_serializing_if = "is_none_or_empty")]
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
    #[serde(default, skip_serializing_if = "is_none_or_empty")]
    pub name: Option<SmolStr>,

    #[serde(default, skip_serializing_if = "is_none_or_empty")]
    pub url: Option<SmolStr>,
}

impl EmbedProvider {
    pub fn is_none(&self) -> bool {
        is_none_or_empty(&self.name) && is_none_or_empty(&self.url)
    }
}

impl EmbedAuthor {
    pub fn is_none(this: &Option<Self>) -> bool {
        match this {
            Some(ref this) => {
                this.name.is_empty() && is_none_or_empty(&this.url) && EmbedMedia::is_empty(&this.icon)
            }
            None => true,
        }
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct EmbedAuthor {
    pub name: SmolStr,

    #[serde(default, skip_serializing_if = "is_none_or_empty")]
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
            can: None,
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
